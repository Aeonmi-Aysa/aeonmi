#!/usr/bin/env python3
import sys
print("Python:", sys.executable)
print("Version:", sys.version)

try:
    from qiskit import QuantumCircuit, QuantumRegister, ClassicalRegister
    print("qiskit import: OK")
except ImportError as e:
    print("qiskit import FAILED:", e)
    sys.exit(1)

try:
    from qiskit_aer import AerSimulator
    print("qiskit_aer import: OK")
except ImportError as e:
    print("qiskit_aer import FAILED:", e)
    sys.exit(1)

# Quick Bell state simulation
qr = QuantumRegister(2, 'q')
cr = ClassicalRegister(2, 'c')
qc = QuantumCircuit(qr, cr)
qc.h(qr[0])
qc.cx(qr[0], qr[1])
qc.measure(qr[0], cr[0])
qc.measure(qr[1], cr[1])

sim = AerSimulator()
job = sim.run(qc, shots=1024)
result = job.result()
counts = result.get_counts()
print("Bell counts:", counts)
print("QISKIT OK")
