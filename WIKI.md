# Aeonmi Wiki — Milestones (April 2026)

## Current milestone snapshot

### M0 — Aeonmi.ai launch alignment ✅
- Repo milestones now explicitly track the Aeonmi.ai live/update milestone.

### M1 — Native runtime consolidation ✅
- `aeonmi native`, `aeonmi run`, and `aeonmi exec` now route through the Rust VM pipeline.
- JavaScript runtime dependency was removed from the execution path.

### M2 — Import system online ✅
- VM-level import resolution is active.
- `.ai` files can load other `.ai` modules without manual concatenation.

### M3 — Mother operational upgrades ✅
- Multi-turn conversation history support is active.
- Persistent Mother memory is stored in `Aeonmi_Master/genesis.json`.
- Dashboard exposes Mother + agent actions in a unified interface.

### M4 — Quantum + identity foundations ✅
- QUBE command set is active (`qube run`, `qube check`).
- Glyph vault + identity flow remains available from CLI (`vault init`, `mint`).

### M5 — Self-hosting shard path 🔄 In progress
- Shard bootstrap is present and running.
- Remaining milestone: full end-to-end shard self-compilation pipeline.

## Next target milestones
- Complete shard self-hosting pass across lexer/parser/AST/codegen/main stages.
- Continue hardening runtime edge cases (`arr[i]`, modulo operator path, parser conflicts).
- Expand Mother memory tooling and maintenance cycles.
