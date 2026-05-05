# Getting Started

## Prerequisites

- Rust toolchain (for local build)
- Python 3 (for dashboard)

## Build

```bash
cargo build --release
```

## Run `.ai` with native VM

```bash
./target/release/aeonmi run examples/hello.ai
./target/release/aeonmi native examples/hello.ai
./target/release/aeonmi exec examples/hello.ai
```

## Run QUBE

```bash
./target/release/aeonmi qube check examples/demo.qube
./target/release/aeonmi qube run examples/demo.qube --diagram
```

## Mother AI

```bash
./target/release/aeonmi mother
./target/release/aeonmi mother --file script.ai --verbose
```

## Dashboard

```bash
python Aeonmi_Master/dashboard.py
```

Open: `http://localhost:7777`

## Basic validation

```bash
cargo test --all --quiet
```

