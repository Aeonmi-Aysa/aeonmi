# Hello Quantum World

Your first quantum program in Aeonmi! This tutorial introduces you to basic quantum programming concepts through simple, hands-on examples.

## 📚 Learning Objectives

By the end of this tutorial, you will:
- Understand what a qubit is and how it differs from classical bits
- Create your first quantum superposition
- Measure quantum states and understand probabilistic outcomes
- Write and run your first Aeonmi quantum program

## 🎯 Prerequisites

- Basic Rust syntax (variables, functions, println!)
- Aeonmi installed on your system
- Curiosity about quantum computing!

## 💡 Concepts Covered

- **Qubits**: The quantum analog of classical bits
- **Superposition**: Being in multiple states simultaneously  
- **Measurement**: Observing quantum states (which changes them!)
- **Quantum functions**: Special functions that work with quantum data

## 🔧 Implementation

### Step 1: Create a New Project

```bash
# Create a new Aeonmi project
aeon new hello-quantum
cd hello-quantum

# Verify the project structure
ls
```

You should see:
```
Aeonmi.toml     # Project configuration
src/
  main.aeon     # Your main program file
```

### Step 2: Your First Quantum Program

Open `src/main.aeon` and replace its contents with:

```rust
// Hello Quantum World - Your first quantum program!

quantum fn main() {
    println!("🌟 Welcome to Quantum Computing with Aeonmi!");
    
    // Create your first qubit in the |0⟩ state
    let my_qubit = qubit(0);
    println!("Created a qubit in state |0⟩");
    
    // Measure it (should always be false/0)
    let result = measure(my_qubit);
    println!("Measured value: {} (probability: {:.1}%)", 
             result.value, result.probability * 100.0);
}
```

### Step 3: Run Your Program

```bash
aeon run
```

You should see output like:
```
🌟 Welcome to Quantum Computing with Aeonmi!
Created a qubit in state |0⟩
Measured value: false (probability: 100.0%)
```

**What happened?** 
- You created a qubit in the definite state |0⟩
- When measured, it always gives the result `false` (representing 0)
- The probability is 100% because there's no uncertainty

### Step 4: Adding Quantum Superposition

Now let's create a true quantum superposition! Modify your program:

```rust
quantum fn main() {
    println!("🌟 Hello Quantum Superposition!");
    
    // Create a qubit and put it in superposition
    let quantum_coin = qubit(0);
    println!("Created qubit in |0⟩");
    
    // Apply Hadamard gate to create superposition
    hadamard(quantum_coin);
    println!("Applied Hadamard gate - now in superposition!");
    println!("State is now: (|0⟩ + |1⟩)/√2");
    
    // Measure the superposition
    let result = measure(quantum_coin);
    println!("Quantum coin flip result: {} ({})", 
             if result.value { "Heads" } else { "Tails" },
             if result.value { "1" } else { "0" });
    
    println!("This was truly random - not pseudo-random!");
}
```

### Step 5: Run and Observe

```bash
aeon run
```

Run it several times to see different outcomes:
```bash
aeon run  # Might show "Heads"
aeon run  # Might show "Tails"  
aeon run  # Might show "Heads"
aeon run  # Might show "Tails"
```

**What's different now?**
- The Hadamard gate puts the qubit in superposition
- Each measurement gives a truly random result
- It's 50% heads, 50% tails - a perfect quantum coin flip!

### Step 6: Multiple Qubits and Correlations

Let's explore quantum entanglement with multiple qubits:

```rust
quantum fn main() {
    println!("🌟 Quantum Entanglement Demo");
    
    // Create two qubits
    let alice = qubit(0);
    let bob = qubit(0);
    
    println!("Created two qubits for Alice and Bob");
    
    // Create entanglement (Bell state)
    hadamard(alice);           // Put Alice's qubit in superposition
    cnot(alice, bob);          // Entangle with Bob's qubit
    
    println!("Entangled the qubits!");
    println!("They are now in the Bell state: (|00⟩ + |11⟩)/√2");
    
    // Measure both qubits
    let alice_result = measure(alice);
    let bob_result = measure(bob);
    
    println!("Alice measured: {}", alice_result.value);
    println!("Bob measured:   {}", bob_result.value);
    
    // Check if they're the same (they always should be!)
    if alice_result.value == bob_result.value {
        println!("✅ Results are correlated - quantum entanglement confirmed!");
    } else {
        println!("❌ Something went wrong - they should always match!");
    }
}
```

### Step 7: Understanding the Results

Run this program multiple times:

```bash
aeon run
aeon run
aeon run
```

You'll notice:
- Sometimes both Alice and Bob measure `false` (both 0)
- Sometimes both measure `true` (both 1) 
- They **never** measure different values
- This correlation happens instantly, no matter how far apart they are!

## 🧪 Experiments

Try these modifications to deepen your understanding:

### Experiment 1: Classical vs Quantum Randomness

Create a program that compares classical and quantum randomness:

```rust
quantum fn main() {
    println!("🎲 Classical vs Quantum Randomness");
    
    // Classical "randomness" (pseudo-random)
    let classical_random = std::random() % 2 == 0;
    println!("Classical result: {}", classical_random);
    
    // Quantum randomness (truly random)
    let q = qubit(0);
    hadamard(q);
    let quantum_random = measure(q).value;
    println!("Quantum result: {}", quantum_random);
    
    println!("The quantum result is fundamentally unpredictable!");
}
```

### Experiment 2: Multiple Measurements

What happens if you try to measure the same qubit twice?

```rust
quantum fn main() {
    println!("🔍 What happens with multiple measurements?");
    
    let q = qubit(0);
    hadamard(q);  // Superposition
    
    println!("Qubit is in superposition...");
    
    let first_measurement = measure(q);
    println!("First measurement: {}", first_measurement.value);
    
    // What will the second measurement show?
    let second_measurement = measure(q);
    println!("Second measurement: {}", second_measurement.value);
    
    println!("Notice: once measured, the state is fixed!");
}
```

**Key insight**: Once you measure a quantum state, it "collapses" to a definite value. Subsequent measurements will always give the same result.

### Experiment 3: Different Initial States

Try starting with different qubit states:

```rust
quantum fn main() {
    println!("🎯 Different Starting States");
    
    // Start with |0⟩
    let q0 = qubit(0);
    hadamard(q0);
    println!("|0⟩ + Hadamard → {}", measure(q0).value);
    
    // Start with |1⟩  
    let q1 = qubit(1);
    hadamard(q1);
    println!("|1⟩ + Hadamard → {}", measure(q1).value);
    
    println!("Both create superpositions, but with different phases!");
}
```

## 🧠 Understanding What You've Learned

### Qubits vs Bits

| Classical Bit | Quantum Qubit |
|---------------|---------------|
| Either 0 or 1 | Can be 0, 1, or both simultaneously |
| Deterministic | Probabilistic when measured |
| Copying is easy | Cannot be perfectly copied |
| Independent | Can be entangled with others |

### Key Quantum Phenomena

1. **Superposition**: Qubits can exist in multiple states at once
2. **Measurement**: Observing a quantum state changes it irreversibly  
3. **Entanglement**: Qubits can be correlated in impossible classical ways
4. **Randomness**: Quantum measurements are fundamentally unpredictable

### Aeonmi Programming Patterns

```rust
// Pattern 1: Create and measure
let q = qubit(0);          // Create qubit
let result = measure(q);   // Measure it

// Pattern 2: Apply gate then measure  
let q = qubit(0);          // Create qubit
hadamard(q);               // Apply quantum gate
let result = measure(q);   // Measure result

// Pattern 3: Multi-qubit operations
let q1 = qubit(0);         // First qubit
let q2 = qubit(0);         // Second qubit  
cnot(q1, q2);              // Two-qubit gate
// Measure both...
```

## 🚀 Next Steps

Congratulations! You've written your first quantum programs and experienced the weirdness of quantum mechanics firsthand. 

### What to explore next:

1. **[Basic Qubit Operations](basic-operations.md)** - Learn about more quantum gates
2. **[Quantum Superposition](superposition.md)** - Dive deeper into superposition
3. **[Quantum Entanglement](entanglement.md)** - Explore spooky action at a distance

### Questions to ponder:

- How is quantum randomness different from flipping a coin?
- Why can't we copy qubits like we copy classical bits?  
- How could quantum entanglement be useful for communication?
- What would happen if we could put classical computers in superposition?

### Try building:

- A quantum dice roller (3 qubits = 8 possible outcomes)
- A quantum magic 8-ball that gives truly random advice
- An entanglement-based communication protocol

## 🎉 Celebration

You're now a quantum programmer! You've:
- ✅ Created qubits and manipulated quantum states
- ✅ Observed quantum superposition and randomness
- ✅ Witnessed quantum entanglement 
- ✅ Written working Aeonmi code

The quantum world is strange, but you're starting to speak its language. Keep experimenting and don't worry if things seem counterintuitive - that's just quantum mechanics being quantum mechanics!

Welcome to the quantum future! 🚀⚛️