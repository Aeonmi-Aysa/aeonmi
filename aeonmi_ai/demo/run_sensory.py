#!/usr/bin/env python3
"""run_sensory.py -- runs Phase 2 + Phase 3 tests and demos"""
import os, sys, subprocess, time

REPO   = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))
BIN    = os.path.join(REPO, "target", "release", "aeonmi.exe")
SENS   = os.path.join(REPO, "aeonmi_ai", "sensory")
FEED   = os.path.join(SENS, "quantum_feed.ai")
TEST   = os.path.join(SENS, "quantum_feed_test.ai")
LEARN  = os.path.join(REPO, "aeonmi_ai", "learning")
LOOP   = os.path.join(LEARN, "loop.ai")
LTEST  = os.path.join(LEARN, "loop_test.ai")
SMOD   = os.path.join(REPO, "aeonmi_ai", "selfmodel")
MODEL  = os.path.join(SMOD, "model.ai")
MTEST  = os.path.join(SMOD, "model_test.ai")
CORE   = os.path.join(REPO, "aeonmi_ai", "core", "mother_core.ai")

os.environ["AEONMI_NATIVE"] = "1"
SEP = "=" * 50

def run(label, path):
    t0 = time.time()
    r  = subprocess.run([BIN, "run", path], capture_output=True,
                        text=True, encoding="utf-8", errors="replace", timeout=30)
    elapsed = time.time() - t0
    print(SEP)
    print(f"{label}  ({elapsed:.2f}s)")
    print(SEP)
    for line in r.stdout.splitlines():
        if line.strip():
            print("  " + line)
    if r.stderr.strip():
        print("[STDERR]", r.stderr[:200])
    passed = r.stdout.count("[PASS]")
    failed = r.stdout.count("[FAIL]")
    return passed, failed

if not os.path.isfile(BIN):
    print("Binary not found:", BIN); sys.exit(1)

p1, f1 = run("quantum_feed.ai  (demo)", FEED)
p2, f2 = run("quantum_feed_test.ai", TEST)
p3, f3 = run("loop.ai  (demo)", LOOP)
p4, f4 = run("loop_test.ai", LTEST)
p5, f5 = run("model.ai  (demo)", MODEL)
p6, f6 = run("model_test.ai", MTEST)
p7, f7 = run("mother_core.ai  (unified)", CORE)

print(SEP)
total_p = p1 + p2 + p3 + p4 + p5 + p6 + p7
total_f = f1 + f2 + f3 + f4 + f5 + f6 + f7
print(f"Results: {total_p} PASS  {total_f} FAIL")
if total_f == 0 and total_p > 0:
    print("=== ALL PASS ===")
