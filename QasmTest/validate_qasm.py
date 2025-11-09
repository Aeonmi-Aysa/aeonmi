"""
Validate QASM output from Aeonmi export using Qiskit.
This script tests that the generated QASM file is syntactically correct
and can be parsed by Qiskit.
"""
import sys

try:
    from qiskit import QuantumCircuit
    from qiskit.qasm2 import load
    
    print("=== Qiskit QASM Validation ===")
    
    qasm_file = r"c:\Users\wlwil\Downloads\Aeonmi-SPACEJEDI\Aeonmi-SPACEJEDI\QasmTest\output\circuit.qasm"
    
    print(f"Loading QASM file: {qasm_file}")
    
    # Load and parse the QASM file
    circuit = load(qasm_file)
    
    print(f"\n✓ QASM file parsed successfully!")
    print(f"✓ Circuit has {circuit.num_qubits} qubits")
    print(f"✓ Circuit has {circuit.num_clbits} classical bits")
    print(f"✓ Circuit has {len(circuit.data)} gates/operations")
    
    print("\nCircuit operations:")
    for i, (gate, qubits, clbits) in enumerate(circuit.data, 1):
        qubit_str = ", ".join(f"q[{circuit.qubits.index(q)}]" for q in qubits)
        if clbits:
            clbit_str = ", ".join(f"c[{circuit.clbits.index(c)}]" for c in clbits)
            print(f"  {i}. {gate.name:10} {qubit_str:15} -> {clbit_str}")
        else:
            print(f"  {i}. {gate.name:10} {qubit_str}")
    
    print("\n" + "="*50)
    print("✓ VALIDATION PASSED: QASM is valid and parseable by Qiskit!")
    print("="*50)
    
    sys.exit(0)
    
except ImportError as e:
    print(f"ERROR: Qiskit not installed - {e}")
    print("Install with: pip install qiskit")
    print("\nAttempting basic syntax check without Qiskit...")
    
    # Basic syntax validation without Qiskit
    try:
        qasm_file = r"c:\Users\wlwil\Downloads\Aeonmi-SPACEJEDI\Aeonmi-SPACEJEDI\QasmTest\output\circuit.qasm"
        with open(qasm_file, 'r') as f:
            content = f.read()
        
        print("\nQASM File Content:")
        print("="*50)
        print(content)
        print("="*50)
        
        # Basic checks
        required = ["OPENQASM 2.0", "include", "qreg", "creg"]
        for req in required:
            if req in content:
                print(f"✓ Contains '{req}'")
            else:
                print(f"✗ Missing '{req}'")
                sys.exit(1)
        
        # Count gates
        gates = ["h ", "cx ", "x ", "measure"]
        for gate in gates:
            count = content.count(gate)
            if count > 0:
                print(f"✓ Found {count} {gate.strip()} gate(s)")
        
        print("\n✓ BASIC VALIDATION PASSED")
        print("(Full Qiskit validation unavailable - install Qiskit for complete verification)")
        sys.exit(0)
        
    except Exception as e2:
        print(f"✗ Basic validation failed: {e2}")
        sys.exit(1)
    
except Exception as e:
    print(f"\n✗ VALIDATION FAILED: {e}")
    import traceback
    traceback.print_exc()
    sys.exit(1)
