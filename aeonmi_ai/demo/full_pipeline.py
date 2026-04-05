#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
full_pipeline.py -- Aeonmi Full Connected Pipeline
===================================================
Market signals -> Oracle -> QASM -> Qiskit Aer -> Conductor -> Mother Core
                                                               (persisted, adaptive)

Each run:
  1. Load prior Mother state from mother_state.json (or start fresh)
  2. Run quantum_hive_demo.ai (AEONMI_NATIVE=1) -> extract verdict + conf
  3. Run Qiskit Bell simulation -> extract entanglement %
  4. Generate mother_session.ai with prior state + live signal baked in
  5. Run mother_session.ai -> Mother processes the signal, adapts threshold
  6. Parse output -> save updated state to mother_state.json
  7. Print the full connected story

Usage:
  python -u aeonmi_ai\\demo\\full_pipeline.py
  Run multiple times to watch Mother learn across sessions.
"""

import os, sys, subprocess, json, time, shutil

# ---- Paths ------------------------------------------------------------------

REPO        = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
BIN         = os.path.join(REPO, "target", "release", "aeonmi.exe")
DEMO_AI     = os.path.join(REPO, "aeonmi_ai", "demo", "quantum_hive_demo.ai")
QK_RUNNER   = os.path.join(os.path.dirname(REPO), "Aeonmi_Master", "qiskit_runner.py")
STATE_FILE  = os.path.join(REPO, "aeonmi_ai", "core", "mother_state.json")
SESSION_AI  = os.path.join(REPO, "aeonmi_ai", "core", "mother_session.ai")

BELL_DESC   = "2 2 1024 4  0 0 0  4 1 0  7 0 0  7 1 1"
SEP         = "=" * 60

os.environ["AEONMI_NATIVE"] = "1"

# ---- State persistence ------------------------------------------------------

EMPTY_STATE = {
    "session":       0,
    "proceed_count": 0,
    "abort_count":   0,
    "predictions":   0,
    "correct":       0,
    "conf_sum":      0,
    "drift":         0,
    "last_verdict":  -1,
    "last_conf":     0,
    "last_threshold": 55,
    "last_accuracy": 0,
}

def load_state():
    if os.path.isfile(STATE_FILE):
        try:
            with open(STATE_FILE, encoding="utf-8") as f:
                s = json.load(f)
            print("[state] Loaded session " + str(s.get("session", 0)) + " from " + STATE_FILE)
            return s
        except Exception as e:
            print("[state] Load failed (" + str(e) + "), starting fresh")
    return dict(EMPTY_STATE)

def save_state(s):
    os.makedirs(os.path.dirname(STATE_FILE), exist_ok=True)
    with open(STATE_FILE, "w", encoding="utf-8") as f:
        json.dump(s, f, indent=2)
    print("[state] Saved session " + str(s["session"]) + " to " + STATE_FILE)

# ---- Run helpers ------------------------------------------------------------

def run_bin(path, timeout=30):
    t0 = time.time()
    r = subprocess.run(
        [BIN, "run", path],
        capture_output=True, text=True,
        encoding="utf-8", errors="replace",
        timeout=timeout,
    )
    return r.stdout, r.stderr, time.time() - t0

def run_qiskit():
    if not os.path.isfile(QK_RUNNER):
        return {"dry_run": True, "circuit_str": "qiskit_runner not found"}
    py = sys.executable
    try:
        r = subprocess.run(
            [py, QK_RUNNER] + BELL_DESC.split(),
            capture_output=True, text=True, timeout=60,
        )
        raw = r.stdout.strip()
        if not raw:
            return {"error": "empty output", "stderr": r.stderr[:100]}
        return json.loads(raw)
    except Exception as e:
        return {"error": str(e)}

def extract_qasm(stdout):
    inside, lines = False, []
    for line in stdout.splitlines():
        s = line.strip()
        if s == "QASM_START": inside = True; continue
        if s == "QASM_END":   break
        if inside: lines.append(line)
    return "\n".join(lines).strip() if lines else None

def parse_vm_signal(stdout):
    """Extract verdict (int) and conf (int) from quantum_hive_demo.ai output."""
    verdict, conf = 2, 16  # defaults
    for line in stdout.splitlines():
        line = line.strip()
        if line.startswith("Oracle decision"):
            for part in line.split():
                if part.startswith("confidence="):
                    try: conf = int(float(part.split("=")[1]))
                    except: pass
        if "Conductor verdict" in line:
            if "ABORT"       in line: verdict = 0
            elif "HOLD"      in line: verdict = 1
            elif "ACCELERATE" in line: verdict = 3
            elif "PROCEED"   in line: verdict = 2
    return verdict, conf

def entanglement_pct(qresult):
    if "error" in qresult or qresult.get("dry_run"):
        return 100  # assume max if sim unavailable
    counts = qresult.get("counts", {})
    total  = qresult.get("total_shots", 1)
    corr   = counts.get("00", 0) + counts.get("11", 0)
    return int(corr * 100 / total)

def verdict_str(v):
    return ["ABORT", "HOLD", "PROCEED", "ACCELERATE"][int(v)] if 0 <= int(v) <= 3 else "?"

def adaptive_threshold(acc):
    if acc >= 80: return 50
    if acc >= 60: return 55
    return 62

# ---- Session .ai generator --------------------------------------------------

MOTHER_FUNCTIONS = '''
◯ a_abs⟨n⟩ {
    if (n < 0) { return 0 - n; }
    return n;
}
◯ a_clamp⟨v, lo, hi⟩ {
    if (v < lo) { return lo; }
    if (v > hi) { return hi; }
    return v;
}
◯ b_j_append⟨j, tag, payload⟩ {
    j.push(tag); j.push(payload); j.push(0);
    return j;
}
◯ b_m_recall⟨m, key⟩ {
    let total = len(m) / 3; let i = 0;
    while (i < total) {
        let base = i * 3;
        let k = m.slice(base, base + 1).pop();
        if (k == key) { return m.slice(base + 1, base + 2).pop(); }
        i = i + 1;
    }
    return 0;
}
◯ b_m_store⟨m, key, val⟩ {
    let total = len(m) / 3; let found = -1; let i = 0;
    while (i < total) {
        let k = m.slice(i * 3, i * 3 + 1).pop();
        if (k == key) { found = i; }
        i = i + 1;
    }
    if (found == -1) { m.push(key); m.push(val); m.push(1); return m; }
    let result = []; let j = 0;
    while (j < total) {
        let b = j * 3;
        let ek = m.slice(b, b + 1).pop();
        let ev = m.slice(b + 1, b + 2).pop();
        let es = m.slice(b + 2, b + 3).pop();
        if (j == found) { result.push(ek); result.push(val);  result.push(1);  }
        else            { result.push(ek); result.push(ev);   result.push(es); }
        j = j + 1;
    }
    return result;
}
◯ c_learn⟨m, verdict⟩ {
    let total = len(m) / 3; let d = []; let i = 0;
    while (i < total) {
        let b = i * 3;
        d.push(m.slice(b, b+1).pop());
        d.push(m.slice(b+1, b+2).pop());
        d.push(m.slice(b+2, b+3).pop() * 9 / 10);
        i = i + 1;
    }
    let is_pos = 0; if (verdict == 2) { is_pos = 1; } if (verdict == 3) { is_pos = 1; }
    if (is_pos == 1) {
        let pc = b_m_recall(d, "proceed_count");
        d = b_m_store(d, "proceed_count", pc + 1);
    } else {
        let ac = b_m_recall(d, "abort_count");
        d = b_m_store(d, "abort_count", ac + 1);
    }
    return d;
}
◯ c_observe⟨s, pred, outcome, conf⟩ {
    let preds = b_m_recall(s, "predictions");
    let corr  = b_m_recall(s, "correct");
    let csum  = b_m_recall(s, "conf_sum");
    let drift = b_m_recall(s, "drift");
    s = b_m_store(s, "predictions", preds + 1);
    s = b_m_store(s, "last_pred",   pred);
    s = b_m_store(s, "last_out",    outcome);
    let hit = 0; if (pred == outcome) { hit = 1; }
    if (hit == 1) {
        s = b_m_store(s, "correct",  corr + 1);
        s = b_m_store(s, "conf_sum", csum + conf);
    }
    s = b_m_store(s, "drift", drift + a_abs(pred - outcome));
    return s;
}
◯ c_verdict_str⟨v⟩ {
    if (v == 0) { return "ABORT"; }
    if (v == 1) { return "HOLD"; }
    if (v == 2) { return "PROCEED"; }
    return "ACCELERATE";
}
◯ d_accuracy⟨s⟩ {
    let preds = b_m_recall(s, "predictions");
    let corr  = b_m_recall(s, "correct");
    if (preds == 0) { return 0; }
    return a_clamp(corr * 100 / preds, 0, 100);
}
◯ d_bias⟨m⟩ {
    let pc  = b_m_recall(m, "proceed_count");
    let ac  = b_m_recall(m, "abort_count");
    let tot = pc + ac;
    if (tot == 0) { return 50; }
    return a_clamp(pc * 100 / tot, 0, 100);
}
◯ d_threshold⟨s⟩ {
    let acc = d_accuracy(s);
    if (acc >= 80) { return 50; }
    if (acc >= 60) { return 55; }
    return 62;
}'''

def generate_session_ai(state, verdict, conf, ent, session_num):
    """Generate mother_session.ai with prior state baked in + live signal."""
    pc   = state["proceed_count"]
    ac   = state["abort_count"]
    preds = state["predictions"]
    corr  = state["correct"]
    csum  = state["conf_sum"]
    drift = state["drift"]
    lv    = state["last_verdict"]
    lc    = state["last_conf"]
    vstr  = verdict_str(verdict)

    # Prior state injected via push calls (safe for all value types)
    m_init = ""
    if pc > 0: m_init += f"    m.push(\"proceed_count\"); m.push({pc}); m.push(1);\n"
    if ac > 0: m_init += f"    m.push(\"abort_count\"); m.push({ac}); m.push(1);\n"
    if lv >= 0:
        m_init += f"    m.push(\"last_verdict\"); m.push({lv}); m.push(1);\n"
        m_init += f"    m.push(\"last_conf\"); m.push({lc}); m.push(1);\n"

    s_init = ""
    if preds > 0:
        s_init += f"    s.push(\"predictions\"); s.push({preds}); s.push(1);\n"
        s_init += f"    s.push(\"correct\"); s.push({corr}); s.push(1);\n"
        s_init += f"    s.push(\"conf_sum\"); s.push({csum}); s.push(1);\n"
        s_init += f"    s.push(\"drift\"); s.push({drift}); s.push(1);\n"

    # Mother predicts same as last verdict (simple persistence-based prediction)
    pred = lv if lv >= 0 else verdict

    ai = f'''⍝ mother_session.ai -- generated by full_pipeline.py
⍝ Session {session_num} | verdict={vstr} conf={conf} ent={ent}%
⍝ Prior state loaded from mother_state.json (session {session_num - 1})
⍝ 12 fns + main = 13 total (at budget limit)
{MOTHER_FUNCTIONS}
◯ main⟨⟩ {{
    let j = []; let m = []; let s = [];

    ⍝ -- Prior state from session {session_num - 1} --
{m_init}{s_init}
    ⍝ -- Live quantum signal: {vstr} conf={conf} ent={ent}% --
    j = b_j_append(j, "quantum_signal", "{vstr}");
    j = b_j_append(j, "entanglement", "{ent}pct");
    m = c_learn(m, {verdict});
    m = b_m_store(m, "last_verdict", {verdict});
    m = b_m_store(m, "last_conf", {conf});
    s = c_observe(s, {pred}, {verdict}, {conf});

    let bias  = d_bias(m);
    let acc   = d_accuracy(s);
    let thr   = d_threshold(s);
    let preds = b_m_recall(s, "predictions");
    let corr2 = b_m_recall(s, "correct");
    let dr    = b_m_recall(s, "drift");
    let pc2   = b_m_recall(m, "proceed_count");
    let ac2   = b_m_recall(m, "abort_count");

    print("MOTHER_SESSION_START");
    print("session=" + {session_num});
    print("verdict={vstr}");
    print("conf={conf}");
    print("ent={ent}");
    print("bias=" + bias);
    print("accuracy=" + acc);
    print("threshold=" + thr);
    print("predictions=" + preds);
    print("correct=" + corr2);
    print("drift=" + dr);
    print("proceed_count=" + pc2);
    print("abort_count=" + ac2);
    print("MOTHER_SESSION_END");
}}
'''
    with open(SESSION_AI, "w", encoding="utf-8") as f:
        f.write(ai)

def parse_session_output(stdout):
    """Extract key=value pairs from MOTHER_SESSION_START/END block."""
    inside, data = False, {}
    for line in stdout.splitlines():
        s = line.strip()
        if "MOTHER_SESSION_START" in s: inside = True; continue
        if "MOTHER_SESSION_END"   in s: break
        if inside and "=" in s:
            k, _, v = s.partition("=")
            try: data[k] = float(v)
            except: data[k] = v
    return data

# ---- Main -------------------------------------------------------------------

def main():
    print()
    print("*** AEONMI FULL PIPELINE ***")
    print("    Quantum Hive -> Mother Core (persistent, adaptive)")
    print(SEP)

    if not os.path.isfile(BIN):
        print("[ERROR] Binary not found:", BIN)
        sys.exit(1)

    # 1. Load prior state
    state    = load_state()
    session  = state["session"] + 1
    print()

    # 2. Run quantum_hive_demo.ai
    print("[1] Running quantum_hive_demo.ai...")
    qhstdout, qhstderr, qhtime = run_bin(DEMO_AI)
    if not qhstdout.strip():
        print("[ERROR] VM returned empty output. stderr:", qhstderr[:200])
        sys.exit(1)
    verdict, conf = parse_vm_signal(qhstdout)
    print("    Oracle verdict : " + verdict_str(verdict) + "  conf=" + str(conf) + "  (" + "{:.2f}".format(qhtime) + "s)")

    # 3. Qiskit simulation
    print("[2] Running Qiskit Aer (Bell circuit)...")
    qresult = run_qiskit()
    ent = entanglement_pct(qresult)
    if qresult.get("dry_run"):
        print("    Dry run (no Qiskit or qiskit_runner not found)")
    elif "error" in qresult:
        print("    Qiskit error:", qresult["error"])
    else:
        counts  = qresult.get("counts", {})
        total   = qresult.get("total_shots", 1)
        most    = qresult.get("most_likely", "?")
        print("    Bell result    : most_likely=|" + str(most) + ">  entanglement=" + str(ent) + "%")
        for st, cnt in sorted(counts.items(), key=lambda x: -x[1]):
            pct = cnt / total * 100
            bar = "#" * int(pct / 5)
            print("      |" + st + ">  " + str(cnt).rjust(5) + " (" + "{:.1f}".format(pct) + "%)  " + bar)

    # 4. Generate + run mother_session.ai
    print("[3] Generating mother_session.ai (session " + str(session) + ")...")
    generate_session_ai(state, verdict, conf, ent, session)
    print("[4] Running Mother Core (adaptive)...")
    mstdout, mstderr, mtime = run_bin(SESSION_AI, timeout=60)
    if not mstdout.strip():
        print("[ERROR] Mother session returned empty output")
        if mstderr: print("[STDERR]", mstderr[:400])
        sys.exit(1)
    print("    Done in " + "{:.2f}".format(mtime) + "s")

    # 5. Parse + save state
    data = parse_session_output(mstdout)
    if not data:
        print("[WARN] Could not parse session output")
        print(mstdout[:400])
    else:
        state["session"]        = session
        state["proceed_count"]  = data.get("proceed_count", state["proceed_count"])
        state["abort_count"]    = data.get("abort_count",   state["abort_count"])
        state["predictions"]    = data.get("predictions",   state["predictions"])
        state["correct"]        = data.get("correct",       state["correct"])
        state["conf_sum"]       = data.get("conf_sum",      state["conf_sum"])
        state["drift"]          = data.get("drift",         state["drift"])
        state["last_verdict"]   = verdict
        state["last_conf"]      = conf
        state["last_threshold"] = data.get("threshold",     55)
        state["last_accuracy"]  = data.get("accuracy",      0)
        save_state(state)

    # 6. Print unified story
    print()
    print(SEP)
    print("SESSION " + str(session) + " COMPLETE")
    print(SEP)
    print("  Quantum signal  : " + verdict_str(verdict) + "  conf=" + str(conf) + "  ent=" + str(ent) + "%")
    print("  Learned bias    : " + str(round(data.get("bias", 0), 1)) + "%  (toward PROCEED)")
    print("  Accuracy        : " + str(round(data.get("accuracy", 0), 1)) + "%")
    print("  Threshold       : " + str(int(data.get("threshold", 55))) + "  (adapted)")
    print("  Predictions     : " + str(int(data.get("predictions", 0))) + "  correct=" + str(int(data.get("correct", 0))) + "  drift=" + str(int(data.get("drift", 0))))
    print("  Total sessions  : " + str(session))
    thr  = int(data.get("threshold", 55))
    bias = data.get("bias", 50)
    if thr == 50:   regime = "AGGRESSIVE  (accuracy >= 80%)"
    elif thr == 55: regime = "BALANCED    (accuracy 60-79%)"
    else:           regime = "CONSERVATIVE (accuracy < 60%)"
    print("  Regime          : " + regime)
    print(SEP)
    print("*** MOTHER ONLINE — run again to accumulate ***")
    print()

if __name__ == "__main__":
    main()
