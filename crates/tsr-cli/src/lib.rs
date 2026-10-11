//! # tsr-cli — library side of the `tsr` binary
//!
//! M0 (PRD §B11) is done when `reconstruct::from_events` rebuilds a real session from the log alone
//! and `tests/m0_reconstruct.rs` passes. Everything here is scaffolding that names the contract;
//! the lane that owns M0 fills it in.

#![forbid(unsafe_code)]

pub mod note_cmd;
pub mod reconcile_cmd;
pub mod review_cmd;

pub mod reconstruct {
    //! Rebuild a session from its event log and check the M0 invariants.
    //!
    //! The invariants are the PRD's, not the implementer's:
    //! - sequence is monotone and every parent precedes its child (B6);
    //! - every gated action (`RunApprovedTest`, `ProposePatch`, `RequestApproval`) has a forecast
    //!   recorded at the *same state version* before it was committed (B5, forecast-gated execution);
    //! - every dispatch is reconciled — `Executed` or an explicit `Unresolved` — never silent (B6);
    //! - every command ran in the sealed sandbox (`Dispatched { in_sandbox: true }`), never on the
    //!   host (B4).

    use std::path::Path;
    use tsr_domain::{ActionKind, CandidateId, Event, EventKind};

    #[derive(Debug, thiserror::Error)]
    pub enum ReconstructError {
        #[error("io: {0}")]
        Io(#[from] std::io::Error),
        #[error("json on line {line}: {source}")]
        Json {
            line: usize,
            source: serde_json::Error,
        },
        #[error("not implemented: {0}")]
        NotImplemented(&'static str),
    }

    /// A session as reconstructed from the log alone.
    #[derive(Debug, Default)]
    pub struct Session {
        pub events: Vec<(i64, Event)>,
    }

    impl Session {
        pub fn monotone_seq(&self) -> bool {
            self.events.windows(2).all(|w| w[0].0 < w[1].0)
        }

        pub fn every_parent_precedes_its_child(&self) -> bool {
            let mut seen = std::collections::HashSet::new();
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
                ActionKind::RunApprovedTest
                    | ActionKind::ProposePatch
                    | ActionKind::RequestApproval
            )
        }

        /// A gated action must have a forecast at the state version current when it was committed.
        /// The state version of a commitment is the number of events before it in this session's
        /// log (the configuration size), which is what the hook records a forecast against.
        pub fn every_gated_action_has_prior_forecast_at_same_state_version(&self) -> bool {
            use std::collections::HashMap;
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

        pub fn every_dispatch_reconciled_or_unresolved(&self) -> bool {
            use std::collections::HashSet;
            let mut dispatched: HashSet<CandidateId> = HashSet::new();
            let mut settled: HashSet<CandidateId> = HashSet::new();
            for (_, e) in &self.events {
                match &e.kind {
                    EventKind::Dispatched { candidate, .. } => {
                        dispatched.insert(*candidate);
                    }
                    EventKind::Executed { candidate, .. }
                    | EventKind::Unresolved { candidate, .. } => {
                        settled.insert(*candidate);
                    }
                    _ => {}
                }
            }
            dispatched.iter().all(|c| settled.contains(c))
        }

        pub fn every_command_ran_in_sandbox(&self) -> bool {
            self.events.iter().all(|(_, e)| match &e.kind {
                EventKind::Dispatched { in_sandbox, .. } => *in_sandbox,
                _ => true,
            })
        }
    }

    /// Rebuild a session from an `events.jsonl` file: one JSON object per line, in sequence order,
    /// each either a bare `Event` or `{"seq": n, "event": Event}`.
    ///
    /// M0 replaces this body. Until then it says so, and the M0 test is red by design.
    pub fn from_events(_path: &Path) -> Result<Session, ReconstructError> {
        Err(ReconstructError::NotImplemented(
            "reconstruct::from_events — M0 lane: parse events.jsonl, order by seq, build Session",
        ))
    }
}
