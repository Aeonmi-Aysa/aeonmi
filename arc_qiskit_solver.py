"""
ARC Challenge Quantum Solver using Qiskit
Task: Fill hollow rectangles with interior color
Challenge: aeonmi_arc_test_001
"""

from qiskit import QuantumCircuit, QuantumRegister, ClassicalRegister
from qiskit_aer import AerSimulator
import numpy as np
import json

def load_arc_challenge(filepath):
    with open(filepath, 'r') as f:
        return json.load(f)

def find_rectangle_border(grid):
    """Find rectangle defined by color 2 border"""
    rows = len(grid)
    cols = len(grid[0