/// Integration tests for the three new AEONMI Web3 modules:
///   1. Wallet  — key-pair generation, balance ledger
///   2. Token   — ERC-20/SPL-style fungible token
///   3. DAO     — Governance: proposals, voting, execution

// ── Wallet ────────────────────────────────────────────────────────────────────

use aeonmi_project::web3::wallet::{Ledger, Wallet};
use aeonmi_project::web3::token::Token;
use aeonmi_project::web3::dao::{Dao, VoteChoice};

#[test]
fn wallet_generate_is_deterministic() {
    let a = Wallet::generate("test-seed-42");
    let b = Wallet::generate("test-seed-42");
    assert_eq!(a.address,              b.address);
    assert_eq!(a.keypair.public_key,   b.keypair.public_key);
    assert_eq!(a.keypair.private_key,  b.keypair.private_key);
}

#[test]
fn wallet_different_seeds_differ() {
    let a = Wallet::generate("seed-a");
    let b = Wallet::generate("seed-b");
    assert_ne!(a.address, b.address);
}

#[test]
fn wallet_sign_and_verify() {
    let w = Wallet::generate("signer");
    let msg = "aeonmi:transfer:100";
    let sig = w.sign(msg);
    assert!(w.verify(msg, &sig));
    assert!(!w.verify("tampered", &sig));
}

#[test]
fn ledger_airdrop_and_balance() {
    let mut ledger = Ledger::new();
    let w = Wallet::generate("faucet-recipient");
    ledger.airdrop(&w.address, 999.0);
    assert_eq!(ledger.balance(&w.address), 999.0);
}

#[test]
fn ledger_transfer_round_trip() {
    let alice = Wallet::generate("alice-ledger");
    let bob   = Wallet::generate("bob-ledger");
    let mut ledger = Ledger::new();
    ledger.airdrop(&alice.address, 500.0);
    ledger.transfer(&alice.address, &bob.address, 200.0).unwrap();
    assert_eq!(ledger.balance(&alice.address), 300.0);
    assert_eq!(ledger.balance(&bob.address),   200.0);
}

#[test]
fn ledger_records_history() {
    let a = Wallet::generate("hist-a");
    let b = Wallet::generate("hist-b");
    let mut ledger = Ledger::new();
    ledger.airdrop(&a.address, 100.0);
    ledger.transfer(&a.address, &b.address, 50.0).unwrap();
    assert_eq!(ledger.history.len(), 2);
    assert_eq!(ledger.history[1].from, a.address);
    assert_eq!(ledger.history[1].to,   b.address);
    assert_eq!(ledger.history[1].amount, 50.0);
}

// ── Token ─────────────────────────────────────────────────────────────────────

#[test]
fn token_mint_increases_supply() {
    let mut t = Token::new("GGT", "GGT", 18, 1_000_000.0);
    t.mint("alice", 1000.0).unwrap();
    assert_eq!(t.total_supply, 1000.0);
    assert_eq!(t.balance_of("alice"), 1000.0);
}

#[test]
fn token_transfer_moves_balance() {
    let mut t = Token::new("GGT", "GGT", 18, 1_000_000.0);
    t.mint("alice", 500.0).unwrap();
    t.transfer("alice", "bob", 300.0).unwrap();
    assert_eq!(t.balance_of("alice"), 200.0);
    assert_eq!(t.balance_of("bob"),   300.0);
    assert_eq!(t.total_supply, 500.0); // supply unchanged
}

#[test]
fn token_burn_reduces_supply() {
    let mut t = Token::new("GGT", "GGT", 18, 1_000_000.0);
    t.mint("alice", 400.0).unwrap();
    t.burn("alice", 150.0).unwrap();
    assert_eq!(t.balance_of("alice"), 250.0);
    assert_eq!(t.total_supply, 250.0);
}

#[test]
fn token_approve_and_transfer_from() {
    let mut t = Token::new("GGT", "GGT", 18, 1_000_000.0);
    t.mint("owner", 1000.0).unwrap();
    t.approve("owner", "spender", 400.0).unwrap();
    t.transfer_from("owner", "recipient", "spender", 300.0).unwrap();
    assert_eq!(t.balance_of("recipient"), 300.0);
    assert_eq!(t.allowance("owner", "spender"), 100.0);
}

#[test]
fn token_exceeds_cap_fails() {
    let mut t = Token::new("Tiny", "TNY", 0, 50.0);
    assert!(t.mint("alice", 100.0).is_err());
}

// ── DAO ───────────────────────────────────────────────────────────────────────

fn sample_dao() -> Dao {
    let mut dao = Dao::new("AEONMI DAO", 0.51);
    dao.add_member("alice", 50);
    dao.add_member("bob",   30);
    dao.add_member("carol", 20);
    dao
}

#[test]
fn dao_proposal_passes_with_majority() {
    let mut dao = sample_dao();
    let pid = dao.propose("alice", "Upgrade VM", "Bump MAX_FRAMES");
    dao.vote(pid, "alice", VoteChoice::For).unwrap();
    dao.vote(pid, "bob",   VoteChoice::For).unwrap();
    let r = dao.tally(pid).unwrap();
    // alice+bob = 80/100 = 80% > 51%
    assert!(r.passed);
    assert_eq!(r.votes_for, 80);
}

#[test]
fn dao_proposal_fails_with_minority() {
    let mut dao = sample_dao();
    let pid = dao.propose("carol", "Risky", "body");
    dao.vote(pid, "carol", VoteChoice::For).unwrap();   // 20/100 = 20% < 51%
    dao.vote(pid, "alice", VoteChoice::Against).unwrap();
    let r = dao.tally(pid).unwrap();
    assert!(!r.passed);
}

#[test]
fn dao_execute_passing_proposal() {
    let mut dao = sample_dao();
    let pid = dao.propose("alice", "Enable feature X", "body");
    dao.vote(pid, "alice", VoteChoice::For).unwrap();
    dao.vote(pid, "bob",   VoteChoice::For).unwrap();
    dao.execute(pid).unwrap();
    assert!(dao.execution_log.contains(&pid));
}

#[test]
fn dao_execute_failing_proposal_errors() {
    let mut dao = sample_dao();
    let pid = dao.propose("carol", "No support", "body");
    // carol (20) For, alice (50) Against → for_fraction = 20/70 ≈ 28.6 % < 51 %
    dao.vote(pid, "carol", VoteChoice::For).unwrap();
    dao.vote(pid, "alice", VoteChoice::Against).unwrap();
    assert!(dao.execute(pid).is_err());
}

#[test]
fn dao_double_vote_rejected() {
    let mut dao = sample_dao();
    let pid = dao.propose("alice", "Test", "body");
    dao.vote(pid, "alice", VoteChoice::For).unwrap();
    assert!(dao.vote(pid, "alice", VoteChoice::Abstain).is_err());
}

#[test]
fn dao_unknown_member_rejected() {
    let mut dao = sample_dao();
    let pid = dao.propose("alice", "Test", "body");
    assert!(dao.vote(pid, "mallory", VoteChoice::For).is_err());
}
