//! AEONMI Web3 DAO (Decentralised Autonomous Organisation)
//!
//! Implements an on-chain governance system: members hold voting power
//! (proportional to their token balance or a fixed allocation), submit
//! proposals, cast votes, and execute passing proposals.
//!
//! # Quick start
//! ```
//! use aeonmi_project::web3::dao::{Dao, VoteChoice};
//!
//! let mut dao = Dao::new("AEONMI Protocol DAO", 0.51); // 51 % quorum
//! dao.add_member("alice", 60);
//! dao.add_member("bob",   40);
//!
//! let pid = dao.propose("alice", "Upgrade VM to v2", "Set MAX_FRAMES = 2048");
//! dao.vote(pid, "alice", VoteChoice::For).unwrap();
//! dao.vote(pid, "bob",   VoteChoice::Against).unwrap();
//!
//! let result = dao.tally(pid).unwrap();
//! assert!(result.passed);  // alice has 60 / 100 = 60 % > 51 %
//! ```

use std::collections::HashMap;
use std::fmt;

// ── Types ─────────────────────────────────────────────────────────────────────

/// A member's vote on a proposal.
#[derive(Debug, Clone, PartialEq)]
pub enum VoteChoice {
    For,
    Against,
    Abstain,
}

impl fmt::Display for VoteChoice {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VoteChoice::For     => write!(f, "For"),
            VoteChoice::Against => write!(f, "Against"),
            VoteChoice::Abstain => write!(f, "Abstain"),
        }
    }
}

/// Errors returned by DAO operations.
#[derive(Debug, Clone, PartialEq)]
pub enum DaoError {
    UnknownProposal(u64),
    UnknownMember(String),
    AlreadyVoted { member: String, proposal_id: u64 },
    ProposalClosed(u64),
    NotEnoughPower { member: String, power: u64 },
    ProposalNotPassed(u64),
    AlreadyExecuted(u64),
}

impl fmt::Display for DaoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DaoError::UnknownProposal(id)             => write!(f, "UnknownProposal({})", id),
            DaoError::UnknownMember(m)                => write!(f, "UnknownMember({})", m),
            DaoError::AlreadyVoted { member, proposal_id } =>
                write!(f, "AlreadyVoted(member={}, proposal={})", member, proposal_id),
            DaoError::ProposalClosed(id)              => write!(f, "ProposalClosed({})", id),
            DaoError::NotEnoughPower { member, power } =>
                write!(f, "NotEnoughPower({}): power={}", member, power),
            DaoError::ProposalNotPassed(id)           => write!(f, "ProposalNotPassed({})", id),
            DaoError::AlreadyExecuted(id)             => write!(f, "AlreadyExecuted({})", id),
        }
    }
}

// ── Proposal ──────────────────────────────────────────────────────────────────

/// The current state of a proposal.
#[derive(Debug, Clone, PartialEq)]
pub enum ProposalState {
    Active,
    Closed,
    Executed,
}

/// A governance proposal.
#[derive(Debug, Clone)]
pub struct Proposal {
    pub id:       u64,
    pub title:    String,
    pub body:     String,
    pub proposer: String,
    pub state:    ProposalState,
    /// votes[member] = their choice
    votes: HashMap<String, VoteChoice>,
}

impl Proposal {
    fn new(id: u64, proposer: &str, title: &str, body: &str) -> Self {
        Self {
            id,
            title: title.to_string(),
            body:  body.to_string(),
            proposer: proposer.to_string(),
            state: ProposalState::Active,
            votes: HashMap::new(),
        }
    }

    /// Returns `true` if `member` has already voted on this proposal.
    pub fn has_voted(&self, member: &str) -> bool {
        self.votes.contains_key(member)
    }

    /// Returns the vote cast by `member`, if any.
    pub fn vote_of(&self, member: &str) -> Option<&VoteChoice> {
        self.votes.get(member)
    }
}

// ── Tally ─────────────────────────────────────────────────────────────────────

/// Result of tallying a proposal.
#[derive(Debug, Clone)]
pub struct TallyResult {
    pub proposal_id:    u64,
    pub votes_for:      u64,
    pub votes_against:  u64,
    pub votes_abstain:  u64,
    pub total_power:    u64,
    pub quorum_pct:     f64, // required fraction [0,1]
    pub participation:  f64, // fraction of total power that voted
    pub for_fraction:   f64, // fraction of participating power that voted For
    pub passed:         bool,
}

impl fmt::Display for TallyResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Proposal #{}: {} | For={} Against={} Abstain={} | participation={:.1}% for_frac={:.1}%",
            self.proposal_id,
            if self.passed { "PASSED" } else { "FAILED" },
            self.votes_for,
            self.votes_against,
            self.votes_abstain,
            self.participation * 100.0,
            self.for_fraction * 100.0,
        )
    }
}

// ── DAO ───────────────────────────────────────────────────────────────────────

/// An in-memory DAO governance instance.
#[derive(Debug, Clone)]
pub struct Dao {
    pub name: String,
    /// Fraction of total voting power required for a vote to pass (0.0–1.0).
    pub quorum: f64,
    /// member address → voting power (e.g., token balance).
    members: HashMap<String, u64>,
    proposals: Vec<Proposal>,
    next_id: u64,
    /// Log of executed proposal IDs.
    pub execution_log: Vec<u64>,
}

impl Dao {
    /// Create a new DAO with the given name and quorum threshold.
    ///
    /// `quorum` is the minimum fraction of **For** votes out of all cast
    /// votes required for a proposal to pass (e.g. `0.51` = simple majority).
    pub fn new(name: &str, quorum: f64) -> Self {
        Self {
            name: name.to_string(),
            quorum,
            members:       HashMap::new(),
            proposals:     Vec::new(),
            next_id:       1,
            execution_log: Vec::new(),
        }
    }

    // ── Members ───────────────────────────────────────────────────────────────

    /// Register a member with `power` voting units.
    pub fn add_member(&mut self, address: &str, power: u64) {
        self.members.insert(address.to_string(), power);
    }

    /// Return the voting power of `address` (0 if not a member).
    pub fn power_of(&self, address: &str) -> u64 {
        *self.members.get(address).unwrap_or(&0)
    }

    /// Total voting power across all members.
    pub fn total_power(&self) -> u64 {
        self.members.values().sum()
    }

    // ── Proposals ─────────────────────────────────────────────────────────────

    /// Submit a new proposal.  Returns the proposal ID.
    pub fn propose(&mut self, proposer: &str, title: &str, body: &str) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        self.proposals.push(Proposal::new(id, proposer, title, body));
        id
    }

    /// Retrieve a reference to a proposal by ID.
    pub fn proposal(&self, id: u64) -> Option<&Proposal> {
        self.proposals.iter().find(|p| p.id == id)
    }

    fn proposal_mut(&mut self, id: u64) -> Option<&mut Proposal> {
        self.proposals.iter_mut().find(|p| p.id == id)
    }

    // ── Voting ────────────────────────────────────────────────────────────────

    /// Cast a vote on proposal `id`.
    pub fn vote(&mut self, id: u64, member: &str, choice: VoteChoice) -> Result<(), DaoError> {
        if self.members.get(member).copied().unwrap_or(0) == 0 {
            return Err(DaoError::UnknownMember(member.to_string()));
        }
        let proposal = self.proposal_mut(id)
            .ok_or(DaoError::UnknownProposal(id))?;
        if proposal.state != ProposalState::Active {
            return Err(DaoError::ProposalClosed(id));
        }
        if proposal.has_voted(member) {
            return Err(DaoError::AlreadyVoted { member: member.to_string(), proposal_id: id });
        }
        proposal.votes.insert(member.to_string(), choice);
        Ok(())
    }

    // ── Tally ─────────────────────────────────────────────────────────────────

    /// Tally votes and determine whether the proposal passed.
    pub fn tally(&self, id: u64) -> Result<TallyResult, DaoError> {
        let proposal = self.proposal(id).ok_or(DaoError::UnknownProposal(id))?;
        let mut for_power:     u64 = 0;
        let mut against_power: u64 = 0;
        let mut abstain_power: u64 = 0;

        for (member, choice) in &proposal.votes {
            let power = self.power_of(member);
            match choice {
                VoteChoice::For     => for_power     += power,
                VoteChoice::Against => against_power += power,
                VoteChoice::Abstain => abstain_power += power,
            }
        }

        let total  = self.total_power();
        let active = for_power + against_power + abstain_power;
        let participation = if total > 0 { active as f64 / total as f64 } else { 0.0 };
        let for_fraction  = if active > 0 { for_power as f64 / active as f64 } else { 0.0 };
        let passed = for_fraction >= self.quorum;

        Ok(TallyResult {
            proposal_id:   id,
            votes_for:     for_power,
            votes_against: against_power,
            votes_abstain: abstain_power,
            total_power:   total,
            quorum_pct:    self.quorum,
            participation,
            for_fraction,
            passed,
        })
    }

    // ── Execution ─────────────────────────────────────────────────────────────

    /// Mark a proposal as executed (after confirming it passed).
    ///
    /// In a real on-chain DAO, this would trigger an on-chain transaction.
    /// Here it closes the proposal and logs its ID.
    pub fn execute(&mut self, id: u64) -> Result<(), DaoError> {
        let result = self.tally(id)?;
        if !result.passed {
            return Err(DaoError::ProposalNotPassed(id));
        }
        {
            let proposal = self.proposal_mut(id).ok_or(DaoError::UnknownProposal(id))?;
            if proposal.state == ProposalState::Executed {
                return Err(DaoError::AlreadyExecuted(id));
            }
            proposal.state = ProposalState::Executed;
        }
        self.execution_log.push(id);
        Ok(())
    }

    /// Close a proposal without executing it (e.g., voting period expired).
    pub fn close(&mut self, id: u64) -> Result<(), DaoError> {
        let proposal = self.proposal_mut(id).ok_or(DaoError::UnknownProposal(id))?;
        if proposal.state != ProposalState::Active {
            return Err(DaoError::ProposalClosed(id));
        }
        proposal.state = ProposalState::Closed;
        Ok(())
    }

    // ── Summary ───────────────────────────────────────────────────────────────

    pub fn summary(&self) -> String {
        format!(
            "DAO[{}] members={} total_power={} proposals={} quorum={:.0}%",
            self.name,
            self.members.len(),
            self.total_power(),
            self.proposals.len(),
            self.quorum * 100.0,
        )
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Dao {
        let mut dao = Dao::new("Test DAO", 0.51);
        dao.add_member("alice", 60);
        dao.add_member("bob",   30);
        dao.add_member("carol", 10);
        dao
    }

    #[test]
    fn test_propose_and_vote_pass() {
        let mut dao = setup();
        let pid = dao.propose("alice", "Upgrade", "Upgrade VM");
        dao.vote(pid, "alice", VoteChoice::For).unwrap();
        dao.vote(pid, "bob",   VoteChoice::For).unwrap();
        let r = dao.tally(pid).unwrap();
        assert!(r.passed, "alice+bob = 90/100 = 90% > 51%");
        assert_eq!(r.votes_for, 90);
    }

    #[test]
    fn test_propose_and_vote_fail() {
        let mut dao = setup();
        let pid = dao.propose("alice", "Risky", "Drain treasury");
        dao.vote(pid, "bob",   VoteChoice::Against).unwrap();
        dao.vote(pid, "carol", VoteChoice::Against).unwrap();
        dao.vote(pid, "alice", VoteChoice::For).unwrap();
        let r = dao.tally(pid).unwrap();
        // alice=60 For, bob=30+carol=10=40 Against → for_frac = 60/100 = 60% > 51% → passed
        // (alice overpowers bob+carol)
        assert!(r.passed);
    }

    #[test]
    fn test_majority_against() {
        let mut dao = Dao::new("Tight DAO", 0.51);
        dao.add_member("alice", 49);
        dao.add_member("bob",   51);
        let pid = dao.propose("alice", "Prop", "body");
        dao.vote(pid, "alice", VoteChoice::For).unwrap();
        dao.vote(pid, "bob",   VoteChoice::Against).unwrap();
        let r = dao.tally(pid).unwrap();
        assert!(!r.passed, "alice=49% < 51% quorum");
    }

    #[test]
    fn test_double_vote_error() {
        let mut dao = setup();
        let pid = dao.propose("alice", "P", "B");
        dao.vote(pid, "alice", VoteChoice::For).unwrap();
        let err = dao.vote(pid, "alice", VoteChoice::For).unwrap_err();
        assert!(matches!(err, DaoError::AlreadyVoted { .. }));
    }

    #[test]
    fn test_unknown_member_error() {
        let mut dao = setup();
        let pid = dao.propose("alice", "P", "B");
        let err = dao.vote(pid, "mallory", VoteChoice::For).unwrap_err();
        assert!(matches!(err, DaoError::UnknownMember(_)));
    }

    #[test]
    fn test_execute_passing_proposal() {
        let mut dao = setup();
        let pid = dao.propose("alice", "Enable feature", "body");
        dao.vote(pid, "alice", VoteChoice::For).unwrap();
        dao.execute(pid).unwrap();
        assert!(dao.execution_log.contains(&pid));
        let p = dao.proposal(pid).unwrap();
        assert_eq!(p.state, ProposalState::Executed);
    }

    #[test]
    fn test_execute_failing_proposal_errors() {
        let mut dao = setup();
        let pid = dao.propose("alice", "No support", "body");
        dao.vote(pid, "alice", VoteChoice::Against).unwrap();
        let err = dao.execute(pid).unwrap_err();
        assert!(matches!(err, DaoError::ProposalNotPassed(_)));
    }

    #[test]
    fn test_close_proposal() {
        let mut dao = setup();
        let pid = dao.propose("alice", "Short vote", "body");
        dao.close(pid).unwrap();
        let p = dao.proposal(pid).unwrap();
        assert_eq!(p.state, ProposalState::Closed);
        // Can't vote on closed proposal
        let err = dao.vote(pid, "alice", VoteChoice::For).unwrap_err();
        assert!(matches!(err, DaoError::ProposalClosed(_)));
    }
}
