@echo off
setlocal

rem Builds the SimBridge companion web app for iPhone Safari.
rem Usage: scripts\build-web.bat [--profile]

for %%I in ("%~dp0..\companion") do set "COMPANION_DIR=%%~fI"
pushd "%COMPANION_DIR%" || exit /b 1

where flutter >nul 2>nul
if errorlevel 1 (
    echo Flutter 3.22 or newer was not found on PATH. Install Flutter from https://docs.flutter.dev/get-started/install. 1>&2
    popd
    exit /b 1
)

set "MODE=--release"
if /I "%~1"=="--profile" set "MODE=--profile"

echo flutter pub get
call flutter pub get
if errorlevel 1 goto :failed

echo flutter build web %MODE%
call flutter build web %MODE% --base-href / --no-web-resources-cdn
if errorlevel 1 goto :failed

popd
echo Built companion\build\web
echo Start the server with scripts\serve-for-iphone.bat, then open the iPhone Safari URL it prints.
exit /b 0

:failed
set "BUILD_EXIT_CODE=%ERRORLEVEL%"
popd
exit /b %BUILD_EXIT_CODE%