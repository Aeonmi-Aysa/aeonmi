#!/usr/bin/env python3
"""
qiskit_runner.py — Aeonmi Quantum Bridge  v2.0
===============================================
Executes an Aeonmi flat circuit descriptor on:
  • Qiskit Aer (local simulator — always available)
  • IBM Brisbane 127-qubit real hardware  (requires IBM_QUANTUM_TOKEN)
  • IonQ  (requires IONQ_API_KEY — future outlet)

Two outlets, both active — Aer for development, IBM/IonQ for production.

Aeonmi circuit descriptor format:
  [n_qubits, n_cbits, shots, op_count, op_type0, op_tgt0, op_ctrl0, ...]

Op types:
  0=H  1=X  2=Y  3=Z  4=CX  5=S  6=T  7=MEASURE

Usage:
  python qiskit_runner.py [descriptor args...]
  python qiskit_runner.py --backend [aer|ibm_brisbane|ibm_kyoto|ionq] [descriptor...]
  echo "2 2 1024 3 0 0 -1 4 1 0 7 0 0 7 1 1" | python qiskit_runner.py

Flags:
  --backend <name>   Target backend (default: aer)
  --shots <n>        Override shot count
  --vote             Enable 3-shot majority vote (noise mitigation)
  --fidelity         Compare Aer vs IBM and compute circuit fidelity
  --dry              Dry-run: describe circuit, no execution

Output (JSON):
  {
    "counts":         {"00": 512, "11": 512},
    "total_shots":    1024,
    "most_likely":    "11",
    "n_qubits":       2,
    "circuit_depth":  3,
    "backend":        "ibm_brisbane",
    "fidelity":       0.94,
    "vote_detail":    [...],
    "noise_mitigated": true
  }
"""

import sys
import json
import os
import argparse
from pathlib import Path

# ── Optional imports ────────────────────────────────────────────────────────

try:
    from qiskit import QuantumCircuit, QuantumRegister, ClassicalRegister
    from qiskit_aer import AerSimulator
    QISKIT_AVAILABLE = True
except ImportError:
    QISKIT_AVAILABLE = False

try:
    from qiskit_ibm_runtime import QiskitRuntimeService, SamplerV2 as Sampler
    from qiskit_ibm_runtime import Session
    from qiskit.transpiler.preset_passmanagers import generate_preset_pass_manager
    IBM_RUNTIME_AVAILABLE = True
except ImportError:
    IBM_RUNTIME_AVAILABLE = False

# IBM token from env
IBM_TOKEN  = os.environ.get("IBM_QUANTUM_TOKEN", "")
IONQ_KEY   = os.environ.get("IONQ_API_KEY", "")

# ── Circuit builder ─────────────────────────────────────────────────────────

def build_circuit(descriptor: list) -> tuple:
    """Build a Qiskit circuit from the Aeonmi flat descriptor.
    Returns (QuantumCircuit, shots)."""
    n_q      = int(descriptor[0])
    n_c      = int(descriptor[1])
    shots    = int(descriptor[2])
    op_count = int(descriptor[3])

    qr = QuantumRegister(n_q, 'q')
    cr = ClassicalRegister(n_c, 'c')
    qc = QuantumCircuit(qr, cr)

    for i in range(op_count):
        base = 4 + i * 3
        if base + 2 >= len(descriptor):
            break
        op_type = int(descriptor[base])
        op_tgt  = int(descriptor[base + 1])
        op_ctrl = int(descriptor[base + 2])

        if   op_type == 0: qc.h(qr[op_tgt])
        elif op_type == 1: qc.x(qr[op_tgt])
        elif op_type == 2: qc.y(qr[op_tgt])
        elif op_type == 3: qc.z(qr[op_tgt])
        elif op_type == 4: qc.cx(qr[op_ctrl], qr[op_tgt])
        elif op_type == 5: qc.s(qr[op_tgt])
        elif op_type == 6: qc.t(qr[op_tgt])
        elif op_type == 7:
            if op_tgt < n_c:
                qc.measure(qr[op_tgt], cr[op_tgt])

    return qc, shots

# ── Backend runners ──────────────────────────────────────────────────────────

def run_aer(descriptor: list, shots_override: int = None) -> dict:
    """Run on Qiskit Aer local simulator."""
    if not QISKIT_AVAILABLE:
        return {"error": "qiskit or qiskit_aer not installed",
                "install": "pip install qiskit qiskit-aer"}
    qc, shots = build_circuit(descriptor)
    if shots_override:
        shots = shots_override
    sim  = AerSimulator()
    job  = sim.run(qc, shots=shots)
    res  = job.result()
    counts = result_to_counts(res)
    return {
        "counts":       counts,
        "total_shots":  sum(counts.values()),
        "most_likely":  max(counts, key=counts.get) if counts else "",
        "n_qubits":     int(descriptor[0]),
        "circuit_depth": qc.depth(),
        "backend":      "aer",
        "fidelity":     1.0,
    }


def run_ibm(descriptor: list, backend_name: str = "ibm_brisbane",
            shots_override: int = None) -> dict:
    """Run on IBM real quantum hardware via qiskit-ibm-runtime."""
    if not IBM_RUNTIME_AVAILABLE:
        return {"error": "qiskit-ibm-runtime not installed",
                "install": "pip install qiskit-ibm-runtime",
                "backend": backend_name}
    if not IBM_TOKEN:
        return {"error": "IBM_QUANTUM_TOKEN env var not set",
                "hint": "export IBM_QUANTUM_TOKEN=your_token",
                "backend": backend_name}

    try:
        service = QiskitRuntimeService(channel="ibm_quantum", token=IBM_TOKEN)
        backend = service.backend(backend_name)
    except Exception as e:
        return {"error": f"IBM backend connection failed: {e}",
                "backend": backend_name}

    qc, shots = build_circuit(descriptor)
    if shots_override:
        shots = shots_override

    try:
        pm = generate_preset_pass_manager(backend=backend, optimization_level=1)
        isa_circuit = pm.run(qc)

        with Session(backend=backend) as session:
            sampler = Sampler(mode=session)
            job = sampler.run([isa_circuit], shots=shots)
            result = job.result()

        # Extract counts from SamplerV2 result format
        pub_result = result[0]
        bitarray = pub_result.data.c
        counts_raw = bitarray.get_counts()

        total = sum(counts_raw.values())
        most_likely = max(counts_raw, key=counts_raw.get) if counts_raw else ""

        return {
            "counts":       counts_raw,
            "total_shots":  total,
            "most_likely":  most_likely,
            "n_qubits":     int(descriptor[0]),
            "circuit_depth": isa_circuit.depth(),
            "backend":      backend_name,
            "fidelity":     None,  # computed separately if --fidelity flag
        }
    except Exception as e:
        return {"error": f"IBM execution failed: {e}", "backend": backend_name}


def run_ionq(descriptor: list, shots_override: int = None) -> dict:
    """IonQ backend — requires qiskit-ionq package."""
    try:
        from qiskit_ionq import IonQProvider
    except ImportError:
        return {"error": "qiskit-ionq not installed",
                "install": "pip install qiskit-ionq",
                "backend": "ionq"}
    if not IONQ_KEY:
        return {"error": "IONQ_API_KEY env var not set", "backend": "ionq"}
    try:
        provider = IonQProvider(token=IONQ_KEY)
        backend  = provider.get_backend("ionq_simulator")
        qc, shots = build_circuit(descriptor)
        if shots_override:
            shots = shots_override
        job    = backend.run(qc, shots=shots)
        result = job.result()
        counts = result_to_counts(result)
        return {
            "counts":       counts,
            "total_shots":  sum(counts.values()),
            "most_likely":  max(counts, key=counts.get) if counts else "",
            "n_qubits":     int(descriptor[0]),
            "circuit_depth": qc.depth(),
            "backend":      "ionq",
            "fidelity":     None,
        }
    except Exception as e:
        return {"error": f"IonQ execution failed: {e}", "backend": "ionq"}


def result_to_counts(result) -> dict:
    """Extract counts dict from a Qiskit result object."""
    try:
        return dict(result.get_counts())
    except Exception:
        return {}

# ── 3-Shot Majority Vote (noise mitigation) ──────────────────────────────────

def majority_vote(descriptor: list, backend_name: str, shots_each: int = 512) -> dict:
    """Run 3 independent shots, combine by majority vote on most_likely state.
    Used for IBM hardware to reduce shot noise effects."""
    results = []
    for i in range(3):
        if backend_name == "aer":
            r = run_aer(descriptor, shots_override=shots_each)
        elif backend_name.startswith("ibm"):
            r = run_ibm(descriptor, backend_name=backend_name,
                        shots_override=shots_each)
        elif backend_name == "ionq":
            r = run_ionq(descriptor, shots_override=shots_each)
        else:
            r = run_aer(descriptor, shots_override=shots_each)
        results.append(r)

    # Combine counts from all 3 runs
    combined: dict = {}
    for r in results:
        if "error" in r:
            continue
        for state, count in (r.get("counts") or {}).items():
            combined[state] = combined.get(state, 0) + count

    if not combined:
        return results[0] if results else {"error": "all vote runs failed"}

    total = sum(combined.values())
    most_likely = max(combined, key=combined.get)

    # Pick the run whose most_likely matches the vote winner
    representative = next(
        (r for r in results if r.get("most_likely") == most_likely),
        results[0]
    )

    return {
        "counts":          combined,
        "total_shots":     total,
        "most_likely":     most_likely,
        "n_qubits":        representative.get("n_qubits", 0),
        "circuit_depth":   representative.get("circuit_depth", 0),
        "backend":         backend_name,
        "fidelity":        representative.get("fidelity"),
        "vote_detail":     [r.get("most_likely", "?") for r in results],
        "noise_mitigated": True,
    }

# ── Circuit Fidelity ─────────────────────────────────────────────────────────

def compute_fidelity(aer_counts: dict, hw_counts: dict) -> float:
    """Compute fidelity as 1 - Total Variation Distance between distributions.
    TVD = 0.5 * Σ |p_i - q_i|
    Fidelity = 1 - TVD  (1.0 = perfect match, 0.0 = completely different)
    """
    all_states = set(aer_counts.keys()) | set(hw_counts.keys())
    aer_total  = max(sum(aer_counts.values()), 1)
    hw_total   = max(sum(hw_counts.values()), 1)

    tvd = 0.0
    for state in all_states:
        p = aer_counts.get(state, 0) / aer_total
        q = hw_counts.get(state, 0) / hw_total
        tvd += abs(p - q)
    tvd *= 0.5

    return round(max(0.0, 1.0 - tvd), 4)


def measure_fidelity(descriptor: list, backend_name: str,
                     shots: int = 1024) -> dict:
    """Run circuit on Aer + target backend, compute fidelity."""
    aer_result = run_aer(descriptor, shots_override=shots)
    if "error" in aer_result:
        return {"error": f"Aer reference run failed: {aer_result['error']}"}

    if backend_name == "aer":
        return {**aer_result, "fidelity": 1.0, "fidelity_note": "aer vs aer = 1.0"}

    if backend_name.startswith("ibm"):
        hw_result = run_ibm(descriptor, backend_name=backend_name,
                            shots_override=shots)
    elif backend_name == "ionq":
        hw_result = run_ionq(descriptor, shots_override=shots)
    else:
        return {"error": f"Unknown backend: {backend_name}"}

    if "error" in hw_result:
        return {**hw_result,
                "fidelity": None,
                "fidelity_note": "hardware run failed — fidelity unavailable"}

    fidelity = compute_fidelity(
        aer_result.get("counts", {}),
        hw_result.get("counts", {})
    )
    hw_result["fidelity"] = fidelity
    hw_result["aer_counts"] = aer_result.get("counts", {})
    hw_result["fidelity_note"] = f"TVD-based: 1 - |Aer-{backend_name}|"
    return hw_result

# ── Dry run ──────────────────────────────────────────────────────────────────

def run_dry(descriptor: list) -> dict:
    """Describe circuit without executing."""
    n_q, n_c, shots, op_count = (int(descriptor[i]) for i in range(4))
    op_names = {0:"H",1:"X",2:"Y",3:"Z",4:"CX",5:"S",6:"T",7:"MEASURE"}
    ops = []
    for i in range(op_count):
        base = 4 + i * 3
        if base + 2 >= len(descriptor):
            break
        ot, tgt, ctrl = int(descriptor[base]), int(descriptor[base+1]), int(descriptor[base+2])
        nm = op_names.get(ot, f"OP{ot}")
        if ot == 4: ops.append(f"{nm}(ctrl={ctrl},tgt={tgt})")
        elif ot == 7: ops.append(f"{nm}(q{tgt}->c{tgt})")
        else: ops.append(f"{nm}(q{tgt})")
    return {
        "dry_run": True, "n_qubits": n_q, "n_cbits": n_c,
        "shots": shots, "ops": ops, "circuit_str": " → ".join(ops)
    }

# ── Backend status ────────────────────────────────────────────────────────────

def backend_status() -> dict:
    """Report which backends are currently available."""
    status = {
        "aer":     {"available": QISKIT_AVAILABLE, "type": "local_simulator"},
        "ibm":     {"available": IBM_RUNTIME_AVAILABLE and bool(IBM_TOKEN),
                    "token_set": bool(IBM_TOKEN),
                    "runtime_installed": IBM_RUNTIME_AVAILABLE,
                    "type": "real_hardware_127q"},
        "ionq":    {"available": bool(IONQ_KEY), "key_set": bool(IONQ_KEY),
                    "type": "ion_trap_hardware"},
    }
    status["recommended"] = (
        "ibm_brisbane" if (IBM_RUNTIME_AVAILABLE and IBM_TOKEN)
        else "aer"
    )
    return status

# ── Main ─────────────────────────────────────────────────────────────────────

def main():
    parser = argparse.ArgumentParser(
        description="Aeonmi Quantum Bridge — Aer + IBM + IonQ",
        add_help=True
    )
    parser.add_argument("descriptor", nargs="*",
                        help="Circuit descriptor tokens")
    parser.add_argument("--backend", default="aer",
                        choices=["aer","ibm_brisbane","ibm_kyoto",
                                 "ibm_sherbrooke","ionq","dry"],
                        help="Target backend (default: aer)")
    parser.add_argument("--shots",   type=int, default=None,
                        help="Override shot count")
    parser.add_argument("--vote",    action="store_true",
                        help="3-shot majority vote for noise mitigation")
    parser.add_argument("--fidelity", action="store_true",
                        help="Measure fidelity vs Aer (for IBM/IonQ)")
    parser.add_argument("--status",  action="store_true",
                        help="Print backend availability and exit")
    parser.add_argument("--dry",     action="store_true",
                        help="Dry run: describe circuit only")

    args, extra = parser.parse_known_args()

    if args.status:
        print(json.dumps(backend_status(), indent=2))
        return

    # Gather descriptor tokens
    raw = list(args.descriptor) + extra
    if not raw:
        raw = sys.stdin.read().strip().split()
    if not raw:
        print(json.dumps({"error": "no descriptor provided"}))
        sys.exit(1)

    try:
        descriptor = [float(x) for x in raw if not x.startswith("--")]
    except ValueError as e:
        print(json.dumps({"error": f"invalid descriptor: {e}"}))
        sys.exit(1)

    if len(descriptor) < 4:
        print(json.dumps({"error": "need at least 4 elements: n_q n_c shots op_count"}))
        sys.exit(1)

    backend = args.backend

    # Dry run
    if args.dry or backend == "dry":
        print(json.dumps(run_dry(descriptor), indent=2))
        return

    # Fidelity measurement
    if args.fidelity:
        result = measure_fidelity(descriptor, backend, shots=args.shots or 1024)
        print(json.dumps(result, indent=2))
        return

    # 3-shot majority vote
    if args.vote:
        result = majority_vote(descriptor, backend,
                               shots_each=args.shots or 512)
        print(json.dumps(result, indent=2))
        return

    # Single run
    if backend == "aer":
        result = run_aer(descriptor, shots_override=args.shots)
    elif backend.startswith("ibm"):
        result = run_ibm(descriptor, backend_name=backend,
                         shots_override=args.shots)
    elif backend == "ionq":
        result = run_ionq(descriptor, shots_override=args.shots)
    else:
        result = run_aer(descriptor, shots_override=args.shots)

    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
