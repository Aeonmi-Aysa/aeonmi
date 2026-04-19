@echo off
echo.
echo  Stopping any process on port 7777...
for /f "tokens=5" %%a in ('netstat -aon ^| findstr ":7777 "') do (
    taskkill /F /PID %%a >nul 2>&1
)
echo  Starting Aeonmi Nexus Dashboard...
echo.
cd /d "%~dp0"
python Aeonmi_Master\dashboard.py
