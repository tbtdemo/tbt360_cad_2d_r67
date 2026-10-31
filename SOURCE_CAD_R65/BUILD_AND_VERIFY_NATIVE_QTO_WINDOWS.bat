@echo off
setlocal
cd /d "%~dp0"
where cargo >nul 2>&1 || (echo [BLOCKED] Need Windows Rust build environment; do NOT install on end-user PC. & exit /b 2)
cargo test -p cadcraft --locked || exit /b 1
cargo build --release -p cadcraft --locked || exit /b 1
if not exist "target\release\cadcraft.exe" (echo [ERROR] Real EXE missing & exit /b 1)
echo [PASS] target\release\cadcraft.exe
endlocal
