//! `cadcraft-cli mcp --connect` end to end: a missing address must fail startup instead of
//! serving drawing edits from a headless session (#103); a real address still bridges.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

const INIT: &str = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2025-06-18"}}"#;
const CIRCLE: &str = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"execute","arguments":{"command":"circle","params":{"center":[5,5],"radius":2}}}}"#;

const PROCESS_TIMEOUT: Duration = Duration::from_secs(10);
const ACCEPT_TIMEOUT: Duration = Duration::from_secs(5);

fn wait_with_timeout(mut child: Child, timeout: Duration) -> Output {
    let start = Instant::now();
    loop {
        match child.try_wait().expect("poll cadcraft-cli") {
            Some(_) => return child.wait_with_output().expect("collect cadcraft-cli output"),
            None if start.elapsed() < timeout => std::thread::sleep(Duration::from_millis(20)),
            None => {
                let _ = child.kill();
                let out = child.wait_with_output().expect("collect timed-out cadcraft-cli output");
                panic!("cadcraft-cli exceeded {timeout:?}: stderr={}", String::from_utf8_lossy(&out.stderr));
            }
        }
    }
}

fn accept_with_timeout(listener: &TcpListener, timeout: Duration) -> TcpStream {
    listener.set_nonblocking(true).expect("set nonblocking");
    let start = Instant::now();
    loop {
        match listener.accept() {
            Ok((stream, _)) => { stream.set_nonblocking(false).expect("set blocking"); return stream; }
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock && start.elapsed() < timeout => std::thread::sleep(Duration::from_millis(20)),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => panic!("accept timed out after {timeout:?}"),
            Err(e) => panic!("accept failed: {e}"),
        }
    }
}

fn join_with_timeout<T>(handle: std::thread::JoinHandle<T>, timeout: Duration, what: &str) -> T {
    let start = Instant::now();
    while !handle.is_finished() {
        if start.elapsed() >= timeout { panic!("{what} did not finish within {timeout:?}"); }
        std::thread::sleep(Duration::from_millis(10));
    }
    handle.join().unwrap_or_else(|_| panic!("{what} panicked"))
}

fn mcp(args: &[&str], input: &str) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_cadcraft-cli"))
        .arg("mcp")
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn cadcraft-cli");
    if let Some(mut stdin) = child.stdin.take() {
        // The process may exit before reading stdin (that is the point); ignore a broken pipe.
        let _ = stdin.write_all(input.as_bytes());
    }
    wait_with_timeout(child, PROCESS_TIMEOUT)
}

fn replies(out: &Output) -> Vec<Value> {
    String::from_utf8_lossy(&out.stdout).lines().filter_map(|l| serde_json::from_str(l).ok()).collect()
}

#[test]
fn connect_without_address_fails_before_serving() {
    let input = format!("{INIT}\n{CIRCLE}\n");
    for args in [&["--connect"][..], &["--connect", ""], &["--connect", "--foo"]] {
        let out = mcp(args, &input);
        assert!(!out.status.success(), "{args:?} must exit non-zero");
        assert!(out.stdout.is_empty(), "{args:?} must not answer MCP requests: {}", String::from_utf8_lossy(&out.stdout));
        let err = String::from_utf8_lossy(&out.stderr);
        assert!(err.contains("--connect needs HOST:PORT"), "{args:?}: {err}");
    }
}

#[test]
fn no_connect_still_serves_headless() {
    let out = mcp(&[], &format!("{INIT}\n{CIRCLE}\n"));
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let r = replies(&out);
    let instructions = r.iter().find(|v| v["id"] == 1).and_then(|v| v["result"]["instructions"].as_str()).unwrap_or_default();
    assert!(instructions.contains("Backend: headless"), "{instructions}");
    let circle = r.iter().find(|v| v["id"] == 2).map(|v| &v["result"]);
    assert!(circle.is_some_and(|c| c.is_object() && c.get("isError").is_none()), "{r:?}");
}

#[test]
fn connect_with_address_routes_to_that_endpoint() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let addr = listener.local_addr().expect("local addr").to_string();
    let server = std::thread::spawn(move || {
        let conn = accept_with_timeout(&listener, ACCEPT_TIMEOUT);
        conn.set_read_timeout(Some(ACCEPT_TIMEOUT)).expect("set read timeout");
        conn.set_write_timeout(Some(ACCEPT_TIMEOUT)).expect("set write timeout");
        let mut reader = BufReader::new(conn.try_clone().expect("clone"));
        let mut line = String::new();
        reader.read_line(&mut line).expect("read request");
        let req: Value = serde_json::from_str(&line).expect("json request");
        let reply = json!({"id": req["id"], "ok": true, "result": {"fixture": "loopback"}});
        let mut w = conn;
        writeln!(w, "{reply}").expect("write reply");
        req
    });
    let call = r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"ui_inspect","arguments":{}}}"#;
    let out = mcp(&["--connect", &addr], &format!("{INIT}\n{call}\n"));
    assert!(out.status.success(), "stderr: {}", String::from_utf8_lossy(&out.stderr));
    let req = join_with_timeout(server, Duration::from_secs(6), "CLI MCP fake server");
    assert_eq!(req["method"], "ui.inspect");
    let r = replies(&out);
    let instructions = r.iter().find(|v| v["id"] == 1).and_then(|v| v["result"]["instructions"].as_str()).unwrap_or_default();
    assert!(instructions.contains(&format!("connected to CADCraft at {addr}")), "{instructions}");
    let text = r.iter().find(|v| v["id"] == 2).and_then(|v| v["result"]["content"][0]["text"].as_str()).unwrap_or_default();
    assert!(text.contains("loopback"), "{r:?}");
}

#[test]
fn connect_to_closed_port_fails() {
    let addr = {
        let l = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
        l.local_addr().expect("local addr").to_string()
    };
    let out = mcp(&["--connect", &addr], &format!("{INIT}\n"));
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("cannot connect to"));
}
