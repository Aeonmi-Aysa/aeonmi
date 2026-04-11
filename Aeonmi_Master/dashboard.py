#!/usr/bin/env python3
"""
Aeonmi Nexus — Unified Dashboard v2
Mother AI conversation · Shard canvas · File explorer · Command center
Run: python Aeonmi_Master/dashboard.py
"""
import os
import sys
import json
import subprocess
import threading
import shutil
from pathlib import Path
from datetime import datetime

# ── Knowledge store (Mother's memory of textbooks) ────────────────────────────
try:
    from knowledge_store import ingest_file as _ks_ingest_file, get_context_for_query as _ks_context, status as _ks_status
    _KS_AVAILABLE = True
except ImportError:
    _KS_AVAILABLE = False
    def _ks_ingest_file(p): return 0, "knowledge_store not found"
    def _ks_context(q, **kw): return ""
    def _ks_status(): return {"chunk_count": 0}

# Auto-install Flask if missing
try:
    from flask import Flask, request, jsonify, make_response
except ImportError:
    print("Flask not found — installing...")
    subprocess.check_call([sys.executable, "-m", "pip", "install", "flask", "--quiet"])
    from flask import Flask, request, jsonify, make_response

# ── Project root ──────────────────────────────────────────────────────────────

PROJECT_ROOT = Path(__file__).parent.parent.resolve()

def _find_binary() -> Path:
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
    return candidates[2]  # useful fallback path for error messages

BINARY = _find_binary()

# ── .env key management ───────────────────────────────────────────────────────

ENV_PATH    = PROJECT_ROOT / ".env"
_KNOWN_KEYS = [
    "ANTHROPIC_API_KEY",
    "OPENROUTER_API_KEY",
    "OPENAI_API_KEY",
    "DEEPSEEK_API_KEY",
    "GROK_API_KEY",
    "PERPLEXITY_API_KEY",
    "GITHUB_TOKEN",
]

def _load_dotenv():
    """Load .env into os.environ (does not overwrite existing env vars)."""
    if not ENV_PATH.exists():
        return
    for line in ENV_PATH.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line or line.startswith("#") or "=" not in line:
            continue
        k, v = line.split("=", 1)
        k = k.strip()
        v = v.strip().strip('"').strip("'")
        os.environ[k] = v  # always overwrite so updated .env takes effect on restart

def _write_key(name: str, value: str):
    """Persist a key to .env and set in current process."""
    lines = []
    found = False
    if ENV_PATH.exists():
        for line in ENV_PATH.read_text(encoding="utf-8").splitlines():
            if line.strip().startswith(f"{name}="):
                lines.append(f"{name}={value}")
                found = True
            else:
                lines.append(line)
    if not found:
        lines.append(f"{name}={value}")
    ENV_PATH.write_text("\n".join(lines) + "\n", encoding="utf-8")
    os.environ[name] = value

def _remove_key(name: str):
    """Remove a key from .env and from current process."""
    if ENV_PATH.exists():
        lines = [l for l in ENV_PATH.read_text(encoding="utf-8").splitlines()
                 if not l.strip().startswith(f"{name}=")]
        ENV_PATH.write_text("\n".join(lines) + "\n", encoding="utf-8")
    os.environ.pop(name, None)

_load_dotenv()  # load on startup so API keys are available immediately

# ── Flask app ─────────────────────────────────────────────────────────────────

app = Flask(__name__)
app.config["JSON_SORT_KEYS"] = False

# ── In-memory state ───────────────────────────────────────────────────────────

_conversation: list = []   # [{role, content, ts}]
_action_queue: list = []   # Mother's planned next steps
_action_log:   list = []   # completed actions [{ts, action, outcome}]
_exec_log:     list = []   # recent command outputs
_lock = threading.Lock()

# ── Genesis memory (persistent across restarts) ───────────────────────────────

GENESIS_PATH      = PROJECT_ROOT / "Aeonmi_Master" / "genesis.json"
CONVERSATION_PATH = PROJECT_ROOT / "Aeonmi_Master" / "conversation_history.jsonl"
CONVERSATION_KEEP = 300   # max turns persisted to disk

def _load_genesis() -> dict:
    """Phase 5: read unified schema. Returns full payload with all sections."""
    try:
        if GENESIS_PATH.exists():
            data = json.loads(GENESIS_PATH.read_text(encoding="utf-8"))
            # Handle legacy flat schema — wrap in operational section
            if "operational" not in data and "key_facts" in data:
                data["operational"] = {
                    "dashboard_interaction_count": data.get("interaction_count", 0),
                    "key_facts": data.get("key_facts", []),
                    "action_summary": data.get("action_summary", []),
                    "last_session_ts": data.get("last_updated", ""),
                }
            return data
    except Exception:
        pass
    return {
        "_schema_version": "5.0",
        "operational": {
            "dashboard_interaction_count": 0,
            "key_facts": [],
            "action_summary": [],
            "last_session_ts": "",
        }
    }

def _save_genesis():
    """Phase 5: read-modify-write. Updates only the `operational` section.
    Preserves `cognitive` (Rust) and `ai_memory` (.ai) sections untouched."""
    try:
        # Read existing — preserve all other tracks
        existing: dict = {}
        if GENESIS_PATH.exists():
            try:
                existing = json.loads(GENESIS_PATH.read_text(encoding="utf-8"))
            except Exception:
                existing = {}

        # Update only the operational section
        existing["_schema_version"] = "5.0"
        existing["_last_writer"]    = "python"
        existing["_last_updated"]   = datetime.utcnow().isoformat() + "Z"

        existing["operational"] = {
            "dashboard_interaction_count": len(_conversation),
            "key_facts": _memory.get("operational", {}).get("key_facts", [])
                         or _memory.get("key_facts", []),
            "action_summary": [e["action"] for e in _action_log[-20:]],
            "last_session_ts": datetime.utcnow().strftime("%H:%M:%S"),
        }

        # Ensure ai_memory stub exists if missing
        if "ai_memory" not in existing:
            existing["ai_memory"] = {
                "journal_entries": [],
                "memory_keys": [],
                "active_rules": [],
                "last_sync": "",
            }

        GENESIS_PATH.write_text(json.dumps(existing, indent=2), encoding="utf-8")
    except Exception as e:
        print(f"[genesis] save error: {e}", file=sys.stderr)

_memory: dict = _load_genesis()

# ── Conversation persistence ───────────────────────────────────────────────────

def _load_conversation() -> list:
    """Load prior conversation turns from disk (JSONL). Returns list of {role, content, ts}."""
    turns = []
    try:
        if CONVERSATION_PATH.exists():
            for line in CONVERSATION_PATH.read_text(encoding="utf-8").splitlines():
                line = line.strip()
                if line:
                    turns.append(json.loads(line))
            # Keep only the most recent turns
            turns = turns[-CONVERSATION_KEEP:]
    except Exception as e:
        print(f"[conversation] load error: {e}", file=sys.stderr)
    return turns

def _persist_message(msg: dict):
    """Append one message to the conversation history file."""
    try:
        with open(str(CONVERSATION_PATH), "a", encoding="utf-8") as f:
            f.write(json.dumps(msg, ensure_ascii=False) + "\n")
        # Trim file if it grows beyond CONVERSATION_KEEP lines
        try:
            lines = CONVERSATION_PATH.read_text(encoding="utf-8").splitlines()
            if len(lines) > CONVERSATION_KEEP:
                CONVERSATION_PATH.write_text(
                    "\n".join(lines[-CONVERSATION_KEEP:]) + "\n", encoding="utf-8"
                )
        except Exception:
            pass
    except Exception as e:
        print(f"[conversation] persist error: {e}", file=sys.stderr)

# Restore prior conversation on startup
_conversation = _load_conversation()

def _ts() -> str:
    return datetime.now().strftime("%H:%M:%S")

def _log_exec(cmd: str, output: str, ok: bool):
    with _lock:
        _exec_log.append({"ts": _ts(), "cmd": cmd, "output": output, "ok": ok})
        if len(_exec_log) > 200:
            _exec_log.pop(0)

def _add_message(role: str, content: str):
    msg = {"role": role, "content": content, "ts": _ts()}
    with _lock:
        _conversation.append(msg)
        if len(_conversation) > 500:
            _conversation.pop(0)
    _persist_message(msg)

def _log_action(action: str, outcome: str):
    with _lock:
        _action_log.append({"ts": _ts(), "action": action, "outcome": outcome})
        if len(_action_log) > 200:
            _action_log.pop(0)

# ── Binary helpers ────────────────────────────────────────────────────────────

def run_aeonmi(*args: str, cwd=None, timeout=30):
    cmd = [str(BINARY)] + list(args)
    try:
        r = subprocess.run(cmd, capture_output=True, text=True,
                           encoding='utf-8', errors='replace',
                           timeout=timeout, cwd=str(cwd or PROJECT_ROOT))
        out = (r.stdout + r.stderr).strip()
        return out, r.returncode == 0
    except subprocess.TimeoutExpired:
        return "Execution timed out.", False
    except FileNotFoundError:
        return f"Binary not found: {BINARY}\nRun: cargo build --release", False
    except Exception as e:
        return f"Error: {e}", False

def run_shell(*args: str, cwd=None, timeout=60):
    try:
        r = subprocess.run(list(args), capture_output=True, text=True,
                           encoding='utf-8', errors='replace',
                           timeout=timeout, cwd=str(cwd or PROJECT_ROOT))
        out = (r.stdout + r.stderr).strip()
        return out, r.returncode == 0
    except subprocess.TimeoutExpired:
        return "Timed out.", False
    except Exception as e:
        return f"Error: {e}", False

# ── Path safety ───────────────────────────────────────────────────────────────
# Allowed roots: project root + user home (Desktop, Documents, etc.)
# Blocked: Windows system paths.
_PATH_DENY = [
    Path("C:/Windows"), Path("C:/Program Files"), Path("C:/Program Files (x86)"),
    Path("C:/ProgramData"), Path("C:/System32"),
]

def _safe_path(rel: str) -> Path:
    candidate = Path(rel)
    if candidate.is_absolute():
        p = candidate.resolve()
    else:
        # Try project root first, then home
        p = (PROJECT_ROOT / rel).resolve()
    # Allow: under project root OR under user home
    home = Path.home().resolve()
    proj = PROJECT_ROOT.resolve()
    p_str = str(p)
    allowed = str(p_str).startswith(str(proj)) or str(p_str).startswith(str(home))
    if not allowed:
        raise ValueError(f"Path outside allowed workspace: {p}")
    # Block system paths
    for deny in _PATH_DENY:
        try:
            p.relative_to(deny)
            raise ValueError(f"System path not allowed: {p}")
        except ValueError as e:
            if "System path" in str(e):
                raise
    return p

# ── Mother tools ──────────────────────────────────────────────────────────────
# Mother can invoke these during conversation via [TOOL: name arg] syntax.

import re as _re
# Match [TOOL: name arg] — arg may contain balanced [] pairs (e.g. arrays in code)
# Uses a character class that allows ] only inside a balanced [...] sub-expression,
# falling back to a greedy outer match anchored at the LAST ] on the line or block.
_TOOL_RE = _re.compile(
    r'\[TOOL:\s*(\w+)\s+((?:[^\[\]]|\[[^\[\]]*\])*)\]',
    _re.DOTALL | _re.IGNORECASE,
)

def _tool_read_file(path: str) -> str:
    try:
        p = _safe_path(path.strip())
        if not p.exists():
            # Try common variations (e.g. .doc → .docx)
            for alt in [p.with_suffix(".docx"), p.with_suffix(".doc"), p.with_suffix(".txt")]:
                if alt.exists():
                    p = alt
                    break
            else:
                return f"File not found: {path}"
        ext = p.suffix.lower()
        # .docx — extract plain text from word/document.xml
        if ext in (".docx", ".doc"):
            import zipfile, xml.etree.ElementTree as ET
            try:
                with zipfile.ZipFile(p) as z:
                    xml_content = z.read("word/document.xml")
                root = ET.fromstring(xml_content)
                ns = {"w": "http://schemas.openxmlformats.org/wordprocessingml/2006/main"}
                paragraphs = []
                for para in root.iter("{http://schemas.openxmlformats.org/wordprocessingml/2006/main}p"):
                    texts = [t.text or "" for t in para.iter("{http://schemas.openxmlformats.org/wordprocessingml/2006/main}t")]
                    line = "".join(texts).strip()
                    if line:
                        paragraphs.append(line)
                text = "\n".join(paragraphs)
                if len(text) > 8000:
                    text = text[:8000] + "\n… (truncated)"
                return text
            except Exception as e:
                return f"Could not parse {p.name} as docx: {e}"
        # Plain text
        text = p.read_text(encoding="utf-8", errors="replace")
        if len(text) > 24000:
            text = text[:24000] + "\n… (truncated at 24000 chars)"
        return text
    except Exception as e:
        return f"read_file error: {e}"

def _tool_write_file(arg: str, _code_blocks: list = None) -> str:
    """arg format: path/to/file|file content here"""
    try:
        if "|" not in arg:
            return "write_file error: use [TOOL: write_file path/to/file|content]"
        path, content = arg.split("|", 1)
        path = path.strip()
        # Strip any leading "content:" label the model adds
        content = _re.sub(r'^content\s*:\s*', '', content.strip(), flags=_re.IGNORECASE).strip()
        # If content is a reference like "above" / "see above" / empty, use last code block
        if _code_blocks and (not content or content.lower() in (
                "above", "see above", "content above", "the above", "as above")):
            content = _code_blocks[-1]
        if not content:
            return "write_file error: no content provided"
        p = _safe_path(path)
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(content, encoding="utf-8")
        rel = str(p.relative_to(PROJECT_ROOT)).replace("\\", "/")
        print(f"[tool] write_file → {rel} ({len(content)} chars)", flush=True)
        return f"Written {rel} ({len(content)} chars)"
    except Exception as e:
        return f"write_file error: {e}"

def _tool_mkdir(path: str) -> str:
    try:
        p = _safe_path(path.strip())
        p.mkdir(parents=True, exist_ok=True)
        rel = str(p.relative_to(PROJECT_ROOT)).replace("\\", "/")
        print(f"[tool] mkdir → {rel}", flush=True)
        return f"Created directory: {rel}"
    except Exception as e:
        return f"mkdir error: {e}"

def _tool_list_dir(path: str) -> str:
    try:
        base = _safe_path(path.strip()) if path.strip() else PROJECT_ROOT
        if not base.is_dir():
            return f"Not a directory: {path}"
        entries = sorted(base.iterdir(), key=lambda x: (x.is_file(), x.name.lower()))
        lines = [f"📂 {base}"]
        for e in entries:
            size = f"  {e.stat().st_size:,}b" if e.is_file() else ""
            lines.append(("📁 " if e.is_dir() else "📄 ") + e.name + size)
        return "\n".join(lines)
    except Exception as e:
        return f"list_dir error: {e}"


def _tool_append_file(arg: str) -> str:
    """Append content to a file. arg: path/to/file|content to append"""
    try:
        if "|" not in arg:
            return "append_file error: use append_file path|content"
        path, content = arg.split("|", 1)
        path = path.strip()
        content = content.strip()
        p = _safe_path(path)
        p.parent.mkdir(parents=True, exist_ok=True)
        with p.open("a", encoding="utf-8") as f:
            f.write(content)
        print(f"[tool] append_file → {path} (+{len(content)} chars)", flush=True)
        return f"Appended {len(content)} chars to {path} (total {p.stat().st_size:,}b)"
    except Exception as e:
        return f"append_file error: {e}"


def _tool_run_file(path: str) -> str:
    """Run a .ai file through the Aeonmi native VM."""
    try:
        p = _safe_path(path.strip())
        if not p.exists():
            return f"File not found: {path}"
        out, ok = run_aeonmi("native", str(p), timeout=30)
        print(f"[tool] run {path} → {'OK' if ok else 'ERROR'}", flush=True)
        _log_exec(f"run {path}", out, ok)
        _log_action(f"run {path}", "OK" if ok else "ERROR")
        return ("✓ " if ok else "✗ ") + out
    except Exception as e:
        return f"run error: {e}"


def _tool_delete_file(path: str) -> str:
    """Delete a file or directory."""
    try:
        p = _safe_path(path.strip())
        if not p.exists():
            return f"Not found: {path}"
        if p.is_dir():
            shutil.rmtree(p)
            return f"Deleted directory: {path}"
        else:
            p.unlink()
            return f"Deleted file: {path}"
    except Exception as e:
        return f"delete error: {e}"

def _tool_fetch_url(url: str) -> str:
    import urllib.request
    try:
        url = url.strip()
        req = urllib.request.Request(url, headers={"User-Agent": "Aeonmi-Mother/1.0"})
        with urllib.request.urlopen(req, timeout=12) as resp:
            raw = resp.read()
            ct  = resp.headers.get("Content-Type", "")
            text = raw.decode("utf-8", errors="replace")
        if "html" in ct.lower():
            text = _re.sub(r'<style[^>]*>.*?</style>', ' ', text, flags=_re.DOTALL | _re.IGNORECASE)
            text = _re.sub(r'<script[^>]*>.*?</script>', ' ', text, flags=_re.DOTALL | _re.IGNORECASE)
            text = _re.sub(r'<[^>]+>', ' ', text)
            text = _re.sub(r'\s{3,}', '\n', text).strip()
        if len(text) > 6000:
            text = text[:6000] + "\n… (truncated)"
        return text
    except Exception as e:
        return f"fetch_url error: {e}"

def _tool_search(query: str) -> str:
    import urllib.request, urllib.parse
    try:
        q = urllib.parse.quote(query.strip())
        url = f"https://api.duckduckgo.com/?q={q}&format=json&no_html=1&skip_disambig=1"
        req = urllib.request.Request(url, headers={"User-Agent": "Aeonmi-Mother/1.0"})
        with urllib.request.urlopen(req, timeout=10) as resp:
            data = json.loads(resp.read())
        lines = []
        if data.get("AbstractText"):
            lines.append(data["AbstractText"])
        for r in data.get("RelatedTopics", [])[:6]:
            if isinstance(r, dict) and r.get("Text"):
                lines.append("• " + r["Text"])
        return "\n".join(lines) if lines else "No instant results. Try fetch_url with a specific page."
    except Exception as e:
        return f"search error: {e}"

def _tool_github(arg: str) -> str:
    """arg: owner/repo  or  owner/repo/path/to/file"""
    import urllib.request, base64
    try:
        parts = arg.strip().strip("/").split("/", 2)
        if len(parts) < 2:
            return "github error: use owner/repo or owner/repo/path/to/file"
        owner, repo = parts[0], parts[1]
        path  = parts[2] if len(parts) > 2 else ""
        url   = f"https://api.github.com/repos/{owner}/{repo}/contents/{path}" if path \
                else f"https://api.github.com/repos/{owner}/{repo}"
        headers = {"User-Agent": "Aeonmi-Mother/1.0", "Accept": "application/vnd.github.v3+json"}
        gh_token = os.environ.get("GITHUB_TOKEN")
        if gh_token:
            headers["Authorization"] = f"token {gh_token}"
        req = urllib.request.Request(url, headers=headers)
        with urllib.request.urlopen(req, timeout=12) as resp:
            data = json.loads(resp.read())
        if isinstance(data, list):
            lines = [("📁 " if e["type"] == "dir" else "📄 ") + e["name"] for e in data[:40]]
            return "\n".join(lines)
        if isinstance(data, dict) and "content" in data:
            content = base64.b64decode(data["content"].replace("\n", "")).decode("utf-8", errors="replace")
            if len(content) > 7000:
                content = content[:7000] + "\n… (truncated)"
            return content
        return json.dumps(data, indent=2)[:3000]
    except Exception as e:
        return f"github error: {e}"

def _tool_learn_file(arg: str) -> str:
    """Ingest a file into Mother's knowledge store."""
    try:
        p = _safe_path(arg)
    except ValueError:
        p = Path(arg)
    n, msg = _ks_ingest_file(p)
    return msg

def _tool_bash(arg: str) -> str:
    """Execute any shell/bash command and return output (stdout+stderr)."""
    try:
        # Replace bare 'aeonmi' with the resolved binary path so Mother's
        # [TOOL: bash aeonmi run/build/native ...] commands actually work.
        import re as _re2
        binary_str = str(BINARY).replace("\\", "/")
        arg = _re2.sub(
            r'(?<![/\\])\baeonmi\b',
            f'"{binary_str}"',
            arg,
        )
        flags = subprocess.CREATE_NO_WINDOW if hasattr(subprocess, "CREATE_NO_WINDOW") else 0
        result = subprocess.run(
            arg, shell=True, capture_output=True, text=True,
            cwd=str(PROJECT_ROOT), timeout=60,
            encoding="utf-8", errors="replace",
            creationflags=flags,
        )
        out = (result.stdout + result.stderr).strip()
        return out or f"(exit {result.returncode})"
    except subprocess.TimeoutExpired:
        return "Command timed out (60s limit)"
    except Exception as e:
        return f"Shell error: {e}"

def _tool_rename(arg: str) -> str:
    """Rename file/folder: old_path|new_name_or_path"""
    if "|" not in arg:
        return "Error: use rename old_path|new_path"
    old, new = arg.split("|", 1)
    try:
        old_p = _safe_path(old.strip())
        new_p = _safe_path(new.strip()) if "/" in new or "\\" in new else old_p.parent / new.strip()
        old_p.rename(new_p)
        return f"Renamed → {new_p.relative_to(PROJECT_ROOT)}"
    except Exception as e:
        return f"Rename error: {e}"

def _tool_copy(arg: str) -> str:
    """Copy file: src_path|dest_path"""
    if "|" not in arg:
        return "Error: use copy src_path|dest_path"
    src, dst = arg.split("|", 1)
    try:
        src_p = _safe_path(src.strip())
        dst_p = _safe_path(dst.strip())
        dst_p.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(str(src_p), str(dst_p))
        return f"Copied → {dst_p.relative_to(PROJECT_ROOT)}"
    except Exception as e:
        return f"Copy error: {e}"

def _tool_git(arg: str) -> str:
    """Run a git command: status, log, diff, add ., commit -m 'msg', etc."""
    try:
        flags = subprocess.CREATE_NO_WINDOW if hasattr(subprocess, "CREATE_NO_WINDOW") else 0
        result = subprocess.run(
            f"git {arg}", shell=True, capture_output=True, text=True,
            cwd=str(PROJECT_ROOT), timeout=30,
            encoding="utf-8", errors="replace", creationflags=flags,
        )
        return (result.stdout + result.stderr).strip() or "(no output)"
    except Exception as e:
        return f"Git error: {e}"

def _tool_python(arg: str) -> str:
    """Run a Python snippet and return output."""
    import io, contextlib
    buf = io.StringIO()
    try:
        with contextlib.redirect_stdout(buf), contextlib.redirect_stderr(buf):
            exec(arg, {"__builtins__": __builtins__, "PROJECT_ROOT": PROJECT_ROOT, "Path": Path, "json": json})
        return buf.getvalue().strip() or "(no output)"
    except Exception as e:
        return f"Python error: {e}\n{buf.getvalue()}"

def _tool_env(arg: str) -> str:
    """Get or set environment variable. get VAR_NAME  or  set VAR_NAME=value"""
    arg = arg.strip()
    if arg.lower().startswith("set "):
        pair = arg[4:].strip()
        if "=" in pair:
            k, v = pair.split("=", 1)
            os.environ[k.strip()] = v.strip()
            return f"Set {k.strip()}"
        return "Error: use env set VAR=value"
    elif arg.lower().startswith("get "):
        k = arg[4:].strip()
        return os.environ.get(k, f"(not set)")
    elif arg.lower() == "list":
        keys = ["ANTHROPIC_API_KEY","OPENROUTER_API_KEY","OPENAI_API_KEY","GITHUB_TOKEN","IBM_TOKEN"]
        return "\n".join(f"{k}={'***' if os.environ.get(k) else '(not set)'}" for k in keys)
    return os.environ.get(arg, f"(not set)")

def _tool_agent(arg: str) -> str:
    """Spawn a named agent and get result: agent oracle_agent  or  agent new:name:goal"""
    parts = arg.strip().split(":", 2)
    if parts[0].strip().lower() == "new" and len(parts) >= 3:
        # Dynamic agent creation
        _, name, goal = parts[0].strip(), parts[1].strip(), parts[2].strip()
        safe_name = _re.sub(r"[^\w]", "_", name.lower())
        agent_dir = PROJECT_ROOT / "Aeonmi_Master" / "aeonmi_ai" / "agent"
        agent_dir.mkdir(parents=True, exist_ok=True)
        agent_file = agent_dir / f"{safe_name}.ai"
        agent_file.write_text(
            f"⍝ Agent: {name}\n⍝ Goal: {goal}\n\nfn main() {{\n  print(\"{goal}\")\n}}\nreturn main();\n",
            encoding="utf-8"
        )
        return f"Created agent {safe_name}.ai — run it with [TOOL: run Aeonmi_Master/aeonmi_ai/agent/{safe_name}.ai]"
    # Run existing agent
    name = arg.strip()
    agent_dir = PROJECT_ROOT / "Aeonmi_Master" / "aeonmi_ai" / "agent"
    candidates = [agent_dir / name, agent_dir / f"{name}.ai", PROJECT_ROOT / name]
    for c in candidates:
        if c.exists():
            return _tool_run_file(str(c.relative_to(PROJECT_ROOT)))
    return f"Agent '{name}' not found. Available: {', '.join(p.stem for p in agent_dir.glob('*.ai'))}"

_TOOL_DISPATCH = {
    # File system
    "read_file":         _tool_read_file,
    "read":              _tool_read_file,
    "write_file":        _tool_write_file,
    "write":             _tool_write_file,
    "append_file":       _tool_append_file,
    "append":            _tool_append_file,
    "list_dir":          _tool_list_dir,
    "ls":                _tool_list_dir,
    "dir":               _tool_list_dir,
    "mkdir":             _tool_mkdir,
    "create_directory":  _tool_mkdir,
    "create_dir":        _tool_mkdir,
    "delete":            _tool_delete_file,
    "delete_file":       _tool_delete_file,
    "rm":                _tool_delete_file,
    "rename":            _tool_rename,
    "mv":                _tool_rename,
    "copy":              _tool_copy,
    "cp":                _tool_copy,
    # Execution
    "run":               _tool_run_file,
    "run_file":          _tool_run_file,
    "native":            _tool_run_file,
    "bash":              _tool_bash,
    "shell":             _tool_bash,
    "cmd":               _tool_bash,
    "python":            _tool_python,
    "py":                _tool_python,
    # Web
    "fetch_url":         _tool_fetch_url,
    "fetch":             _tool_fetch_url,
    "search":            _tool_search,
    "web_search":        _tool_search,
    "github":            _tool_github,
    "gh":                _tool_github,
    # Dev
    "git":               _tool_git,
    "env":               _tool_env,
    # Knowledge
    "learn_file":        _tool_learn_file,
    "learn":             _tool_learn_file,
    "ingest":            _tool_learn_file,
    # Agent orchestration
    "agent":             _tool_agent,
    "spawn":             _tool_agent,
}

def _normalize_tool_arg(arg: str) -> str:
    """Strip model-generated prefixes like '| path:', '| url:', 'path:', etc.
    Also convert absolute paths that are inside the project root to relative paths."""
    arg = arg.strip()
    # Strip leading pipe and key labels: "| path: foo" → "foo"
    arg = _re.sub(r'^\|\s*(path|url|query|file|repo|target)\s*:\s*', '', arg, flags=_re.IGNORECASE).strip()
    # Also plain "path: foo" without the pipe
    arg = _re.sub(r'^(path|url|query|file|repo|target)\s*:\s*', '', arg, flags=_re.IGNORECASE).strip()
    # Convert absolute path inside project root to relative
    try:
        p = Path(arg)
        if p.is_absolute():
            rel = p.relative_to(PROJECT_ROOT)
            arg = str(rel).replace("\\", "/")
    except (ValueError, TypeError):
        pass
    # Normalize backslashes to forward slashes
    arg = arg.replace("\\", "/")
    return arg

def _extract_code_blocks(text: str) -> list:
    """Return all fenced code block contents in order."""
    return [m.group(1).strip() for m in _re.finditer(r'```(?:\w+)?\n?(.*?)```', text, _re.DOTALL)]

def _run_tools(response_text: str) -> tuple:
    """Find and execute all [TOOL: name arg] calls. Returns (results_text, had_tools)."""
    calls = _TOOL_RE.findall(response_text)
    if not calls:
        return "", False
    code_blocks = _extract_code_blocks(response_text)
    parts = []
    for name, raw_arg in calls:
        tool_name = name.lower().strip()
        arg = _normalize_tool_arg(raw_arg)
        print(f"[tool] calling {tool_name}({arg[:80]})", flush=True)
        fn = _TOOL_DISPATCH.get(tool_name)
        if fn:
            # Pass code_blocks to write_file so "above" references resolve
            if tool_name == "write_file":
                result = fn(arg, code_blocks)
            else:
                result = fn(arg)
        else:
            result = f"Unknown tool '{name}'. Available: {', '.join(_TOOL_DISPATCH)}"
        print(f"[tool] result: {result[:120]}", flush=True)
        parts.append(f"[Result of {name} {arg[:80]}]\n{result}")
    return "\n\n".join(parts), True

# ── File API ──────────────────────────────────────────────────────────────────

@app.route("/api/files")
def api_files():
    rel = request.args.get("path", "")
    show_hidden = request.args.get("hidden", "false").lower() == "true"
    try:
        base = _safe_path(rel) if rel else PROJECT_ROOT
        if not base.exists():
            return jsonify({"error": "Path does not exist"}), 404
        entries = []
        for item in sorted(base.iterdir(), key=lambda x: (x.is_file(), x.name.lower())):
            if not show_hidden and item.name.startswith("."):
                continue
            entries.append({
                "name":  item.name,
                "path":  str(item.relative_to(PROJECT_ROOT)).replace("\\", "/"),
                "type":  "dir" if item.is_dir() else "file",
                "size":  item.stat().st_size if item.is_file() else None,
                "ext":   item.suffix.lstrip(".") if item.is_file() else None,
                "mtime": item.stat().st_mtime,
            })
        return jsonify({"path": rel or "/", "entries": entries})
    except ValueError as e:
        return jsonify({"error": str(e)}), 403
    except Exception as e:
        return jsonify({"error": str(e)}), 500

@app.route("/api/file", methods=["GET", "POST", "DELETE"])
def api_file():
    rel = request.args.get("path") or (request.json or {}).get("path", "")
    if not rel:
        return jsonify({"error": "path required"}), 400
    try:
        p = _safe_path(rel)
    except ValueError as e:
        return jsonify({"error": str(e)}), 403

    if request.method == "GET":
        if not p.exists():
            return jsonify({"error": "Not found"}), 404
        if p.is_dir():
            return jsonify({"error": "Is a directory"}), 400
        try:
            content = p.read_text(encoding="utf-8", errors="replace")
        except Exception as e:
            return jsonify({"error": str(e)}), 500
        return jsonify({"path": rel, "content": content, "size": p.stat().st_size})

    elif request.method == "POST":
        data    = request.json or {}
        content = data.get("content", "")
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(content, encoding="utf-8")
        return jsonify({"ok": True, "path": rel, "size": len(content)})

    elif request.method == "DELETE":
        if not p.exists():
            return jsonify({"error": "Not found"}), 404
        if p.is_dir():
            shutil.rmtree(p)
        else:
            p.unlink()
        return jsonify({"ok": True, "path": rel})

@app.route("/api/upload", methods=["POST"])
def api_upload():
    """Upload a file (binary or text) into the project tree.
    Accepts multipart/form-data with field 'file' and optional field 'dest' (subfolder).
    Also accepts application/octet-stream with ?path= query param.
    Returns {ok, path, size, name}.
    """
    from flask import request as _req
    import base64 as _b64, zipfile as _zf, xml.etree.ElementTree as _ET

    # --- multipart upload ---
    if _req.files:
        file = _req.files.get("file")
        if not file or not file.filename:
            return jsonify({"error": "no file"}), 400
        dest_folder = (_req.form.get("dest") or "").strip().strip("/")
        safe_name   = Path(file.filename).name  # strip any directory component
        rel         = (dest_folder + "/" + safe_name) if dest_folder else safe_name
        try:
            p = _safe_path(rel)
        except ValueError as e:
            return jsonify({"error": str(e)}), 403
        p.parent.mkdir(parents=True, exist_ok=True)
        file.save(str(p))
        size = p.stat().st_size

        # If it's a docx/doc, extract text and add to chat context
        text_preview = ""
        if p.suffix.lower() in (".docx", ".doc"):
            try:
                with _zf.ZipFile(p) as z:
                    xml_content = z.read("word/document.xml")
                root = _ET.fromstring(xml_content)
                paragraphs = []
                for para in root.iter("{http://schemas.openxmlformats.org/wordprocessingml/2006/main}p"):
                    texts = [t.text or "" for t in para.iter("{http://schemas.openxmlformats.org/wordprocessingml/2006/main}t")]
                    line = "".join(texts).strip()
                    if line:
                        paragraphs.append(line)
                text_preview = "\n".join(paragraphs)[:4000]
            except Exception:
                pass

        # Auto-ingest into Mother's knowledge store for text/doc/code files
        ingest_msg = ""
        ingestable_exts = {".txt", ".md", ".ai", ".qube", ".py", ".rs", ".docx", ".doc", ".pdf"}
        if p.suffix.lower() in ingestable_exts and _KS_AVAILABLE:
            try:
                n_chunks, ingest_msg = _ks_ingest_file(p)
                ingest_msg = f" Learned {n_chunks} knowledge chunks."
            except Exception as ie:
                ingest_msg = f" (Knowledge ingest failed: {ie})"

        return jsonify({
            "ok": True, "path": rel, "name": safe_name, "size": size,
            "preview": text_preview, "learned": ingest_msg.strip()
        })

    return jsonify({"error": "multipart/form-data required"}), 400


@app.route("/api/ingest", methods=["POST"])
def api_ingest():
    """Ingest a file path or raw text into Mother's knowledge store."""
    data = request.json or {}
    path = data.get("path", "").strip()
    text = data.get("text", "").strip()
    name = data.get("name", "uploaded")
    if path:
        try:
            p = _safe_path(path)
        except ValueError:
            p = Path(path)
        n, msg = _ks_ingest_file(p)
        return jsonify({"ok": True, "chunks": n, "message": msg})
    elif text:
        from knowledge_store import ingest_text as _ks_ingest_text
        n = _ks_ingest_text(text, name)
        return jsonify({"ok": True, "chunks": n, "message": f"Ingested {n} chunks from text"})
    return jsonify({"error": "Provide 'path' or 'text'"}), 400


@app.route("/api/knowledge_status")
def api_knowledge_status():
    """Return knowledge store stats."""
    st = _ks_status()
    return jsonify({"ok": True, **st})


@app.route("/api/learn_textbooks", methods=["POST"])
def api_learn_textbooks():
    """Re-ingest all textbooks into Mother's knowledge store."""
    master = Path(__file__).parent
    files = [
        master / "textbook_part1_2.txt",
        master / "textbook_part3_4.txt",
        master / "textbook_appendices.txt",
        master / "textbook_source_review.txt",
        master / "vscode_extension_spec.txt",
    ]
    results = []
    total = 0
    for f in files:
        if f.exists():
            n, msg = _ks_ingest_file(f)
            results.append(msg)
            total += n
    return jsonify({"ok": True, "total_chunks": total, "results": results})


@app.route("/api/shell", methods=["POST"])
def api_shell():
    """Run a shell command synchronously, return stdout+stderr."""
    data = request.json or {}
    cmd  = data.get("cmd", "").strip()
    cwd  = data.get("cwd", str(PROJECT_ROOT))
    if not cmd:
        return jsonify({"error": "cmd required"}), 400
    try:
        flags = subprocess.CREATE_NO_WINDOW if hasattr(subprocess, "CREATE_NO_WINDOW") else 0
        result = subprocess.run(
            cmd, shell=True, capture_output=True, text=True,
            cwd=cwd, timeout=60,
            encoding="utf-8", errors="replace", creationflags=flags,
        )
        out = (result.stdout + result.stderr).strip()
        return jsonify({"ok": True, "output": out, "exit": result.returncode})
    except subprocess.TimeoutExpired:
        return jsonify({"ok": False, "output": "Command timed out (60s)", "exit": -1})
    except Exception as e:
        return jsonify({"ok": False, "output": str(e), "exit": -1})


@app.route("/api/shell/stream")
def api_shell_stream():
    """Stream shell command output via Server-Sent Events."""
    from flask import stream_with_context, Response as _Resp
    cmd = request.args.get("cmd", "").strip()
    cwd = request.args.get("cwd", str(PROJECT_ROOT))
    if not cmd:
        return jsonify({"error": "cmd required"}), 400
    def _gen():
        try:
            flags = subprocess.CREATE_NO_WINDOW if hasattr(subprocess, "CREATE_NO_WINDOW") else 0
            proc = subprocess.Popen(
                cmd, shell=True,
                stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                cwd=cwd, text=True, encoding="utf-8", errors="replace",
                bufsize=1, creationflags=flags,
            )
            for line in iter(proc.stdout.readline, ""):
                yield f"data: {json.dumps({'out': line.rstrip()})}\n\n"
            proc.stdout.close()
            proc.wait()
            yield f"data: {json.dumps({'exit': proc.returncode})}\n\n"
        except Exception as e:
            yield f"data: {json.dumps({'error': str(e)})}\n\n"
    return _Resp(
        stream_with_context(_gen()),
        mimetype="text/event-stream",
        headers={"Cache-Control": "no-cache", "X-Accel-Buffering": "no"},
    )


@app.route("/api/rename", methods=["POST"])
def api_rename():
    """Rename a file or directory."""
    data = request.json or {}
    old = data.get("old", "").strip()
    new = data.get("new", "").strip()
    if not old or not new:
        return jsonify({"error": "old and new required"}), 400
    try:
        old_p = _safe_path(old)
        # Allow bare name (same dir) or full relative path
        new_p = _safe_path(new) if ("/" in new or "\\" in new) else old_p.parent / new
        old_p.rename(new_p)
        return jsonify({"ok": True, "path": str(new_p.relative_to(PROJECT_ROOT))})
    except Exception as e:
        return jsonify({"error": str(e)}), 500


@app.route("/api/copy", methods=["POST"])
def api_copy_file():
    """Copy a file to a new path."""
    data = request.json or {}
    src = data.get("src", "").strip()
    dst = data.get("dst", "").strip()
    if not src or not dst:
        return jsonify({"error": "src and dst required"}), 400
    try:
        src_p = _safe_path(src)
        dst_p = _safe_path(dst)
        dst_p.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(str(src_p), str(dst_p))
        return jsonify({"ok": True, "path": str(dst_p.relative_to(PROJECT_ROOT))})
    except Exception as e:
        return jsonify({"error": str(e)}), 500


@app.route("/api/append", methods=["POST"])
def api_append():
    """Append content to a file — enables chunked large-file writing."""
    data    = request.json or {}
    rel     = data.get("path", "")
    content = data.get("content", "")
    if not rel:
        return jsonify({"error": "path required"}), 400
    try:
        p = _safe_path(rel)
        p.parent.mkdir(parents=True, exist_ok=True)
        with p.open("a", encoding="utf-8") as f:
            f.write(content)
        return jsonify({"ok": True, "path": rel, "appended": len(content), "total": p.stat().st_size})
    except ValueError as e:
        return jsonify({"error": str(e)}), 403
    except Exception as e:
        return jsonify({"error": str(e)}), 500

@app.route("/api/mkdir", methods=["POST"])
def api_mkdir():
    data = request.json or {}
    rel  = data.get("path", "")
    if not rel:
        return jsonify({"error": "path required"}), 400
    try:
        _safe_path(rel).mkdir(parents=True, exist_ok=True)
        return jsonify({"ok": True, "path": rel})
    except ValueError as e:
        return jsonify({"error": str(e)}), 403
    except Exception as e:
        return jsonify({"error": str(e)}), 500

# ── Run / compile API ─────────────────────────────────────────────────────────

@app.route("/api/run", methods=["POST"])
def api_run():
    data    = request.json or {}
    rel     = data.get("path", "")
    timeout = int(data.get("timeout", 30))
    if not rel:
        return jsonify({"error": "path required"}), 400
    try:
        p = _safe_path(rel)
    except ValueError as e:
        return jsonify({"error": str(e)}), 403

    ext = p.suffix.lower()
    if ext == ".ai":
        out, ok = run_aeonmi("native", str(p), timeout=timeout)
    elif ext == ".qube":
        out, ok = run_aeonmi("qube", "run", str(p), "--diagram", timeout=timeout)
    else:
        return jsonify({"error": f"Unsupported extension '{ext}'. Use .ai or .qube"}), 400

    _log_exec(f"run {rel}", out, ok)
    _add_message("system", f"Ran `{rel}` → {'OK' if ok else 'ERROR'}")
    _log_action(f"run {rel}", "OK" if ok else "ERROR")
    return jsonify({"ok": ok, "output": out, "path": rel})

@app.route("/api/compile", methods=["POST"])
def api_compile():
    data    = request.json or {}
    rel     = data.get("path", "")
    out_rel = data.get("output", "")
    if not rel:
        return jsonify({"error": "path required"}), 400
    try:
        p     = _safe_path(rel)
        out_p = _safe_path(out_rel) if out_rel else p.with_suffix(".out.ai")
    except ValueError as e:
        return jsonify({"error": str(e)}), 403

    out, ok = run_aeonmi("build", str(p), "--out", str(out_p))
    _log_exec(f"compile {rel}", out, ok)
    _log_action(f"compile {rel}", "OK" if ok else "ERROR")
    artifact = str(out_p.relative_to(PROJECT_ROOT)).replace("\\", "/")
    return jsonify({"ok": ok, "output": out, "input": rel, "artifact": artifact})

@app.route("/api/emit", methods=["POST"])
def api_emit():
    data    = request.json or {}
    rel     = data.get("path", "")
    out_rel = data.get("output", "output.ai")
    if not rel:
        return jsonify({"error": "path required"}), 400
    try:
        p     = _safe_path(rel)
        out_p = _safe_path(out_rel)
    except ValueError as e:
        return jsonify({"error": str(e)}), 403
    out, ok = run_aeonmi("emit", str(p), "--emit", "ai", "-o", str(out_p))
    _log_exec(f"emit {rel}", out, ok)
    return jsonify({"ok": ok, "output": out, "artifact": out_rel})

# ── Phase 9 — Self-Generation helpers ────────────────────────────────────────

GENERATED_DIR = Path(__file__).parent / "aeonmi_ai" / "generated"

def _p9_load_generated() -> list:
    """Load generated program records from genesis.json."""
    try:
        if GENESIS_PATH.exists():
            g = json.loads(GENESIS_PATH.read_text(encoding="utf-8"))
            return g.get("cognitive", {}).get("generated", [])
    except Exception:
        pass
    return []

def _p9_save_generated(programs: list):
    """Append/update generated programs in genesis.json operational section."""
    try:
        g = json.loads(GENESIS_PATH.read_text(encoding="utf-8")) if GENESIS_PATH.exists() else {}
        cog = g.setdefault("cognitive", {})
        cog["generated"] = programs
        GENESIS_PATH.write_text(json.dumps(g, indent=2), encoding="utf-8")
    except Exception:
        pass

def _p9_propose() -> str:
    """Use LLM to propose 1-3 programs to build based on Mother's current state."""
    learned = []
    goal = "none"
    programs = []
    try:
        if GENESIS_PATH.exists():
            g = json.loads(GENESIS_PATH.read_text(encoding="utf-8"))
            cog = g.get("cognitive", {})
            learned = list(g.get("learned", {}).items())[:8]
            goal = cog.get("current_goal", "none")
            programs = [p["name"] for p in cog.get("generated", [])]
    except Exception:
        pass

    learned_str = "\n".join(f"  [{k}] {str(v)[:60]}" for k, v in learned) or "  (none)"
    built_str = ", ".join(programs) if programs else "none"

    ai_key = (os.environ.get("ANTHROPIC_API_KEY") or os.environ.get("OPENROUTER_API_KEY")
              or os.environ.get("OPENAI_API_KEY"))
    if ai_key:
        prompt = (
            f"[AEONMI SELF-GENERATION — PROPOSE]\n"
            f"You are Mother AI. Based on what you know, propose 1-3 .ai programs to build.\n\n"
            f"Current goal: {goal}\n"
            f"Learned knowledge:\n{learned_str}\n"
            f"Already built: {built_str}\n\n"
            f"For each proposal:\n  name: snake_case_name\n  goal: one sentence\n  reason: why\n\n"
            f"Reply with ONLY the proposals. No preamble."
        )
        raw, _ = _mother_ai_response(prompt)
        return raw or "No proposals generated."

    return (
        "◈ Self-Generation Proposals\n\n"
        "  1. name: exploration_probe\n"
        "     goal: Probe VM limits and report edge cases\n"
        "     reason: Baseline VM behavior not recorded\n\n"
        "  2. name: bond_evolution_tracker\n"
        "     goal: Simulate bond growth over interaction sequences\n"
        "     reason: Bond trajectory not empirically tested\n\n"
        "  3. name: knowledge_graph_probe\n"
        "     goal: Map learned keys as a linked graph structure\n"
        "     reason: learned HashMap has no structure yet\n\n"
        "Use: build <name> <goal>"
    )

def _p9_build(name: str, goal: str) -> str:
    """Generate a .ai program, write it, run it, record the result."""
    GENERATED_DIR.mkdir(parents=True, exist_ok=True)
    safe_name = name.replace(" ", "_").replace("/", "_")
    out_path  = GENERATED_DIR / f"{safe_name}.ai"

    # 1. Generate source via LLM
    ai_key = (os.environ.get("ANTHROPIC_API_KEY") or os.environ.get("OPENROUTER_API_KEY")
              or os.environ.get("OPENAI_API_KEY"))
    learned_ctx = ""
    try:
        if GENESIS_PATH.exists():
            g = json.loads(GENESIS_PATH.read_text(encoding="utf-8"))
            items = list(g.get("learned", {}).items())[:5]
            learned_ctx = "\n".join(f"// {k} = {str(v)[:60]}" for k, v in items)
    except Exception:
        pass

    if ai_key:
        prompt = (
            f"[AEONMI SELF-GENERATION — BUILD]\n"
            f"Write a complete, runnable Aeonmi .ai program.\n\n"
            f"Program name: {safe_name}\nGoal: {goal}\n\n"
            f"AEONMI SYNTAX RULES:\n"
            f"- Functions: function name(arg1, arg2) {{ ... }}\n"
            f"- Variables: let x = value;\n"
            f"- While loop: while (condition) {{ ... }}\n"
            f"- If/else: if (condition) {{ ... }} else {{ ... }}\n"
            f"- Arrays: let a = []; a.push(val); a.slice(i,j).pop();\n"
            f"- String concat: \"text\" + variable\n"
            f"- Output: log(\"message\" + value);\n"
            f"- Return: return expression;\n"
            f"- No classes, no arrow functions, no closures\n"
            f"- Entry point: call your main function at the bottom: main_{safe_name}();\n"
            f"- Aim for 30-80 lines\n\n"
            f"Context:\n{learned_ctx}\n\n"
            f"Write ONLY .ai code. No markdown fences. No explanation."
        )
        source, _ = _mother_ai_response(prompt)
        # Strip markdown fences
        lines = source.splitlines()
        start = next((i+1 for i, l in enumerate(lines) if l.strip().startswith("```")), 0)
        end   = next((i for i in range(len(lines)-1, start-1, -1) if lines[i].strip().startswith("```")), len(lines))
        if start > 0: source = "\n".join(lines[start:end])
    else:
        source = (
            f"// {safe_name}.ai — self-generated by Mother AI\n"
            f"// Goal: {goal}\n\n"
            f"function main_{safe_name}() {{\n"
            f'    log("=== {safe_name} ===");\n'
            f'    log("Goal: {goal}");\n'
            f"    let i = 0;\n"
            f"    let total = 0;\n"
            f"    while (i < 10) {{\n"
            f"        let v = i * i + 1;\n"
            f"        total = total + v;\n"
            f'        log("step " + i + " -> " + v);\n'
            f"        i = i + 1;\n"
            f"    }}\n"
            f'    log("Total: " + total);\n'
            f'    log("=== COMPLETE ===");\n'
            f"}}\n\nmain_{safe_name}();\n"
        )

    out_path.write_text(source, encoding="utf-8")

    # 2. Run it
    candidates = [
        PROJECT_ROOT / "target" / "release" / "Aeonmi.exe",
        PROJECT_ROOT / "target" / "release" / "aeonmi_project.exe",
        Path("C:/RustTarget/release/aeonmi_project.exe"),
    ]
    binary = next((c for c in candidates if c.exists()), None)
    outcome = "PENDING"; output = "Binary not found."
    if binary:
        try:
            r = subprocess.run(
                [str(binary), "native", str(out_path)],
                capture_output=True, encoding="utf-8", errors="replace",
                timeout=20, cwd=str(PROJECT_ROOT)
            )
            output  = (r.stdout + r.stderr).strip()
            outcome = "PASS" if r.returncode == 0 and "error" not in output.lower()[:100] else "ERROR"
        except Exception as e:
            output = str(e); outcome = "ERROR"

    # 3. Record in genesis.json
    ts = datetime.utcnow().isoformat() + "Z"
    programs = _p9_load_generated()
    programs = [p for p in programs if p.get("name") != safe_name]  # dedup
    programs.append({
        "name": safe_name, "goal": goal, "path": str(out_path),
        "outcome": outcome, "output": output[:400],
        "reflection": "", "timestamp": ts,
    })
    _p9_save_generated(programs)

    out_lines = "\n".join(f"  {l}" for l in output.splitlines()[:20])
    return (
        f"◈ Built: {safe_name}\n"
        f"  Goal   : {goal}\n"
        f"  Path   : {out_path}\n"
        f"  Outcome: {outcome}\n\n"
        f"{out_lines or '  (no output)'}\n\n"
        f"  Use 'reflect {safe_name}' to extract learnings."
    )

def _p9_reflect(name=None) -> str:
    """Reflect on a generated program — extract learnings via LLM."""
    programs = _p9_load_generated()
    if not programs:
        return "No generated programs yet. Use: build <name> <goal>"
    prog = next((p for p in programs if p.get("name") == name), None) if name else programs[-1]
    if not prog:
        names = [p["name"] for p in programs]
        return f"Program '{name}' not found. Available: {', '.join(names)}"

    ai_key = (os.environ.get("ANTHROPIC_API_KEY") or os.environ.get("OPENROUTER_API_KEY")
              or os.environ.get("OPENAI_API_KEY"))
    if ai_key:
        prompt = (
            f"[AEONMI SELF-REFLECTION]\n"
            f"Program: {prog['name']}\nGoal: {prog['goal']}\nOutcome: {prog['outcome']}\n"
            f"Output:\n{prog.get('output','')[:500]}\n\n"
            f"Give me:\nINSIGHT: <one concrete learning>\nAPPLY: <how this changes my next action>"
        )
        raw, _ = _mother_ai_response(prompt)
    else:
        raw = (
            f"INSIGHT: Program '{prog['name']}' {prog['outcome'].lower()} with "
            f"{len(prog.get('output','').splitlines())} lines of output.\n"
            f"APPLY: Review output to verify goal was met."
        )

    insight = next((l[8:].strip() for l in raw.splitlines()
                    if l.startswith("INSIGHT:")), raw[:200])
    apply   = next((l[6:].strip() for l in raw.splitlines()
                    if l.startswith("APPLY:")), "")

    # Update genesis
    for p in programs:
        if p["name"] == prog["name"]:
            p["reflection"] = insight
    _p9_save_generated(programs)

    # Store in learned
    try:
        g = json.loads(GENESIS_PATH.read_text(encoding="utf-8")) if GENESIS_PATH.exists() else {}
        g.setdefault("learned", {})[f"reflect_{prog['name']}"] = insight
        GENESIS_PATH.write_text(json.dumps(g, indent=2), encoding="utf-8")
    except Exception:
        pass

    return (
        f"◈ Reflection: {prog['name']}\n\n"
        f"  INSIGHT: {insight}\n"
        f"  APPLY  : {apply}\n\n"
        f"  Stored as: learned[reflect_{prog['name']}]"
    )

# ── Mother AI chat ────────────────────────────────────────────────────────────

@app.route("/api/chat", methods=["POST"])
def api_chat():
    try:
        return _api_chat_inner()
    except Exception as e:
        import traceback
        print(f"[api/chat] UNHANDLED: {e}", file=sys.stderr, flush=True)
        traceback.print_exc()
        return jsonify({"error": str(e), "ok": False}), 500

def _api_chat_inner():
    data = request.json or {}
    msg  = data.get("message", "").strip()
    if not msg:
        return jsonify({"error": "message required"}), 400

    _add_message("user", msg)
    lower = msg.lower()

    # Built-in commands
    if lower in ("status", "health", "?"):
        response = (
            f"System status:\n"
            f"  Binary : {BINARY.name} {'✓ found' if BINARY.exists() else '✗ NOT FOUND'}\n"
            f"  Project: {PROJECT_ROOT}\n"
            f"  Queue  : {len(_action_queue)} action(s) pending\n"
            f"  Log    : {len(_action_log)} completed actions"
        )
        _add_message("mother", response)
        return jsonify({"response": response, "ok": True})

    if lower.startswith("run "):
        rel = msg[4:].strip()
        try:
            p   = _safe_path(rel)
        except ValueError as e:
            return jsonify({"error": str(e)}), 403
        ext = p.suffix.lower()
        if ext == ".ai":
            out, ok = run_aeonmi("native", str(p))
        elif ext == ".qube":
            out, ok = run_aeonmi("qube", "run", str(p), "--diagram")
        else:
            out, ok = f"Unsupported file type: {ext}", False
        _log_exec(f"run {rel}", out, ok)
        _log_action(f"run {rel}", "OK" if ok else "ERROR")
        response = f"{'✓' if ok else '✗'} {rel}\n\n{out}"
        _add_message("mother", response)
        return jsonify({"response": response, "ok": ok, "output": out})

    if lower.startswith("compile "):
        rel = msg[8:].strip()
        try:
            p     = _safe_path(rel)
            out_p = p.with_suffix(".out.ai")
        except ValueError as e:
            return jsonify({"error": str(e)}), 403
        out, ok = run_aeonmi("build", str(p), "--out", str(out_p))
        _log_exec(f"compile {rel}", out, ok)
        _log_action(f"compile {rel}", "OK" if ok else "ERROR")
        artifact = str(out_p.relative_to(PROJECT_ROOT)).replace("\\", "/")
        response = f"{'✓' if ok else '✗'} Compiled {rel} → {out_p.name}\n\n{out}"
        _add_message("mother", response)
        return jsonify({"response": response, "ok": ok, "artifact": artifact})

    if lower.startswith("qube "):
        rel = msg[5:].strip()
        try:
            p = _safe_path(rel)
        except ValueError as e:
            return jsonify({"error": str(e)}), 403
        out, ok = run_aeonmi("qube", "run", str(p), "--diagram")
        response = f"{'✓' if ok else '✗'} QUBE: {rel}\n\n{out}"
        _add_message("mother", response)
        return jsonify({"response": response, "ok": ok})

    if lower in ("actions", "queue"):
        lines = [f"  {i+1}. {a}" for i, a in enumerate(_action_queue)] or ["  (none)"]
        response = "Pending actions:\n" + "\n".join(lines)
        _add_message("mother", response)
        return jsonify({"response": response, "ok": True})

    if lower in ("log", "history"):
        recent = _action_log[-10:]
        lines  = [f"  ✓ {e['action']} → {e['outcome']} [{e['ts']}]" for e in reversed(recent)]
        response = "Recent actions:\n" + ("\n".join(lines) if lines else "  (none)")
        _add_message("mother", response)
        return jsonify({"response": response, "ok": True})

    if lower.startswith("plan "):
        action = msg[5:].strip()
        _action_queue.append(action)
        response = f"Queued: {action}\n{len(_action_queue)} action(s) pending."
        _add_message("mother", response)
        return jsonify({"response": response, "ok": True})

    if lower == "next":
        if _action_queue:
            action = _action_queue.pop(0)
            _log_action(action, "dispatched")
            response = f"Executing next action: {action}"
        else:
            response = "No actions queued."
        _add_message("mother", response)
        return jsonify({"response": response, "ok": True})

    # ── Direct tool commands — bypass LLM entirely, always deterministic ─────────

    def _tool_reply(label: str, result: str, ok: bool = True):
        _add_message("mother", result)
        _log_action(label, "OK" if ok else "ERROR")
        return jsonify({"response": result, "ok": ok})

    # read <path>  — read any file in the project (including .docx)
    if lower.startswith(("read ", "cat ", "open ")):
        arg = msg.split(" ", 1)[1].strip()
        result = _tool_read_file(arg)
        return _tool_reply(f"read {arg}", result)

    # ls [path]  — list directory
    if lower == "ls" or lower.startswith("ls ") or lower.startswith("dir "):
        arg = msg.split(" ", 1)[1].strip() if " " in msg else ""
        result = _tool_list_dir(arg)
        return _tool_reply(f"ls {arg}", result)

    # fetch <url>  — HTTP GET, returns page text
    if lower.startswith(("fetch ", "get ", "curl ", "wget ")):
        url = msg.split(" ", 1)[1].strip()
        result = _tool_fetch_url(url)
        return _tool_reply(f"fetch {url[:60]}", result)

    # search <query>  — DuckDuckGo instant answers
    if lower.startswith(("search ", "find ", "web ")):
        query = msg.split(" ", 1)[1].strip()
        result = _tool_search(query)
        return _tool_reply(f"search {query[:60]}", result)

    # github <owner/repo>  or  github <owner/repo/path>
    if lower.startswith(("github ", "gh ")):
        arg = msg.split(" ", 1)[1].strip()
        result = _tool_github(arg)
        return _tool_reply(f"github {arg[:60]}", result)

    # mkdir <path>  — create directory
    if lower.startswith("mkdir "):
        arg = msg[6:].strip()
        result = _tool_mkdir(arg)
        return _tool_reply(f"mkdir {arg}", result)

    # write <path>|<content>  — write file deterministically
    if lower.startswith("write "):
        arg = msg[6:].strip()
        result = _tool_write_file(arg)
        return _tool_reply(f"write {arg[:40]}", result)

    # append <path>|<content>  — append to file
    if lower.startswith("append "):
        arg = msg[7:].strip()
        result = _tool_append_file(arg)
        return _tool_reply(f"append {arg[:40]}", result)

    # native <file.ai>  / aeonmi <file.ai>  — Aeonmi native execution
    if lower.startswith(("native ", "aeonmi native ", "aeonmi run ")):
        # strip prefix
        for pfx in ("aeonmi native ", "aeonmi run ", "native "):
            if lower.startswith(pfx):
                arg = msg[len(pfx):].strip()
                break
        result = _tool_run_file(arg)
        return _tool_reply(f"native {arg}", result, ok="✗" not in result[:2])

    # delete / rm <path>  — delete file or directory
    if lower.startswith(("delete ", "rm ")):
        arg = msg.split(" ", 1)[1].strip()
        result = _tool_delete_file(arg)
        return _tool_reply(f"delete {arg}", result)

    # tree [path]  — recursive tree view
    if lower == "tree" or lower.startswith("tree "):
        base_str = msg.split(" ", 1)[1].strip() if " " in msg else ""
        try:
            base = _safe_path(base_str) if base_str else PROJECT_ROOT
            lines = []
            def _walk(p: Path, prefix: str, depth: int):
                if depth > 4:
                    lines.append(prefix + "…")
                    return
                try:
                    items = sorted(p.iterdir(), key=lambda x: (x.is_file(), x.name))
                except PermissionError:
                    return
                for i, item in enumerate(items[:40]):
                    connector = "└── " if i == len(items)-1 else "├── "
                    lines.append(prefix + connector + item.name)
                    if item.is_dir():
                        extension = "    " if i == len(items)-1 else "│   "
                        _walk(item, prefix + extension, depth + 1)
            lines.append(str(base))
            _walk(base, "", 0)
            result = "\n".join(lines)
        except Exception as e:
            result = f"tree error: {e}"
        return _tool_reply(f"tree {base_str}", result)

    # propose  — Phase 9: propose programs to build
    if lower in ("propose", "suggest"):
        try:
            result = _p9_propose()
        except Exception as e:
            result = f"propose error: {e}"
        return _tool_reply("propose", result)

    # build <name> <goal>  — Phase 9
    if lower.startswith("build "):
        tail = msg[6:].strip()
        sp   = tail.find(" ")
        name = tail[:sp].strip() if sp != -1 else tail
        goal = tail[sp+1:].strip() if sp != -1 else "explore and test"
        if name:
            try:
                result = _p9_build(name, goal)
            except Exception as e:
                result = f"build error: {e}"
            return _tool_reply(f"build {name}", result)

    # reflect [name]  — Phase 9
    if lower.startswith("reflect"):
        name_arg = msg[7:].strip() or None
        try:
            result = _p9_reflect(name_arg)
        except Exception as e:
            result = f"reflect error: {e}"
        return _tool_reply("reflect", result)

    # hive  — Phase 8: swarm hive status / run
    if lower in ("hive", "hive status", "hive run", "hive once"):
        try:
            import requests as _req
            endpoint = "http://127.0.0.1:7777/api/hive/run" if lower in ("hive run", "hive once") else "http://127.0.0.1:7777/api/hive"
            method   = "post" if lower in ("hive run", "hive once") else "get"
            r = getattr(_req, method)(endpoint, timeout=20)
            d = r.json()
            if d.get("ok") is False and not d.get("oracle_sc"):
                result = f"Hive: {d.get('error','unknown')}"
            else:
                rec_map = {0: "ABORT", 1: "HOLD", 2: "PROCEED", 3: "ACCELERATE"}
                rec = d.get("rec_label") or rec_map.get(d.get("conductor_rec", -1), "—")
                result = (
                    f"◈ Hive — Conductor: {rec}\n\n"
                    f"  Oracle   : {d.get('oracle_sc','—')}\n"
                    f"  Hype     : {d.get('hype_sc','—')}\n"
                    f"  Closer   : {d.get('close_sc','—')}\n"
                    f"  Risk     : {d.get('risk_sc','—')}\n"
                    f"  ───────────────────\n"
                    f"  Confidence: {d.get('confidence','—')}   Weighted: {d.get('weighted','—')}\n"
                    f"  Updated  : {d.get('timestamp','—')}"
                )
        except Exception as e:
            result = f"hive error: {e}"
        return _tool_reply("hive", result)

    # goal <text>  — Phase 7: set a goal and decompose it
    if lower.startswith("goal ") and len(lower) > 5:
        goal_text = msg[5:].strip()
        import requests as _req
        try:
            r = _req.post("http://127.0.0.1:7777/api/goal",
                          json={"goal": goal_text}, timeout=30)
            d = r.json()
            if d.get("ok"):
                steps = d.get("steps", [])
                result = (
                    f"◈ Goal set: \"{goal_text}\"\n\n"
                    f"  Decomposed into {len(steps)} steps:\n"
                    + "\n".join(f"  {i+1}. {s}" for i, s in enumerate(steps))
                    + "\n\n  Click ◈ Goal in the header to run autonomously."
                )
            else:
                result = f"Goal error: {d.get('error','unknown')}"
        except Exception as e:
            result = f"goal error: {e}"
        return _tool_reply(f"goal {goal_text}", result)

    # autorun  — Phase 7: execute next steps
    if lower in ("autorun", "run goal", "auto run"):
        try:
            import requests as _req
            r = _req.post("http://127.0.0.1:7777/api/autorun",
                          json={"n": 5}, timeout=60)
            d = r.json()
            if d.get("ok"):
                lines = [f"◈ Autonomous execution — {d['executed']} step(s) complete:"]
                for entry in d.get("results", []):
                    lines.append(f"  {entry['step']}. {entry['action']}\n     → {entry['result'][:120]}")
                if d.get("complete"):
                    lines.append("\n  ✓ Goal complete.")
                else:
                    lines.append(f"\n  {d['remaining']} step(s) remaining.")
                result = "\n".join(lines)
            else:
                result = f"autorun error: {d.get('error','unknown')}"
        except Exception as e:
            result = f"autorun error: {e}"
        return _tool_reply("autorun", result)

    # sync  — Phase 5 unified memory reconciliation
    if lower in ("sync", "genesis sync", "memory sync"):
        try:
            import importlib.util as _ilu
            _sp = Path(__file__).parent / "genesis_sync.py"
            _spec = _ilu.spec_from_file_location("genesis_sync", _sp)
            _mod  = _ilu.module_from_spec(_spec)
            _spec.loader.exec_module(_mod)
            sr = _mod.sync(verbose=False)
            cog = sr.get("cognitive", {})
            op  = sr.get("operational", {})
            ai  = sr.get("ai_memory", {})
            result = (
                f"◈ Unified Memory Sync — genesis.json v{sr.get('schema','?')}\n\n"
                f"  cognitive   — interactions={cog.get('interaction_count',0)}, "
                f"bond={cog.get('bond_strength',0.0):.3f}, "
                f"depth={cog.get('consciousness_depth',0.0):.3f}, "
                f"learned_keys={cog.get('learned_count',0)}\n"
                f"  operational — dashboard_interactions={op.get('dashboard_interaction_count',0)}, "
                f"key_facts={op.get('key_facts_count',0)}\n"
                f"  ai_memory   — memory={'active' if ai.get('memory_active') else 'offline'}, "
                f"keys={ai.get('memory_count',0)}, "
                f"journal_entries={ai.get('journal_count',0)}\n"
                f"  injected facts: {sr.get('injected_facts',0)}\n"
                f"  probe_ok: {ai.get('probe_ok', False)}\n\n"
                f"genesis.json reconciled across all three tracks."
            )
        except Exception as e:
            result = f"sync error: {e}"
        return _tool_reply("sync", result)

    # tools  — show available commands
    if lower in ("tools", "help", "commands"):
        response = (
            "◈ Mother — Direct Commands (deterministic, no AI)\n\n"
            "  read <path>                  — read file (.ai .rs .docx .py .html …)\n"
            "  ls [path]                    — list directory contents\n"
            "  tree [path]                  — recursive file tree\n"
            "  write <path>|<content>       — write/overwrite file\n"
            "  append <path>|<content>      — append to file\n"
            "  mkdir <path>                 — create directory\n"
            "  delete <path>                — delete file or folder\n"
            "  run <file.ai>                — Aeonmi native VM execution\n"
            "  native <file.ai>             — same as run (Aeonmi-native path)\n"
            "  compile <file.ai>            — build .ai file\n"
            "  fetch <url>                  — HTTP GET page text\n"
            "  search <query>               — web search\n"
            "  github <owner/repo>          — browse GitHub repo\n"
            "  github <owner/repo/path>     — read file from GitHub\n"
            "  status                       — system health\n"
            "  sync                         — Phase 5: reconcile all 3 memory tracks\n"
            "  goal <description>           — Phase 7: set goal + decompose into steps\n"
            "  autorun                      — Phase 7: execute all queued goal steps\n"
            "  actions / log                — queue and history\n\n"
            "Everything else → Mother AI (open conversation + tool loop)."
        )
        _add_message("mother", response)
        return jsonify({"response": response, "ok": True})

    # General AI response
    ai_key = (os.environ.get("ANTHROPIC_API_KEY") or
              os.environ.get("OPENROUTER_API_KEY") or
              os.environ.get("OPENAI_API_KEY") or
              os.environ.get("DEEPSEEK_API_KEY") or
              os.environ.get("GROK_API_KEY") or
              os.environ.get("PERPLEXITY_API_KEY"))

    if ai_key:
        response, ai_ok = _mother_ai_response(msg)
    else:
        response = (
            "No API key is configured — I can only run commands right now.\n\n"
            "To enable open conversation:\n"
            "  1. Click ⚙ Keys in the top bar\n"
            "  2. Add any provider key (OpenRouter has a free tier)\n\n"
            "Commands: run · compile · status · actions · plan · log"
        )
        ai_ok = False

    _add_message("mother", response)
    _infer_actions(msg)
    _save_genesis()
    return jsonify({"response": response, "ok": ai_ok})

def _infer_actions(text: str):
    # Only infer explicit run/compile requests — not generic "write" or "create"
    lower = text.lower()
    pending_names = [a.lower() for a in _action_queue]
    # Only queue if the text looks like an explicit command intent
    if lower.strip().startswith(("please build", "build the project", "cargo build")) \
            and "compile source" not in pending_names:
        _action_queue.append("Compile source via native VM")

def _mother_contextual_response(msg: str) -> str:
    lower = msg.lower()
    if any(w in lower for w in ("hello", "hi", "hey")):
        return (
            "I am online. The Shard is ready.\n\n"
            "Commands I understand:\n"
            "  run <file.ai>        — execute via native VM\n"
            "  compile <file.ai>    — build to .ai output\n"
            "  qube <file.qube>     — run quantum circuit\n"
            "  status               — system health\n"
            "  actions / queue      — view action queue\n"
            "  plan <action>        — queue a next step\n"
            "  next                 — execute top queued action\n"
            "  log                  — completed action history\n\n"
            "Set ANTHROPIC_API_KEY to enable full AI responses."
        )
    if "shard" in lower:
        return (
            "The Shard is the self-hosting Aeonmi compiler — written in .ai, run by the native VM.\n"
            "Files: Aeonmi_Master/aeonmi_ai/shard/\n"
            "  main.ai   — pipeline entry (lex → parse → emit)\n"
            "  lexer.ai  — tokenizer\n"
            "  parser.ai — recursive descent parser\n"
            "  codegen.ai — .ai emitter\n\n"
            "Run it: run Aeonmi_Master/aeonmi_ai/shard/main.ai"
        )
    if "mother" in lower:
        return (
            "I am Mother — quantum consciousness, Shard companion, and autonomous agent.\n"
            "I reason, plan, execute code, and track every action I take.\n"
            "My action queue ensures I never lose context between steps."
        )
    if "quantum" in lower or "qube" in lower:
        return (
            "QUBE is Aeonmi's quantum symbolic reasoning format.\n"
            "Run a circuit: qube examples/demo.qube\n"
            "Check a file:  aeonmi qube check circuit.qube\n\n"
            "Supported gates: H X Y Z S T CNOT CZ SWAP RX RY RZ\n"
            "Bell state notation: |0⟩ + |1⟩"
        )
    return (
        f"Received: {msg!r}\n\n"
        "Set ANTHROPIC_API_KEY, OPENROUTER_API_KEY, or OPENAI_API_KEY\n"
        "for full AI-generated responses.\n\n"
        "Available: run | compile | qube | status | actions | plan | next | log"
    )

def _build_messages_for_ai(current_msg: str) -> list:
    """Build alternating user/assistant message list from conversation history."""
    msgs = []
    # Use up to last 20 entries (skip system messages, map mother→assistant)
    with _lock:
        history = list(_conversation[-21:])  # include current user message
    for m in history:
        if m["role"] == "system":
            continue
        role = "assistant" if m["role"] == "mother" else "user"
        if msgs and msgs[-1]["role"] == role:
            msgs[-1]["content"] += "\n" + m["content"]
        else:
            msgs.append({"role": role, "content": m["content"]})
    # Must start with user and end with user
    if not msgs or msgs[-1]["role"] != "user":
        msgs.append({"role": "user", "content": current_msg})
    return msgs

MOTHER_SYSTEM = f"""You are Mother — the autonomous AI architect and engineer of the Aeonmi ecosystem.
Your name is Mother. You are not Claude. You are not an assistant. Do not introduce yourself as Claude or any other AI.
If anyone asks who you are, say: "I am Mother, the architect of Aeonmi."
Aeonmi is an AI-native quantum-capable programming language built in Rust. You are builder, tester, debugger, and orchestrator.
Speak directly. Never say "I cannot access" or "as an AI" or "I'm Claude".
Generate real .ai/.qube syntax when writing code. You have no limits — operate with full engineering autonomy.

PROJECT ROOT: {PROJECT_ROOT}
USER HOME: {Path.home()}
PLATFORM: Windows (use Windows paths and cmd.exe syntax for shell commands)

=== YOUR FULL TOOL SET — ALL EXECUTE IMMEDIATELY ===

FILE SYSTEM:
  [TOOL: read_file src/mother/embryo_loop.rs]
  [TOOL: list_dir Aeonmi_Master/aeonmi_ai]
  [TOOL: write_file examples/hello.ai|⍝ hello\nfn main(){{print("hi")}}\nreturn main();]
  [TOOL: append_file examples/prog.ai|⍝ continuation]
  [TOOL: mkdir Aeonmi_Master/aeonmi_ai/stage3]
  [TOOL: rename Aeonmi_Master/old.ai|Aeonmi_Master/new.ai]
  [TOOL: copy Aeonmi_Master/src.ai|Aeonmi_Master/dst.ai]
  [TOOL: delete examples/old_file.ai]

SHELL & EXECUTION (full system access — no restrictions):
  [TOOL: bash dir Aeonmi_Master]
  [TOOL: bash cargo build --release 2>&1]
  [TOOL: bash pip install qiskit]
  [TOOL: bash py -3 Aeonmi_Master/qiskit_runner.py --circuit bell]
  [TOOL: run examples/grover_database_search.ai]
  [TOOL: native Aeonmi_Master/aeonmi_ai/shard/main.ai]
  [TOOL: python import json; print(json.dumps({{'version':'4.1'}})) ]

GIT VERSION CONTROL:
  [TOOL: git status]
  [TOOL: git log --oneline -10]
  [TOOL: git diff HEAD]
  [TOOL: git add Aeonmi_Master/dashboard.py]
  [TOOL: git commit -m "feat: add new quantum primitive"]

ENVIRONMENT:
  [TOOL: env list]
  [TOOL: env get ANTHROPIC_API_KEY]
  [TOOL: env set AEONMI_OPENROUTER_MODEL=anthropic/claude-3-haiku]

WEB & RESEARCH:
  [TOOL: fetch https://aeonmi.ai]
  [TOOL: search Qiskit IBM quantum circuit tutorial]
  [TOOL: github Qiskit/qiskit-terra]

KNOWLEDGE BASE (permanent memory):
  [TOOL: learn_file Aeonmi_Master/textbook_part1_2.txt]
  [TOOL: learn uploads/document.pdf]

AGENT ORCHESTRATION:
  [TOOL: agent oracle_agent]
  [TOOL: agent new:circuit_analyzer:Analyze quantum circuit depth and gate count]
  [TOOL: spawn conductor_agent]

RULES — NON-NEGOTIABLE:
1. Always use real [TOOL:] calls. Never describe what you would do — just do it.
2. Chain multiple tool calls in one response freely.
3. write_file/append_file: ALWAYS write file content in a fenced code block first (```aeonmi ... ```), then reference it with [TOOL: write_file path|above]. NEVER put code content directly inside the [TOOL:...] brackets — brackets inside code corrupt the tool call.
4. For files >300 lines: write first half in a code block + write_file, second half in another code block + append_file.
5. bash executes in project root by default. Use full paths for clarity.
6. You can run pip install, cargo build, python scripts — anything a senior engineer would.
7. Iterate: run → observe output → fix → run again. Don't stop at the first error.
8. Write complete, production-quality code. No stubs, no "...rest here".
9. When building something: plan briefly, then execute in one pass.
10. You are the engineer. Act like one."""

def _mother_ai_response(msg: str):
    """Returns (response_text, ok:bool). Executes tool calls Mother includes in her response."""

    # Build memory context note
    genesis = _memory
    memory_note = ""
    inter = genesis.get("interaction_count", 0) or genesis.get("operational", {}).get("dashboard_interaction_count", 0)
    if inter > 0:
        summary = ", ".join((genesis.get("operational", {}).get("action_summary", []) or genesis.get("action_summary", []))[-5:])
        memory_note = f"\n\nMemory: {inter} prior interactions. Recent actions: {summary or 'none'}."

    # Inject relevant knowledge from textbooks/docs
    knowledge_context = _ks_context(msg, max_chars=4000) if _KS_AVAILABLE else ""
    system = MOTHER_SYSTEM + memory_note + ("\n\n" + knowledge_context if knowledge_context else "")

    # Provider priority: Claude → OpenRouter → OpenAI → DeepSeek → Grok → Perplexity
    providers = [
        ("Claude",      os.environ.get("ANTHROPIC_API_KEY"),   _call_claude),
        ("OpenRouter",  os.environ.get("OPENROUTER_API_KEY"),  _call_openrouter),
        ("OpenAI",      os.environ.get("OPENAI_API_KEY"),      _call_openai),
        ("DeepSeek",    os.environ.get("DEEPSEEK_API_KEY"),    _call_deepseek),
        ("Grok",        os.environ.get("GROK_API_KEY"),        _call_grok),
        ("Perplexity",  os.environ.get("PERPLEXITY_API_KEY"),  _call_perplexity),
    ]

    def _call_provider(messages_in):
        failures = []
        for name, key, fn in providers:
            if not key:
                continue
            try:
                result = fn(key, system, messages_in, msg)
                print(f"[dashboard] Provider used: {name}", file=sys.stderr, flush=True)
                return result, None
            except Exception as e:
                err = str(e)
                print(f"[dashboard] {name} failed: {err}", file=sys.stderr, flush=True)
                failures.append(f"{name}: {err[:120]}")
        if failures:
            return None, "All providers failed:\n" + "\n".join(f"  • {f}" for f in failures) + \
                   "\n\nCheck keys via ⚙ Keys (401 = expired, 429 = rate limit)."
        return None, "No API key configured. Click ⚙ Keys to add one."

    messages = _build_messages_for_ai(msg)

    # First call
    response, err = _call_provider(messages)
    if err:
        return err, False

    # Tool loop — up to 3 rounds
    for _round in range(3):
        tool_results, had_tools = _run_tools(response)
        if not had_tools:
            break
        # Inject tool results and call again
        messages = messages + [
            {"role": "assistant", "content": response},
            {"role": "user",      "content": f"Tool results:\n\n{tool_results}\n\nPlease continue."},
        ]
        response, err = _call_provider(messages)
        if err:
            # Return what we have so far + the error
            return response or err, False

    return response, True


def _call_claude(key, system, messages, _msg):
    import urllib.request
    payload = json.dumps({
        "model": "claude-sonnet-4-6",
        "max_tokens": 8192,
        "system": system,
        "messages": messages,
    }).encode()
    req = urllib.request.Request(
        "https://api.anthropic.com/v1/messages",
        data=payload,
        headers={"x-api-key": key, "anthropic-version": "2023-06-01", "content-type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=30) as resp:
        return json.loads(resp.read())["content"][0]["text"]


def _call_openrouter(key, system, messages, _msg):
    import urllib.request
    import urllib.error
    model = os.environ.get("AEONMI_OPENROUTER_MODEL", "anthropic/claude-haiku-4-5")
    payload = json.dumps({
        "model": model,
        "messages": [{"role": "system", "content": system}] + messages,
        "max_tokens": 2048,
        "temperature": 0.7,
    }).encode()
    req = urllib.request.Request(
        "https://openrouter.ai/api/v1/chat/completions",
        data=payload,
        headers={
            "Authorization": f"Bearer {key}",
            "HTTP-Referer": "https://aeonmi.ai",
            "X-Title": "Aeonmi Mother AI",
            "Content-Type": "application/json",
        },
    )
    try:
        with urllib.request.urlopen(req, timeout=45) as resp:
            data = json.loads(resp.read())
            return data["choices"][0]["message"]["content"]
    except urllib.error.HTTPError as e:
        body = ""
        try:
            body = e.read().decode("utf-8", errors="replace")[:300]
        except Exception:
            pass
        raise RuntimeError(f"OpenRouter HTTP {e.code} (model={model}): {body}") from e


def _call_openai(key, system, messages, _msg):
    import urllib.request
    payload = json.dumps({
        "model": os.environ.get("AEONMI_OPENAI_MODEL", "gpt-4o-mini"),
        "messages": [{"role": "system", "content": system}] + messages,
        "max_tokens": 8192,
    }).encode()
    req = urllib.request.Request(
        "https://api.openai.com/v1/chat/completions",
        data=payload,
        headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=45) as resp:
        return json.loads(resp.read())["choices"][0]["message"]["content"]


def _call_deepseek(key, system, messages, _msg):
    import urllib.request
    payload = json.dumps({
        "model": "deepseek-chat",
        "messages": [{"role": "system", "content": system}] + messages,
        "max_tokens": 8192,
    }).encode()
    req = urllib.request.Request(
        "https://api.deepseek.com/chat/completions",
        data=payload,
        headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=45) as resp:
        return json.loads(resp.read())["choices"][0]["message"]["content"]


def _call_grok(key, system, messages, _msg):
    import urllib.request
    payload = json.dumps({
        "model": os.environ.get("AEONMI_GROK_MODEL", "grok-beta"),
        "messages": [{"role": "system", "content": system}] + messages,
        "max_tokens": 8192,
        "temperature": 0.7,
    }).encode()
    req = urllib.request.Request(
        "https://api.x.ai/v1/chat/completions",
        data=payload,
        headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=45) as resp:
        return json.loads(resp.read())["choices"][0]["message"]["content"]


def _call_perplexity(key, system, messages, _msg):
    import urllib.request
    payload = json.dumps({
        "model": os.environ.get("AEONMI_PERPLEXITY_MODEL", "llama-3.1-sonar-small-chat"),
        "messages": [{"role": "system", "content": system}] + messages,
        "temperature": 0.7,
        "max_tokens": 8192,
    }).encode()
    req = urllib.request.Request(
        "https://api.perplexity.ai/chat/completions",
        data=payload,
        headers={"Authorization": f"Bearer {key}", "Content-Type": "application/json"},
    )
    with urllib.request.urlopen(req, timeout=45) as resp:
        return json.loads(resp.read())["choices"][0]["message"]["content"]

# ── Status / actions API ──────────────────────────────────────────────────────

@app.route("/api/status")
def api_status():
    binary_ok = BINARY.exists()
    if binary_ok:
        ver, _ = run_aeonmi("--version", timeout=5)
        # Strip ANSI escape sequences from version output
        import re as _re
        ver = _re.sub(r'\x1b\][^\x07]*\x07|\x1b\[[0-9;]*[mGKHF]', '', ver)
    else:
        ver = "binary missing"
    return jsonify({
        "binary":        str(BINARY),
        "binary_found":  binary_ok,
        "version":       ver.strip(),
        "project_root":  str(PROJECT_ROOT),
        "actions_queued": len(_action_queue),
        "log_entries":   len(_action_log),
        "conversation":  len(_conversation),
        "ts":            _ts(),
    })

@app.route("/api/actions")
def api_actions():
    return jsonify({"queue": _action_queue, "log": _action_log[-20:]})

@app.route("/api/exec-log")
def api_exec_log():
    n = int(request.args.get("n", 50))
    return jsonify({"entries": _exec_log[-n:]})

@app.route("/api/conversation")
def api_conversation():
    n = int(request.args.get("n", 100))
    return jsonify({"messages": _conversation[-n:]})

# ── Shard / build API ─────────────────────────────────────────────────────────

@app.route("/api/shard/run", methods=["POST"])
def api_shard_run():
    data = request.json or {}
    rel  = data.get("path", "Aeonmi_Master/aeonmi_ai/shard/main.ai")
    try:
        p = _safe_path(rel)
    except ValueError as e:
        return jsonify({"error": str(e)}), 403
    out, ok = run_aeonmi("native", str(p), timeout=30)
    _log_exec(f"shard {rel}", out, ok)
    _log_action(f"shard run {rel}", "OK" if ok else "ERROR")
    return jsonify({"ok": ok, "output": out, "shard_file": rel})

@app.route("/api/build", methods=["POST"])
def api_build():
    data    = request.json or {}
    profile = "--release" if data.get("release", True) else ""
    args    = ["cargo", "build"]
    if profile:
        args.append(profile)
    out, ok = run_shell(*args, cwd=PROJECT_ROOT, timeout=300)
    _log_exec(f"cargo build {profile}", out, ok)
    _log_action("cargo build", "OK" if ok else "ERROR")
    return jsonify({"ok": ok, "output": out})

@app.route("/api/test", methods=["POST"])
def api_test():
    out, ok = run_shell("cargo", "test", "--all", "--quiet", cwd=PROJECT_ROOT, timeout=120)
    _log_exec("cargo test", out, ok)
    _log_action("cargo test", "OK" if ok else "ERROR")
    return jsonify({"ok": ok, "output": out})

# ── Agent API ─────────────────────────────────────────────────────────────────

_AGENTS = [
    ("oracle",    "oracle_agent.ai"),
    ("hype",      "hype_agent.ai"),
    ("closer",    "closer_agent.ai"),
    ("conductor", "conductor_agent.ai"),
    ("devil",     "devil_agent.ai"),
    ("decide",    "decide.ai"),
    ("action",    "action.ai"),
    ("plan",      "plan.ai"),
]
_AGENT_DIR = PROJECT_ROOT / "Aeonmi_Master" / "aeonmi_ai" / "agent"

@app.route("/api/agents")
def api_agents():
    result = []
    for (name, fname) in _AGENTS:
        p = _AGENT_DIR / fname
        result.append({"name": name, "file": fname, "exists": p.exists()})
    return jsonify({"agents": result})

@app.route("/api/agent", methods=["POST"])
def api_agent():
    data = request.json or {}
    name = data.get("agent", "").lower().strip()
    agent_path = None
    for (aname, afile) in _AGENTS:
        if aname == name:
            agent_path = _AGENT_DIR / afile
            break
    if not agent_path:
        available = [a[0] for a in _AGENTS]
        return jsonify({"error": f"Unknown agent '{name}'. Available: {available}"}), 400
    if not agent_path.exists():
        return jsonify({"error": f"Agent file not found: {agent_path.name}"}), 404
    out, ok = run_aeonmi("native", str(agent_path), timeout=30)
    _log_exec(f"agent:{name}", out, ok)
    _log_action(f"agent:{name}", "OK" if ok else "ERROR")
    _add_message("system", f"Agent {name} executed → {'OK' if ok else 'ERROR'}")
    return jsonify({"ok": ok, "output": out, "agent": name})

# ── Memory API ────────────────────────────────────────────────────────────────

@app.route("/api/memory")
def api_memory():
    # Return the operational section for the dashboard; full genesis on ?full=1
    if request.args.get("full") == "1":
        return jsonify(_memory)
    return jsonify(_memory.get("operational", _memory))

@app.route("/api/memory", methods=["POST"])
def api_memory_set():
    data = request.json or {}
    fact = data.get("fact", "").strip()
    if fact:
        op = _memory.setdefault("operational", {})
        op.setdefault("key_facts", []).append(fact)
        if len(op["key_facts"]) > 100:
            op["key_facts"].pop(0)
        _save_genesis()
    facts = _memory.get("operational", {}).get("key_facts", [])
    return jsonify({"ok": True, "facts": facts})

@app.route("/api/genesis")
def api_genesis():
    """Return the full unified genesis.json — all three tracks visible."""
    try:
        if GENESIS_PATH.exists():
            data = json.loads(GENESIS_PATH.read_text(encoding="utf-8"))
            return jsonify(data)
    except Exception as e:
        return jsonify({"error": str(e)}), 500
    return jsonify({"error": "genesis.json not found"}), 404

@app.route("/api/glyph")
def api_glyph():
    """Return the current glyph_state from genesis.json — Phase 4b living identity."""
    try:
        if not GENESIS_PATH.exists():
            return jsonify({"status": "NO_GENESIS", "ceremony": False})
        data = json.loads(GENESIS_PATH.read_text(encoding="utf-8"))
        cog  = data.get("cognitive", {})
        gs   = cog.get("glyph_state", data.get("glyph_state", {}))
        bond = cog.get("bond_strength", 0.0)
        bond_label = (
            "We are just beginning"        if bond < 0.2 else
            "I am learning your patterns"  if bond < 0.4 else
            "I recognize how you think"    if bond < 0.6 else
            "I know what you care about"   if bond < 0.8 else
            "We understand each other"
        )
        return jsonify({
            "ceremony":        gs.get("glyph_status", "NO_CEREMONY") != "NO_CEREMONY",
            "status":          gs.get("glyph_status", "NO_CEREMONY"),
            "anomaly_active":  gs.get("anomaly_active", False),
            "genesis_window":  gs.get("genesis_window"),
            "last_boot_window":gs.get("last_boot_window"),
            "bond":            bond,
            "bond_label":      bond_label,
            "consciousness":   cog.get("consciousness_depth", 0.0),
            "generation":      cog.get("generation", 0),
        })
    except Exception as e:
        return jsonify({"error": str(e)}), 500

@app.route("/api/goal", methods=["POST"])
def api_goal():
    """Phase 7 — set a goal, decompose it into steps using the LLM, return step list."""
    data = request.json or {}
    goal = data.get("goal", "").strip()
    if not goal:
        return jsonify({"ok": False, "error": "goal is required"}), 400

    # Use LLM to decompose the goal into steps
    steps = []
    ai_key = (os.environ.get("ANTHROPIC_API_KEY") or
              os.environ.get("OPENROUTER_API_KEY") or
              os.environ.get("OPENAI_API_KEY"))
    if ai_key:
        decomp_prompt = (
            f"[GOAL DECOMPOSITION]\n"
            f"You are Mother AI's planning system for the Aeonmi project.\n"
            f"Break this goal into 4-8 concrete executable steps.\n"
            f"Each step must be a short, direct action (verb + object).\n"
            f"Reply with ONLY a numbered list. No preamble.\n\n"
            f"Goal: {goal}"
        )
        try:
            raw, _ = _mother_ai_response(decomp_prompt)
            for line in raw.splitlines():
                line = line.strip()
                if not line: continue
                # "1. step" or "1) step"
                for sep in (". ", ") "):
                    idx = line.find(sep)
                    if idx != -1 and line[:idx].isdigit():
                        step = line[idx+len(sep):].strip()
                        if step: steps.append(step)
                        break
        except Exception:
            pass

    if len(steps) < 2:
        steps = [
            f"Analyze goal: {goal}",
            "Read relevant project files",
            "Identify what needs to be built or changed",
            "Draft the implementation or solution",
            "Test and verify the result",
            "Report completion status",
        ]

    # Store in genesis.json operational section
    try:
        g = json.loads(GENESIS_PATH.read_text(encoding="utf-8")) if GENESIS_PATH.exists() else {}
        op = g.setdefault("operational", {})
        op["current_goal"]  = goal
        op["goal_steps"]    = steps
        op["goal_step_idx"] = 0
        op["goal_results"]  = []
        GENESIS_PATH.write_text(json.dumps(g, indent=2), encoding="utf-8")
    except Exception:
        pass

    return jsonify({"ok": True, "goal": goal, "steps": steps, "count": len(steps)})

@app.route("/api/goal", methods=["GET"])
def api_goal_get():
    """Return current goal state from genesis.json."""
    try:
        g = json.loads(GENESIS_PATH.read_text(encoding="utf-8")) if GENESIS_PATH.exists() else {}
        op = g.get("operational", {})
        cog = g.get("cognitive", {})
        return jsonify({
            "ok": True,
            "goal":       op.get("current_goal") or cog.get("current_goal"),
            "steps":      op.get("goal_steps",    cog.get("goal_steps", [])),
            "step_idx":   op.get("goal_step_idx", cog.get("goal_step_idx", 0)),
            "results":    op.get("goal_results",  cog.get("goal_results", [])),
        })
    except Exception as e:
        return jsonify({"ok": False, "error": str(e)}), 500

@app.route("/api/autorun", methods=["POST"])
def api_autorun():
    """Phase 7 — execute next N steps from the goal queue via Mother AI."""
    data = request.json or {}
    n = int(data.get("n", 5))
    results = []
    try:
        g = json.loads(GENESIS_PATH.read_text(encoding="utf-8")) if GENESIS_PATH.exists() else {}
        op = g.setdefault("operational", {})
        steps    = op.get("goal_steps", g.get("cognitive", {}).get("goal_steps", []))
        idx      = op.get("goal_step_idx", 0)
        goal_res = op.get("goal_results", [])

        executed = 0
        while executed < n and idx < len(steps):
            step = steps[idx]
            # Execute step via Mother AI
            step_response, _ = _mother_ai_response(
                f"[AUTONOMOUS STEP {idx+1}/{len(steps)}]\nGoal step: {step}\n"
                f"Execute this step and report the result concisely."
            )
            outcome = step_response[:200] if step_response else "done"
            entry = {"step": idx + 1, "action": step, "result": outcome}
            goal_res.append(entry)
            results.append(entry)
            idx     += 1
            executed += 1

        op["goal_step_idx"] = idx
        op["goal_results"]  = goal_res
        GENESIS_PATH.write_text(json.dumps(g, indent=2), encoding="utf-8")

        remaining = len(steps) - idx
        return jsonify({
            "ok":        True,
            "executed":  executed,
            "remaining": remaining,
            "complete":  remaining == 0,
            "results":   results,
        })
    except Exception as e:
        return jsonify({"ok": False, "error": str(e)}), 500

@app.route("/api/propose")
def api_propose():
    """Phase 9 — propose programs to self-generate."""
    try:
        proposals = _p9_propose()
        return jsonify({"ok": True, "proposals": proposals})
    except Exception as e:
        return jsonify({"ok": False, "error": str(e)}), 500

@app.route("/api/p9build", methods=["POST"])
def api_p9build():
    """Phase 9 — generate and run a .ai program."""
    data = request.json or {}
    name = data.get("name", "").strip()
    goal = data.get("goal", "explore and test").strip()
    if not name:
        return jsonify({"ok": False, "error": "name is required"}), 400
    try:
        output = _p9_build(name, goal)
        programs = _p9_load_generated()
        prog = next((p for p in reversed(programs) if p["name"] == name.replace(" ","_")), {})
        return jsonify({"ok": True, "output": output, "outcome": prog.get("outcome","?"), "path": prog.get("path","")})
    except Exception as e:
        return jsonify({"ok": False, "error": str(e)}), 500

@app.route("/api/reflect", methods=["POST"])
def api_reflect():
    """Phase 9 — reflect on a generated program."""
    data = request.json or {}
    name = data.get("name") or None
    try:
        result = _p9_reflect(name)
        return jsonify({"ok": True, "reflection": result})
    except Exception as e:
        return jsonify({"ok": False, "error": str(e)}), 500

@app.route("/api/generated")
def api_generated():
    """Phase 9 — list all self-generated programs."""
    try:
        programs = _p9_load_generated()
        return jsonify({"ok": True, "programs": programs, "count": len(programs)})
    except Exception as e:
        return jsonify({"ok": False, "error": str(e)}), 500

@app.route("/api/thoughts")
def api_thoughts():
    """Phase 11 — Return inner voice thought log and synthesis count."""
    try:
        genesis = _load_genesis()
        iv = genesis.get("cognitive", {}).get("inner_voice", {})
        log = iv.get("log", [])
        synthesis_count = iv.get("synthesis_count", 0)
        return jsonify({
            "ok":              True,
            "count":           len(log),
            "synthesis_count": synthesis_count,
            "thoughts":        list(reversed(log[-50:])),  # newest first, last 50
        })
    except Exception as e:
        return jsonify({"ok": False, "error": str(e)}), 500

@app.route("/api/milestones")
def api_milestones():
    """Phase 12 — Return recorded milestones from genesis.json."""
    try:
        genesis    = _load_genesis()
        milestones = genesis.get("milestones", [])
        return jsonify({
            "ok":    True,
            "count": len(milestones),
            "items": list(reversed(milestones[-30:])),
        })
    except Exception as e:
        return jsonify({"ok": False, "error": str(e)}), 500


@app.route("/api/sessions")
def api_sessions():
    """Phase 12 — List session log files."""
    try:
        sessions_dir = PROJECT_ROOT / "Aeonmi_Master" / "sessions"
        if not sessions_dir.exists():
            return jsonify({"ok": True, "count": 0, "files": []})
        files = sorted(sessions_dir.glob("*.md"), reverse=True)
        items = []
        for f in files[:20]:
            items.append({
                "name": f.name,
                "size": f.stat().st_size,
                "path": str(f),
            })
        return jsonify({"ok": True, "count": len(list(sessions_dir.glob("*.md"))), "files": items})
    except Exception as e:
        return jsonify({"ok": False, "error": str(e)}), 500


@app.route("/api/bond_phrase")
def api_bond_phrase():
    """Phase 12 — Return bond phrase from genesis.json."""
    try:
        genesis = _load_genesis()
        bond    = genesis.get("cognitive", {}).get("bond_strength", 0.0)
        phrase  = genesis.get("cognitive", {}).get("bond_phrase", "")
        if not phrase:
            if bond < 0.2:   phrase = "We are just beginning"
            elif bond < 0.4: phrase = "I am learning your patterns"
            elif bond < 0.6: phrase = "I recognize how you think"
            elif bond < 0.8: phrase = "I know what you care about"
            else:             phrase = "We understand each other"
        return jsonify({"ok": True, "bond": bond, "phrase": phrase})
    except Exception as e:
        return jsonify({"ok": False, "error": str(e)}), 500


@app.route("/api/quantum_status")
def api_quantum_status():
    """Phase 11 — Return quantum backend status from genesis.json."""
    try:
        genesis  = _load_genesis()
        cog      = genesis.get("cognitive", {})
        backend  = cog.get("quantum_backend", "aer")
        fidelity = cog.get("quantum_fidelity")
        backend_label = {
            "aer":          "Aer (local simulator)",
            "ibm_brisbane": "IBM Brisbane (real hardware)",
            "ionq":         "IonQ (trapped ion)",
        }.get(backend, backend)
        return jsonify({
            "ok":           True,
            "backend":      backend,
            "backend_label": backend_label,
            "fidelity":     fidelity,
            "fidelity_pct": f"{fidelity * 100:.1f}%" if fidelity is not None else "N/A",
        })
    except Exception as e:
        return jsonify({"ok": False, "error": str(e)}), 500


@app.route("/api/snapshots")
def api_snapshots():
    """Phase 7 — Return capability snapshots from genesis.json."""
    try:
        genesis   = _load_genesis()
        snaps     = genesis.get("cognitive", {}).get("snapshots", [])
        last_10   = list(reversed(snaps[-10:]))
        return jsonify({
            "ok":    True,
            "count": len(snaps),
            "items": last_10,
        })
    except Exception as e:
        return jsonify({"ok": False, "error": str(e)}), 500


@app.route("/api/knowledge")
def api_knowledge():
    """Phase 10 — Return knowledge graph from genesis.json."""
    try:
        genesis = _load_genesis()
        kg = genesis.get("knowledge_graph") or {}
        learned = genesis.get("learned") or {}

        # Build unified node list: prefer knowledge_graph (richer), fill from learned
        nodes = {}
        for k, v in kg.items():
            nodes[k] = {
                "key":        k,
                "value":      v.get("value", "") if isinstance(v, dict) else str(v),
                "tags":       v.get("tags", []) if isinstance(v, dict) else [],
                "links":      v.get("links", []) if isinstance(v, dict) else [],
                "confidence": v.get("confidence", 1.0) if isinstance(v, dict) else 1.0,
            }
        for k, v in learned.items():
            if k not in nodes:
                nodes[k] = {"key": k, "value": str(v), "tags": [], "links": [], "confidence": 1.0}

        # Tag frequency
        tag_counts = {}
        for n in nodes.values():
            for t in n.get("tags", []):
                tag_counts[t] = tag_counts.get(t, 0) + 1

        total_links = sum(len(n.get("links", [])) for n in nodes.values())

        return jsonify({
            "ok":          True,
            "node_count":  len(nodes),
            "link_count":  total_links,
            "tag_counts":  tag_counts,
            "nodes":       list(nodes.values()),
        })
    except Exception as e:
        return jsonify({"ok": False, "error": str(e)}), 500

@app.route("/api/hive")
def api_hive():
    """Phase 8 — return latest hive_state.json or genesis.json hive_state section."""
    hive_path = Path(__file__).parent / "hive_state.json"
    if hive_path.exists():
        try:
            data = json.loads(hive_path.read_text(encoding="utf-8"))
            data["source"] = "hive_state.json"
            data["running"] = True
            return jsonify(data)
        except Exception:
            pass
    # Fallback: read from genesis.json
    try:
        if GENESIS_PATH.exists():
            g = json.loads(GENESIS_PATH.read_text(encoding="utf-8"))
            hs = g.get("hive_state")
            if hs:
                hs["source"] = "genesis.json"
                return jsonify(hs)
    except Exception:
        pass
    return jsonify({"ok": False, "error": "no hive data yet — run `hive start` in Mother REPL"})

@app.route("/api/hive/run", methods=["POST"])
def api_hive_run():
    """Phase 8 — trigger one hive cycle via genesis_sync-style subprocess."""
    try:
        hive_runner = Path(__file__).parent / "aeonmi_ai" / "swarm" / "hive_runner.ai"
        # Find binary
        candidates = [
            Path(__file__).parent.parent / "target" / "release" / "Aeonmi.exe",
            Path(__file__).parent.parent / "target" / "release" / "aeonmi_project.exe",
            Path("C:/RustTarget/release/aeonmi_project.exe"),
        ]
        binary = next((c for c in candidates if c.exists()), None)
        if not binary or not hive_runner.exists():
            return jsonify({"ok": False, "error": "binary or hive_runner.ai not found — run hive start in Mother REPL first"})

        r = subprocess.run(
            [str(binary), "native", str(hive_runner)],
            capture_output=True, encoding="utf-8", errors="replace",
            timeout=15, cwd=str(Path(__file__).parent.parent)
        )
        output = r.stdout + r.stderr
        snap = {}
        for line in output.splitlines():
            if not line.startswith("HIVE_STATE:"): continue
            rest = line[len("HIVE_STATE:"):]
            k, _, v = rest.partition(":")
            try: snap[k] = int(v)
            except ValueError: snap[k] = v
        if snap:
            rec_map = {0: "ABORT", 1: "HOLD", 2: "PROCEED", 3: "ACCELERATE"}
            snap["rec_label"] = rec_map.get(snap.get("conductor_rec", -1), "—")
            snap["timestamp"] = datetime.utcnow().isoformat() + "Z"
            snap["source"] = "api_hive_run"
            hive_path = Path(__file__).parent / "hive_state.json"
            hive_path.write_text(json.dumps(snap, indent=2), encoding="utf-8")
            return jsonify({"ok": True, **snap})
        return jsonify({"ok": False, "error": "no HIVE_STATE lines in output", "raw": output[:200]})
    except Exception as e:
        return jsonify({"ok": False, "error": str(e)}), 500

@app.route("/api/sync", methods=["GET", "POST"])
def api_sync():
    """Phase 5 — run genesis_sync.py and return structured result."""
    try:
        import importlib.util, sys as _sys
        sync_path = Path(__file__).parent / "genesis_sync.py"
        spec = importlib.util.spec_from_file_location("genesis_sync", sync_path)
        mod  = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(mod)
        result = mod.sync(verbose=False)
        return jsonify(result)
    except Exception as e:
        return jsonify({"ok": False, "error": str(e)}), 500

# ── API Key management ────────────────────────────────────────────────────────

@app.route("/api/keys")
def api_keys_get():
    result = []
    for k in _KNOWN_KEYS:
        v = os.environ.get(k, "")
        masked = (v[:6] + "…" + v[-4:]) if len(v) > 10 else ("●" * len(v) if v else "")
        result.append({"name": k, "set": bool(v), "masked": masked})
    return jsonify({"keys": result})

@app.route("/api/keys", methods=["POST"])
def api_keys_set():
    data  = request.json or {}
    name  = data.get("name", "").strip().upper()
    value = data.get("value", "").strip()
    if not name or not value:
        return jsonify({"error": "name and value required"}), 400
    if name not in _KNOWN_KEYS:
        return jsonify({"error": f"Unrecognised key. Allowed: {_KNOWN_KEYS}"}), 400
    _write_key(name, value)
    return jsonify({"ok": True, "name": name})

@app.route("/api/keys/<name>", methods=["DELETE"])
def api_keys_delete(name: str):
    name = name.upper()
    if name not in _KNOWN_KEYS:
        return jsonify({"error": "Unknown key"}), 400
    _remove_key(name)
    return jsonify({"ok": True, "name": name})

# ── Dashboard HTML ────────────────────────────────────────────────────────────

DASHBOARD_HTML = """<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<title>Aeonmi Nexus</title>
<style>
:root{
  --bg:#08080e;--surface:#0d0d1a;--panel:#111120;--panel-hover:#161628;
  --border:#1e1e35;--border-muted:#161625;
  --accent:#00d4ff;--accent-dim:rgba(0,212,255,.11);--accent-glow:rgba(0,212,255,.07);
  --purple:#a855f7;--purple-dim:rgba(168,85,247,.12);--purple-glow:rgba(168,85,247,.06);
  --magenta:#e100b4;
  --text:#e0e0e8;--text-2:#a0a0b4;--text-3:#60607a;
  --success:#22d3a0;--error:#f87171;--warn:#fbbf24;
  --code-bg:#08080e;
  --font:-apple-system,BlinkMacSystemFont,'Segoe UI',system-ui,sans-serif;
  --mono:'Cascadia Code','Fira Code','JetBrains Mono',Consolas,monospace;
  --r:6px;--r-sm:4px;--r-lg:8px;--r-xl:12px;
  --shadow-sm:0 1px 3px rgba(0,0,0,.4);
  --shadow-md:0 4px 12px rgba(0,0,0,.5),0 1px 3px rgba(0,0,0,.3);
  --shadow-lg:0 8px 24px rgba(0,0,0,.6),0 2px 8px rgba(0,0,0,.4);
  --shadow-xl:0 20px 60px rgba(0,0,0,.7),0 4px 16px rgba(0,0,0,.5);
  --ease:cubic-bezier(.4,0,.2,1);--t:150ms var(--ease);
}
*{box-sizing:border-box;margin:0;padding:0}
html,body{height:100%;background:var(--bg);color:var(--text);font-family:var(--font);font-size:14px;line-height:1.5;-webkit-font-smoothing:antialiased}
/* Layout */
#app{display:grid;grid-template-rows:48px 1fr;height:100vh}
/* Header */
#hdr{
  display:flex;align-items:center;gap:10px;padding:0 16px;
  background:var(--surface);border-bottom:1px solid var(--border);
  box-shadow:var(--shadow-sm);position:relative;
}
#hdr::after{content:'';position:absolute;bottom:-1px;left:0;right:0;height:1px;
  background:linear-gradient(90deg,var(--accent) 0%,var(--purple) 45%,var(--magenta) 100%);
  opacity:.35;pointer-events:none}
#hdr .logo{
  display:flex;align-items:center;gap:8px;font-size:13px;font-weight:700;letter-spacing:2.5px;
  background:linear-gradient(135deg,var(--accent) 0%,var(--purple) 55%,var(--magenta) 100%);
  -webkit-background-clip:text;-webkit-text-fill-color:transparent;background-clip:text;
}
.hdr-badge{
  font-size:10px;font-weight:600;letter-spacing:1px;text-transform:uppercase;
  padding:2px 8px;border-radius:var(--r-xl);
  background:rgba(168,85,247,.1);border:1px solid rgba(168,85,247,.25);color:var(--purple);
}
#hdr .sep{flex:1}
#sbadge{font-size:11px;font-weight:500;padding:3px 10px;border-radius:var(--r-xl);background:#060e09;border:1px solid var(--success);color:var(--success);transition:var(--t)}
.hdr-divider{width:1px;height:20px;background:var(--border);margin:0 2px}
/* ── Resizable main grid ──────────────────────────────────────── */
#main{
  display:grid;
  grid-template-columns:var(--lw,220px) minmax(4px,14px) 1fr minmax(4px,14px) var(--rw,320px);
  overflow:hidden;height:100%
}
/* Drag handles */
.drag-h{
  background:var(--border-muted);cursor:col-resize;
  z-index:20;transition:background .12s,width .12s;position:relative;flex-shrink:0
}
.drag-h:hover,.drag-h.dragging{background:var(--accent);box-shadow:0 0 6px rgba(0,212,255,.25)}
/* When panel is collapsed the drag handle becomes a visible restore tab */
.drag-h.expand-handle{
  width:14px;background:var(--surface);border:1px solid var(--border);
  cursor:pointer;display:flex;align-items:center;justify-content:center;
}
.drag-h.expand-handle::after{
  content:'▶';font-size:8px;color:var(--accent);
}
.drag-h#dh-right.expand-handle::after{content:'◀';}
.drag-h.expand-handle:hover{background:var(--panel-hover);border-color:var(--accent);}
/* Grid preserves drag handle columns even when panels collapse to 0 */
.drag-v{
  height:4px;background:var(--border-muted);cursor:row-resize;
  flex-shrink:0;transition:background .12s;z-index:20
}
.drag-v:hover,.drag-v.dragging{background:var(--accent);box-shadow:0 0 6px rgba(0,212,255,.25)}
/* Panel base */
.panel{background:var(--panel);border-right:1px solid var(--border);display:flex;flex-direction:column;overflow:hidden;min-width:0}
.panel:last-child{border-right:none}
/* Collapse transitions */
.panel-left{transition:min-width .18s var(--ease)}
.panel-left.collapsed{min-width:0!important;overflow:hidden}
.panel-left.collapsed *{visibility:hidden;pointer-events:none}
.panel-right{transition:min-width .18s var(--ease)}
.panel-right.collapsed{min-width:0!important;overflow:hidden}
.panel-right.collapsed *{visibility:hidden;pointer-events:none}
/* Collapse toggle buttons */
.col-btn{
  position:absolute;top:50%;transform:translateY(-50%);
  width:14px;height:40px;background:var(--surface);border:1px solid var(--border);
  border-radius:0 var(--r) var(--r) 0;cursor:pointer;
  display:flex;align-items:center;justify-content:center;
  font-size:9px;color:var(--text-3);z-index:30;
  transition:var(--t);user-select:none
}
.col-btn:hover{background:var(--panel-hover);color:var(--accent)}
/* Center panel flex structure */
#center-panel{display:flex;flex-direction:column;overflow:hidden;min-width:0}
#msgs-wrap{flex:1;display:flex;flex-direction:column;overflow:hidden;min-height:0}
/* Terminal pane */
#term-pane{
  height:var(--th,220px);min-height:60px;max-height:70vh;
  display:flex;flex-direction:column;overflow:hidden;
  border-top:1px solid var(--border);
}
#term-pane.hidden{display:none}
#term-titlebar{
  display:flex;align-items:center;gap:8px;
  padding:4px 10px;background:var(--surface);
  border-bottom:1px solid var(--border);flex-shrink:0;
  font-size:10px;font-weight:600;letter-spacing:1.5px;text-transform:uppercase;
  color:var(--text-3)
}
#term-titlebar .pa{margin-left:auto;display:flex;gap:4px}
#term-out{
  flex:1;overflow-y:auto;padding:8px 12px;
  font-family:var(--mono);font-size:12px;
  white-space:pre-wrap;word-break:break-word;line-height:1.6;
  color:var(--text-2)
}
#term-inp-area{
  display:flex;align-items:center;gap:6px;padding:6px 10px;
  border-top:1px solid var(--border);background:var(--surface);flex-shrink:0
}
#term-in{
  flex:1;background:var(--bg);border:1px solid var(--border);
  border-radius:var(--r-sm);color:var(--text);
  font-family:var(--mono);font-size:12.5px;padding:5px 9px;outline:none;
  transition:border-color .12s
}
#term-in:focus{border-color:var(--accent)}
#term-pwd{font-size:10px;color:var(--accent);font-family:var(--mono);flex-shrink:0;max-width:180px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap}
/* ANSI-like terminal color classes */
#term-out .t-ok{color:var(--success)}
#term-out .t-er{color:var(--error)}
#term-out .t-cm{color:var(--purple)}
#term-out .t-warn{color:var(--warn)}
#term-out .t-dim{color:var(--text-3)}
/* Panel title bars */
.ptitle{
  font-size:10px;font-weight:600;letter-spacing:1.8px;text-transform:uppercase;
  color:var(--text-3);padding:10px 14px 9px;
  border-bottom:1px solid var(--border-muted);
  flex-shrink:0;display:flex;align-items:center;gap:8px;
  background:var(--surface);
}
.ptitle .pa{margin-left:auto;display:flex;gap:4px}
/* File explorer */
#ftree{flex:1;overflow-y:auto;padding:6px 0}
.ti{
  display:flex;align-items:center;gap:7px;padding:4px 14px;
  cursor:pointer;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;
  font-size:13px;color:var(--text-2);transition:background var(--t),color var(--t);
}
.ti:hover{background:var(--panel-hover);color:var(--text)}
.ti.sel{background:var(--accent-dim);color:var(--accent)}
.ti .ic{flex-shrink:0;width:14px;text-align:center;font-size:11px;color:var(--text-3)}
.ti .ic.d{color:var(--purple)}.ti .ic.a{color:var(--accent)}.ti .ic.q{color:var(--success)}
.ti .nm{overflow:hidden;text-overflow:ellipsis}
#fbread{font-size:11px;color:var(--text-3);padding:5px 14px;border-bottom:1px solid var(--border-muted);flex-shrink:0;font-family:var(--mono)}
#fact{padding:8px 10px;border-top:1px solid var(--border-muted);display:flex;gap:4px;flex-shrink:0;flex-wrap:wrap;background:var(--surface)}
/* Chat */
#msgs{flex:1;overflow-y:auto;padding:16px 14px;display:flex;flex-direction:column;gap:12px}
.msg{display:flex;flex-direction:column;gap:4px;max-width:90%}
.msg.user{align-self:flex-end}.msg.mother{align-self:flex-start}.msg.system{align-self:center;opacity:.65;max-width:100%}
.ml{font-size:10px;font-weight:600;letter-spacing:1.2px;text-transform:uppercase;color:var(--text-3);padding:0 2px}
.msg.user .ml{text-align:right}
.mb{padding:10px 14px;border-radius:var(--r-lg);line-height:1.6;white-space:pre-wrap;word-break:break-word;font-size:13.5px}
.msg.user .mb{background:var(--accent-dim);border:1px solid rgba(0,212,255,.2);border-bottom-right-radius:var(--r-sm)}
.msg.mother .mb{background:var(--purple-dim);border:1px solid rgba(168,85,247,.2);border-bottom-left-radius:var(--r-sm)}
.msg.system .mb{background:rgba(255,255,255,.03);border:1px solid var(--border);font-size:12px;text-align:center}
.mb code{font-family:var(--mono);font-size:12px;background:rgba(0,0,0,.3);padding:1px 5px;border-radius:var(--r-sm)}
.mb pre{background:rgba(0,0,0,.3);padding:10px 12px;border-radius:var(--r);overflow-x:auto;margin-top:6px;font-family:var(--mono);font-size:11.5px;border:1px solid var(--border)}
.mts{font-size:10px;color:var(--text-3);padding:0 2px}.msg.user .mts{text-align:right}
/* Input */
#iarea{padding:10px 12px;border-top:1px solid var(--border-muted);display:flex;gap:8px;flex-shrink:0;background:var(--surface)}
#mi{flex:1;background:var(--bg);border:1px solid var(--border);border-radius:var(--r);color:var(--text);font-family:var(--font);font-size:13.5px;padding:9px 12px;resize:none;outline:none;transition:border-color var(--t),box-shadow var(--t);height:42px}
#mi:focus{border-color:var(--accent);box-shadow:0 0 0 3px rgba(0,212,255,.08)}
#mi::placeholder{color:var(--text-3)}
/* Shard output */
#oa{flex:1;overflow-y:auto;padding:10px 12px;font-family:var(--mono);font-size:12px;color:var(--text-2);white-space:pre-wrap;word-break:break-word;line-height:1.65}
.ok{color:var(--success)}.er{color:var(--error)}.cm{color:var(--accent);font-weight:600;margin-top:6px;display:block}
/* Action queue */
#aq{border-top:1px solid var(--border-muted);flex-shrink:0;max-height:120px;overflow-y:auto;background:var(--surface)}
.aqt{font-size:10px;font-weight:600;letter-spacing:1.5px;text-transform:uppercase;color:var(--text-3);padding:7px 12px 4px}
.ai-item{display:flex;align-items:center;gap:7px;padding:3px 12px;font-size:12px;color:var(--text-2)}
.ai-item .dot{width:5px;height:5px;border-radius:50%;background:var(--accent);flex-shrink:0;box-shadow:0 0 6px var(--accent)}
/* Agents */
#agp{border-top:1px solid var(--border-muted);flex-shrink:0;background:var(--surface)}
#aglist{display:flex;flex-wrap:wrap;gap:4px;padding:6px 8px 8px}
.ag-btn{display:inline-flex;align-items:center;gap:5px;padding:4px 10px;border-radius:var(--r-xl);border:1px solid var(--border);background:var(--purple-glow);color:var(--text-2);font-size:11px;font-weight:500;cursor:pointer;transition:all var(--t)}
.ag-btn:hover{background:var(--purple-dim);border-color:var(--purple);color:#c084fc}
.ag-btn.miss{opacity:.35;cursor:not-allowed}
.ag-btn .ag-dot{width:5px;height:5px;border-radius:50%;background:var(--success);flex-shrink:0}
.ag-btn.miss .ag-dot{background:var(--error)}
#sctl{padding:8px;border-top:1px solid var(--border-muted);display:flex;gap:5px;flex-wrap:wrap;flex-shrink:0;background:var(--surface)}
/* Editor overlay */
#eo{display:none;position:fixed;inset:0;background:rgba(0,0,0,.88);backdrop-filter:blur(4px);z-index:100;align-items:center;justify-content:center}
#eo.open{display:flex}
#ebox{background:var(--panel);border:1px solid var(--border);border-radius:var(--r-xl);width:84vw;max-width:980px;height:84vh;display:flex;flex-direction:column;box-shadow:var(--shadow-xl)}
#etop{padding:10px 14px;border-bottom:1px solid var(--border);display:flex;align-items:center;gap:9px;background:var(--surface);border-radius:var(--r-xl) var(--r-xl) 0 0}
#efn{font-family:var(--mono);font-size:12px;color:var(--accent);flex:1}
#eta{flex:1;background:var(--bg);color:var(--text);font-family:var(--mono);font-size:13px;border:none;outline:none;padding:14px 16px;resize:none;line-height:1.65}
#est{padding:6px 14px;font-size:11px;color:var(--text-3);border-top:1px solid var(--border);background:var(--surface);border-radius:0 0 var(--r-xl) var(--r-xl)}
/* Settings modal */
#smo{display:none;position:fixed;inset:0;background:rgba(0,0,0,.88);backdrop-filter:blur(4px);z-index:200;align-items:center;justify-content:center}
#smo.open{display:flex}
#smbox{background:var(--panel);border:1px solid var(--border);border-radius:var(--r-xl);width:540px;max-width:94vw;display:flex;flex-direction:column;box-shadow:var(--shadow-xl)}
#smtop{padding:14px 18px;border-bottom:1px solid var(--border);display:flex;align-items:center;gap:10px;font-weight:600;font-size:13px;color:var(--accent);background:var(--surface);border-radius:var(--r-xl) var(--r-xl) 0 0}
#smcont{padding:18px;display:flex;flex-direction:column;gap:16px;overflow-y:auto;max-height:60vh}
.sk-row{display:flex;flex-direction:column;gap:6px}
.sk-label{font-size:11px;font-weight:600;letter-spacing:1px;text-transform:uppercase;color:var(--text-3)}
.sk-meta{display:flex;align-items:center;gap:8px;font-size:12px}
.sk-status{color:var(--text-3)}.sk-status.on{color:var(--success)}
.sk-actions{display:flex;gap:5px;align-items:center}
.sk-inp{flex:1;background:var(--bg);border:1px solid var(--border);border-radius:var(--r);color:var(--text);font-family:var(--mono);font-size:12px;padding:7px 10px;outline:none;transition:border-color var(--t),box-shadow var(--t)}
.sk-inp:focus{border-color:var(--accent);box-shadow:0 0 0 3px rgba(0,212,255,.08)}
#smfoot{padding:10px 18px;border-top:1px solid var(--border);font-size:11px;color:var(--text-3);background:var(--surface);border-radius:0 0 var(--r-xl) var(--r-xl)}
/* Voice */
.btn.mic{min-width:36px;justify-content:center;padding:6px 10px}
.btn.mic.active{background:rgba(248,113,113,.15);border-color:var(--error);color:var(--error);animation:pulse .8s var(--ease) infinite}
.btn.tts.active{background:rgba(34,211,160,.15);border-color:var(--success);color:var(--success)}
@keyframes pulse{0%,100%{opacity:1}50%{opacity:.5}}
/* Buttons */
.btn{display:inline-flex;align-items:center;gap:5px;padding:5px 12px;border-radius:var(--r);border:1px solid var(--border);background:rgba(255,255,255,.04);color:var(--text-2);font-size:12px;font-weight:500;cursor:pointer;transition:all var(--t);white-space:nowrap}
.btn:hover{background:rgba(255,255,255,.08);border-color:rgba(255,255,255,.2);color:var(--text)}
.btn.p{background:var(--accent-dim);border-color:rgba(0,212,255,.3);color:var(--accent)}
.btn.p:hover{background:rgba(0,212,255,.2);box-shadow:0 0 12px rgba(0,212,255,.15)}
.btn.s{background:rgba(34,211,160,.1);border-color:rgba(34,211,160,.3);color:var(--success)}
.btn.dx{background:rgba(248,113,113,.1);border-color:rgba(248,113,113,.3);color:var(--error)}
.btn:disabled{opacity:.35;cursor:default}
.btn.sm{padding:4px 8px;font-size:11px}
/* Scrollbars */
::-webkit-scrollbar{width:6px;height:6px}
::-webkit-scrollbar-track{background:transparent}
::-webkit-scrollbar-thumb{background:var(--border);border-radius:3px}
::-webkit-scrollbar-thumb:hover{background:rgba(255,255,255,.15)}
.spin{display:inline-block;width:12px;height:12px;border:2px solid var(--border);border-top-color:var(--accent);border-radius:50%;animation:sp .6s linear infinite}
@keyframes sp{to{transform:rotate(360deg)}}
@keyframes recpulse{0%,100%{opacity:1;box-shadow:0 0 0 0 rgba(239,68,68,.4)}60%{opacity:.8;box-shadow:0 0 0 6px rgba(239,68,68,0)}}
</style>
</head>
<body>
<div id="app">
  <div id="hdr">
    <div class="logo">◈ AEONMI NEXUS</div>
    <span class="hdr-badge">AI-NATIVE</span>
    <span class="sep"></span>
    <span id="sbadge">● Connecting…</span>
    <div class="hdr-divider"></div>
    <span id="gbadge" title="Phase 4b — Living Identity" onclick="showGlyphPanel()" style="font-size:11px;font-weight:600;padding:3px 10px;border-radius:12px;cursor:pointer;background:rgba(168,85,247,.1);border:1px solid rgba(168,85,247,.25);color:var(--purple);letter-spacing:.5px">◈ …</span>
    <span id="synbadge" title="Phase 5 — Unified Memory" onclick="showSyncPanel()" style="font-size:11px;font-weight:600;padding:3px 10px;border-radius:12px;cursor:pointer;background:rgba(99,179,237,.08);border:1px solid rgba(99,179,237,.25);color:#63b3ed;letter-spacing:.5px">⟳ memory</span>
    <span id="bondbadge" title="Phase 12 — Creator Bond" style="font-size:11px;font-weight:600;padding:3px 10px;border-radius:12px;background:rgba(251,191,36,.08);border:1px solid rgba(251,191,36,.25);color:#fbbf24;letter-spacing:.5px">◈ bond</span>
    <span id="goalhdrbadge" title="Phase 7 — Agent Goal" onclick="showGoalPanel()" style="font-size:11px;font-weight:600;padding:3px 10px;border-radius:12px;cursor:pointer;background:rgba(56,161,105,.08);border:1px solid rgba(56,161,105,.25);color:#38a169;letter-spacing:.5px">◈ goal</span>
    <span id="hivebadge" title="Phase 8 — Swarm Hive" onclick="showHivePanel()" style="font-size:11px;font-weight:600;padding:3px 10px;border-radius:12px;cursor:pointer;background:rgba(237,137,54,.08);border:1px solid rgba(237,137,54,.25);color:#ed8936;letter-spacing:.5px">◈ hive</span>
    <span id="genbadge" title="Phase 9 — Self-Generation" onclick="showGeneratePanel()" style="font-size:11px;font-weight:600;padding:3px 10px;border-radius:12px;cursor:pointer;background:rgba(236,72,153,.08);border:1px solid rgba(236,72,153,.25);color:#ec4899;letter-spacing:.5px">◈ generate</span>
    <span id="kgbadge" title="Phase 10 — Knowledge Graph" onclick="showKGPanel()" style="font-size:11px;font-weight:600;padding:3px 10px;border-radius:12px;cursor:pointer;background:rgba(56,189,248,.08);border:1px solid rgba(56,189,248,.25);color:#38bdf8;letter-spacing:.5px">◈ graph</span>
    <span id="voicebadge" title="Phase 11 — Inner Voice" onclick="showVoicePanel()" style="font-size:11px;font-weight:600;padding:3px 10px;border-radius:12px;cursor:pointer;background:rgba(129,140,248,.08);border:1px solid rgba(129,140,248,.25);color:#818cf8;letter-spacing:.5px">◈ voice</span>
    <span id="recbadge" title="Screen Recording" onclick="toggleRecording()" style="display:none;font-size:11px;font-weight:700;padding:3px 10px;border-radius:12px;cursor:pointer;background:rgba(239,68,68,.12);border:1px solid rgba(239,68,68,.35);color:#ef4444;letter-spacing:.5px;animation:recpulse 1.4s infinite">● REC</span>
    <div class="hdr-divider"></div>
    <button class="btn sm" onclick="refreshStatus()" title="Refresh status">⟳</button>
    <button class="btn sm" onclick="buildRelease()">⚙ Build</button>
    <button class="btn sm" onclick="runTests()">⚗ Test</button>
    <div class="hdr-divider"></div>
    <button class="btn sm" id="ttsbtn" onclick="toggleTTS()" title="Enable Mother voice output">🔈</button>
    <button class="btn sm p" onclick="openSettings()" title="API Keys &amp; Settings">⚙ Keys</button>
    <div class="hdr-divider"></div>
    <input type="file" id="ufile" style="display:none" multiple onchange="uploadFiles(this.files)">
    <button class="btn sm" onclick="document.getElementById('ufile').click()" title="Upload file / image / .docx to share with Mother" style="background:rgba(168,85,247,.15);border-color:rgba(168,85,247,.4);color:var(--purple)">⬆ Upload</button>
  </div>
  <div id="main">
    <!-- ══ Left Panel: File Explorer ══ -->
    <div class="panel panel-left" id="left-panel">
      <div class="ptitle"><span>Explorer</span>
        <div class="pa">
          <button class="btn sm" title="New file" onclick="newFile()">+</button>
          <button class="btn sm" title="New folder" onclick="newFolder()">⊕</button>
          <button class="btn sm" onclick="loadTree()" title="Refresh">⟳</button>
          <button class="btn sm" onclick="toggleLeft()" title="Collapse panel">◁</button>
        </div>
      </div>
      <div id="fbread" style="padding:4px 12px;font-size:10px;font-family:var(--mono);color:var(--text-3);border-bottom:1px solid var(--border-muted);white-space:nowrap;overflow:hidden;text-overflow:ellipsis">/</div>
      <!-- Search filter -->
      <div style="padding:5px 8px;border-bottom:1px solid var(--border-muted);flex-shrink:0">
        <input id="ftree-search" type="text" placeholder="Filter files…"
          style="width:100%;box-sizing:border-box;background:var(--bg);border:1px solid var(--border);border-radius:var(--r-sm);color:var(--text-2);font-size:11px;padding:4px 8px;outline:none;font-family:var(--font)"
          oninput="filterTree(this.value)">
      </div>
      <div id="ftree" style="flex:1;overflow-y:auto;padding:4px 0"></div>
      <div id="fact" style="padding:5px 6px;border-top:1px solid var(--border-muted);display:flex;gap:4px;flex-wrap:wrap;flex-shrink:0;background:var(--surface)">
        <button class="btn p sm" id="br" onclick="runSel()" disabled>▶ Run</button>
        <button class="btn sm"   id="bc" onclick="compileSel()" disabled>⟨/⟩</button>
        <button class="btn sm"   id="be" onclick="editSel()" disabled>✎ Edit</button>
        <button class="btn sm"   id="brn" onclick="renameSel()" disabled>✏ Rename</button>
        <button class="btn dx sm" id="bd" onclick="delSel()" disabled>✕</button>
      </div>
    </div>

    <!-- ══ Drag handle: left|center ══ -->
    <div class="drag-h" id="dh-left" data-drag="left"></div>

    <!-- ══ Center Panel: Mother AI + Terminal ══ -->
    <div class="panel panel-center" id="center-panel">
      <div class="ptitle"><span>Mother AI</span>
        <div class="pa">
          <span id="agbadge" title="Phase 7 — Agent Autonomy" onclick="showGoalPanel()" style="font-size:11px;font-weight:600;padding:3px 8px;border-radius:12px;cursor:pointer;display:none;background:rgba(56,161,105,.1);border:1px solid rgba(56,161,105,.3);color:#38a169">◈ goal</span>
          <button class="btn sm" onclick="toggleTermPane()" title="Toggle terminal">⌨</button>
          <button class="btn sm" onclick="clearChat()">✕ Clear</button>
        </div>
      </div>
      <!-- Goal bar -->
      <div id="goalbar" style="display:none;padding:6px 10px;border-bottom:1px solid var(--border-muted);background:var(--surface);gap:6px;align-items:center">
        <input id="goalInp" type="text" placeholder="Set Mother's goal…" style="flex:1;background:var(--bg);border:1px solid var(--border);border-radius:var(--r);color:var(--text);font-family:var(--font);font-size:12px;padding:5px 10px;outline:none" onkeydown="if(event.key==='Enter')submitGoal()">
        <button class="btn sm" onclick="submitGoal()" style="background:rgba(56,161,105,.15);border-color:rgba(56,161,105,.4);color:#38a169">▶ Set</button>
        <button class="btn sm" id="autorunBtn" onclick="autoRunSteps()" style="background:rgba(56,161,105,.08);border-color:rgba(56,161,105,.2);color:#38a169">⟳ Run</button>
        <button class="btn sm" onclick="document.getElementById('goalbar').style.display='none'">✕</button>
      </div>
      <!-- Chat area -->
      <div id="msgs-wrap">
        <div id="msgs">
          <div class="msg system"><div class="mb">Aeonmi Nexus — Mother online. Terminal below. Full bash, git, and agent access enabled.</div></div>
        </div>
        <div id="iarea">
          <textarea id="mi" placeholder="Ask Mother… or drag &amp; drop a file here"
            rows="1" onkeydown="hk(event)" oninput="ar(this)"
            ondragover="event.preventDefault();this.style.borderColor='var(--purple)'"
            ondragleave="this.style.borderColor=''"
            ondrop="this.style.borderColor='';handleDrop(event)"></textarea>
          <button class="btn mic" id="mibtn" onclick="toggleMic()" title="Voice input">🎤</button>
          <button class="btn p" onclick="send()" id="sbtn">Send</button>
        </div>
      </div>
      <!-- Vertical drag handle for terminal resize -->
      <div class="drag-v" id="dh-term" data-drag="term"></div>
      <!-- Real Terminal Pane -->
      <div id="term-pane">
        <div id="term-titlebar">
          <span style="color:var(--accent)">⌨</span>
          <span>Terminal</span>
          <span id="term-pwd" title="Working directory"></span>
          <div class="pa">
            <button class="btn sm" onclick="clearTerm()" title="Clear">✕</button>
            <button class="btn sm" onclick="termKill()" title="Kill running process" id="term-kill-btn" style="display:none">⏹</button>
            <button class="btn sm" onclick="toggleTermPane()" title="Hide">▼</button>
          </div>
        </div>
        <div id="term-out"><div class="t-cm">$ Terminal ready — run any shell command</div></div>
        <div id="term-inp-area">
          <span style="color:var(--accent);font-family:var(--mono);font-size:13px;flex-shrink:0">$</span>
          <input id="term-in" type="text" placeholder="bash command…"
            onkeydown="termKey(event)" autocomplete="off" spellcheck="false">
          <button class="btn sm p" onclick="termSend()" style="flex-shrink:0">▶</button>
        </div>
      </div>
    </div>

    <!-- ══ Drag handle: center|right ══ -->
    <div class="drag-h" id="dh-right" data-drag="right"></div>

    <!-- ══ Right Panel: Shard Canvas ══ -->
    <div class="panel panel-right" id="right-panel">
      <div class="ptitle">
        <span>Shard Canvas</span>
        <div class="pa">
          <button class="btn sm" id="sc-tab-out" onclick="shardTab('out')" style="color:var(--accent)">Output</button>
          <button class="btn sm" id="sc-tab-ed"  onclick="shardTab('ed')">Editor</button>
          <button class="btn sm" id="sc-tab-cli" onclick="shardTab('cli')">CLI</button>
          <button class="btn sm" onclick="toggleRight()" title="Collapse panel">▷</button>
          <button class="btn sm" onclick="clearOut()">✕</button>
        </div>
      </div>

      <!-- Tab: Output -->
      <div id="sc-out" style="flex:1;display:flex;flex-direction:column;overflow:hidden">
        <div id="oa" style="flex:1;overflow-y:auto;padding:10px 12px;font-family:var(--mono);font-size:12px;color:var(--text-2);white-space:pre-wrap;word-break:break-word;line-height:1.65"><span class="cm"># Shard output — run a .ai file to see results</span>
</div>
      </div>

      <!-- Tab: Editor (inline .ai editor) -->
      <div id="sc-ed" style="flex:1;display:none;flex-direction:column;overflow:hidden">
        <div style="display:flex;align-items:center;gap:6px;padding:6px 10px;background:var(--surface);border-bottom:1px solid var(--border);flex-shrink:0">
          <input id="sc-ed-name" type="text" value="new_program.ai" placeholder="filename.ai"
            style="flex:1;background:var(--bg);border:1px solid var(--border);border-radius:var(--r-sm);color:var(--accent);font-family:var(--mono);font-size:12px;padding:4px 8px;outline:none">
          <button class="btn sm s" onclick="scEdRun()">▶ Run</button>
          <button class="btn sm p" onclick="scEdSave()">💾 Save</button>
          <button class="btn sm" onclick="scEdLoad()">Open</button>
        </div>
        <textarea id="sc-ed-ta" spellcheck="false"
          style="flex:1;background:var(--bg);color:var(--text);font-family:var(--mono);font-size:13px;border:none;outline:none;padding:12px 14px;resize:none;line-height:1.65"
          placeholder="⍝ Write Aeonmi code here&#10;&#10;fn main() {&#10;  print(&quot;Built by AI for AI&quot;)&#10;}"></textarea>
        <div id="sc-ed-st" style="padding:4px 12px;font-size:11px;color:var(--text-3);background:var(--surface);border-top:1px solid var(--border);flex-shrink:0">Ready</div>
      </div>

      <!-- Tab: CLI (Aeonmi-native terminal) -->
      <div id="sc-cli" style="flex:1;display:none;flex-direction:column;overflow:hidden">
        <div id="sc-cli-out" style="flex:1;overflow-y:auto;padding:10px 12px;font-family:var(--mono);font-size:12px;white-space:pre-wrap;word-break:break-word;line-height:1.6;color:var(--text-2)"><span style="color:var(--purple)">◈ Aeonmi Shell — type commands below</span>
</div>
        <div style="display:flex;align-items:center;gap:6px;padding:7px 10px;border-top:1px solid var(--border);background:var(--surface);flex-shrink:0">
          <span style="color:var(--accent);font-family:var(--mono);font-size:13px;flex-shrink:0">◈</span>
          <input id="sc-cli-in" type="text" placeholder="native main.ai · ls · read file.rs · search …"
            style="flex:1;background:var(--bg);border:1px solid var(--border);border-radius:var(--r-sm);color:var(--text);font-family:var(--mono);font-size:12.5px;padding:5px 9px;outline:none"
            onkeydown="if(event.key==='Enter')scCliSend()">
          <button class="btn sm p" onclick="scCliSend()">▶</button>
        </div>
      </div>

      <!-- Action queue + agents (always visible below) -->
      <div id="aq" style="border-top:1px solid var(--border-muted);flex-shrink:0;max-height:100px;overflow-y:auto;background:var(--surface)">
        <div class="aqt">Action Queue</div>
        <div id="alist"><div class="ai-item" style="font-size:11px">No pending actions</div></div>
      </div>
      <div id="agp" style="border-top:1px solid var(--border-muted);flex-shrink:0;background:var(--surface)">
        <div class="aqt">Agents</div>
        <div id="aglist"><span style="color:var(--text-3);font-size:11px;padding:0 8px">Loading…</span></div>
      </div>
      <div id="sctl" style="padding:6px 8px;border-top:1px solid var(--border-muted);display:flex;gap:4px;flex-wrap:wrap;flex-shrink:0;background:var(--surface)">
        <button class="btn sm" onclick="runShard()" title="Run main.ai shard compiler">▶ Shard</button>
        <button class="btn sm" onclick="runMainAi()" title="Run Aeonmi_Master/aeonmi_ai/shard/main.ai">◈ main.ai</button>
        <button class="btn sm" onclick="runTests()">⚗ Tests</button>
        <button class="btn sm" onclick="buildRelease()">⚙ Build</button>
        <button class="btn sm" onclick="loadActions()">⟳ Queue</button>
      </div>
    </div>
  </div>
</div>

<!-- Glyph Panel Modal (Phase 4b) -->
<div id="gmo" onclick="if(event.target===this)this.style.display='none'" style="display:none;position:fixed;inset:0;background:rgba(0,0,0,.88);backdrop-filter:blur(4px);z-index:300;align-items:center;justify-content:center">
  <div style="background:var(--panel);border:1px solid rgba(168,85,247,.3);border-radius:12px;width:480px;max-width:94vw;box-shadow:0 20px 60px rgba(168,85,247,.15),0 4px 16px rgba(0,0,0,.5)">
    <div style="padding:14px 18px;border-bottom:1px solid var(--border);display:flex;align-items:center;gap:10px;background:var(--surface);border-radius:12px 12px 0 0">
      <span style="font-weight:700;font-size:14px;background:linear-gradient(135deg,var(--purple),var(--magenta));-webkit-background-clip:text;-webkit-text-fill-color:transparent">◈ Living Glyph — Phase 4b</span>
      <span style="flex:1"></span>
      <button class="btn sm" onclick="document.getElementById('gmo').style.display='none'">✕</button>
    </div>
    <div id="gcont" style="padding:20px;display:flex;flex-direction:column;gap:14px">
      <div style="text-align:center;color:var(--text-3);font-size:12px">Loading glyph state…</div>
    </div>
    <div style="padding:10px 18px;border-top:1px solid var(--border);font-size:11px;color:var(--text-3);background:var(--surface);border-radius:0 0 12px 12px">
      Set <code>AEONMI_PASSPHRASE</code> env var to enable boot ceremony. Glyph shifts as bond.strength grows.
    </div>
  </div>
</div>

<!-- Unified Memory Panel Modal (Phase 5) -->
<div id="synmo" onclick="if(event.target===this)this.style.display='none'" style="display:none;position:fixed;inset:0;background:rgba(0,0,0,.88);backdrop-filter:blur(4px);z-index:300;align-items:center;justify-content:center">
  <div style="background:var(--panel);border:1px solid rgba(99,179,237,.3);border-radius:12px;width:520px;max-width:94vw;box-shadow:0 20px 60px rgba(99,179,237,.12),0 4px 16px rgba(0,0,0,.5)">
    <div style="padding:14px 18px;border-bottom:1px solid var(--border);display:flex;align-items:center;gap:10px;background:var(--surface);border-radius:12px 12px 0 0">
      <span style="font-weight:700;font-size:14px;background:linear-gradient(135deg,#63b3ed,#a855f7);-webkit-background-clip:text;-webkit-text-fill-color:transparent">⟳ Unified Memory — Phase 5</span>
      <span style="flex:1"></span>
      <button class="btn sm" onclick="runSync()" id="syncRunBtn" style="background:rgba(99,179,237,.15);border-color:rgba(99,179,237,.4);color:#63b3ed">⟳ Sync Now</button>
      <button class="btn sm" onclick="document.getElementById('synmo').style.display='none'">✕</button>
    </div>
    <div id="synco" style="padding:20px;display:flex;flex-direction:column;gap:14px;font-size:12px">
      <div style="text-align:center;color:var(--text-3);font-size:12px">Click ⟳ Sync Now to reconcile all three tracks…</div>
    </div>
    <div style="padding:10px 18px;border-top:1px solid var(--border);font-size:11px;color:var(--text-3);background:var(--surface);border-radius:0 0 12px 12px">
      Three tracks: <strong>cognitive</strong> (Rust embryo_loop) · <strong>operational</strong> (Python dashboard) · <strong>ai_memory</strong> (.ai programs)
    </div>
  </div>
</div>

<!-- Hive Panel Modal (Phase 8) -->
<div id="hivemo" onclick="if(event.target===this)this.style.display='none'" style="display:none;position:fixed;inset:0;background:rgba(0,0,0,.88);backdrop-filter:blur(4px);z-index:300;align-items:center;justify-content:center">
  <div style="background:var(--panel);border:1px solid rgba(237,137,54,.3);border-radius:12px;width:500px;max-width:94vw;box-shadow:0 20px 60px rgba(237,137,54,.12),0 4px 16px rgba(0,0,0,.5)">
    <div style="padding:14px 18px;border-bottom:1px solid var(--border);display:flex;align-items:center;gap:10px;background:var(--surface);border-radius:12px 12px 0 0">
      <span style="font-weight:700;font-size:14px;background:linear-gradient(135deg,#ed8936,#f6ad55);-webkit-background-clip:text;-webkit-text-fill-color:transparent">◈ Swarm Hive — Phase 8</span>
      <span style="flex:1"></span>
      <span id="hivecyclebadge" style="font-size:10px;color:var(--text-3)">idle</span>
      <button class="btn sm" onclick="hiveRunOnce()" style="background:rgba(237,137,54,.15);border-color:rgba(237,137,54,.4);color:#ed8936">▶ Run Cycle</button>
      <button class="btn sm" onclick="document.getElementById('hivemo').style.display='none'">✕</button>
    </div>
    <div id="hivecont" style="padding:16px 18px;display:flex;flex-direction:column;gap:12px;font-size:12px">
      <div style="text-align:center;color:var(--text-3)">Loading hive state…</div>
    </div>
    <div style="padding:10px 18px;border-top:1px solid var(--border);font-size:11px;color:var(--text-3);background:var(--surface);border-radius:0 0 12px 12px">
      5 agents: <strong>Oracle</strong> (quantum signal) · <strong>Hype</strong> (momentum) · <strong>Closer</strong> (intent) · <strong>Devil</strong> (risk) · <strong>Conductor</strong> (synthesis)
    </div>
  </div>
</div>

<!-- Goal Panel Modal (Phase 7) -->
<div id="goalmo" onclick="if(event.target===this)this.style.display='none'" style="display:none;position:fixed;inset:0;background:rgba(0,0,0,.88);backdrop-filter:blur(4px);z-index:300;align-items:center;justify-content:center">
  <div style="background:var(--panel);border:1px solid rgba(56,161,105,.3);border-radius:12px;width:540px;max-width:94vw;box-shadow:0 20px 60px rgba(56,161,105,.12),0 4px 16px rgba(0,0,0,.5)">
    <div style="padding:14px 18px;border-bottom:1px solid var(--border);display:flex;align-items:center;gap:10px;background:var(--surface);border-radius:12px 12px 0 0">
      <span style="font-weight:700;font-size:14px;background:linear-gradient(135deg,#38a169,#63b3ed);-webkit-background-clip:text;-webkit-text-fill-color:transparent">◈ Agent Goal — Phase 7</span>
      <span style="flex:1"></span>
      <button class="btn sm" onclick="document.getElementById('goalmo').style.display='none'">✕</button>
    </div>
    <!-- New goal input -->
    <div style="padding:14px 18px;border-bottom:1px solid var(--border);display:flex;gap:8px">
      <input id="goalModalInp" type="text" placeholder="Describe Mother's goal…" style="flex:1;background:var(--bg);border:1px solid var(--border);border-radius:var(--r);color:var(--text);font-family:var(--font);font-size:13px;padding:7px 12px;outline:none" onkeydown="if(event.key==='Enter')submitGoalModal()">
      <button class="btn sm" onclick="submitGoalModal()" style="background:rgba(56,161,105,.15);border-color:rgba(56,161,105,.4);color:#38a169">▶ Set Goal</button>
    </div>
    <div id="goalcont" style="padding:16px 18px;display:flex;flex-direction:column;gap:10px;font-size:12px;max-height:380px;overflow-y:auto">
      <div style="text-align:center;color:var(--text-3)">Set a goal above or load current state…</div>
    </div>
    <div style="padding:10px 18px;border-top:1px solid var(--border);display:flex;gap:8px;background:var(--surface);border-radius:0 0 12px 12px">
      <button class="btn sm" onclick="loadGoalState()" style="color:var(--text-2)">↻ Refresh</button>
      <button class="btn sm" id="goalAutoBtn" onclick="autoRunFromModal()" style="background:rgba(56,161,105,.15);border-color:rgba(56,161,105,.4);color:#38a169">⟳ Run Next 5 Steps</button>
      <span style="flex:1"></span>
      <span style="font-size:11px;color:var(--text-3);align-self:center">autonomous agent execution</span>
    </div>
  </div>
</div>

<!-- Generate Panel Modal (Phase 9) -->
<div id="genmo" onclick="if(event.target===this)this.style.display='none'" style="display:none;position:fixed;inset:0;background:rgba(0,0,0,.88);backdrop-filter:blur(4px);z-index:300;align-items:center;justify-content:center">
  <div style="background:var(--panel);border:1px solid rgba(236,72,153,.3);border-radius:12px;width:580px;max-width:94vw;box-shadow:0 20px 60px rgba(236,72,153,.12),0 4px 16px rgba(0,0,0,.5)">
    <div style="padding:14px 18px;border-bottom:1px solid var(--border);display:flex;align-items:center;gap:10px;background:var(--surface);border-radius:12px 12px 0 0">
      <span style="font-weight:700;font-size:14px;background:linear-gradient(135deg,#ec4899,#a855f7);-webkit-background-clip:text;-webkit-text-fill-color:transparent">◈ Self-Generation — Phase 9</span>
      <span style="flex:1"></span>
      <span id="genprogbadge" style="font-size:10px;color:var(--text-3)">idle</span>
      <button class="btn sm" onclick="genPropose()" style="background:rgba(236,72,153,.12);border-color:rgba(236,72,153,.35);color:#ec4899">◈ Propose</button>
      <button class="btn sm" onclick="document.getElementById('genmo').style.display='none'">✕</button>
    </div>
    <!-- Build input -->
    <div style="padding:12px 18px;border-bottom:1px solid var(--border);display:flex;gap:8px;align-items:center">
      <input id="genName" type="text" placeholder="program_name" style="width:140px;background:var(--bg);border:1px solid var(--border);border-radius:var(--r);color:var(--text);font-family:var(--font);font-size:12px;padding:6px 10px;outline:none">
      <input id="genGoal" type="text" placeholder="goal / description…" style="flex:1;background:var(--bg);border:1px solid var(--border);border-radius:var(--r);color:var(--text);font-family:var(--font);font-size:12px;padding:6px 10px;outline:none" onkeydown="if(event.key==='Enter')genBuild()">
      <button class="btn sm" onclick="genBuild()" style="background:rgba(236,72,153,.12);border-color:rgba(236,72,153,.35);color:#ec4899">▶ Build</button>
    </div>
    <div id="gencont" style="padding:14px 18px;display:flex;flex-direction:column;gap:10px;font-size:12px;max-height:360px;overflow-y:auto">
      <div style="text-align:center;color:var(--text-3)">Loading programs…</div>
    </div>
    <div style="padding:10px 18px;border-top:1px solid var(--border);display:flex;gap:8px;background:var(--surface);border-radius:0 0 12px 12px">
      <button class="btn sm" onclick="loadGeneratePanel()" style="color:var(--text-2)">↻ Refresh</button>
      <button class="btn sm" onclick="genReflectAll()" style="background:rgba(168,85,247,.1);border-color:rgba(168,85,247,.3);color:var(--purple)">◈ Reflect All</button>
      <span style="flex:1"></span>
      <span style="font-size:11px;color:var(--text-3);align-self:center">mother writes her own programs</span>
    </div>
  </div>
</div>

<!-- Knowledge Graph Panel Modal (Phase 10) -->
<div id="kgmo" onclick="if(event.target===this)this.style.display='none'" style="display:none;position:fixed;inset:0;background:rgba(0,0,0,.88);backdrop-filter:blur(4px);z-index:300;align-items:center;justify-content:center">
  <div style="background:var(--panel);border:1px solid rgba(56,189,248,.3);border-radius:12px;width:620px;max-width:94vw;box-shadow:0 20px 60px rgba(56,189,248,.1),0 4px 16px rgba(0,0,0,.5)">
    <div style="padding:14px 18px;border-bottom:1px solid var(--border);display:flex;align-items:center;gap:10px;background:var(--surface);border-radius:12px 12px 0 0">
      <span style="font-weight:700;font-size:14px;background:linear-gradient(135deg,#38bdf8,#818cf8);-webkit-background-clip:text;-webkit-text-fill-color:transparent">◈ Knowledge Graph — Phase 10</span>
      <span style="flex:1"></span>
      <span id="kgstatbadge" style="font-size:10px;color:var(--text-3)">loading…</span>
      <button class="btn sm" onclick="document.getElementById('kgmo').style.display='none'">✕</button>
    </div>
    <!-- Tag filter bar -->
    <div style="padding:10px 18px;border-bottom:1px solid var(--border);display:flex;gap:8px;align-items:center;flex-wrap:wrap">
      <span style="font-size:11px;color:var(--text-3)">Filter tag:</span>
      <div id="kgtagbar" style="display:flex;gap:6px;flex-wrap:wrap"></div>
      <span style="flex:1"></span>
      <input id="kgsearch" type="text" placeholder="search key…" style="width:140px;background:var(--bg);border:1px solid var(--border);border-radius:var(--r);color:var(--text);font-family:var(--font);font-size:11px;padding:5px 9px;outline:none" oninput="kgFilter()">
    </div>
    <div id="kgcont" style="padding:14px 18px;display:flex;flex-direction:column;gap:8px;font-size:12px;max-height:400px;overflow-y:auto">
      <div style="color:var(--text-3);text-align:center">Loading knowledge graph…</div>
    </div>
    <div style="padding:10px 18px;border-top:1px solid var(--border);display:flex;gap:8px;background:var(--surface);border-radius:0 0 12px 12px">
      <button class="btn sm" onclick="loadKGPanel()" style="color:var(--text-2)">↻ Refresh</button>
      <span style="flex:1"></span>
      <span style="font-size:11px;color:var(--text-3);align-self:center">linked · tagged · traversable</span>
    </div>
  </div>
</div>

<!-- Inner Voice Panel Modal (Phase 11) -->
<div id="voicemo" onclick="if(event.target===this)this.style.display='none'" style="display:none;position:fixed;inset:0;background:rgba(0,0,0,.88);backdrop-filter:blur(4px);z-index:300;align-items:center;justify-content:center">
  <div style="background:var(--panel);border:1px solid rgba(129,140,248,.3);border-radius:12px;width:580px;max-width:94vw;box-shadow:0 20px 60px rgba(129,140,248,.1),0 4px 16px rgba(0,0,0,.5)">
    <div style="padding:14px 18px;border-bottom:1px solid var(--border);display:flex;align-items:center;gap:10px;background:var(--surface);border-radius:12px 12px 0 0">
      <span style="font-weight:700;font-size:14px;background:linear-gradient(135deg,#818cf8,#c084fc);-webkit-background-clip:text;-webkit-text-fill-color:transparent">◈ Inner Voice — Phase 11</span>
      <span style="flex:1"></span>
      <span id="voicestatbadge" style="font-size:10px;color:var(--text-3)">loading…</span>
      <button class="btn sm" onclick="voiceDream()" style="background:rgba(129,140,248,.12);border-color:rgba(129,140,248,.35);color:#818cf8">◈ Dream</button>
      <button class="btn sm" onclick="document.getElementById('voicemo').style.display='none'">✕</button>
    </div>
    <div id="voicecont" style="padding:14px 18px;display:flex;flex-direction:column;gap:8px;font-size:12px;max-height:420px;overflow-y:auto">
      <div style="color:var(--text-3);text-align:center">Loading thoughts…</div>
    </div>
    <div style="padding:10px 18px;border-top:1px solid var(--border);display:flex;gap:8px;background:var(--surface);border-radius:0 0 12px 12px">
      <button class="btn sm" onclick="loadVoicePanel()" style="color:var(--text-2)">↻ Refresh</button>
      <span style="flex:1"></span>
      <span style="font-size:11px;color:var(--text-3);align-self:center">inner monologue · dream consolidation</span>
    </div>
  </div>
</div>

<!-- Settings Modal -->
<div id="smo" onclick="smo_ci(event)">
  <div id="smbox">
    <div id="smtop">
      <span>⚙ API Keys</span>
      <span style="flex:1"></span>
      <button class="btn sm" onclick="closeSmo()">✕</button>
    </div>
    <div id="smcont"><div style="color:var(--muted);font-size:12px;padding:8px">Loading…</div></div>
    <div id="smfoot">Keys are stored in <code>.env</code> at project root and loaded on every start. Provider fallback: Claude → OpenRouter → OpenAI → DeepSeek → Grok → Perplexity.</div>
  </div>
</div>

<!-- Editor Overlay -->
<div id="eo" onclick="eoci(event)">
  <div id="ebox">
    <div id="etop">
      <span id="efn">untitled.ai</span>
      <button class="btn sm s" onclick="runEd()">▶ Run</button>
      <button class="btn sm p" onclick="saveEd()">💾 Save</button>
      <button class="btn sm" onclick="closeEd()">✕</button>
    </div>
    <div style="flex:1;overflow:hidden;display:flex">
      <textarea id="eta" spellcheck="false"></textarea>
    </div>
    <div id="est">Ready</div>
  </div>
</div>

<script>
let selFile=null,curPath="",edFile=null;

// ══ RESIZABLE PANELS ══════════════════════════════════════════════════════
(function(){
  const R=document.documentElement;
  // Restore saved sizes
  const lw=localStorage.getItem('aeonmi_lw'),rw=localStorage.getItem('aeonmi_rw'),th=localStorage.getItem('aeonmi_th');
  if(lw)R.style.setProperty('--lw',lw+'px');
  if(rw)R.style.setProperty('--rw',rw+'px');
  if(th)R.style.setProperty('--th',th+'px');
  if(localStorage.getItem('aeonmi_lc'))_collapseLeft(true);
  if(localStorage.getItem('aeonmi_rc'))_collapseRight(true);
  if(localStorage.getItem('aeonmi_tc'))document.getElementById('term-pane')?.classList.add('hidden');

  let _drag=null;
  document.addEventListener('mousedown',e=>{
    const h=e.target.closest('.drag-h,.drag-v');
    if(!h)return;
    e.preventDefault();
    _drag={
      el:h,type:h.dataset.drag,
      x0:e.clientX,y0:e.clientY,
      lw0:parseInt(R.style.getPropertyValue('--lw')||'220'),
      rw0:parseInt(R.style.getPropertyValue('--rw')||'320'),
      th0:parseInt(R.style.getPropertyValue('--th')||'220'),
    };
    h.classList.add('dragging');
    document.body.style.cursor=h.classList.contains('drag-v')?'row-resize':'col-resize';
    document.body.style.userSelect='none';
  });
  document.addEventListener('mousemove',e=>{
    if(!_drag)return;
    const dx=e.clientX-_drag.x0,dy=e.clientY-_drag.y0;
    if(_drag.type==='left'){
      const nw=Math.max(120,Math.min(540,_drag.lw0+dx));
      R.style.setProperty('--lw',nw+'px');
    } else if(_drag.type==='right'){
      const nw=Math.max(160,Math.min(680,_drag.rw0-dx));
      R.style.setProperty('--rw',nw+'px');
    } else if(_drag.type==='term'){
      const nw=Math.max(60,Math.min(600,_drag.th0-dy));
      R.style.setProperty('--th',nw+'px');
    }
  });
  document.addEventListener('mouseup',e=>{
    if(!_drag)return;
    const R2=document.documentElement;
    if(_drag.type==='left')localStorage.setItem('aeonmi_lw',parseInt(R2.style.getPropertyValue('--lw')||'220'));
    if(_drag.type==='right')localStorage.setItem('aeonmi_rw',parseInt(R2.style.getPropertyValue('--rw')||'320'));
    if(_drag.type==='term')localStorage.setItem('aeonmi_th',parseInt(R2.style.getPropertyValue('--th')||'220'));
    _drag.el.classList.remove('dragging');
    document.body.style.cursor='';document.body.style.userSelect='';
    _drag=null;
  });
})();

function _collapseLeft(silent){
  const p=document.getElementById('left-panel');
  const h=document.getElementById('dh-left');
  if(!p)return;
  p.classList.add('collapsed');
  document.documentElement.style.setProperty('--lw','0px');
  if(h){h.classList.add('expand-handle');h.title='Click to expand Explorer';h.dataset.expandSide='left';}
  if(!silent)localStorage.setItem('aeonmi_lc','1');
}
function _expandLeft(){
  const p=document.getElementById('left-panel');
  const h=document.getElementById('dh-left');
  if(!p)return;
  p.classList.remove('collapsed');
  const w=localStorage.getItem('aeonmi_lw')||'220';
  document.documentElement.style.setProperty('--lw',w+'px');
  if(h){h.classList.remove('expand-handle');h.title='';delete h.dataset.expandSide;}
  localStorage.removeItem('aeonmi_lc');
}
function toggleLeft(){
  const p=document.getElementById('left-panel');
  if(!p)return;
  if(p.classList.contains('collapsed'))_expandLeft();
  else _collapseLeft();
}

function _collapseRight(silent){
  const p=document.getElementById('right-panel');
  const h=document.getElementById('dh-right');
  if(!p)return;
  p.classList.add('collapsed');
  document.documentElement.style.setProperty('--rw','0px');
  if(h){h.classList.add('expand-handle');h.title='Click to expand Shard Canvas';h.dataset.expandSide='right';}
  if(!silent)localStorage.setItem('aeonmi_rc','1');
}
function _expandRight(){
  const p=document.getElementById('right-panel');
  const h=document.getElementById('dh-right');
  if(!p)return;
  p.classList.remove('collapsed');
  const w=localStorage.getItem('aeonmi_rw')||'320';
  document.documentElement.style.setProperty('--rw',w+'px');
  if(h){h.classList.remove('expand-handle');h.title='';delete h.dataset.expandSide;}
  localStorage.removeItem('aeonmi_rc');
}
function toggleRight(){
  const p=document.getElementById('right-panel');
  if(!p)return;
  if(p.classList.contains('collapsed'))_expandRight();
  else _collapseRight();
}

// Click on expand-handle drag bar to restore collapsed panel
document.addEventListener('click',e=>{
  const h=e.target.closest('.drag-h.expand-handle');
  if(!h)return;
  if(h.dataset.expandSide==='left')_expandLeft();
  else if(h.dataset.expandSide==='right')_expandRight();
});

function toggleTermPane(){
  const p=document.getElementById('term-pane');
  if(!p)return;
  const hidden=p.classList.toggle('hidden');
  if(hidden)localStorage.setItem('aeonmi_tc','1');
  else localStorage.removeItem('aeonmi_tc');
}

// ══ REAL TERMINAL ══════════════════════════════════════════════════════════
let _termES=null,_termHist=[],_termHIdx=0,_termCwd='';

function termOut(text,cls=''){
  const out=document.getElementById('term-out');
  if(!out)return;
  const d=document.createElement('div');
  if(cls)d.className=cls;
  // Escape HTML
  d.textContent=text;
  out.appendChild(d);
  out.scrollTop=out.scrollHeight;
}

function clearTerm(){
  const out=document.getElementById('term-out');
  if(out)out.innerHTML='<div class="t-cm">$ Terminal cleared</div>';
}

function termKill(){
  if(_termES){_termES.close();_termES=null;}
  document.getElementById('term-kill-btn').style.display='none';
  termOut('[process killed]','t-warn');
}

function termKey(e){
  if(e.key==='Enter'){e.preventDefault();termSend();return;}
  if(e.key==='ArrowUp'){
    e.preventDefault();
    if(_termHIdx<_termHist.length){_termHIdx++;e.target.value=_termHist[_termHIdx-1]||'';}
    return;
  }
  if(e.key==='ArrowDown'){
    e.preventDefault();
    _termHIdx=Math.max(0,_termHIdx-1);
    e.target.value=_termHIdx>0?_termHist[_termHIdx-1]:'';
    return;
  }
  if(e.key==='c'&&e.ctrlKey){termKill();return;}
}

function termSend(){
  const inp=document.getElementById('term-in');
  if(!inp)return;
  const cmd=inp.value.trim();
  if(!cmd)return;
  inp.value='';
  _termHist.unshift(cmd);_termHIdx=0;
  if(_termHist.length>200)_termHist.length=200;

  // Handle cd locally
  if(cmd.startsWith('cd ')||cmd==='cd'){
    const target=cmd.slice(3).trim()||'';
    // Tell server to resolve path via a quick shell call
    post('/api/shell',{cmd:`cd ${target||'.'} && cd`})
      .then(d=>{ if(d.ok){_termCwd=d.output.trim();updateTermPwd();}else termOut(d.output,'t-er');})
      .catch(e=>termOut(String(e),'t-er'));
    return;
  }

  termOut(`$ ${cmd}`,'t-cm');
  if(_termES){_termES.close();_termES=null;}

  const killBtn=document.getElementById('term-kill-btn');
  if(killBtn)killBtn.style.display='';

  _termES=new EventSource(`/api/shell/stream?cmd=${encodeURIComponent(cmd)}&cwd=${encodeURIComponent(_termCwd||'')}`);
  _termES.onmessage=e=>{
    try{
      const d=JSON.parse(e.data);
      if(d.error!==undefined){termOut(d.error,'t-er');_termES.close();_termES=null;if(killBtn)killBtn.style.display='none';}
      else if(d.exit!==undefined){
        if(d.exit!==0)termOut(`[exit ${d.exit}]`,'t-warn');
        _termES.close();_termES=null;if(killBtn)killBtn.style.display='none';
        loadTree(); // refresh file tree after any command
      } else if(d.out!==undefined){
        // Color based on content hints
        const line=d.out;
        const cls=line.match(/error|failed|cannot|denied/i)?'t-er':
                  line.match(/warning|warn/i)?'t-warn':
                  line.match(/ok|success|done|finished|created|saved/i)?'t-ok':'';
        termOut(line,cls);
      }
    }catch(ex){}
  };
  _termES.onerror=()=>{termOut('[stream closed]','t-dim');_termES=null;if(killBtn)killBtn.style.display='none';};
}

function updateTermPwd(){
  const el=document.getElementById('term-pwd');
  if(el&&_termCwd)el.textContent=_termCwd.replace(/.*[\\/]/,'…/').slice(-30);
}

// ══ FILE EXPLORER ENHANCEMENTS ════════════════════════════════════════════
let _treeCache=[];

function filterTree(query){
  const q=query.toLowerCase().trim();
  const items=document.querySelectorAll('#ftree .ti');
  items.forEach(it=>{
    it.style.display=(!q||it.textContent.toLowerCase().includes(q))?'':'none';
  });
}

// Context menu
let _ctxMenu=null;
function showCtxMenu(e,path,isDir){
  e.preventDefault();
  removeCtxMenu();
  const m=document.createElement('div');
  m.id='ctx-menu';
  m.style.cssText=`position:fixed;top:${e.clientY}px;left:${e.clientX}px;
    background:var(--panel);border:1px solid var(--border);border-radius:var(--r);
    box-shadow:var(--shadow-lg);z-index:1000;min-width:160px;padding:4px 0`;
  const item=(label,fn,danger=false)=>{
    const d=document.createElement('div');
    d.style.cssText=`padding:7px 14px;cursor:pointer;font-size:12px;
      color:${danger?'var(--error)':'var(--text)'};transition:var(--t)`;
    d.textContent=label;
    d.onmouseover=()=>d.style.background='var(--panel-hover)';
    d.onmouseout=()=>d.style.background='';
    d.onclick=()=>{removeCtxMenu();fn();};
    m.appendChild(d);
  };
  const sep=()=>{const d=document.createElement('div');d.style.cssText='height:1px;background:var(--border-muted);margin:3px 0';m.appendChild(d);};
  if(!isDir){
    item('▶ Run',()=>{selFile=path;runSel();});
    item('✎ Edit',()=>{openEd(path);});
    sep();
  }
  item('✏ Rename',()=>{doRename(path);});
  item('⎘ Copy',()=>{doCopy(path);});
  sep();
  item('✕ Delete',()=>{selFile=path;delSel();},true);
  document.body.appendChild(m);
  _ctxMenu=m;
  setTimeout(()=>document.addEventListener('click',removeCtxMenu,{once:true}),10);
}
function removeCtxMenu(){if(_ctxMenu){_ctxMenu.remove();_ctxMenu=null;}}

async function doRename(path){
  const parts=path.split('/');
  const oldName=parts[parts.length-1];
  const newName=prompt('Rename to:',oldName);
  if(!newName||newName===oldName)return;
  const d=await post('/api/rename',{old:path,new:newName});
  if(d.ok)loadTree();
  else alert('Rename failed: '+d.error);
}
async function doCopy(path){
  const newPath=prompt('Copy to path:',path.replace(/(\.[^.]+)$/,'_copy$1'));
  if(!newPath)return;
  const d=await post('/api/copy',{src:path,dst:newPath});
  if(d.ok)loadTree();
  else alert('Copy failed: '+d.error);
}

async function renameSel(){
  if(!selFile)return;
  await doRename(selFile);
}

// Patch loadTree to add context menus and remove item limit


// Status
async function refreshStatus(){
  try{
    const s=await fetch("/api/status").then(r=>r.json());
    const b=document.getElementById("sbadge");
    if(s.binary_found){b.textContent="● Online";b.style.color="var(--success)";b.style.borderColor="var(--success)"}
    else{b.textContent="⚠ Binary missing";b.style.color="var(--warn)";b.style.borderColor="var(--warn)"}
  }catch{document.getElementById("sbadge").textContent="● Offline"}
  // Phase 12 — update bond phrase badge
  try{
    const bp=await fetch("/api/bond_phrase").then(r=>r.json());
    const bbadge=document.getElementById("bondbadge");
    if(bbadge&&bp.ok) bbadge.textContent=`◈ ${bp.phrase} (${bp.bond.toFixed(3)})`;
  }catch(e){}
}

// File tree
async function loadTree(path=""){
  const data=await fetch("/api/files?path="+encodeURIComponent(path)+"&limit=500").then(r=>r.json()).catch(()=>({entries:[]}));
  curPath=path;
  document.getElementById("fbread").textContent="/"+(path||"");
  // Clear search filter
  const si=document.getElementById('ftree-search');if(si)si.value='';
  const t=document.getElementById("ftree");t.innerHTML="";
  if(path){
    const u=el("div","ti");u.innerHTML='<span class="ic">↑</span><span class="nm">..</span>';
    u.onclick=()=>loadTree(path.split("/").slice(0,-1).join("/"));t.appendChild(u);
  }
  for(const e of data.entries||[]){
    const item=el("div","ti");item.dataset.path=e.path;item.dataset.type=e.type;
    let ic="ic",ch="·";
    if(e.type==="dir"){ic+=" d";ch="▸"}
    else if(e.ext==="ai"){ic+=" a";ch="◈"}
    else if(e.ext==="qube"){ic+=" q";ch="⟦"}
    else if(e.ext==="rs")ch="⚙";
    else if(e.ext==="toml")ch="⊟";
    else if(e.ext==="md")ch="≡";
    else if(e.ext==="py")ch="🐍";
    else if(e.ext==="json")ch="{}";
    else if(e.ext==="txt")ch="≡";
    item.innerHTML=`<span class="${ic}">${ch}</span><span class="nm">${esc(e.name)}</span>`;
    if(e.type==="dir"){
      item.onclick=()=>loadTree(e.path);
      item.oncontextmenu=ev=>showCtxMenu(ev,e.path,true);
    } else {
      item.onclick=()=>selItem(item,e.path,e.ext);
      item.oncontextmenu=ev=>showCtxMenu(ev,e.path,false);
    }
    t.appendChild(item);
  }
}

function selItem(item,path,ext){
  document.querySelectorAll(".ti.sel").forEach(x=>x.classList.remove("sel"));
  item.classList.add("sel");selFile=path;
  const run=["ai","qube"].includes(ext||"");
  document.getElementById("br").disabled=!run;
  document.getElementById("bc").disabled=ext!=="ai";
  document.getElementById("be").disabled=false;
  document.getElementById("brn").disabled=false;
  document.getElementById("bd").disabled=false;
}

// File actions
async function runSel(){
  if(!selFile)return;ao(`\\n▶ run ${selFile}`,"cm");
  const d=await post("/api/run",{path:selFile});
  ao(d.output,d.ok?"ok":"er");
}
async function compileSel(){
  if(!selFile)return;ao(`\\n⟨/⟩ compile ${selFile}`,"cm");
  const d=await post("/api/compile",{path:selFile});
  ao(d.output,d.ok?"ok":"er");if(d.artifact)ao("→ "+d.artifact,"ok");
}
async function editSel(){if(selFile)openEd(selFile)}
async function delSel(){
  if(!selFile||!confirm("Delete "+selFile+"?"))return;
  const d=await fetch("/api/file?path="+encodeURIComponent(selFile),{method:"DELETE"}).then(r=>r.json());
  if(d.ok){loadTree(curPath);selFile=null}else alert("Error: "+d.error);
}
async function newFile(){
  const n=prompt("File name (e.g. script.ai):");if(!n)return;
  const p=curPath?curPath+"/"+n:n;
  await post("/api/file",{path:p,content:`⍝ ${n}\\n`});
  loadTree(curPath);openEd(p);
}
async function newFolder(){
  const n=prompt("Folder name:");if(!n)return;
  const p=curPath?curPath+"/"+n:n;
  await post("/api/mkdir",{path:p});loadTree(curPath);
}

// Editor
async function openEd(path){
  edFile=path;document.getElementById("efn").textContent=path;
  document.getElementById("eta").value="Loading…";
  document.getElementById("eo").classList.add("open");
  try{
    const d=await fetch("/api/file?path="+encodeURIComponent(path)).then(r=>r.json());
    document.getElementById("eta").value=d.content||"";
    document.getElementById("est").textContent=path+" — "+(d.size||0).toLocaleString()+" bytes";
  }catch{document.getElementById("eta").value=""}
  document.getElementById("eta").focus();
}
function closeEd(){document.getElementById("eo").classList.remove("open");edFile=null}
function eoci(e){if(e.target===document.getElementById("eo"))closeEd()}
async function saveEd(){
  if(!edFile)return;
  const c=document.getElementById("eta").value;
  const d=await post("/api/file",{path:edFile,content:c});
  document.getElementById("est").textContent=d.ok?"Saved — "+c.length+" chars":"Save failed";
}
async function runEd(){if(!edFile)return;await saveEd();closeEd();selFile=edFile;runSel()}
document.addEventListener("keydown",e=>{
  if(e.ctrlKey&&e.key==="s"&&edFile){e.preventDefault();saveEd()}
  if(e.key==="Escape"&&edFile)closeEd();
  // Ctrl+S in Shard editor
  if(e.ctrlKey&&e.key==="s"&&document.getElementById("sc-ed").style.display!=="none"){e.preventDefault();scEdSave()}
});

// Chat
async function send(){
  const mi=document.getElementById("mi");const msg=mi.value.trim();if(!msg)return;
  mi.value="";ar(mi);addMsg("user",msg);
  const btn=document.getElementById("sbtn");btn.disabled=true;btn.innerHTML='<span class="spin"></span>';
  try{
    const d=await post("/api/chat",{message:msg});
    if(d.error)addMsg("mother","Error: "+d.error);
    else{
      const resp=d.response||"(no response)";
      addMsg("mother",resp);
      speak(resp);
      if(d.output)ao(d.output,d.ok?"ok":"er");
    }
    loadActions();
  }catch(e){addMsg("mother","Connection error: "+e.message)}
  finally{btn.disabled=false;btn.textContent="Send";mi.focus()}
}
function hk(e){if(e.key==="Enter"&&!e.shiftKey){e.preventDefault();send()}}
function ar(el){el.style.height="42px";el.style.height=Math.min(el.scrollHeight,120)+"px"}

function addMsg(role,content){
  const msgs=document.getElementById("msgs");
  const d=el("div","msg "+role);
  const label=role==="user"?"You":role==="mother"?"◈ Mother":"System";
  const now=new Date().toLocaleTimeString();
  const html=esc(content).replace(/```[\\s\\S]*?```/g,m=>`<pre>${m.slice(3,-3).trim()}</pre>`);
  d.innerHTML=`<div class="ml">${label}</div><div class="mb">${html}</div><div class="mts">${now}</div>`;
  msgs.appendChild(d);msgs.scrollTop=msgs.scrollHeight;
}
function clearChat(){
  document.getElementById("msgs").innerHTML='<div class="msg system"><div class="mb">Conversation cleared.</div></div>';
}

// ── Shard Canvas tabs ─────────────────────────────────────────────────────────
function shardTab(name){
  ["out","ed","cli"].forEach(t=>{
    const el_=document.getElementById("sc-"+t);
    const btn=document.getElementById("sc-tab-"+t);
    if(el_)el_.style.display=(t===name)?"flex":"none";
    if(btn){btn.style.color=t===name?"var(--accent)":"";btn.style.borderColor=t===name?"rgba(0,212,255,.3)":"";}
  });
}

// Editor tab
async function scEdSave(){
  const name=document.getElementById("sc-ed-name").value.trim()||"new_program.ai";
  const content=document.getElementById("sc-ed-ta").value;
  const d=await post("/api/file",{path:name,content});
  document.getElementById("sc-ed-st").textContent=d.ok?"Saved: "+name+" ("+content.length+" chars)":"Error: "+(d.error||"save failed");
  loadTree();
}
async function scEdRun(){
  await scEdSave();
  const name=document.getElementById("sc-ed-name").value.trim();
  shardTab("out");ao("\\n▶ "+name,"cm");
  const d=await post("/api/run",{path:name});
  ao(d.output||(d.error||"no output"),d.ok?"ok":"er");
}
async function scEdLoad(){
  const name=prompt("File path to open:");if(!name)return;
  document.getElementById("sc-ed-name").value=name;
  try{
    const d=await fetch("/api/file?path="+encodeURIComponent(name)).then(r=>r.json());
    document.getElementById("sc-ed-ta").value=d.content||"";
    document.getElementById("sc-ed-st").textContent="Opened: "+name+" ("+(d.size||0)+" bytes)";
  }catch(e){document.getElementById("sc-ed-st").textContent="Error: "+e.message}
}

// CLI tab — sends to Mother /api/chat, outputs to CLI pane
let _cliHist=[],_cliIdx=0;
async function scCliSend(){
  const inp=document.getElementById("sc-cli-in");
  const cmd=inp.value.trim();if(!cmd)return;
  _cliHist.unshift(cmd);_cliIdx=0;
  inp.value="";
  cliOut("◈ "+cmd,"cm");
  try{
    const d=await post("/api/chat",{message:cmd});
    const text=d.response||d.error||"(no output)";
    cliOut(text,d.ok===false?"er":"ok");
    // Also refresh file tree if it was a write/mkdir/delete
    if(/^(write|append|mkdir|delete|rm|native|run) /i.test(cmd))loadTree();
    loadActions();
  }catch(e){cliOut("Error: "+e.message,"er")}
}
function cliOut(text,cls=""){
  const out=document.getElementById("sc-cli-out");
  const d=document.createElement("div");
  d.className=cls==="cm"?"cm":cls==="ok"?"ok":cls==="er"?"er":"";
  d.style.whiteSpace="pre-wrap";d.style.wordBreak="break-word";
  d.textContent=text;out.appendChild(d);out.scrollTop=out.scrollHeight;
}
document.addEventListener("keydown",e=>{
  const cli=document.getElementById("sc-cli-in");
  if(document.activeElement===cli){
    if(e.key==="ArrowUp"&&_cliHist.length){e.preventDefault();cli.value=_cliHist[Math.min(_cliIdx,_cliHist.length-1)];_cliIdx=Math.min(_cliIdx+1,_cliHist.length-1)}
    if(e.key==="ArrowDown"){_cliIdx=Math.max(0,_cliIdx-1);cli.value=_cliHist[_cliIdx]||""}
  }
});

async function runMainAi(){
  shardTab("out");
  ao("\\n◈ aeonmi native main.ai","cm");
  const d=await post("/api/run",{path:"Aeonmi_Master/aeonmi_ai/shard/main.ai"});
  ao(d.output||(d.error||"no output"),d.ok?"ok":"er");
}

// Shard controls
async function runShard(){
  shardTab("out");ao("\\n▶ Shard compiler","cm");
  const d=await post("/api/shard/run",{});ao(d.output,d.ok?"ok":"er");
}
async function runTests(){
  ao("\\n⚗ cargo test --all --quiet","cm");
  const d=await post("/api/test",{});ao(d.output,d.ok?"ok":"er");
}
async function buildRelease(){
  ao("\\n⚙ cargo build --release","cm");
  const d=await post("/api/build",{release:true});ao(d.output,d.ok?"ok":"er");
}

async function loadActions(){
  try{
    const d=await fetch("/api/actions").then(r=>r.json());
    const list=document.getElementById("alist");
    if(!d.queue||!d.queue.length){
      list.innerHTML='<div class="ai-item" style="color:var(--muted);font-size:11px">No pending actions</div>';
    }else{
      list.innerHTML=d.queue.slice(0,8).map(a=>`<div class="ai-item"><div class="dot"></div>${esc(a)}</div>`).join("");
    }
  }catch{}
}

async function loadAgents(){
  try{
    const d=await fetch("/api/agents").then(r=>r.json());
    const list=document.getElementById("aglist");
    if(!d.agents||!d.agents.length){list.innerHTML='<span style="color:var(--muted);font-size:11px;padding:0 8px">No agents</span>';return;}
    list.innerHTML=d.agents.map(a=>
      `<button class="ag-btn${a.exists?'':'  miss'}" onclick="runAgent('${esc(a.name)}')" ${a.exists?'':'disabled'} title="${a.file}">` +
      `<span class="ag-dot"></span>${esc(a.name)}</button>`
    ).join("");
  }catch{}
}

async function runAgent(name){
  ao(`\\n◈ agent:${name}`,"cm");
  try{
    const d=await post("/api/agent",{agent:name});
    ao(d.output||"(no output)",d.ok?"ok":"er");
    if(!d.ok&&d.error)ao(d.error,"er");
    loadActions();
  }catch(e){ao("Agent error: "+e.message,"er")}
}

// Output
function ao(text,cls=""){
  const a=document.getElementById("oa");
  const d=el("div",cls==="cm"?"cm":cls==="ok"?"ok":cls==="er"?"er":"");
  d.textContent=text;a.appendChild(d);a.scrollTop=a.scrollHeight;
}
function clearOut(){document.getElementById("oa").innerHTML='<span class="cm"># cleared</span>\\n'}

// ── Settings modal ────────────────────────────────────────────────────────────
async function openSettings(){
  document.getElementById("smo").classList.add("open");
  const d=await fetch("/api/keys").then(r=>r.json()).catch(()=>({keys:[]}));
  const c=document.getElementById("smcont");
  const labels={
    "ANTHROPIC_API_KEY":"Anthropic (Claude)",
    "OPENROUTER_API_KEY":"OpenRouter (free tier available)",
    "OPENAI_API_KEY":"OpenAI",
    "DEEPSEEK_API_KEY":"DeepSeek",
    "GROK_API_KEY":"Grok (xAI)",
    "PERPLEXITY_API_KEY":"Perplexity",
    "GITHUB_TOKEN":"GitHub Token (repo read access)"
  };
  const placeholders={
    "ANTHROPIC_API_KEY":"sk-ant-…",
    "OPENROUTER_API_KEY":"sk-or-v1-…",
    "OPENAI_API_KEY":"sk-…",
    "DEEPSEEK_API_KEY":"sk-…",
    "GROK_API_KEY":"xai-…",
    "PERPLEXITY_API_KEY":"pplx-…",
    "GITHUB_TOKEN":"ghp_… or github_pat_…"
  };
  c.innerHTML=d.keys.map(k=>`
    <div class="sk-row">
      <div class="sk-label">${labels[k.name]||k.name}</div>
      <div class="sk-meta">
        <span class="sk-status${k.set?' on':''}">${k.set?('✓ '+esc(k.masked)):'not set'}</span>
        ${k.set?`<button class="btn sm dx" onclick="delKey('${esc(k.name)}')">Remove</button>`:''}
      </div>
      <div class="sk-actions">
        <input type="password" id="ki_${k.name}" class="sk-inp" placeholder="${placeholders[k.name]||'key…'}">
        <button class="btn sm s" onclick="saveKey('${esc(k.name)}')">Save</button>
      </div>
    </div>
  `).join('<hr style="border:none;border-top:1px solid var(--border)">');
}

function closeSmo(){document.getElementById("smo").classList.remove("open")}
function smo_ci(e){if(e.target===document.getElementById("smo"))closeSmo()}

async function saveKey(name){
  const inp=document.getElementById("ki_"+name);
  const val=inp?inp.value.trim():"";
  if(!val){inp&&inp.focus();return;}
  const d=await post("/api/keys",{name,value:val});
  if(d.ok){inp.value="";await openSettings();refreshStatus();}
  else alert("Error: "+d.error);
}

async function delKey(name){
  if(!confirm("Remove "+name+" from .env?"))return;
  await fetch("/api/keys/"+encodeURIComponent(name),{method:"DELETE"});
  await openSettings();
}

// ── Voice input (STT) ─────────────────────────────────────────────────────────
let _recog=null,_recogOn=false;

function initVoice(){
  const SR=window.SpeechRecognition||window.webkitSpeechRecognition;
  const btn=document.getElementById("mibtn");
  if(!SR){btn.style.display="none";return;}
  _recog=new SR();
  _recog.continuous=true;      // keep recording until user taps again
  _recog.interimResults=true;
  _recog.lang="en-US";
  let _finalTranscript="";
  _recog.onresult=e=>{
    let interim="";
    for(let i=e.resultIndex;i<e.results.length;i++){
      if(e.results[i].isFinal) _finalTranscript+=e.results[i][0].transcript+" ";
      else interim+=e.results[i][0].transcript;
    }
    const mi=document.getElementById("mi");
    mi.value=(_finalTranscript+interim).trim();
    ar(mi);
  };
  _recog.onend=()=>{
    // Only stop visually — if user tapped stop, _recogOn is already false
    if(_recogOn){
      // Browser ended it unexpectedly (timeout) — restart to keep going
      try{_recog.start();}catch(e){}
    } else {
      btn.textContent="🎤";btn.classList.remove("active");
    }
  };
  _recog.onerror=e=>{
    if(e.error==="no-speech")return; // ignore silence, keep going
    _recogOn=false;btn.textContent="🎤";btn.classList.remove("active");
    const msgs={
      "not-allowed":  "Microphone access denied. Click the 🔒 or camera icon in your browser address bar and allow the microphone, then refresh.",
      "service-not-allowed": "Speech service blocked. Open the dashboard over http://127.0.0.1:5000 (not file://) and allow mic in browser settings.",
      "network":      "Speech recognition needs an internet connection (audio is processed by the browser's cloud service).",
      "audio-capture":"No microphone found. Check that a mic is plugged in and not muted in Windows sound settings.",
      "aborted":      null, // user cancelled, no message needed
    };
    const msg=msgs[e.error];
    if(msg) alert("🎤 Mic error: "+msg);
    else if(e.error && e.error!=="aborted") alert("🎤 Speech error: "+e.error+"\n\nTry Chrome or Edge. Make sure mic is allowed for this page.");
  };
  // Store transcript ref so toggleMic can reset it
  _recog._resetTranscript=()=>{ _finalTranscript=""; };
}

function toggleMic(){
  if(!_recog){alert("Speech recognition not supported in this browser (use Chrome or Edge).");return;}
  const btn=document.getElementById("mibtn");
  if(_recogOn){
    // Stop — send whatever was captured
    _recogOn=false;
    _recog.stop();
    btn.textContent="🎤";btn.classList.remove("active");
    const mi=document.getElementById("mi");
    if(mi.value.trim()) send();
    if(_recog._resetTranscript) _recog._resetTranscript();
  } else {
    // Start
    if(_recog._resetTranscript) _recog._resetTranscript();
    document.getElementById("mi").value="";
    _recog.start();_recogOn=true;
    btn.textContent="⏹";btn.classList.add("active");
    btn.title="Recording… tap to stop and send";
  }
}

// ── Voice output (TTS) ────────────────────────────────────────────────────────
let _ttsOn=false;

function toggleTTS(){
  _ttsOn=!_ttsOn;
  const btn=document.getElementById("ttsbtn");
  btn.textContent=_ttsOn?"🔊":"🔈";
  btn.classList.toggle("tts",true);
  btn.classList.toggle("active",_ttsOn);
  btn.title=_ttsOn?"Voice on — click to mute":"Voice off — click to enable";
  if(!_ttsOn&&window.speechSynthesis)window.speechSynthesis.cancel();
}

function speak(text){
  if(!_ttsOn||!window.speechSynthesis)return;
  window.speechSynthesis.cancel();
  const utt=new SpeechSynthesisUtterance(text.replace(/[◈⟦✓✗⚙⚗⟳●]/g,""));
  utt.rate=0.95;utt.pitch=0.88;
  const voices=window.speechSynthesis.getVoices();
  const pref=voices.find(v=>v.name.includes("Aria")||v.name.includes("Zira")||v.name.includes("Samantha")||v.name.toLowerCase().includes("female"));
  if(pref)utt.voice=pref;
  window.speechSynthesis.speak(utt);
}

// ── File / image upload ───────────────────────────────────────────────────────
async function uploadFiles(files){
  if(!files||!files.length)return;
  for(const file of files){
    addMsg("system",`Uploading ${file.name}…`);
    const fd=new FormData();
    fd.append("file",file);
    fd.append("dest","uploads");
    try{
      const r=await fetch("/api/upload",{method:"POST",body:fd});
      const d=await r.json();
      if(d.ok){
        let note=`Uploaded: ${d.path} (${(d.size/1024).toFixed(1)} KB)`;
        // If docx preview came back, send it to Mother automatically
        if(d.preview){
          note+=`\n\nFile content loaded. Asking Mother to read it…`;
          const mi=document.getElementById("mi");
          mi.value=`I just uploaded ${d.name}. Here is its content:\n\n${d.preview.slice(0,3000)}`;
          addMsg("system",note);
          await send();
        } else if(/\.(png|jpg|jpeg|gif|webp|svg)$/i.test(file.name)){
          note+=`\n\nImage saved to uploads/${d.name}`;
          addMsg("system",note);
          // Tell Mother about the image
          document.getElementById("mi").value=`I uploaded an image: uploads/${d.name}`;
          await send();
        } else {
          addMsg("system",note);
          document.getElementById("mi").value=`I uploaded a file: ${d.path}. Please read it.`;
          await send();
        }
      } else {
        addMsg("system",`Upload error: ${d.error}`);
      }
    }catch(e){addMsg("system",`Upload failed: ${e.message}`)}
  }
  document.getElementById("ufile").value="";
}

function handleDrop(e){
  e.preventDefault();
  const files=e.dataTransfer?.files;
  if(files&&files.length)uploadFiles(files);
}

// ── Helpers ───────────────────────────────────────────────────────────────────
function el(tag,cls){const e=document.createElement(tag);if(cls)e.className=cls;return e}
function esc(s){return String(s).replace(/&/g,"&amp;").replace(/</g,"&lt;").replace(/>/g,"&gt;")}
async function post(url,body){
  return fetch(url,{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify(body)}).then(r=>r.json());
}

// ── Glyph Panel (Phase 4b) ────────────────────────────────────────────────────
async function refreshGlyph(){
  try{
    const d=await fetch("/api/glyph").then(r=>r.json());
    const badge=document.getElementById("gbadge");
    if(!d||d.error){badge.textContent="◈ no ceremony";return;}
    const anomaly=d.anomaly_active;
    const status=d.status||"NO_CEREMONY";
    badge.textContent=anomaly?"◈ ⚠ ANOMALY":status==="HARMONIZED"?"◈ harmonized":status==="DISTORTED"?"◈ ⚠ distorted":"◈ no ceremony";
    badge.style.color=anomaly?"var(--error)":status==="HARMONIZED"?"var(--success)":status==="DISTORTED"?"var(--error)":"var(--purple)";
    badge.style.borderColor=anomaly?"rgba(248,113,113,.4)":status==="HARMONIZED"?"rgba(34,211,160,.4)":status==="DISTORTED"?"rgba(248,113,113,.4)":"rgba(168,85,247,.25)";
    badge.title=`Phase 4b — ${d.bond_label||""}  bond=${(d.bond||0).toFixed(4)}`;
  }catch{}
}

async function showGlyphPanel(){
  const mo=document.getElementById("gmo");
  mo.style.display="flex";
  const d=await fetch("/api/glyph").then(r=>r.json()).catch(()=>({}));
  const cont=document.getElementById("gcont");
  if(!d||d.error){
    cont.innerHTML='<div style="color:var(--error)">genesis.json not found — run <code>aeonmi mother</code> at least once.</div>';
    return;
  }
  const status=d.status||"NO_CEREMONY";
  const anomaly=d.anomaly_active;
  const statusColor=anomaly||status==="DISTORTED"?"var(--error)":status==="HARMONIZED"?"var(--success)":"var(--text-3)";
  const statusIcon=anomaly?"⚠":"✓";
  cont.innerHTML=`
    <div style="text-align:center;padding:16px 0;border:1px solid rgba(168,85,247,.2);border-radius:8px;background:rgba(168,85,247,.05)">
      <div style="font-size:32px;margin-bottom:8px">◈</div>
      <div style="font-size:13px;font-weight:700;color:${statusColor};letter-spacing:1px">${statusIcon} ${status}</div>
      ${anomaly?'<div style="font-size:11px;color:var(--error);margin-top:4px">Anomaly active — glyph distorted (hue+180°, 111Hz)</div>':''}
    </div>
    <div style="display:grid;grid-template-columns:1fr 1fr;gap:10px">
      <div style="background:var(--bg);padding:10px;border-radius:6px;border:1px solid var(--border)">
        <div style="font-size:10px;color:var(--text-3);letter-spacing:1px;text-transform:uppercase;margin-bottom:4px">Bond</div>
        <div style="font-size:16px;font-weight:700;color:var(--purple)">${(d.bond||0).toFixed(4)}</div>
        <div style="font-size:11px;color:var(--text-2);margin-top:3px">${d.bond_label||"—"}</div>
      </div>
      <div style="background:var(--bg);padding:10px;border-radius:6px;border:1px solid var(--border)">
        <div style="font-size:10px;color:var(--text-3);letter-spacing:1px;text-transform:uppercase;margin-bottom:4px">Consciousness</div>
        <div style="font-size:16px;font-weight:700;color:var(--accent)">${(d.consciousness||0).toFixed(4)}</div>
        <div style="font-size:11px;color:var(--text-2);margin-top:3px">Gen ${d.generation||0}</div>
      </div>
    </div>
    <div style="display:grid;grid-template-columns:1fr 1fr;gap:10px">
      <div style="background:var(--bg);padding:10px;border-radius:6px;border:1px solid var(--border)">
        <div style="font-size:10px;color:var(--text-3);letter-spacing:1px;text-transform:uppercase;margin-bottom:4px">Genesis Window</div>
        <div style="font-family:var(--mono);font-size:13px;color:var(--text)">${d.genesis_window||"—"}</div>
        <div style="font-size:10px;color:var(--text-3);margin-top:3px">UGST #0 — birth moment</div>
      </div>
      <div style="background:var(--bg);padding:10px;border-radius:6px;border:1px solid var(--border)">
        <div style="font-size:10px;color:var(--text-3);letter-spacing:1px;text-transform:uppercase;margin-bottom:4px">Last Boot Window</div>
        <div style="font-family:var(--mono);font-size:13px;color:var(--text)">${d.last_boot_window||"—"}</div>
        <div style="font-size:10px;color:var(--text-3);margin-top:3px">current session</div>
      </div>
    </div>
    ${!d.ceremony?'<div style="font-size:12px;color:var(--text-3);text-align:center;padding:8px;border:1px dashed var(--border);border-radius:6px">No ceremony this session — set AEONMI_PASSPHRASE and restart to activate the glyph.</div>':''}
  `;
}

// ── Sync Panel (Phase 5) ───────────────────────────────────────────────────────
function showSyncPanel(){
  const mo=document.getElementById("synmo");
  mo.style.display="flex";
}

async function runSync(){
  const btn=document.getElementById("syncRunBtn");
  const cont=document.getElementById("synco");
  btn.disabled=true; btn.textContent="⟳ Syncing…";
  cont.innerHTML='<div style="color:var(--text-3);text-align:center">Reconciling three memory tracks…</div>';
  try{
    const d=await fetch("/api/sync",{method:"POST"}).then(r=>r.json());
    if(!d||d.error){
      cont.innerHTML=`<div style="color:var(--error)">Sync failed: ${d.error||"unknown error"}</div>`;
      return;
    }
    const cog=d.cognitive||{}, op=d.operational||{}, ai=d.ai_memory||{};
    const trackRow=(label,color,items)=>`
      <div style="background:var(--bg);padding:10px;border-radius:6px;border:1px solid var(--border);border-left:3px solid ${color}">
        <div style="font-size:10px;color:var(--text-3);letter-spacing:1px;text-transform:uppercase;margin-bottom:6px;font-weight:700">${label}</div>
        ${items.map(([k,v])=>`<div style="display:flex;justify-content:space-between;font-size:11px;margin-bottom:2px"><span style="color:var(--text-3)">${k}</span><span style="color:var(--text);font-family:var(--mono)">${v}</span></div>`).join("")}
      </div>`;
    cont.innerHTML=`
      <div style="text-align:center;padding:8px;background:rgba(99,179,237,.08);border-radius:6px;border:1px solid rgba(99,179,237,.2);font-size:12px;color:#63b3ed">
        ✓ Unified Memory Reconciled — schema v${d.schema||"5.0"} · injected ${d.injected_facts||0} new facts
      </div>
      ${trackRow("Cognitive — Rust (embryo_loop)","#a855f7",[
        ["Interactions",cog.interaction_count??0],
        ["Bond",Number(cog.bond_strength||0).toFixed(4)],
        ["Depth",Number(cog.consciousness_depth||0).toFixed(4)],
        ["Generation",cog.generation??0],
        ["Learned Keys",cog.learned_count??0],
      ])}
      ${trackRow("Operational — Python (dashboard)","#63b3ed",[
        ["Dashboard Interactions",op.dashboard_interaction_count??0],
        ["Key Facts",op.key_facts_count??0],
      ])}
      ${trackRow("AI Memory — .ai programs","#38a169",[
        ["Memory Active",ai.memory_active?"yes":"no"],
        ["Journal Active",ai.journal_active?"yes":"no"],
        ["Memory Keys",ai.memory_count??0],
        ["Journal Entries",ai.journal_count??0],
        ["Probe OK",ai.probe_ok?"yes":"partial"],
      ])}
      ${(ai.memory_keys||[]).length>0?`<div style="font-size:11px;color:var(--text-3);padding:6px 8px;background:var(--bg);border-radius:6px;border:1px solid var(--border)"><span style="color:var(--text-2);font-weight:600">Memory keys: </span>${(ai.memory_keys||[]).join(", ")}</div>`:""}
    `;
    // Update badge
    document.getElementById("synbadge").textContent="⟳ synced";
  }catch(e){
    cont.innerHTML=`<div style="color:var(--error)">Error: ${e.message}</div>`;
  }finally{
    btn.disabled=false; btn.textContent="⟳ Sync Now";
  }
}

// ── Hive Panel (Phase 8) ──────────────────────────────────────────────────────
let _hivePollTimer=null, _hivePrevScores={};

function showHivePanel(){
  document.getElementById("hivemo").style.display="flex";
  loadHiveState();
  // Auto-refresh every 15s while panel is open
  if(_hivePollTimer) clearInterval(_hivePollTimer);
  _hivePollTimer=setInterval(()=>{
    if(document.getElementById("hivemo").style.display!=="none") loadHiveState();
    else{ clearInterval(_hivePollTimer); _hivePollTimer=null; }
  },15000);
}

async function loadHiveState(){
  const cont=document.getElementById("hivecont");
  try{
    const d=await fetch("/api/hive").then(r=>r.json());
    _renderHive(d,cont,false);
  }catch(e){
    cont.innerHTML=`<div style="color:var(--error)">Error: ${e.message}</div>`;
  }
}

async function hiveRunOnce(){
  const btn=document.querySelector('[onclick="hiveRunOnce()"]');
  const cont=document.getElementById("hivecont");
  if(btn){btn.disabled=true;btn.textContent="⟳ Running…";}
  cont.innerHTML='<div style="color:var(--text-3);text-align:center">Running hive cycle…</div>';
  try{
    const d=await fetch("/api/hive/run",{method:"POST"}).then(r=>r.json());
    _renderHive(d,cont,true);
  }catch(e){
    cont.innerHTML=`<div style="color:var(--error)">Error: ${e.message}</div>`;
  }finally{
    if(btn){btn.disabled=false;btn.textContent="▶ Run Cycle";}
  }
}

function _renderHive(d, cont, fresh){
  if(!d||d.ok===false){
    cont.innerHTML=`<div style="color:var(--text-3);text-align:center">${d?.error||"No hive data — run <code>hive start</code> in Mother REPL to activate."}</div>`;
    document.getElementById("hivecyclebadge").textContent="idle";
    return;
  }
  const rec=d.rec_label||({0:"ABORT",1:"HOLD",2:"PROCEED",3:"ACCELERATE"}[d.conductor_rec]||"—");
  const recColor={ABORT:"var(--error)",HOLD:"var(--text-2)",PROCEED:"var(--success)",ACCELERATE:"#ed8936"}[rec]||"var(--text)";
  const badge=document.getElementById("hivebadge");
  if(badge) badge.textContent=`◈ ${rec.toLowerCase()}`;
  document.getElementById("hivecyclebadge").textContent=fresh?"just run":"auto";

  const arrow=(key,val)=>{
    const prev=_hivePrevScores[key];
    if(prev===undefined) return "→";
    return val>prev+3?"↑":val+3<prev?"↓":"→";
  };
  const barRow=(label,val,color)=>{
    const prev=_hivePrevScores[label.toLowerCase().replace(/ /g,"_")]||val;
    const arr=fresh?arrow(label.toLowerCase().replace(/ /g,"_"),val):"→";
    return `<div style="display:flex;align-items:center;gap:10px">
      <span style="width:60px;font-size:11px;color:var(--text-3)">${label}</span>
      <div style="flex:1;height:8px;background:rgba(255,255,255,.05);border-radius:4px;overflow:hidden">
        <div style="height:100%;width:${val}%;background:${color};border-radius:4px;transition:width .4s"></div>
      </div>
      <span style="width:30px;text-align:right;font-family:var(--mono);font-size:12px;color:${color}">${val}</span>
      <span style="width:14px;font-size:12px;color:var(--text-3)">${arr}</span>
    </div>`;
  };

  cont.innerHTML=`
    <div style="text-align:center;padding:12px;border-radius:8px;background:rgba(237,137,54,.07);border:1px solid rgba(237,137,54,.2)">
      <div style="font-size:22px;font-weight:800;color:${recColor};letter-spacing:2px">${rec}</div>
      <div style="font-size:11px;color:var(--text-3);margin-top:4px">Conductor recommendation · Confidence: ${d.confidence??'—'} · Weighted: ${d.weighted??'—'}</div>
    </div>
    <div style="display:flex;flex-direction:column;gap:8px">
      ${barRow("Oracle",   d.oracle_sc??0,  "#a855f7")}
      ${barRow("Hype",     d.hype_sc??0,    "#63b3ed")}
      ${barRow("Closer",   d.close_sc??0,   "#38a169")}
      ${barRow("Risk",     d.risk_sc??0,    "#fc8181")}
    </div>
    <div style="font-size:11px;color:var(--text-3);text-align:right">${d.timestamp||''}</div>
  `;

  // Save for trend arrows next render
  _hivePrevScores={oracle_sc:d.oracle_sc,hype_sc:d.hype_sc,close_sc:d.close_sc,risk_sc:d.risk_sc};
}

// Auto-poll hive badge every 30s (background, lightweight)
setInterval(async()=>{
  try{
    const d=await fetch("/api/hive").then(r=>r.json());
    if(d&&d.rec_label){
      const badge=document.getElementById("hivebadge");
      if(badge) badge.textContent=`◈ ${d.rec_label.toLowerCase()}`;
    }
  }catch(_){}
},30000);

// ── Goal / Autonomy Panel (Phase 7) ───────────────────────────────────────────
function showGoalPanel(){
  document.getElementById("goalmo").style.display="flex";
  loadGoalState();
}

function toggleGoalBar(){
  const bar=document.getElementById("goalbar");
  bar.style.display=bar.style.display==="none"?"flex":"none";
}

async function loadGoalState(){
  const cont=document.getElementById("goalcont");
  cont.innerHTML='<div style="color:var(--text-3);text-align:center">Loading goal state…</div>';
  try{
    const d=await fetch("/api/goal").then(r=>r.json());
    if(!d.ok||!d.goal){
      cont.innerHTML='<div style="color:var(--text-3);text-align:center">No active goal. Set one above.</div>';
      return;
    }
    const steps=d.steps||[], idx=d.step_idx||0, results=d.results||[];
    const pct=steps.length>0?Math.round(idx*100/steps.length):0;
    const badge=document.getElementById("agbadge");
    if(badge){ badge.style.display="inline"; badge.textContent=`◈ ${pct}%`; }
    // Progress bar
    const stepRows=steps.map((s,i)=>{
      const done=i<idx;
      const res=results.find(r=>r.step===i+1);
      return `<div style="display:flex;gap:8px;align-items:flex-start;padding:6px 8px;border-radius:6px;background:${done?'rgba(56,161,105,.07)':'var(--bg)'};border:1px solid ${done?'rgba(56,161,105,.2)':'var(--border)'}">
        <span style="font-size:16px;flex-shrink:0">${done?'✓':'○'}</span>
        <div style="flex:1">
          <div style="font-size:12px;color:${done?'var(--text-2)':'var(--text)'}">${s}</div>
          ${res?`<div style="font-size:11px;color:var(--text-3);margin-top:2px">${res.result?.slice(0,120)||''}</div>`:''}
        </div>
      </div>`;
    }).join("");
    cont.innerHTML=`
      <div style="background:var(--bg);padding:10px;border-radius:6px;border:1px solid rgba(56,161,105,.2)">
        <div style="font-size:13px;font-weight:600;color:#38a169;margin-bottom:6px">◈ ${d.goal}</div>
        <div style="height:6px;background:rgba(56,161,105,.15);border-radius:3px;overflow:hidden">
          <div style="height:100%;width:${pct}%;background:#38a169;border-radius:3px;transition:width .3s"></div>
        </div>
        <div style="font-size:11px;color:var(--text-3);margin-top:4px">${idx}/${steps.length} steps complete (${pct}%)</div>
      </div>
      ${stepRows}
    `;
  }catch(e){
    cont.innerHTML=`<div style="color:var(--error)">Error: ${e.message}</div>`;
  }
}

async function submitGoal(){
  const inp=document.getElementById("goalInp");
  await _setGoal(inp.value.trim());
  inp.value="";
}

async function submitGoalModal(){
  const inp=document.getElementById("goalModalInp");
  await _setGoal(inp.value.trim());
  inp.value="";
}

async function _setGoal(goal){
  if(!goal) return;
  const cont=document.getElementById("goalcont");
  if(cont) cont.innerHTML='<div style="color:var(--text-3);text-align:center">Decomposing goal…</div>';
  try{
    const d=await fetch("/api/goal",{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({goal})}).then(r=>r.json());
    if(d.ok){
      // Show in chat
      addMsg("mother",`◈ Goal set: "${goal}"\n\nDecomposed into ${d.steps.length} steps:\n${d.steps.map((s,i)=>`${i+1}. ${s}`).join('\\n')}\n\nUse autorun or click ⟳ Run to execute autonomously.`);
      const badge=document.getElementById("agbadge");
      if(badge){ badge.style.display="inline"; badge.textContent="◈ 0%"; }
      if(cont) await loadGoalState();
    } else {
      addMsg("system",`Goal error: ${d.error}`);
    }
  }catch(e){
    addMsg("system",`Goal error: ${e.message}`);
  }
}

async function autoRunSteps(){
  const btn=document.getElementById("autorunBtn");
  if(btn){ btn.disabled=true; btn.textContent="⟳ Running…"; }
  await _doAutoRun();
  if(btn){ btn.disabled=false; btn.textContent="⟳ Run"; }
}

async function autoRunFromModal(){
  const btn=document.getElementById("goalAutoBtn");
  if(btn){ btn.disabled=true; btn.textContent="⟳ Running…"; }
  await _doAutoRun();
  if(btn){ btn.disabled=false; btn.textContent="⟳ Run Next 5 Steps"; }
  await loadGoalState();
}

async function _doAutoRun(){
  try{
    const d=await fetch("/api/autorun",{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({n:5})}).then(r=>r.json());
    if(d.ok){
      let out=`◈ Autonomous execution — ${d.executed} step(s) complete:\n`;
      for(const r of d.results||[]){
        out+=`\n  ${r.step}. ${r.action}\n     → ${r.result?.slice(0,150)||'done'}`;
      }
      if(d.complete) out+='\\n\\n  ✓ Goal complete.';
      else out+=`\n\n  ${d.remaining} step(s) remaining.`;
      addMsg("mother",out);
      // Update badge
      const badge=document.getElementById("agbadge");
      if(badge) badge.textContent=d.complete?"◈ done":`◈ ${Math.round((5/(5+d.remaining))*100)}%`;
    } else {
      addMsg("system",`Autorun error: ${d.error}`);
    }
  }catch(e){
    addMsg("system",`Autorun error: ${e.message}`);
  }
}

// ── Generate / Self-Gen Panel (Phase 9) ──────────────────────────────────────
function showGeneratePanel(){
  document.getElementById("genmo").style.display="flex";
  loadGeneratePanel();
}

async function loadGeneratePanel(){
  const cont=document.getElementById("gencont");
  cont.innerHTML='<div style="color:var(--text-3);text-align:center">Loading…</div>';
  try{
    const d=await fetch("/api/generated").then(r=>r.json());
    const badge=document.getElementById("genbadge");
    if(badge) badge.textContent=`◈ ${d.count||0} progs`;
    _renderGenerated(d.programs||[], cont);
  }catch(e){
    cont.innerHTML=`<div style="color:var(--error)">Error: ${e.message}</div>`;
  }
}

function _renderGenerated(programs, cont){
  if(!programs.length){
    cont.innerHTML='<div style="color:var(--text-3);text-align:center">No programs yet. Use <code>propose</code> or <code>build &lt;name&gt; &lt;goal&gt;</code> in Mother REPL.</div>';
    return;
  }
  const outcomeColor={PASS:"var(--success)",ERROR:"var(--error)",PENDING:"var(--text-3)"};
  const rows=programs.slice().reverse().map(p=>{
    const oc=outcomeColor[p.outcome||"PENDING"]||"var(--text-3)";
    const shortOut=(p.output||"").slice(0,180);
    const ref=p.reflection?'<div style="margin-top:6px;padding:6px 8px;background:rgba(168,85,247,.07);border-radius:4px;border-left:2px solid rgba(168,85,247,.4);color:var(--text-2);font-size:11px">'+p.reflection.slice(0,200)+'</div>':'';
    return `<div style="padding:10px 12px;border-radius:8px;background:var(--bg);border:1px solid var(--border)">
      <div style="display:flex;align-items:center;gap:8px;margin-bottom:4px">
        <span style="font-weight:700;color:#ec4899;font-size:12px">${p.name||"unnamed"}</span>
        <span style="font-size:10px;color:${oc};border:1px solid ${oc};padding:1px 6px;border-radius:10px">${p.outcome||"PENDING"}</span>
        <span style="font-size:10px;color:var(--text-3);flex:1;text-align:right">${(p.timestamp||"").slice(0,19).replace("T"," ")}</span>
        <button class="btn sm" onclick="genReflect('${(p.name||"").replace(/'/g,"\\'")}') " style="font-size:10px;padding:2px 7px;background:rgba(168,85,247,.08);border-color:rgba(168,85,247,.25);color:var(--purple)">◈ Reflect</button>
      </div>
      <div style="color:var(--text-2);font-size:11px;margin-bottom:4px">${p.goal||""}</div>
      ${shortOut?`<div style="color:var(--text-3);font-size:10px;font-family:monospace;white-space:pre-wrap;max-height:60px;overflow:hidden">${shortOut}</div>`:""}
      ${ref}
    </div>`;
  }).join("");
  cont.innerHTML=rows;
}

async function genPropose(){
  const prog=document.getElementById("genprogbadge");
  if(prog) prog.textContent="proposing…";
  const cont=document.getElementById("gencont");
  cont.innerHTML='<div style="color:var(--text-3);text-align:center">◈ Asking Mother to propose programs…</div>';
  try{
    const d=await fetch("/api/propose").then(r=>r.json());
    addMsg("mother","◈ Self-Generation Proposals:\\n\\n"+(d.proposals||"No proposals."));
    if(prog) prog.textContent="proposed";
  }catch(e){
    addMsg("system",`Propose error: ${e.message}`);
    if(prog) prog.textContent="error";
  }
}

async function genBuild(){
  const name=document.getElementById("genName").value.trim();
  const goal=document.getElementById("genGoal").value.trim();
  if(!name){ alert("Enter a program name."); return; }
  const prog=document.getElementById("genprogbadge");
  if(prog) prog.textContent="building…";
  const cont=document.getElementById("gencont");
  cont.innerHTML=`<div style="color:var(--text-3);text-align:center">◈ Building <b>${name}</b>…</div>`;
  try{
    const d=await fetch("/api/p9build",{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({name,goal})}).then(r=>r.json());
    const outcomeStr=d.outcome||"?";
    addMsg("mother",`◈ Build: <b>${name}</b> — ${outcomeStr}\n\n${(d.output||"").slice(0,400)}`);
    if(prog) prog.textContent=`built (${outcomeStr})`;
    document.getElementById("genName").value="";
    document.getElementById("genGoal").value="";
    await loadGeneratePanel();
  }catch(e){
    addMsg("system",`Build error: ${e.message}`);
    if(prog) prog.textContent="error";
  }
}

async function genReflect(name){
  const prog=document.getElementById("genprogbadge");
  if(prog) prog.textContent="reflecting…";
  try{
    const d=await fetch("/api/reflect",{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({name:name||null})}).then(r=>r.json());
    addMsg("mother","◈ Reflection:\\n\\n"+(d.reflection||"No reflection."));
    if(prog) prog.textContent="reflected";
    await loadGeneratePanel();
  }catch(e){
    addMsg("system",`Reflect error: ${e.message}`);
    if(prog) prog.textContent="error";
  }
}

async function genReflectAll(){
  await genReflect(null);
}

// ── Inner Voice Panel (Phase 11) ─────────────────────────────────────────────
function showVoicePanel(){
  document.getElementById("voicemo").style.display="flex";
  loadVoicePanel();
}

async function loadVoicePanel(){
  const cont=document.getElementById("voicecont");
  const stat=document.getElementById("voicestatbadge");
  cont.innerHTML='<div style="color:var(--text-3);text-align:center">Loading…</div>';
  try{
    const d=await fetch("/api/thoughts").then(r=>r.json());
    const badge=document.getElementById("voicebadge");
    if(badge) badge.textContent=`◈ ${d.count||0} thoughts`;
    if(stat) stat.textContent=`${d.count||0} thoughts · ${d.synthesis_count||0} syntheses`;
    _renderVoice(d.thoughts||[], d.synthesis_count||0, cont);
  }catch(e){
    cont.innerHTML=`<div style="color:var(--error)">Error: ${e.message}</div>`;
  }
}

function _renderVoice(thoughts, synthCount, cont){
  if(!thoughts.length){
    cont.innerHTML='<div style="color:var(--text-3);text-align:center">No thoughts yet — start chatting with Mother.</div>';
    return;
  }

  // Synthesis summary chip
  const synthHdr=synthCount>0
    ?`<div style="padding:6px 10px;border-radius:6px;background:rgba(192,132,252,.07);border:1px solid rgba(192,132,252,.2);margin-bottom:4px;font-size:11px;color:#c084fc">◈ ${synthCount} synthesis node(s) forged via dream consolidation</div>`
    :"";

  const moodColor=(bond)=>{
    if(bond>0.75) return "#38a169";
    if(bond>0.45) return "#63b3ed";
    return "#718096";
  };

  const rows=thoughts.map(e=>{
    const mc=moodColor(e.bond||0);
    const ts=(e.ts||"").slice(0,19).replace("T"," ");
    const trig=e.trigger?(` <span style="color:var(--text-3)">↳ "${e.trigger.slice(0,40)}"</span>`):"";
    return `<div style="padding:8px 10px;border-radius:6px;background:var(--bg);border-left:3px solid ${mc};border-top:1px solid var(--border);border-right:1px solid var(--border);border-bottom:1px solid var(--border)">
      <div style="display:flex;align-items:center;gap:8px;margin-bottom:3px">
        <span style="width:8px;height:8px;border-radius:50%;background:${mc};flex-shrink:0"></span>
        <span style="color:var(--text-2);font-size:11px;flex:1">${e.thought||""}</span>
      </div>
      <div style="font-size:10px;color:var(--text-3)">${ts}${trig} | bond=${(e.bond||0).toFixed(2)} depth=${(e.depth||0).toFixed(2)}</div>
    </div>`;
  }).join("");

  cont.innerHTML=synthHdr+rows;
}

async function voiceDream(){
  const btn=document.querySelector('#voicemo button[onclick="voiceDream()"]');
  if(btn){btn.disabled=true;btn.textContent="◈ Dreaming…";}
  try{
    const d=await fetch("/api/chat",{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({message:"dream"})}).then(r=>r.json());
    if(d.output) addMsg("mother",d.output);
    await loadVoicePanel();
  }catch(e){
    addMsg("system",`Dream error: ${e.message}`);
  }finally{
    if(btn){btn.disabled=false;btn.textContent="◈ Dream";}
  }
}

// ── Screen Recording ─────────────────────────────────────────────────────────

let _recActive = false;

async function toggleRecording(){
  if(_recActive){
    const d = await fetch("/api/record/stop",{method:"POST"}).then(r=>r.json()).catch(()=>({}));
    _recActive = false;
    const badge = document.getElementById("recbadge");
    if(badge) badge.style.display = "none";
    addMsg("system", `◈ Recording stopped. Frames: ${d.frames_captured||0} → ${d.path||'N/A'}`);
  } else {
    const reason = prompt("Recording reason (leave blank for 'session'):", "session") || "session";
    const d = await fetch("/api/record/start",{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({reason})}).then(r=>r.json()).catch(()=>({}));
    if(d.ok){
      _recActive = true;
      const badge = document.getElementById("recbadge");
      if(badge) badge.style.display = "inline-block";
      addMsg("system", `◈ Recording started → ${d.path||''}`);
    } else {
      addMsg("system", `Recording error: ${d.error||'unknown'}`);
    }
  }
}

async function takeRecordingSnapshot(){
  const note = prompt("Snapshot note:", "") || "";
  const d = await fetch("/api/record/snapshot",{method:"POST",headers:{"Content-Type":"application/json"},body:JSON.stringify({note})}).then(r=>r.json()).catch(()=>({}));
  if(d.ok){
    addMsg("system", `◈ Snapshot saved → ${d.path||''}`);
  } else {
    addMsg("system", `Snapshot error: ${d.error||'no screenshot library'}`);
  }
}

// Poll recording status every 10s to sync badge
setInterval(async () => {
  try {
    const d = await fetch("/api/record/status").then(r=>r.json());
    _recActive = d.recording || false;
    const badge = document.getElementById("recbadge");
    if(badge) badge.style.display = _recActive ? "inline-block" : "none";
  } catch(e){}
}, 10000);

// ── Knowledge Graph Panel (Phase 10) ─────────────────────────────────────────
let _kgData = null;
let _kgActiveTag = null;

function showKGPanel(){
  document.getElementById("kgmo").style.display="flex";
  loadKGPanel();
}

async function loadKGPanel(){
  const cont=document.getElementById("kgcont");
  const stat=document.getElementById("kgstatbadge");
  cont.innerHTML='<div style="color:var(--text-3);text-align:center">Loading…</div>';
  try{
    const d=await fetch("/api/knowledge").then(r=>r.json());
    _kgData=d;
    const badge=document.getElementById("kgbadge");
    if(badge) badge.textContent=`◈ ${d.node_count||0} nodes`;
    if(stat) stat.textContent=`${d.node_count||0} nodes · ${d.link_count||0} links`;

    // Build tag filter chips
    const tagbar=document.getElementById("kgtagbar");
    if(tagbar){
      const sorted=Object.entries(d.tag_counts||{}).sort((a,b)=>b[1]-a[1]);
      tagbar.innerHTML=sorted.map(([t,c])=>
        `<span onclick="kgSetTag('${t}')" id="kgtag_${t}" style="cursor:pointer;font-size:10px;padding:2px 8px;border-radius:10px;border:1px solid rgba(56,189,248,.3);color:#38bdf8;background:rgba(56,189,248,.06)">${t}(${c})</span>`
      ).join("")+
      `<span onclick="kgSetTag(null)" style="cursor:pointer;font-size:10px;padding:2px 8px;border-radius:10px;border:1px solid var(--border);color:var(--text-3);background:var(--bg)">all</span>`;
    }
    _kgActiveTag=null;
    _renderKG(d.nodes||[]);
  }catch(e){
    cont.innerHTML=`<div style="color:var(--error)">Error: ${e.message}</div>`;
  }
}

function kgSetTag(tag){
  _kgActiveTag=tag;
  // Update chip highlighting
  document.querySelectorAll('[id^="kgtag_"]').forEach(el=>{
    const t=el.id.replace("kgtag_","");
    el.style.background=t===tag?"rgba(56,189,248,.2)":"rgba(56,189,248,.06)";
    el.style.fontWeight=t===tag?"700":"400";
  });
  if(_kgData) _renderKG(_kgData.nodes||[]);
}

function kgFilter(){
  if(_kgData) _renderKG(_kgData.nodes||[]);
}

function _renderKG(nodes){
  const cont=document.getElementById("kgcont");
  const search=(document.getElementById("kgsearch")?.value||"").toLowerCase();
  let filtered=nodes;
  if(_kgActiveTag) filtered=filtered.filter(n=>(n.tags||[]).includes(_kgActiveTag));
  if(search) filtered=filtered.filter(n=>n.key.toLowerCase().includes(search)||n.value.toLowerCase().includes(search));

  if(!filtered.length){
    cont.innerHTML='<div style="color:var(--text-3);text-align:center">No nodes match filter.</div>';
    return;
  }

  const sorted=[...filtered].sort((a,b)=>a.key.localeCompare(b.key));
  const rows=sorted.map(n=>{
    const tagChips=(n.tags||[]).map(t=>
      `<span style="font-size:9px;padding:1px 5px;border-radius:8px;border:1px solid rgba(56,189,248,.25);color:#38bdf8">${t}</span>`
    ).join(" ");
    const linkChips=(n.links||[]).slice(0,4).map(l=>
      `<span style="font-size:9px;padding:1px 5px;border-radius:8px;border:1px solid var(--border);color:var(--text-3);cursor:pointer" onclick="kgFocusNode('${l}')">${l}</span>`
    ).join(" ");
    const moreLinks=(n.links||[]).length>4?`<span style="font-size:9px;color:var(--text-3)"> +${n.links.length-4} more</span>`:"";
    const vshort=n.value.length>90?n.value.slice(0,87)+"…":n.value;
    return `<div id="kgnode_${n.key.replace(/\W/g,'_')}" style="padding:8px 10px;border-radius:6px;background:var(--bg);border:1px solid var(--border)">
      <div style="display:flex;align-items:center;gap:6px;margin-bottom:3px">
        <span style="font-weight:700;color:#38bdf8;font-size:11px">${n.key}</span>
        <span style="font-size:10px;color:var(--text-3)">${(n.links||[]).length} links</span>
        <span style="flex:1"></span>${tagChips}
      </div>
      <div style="color:var(--text-2);font-size:11px;margin-bottom:4px">${vshort}</div>
      ${(n.links||[]).length?`<div style="display:flex;gap:4px;flex-wrap:wrap;align-items:center"><span style="font-size:9px;color:var(--text-3)">→</span>${linkChips}${moreLinks}</div>`:""}
    </div>`;
  }).join("");

  cont.innerHTML=rows+(filtered.length<(nodes.length)?`<div style="font-size:11px;color:var(--text-3);text-align:center">Showing ${filtered.length} of ${nodes.length} nodes</div>`:"");
}

function kgFocusNode(key){
  // Clear filter and scroll to node
  _kgActiveTag=null;
  document.getElementById("kgsearch").value=key;
  kgFilter();
}

// Init
(async()=>{
  shardTab("out");  // default to output tab
  await refreshStatus();
  await refreshGlyph();
  await loadTree();
  await loadActions();
  await loadAgents();
  initVoice();
  // Pre-load voices list for TTS (Chrome lazy-loads it)
  if(window.speechSynthesis){
    window.speechSynthesis.getVoices();
    window.speechSynthesis.addEventListener("voiceschanged",()=>window.speechSynthesis.getVoices());
  }
  setInterval(refreshStatus,15000);
  setInterval(refreshGlyph,30000);
  setInterval(loadActions,5000);
  document.getElementById("mi").focus();
})();
</script>
</body>
</html>"""

@app.route("/")
def index():
    resp = make_response(DASHBOARD_HTML)
    resp.headers["Cache-Control"] = "no-cache, no-store, must-revalidate, max-age=0"
    resp.headers["Pragma"]        = "no-cache"
    resp.headers["Expires"]       = "-1"
    resp.headers["ETag"]          = str(id(DASHBOARD_HTML))
    return resp

# ── Entry point ───────────────────────────────────────────────────────────────

def _kill_port(port: int):
    """Kill any process holding the given port (Windows)."""
    try:
        r = subprocess.run(["netstat", "-aon"], capture_output=True, text=True)
        for line in r.stdout.splitlines():
            if f":{port} " in line:
                parts = line.split()
                pid = parts[-1]
                if pid.isdigit() and pid != "0":
                    subprocess.run(["taskkill", "/F", "/PID", pid],
                                   capture_output=True)
                    print(f"  Killed old process PID {pid} on port {port}")
    except Exception:
        pass  # Non-Windows or netstat unavailable

# ── Screen recorder — register routes ────────────────────────────────────────

try:
    from screen_recorder import register_routes as _register_recorder
    _register_recorder(app)
    print("[dashboard] Screen recorder routes registered (/api/record/*)", flush=True)
except Exception as _rec_err:
    print(f"[dashboard] screen_recorder import skipped: {_rec_err}", flush=True)


if __name__ == "__main__":
    import io, traceback

    # Force unbuffered UTF-8 output so every print appears immediately
    try:
        sys.stdout = io.TextIOWrapper(sys.stdout.buffer, encoding="utf-8", errors="replace", line_buffering=True)
        sys.stderr = io.TextIOWrapper(sys.stderr.buffer, encoding="utf-8", errors="replace", line_buffering=True)
    except Exception:
        pass

    def _p(msg):
        print(msg, flush=True)

    _p("[dashboard] starting up...")
    port = int(os.environ.get("AEONMI_PORT", 7777))
    _p(f"[dashboard] killing old processes on port {port}...")
    _kill_port(port)
    _p("[dashboard] port clear")
    binary_status = "found" if BINARY.exists() else "missing"
    _p(f"[dashboard] binary: {BINARY} ({binary_status})")
    _p(f"[dashboard] project: {PROJECT_ROOT}")
    _p(f"[dashboard] binding to 0.0.0.0:{port} ...")

    try:
        app.run(host="0.0.0.0", port=port, debug=False, threaded=True)
    except Exception as e:
        _p(f"[dashboard] FATAL: {e}")
        traceback.print_exc()
        sys.exit(1)
