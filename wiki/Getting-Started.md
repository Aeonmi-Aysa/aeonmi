# Getting Started

## Prerequisites

- Rust toolchain (stable)
- Git
- Optional: Python 3 (for dashboard tooling)

## Build

```bash
cargo build --release
```

## Run an Aeonmi program

```bash
./target/release/Aeonmi native examples/hello.ai
```

## Common command patterns

```bash
# Emit canonical .ai output
aeonmi emit demo.ai -o output.ai

# Build a source file through the pipeline
aeonmi build src/main.ai --out output.ai

# Run QUBE file
aeonmi qube run circuit.qube --diagram

# Start Mother AI loop
aeonmi mother
```

## Optional dashboard

```bash
python Aeonmi_Master/dashboard.py
```

Then open `http://localhost:7777`.

