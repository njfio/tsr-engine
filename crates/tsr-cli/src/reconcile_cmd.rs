//! `tsr task reconcile <task>` — outcome reconciliation (PRD §B4, §B6, §B11 M2 item 3).
//!
//! Intent is committed before the effect (`Dispatched`); completion is reconciled after
//! (`Executed` or an explicit `Unresolved`). A crash between the two leaves a dispatch with
//! neither, and this command is what closes that gap. It never guesses: it asks a
//! [`Reconciler`] — a tool-specific check — and records what the check says.
//!
//! The rules, from §B6 *Dispatch*:
//!
//! - an **idempotent** tool yields at most one observed effect across a crash; an unknown
//!   outcome is recorded as `Unresolved` and may be retried once, charged to the same budget;
//! - a **non-idempotent** tool with an unknown outcome stays blocked until a tool-specific
//!   check resolves it — this command records the `Unresolved` and does not retry.
//!
//! Nothing here writes a weight or a score. Every event it appends is a fact about what
//! happened, and the log is the only place it writes.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::ExitCode;

use tsr_domain::{
    CandidateId, Event, EventId, EventKind, ExecError, ExecOutcome, Hash, Source, TaskId,
};
use tsr_store::{Store, StoreError};

/// The reason recorded for an idempotent tool whose outcome no check could settle.
pub const UNKNOWN_IDEMPOTENT: &str =
    "unknown after crash; idempotent, retry charged to the same budget";

/// The reason recorded for a non-idempotent tool whose outcome no check could settle.
pub const UNKNOWN_NON_IDEMPOTENT: &str =
    "unknown after crash; non-idempotent, blocked until a tool-specific check resolves it";

/// The reason recorded when the operator cancels a dispatch.
pub const CANCELLED: &str = "cancelled";

/// What a tool-specific check found about a dispatch whose outcome the log does not hold.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Reconciled {
    /// The effect is known to have happened: the tool's own record of it exists.
    Executed {
        exit_code: Option<i32>,
        log_digest: Option<Hash>,
        duration_ms: u64,
    },
    /// The effect is known not to have happened.
    NotExecuted,
    /// The check cannot say. The caller decides what an unknown means for this tool.
    Unknown,
}

/// A tool-specific check for a dispatch with no recorded outcome.
///
/// Implementations answer from evidence the tool itself keeps — a log digest present in the
/// CAS, an idempotency key the tool has seen — never from a guess. `Unknown` is a first-class
/// answer: it is what makes the idempotent/non-idempotent split in §B6 meaningful.
pub trait Reconciler {
    /// Whether a retry of this dispatch is safe: the tool yields at most one effect.
    fn idempotent(&self, candidate: &CandidateId) -> bool;

    /// The tool-specific check. `intent` is the dispatch's idempotency key.
    fn check(&self, candidate: &CandidateId, intent: &str) -> Reconciled;
}

/// The first [`Reconciler`]: answers from what the tool left behind.
///
/// Two checks, both tool-specific and both cheap:
///
/// - **a log digest present in the CAS** — the tool wrote its log under `cas/<blake3>`; if the
///   digest is on disk the effect happened, and the digest is the evidence;
/// - **an idempotency key seen** — the tool's own record of the keys it has accepted; a key it
///   has seen means the effect happened even if the log digest is gone.
///
/// Anything else is `Unknown`. The CAS root is the store's sibling `cas/` directory, which is
/// where §B6 puts artifacts; a missing directory is not an error, it is simply no evidence.
pub struct ToolCheck {
    /// `cas/<blake3>` — artifacts (diffs, logs, feature views, bundles).
    cas_root: std::path::PathBuf,
    /// Idempotency keys the tool has seen, by candidate.
    seen_keys: BTreeMap<CandidateId, BTreeSet<String>>,
    /// Candidates whose tool is idempotent. Absent means non-idempotent: the safe default.
    idempotent: BTreeSet<CandidateId>,
    /// Log digests the tool is known to have produced, by candidate.
    log_digests: BTreeMap<CandidateId, Hash>,
}

impl ToolCheck {
    /// A check over a CAS directory, with no keys seen and every tool non-idempotent.
    pub fn new(cas_root: impl Into<std::path::PathBuf>) -> Self {
        Self {
            cas_root: cas_root.into(),
            seen_keys: BTreeMap::new(),
            idempotent: BTreeSet::new(),
            log_digests: BTreeMap::new(),
        }
    }

    /// The CAS root this check reads.
    pub fn cas_root(&self) -> &Path {
        &self.cas_root
    }

    /// Declare a candidate's tool idempotent: a retry yields at most one effect.
    pub fn with_idempotent(mut self, candidate: CandidateId) -> Self {
        self.idempotent.insert(candidate);
        self
    }

    /// Record that the tool has seen an idempotency key for this candidate.
    pub fn with_seen_key(mut self, candidate: CandidateId, key: impl Into<String>) -> Self {
        self.seen_keys
            .entry(candidate)
            .or_default()
            .insert(key.into());
        self
    }

    /// Record the log digest the tool produced for this candidate.
    pub fn with_log_digest(mut self, candidate: CandidateId, digest: Hash) -> Self {
        self.log_digests.insert(candidate, digest);
        self
    }

    /// Whether `cas/<digest>` exists.
    fn digest_in_cas(&self, digest: &Hash) -> bool {
        self.cas_root.join(digest.to_hex()).is_file()
    }
}

impl Reconciler for ToolCheck {
    fn idempotent(&self, candidate: &CandidateId) -> bool {
        self.idempotent.contains(candidate)
    }

    fn check(&self, candidate: &CandidateId, intent: &str) -> Reconciled {
        // The tool's own record of the key it accepted is the strongest evidence: it means the
        // effect happened even if the log has since been pruned.
        if self
            .seen_keys
            .get(candidate)
            .is_some_and(|keys| keys.contains(intent))
        {
            let log_digest = self.log_digests.get(candidate).copied();
            return Reconciled::Executed {
                exit_code: None,
                log_digest,
                duration_ms: 0,
            };
        }
        // A log digest present in the CAS is the effect's own artifact.
        if let Some(digest) = self.log_digests.get(candidate) {
            if self.digest_in_cas(digest) {
                return Reconciled::Executed {
                    exit_code: None,
                    log_digest: Some(*digest),
                    duration_ms: 0,
                };
            }
        }
        Reconciled::Unknown
    }
}

/// A dispatch the log holds with neither `Executed` nor `Unresolved`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingDispatch {
    pub candidate: CandidateId,
    /// The idempotency key the intent carried. The log does not store it, so it is derived from
    /// the candidate: one candidate is one intent, and a retry is the same intent.
    pub intent: String,
    /// The event that recorded the dispatch; the reconciliation's parent.
    pub dispatched: EventId,
}

/// The idempotency key of a dispatch. `DispatchIntent.idempotency_key` is not part of the
/// event, so the key is the candidate's own id: the same candidate dispatched twice is the
/// same intent, which is exactly what makes a retry safe to charge to the same budget.
pub fn intent_key(candidate: &CandidateId) -> String {
    candidate.to_string()
}

/// Every `Dispatched` candidate in `events` with neither `Executed` nor `Unresolved`.
///
/// A candidate dispatched more than once is one pending dispatch: the log's `Dispatched` events
/// are the intent, and the first `Executed` or `Unresolved` settles all of them.
pub fn pending_dispatches(events: &[(i64, Event)]) -> Vec<PendingDispatch> {
    let mut dispatched: BTreeMap<CandidateId, EventId> = BTreeMap::new();
    let mut settled: BTreeSet<CandidateId> = BTreeSet::new();
    for (_, e) in events {
        match &e.kind {
            EventKind::Dispatched { candidate, .. } => {
                dispatched.entry(*candidate).or_insert(e.id);
            }
            EventKind::Executed { candidate, .. } | EventKind::Unresolved { candidate, .. } => {
                settled.insert(*candidate);
            }
            _ => {}
        }
    }
    dispatched
        .into_iter()
        .filter(|(c, _)| !settled.contains(c))
        .map(|(candidate, dispatched)| PendingDispatch {
            intent: intent_key(&candidate),
            candidate,
            dispatched,
        })
        .collect()
}

/// What reconciling one dispatch appended.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Resolution {
    /// The check found the effect: an `Executed` was appended.
    Executed,
    /// The check could not say and the tool is idempotent: an `Unresolved` was appended and the
    /// dispatch may be retried once.
    UnresolvedIdempotent,
    /// The check could not say and the tool is not idempotent: an `Unresolved` was appended and
    /// the dispatch stays blocked.
    UnresolvedBlocked,
}

impl Resolution {
    /// The event kind this resolution appends.
    pub fn kind(&self) -> &'static str {
        match self {
            Resolution::Executed => "executed",
            Resolution::UnresolvedIdempotent | Resolution::UnresolvedBlocked => "unresolved",
        }
    }

    /// Whether the dispatch may be retried once, charged to the same budget.
    pub fn retryable(&self) -> bool {
        matches!(self, Resolution::UnresolvedIdempotent)
    }
}

/// Reconcile one pending dispatch through `reconciler`, appending exactly one event.
///
/// The event's parent is the `Dispatched` event, so the reconciliation is causally attached to
/// the intent it settles. `recorded_at` is provenance only; ordering is the store's sequence.
pub fn reconcile_one(
    store: &Store,
    task: &TaskId,
    pending: &PendingDispatch,
    reconciler: &dyn Reconciler,
    recorded_at: &str,
) -> Result<Resolution, StoreError> {
    let idempotent = reconciler.idempotent(&pending.candidate);
    let (kind, resolution) = match reconciler.check(&pending.candidate, &pending.intent) {
        Reconciled::Executed {
            exit_code,
            log_digest,
            duration_ms,
        } => (
            EventKind::Executed {
                candidate: pending.candidate,
                exit_code,
                log_digest,
                duration_ms,
            },
            Resolution::Executed,
        ),
        Reconciled::NotExecuted => (
            EventKind::Unresolved {
                candidate: pending.candidate,
                reason: if idempotent {
                    UNKNOWN_IDEMPOTENT.to_string()
                } else {
                    UNKNOWN_NON_IDEMPOTENT.to_string()
                },
            },
            if idempotent {
                Resolution::UnresolvedIdempotent
            } else {
                Resolution::UnresolvedBlocked
            },
        ),
        Reconciled::Unknown => (
            EventKind::Unresolved {
                candidate: pending.candidate,
                reason: if idempotent {
                    UNKNOWN_IDEMPOTENT.to_string()
                } else {
                    UNKNOWN_NON_IDEMPOTENT.to_string()
                },
            },
            if idempotent {
                Resolution::UnresolvedIdempotent
            } else {
                Resolution::UnresolvedBlocked
            },
        ),
    };
    let event = Event::new(
        *task,
        vec![pending.dispatched],
        Source::Rules,
        kind,
        recorded_at,
    );
    store.append(&event)?;
    Ok(resolution)
}

/// Reconcile every pending dispatch of a task. Returns one resolution per dispatch, in
/// candidate order.
pub fn reconcile_task(
    store: &Store,
    task: &TaskId,
    reconciler: &dyn Reconciler,
    recorded_at: &str,
) -> Result<Vec<(PendingDispatch, Resolution)>, StoreError> {
    let events = store.by_task(task)?;
    let mut out = Vec::new();
    for pending in pending_dispatches(&events) {
        let resolution = reconcile_one(store, task, &pending, reconciler, recorded_at)?;
        out.push((pending, resolution));
    }
    Ok(out)
}

/// Append `Unresolved { reason: "cancelled" }` for a dispatch the operator cancelled.
///
/// Cancellation is not a crash: the effect is known not to have happened, and the reason says
/// so. The caller appends `TaskClosed { reason: "operator" }` beside it.
pub fn record_cancellation(
    store: &Store,
    task: &TaskId,
    pending: &PendingDispatch,
    recorded_at: &str,
) -> Result<(), StoreError> {
    let event = Event::new(
        *task,
        vec![pending.dispatched],
        Source::Operator,
        EventKind::Unresolved {
            candidate: pending.candidate,
            reason: CANCELLED.to_string(),
        },
        recorded_at,
    );
    store.append(&event)?;
    Ok(())
}

/// The `Executor`-shaped reconciliation: ask the executor what happened to a handle.
///
/// This is the bridge between the trait in `tsr-domain` and the log. An executor that answers
/// `ExecOutcome::Unresolved` is treated exactly like a check that said `Unknown`.
pub fn from_exec_outcome(outcome: &ExecOutcome) -> Result<Reconciled, ExecError> {
    match outcome {
        ExecOutcome::Completed {
            exit_code,
            log_digest,
            duration_ms,
        } => Ok(Reconciled::Executed {
            exit_code: Some(*exit_code),
            log_digest: Some(*log_digest),
            duration_ms: *duration_ms,
        }),
        ExecOutcome::Failed { .. } => Ok(Reconciled::Executed {
            exit_code: None,
            log_digest: None,
            duration_ms: 0,
        }),
        ExecOutcome::Cancelled => Ok(Reconciled::NotExecuted),
        ExecOutcome::Unresolved { .. } => Ok(Reconciled::Unknown),
    }
}

/// Run `tsr task reconcile <task>`.
///
/// The store path is the task's log; the CAS root is its sibling `cas/` directory. Exit codes:
///
/// | code | meaning |
/// | --- | --- |
/// | 0 | every pending dispatch was reconciled |
/// | 1 | at least one dispatch is blocked (non-idempotent, unknown) |
/// | 2 | the task id or the store is malformed |
pub fn run(task_hex: &str, store_path: &str) -> ExitCode {
    let task = match Hash::from_hex(task_hex) {
        Ok(h) => TaskId(h),
        Err(e) => {
            eprintln!("tsr: invalid task id {task_hex:?}: {e}");
            return ExitCode::from(2);
        }
    };
    let store = match Store::open(Path::new(store_path)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tsr: cannot open store {store_path:?}: {e}");
            return ExitCode::from(2);
        }
    };
    let cas_root = Path::new(store_path)
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("cas");
    let reconciler = ToolCheck::new(cas_root);
    let recorded_at = now_iso8601();

    let resolutions = match reconcile_task(&store, &task, &reconciler, &recorded_at) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("tsr: reconcile failed: {e}");
            return ExitCode::from(2);
        }
    };

    let mut blocked = 0usize;
    for (pending, resolution) in &resolutions {
        println!(
            "{} {} {}",
            pending.candidate,
            resolution.kind(),
            if resolution.retryable() {
                "retryable"
            } else {
                "blocked"
            }
        );
        if !resolution.retryable() && *resolution != Resolution::Executed {
            blocked += 1;
        }
    }
    if resolutions.is_empty() {
        println!("no pending dispatches");
    }
    if blocked > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::from(0)
    }
}

/// Wall-clock provenance only; ordering is the store's sequence and causation (B6).
fn now_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}
