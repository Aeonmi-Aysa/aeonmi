#!/usr/bin/env python3
"""
qiskit_runner.py — Aeonmi Qiskit Bridge
========================================
Executes an Aeonmi flat circuit descriptor on Qiskit Aer simulator
or real IBM Quantum hardware.

Aeonmi circuit descriptor format (from qiskit.ai qk_new/qk_add_gate):
  [n_qubits, n_cbits, shots, op_count, op_type0, op_tgt0, op_ctrl0, ...]

Op types:
  0 = H (Hadamard)
  1 = X (Pauli-X / NOT)
  2 = Y (Pauli-Y)
  3 = Z (Pauli-Z)
  4 = CX (CNOT, ctrl=op_ctrl, tgt=op_tgt)
  5 = S (phase)
  6 = T (T gate)
  7 = MEASURE (tgt qubit -> tgt classical bit)

Usage:
  python qiskit_runner.py <n_q> <n_c> <shots> <op_count> [op_type tgt ctrl ...]

  Or pipe a space-separated descriptor:
  echo "2 2 1024 3 0 0 -1 4 1 0 7 0 0 7 1 1" | python qiskit_runner.py

Output (JSON to stdout):
  {"counts": {"00": 512, "11": 512}, "total_shots": 1024, "most_likely": "11"}

Example — Bell state (H q0, CX q0->q1, measure both):
  python qiskit_runner.py 2 2 1024 3 0 0 -1 4 1 0 7 0 0 7 1 1
"""

import sys
import json

try:
    from qiskit import QuantumCircuit, QuantumRegister, ClassicalRegister
    from qiskit_aer import AerSimulator
    QISKIT_AVAILABLE = True
except ImportError:
    QISKIT_AVAILABLE = False


def build_circuit(descriptor):
    """Build a Qiskit circuit from the Aeonmi flat descriptor."""
    n_q = int(descriptor[0])
    n_c = int(descriptor[1])
    shots = int(descriptor[2])
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

        if op_type == 0:    # H
            qc.h(qr[op_tgt])
        elif op_type == 1:  # X
            qc.x(qr[op_tgt])
        elif op_type == 2:  # Y
            qc.y(qr[op_tgt])
        elif op_type == 3:  # Z
            qc.z(qr[op_tgt])
        elif op_type == 4:  # CX
            qc.cx(qr[op_ctrl], qr[op_tgt])
        elif op_type == 5:  # S
            qc.s(qr[op_tgt])
        elif op_type == 6:  # T
            qc.t(qr[op_tgt])
        elif op_type == 7:  # MEASURE
            if op_tgt < n_c:
                qc.measure(qr[op_tgt], cr[op_tgt])

    return qc, shots


def run_circuit(descriptor):
    """Run circuit and return counts dict + metadata."""
    if not QISKIT_AVAILABLE:
        return {
            "error": "qiskit or qiskit_aer not installed",
            "install": "pip install qiskit qiskit-aer",
            "descriptor": descriptor
        }

    qc, shots = build_circuit(descriptor)
    simulator = AerSimulator()
    job = simulator.run(qc, shots=shots)
    result = job.result()
    counts = result.get_counts()

    total = sum(counts.values())
    most_likely = max(counts, key=counts.get) if counts else ""

    return {
        "counts": counts,
        "total_shots": total,
        "most_likely": most_likely,
        "n_qubits": int(descriptor[0]),
        "circuit_depth": qc.depth()
    }


def run_dry(descriptor):
    """Dry run: describe circuit without executing (no Qiskit needed)."""
    n_q = int(descriptor[0])
    n_c = int(descriptor[1])
    shots = int(descriptor[2])
    op_count = int(descriptor[3])

    op_names = {0: "H", 1: "X", 2: "Y", 3: "Z", 4: "CX", 5: "S", 6: "T", 7: "MEASURE"}
    ops = []
    for i in range(op_count):
        base = 4 + i * 3
        if base + 2 >= len(descriptor):
            break
        op_type = int(descriptor[base])
        op_tgt  = int(descriptor[base + 1])
        op_ctrl = int(descriptor[base + 2])
        name = op_names.get(op_type, f"OP{op_type}")
        if op_type == 4:
            ops.append(f"{name}(ctrl={op_ctrl},tgt={op_tgt})")
        elif op_type == 7:
            ops.append(f"{name}(q{op_tgt}->c{op_tgt})")
        else:
            ops.append(f"{name}(q{op_tgt})")

    return {
        "dry_run": True,
        "n_qubits": n_q,
        "n_cbits": n_c,
        "shots": shots,
        "ops": ops,
        "circuit_str": " → ".join(ops)
    }


def main():
    # Parse descriptor from args or stdin
    if len(sys.argv) > 1:
        raw = sys.argv[1:]
    else:
        raw = sys.stdin.read().strip().split()

    if not raw:
        print(json.dumps({"error": "no descriptor provided"}))
        sys.exit(1)

    try:
        descriptor = [float(x) for x in raw]
    except ValueError as e:
        print(json.dumps({"error": f"invalid descriptor: {e}"}))
        sys.exit(1)

    if len(descriptor) < 4:
        print(json.dumps({"error": "descriptor must have at least 4 elements: n_q n_c shots op_count"}))
        sys.exit(1)

    # Dry run mode if --dry flag or no Qiskit
    if "--dry" in sys.argv or not QISKIT_AVAILABLE:
        result = run_dry(descriptor)
    else:
        result = run_circuit(descriptor)

    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
