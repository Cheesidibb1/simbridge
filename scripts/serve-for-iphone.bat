@echo off
setlocal

rem Builds the web app if needed, then starts the server for iPhone Safari.
rem Usage: scripts\serve-for-iphone.bat [port]

for %%I in ("%~dp0..") do set "ROOT=%%~fI"
set "PORT=%~1"
if not defined PORT set "PORT=8080"
set "WEB_DIR=%ROOT%\companion\build\web"

echo Building the latest companion web app...
call "%~dp0build-web.bat"
if errorlevel 1 exit /b 1

where cargo >nul 2>nul
if errorlevel 1 (
    echo Cargo was not found on PATH. Install Rust from https://www.rust-lang.org/tools/install. 1>&2
    exit /b 1
)

pushd "%ROOT%\server" || exit /b 1
cargo run --release -- --port "%PORT%" --web-dir "%WEB_DIR%"
set "SERVER_EXIT_CODE=%ERRORLEVEL%"
popd
exit /b %SERVER_EXIT_CODE%