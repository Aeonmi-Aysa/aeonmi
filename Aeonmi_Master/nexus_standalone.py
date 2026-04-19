#!/usr/bin/env python3
"""
nexus_standalone.py -- Aeonmi Nexus Launcher
Starts dashboard.py (Flask), then opens the Nexus in Edge app-mode window.
Falls back to system browser if Edge isn't found.
"""

import sys
import os
import subprocess
import threading
import time
import traceback
import signal
from pathlib import Path

SCRIPT_DIR    = Path(__file__).parent.resolve()
PROJECT_ROOT  = SCRIPT_DIR.parent
DASHBOARD     = SCRIPT_DIR / "dashboard.py"
DASHBOARD_URL = "http://localhost:7777/"

WINDOW_TITLE  = "Aeonmi Nexus"
WINDOW_W      = 1440
WINDOW_H      = 900

dashboard_proc = None

# ── Dashboard server ──────────────────────────────────────────────────────────

def start_dashboard():
    global dashboard_proc
    python = sys.executable
    env = os.environ.copy()
    env["PYTHONIOENCODING"] = "utf-8"
    env["FLASK_ENV"] = "production"

    try:
        # Let stdout/stderr go directly to console — no pipe — so errors are visible immediately
        dashboard_proc = subprocess.Popen(
            [python, str(DASHBOARD)],
            cwd=str(PROJECT_ROOT),
            env=env,
        )
        print(f"[nexus] Dashboard started (PID {dashboard_proc.pid})", flush=True)
        dashboard_proc.wait()  # block until dashboard exits (so its output isn't lost)
        print(f"[nexus] Dashboard process exited (code {dashboard_proc.returncode})", flush=True)
    except Exception as e:
        print(f"[nexus] ERROR starting dashboard: {e}", flush=True)
        traceback.print_exc()


def stop_dashboard():
    global dashboard_proc
    if dashboard_proc and dashboard_proc.poll() is None:
        dashboard_proc.terminate()
        try:
            dashboard_proc.wait(timeout=3)
        except Exception:
            dashboard_proc.kill()
    dashboard_proc = None
    print("[nexus] Dashboard stopped", flush=True)


def wait_for_dashboard(timeout=20.0) -> bool:
    import urllib.request
    deadline = time.time() + timeout
    while time.time() < deadline:
        # Check if dashboard process died
        if dashboard_proc is not None and dashboard_proc.poll() is not None:
            print(f"[nexus] ERROR: Dashboard process exited early (code {dashboard_proc.returncode})", flush=True)
            return False
        try:
            with urllib.request.urlopen(DASHBOARD_URL, timeout=1) as r:
                if r.status == 200:
                    print("[nexus] Dashboard ready", flush=True)
                    return True
        except Exception:
            time.sleep(0.4)
    print("[nexus] WARNING: Dashboard did not respond in time — opening anyway", flush=True)
    return False


# ── Native window (Edge app mode — no pywebview needed) ───────────────────────

def find_edge():
    candidates = [
        r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
        r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    ]
    for p in candidates:
        if Path(p).exists():
            return p
    # Try where
    try:
        r = subprocess.run(["where", "msedge"], capture_output=True, text=True)
        if r.returncode == 0:
            return r.stdout.strip().splitlines()[0]
    except Exception:
        pass
    return None


def find_chrome():
    candidates = [
        r"C:\Program Files\Google\Chrome\Application\chrome.exe",
        r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        os.path.expandvars(r"%LOCALAPPDATA%\Google\Chrome\Application\chrome.exe"),
    ]
    for p in candidates:
        if Path(p).exists():
            return p
    return None


def launch_window(url: str):
    """Open URL as a standalone app window (no browser chrome)."""

    # 1. Try Edge app mode
    edge = find_edge()
    if edge:
        print(f"[nexus] Opening in Edge app mode: {edge}", flush=True)
        try:
            subprocess.Popen([
                edge,
                f"--app={url}",
                f"--window-size={WINDOW_W},{WINDOW_H}",
                "--window-position=80,60",
                "--disable-extensions",
            ])
            return True
        except Exception as e:
            print(f"[nexus] Edge launch failed: {e}", flush=True)

    # 2. Try Chrome app mode
    chrome = find_chrome()
    if chrome:
        print(f"[nexus] Opening in Chrome app mode: {chrome}", flush=True)
        try:
            subprocess.Popen([
                chrome,
                f"--app={url}",
                f"--window-size={WINDOW_W},{WINDOW_H}",
            ])
            return True
        except Exception as e:
            print(f"[nexus] Chrome launch failed: {e}", flush=True)

    # 3. Try pywebview
    try:
        import webview
        print("[nexus] Opening in pywebview window...", flush=True)
        window = webview.create_window(
            WINDOW_TITLE, url=url,
            width=WINDOW_W, height=WINDOW_H,
            resizable=True, text_select=True,
            background_color="#08080e",
        )
        def on_closed():
            stop_dashboard()
        window.events.closed += on_closed
        webview.start(debug=False, private_mode=False)
        return True
    except ImportError:
        pass
    except Exception as e:
        print(f"[nexus] pywebview failed: {e}", flush=True)

    # 4. Fallback — system browser
    import webbrowser
    print("[nexus] Falling back to system browser...", flush=True)
    webbrowser.open(url)
    return False


# ── Main ──────────────────────────────────────────────────────────────────────

def main():
    print("""
  +------------------------------------------------------+
  |        AEONMI NEXUS -- STANDALONE LAUNCHER           |
  +------------------------------------------------------+
  |  3-panel Nexus: Files | Mother AI | Shard Canvas     |
  +------------------------------------------------------+
""", flush=True)

    def shutdown(sig, frame):
        print("\n[nexus] Shutting down...", flush=True)
        stop_dashboard()
        sys.exit(0)

    try:
        signal.signal(signal.SIGINT, shutdown)
        signal.signal(signal.SIGTERM, shutdown)
    except Exception as e:
        print(f"[nexus] Signal setup warning: {e}", flush=True)

    # Start dashboard in background thread (streams output to console)
    t = threading.Thread(target=start_dashboard, daemon=True)
    t.start()

    # Wait for it to be ready
    ready = wait_for_dashboard(timeout=20.0)

    if not ready and dashboard_proc is not None and dashboard_proc.poll() is not None:
        print("[nexus] Dashboard failed to start. Check errors above.", flush=True)
        input("Press Enter to exit...")
        sys.exit(1)

    # Launch the native window
    used_embedded = launch_window(DASHBOARD_URL)

    if not used_embedded:
        # Browser opened — keep server alive until user quits
        print("[nexus] Server running. Close this window or press Ctrl+C to stop.", flush=True)
        try:
            while True:
                time.sleep(1)
                if dashboard_proc and dashboard_proc.poll() is not None:
                    print("[nexus] Dashboard server exited.", flush=True)
                    break
        except KeyboardInterrupt:
            pass
    else:
        # Edge/Chrome app window opened — keep server alive while browser is open
        # We can't detect browser close, so just keep server running
        print("[nexus] App window launched. Keep this console open to maintain the server.", flush=True)
        print("[nexus] Press Ctrl+C or close this window to stop.", flush=True)
        try:
            while True:
                time.sleep(2)
                if dashboard_proc and dashboard_proc.poll() is not None:
                    print("[nexus] Dashboard server exited unexpectedly.", flush=True)
                    break
        except KeyboardInterrupt:
            pass

    stop_dashboard()


if __name__ == "__main__":
    try:
        main()
    except Exception:
        print("\n[nexus] FATAL ERROR:", flush=True)
        traceback.print_exc()
        input("\nPress Enter to exit...")
        sys.exit(1)
