#!/usr/bin/env python3
import sys
import json

try:
    from qiskit import Aer, QuantumCircuit, execute
except ImportError:
    Aer = None  # type: ignore
    from qiskit import BasicAer, QuantumCircuit, execute  # type: ignore
else:
    try:
        from qiskit import BasicAer  # type: ignore
    except ImportError:
        BasicAer = None  # type: ignore

if len(sys.argv) < 2:
    print("Usage: qiskit_runner.py <qasm_file> [shots]", file=sys.stderr)
    sys.exit(1)

qasm_file = sys.argv[1]
shots = 1024
if len(sys.argv) >= 3:
    try:
        shots = int(sys.argv[2])
    except ValueError:
        print(f"Invalid shots value: {sys.argv[2]}", file=sys.stderr)
        sys.exit(1)

try:
    qc = QuantumCircuit.from_qasm_file(qasm_file)
except Exception as exc:
    print(f"Error loading QASM file: {exc}", file=sys.stderr)
    sys.exit(2)

def resolve_backend():
    if Aer is not None:
        try:
            return Aer.get_backend('qasm_simulator')
        except Exception:
            if BasicAer is not None:
                return BasicAer.get_backend('qasm_simulator')
            raise
    if BasicAer is not None:
        return BasicAer.get_backend('qasm_simulator')
    raise RuntimeError('No Qiskit simulator backend available')

try:
    backend = resolve_backend()
except Exception as exc:
    print(f"Backend resolution error: {exc}", file=sys.stderr)
    sys.exit(2)

try:
    job = execute(qc, backend=backend, shots=shots)
    result = job.result()
except Exception as exc:
    print(f"Execution error: {exc}", file=sys.stderr)
    sys.exit(3)

try:
    counts = result.get_counts(qc)
except Exception:
    counts = {}

print(json.dumps(counts))
