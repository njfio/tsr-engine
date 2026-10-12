//! D10 + R2 + R3 (resolution): the resolution rules the M2 repair names.
//!
//! * **D10** — `intuition_executed` never became true: after `Proposed(Intuition)` → `Committed`
//!   → `Executed` → closed, `TaskFacts::from_events` left the flag false, so the
//!   `intuition_executed` named event resolved false instead of true.
//! * **R2** — a `TaskClosed { reason: "operator" }` (the lane's own cancellation close) must
//!   censor the common decision event as an operator stop, exactly as a `Stop { reason:
//!   "operator" }` does.
//! * **R3** — the hook writes `Stop { done | operator }`, never `TaskClosed`, so a hook-recorded
//!   forecast could never resolve: `resolve_common` waited for `TaskClosed`. A hook `Stop { done }`
//!   is the task's close; `Stop { operator }` is an operator stop.

use std::path::PathBuf;

use tsr_domain::{
    ActionKind, Candidate, CandidateId, Decision, Event, EventKind, Forecast, ForecastId,
    Forecaster, Hash, Horizon, Probability, Source, Support, TaskId, Utility,
};
use tsr_forecast::outcomes::common_decision_event_id;
use tsr_forecast::resolve::{resolve, Resolution, TaskFacts};
use tsr_store::Store;

fn task(seed: u8) -> TaskId {
    TaskId(Hash::of(&[b"m2-resolution-task".as_slice(), &[seed]]))
}

fn candidate_id(seed: u8) -> CandidateId {
    CandidateId(Hash::of(&[b"m2-resolution-candidate".as_slice(), &[seed]]))
}

/// A forecast about the common decision event, by one forecaster.
fn common_forecast(seed: u8, source: Forecaster, raw: f64) -> Forecast {
    Forecast {
        id: ForecastId(Hash::of(&[b"m2-resolution-forecast".as_slice(), &[seed]])),
        event: common_decision_event_id(),
        horizon: Horizon::TaskClose,
        source,
        raw: Probability::new(raw).unwrap(),
        calibrated: None,
        support: Support { episodes: 1 },
        state_version: 0,
    }
}

/// A forecast about a named event by id.
fn named_forecast(seed: u8, event: tsr_domain::OutcomeDefId, raw: f64) -> Forecast {
    Forecast {
        id: ForecastId(Hash::of(&[b"m2-resolution-named".as_slice(), &[seed]])),
        event,
        horizon: Horizon::TaskClose,
        source: Forecaster::Generator,
        raw: Probability::new(raw).unwrap(),
        calibrated: None,
        support: Support { episodes: 1 },
        state_version: 0,
    }
}

fn append(store: &Store, t: &TaskId, parents: Vec<tsr_domain::EventId>, kind: EventKind) -> Event {
    let ev = Event::new(*t, parents, Source::Rules, kind, "unix:1");
    store.append(&ev).expect("append");
    ev
}

/// D10: `Proposed(Intuition)` → `Committed` → `Executed` → closed must resolve
/// `intuition_executed` true. On the current code `TaskFacts::from_events` never sets
/// `intuition_executed`, so the named event resolves false.
#[test]
fn d10_intuition_executed_resolves_true() {
    let store = Store::open_in_memory().unwrap();
    let t = task(1);
    let intuition = candidate_id(1);

    let mut parents = Vec::new();
    let opened = append(
        &store,
        &t,
        parents.clone(),
        EventKind::TaskOpened { task: t },
    );
    parents = vec![opened.id];

    let cand = Candidate::new(
        ActionKind::Inspect,
        Source::Intuition,
        Hash::of(&[b"intuition-args"]),
        parents.clone(),
        Vec::new(),
    )
    .expect("intuition candidate");
    let proposed = append(
        &store,
        &t,
        parents.clone(),
        EventKind::Proposed {
            candidate: cand.clone(),
        },
    );
    parents = vec![proposed.id];

    let committed = append(
        &store,
        &t,
        parents.clone(),
        EventKind::Committed {
            candidate: cand.id,
            decision: Decision::Selected {
                value: cand.id,
                utility: Utility {
                    p_success: 0.5,
                    value: 1.0,
                    cost: 0.0,
                    ev: 0.5,
                },
                propensity: 1.0,
                evidence: vec![proposed.id],
            },
        },
    );
    parents = vec![committed.id];

    let executed = append(
        &store,
        &t,
        parents.clone(),
        EventKind::Executed {
            candidate: cand.id,
            exit_code: Some(0),
            log_digest: Some(Hash::of(&[b"log"])),
            duration_ms: 10,
        },
    );
    parents = vec![executed.id];

    append(
        &store,
        &t,
        parents,
        EventKind::TaskClosed {
            task: t,
            reason: "done".to_string(),
        },
    );

    let events: Vec<Event> = store
        .by_task(&t)
        .unwrap()
        .into_iter()
        .map(|(_, e)| e)
        .collect();
    let facts = TaskFacts::from_events(&events);
    assert!(
        facts.intuition_executed,
        "D10: an Executed of the Intuition candidate sets intuition_executed"
    );

    let f = named_forecast(1, tsr_forecast::resolve::event_id("intuition_executed").unwrap(), 0.5);
    let r = resolve(&f, &facts);
    assert_eq!(
        r.resolution,
        Resolution::True,
        "D10: intuition_executed resolves true after the intuition candidate executed"
    );
    let _ = intuition;
}

/// R2: a `TaskClosed { reason: "operator" }` censors the common decision event as an operator
/// stop, even when the diff was accepted.
#[test]
fn r2_taskclosed_operator_censors() {
    let store = Store::open_in_memory().unwrap();
    let t = task(2);

    let mut parents = Vec::new();
    let opened = append(
        &store,
        &t,
        parents.clone(),
        EventKind::TaskOpened { task: t },
    );
    parents = vec![opened.id];

    let closed = append(
        &store,
        &t,
        parents,
        EventKind::TaskClosed {
            task: t,
            reason: "operator".to_string(),
        },
    );
    let _ = closed;

    let events: Vec<Event> = store
        .by_task(&t)
        .unwrap()
        .into_iter()
        .map(|(_, e)| e)
        .collect();
    let facts = TaskFacts::from_events(&events);
    assert!(
        facts.operator_stop,
        "R2: TaskClosed{{reason: operator}} is an operator stop"
    );

    let f = common_forecast(2, Forecaster::Generator, 0.8);
    let r = resolve(&f, &facts);
    assert_eq!(
        r.resolution,
        Resolution::Censored,
        "R2: TaskClosed{{operator}} censors the common decision event"
    );
}

/// R3: a hook `Stop { reason: "done" }` is the task's close, so a hook-recorded forecast about the
/// common decision event resolves true when the diff was accepted and the criteria were met.
#[test]
fn r3_hook_stop_done_resolves_forecast() {
    let store = Store::open_in_memory().unwrap();
    let t = task(3);

    let mut parents = Vec::new();
    let opened = append(
        &store,
        &t,
        parents.clone(),
        EventKind::TaskOpened { task: t },
    );
    parents = vec![opened.id];

    let stop = append(
        &store,
        &t,
        parents,
        EventKind::Stop {
            reason: "done".to_string(),
        },
    );
    let _ = stop;

    let events: Vec<Event> = store
        .by_task(&t)
        .unwrap()
        .into_iter()
        .map(|(_, e)| e)
        .collect();
    let mut facts = TaskFacts::from_events(&events);
    assert!(
        facts.closed,
        "R3: a hook Stop{{done}} closes the task"
    );
    assert!(
        facts.criteria_met,
        "R3: a hook Stop{{done}} means the criteria were met"
    );

    // The operator accepted the return: an accepted label recorded outside the log.
    facts = facts.with_label("accepted", events.last().unwrap().id);

    let f = common_forecast(3, Forecaster::Generator, 0.8);
    let r = resolve(&f, &facts);
    assert_eq!(
        r.resolution,
        Resolution::True,
        "R3: a hook Stop{{done}} resolves the common decision event true"
    );
}

/// R3 (continued): the real M0 session log, copied to a temp dir, plus an accepted operator label,
/// must resolve its common-event forecast true. The hook wrote `Stop { done }`, never `TaskClosed`.
#[test]
fn r3_hook_stop_done_resolves_forecast_over_m0_session() {
    // Copy the frozen M0 session log to a temp dir; never touch fixtures/m0 itself.
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/m0/claude-session.events.jsonl");
    let dir = std::env::temp_dir().join(format!("m2-resolution-m0-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let copy = dir.join("claude-session.events.jsonl");
    std::fs::copy(&src, &copy).expect("copy the M0 session log");

    // Load the copied log into a store.
    let db = dir.join("tsr.db");
    let store = Store::open(&db).unwrap();
    let text = std::fs::read_to_string(&copy).unwrap();
    let mut task: Option<TaskId> = None;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(line).unwrap();
        let ev: Event = serde_json::from_value(v["event"].clone()).unwrap();
        task = Some(ev.task);
        store.append(&ev).unwrap();
    }
    let t = task.expect("the session has a task");

    let events: Vec<Event> = store
        .by_task(&t)
        .unwrap()
        .into_iter()
        .map(|(_, e)| e)
        .collect();
    let mut facts = TaskFacts::from_events(&events);
    // The operator accepted the return: an accepted label recorded outside the log.
    facts = facts.with_label("accepted", events.last().unwrap().id);

    assert!(
        facts.closed,
        "R3: the hook's Stop{{done}} closes the session task"
    );

    let f = common_forecast(4, Forecaster::Generator, 0.8);
    let r = resolve(&f, &facts);
    assert_eq!(
        r.resolution,
        Resolution::True,
        "R3: the M0 session's common-event forecast resolves true once Stop{{done}} closes it"
    );
}