@echo off
REM ============================================================
REM  launch_nexus.bat -- Aeonmi Nexus Launcher
REM ============================================================

title Aeonmi Nexus Server

cd /d "%~dp0.."

where python >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Python not found. Install from https://python.org
    pause & exit /b 1
)

python -c "import flask" >nul 2>&1
if %errorlevel% neq 0 (
    echo Installing flask...
    pip install flask --quiet
)

REM Kill any existing process on 7777 first
for /f "tokens=5" %%p in ('netstat -aon 2^>nul ^| findstr ":7777 "') do (
    taskkill /F /PID %%p >nul 2>&1
)

echo.
echo  +-------------------------------------------------+
echo  ^|  AEONMI NEXUS  ^|  http://localhost:7777         ^|
echo  ^|  Keep this window open. Ctrl+C to stop.        ^|
echo  +-------------------------------------------------+
echo.

REM Generate timestamp to bust Edge's aggressive app-mode cache
for /f %%t in ('python -c "import time; print(int(time.time()))"') do set TS=%%t

REM Kill any existing Edge --app=localhost:7777 instances (stale cached window)
taskkill /F /IM msedge.exe /FI "WINDOWTITLE eq Aeonmi Nexus" >nul 2>&1

set EDGE1=C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe
set EDGE2=C:\Program Files\Microsoft\Edge\Application\msedge.exe
if exist "%EDGE1%" (
    start "" "%EDGE1%" --app=http://localhost:7777/?v=%TS% --window-size=1440,900 --window-position=80,60 --disable-application-cache
) else if exist "%EDGE2%" (
    start "" "%EDGE2%" --app=http://localhost:7777/?v=%TS% --window-size=1440,900 --window-position=80,60 --disable-application-cache
) else (
    start http://localhost:7777/?v=%TS%
)

REM Run Flask in foreground
python -u Aeonmi_Master\dashboard.py

echo.
echo [nexus] Server stopped.
pause
