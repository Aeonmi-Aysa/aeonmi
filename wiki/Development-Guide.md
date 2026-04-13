# Development Guide

## Local workflow

1. Build
   ```bash
   cargo build --release
   ```
2. Run tests
   ```bash
   cargo test --all --quiet
   ```
3. Run language-level stdlib tests
   ```bash
   ./target/release/Aeonmi native aeonmi_ai/stdlib/tests/test_math_builtins.ai
   ./target/release/Aeonmi native aeonmi_ai/stdlib/tests/test_string_builtins.ai
   ./target/release/Aeonmi native aeonmi_ai/stdlib/tests/test_collections_builtins.ai
   ```

## Where to work by concern

- Syntax/token bugs: `src/core/lexer.rs`, `src/core/parser.rs`
- Runtime behavior/builtins: `src/core/vm.rs`
- CLI behavior: `src/cli.rs`, `src/commands/`
- Mother AI loop: `src/mother/`
- Quantum/QUBE: `src/qube/`, `src/core/titan/`
- Dashboard UX: `Aeonmi_Master/dashboard.py`

## Documentation map

- Language spec: `docs/LANGUAGE_SPEC_CURRENT.md`
- User guide: `docs/Aeonmi_Language_Guide.md`
- QUBE spec: `docs/QUBE_SPEC.md`
- Architecture notes: `MOTHER_AI_ARCHITECTURE.md`, `SHARD_STATUS.md`

