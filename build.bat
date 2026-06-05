@echo off
echo =========================================
echo Building rMonitor (Rust System Monitor)
echo =========================================
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo [ERROR] Cargo build failed!
    exit /b %ERRORLEVEL%
)
copy /y target\release\rmon.exe .\rmon.exe
echo =========================================
echo Build successful! Run .\rmon.exe to start.
echo =========================================
