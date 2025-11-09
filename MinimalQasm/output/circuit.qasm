OPENQASM 2.0;
include "qelib1.inc";
qreg q[1];
creg c[1];

// Qubit variable mapping:
// q -> q[0]

// Function: main
// Line 4: variable 'q' (classical - not representable in QASM)
h q[0];  // superpose q
x q[0];  // dod q
measure q[0] -> c[0];  // measure q

