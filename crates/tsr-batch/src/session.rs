//! The recorded session: the engine's events, written through `tsr-store`, gated by `tsr-policy`.
//!
//! A batch run drives one engine task per corpus task. Every step the driver takes is recorded as
//! an [`Event`] appended to the store, exactly the way a record-only hook would: the hook observes
//! a tool call and writes the event; it does not decide anything. The gate is consulted *before* a
//! gated action is committed, and its verdict is recorded — an `allow` lets the action proceed, a
//! `deny` is recorded as a [`EventKind::Denied`] with the gate's own reason line, and the action
//! does not happen.
//!
//! ## The state version
//!
//! `tsr_cli::reconstruct::Session` checks that a gated action has a forecast at the state version
//! current when it was committed, where that version is the number of events before the commitment
//! in the session's log. [`SessionCtx::state_version`] is exactly that number, so a forecast
//! recorded now carries the version the commitment will be checked against.
//!
//! ## Invariants this module keeps
//!
//! - sequence is monotone and every parent precedes its child: each event's parents are the ids of
//!   events already appended, and the store refuses a parent that is not in the log;
//! - every gated action has a prior forecast at the same state version: [`SessionCtx::commit`]
//!   refuses to commit a gated candidate the gate denied for want of a forecast;
//! - every dispatch is reconciled or unresolved: [`SessionCtx::dispatch`] records a `Dispatched`,
//!   and the caller must follow it with [`SessionCtx::execute`] or [`SessionCtx::unresolved`].

use std::path::Path;

use tsr_domain::{
    ActionKind, Candidate, CandidateId, Event, EventId, EventKind, Forecast, Hash, Observation,
    OutcomeDefId, Probability, Source, Support, TaskId,
};
use tsr_policy::gate::{self, ActionArgs, Decision, GateInput, Grant, Verdict};
use tsr_policy::profile::Profile;
use tsr_store::{Store, StoreError};

/// Why a session step failed.
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("store: {0}")]
    Store(#[from] StoreError),
    #[error("contract: {0}")]
    Contract(#[from] tsr_domain::ContractError),
    #[error("a gated action was denied: {0}")]
    Denied(String),
    #[error("no candidate with id {0} has been proposed")]
    UnknownCandidate(String),
    #[error("a dispatch for {0} is already open")]
    DispatchAlreadyOpen(String),
    #[error("no dispatch is open for {0}")]
    NoOpenDispatch(String),
}

/// The recorded session for one engine task.
///
/// It owns the store and a mirror of the log in sequence order, so the gate can be evaluated
/// against the same events the store holds without a round trip.
pub struct SessionCtx {
    store: Store,
    task: TaskId,
    profile: Profile,
    grant: Grant,
    log: Vec<Event>,
    /// Candidates proposed but not yet committed, by id.
    proposed: std::collections::HashMap<CandidateId, Candidate>,
    /// Candidates with an open dispatch (dispatched, not yet executed or unresolved).
    open_dispatches: std::collections::HashSet<CandidateId>,
    /// The `recorded_at` stamp the next event carries. A batch run is deterministic, so the clock
    /// is an input, not ambient state.
    clock: String,
}

impl SessionCtx {
    /// Open a session over a store, for a task, under a profile and a grant.
    pub fn new(
        store: Store,
        task: TaskId,
        profile: Profile,
        grant: Grant,
        clock: impl Into<String>,
    ) -> Self {
        Self {
            store,
            task,
            profile,
            grant,
            log: Vec::new(),
            proposed: std::collections::HashMap::new(),
            open_dispatches: std::collections::HashSet::new(),
            clock: clock.into(),
        }
    }

    /// The task this session records.
    pub fn task(&self) -> TaskId {
        self.task
    }

    /// The number of events recorded so far. This is the state version a forecast is made against
    /// and the version a commitment is checked at.
    pub fn state_version(&self) -> u64 {
        self.log.len() as u64
    }

    /// The events recorded so far, in sequence order.
    pub fn events(&self) -> &[Event] {
        &self.log
    }

    /// The store, for a caller that needs to read the log back (the invariant check does).
    pub fn store(&self) -> &Store {
        &self.store
    }

    /// The profile this session gates against.
    pub fn profile(&self) -> &Profile {
        &self.profile
    }

    /// The grant this session runs under.
    pub fn grant(&self) -> &Grant {
        &self.grant
    }

    /// Append an event whose parents are the current head (or none, for the first event).
    fn append(&mut self, source: Source, kind: EventKind) -> Result<EventId, SessionError> {
        let parents = self.log.last().map(|e| vec![e.id]).unwrap_or_default();
        let ev = Event::new(self.task, parents, source, kind, self.clock.clone());
        self.store.append(&ev)?;
        let id = ev.id;
        self.log.push(ev);
        Ok(id)
    }

    /// Record `TaskOpened`. The first event of every session.
    pub fn open_task(&mut self) -> Result<EventId, SessionError> {
        self.append(Source::Rules, EventKind::TaskOpened { task: self.task })
    }

    /// Record an observation (evidence, a return, experience, or missing evidence).
    pub fn observe(&mut self, observation: Observation) -> Result<EventId, SessionError> {
        self.append(Source::Rules, EventKind::Observed { observation })
    }

    /// Propose a candidate. The candidate is remembered so a later commit can find its kind.
    pub fn propose(&mut self, candidate: Candidate) -> Result<EventId, SessionError> {
        self.proposed.insert(candidate.id, candidate.clone());
        self.append(Source::Rules, EventKind::Proposed { candidate })
    }

    /// Record a forecast for a candidate at the current state version.
    ///
    /// The forecast's `state_version` is forced to [`SessionCtx::state_version`] so a forecast
    /// recorded here is always the one the gate and the reconstruct invariant look for.
    pub fn forecast(
        &mut self,
        candidate: CandidateId,
        event: OutcomeDefId,
        raw: f64,
    ) -> Result<EventId, SessionError> {
        let forecast = Forecast {
            id: tsr_domain::ForecastId(Hash::of(&[
                candidate.to_string().as_bytes(),
                &(self.state_version() + 1).to_le_bytes(),
            ])),
            event,
            horizon: tsr_domain::Horizon::TaskClose,
            source: tsr_domain::Forecaster::Controller,
            raw: Probability::new(raw)?,
            calibrated: None,
            support: Support { episodes: 0 },
            state_version: self.state_version() + 1,
        };
        self.append(
            Source::Rules,
            EventKind::Forecast {
                forecast,
                candidate,
            },
        )
    }

    /// Evaluate the gate for a candidate and its typed arguments at the current state version.
    pub fn gate(&self, candidate: &Candidate, args: ActionArgs) -> Verdict {
        let input = GateInput {
            profile: &self.profile,
            grant: &self.grant,
            log: &self.log,
            candidate,
            state_version: self.state_version(),
            args,
        };
        gate::evaluate(&input)
    }

    /// Commit a candidate, gating it first.
    ///
    /// The gate is consulted with `args`; a `Deny` is recorded as a [`EventKind::Denied`] carrying
    /// the gate's reason and returned as [`SessionError::Denied`], so the caller can record the
    /// denial and move on. An `Allow` records the commitment. An `Ask` is treated as a denial for
    /// the batch runner's purposes (no operator is present to approve), and its reason is recorded.
    pub fn commit(
        &mut self,
        candidate: &Candidate,
        args: ActionArgs,
    ) -> Result<EventId, SessionError> {
        let verdict = self.gate(candidate, args);
        match verdict.decision {
            Decision::Allow => {
                let decision = tsr_domain::Decision::Selected {
                    value: candidate.id,
                    utility: tsr_domain::Utility {
                        p_success: 1.0,
                        value: 1.0,
                        cost: 0.0,
                        ev: 1.0,
                    },
                    propensity: 1.0,
                    evidence: Vec::new(),
                };
                self.append(
                    Source::Rules,
                    EventKind::Committed {
                        candidate: candidate.id,
                        decision,
                    },
                )
            }
            Decision::Deny | Decision::Ask => {
                self.append(
                    Source::Rules,
                    EventKind::Denied {
                        candidate: candidate.id,
                        reason: verdict.reason.clone(),
                    },
                )?;
                Err(SessionError::Denied(verdict.reason))
            }
        }
    }

    /// Commit a candidate without consulting the gate.
    ///
    /// This is system 1's path: the record-only hook records what it sees and gates nothing. It is
    /// deliberately separate from [`SessionCtx::commit`] so that a reader can see, at the call site,
    /// which system is running.
    pub fn commit_ungated(&mut self, candidate: &Candidate) -> Result<EventId, SessionError> {
        let decision = tsr_domain::Decision::Selected {
            value: candidate.id,
            utility: tsr_domain::Utility {
                p_success: 1.0,
                value: 1.0,
                cost: 0.0,
                ev: 1.0,
            },
            propensity: 1.0,
            evidence: Vec::new(),
        };
        self.append(
            Source::Rules,
            EventKind::Committed {
                candidate: candidate.id,
                decision,
            },
        )
    }

    /// Record a dispatch of a committed candidate. `in_sandbox` is always true: every command runs
    /// in the sealed sibling sandbox (B4).
    pub fn dispatch(&mut self, candidate: CandidateId) -> Result<EventId, SessionError> {
        if !self.open_dispatches.insert(candidate) {
            return Err(SessionError::DispatchAlreadyOpen(candidate.to_string()));
        }
        self.append(
            Source::Rules,
            EventKind::Dispatched {
                candidate,
                in_sandbox: true,
            },
        )
    }

    /// Reconcile an open dispatch with an execution result.
    pub fn execute(
        &mut self,
        candidate: CandidateId,
        exit_code: Option<i32>,
        log_digest: Option<Hash>,
        duration_ms: u64,
    ) -> Result<EventId, SessionError> {
        if !self.open_dispatches.remove(&candidate) {
            return Err(SessionError::NoOpenDispatch(candidate.to_string()));
        }
        self.append(
            Source::Rules,
            EventKind::Executed {
                candidate,
                exit_code,
                log_digest,
                duration_ms,
            },
        )
    }

    /// Reconcile an open dispatch as unresolved, with a reason.
    pub fn unresolved(
        &mut self,
        candidate: CandidateId,
        reason: impl Into<String>,
    ) -> Result<EventId, SessionError> {
        if !self.open_dispatches.remove(&candidate) {
            return Err(SessionError::NoOpenDispatch(candidate.to_string()));
        }
        self.append(
            Source::Rules,
            EventKind::Unresolved {
                candidate,
                reason: reason.into(),
            },
        )
    }

    /// Record a denial explicitly (used when the driver records a gate denial it already saw).
    pub fn record_denial(
        &mut self,
        candidate: CandidateId,
        reason: impl Into<String>,
    ) -> Result<EventId, SessionError> {
        self.append(
            Source::Rules,
            EventKind::Denied {
                candidate,
                reason: reason.into(),
            },
        )
    }

    /// Close the task with a reason.
    pub fn close(&mut self, reason: impl Into<String>) -> Result<EventId, SessionError> {
        self.append(
            Source::Rules,
            EventKind::TaskClosed {
                task: self.task,
                reason: reason.into(),
            },
        )
    }

    /// The kind of a proposed candidate, if it was proposed in this session.
    pub fn kind_of(&self, candidate: CandidateId) -> Option<ActionKind> {
        self.proposed.get(&candidate).map(|c| c.kind)
    }

    /// The candidate with this id, if it was proposed in this session.
    pub fn candidate(&self, candidate: CandidateId) -> Option<&Candidate> {
        self.proposed.get(&candidate)
    }

    /// The head root of the store: one hash for the whole log (B9, state isolation).
    pub fn head_root(&self) -> Result<Hash, SessionError> {
        Ok(self.store.head_root()?)
    }

    /// Open a session over a fresh on-disk store at `path`.
    pub fn open_store(path: &Path) -> Result<Store, StoreError> {
        Store::open(path)
    }
}
