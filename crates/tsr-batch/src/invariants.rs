//! The session-invariant check (PRD §B6, §B5, §B4).
//!
//! Every recorded session must satisfy `tsr_cli::reconstruct::Session`'s invariants. `tsr-cli`
//! depends on this crate (it owns the `tsr task run` command), so this module cannot depend on
//! `tsr-cli` back: it re-states the same predicates over the same `(seq, Event)` log, and
//! `crates/tsr-cli/tests/m3_batch_invariants.rs` asserts the two agree by running a scripted batch
//! session through `tsr_cli::reconstruct::Session` itself.
//!
//! The invariants, as `tsr_cli::reconstruct` states them:
//!
//! - sequence is monotone and every parent precedes its child (B6);
//! - every gated action (`RunApprovedTest`, `ProposePatch`, `RequestApproval`) has a forecast at the
//!   same state version before it was committed (B5);
//! - every dispatch is reconciled — `Executed` or an explicit `Unresolved` (B6);
//! - every command ran in the sealed sandbox (B4).

use std::collections::{HashMap, HashSet};

use tsr_domain::{ActionKind, CandidateId, Event, EventKind, TaskId};
use tsr_store::Store;

/// Which invariant a session violated.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvariantViolation {
    /// The store's sequence is not monotone.
    NonMonotoneSeq,
    /// A parent event does not precede its child.
    ParentAfterChild,
    /// A gated action has no forecast at the state version it was committed at.
    GatedActionWithoutForecast,
    /// A dispatch was neither executed nor marked unresolved.
    UnreconciledDispatch,
    /// A command ran outside the sealed sandbox.
    CommandOutsideSandbox,
}

impl InvariantViolation {
    /// The invariant's name, as the PRD states it.
    pub fn as_str(self) -> &'static str {
        match self {
            InvariantViolation::NonMonotoneSeq => "sequence is monotone",
            InvariantViolation::ParentAfterChild => "every parent precedes its child",
            InvariantViolation::GatedActionWithoutForecast => {
                "every gated action has a prior forecast at the same state version"
            }
            InvariantViolation::UnreconciledDispatch => {
                "every dispatch is reconciled or unresolved"
            }
            InvariantViolation::CommandOutsideSandbox => "every command ran in the sealed sandbox",
        }
    }
}

/// A session as reconstructed from the log alone: `(seq, Event)` in sequence order.
#[derive(Debug, Default, Clone)]
pub struct Session {
    /// The events, in sequence order.
    pub events: Vec<(i64, Event)>,
}

impl Session {
    /// Sequence is strictly increasing.
    pub fn monotone_seq(&self) -> bool {
        self.events.windows(2).all(|w| w[0].0 < w[1].0)
    }

    /// Every parent event precedes its child.
    pub fn every_parent_precedes_its_child(&self) -> bool {
        let mut seen = HashSet::new();
        for (_, e) in &self.events {
            if !e.parents.iter().all(|p| seen.contains(p)) {
                return false;
            }
            seen.insert(e.id);
        }
        true
    }

    fn is_gated(kind: ActionKind) -> bool {
        matches!(
            kind,
            ActionKind::RunApprovedTest | ActionKind::ProposePatch | ActionKind::RequestApproval
        )
    }

    /// A gated action must have a forecast at the state version current when it was committed.
    /// The state version of a commitment is the number of events before it in this session's log
    /// (the configuration size), which is what the hook records a forecast against.
    pub fn every_gated_action_has_prior_forecast_at_same_state_version(&self) -> bool {
        let mut proposed: HashMap<CandidateId, ActionKind> = HashMap::new();
        let mut forecast_versions: HashMap<CandidateId, Vec<u64>> = HashMap::new();
        for (i, (_, e)) in self.events.iter().enumerate() {
            match &e.kind {
                EventKind::Proposed { candidate } => {
                    proposed.insert(candidate.id, candidate.kind);
                }
                EventKind::Forecast {
                    forecast,
                    candidate,
                } => {
                    forecast_versions
                        .entry(*candidate)
                        .or_default()
                        .push(forecast.state_version);
                }
                EventKind::Committed { candidate, .. } => {
                    let kind = match proposed.get(candidate) {
                        Some(k) => *k,
                        None => return false,
                    };
                    if Self::is_gated(kind) {
                        let version_now = i as u64;
                        let ok = forecast_versions
                            .get(candidate)
                            .map(|vs| vs.contains(&version_now))
                            .unwrap_or(false);
                        if !ok {
                            return false;
                        }
                    }
                }
                _ => {}
            }
        }
        true
    }

    /// Every dispatch is reconciled — `Executed` or an explicit `Unresolved`.
    pub fn every_dispatch_reconciled_or_unresolved(&self) -> bool {
        let mut dispatched: HashSet<CandidateId> = HashSet::new();
        let mut settled: HashSet<CandidateId> = HashSet::new();
        for (_, e) in &self.events {
            match &e.kind {
                EventKind::Dispatched { candidate, .. } => {
                    dispatched.insert(*candidate);
                }
                EventKind::Executed { candidate, .. } | EventKind::Unresolved { candidate, .. } => {
                    settled.insert(*candidate);
                }
                _ => {}
            }
        }
        dispatched.iter().all(|c| settled.contains(c))
    }

    /// Every command ran in the sealed sandbox.
    pub fn every_command_ran_in_sandbox(&self) -> bool {
        self.events.iter().all(|(_, e)| match &e.kind {
            EventKind::Dispatched { in_sandbox, .. } => *in_sandbox,
            _ => true,
        })
    }
}

/// Build a [`Session`] from the store's log for a task.
pub fn session_from_store(store: &Store, task: &TaskId) -> Result<Session, tsr_store::StoreError> {
    let events = store.by_task(task)?;
    Ok(Session { events })
}

/// Check every invariant over a session, returning the first violation, if any.
pub fn check_session_invariants(session: &Session) -> Result<(), InvariantViolation> {
    if !session.monotone_seq() {
        return Err(InvariantViolation::NonMonotoneSeq);
    }
    if !session.every_parent_precedes_its_child() {
        return Err(InvariantViolation::ParentAfterChild);
    }
    if !session.every_gated_action_has_prior_forecast_at_same_state_version() {
        return Err(InvariantViolation::GatedActionWithoutForecast);
    }
    if !session.every_dispatch_reconciled_or_unresolved() {
        return Err(InvariantViolation::UnreconciledDispatch);
    }
    if !session.every_command_ran_in_sandbox() {
        return Err(InvariantViolation::CommandOutsideSandbox);
    }
    Ok(())
}

/// Check the invariants over a store's log for a task, returning the first violation.
pub fn check_store_invariants(store: &Store, task: &TaskId) -> Result<(), InvariantViolation> {
    let session =
        session_from_store(store, task).map_err(|_| InvariantViolation::NonMonotoneSeq)?;
    check_session_invariants(&session)
}
