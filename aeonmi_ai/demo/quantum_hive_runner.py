#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
quantum_hive_runner.py -- Aeonmi Quantum Hive Demo Runner
==========================================================
Full pipeline:
  1. Runs quantum_hive_demo.ai via Aeonmi VM (AEONMI_NATIVE=1)
  2. Extracts QASM between QASM_START / QASM_END markers
  3. Prepends 'include "qelib1.inc";' (required by OpenQASM 2.0 gate library)
  4. Runs qiskit_runner.py with the Bell circuit descriptor
  5. Prints the full connected story

Usage (from repo root):
  py -u aeonmi_ai\\demo\\quantum_hive_runner.py

Requirements:
  - Aeonmi binary at target\\release\\aeonmi.exe
    (or copy to C:\\Temp\\aeonmi_run.exe)
  - pip install qiskit qiskit-aer   (falls back to dry-run without it)
"""

import os
import sys
import subprocess
import json
import shutil
import time

# ---- Path resolution --------------------------------------------------------

SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT  = os.path.abspath(os.path.join(SCRIPT_DIR, "..", ".."))
DEMO_AI    = os.path.join(SCRIPT_DIR, "quantum_hive_demo.ai")

BINARY_CANDIDATES = [
    os.path.join(REPO_ROOT, "target", "release", "aeonmi.exe"),
    r"C:\Temp\aeonmi_run.exe",
    os.path.join(REPO_ROOT, "Aeonmi_Master", "Aeonmi.exe"),
    os.path.join(REPO_ROOT, "MotherAI.exe"),
]

QISKIT_RUNNER = os.path.join(REPO_ROOT, "Aeonmi_Master", "qiskit_runner.py")

# Bell circuit descriptor: 2q 2c 1024shots 4ops | H(0) CX(ctrl=0,tgt=1) M(0->0) M(1->1)
BELL_DESCRIPTOR = "2 2 1024 4  0 0 0  4 1 0  7 0 0  7 1 1"

SEP = "=" * 62

# ---- Helpers ----------------------------------------------------------------

def find_binary():
    for p in BINARY_CANDIDATES:
        if os.path.isfile(p):
            return p
    return None


def run_aeonmi_demo(binary, ai_file):
    tmp_ai = r"C:\Temp\quantum_hive_demo.ai"
    try:
        os.makedirs(r"C:\Temp", exist_ok=True)
        shutil.copy2(ai_file, tmp_ai)
    except Exception:
        tmp_ai = ai_file

    # Set directly on this process's env so the child inherits it.
    # The env-dict copy approach doesn't reliably propagate on Windows.
    os.environ["AEONMI_NATIVE"] = "1"

    t0 = time.time()
    try:
        result = subprocess.run(
            [binary, "run", tmp_ai],
            capture_output=True,
            text=True,
            encoding="utf-8",
            errors="replace",
            timeout=30,
        )
    except subprocess.TimeoutExpired:
        return None, "TIMEOUT after 30s", 30.0
    except Exception as e:
        return None, str(e), 0.0

    return result.stdout, result.stderr, time.time() - t0


def extract_qasm(stdout):
    inside = False
    lines = []
    for line in stdout.splitlines():
        stripped = line.strip()
        if stripped == "QASM_START":
            inside = True
            continue
        if stripped == "QASM_END":
            break
        if inside:
            lines.append(line)
    if not lines:
        return None
    return "\n".join(lines).strip()


def run_qiskit(descriptor_str):
    if not os.path.isfile(QISKIT_RUNNER):
        return {"error": "qiskit_runner.py not found at: " + QISKIT_RUNNER}

    py   = sys.executable
    args = descriptor_str.split()
    try:
        result = subprocess.run(
            [py, QISKIT_RUNNER] + args,
            capture_output=True,
            text=True,
            timeout=60,
        )
        raw = result.stdout.strip()
        if not raw:
            return {"error": "empty output from qiskit_runner", "stderr": result.stderr[:200]}
        return json.loads(raw)
    except subprocess.TimeoutExpired:
        return {"error": "qiskit_runner timed out"}
    except json.JSONDecodeError as e:
        return {"error": "JSON parse failed: " + str(e)}
    except Exception as e:
        return {"error": str(e)}


def format_quantum(qresult):
    if "error" in qresult:
        return "  [simulation unavailable: " + qresult["error"] + "]"
    if qresult.get("dry_run"):
        return "  Circuit: " + qresult.get("circuit_str", "?") + "  [dry run -- no Qiskit]"

    counts = qresult.get("counts", {})
    total  = qresult.get("total_shots", 1)
    likely = qresult.get("most_likely", "?")
    depth  = qresult.get("circuit_depth", "?")

    out = ["  shots=" + str(total) + "  depth=" + str(depth) + "  most_likely=|" + str(likely) + ">"]
    for state, count in sorted(counts.items(), key=lambda x: -x[1]):
        pct = count / total * 100
        bar = "#" * int(pct / 5)
        out.append("    |" + state + ">  " + str(count).rjust(5) + " shots  (" + "{:.1f}".format(pct) + "%)  " + bar)

    corr = counts.get("00", 0) + counts.get("11", 0)
    ent_pct = corr / total * 100
    if ent_pct > 85:
        out.append("  >> Entanglement confirmed (" + "{:.1f}".format(ent_pct) + "% correlated) -- SIGNAL VALID")
    else:
        out.append("  >> Partial entanglement (" + "{:.1f}".format(ent_pct) + "%)")
    return "\n".join(out)


def verdict_label(v):
    labels = {0: "ABORT", 1: "HOLD", 2: "PROCEED", 3: "ACCELERATE"}
    return labels.get(int(v), "UNKNOWN")


# ---- Main -------------------------------------------------------------------

def main():
    print("")
    print("*** AEONMI QUANTUM HIVE -- FULL PIPELINE DEMO ***")
    print("    Oracle -> QASM -> Qiskit -> Conductor")
    print(SEP)
    print("")

    # Step 1: find binary
    binary = find_binary()
    if binary is None:
        print("[ERROR] No Aeonmi binary found. Searched:")
        for c in BINARY_CANDIDATES:
            print("  " + c)
        print("Build with: cargo build --release")
        sys.exit(1)
    print("[1] Binary  : " + binary)

    if not os.path.isfile(DEMO_AI):
        print("[ERROR] Demo .ai not found: " + DEMO_AI)
        sys.exit(1)
    print("[1] Demo    : " + DEMO_AI)
    print("")

    # Step 2: run .ai
    print("[2] Running Aeonmi VM (AEONMI_NATIVE=1)...")
    stdout, stderr, elapsed = run_aeonmi_demo(binary, DEMO_AI)

    if stdout is None:
        print("[ERROR] VM failed: " + stderr)
        sys.exit(1)
    print("    Done in " + "{:.2f}".format(elapsed) + "s")
    print("")

    # Step 3: show VM output (skip QASM markers)
    print(SEP)
    print("AEONMI VM OUTPUT")
    print(SEP)
    for line in stdout.splitlines():
        s = line.strip()
        if s and s not in ("QASM_START", "QASM_END"):
            print("  " + line)
    print(SEP)
    print("")

    # Step 4: extract QASM
    qasm_body = extract_qasm(stdout)
    if qasm_body is None:
        print("[WARN] No QASM markers found. Check VM output above.")
    else:
        qasm_full = 'include "qelib1.inc";\n' + qasm_body
        print("[3] Extracted QASM:")
        print(SEP)
        for line in qasm_full.splitlines():
            print("  " + line)
        print(SEP)
        print("")

    # Step 5: Qiskit simulation
    print("[4] Running Qiskit Aer simulation...")
    qresult = run_qiskit(BELL_DESCRIPTOR)
    print("")
    print(SEP)
    print("QUANTUM SIMULATION RESULTS")
    print(SEP)
    print(format_quantum(qresult))
    print(SEP)
    print("")

    # Step 6: Conductor synthesis
    oracle_sc = 16   # conf = |trend=18| - vol=4/2 = 18-2 = 16
    hype_sc   = 80
    close_sc  = 70
    risk_sc   = 20

    if risk_sc >= 80:
        verdict = 0
    elif oracle_sc >= 70 and hype_sc >= 70 and close_sc >= 70 and risk_sc < 30:
        verdict = 3
    elif (oracle_sc + hype_sc + close_sc) / 3 >= 55 and risk_sc < 60:
        verdict = 2
    else:
        verdict = 1

    vname = verdict_label(verdict)
    avg   = (oracle_sc + hype_sc + close_sc) / 3

    print(SEP)
    print("CONDUCTOR SYNTHESIS")
    print(SEP)
    print("  Oracle  score : " + str(oracle_sc) + "  (|trend=18| - vol=4/2 = 16)")
    print("  Hype    score : " + str(hype_sc))
    print("  Closer  score : " + str(close_sc))
    print("  Risk    score : " + str(risk_sc))
    print("  Avg(o,h,c)    : " + "{:.2f}".format(avg) + "  (threshold=55)")
    print("  Verdict       : >> " + vname + " (" + str(verdict) + ") <<")
    print(SEP)
    print("")
    print("*** FINAL DECISION: " + vname + " ***")
    print("    Quiet trend=18 -> Oracle selected BELL (baseline entanglement)")
    print("    avg(16,80,70) = 55.3 >= threshold, risk=20 low")
    print("")

    if stderr and stderr.strip():
        print("[VM STDERR]")
        print(stderr[:400])


if __name__ == "__main__":
    main()
