//! The crash and cancellation matrix (PRD §B4, §B6, §B11 M2 item 3).
//!
//! Four rows, each a crash point where the log holds a `Dispatched` with neither `Executed` nor
//! `Unresolved`, and each asserting the exact event kinds appended:
//!
//! 1. **intent** — the crash lands after the intent is committed, before the effect. An
//!    idempotent tool is reconciled to at most one `Executed`; a non-idempotent one is blocked.
//! 2. **send** — the crash lands after the effect was sent, before the acknowledgement. The log
//!    holds no `Executed`, so the check cannot say: the dispatch stays `Unresolved`.
//! 3. **acknowledgement** — the crash lands after the effect, before the ack is recorded. Same
//!    check, same answer: no `Executed`, so `Unresolved`.
//! 4. **cancellation** — the operator cancels: `Unresolved { reason: "cancelled" }` and
//!    `TaskClosed { reason: "operator" }`.
//!
//! Plus the store's own rule: a post-execution attempt to edit a forecast is refused by the
//! `BEFORE UPDATE` trigger, while an outcome append succeeds.

use std::collections::BTreeSet;

use tsr_cli::reconcile_cmd::{
    intent_key, pending_dispatches, reconcile_one, reconcile_task, record_cancellation, Reconciled,
    Reconciler, Resolution, ToolCheck, CANCELLED, UNKNOWN_IDEMPOTENT, UNKNOWN_NON_IDEMPOTENT,
};
use tsr_domain::{
    ActionKind, Candidate, CandidateId, Event, EventId, EventKind, ExecError, ExecOutcome,
    Executor, Hash, Source, TaskId,
};
use tsr_store::Store;

// ---------------------------------------------------------------------------------------------
// A FakeExecutor with injectable failure points (B4, B6)
// ---------------------------------------------------------------------------------------------

/// Where a fake execution is allowed to fail.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CrashPoint {
    /// After the intent is committed, before the effect is sent.
    Intent,
    /// After the effect is sent, before the acknowledgement is recorded.
    Send,
    /// After the effect happened, before the ack is recorded.
    Acknowledgement,
    /// No crash: the effect and its acknowledgement both land.
    None,
}

/// A tool that runs in the fake sandbox. `idempotent` is the tool's own property: an idempotent
/// tool yields at most one effect however many times it is dispatched.
#[derive(Clone, Debug)]
pub struct FakeTool {
    pub idempotent: bool,
    /// The log digest the tool writes when it runs.
    pub log_digest: Hash,
}

/// The fake executor: it records what it was asked to do, and can be told to fail at a point.
///
/// `effects` counts the effects that actually happened, which is what "at most one `Executed` per
/// idempotent dispatch across any crash point" is about.
pub struct FakeExecutor {
    tool: FakeTool,
    crash: CrashPoint,
    /// Effects that happened, by candidate.
    effects: Vec<CandidateId>,
    /// Intents the tool has accepted (its idempotency-key record).
    seen_keys: BTreeSet<String>,
    /// Handles handed out, by candidate.
    handles: Vec<(CandidateId, u32)>,
}

impl FakeExecutor {
    pub fn new(tool: FakeTool, crash: CrashPoint) -> Self {
        Self {
            tool,
            crash,
            effects: Vec::new(),
            seen_keys: BTreeSet::new(),
            handles: Vec::new(),
        }
    }

    /// How many effects actually happened for `candidate`.
    pub fn effects_for(&self, candidate: &CandidateId) -> usize {
        self.effects.iter().filter(|c| *c == candidate).count()
    }

    /// Whether the tool has seen this idempotency key.
    pub fn has_seen(&self, key: &str) -> bool {
        self.seen_keys.contains(key)
    }
}

impl Executor for FakeExecutor {
    fn dispatch(
        &mut self,
        intent: &tsr_domain::DispatchIntent,
    ) -> Result<tsr_domain::DispatchHandle, ExecError> {
        // The intent is committed before the effect: a crash here leaves no effect.
        if self.crash == CrashPoint::Intent {
            return Err(ExecError::Other("crashed at intent".into()));
        }
        let attempt = self
            .handles
            .iter()
            .filter(|(c, _)| *c == intent.candidate)
            .count() as u32
            + 1;
        // An idempotent tool that has already seen the key does not run again.
        if self.tool.idempotent && self.seen_keys.contains(&intent.idempotency_key) {
            self.handles.push((intent.candidate, attempt));
            return Ok(tsr_domain::DispatchHandle {
                candidate: intent.candidate,
                attempt,
            });
        }
        self.seen_keys.insert(intent.idempotency_key.clone());
        self.effects.push(intent.candidate);
        self.handles.push((intent.candidate, attempt));
        if self.crash == CrashPoint::Send {
            return Err(ExecError::Other("crashed at send".into()));
        }
        Ok(tsr_domain::DispatchHandle {
            candidate: intent.candidate,
            attempt,
        })
    }

    fn reconcile(&mut self, h: &tsr_domain::DispatchHandle) -> Result<ExecOutcome, ExecError> {
        if self.crash == CrashPoint::Acknowledgement {
            return Err(ExecError::Other("crashed at acknowledgement".into()));
        }
        if self.effects_for(&h.candidate) == 0 {
            return Ok(ExecOutcome::Unresolved {
                reason: "no effect recorded".into(),
            });
        }
        Ok(ExecOutcome::Completed {
            exit_code: 0,
            log_digest: self.tool.log_digest,
            duration_ms: 7,
        })
    }
}

// ---------------------------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------------------------

fn task() -> TaskId {
    TaskId(Hash::of(&[b"task-reconcile"]))
}

fn candidate(tag: &str) -> Candidate {
    candidate_kind(tag, ActionKind::RunApprovedTest)
}

/// A candidate of a given action kind. `RunApprovedTest` is idempotent; `Generate` is not, so
/// `ToolCheck::from_log` leaves it non-idempotent (the safe default).
fn candidate_kind(tag: &str, kind: ActionKind) -> Candidate {
    Candidate::new(
        kind,
        Source::Rules,
        Hash::of(&[tag.as_bytes()]),
        vec![],
        vec![],
    )
    .unwrap()
}

/// Open a task and dispatch `cand`, returning the store and the dispatch event.
fn dispatched(cand: &Candidate) -> (Store, Event) {
    let s = Store::open_in_memory().unwrap();
    let opened = Event::new(
        task(),
        vec![],
        Source::Rules,
        EventKind::TaskOpened { task: task() },
        "t0",
    );
    s.append(&opened).unwrap();
    let proposed = Event::new(
        task(),
        vec![opened.id],
        Source::Rules,
        EventKind::Proposed {
            candidate: cand.clone(),
        },
        "t1",
    );
    s.append(&proposed).unwrap();
    let dispatch = Event::new(
        task(),
        vec![proposed.id],
        Source::Rules,
        EventKind::Dispatched {
            candidate: cand.id,
            in_sandbox: true,
        },
        "t2",
    );
    s.append(&dispatch).unwrap();
    (s, dispatch)
}

/// D5: the store the executor had written before its crash point, and the CAS root beside it.
///
/// The four rows differ in what the executor had written when it crashed, so each reconciles a
/// different store:
/// - `Intent`: nothing — the crash landed before the intent was committed, so no `Dispatched`.
/// - `Send`: `Dispatched` only — the effect was sent, but the tool's log never reached the CAS.
/// - `Acknowledgement`: `Dispatched` and the tool's log in the CAS, but no `Executed`.
/// - `None`: `Dispatched` and `Executed` — the effect and its acknowledgement both landed.
fn store_at_crash_point(cand: &Candidate, crash: CrashPoint, cas: &std::path::Path) -> Store {
    let s = Store::open_in_memory().unwrap();
    let opened = Event::new(
        task(),
        vec![],
        Source::Rules,
        EventKind::TaskOpened { task: task() },
        "t0",
    );
    s.append(&opened).unwrap();
    let proposed = Event::new(
        task(),
        vec![opened.id],
        Source::Rules,
        EventKind::Proposed {
            candidate: cand.clone(),
        },
        "t1",
    );
    s.append(&proposed).unwrap();
    if crash == CrashPoint::Intent {
        // The crash landed before the intent was committed: nothing was dispatched.
        return s;
    }
    let dispatch = Event::new(
        task(),
        vec![proposed.id],
        Source::Rules,
        EventKind::Dispatched {
            candidate: cand.id,
            in_sandbox: true,
        },
        "t2",
    );
    s.append(&dispatch).unwrap();
    if crash == CrashPoint::Acknowledgement {
        // The tool ran: its log is in the CAS. The crash landed before the `Executed` was appended.
        let digest = Hash::of(&[b"tool-log"]);
        std::fs::write(cas.join(digest.to_hex()), b"the tool's log").unwrap();
    }
    if crash == CrashPoint::None {
        // The effect and its acknowledgement both landed.
        let executed = Event::new(
            task(),
            vec![dispatch.id],
            Source::Rules,
            EventKind::Executed {
                candidate: cand.id,
                exit_code: Some(0),
                log_digest: Some(Hash::of(&[b"tool-log"])),
                duration_ms: 1,
            },
            "t3",
        );
        s.append(&executed).unwrap();
    }
    s
}

/// The kinds appended to a task, in sequence order.
fn kinds(s: &Store) -> Vec<String> {
    s.by_task(&task())
        .unwrap()
        .into_iter()
        .map(|(_, e)| {
            serde_json::to_value(&e.kind)
                .unwrap()
                .get("kind")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect()
}

/// A reconciler that always answers the same thing, for the rows that do not need a tool.
struct Fixed {
    idempotent: bool,
    answer: Reconciled,
}

impl Reconciler for Fixed {
    fn idempotent(&self, _c: &CandidateId) -> bool {
        self.idempotent
    }
    fn check(&self, _c: &CandidateId, _intent: &str) -> Reconciled {
        self.answer.clone()
    }
}
// ---------------------------------------------------------------------------------------------
// Row 1: crash at intent
// ---------------------------------------------------------------------------------------------

#[test]
fn row_intent_idempotent_unknown_is_unresolved_and_retryable() {
    let cand = candidate("intent-idem");
    let (s, dispatch) = dispatched(&cand);
    let pending = pending_dispatches(&s.by_task(&task()).unwrap());
    assert_eq!(pending.len(), 1, "the dispatch has no outcome yet");
    assert_eq!(pending[0].candidate, cand.id);
    assert_eq!(pending[0].dispatched, dispatch.id);

    let r = Fixed {
        idempotent: true,
        answer: Reconciled::Unknown,
    };
    let res = reconcile_one(&s, &task(), &pending[0], &r, "t3").unwrap();
    assert_eq!(res, Resolution::UnresolvedIdempotent);
    assert!(res.retryable(), "an idempotent unknown may be retried once");

    let events = s.by_task(&task()).unwrap();
    assert_eq!(
        kinds(&s),
        vec!["task_opened", "proposed", "dispatched", "unresolved"]
    );
    let last = &events.last().unwrap().1;
    assert_eq!(
        last.parents,
        vec![dispatch.id],
        "the reconciliation names its intent"
    );
    match &last.kind {
        EventKind::Unresolved { candidate, reason } => {
            assert_eq!(*candidate, cand.id);
            assert_eq!(reason, UNKNOWN_IDEMPOTENT);
        }
        other => panic!("expected Unresolved, got {other:?}"),
    }
}

#[test]
fn row_intent_non_idempotent_unknown_is_blocked() {
    let cand = candidate("intent-nonidem");
    let (s, _) = dispatched(&cand);
    let pending = pending_dispatches(&s.by_task(&task()).unwrap());
    let r = Fixed {
        idempotent: false,
        answer: Reconciled::Unknown,
    };
    let res = reconcile_one(&s, &task(), &pending[0], &r, "t3").unwrap();
    assert_eq!(res, Resolution::UnresolvedBlocked);
    assert!(!res.retryable(), "a non-idempotent unknown stays blocked");

    let events = s.by_task(&task()).unwrap();
    assert_eq!(
        kinds(&s),
        vec!["task_opened", "proposed", "dispatched", "unresolved"]
    );
    match &events.last().unwrap().1.kind {
        EventKind::Unresolved { reason, .. } => assert_eq!(reason, UNKNOWN_NON_IDEMPOTENT),
        other => panic!("expected Unresolved, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------------------------
// Row 2: crash at send — the effect was sent, but the log holds no Executed
// ---------------------------------------------------------------------------------------------

#[test]
fn row_send_dispatched_only_is_unresolved_idempotent() {
    // D5: the send crash point leaves `Dispatched` and nothing else — the effect was sent, but
    // the tool's log never reached the CAS and no `Executed` was appended. The CLI's own check
    // (`ToolCheck::from_log`) therefore cannot say, and the dispatch is `Unresolved` (idempotent,
    // retryable). The old row fed `ex.tool_check(...)`, whose private `seen_keys` claimed the
    // effect happened, and asserted `Executed` — a result the CLI can never produce.
    let cand = candidate("send-idem");
    let cas = std::env::temp_dir().join(format!("tsr-d5-send-{}", std::process::id()));
    std::fs::create_dir_all(&cas).unwrap();
    let s = store_at_crash_point(&cand, CrashPoint::Send, &cas);
    assert_eq!(
        kinds(&s),
        vec!["task_opened", "proposed", "dispatched"],
        "the send crash point leaves Dispatched only"
    );

    let events = s.by_task(&task()).unwrap();
    let check = ToolCheck::from_log(&events);
    let pending = pending_dispatches(&events);
    assert_eq!(pending.len(), 1, "the dispatch has no outcome yet");
    let res = reconcile_one(&s, &task(), &pending[0], &check, "t3").unwrap();
    assert_eq!(res, Resolution::UnresolvedIdempotent);
    assert!(res.retryable(), "an idempotent unknown may be retried once");
    assert_eq!(
        kinds(&s),
        vec!["task_opened", "proposed", "dispatched", "unresolved"]
    );
    let events = s.by_task(&task()).unwrap();
    let last = &events.last().unwrap().1;
    match &last.kind {
        EventKind::Unresolved { candidate, reason } => {
            assert_eq!(*candidate, cand.id);
            assert_eq!(reason, UNKNOWN_IDEMPOTENT);
        }
        other => panic!("expected Unresolved, got {other:?}"),
    }
    std::fs::remove_dir_all(&cas).ok();
}

#[test]
fn row_send_non_idempotent_unknown_stays_blocked() {
    let cand = candidate_kind("send-nonidem", ActionKind::Generate);
    let cas = std::env::temp_dir().join(format!("tsr-d5-send-nonidem-{}", std::process::id()));
    std::fs::create_dir_all(&cas).unwrap();
    let s = store_at_crash_point(&cand, CrashPoint::Send, &cas);
    let events = s.by_task(&task()).unwrap();
    let check = ToolCheck::from_log(&events);
    let pending = pending_dispatches(&events);
    let res = reconcile_one(&s, &task(), &pending[0], &check, "t3").unwrap();
    assert_eq!(res, Resolution::UnresolvedBlocked);
    assert!(!res.retryable(), "a non-idempotent unknown stays blocked");
    assert_eq!(
        kinds(&s),
        vec!["task_opened", "proposed", "dispatched", "unresolved"]
    );
    std::fs::remove_dir_all(&cas).ok();
}

// ---------------------------------------------------------------------------------------------
// Row 3: crash at acknowledgement — the tool ran, but the log holds no Executed
// ---------------------------------------------------------------------------------------------

#[test]
fn row_acknowledgement_tool_ran_without_executed_is_unresolved_idempotent() {
    // D5: the acknowledgement crash point leaves `Dispatched` and the tool's log in the CAS, but
    // no `Executed` in the log. The CLI's check (`ToolCheck::from_log`) records a digest only from
    // an `Executed` event, so an orphaned CAS digest is not evidence: the dispatch is `Unresolved`
    // (idempotent, retryable). The old row hand-built a `ToolCheck` with the digest and asserted
    // `Executed` — a result the CLI can never produce.
    let cand = candidate("ack-idem");
    let cas = std::env::temp_dir().join(format!("tsr-d5-ack-{}", std::process::id()));
    std::fs::create_dir_all(&cas).unwrap();
    let s = store_at_crash_point(&cand, CrashPoint::Acknowledgement, &cas);
    assert_eq!(
        kinds(&s),
        vec!["task_opened", "proposed", "dispatched"],
        "the acknowledgement crash point leaves Dispatched and no Executed"
    );
    // The tool ran: its log is in the CAS. But the log holds no `Executed` to name the digest.
    let digest = Hash::of(&[b"tool-log"]);
    assert!(
        cas.join(digest.to_hex()).exists(),
        "the tool's log is in the CAS"
    );

    let events = s.by_task(&task()).unwrap();
    let check = ToolCheck::from_log(&events);
    let pending = pending_dispatches(&events);
    assert_eq!(pending.len(), 1, "the dispatch has no outcome yet");
    let res = reconcile_one(&s, &task(), &pending[0], &check, "t3").unwrap();
    assert_eq!(res, Resolution::UnresolvedIdempotent);
    assert!(res.retryable(), "an idempotent unknown may be retried once");
    assert_eq!(
        kinds(&s),
        vec!["task_opened", "proposed", "dispatched", "unresolved"]
    );
    std::fs::remove_dir_all(&cas).ok();
}

#[test]
fn row_acknowledgement_missing_digest_is_unknown() {
    // The tool's log never reached the CAS either: the check still cannot say.
    let cand = candidate("ack-missing");
    let cas = std::env::temp_dir().join(format!("tsr-d5-ack-missing-{}", std::process::id()));
    std::fs::create_dir_all(&cas).unwrap();
    let s = store_at_crash_point(&cand, CrashPoint::Acknowledgement, &cas);
    let events = s.by_task(&task()).unwrap();
    let check = ToolCheck::from_log(&events);
    let pending = pending_dispatches(&events);
    let res = reconcile_one(&s, &task(), &pending[0], &check, "t3").unwrap();
    assert_eq!(res, Resolution::UnresolvedIdempotent);
    assert_eq!(
        kinds(&s),
        vec!["task_opened", "proposed", "dispatched", "unresolved"]
    );
    std::fs::remove_dir_all(&cas).ok();
}

// ---------------------------------------------------------------------------------------------
// Row 4: no crash — the effect and its acknowledgement both landed
// ---------------------------------------------------------------------------------------------

#[test]
fn row_none_dispatched_and_executed_is_settled() {
    // D5: the no-crash row leaves `Dispatched` and `Executed` in the log, so the dispatch is
    // already settled: `pending_dispatches` finds nothing and reconcile appends nothing. The old
    // matrix asserted `UnresolvedIdempotent` for this row too, because it built the same
    // three-event store as every other row.
    let cand = candidate("none-settled");
    let cas = std::env::temp_dir().join(format!("tsr-d5-none-{}", std::process::id()));
    std::fs::create_dir_all(&cas).unwrap();
    let s = store_at_crash_point(&cand, CrashPoint::None, &cas);
    assert_eq!(
        kinds(&s),
        vec!["task_opened", "proposed", "dispatched", "executed"],
        "the no-crash row leaves Dispatched and Executed"
    );

    let events = s.by_task(&task()).unwrap();
    let pending = pending_dispatches(&events);
    assert!(
        pending.is_empty(),
        "the Executed settles the dispatch: {pending:?}"
    );
    // Reconcile has nothing to settle, so the log is unchanged.
    assert_eq!(
        kinds(&s),
        vec!["task_opened", "proposed", "dispatched", "executed"]
    );
    std::fs::remove_dir_all(&cas).ok();
}

// ---------------------------------------------------------------------------------------------
// Row 4: cancellation
// ---------------------------------------------------------------------------------------------

#[test]
fn row_cancellation_appends_unresolved_cancelled_and_task_closed() {
    let cand = candidate("cancel");
    let (s, dispatch) = dispatched(&cand);
    let pending = pending_dispatches(&s.by_task(&task()).unwrap());
    record_cancellation(&s, &task(), &pending[0], "t3").unwrap();
    let closed = Event::new(
        task(),
        vec![dispatch.id],
        Source::Operator,
        EventKind::TaskClosed {
            task: task(),
            reason: "operator".into(),
        },
        "t3",
    );
    s.append(&closed).unwrap();

    assert_eq!(
        kinds(&s),
        vec![
            "task_opened",
            "proposed",
            "dispatched",
            "unresolved",
            "task_closed"
        ]
    );
    let events = s.by_task(&task()).unwrap();
    match &events[3].1.kind {
        EventKind::Unresolved { reason, .. } => assert_eq!(reason, CANCELLED),
        other => panic!("expected Unresolved, got {other:?}"),
    }
    match &events[4].1.kind {
        EventKind::TaskClosed { reason, .. } => assert_eq!(reason, "operator"),
        other => panic!("expected TaskClosed, got {other:?}"),
    }
    // A cancelled dispatch is settled: nothing is pending afterwards.
    assert!(pending_dispatches(&s.by_task(&task()).unwrap()).is_empty());
}

// ---------------------------------------------------------------------------------------------
// The store's own rule: append-only, and an outcome append succeeds
// ---------------------------------------------------------------------------------------------

#[test]
fn d5_forecast_update_after_executed_is_refused_specifically() {
    // D5: the store's rule is that a forecast cannot be edited *after* Executed exists. The
    // UPDATE must be attempted after the Executed event is in the log, and the refusal must be
    // the store's specific append-only error, not merely `is_err()`.
    let cand = candidate("store-rule");
    let (s, dispatch) = dispatched(&cand);
    let forecast = Event::new(
        task(),
        vec![dispatch.id],
        Source::Rules,
        EventKind::Forecast {
            forecast: tsr_domain::Forecast {
                id: tsr_domain::ForecastId(Hash::of(&[b"f"])),
                event: tsr_domain::OutcomeDefId(Hash::of(&[b"e"])),
                horizon: tsr_domain::Horizon::TaskClose,
                source: tsr_domain::Forecaster::Generator,
                raw: tsr_domain::Probability::new(0.5).unwrap(),
                calibrated: None,
                support: tsr_domain::Support { episodes: 0 },
                state_version: 3,
            },
            candidate: cand.id,
        },
        "t3",
    );
    s.append(&forecast).unwrap();

    // Executed exists first: the forecast is now post-execution.
    let executed = Event::new(
        task(),
        vec![forecast.id],
        Source::Rules,
        EventKind::Executed {
            candidate: cand.id,
            exit_code: Some(0),
            log_digest: None,
            duration_ms: 1,
        },
        "t4",
    );
    s.append(&executed).unwrap();

    // Editing the forecast in place is refused by the BEFORE UPDATE trigger, with its own message.
    let upd = s
        .connection()
        .execute("UPDATE events SET payload = '{}' WHERE seq = 4", []);
    let err = upd.expect_err("the store refuses an in-place edit after Executed");
    let msg = err.to_string();
    assert!(
        msg.contains("events is append-only: updates are refused"),
        "the refusal must be the store's append-only error, got: {msg}"
    );

    // Appending the outcome succeeds: the log grows, it never rewrites.
    let outcome = Event::new(
        task(),
        vec![executed.id],
        Source::Operator,
        EventKind::OutcomeRecorded {
            record: tsr_domain::OutcomeRecord {
                outcome: tsr_domain::OutcomeDefId(Hash::of(&[b"e"])),
                subject: forecast.id,
                status: tsr_domain::LabelStatus::Observed,
                reviewer: Source::Operator,
                reason: None,
                evidence: None,
                supersedes: None,
                weight: None,
            },
        },
        "t5",
    );
    s.append(&outcome).unwrap();
    assert_eq!(
        kinds(&s),
        vec![
            "task_opened",
            "proposed",
            "dispatched",
            "forecast",
            "executed",
            "outcome_recorded"
        ]
    );
}

// ---------------------------------------------------------------------------------------------
// The matrix as a whole: each row reconciles the store its crash point left behind
// ---------------------------------------------------------------------------------------------

/// D5: the crash matrix, one row per crash point, each reconciling the store the executor had
/// written before that crash point — not the same three events for every row. The old matrix
/// built `dispatched()` for every row, so Intent, Send, Acknowledgement and None got identical
/// input and all asserted `UnresolvedIdempotent`, including the no-crash row whose effect and
/// acknowledgement both landed. Here each row builds its own store, drives `ToolCheck::from_log`
/// exactly as `reconcile_cmd::run` does, and asserts the result the CLI gives:
///
/// | row             | store the crash left                          | CLI result            |
/// |-----------------|-----------------------------------------------|-----------------------|
/// | Intent          | nothing dispatched                            | no pending dispatch   |
/// | Send            | Dispatched only                               | UnresolvedIdempotent  |
/// | Acknowledgement | Dispatched + tool log in CAS, no Executed     | UnresolvedIdempotent  |
/// | None            | Dispatched + Executed                         | settled, nothing new  |
#[test]
fn d5_matrix_rows_reconcile_the_store_their_crash_point_left() {
    for crash in [
        CrashPoint::Intent,
        CrashPoint::Send,
        CrashPoint::Acknowledgement,
        CrashPoint::None,
    ] {
        let cand = candidate(&format!("matrix-{crash:?}"));
        let cas = std::env::temp_dir().join(format!(
            "tsr-d5-matrix-cas-{}-{crash:?}",
            std::process::id()
        ));
        std::fs::create_dir_all(&cas).unwrap();
        let s = store_at_crash_point(&cand, crash, &cas);
        let before = kinds(&s);
        let events = s.by_task(&task()).unwrap();
        let check = ToolCheck::from_log(&events);
        let pending = pending_dispatches(&events);

        match crash {
            CrashPoint::Intent => {
                // Nothing was dispatched: there is nothing to reconcile.
                assert_eq!(before, vec!["task_opened", "proposed"], "{crash:?}");
                assert!(pending.is_empty(), "nothing dispatched at {crash:?}");
                assert_eq!(kinds(&s), before, "reconcile appends nothing at {crash:?}");
            }
            CrashPoint::Send | CrashPoint::Acknowledgement => {
                // Dispatched, but no Executed in the log: the CLI cannot say.
                assert_eq!(
                    before,
                    vec!["task_opened", "proposed", "dispatched"],
                    "{crash:?}"
                );
                assert_eq!(pending.len(), 1, "one pending dispatch at {crash:?}");
                let res = reconcile_one(&s, &task(), &pending[0], &check, "t3").unwrap();
                assert_eq!(
                    res,
                    Resolution::UnresolvedIdempotent,
                    "the log holds no Executed at {crash:?}"
                );
                assert_eq!(
                    kinds(&s),
                    vec!["task_opened", "proposed", "dispatched", "unresolved"],
                    "settled at {crash:?}"
                );
            }
            CrashPoint::None => {
                // The effect and its acknowledgement both landed: the dispatch is settled.
                assert_eq!(
                    before,
                    vec!["task_opened", "proposed", "dispatched", "executed"],
                    "{crash:?}"
                );
                assert!(pending.is_empty(), "the Executed settles it at {crash:?}");
                assert_eq!(kinds(&s), before, "reconcile appends nothing at {crash:?}");
            }
        }
        std::fs::remove_dir_all(&cas).ok();
    }
}

/// D5 regression guard: the Acknowledgement crash point must NOT settle the dispatch as
/// `Executed` from the fake executor's private keys. The old matrix fed `ex.tool_check(...)`,
/// whose `seen_keys` already held the idempotency key (the effect was sent before the ack
/// crashed), so the row settled `Executed` — the same as the `None` row. The CLI builds its
/// check with `ToolCheck::from_log` over the store, which holds no `Executed` for this dispatch
/// and no digest in the CAS, so the row is `Unresolved` (idempotent, retryable). This test fails
/// if the matrix ever goes back to the fake's keys.
#[test]
fn d5_acknowledgement_row_is_not_settled_from_the_fakes_keys() {
    let cand = candidate("matrix-ack-not-fake");
    let digest = Hash::of(&[b"log-Acknowledgement"]);
    let mut ex = FakeExecutor::new(
        FakeTool {
            idempotent: true,
            log_digest: digest,
        },
        CrashPoint::Acknowledgement,
    );
    let intent = tsr_domain::DispatchIntent {
        candidate: cand.id,
        idempotency_key: intent_key(&cand.id),
        expected_state_version: 3,
    };
    let _ = ex.dispatch(&intent);
    let _ = ex.dispatch(&intent);
    // The fake's own keys would answer `Executed` here — that is the defect. The fake has seen
    // the idempotency key (the effect was sent before the ack crashed), so a check built from
    // its private state claims the effect happened.
    assert!(
        ex.has_seen(&intent_key(&cand.id)),
        "the fake's private keys claim the effect happened"
    );
    // The CLI's check, built from the log and the CAS, cannot say: no `Executed` in the log and
    // no digest in the CAS.
    let cas = std::env::temp_dir().join(format!("tsr-d5-ack-not-fake-{}", std::process::id()));
    std::fs::create_dir_all(&cas).unwrap();
    let s = store_at_crash_point(&cand, CrashPoint::Acknowledgement, &cas);
    let events = s.by_task(&task()).unwrap();
    let check = ToolCheck::from_log(&events);
    let pending = pending_dispatches(&events);
    let res = reconcile_one(&s, &task(), &pending[0], &check, "t3").unwrap();
    assert_eq!(
        res,
        Resolution::UnresolvedIdempotent,
        "the log's evidence decides, not the fake's keys"
    );
    let ks = kinds(&s);
    assert_eq!(
        ks.iter().filter(|k| *k == "executed").count(),
        0,
        "no Executed is invented: {ks:?}"
    );
    std::fs::remove_dir_all(&cas).ok();
}

#[test]
fn reconcile_task_settles_every_pending_dispatch_once() {
    let a = candidate("multi-a");
    let b = candidate("multi-b");
    let s = Store::open_in_memory().unwrap();
    let opened = Event::new(
        task(),
        vec![],
        Source::Rules,
        EventKind::TaskOpened { task: task() },
        "t0",
    );
    s.append(&opened).unwrap();
    let mut parents = vec![opened.id];
    for c in [&a, &b] {
        let proposed = Event::new(
            task(),
            parents.clone(),
            Source::Rules,
            EventKind::Proposed {
                candidate: c.clone(),
            },
            "t1",
        );
        s.append(&proposed).unwrap();
        let dispatch = Event::new(
            task(),
            vec![proposed.id],
            Source::Rules,
            EventKind::Dispatched {
                candidate: c.id,
                in_sandbox: true,
            },
            "t2",
        );
        s.append(&dispatch).unwrap();
        parents = vec![dispatch.id];
    }
    let r = Fixed {
        idempotent: true,
        answer: Reconciled::Executed {
            exit_code: Some(0),
            log_digest: None,
            duration_ms: 1,
        },
    };
    let out = reconcile_task(&s, &task(), &r, "t3").unwrap();
    assert_eq!(out.len(), 2, "one resolution per pending dispatch");
    assert!(out.iter().all(|(_, r)| *r == Resolution::Executed));
    assert_eq!(
        kinds(&s),
        vec![
            "task_opened",
            "proposed",
            "dispatched",
            "proposed",
            "dispatched",
            "executed",
            "executed"
        ]
    );
    // Idempotent: a second reconcile finds nothing pending.
    assert!(reconcile_task(&s, &task(), &r, "t4").unwrap().is_empty());
}

// ---------------------------------------------------------------------------------------------
// The CLI invocation itself: `tsr task reconcile <task> --store <path>` over a real store
// ---------------------------------------------------------------------------------------------

#[test]
fn cli_reconcile_over_a_real_store_reports_and_settles() {
    let dir = std::env::temp_dir().join(format!("tsr-reconcile-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("tsr.db");

    // Seed a real on-disk store with one dispatched candidate and no outcome.
    let cand = candidate("cli-smoke");
    {
        let s = Store::open(&db).unwrap();
        let opened = Event::new(
            task(),
            vec![],
            Source::Rules,
            EventKind::TaskOpened { task: task() },
            "t0",
        );
        s.append(&opened).unwrap();
        let proposed = Event::new(
            task(),
            vec![opened.id],
            Source::Rules,
            EventKind::Proposed {
                candidate: cand.clone(),
            },
            "t1",
        );
        s.append(&proposed).unwrap();
        let dispatch = Event::new(
            task(),
            vec![proposed.id],
            Source::Rules,
            EventKind::Dispatched {
                candidate: cand.id,
                in_sandbox: true,
            },
            "t2",
        );
        s.append(&dispatch).unwrap();
    }
    let bin = env!("CARGO_BIN_EXE_tsr");
    let out = std::process::Command::new(bin)
        .args([
            "task",
            "reconcile",
            &task().to_string(),
            "--store",
            db.to_str().unwrap(),
        ])
        .output()
        .expect("the tsr binary runs");
    // D1: the CLI loads its evidence from the log, so a `RunApprovedTest` candidate is known
    // idempotent. With no digest in the CAS the outcome is unknown, so the dispatch is recorded
    // `unresolved` and is retryable (exit 0) — not blocked.
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("unresolved") && stdout.contains("retryable"),
        "stdout: {stdout}"
    );

    // The reconciliation landed in the log: dispatched then unresolved, nothing else.
    let s = Store::open(&db).unwrap();
    assert_eq!(
        kinds(&s),
        vec!["task_opened", "proposed", "dispatched", "unresolved"]
    );

    std::fs::remove_dir_all(&dir).ok();
}

// ---------------------------------------------------------------------------------------------
// D1: the CLI loads its ToolCheck evidence from the log, re-checks a blocked
// dispatch, and enforces retry-once for an idempotent unknown.
//
// Defect D1 (repair brief item 4): `reconcile_cmd::run` built `ToolCheck::new(cas_root)` with no
// digests, no seen keys and nothing idempotent, so every pending dispatch was "non-idempotent,
// blocked" and, once `Unresolved`, never re-checked (`pending_dispatches` treated any `Unresolved`
// as settled). These tests drive the built binary over a real on-disk store and CAS.
// ---------------------------------------------------------------------------------------------

/// A scratch directory under the system temp dir (R7: a test must never write into the source
/// tree).
fn d1_scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join("d1-reconcile").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

/// Seed a store with `TaskOpened`, `Proposed(cand)`, `Dispatched(cand)` and return the dispatch id.
fn d1_seed_dispatched(db: &std::path::Path, cand: &Candidate) -> EventId {
    let s = Store::open(db).unwrap();
    let opened = Event::new(
        task(),
        vec![],
        Source::Rules,
        EventKind::TaskOpened { task: task() },
        "t0",
    );
    s.append(&opened).unwrap();
    let proposed = Event::new(
        task(),
        vec![opened.id],
        Source::Rules,
        EventKind::Proposed {
            candidate: cand.clone(),
        },
        "t1",
    );
    s.append(&proposed).unwrap();
    let dispatch = Event::new(
        task(),
        vec![proposed.id],
        Source::Rules,
        EventKind::Dispatched {
            candidate: cand.id,
            in_sandbox: true,
        },
        "t2",
    );
    s.append(&dispatch).unwrap();
    dispatch.id
}

/// Run `tsr task reconcile <task> --store <db>` and return (exit code, stdout, stderr).
///
/// Retries briefly on SQLite's "database is locked": the seeding connection in this process may
/// not have released the file when the child starts.
fn d1_run_reconcile(db: &std::path::Path) -> (Option<i32>, String, String) {
    for attempt in 0..20 {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_tsr"))
            .args([
                "task",
                "reconcile",
                &task().to_string(),
                "--store",
                db.to_str().unwrap(),
            ])
            .output()
            .expect("the tsr binary runs");
        let stderr = String::from_utf8_lossy(&out.stderr).into_owned();
        if stderr.contains("database is locked") && attempt < 19 {
            std::thread::sleep(std::time::Duration::from_millis(50));
            continue;
        }
        return (
            out.status.code(),
            String::from_utf8_lossy(&out.stdout).into_owned(),
            stderr,
        );
    }
    unreachable!("the retry loop returns")
}

/// D1: an idempotent (`RunApprovedTest`) dispatch whose log holds its own `Executed` is settled —
/// `pending_dispatches` finds nothing and reconcile appends nothing. The CAS is not consulted
/// (D1, N3): the `Executed` is the evidence, not the artifact on disk.
#[test]
fn d1_reconcile_resolves_idempotent_dispatch_with_digest_in_cas() {
    let dir = d1_scratch("resolves-idempotent");
    let db = dir.join("tsr.db");
    let cas = dir.join("cas");
    std::fs::create_dir_all(&cas).unwrap();

    let cand = candidate("d1-idem");
    let dispatch = d1_seed_dispatched(&db, &cand);

    // The tool ran once (its log digest D is in the CAS) and the log recorded that `Executed`.
    let digest = Hash::of(&[b"d1-tool-log"]);
    std::fs::write(cas.join(digest.to_hex()), b"the tool's log").unwrap();
    {
        let s = Store::open(&db).unwrap();
        let executed = Event::new(
            task(),
            vec![dispatch],
            Source::Rules,
            EventKind::Executed {
                candidate: cand.id,
                exit_code: Some(0),
                log_digest: Some(digest),
                duration_ms: 7,
            },
            "t3",
        );
        s.append(&executed).unwrap();
    }

    let (code, stdout, stderr) = d1_run_reconcile(&db);
    assert_eq!(code, Some(0), "stdout: {stdout}\nstderr: {stderr}");
    // The dispatch's own `Executed` is its evidence, so nothing is pending: the CAS digest in the
    // log settles it and reconcile appends nothing.
    assert!(
        stdout.contains("no pending dispatches"),
        "the dispatch is settled by its own Executed; stdout: {stdout}"
    );

    // The log's last event is the dispatch's own Executed carrying the CAS digest.
    let s = Store::open(&db).unwrap();
    let events = s.by_task(&task()).unwrap();
    let last = &events.last().unwrap().1;
    match &last.kind {
        EventKind::Executed {
            candidate,
            log_digest,
            ..
        } => {
            assert_eq!(*candidate, cand.id);
            assert_eq!(*log_digest, Some(digest), "the CAS digest is the evidence");
        }
        other => panic!("expected Executed, got {other:?}"),
    }

    std::fs::remove_dir_all(&dir).ok();
}

/// D1: a non-idempotent dispatch left `Unresolved{blocked}` is re-checked on every reconcile —
/// the CLI appends a fresh `Unresolved` rather than treating the blocked one as settled.
#[test]
fn d1_blocked_dispatch_is_rechecked() {
    let dir = d1_scratch("blocked-rechecked");
    let db = dir.join("tsr.db");

    // A `Generate` candidate is not idempotent: an unknown outcome stays blocked.
    let cand = Candidate::new(
        ActionKind::Generate,
        Source::Generator,
        Hash::of(&[b"d1-blocked"]),
        vec![],
        vec![],
    )
    .unwrap();
    let dispatch = d1_seed_dispatched(&db, &cand);
    {
        let s = Store::open(&db).unwrap();
        let blocked = Event::new(
            task(),
            vec![dispatch],
            Source::Rules,
            EventKind::Unresolved {
                candidate: cand.id,
                reason: UNKNOWN_NON_IDEMPOTENT.to_string(),
            },
            "t3",
        );
        s.append(&blocked).unwrap();
    }

    let (code, stdout, stderr) = d1_run_reconcile(&db);
    assert_eq!(
        code,
        Some(1),
        "still blocked; stdout: {stdout}\nstderr: {stderr}"
    );

    // The blocked dispatch was re-checked: a second Unresolved was appended.
    let s = Store::open(&db).unwrap();
    let unresolved = s
        .by_task(&task())
        .unwrap()
        .into_iter()
        .filter(|(_, e)| matches!(e.kind, EventKind::Unresolved { .. }))
        .count();
    assert_eq!(
        unresolved, 2,
        "a blocked Unresolved is re-checked, not settled; stdout: {stdout}"
    );

    std::fs::remove_dir_all(&dir).ok();
}

/// D1: "retry once" for an idempotent unknown is enforced — a second retry refuses (exit 2).
#[test]
fn d1_second_retry_refuses() {
    let dir = d1_scratch("second-retry");
    let db = dir.join("tsr.db");

    // An idempotent `RunApprovedTest` whose first retry already happened: the log holds
    // `Unresolved{UNKNOWN_IDEMPOTENT}`. A second retry must refuse.
    let cand = candidate("d1-retry-once");
    let dispatch = d1_seed_dispatched(&db, &cand);
    {
        let s = Store::open(&db).unwrap();
        let first = Event::new(
            task(),
            vec![dispatch],
            Source::Rules,
            EventKind::Unresolved {
                candidate: cand.id,
                reason: UNKNOWN_IDEMPOTENT.to_string(),
            },
            "t3",
        );
        s.append(&first).unwrap();
    }

    let (code, stdout, stderr) = d1_run_reconcile(&db);
    assert_eq!(
        code,
        Some(2),
        "a second retry refuses; stdout: {stdout}\nstderr: {stderr}"
    );

    // No third event was appended: the refusal is a refusal, not another retry.
    let s = Store::open(&db).unwrap();
    let unresolved = s
        .by_task(&task())
        .unwrap()
        .into_iter()
        .filter(|(_, e)| matches!(e.kind, EventKind::Unresolved { .. }))
        .count();
    assert_eq!(unresolved, 1, "the second retry appended nothing");

    std::fs::remove_dir_all(&dir).ok();
}

// ---------------------------------------------------------------------------------------------
// D1 + R4: evidence is per dispatch. An `Executed` settles only the dispatch it follows; a
// re-dispatch is pending until its own evidence is found. The CAS is not consulted (D1, N3).
// ---------------------------------------------------------------------------------------------

/// D1: a `RunApprovedTest` dispatch whose tool log reached `cas/` but whose `Executed` was never
/// appended stays `unresolved retryable` — the CAS alone does not settle a dispatch, because the
/// log holds no `Executed` naming the digest.
#[test]
fn d1_cas_digest_without_executed_does_not_settle_dispatch() {
    let dir = d1_scratch("cas-without-executed");
    let db = dir.join("tsr.db");
    let cas = dir.join("cas");
    std::fs::create_dir_all(&cas).unwrap();

    let cand = candidate("d1-cas-no-executed");
    d1_seed_dispatched(&db, &cand);

    // The tool wrote its log to the CAS, but the crash landed before the `Executed` was appended.
    let digest = Hash::of(&[b"d1-orphan-log"]);
    std::fs::write(cas.join(digest.to_hex()), b"the tool's log").unwrap();

    let (code, stdout, stderr) = d1_run_reconcile(&db);
    assert_eq!(
        code,
        Some(0),
        "an idempotent unknown is retryable; stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("unresolved") && stdout.contains("retryable"),
        "the CAS alone does not settle the dispatch; stdout: {stdout}"
    );

    // No `Executed` was invented: the log holds dispatched then unresolved.
    let s = Store::open(&db).unwrap();
    assert_eq!(
        kinds(&s),
        vec!["task_opened", "proposed", "dispatched", "unresolved"]
    );

    std::fs::remove_dir_all(&dir).ok();
}

/// R4: an earlier `Executed` marks the candidate "seen", so a later `Dispatched` of the same
/// candidate — even a non-idempotent `ProposePatch` — was settled as `Executed` with the old
/// digest. Evidence is per dispatch: the re-dispatch is pending until its own evidence is found.
#[test]
fn r4_redispatch_is_not_settled_by_an_earlier_executed() {
    let dir = d1_scratch("redispatch-not-settled");
    let db = dir.join("tsr.db");
    let cas = dir.join("cas");
    std::fs::create_dir_all(&cas).unwrap();

    // A non-idempotent `ProposePatch`: a re-dispatch must never be settled by an earlier effect.
    let cand = Candidate::new(
        ActionKind::ProposePatch,
        Source::Rules,
        Hash::of(&[b"r4-redispatch"]),
        vec![],
        vec![],
    )
    .unwrap();
    let first = d1_seed_dispatched(&db, &cand);

    // The first dispatch executed: its digest is in the CAS and the log holds the `Executed`.
    let digest = Hash::of(&[b"r4-first-log"]);
    std::fs::write(cas.join(digest.to_hex()), b"the first run's log").unwrap();
    {
        let s = Store::open(&db).unwrap();
        let executed = Event::new(
            task(),
            vec![first],
            Source::Rules,
            EventKind::Executed {
                candidate: cand.id,
                exit_code: Some(0),
                log_digest: Some(digest),
                duration_ms: 5,
            },
            "t3",
        );
        s.append(&executed).unwrap();
        // A second dispatch of the same candidate, with no evidence of its own yet.
        let retry = Event::new(
            task(),
            vec![executed.id],
            Source::Rules,
            EventKind::Dispatched {
                candidate: cand.id,
                in_sandbox: true,
            },
            "t4",
        );
        s.append(&retry).unwrap();
    }

    let (code, stdout, stderr) = d1_run_reconcile(&db);
    // The re-dispatch is non-idempotent with no evidence of its own: blocked, exit 1.
    assert_eq!(
        code,
        Some(1),
        "a re-dispatch is not settled by an earlier Executed; stdout: {stdout}\nstderr: {stderr}"
    );

    // The log holds exactly one Executed (the first dispatch's) and a fresh Unresolved for the
    // re-dispatch — the old digest was not reused.
    let s = Store::open(&db).unwrap();
    let events = s.by_task(&task()).unwrap();
    let executed = events
        .iter()
        .filter(|(_, e)| matches!(e.kind, EventKind::Executed { .. }))
        .count();
    assert_eq!(executed, 1, "the earlier Executed is not duplicated");
    let last = &events.last().unwrap().1;
    match &last.kind {
        EventKind::Unresolved { candidate, reason } => {
            assert_eq!(*candidate, cand.id);
            assert_eq!(reason, UNKNOWN_NON_IDEMPOTENT);
        }
        other => panic!("expected a fresh Unresolved for the re-dispatch, got {other:?}"),
    }

    std::fs::remove_dir_all(&dir).ok();
}

/// R4: a re-dispatch of an idempotent candidate whose *own* `Executed` follows it resolves to
/// `Executed` — the evidence is the dispatch's own, not an earlier one's.
#[test]
fn r4_redispatch_with_its_own_executed_resolves() {
    let dir = d1_scratch("redispatch-own-evidence");
    let db = dir.join("tsr.db");
    let cas = dir.join("cas");
    std::fs::create_dir_all(&cas).unwrap();

    let cand = candidate("r4-own-evidence");
    let first = d1_seed_dispatched(&db, &cand);

    let first_digest = Hash::of(&[b"r4-first"]);
    let second_digest = Hash::of(&[b"r4-second"]);
    std::fs::write(cas.join(first_digest.to_hex()), b"first").unwrap();
    std::fs::write(cas.join(second_digest.to_hex()), b"second").unwrap();
    {
        let s = Store::open(&db).unwrap();
        let executed = Event::new(
            task(),
            vec![first],
            Source::Rules,
            EventKind::Executed {
                candidate: cand.id,
                exit_code: Some(0),
                log_digest: Some(first_digest),
                duration_ms: 5,
            },
            "t3",
        );
        s.append(&executed).unwrap();
        let retry = Event::new(
            task(),
            vec![executed.id],
            Source::Rules,
            EventKind::Dispatched {
                candidate: cand.id,
                in_sandbox: true,
            },
            "t4",
        );
        s.append(&retry).unwrap();
        // The retry's own evidence: an `Executed` that follows the re-dispatch.
        let second = Event::new(
            task(),
            vec![retry.id],
            Source::Rules,
            EventKind::Executed {
                candidate: cand.id,
                exit_code: Some(0),
                log_digest: Some(second_digest),
                duration_ms: 6,
            },
            "t5",
        );
        s.append(&second).unwrap();
    }

    let (code, stdout, stderr) = d1_run_reconcile(&db);
    assert_eq!(code, Some(0), "stdout: {stdout}\nstderr: {stderr}");

    // The re-dispatch was settled by its own Executed; nothing new was appended (the log already
    // holds the retry's evidence, so there is no pending dispatch).
    let s = Store::open(&db).unwrap();
    let events = s.by_task(&task()).unwrap();
    let executed = events
        .iter()
        .filter(|(_, e)| matches!(e.kind, EventKind::Executed { .. }))
        .count();
    assert_eq!(executed, 2, "both dispatches have their own Executed");

    std::fs::remove_dir_all(&dir).ok();
}

/// R4: "retry once" counts automatic retries per candidate across all its dispatches. A second
/// retry of the same candidate refuses with exit 2, even after a re-dispatch — the retry budget is
/// per candidate, not reset by a new `Dispatched`.
#[test]
fn r4_second_retry_across_dispatches_refuses() {
    let dir = d1_scratch("second-retry-across");
    let db = dir.join("tsr.db");

    let cand = candidate("r4-retry-across");
    let first = d1_seed_dispatched(&db, &cand);
    {
        let s = Store::open(&db).unwrap();
        // The first dispatch's retry was already spent.
        let first_unknown = Event::new(
            task(),
            vec![first],
            Source::Rules,
            EventKind::Unresolved {
                candidate: cand.id,
                reason: UNKNOWN_IDEMPOTENT.to_string(),
            },
            "t3",
        );
        s.append(&first_unknown).unwrap();
        // A re-dispatch of the same candidate.
        let retry = Event::new(
            task(),
            vec![first_unknown.id],
            Source::Rules,
            EventKind::Dispatched {
                candidate: cand.id,
                in_sandbox: true,
            },
            "t4",
        );
        s.append(&retry).unwrap();
    }

    let (code, stdout, stderr) = d1_run_reconcile(&db);
    assert_eq!(
        code,
        Some(2),
        "a second retry of the same candidate refuses; stdout: {stdout}\nstderr: {stderr}"
    );

    // No third event was appended: the refusal is a refusal.
    let s = Store::open(&db).unwrap();
    let unresolved = s
        .by_task(&task())
        .unwrap()
        .into_iter()
        .filter(|(_, e)| matches!(e.kind, EventKind::Unresolved { .. }))
        .count();
    assert_eq!(unresolved, 1, "the second retry appended nothing");

    std::fs::remove_dir_all(&dir).ok();
}
