//! The crash and cancellation matrix (PRD §B4, §B6, §B11 M2 item 3).
//!
//! Four rows, each a crash point where the log holds a `Dispatched` with neither `Executed` nor
//! `Unresolved`, and each asserting the exact event kinds appended:
//!
//! 1. **intent** — the crash lands after the intent is committed, before the effect. An
//!    idempotent tool is reconciled to at most one `Executed`; a non-idempotent one is blocked.
//! 2. **send** — the crash lands after the effect was sent, before the acknowledgement. The
//!    tool's own check (a log digest in the CAS, an idempotency key seen) settles it.
//! 3. **acknowledgement** — the crash lands after the effect, before the ack is recorded. Same
//!    check, different evidence.
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

    /// The tool's own check, as a [`Reconciler`] would ask it.
    pub fn tool_check(&self, cas_root: &std::path::Path, candidate: CandidateId) -> ToolCheck {
        let mut check = ToolCheck::new(cas_root).with_log_digest(candidate, self.tool.log_digest);
        if self.tool.idempotent {
            check = check.with_idempotent(candidate);
        }
        for key in &self.seen_keys {
            check = check.with_seen_key(candidate, key.clone());
        }
        check
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
    Candidate::new(
        ActionKind::RunApprovedTest,
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
// Row 2: crash at send — the tool's own check settles it
// ---------------------------------------------------------------------------------------------

#[test]
fn row_send_idempotent_check_finds_the_effect_and_appends_executed() {
    let cand = candidate("send-idem");
    let (s, dispatch) = dispatched(&cand);
    let digest = Hash::of(&[b"tool-log"]);
    let mut ex = FakeExecutor::new(
        FakeTool {
            idempotent: true,
            log_digest: digest,
        },
        CrashPoint::Send,
    );
    // The effect was sent, so the tool has seen the key even though the ack never landed.
    let intent = tsr_domain::DispatchIntent {
        candidate: cand.id,
        idempotency_key: intent_key(&cand.id),
        expected_state_version: 3,
    };
    assert!(ex.dispatch(&intent).is_err(), "the crash is at send");
    assert!(ex.has_seen(&intent.idempotency_key), "the effect happened");

    let pending = pending_dispatches(&s.by_task(&task()).unwrap());
    let check = ex.tool_check(std::path::Path::new("/nonexistent-cas"), cand.id);
    let res = reconcile_one(&s, &task(), &pending[0], &check, "t3").unwrap();
    assert_eq!(res, Resolution::Executed);

    let events = s.by_task(&task()).unwrap();
    assert_eq!(
        kinds(&s),
        vec!["task_opened", "proposed", "dispatched", "executed"]
    );
    let last = &events.last().unwrap().1;
    assert_eq!(last.parents, vec![dispatch.id]);
    match &last.kind {
        EventKind::Executed {
            candidate,
            log_digest,
            ..
        } => {
            assert_eq!(*candidate, cand.id);
            assert_eq!(*log_digest, Some(digest));
        }
        other => panic!("expected Executed, got {other:?}"),
    }
    assert_eq!(ex.effects_for(&cand.id), 1, "exactly one effect");
}

#[test]
fn row_send_non_idempotent_unknown_stays_blocked() {
    let cand = candidate("send-nonidem");
    let (s, _) = dispatched(&cand);
    let mut ex = FakeExecutor::new(
        FakeTool {
            idempotent: false,
            log_digest: Hash::of(&[b"tool-log-2"]),
        },
        CrashPoint::Send,
    );
    let intent = tsr_domain::DispatchIntent {
        candidate: cand.id,
        idempotency_key: intent_key(&cand.id),
        expected_state_version: 3,
    };
    assert!(ex.dispatch(&intent).is_err());
    // The tool has no record of the key (it is not idempotent, so it keeps none), and the log
    // digest is not in the CAS: the check cannot say.
    let check = ToolCheck::new("/nonexistent-cas");
    let pending = pending_dispatches(&s.by_task(&task()).unwrap());
    let res = reconcile_one(&s, &task(), &pending[0], &check, "t3").unwrap();
    assert_eq!(res, Resolution::UnresolvedBlocked);
    assert_eq!(
        kinds(&s),
        vec!["task_opened", "proposed", "dispatched", "unresolved"]
    );
}

// ---------------------------------------------------------------------------------------------
// Row 3: crash at acknowledgement — the CAS digest settles it
// ---------------------------------------------------------------------------------------------

#[test]
fn row_acknowledgement_cas_digest_settles_the_dispatch() {
    let cand = candidate("ack-idem");
    let (s, _) = dispatched(&cand);
    let digest = Hash::of(&[b"tool-log-3"]);
    let dir = std::env::temp_dir().join(format!("tsr-cas-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(digest.to_hex()), b"the tool's log").unwrap();

    let check = ToolCheck::new(&dir)
        .with_idempotent(cand.id)
        .with_log_digest(cand.id, digest);
    let pending = pending_dispatches(&s.by_task(&task()).unwrap());
    let res = reconcile_one(&s, &task(), &pending[0], &check, "t3").unwrap();
    assert_eq!(res, Resolution::Executed);
    assert_eq!(
        kinds(&s),
        vec!["task_opened", "proposed", "dispatched", "executed"]
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn row_acknowledgement_missing_digest_is_unknown() {
    let cand = candidate("ack-missing");
    let (s, _) = dispatched(&cand);
    let digest = Hash::of(&[b"never-written"]);
    let check = ToolCheck::new("/nonexistent-cas")
        .with_idempotent(cand.id)
        .with_log_digest(cand.id, digest);
    let pending = pending_dispatches(&s.by_task(&task()).unwrap());
    let res = reconcile_one(&s, &task(), &pending[0], &check, "t3").unwrap();
    assert_eq!(res, Resolution::UnresolvedIdempotent);
    assert_eq!(
        kinds(&s),
        vec!["task_opened", "proposed", "dispatched", "unresolved"]
    );
}

#[test]
fn d5_acknowledgement_crash_point_drives_fake_executor_reconcile() {
    // D5: the acknowledgement crash point must be exercised through the executor's own
    // `reconcile`, not only through a hand-built ToolCheck. The crash lands after the effect
    // happened, before the ack is recorded: `reconcile` must fail, and the tool's check must
    // then settle the dispatch to exactly one Executed.
    let cand = candidate("ack-executor");
    let (s, dispatch) = dispatched(&cand);
    let digest = Hash::of(&[b"tool-log-ack-executor"]);
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
    // The effect happened (the tool saw the key), but the acknowledgement never landed.
    let handle = ex.dispatch(&intent).expect("the effect is sent");
    assert!(ex.has_seen(&intent.idempotency_key), "the effect happened");
    assert!(
        ex.reconcile(&handle).is_err(),
        "the acknowledgement crash point must make reconcile fail"
    );
    assert_eq!(ex.effects_for(&cand.id), 1, "exactly one effect");

    // The tool's own check settles it: the log digest is in the CAS.
    let dir = std::env::temp_dir().join(format!("tsr-cas-ack-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(digest.to_hex()), b"the tool's log").unwrap();
    let check = ex.tool_check(&dir, cand.id);
    let pending = pending_dispatches(&s.by_task(&task()).unwrap());
    let res = reconcile_one(&s, &task(), &pending[0], &check, "t3").unwrap();
    assert_eq!(res, Resolution::Executed);
    assert_eq!(
        kinds(&s),
        vec!["task_opened", "proposed", "dispatched", "executed"]
    );
    let events = s.by_task(&task()).unwrap();
    assert_eq!(events.last().unwrap().1.parents, vec![dispatch.id]);
    assert_eq!(ex.effects_for(&cand.id), 1, "still exactly one effect");
    std::fs::remove_dir_all(&dir).ok();
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
// The matrix as a whole: at most one Executed per idempotent dispatch, across every crash point
// ---------------------------------------------------------------------------------------------

#[test]
fn d5_at_most_one_executed_per_idempotent_dispatch_across_every_crash_point() {
    for crash in [
        CrashPoint::Intent,
        CrashPoint::Send,
        CrashPoint::Acknowledgement,
        CrashPoint::None,
    ] {
        let cand = candidate(&format!("matrix-{crash:?}"));
        let (s, _) = dispatched(&cand);
        let digest = Hash::of(&[format!("log-{crash:?}").as_bytes()]);
        let mut ex = FakeExecutor::new(
            FakeTool {
                idempotent: true,
                log_digest: digest,
            },
            crash,
        );
        let intent = tsr_domain::DispatchIntent {
            candidate: cand.id,
            idempotency_key: intent_key(&cand.id),
            expected_state_version: 3,
        };
        // Dispatch, then retry once: an idempotent tool yields at most one effect.
        let _ = ex.dispatch(&intent);
        let _ = ex.dispatch(&intent);
        assert!(
            ex.effects_for(&cand.id) <= 1,
            "idempotent tool ran twice at {crash:?}"
        );

        let pending = pending_dispatches(&s.by_task(&task()).unwrap());
        let check = ex.tool_check(std::path::Path::new("/nonexistent-cas"), cand.id);
        let res = reconcile_one(&s, &task(), &pending[0], &check, "t3").unwrap();
        // Whatever the crash point, the reconciliation appends exactly one settling event.
        let ks = kinds(&s);
        assert_eq!(ks.len(), 4, "one settling event at {crash:?}: {ks:?}");
        assert!(
            ks[3] == "executed" || ks[3] == "unresolved",
            "settled at {crash:?}: {ks:?}"
        );
        // D5: the log itself holds at most one Executed for the idempotent dispatch, at every
        // crash point — the effect is never recorded twice.
        let executed = ks.iter().filter(|k| *k == "executed").count();
        assert!(
            executed <= 1,
            "at most one Executed per idempotent dispatch at {crash:?}: {ks:?}"
        );
        if crash == CrashPoint::Intent {
            assert_eq!(res, Resolution::UnresolvedIdempotent);
        } else {
            assert_eq!(res, Resolution::Executed);
            assert_eq!(executed, 1, "the effect is recorded once at {crash:?}");
        }
    }
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
// D1: the CLI loads its ToolCheck evidence from the log and the CAS, re-checks a blocked
// dispatch, and enforces retry-once for an idempotent unknown.
//
// Defect D1 (repair brief item 4): `reconcile_cmd::run` built `ToolCheck::new(cas_root)` with no
// digests, no seen keys and nothing idempotent, so every pending dispatch was "non-idempotent,
// blocked" and, once `Unresolved`, never re-checked (`pending_dispatches` treated any `Unresolved`
// as settled). These tests drive the built binary over a real on-disk store and CAS.
// ---------------------------------------------------------------------------------------------

/// A scratch directory under the target dir (never `/tmp`, which this host prunes).
fn d1_scratch(name: &str) -> std::path::PathBuf {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/d1-reconcile")
        .join(name);
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

/// D1: an idempotent (`RunApprovedTest`) dispatch whose log digest is in the CAS resolves to
/// `Executed` — the CLI loads the digest from the log's `Executed` event and confirms the artifact
/// is on disk. Before the fix the ToolCheck had no digest and no idempotency, so it stayed blocked.
#[test]
fn d1_reconcile_resolves_idempotent_dispatch_with_digest_in_cas() {
    let dir = d1_scratch("resolves-idempotent");
    let db = dir.join("tsr.db");
    let cas = dir.join("cas");
    std::fs::create_dir_all(&cas).unwrap();

    let cand = candidate("d1-idem");
    let dispatch = d1_seed_dispatched(&db, &cand);

    // The tool ran once (its log digest D is in the CAS) and the log recorded that `Executed`;
    // a crash then left a second `Dispatched` with no outcome. The retry's effect already happened.
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
    assert_eq!(code, Some(0), "stdout: {stdout}\nstderr: {stderr}");
    assert!(
        stdout.contains("executed"),
        "the retry resolves to Executed; stdout: {stdout}"
    );

    // The reconciliation landed: the retry is settled by an Executed carrying the CAS digest.
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
