# RUST_TO_QUBE_AI_NOTES

## Recurring conversion rules

- Rust `struct` -> `.ai` flat-array state + constructor/accessor/update functions.
- Rust `enum` -> tagged value in `.ai` (integer tag + payload array).
- Rust `Result<T,E>` -> `[ok_flag, payload]` where `ok_flag` is `1/0`.
- Rust `Option<T>` -> sentinel (`null`/empty-string/-1) plus guard helpers.
- Rust `HashMap<K,V>` -> flat key/value array or table arrays (`[k,v,k,v,...]`).
- Rust ownership/mutability -> explicit state threading between functions.
- Rust iterators -> explicit loops (`while`/indexed traversal) in `.ai`.
- Rust parser/lexer/AST/IR layers -> prefer `.qube` for grammar-heavy expression, `.ai` for operational orchestration.

## Idiomatic replacement patterns

- Error propagation: replace `?` with explicit result checks and early return arrays.
- Trait-based polymorphism: encode strategy dispatch via tag + dispatch function.
- Generics: specialize to concrete operational types used in Aeonmi runtime flow.
- Async/concurrency: preserve sequencing deterministically; mark async bridge boundaries explicitly.

## Interface preservation checklist

For each file translation:
1. Preserve public entry-point names/intent.
2. Preserve data-flow side effects and state transitions.
3. Preserve diagnostics/error pathways.
4. Annotate known deviations and test needs.
