# CLI Reference

## Main command format

```bash
aeonmi <command> [options]
```

## Core commands

- `emit <input>`: Emit compiled output (`--emit ai|js`, `-o/--out`)
- `run <input>`: Run `.ai` file (supports native and bytecode modes)
- `build [input]`: Pipeline build with optional output path
- `format <inputs...>`: Format source files
- `lint <inputs...>`: Lint source files
- `edit [file]`: Line editor or `--tui`
- `tokens <input>` / `ast <input>`: Debug parse internals

## Ecosystem commands

- `qube run <file.qube>`: Execute QUBE circuits
- `qube check <file.qube>`: Validate QUBE source
- `mother`: Launch Mother AI loop
- `vault ...`: Identity and vault operations
- `mint <file.ai>`: Generate metadata/minting artifacts

## Useful global flags

- `--pretty-errors`
- `--no-sema`
- `--debug-titan`
- `--config <file>`

