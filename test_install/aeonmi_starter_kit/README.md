# Aeonmi Quantum Starter Kit

**Build your first quantum circuit in 10 minutes.**

---

## What You Get

This starter kit includes:
- `hello_quantum.ai` — Your first Bell state (entangled qubits)
- `grover_search.ai` — Quantum database search with √N speedup
- `qft_pattern.ai` — Quantum Fourier Transform demonstration
- `entanglement_demo.ai` — 3-qubit GHZ state (tripartite entanglement)
- `run.bat` — One-click launcher (Windows)

---

## Prerequisites

1. **Python 3.8+** with pip
2. **Qiskit** quantum framework:
   ```bash
   pip install qiskit qiskit-aer
   ```
3. **Aeonmi compiler** (included in parent directory)

---

## Quick Start

### Windows:
```cmd
run.bat
```

### Linux/Mac:
```bash
../aeonmi run hello_quantum.ai
```

---

## What Just Happened?

When you ran `hello_quantum.ai`, Aeonmi:

1. **Compiled** your `.ai` source to bytecode
2. **Generated** a quantum circuit descriptor
3. **Executed** the circuit on Qiskit's Aer simulator
4. **Returned** measurement results to your program

The circuit created a **Bell state** — two qubits in perfect quantum entanglement. When you measure one, you instantly know the other, regardless of distance.

---

## Understanding the Code

```aeonmi
import quantum;

function main() {
    ⍝ Circuit descriptor: "n_qubits n_classical shots n_ops [ops...