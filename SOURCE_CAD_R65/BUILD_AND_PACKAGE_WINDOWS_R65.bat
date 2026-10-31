@echo off
setlocal EnableExtensions
cd /d "%~dp0"
where cargo >nul 2>nul
if errorlevel 1 (
 echo ERROR: Rust Cargo is not installed. Install Rust 1.95+ on the BUILD computer.
 pause
 exit /b 2
)
if not exist "target\release" mkdir "target\release"
echo Running UI regression tests...
cargo test -p cadcraft-ui-egui grip_pick_ --locked
if errorlevel 1 (
 echo ERROR: UI tests failed. No package created.
 pause
 exit /b 3
)
echo Building Windows desktop executable from current source...
cargo build --release -p cadcraft --locked
if errorlevel 1 (
 echo ERROR: Build failed. No package created.
 pause
 exit /b 4
)
if not exist "target\release\cadcraft.exe" (
 echo ERROR: New executable missing. No package created.
 pause
 exit /b 5
)
if not exist "R65_WINDOWS_READY" mkdir "R65_WINDOWS_READY"
copy /y "target\release\cadcraft.exe" "R65_WINDOWS_READY\TBT360.exe" >nul
copy /y "CHAY_TBT360_R65.bat" "R65_WINDOWS_READY\CHAY_TBT360_R65.bat" >nul
echo SUCCESS: R65_WINDOWS_READY contains a newly compiled executable.
pause
