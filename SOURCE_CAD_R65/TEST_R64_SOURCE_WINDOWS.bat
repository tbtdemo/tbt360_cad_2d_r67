@echo off
setlocal
cd /d "%~dp0"
where cargo >nul 2>nul
if errorlevel 1 (
  echo Rust Cargo not found. This BAT is only for developer testing, not running TBT360.
  pause
  exit /b 1
)
cargo test -p cadcraft-ui-egui grip_pick_prefers_nearest_and_rejects_outside_canvas
set RC=%ERRORLEVEL%
pause
exit /b %RC%
