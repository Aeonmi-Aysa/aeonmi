# Architecture

## Repository layout

```text
src/
  main.rs                # Main CLI entrypoint and runtime routing
  cli.rs                 # Command-line schema
  commands/              # Command implementations
  core/                  # Language pipeline + VM
  mother/                # Mother AI orchestration
  qube/                  # QUBE language/runtime
  glyph/                 # Identity and vault-related cryptographic flows
  vault/                 # Vault command subsystem
  integration/           # Unified runtime composition

aeonmi_ai/
  stdlib/                # Standard library modules in .ai
  stdlib/tests/          # Language-level tests

Aeonmi_Master/
  dashboard.py           # Unified dashboard web app
  aeonmi_ai/             # Higher-level AI modules and shard components
```

## Execution model

1. Source is tokenized in `src/core/lexer.rs`
2. Parser builds AST in `src/core/parser.rs`
3. AST is lowered to IR in `src/core/lowering.rs`
4. Tree-walk VM executes in `src/core/vm.rs`

## Multi-binary workspace

- `Aeonmi` (primary CLI)
- `MotherAI` (standalone Mother AI binary)
- `aeonmi` / `aeonmi_project` aliases for compatibility and tests

## Feature flags (high-level)

- `full-suite` (default)
- `quantum`
- `mother-ai`
- `titan-libraries`
- `holographic`
- `voice`
- `minimal`

