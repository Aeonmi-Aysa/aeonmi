# Aeonmi Language — VS Code Extension

Adds syntax highlighting, bracket matching, and code snippets for Aeonmi and QUBE files inside Visual Studio Code.

## Supported File Types

| Extension | Language |
|-----------|----------|
| `.ai`     | Aeonmi   |
| `.aeon`   | Aeonmi   |
| `.aeonmi` | Aeonmi   |
| `.qube`   | QUBE     |

## Features

### Syntax Highlighting

**Aeonmi (`.ai`):**
- Keywords: `let`, `const`, `fn`, `if`, `while`, `return`, …
- Quantum keywords: `qubit`, `superpose`, `entangle`, `measure`, `circuit`, …
- Glyph operators: `⧉`, `⟨⟩`, `…`, `⊗`, `↦`, `‥`, `←`
- Builtin functions: `print`, `log`, `len`, `push`, `slice`, `concat`, …
- Strings, numbers, comments

**QUBE (`.qube`):**
- Keywords: `state`, `apply`, `collapse`, `assert`, `log`
- Gate names: `H`, `X`, `Y`, `Z`, `S`, `T`, `CNOT`, `CZ`, `SWAP`, `Rx`, `Ry`, `Rz`
- Qubit literals: `|0⟩`, `|1⟩`, `|+⟩`, …
- Comment glyphs: `∴` (therefore), `∵` (because)
- Set membership operator: `∈`

### Code Snippets

**Aeonmi snippets** (trigger in `.ai` files):

| Prefix    | Inserts                    |
|-----------|----------------------------|
| `fn`      | Function definition        |
| `let`     | Variable binding           |
| `if`      | If/else block              |
| `while`   | While loop                 |
| `qubit`   | Qubit declaration          |
| `measure` | Qubit measurement          |
| `tensor`  | Tensor product expression  |
| `bind`    | Symbolic binding `↦`       |
| `main`    | Main function + call       |
| `import`  | Import statement           |

**QUBE snippets** (trigger in `.qube` files):

| Prefix    | Inserts                     |
|-----------|-----------------------------|
| `state`   | State declaration           |
| `super`   | Equal superposition state   |
| `apply`   | Gate application            |
| `cnot`    | CNOT gate                   |
| `collapse`| Measurement                 |
| `assert`  | Assertion                   |
| `bell`    | Full Bell state circuit     |
| `log`     | Log statement               |

### Bracket Matching

Auto-closes and highlights matching pairs for: `()`, `[]`, `{}`, `⟨⟩`, `⧉`.

## Installation

### From source (development)

1. Copy this `vscode-aeonmi/` folder into your VS Code extensions directory:
   - **Linux/macOS:** `~/.vscode/extensions/aeonmi-0.1.0/`
   - **Windows:** `%USERPROFILE%\.vscode\extensions\aeonmi-0.1.0\`
2. Reload VS Code.

### From VSIX (when published)

```
code --install-extension aeonmi-0.1.0.vsix
```

## Building the VSIX

Requires Node.js and `vsce`:

```bash
npm install -g @vscode/vsce
cd vscode-aeonmi
vsce package
```

## Project Links

- [Aeonmi repository](https://github.com/Aeonmi-Aysa/aeonmi)
- [QUBE grammar reference](../docs/grammar_qube.md)
- [Language specification](../docs/language_spec.md)
