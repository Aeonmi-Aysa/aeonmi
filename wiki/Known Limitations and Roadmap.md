# Known Limitations and Roadmap

## Current known limitations

- Array subscript ergonomics are still inconsistent (`arr[i]` caveats).
- `%` / `fmod()` behavior has parser/runtime caveats in some flows.
- Object-literal-as-argument parsing can be fragile in edge cases.
- Shard self-hosting path is not fully complete end-to-end.

## Stable implemented areas

- Native `.ai` execution path
- QUBE parse/execute commands
- Mother command flow
- Dashboard runtime
- Vault + mint flows

## Active direction

- Continue Shard self-hosting completion
- Tighten parser/VM edge-case behavior
- Keep docs aligned with verified command/runtime behavior

