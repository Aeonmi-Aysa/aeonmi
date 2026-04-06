# BLOCKERS_AND_LANGUAGE_GAPS

## Current blockers/gaps (initial)

1. **Rust trait/generic-heavy modules**
   - Some abstractions require manual specialization for `.ai`.
2. **Borrow/lifetime semantics**
   - No direct equivalent; requires explicit ownership/state threading rewrite.
3. **Concurrency/async surfaces**
   - Must be mapped to deterministic orchestration or explicit bridge APIs.
4. **Cryptography/perf-critical primitives**
   - May remain host-backed while exposing `.ai` orchestration interfaces.
5. **External Python bridges (Qiskit/dashboard/infrastructure)**
   - Must stay operational until equivalent native `.ai/.qube` path is proven.

## Proposed language/runtime additions

- Native dictionary/map type with deterministic iteration mode.
- First-class result/error helpers and ergonomic pattern matching.
- Bridge-safe async task API with deterministic scheduling profile.
- Standardized serialization helpers for state sync (genesis/vault interfaces).

## Follow-up recommendations

- Maintain per-file parity notes during translation.
- Keep explicit blocked status in matrix (not silent skips).
- Add targeted equivalence tests for each translated subsystem.
