use std::sync::mpsc;
use std::time::{Duration, Instant};

use cadcraft_doc::Drawing;
use cadcraft_engine::cmd::file::{IoHooks, io, set_io};

fn no_read(_: &[u8], _: &str) -> Result<Drawing, String> { Err("unused".into()) }
fn write_a(_: &Drawing, _: &str) -> Result<Vec<u8>, String> { Ok(b"A".to_vec()) }
fn write_b(_: &Drawing, _: &str) -> Result<Vec<u8>, String> { Ok(b"B".to_vec()) }

fn join_with_timeout<T>(handle: std::thread::JoinHandle<T>, timeout: Duration, what: &str) -> T {
    let start = Instant::now();
    while !handle.is_finished() {
        if start.elapsed() >= timeout { panic!("{what} did not finish within {timeout:?}"); }
        std::thread::sleep(Duration::from_millis(10));
    }
    handle.join().unwrap_or_else(|_| panic!("{what} panicked"))
}

#[test]
fn io_hooks_are_isolated_per_test_thread() {
    let (ready_tx, ready_rx) = mpsc::channel();
    let (release_a_tx, release_a_rx) = mpsc::channel();
    let (release_b_tx, release_b_rx) = mpsc::channel();
    let run = |write: fn(&Drawing, &str) -> Result<Vec<u8>, String>, expected: &'static [u8], release: mpsc::Receiver<()>| {
        let ready = ready_tx.clone();
        std::thread::spawn(move || {
            set_io(IoHooks { read: no_read, write, plot: None });
            ready.send(()).expect("announce I/O hook ready");
            release.recv_timeout(Duration::from_secs(2)).expect("release I/O hook worker");
            let hooks = io().expect("thread-local I/O hooks");
            assert_eq!((hooks.write)(&Drawing::new_metric(), "probe.dxf").unwrap(), expected);
        })
    };

    let a = run(write_a, b"A", release_a_rx);
    let b = run(write_b, b"B", release_b_rx);
    ready_rx.recv_timeout(Duration::from_secs(2)).expect("first I/O hook worker ready");
    ready_rx.recv_timeout(Duration::from_secs(2)).expect("second I/O hook worker ready");
    release_a_tx.send(()).expect("release first I/O hook worker");
    release_b_tx.send(()).expect("release second I/O hook worker");
    join_with_timeout(a, Duration::from_secs(3), "first I/O hook worker");
    join_with_timeout(b, Duration::from_secs(3), "second I/O hook worker");
}
