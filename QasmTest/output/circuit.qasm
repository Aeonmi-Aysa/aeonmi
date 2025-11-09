OPENQASM 2.0;
include "qelib1.inc";
qreg q[2];
creg c[2];

// Qubit variable mapping:
// q0 -> q[0]
// q1 -> q[1]

// Function: main
// Line 4: print statement (not representable in QASM)
// Line 5: variable 'q0' (classical - not representable in QASM)
// Line 6: variable 'q1' (classical - not representable in QASM)
// Line 7: print statement (not representable in QASM)
h q[0];  // superpose q0
// Line 9: print statement (not representable in QASM)
h q[0];  // entangle q0 and q1 (step 1: H on control)
cx q[0], q[1];  // entangle q0 and q1 (step 2: CNOT)
// Line 11: print statement (not representable in QASM)
x q[1];  // dod q1
// Line 13: print statement (not representable in QASM)
measure q[0] -> c[0];  // measure q0
measure q[1] -> c[1];  // measure q1
// Line 16: print statement (not representable in QASM)
// Line 17: print statement (not representable in QASM)

