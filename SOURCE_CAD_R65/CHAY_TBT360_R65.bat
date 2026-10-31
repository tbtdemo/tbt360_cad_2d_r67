@echo off
setlocal EnableExtensions
cd /d "%~dp0"
if not exist "%~dp0TBT360.exe" (
 echo ERROR: This package has SOURCE ONLY, not a newly built R65 executable.
 echo Build on a Windows developer computer with BUILD_AND_PACKAGE_WINDOWS_R65.bat.
 pause
 exit /b 2
)
start "TBT360" "%~dp0TBT360.exe" %*
