#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
nexus_relay.py -- Aeonmi Nexus Relay Server
============================================
HTTP bridge between aeonmi_shard_crystal.html and the live Mother embryo_loop.

Listens at localhost:9393
POST /mother  {input:"text"}  -> {output:"...", error:"..."}
GET  /health                  -> {status:"ok", ...}
GET  /genesis                 -> genesis.json contents
POST /setkey  {provider:"claude", key:"sk-ant-..."}

Start:  python Aeonmi_Master/nexus_relay.py
"""

import json
import os
import re
import sys
import subprocess
import threading
import time
import queue
import signal
from http.server import HTTPServer, BaseHTTPRequestHandler
from pathlib import Path

_ANSI_RE = re.compile(r'\x1b\[[0-9;]*[mGKHF]|\x1b\[[0-9;]*m')

def strip_ansi(text: str) -> str:
    return _ANSI_RE.sub("", text)

# Force UTF-8 on Windows terminal
if sys.platform == "win32":
    import io
    try:
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace", line_buffering=True)
        sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding="utf-8", errors="replace", line_buffering=True)
    except Exception:
        pass

# ── Config ────────────────────────────────────────────────────────────────────

PORT        = 9393
HOST        = "localhost"
PROMPT_MARK = "mother"          # part of REPL prompt "# mother >" or "mother >"
TIMEOUT     = 60.0

SCRIPT_DIR   = Path(__file__).parent.resolve()
PROJECT_ROOT = SCRIPT_DIR.parent
GENESIS_PATH = SCRIPT_DIR / "genesis.json"

# Binary resolution
def _find_binary():
    candidates = [
        PROJECT_ROOT / "target" / "release" / "Aeonmi.exe",
        PROJECT_ROOT / "target" / "release" / "aeonmi_project.exe",
        Path("C:/RustTarget/release/aeonmi_project.exe"),
        Path("C:/RustTarget/release/Aeonmi.exe"),
        PROJECT_ROOT / "target" / "release" / "aeonmi",
    ]
    for c in candidates:
        if c.exists():
            return c
    return candidates[2]  # default fallback with useful path in error

BINARY = _find_binary()

# ── Threaded stdout reader ────────────────────────────────────────────────────

class AsyncReader:
    """Reads lines from a subprocess stdout in a background thread."""

    def __init__(self, stream):
        self._q     = queue.Queue()
        self._stream = stream
        self._t     = threading.Thread(target=self._run, daemon=True)
        self._t.start()

    def _run(self):
        try:
            for line in iter(self._stream.readline, ""):
                self._q.put(line)
        except Exception:
            pass
        self._q.put(None)  # sentinel

    def readline(self, timeout=1.0):
        """Return next line or None on timeout/EOF."""
        try:
            return self._q.get(timeout=timeout)
        except queue.Empty:
            return ""

    def read_until(self, marker: str, timeout: float = 60.0) -> str:
        """Read lines until one contains marker, or timeout."""
        buf   = []
        deadline = time.time() + timeout
        while time.time() < deadline:
            remaining = deadline - time.time()
            line = self.readline(timeout=min(1.0, remaining))
            if line is None:          # EOF
                break
            if line:
                buf.append(line)
                if marker in line:
                    break
        return "".join(buf)


# ── Mother process manager ────────────────────────────────────────────────────

class MotherProcess:
    def __init__(self):
        self._proc   = None
        self._reader = None
        self._lock   = threading.Lock()
        self._boot   = ""

    def _env(self):
        env = os.environ.copy()
        env["AEONMI_PASSPHRASE"] = ""   # skip glyph ceremony in relay
        env["PYTHONIOENCODING"] = "utf-8"
        return env

    def start(self) -> bool:
        if not BINARY.exists():
            print(f"[relay] ERROR: binary not found: {BINARY}")
            return False
        try:
            self._proc = subprocess.Popen(
                [str(BINARY), "mother"],
                stdin=subprocess.PIPE,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,   # merge stderr into stdout
                text=True,
                encoding="utf-8",
                errors="replace",
                env=self._env(),
                cwd=str(PROJECT_ROOT),
            )
            self._reader = AsyncReader(self._proc.stdout)
            print(f"[relay] Mother process started (PID {self._proc.pid})")
            # Drain boot banner -- wait for the REPL prompt
            self._boot = self._reader.read_until("mother", timeout=12.0)
            print(f"[relay] Mother ready ({len(self._boot)} boot chars captured)")
            return True
        except Exception as e:
            print(f"[relay] Failed to start Mother: {e}")
            return False

    def alive(self) -> bool:
        return self._proc is not None and self._proc.poll() is None

    def stop(self):
        if self._proc and self._proc.poll() is None:
            try:
                self._proc.stdin.write("exit\n")
                self._proc.stdin.flush()
            except Exception:
                pass
            try:
                self._proc.terminate()
                self._proc.wait(timeout=3)
            except Exception:
                self._proc.kill()
        self._proc  = None
        self._reader = None

    def restart(self) -> bool:
        print("[relay] Restarting Mother...")
        self.stop()
        time.sleep(0.5)
        return self.start()

    def send(self, text: str) -> tuple:
        """Send input to Mother, return (output, error). Thread-safe."""
        with self._lock:
            if not self.alive():
                if not self.restart():
                    return "", "Mother process could not be started."
            try:
                self._proc.stdin.write(text.strip() + "\n")
                self._proc.stdin.flush()
                raw   = self._reader.read_until("mother", timeout=TIMEOUT)
                cleaned = strip_ansi(raw)
                lines = [l for l in cleaned.splitlines()
                         if not ("mother" in l.lower() and len(l.strip()) < 25)]
                return "\n".join(lines).strip(), ""
            except BrokenPipeError:
                self._proc = None
                return "", "Mother pipe broke -- will restart on next request."
            except Exception as e:
                return "", f"Relay error: {e}"


mother = MotherProcess()


# ── HTTP handler ──────────────────────────────────────────────────────────────

class NexusHandler(BaseHTTPRequestHandler):

    def log_message(self, fmt, *args):
        if args and str(args[1]) not in ("200", "204"):
            print(f"[relay] {self.address_string()} {fmt % args}")

    def _cors(self):
        self.send_header("Access-Control-Allow-Origin", "*")
        self.send_header("Access-Control-Allow-Methods", "GET, POST, OPTIONS")
        self.send_header("Access-Control-Allow-Headers", "Content-Type")

    def _json(self, code: int, obj: dict):
        body = json.dumps(obj, ensure_ascii=False).encode("utf-8")
        self.send_response(code)
        self.send_header("Content-Type", "application/json; charset=utf-8")
        self.send_header("Content-Length", str(len(body)))
        self._cors()
        self.end_headers()
        self.wfile.write(body)

    def do_OPTIONS(self):
        self.send_response(204)
        self._cors()
        self.end_headers()

    def do_GET(self):
        if self.path == "/health":
            self._json(200, {
                "status": "ok",
                "mother_alive": mother.alive(),
                "pid": mother._proc.pid if mother._proc else None,
                "binary": str(BINARY),
                "binary_exists": BINARY.exists(),
            })

        elif self.path == "/genesis":
            try:
                data = json.loads(GENESIS_PATH.read_text(encoding="utf-8"))
                self._json(200, data)
            except Exception as e:
                self._json(200, {"error": str(e)})

        elif self.path == "/boot":
            self._json(200, {"output": mother._boot})

        else:
            self._json(404, {"error": "not found"})

    def do_POST(self):
        length = int(self.headers.get("Content-Length", 0))
        body   = self.rfile.read(length) if length > 0 else b"{}"
        try:
            payload = json.loads(body.decode("utf-8"))
        except Exception:
            self._json(400, {"error": "invalid JSON"})
            return

        if self.path == "/mother":
            text = payload.get("input", "").strip()
            if not text:
                self._json(400, {"error": "empty input"})
                return
            print(f"[relay] <- {text!r}")
            output, error = mother.send(text)
            print(f"[relay] -> {output[:80]!r}{'...' if len(output) > 80 else ''}")
            self._json(200, {"output": output, "error": error})

        elif self.path == "/setkey":
            provider = payload.get("provider", "").lower().strip()
            key      = payload.get("key", "").strip()
            env_map  = {
                "claude":     "ANTHROPIC_API_KEY",
                "anthropic":  "ANTHROPIC_API_KEY",
                "openai":     "OPENAI_API_KEY",
                "openrouter": "OPENROUTER_API_KEY",
                "deepseek":   "DEEPSEEK_API_KEY",
                "perplexity": "PERPLEXITY_API_KEY",
                "grok":       "GROK_API_KEY",
                "xai":        "GROK_API_KEY",
            }
            if provider not in env_map:
                self._json(400, {"error": f"Unknown provider. Supported: {', '.join(env_map)}"})
                return
            env_var = env_map[provider]
            if key:
                os.environ[env_var] = key
            else:
                os.environ.pop(env_var, None)
            ok = mother.restart()
            self._json(200, {
                "ok": ok,
                "env_var": env_var,
                "active": bool(key),
                "message": f"{provider} key {'set' if key else 'cleared'} -- Mother restarted.",
            })
        else:
            self._json(404, {"error": "not found"})


# ── Main ──────────────────────────────────────────────────────────────────────

def main():
    binary_exists = BINARY.exists()
    print(f"[relay] Aeonmi Nexus Relay -- localhost:{PORT}")
    print(f"[relay] Binary : {BINARY}")
    print(f"[relay] Exists : {'YES' if binary_exists else 'NO -- build with: cargo build --release'}")

    # Start Mother in background so HTTP server can accept requests immediately
    threading.Thread(target=mother.start, daemon=True).start()

    def shutdown(sig, frame):
        print("\n[relay] Shutting down...")
        mother.stop()
        sys.exit(0)

    signal.signal(signal.SIGINT, shutdown)
    if hasattr(signal, "SIGTERM"):
        signal.signal(signal.SIGTERM, shutdown)

    server = HTTPServer((HOST, PORT), NexusHandler)
    print(f"[relay] Listening at http://{HOST}:{PORT}/")
    print(f"[relay] Open Aeonmi_Master/aeonmi_shard_crystal.html or run nexus_standalone.py")
    print(f"[relay] Ctrl+C to stop\n")

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        pass
    finally:
        mother.stop()
        server.server_close()


if __name__ == "__main__":
    main()
