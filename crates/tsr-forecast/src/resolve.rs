//! Resolution of a forecast against the task's returns.
//!
//! Given the events of a task, decide for a forecast whether its named event resolved true,
//! resolved false, was censored, or is still unresolved. The decision is a pure function of
//! the events: nothing is stored, and the same log always resolves the same way.

use serde::{Deserialize, Serialize};
use tsr_domain::{Event, EventKind, Forecast, Forecaster, Observation, OutcomeDefId, Source};

use crate::outcomes::{named_event_by_id, CensoringRule, NamedEvent, RequiredEvidence};

/// The resolution of one forecast against the events of its task.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Resolution {
    /// The named event happened.
    True,
    /// The named event did not happen.
    False,
    /// The event never resolved and the censoring rule applies; excluded from scoring.
    Censored,
    /// Not enough evidence yet to decide; excluded from scoring.
    Unresolved,
}

impl Resolution {
    /// The outcome as a number for scoring: `Some(1.0)` / `Some(0.0)` for a resolved event,
    /// `None` for censored or unresolved (which are never scored).
    pub fn outcome(self) -> Option<f64> {
        match self {
            Resolution::True => Some(1.0),
            Resolution::False => Some(0.0),
            Resolution::Censored | Resolution::Unresolved => None,
        }
    }

    /// A short engineering name, for the Mirror and for diagnostics.
    pub fn name(self) -> &'static str {
        match self {
            Resolution::True => "resolved_true",
            Resolution::False => "resolved_false",
            Resolution::Censored => "censored",
            Resolution::Unresolved => "unresolved",
        }
    }
}

/// A resolved forecast: the forecast, its resolution, and the events that decided it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResolvedForecast {
    pub forecast: Forecast,
    pub resolution: Resolution,
    /// The events the resolution was read from, in log order. Empty when unresolved.
    pub evidence: Vec<tsr_domain::EventId>,
}

/// The facts a resolution reads from a task's events. Built once per task by
/// [`TaskFacts::from_events`], then queried per forecast.
#[derive(Clone, Debug, Default)]
pub struct TaskFacts {
    /// `(candidate, exit_code)` for every `Executed` event.
    pub executed: Vec<(tsr_domain::CandidateId, Option<i32>)>,
    /// True when a `TaskClosed` event exists.
    pub closed: bool,
    /// The reason on the `TaskClosed` event, if any.
    pub close_reason: Option<String>,
    /// True when an `Unresolved` event exists (cancellation or a crash).
    pub unresolved: bool,
    /// True when an `Unresolved { reason: "cancelled" }` event exists.
    pub cancelled: bool,
    /// Operator labels seen on `Observed` events, in log order: `(label, event id)`.
    pub labels: Vec<(String, tsr_domain::EventId)>,
    /// True when an operator `Observed` event carries an `Experience` note.
    pub operator_experience: bool,
    /// True when a `Proposed` event carries an Intuition candidate.
    pub intuition_proposed: bool,
    /// The ids of the Intuition candidates a `Proposed` event named, so a later `Committed` can
    /// be recognised as selecting one.
    pub intuition_candidates: Vec<tsr_domain::CandidateId>,
    /// True when a `Committed` decision names an Intuition candidate.
    pub intuition_committed: bool,
    /// True when an `Executed` event exists for an Intuition candidate.
    pub intuition_executed: bool,
    /// True when a `Forecast` event exists for the common decision event.
    pub common_event_forecast: bool,
    /// True when a `Stop { reason: "operator" }` event exists: an operator stop for external
    /// reasons, which censors the common decision event (B7).
    pub operator_stop: bool,
    /// True when a `Stop { reason: "budget" }` or `TaskClosed { reason: "budget" }` event exists:
    /// a budget ceiling was breached, so the common decision event resolves false (B7).
    pub budget_breached: bool,
    /// True when the task closed with reason `done`: its criteria were met (B7).
    pub criteria_met: bool,
    /// The ids of the events that carried each fact, for evidence.
    pub evidence: Vec<tsr_domain::EventId>,
}

impl TaskFacts {
    /// Fold a task's events into the facts a resolution reads. Order is the log's order.
    pub fn from_events(events: &[Event]) -> Self {
        let mut f = TaskFacts::default();
        for ev in events {
            match &ev.kind {
                EventKind::Executed {
                    candidate,
                    exit_code,
                    ..
                } => {
                    f.executed.push((*candidate, *exit_code));
                    // D10: an execution of an Intuition candidate is the `intuition_executed`
                    // stage. The candidate's source is known from the `Proposed` event that named
                    // it, which precedes this `Executed` in log order.
                    if f.intuition_candidates.contains(candidate) {
                        f.intuition_executed = true;
                    }
                    f.evidence.push(ev.id);
                }
                EventKind::TaskClosed { reason, .. } => {
                    f.closed = true;
                    f.close_reason = Some(reason.clone());
                    if reason == "done" {
                        f.criteria_met = true;
                    }
                    if reason == "budget" {
                        f.budget_breached = true;
                    }
                    // R2: the lane's own cancellation close is an operator stop for external
                    // reasons, so it censors the common decision event exactly as a
                    // `Stop { reason: "operator" }` does.
                    if reason == "operator" {
                        f.operator_stop = true;
                    }
                    f.evidence.push(ev.id);
                }
                EventKind::Stop { reason } => {
                    // The Stop stage of the hook (B4/B5): `operator` is an external stop that
                    // censors the common event; `budget` is a breached ceiling that resolves it
                    // false. The hook records it and never blocks.
                    //
                    // R3: the hook writes `Stop { done | operator }` and never `TaskClosed`, so a
                    // hook-recorded forecast could never resolve. A `Stop { done }` is the task's
                    // close (its criteria were met); `Stop { operator }` is an operator stop.
                    if reason == "operator" {
                        f.operator_stop = true;
                    }
                    if reason == "done" {
                        f.closed = true;
                        f.close_reason = Some(reason.clone());
                        f.criteria_met = true;
                    }
                    if reason == "budget" {
                        f.budget_breached = true;
                    }
                    f.evidence.push(ev.id);
                }
                EventKind::Unresolved { reason, .. } => {
                    f.unresolved = true;
                    if reason == "cancelled" {
                        f.cancelled = true;
                    }
                    f.evidence.push(ev.id);
                }
                EventKind::OutcomeRecorded { record } => {
                    // `tsr review` records labels as `OutcomeRecorded` events; `resolve` reads the
                    // label *names* ("accepted"/"rejected"/"disputed"), so map the status the way
                    // the review command does.
                    let label = match record.status {
                        tsr_domain::LabelStatus::Observed => "accepted",
                        tsr_domain::LabelStatus::Rejected => "rejected",
                        tsr_domain::LabelStatus::Disputed => "disputed",
                        _ => record.status.name(),
                    };
                    f.labels.push((label.to_string(), ev.id));
                    f.evidence.push(ev.id);
                }
                EventKind::Observed { observation } => match observation {
                    Observation::Experience { .. } => {
                        f.operator_experience = true;
                        f.evidence.push(ev.id);
                    }
                    Observation::Return { .. } => {
                        f.evidence.push(ev.id);
                    }
                    _ => {}
                },
                EventKind::Proposed { candidate } => {
                    if candidate.source == Source::Intuition {
                        f.intuition_proposed = true;
                        f.intuition_candidates.push(candidate.id);
                        f.evidence.push(ev.id);
                    }
                }
                EventKind::Committed { candidate, .. } => {
                    // A commit of an Intuition candidate is the `intuition_selected` stage: the
                    // candidate's source is known from the `Proposed` event that named it.
                    if f.intuition_candidates.contains(candidate) {
                        f.intuition_committed = true;
                        f.evidence.push(ev.id);
                    }
                }
                EventKind::Forecast { forecast, .. } => {
                    if forecast.event == crate::outcomes::common_decision_event_id() {
                        f.common_event_forecast = true;
                    }
                    f.evidence.push(ev.id);
                }
                _ => {}
            }
        }
        f
    }

    /// Record an operator label seen outside the event log (the review command's own record).
    pub fn with_label(mut self, label: &str, id: tsr_domain::EventId) -> Self {
        self.labels.push((label.to_string(), id));
        self
    }

    /// Record that a commit named an Intuition candidate.
    pub fn with_intuition_commit(mut self, id: tsr_domain::EventId) -> Self {
        self.intuition_committed = true;
        self.evidence.push(id);
        self
    }

    /// Record that an Intuition candidate executed.
    pub fn with_intuition_execution(mut self, id: tsr_domain::EventId) -> Self {
        self.intuition_executed = true;
        self.evidence.push(id);
        self
    }

    /// The operator's label for the promoted diff, if one was recorded.
    pub fn label(&self) -> Option<&str> {
        self.labels.last().map(|(l, _)| l.as_str())
    }
}

/// Resolve one forecast against the facts of its task.
///
/// The rules are the §B7 table's, read as data from [`NamedEvent`]:
/// - `test_pass`: true when an `Executed` event carries exit code 0; false when one carries a
///   non-zero exit code; censored on cancellation; unresolved with no execution.
/// - `diff_accepted`: true when the operator labelled the diff `accepted`; false on
///   `rejected`; `disputed` is a resolution of false (the diff was modified); censored when
///   the task closed without a label.
/// - `task_accepted_in_budget`: true when the task closed, the diff was accepted and no
///   budget ceiling was breached; false when it closed rejected; censored on an operator stop
///   for external reasons; unresolved while the task is open.
/// - the four Intuition stages: true when the stage's evidence exists, false when the stage
///   was passed over, unresolved before the stage is reached.
pub fn resolve(forecast: &Forecast, facts: &TaskFacts) -> ResolvedForecast {
    let def = named_event_by_id(&forecast.event);
    let (resolution, evidence) = match def {
        None => (Resolution::Unresolved, Vec::new()),
        Some(def) => resolve_named(&def, forecast, facts),
    };
    ResolvedForecast {
        forecast: forecast.clone(),
        resolution,
        evidence,
    }
}

fn resolve_named(
    def: &NamedEvent,
    forecast: &Forecast,
    facts: &TaskFacts,
) -> (Resolution, Vec<tsr_domain::EventId>) {
    match def.name.as_str() {
        "test_pass" => resolve_test_pass(def, forecast, facts),
        "diff_accepted" => resolve_diff_accepted(def, facts),
        "task_accepted_in_budget" => resolve_common(def, facts),
        "intuition_offered" => resolve_stage(def, facts.intuition_proposed, facts),
        "intuition_selected" => resolve_stage(def, facts.intuition_committed, facts),
        "intuition_executed" => resolve_stage(def, facts.intuition_executed, facts),
        "intuition_observed" => {
            resolve_stage(def, facts.common_event_forecast && facts.closed, facts)
        }
        _ => (Resolution::Unresolved, Vec::new()),
    }
}

fn resolve_test_pass(
    def: &NamedEvent,
    _forecast: &Forecast,
    facts: &TaskFacts,
) -> (Resolution, Vec<tsr_domain::EventId>) {
    if facts.cancelled {
        return (Resolution::Censored, facts.evidence.clone());
    }
    match facts.executed.last() {
        Some((_, Some(0))) => (Resolution::True, facts.evidence.clone()),
        Some((_, Some(_))) => (Resolution::False, facts.evidence.clone()),
        Some((_, None)) => (Resolution::Unresolved, facts.evidence.clone()),
        None => {
            if facts.unresolved {
                (Resolution::Censored, facts.evidence.clone())
            } else {
                (Resolution::Unresolved, Vec::new())
            }
        }
    }
    .pipe_evidence(def, facts.closed)
}

fn resolve_diff_accepted(
    def: &NamedEvent,
    facts: &TaskFacts,
) -> (Resolution, Vec<tsr_domain::EventId>) {
    match facts.label() {
        Some("accepted") => (Resolution::True, facts.evidence.clone()),
        Some("rejected") | Some("disputed") => (Resolution::False, facts.evidence.clone()),
        _ => {
            if facts.closed {
                (Resolution::Censored, facts.evidence.clone())
            } else {
                (Resolution::Unresolved, Vec::new())
            }
        }
    }
    .pipe_evidence(def, facts.closed)
}

fn resolve_common(def: &NamedEvent, facts: &TaskFacts) -> (Resolution, Vec<tsr_domain::EventId>) {
    // §B7: censored on an operator stop for external reasons, whether or not the task has closed.
    if facts.operator_stop {
        return (Resolution::Censored, facts.evidence.clone()).pipe_evidence(def, facts.closed);
    }
    if !facts.closed {
        return (Resolution::Unresolved, Vec::new()).pipe_evidence(def, facts.closed);
    }
    match facts.label() {
        // True only when the criteria were met, the diff accepted and no budget ceiling breached.
        Some("accepted") if facts.criteria_met && !facts.budget_breached => {
            (Resolution::True, facts.evidence.clone())
        }
        // Accepted but the criteria were unmet or a ceiling was breached: the event did not happen.
        Some("accepted") => (Resolution::False, facts.evidence.clone()),
        Some("rejected") | Some("disputed") => (Resolution::False, facts.evidence.clone()),
        // Closed with no label: never reviewed, so censored.
        _ => (Resolution::Censored, facts.evidence.clone()),
    }
    .pipe_evidence(def, facts.closed)
}

fn resolve_stage(
    def: &NamedEvent,
    reached: bool,
    facts: &TaskFacts,
) -> (Resolution, Vec<tsr_domain::EventId>) {
    if reached {
        (Resolution::True, facts.evidence.clone())
    } else if facts.closed {
        (Resolution::False, facts.evidence.clone())
    } else {
        (Resolution::Unresolved, Vec::new())
    }
    .pipe_evidence(def, facts.closed)
}

/// A tiny helper so each arm can attach the definition's censoring rule without repeating it.
trait PipeEvidence {
    fn pipe_evidence(
        self,
        def: &NamedEvent,
        closed: bool,
    ) -> (Resolution, Vec<tsr_domain::EventId>);
}

impl PipeEvidence for (Resolution, Vec<tsr_domain::EventId>) {
    fn pipe_evidence(
        self,
        def: &NamedEvent,
        closed: bool,
    ) -> (Resolution, Vec<tsr_domain::EventId>) {
        let (mut r, ev) = self;
        // A censoring rule turns an unresolved record into a censored one, but only once the task
        // has closed: while it is still open the event is unresolved, not censored (B7). The rule
        // is data, so the mapping is explicit here. `NeverReviewed` and `OperatorStopExternal`
        // both censor; `TimeoutOrCancellation` censors a dispatch that never resolved;
        // `OfferNotSelected` does not censor (an offer passed over is recorded with what was
        // selected instead).
        if r == Resolution::Unresolved && closed {
            match def.censoring {
                CensoringRule::NeverReviewed
                | CensoringRule::OperatorStopExternal
                | CensoringRule::TimeoutOrCancellation => r = Resolution::Censored,
                CensoringRule::OfferNotSelected => {}
            }
        }
        (r, ev)
    }
}

/// The forecaster's name as it prints in the Mirror: `generator`, `controller`, `operator`,
/// or the external forecaster's own name.
pub fn forecaster_name(f: &Forecaster) -> String {
    match f {
        Forecaster::Generator => "generator".into(),
        Forecaster::Controller => "controller".into(),
        Forecaster::Operator => "operator".into(),
        Forecaster::External(name) => name.clone(),
    }
}

/// The evidence a named event requires, for diagnostics: which of the required evidence is
/// present in the facts.
pub fn missing_evidence(def: &NamedEvent, facts: &TaskFacts) -> Vec<&'static str> {
    let mut missing = Vec::new();
    for req in &def.required_evidence {
        let present = match req {
            RequiredEvidence::ExecutedExitCode => !facts.executed.is_empty(),
            RequiredEvidence::OperatorLabel => facts.label().is_some(),
            RequiredEvidence::TaskClosed => facts.closed,
            RequiredEvidence::IntuitionCandidate => facts.intuition_proposed,
            RequiredEvidence::IntuitionCommitted => facts.intuition_committed,
            RequiredEvidence::IntuitionExecuted => facts.intuition_executed,
            RequiredEvidence::CommonEventResolved => facts.common_event_forecast && facts.closed,
        };
        if !present {
            missing.push(req.name());
        }
    }
    missing
}

/// The id of a named event, for callers that only have the name.
pub fn event_id(name: &str) -> Option<OutcomeDefId> {
    crate::outcomes::named_event_id(name)
}
