//! D10 / D11 — the common decision event's resolution and the residual's sign.
//!
//! D10: `resolve_common` ignored the budget and the criteria, its operator-stop censoring branch
//! was unreachable, `pipe_evidence` did nothing, and the `Committed` arm never set
//! `intuition_committed` (so `intuition_selected`/`intuition_executed` resolved false at close).
//! D11: the residual was `observed - forecast`; the brief and PRD §B7 say `forecast - observed`.
//!
//! Every test builds a task's event log and resolves a forecast against it, so it exercises the
//! same path the CLI does.

use tsr_domain::{
    ActionKind, Candidate, Decision, Event, EventKind, Forecast, Forecaster, Hash, Horizon,
    LabelStatus, OutcomeRecord, Probability, Source, TaskId, Utility,
};
use tsr_forecast::outcomes::{common_decision_event_id, named_event_id};
use tsr_forecast::resolve::{resolve, Resolution, TaskFacts};
use tsr_forecast::score::residual_proxy;

fn task() -> TaskId {
    TaskId(Hash::of(&[b"d10-task"]))
}

fn stamp(i: usize) -> String {
    format!("2026-10-11T00:00:{i:02}Z")
}

/// Append an event to a log, chaining parents so the ids are distinct.
fn push(log: &mut Vec<Event>, kind: EventKind, source: Source) {
    let parents = log.last().map(|e| vec![e.id]).unwrap_or_default();
    let ev = Event::new(task(), parents, source, kind, stamp(log.len()));
    log.push(ev);
}

fn label(log: &mut Vec<Event>, status: LabelStatus) {
    let subject = log.last().map(|e| e.id).expect("a prior event");
    push(
        log,
        EventKind::OutcomeRecorded {
            record: OutcomeRecord {
                outcome: named_event_id("diff_accepted").unwrap(),
                subject,
                status,
                reviewer: Source::Operator,
                reason: None,
                evidence: None,
                supersedes: None,
                weight: None,
            },
        },
        Source::Operator,
    );
}

fn common_forecast() -> Forecast {
    Forecast::new(
        common_decision_event_id(),
        Horizon::TaskCloseDays(7),
        Forecaster::Generator,
        Probability::new(0.5).unwrap(),
        0,
    )
}

fn resolve_common(log: &[Event]) -> Resolution {
    let facts = TaskFacts::from_events(log);
    resolve(&common_forecast(), &facts).resolution
}

// ---------------------------------------------------------------------------------------------
// D10 — the §B7 table, one test per row.
// ---------------------------------------------------------------------------------------------

/// D10: true when the task closes with criteria met, the diff accepted, inside every budget.
#[test]
fn d10_common_true_when_closed_criteria_met_accepted_in_budget() {
    let mut log = Vec::new();
    push(
        &mut log,
        EventKind::TaskOpened { task: task() },
        Source::Rules,
    );
    label(&mut log, LabelStatus::Observed);
    push(
        &mut log,
        EventKind::TaskClosed {
            task: task(),
            reason: "done".into(),
        },
        Source::Rules,
    );
    assert_eq!(resolve_common(&log), Resolution::True);
}

/// D10: false when the task closes rejected.
#[test]
fn d10_common_false_when_closed_rejected() {
    let mut log = Vec::new();
    push(
        &mut log,
        EventKind::TaskOpened { task: task() },
        Source::Rules,
    );
    label(&mut log, LabelStatus::Rejected);
    push(
        &mut log,
        EventKind::TaskClosed {
            task: task(),
            reason: "done".into(),
        },
        Source::Rules,
    );
    assert_eq!(resolve_common(&log), Resolution::False);
}

/// D10: false when a budget ceiling was breached, even though the diff was accepted.
#[test]
fn d10_common_false_when_budget_ceiling_breached() {
    let mut log = Vec::new();
    push(
        &mut log,
        EventKind::TaskOpened { task: task() },
        Source::Rules,
    );
    label(&mut log, LabelStatus::Observed);
    push(
        &mut log,
        EventKind::Stop {
            reason: "budget".into(),
        },
        Source::Rules,
    );
    push(
        &mut log,
        EventKind::TaskClosed {
            task: task(),
            reason: "budget".into(),
        },
        Source::Rules,
    );
    assert_eq!(resolve_common(&log), Resolution::False);
}

/// D10: censored on an operator stop for external reasons (the branch was unreachable).
#[test]
fn d10_common_censored_on_operator_stop() {
    let mut log = Vec::new();
    push(
        &mut log,
        EventKind::TaskOpened { task: task() },
        Source::Rules,
    );
    push(
        &mut log,
        EventKind::Stop {
            reason: "operator".into(),
        },
        Source::Operator,
    );
    assert_eq!(resolve_common(&log), Resolution::Censored);
}

/// D10: censored when the task closed but was never reviewed.
#[test]
fn d10_common_censored_when_never_reviewed() {
    let mut log = Vec::new();
    push(
        &mut log,
        EventKind::TaskOpened { task: task() },
        Source::Rules,
    );
    push(
        &mut log,
        EventKind::TaskClosed {
            task: task(),
            reason: "done".into(),
        },
        Source::Rules,
    );
    assert_eq!(resolve_common(&log), Resolution::Censored);
}

/// D10: unresolved while the task is still open.
#[test]
fn d10_common_unresolved_while_open() {
    let mut log = Vec::new();
    push(
        &mut log,
        EventKind::TaskOpened { task: task() },
        Source::Rules,
    );
    assert_eq!(resolve_common(&log), Resolution::Unresolved);
}

/// D10: the `Committed` arm sets `intuition_committed`, so `intuition_selected` resolves true at
/// close when a commit named an Intuition candidate.
#[test]
fn d10_committed_arm_sets_intuition_committed() {
    let mut log = Vec::new();
    push(
        &mut log,
        EventKind::TaskOpened { task: task() },
        Source::Rules,
    );
    let candidate = Candidate::new(
        ActionKind::ProposePatch,
        Source::Intuition,
        Hash::of(&[b"intuition-args"]),
        Vec::new(),
        Vec::new(),
    )
    .expect("a valid candidate");
    push(
        &mut log,
        EventKind::Proposed {
            candidate: candidate.clone(),
        },
        Source::Intuition,
    );
    push(
        &mut log,
        EventKind::Committed {
            candidate: candidate.id,
            decision: Decision::Selected {
                value: candidate.id,
                utility: Utility {
                    p_success: 0.5,
                    value: 1.0,
                    cost: 0.0,
                    ev: 0.5,
                },
                propensity: 1.0,
                evidence: Vec::new(),
            },
        },
        Source::Operator,
    );
    push(
        &mut log,
        EventKind::TaskClosed {
            task: task(),
            reason: "done".into(),
        },
        Source::Rules,
    );
    let facts = TaskFacts::from_events(&log);
    assert!(
        facts.intuition_committed,
        "a Committed event naming an Intuition candidate must set intuition_committed"
    );
    let f = Forecast::new(
        named_event_id("intuition_selected").unwrap(),
        Horizon::TaskCloseDays(7),
        Forecaster::Generator,
        Probability::new(0.5).unwrap(),
        0,
    );
    assert_eq!(resolve(&f, &facts).resolution, Resolution::True);
}

/// D10: `pipe_evidence` applies the censoring rule — an unresolved-at-close with the
/// `never_reviewed` rule becomes censored, not left unresolved.
#[test]
fn d10_pipe_evidence_censors_never_reviewed() {
    // `diff_accepted` carries the `never_reviewed` rule; closed with no label must censor.
    let mut log = Vec::new();
    push(
        &mut log,
        EventKind::TaskOpened { task: task() },
        Source::Rules,
    );
    push(
        &mut log,
        EventKind::TaskClosed {
            task: task(),
            reason: "done".into(),
        },
        Source::Rules,
    );
    let facts = TaskFacts::from_events(&log);
    let f = Forecast::new(
        named_event_id("diff_accepted").unwrap(),
        Horizon::TaskCloseDays(7),
        Forecaster::Generator,
        Probability::new(0.5).unwrap(),
        0,
    );
    assert_eq!(resolve(&f, &facts).resolution, Resolution::Censored);
}

// ---------------------------------------------------------------------------------------------
// D11 — the residual's sign.
// ---------------------------------------------------------------------------------------------

/// D11: the residual is forecast minus observed, as the brief and PRD §B7 say.
#[test]
fn d11_residual_is_forecast_minus_observed() {
    // A forecast of 0.8 that resolved true: forecast - observed = 0.8 - 1.0 = -0.2.
    let mut log = Vec::new();
    push(
        &mut log,
        EventKind::TaskOpened { task: task() },
        Source::Rules,
    );
    label(&mut log, LabelStatus::Observed);
    push(
        &mut log,
        EventKind::TaskClosed {
            task: task(),
            reason: "done".into(),
        },
        Source::Rules,
    );
    let facts = TaskFacts::from_events(&log);
    let f = Forecast::new(
        common_decision_event_id(),
        Horizon::TaskCloseDays(7),
        Forecaster::Generator,
        Probability::new(0.8).unwrap(),
        0,
    );
    let r = resolve(&f, &facts);
    assert_eq!(r.resolution, Resolution::True);
    let proxy = residual_proxy(&r).expect("a resolved forecast has a residual");
    assert!(
        (proxy.value - (-0.2)).abs() < 1e-9,
        "residual must be forecast - observed = -0.2, got {}",
        proxy.value
    );
}
