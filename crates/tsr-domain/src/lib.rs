//! # tsr-domain — the contract
//!
//! The typed facts, derived views and traits of the TSR Decision Engine, as specified in
//! `docs/PRD.md` §B6 (v0.3.1). This crate has no I/O, calls no model, and stores no weight.
//! Everything here is either a fact (content-addressed, append-only once recorded) or a pure
//! function of a snapshot.
//!
//! Terminology follows `docs/PRD.md` §B2 and Appendix A: every number derived from a TSR term is a
//! labeled engineering proxy ([`Proxy`]); no field is named after a TSR term unless B2 says so.
//!
//! Lineage is intrinsic: an id is the blake3 hash of a record's payload and its parents' ids, so
//! nothing can be altered or removed without renaming every descendant ([`Hash::of`]).

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::fmt;

// ---------------------------------------------------------------------------------------------
// Content addressing
// ---------------------------------------------------------------------------------------------

/// A content address: blake3 over length-prefixed parts (payload bytes, then each parent id).
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Hash(#[serde(with = "hex_bytes")] pub [u8; 32]);

impl Hash {
    /// Hash a record from its parts. Each part is length-prefixed so `["ab","c"]` ≠ `["a","bc"]`.
    pub fn of(parts: &[&[u8]]) -> Self {
        let mut h = blake3::Hasher::new();
        for p in parts {
            h.update(&(p.len() as u64).to_le_bytes());
            h.update(p);
        }
        Self(*h.finalize().as_bytes())
    }

    /// Hash a payload together with its parents, in order. This is how every id is minted.
    pub fn of_with_parents(payload: &[u8], parents: &[Hash]) -> Self {
        let mut parts: Vec<&[u8]> = Vec::with_capacity(1 + parents.len());
        parts.push(payload);
        for p in parents {
            parts.push(&p.0);
        }
        Self::of(&parts)
    }

    pub fn to_hex(&self) -> String {
        const DIGITS: &[u8; 16] = b"0123456789abcdef";
        let mut s = String::with_capacity(64);
        for b in self.0 {
            s.push(DIGITS[(b >> 4) as usize] as char);
            s.push(DIGITS[(b & 0x0f) as usize] as char);
        }
        s
    }

    pub fn from_hex(s: &str) -> Result<Self, ContractError> {
        let bytes = s.as_bytes();
        if bytes.len() != 64 {
            return Err(ContractError::Invalid("hash must be 64 hex characters"));
        }
        let nib = |c: u8| -> Result<u8, ContractError> {
            match c {
                b'0'..=b'9' => Ok(c - b'0'),
                b'a'..=b'f' => Ok(c - b'a' + 10),
                b'A'..=b'F' => Ok(c - b'A' + 10),
                _ => Err(ContractError::Invalid("hash has a non-hex character")),
            }
        };
        let mut out = [0u8; 32];
        for (i, pair) in bytes.chunks(2).enumerate() {
            out[i] = (nib(pair[0])? << 4) | nib(pair[1])?;
        }
        Ok(Self(out))
    }
}

impl fmt::Debug for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.to_hex()[..12])
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

mod hex_bytes {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8; 32], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&super::Hash(*bytes).to_hex())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 32], D::Error> {
        let s = String::deserialize(d)?;
        super::Hash::from_hex(&s)
            .map(|h| h.0)
            .map_err(serde::de::Error::custom)
    }
}

macro_rules! id_newtype {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Hash);

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}({:?})", stringify!($name), self.0)
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(&self.0, f)
            }
        }
    };
}

id_newtype!(/// Id of any event in the log. Events are occurrences; candidates are possibilities.
    EventId);
id_newtype!(/// Id of a candidate (a possibility). Becomes an occurrence only when committed and executed.
    CandidateId);
id_newtype!(/// Id of a preference in the Culmination ledger.
    PreferenceId);
id_newtype!(/// Id of a contrast record (desired minus actual, against open criteria).
    ContrastId);
id_newtype!(/// Id of a forecast.
    ForecastId);
id_newtype!(/// Id of a named criterion of a task.
    CriterionId);
id_newtype!(/// Id of a task.
    TaskId);
id_newtype!(/// Id of an outcome definition, versioned (`test_pass@v2`).
    OutcomeDefId);

// ---------------------------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------------------------

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ContractError {
    #[error("invalid value: {0}")]
    Invalid(&'static str),
    #[error("duplicate id in a set that requires uniqueness")]
    Duplicate,
    #[error("a candidate cannot conflict with itself")]
    SelfConflict,
    #[error("reference outside the stored snapshot")]
    OutsideSnapshot,
}

// ---------------------------------------------------------------------------------------------
// Proxies (Appendix A, rule 2)
// ---------------------------------------------------------------------------------------------

/// A number derived from a TSR term. Its `Display` prints the engineering name, never the term:
/// `ev=0.42`, `support=17`, `residual=-0.31`, `resonance=0.80`. No output ever reads
/// "Attraction = 0.73".
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Proxy<T> {
    pub name: String,
    pub value: T,
}

impl<T> Proxy<T> {
    pub fn new(name: &str, value: T) -> Self {
        Self {
            name: name.to_string(),
            value,
        }
    }
}

impl<T: fmt::Display> fmt::Display for Proxy<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}={}", self.name, self.value)
    }
}

// ---------------------------------------------------------------------------------------------
// Actions, sources, candidates (B5, B6)
// ---------------------------------------------------------------------------------------------

/// The six typed actions. `Wait` is a task state, not an action.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    Inspect,
    RunApprovedTest,
    Generate,
    ProposePatch,
    RequestApproval,
    Stop,
}

/// Where a candidate or a preference came from. Admission to the ledger is by source (B2.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    Generator,
    Council,
    Intuition,
    Operator,
    Rules,
}

/// A possibility. It becomes an occurrence only when committed and executed, and the two are
/// typed apart: an occurrence is an [`Event`], never a `Candidate`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Candidate {
    /// blake3 of (kind, args, parents).
    pub id: CandidateId,
    pub kind: ActionKind,
    pub source: Source,
    /// Digest of the schema-validated arguments; the body lives in the CAS.
    pub args: Hash,
    /// Causality: the events this candidate depends on.
    pub parents: Vec<EventId>,
    /// Exclusion: at most one of a conflict set is committed.
    pub conflicts_with: Vec<CandidateId>,
    /// Intuition only: the preferences this step serves (B8, the compose step).
    pub support: Vec<PreferenceId>,
    /// Per candidate, not per request (A2 §14).
    pub forecasts: Vec<Forecast>,
}

impl Candidate {
    /// Mint a candidate. The id commits to kind, args and parents, so lineage is intrinsic.
    pub fn new(
        kind: ActionKind,
        source: Source,
        args: Hash,
        parents: Vec<EventId>,
        conflicts_with: Vec<CandidateId>,
    ) -> Result<Self, ContractError> {
        let payload = serde_json::to_vec(&(kind, &args)).expect("serializable");
        let parent_hashes: Vec<Hash> = parents.iter().map(|p| p.0).collect();
        let id = CandidateId(Hash::of_with_parents(&payload, &parent_hashes));
        if conflicts_with.contains(&id) {
            return Err(ContractError::SelfConflict);
        }
        if has_duplicates(conflicts_with.iter()) {
            return Err(ContractError::Duplicate);
        }
        Ok(Self {
            id,
            kind,
            source,
            args,
            parents,
            conflicts_with,
            support: Vec::new(),
            forecasts: Vec::new(),
        })
    }
}

fn has_duplicates<I, T>(iter: I) -> bool
where
    I: Iterator<Item = T>,
    T: Ord + Clone,
{
    let mut v: Vec<T> = iter.collect();
    let n = v.len();
    v.sort();
    v.dedup();
    v.len() != n
}

// ---------------------------------------------------------------------------------------------
// Forecasts (B7)
// ---------------------------------------------------------------------------------------------

/// Who made a forecast. Three in v1, plus an optional external benchmark forecaster.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Forecaster {
    Generator,
    Controller,
    Operator,
    External(String),
}

/// A probability in `[0, 1]`, finite. Constructors reject anything else (B6).
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize, Deserialize)]
#[serde(try_from = "f64", into = "f64")]
pub struct Probability(f64);

impl Probability {
    pub fn new(p: f64) -> Result<Self, ContractError> {
        if !p.is_finite() || !(0.0..=1.0).contains(&p) {
            return Err(ContractError::Invalid(
                "probability must be finite and within [0, 1]",
            ));
        }
        Ok(Self(p))
    }
    pub fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for Probability {
    type Error = ContractError;
    fn try_from(p: f64) -> Result<Self, Self::Error> {
        Self::new(p)
    }
}

impl From<Probability> for f64 {
    fn from(p: Probability) -> f64 {
        p.0
    }
}

/// A calibrated probability carries the calibrator that produced it and the population it was
/// validated on. Absent a validated calibrator, there is no `Calibrated` — never a fabricated one.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Calibrated {
    pub p: Probability,
    pub calibrator: String,
    pub population: String,
}

/// Same-family episodes within feature range. Drives abstention: thin support means abstain or
/// explore (the Intuition–Expectation gap, B2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Support {
    pub episodes: u32,
}

/// The horizon over which a named event resolves.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Horizon {
    ThisDispatch,
    TaskClose,
    TaskCloseDays(u32),
}

/// A forecast about a named event, by one forecaster, recorded before the commitment it
/// belongs to. `raw` is always recorded; `calibrated` only under a validated calibrator (A2 §5).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Forecast {
    pub id: ForecastId,
    /// `task_accepted_in_budget@v1` is the common decision event; `test_pass`, `diff_accepted`
    /// are features and calibration diagnostics (B7, B8).
    pub event: OutcomeDefId,
    pub horizon: Horizon,
    pub source: Forecaster,
    pub raw: Probability,
    pub calibrated: Option<Calibrated>,
    pub support: Support,
    /// The snapshot version the forecast was made against. A gated action is denied until a
    /// forecast exists for the *current* version (B5, forecast-gated execution).
    pub state_version: u64,
}

impl Forecast {
    /// Record a forecast for a named event at a state version. `raw` is always recorded;
    /// `calibrated` stays `None` until a validated calibrator exists (A2 §5) — never fabricated.
    /// The id commits to the event, the horizon, the forecaster, the raw probability and the
    /// state version, so a forecast cannot be silently re-pointed at another version.
    pub fn new(
        event: OutcomeDefId,
        horizon: Horizon,
        source: Forecaster,
        raw: Probability,
        state_version: u64,
    ) -> Self {
        let payload = serde_json::to_vec(&(&event, &horizon, &source, &raw, state_version))
            .expect("serializable");
        Self {
            id: ForecastId(Hash::of(&[&payload])),
            event,
            horizon,
            source,
            raw,
            calibrated: None,
            support: Support { episodes: 0 },
            state_version,
        }
    }
}

// ---------------------------------------------------------------------------------------------
// The two records from one return (B2.1): Surprise stays physical; only Contrast expands.
// ---------------------------------------------------------------------------------------------

/// Forecast residual: what happened against what was expected. Updates Expectation. Never an
/// input to expansion.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Surprise {
    pub forecast: ForecastId,
    /// outcome (0 or 1) minus raw probability; in `[-1, 1]`.
    pub residual: f64,
}

/// How a criterion went unmet. The dimension of a contrast, so the derived preference can mean
/// something — a magnitude cannot say whether it concerns reliability, simplicity or speed.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnmetKind {
    WrongBehaviour,
    Regression,
    Cost,
    Rejected { reason: String },
}

/// Desired condition minus actual condition, against the task's named open criteria. The only
/// input to expansion. Evidence is referenced, never copied into a preference.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Contrast {
    pub id: ContrastId,
    pub from_return: EventId,
    pub unmet: Vec<(CriterionId, UnmetKind)>,
    pub evidence: Vec<EventId>,
}

// ---------------------------------------------------------------------------------------------
// The Culmination ledger (B2.1, B6, B8)
// ---------------------------------------------------------------------------------------------

/// Scope of a preference: the scale at which it was stated.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    Task(TaskId),
    Family(String),
    Repository(String),
    Operator,
}

/// Something wanted, as stated. Short and typed; never a transcript. Status lives in the active
/// view ([`PreferenceStatus`]); the node itself never changes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Preference {
    /// blake3 of (text, scope, parents).
    pub id: PreferenceId,
    pub scope: Scope,
    pub text: String,
    /// expanded-from: lineage.
    pub parents: Vec<PreferenceId>,
    /// Operator statements (including rejection reasons) enter directly; Generator statements
    /// wait for review (B2.1, admission by source).
    pub origin: Source,
}

impl Preference {
    pub fn new(
        text: impl Into<String>,
        scope: Scope,
        parents: Vec<PreferenceId>,
        origin: Source,
    ) -> Result<Self, ContractError> {
        let text = text.into();
        if text.trim().is_empty() {
            return Err(ContractError::Invalid("a preference needs text"));
        }
        if text.len() > 512 {
            return Err(ContractError::Invalid(
                "a preference is short and typed, never a transcript (max 512 bytes)",
            ));
        }
        let payload = serde_json::to_vec(&(&text, &scope)).expect("serializable");
        let parent_hashes: Vec<Hash> = parents.iter().map(|p| p.0).collect();
        let id = PreferenceId(Hash::of_with_parents(&payload, &parent_hashes));
        Ok(Self {
            id,
            scope,
            text,
            parents,
            origin,
        })
    }
}

/// Status in the active view. The structure beneath never changes; recurrence is evidence to
/// interpret, not authority (A2, review finding 4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PreferenceStatus {
    Open,
    Fulfilled,
    Superseded(PreferenceId),
    Withdrawn,
    Overlapping(PreferenceId),
    OutOfScope,
}

/// An event: contrast crossed as expanded wanting (the Filter's write rule).
///
/// The produced preference's content travels with the event so the ledger can rebuild the node
/// from the log alone: `produced` must equal `Preference::new(text, scope, parents, origin).id`,
/// and the ledger's write path refuses (exit 2) when it does not. The node itself is never stored
/// as a separate record — the log is the only store.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Expansion {
    pub from_contrast: ContrastId,
    pub produced: PreferenceId,
    /// The preferences that made this contrast matter. Vortex detection reads this (B9.1).
    pub catalyzed_by: Vec<PreferenceId>,
    /// The produced preference's text. `produced` is the content address of (text, scope, parents).
    pub text: String,
    /// The produced preference's scope.
    pub scope: Scope,
    /// The produced preference's expanded-from lineage.
    pub parents: Vec<PreferenceId>,
    /// The produced preference's origin: Operator enters directly, Generator waits for review.
    pub origin: Source,
}

/// Where a selected preference strains against the present (the tensions step of Intuition).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tension {
    pub preference: PreferenceId,
    pub against: TensionWith,
    pub note: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TensionWith {
    Belief(CandidateId),
    Expectation(ForecastId),
}

/// A self-reinforcing cluster of open preferences. First build: a resonance community with
/// measured recurrence; the autocatalytic-set model is gated (B9.1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Vortex {
    pub members: Vec<PreferenceId>,
    pub resonance: Proxy<f64>,
    pub recurrence: Proxy<u32>,
}

// ---------------------------------------------------------------------------------------------
// Decisions (B8)
// ---------------------------------------------------------------------------------------------

/// Decomposed so a ranking explains itself. `p_success` is always
/// P(task_accepted_in_budget | state, action) — one common event for every candidate.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Utility {
    pub p_success: f64,
    pub value: f64,
    pub cost: f64,
    pub ev: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AbstainReason {
    NoEligibleCandidate,
    ThinSupport { episodes: u32, floor: u32 },
    NearTie { margin: f64 },
    MissingEvidence,
    OutOfDistribution,
    InvalidProviderOutput,
    HighValueLowConfidence,
}

/// The safe next step an abstention names (FR 004).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SafeNext {
    Inspect,
    RequestApproval,
    Stop,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision<T> {
    Selected {
        value: T,
        utility: Utility,
        /// Probability the policy selected this candidate; 1.0 for a deterministic choice, less
        /// under logged ε-exploration. Never unknown (A2 §6).
        propensity: f64,
        evidence: Vec<EventId>,
    },
    Abstain {
        reason: AbstainReason,
        next: SafeNext,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum DecisionError {
    #[error("stale state version: expected {expected}, found {found}")]
    StateConflict { expected: u64, found: u64 },
    #[error("contract violation: {0}")]
    Contract(#[from] ContractError),
}

// ---------------------------------------------------------------------------------------------
// Tune and Focus: operator settings, never scalars of the terms (B2)
// ---------------------------------------------------------------------------------------------

/// Receptivity settings. Sits at C² × M.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Tune {
    pub admitted_sources: Vec<Source>,
    pub candidate_kinds: Vec<ActionKind>,
    /// Horizon of the flow fold over the ledger's history.
    pub flow_horizon: u32,
    /// At most this many Intuition candidates per snapshot (default 3).
    pub intuition_max: u8,
    /// Probability that Intuition's seed is the leading-edge frontier or a low-mass cluster.
    pub quiet_route: f64,
    /// ε among near-ties; the only randomness in the system, logged with its propensity.
    pub epsilon: f64,
    pub support_floor: u32,
}

impl Default for Tune {
    fn default() -> Self {
        Self {
            admitted_sources: vec![
                Source::Rules,
                Source::Generator,
                Source::Council,
                Source::Operator,
            ],
            candidate_kinds: vec![
                ActionKind::Inspect,
                ActionKind::RunApprovedTest,
                ActionKind::Generate,
                ActionKind::ProposePatch,
                ActionKind::RequestApproval,
                ActionKind::Stop,
            ],
            flow_horizon: 200,
            intuition_max: 3,
            quiet_route: 0.0,
            epsilon: 0.0,
            support_floor: 30,
        }
    }
}

/// Commitment settings. Sits at M × C².
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Focus {
    pub budget: Budget,
    pub commit_margin: f64,
    /// Precision weighting on Expectation (predictive updating, B2.1).
    pub precision: f64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Budget {
    pub generator_tokens: u64,
    pub tool_minutes: u32,
    pub wall_clock_secs: u64,
    pub generator_calls: u32,
    pub patch_bytes: u64,
}

// ---------------------------------------------------------------------------------------------
// Snapshot, observations, events
// ---------------------------------------------------------------------------------------------

/// A configuration: a consistent, causally closed set of occurred events, materialized at a
/// version. Awareness, including explicit missing-evidence facts.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub task: TaskId,
    pub version: u64,
    /// One hash for the whole state at this decision: pinned beside the forecast, and the thing
    /// an evaluation fork starts from (B9, state isolation).
    pub root: Hash,
    pub open_criteria: Vec<CriterionId>,
    pub missing_evidence: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Observation {
    Evidence {
        digest: Hash,
        source: String,
    },
    Return {
        surprise: Vec<Surprise>,
        contrast: Option<Contrast>,
    },
    /// Operator-reported experience: evidence about usefulness and consequences that tests miss.
    /// Never reward truth; never presented as anything the system feels.
    Experience {
        note: String,
    },
    MissingEvidence {
        what: String,
    },
}

/// The kinds of event the log holds. Candidates recorded are possibilities; only `Committed` and
/// `Executed` are occurrences in the event-structure sense (B6).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EventKind {
    TaskOpened {
        task: TaskId,
    },
    Observed {
        observation: Observation,
    },
    Proposed {
        candidate: Candidate,
    },
    /// The model's own turn (PRD B4 line 218: "the model's own turn is Generate").
    Generate {
        candidate: CandidateId,
    },
    Forecast {
        forecast: Forecast,
        candidate: CandidateId,
    },
    Committed {
        candidate: CandidateId,
        decision: Decision<CandidateId>,
    },
    Dispatched {
        candidate: CandidateId,
        in_sandbox: bool,
    },
    Executed {
        candidate: CandidateId,
        exit_code: Option<i32>,
        log_digest: Option<Hash>,
        duration_ms: u64,
    },
    Unresolved {
        candidate: CandidateId,
        reason: String,
    },
    Expanded {
        expansion: Expansion,
    },
    StatusSet {
        preference: PreferenceId,
        status: PreferenceStatus,
        by: Source,
    },
    /// A label on a return (B5, B7). The record carries its own provenance; a correction appends
    /// a superseding record rather than rewriting the one it replaces.
    OutcomeRecorded {
        record: OutcomeRecord,
    },
    Approved {
        candidate: CandidateId,
        scope: String,
        expires_at: String,
    },
    Denied {
        candidate: CandidateId,
        reason: String,
    },
    TaskClosed {
        task: TaskId,
        reason: String,
    },
    /// The Stop stage of the hook (PRD B4 line 218: "the Stop hook is Stop"). The reason is the
    /// typed `Stop` reason of B5 (`done`, `budget`, `blocked`, `unsafe`, `operator`); the hook
    /// records it and never blocks.
    Stop {
        reason: String,
    },
}

/// An event in the append-only log. `id` = blake3(payload, parents).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub schema_version: u32,
    pub task: TaskId,
    pub parents: Vec<EventId>,
    /// Wall-clock provenance only; ordering is the store's sequence and causation (B6).
    pub recorded_at: String,
    pub observed_at: Option<String>,
    pub source: Source,
    pub kind: EventKind,
}

impl Event {
    pub const SCHEMA_VERSION: u32 = 1;

    pub fn new(
        task: TaskId,
        parents: Vec<EventId>,
        source: Source,
        kind: EventKind,
        recorded_at: impl Into<String>,
    ) -> Self {
        let recorded_at = recorded_at.into();
        let payload =
            serde_json::to_vec(&(Self::SCHEMA_VERSION, &task, &source, &kind, &recorded_at))
                .expect("serializable");
        let parent_hashes: Vec<Hash> = parents.iter().map(|p| p.0).collect();
        let id = EventId(Hash::of_with_parents(&payload, &parent_hashes));
        Self {
            id,
            schema_version: Self::SCHEMA_VERSION,
            task,
            parents,
            recorded_at,
            observed_at: None,
            source,
            kind,
        }
    }

    /// The state version this event is recorded against: the number of events in the task
    /// before it (B5: "a forecast for the current state version").
    pub fn state_version(&self, seq: i64) -> u64 {
        (seq - 1).max(0) as u64
    }
}

// ---------------------------------------------------------------------------------------------
// Outcome definitions (B7)
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutcomeDef {
    pub id: OutcomeDefId,
    pub name: String,
    pub version: u32,
    pub horizon: Horizon,
    pub resolves_true_when: String,
    pub censored_when: String,
    pub intervention_rule: String,
}

// ---------------------------------------------------------------------------------------------
// Outcome records (B5, B7): the label a return awaits, and its provenance
// ---------------------------------------------------------------------------------------------

/// Where an outcome record stands. `pending` is the state a return is in before review; the rest
/// are the labels a reviewer can give it (B5, B7). `disputed` is a label, not a status of the
/// tool: an operator-modified diff resolves as `disputed` (B7, `diff_accepted`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LabelStatus {
    /// Awaiting a label.
    Pending,
    /// The reviewer accepted the return.
    Observed,
    /// The return was censored: it never resolved (timeout, cancellation, never reviewed).
    Censored,
    /// The evidence the return needed is missing.
    Missing,
    /// The reviewer disputed the return.
    Disputed,
    /// The reviewer rejected the return: it was green but the reviewer refused it (B7).
    Rejected,
    /// The return is invalid: it cannot be scored.
    Invalid,
}

impl LabelStatus {
    /// The engineering name, for the CLI and for the log's own tags.
    pub fn name(self) -> &'static str {
        match self {
            LabelStatus::Pending => "pending",
            LabelStatus::Observed => "observed",
            LabelStatus::Censored => "censored",
            LabelStatus::Missing => "missing",
            LabelStatus::Disputed => "disputed",
            LabelStatus::Rejected => "rejected",
            LabelStatus::Invalid => "invalid",
        }
    }

    /// Parse the CLI's spelling of a status.
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "pending" => LabelStatus::Pending,
            "observed" => LabelStatus::Observed,
            "censored" => LabelStatus::Censored,
            "missing" => LabelStatus::Missing,
            "disputed" => LabelStatus::Disputed,
            "rejected" => LabelStatus::Rejected,
            "invalid" => LabelStatus::Invalid,
            _ => return None,
        })
    }
}

/// The label a return carries, with its own provenance (B5, B7).
///
/// An outcome record is separate from the decision record: it names the outcome definition it
/// resolves, the status, the reviewer, the observed and recorded times, and the evidence digest.
/// A correction appends a superseding record — `supersedes` names the record it replaces — and
/// nothing is ever rewritten. The rejection reason is stored apart from tool results: it lives in
/// `reason`, never in the evidence digest.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct OutcomeRecord {
    /// The outcome definition this record resolves (`diff_accepted@v1`, ...).
    pub outcome: OutcomeDefId,
    /// The event the record is about: the return it labels.
    pub subject: EventId,
    pub status: LabelStatus,
    /// The reviewer: `Source::Operator` for a human label.
    pub reviewer: Source,
    /// The operator's reason, when the label is `rejected`/`disputed`. Stored apart from tool
    /// results; it enters the ledger directly as an Operator preference (B2.1, B7).
    pub reason: Option<String>,
    /// The evidence digest the label rests on, when there is one.
    pub evidence: Option<Hash>,
    /// The record this one supersedes, when it is a correction. Never a rewrite.
    pub supersedes: Option<EventId>,
    /// The inverse-probability weight when the return was sampled for review (B7). `None` when
    /// the return was reviewed unconditionally (every failure, every green on the eval set).
    pub weight: Option<f64>,
}

// ---------------------------------------------------------------------------------------------
// Execution (B4, B6)
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DispatchIntent {
    pub candidate: CandidateId,
    pub idempotency_key: String,
    pub expected_state_version: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DispatchHandle {
    pub candidate: CandidateId,
    pub attempt: u32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecOutcome {
    Completed {
        exit_code: i32,
        log_digest: Hash,
        duration_ms: u64,
    },
    Failed {
        reason: String,
    },
    Cancelled,
    Unresolved {
        reason: String,
    },
}

#[derive(Debug, thiserror::Error)]
pub enum ExecError {
    #[error("no sealed sandbox available; refusing to run unsandboxed")]
    NoSandbox,
    #[error("policy denied: {0}")]
    Denied(String),
    #[error("execution failed: {0}")]
    Other(String),
}

/// Something that happened in execution, as the hook or the executor reports it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ExecEvent {
    pub candidate: CandidateId,
    pub outcome: ExecOutcome,
}

// ---------------------------------------------------------------------------------------------
// Traits
// ---------------------------------------------------------------------------------------------

/// A domain plugs in intents, candidates, outcome definitions and observers. Coding is the first.
pub trait Domain {
    fn candidates(&self, snap: &Snapshot) -> Vec<Candidate>;
    fn outcome_defs(&self) -> &[OutcomeDef];
    /// Yields the Surprise and Contrast records from a return (B2.1): two records, kept apart.
    fn observe(&self, ev: &ExecEvent) -> Vec<Observation>;
}

/// The generator's council: composes bounded next steps from preferences and tensions. The
/// engine selects, analyses and bounds; the council composes (B8, the Intuition mechanism).
pub trait Council {
    fn compose(&self, prefs: &[Preference], tensions: &[Tension], max: u8) -> Vec<Candidate>;
}

/// The Culmination ledger as views over the event structure. Two implementations behind one
/// trait — resonance selection first, the autocatalytic model second, gated (B9.1). Nothing in
/// an implementation may store a weight: every strength is a fold over the hashed history.
pub trait Culmination {
    /// Append only.
    fn expand(&mut self, e: Expansion) -> PreferenceId;
    /// Active view; the structure never changes.
    fn set_status(&mut self, p: PreferenceId, s: PreferenceStatus, by: Source);
    /// Resonance, plus the quiet route. A fold, nothing stored.
    fn select(&self, snap: &Snapshot, tune: &Tune) -> Vec<PreferenceId>;
    fn tensions(&self, prefs: &[PreferenceId], snap: &Snapshot) -> Vec<Tension>;
    /// `source: Intuition`, at most `tune.intuition_max`, each with `support`.
    fn compose(
        &self,
        prefs: &[PreferenceId],
        t: &[Tension],
        council: &dyn Council,
        tune: &Tune,
    ) -> Vec<Candidate>;
}

/// Pure function of a snapshot, the candidates and the operator's settings.
pub trait DecisionEngine {
    fn decide(
        &self,
        snap: &Snapshot,
        cands: &[Candidate],
        tune: &Tune,
        focus: &Focus,
    ) -> Result<Decision<CandidateId>, DecisionError>;
}

/// Path A: `tsr exec` in the sealed sibling sandbox. Containment and reconciliation live here,
/// not in the hook (B4).
pub trait Executor {
    fn dispatch(&mut self, intent: &DispatchIntent) -> Result<DispatchHandle, ExecError>;
    fn reconcile(&mut self, h: &DispatchHandle) -> Result<ExecOutcome, ExecError>;
}

// ---------------------------------------------------------------------------------------------
// Tests: the contract's own invariants
// ---------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn h(s: &str) -> Hash {
        Hash::of(&[s.as_bytes()])
    }

    fn tid() -> TaskId {
        TaskId(h("task-1"))
    }

    #[test]
    fn hash_is_deterministic_and_length_prefixed() {
        assert_ne!(
            Hash::of(&[b"ab".as_slice(), b"c".as_slice()]),
            Hash::of(&[b"a".as_slice(), b"bc".as_slice()])
        );
        assert_eq!(h("x"), h("x"));
        let hx = h("x");
        assert_eq!(Hash::from_hex(&hx.to_hex()).unwrap(), hx);
    }

    #[test]
    fn probability_rejects_nonfinite_and_out_of_range() {
        assert!(Probability::new(f64::NAN).is_err());
        assert!(Probability::new(1.5).is_err());
        assert!(Probability::new(-0.1).is_err());
        assert!(Probability::new(0.0).is_ok());
        assert!(Probability::new(1.0).is_ok());
        let json = serde_json::to_string(&Probability::new(0.25).unwrap()).unwrap();
        assert_eq!(json, "0.25");
        assert!(serde_json::from_str::<Probability>("2.0").is_err());
    }

    #[test]
    fn candidate_id_commits_to_parents_and_rejects_self_conflict() {
        let p1 = EventId(h("e1"));
        let a = Candidate::new(
            ActionKind::Inspect,
            Source::Rules,
            h("args"),
            vec![p1],
            vec![],
        )
        .unwrap();
        let b = Candidate::new(
            ActionKind::Inspect,
            Source::Rules,
            h("args"),
            vec![],
            vec![],
        )
        .unwrap();
        assert_ne!(a.id, b.id, "different parents, different identity");
        let c = Candidate::new(
            ActionKind::Inspect,
            Source::Rules,
            h("args"),
            vec![p1],
            vec![a.id],
        );
        assert_eq!(c.unwrap_err(), ContractError::SelfConflict);
        let dup = Candidate::new(
            ActionKind::Stop,
            Source::Rules,
            h("s"),
            vec![],
            vec![a.id, a.id],
        );
        assert_eq!(dup.unwrap_err(), ContractError::Duplicate);
    }

    #[test]
    fn preference_is_short_and_typed() {
        assert!(Preference::new("   ", Scope::Operator, vec![], Source::Operator).is_err());
        let long = "x".repeat(513);
        assert!(Preference::new(long, Scope::Operator, vec![], Source::Operator).is_err());
        let p = Preference::new(
            "existing behaviour unchanged",
            Scope::Family("parser".into()),
            vec![],
            Source::Operator,
        )
        .unwrap();
        let q = Preference::new(
            "existing behaviour unchanged",
            Scope::Family("parser".into()),
            vec![p.id],
            Source::Operator,
        )
        .unwrap();
        assert_ne!(p.id, q.id, "an expansion with lineage is a new node");
    }

    #[test]
    fn event_id_changes_with_parents_and_round_trips_json() {
        let e1 = Event::new(
            tid(),
            vec![],
            Source::Rules,
            EventKind::TaskOpened { task: tid() },
            "2026-10-11T00:00:00Z",
        );
        let e2 = Event::new(
            tid(),
            vec![e1.id],
            Source::Rules,
            EventKind::TaskOpened { task: tid() },
            "2026-10-11T00:00:00Z",
        );
        assert_ne!(e1.id, e2.id);
        let json = serde_json::to_string(&e2).unwrap();
        let back: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(back, e2);
    }

    #[test]
    fn generate_kind_round_trips_and_state_version_counts_prior_events() {
        let cid = CandidateId(h("candidate-1"));
        let kind = EventKind::Generate { candidate: cid };
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(
            json,
            format!(
                "{{\"kind\":\"generate\",\"candidate\":\"{}\"}}",
                cid.0.to_hex()
            )
        );
        let back: EventKind = serde_json::from_str(&json).unwrap();
        assert_eq!(back, kind);

        let e = Event::new(
            tid(),
            vec![],
            Source::Generator,
            EventKind::Generate { candidate: cid },
            "2026-10-11T00:00:00Z",
        );
        assert_eq!(e.state_version(1), 0, "the first event is state version 0");
        assert_eq!(e.state_version(5), 4, "four events precede the fifth");
    }

    #[test]
    fn stop_kind_round_trips() {
        let kind = EventKind::Stop {
            reason: "done".into(),
        };
        let json = serde_json::to_string(&kind).unwrap();
        assert_eq!(json, "{\"kind\":\"stop\",\"reason\":\"done\"}");
        let back: EventKind = serde_json::from_str(&json).unwrap();
        assert_eq!(back, kind);

        let e = Event::new(
            tid(),
            vec![],
            Source::Rules,
            EventKind::Stop {
                reason: "done".into(),
            },
            "2026-10-11T00:00:00Z",
        );
        let json = serde_json::to_string(&e).unwrap();
        let back: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(back, e);
    }

    #[test]
    fn forecast_new_pins_the_state_version_and_round_trips() {
        let f = Forecast::new(
            OutcomeDefId(h("task_accepted_in_budget@v1")),
            Horizon::TaskClose,
            Forecaster::Generator,
            Probability::new(0.7).unwrap(),
            4,
        );
        assert_eq!(f.state_version, 4);
        assert_eq!(f.raw.get(), 0.7);
        assert!(f.calibrated.is_none(), "no calibrator, no Calibrated");
        assert_eq!(f.support.episodes, 0);

        let json = serde_json::to_string(&f).unwrap();
        let back: Forecast = serde_json::from_str(&json).unwrap();
        assert_eq!(back, f);

        // The id commits to the state version: the same forecast at another version is another id.
        let g = Forecast::new(
            OutcomeDefId(h("task_accepted_in_budget@v1")),
            Horizon::TaskClose,
            Forecaster::Generator,
            Probability::new(0.7).unwrap(),
            5,
        );
        assert_ne!(f.id, g.id);
    }

    #[test]
    fn proxy_prints_the_engineering_name_never_the_term() {
        let ev = Proxy::new("ev", 0.42);
        assert_eq!(ev.to_string(), "ev=0.42");
    }
}
