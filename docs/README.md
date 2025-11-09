# Aeonmi Documentation v1.0

Welcome to the official documentation for **Aeonmi** - the quantum-classical hybrid programming language and development platform.

![Aeonmi Logo](https://via.placeholder.com/400x100/4A90E2/FFFFFF?text=Aeonmi+v1.0)

## 📚 Documentation Overview

Aeonmi is designed to make quantum programming accessible to developers and students while providing enterprise-grade tooling for quantum-classical hybrid applications. This documentation will guide you through every aspect of the platform.

### 🎯 What is Aeonmi?

Aeonmi is a revolutionary programming language that seamlessly blends classical and quantum computing paradigms. Unlike traditional quantum programming frameworks, Aeonmi provides:

- **Native quantum syntax** with intuitive qubit operations
- **Integrated classical programming** for full application development
- **Educational transparency** showing compiler internals
- **Enterprise-ready toolchain** with CLI and web-based editor
- **Sandboxed execution** for safe learning environments

## 📖 Documentation Sections

### 🔤 [Language Reference](language/)
Complete syntax guide, keywords, and language constructs
- [Syntax Overview](language/syntax.md)
- [Data Types & Variables](language/types.md)
- [Functions & Classes](language/functions.md)
- [Quantum Operations](language/quantum.md)
- [Control Flow](language/control-flow.md)
- [Modules & Imports](language/modules.md)

### 🔧 [Toolchain Guide](toolchain/)
Learn to use the Aeonmi development tools
- [Installation](toolchain/installation.md)
- [CLI Commands (`aeon`)](toolchain/cli.md)
- [Shard Editor](toolchain/editor.md)
- [Project Structure](toolchain/projects.md)
- [Build System](toolchain/building.md)
- [Testing Framework](toolchain/testing.md)

### ⚛️ [Quantum Programming](quantum/)
Master quantum programming with Aeonmi
- [Quantum Basics](quantum/basics.md)
- [Qubit Management](quantum/qubits.md)
- [Quantum Gates](quantum/gates.md)
- [Quantum Circuits](quantum/circuits.md)
- [Measurement & Results](quantum/measurement.md)
- [Quantum Algorithms](quantum/algorithms.md)

### 🎓 [Tutorials & Examples](tutorials/)
Step-by-step learning with hands-on projects
- [Getting Started](tutorials/getting-started.md)
- [Hello World](tutorials/hello-world.md)
- [Your First Quantum Program](tutorials/first-quantum.md)
- [Quantum Teleportation](tutorials/teleportation.md)
- [Grover's Algorithm](tutorials/grovers.md)
- [Hybrid Applications](tutorials/hybrid-apps.md)

### 💡 [Example Projects](examples/)
Real-world Aeonmi applications and templates
- [Quantum Random Numbers](examples/quantum-random.md)
- [Quantum Key Distribution](examples/qkd.md)
- [Machine Learning Integration](examples/ml-quantum.md)
- [Educational Demos](examples/educational.md)

### 🤝 [Contributing](contributing/)
Help improve Aeonmi for everyone
- [Development Setup](contributing/setup.md)
- [Architecture Overview](contributing/architecture.md)
- [Contributing Guide](contributing/guide.md)
- [Testing Guidelines](contributing/testing.md)

## 🚀 Quick Start

### Installation
```bash
# Download and install Aeonmi
curl -sSL https://aeonmi.dev/install | bash

# Or build from source
git clone https://github.com/DarthMetaCrypro/Aeonmi.git
cd Aeonmi
cargo install --path .
```

### Your First Program
```rust
// hello_quantum.aeon
use quantum;

fn main() {
    println!("Hello, Quantum World!");
    
    // Create a qubit in superposition
    let q = qubit(0);
    hadamard(q);
    
    // Measure and display result
    let result = measure(q);
    println!("Quantum measurement: {}", result);
}
```

```bash
# Compile and run
aeon build hello_quantum.aeon
aeon run hello_quantum.aeon
```

### Open in Shard Editor
```bash
# Launch the integrated web editor
aeon editor

# Or open a specific project
aeon editor my_project/
```

## 🎯 Learning Paths

### For Students
1. Start with [Getting Started Tutorial](tutorials/getting-started.md)
2. Learn [Quantum Basics](quantum/basics.md)
3. Try [Example Projects](examples/)
4. Use Learning Mode: `aeon run --verbose --explain`

### For Developers
1. Review [Language Reference](language/)
2. Master the [CLI Toolchain](toolchain/)
3. Explore [Advanced Examples](examples/)
4. Contribute to [Open Source](contributing/)

### For Educators
1. Use [Educational Examples](examples/educational.md)
2. Set up [Sandboxed Environments](toolchain/sandbox.md)
3. Enable [Learning Mode](toolchain/learning-mode.md)
4. Access [Curriculum Resources](tutorials/curriculum.md)

## 🔍 Key Features

### 🌟 Unique Advantages
- **Quantum-Native**: First-class quantum operations and circuit design
- **Educational**: Transparent compilation with learning mode
- **Integrated**: Unified CLI and web editor experience
- **Safe**: Sandboxed execution for secure learning
- **Modern**: Built with Rust for performance and safety

### 🎯 Use Cases
- **Education**: Teaching quantum computing concepts
- **Research**: Prototyping quantum algorithms
- **Enterprise**: Building quantum-classical hybrid applications
- **Experimentation**: Exploring quantum programming patterns

## 📊 Version Information

- **Current Version**: v1.0.0
- **Rust Version**: 1.70+
- **Quantum Backends**: Qiskit, Local Simulator
- **Platform Support**: Windows, macOS, Linux

## 🆘 Getting Help

- **Documentation**: You're reading it! 📚
- **GitHub Issues**: [Report bugs or request features](https://github.com/DarthMetaCrypro/Aeonmi/issues)
- **Community Forum**: [Join discussions](https://github.com/DarthMetaCrypro/Aeonmi/discussions)
- **Learning Mode**: Use `aeon explain` for detailed compiler insights

## 🎉 Welcome to Quantum Programming!

Aeonmi makes quantum programming accessible without sacrificing power. Whether you're a student learning quantum concepts or an enterprise developer building the next generation of applications, Aeonmi provides the tools you need.

Ready to start? Head to [Getting Started](tutorials/getting-started.md) and build your first quantum program in minutes!

---

*Made with ⚛️ by the Aeonmi Team*