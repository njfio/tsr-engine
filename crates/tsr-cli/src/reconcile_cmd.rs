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
use std::process::ExitCode;

use tsr_domain::{
    ActionKind, CandidateId, Event, EventId, EventKind, ExecError, ExecOutcome, Hash, Source,
    TaskId,
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
/// Implementations answer from evidence the log itself holds — the `Executed` that follows the
/// dispatch — never from a guess. `Unknown` is a first-class answer: it is what makes the
/// idempotent/non-idempotent split in §B6 meaningful.
pub trait Reconciler {
    /// Whether a retry of this dispatch is safe: the tool yields at most one effect.
    fn idempotent(&self, candidate: &CandidateId) -> bool;

    /// The tool-specific check. `intent` is the dispatch's idempotency key.
    fn check(&self, candidate: &CandidateId, intent: &str) -> Reconciled;
}

/// The first [`Reconciler`]: answers from what the log records.
///
/// One check, tool-specific and cheap: **an `Executed` that follows the dispatch** — the log's own
/// record that the effect happened. The `Executed`'s `log_digest` is the artifact the tool produced
/// for that dispatch, and the candidate's idempotency key is one the tool has seen.
///
/// Anything else is `Unknown`. The CAS is **not** consulted: main's `exec` hashes the tool log but
/// never writes it to `cas/`, so no artifact on disk can be linked to a dispatch without an
/// `Executed` (D1, N3 — see `docs/m2-evidence.md`). A check that keeps no artifacts cannot confirm
/// one, and an unconfirmed digest is not evidence.
pub struct ToolCheck {
    /// Candidates whose tool is idempotent. Absent means non-idempotent: the safe default.
    idempotent: BTreeSet<CandidateId>,
}

impl ToolCheck {
    /// A check with every tool non-idempotent.
    pub fn new() -> Self {
        Self {
            idempotent: BTreeSet::new(),
        }
    }

    /// Declare a candidate's tool idempotent: a retry yields at most one effect.
    pub fn with_idempotent(mut self, candidate: CandidateId) -> Self {
        self.idempotent.insert(candidate);
        self
    }

    /// Build a check from the log's own evidence (D1, R4).
    ///
    /// The log is the tool's record of what it did, so it is where the check's evidence comes
    /// from — never a guess:
    ///
    /// - a `Proposed` candidate whose action kind is idempotent (`RunApprovedTest`) marks the
    ///   candidate idempotent: a retry yields at most one effect.
    ///
    /// A dispatch is *pending* only when no `Executed` follows it ([`pending_dispatches`]), so the
    /// log holds no `Executed` to name a digest for it: the check answers `Unknown` and the caller
    /// records the `Unresolved`. The CAS is not consulted (D1, N3): main's `exec` never writes the
    /// tool log to `cas/`, so a digest on disk cannot be linked to a dispatch without an `Executed`.
    pub fn from_log(events: &[(i64, Event)]) -> Self {
        let mut check = Self::new();
        for (_, e) in events {
            if let EventKind::Proposed { candidate } = &e.kind {
                if candidate.kind == ActionKind::RunApprovedTest {
                    check.idempotent.insert(candidate.id);
                }
            }
        }
        check
    }
}

impl Default for ToolCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl Reconciler for ToolCheck {
    fn idempotent(&self, candidate: &CandidateId) -> bool {
        self.idempotent.contains(candidate)
    }

    fn check(&self, _candidate: &CandidateId, _intent: &str) -> Reconciled {
        // A pending dispatch has no `Executed` following it, so the log holds no evidence to
        // settle it: the check cannot say. The CAS is not consulted (D1, N3).
        Reconciled::Unknown
    }
}

/// A dispatch the log holds with neither `Executed` nor a settling `Unresolved`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingDispatch {
    pub candidate: CandidateId,
    /// The idempotency key the intent carried. The log does not store it, so it is derived from
    /// the candidate: one candidate is one intent, and a retry is the same intent.
    pub intent: String,
    /// The event that recorded the dispatch; the reconciliation's parent.
    pub dispatched: EventId,
    /// Whether an idempotent unknown was already recorded for this dispatch after its last
    /// `Dispatched`. §B6 allows one retry; a second is refused.
    pub retried: bool,
}

/// The idempotency key of a dispatch. `DispatchIntent.idempotency_key` is not part of the
/// event, so the key is the candidate's own id: the same candidate dispatched twice is the
/// same intent, which is exactly what makes a retry safe to charge to the same budget.
pub fn intent_key(candidate: &CandidateId) -> String {
    candidate.to_string()
}

/// Every `Dispatched` candidate in `events` whose outcome the log does not settle.
///
/// A candidate dispatched more than once is one pending dispatch: the log's `Dispatched` events
/// are the intent, and a settling event *after the last* `Dispatched` closes it. What settles a
/// dispatch (§B6):
///
/// - an `Executed` — the effect is known to have happened;
/// - an `Unresolved { reason: "cancelled" }` — the operator cancelled it.
///
/// An `Unresolved` recording an *unknown* outcome does **not** settle the dispatch: an idempotent
/// unknown may be retried once (and a second retry is refused by [`reconcile_one`]), and a
/// non-idempotent unknown stays blocked until a tool-specific check resolves it. Both are
/// re-checked on every reconcile.
pub fn pending_dispatches(events: &[(i64, Event)]) -> Vec<PendingDispatch> {
    // The last `Dispatched` seq per candidate, and whether an idempotent unknown was recorded
    // after it.
    let mut dispatched: BTreeMap<CandidateId, (i64, EventId)> = BTreeMap::new();
    let mut settled: BTreeSet<CandidateId> = BTreeSet::new();
    let mut retried: BTreeSet<CandidateId> = BTreeSet::new();
    for (seq, e) in events {
        match &e.kind {
            EventKind::Dispatched { candidate, .. } => {
                // A later dispatch re-opens the candidate: a re-dispatch is pending until its own
                // `Executed` is found. The retry budget is **not** reset (D1, R4): "retry once"
                // counts automatic retries per candidate across all its dispatches, so a second
                // retry of the same candidate refuses.
                dispatched.insert(*candidate, (*seq, e.id));
                settled.remove(candidate);
            }
            EventKind::Executed { candidate, .. } => {
                settled.insert(*candidate);
            }
            EventKind::Unresolved { candidate, reason } => {
                if reason == CANCELLED {
                    settled.insert(*candidate);
                } else if reason == UNKNOWN_IDEMPOTENT {
                    retried.insert(*candidate);
                }
                // An unknown non-idempotent outcome leaves the dispatch pending: it is re-checked.
            }
            _ => {}
        }
    }
    dispatched
        .into_iter()
        .filter(|(c, _)| !settled.contains(c))
        .map(|(candidate, (_, dispatched))| PendingDispatch {
            intent: intent_key(&candidate),
            candidate,
            dispatched,
            retried: retried.contains(&candidate),
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
    /// The check could not say and the idempotent dispatch was already retried once: §B6 allows
    /// one retry, so this one is refused and nothing is appended.
    Refused,
}

impl Resolution {
    /// The event kind this resolution appends.
    pub fn kind(&self) -> &'static str {
        match self {
            Resolution::Executed => "executed",
            Resolution::UnresolvedIdempotent | Resolution::UnresolvedBlocked => "unresolved",
            Resolution::Refused => "refused",
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
    // D1, R4: the evidence is the dispatch's own. An `Executed` that follows this `Dispatched`
    // settles it; a re-dispatch has no such `Executed` yet, so it stays pending — never settled by
    // an earlier dispatch's effect. The CAS is not consulted (D1, N3): main's `exec` never writes
    // the tool log to `cas/`, so no artifact on disk can be linked to a dispatch without an
    // `Executed`.
    let reconciled = reconciler.check(&pending.candidate, &pending.intent);
    let (kind, resolution) = match reconciled {
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
        Reconciled::NotExecuted | Reconciled::Unknown => {
            // §B6: an idempotent unknown may be retried once. If the log already holds that one
            // retry, a second is refused — nothing is appended.
            if idempotent && pending.retried {
                return Ok(Resolution::Refused);
            }
            (
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
            )
        }
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
/// The store path is the task's log; reconcile reads only the log and never the CAS (D1, N3).
/// Exit codes:
///
/// | code | meaning |
/// | --- | --- |
/// | 0 | every pending dispatch was reconciled |
/// | 1 | at least one dispatch is blocked (non-idempotent, unknown) |
/// | 2 | the task id or the store is malformed, or a second retry of an idempotent unknown was refused |
pub fn run(task_hex: &str, store_path: Option<&str>) -> ExitCode {
    let task = match Hash::from_hex(task_hex) {
        Ok(h) => TaskId(h),
        Err(e) => {
            eprintln!("tsr: invalid task id {task_hex:?}: {e}");
            return ExitCode::from(2);
        }
    };
    // R5: the store is the one the M0 commands use — `--store` when given, else the shared
    // resolver (`TSR_DB`, else `.tsr/tsr.db`). Reconcile only reads the log, so a store that does
    // not exist is an error naming the path, never a store created here.
    let path = match store_path {
        Some(p) if !p.is_empty() => std::path::PathBuf::from(p),
        _ => crate::store_path::db_path(),
    };
    if !path.is_file() {
        eprintln!("tsr: no store at {}", path.display());
        return ExitCode::from(2);
    }
    let store = match Store::open(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tsr: cannot open store {}: {e}", path.display());
            return ExitCode::from(2);
        }
    };
    // D1: the check's evidence comes from the log, not from an empty ToolCheck. The log holds the
    // `Executed` that settles a dispatch and the action kinds that decide idempotency. The CAS is
    // not consulted (D1, N3): main's `exec` never writes the tool log to `cas/`, so no artifact on
    // disk can be linked to a dispatch without an `Executed`.
    let events = match store.by_task(&task) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("tsr: cannot read the log: {e}");
            return ExitCode::from(2);
        }
    };
    let reconciler = ToolCheck::from_log(&events);
    let recorded_at = now_iso8601();

    let resolutions = match reconcile_task(&store, &task, &reconciler, &recorded_at) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("tsr: reconcile failed: {e}");
            return ExitCode::from(2);
        }
    };

    let mut blocked = 0usize;
    let mut refused = 0usize;
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
        match resolution {
            Resolution::Executed | Resolution::UnresolvedIdempotent => {}
            Resolution::UnresolvedBlocked => blocked += 1,
            // B6 allows one retry of an idempotent unknown; a second is refused.
            Resolution::Refused => refused += 1,
        }
    }
    if resolutions.is_empty() {
        println!("no pending dispatches");
    }
    if refused > 0 {
        ExitCode::from(2)
    } else if blocked > 0 {
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
