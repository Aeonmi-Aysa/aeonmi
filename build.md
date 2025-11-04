# Aeonmi Shard Build & Operations Guide

This repository contains a fully configured Aeonmi shard application.  The project behaves like a Rust crate while using `.ai` files as its primary source language.  The tooling mirrors Cargo so you can build, test, check, and run the project using either `cargo` or the dedicated `aeonmi` command line interface.

## Project Layout

```
Aeonmi.toml           # Aeonmi project manifest
src/
  main.ai            # Default entry point (fn main)
  engine.ai          # Shared runtime helpers
  logic.ai           # Additional functions and inline tests
tests/
  arithmetic.ai      # Project level tests automatically discovered
```

All project metadata lives in `Aeonmi.toml`.  The `[aeonmi]` table defines the entry point, modules, and optional test listings.  Modules listed in the manifest are merged during builds so their functions (`fn banner`, `fn describe_logic`, etc.) become available to `src/main.ai`.

## Requirements

* Rust toolchain with Cargo
* No JavaScript runtime is required.  Execution happens through the Aeonmi VM embedded in the Rust binary.

## Building the Project

### Development Build

```bash
cargo build
```

The build produces the Aeonmi CLI binary and generates an Aeonmi bundle at `target/debug/aeonmi-shard.bundle.json`.

You can also use the Aeonmi CLI directly:

```bash
cargo run --bin aeonmi -- build
```

Or the convenience script:

```bash
./scripts/build.sh
```

### Release Build

```bash
cargo build --release
cargo run --bin aeonmi -- build --release
./scripts/build.sh --release
```

Release mode creates `target/release/aeonmi-shard.bundle.json`.

## Checking the Project

Syntax and semantic validation without producing artifacts:

```bash
cargo check
cargo run --bin aeonmi -- check
./scripts/check.sh
```

## Running Tests

Tests run through the Aeonmi VM and cover inline `test` blocks plus files in `tests/`.

```bash
cargo test
cargo run --bin aeonmi -- test
./scripts/test.sh
```

Use `--release` or `--filter <pattern>` to refine execution:

```bash
cargo run --bin aeonmi -- test --filter addition
```

## Running the Application

Run the compiled shard directly:

```bash
cargo run -- run
```

The CLI loads `Aeonmi.toml`, builds the shard if necessary, and executes `fn main` from `src/main.ai`.

### Running with Scripts

For convenience you can wrap execution with Cargo:

```bash
cargo run -- run --release
```

Or use the CLI without rebuilding the binary:

```bash
cargo run --bin aeonmi -- run
```

## Scripts Summary

| Script | Purpose |
| --- | --- |
| `./scripts/build.sh` | Build the Aeonmi project (debug by default) |
| `./scripts/check.sh` | Validate sources without emitting bundles |
| `./scripts/test.sh`  | Execute Aeonmi unit/integration tests |

All scripts accept the same flags exposed by the `aeonmi` CLI (e.g., `--release`).

## Generated Artifacts

Successful builds create bundle files under `target/<profile>/`.  These JSON bundles list the package metadata and compiled module index and serve as the deployable output for Aeonmi runtimes.