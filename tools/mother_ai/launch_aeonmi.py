#!/usr/bin/env python3
"""
Aeonmi Command Center — Standalone Launcher
Starts the dashboard server if not running, then opens Chrome in --app mode.
"""
import subprocess, sys, time, urllib.request, os, pathlib, shutil

PYTHON  = r"C:\Users\wlwil\AppData\Local\Programs\Python\Python311\python.exe"
DASH    = r"C:\Temp\dashboard.py"
PORT    = 7777
URL     = f"http://localhost:{PORT}"
LOG     = r"C:\Temp\dash_server.log"

CHROME_PATHS = [
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    os.path.expandvars(r"%LOCALAPPDATA%\Google\Chrome\Application\chrome.exe"),
]

def server_alive():
    try:
        urllib.request.urlopen(URL + "/api/status", timeout=2)
        return True
    except Exception:
        return False

def find_chrome():
    for p in CHROME_PATHS:
        if os.path.exists(p):
            return p
    # Try PATH
    found = shutil.which("chrome") or shutil.which("google-chrome")
    return found

def start_server():
    print("Starting Aeonmi dashboard server...")
    with open(LOG, "w") as log:
        subprocess.Popen(
            [PYTHON, DASH],
            stdout=log, stderr=log,
            creationflags=subprocess.CREATE_NO_WINDOW
        )

def wait_for_server(timeout=20):
    for i in range(timeout * 2):
        if server_alive():
            print(f"Server ready after {i*0.5:.1f}s")
            return True
        time.sleep(0.5)
        if i % 4 == 0:
            print(f"  waiting... ({i*0.5:.0f}s)")
    return False

def launch_app():
    chrome = find_chrome()
    if not chrome:
        print("Chrome not found — opening default browser instead")
        import webbrowser
        webbrowser.open(URL)
        return

    profile_dir = r"C:\Temp\aeonmi_chrome_profile"
    os.makedirs(profile_dir, exist_ok=True)

    cmd = [
        chrome,
        f"--app={URL}",
        f"--user-data-dir={profile_dir}",
        "--window-size=1440,900",
        "--window-position=80,40",
        "--no-first-run",
        "--no-default-browser-check",
        "--disable-extensions",
        "--disable-background-networking",
        "--disable-sync",
    ]
    print("Launching Aeonmi in app window...")
    subprocess.Popen(cmd)

if __name__ == "__main__":
    if not server_alive():
        start_server()
        ok = wait_for_server(20)
        if not ok:
            print("ERROR: Server did not start in time. Check:", LOG)
            sys.exit(1)
    else:
        print("Server already running.")
    launch_app()
    print("Done.")
