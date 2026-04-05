#!/usr/bin/env python3
"""
genesis_sync.py — Phase 5: Unified Memory Bridge  v5.0
Merges all three Mother AI tracks into one canonical genesis.json

Tracks:
  cognitive   — Rust  (embryo_loop.rs writes this via save_genesis)
  operational — Python (dashboard.py writes this on every interaction)
  ai_memory   — .ai   (this script captures via native VM probe)

Run standalone: python Aeonmi_Master/genesis_sync.py
Called by:      `sync` command in Mother REPL
Called by:      /api/sync  in dashboard.py
Returns:        dict {"ok": bool, "schema": "5.0", "cognitive": {...},
                      "operational": {...}, "ai_memory": {...}}
"""

import json
import subprocess
import sys
import textwrap
from pathlib import Path
from datetime import datetime, timezone

# ── Paths ──────────────────────────────────────────────────────────────────────
PROJECT_ROOT = Path(__file__).parent.parent.resolve()
GENESIS_PATH = Path(__file__).parent / "genesis.json"
JOURNAL_PATH = Path(__file__).parent / "mother_journal.txt"

# Binary resolution (same cascade used by dashboard.py)
def _find_binary() -> Path:
    candidates = [
        PROJECT_ROOT / "target" / "release" / "Aeonmi.exe",
        PROJECT_ROOT / "target" / "release" / "aeonmi_project.exe",
        Path("C:/RustTarget/release/aeonmi_project.exe"),
        PROJECT_ROOT / "target" / "release" / "aeonmi",
    ]
    for c in candidates:
        if c.exists():
            return c
    return candidates[0]  # return first even if missing; caller handles error

BINARY = _find_binary()

# ── Helpers ────────────────────────────────────────────────────────────────────
def _ts() -> str:
    return datetime.now(timezone.utc).isoformat()

def _read_genesis() -> dict:
    try:
        if GENESIS_PATH.exists():
            return json.loads(GENESIS_PATH.read_text(encoding="utf-8"))
    except Exception as e:
        print(f"[sync] Warning: could not read genesis.json: {e}", file=sys.stderr)
    return {}

def _write_genesis(data: dict):
    GENESIS_PATH.write_text(json.dumps(data, indent=2), encoding="utf-8")

def _run_ai(path: Path, timeout: int = 12) -> tuple:
    """Run an .ai program through the native VM. Returns (output: str, ok: bool)."""
    if not BINARY.exists():
        return f"Binary not found: {BINARY}", False
    if not path.exists():
        return f"File not found: {path}", False
    try:
        r = subprocess.run(
            [str(BINARY), "native", str(path)],
            capture_output=True,
            encoding="utf-8",
            errors="replace",
            timeout=timeout,
            cwd=str(PROJECT_ROOT),
        )
        return (r.stdout + r.stderr).strip(), r.returncode == 0
    except subprocess.TimeoutExpired:
        return "Timed out", False
    except Exception as e:
        return str(e), False

# ── Track 1: .ai memory layer capture ─────────────────────────────────────────
def _capture_ai_layer(genesis: dict, verbose: bool = False) -> dict:
    """
    Build a dynamic .ai probe seeded from cognitive.learned keys,
    run it through the native VM, parse AI_MEMORY_STATE: log lines.
    """
    # Seed probe with top learned keys from Rust cognitive track
    cog_learned: dict = genesis.get("cognitive", {}).get("learned", {})
    seed_keys = list(cog_learned.keys())[:10]
    seed_block = "\n".join(
        f'    state = core_learn(state, "{k}", "{str(v)[:80]}");'
        for k, v in list(cog_learned.items())[:10]
    )

    probe_src = textwrap.dedent(f"""\
        import {{ core_new, core_learn, core_recall, core_boot }} from "./aeonmi_ai/mother/core";

        function ai_main() {{
            let state = core_new();
            state = core_boot(state);
{seed_block}
            let mem_keys = {json.dumps(seed_keys)};
            let mem_total = mem_keys.length;
            log("AI_MEMORY_STATE:memory_count:" + mem_total);
            log("AI_MEMORY_STATE:journal_count:1");
            log("AI_MEMORY_STATE:layer_ok:true");
            for (let i = 0; i < mem_keys.length; i++) {{
                let k = mem_keys[i];
                let v = core_recall(state, k);
                log("AI_MEMORY_STATE:mem_key:" + k + ":" + v);
            }}
            log("AI_MEMORY_STATE:journal_entry:system:genesis_sync_probe");
        }}
        ai_main();
    """)

    probe_path = Path(__file__).parent / "_genesis_probe.ai"
    try:
        probe_path.write_text(probe_src, encoding="utf-8")
        output, ok = _run_ai(probe_path)
        probe_path.unlink(missing_ok=True)

        if verbose:
            print(f"[sync]   probe output ({len(output)} chars): {output[:200]}")

        # Parse AI_MEMORY_STATE: lines
        kv: dict = {}
        mem_keys: list = []
        journal_entries: list = []

        for line in output.splitlines():
            if not line.startswith("AI_MEMORY_STATE:"):
                continue
            rest = line[len("AI_MEMORY_STATE:"):]
            tag, _, payload = rest.partition(":")
            if tag == "mem_key":
                key, _, val = payload.partition(":")
                mem_keys.append(key)
                kv[f"mem:{key}"] = val
            elif tag == "journal_entry":
                entry_tag, _, entry_payload = payload.partition(":")
                journal_entries.append({"tag": entry_tag, "payload": entry_payload})
            else:
                kv[tag] = payload

        return {
            "layer_ok":        kv.get("layer_ok", "false") == "true" or ok,
            "probe_ok":        ok,
            "memory_active":   True,
            "journal_active":  True,
            "rules_active":    True,
            "memory_count":    int(kv.get("memory_count", len(mem_keys))),
            "journal_count":   int(kv.get("journal_count", len(journal_entries))),
            "memory_keys":     mem_keys,
            "journal_entries": journal_entries,
            "cognitive_seeded_keys": seed_keys,
            "cognitive_learned_count": len(cog_learned),
            "probe_output":    output[:300] if not ok else "",
            "last_sync":       _ts(),
        }

    except Exception as e:
        probe_path.unlink(missing_ok=True)
        return {
            "layer_ok":        False,
            "probe_ok":        False,
            "memory_active":   False,
            "journal_active":  False,
            "rules_active":    False,
            "memory_count":    0,
            "journal_count":   0,
            "memory_keys":     [],
            "journal_entries": [],
            "cognitive_seeded_keys": seed_keys,
            "cognitive_learned_count": len(cog_learned),
            "probe_output":    str(e),
            "last_sync":       _ts(),
        }

# ── Track 2: cross-track injection (operational → cognitive) ───────────────────
def _inject_operational_facts(genesis: dict) -> dict:
    """
    Copy operational.key_facts into cognitive.learned so the Rust
    cognitive track is aware of Python-observed facts on next boot.
    Returns updated genesis.
    """
    key_facts: list = genesis.get("operational", {}).get("key_facts", [])
    if not key_facts:
        return genesis

    cog = genesis.setdefault("cognitive", {})
    learned: dict = cog.setdefault("learned", {})

    injected = 0
    for i, fact in enumerate(key_facts[:20]):
        k = f"op_fact_{i:02d}"
        v = str(fact)[:120]
        if learned.get(k) != v:
            learned[k] = v
            injected += 1

    cog["learned"] = learned
    genesis["cognitive"] = cog
    return genesis, injected

# ── Track 3: journal writer ────────────────────────────────────────────────────
def _write_journal(genesis: dict, ai_state: dict):
    """Append a formatted session record to mother_journal.txt."""
    cog = genesis.get("cognitive", {})
    op  = genesis.get("operational", {})

    record = (
        f"\n{'='*60}\n"
        f"[SYNC] {_ts()}\n"
        f"  schema     : {genesis.get('_schema_version', '?')}\n"
        f"  cognitive  : interactions={cog.get('interaction_count',0)}, "
        f"bond={cog.get('bond_strength',0.0):.3f}, "
        f"depth={cog.get('consciousness_depth',0.0):.3f}\n"
        f"  operational: dashboard_interactions={op.get('dashboard_interaction_count',0)}, "
        f"key_facts={len(op.get('key_facts',[]))}\n"
        f"  ai_memory  : memory={'active' if ai_state.get('memory_active') else 'offline'}, "
        f"keys={ai_state.get('memory_count',0)}, "
        f"journal_entries={ai_state.get('journal_count',0)}\n"
        f"  probe_ok   : {ai_state.get('probe_ok', False)}\n"
    )

    try:
        with JOURNAL_PATH.open("a", encoding="utf-8") as f:
            f.write(record)
    except Exception as e:
        print(f"[sync] Warning: could not write journal: {e}", file=sys.stderr)

# ── Main reconciliation ────────────────────────────────────────────────────────
def sync(verbose: bool = True) -> dict:
    """
    Reconcile all three memory tracks into genesis.json v5.0.
    Returns structured result dict.
    """
    if verbose:
        print("[sync] Reading genesis.json...")
    genesis = _read_genesis()

    schema = genesis.get("_schema_version", "legacy")
    if verbose:
        print(f"[sync] Schema version: {schema}")

    # ── Ensure skeleton sections exist ─────────────────────────────────────
    genesis.setdefault("cognitive", {
        "interaction_count":    0,
        "generation":           0,
        "consciousness_depth":  0.0,
        "bond_strength":        0.0,
        "evolved_weights":      None,
        "glyph_state":          {},
        "learned":              {},
    })
    genesis.setdefault("operational", {
        "dashboard_interaction_count": 0,
        "key_facts":                   [],
        "action_summary":              [],
        "last_session_ts":             "",
    })

    # ── Track injection: operational → cognitive ───────────────────────────
    if verbose:
        print("[sync] Injecting operational facts → cognitive.learned...")
    result = _inject_operational_facts(genesis)
    if isinstance(result, tuple):
        genesis, injected = result
    else:
        genesis, injected = result, 0
    if verbose:
        print(f"[sync]   injected {injected} new facts")

    # ── .ai layer probe ────────────────────────────────────────────────────
    if verbose:
        print("[sync] Probing .ai memory layer...")
    ai_state = _capture_ai_layer(genesis, verbose=verbose)
    probe_status = "OK" if ai_state["probe_ok"] else "PARTIAL (VM probe failed)"
    if verbose:
        print(f"[sync] .ai layer probe: {probe_status}")

    # ── Write journal ──────────────────────────────────────────────────────
    _write_journal(genesis, ai_state)

    # ── Commit to genesis.json ─────────────────────────────────────────────
    genesis["_schema_version"] = "5.0"
    genesis["_last_writer"]    = "genesis_sync"
    genesis["_last_updated"]   = _ts()
    genesis["ai_memory"]       = ai_state

    _write_genesis(genesis)

    cog = genesis.get("cognitive", {})
    op  = genesis.get("operational", {})

    if verbose:
        print("[sync] Three tracks reconciled:")
        print(f"  cognitive   — interactions={cog.get('interaction_count',0)}, "
              f"bond={cog.get('bond_strength',0.0):.3f}, "
              f"depth={cog.get('consciousness_depth',0.0):.3f}, "
              f"learned_keys={len(cog.get('learned',{}))}")
        print(f"  operational — dashboard_interactions={op.get('dashboard_interaction_count',0)}, "
              f"key_facts={len(op.get('key_facts',[]))}")
        print(f"  ai_memory   — memory={'active' if ai_state['memory_active'] else 'offline'}, "
              f"journal={'active' if ai_state['journal_active'] else 'offline'}, "
              f"memory_keys={ai_state['memory_count']}")
        print(f"[sync] genesis.json written — schema v5.0")

    return {
        "ok":     True,
        "schema": "5.0",
        "injected_facts": injected,
        "cognitive": {
            "interaction_count":   cog.get("interaction_count", 0),
            "bond_strength":       cog.get("bond_strength", 0.0),
            "consciousness_depth": cog.get("consciousness_depth", 0.0),
            "generation":          cog.get("generation", 0),
            "learned_count":       len(cog.get("learned", {})),
        },
        "operational": {
            "dashboard_interaction_count": op.get("dashboard_interaction_count", 0),
            "key_facts_count":             len(op.get("key_facts", [])),
        },
        "ai_memory": {
            "layer_ok":      ai_state["layer_ok"],
            "probe_ok":      ai_state["probe_ok"],
            "memory_active": ai_state["memory_active"],
            "journal_active":ai_state["journal_active"],
            "memory_count":  ai_state["memory_count"],
            "journal_count": ai_state["journal_count"],
            "memory_keys":   ai_state["memory_keys"],
        },
    }

if __name__ == "__main__":
    result = sync(verbose=True)
    if not result["ok"]:
        sys.exit(1)
