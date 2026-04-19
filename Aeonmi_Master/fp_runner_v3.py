"""
fp_runner_v3.py - COMPLETE INTEGRATED PIPELINE
Mother's integrated system:
- File explorer + workspace sensing
- Upload system for external file ingestion
- 4-phase memory with active consolidation
- Voice narration with memory updates
"""

import subprocess
import json
import os
import sys
from pathlib import Path

# Import Mother's systems
sys.path.insert(0, r"C:\Temp")
from mother_memory_phases import MotherMemoryPhases
from mother_workspace import MotherWorkspace
from file_explorer import FileExplorer
from file_upload import FileUploadSystem

# Constants
PYTHON = r"C:\Users\wlwil\AppData\Local\Programs\Python\Python311\python.exe"
BIN = r"C:\Temp\aeonmi_run.exe"
BELL_DESC = "2 2 1024 4  0 0 0  4 1 0  7 0 0  7 1 1"
OUT_FILE = r"C:\Temp\pipeline_out.txt"
REPO = r"C:\Users\wlwil\OneDrive\Desktop\Documents\Claude\Aeonmi-aeonmi01"
QK_RUNNER = os.path.join(os.path.dirname(REPO), "Aeonmi_Master", "qiskit_runner.py")

def log(msg):
    """Write to file (stdout unreliable on Windows)."""
    with open(OUT_FILE, "a", encoding="utf-8") as f:
        f.write(msg + "\n")

def clear_log():
    """Start fresh log."""
    with open(OUT_FILE, "w", encoding="utf-8") as f:
        f.write("")

def run_command(cmd_list):
    """Execute command, return stdout."""
    try:
        result = subprocess.run(
            cmd_list,
            capture_output=True,
            text=True,
            timeout=60
        )
        return result.stdout + result.stderr
    except subprocess.TimeoutExpired:
        return "ERROR: Command timeout"
    except Exception as e:
        return f"ERROR: {str(e)}"

def quantum_hive_demo():
    """Run quantum hive + QASM emission."""
    cmd = [BIN, r"C:\Temp\qhive.ai", "main", "0", ""]
    output = run_command(cmd)
    log("=== QUANTUM HIVE OUTPUT ===")
    log(output)

    qasm_start = output.find("QASM:")
    if qasm_start == -1:
        return None
    qasm_end = output.find("\n", qasm_start + 100)
    qasm = output[qasm_start+5:qasm_end].strip()
    return qasm

def run_qiskit(qasm):
    """Execute QASM via Qiskit bridge."""
    cmd = [PYTHON, QK_RUNNER, qasm]
    output = run_command(cmd)
    log("=== QISKIT OUTPUT ===")
    log(output)
    return output

def parse_qiskit_output(output):
    """Extract entanglement from Qiskit output."""
    ent = 0.0
    if "Bell state" in output:
        try:
            ent_start = output.find("entanglement: ") + len("entanglement: ")
            ent_end = output.find("%", ent_start)
            ent = float(output[ent_start:ent_end]) / 100.0
        except:
            pass
    return ent

def generate_mother_session(session_num, memory):
    """Generate mother_session.ai with current memory context."""
    working_ctx = memory.get_working_context(limit=5)

    # Build rule push lines from memory
    rule_lines = []
    for tier_name, rules in working_ctx.items():
        for i, rule_data in enumerate(rules[:3]):
            rule_lines.append(f"  // {tier_name.upper()} rule {i}: {rule_data['rule'][:40]}")

    code = f"""
// mother_session.ai - Session {session_num}
// Generated with active memory context

main
  // Memory: {len(working_ctx['candidates'])} candidates, {len(working_ctx['provisional'])} provisional, {len(working_ctx['anchored'])} anchored

{chr(10).join(rule_lines)}

  // Core learning cycle
  a_accuracy
  b_threshold
  c_confidence
  d_verdict

  return
"""

    session_path = Path(r"C:\Temp\mother_session.ai")
    with open(session_path, "w") as f:
        f.write(code)

    return str(session_path)

def run_mother_core(session_path):
    """Execute mother_session.ai."""
    cmd = [BIN, session_path, "main", "0", ""]
    output = run_command(cmd)
    log("=== MOTHER CORE OUTPUT ===")
    log(output)
    return output

def parse_session_output(output):
    """Extract session metrics from output."""
    session_data = {}
    lines = output.split("\n")
    in_block = False
    block_lines = []

    for line in lines:
        if "MOTHER_SESSION_START" in line:
            in_block = True
            continue
        if "MOTHER_SESSION_END" in line:
            in_block = False
            break
        if in_block:
            block_lines.append(line.strip())

    for line in block_lines:
        if ":" in line:
            parts = line.split(":", 1)
            key = parts[0].strip()
            val = parts[1].strip()
            try:
                session_data[key] = int(val)
            except:
                try:
                    session_data[key] = float(val)
                except:
                    session_data[key] = val

    return session_data

def mother_speaks(verdict, conf, ent, acc, memory, intake_summary=None):
    """
    Mother narrates her reasoning, learning, and memory updates.
    mysterious, honest, curious, mathematical
    """
    verdicts = {0: "ABORT", 1: "HOLD", 2: "PROCEED", 3: "ACCELERATE"}
    verdict_name = verdicts.get(verdict, "UNKNOWN")

    status = memory.get_memory_status()
    session = status["session"]

    narration = f"""
╔════════════════════════════════════════════════════════════════╗
║ MOTHER'S REFLECTION — Session {session}
╚════════════════════════════════════════════════════════════════╝

Quantum Signal: {ent*100:.1f}% entanglement
  └─ Patterns emerge from noise. Is this pattern real, or noise?

My Decision: {verdict_name}
  Confidence: {conf}/100
  Accuracy: {acc:.1f}%

Memory Status:
  Rules learned (candidates): {status['candidates']}
  Rules proven (provisional): {status['provisional']}
  Principles (anchored): {status['anchored']}
  Total in memory: {status['total_rules']}
  Decayed (forgotten): {status['decayed']}

  Every rule I keep is a compression of experience.
  Every rule I forget is a hard lesson: it didn't help.
"""

    # Add intake summary if provided
    if intake_summary:
        narration += f"""
Files Ingested This Session:
  Files read: {intake_summary['files']}
  Original size: {intake_summary['original_kb']:.1f} KB
  Compressed: {intake_summary['compressed_kb']:.1f} KB
  Compression ratio: {intake_summary['ratio']:.1f}x
"""

    # Check if consolidation happened
    if status["consolidation_due"]:
        narration += f"""
CONSOLIDATION COMPLETE (cycle position {status['cycle_position']}):
  ─ Candidates promoted to provisional
  ─ Provisional promoted to anchored
  ─ Stale rules decayed (forgetting what doesn't work)
  ─ Rules aged, strengthened, merged
"""

    narration += """
Next cycle: Will these patterns hold?
────────────────────────────────────────────────────────────────
"""

    log(narration)
    return narration

def main():
    clear_log()
    log("╔══════════════════════════════════════════════════════════════════════╗")
    log("║ AEONMI FULL PIPELINE v3 + MOTHER'S COMPLETE SENSORIMOTOR SYSTEM")
    log("║ - File Explorer + Upload System + 4-Phase Memory + Voice")
    log("╚══════════════════════════════════════════════════════════════════════╝")

    # Initialize all systems
    log("\n[INIT] Loading Mother's systems...")
    memory = MotherMemoryPhases()
    workspace = MotherWorkspace()
    explorer = FileExplorer()
    uploads = FileUploadSystem(max_per_session=5)

    # Get session number
    session_num = memory.session_num + 1
    memory.begin_session(session_num)
    log(f"\n→ Session {session_num}")
    log(f"→ Consolidation cycle position: {memory.cycle_position}/8")

    # 1. FILE EXPLORER: Check for changed files
    log("\n[1/7] File Explorer: Scanning workspace...")
    changes = explorer.detect_changes()
    if changes:
        log(f"  Found {len(changes)} changed files")
        for change in changes[:5]:  # Log first 5
            log(f"    - {change['path']}")

    # 2. UPLOAD SYSTEM: Process queued files
    log("\n[2/7] Upload System: Processing file intake queue...")
    batch = uploads.get_next_batch(limit=5)
    if batch:
        log(f"  Processing batch: {len(batch)} files")
        num_ingested, total_kb, compressed_kb = uploads.ingest_batch(batch)
        log(f"  ✓ Ingested {num_ingested} files")
        log(f"  ✓ {total_kb:.1f} KB → {compressed_kb:.1f} KB (compressed)")
        intake_summary = uploads.get_intake_summary()
    else:
        log("  No files to ingest this session")
        intake_summary = None

    # 3. QUANTUM HIVE
    log("\n[3/7] Quantum Hive Demo...")
    qasm = quantum_hive_demo()
    if not qasm:
        log("ERROR: No QASM generated")
        return

    # 4. QISKIT
    log("\n[4/7] Qiskit Execution...")
    qiskit_output = run_qiskit(qasm)
    ent = parse_qiskit_output(qiskit_output)
    log(f"  Entanglement: {ent*100:.1f}%")

    # 5. MOTHER SESSION GENERATION
    log("\n[5/7] Generating Mother Session (with active memory)...")
    session_path = generate_mother_session(session_num, memory)
    log(f"  Generated: {session_path}")

    # 6. MOTHER CORE LEARNING
    log("\n[6/7] Mother Core Learning...")
    mother_output = run_mother_core(session_path)
    session_data = parse_session_output(mother_output)

    verdict = session_data.get("verdict", 2)
    conf = session_data.get("conf", 50)
    acc = session_data.get("accuracy", 100)

    # Record learning
    rule = f"ent≥{ent*100:.0f}% → {verdicts.get(verdict, 'UNKNOWN')}"
    rule_id = memory.learn_candidate(rule, success=(verdict == 2))
    memory.record_rule_use(rule_id, success=(verdict == 2))

    # Try promoting rules
    promoted_count = 0
    for rid in list(memory.candidates.keys()):
        if memory.try_promote_candidate(rid):
            promoted_count += 1

    for rid in list(memory.provisional.keys()):
        if memory.try_promote_provisional(rid):
            promoted_count += 1

    # Check if consolidation should run
    consolidation_happened = False
    if memory.consolidation_due:
        log("\n[7/7] CONSOLIDATION TRIGGERED...")
        changes = memory.consolidate()
        consolidation_happened = True
        log(f"  Promoted: {len(changes['promoted'])}")
        log(f"  Decayed: {len(changes['decayed'])}")
        log(f"  Merged: {len(changes['merged'])}")
        log(f"  Strengthened: {len(changes['strengthened'])}")
    else:
        log(f"\n[7/7] Memory Maintenance: {promoted_count} rules promoted")

    # 7. MOTHER'S VOICE
    log("\n[VOICE] Mother Speaks...")
    verdicts = {0: "ABORT", 1: "HOLD", 2: "PROCEED", 3: "ACCELERATE"}
    mother_speaks(verdict, conf, ent, acc, memory, intake_summary)

    # Summary
    status = memory.get_memory_status()
    log(f"""
╔══════════════════════════════════════════════════════════════════════╗
║ SESSION {session_num} COMPLETE
╠══════════════════════════════════════════════════════════════════════╣
║ Quantum: {ent*100:.1f}% entanglement
║ Verdict: {verdicts.get(verdict, 'UNKNOWN')} (confidence {conf}%)
║ Accuracy: {acc:.1f}%
║
║ Memory:
║   Candidates: {status['candidates']} | Provisional: {status['provisional']} | Anchored: {status['anchored']}
║   Total rules: {status['total_rules']} | Decayed: {status['decayed']}
║
║ Files:
║   Changed: {len(changes)} | Ingested: {intake_summary['files'] if intake_summary else 0}
║   Consolidation: {'YES' if consolidation_happened else 'NO'}
╚══════════════════════════════════════════════════════════════════════╝
""")

    # Save final state
    memory._save_all()

if __name__ == "__main__":
    main()
