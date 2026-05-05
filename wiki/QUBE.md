# QUBE

QUBE is Aeonmi’s `.qube` quantum circuit format with a dedicated parser and executor.

## Commands

```bash
aeonmi qube check circuit.qube
aeonmi qube run circuit.qube
aeonmi qube run circuit.qube --diagram
```

## Current behavior

- Parses QUBE source into QUBE AST
- Executes supported gates and measurements
- Emits execution summary and optional text diagram

## Spec

See full specification: `docs/QUBE_SPEC.md`

