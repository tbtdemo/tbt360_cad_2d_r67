@echo off
setlocal EnableExtensions
cd /d "%~dp0"
if not exist "%~dp0TBT360_CAD_R65.exe" (
 echo [ERROR] Ban portable chua co EXE duoc build tu source R65.
 exit /b 2
)
start "TBT360 CAD R65" "%~dp0TBT360_CAD_R65.exe" %*
