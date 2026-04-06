# LANGUAGE_MASTERY_SUMMARY

## Distilled language understanding

Aeonmi is an AI-native language/runtime ecosystem with three cooperating tracks: Rust cognitive/runtime core, `.ai` operational layer, and Python external bridges.

- `.ai` is the primary operational language for Mother systems, agents, learning flows, orchestration, and operational modules.
- `.qube` is the symbolic/circuit quantum representation used for circuit expression and quantum execution semantics.
- The implementation truth currently lives in Rust (`lexer -> parser -> AST -> IR -> VM` + QUBE executor + Mother subsystems).

## .ai grammar/semantics (observed)

- Function forms:
  - `function name(args) { ... }`
  - `◯ name⟨args⟩ { ... }` (circle-form used heavily in educational/Shard-style code)
- Comments:
  - `// ...`
  - `⍝ ...` (APL-style, prevalent in language-resource examples)
- Variables and control:
  - `let x = ...;`
  - `if (...) { ... }`
  - `while (...) { ... }`
  - `for item in items { ... }`
- Core collection style:
  - Flat arrays with record packing (2-field, 3-field layouts)
  - Access idiom frequently uses `slice(...).pop()` in educational modules
- Built-in ecosystem functions include runtime operations (`print`, logging-style calls, quantum entry points)

## .qube grammar/semantics (observed)

Both lightweight symbolic forms and block forms are present in docs/resources.

- Symbolic style:
  - `state q0 = |0⟩`
  - `apply H -> q0`
  - `apply CNOT(q0, q1)`
  - `collapse q0 -> r0`
  - `assert r0 ∈ {0, 1}`
- Block style (implemented in parser/executor tests):
  - `circuit { ... }`, `meta { ... }`, `execute`, `expected { ... }`

## Runtime model map

1. **Language pipeline**: source -> lexer -> parser -> AST -> lowering/IR -> VM/bytecode
2. **QUBE path**: `.qube` lexer/parser/AST -> executor -> simulation/backend bridge
3. **Mother path** (`src/mother/`): quantum core, emotional core, language evolution, attention, neural, graph, inner voice, embryo loop
4. **Persistence/security**: vault/encryption/glyph subsystems
5. **External bridge layer**: Python (dashboard, qiskit_runner, sync/relay adapters)

## Mother architecture (internal map)

Rust modules indicate explicit subsystems:
- `quantum_core`
- `emotional_core`
- `language_evolution`
- `quantum_attention`
- `neural`
- `knowledge_graph`
- `inner_voice`
- `embryo_loop`

These compose the cognitive runtime and interaction loop with persistent state (`genesis.json` in docs narrative).

## Titan / Vault / Agent / Quantum intent

- **Titan**: extensive math/quantum/numerics library set in `src/core/titan/`
- **Vault/Glyph**: domain, security, identity, and persistence primitives
- **Agent ecosystem**: strong `.ai` operational modules in `Aeonmi_Master/aeonmi_ai/`
- **Quantum integration**: native simulation path + Python/Qiskit hardware bridge path

## Naming/style conventions

From docs + code/resource examples:
- `function` is canonical keyword in runtime language examples
- Prefix families common in educational guidance: `ai_`, `quantum_`, `mother_`, `get_`, `set_`
- Flat-array deterministic state passing is encouraged for portability and self-hosting progression

## Translation strategy anchor

- Preserve behavioral intent first, then optimize idiomatic `.ai/.qube` expression.
- Use `.qube` for grammar-heavy language-core + quantum-circuit representations.
- Use `.ai` for orchestration, subsystem logic, stateful Mother/agent/runtime code.
- Explicitly mark parity gaps/blockers; do not silently skip hard constructs.
