# Contributing to Aeonmi

Thank you for your interest in contributing! Aeonmi is an open, community-welcoming project.  
Every contribution — from a typo fix to a new language feature — is valued.

---

## Ways to Contribute

| Type | Examples |
|------|---------|
| 🐛 **Bug reports** | Parser crashes, VM wrong output, CLI errors |
| ✨ **Features** | New language constructs, QUBE gates, CLI commands |
| 📝 **Documentation** | Improve docs, add examples, fix typos |
| 🧪 **Tests** | Add test cases, improve coverage |
| 🎨 **Examples** | New `.ai` programs in `examples/` |
| 🔬 **Quantum** | New QUBE algorithms, simulator improvements |
| 🌐 **Web3** | Smart contract patterns, wallet improvements |

### Good First Issues

🟢 **Easy — great for newcomers:**
- Add a missing parser error message (see `tests/diagnostics.rs`)
- Write a new `.ai` example program
- Fix a typo or improve a doc page
- Add a test case for an edge case you found

🟡 **Medium:**
- Add a built-in function to the VM (`src/core/vm.rs`)
- Improve QUBE ASCII diagram rendering (`src/qube/executor.rs`)
- Implement a missing linter check in `src/commands/lint.rs`

🔴 **Advanced:**
- Complete the Titan LLVM IR backend
- Implement the IBM Quantum REST bridge
- Improve the Mother AI code generation

---

## Development Setup

**Requirements:** Rust 1.74+ from [rustup.rs](https://rustup.rs)

```bash
# Fork and clone
git clone https://github.com/YOUR_USERNAME/aeonmi
cd aeonmi

# Build
cargo build --no-default-features --features "quantum,mother-ai"

# Run all tests (should be 165+ passing)
cargo test --no-default-features --features "quantum,mother-ai"
```

---

## Making Changes

1. Create a branch from `main`:
   ```bash
   git checkout main
   git pull origin main
   git checkout -b feat/my-feature
   ```

2. Make your changes. Keep PRs **focused** — one logical change per PR.

3. Run tests:
   ```bash
   cargo test --no-default-features --features "quantum,mother-ai"
   ```

4. Commit with a clear message (see [commit style](#commit-style)):
   ```bash
   git commit -m "feat: add reverse() builtin to VM"
   ```

5. Push and open a Pull Request against `main`.

---

## Commit Style

We use conventional commit prefixes:

| Prefix | When to use |
|--------|-------------|
| `feat:` | New user-facing feature |
| `fix:` | Bug fix |
| `refactor:` | Code restructure, no behavior change |
| `test:` | Add or update tests |
| `docs:` | Documentation only |
| `chore:` | Build scripts, dependencies, CI |
| `perf:` | Performance improvement |
| `ci:` | CI/CD pipeline changes |

**Examples:**
```
feat: add for-in loop support to VM
fix: parser stack overflow on deeply nested closures
docs: add QUBE Bell-pair example to README
test: add edge cases for genesis array spread operator
```

---

## Code Style

- Follow standard Rust conventions (`rustfmt`)
- Run `cargo clippy` before submitting
- Keep functions short and focused
- Add doc comments to public functions
- Use descriptive variable names (avoid single letters except in math contexts)

---

## Repository Hygiene

**Never commit:**
- `target/` directory (Rust build artifacts)
- `node_modules/` 
- `*.exe` or `*.dll` binaries
- `build_output.txt`, `warnings.txt`, or similar build logs
- Temporary test files (`__exec_tmp*`)

Run before every push:
```bash
git status --short
```

If you accidentally commit a large file (>1MB), remove it and open an issue.  
All text files use **LF line endings** (enforced by `.gitattributes`).

---

## Pull Request Guidelines

- **Title:** Use the same conventional prefix as your commits
- **Description:** Explain what and why, not just how
- **Link issues:** `Closes #123` or `Related to #456`
- **Tests:** Include tests for any new behavior
- **Docs:** Update docs for any user-visible change
- **One thing:** One PR = one logical change

---

## Security

Do **not** open public issues for security vulnerabilities.  
See [`SECURITY.md`](SECURITY.md) for the coordinated disclosure process.

---

## Questions?

Open a GitHub Discussion or an issue tagged `question`.  
We're friendly — no question is too small.

---

*Thank you for helping build the future of AI-first programming!* ⧉
