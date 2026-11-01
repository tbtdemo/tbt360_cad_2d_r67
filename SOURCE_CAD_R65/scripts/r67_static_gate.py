from pathlib import Path
import hashlib
import re
import sys
import tomllib

ROOT = Path(__file__).resolve().parents[2]
SRC = ROOT / "SOURCE_CAD_R65"
errors = []

def read(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        return path.read_text(encoding="utf-8-sig")

def require(cond: bool, message: str) -> None:
    if not cond:
        errors.append(message)

# Engine compile regressions that previously made several lanes fail together.
divide = read(SRC / "crates/engine/src/cmd/draw2/divide.rs")
require("source:Handle" in divide.replace(" ", ""), "divide.rs: place_markers must keep source: Handle")
place_calls = re.findall(r"place_markers\s*\(([^;]+?)\)\?", divide, re.S)
require(len(place_calls) >= 2, "divide.rs: expected DIVIDE/MEASURE place_markers call sites")
for idx, call in enumerate(place_calls, 1):
    compact = re.sub(r"\s+", "", call)
    require(compact.endswith(",h"), f"divide.rs: place_markers call #{idx} must pass source handle h")

tan = read(SRC / "crates/engine/src/cmd/draw2/tancircle.rs")
copy_pos = tan.find("let (a,b,third)=(*a,*b,*third);")
clear_pos = tan.find("self.picks.clear();", copy_pos if copy_pos >= 0 else 0)
require(copy_pos >= 0 and clear_pos > copy_pos, "tancircle.rs: copy picked tuples before closure mutates self.picks")

# QTO remains a CAD-native feature, not BIM collaboration runtime state.
main = read(SRC / "apps/cadcraft/src/main.rs")
takeoff = read(SRC / "apps/cadcraft/src/takeoff.rs")
require("takeoff::panel(ui, &self.cad, &mut self.takeoff_state, &qto_scope)" in main, "cadcraft main: live QTO panel signature drifted")
require("crate::bim_collab::" not in takeoff and "bim_collab_state" not in takeoff, "takeoff.rs: BIM collaboration runtime dependency reintroduced")
require("source_geometry_signature" in takeoff and "source_handle" in takeoff, "takeoff.rs: CAD source identity fields missing")


# Workflow package/test targets must exist before Cargo spends time resolving/building them.
workspace_packages = {}
for cargo in list((SRC / "crates").glob("*/Cargo.toml")) + list((SRC / "apps").glob("*/Cargo.toml")) + [SRC / "xtask/Cargo.toml"]:
    text = read(cargo)
    pm = re.search(r'^name\s*=\s*"([^"]+)"', text, re.M)
    if pm:
        workspace_packages[pm.group(1)] = cargo
for package in ["cadcraft", "cadcraft-cli", "cadcraft-web", "cadcraft-mcp", "cadcraft-engine", "cadcraft-io", "cadcraft-ui-egui"]:
    require(package in workspace_packages, f"workspace: package {package} referenced by R67 CI is missing")

integration_targets = {
    "cadcraft-engine": ["xref", "insert_file", "undo_control", "io_hooks_thread_local", "clipboard"],
}
for package, targets in integration_targets.items():
    cargo = workspace_packages.get(package)
    if cargo is None:
        continue
    tests_dir = cargo.parent / "tests"
    for target in targets:
        require((tests_dir / f"{target}.rs").is_file(), f"{package}: CI integration target --test {target} has no {target}.rs")

# Any nested integration suite at tests/it/main.rs must be an explicit Cargo target. Cargo's
# normal integration-test discovery is top-level-file based; leaving a nested suite undeclared can
# make `--test it` unavailable and, worse, let Full Workspace report green without compiling it.
for package, cargo in workspace_packages.items():
    nested = cargo.parent / "tests/it/main.rs"
    if not nested.is_file():
        continue
    manifest_text = read(cargo)
    require(re.search(r'\[\[test\]\]\s*name\s*=\s*"it"\s*path\s*=\s*"tests/it/main\.rs"', manifest_text, re.S) is not None,
            f"{package}: nested tests/it/main.rs must be declared as [[test]] name=it")

# Every module named by a nested integration root must resolve to a real source file. Also check
# actual external `use crate_name::...` roots against direct dependencies/dev-dependencies so a
# newly-enabled harness cannot fail later because its dormant test dependency was never declared.
for package, cargo in workspace_packages.items():
    nested = cargo.parent / "tests/it/main.rs"
    if not nested.is_file():
        continue
    root_text = read(nested)
    path_overrides = {name: rel for rel, name in re.findall(r'#\[path\s*=\s*"([^"]+)"\]\s*mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;', root_text, re.S)}
    for name in re.findall(r'(?m)^\s*mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;', root_text):
        if name in path_overrides:
            target = (nested.parent / path_overrides[name]).resolve()
            require(target.is_file(), f"{package}: #[path] module {name} does not resolve: {target}")
        else:
            direct = nested.parent / f"{name}.rs"
            module = nested.parent / name / "mod.rs"
            require(direct.is_file() or module.is_file(), f"{package}: nested test module {name} has no source file")

    data = tomllib.loads(read(cargo))
    declared = set()
    for section in ("dependencies", "dev-dependencies", "build-dependencies"):
        declared.update(k.replace("-", "_") for k in data.get(section, {}))
    for key, value in data.items():
        if key.startswith("target") and isinstance(value, dict):
            for section in ("dependencies", "dev-dependencies", "build-dependencies"):
                declared.update(k.replace("-", "_") for k in value.get(section, {}))
    own = data.get("package", {}).get("name", "").replace("-", "_")
    external = set()
    for test_file in nested.parent.rglob("*.rs"):
        code = read(test_file)
        code = re.sub(r"//.*", "", code)
        code = re.sub(r"/\*.*?\*/", "", code, flags=re.S)
        for dep in re.findall(r'(?m)^\s*use\s+([a-z_][A-Za-z0-9_]*)::', code):
            if dep not in {"std", "core", "alloc", "crate", "self", "super"} and dep != own:
                external.add(dep)
    for dep in sorted(external):
        require(dep in declared, f"{package}: nested tests import undeclared direct dependency {dep}")

# Focused lanes must not accidentally compile unrelated integration harnesses. Full Workspace owns
# the complete nested suites; UI unit filters therefore stay on --lib.
workflow = read(ROOT / ".github/workflows/parallel-r66.yml")
for filt in ["menus::tests", "layers::tests", "palettes::tests", "i18n::tests"]:
    require(f"cargo test --locked -p cadcraft-ui-egui --lib {filt}" in workflow,
            f"workflow: UI filter {filt} must use --lib to keep the lane isolated")
require("cargo test --locked -p cadcraft-ui-egui --test it --no-run" in workflow,
        "workflow: UI lane must compile the complete nested integration harness once")

# Nested integration tests run in parallel by default. Fixed temp-directory names are a source of
# nondeterministic Windows CI failures because one test can delete or overwrite another test's files.
# Audit every nested suite, not only the Engine/UI packages, because Full Workspace runs all of them.
nested_suites = [
    SRC / "apps/cadcraft-cli/tests/it",
    SRC / "crates/doc/tests/it",
    SRC / "crates/engine/tests/it",
    SRC / "crates/io/tests/it",
    SRC / "crates/mcp/tests/it",
    SRC / "crates/render/tests/it",
    SRC / "crates/ui-egui/tests/it",
]
for suite in nested_suites:
    for test_file in suite.rglob("*.rs"):
        code = read(test_file)
        for line_no, line in enumerate(code.splitlines(), 1):
            if "temp_dir()" in line and ".join(\"" in line:
                require("process::id()" in line, f"{test_file.relative_to(ROOT)}:{line_no}: fixed temp path must include process::id()")
        # Compile-time fixtures are easy to break when tests are moved/renamed. Resolve every literal
        # include_str!/include_bytes! relative to the source file exactly as rustc does.
        for macro, rel in re.findall(r'include_(str|bytes)!\(\s*"([^"]+)"\s*\)', code):
            fixture = (test_file.parent / rel).resolve()
            require(fixture.exists(), f"{test_file.relative_to(ROOT)}: include_{macro}! fixture missing: {rel}")


# Network/process tests must fail fast instead of blocking a Windows lane until the job timeout.
# Guard the exact high-risk suites where a failed client connection used to leave server threads
# stuck in TcpListener::accept() or child processes stuck in wait_with_output().
cli_mcp = read(SRC / "apps/cadcraft-cli/tests/it/mcp_connect.rs")
cli_pipe = read(SRC / "apps/cadcraft-cli/tests/it/cli.rs")
mcp_backend = read(SRC / "crates/mcp/src/backend.rs")
mcp_tests = read(SRC / "crates/mcp/src/tests.rs")
require("accept_with_timeout" in cli_mcp and "wait_with_timeout" in cli_mcp, "cadcraft-cli mcp_connect: accept/process watchdog missing")
require("wait_with_timeout" in cli_pipe, "cadcraft-cli cli: subprocess watchdog missing")
require("fn accept_with_timeout" in mcp_backend, "cadcraft-mcp backend tests: accept watchdog missing")
require("fn accept_with_timeout" in mcp_tests, "cadcraft-mcp protocol tests: accept watchdog missing")
require('listener.accept().expect("accept")' not in cli_mcp, "cadcraft-cli mcp_connect: blocking accept reintroduced")
require('child.wait_with_output().expect("wait for cadcraft-cli")' not in cli_mcp, "cadcraft-cli mcp_connect: unbounded child wait reintroduced")

# Capture/autosave tests must not leak process-global state or wait forever for an async reply.
capture_tests = read(SRC / "crates/ui-egui/src/capture/tests.rs")
autosave_tests = read(SRC / "crates/engine/tests/autosave.rs")
require("fn recv_reply" in capture_tests and "recv_timeout" in capture_tests, "ui capture tests: bounded reply helper missing")
require(".recv().expect(" not in capture_tests, "ui capture tests: unbounded mpsc recv reintroduced")
require("struct AutosaveGlobals" in autosave_tests and "impl Drop for AutosaveGlobals" in autosave_tests, "engine autosave tests: process-global SAVETIME/SAVEFILEPATH restore guard missing")
require("let _globals = AutosaveGlobals::capture();" in autosave_tests, "engine autosave tests: global restore guard is not used by the mutating test")



# Installer smoke must never block the release runner indefinitely, and CLI loopback workers
# must bound reads/writes after accept before their thread is joined.
workflow = read(ROOT / ".github/workflows/parallel-r66.yml")
require("function Invoke-BoundedProcess" in workflow and "WaitForExit($timeoutSeconds * 1000)" in workflow, "release smoke: bounded process helper missing")
require('Start-Process -FilePath $setup' not in workflow or '-Wait -PassThru' not in workflow.split('Smoke test Windows installer and uninstall', 1)[1], "release smoke: unbounded installer wait reintroduced")
require('set_read_timeout(Some(ACCEPT_TIMEOUT))' in cli_mcp and 'set_write_timeout(Some(ACCEPT_TIMEOUT))' in cli_mcp, "cadcraft-cli mcp_connect: accepted socket timeout missing")
require('fn join_with_timeout' in cli_mcp and 'CLI MCP fake server' in cli_mcp, "cadcraft-cli mcp_connect: bounded fake-server join missing")

io_hooks_test = read(SRC / "crates/engine/tests/io_hooks_thread_local.rs")
require("Barrier" not in io_hooks_test, "engine io_hooks_thread_local: blocking Barrier reintroduced")
require("recv_timeout" in io_hooks_test and "fn join_with_timeout" in io_hooks_test, "engine io_hooks_thread_local: bounded worker handshake/join missing")


# Release artifact smoke must validate the actual installed contents and avoid teardown hangs.
workflow = read(ROOT / ".github/workflows/parallel-r66.yml")
require("Setup SHA256 sidecar mismatch" in workflow, "release smoke: setup SHA256 sidecar is not verified")
require("Installed CLI differs from release build" in workflow, "release smoke: installed CLI hash is not verified")
require("Start Menu shortcut points to wrong target" in workflow, "release smoke: shortcut target is not verified")
require("Installed GUI VersionInfo mismatch" in workflow, "release smoke: installed GUI version metadata is not verified")
require("Installed CLI reports wrong release version" in workflow, "release smoke: installed CLI version is not verified")
require("Portable GUI did not terminate after forced stop" in workflow, "portable smoke: bounded GUI teardown missing")
require("Installed GUI did not terminate after forced stop" in workflow, "installer smoke: bounded GUI teardown missing")
require("function Wait-PathGone" in workflow, "installer smoke: bounded artifact cleanup helper missing")

control_server = read(SRC / "apps/cadcraft/src/control_server.rs")
ui_control = read(SRC / "crates/ui-egui/src/control.rs")
require("fn join_with_timeout" in control_server and "busy-queue client" in control_server, "control-server tests: bounded worker join missing")
require("CADCRAFT_CONTROL_CAPABILITIES" in control_server, "control-server authorization: capability configuration missing")
require("fn required_capabilities" in control_server and "CAP_FILE_WRITE" in control_server and "CAP_AUTOMATION" in control_server, "control-server authorization: capability classifier missing")
require("forbidden: capability" in control_server and "requiredCapabilities" in control_server and '"state": "not-run"' in control_server, "control-server authorization: deny-before-dispatch response missing")
require("capability_gate_rejects_mutations_before_dispatch" in control_server and "capability_gate_separates_file_read_and_file_write" in control_server, "control-server authorization: capability regressions missing")
require("control reply worker did not finish within 2s" in ui_control, "ui control tests: bounded reply worker join missing")

# Windows release resources are mandatory in CI/release mode: missing icon or malformed release
# version must fail the build instead of silently producing a partially branded executable.
build_rs = read(SRC / "apps/cadcraft/build.rs")
require('let strict = std::env::var_os("CADCRAFT_REQUIRE_WINRES").is_some();' in build_rs, "cadcraft build.rs: strict Windows resource mode missing")
require('if strict && !icon.is_file()' in build_rs, "cadcraft build.rs: required icon is not enforced in strict mode")
require('invalid CAD360_RELEASE_VERSION for Windows resources' in build_rs, "cadcraft build.rs: malformed release version does not fail strict mode")
require('if strict {' in build_rs and 'embedding Windows resources failed' in build_rs, "cadcraft build.rs: resource compiler failure is not fatal in strict mode")

# Test network workers must have bounded reads and bounded joins; accept timeouts alone are not
# enough when a client connects but never sends a complete request.
require('set_read_timeout(Some(timeout))' in mcp_backend and 'join_with_timeout' in mcp_backend, "cadcraft-mcp backend tests: stream/join watchdog missing")
require('set_read_timeout(Some(timeout))' in mcp_tests and 'did not finish within 6s' in mcp_tests, "cadcraft-mcp protocol tests: stream/join watchdog missing")

# cadcraft-web deliberately has a native stub so Full Workspace can compile every member on Windows.
web_main = SRC / "apps/cadcraft-web/src/main.rs"
web_text = read(web_main)
require('#[cfg(not(target_arch = "wasm32"))]' in web_text and 'fn main()' in web_text, "cadcraft-web: native Windows stub main missing; --workspace --all-targets would fail")

# Release version and Windows installer must stay aligned.
config = read(SRC / ".cargo/config.toml")
iss = read(ROOT / "installer/TBT360_CAD_R65.iss")
m = re.search(r'CAD360_RELEASE_VERSION\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"', config)
i = re.search(r'#define\s+MyAppVersion\s+"([0-9]+\.[0-9]+\.[0-9]+)"', iss)
require(bool(m and i), "release version missing from Cargo env or installer")
if m and i:
    require(m.group(1) == i.group(1) == "0.68.0", f"release version mismatch: Cargo={m.group(1)} Installer={i.group(1)}")
require('DestName: "CAD360.exe"' in iss, "installer: GUI must install as CAD360.exe")
require('Filename: "{app}\\CAD360.exe"' in iss, "installer: shortcuts/run target must use CAD360.exe")


# Optimistic save conflict guard: saving back to the same drawing file must not silently overwrite
# bytes changed by another process after OPEN/QSAVE.
file_cmd = read(SRC / "crates/engine/src/cmd/file.rs")
doc_state = read(SRC / "crates/engine/src/doc_state.rs")
require("pub file_fingerprint: Option<u64>" in doc_state, "engine save conflict: DocState file fingerprint missing")
require("fn ensure_file_not_changed_elsewhere" in file_cmd, "engine save conflict: optimistic save guard missing")
require("save conflict:" in file_cmd and "changed on disk" in file_cmd, "engine save conflict: external-change error missing")
require("qsave_refuses_overwriting_a_file_changed_by_another_process" in file_cmd, "engine save conflict: regression test missing")


# UI handover safeguards: command failures must be visible outside command history, the live QTO
# workspace must not hard-split CAD 50/50, and dock palettes must yield before the drawing becomes
# too narrow to edit reliably on mixed-DPI/smaller windows.
ui_lib = read(SRC / "crates/ui-egui/src/lib.rs")
desktop_main = read(SRC / "apps/cadcraft/src/main.rs")
docking = read(SRC / "crates/ui-egui/src/docking.rs")
require("self.set_status(e.clone());" in ui_lib and "command_errors_are_visible_in_the_status_bar_as_well_as_history" in ui_lib, "ui handover: command errors are not surfaced in the status bar")
require("fn takeoff_pane_width" in desktop_main and "ui.columns(2" not in desktop_main, "ui handover: QTO workspace still uses a rigid 50/50 CAD split")
require("QTO must never consume half of the CAD workspace" in desktop_main, "ui handover: responsive QTO pane regression missing")
require("Panel::Canvas => (egui::vec2(360.0, 220.0)" in docking, "ui handover: CAD canvas minimum dock area missing")

# Locked-layer editing must be blocked in both full Properties and Quick Properties. The Engine
# remains authoritative, but the UI must not advertise editable controls for locked selections.
palettes = read(SRC / "crates/ui-egui/src/palettes.rs")
quick = read(SRC / "crates/ui-egui/src/quick.rs")
require("let selection_locked = !locked_layers.is_empty();" in palettes and "ui.disable();" in palettes and "Read-only — locked layer" in palettes, "ui locked layer: Properties palette read-only guard missing")
require("let selection_locked = !locked_layers.is_empty();" in quick and "ui.add_enabled_ui(!selection_locked" in quick and "Read-only — locked layer" in quick, "ui locked layer: Quick Properties read-only guard missing")

# Collaboration Center must stay task-oriented instead of regressing to one long technical form.
bim_collab = read(SRC / "apps/cadcraft/src/bim_collab.rs")
require("pub enum CollaborationTab" in bim_collab and "Phối hợp thiết kế" in bim_collab and "Không gian làm việc cộng tác BIM" in bim_collab, "BIM collaboration: approved Coordination Center shell missing")
for label in ["Tổng quan", "Không gian phối hợp", "Issue / Clash", "Đồng bộ", "Quyền truy cập"]:
    require(label in bim_collab, f"BIM collaboration: missing task tab {label}")
require("Người đang phối hợp" in bim_collab and "Issue mở" in bim_collab and "Thay đổi chờ đồng bộ" in bim_collab and "Quy trình phối hợp" in bim_collab and "Danh sách Issue / Clash" in bim_collab and "Ai đang làm gì" in bim_collab and "Thông tin phiên thiết kế" in bim_collab, "BIM collaboration: approved dashboard/workflow layout regressed")
require("Thao tác nhanh" in bim_collab and "Tạo Issue từ selection" in bim_collab and "Chỉ việc của tôi" in bim_collab and "Cần đồng bộ và xử lý xung đột trước" in bim_collab and "Mở xử lý conflict" in bim_collab, "BIM collaboration: task-oriented quick actions/filter/safe-close workflow regressed")
require("pub enum IssueViewFilter" in bim_collab and "Đang hoạt động" in bim_collab and "Tập trung Issue" in bim_collab and "Xóa lọc" in bim_collab and "Cửa sổ hẹp: panel Ai đang làm gì / Phiên thiết kế được thu gọn" in bim_collab and "Tất cả bộ môn" in bim_collab and "Việc tiếp theo của tôi" in bim_collab and "Hiển thị {} / {} Issue" in bim_collab, "BIM collaboration: responsive triage/filter/focus workflow regressed")
require("pub enum IssuePriority" in bim_collab and "P0 Khẩn cấp" in bim_collab and "Tất cả ưu tiên" in bim_collab and "Chọn tất cả đang lọc" in bim_collab and "Nhận {} Issue" in bim_collab and "Hoàn tất {} Issue" in bim_collab, "BIM collaboration: priority/bulk triage workflow regressed")
require("collaboration_center_opens_on_overview_with_task_oriented_tabs" in bim_collab, "BIM collaboration: tab/default regression test missing")
require("mod bim_collab;" in desktop_main and "BimCollab" in desktop_main and "bim_collab_state: bim_collab::State" in desktop_main and "BIM CỘNG TÁC" in desktop_main and "bim_collab::panel" in desktop_main, "BIM collaboration: workspace is not wired into the real desktop host")
require("self.cad.ui(ui);" in desktop_main and "bim_coordination_workspace" in desktop_main and "Quay lại bản vẽ" in desktop_main, "BIM collaboration: CAD360 shell-preserving in-context workspace regressed")

# High-risk source files must be protected by SHA256SUMS and hashes must be current.
manifest = read(ROOT / "SHA256SUMS.txt")
critical = [
    "SOURCE_CAD_R65/crates/engine/src/cmd/draw2/divide.rs",
    "SOURCE_CAD_R65/crates/engine/src/cmd/draw2/tancircle.rs",
    "SOURCE_CAD_R65/crates/engine/src/cmd/clipboard.rs",
    "SOURCE_CAD_R65/crates/engine/src/cmd/blocks/insert_file.rs",
    "SOURCE_CAD_R65/crates/engine/src/cmd/blocks/xref/load.rs",
    "SOURCE_CAD_R65/crates/engine/src/cmd/blocks/xref/bind.rs",
    "SOURCE_CAD_R65/crates/engine/src/cmd/exchange.rs",
    "SOURCE_CAD_R65/crates/engine/src/cmd/file.rs",
    "SOURCE_CAD_R65/crates/engine/src/doc_state.rs",
    "SOURCE_CAD_R65/apps/cadcraft-cli/Cargo.toml",
    "SOURCE_CAD_R65/crates/doc/Cargo.toml",
    "SOURCE_CAD_R65/crates/engine/Cargo.toml",
    "SOURCE_CAD_R65/crates/io/Cargo.toml",
    "SOURCE_CAD_R65/crates/mcp/Cargo.toml",
    "SOURCE_CAD_R65/crates/render/Cargo.toml",
    "SOURCE_CAD_R65/crates/ui-egui/Cargo.toml",
    "SOURCE_CAD_R65/crates/engine/tests/it/compare_export.rs",
    "SOURCE_CAD_R65/crates/ui-egui/tests/it/control_methods.rs",
    "SOURCE_CAD_R65/apps/cadcraft-cli/tests/it/mcp_connect.rs",
    "SOURCE_CAD_R65/apps/cadcraft-cli/tests/it/cli.rs",
    "SOURCE_CAD_R65/crates/mcp/src/backend.rs",
    "SOURCE_CAD_R65/crates/mcp/src/tests.rs",
    "SOURCE_CAD_R65/crates/ui-egui/src/capture/tests.rs",
    "SOURCE_CAD_R65/crates/engine/tests/autosave.rs",
    "SOURCE_CAD_R65/crates/engine/tests/io_hooks_thread_local.rs",
    "SOURCE_CAD_R65/apps/cadcraft/src/control_server.rs",
    "SOURCE_CAD_R65/crates/ui-egui/src/control.rs",
    "SOURCE_CAD_R65/apps/cadcraft/build.rs",
    "SOURCE_CAD_R65/apps/cadcraft/src/takeoff.rs",
    "SOURCE_CAD_R65/crates/ui-egui/src/docking.rs",
    "SOURCE_CAD_R65/crates/ui-egui/src/lib.rs",
    "SOURCE_CAD_R65/crates/ui-egui/src/palettes.rs",
    "SOURCE_CAD_R65/crates/ui-egui/src/quick.rs",
    "SOURCE_CAD_R65/apps/cadcraft/src/main.rs",
    "SOURCE_CAD_R65/apps/cadcraft/src/bim_collab.rs",
    "installer/TBT360_CAD_R65.iss",
]
entries = {}
for line in manifest.splitlines():
    line = line.strip()
    if not line:
        continue
    mm = re.match(r"^([0-9a-fA-F]{64})\s+(.+)$", line)
    if mm:
        entries[mm.group(2).lstrip("*")] = mm.group(1).lower()
for rel in critical:
    require(rel in entries, f"SHA256SUMS.txt: missing critical path {rel}")
    path = ROOT / rel
    if path.exists() and rel in entries:
        # Git for Windows may materialize text files as CRLF.  The manifest is
        # generated from canonical LF source, so normalize line endings before
        # hashing to make this release gate deterministic on Linux and Windows.
        data = path.read_bytes().replace(b"\r\n", b"\n")
        actual = hashlib.sha256(data).hexdigest()
        require(actual == entries[rel], f"SHA256SUMS.txt: stale hash for {rel}")

# Prevent old compile regressions from silently coming back.
all_rust = "\n".join(read(p) for p in (SRC / "crates/engine/src").rglob("*.rs"))
require("egui::SidePanel" not in read(SRC / "apps/cadcraft/src/main.rs"), "legacy egui::SidePanel API reintroduced in desktop host")

if errors:
    for e in errors:
        print(f"R67 STATIC GATE ERROR: {e}", file=sys.stderr)
    raise SystemExit(1)
print("R67 static preflight gate passed")