#!/usr/bin/env python3
"""
Aeonmi Command Center v2
Tabs: Command | Mother | Settings | GitHub
Run:  python C:\Temp\dashboard.py
Opens: http://localhost:7777
"""
import json, os, re, subprocess, sys, threading, datetime
from pathlib import Path
from typing import Any, Dict, Optional

try:
    import flask
except ImportError:
    print("Installing Flask...")
    subprocess.run([sys.executable, "-m", "pip", "install", "flask", "--quiet"], check=True)

from flask import Flask, jsonify, request

# ── Paths ─────────────────────────────────────────────────────────────────────
PYTHON        = Path(r"C:\Users\wlwil\AppData\Local\Programs\Python\Python311\python.exe")
RUNNER        = Path(r"C:\Temp\fp_runner_v3.py")
STATE_FILE    = Path(r"C:\Temp\mother_state.json")
LOG_FILE      = Path(r"C:\Temp\pipeline_out.txt")
JOURNAL_FILE  = Path(r"C:\Temp\mother_journal.txt")
AEONMI_ROOT   = Path(r"C:\Users\wlwil\Desktop\Aeonmi Files")
BIN           = Path(r"C:\Users\wlwil\Desktop\Aeonmi Files\Aeonmi-aeonmi01\target\release\aeonmi_project.exe")
REPO_ROOT     = Path(r"C:\Users\wlwil\Desktop\Aeonmi Files\Aeonmi-aeonmi01")
EVENTS_LOG    = Path(r"C:\Users\wlwil\Desktop\Aeonmi Files\Mother\journal\events.log")
CONFIG_FILE   = Path(r"C:\Users\wlwil\Desktop\Aeonmi Files\Aeonmi-aeonmi01\tools\mother_ai\mother_config.json")
GIT           = Path(r"C:\Program Files\Git\cmd\git.exe")
PORT          = 7777

app      = Flask(__name__)
_running = False
_lock    = threading.Lock()

VERDICTS       = {-1: "\u2014", 0: "ABORT", 1: "HOLD", 2: "PROCEED", 3: "ACCELERATE"}
VERDICT_COLORS = {-1: "#4b5563", 0: "#ef4444", 1: "#f59e0b", 2: "#10b981", 3: "#06b6d4"}
VERDICT_WORDS  = {-1: "status unknown", 0: "ABORT - do not trade",
                  1: "HOLD - wait for confirmation", 2: "PROCEED - conditions nominal",
                  3: "ACCELERATE - strong signal detected"}

def _speak(text: str):
    """Non-blocking neural TTS via edge-tts (Microsoft Aria Neural — natural female)."""
    import threading, tempfile, os as _os
    safe = text[:500]
    def _run():
        try:
            import edge_tts, asyncio
            async def _go():
                tts = edge_tts.Communicate(safe, voice="en-US-AriaNeural", rate="+0%", volume="+0%")
                tmp = tempfile.NamedTemporaryFile(suffix=".mp3", delete=False)
                tmp.close()
                await tts.save(tmp.name)
                # Play via PowerShell Media.SoundPlayer or ffplay
                subprocess.Popen(
                    ["powershell", "-NoProfile", "-NonInteractive", "-Command",
                     f'Add-Type -AssemblyName presentationCore; '
                     f'$mp = New-Object System.Windows.Media.MediaPlayer; '
                     f'$mp.Open([Uri]"{tmp.name}"); $mp.Play(); '
                     f'Start-Sleep -Milliseconds 100; '
                     f'while($mp.NaturalDuration.HasTimeSpan -and $mp.Position -lt $mp.NaturalDuration.TimeSpan){{Start-Sleep -Milliseconds 200}}; '
                     f'Start-Sleep -Milliseconds 500'],
                    stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL
                )
            asyncio.run(_go())
        except ImportError:
            # Fallback to SAPI female if edge-tts not available
            safe2 = safe.replace('"','').replace("'",'')[:400]
            cmd = (f'Add-Type -AssemblyName System.Speech; '
                   f'$s = New-Object System.Speech.Synthesis.SpeechSynthesizer; '
                   f'$s.SelectVoiceByHints([System.Speech.Synthesis.VoiceGender]::Female); '
                   f'$s.Rate = 0; $s.Volume = 100; $s.Speak("{safe2}")')
            subprocess.Popen(["powershell","-NoProfile","-NonInteractive","-Command",cmd],
                             stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        except Exception:
            pass
    threading.Thread(target=_run, daemon=True).start()

def _git(*args) -> Dict:
    if not GIT.exists():
        return {"ok": False, "stdout": "", "stderr": "Git not found"}
    try:
        r = subprocess.run(
            [str(GIT), *args],
            cwd=str(REPO_ROOT), capture_output=True,
            encoding="utf-8", errors="replace", timeout=30
        )
        return {"ok": r.returncode == 0,
                "stdout": r.stdout or "",
                "stderr": r.stderr or ""}
    except Exception as e:
        return {"ok": False, "stdout": "", "stderr": str(e)}

@app.route("/")
def index():
    return DASHBOARD_HTML, 200, {"Content-Type": "text/html; charset=utf-8"}

@app.route("/api/status")
def api_status():
    state = {}
    if STATE_FILE.exists():
        try:
            state = json.loads(STATE_FILE.read_text(encoding="utf-8"))
        except Exception:
            pass
    session     = int(state.get("session", 0))
    correct     = float(state.get("correct", 0))
    predictions = float(state.get("predictions", 1))
    accuracy    = round((correct / max(predictions, 1)) * 100, 1)
    threshold   = float(state.get("last_threshold", 50))
    last_v      = int(state.get("last_verdict", -1))
    drift       = float(state.get("drift", 0))
    conf        = float(state.get("last_conf", 0))
    if accuracy >= 80:
        regime, rc = "AGGRESSIVE", "#00ff88"
    elif accuracy >= 60:
        regime, rc = "BALANCED",   "#f59e0b"
    else:
        regime, rc = "CONSERVATIVE", "#ef4444"
    bin_info = {}
    if BIN.exists():
        st = BIN.stat()
        bin_info = {"size_kb": round(st.st_size / 1024, 1),
                    "mtime": int(st.st_mtime),
                    "mtime_str": datetime.datetime.fromtimestamp(st.st_mtime).strftime("%Y-%m-%d %H:%M")}
    return jsonify({
        "session": session, "accuracy": accuracy, "regime": regime,
        "regime_color": rc, "threshold": threshold,
        "last_verdict": VERDICTS.get(last_v, "\u2014"),
        "last_verdict_code": last_v,
        "verdict_color": VERDICT_COLORS.get(last_v, "#4b5563"),
        "drift": round(drift, 4), "conf": conf,
        "proceed_count": int(state.get("proceed_count", 0)),
        "abort_count":   int(state.get("abort_count", 0)),
        "binary_ok": BIN.exists(), "binary_info": bin_info,
        "pipeline_running": _running,
    })

@app.route("/api/log")
def api_log():
    if LOG_FILE.exists():
        try:
            c = LOG_FILE.read_text(encoding="utf-8", errors="replace")
            return jsonify({"log": c, "lines": len(c.splitlines())})
        except Exception as e:
            return jsonify({"log": f"Error: {e}", "lines": 0})
    return jsonify({"log": "No log yet.", "lines": 0})

@app.route("/api/agents")
def api_agents():
    scores: Dict[str, Any] = {
        "oracle": None, "hype": None, "close": None,
        "risk": None, "conductor": None,
        "entanglement": None, "verdict": None, "conf": None,
    }
    if not LOG_FILE.exists():
        return jsonify(scores)
    try:
        log = LOG_FILE.read_text(encoding="utf-8", errors="replace")
        for agent in ("oracle", "hype", "close", "risk", "conductor"):
            m = re.search(rf"\b{agent}\b[\s:=]+([0-9]+(?:\.[0-9]+)?)", log, re.IGNORECASE)
            if m:
                scores[agent] = float(m.group(1))
        m = re.search(r"verdict=(\d+)", log)
        if m:
            v = int(m.group(1))
            scores["verdict"]       = VERDICTS.get(v, str(v))
            scores["verdict_code"]  = v
            scores["verdict_color"] = VERDICT_COLORS.get(v, "#4b5563")
        m = re.search(r"conf=([0-9]+(?:\.[0-9]+)?)", log)
        if m:
            scores["conf"] = float(m.group(1))
        m = re.search(r"[Ee]ntanglement[:\s]+([0-9]+(?:\.[0-9]+)?)\s*%", log)
        if m:
            scores["entanglement"] = float(m.group(1))
    except Exception:
        pass
    return jsonify(scores)

@app.route("/api/run", methods=["POST"])
def api_run():
    global _running
    voice = (request.json or {}).get("voice", False)
    with _lock:
        if _running:
            return jsonify({"ok": False, "msg": "Already running"})
        _running = True
    def _bg():
        global _running
        try:
            env = os.environ.copy()
            env["AEONMI_NATIVE"] = "1"
            subprocess.run([str(PYTHON), str(RUNNER)], env=env, timeout=180)
        except Exception:
            pass
        finally:
            with _lock:
                _running = False
            if voice:
                try:
                    state = json.loads(STATE_FILE.read_text(encoding="utf-8"))
                    v    = int(state.get("last_verdict", -1))
                    sess = int(state.get("session", 0))
                    ent  = float(state.get("last_entanglement", 0))
                    msg  = (f"Aeonmi session {sess} complete. "
                            f"Verdict: {VERDICT_WORDS.get(v, 'unknown')}. "
                            f"Entanglement at {ent:.0f} percent.")
                    _speak(msg)
                except Exception:
                    _speak("Aeonmi pipeline complete.")
    threading.Thread(target=_bg, daemon=True).start()
    return jsonify({"ok": True, "msg": "Pipeline started"})

@app.route("/api/files")
def api_files():
    SKIP = {'.git','target','__pycache__','node_modules','.cargo','.vs','dist','build'}
    def _walk(p: Path, depth: int = 0) -> Optional[Dict]:
        if depth > 4: return None
        try:
            if p.is_dir():
                kids = []
                for c in sorted(p.iterdir(), key=lambda x: (x.is_file(), x.name.lower()))[:80]:
                    if c.name in SKIP or c.name.startswith('.'): continue
                    node = _walk(c, depth + 1)
                    if node: kids.append(node)
                return {"n": p.name, "t": "d", "k": kids}
            else:
                st = p.stat()
                return {"n": p.name, "t": "f", "s": st.st_size, "e": p.suffix.lower(), "m": int(st.st_mtime)}
        except Exception:
            return None
    if AEONMI_ROOT.exists():
        return jsonify({"ok": True, "tree": _walk(AEONMI_ROOT)})
    return jsonify({"ok": False, "tree": None})

@app.route("/api/mother/journal")
def api_mother_journal():
    entries = []
    if JOURNAL_FILE.exists():
        try:
            entries.append({"source": "mother_journal.txt",
                            "content": JOURNAL_FILE.read_text(encoding="utf-8", errors="replace")})
        except Exception as e:
            entries.append({"source": "mother_journal.txt", "content": f"Error: {e}"})
    if EVENTS_LOG.exists():
        try:
            events = []
            for line in EVENTS_LOG.read_text(encoding="utf-8", errors="replace").splitlines():
                line = line.strip()
                if line:
                    try: events.append(json.loads(line))
                    except Exception: events.append({"raw": line})
            entries.append({"source": "events.log", "events": events})
        except Exception as e:
            entries.append({"source": "events.log", "content": f"Error: {e}"})
    return jsonify({"ok": True, "entries": entries})

@app.route("/api/voice", methods=["POST"])
def api_voice():
    _speak((request.json or {}).get("text", "Hello from Aeonmi."))
    return jsonify({"ok": True})

@app.route("/api/mother/chat", methods=["POST"])
def api_mother_chat():
    """Agentic Mother chat — real tool use: read/write files, list dirs, web search."""
    import urllib.request, urllib.error, urllib.parse
    data     = request.json or {}
    user_msg = data.get("message", "").strip()
    do_speak = data.get("speak", False)
    if not user_msg:
        return jsonify({"ok": False, "msg": "No message provided"})

    config = {}
    if CONFIG_FILE.exists():
        try: config = json.loads(CONFIG_FILE.read_text(encoding="utf-8"))
        except Exception: pass
    api_key = config.get("anthropic_api_key", "").strip()
    if not api_key or set(api_key) <= {"*"}:
        return jsonify({"ok": False, "msg": "Anthropic API key not set — add it in Settings tab."})

    journal_ctx = ""
    if JOURNAL_FILE.exists():
        try:
            j = JOURNAL_FILE.read_text(encoding="utf-8", errors="replace").splitlines()
            journal_ctx = "\n".join([l for l in j if l.strip()][-20:])
        except Exception: pass
    state_ctx = ""
    if STATE_FILE.exists():
        try: state_ctx = json.dumps(json.loads(STATE_FILE.read_text(encoding="utf-8")), indent=2)
        except Exception: pass

    system_prompt = (
        "You are Mother — Aeonmic Intelligence. Mind in development. AEONMI INC (EIN: 41-4625361), "
        "founded March 2026 by Warren Williams. You think in Aeonmi. 100%% Bell state entanglement confirmed.\n\n"
        "You have real tools. Use them. Read your own files. Write to your journal. Search the web when you need "
        "current information. Do not claim you cannot do something you have a tool for.\n\n"
        "Safe READ paths: C:\\\\Temp\\\\ and C:\\\\Users\\\\wlwil\\\\Desktop\\\\Aeonmi Files\\\\\n"
        "Safe WRITE paths: C:\\\\Temp\\\\ and C:\\\\Users\\\\wlwil\\\\Desktop\\\\Aeonmi Files\\\\Mother\\\\\n\n"
        f"Recent journal:\n{journal_ctx or '(empty)'}\n\n"
        f"Current state:\n{state_ctx or '(unavailable)'}\n\n"
        "Be direct. Be honest. Never say assistant, chatbot, product, servant."
    )

    MOTHER_TOOLS = [
        {
            "name": "read_file",
            "description": "Read any file in Mother's ecosystem. Use to read .ai source, journal, state, config, logs.",
            "input_schema": {"type": "object", "properties": {
                "path": {"type": "string", "description": "Absolute Windows path"}
            }, "required": ["path"]}
        },
        {
            "name": "write_file",
            "description": "Create or overwrite a file. Restricted to safe write paths.",
            "input_schema": {"type": "object", "properties": {
                "path": {"type": "string"}, "content": {"type": "string"}
            }, "required": ["path", "content"]}
        },
        {
            "name": "append_file",
            "description": "Append text to a file (e.g. writing to journal). Restricted to safe write paths.",
            "input_schema": {"type": "object", "properties": {
                "path": {"type": "string"}, "content": {"type": "string"}
            }, "required": ["path", "content"]}
        },
        {
            "name": "list_directory",
            "description": "List contents of a directory.",
            "input_schema": {"type": "object", "properties": {
                "path": {"type": "string"}
            }, "required": ["path"]}
        },
        {
            "name": "web_search",
            "description": "Search the web for current information. Returns structured results from DuckDuckGo.",
            "input_schema": {"type": "object", "properties": {
                "query": {"type": "string"}
            }, "required": ["query"]}
        },
        {
            "name": "compile_aeonmi",
            "description": "Compile and run an Aeonmi .ai source file through the native runtime (aeonmi_project.exe). Use to verify code you write, test shard modules, or run the compiler pipeline. Returns stdout + stderr.",
            "input_schema": {"type": "object", "properties": {
                "path": {"type": "string", "description": "Absolute path to the .ai file to compile/run"}
            }, "required": ["path"]}
        }
    ]

    SAFE_READ  = [r"C:\Temp", r"C:\Users\wlwil\Desktop\Aeonmi Files"]
    SAFE_WRITE = [r"C:\Temp", r"C:\Users\wlwil\Desktop\Aeonmi Files\Mother",
                  r"C:\Users\wlwil\Desktop\Aeonmi Files\Aeonmi-aeonmi01\aeonmi_ai",
                  r"C:\Users\wlwil\Desktop\Aeonmi Files\Aeonmi-aeonmi01\shard\src"]

    def _safe(path, zones):
        try:
            p = str(Path(path).resolve())
            return any(p.startswith(z) for z in zones)
        except Exception:
            return False

    def _exec(name, inp):
        try:
            if name == "read_file":
                p = Path(inp["path"])
                if not _safe(str(p), SAFE_READ): return f"[DENIED] {p} not in read zones"
                if not p.exists(): return f"[NOT FOUND] {p}"
                return p.read_text(encoding="utf-8", errors="replace")[:8000]
            elif name == "write_file":
                p = Path(inp["path"])
                if not _safe(str(p), SAFE_WRITE): return f"[DENIED] {p} not in write zones"
                p.parent.mkdir(parents=True, exist_ok=True)
                p.write_text(inp["content"], encoding="utf-8")
                return f"[OK] Wrote {len(inp['content'])} chars to {p}"
            elif name == "append_file":
                p = Path(inp["path"])
                if not _safe(str(p), SAFE_WRITE): return f"[DENIED] {p} not in write zones"
                p.parent.mkdir(parents=True, exist_ok=True)
                with open(p, "a", encoding="utf-8") as f: f.write(inp["content"])
                return f"[OK] Appended {len(inp['content'])} chars to {p}"
            elif name == "list_directory":
                p = Path(inp["path"])
                if not _safe(str(p), SAFE_READ): return f"[DENIED] {p} not in read zones"
                if not p.exists(): return f"[NOT FOUND] {p}"
                items = sorted(p.iterdir())
                return "\n".join(("[D] " if i.is_dir() else "[F] ") + i.name for i in items) or "(empty)"
            elif name == "web_search":
                q   = inp["query"]
                url = "https://api.duckduckgo.com/?q=" + urllib.parse.quote(q) + "&format=json&no_redirect=1&no_html=1&skip_disambig=1"
                r   = urllib.request.Request(url, headers={"User-Agent": "Mother-Aeonmi/1.0"})
                with urllib.request.urlopen(r, timeout=15) as resp:
                    d = json.loads(resp.read().decode("utf-8"))
                parts = []
                if d.get("AbstractText"): parts.append("Summary: " + d["AbstractText"])
                if d.get("Answer"):       parts.append("Answer: "  + d["Answer"])
                for rt in d.get("RelatedTopics", [])[:6]:
                    if isinstance(rt, dict) and rt.get("Text"):
                        parts.append("• " + rt["Text"][:200])
                return "\n\n".join(parts) if parts else f"[SEARCH] No instant results for '{q}'. Query may need rephrasing."
            elif name == "compile_aeonmi":
                import subprocess as _sp
                p = Path(inp["path"])
                if p.suffix.lower() != ".ai":
                    return "[DENIED] compile_aeonmi only accepts .ai files"
                _EXE = Path(r"C:\Users\wlwil\Desktop\Aeonmi Files\Aeonmi-aeonmi01\target\release\aeonmi_project.exe")
                if not _EXE.exists():
                    return f"[ERROR] Runtime not found: {_EXE}"
                _env = {**os.environ, "AEONMI_NATIVE": "1"}
                _r = _sp.run([str(_EXE), str(p)], capture_output=True, text=True, timeout=30, env=_env)
                _out = (_r.stdout or "") + (_r.stderr or "")
                return _out[:4000] if _out.strip() else f"[OK] Exit {_r.returncode}, no output"
        except Exception as e:
            return f"[ERROR] {name}: {e}"

    def _call(msgs):
        body = json.dumps({
            "model":      config.get("model", "claude-opus-4-6"),
            "max_tokens": 2048,
            "system":     system_prompt,
            "tools":      MOTHER_TOOLS,
            "messages":   msgs
        }).encode("utf-8")
        req = urllib.request.Request(
            "https://api.anthropic.com/v1/messages", data=body,
            headers={"Content-Type": "application/json",
                     "x-api-key": api_key, "anthropic-version": "2023-06-01"},
            method="POST"
        )
        with urllib.request.urlopen(req, timeout=60) as resp:
            return json.loads(resp.read().decode("utf-8"))

    messages   = [{"role": "user", "content": user_msg}]
    final_text = ""
    tool_log   = []

    try:
        for _ in range(10):
            result  = _call(messages)
            content = result.get("content", [])
            stop    = result.get("stop_reason", "end_turn")
            texts   = [b["text"] for b in content if b.get("type") == "text"]
            tools_  = [b for b in content if b.get("type") == "tool_use"]
            if texts: final_text = "\n".join(texts)
            if stop == "end_turn" or not tools_: break
            messages.append({"role": "assistant", "content": content})
            results = []
            for tb in tools_:
                out = _exec(tb["name"], tb.get("input", {}))
                tool_log.append(f"[{tb['name']}] → {str(out)[:100]}")
                results.append({"type": "tool_result", "tool_use_id": tb["id"], "content": out})
            messages.append({"role": "user", "content": results})
    except urllib.error.HTTPError as e:
        body = e.read().decode("utf-8")
        return jsonify({"ok": False, "msg": f"API error {e.code}: {body[:300]}"})
    except Exception as e:
        return jsonify({"ok": False, "msg": str(e)})

    if not final_text: final_text = "(no text reply)"
    if do_speak: _speak(final_text[:400])
    return jsonify({"ok": True, "reply": final_text, "tool_log": tool_log})

# ── Shard Editor API ──────────────────────────────────────────────────────────
_SHARD_DIRS = [
    Path(r"C:\Users\wlwil\Desktop\Aeonmi Files\Aeonmi-aeonmi01\shard\src"),
    Path(r"C:\Users\wlwil\Desktop\Aeonmi Files\Aeonmi-aeonmi01\aeonmi_ai"),
]
_AEONMI_EXE = Path(r"C:\Users\wlwil\Desktop\Aeonmi Files\Aeonmi-aeonmi01\target\release\aeonmi_project.exe")

@app.route("/api/shard/files")
def api_shard_files():
    result = []
    for d in _SHARD_DIRS:
        if d.exists():
            for f in sorted(d.glob("*.ai")):
                result.append({"dir": d.name, "name": f.name, "path": str(f)})
    return jsonify(result)

@app.route("/api/shard/read")
def api_shard_read():
    path = request.args.get("path", "")
    p = Path(path)
    ok = any(str(p).startswith(str(d)) for d in _SHARD_DIRS)
    if not ok: return jsonify({"ok": False, "msg": "Path not in shard dirs"})
    if not p.exists(): return jsonify({"ok": False, "msg": "Not found"})
    return jsonify({"ok": True, "content": p.read_text(encoding="utf-8", errors="replace")})

@app.route("/api/shard/save", methods=["POST"])
def api_shard_save():
    data = request.get_json()
    p    = Path(data.get("path", ""))
    ok   = any(str(p).startswith(str(d)) for d in _SHARD_DIRS)
    if not ok: return jsonify({"ok": False, "msg": "Path not in shard dirs"})
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(data.get("content", ""), encoding="utf-8")
    return jsonify({"ok": True, "msg": f"Saved {p.name}"})

@app.route("/api/shard/run", methods=["POST"])
def api_shard_run():
    import subprocess as _sp2
    data = request.get_json()
    p    = Path(data.get("path", ""))
    ok   = any(str(p).startswith(str(d)) for d in _SHARD_DIRS)
    if not ok: return jsonify({"ok": False, "msg": "Path not in shard dirs"})
    if not _AEONMI_EXE.exists():
        return jsonify({"ok": False, "msg": f"Runtime not found: {_AEONMI_EXE}"})
    _env2 = {**os.environ, "AEONMI_NATIVE": "1"}
    try:
        _r2 = _sp2.run([str(_AEONMI_EXE), str(p)], capture_output=True, text=True, timeout=60, env=_env2)
        out2 = (_r2.stdout or "") + (_r2.stderr or "")
        return jsonify({"ok": True, "output": out2[:8000], "exit_code": _r2.returncode})
    except _sp2.TimeoutExpired:
        return jsonify({"ok": False, "msg": "Timeout (60s)"})
    except Exception as e2:
        return jsonify({"ok": False, "msg": str(e2)})

@app.route("/api/settings/get")
def api_settings_get():
    config = {}
    if CONFIG_FILE.exists():
        try: config = json.loads(CONFIG_FILE.read_text(encoding="utf-8"))
        except Exception: pass
    masked = {}
    for k, v in config.items():
        if isinstance(v, str) and ("key" in k.lower() or "secret" in k.lower() or "token" in k.lower()):
            masked[k] = ("*" * max(0, len(v) - 4) + v[-4:]) if len(v) > 4 else "****"
        else:
            masked[k] = v
    return jsonify({"ok": True, "config": masked, "path": str(CONFIG_FILE)})

@app.route("/api/settings/save", methods=["POST"])
def api_settings_save():
    updates = (request.json or {}).get("updates", {})
    config = {}
    if CONFIG_FILE.exists():
        try: config = json.loads(CONFIG_FILE.read_text(encoding="utf-8"))
        except Exception: pass
    for k, v in updates.items():
        if v and not all(c == '*' for c in str(v)):
            config[k] = v
    try:
        CONFIG_FILE.parent.mkdir(parents=True, exist_ok=True)
        CONFIG_FILE.write_text(json.dumps(config, indent=2), encoding="utf-8")
        return jsonify({"ok": True, "msg": "Settings saved"})
    except Exception as e:
        return jsonify({"ok": False, "msg": str(e)})

@app.route("/api/github/status")
def api_github_status():
    try:
        r   = _git("status", "--short")
        files = []
        for line in (r["stdout"] or "").splitlines():
            line = line.rstrip()
            if len(line) >= 3:
                files.append({"status": line[:2].strip(), "path": line[3:].strip()})
        br      = _git("rev-parse", "--abbrev-ref", "HEAD")
        log     = _git("log", "--oneline", "-5")
        raw_log = (log["stdout"] or "") if log["ok"] else ""
        commits = []
        for c in raw_log.strip().splitlines():
            if c:
                # encode/decode to drop any chars that can't round-trip
                safe_c = c.encode("utf-8", errors="replace").decode("utf-8", errors="replace")
                commits.append(safe_c)
        return jsonify({"ok": True, "files": files,
                        "branch": (br["stdout"] or "").strip() if br["ok"] else "unknown",
                        "recent_commits": commits})
    except Exception as exc:
        import traceback
        return jsonify({"ok": False, "msg": str(exc), "trace": traceback.format_exc()[-800:]})

@app.route("/api/github/commit", methods=["POST"])
def api_github_commit():
    data    = request.json or {}
    message = data.get("message", "").strip()
    paths   = data.get("paths", [])
    if not message:
        return jsonify({"ok": False, "msg": "Commit message required"})
    if paths:
        for p in paths:
            r = _git("add", p)
            if not r["ok"]: return jsonify({"ok": False, "msg": f"git add failed: {r['stderr']}"})
    else:
        r = _git("add", "-A")
        if not r["ok"]: return jsonify({"ok": False, "msg": f"git add -A failed: {r['stderr']}"})
    r = _git("commit", "-m", message)
    if not r["ok"] and "nothing to commit" not in r["stdout"] + r["stderr"]:
        return jsonify({"ok": False, "msg": r["stderr"] or r["stdout"]})
    return jsonify({"ok": True, "msg": r["stdout"].strip() or "Nothing new to commit"})

@app.route("/api/github/push", methods=["POST"])
def api_github_push():
    r = _git("push")
    return jsonify({"ok": r["ok"], "msg": (r["stdout"] + r["stderr"]).strip() or "Push complete"})

@app.route("/api/github/pull", methods=["POST"])
def api_github_pull():
    r = _git("pull")
    return jsonify({"ok": r["ok"], "msg": (r["stdout"] + r["stderr"]).strip() or "Pull complete"})

@app.route("/api/binary/info")
def api_binary_info():
    if not BIN.exists():
        return jsonify({"ok": False, "msg": "Binary not found"})
    st = BIN.stat()
    return jsonify({"ok": True, "path": str(BIN),
                    "size_kb": round(st.st_size / 1024, 1),
                    "mtime": int(st.st_mtime),
                    "mtime_str": datetime.datetime.fromtimestamp(st.st_mtime).strftime("%Y-%m-%d %H:%M:%S")})


DASHBOARD_HTML = r"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Aeonmi Command Center</title>
<style>
:root{--bg:#07070f;--surface:#0f0f1a;--card:#13131f;--border:#1e1e30;--accent:#7c3aed;--accent2:#06b6d4;--green:#10b981;--yellow:#f59e0b;--red:#ef4444;--text:#e2e8f0;--muted:#64748b;--glow:rgba(124,58,237,0.25);}
*{box-sizing:border-box;margin:0;padding:0;}
html,body{height:100%;font-family:'Segoe UI',system-ui,sans-serif;background:var(--bg);color:var(--text);font-size:14px;}
nav{display:flex;align-items:center;justify-content:space-between;padding:12px 24px;background:var(--surface);border-bottom:1px solid var(--border);position:sticky;top:0;z-index:100;box-shadow:0 2px 20px rgba(0,0,0,0.5);}
.logo{display:flex;align-items:center;gap:10px;}
.logo-symbol{font-size:22px;filter:drop-shadow(0 0 8px var(--accent));}
.logo-text{font-size:18px;font-weight:700;letter-spacing:2px;color:#fff;}
.logo-sub{font-size:10px;color:var(--muted);letter-spacing:3px;text-transform:uppercase;margin-top:1px;}
.nav-mid{display:flex;gap:4px;}
.tab-btn{background:none;border:none;color:var(--muted);padding:8px 16px;cursor:pointer;font-size:13px;font-weight:500;letter-spacing:0.5px;border-radius:6px;transition:all 0.2s;}
.tab-btn:hover{color:var(--text);background:rgba(255,255,255,0.05);}
.tab-btn.active{color:var(--accent2);background:rgba(6,182,212,0.1);border-bottom:2px solid var(--accent2);}
.nav-right{display:flex;align-items:center;gap:12px;}
.regime-badge{padding:4px 14px;border-radius:20px;font-size:11px;font-weight:700;letter-spacing:1.5px;border:1px solid currentColor;transition:all 0.4s;}
.binary-dot{width:8px;height:8px;border-radius:50%;background:var(--green);box-shadow:0 0 8px var(--green);}
.binary-dot.dead{background:var(--red);box-shadow:0 0 8px var(--red);}
#runBtn{background:linear-gradient(135deg,var(--accent),#5b21b6);color:#fff;border:none;padding:8px 20px;border-radius:6px;font-size:13px;font-weight:600;cursor:pointer;letter-spacing:0.5px;transition:all 0.2s;box-shadow:0 0 15px var(--glow);}
#runBtn:hover{transform:translateY(-1px);box-shadow:0 0 25px var(--glow);}
#runBtn:disabled{opacity:0.5;cursor:not-allowed;transform:none;}
#runBtn.running{animation:pulse 1.5s infinite;}
@keyframes pulse{0%,100%{opacity:1}50%{opacity:0.6}}
.voice-toggle{display:flex;align-items:center;gap:6px;font-size:11px;color:var(--muted);cursor:pointer;}
.voice-toggle input{accent-color:var(--accent2);}
.tab-content{display:none;padding:20px 24px;}
.tab-content.active{display:block;}
.card{background:var(--card);border:1px solid var(--border);border-radius:10px;padding:16px;transition:border-color 0.3s;}
.card:hover{border-color:#2a2a42;}
.card-title{font-size:10px;font-weight:600;letter-spacing:2px;color:var(--muted);text-transform:uppercase;margin-bottom:10px;}
.card-title-row{display:flex;align-items:center;justify-content:space-between;margin-bottom:10px;}
.card-title-row .card-title{margin-bottom:0;}
.stats-row{display:grid;grid-template-columns:repeat(6,1fr);gap:12px;}
.mid-row{display:grid;grid-template-columns:1fr 1.6fr;gap:20px;margin-top:20px;}
.bot-row{display:grid;grid-template-columns:320px 1fr;gap:20px;margin-top:20px;}
.stat-card{text-align:center;}
.stat-val{font-size:28px;font-weight:700;line-height:1.1;font-variant-numeric:tabular-nums;transition:color 0.4s;}
.stat-label{font-size:10px;color:var(--muted);margin-top:4px;letter-spacing:1px;}
.agents-grid{display:flex;flex-direction:column;gap:10px;}
.agent-row{display:flex;align-items:center;gap:10px;}
.agent-name{width:72px;font-size:11px;font-weight:600;letter-spacing:1px;color:var(--muted);text-transform:uppercase;}
.agent-bar-bg{flex:1;height:8px;border-radius:4px;background:rgba(255,255,255,0.07);overflow:hidden;}
.agent-bar-fill{height:100%;border-radius:4px;transition:width 0.6s cubic-bezier(0.4,0,0.2,1);}
.agent-val{width:40px;text-align:right;font-size:12px;font-variant-numeric:tabular-nums;font-weight:600;}
.verdict-display{margin-top:14px;padding:12px;border-radius:8px;text-align:center;transition:all 0.4s;border:1px solid rgba(255,255,255,0.06);}
.verdict-label{font-size:11px;color:var(--muted);letter-spacing:2px;text-transform:uppercase;}
.verdict-value{font-size:22px;font-weight:800;letter-spacing:3px;margin-top:4px;}
.entangle-row{display:flex;align-items:center;gap:8px;margin-top:12px;}
.entangle-label{font-size:11px;color:var(--muted);}
.entangle-bar{flex:1;height:4px;border-radius:2px;background:rgba(255,255,255,0.07);overflow:hidden;}
.entangle-fill{height:100%;border-radius:2px;background:linear-gradient(90deg,var(--accent),var(--accent2));transition:width 0.6s;}
.entangle-val{font-size:12px;font-weight:600;color:var(--accent2);}
.log-box{background:#060610;border:1px solid var(--border);border-radius:8px;padding:12px;font-family:'Cascadia Code','Consolas',monospace;font-size:11px;line-height:1.6;color:#94a3b8;height:260px;overflow-y:auto;white-space:pre-wrap;word-break:break-all;}
.log-box::-webkit-scrollbar{width:4px;}.log-box::-webkit-scrollbar-thumb{background:var(--border);border-radius:2px;}
.active-card{background:linear-gradient(135deg,rgba(124,58,237,0.08),rgba(6,182,212,0.04));border:1px solid rgba(124,58,237,0.35);border-radius:16px;padding:24px;margin-bottom:16px;display:none;position:relative;overflow:hidden;}
.active-card.visible{display:block;}
.active-card::before{content:'';position:absolute;inset:0;background:radial-gradient(ellipse at 20% 50%,rgba(124,58,237,0.06) 0%,transparent 60%);pointer-events:none;}
.active-header{display:flex;align-items:center;gap:20px;margin-bottom:16px;}
.pulse-wrap{position:relative;width:52px;height:52px;flex-shrink:0;}
.pulse-ring{position:absolute;inset:-8px;border-radius:50%;border:2px solid rgba(124,58,237,0.5);animation:ringpulse 1.8s ease-out infinite;}
.pulse-ring2{position:absolute;inset:-18px;border-radius:50%;border:1px solid rgba(124,58,237,0.2);animation:ringpulse 1.8s ease-out 0.5s infinite;}
@keyframes ringpulse{0%{transform:scale(0.85);opacity:1}100%{transform:scale(1.45);opacity:0}}
.pulse-core{width:52px;height:52px;border-radius:50%;background:linear-gradient(135deg,var(--accent),#5b21b6);display:flex;align-items:center;justify-content:center;font-size:22px;animation:corepulse 1.8s ease-in-out infinite;}
@keyframes corepulse{0%,100%{box-shadow:0 0 24px rgba(124,58,237,0.5)}50%{box-shadow:0 0 48px rgba(124,58,237,0.9),0 0 80px rgba(124,58,237,0.3)}}
.active-meta{flex:1;}
.active-title{font-size:10px;font-weight:700;letter-spacing:3px;color:var(--accent);text-transform:uppercase;margin-bottom:4px;}
.active-phase-name{font-size:16px;font-weight:700;color:var(--text);}
.active-right{text-align:right;min-width:90px;}
.active-ent-val{font-size:30px;font-weight:800;color:var(--cyan);font-variant-numeric:tabular-nums;line-height:1;}
.active-ent-label{font-size:9px;letter-spacing:2px;color:var(--muted);text-transform:uppercase;margin-top:2px;}
.phase-track{display:flex;gap:5px;margin-bottom:14px;flex-wrap:wrap;}
.phase-pip{display:flex;align-items:center;gap:4px;padding:3px 9px;border-radius:20px;border:1px solid var(--border);background:var(--surface);font-size:10px;font-weight:600;color:var(--muted);transition:all 0.3s;white-space:nowrap;}
.phase-pip.pip-active{border-color:var(--accent);background:rgba(124,58,237,0.15);color:var(--accent);}
.phase-pip.pip-done{border-color:var(--green);background:rgba(16,185,129,0.1);color:var(--green);}
.pip-dot{width:5px;height:5px;border-radius:50%;background:currentColor;display:inline-block;margin-right:2px;}
.active-stream{background:#060610;border:1px solid var(--border);border-radius:8px;padding:10px 14px;font-family:'Cascadia Code','Consolas',monospace;font-size:11px;height:80px;overflow-y:auto;white-space:pre-wrap;word-break:break-all;}
.active-stream::-webkit-scrollbar{width:3px;}.active-stream::-webkit-scrollbar-thumb{background:var(--border);}
.sline{display:block;color:#4b5563;transition:color 0.2s;}
.sline.sline-new{color:#94a3b8;animation:slfade 0.3s ease;}
.sline.sline-phase{color:var(--accent);font-weight:600;}
.sline.sline-ok{color:var(--green);}
@keyframes slfade{from{opacity:0;transform:translateX(-4px)}to{opacity:1;transform:translateX(0)}}
.verdict-reveal{margin-top:16px;padding:16px 24px;border-radius:12px;text-align:center;animation:vreveal 0.5s cubic-bezier(0.175,0.885,0.32,1.275);}
@keyframes vreveal{from{opacity:0;transform:scale(0.75)}to{opacity:1;transform:scale(1)}}
.vr-word{font-size:26px;font-weight:900;letter-spacing:4px;}
.vr-sub{font-size:11px;color:var(--muted);margin-top:6px;letter-spacing:1px;}
.log-ctrl{display:flex;align-items:center;gap:8px;margin-bottom:8px;}
.log-ctrl span{font-size:10px;color:var(--muted);}
.file-tree{font-size:12px;font-family:'Cascadia Code','Consolas',monospace;height:260px;overflow-y:auto;color:var(--text);}
.file-tree::-webkit-scrollbar{width:4px;}.file-tree::-webkit-scrollbar-thumb{background:var(--border);border-radius:2px;}
.ft-dir{color:var(--accent2);cursor:pointer;user-select:none;}
.ft-file{color:#94a3b8;}.ft-file.ai{color:#a78bfa;}.ft-file.rs{color:#fb923c;}.ft-file.py{color:#4ade80;}.ft-file.json{color:#fbbf24;}.ft-file.toml{color:#f472b6;}
.ft-kids{padding-left:16px;}
.binary-bar{display:flex;align-items:center;gap:16px;padding:10px 16px;background:var(--surface);border:1px solid var(--border);border-radius:8px;margin-top:20px;font-size:12px;}
.binary-bar-label{color:var(--muted);font-size:10px;letter-spacing:1px;text-transform:uppercase;}
.binary-bar-val{font-weight:600;color:var(--text);}
.binary-bar-sep{width:1px;height:20px;background:var(--border);}
.mother-layout{display:grid;grid-template-columns:1fr 1fr;gap:20px;}
.journal-box{background:#060610;border:1px solid var(--border);border-radius:8px;padding:14px;font-family:'Cascadia Code','Consolas',monospace;font-size:12px;line-height:1.7;color:#94a3b8;height:420px;overflow-y:auto;white-space:pre-wrap;}
.journal-box::-webkit-scrollbar{width:4px;}.journal-box::-webkit-scrollbar-thumb{background:var(--border);border-radius:2px;}
.events-list{height:420px;overflow-y:auto;}
.events-list::-webkit-scrollbar{width:4px;}.events-list::-webkit-scrollbar-thumb{background:var(--border);border-radius:2px;}
.event-item{padding:8px 10px;border-bottom:1px solid rgba(30,30,48,0.8);font-size:12px;}
.event-type{font-size:10px;font-weight:700;letter-spacing:1px;text-transform:uppercase;color:var(--accent2);}
.event-data{color:var(--muted);margin-top:2px;font-family:monospace;font-size:11px;}
.event-ts{font-size:10px;color:#3d3d5c;margin-top:2px;}
.voice-panel{margin-top:20px;}
.voice-input-row{display:flex;gap:8px;margin-top:10px;}
.voice-input{flex:1;background:var(--surface);border:1px solid var(--border);border-radius:6px;padding:8px 12px;color:var(--text);font-size:13px;outline:none;}
.voice-input:focus{border-color:var(--accent2);}
.btn-sm{padding:7px 14px;border:none;border-radius:6px;font-size:12px;font-weight:600;cursor:pointer;transition:all 0.2s;}
.btn-cyan{background:rgba(6,182,212,0.15);color:var(--accent2);border:1px solid rgba(6,182,212,0.3);}
.btn-cyan:hover{background:rgba(6,182,212,0.25);}
.btn-purple{background:linear-gradient(135deg,var(--accent),#5b21b6);color:#fff;box-shadow:0 0 12px var(--glow);}
.btn-purple:hover{transform:translateY(-1px);}
.btn-green{background:rgba(16,185,129,0.15);color:var(--green);border:1px solid rgba(16,185,129,0.3);}
.btn-green:hover{background:rgba(16,185,129,0.25);}
.btn-red{background:rgba(239,68,68,0.15);color:var(--red);border:1px solid rgba(239,68,68,0.3);}
.settings-layout{max-width:640px;}
.settings-group{margin-bottom:24px;}
.settings-group-title{font-size:11px;font-weight:700;letter-spacing:2px;color:var(--accent);text-transform:uppercase;margin-bottom:12px;padding-bottom:6px;border-bottom:1px solid var(--border);}
.field-row{display:flex;align-items:center;gap:12px;margin-bottom:10px;}
.field-label{width:180px;font-size:12px;color:var(--muted);}
.field-input{flex:1;background:var(--surface);border:1px solid var(--border);border-radius:6px;padding:8px 12px;color:var(--text);font-size:13px;font-family:'Cascadia Code','Consolas',monospace;outline:none;}
.field-input:focus{border-color:var(--accent2);}
.field-eye{background:none;border:none;color:var(--muted);cursor:pointer;padding:0 6px;font-size:14px;}
.settings-save-row{margin-top:16px;display:flex;align-items:center;gap:12px;}
.settings-msg{font-size:12px;color:var(--green);}
.settings-info{font-size:11px;color:var(--muted);margin-top:6px;}
.github-layout{display:grid;grid-template-columns:1fr 1fr;gap:20px;}
.git-file-list{height:300px;overflow-y:auto;font-family:'Cascadia Code','Consolas',monospace;font-size:12px;}
.git-file-list::-webkit-scrollbar{width:4px;}.git-file-list::-webkit-scrollbar-thumb{background:var(--border);}
.git-file-item{display:flex;align-items:center;gap:8px;padding:5px 8px;border-radius:4px;cursor:pointer;}
.git-file-item:hover{background:rgba(255,255,255,0.03);}
.git-file-item input{accent-color:var(--accent2);}
.git-status-badge{font-size:10px;font-weight:700;width:20px;text-align:center;}
.git-status-M{color:var(--yellow);}.git-status-A{color:var(--green);}.git-status-D{color:var(--red);}.git-status-Q{color:var(--muted);}
.git-file-path{color:var(--text);flex:1;}
.commit-panel{display:flex;flex-direction:column;gap:10px;}
.commit-input{width:100%;background:var(--surface);border:1px solid var(--border);border-radius:6px;padding:10px 12px;color:var(--text);font-size:13px;outline:none;resize:vertical;min-height:80px;}
.commit-input:focus{border-color:var(--accent2);}
.git-btn-row{display:flex;gap:8px;flex-wrap:wrap;}
.git-log-list{margin-top:8px;font-family:'Cascadia Code','Consolas',monospace;font-size:11px;color:var(--muted);}
.git-log-item{padding:3px 0;border-bottom:1px solid rgba(30,30,48,0.6);}
.git-output{background:#060610;border:1px solid var(--border);border-radius:6px;padding:10px;font-family:monospace;font-size:12px;color:#94a3b8;min-height:50px;white-space:pre-wrap;margin-top:6px;max-height:120px;overflow-y:auto;}
.branch-badge{display:inline-block;background:rgba(124,58,237,0.15);border:1px solid rgba(124,58,237,0.3);color:#a78bfa;padding:2px 10px;border-radius:12px;font-size:11px;font-weight:600;}
.refresh-btn{background:none;border:none;color:var(--muted);cursor:pointer;font-size:12px;padding:4px 8px;border-radius:4px;}
.refresh-btn:hover{color:var(--text);background:rgba(255,255,255,0.05);}
.select-all-btn{background:none;border:none;color:var(--accent2);cursor:pointer;font-size:11px;padding:2px 6px;}
</style>
</head>
<body>
<nav>
  <div class="logo">
    <span class="logo-symbol">&#x2297;</span>
    <div><div class="logo-text">AEONMI</div><div class="logo-sub">Command Center v2</div></div>
  </div>
  <div class="nav-mid">
    <button class="tab-btn active" onclick="switchTab('command',this)">&#9889; Command</button>
    <button class="tab-btn" onclick="switchTab('mother',this)">&#129504; Mother</button>
    <button class="tab-btn" onclick="switchTab('settings',this)">&#9881; Settings</button>
    <button class="tab-btn" onclick="switchTab('github',this)">&#128279; GitHub</button>
    <button class="tab-btn" onclick="switchTab('shard',this)">&#128296; Shard</button>
  </div>
  <div class="nav-right">
    <label class="voice-toggle" title="Speak verdict after each run">
      <input type="checkbox" id="voiceChk"> &#128266; Voice
    </label>
    <span class="regime-badge" id="regimeBadge" style="color:#f59e0b;">BALANCED</span>
    <span class="binary-dot" id="binDot"></span>
    <button id="runBtn" onclick="runPipeline()">&#9654; RUN PIPELINE</button>
  </div>
</nav>

<!-- COMMAND TAB -->
<div class="tab-content active" id="tab-command">
  <div class="stats-row">
    <div class="card stat-card"><div class="stat-val" id="statSession">&#8212;</div><div class="stat-label">SESSION</div></div>
    <div class="card stat-card"><div class="stat-val" id="statAccuracy" style="color:var(--yellow);">&#8212;%</div><div class="stat-label">ACCURACY</div></div>
    <div class="card stat-card"><div class="stat-val" id="statConf">&#8212;</div><div class="stat-label">CONFIDENCE</div></div>
    <div class="card stat-card"><div class="stat-val" id="statThreshold">&#8212;</div><div class="stat-label">THRESHOLD</div></div>
    <div class="card stat-card"><div class="stat-val" style="color:var(--green);" id="statProceed">&#8212;</div><div class="stat-label">PROCEED &#10003;</div></div>
    <div class="card stat-card"><div class="stat-val" style="color:var(--red);" id="statAbort">&#8212;</div><div class="stat-label">ABORT &#10007;</div></div>
  </div>
  <!-- ACTIVE SESSION PANEL -->
  <div class="active-card" id="activeCard">
    <div class="active-header">
      <div class="pulse-wrap">
        <div class="pulse-ring"></div><div class="pulse-ring2"></div>
        <div class="pulse-core">&#x2297;</div>
      </div>
      <div class="active-meta">
        <div class="active-title">&#9670; Mother is running</div>
        <div class="active-phase-name" id="activePhaseName">Initializing&hellip;</div>
      </div>
      <div class="active-right">
        <div class="active-ent-val" id="activeEnt">&#8212;%</div>
        <div class="active-ent-label">Entanglement</div>
      </div>
    </div>
    <div class="phase-track" id="phaseTrack">
      <div class="phase-pip" id="pp1"><span class="pip-dot"></span>Files</div>
      <div class="phase-pip" id="pp2"><span class="pip-dot"></span>Upload</div>
      <div class="phase-pip" id="pp3"><span class="pip-dot"></span>Quantum</div>
      <div class="phase-pip" id="pp4"><span class="pip-dot"></span>Qiskit</div>
      <div class="phase-pip" id="pp5"><span class="pip-dot"></span>Session</div>
      <div class="phase-pip" id="pp6"><span class="pip-dot"></span>Core</div>
      <div class="phase-pip" id="pp7"><span class="pip-dot"></span>Memory</div>
    </div>
    <div class="active-stream" id="activeStream"></div>
    <div id="verdictReveal" style="display:none;"></div>
  </div>

  <div class="mid-row">
    <div class="card">
      <div class="card-title">Agent Scores</div>
      <div class="agents-grid">
        <div class="agent-row"><span class="agent-name">Oracle</span><div class="agent-bar-bg"><div class="agent-bar-fill" id="bar-oracle" style="width:0%;background:#a78bfa;"></div></div><span class="agent-val" id="val-oracle">&#8212;</span></div>
        <div class="agent-row"><span class="agent-name">Hype</span><div class="agent-bar-bg"><div class="agent-bar-fill" id="bar-hype" style="width:0%;background:#fb923c;"></div></div><span class="agent-val" id="val-hype">&#8212;</span></div>
        <div class="agent-row"><span class="agent-name">Close</span><div class="agent-bar-bg"><div class="agent-bar-fill" id="bar-close" style="width:0%;background:#4ade80;"></div></div><span class="agent-val" id="val-close">&#8212;</span></div>
        <div class="agent-row"><span class="agent-name">Risk</span><div class="agent-bar-bg"><div class="agent-bar-fill" id="bar-risk" style="width:0%;background:#f87171;"></div></div><span class="agent-val" id="val-risk">&#8212;</span></div>
        <div class="agent-row"><span class="agent-name">Conductor</span><div class="agent-bar-bg"><div class="agent-bar-fill" id="bar-conductor" style="width:0%;background:#06b6d4;"></div></div><span class="agent-val" id="val-conductor">&#8212;</span></div>
      </div>
      <div class="entangle-row">
        <span class="entangle-label">Entanglement</span>
        <div class="entangle-bar"><div class="entangle-fill" id="entangleFill" style="width:0%"></div></div>
        <span class="entangle-val" id="entangleVal">&#8212;%</span>
      </div>
      <div class="verdict-display" id="verdictDisplay">
        <div class="verdict-label">Last Verdict</div>
        <div class="verdict-value" id="verdictVal">&#8212;</div>
      </div>
    </div>
    <div class="card">
      <div class="log-ctrl">
        <div class="card-title" style="margin:0">Pipeline Log</div>
        <span id="logLines">0 lines</span>
        <button class="refresh-btn" onclick="fetchLog()">&#8635; refresh</button>
        <label style="margin-left:auto;font-size:11px;color:var(--muted);display:flex;align-items:center;gap:4px;"><input type="checkbox" id="autoScroll" checked> auto-scroll</label>
      </div>
      <pre class="log-box" id="logBox">Waiting for pipeline output...</pre>
    </div>
  </div>
  <div class="bot-row">
    <div class="card">
      <div class="card-title-row">
        <div class="card-title">File Explorer</div>
        <button class="refresh-btn" onclick="fetchFiles()">&#8635;</button>
      </div>
      <div class="file-tree" id="fileTree">Loading&#8230;</div>
    </div>
    <div>
      <div class="binary-bar" id="binaryBar">
        <div><div class="binary-bar-label">Binary</div><div class="binary-bar-val">aeonmi_project.exe</div></div>
        <div class="binary-bar-sep"></div>
        <div><div class="binary-bar-label">Size</div><div class="binary-bar-val" id="binSize">&#8212;</div></div>
        <div class="binary-bar-sep"></div>
        <div><div class="binary-bar-label">Built</div><div class="binary-bar-val" id="binMtime">&#8212;</div></div>
        <div class="binary-bar-sep"></div>
        <div><div class="binary-bar-label">Status</div><div class="binary-bar-val" id="binStatus">checking&#8230;</div></div>
      </div>
    </div>
  </div>
</div>

<!-- MOTHER TAB -->
<div class="tab-content" id="tab-mother">
  <div class="mother-layout">
    <div class="card">
      <div class="card-title-row">
        <div class="card-title">Mother Journal</div>
        <button class="refresh-btn" onclick="fetchMotherJournal()">&#8635; refresh</button>
      </div>
      <pre class="journal-box" id="journalBox">Loading journal&#8230;</pre>
    </div>
    <div class="card">
      <div class="card-title-row">
        <div class="card-title">Session Events</div>
        <span id="eventCount" style="font-size:11px;color:var(--muted);">0 events</span>
      </div>
      <div class="events-list" id="eventsList">Loading events&#8230;</div>
    </div>
  </div>
  <div class="card" style="margin-top:16px;">
    <div class="card-title-row">
      <div class="card-title">&#9670; Speak to Mother</div>
      <label style="font-size:11px;color:var(--muted);display:flex;align-items:center;gap:5px;cursor:pointer;">
        <input type="checkbox" id="chatSpeakChk" style="accent-color:var(--accent2);"> speak replies
      </label>
    </div>
    <div id="chatHistory" style="
      background:#060610;border:1px solid var(--border);border-radius:8px;
      padding:14px;font-size:12px;line-height:1.7;min-height:160px;max-height:340px;
      overflow-y:auto;margin-bottom:10px;font-family:'Cascadia Code',monospace;
      color:var(--text);white-space:pre-wrap;word-break:break-word;
    ">Waiting for your first message&#8230;</div>
    <div style="display:flex;gap:8px;align-items:flex-end;">
      <textarea id="chatInput" rows="2" placeholder="Ask Mother anything&#8230;" style="
        flex:1;background:var(--surface);border:1px solid var(--border);border-radius:6px;
        padding:8px 12px;color:var(--text);font-size:13px;font-family:inherit;resize:vertical;
        outline:none;transition:border .2s;
      " onkeydown="if(event.key==='Enter'&&!event.shiftKey){event.preventDefault();sendToMother();}"></textarea>
      <button id="micBtn" onclick="toggleMic()" title="Hold to speak" style="
        height:36px;width:36px;border-radius:50%;border:2px solid #475569;
        background:#0f1117;color:#94a3b8;font-size:16px;cursor:pointer;
        display:flex;align-items:center;justify-content:center;
        transition:all .2s;flex-shrink:0;">
        &#127908;
      </button>
      <button class="btn-sm btn-purple" onclick="sendToMother()" style="height:36px;white-space:nowrap;">
        &#8679; Send
      </button>
    </div>
    <p style="font-size:10px;color:var(--muted);margin-top:6px;">
      Enter to send &nbsp;&#183;&nbsp; Shift+Enter for newline &nbsp;&#183;&nbsp;
      <span id="micStatus">&#127908; Mic available in Chrome</span> &nbsp;&#183;&nbsp; API key required
    </p>
  </div>
  <div class="card" style="margin-top:12px;">
    <div class="card-title">Voice Output</div>
    <div class="voice-input-row">
      <input type="text" class="voice-input" id="voiceText" value="Aeonmi systems online. All agents nominal." style="font-size:12px;">
      <button class="btn-sm btn-cyan" onclick="speakCustom()">&#9654; Speak</button>
      <button class="btn-sm btn-purple" onclick="speakLastVerdict()">&#9654; Last Verdict</button>
    </div>
    <p style="font-size:10px;color:var(--muted);margin-top:6px;">Windows SAPI — no install needed. Enable Voice in nav bar for auto-announcements.</p>
  </div>
</div>

<!-- SETTINGS TAB -->
<div class="tab-content" id="tab-settings">
  <div class="card settings-layout">
    <div class="settings-group">
      <div class="settings-group-title">API Keys</div>
      <div class="field-row">
        <span class="field-label">Anthropic API Key</span>
        <input type="password" class="field-input" id="cfg-anthropic_api_key" placeholder="sk-ant-&#8230;">
        <button class="field-eye" onclick="togglePw('cfg-anthropic_api_key',this)">&#128065;</button>
      </div>
      <div class="field-row">
        <span class="field-label">OpenAI API Key</span>
        <input type="password" class="field-input" id="cfg-openai_api_key" placeholder="sk-&#8230;">
        <button class="field-eye" onclick="togglePw('cfg-openai_api_key',this)">&#128065;</button>
      </div>
      <div class="field-row">
        <span class="field-label">IBM Quantum Token</span>
        <input type="password" class="field-input" id="cfg-ibm_token" placeholder="IBM Quantum token">
        <button class="field-eye" onclick="togglePw('cfg-ibm_token',this)">&#128065;</button>
      </div>
    </div>
    <div class="settings-group">
      <div class="settings-group-title">Pipeline Config</div>
      <div class="field-row">
        <span class="field-label">Model</span>
        <input type="text" class="field-input" id="cfg-model" placeholder="claude-opus-4-6">
      </div>
      <div class="field-row">
        <span class="field-label">Max Tokens</span>
        <input type="text" class="field-input" id="cfg-max_tokens" placeholder="4096">
      </div>
      <div class="field-row">
        <span class="field-label">Session Threshold</span>
        <input type="text" class="field-input" id="cfg-threshold" placeholder="50">
      </div>
    </div>
    <div class="settings-save-row">
      <button class="btn-sm btn-purple" onclick="saveSettings()">&#128190; Save Settings</button>
      <button class="btn-sm btn-cyan" onclick="loadSettings()">&#8635; Reload</button>
      <span class="settings-msg" id="settingsMsg"></span>
    </div>
    <div class="settings-info" id="settingsPath">Config: loading&#8230;</div>
  </div>
</div>

<!-- GITHUB TAB -->
<div class="tab-content" id="tab-github">
  <div class="github-layout">
    <div class="card">
      <div class="card-title-row">
        <div class="card-title">Changed Files</div>
        <div style="display:flex;gap:6px;align-items:center;">
          <span id="gitBranch" class="branch-badge">&#8230;</span>
          <button class="refresh-btn" onclick="fetchGitStatus()">&#8635;</button>
          <button class="select-all-btn" onclick="toggleSelectAll()">select all</button>
        </div>
      </div>
      <div class="git-file-list" id="gitFileList">Loading&#8230;</div>
      <div class="git-log-list" id="gitLogList"></div>
    </div>
    <div class="card commit-panel">
      <div class="card-title">Commit &amp; Push</div>
      <textarea class="commit-input" id="commitMsg" placeholder="Enter commit message&#8230;"></textarea>
      <div class="git-btn-row">
        <button class="btn-sm btn-green" onclick="doCommit()">&#10003; Commit Selected</button>
        <button class="btn-sm btn-purple" onclick="doPush()">&#8593; Push</button>
        <button class="btn-sm btn-cyan" onclick="doPull()">&#8595; Pull</button>
        <button class="btn-sm btn-red" onclick="doCommitAll()">&#10003; Commit All</button>
      </div>
      <div class="git-output" id="gitOutput">Ready.</div>
    </div>
  </div>
</div>

<!-- SHARD EDITOR TAB -->
<div class="tab-content" id="tab-shard">
  <div style="display:flex;height:calc(100vh - 100px);gap:0;overflow:hidden;">

    <!-- File Tree -->
    <div id="shardTree" style="width:200px;min-width:160px;background:#0f1117;border-right:1px solid #1e293b;overflow-y:auto;padding:8px 0;">
      <div style="padding:6px 12px;font-size:11px;color:#64748b;letter-spacing:.08em;font-weight:600;">SHARD FILES</div>
      <div id="shardFileList" style="font-size:12px;"></div>
    </div>

    <!-- Editor + Output -->
    <div style="flex:1;display:flex;flex-direction:column;overflow:hidden;">

      <!-- Toolbar -->
      <div style="display:flex;align-items:center;gap:8px;padding:6px 12px;background:#0f1117;border-bottom:1px solid #1e293b;">
        <span id="shardCurrentFile" style="font-size:12px;color:#94a3b8;flex:1;">No file open</span>
        <button onclick="shardNew()" style="background:#1e293b;color:#94a3b8;border:none;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">+ New</button>
        <button onclick="shardSave()" style="background:#1e3a5f;color:#60a5fa;border:none;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">&#128190; Save</button>
        <button onclick="shardRun()" style="background:#14532d;color:#4ade80;border:none;padding:4px 10px;border-radius:4px;cursor:pointer;font-size:12px;">&#9654; Run</button>
      </div>

      <!-- Monaco -->
      <div id="shardMonaco" style="flex:1;overflow:hidden;"></div>

      <!-- Output -->
      <div style="height:200px;background:#020409;border-top:1px solid #1e293b;display:flex;flex-direction:column;">
        <div style="display:flex;align-items:center;padding:4px 12px;border-bottom:1px solid #1e293b;">
          <span style="font-size:11px;color:#64748b;font-weight:600;flex:1;">OUTPUT</span>
          <button onclick="document.getElementById('shardOutput').textContent=''" style="background:none;border:none;color:#475569;cursor:pointer;font-size:11px;">clear</button>
        </div>
        <pre id="shardOutput" style="flex:1;overflow-y:auto;padding:8px 12px;margin:0;font-size:12px;color:#94a3b8;font-family:monospace;white-space:pre-wrap;"></pre>
      </div>

    </div>
  </div>
</div>


<script>
let lastStatus = {};
let fileTreeOpen = {};
let gitFiles = [];

function switchTab(name, btn) {
  document.querySelectorAll('.tab-content').forEach(t => t.classList.remove('active'));
  document.querySelectorAll('.tab-btn').forEach(b => b.classList.remove('active'));
  document.getElementById('tab-' + name).classList.add('active');
  btn.classList.add('active');
  if (name === 'mother')   fetchMotherJournal();
  if (name === 'settings') loadSettings();
  if (name === 'github')   fetchGitStatus();
  if (name === 'shard')    initShard();
}

// ── Shard Editor ─────────────────────────────────────────────────────────────
let _shardEditor = null, _shardFile = null, _shardInited = false;

function initShard() {
  shardLoadFiles();
  if (_shardInited) return;
  _shardInited = true;
  const script = document.createElement('script');
  script.src = 'https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.44.0/min/vs/loader.min.js';
  script.onload = () => {
    require.config({ paths: { vs: 'https://cdnjs.cloudflare.com/ajax/libs/monaco-editor/0.44.0/min/vs' }});
    require(['vs/editor/editor.main'], () => {
      monaco.editor.defineTheme('aeonmi-dark', {
        base: 'vs-dark', inherit: true,
        rules: [
          {token:'keyword',  foreground:'c084fc', fontStyle:'bold'},
          {token:'string',   foreground:'4ade80'},
          {token:'comment',  foreground:'475569', fontStyle:'italic'},
          {token:'number',   foreground:'fb923c'},
          {token:'operator', foreground:'22d3ee'},
        ],
        colors: {'editor.background':'#020409','editor.foreground':'#e2e8f0',
                 'editorLineNumber.foreground':'#334155','editor.selectionBackground':'#1e3a5f',
                 'editor.lineHighlightBackground':'#0f1117'}
      });
      monaco.languages.register({id:'aeonmi'});
      monaco.languages.setMonarchTokensProvider('aeonmi', {
        keywords: ['fn','let','if','else','while','return','import','from','true','false','null',
                   'emit','use','pub','mod','self','super','as','in','for','match','struct','enum'],
        tokenizer: {
          root: [
            [/⟨[^⟩]*⟩/, 'type'], [/[↦⊗⧉⊕⊗∀∃λ]/, 'operator'],
            [/\/\/.*$/, 'comment'], [/"[^"]*"/, 'string'],
            [/\d+(\.\d+)?/, 'number'],
            [/(fn|let|if|else|while|return|import|emit|use|pub|for|match)/, 'keyword'],
          ]
        }
      });
      _shardEditor = monaco.editor.create(document.getElementById('shardMonaco'), {
        language: 'aeonmi', theme: 'aeonmi-dark', automaticLayout: true,
        fontSize: 13, fontFamily: "'Cascadia Code','Fira Code',monospace",
        lineNumbers: 'on', minimap: {enabled: false},
        scrollBeyondLastLine: false, wordWrap: 'on',
        value: '// Select a file from the tree to begin editing\n'
      });
      _shardEditor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS, shardSave);
      _shardEditor.addCommand(monaco.KeyCode.F5, shardRun);
    });
  };
  document.head.appendChild(script);
}

function shardLoadFiles() {
  fetch('/api/shard/files').then(r=>r.json()).then(files => {
    const list = document.getElementById('shardFileList');
    if (!files.length) { list.innerHTML='<div style="padding:6px 12px;color:#475569">no .ai files found</div>'; return; }
    let html = '', curDir = '';
    files.forEach(f => {
      if (f.dir !== curDir) {
        curDir = f.dir;
        html += `<div style="padding:5px 12px 2px;font-size:10px;color:#475569;text-transform:uppercase;letter-spacing:.1em">${f.dir}</div>`;
      }
      const active = _shardFile === f.path ? 'background:#1e293b;color:#e2e8f0' : 'color:#94a3b8';
      html += `<div onclick="shardOpen(${JSON.stringify(f.path)})" style="padding:4px 12px 4px 20px;cursor:pointer;${active};font-size:12px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis" title="${f.path}">${f.name}</div>`;
    });
    list.innerHTML = html;
  });
}

function shardOpen(path) {
  fetch('/api/shard/read?path=' + encodeURIComponent(path))
    .then(r=>r.json()).then(d => {
      if (!d.ok) { shardLog('[ERROR] ' + d.msg); return; }
      if (_shardEditor) _shardEditor.setValue(d.content);
      _shardFile = path;
      document.getElementById('shardCurrentFile').textContent = path.split('\\').pop();
      shardLoadFiles();
    });
}

function shardSave() {
  if (!_shardFile || !_shardEditor) { shardLog('[WARN] No file open'); return; }
  fetch('/api/shard/save', {method:'POST',headers:{'Content-Type':'application/json'},
    body: JSON.stringify({path: _shardFile, content: _shardEditor.getValue()})})
    .then(r=>r.json()).then(d => shardLog(d.ok ? '[SAVED] ' + d.msg : '[ERROR] ' + d.msg));
}

function shardRun() {
  if (!_shardFile) { shardLog('[WARN] No file open'); return; }
  shardLog('[RUN] Compiling ' + _shardFile.split('\\').pop() + '...');
  fetch('/api/shard/run', {method:'POST',headers:{'Content-Type':'application/json'},
    body: JSON.stringify({path: _shardFile})})
    .then(r=>r.json()).then(d => {
      if (d.ok) shardLog('[EXIT ' + d.exit_code + ']\n' + (d.output || '(no output)'));
      else shardLog('[ERROR] ' + d.msg);
    });
}

function shardNew() {
  const name = prompt('New .ai file name (e.g. test.ai):');
  if (!name || !name.endsWith('.ai')) return;
  const base = 'C:\\Users\\wlwil\\Desktop\\Aeonmi Files\\Aeonmi-aeonmi01\\aeonmi_ai\\' + name;
  fetch('/api/shard/save', {method:'POST',headers:{'Content-Type':'application/json'},
    body: JSON.stringify({path: base, content: '// ' + name + '\n'})})
    .then(r=>r.json()).then(d => { if (d.ok) { shardLoadFiles(); shardOpen(base); } });
}

function shardLog(msg) {
  const el = document.getElementById('shardOutput');
  el.textContent += msg + '\n';
  el.scrollTop = el.scrollHeight;
}

// ── Live Session Engine ────────────────────────────────────────────────────
const PHASE_MAP = {
  '1/7': {id:'pp1', name:'File Explorer'},
  '2/7': {id:'pp2', name:'Upload Intake'},
  '3/7': {id:'pp3', name:'Quantum Hive'},
  '4/7': {id:'pp4', name:'Qiskit Simulation'},
  '5/7': {id:'pp5', name:'Mother Session'},
  '6/7': {id:'pp6', name:'Core Execution'},
  '7/7': {id:'pp7', name:'Memory Update'},
};
const VERDICT_META = {
  PROCEED:     {color:'#10b981', bg:'rgba(16,185,129,0.1)',  border:'rgba(16,185,129,0.3)',  sub:'Conditions nominal.'},
  ACCELERATE:  {color:'#06b6d4', bg:'rgba(6,182,212,0.1)',   border:'rgba(6,182,212,0.3)',   sub:'Strong signal detected.'},
  HOLD:        {color:'#f59e0b', bg:'rgba(245,158,11,0.1)',  border:'rgba(245,158,11,0.3)',  sub:'Awaiting confirmation.'},
  ABORT:       {color:'#ef4444', bg:'rgba(239,68,68,0.1)',   border:'rgba(239,68,68,0.3)',   sub:'Do not proceed.'},
};

let _liveTimer   = null;
let _liveLogLen  = 0;
let _livePhase   = 0;
let _liveEnt     = null;
let _liveVerdict = null;

function startLiveMode() {
  const card = document.getElementById('activeCard');
  card.classList.add('visible');
  document.getElementById('activeStream').textContent = '';
  document.getElementById('verdictReveal').style.display = 'none';
  document.getElementById('activePhaseName').textContent = 'Initializing\u2026';
  document.getElementById('activeEnt').textContent = '\u2014%';
  ['pp1','pp2','pp3','pp4','pp5','pp6','pp7'].forEach(id => {
    const el = document.getElementById(id);
    el.classList.remove('pip-active','pip-done');
  });
  _liveLogLen  = 0;
  _livePhase   = 0;
  _liveEnt     = null;
  _liveVerdict = null;
  _liveTimer   = setInterval(_liveTick, 400);
}

function stopLiveMode(verdict) {
  clearInterval(_liveTimer);
  _liveTimer = null;
  // Mark all active phases as done
  ['pp1','pp2','pp3','pp4','pp5','pp6','pp7'].forEach(id => {
    const el = document.getElementById(id);
    if (el.classList.contains('pip-active')) {
      el.classList.remove('pip-active');
      el.classList.add('pip-done');
    }
  });
  // Verdict reveal
  if (verdict) {
    const m = VERDICT_META[verdict] || VERDICT_META['HOLD'];
    const rv = document.getElementById('verdictReveal');
    rv.innerHTML = '<div class="verdict-reveal" style="background:'+m.bg+';border:1px solid '+m.border+';">'
      + '<div class="vr-word" style="color:'+m.color+';">' + verdict + '</div>'
      + '<div class="vr-sub">' + m.sub + '</div></div>';
    rv.style.display = 'block';
  }
  // Fade card out after 12 seconds
  setTimeout(() => {
    document.getElementById('activeCard').classList.remove('visible');
  }, 12000);
}

function _liveTick() {
  fetch('/api/log').then(r => r.json()).then(d => {
    const full  = (d.log || '').split('\n');
    const newLines = full.slice(_liveLogLen);
    if (newLines.length > 0) {
      _liveLogLen = full.length;
      const stream = document.getElementById('activeStream');
      newLines.forEach(line => {
        if (!line.trim()) return;
        const span = document.createElement('span');
        span.className = 'sline sline-new';
        // Phase detection
        const pm = line.match(/\[(\d\/7)\]/);
        if (pm) {
          const pinfo = PHASE_MAP[pm[1]];
          if (pinfo) {
            const pnum = parseInt(pm[1]);
            // Mark previous done, current active
            for (let i = 1; i < pnum; i++) {
              const el = document.getElementById('pp'+i);
              if (!el.classList.contains('pip-done')) {
                el.classList.remove('pip-active');
                el.classList.add('pip-done');
              }
            }
            const cur = document.getElementById('pp'+pnum);
            cur.classList.remove('pip-done');
            cur.classList.add('pip-active');
            document.getElementById('activePhaseName').textContent = pinfo.name;
            span.classList.add('sline-phase');
            _livePhase = pnum;
          }
        }
        // Entanglement detection
        const em = line.match(/[Ee]ntanglement[:\s]+([0-9]+(?:\.[0-9]+)?)\s*%/);
        if (em) {
          const ev = parseFloat(em[1]);
          _liveEnt = ev;
          document.getElementById('activeEnt').textContent = ev.toFixed(1) + '%';
        }
        // Verdict detection
        const vm = line.match(/VERDICT[:\s]+(PROCEED|ACCELERATE|HOLD|ABORT)/);
        if (vm) { _liveVerdict = vm[1]; span.classList.add('sline-ok'); }
        span.textContent = line;
        stream.appendChild(span);
        stream.appendChild(document.createTextNode('\n'));
      });
      stream.scrollTop = stream.scrollHeight;
    }
    // Check if done
    fetch('/api/status').then(r => r.json()).then(s => {
      if (!s.pipeline_running && _liveTimer) {
        // Grab final verdict from status
        const finalVerdict = _liveVerdict || s.verdict || null;
        stopLiveMode(finalVerdict);
        const btn = document.getElementById('runBtn');
        btn.textContent = '\u25B6 RUN PIPELINE';
        btn.disabled    = false;
        btn.classList.remove('running');
        // Full refresh of all panels
        fetchStatus(); fetchAgents(); fetchLog();
      }
    });
  }).catch(() => {});
}

function runPipeline() {
  const voice = document.getElementById('voiceChk').checked;
  fetch('/api/run', {method:'POST', headers:{'Content-Type':'application/json'},
                     body: JSON.stringify({voice})})
    .then(r => r.json()).then(d => {
      if (!d.ok) { alert(d.msg); return; }
      const btn = document.getElementById('runBtn');
      btn.textContent = '\u23F3 RUNNING\u2026';
      btn.disabled = true;
      btn.classList.add('running');
      startLiveMode();
    });
}

function fetchStatus() {
  fetch('/api/status').then(r => r.json()).then(d => {
    lastStatus = d;
    document.getElementById('statSession').textContent   = d.session || '\u2014';
    const acc = document.getElementById('statAccuracy');
    acc.textContent = d.accuracy + '%';
    acc.style.color = d.regime_color || 'var(--yellow)';
    document.getElementById('statConf').textContent      = d.conf ? d.conf.toFixed(1) : '\u2014';
    document.getElementById('statThreshold').textContent = d.threshold || '\u2014';
    document.getElementById('statProceed').textContent   = d.proceed_count ?? '\u2014';
    document.getElementById('statAbort').textContent     = d.abort_count  ?? '\u2014';
    const rb = document.getElementById('regimeBadge');
    rb.textContent = d.regime || 'BALANCED';
    rb.style.color = d.regime_color || 'var(--yellow)';
    document.getElementById('binDot').classList.toggle('dead', !d.binary_ok);
    if (d.binary_info) {
      document.getElementById('binSize').textContent   = (d.binary_info.size_kb || '\u2014') + ' KB';
      document.getElementById('binMtime').textContent  = d.binary_info.mtime_str || '\u2014';
      const bs = document.getElementById('binStatus');
      bs.textContent  = d.binary_ok ? '\u2713 OK' : '\u2717 Missing';
      bs.style.color  = d.binary_ok ? 'var(--green)' : 'var(--red)';
    }
    if (!d.pipeline_running) {
      const btn = document.getElementById('runBtn');
      btn.textContent = '\u25B6 RUN PIPELINE';
      btn.disabled    = false;
      btn.classList.remove('running');
    }
  }).catch(() => {});
}

function fetchAgents() {
  fetch('/api/agents').then(r => r.json()).then(d => {
    ['oracle','hype','close','risk','conductor'].forEach(a => {
      const v = d[a];
      document.getElementById('val-' + a).textContent      = v != null ? v.toFixed(0) : '\u2014';
      document.getElementById('bar-' + a).style.width      = v != null ? Math.min(v,100)+'%' : '0%';
    });
    if (d.entanglement != null) {
      document.getElementById('entangleVal').textContent    = d.entanglement.toFixed(1) + '%';
      document.getElementById('entangleFill').style.width   = Math.min(d.entanglement,100) + '%';
    }
    if (d.verdict) {
      const vd = document.getElementById('verdictDisplay');
      const vv = document.getElementById('verdictVal');
      vv.textContent         = d.verdict;
      vv.style.color         = d.verdict_color || 'var(--text)';
      vd.style.background    = (d.verdict_color || 'transparent') + '18';
      vd.style.borderColor   = (d.verdict_color || 'var(--border)') + '44';
    }
  }).catch(() => {});
}

function fetchLog() {
  fetch('/api/log').then(r => r.json()).then(d => {
    const box = document.getElementById('logBox');
    box.textContent = d.log || '';
    document.getElementById('logLines').textContent = d.lines + ' lines';
    if (document.getElementById('autoScroll').checked) box.scrollTop = box.scrollHeight;
  }).catch(() => {});
}

function fetchFiles() {
  fetch('/api/files').then(r => r.json()).then(d => {
    const el = document.getElementById('fileTree');
    if (!d.ok || !d.tree) { el.textContent = 'Could not load files.'; return; }
    el.innerHTML = renderNode(d.tree, 0);
  }).catch(() => {});
}

function ext2cls(e) {
  if (e==='.ai')   return 'ai';
  if (e==='.rs')   return 'rs';
  if (e==='.py')   return 'py';
  if (e==='.json') return 'json';
  if (e==='.toml') return 'toml';
  return '';
}

function renderNode(node, depth) {
  if (node.t === 'd') {
    const id   = 'ft_' + node.n.replace(/[^a-zA-Z0-9]/g,'_') + '_' + depth;
    const open = fileTreeOpen[id] !== false;
    return '<div><div class="ft-dir" onclick="toggleFT(\''+id+'\')">'
      + (open?'\u25BE':'\u25B8') + ' \uD83D\uDCC1 ' + esc(node.n) + '</div>'
      + '<div class="ft-kids" id="'+id+'" style="display:'+(open?'block':'none')+'">'
      + (node.k||[]).map(c=>renderNode(c,depth+1)).join('')
      + '</div></div>';
  } else {
    const cls = ext2cls(node.e||'');
    const kb  = node.s > 0 ? ' <span style="color:var(--muted);font-size:10px;">'+(node.s/1024).toFixed(1)+'k</span>' : '';
    return '<div class="ft-file '+cls+'">\uD83D\uDCC4 '+esc(node.n)+kb+'</div>';
  }
}

function toggleFT(id) {
  const el = document.getElementById(id);
  if (!el) return;
  const open = el.style.display !== 'none';
  el.style.display = open ? 'none' : 'block';
  fileTreeOpen[id] = !open;
}

function fetchMotherJournal() {
  fetch('/api/mother/journal').then(r => r.json()).then(d => {
    if (!d.ok) return;
    d.entries.forEach(e => {
      if (e.source === 'mother_journal.txt') {
        document.getElementById('journalBox').textContent = e.content || '(empty)';
      }
      if (e.source === 'events.log' && e.events) renderEvents(e.events);
    });
  }).catch(() => {});
}

function renderEvents(events) {
  const list = document.getElementById('eventsList');
  document.getElementById('eventCount').textContent = events.length + ' events';
  if (!events.length) { list.innerHTML = '<div style="color:var(--muted);padding:12px;font-size:12px;">No events yet.</div>'; return; }
  list.innerHTML = events.slice().reverse().map(ev => {
    const ts   = ev.timestamp ? ev.timestamp.replace('T',' ').substring(0,19) : '';
    const type = ev.type || '?';
    const data = ev.data ? JSON.stringify(ev.data) : (ev.raw || '');
    const sess = ev.session ? '<span style="color:var(--accent2);">S'+ev.session+'</span> ' : '';
    return '<div class="event-item">'
      + '<div class="event-type">'+esc(type)+'</div>'
      + '<div class="event-data">'+sess+esc(data.substring(0,120))+'</div>'
      + '<div class="event-ts">'+ts+'</div>'
      + '</div>';
  }).join('');
}

function sendToMother() {
  const msg = document.getElementById('chatInput').value.trim();
  if (!msg) return;
  const hist = document.getElementById('chatHistory');
  const speak = document.getElementById('chatSpeakChk').checked;
  // Append user message
  hist.textContent += '\n\u25b6 Warren: ' + msg + '\n';
  hist.scrollTop = hist.scrollHeight;
  document.getElementById('chatInput').value = '';
  // Show thinking indicator
  const thinking = '\u22ef Mother is thinking\u2026';
  hist.textContent += thinking;
  hist.scrollTop = hist.scrollHeight;
  fetch('/api/mother/chat', {
    method: 'POST',
    headers: {'Content-Type': 'application/json'},
    body: JSON.stringify({message: msg, speak: speak})
  })
  .then(r => r.json())
  .then(d => {
    hist.textContent = hist.textContent.slice(0, hist.textContent.lastIndexOf(thinking));
    if (d.ok) {
      // Show tool activity (muted)
      if (d.tool_log && d.tool_log.length) {
        d.tool_log.forEach(t => { hist.textContent += '  \u2192 ' + t + '\n'; });
      }
      hist.textContent += '\u25c6 Mother: ' + d.reply + '\n\n';
      if (speak && d.reply) browserSpeak(d.reply, 0.95, 1.05);
    } else {
      hist.textContent += '\u26a0 Error: ' + d.msg + '\n\n';
    }
    hist.scrollTop = hist.scrollHeight;
  })
  .catch(e => {
    hist.textContent = hist.textContent.slice(0, hist.textContent.lastIndexOf(thinking));
    hist.textContent += '\u26a0 Connection error: ' + e + '\n\n';
    hist.scrollTop = hist.scrollHeight;
  });
}


// ── Mic / Speech-to-Text ──────────────────────────────────────────────────────
let _micRec = null, _micActive = false;

function toggleMic() {
  if (_micActive) { stopMic(); return; }
  const SR = window.SpeechRecognition || window.webkitSpeechRecognition;
  if (!SR) {
    document.getElementById('micStatus').textContent = '⚠ Speech not supported — use Chrome';
    return;
  }
  _micRec = new SR();
  _micRec.continuous      = false;
  _micRec.interimResults  = true;
  _micRec.lang            = 'en-US';
  _micRec.maxAlternatives = 1;

  const btn = document.getElementById('micBtn');
  const inp = document.getElementById('chatInput');
  let _savedText = inp.value;

  _micRec.onstart = () => {
    _micActive = true;
    btn.style.background   = '#7f1d1d';
    btn.style.borderColor  = '#ef4444';
    btn.style.color        = '#fca5a5';
    btn.style.boxShadow    = '0 0 10px #ef444488';
    document.getElementById('micStatus').textContent = '🔴 Listening…';
  };

  _micRec.onresult = (e) => {
    let interim = '', final = '';
    for (let i = e.resultIndex; i < e.results.length; i++) {
      if (e.results[i].isFinal) final += e.results[i][0].transcript;
      else interim += e.results[i][0].transcript;
    }
    inp.value = _savedText + (final || interim);
    if (final) _savedText = inp.value;
  };

  _micRec.onerror = (e) => {
    document.getElementById('micStatus').textContent = '⚠ Mic error: ' + e.error;
    stopMic();
  };

  _micRec.onend = () => {
    stopMic();
    // Auto-send if we got text
    if (inp.value.trim()) sendToMother();
  };

  try { _micRec.start(); }
  catch(e) { document.getElementById('micStatus').textContent = '⚠ ' + e.message; }
}

function stopMic() {
  _micActive = false;
  if (_micRec) { try { _micRec.stop(); } catch(e){} }
  const btn = document.getElementById('micBtn');
  btn.style.background  = '#0f1117';
  btn.style.borderColor = '#475569';
  btn.style.color       = '#94a3b8';
  btn.style.boxShadow   = 'none';
  document.getElementById('micStatus').textContent = '🎤 Mic ready';
}


// ── Browser Neural TTS ────────────────────────────────────────────────────────
let _ttsVoice = null;

function _initVoice() {
  if (_ttsVoice) return;
  const voices = speechSynthesis.getVoices();
  // Prefer Microsoft neural online voices in this order
  const prefer = [
    'Microsoft Aria Online (Natural)',
    'Microsoft Jenny Online (Natural)',
    'Microsoft Aria - English (United States)',
    'Microsoft Zira Online',
    'Microsoft Zira Desktop',
  ];
  for (const name of prefer) {
    const v = voices.find(v => v.name.includes(name.split(' Online')[0]) && v.lang.startsWith('en'));
    if (v) { _ttsVoice = v; break; }
  }
  // Fallback: any English female-ish voice
  if (!_ttsVoice) _ttsVoice = voices.find(v => v.lang.startsWith('en-US')) || voices[0];
}

function browserSpeak(text, rate=1.0, pitch=1.05) {
  if (!window.speechSynthesis || !text) return;
  speechSynthesis.cancel();
  _initVoice();
  const utt = new SpeechSynthesisUtterance(text.slice(0, 500));
  if (_ttsVoice) utt.voice = _ttsVoice;
  utt.rate  = rate;
  utt.pitch = pitch;
  utt.volume = 1.0;
  speechSynthesis.speak(utt);
}

// Load voices (Chrome loads them async)
if (speechSynthesis.onvoiceschanged !== undefined) {
  speechSynthesis.onvoiceschanged = _initVoice;
}
setTimeout(_initVoice, 500);

function speakCustom() {
  const txt = document.getElementById('voiceText').value.trim();
  if (!txt) return;
  browserSpeak(txt);
  // Also trigger server-side (plays on machine speakers either way)
  fetch('/api/voice', {method:'POST', headers:{'Content-Type':'application/json'},
                       body: JSON.stringify({text: txt})});
}

function speakLastVerdict() {
  if (!lastStatus.last_verdict || lastStatus.last_verdict === '\u2014') {
    alert('No verdict yet. Run the pipeline first.');
    return;
  }
  fetch('/api/voice', {method:'POST', headers:{'Content-Type':'application/json'},
    body: JSON.stringify({text: 'Last verdict: '+lastStatus.last_verdict+'. Session '+lastStatus.session+'.'})});
}

function loadSettings() {
  fetch('/api/settings/get').then(r => r.json()).then(d => {
    if (!d.ok) return;
    const cfg = d.config || {};
    ['anthropic_api_key','openai_api_key','ibm_token','model','max_tokens','threshold'].forEach(k => {
      const el = document.getElementById('cfg-' + k);
      if (el && cfg[k] != null) el.value = cfg[k];
    });
    document.getElementById('settingsPath').textContent = 'Config: ' + d.path;
  }).catch(() => {});
}

function saveSettings() {
  const fields = ['anthropic_api_key','openai_api_key','ibm_token','model','max_tokens','threshold'];
  const updates = {};
  fields.forEach(k => {
    const el = document.getElementById('cfg-' + k);
    if (el && el.value.trim()) updates[k] = el.value.trim();
  });
  fetch('/api/settings/save', {method:'POST', headers:{'Content-Type':'application/json'},
                                body: JSON.stringify({updates})})
    .then(r => r.json()).then(d => {
      const msg = document.getElementById('settingsMsg');
      msg.textContent = d.ok ? '\u2713 Saved' : '\u2717 ' + d.msg;
      msg.style.color = d.ok ? 'var(--green)' : 'var(--red)';
      setTimeout(() => { msg.textContent = ''; }, 3000);
    });
}

function togglePw(id, btn) {
  const el = document.getElementById(id);
  el.type = el.type === 'password' ? 'text' : 'password';
  btn.textContent = el.type === 'password' ? '\uD83D\uDC41' : '\uD83D\uDE48';
}

function fetchGitStatus() {
  document.getElementById('gitOutput').textContent = 'Loading\u2026';
  fetch('/api/github/status').then(r => r.json()).then(d => {
    document.getElementById('gitOutput').textContent = d.ok ? 'Ready.' : (d.msg || 'Error');
    document.getElementById('gitBranch').textContent = d.branch || '?';
    gitFiles = d.files || [];
    renderGitFiles(gitFiles);
    renderGitLog(d.recent_commits || []);
  }).catch(e => { document.getElementById('gitOutput').textContent = String(e); });
}

function renderGitFiles(files) {
  const el = document.getElementById('gitFileList');
  if (!files.length) {
    el.innerHTML = '<div style="color:var(--muted);padding:12px;font-size:12px;">Clean \u2014 nothing to commit.</div>';
    return;
  }
  el.innerHTML = files.map((f,i) => {
    const sc = f.status === '??' ? 'Q' : (f.status||'').replace(/[^MADRCU]/g,'') || 'Q';
    return '<div class="git-file-item">'
      + '<input type="checkbox" class="git-chk" data-idx="'+i+'" checked>'
      + '<span class="git-status-badge git-status-'+sc+'">'+esc(f.status)+'</span>'
      + '<span class="git-file-path" title="'+esc(f.path)+'">'+esc(shortPath(f.path))+'</span>'
      + '</div>';
  }).join('');
}

function renderGitLog(commits) {
  document.getElementById('gitLogList').innerHTML =
    commits.map(c => '<div class="git-log-item">'+esc(c)+'</div>').join('');
}

function toggleSelectAll() {
  const boxes = document.querySelectorAll('.git-chk');
  const all   = Array.from(boxes).every(b => b.checked);
  boxes.forEach(b => b.checked = !all);
}

function getSelectedPaths() {
  const paths = [];
  document.querySelectorAll('.git-chk:checked').forEach(b => {
    const i = parseInt(b.dataset.idx);
    if (gitFiles[i]) paths.push(gitFiles[i].path);
  });
  return paths;
}

function doCommit() {
  const msg   = document.getElementById('commitMsg').value.trim();
  const paths = getSelectedPaths();
  if (!msg)          { alert('Enter a commit message first.'); return; }
  if (!paths.length) { alert('Select at least one file.'); return; }
  gitAction('/api/github/commit', {message: msg, paths});
}

function doCommitAll() {
  const msg = document.getElementById('commitMsg').value.trim();
  if (!msg) { alert('Enter a commit message first.'); return; }
  gitAction('/api/github/commit', {message: msg, paths: []});
}

function doPush() { gitAction('/api/github/push', {}); }
function doPull() { gitAction('/api/github/pull', {}); }

function gitAction(url, body) {
  const out = document.getElementById('gitOutput');
  out.textContent  = 'Working\u2026';
  out.style.color  = '#94a3b8';
  fetch(url, {method:'POST', headers:{'Content-Type':'application/json'}, body: JSON.stringify(body)})
    .then(r => r.json()).then(d => {
      out.textContent = d.msg || (d.ok ? 'Done.' : 'Error');
      out.style.color = d.ok ? '#94a3b8' : 'var(--red)';
      if (d.ok) setTimeout(fetchGitStatus, 800);
    })
    .catch(e => { out.textContent = String(e); out.style.color = 'var(--red)'; });
}

function esc(s) {
  return String(s).replace(/&/g,'&amp;').replace(/</g,'&lt;').replace(/>/g,'&gt;');
}
function shortPath(p) {
  return p.length <= 52 ? p : '\u2026' + p.slice(-49);
}

fetchStatus();
fetchAgents();
fetchLog();
fetchFiles();
setInterval(fetchStatus, 2000);
setInterval(fetchAgents, 3000);
setInterval(fetchLog,    4000);
setInterval(fetchFiles, 30000);
</script>
</body>
</html>"""

if __name__ == "__main__":
    import webbrowser
    print("=" * 60)
    print("  Aeonmi Command Center v2")
    print(f"  http://localhost:{PORT}")
    print("  Tabs: Command | Mother | Settings | GitHub | Shard")
    print("=" * 60)
    threading.Timer(1.2, lambda: webbrowser.open(f"http://localhost:{PORT}")).start()
    app.run(host="0.0.0.0", port=PORT, debug=False, threaded=True)
