# Security Policy

## Supported Versions

The following table shows which versions of Aeonmi receive active security updates:

| Version | Supported |
|---------|-----------|
| 0.2.x   | ✅ |
| < 0.2   | ❌ |

Only actively supported versions will receive patches for security vulnerabilities.
Unsupported versions should be upgraded immediately to ensure security compliance.

---

## Reporting a Vulnerability

Security is a top priority for the Aeonmi project. If you discover a security
issue, **do not open a public GitHub issue**. Public disclosure of vulnerabilities
before a fix is released can put users at risk.

Instead, please follow these steps:

1. **Email:** Send a detailed report to **tech@aeonmi.com**  
   (Until the new mailbox is fully operational, you may also use **digital@darkmeta.ai**.)
2. **Required information:**
   - A detailed description of the vulnerability
   - Steps to reproduce the issue
   - Potential impact assessment
   - Any suggested mitigations
3. **Encrypted communication:** If your report contains sensitive details, request
   the Aeonmi Security Team's PGP key for encryption.
4. **Acknowledgement:** You will receive an acknowledgement within **48 hours**
   confirming receipt of your report.

### Investigation Timeline

- Initial assessment within **5 business days**.
- Status updates provided at least every **7 days** until the issue is resolved.

### Resolution & Disclosure

- Critical issues will be patched as soon as possible.
- Once a fix is released, a public advisory will be issued along with credit
  (unless anonymity is requested).
- We follow **coordinated disclosure** best practices.

---

## Scope of Security Reports

We welcome reports for:

- Remote code execution
- Privilege escalation
- Authentication bypass
- Information leaks / credential exposure
- Data integrity compromise
- Quantum-related encryption or security bypass vulnerabilities in Aeonmi/Q.U.B.E. subsystems

We **do not** consider the following to be vulnerabilities:

- General feature requests
- Outdated dependencies in non-critical dev tools
- Theoretical issues with no practical exploit

---

## Security Model & Known Design Decisions

The following are **intentional design decisions**, not bugs. Understanding them
helps maintainers triage reports correctly.

### `exec_cmd` Builtin — Arbitrary Shell Execution

`.ai` scripts can call `exec_cmd("…")`, which passes the command string to
`sh -c` (Unix) or `cmd /C` (Windows) and runs it with the same OS-level
privileges as the `aeonmi` process.

**This is intentional** — Aeonmi is a general-purpose scripting language and
`exec_cmd` is equivalent to Python's `os.system()` or Node.js `child_process.exec()`.

**What this means for you:**
- Only execute `.ai` scripts from sources you trust.
- Never run untrusted or network-fetched `.ai` files directly.
- A future `--safe-mode` flag that disables `exec_cmd` is on the roadmap
  (see `roadmap.md`).

### API Key Store — Local File Protection Model

The `aeonmi apikey set` command stores provider API keys encrypted with
**ChaCha20** in `~/.aeonmi/keys.json`.

**Key derivation (as of v2 scheme, March 2026):**

```
key = SHA256(username || hostname || "AEONMI_API_KEY_SALT_v2" || install_salt)
```

where `install_salt` is a 32-byte cryptographically-random value generated on
first use and stored at `~/.aeonmi/keystore.salt`.

**Security properties:**
- Keys are not stored in plaintext.
- The encryption key requires reading `~/.aeonmi/keystore.salt`; it is **not**
  reconstructable from username + hostname alone.
- Both files are protected by normal OS file permissions (`~/.aeonmi/`).
- The optional `kdf-argon2` Cargo feature replaces SHA256 with **Argon2id**
  for stronger key stretching.

**What this does NOT protect against:**
- An attacker with read access to your entire `~/.aeonmi/` directory.
- Memory forensics (keys are briefly decrypted in process memory during use;
  sensitive buffers are zeroed via `zeroize` after use).

### ⚠️ Migration Note (upgrading from builds before March 2026)

The key derivation scheme was upgraded in March 2026 (static salt → random
install salt). Existing entries in `keys.json` that were stored with the old
scheme (`v1`) **cannot be decrypted** by the new key material. After upgrading:

```sh
# Clear old encrypted entries and re-add your keys
aeonmi apikey delete <provider>
aeonmi apikey set <provider> <your-key>
```

### File I/O Builtins — No Sandbox

The builtins `read_file`, `write_file`, `append_file`, `delete_file`, and
`read_dir` have unrestricted filesystem access (subject to OS permissions).
This is intentional for a general-purpose scripting language. Do not run
untrusted scripts with access to sensitive directories.

---

## Responsible Disclosure Policy

The Aeonmi team operates on a responsible disclosure model:

- No legal action will be taken against security researchers following this policy.
- Do not exploit the vulnerability beyond the extent necessary to prove it exists.
- Do not publicly disclose the vulnerability prior to coordinated release.
