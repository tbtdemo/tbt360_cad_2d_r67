@echo off
setlocal
cd /d "%~dp0SOURCE_CAD_R65"
where cargo >nul 2>&1
if errorlevel 1 (echo Rust/Cargo 1.95+ required ON BUILD MACHINE OR WINDOWS CI. & pause & exit /b 1)
cargo test -p cadcraft --locked
if errorlevel 1 (echo Native tests failed. & pause & exit /b 1)
cargo build --release --locked -p cadcraft --bin cadcraft
if errorlevel 1 (echo Native build failed. & pause & exit /b 1)
if not exist "target\release\cadcraft.exe" (echo Missing binary. & pause & exit /b 1)
if not exist "..\BUILD_OUTPUT_R65" mkdir "..\BUILD_OUTPUT_R65"
copy /y "target\release\cadcraft.exe" "..\BUILD_OUTPUT_R65\cadcraft.exe" >nul
copy /y "..\CHAY_EXE_R65_SAU_BUILD.bat" "..\BUILD_OUTPUT_R65\CHAY_TBT360.bat" >nul
certutil -hashfile "..\BUILD_OUTPUT_R65\cadcraft.exe" SHA256
echo Build complete: BUILD_OUTPUT_R65\cadcraft.exe
pause
