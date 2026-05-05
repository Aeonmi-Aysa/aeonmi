# CLI Reference

## Core runtime

- `aeonmi run <file.ai>` — run `.ai` via native VM
- `aeonmi native <file.ai>` — explicit native VM path
- `aeonmi exec <file>` — extension-based execution:
  - `.ai` → native VM
  - `.qube` → QUBE executor
  - `.py` → python passthrough
  - `.rs` → temporary `rustc` compile+run

## Build/emit

- `aeonmi emit <input> --emit ai|js -o <out>`
- `aeonmi build [input.ai] [--out file]`
- `aeonmi format <inputs...>`
- `aeonmi lint <inputs...> [--fix]`

## QUBE

- `aeonmi qube run <file.qube> [--diagram]`
- `aeonmi qube check <file.qube>`

## Mother and AI

- `aeonmi mother [--file file.ai] [--creator name] [--verbose]`
- `aeonmi repl`
- `aeonmi ai ...` (AI helper subcommands)

## Identity + mint

- `aeonmi vault ...` (vault subcommands)
- `aeonmi mint <file.ai> [--personality ...] [--anchor] [--out file]`

## Metrics and ops

- `aeonmi metrics-dump`
- `aeonmi metrics-flush`
- `aeonmi metrics-path`
- `aeonmi metrics-top`
- `aeonmi metrics-config`
- `aeonmi metrics-deep`
- `aeonmi metrics-export <file.csv>`

## Tool passthrough

- `aeonmi cargo <args...>`
- `aeonmi python <args...>`

