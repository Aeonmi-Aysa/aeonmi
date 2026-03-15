# AEONMI Web3 Guide

**Version:** 1.0 | **Date:** March 2026 | **Module:** `src/web3/`

---

## Overview

AEONMI ships three production-quality Web3 modules that let `.ai` programs work
with blockchain primitives directly — no external RPC, no wallet browser
extension required.  All logic runs in-memory; the modules are designed for
prototyping, education, and in-game / simulation contexts.

| Module | Source | CLI command | Purpose |
|--------|--------|-------------|---------|
| Wallet | `src/web3/wallet.rs` | `aeonmi wallet` | Key-pair generation, addresses, balance ledger |
| Token  | `src/web3/token.rs`  | `aeonmi token`  | ERC-20/SPL fungible token |
| DAO    | `src/web3/dao.rs`    | `aeonmi dao`    | Governance: proposals, voting, execution |

---

## 1. Wallet

### Concept

Each wallet is derived **deterministically** from a seed string using SHA-256
double-hashing.  The same seed always produces the same address — handy for
testing and repeatable simulations.

Addresses follow the format `AEON<16 hex chars>`, e.g. `AEON3f2a8b9c4d1e7f0a`.

### API

```rust
use aeonmi_project::web3::wallet::{Wallet, Ledger};

// Generate (deterministic)
let alice = Wallet::generate("alice");
println!("{}", alice.address);  // AEON…

// Sign / verify
let sig = alice.sign("transfer:100:AEON…");
assert!(alice.verify("transfer:100:AEON…", &sig));

// Ledger (in-memory balance sheet)
let mut ledger = Ledger::new();
ledger.airdrop(&alice.address, 1_000.0);       // faucet
ledger.transfer(&alice.address, &bob.address, 250.0).unwrap();

println!("{}", ledger.balance(&alice.address)); // 750.0
println!("{}", ledger.summary());               // formatted table
```

### CLI

```bash
# Generate a wallet and show its address + public key
aeonmi wallet new alice

# Check balance (demo: always starts at 0 in CLI mode)
aeonmi wallet balance alice

# Airdrop AEON to a wallet
aeonmi wallet airdrop alice 1000

# Transfer between wallets
aeonmi wallet transfer alice bob 250
```

Sample output:

```
✅ Wallet generated
   Name:    alice
   Address: AEON3f2a8b9c4d1e7f
   PubKey:  a1b2c3d4e5f6…
```

### Error handling

| Error | Meaning |
|-------|---------|
| `LedgerError::InsufficientFunds` | From-balance < requested amount |
| `LedgerError::InvalidAmount`     | Amount ≤ 0 or non-finite |

---

## 2. Token (ERC-20 / SPL)

### Concept

`Token` models a fungible token with:

* **Mint** — create new tokens up to `max_supply`
* **Burn** — destroy tokens, reducing supply
* **Transfer** — move tokens between accounts
* **Approve / TransferFrom** — the classic ERC-20 allowance pattern used by
  DEX routers and staking contracts

### API

```rust
use aeonmi_project::web3::token::Token;

// Create token (name, symbol, decimals, max_supply)
let mut ggt = Token::new("Genesis Glyph Token", "GGT", 18, 1_000_000.0);

// Mint
ggt.mint("AEON_alice", 10_000.0).unwrap();

// Transfer
ggt.transfer("AEON_alice", "AEON_bob", 3_000.0).unwrap();

// Burn
ggt.burn("AEON_alice", 500.0).unwrap();

// Approve + TransferFrom
ggt.approve("AEON_alice", "AEON_router", 2_000.0).unwrap();
ggt.transfer_from("AEON_alice", "AEON_pool", "AEON_router", 1_500.0).unwrap();

println!("{}", ggt.summary());
// Token[Genesis Glyph Token (GGT)] decimals=18 supply=9500.0/1000000.0
```

### CLI

```bash
# Show token metadata
aeonmi token info

# Mint tokens
aeonmi token mint AEON3f2a8b9c 5000

# Transfer
aeonmi token transfer AEON3f2a8b9c AEON0011aabb 1000

# Burn
aeonmi token burn AEON3f2a8b9c 250

# Balance check
aeonmi token balance AEON3f2a8b9c
```

### ERC-20 Equivalence Table

| ERC-20 function | AEONMI equivalent |
|-----------------|-------------------|
| `balanceOf(account)` | `token.balance_of(account)` |
| `transfer(to, amount)` | `token.transfer(from, to, amount)` |
| `approve(spender, amount)` | `token.approve(owner, spender, amount)` |
| `transferFrom(from, to, amount)` | `token.transfer_from(from, to, spender, amount)` |
| `allowance(owner, spender)` | `token.allowance(owner, spender)` |
| `totalSupply()` | `token.total_supply` |
| `mint(to, amount)` *(OpenZeppelin)* | `token.mint(to, amount)` |
| `burn(from, amount)` *(OpenZeppelin)* | `token.burn(from, amount)` |

### Error handling

| Error | Meaning |
|-------|---------|
| `TokenError::InsufficientBalance`  | Not enough balance to transfer/burn |
| `TokenError::InsufficientAllowance` | Spender allowance < requested amount |
| `TokenError::OverflowSupply`        | Minting would exceed `max_supply` |
| `TokenError::InvalidAmount`         | Amount ≤ 0 or non-finite |

---

## 3. DAO Governance

### Concept

A **DAO** (Decentralised Autonomous Organisation) lets token holders govern a
protocol through on-chain proposals.

```
Members hold voting power → Submit proposals → Cast votes →
Tally (For / Against / Abstain) → Execute if quorum met
```

The quorum threshold is configured at DAO creation as a fraction (e.g., `0.51`
= simple majority of **participating** votes).

### API

```rust
use aeonmi_project::web3::dao::{Dao, VoteChoice};

// Create DAO with 51 % quorum (simple majority)
let mut dao = Dao::new("AEONMI Protocol DAO", 0.51);

// Register members with voting power
dao.add_member("alice", 60);
dao.add_member("bob",   30);
dao.add_member("carol", 10);

// Submit a proposal (returns proposal ID)
let pid = dao.propose("alice", "Upgrade VM to v2", "Set MAX_FRAMES = 2048");

// Vote
dao.vote(pid, "alice", VoteChoice::For).unwrap();
dao.vote(pid, "bob",   VoteChoice::Against).unwrap();

// Tally
let result = dao.tally(pid).unwrap();
println!("{}", result);
// Proposal #1: PASSED | For=60 Against=30 Abstain=0 | participation=90.0% for_frac=66.7%

// Execute (only succeeds if result.passed)
dao.execute(pid).unwrap();
println!("Executed proposals: {:?}", dao.execution_log);
```

### CLI

```bash
# Show DAO membership and voting-power summary
aeonmi dao status

# Submit a new proposal
aeonmi dao propose "Upgrade VM" "Set MAX_FRAMES = 2048"

# Vote on proposal #1
aeonmi dao vote 1 alice for
aeonmi dao vote 1 bob   against

# Tally votes for proposal #1
aeonmi dao tally 1

# Execute proposal #1 (must have passed)
aeonmi dao execute 1
```

Sample tally output:

```
📊 Proposal #1: PASSED | For=60 Against=30 Abstain=0 | participation=90.0% for_frac=66.7%
```

### Proposal lifecycle

```
propose()  →  Active  →  vote()  →  tally()  →  execute() → Executed
                                                    ↓ (if failed)
                                               close() → Closed
```

| State | Meaning |
|-------|---------|
| `Active`   | Accepting votes |
| `Closed`   | Voting ended, not executed |
| `Executed` | Proposal accepted and executed |

### Error handling

| Error | Meaning |
|-------|---------|
| `DaoError::UnknownProposal(id)` | No proposal with that ID |
| `DaoError::UnknownMember(addr)` | Voter is not a registered member |
| `DaoError::AlreadyVoted`        | Member already voted on this proposal |
| `DaoError::ProposalClosed(id)`  | Can't vote/execute on a non-active proposal |
| `DaoError::ProposalNotPassed(id)` | Execute called but quorum not met |
| `DaoError::AlreadyExecuted(id)` | Proposal already executed |

---

## Using Web3 modules in `.ai` scripts

The modules are available as builtins — call them from native `.ai` code via
the `aeonmi run` command once they are wired into the VM (planned for Phase 6):

```javascript
// examples/web3_demo.ai
let alice = wallet_generate("alice");
let bob   = wallet_generate("bob");

wallet_airdrop(alice.address, 1000.0);
wallet_transfer(alice.address, bob.address, 300.0);

log("Alice: " + wallet_balance(alice.address));  // 700.0
log("Bob:   " + wallet_balance(bob.address));    // 300.0

let ggt = token_new("Genesis Glyph Token", "GGT", 18, 1000000.0);
token_mint(ggt, alice.address, 5000.0);
token_transfer(ggt, alice.address, bob.address, 1000.0);

let dao = dao_new("Protocol DAO", 0.51);
dao_add_member(dao, alice.address, 60);
dao_add_member(dao, bob.address, 40);

let pid = dao_propose(dao, alice.address, "Enable staking", "body");
dao_vote(dao, pid, alice.address, "for");
dao_execute(dao, pid);
```

---

## Architecture

```
src/web3/
├── mod.rs       — Module root; re-exports wallet, token, dao
├── wallet.rs    — KeyPair, Wallet, Ledger, LedgerError
├── token.rs     — Token, TokenError
└── dao.rs       — Dao, Proposal, TallyResult, VoteChoice, DaoError
```

All three modules have zero external dependencies beyond `sha2` (already a
project dependency).  They compile under `--no-default-features --features
"quantum,mother-ai"`.

---

## Test coverage

| Test file | Tests | Description |
|-----------|-------|-------------|
| `src/web3/wallet.rs` (inline) | 7 | Unit: determinism, sign, balance, transfer, history |
| `src/web3/token.rs` (inline) | 9 | Unit: mint, burn, transfer, approve, cap |
| `src/web3/dao.rs` (inline) | 8 | Unit: vote, tally, execute, close, error paths |
| `tests/web3_integration.rs` | 17 | Integration: cross-module scenarios |

Run with:

```bash
cargo test --no-default-features --features "quantum,mother-ai" web3
```

Expected output: **40 tests, 0 failures**.

---

## Roadmap

| # | Feature | Module |
|---|---------|--------|
| W-1 | Persistent ledger (TOML/JSON file backend) | wallet |
| W-2 | ECDSA signing with `k256` crate | wallet |
| T-1 | Events / transfer log (ERC-20 `event Transfer`) | token |
| T-2 | Token factory (deploy multiple token types) | token |
| D-1 | Time-locked proposals (block/timestamp expiry) | dao |
| D-2 | Delegated voting | dao |
| D-3 | Multi-sig execution threshold | dao |
| X-1 | VM builtins: `wallet_generate`, `token_mint`, `dao_vote` | vm |
| X-2 | Solana Anchor stubs for each module | codegen |
