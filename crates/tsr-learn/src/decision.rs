//! The common decision event as a **pure view** over the log (PRD §B8, §B7).
//! A1/A3/A11: events are indexed within their own task, and a forecast must precede the commitment.
//!
//! One row per commitment of a gated action — chosen actions only, so an unchosen `Proposed`
//! candidate is never a row. Each row carries the task, the commitment's event id and its 0-based
//! index in the task's log, the action kind, and every forecast on the common event paired to it by
//! `tsr_cli::reconstruct`'s rule: the forecast names the committed candidate **and** no event other
//! than a `Forecast` lies between it and the commitment in the task's log (F5).
//!
//! The view is pure: the same events yield the same rows, and events appended after a commitment
//! change none of its features. Nothing here reads `Committed.decision.utility` — main's hook writes
//! a placeholder `p_success` of 0.5 there, and reading it would fabricate a feature.

use serde::{Deserialize, Serialize};
use tsr_domain::{
    ActionKind, CandidateId, Event, EventId, EventKind, Forecast, Forecaster, TaskId,
};

use crate::decision_event_id;

/// A gated action: the three kinds whose commitment is forecast-gated (B5).
pub fn is_gated(kind: ActionKind) -> bool {
    matches!(
        kind,
        ActionKind::RunApprovedTest | ActionKind::ProposePatch | ActionKind::RequestApproval
    )
}

/// One row of the common decision event: a chosen, committed gated action.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DecisionRow {
    /// The task the commitment belongs to.
    pub task: TaskId,
    /// The commitment's event id.
    pub commitment: EventId,
    /// The commitment's 0-based index in the task's log.
    pub index: u64,
    /// The committed action's kind.
    pub action: ActionKind,
    /// Every forecast on the common event paired to this commitment, in log order.
    pub forecasts: Vec<Forecast>,
}

impl DecisionRow {
    /// The generator's raw probability on the common event, if the generator forecast this row.
    pub fn generator_raw(&self) -> Option<f64> {
        self.forecasts
            .iter()
            .find(|f| f.source == Forecaster::Generator)
            .map(|f| f.raw.get())
    }

    /// The operator's raw probability on the common event, if the operator forecast this row.
    pub fn operator_raw(&self) -> Option<f64> {
        self.forecasts
            .iter()
            .find(|f| f.source == Forecaster::Operator)
            .map(|f| f.raw.get())
    }

    /// A controller forecast already logged on the common event, if any (scored separately).
    pub fn controller_logged(&self) -> Option<f64> {
        self.forecasts
            .iter()
            .find(|f| f.source == Forecaster::Controller)
            .map(|f| f.raw.get())
    }
}

/// Build the common decision event as a pure view over `events` (in sequence order).
///
/// A row is emitted for every `Committed` whose decision is `Selected` and whose candidate is a
/// gated action. An unchosen `Proposed` candidate — one never committed — yields no row.
///
/// **A1/A3/A11: the index is task-local.** Main's hook records a forecast's `state_version` as the
/// number of events in the task's own log (`hook.rs` `event_count`), and the brief says "0-based
/// index in the task's log". So each event is indexed within its own task, not by its position in
/// the whole session log: a session that holds several tasks would otherwise shift every task after
/// the first. The commitment's `index` is its 0-based position in its task's log.
///
/// A forecast is paired when it is on the common event, names the committed candidate, and no event
/// other than a `Forecast` lies between it and the commitment in the task's log (F5: forecasts may
/// follow one another, so a generator forecast followed by an operator forecast both pair). A
/// forecast appended after the commitment — after the execute or the close — never pairs (§B9's
/// post-execution forecast edit).
pub fn decision_rows(events: &[(i64, Event)]) -> Vec<DecisionRow> {
    use std::collections::HashMap;

    let common = decision_event_id();

    // Per task: the candidate kinds proposed, and the forecasts recorded, in log order. Each
    // forecast is stored with its 0-based index in the task's own log, so the pairing rule can
    // require it to precede the commitment.
    let mut kinds: HashMap<CandidateId, ActionKind> = HashMap::new();
    // A1/A3/A11: keyed by (task, candidate), not candidate alone. Synthetic candidates are minted
    // from a hash of (kind, args), so `mint(RunApprovedTest, "step_3")` is the same id in every
    // task; keying by candidate alone would pair a commitment with another task's forecast.
    let mut forecasts: HashMap<(TaskId, CandidateId), Vec<(u64, Forecast)>> = HashMap::new();
    // The 0-based index of each event within its own task's log.
    let mut event_index: HashMap<EventId, u64> = HashMap::new();
    // The running count of events seen so far for each task.
    let mut task_len: HashMap<TaskId, u64> = HashMap::new();
    // F5: per task, whether the event at each 0-based index is a `Forecast`. The pairing rule is
    // "no event other than a Forecast lies between the forecast and the commitment", so the paired
    // forecasts are the contiguous run of Forecasts immediately before the commitment.
    let mut is_forecast: HashMap<TaskId, Vec<bool>> = HashMap::new();

    for (_, e) in events {
        let local = task_len.entry(e.task).or_insert(0);
        event_index.insert(e.id, *local);
        *local += 1;
        let flags = is_forecast.entry(e.task).or_default();
        flags.push(matches!(e.kind, EventKind::Forecast { .. }));
        match &e.kind {
            EventKind::Proposed { candidate } => {
                kinds.insert(candidate.id, candidate.kind);
            }
            EventKind::Forecast {
                forecast,
                candidate,
            } => {
                if forecast.event == common {
                    forecasts
                        .entry((e.task, *candidate))
                        .or_default()
                        .push((event_index[&e.id], forecast.clone()));
                }
            }
            _ => {}
        }
    }

    let mut rows = Vec::new();
    for (_, e) in events {
        let EventKind::Committed {
            candidate,
            decision,
        } = &e.kind
        else {
            continue;
        };
        // Chosen actions only: an unchosen Proposed candidate is never a row.
        let tsr_domain::Decision::Selected { value, .. } = decision else {
            continue;
        };
        // The decision's selected value must be the committed candidate.
        if value != candidate {
            continue;
        }
        let Some(kind) = kinds.get(candidate).copied() else {
            continue;
        };
        if !is_gated(kind) {
            continue;
        }
        let index = event_index.get(&e.id).copied().unwrap_or(0);
        // F5: the paired forecasts are the contiguous run of Forecasts immediately before the
        // commitment — a forecast pairs when no event other than a Forecast lies between it and the
        // commitment (forecasts may follow one another). Walk back from the commitment while the
        // preceding event is a Forecast; those indices are the paired positions.
        let flags = is_forecast.get(&e.task);
        let mut paired_positions: Vec<u64> = Vec::new();
        let mut pos = index;
        while pos > 0 {
            let prev = pos - 1;
            if flags
                .and_then(|f| f.get(prev as usize))
                .copied()
                .unwrap_or(false)
            {
                paired_positions.push(prev);
                pos = prev;
            } else {
                break;
            }
        }
        let paired: Vec<Forecast> = forecasts
            .get(&(e.task, *candidate))
            .map(|fs| {
                fs.iter()
                    // The forecast must precede the commitment and lie in the contiguous run of
                    // Forecasts immediately before it.
                    .filter(|(at, _)| paired_positions.contains(at))
                    .map(|(_, f)| f.clone())
                    .collect()
            })
            .unwrap_or_default();
        rows.push(DecisionRow {
            task: e.task,
            commitment: e.id,
            index,
            action: kind,
            forecasts: paired,
        });
    }
    rows
}

/// The diff label of a review (B7's `diff_accepted`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiffLabel {
    Accepted,
    Rejected,
    Disputed,
}

/// A label record: the review outcome for a task, from the labels input (JSONL).
///
/// `diff_label: null` means the task was never reviewed. `disputed` resolves false, as lane 04
/// resolves `diff_accepted`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LabelRecord {
    pub task: TaskId,
    pub criteria_met: bool,
    pub diff_label: Option<DiffLabel>,
    pub reviewer: String,
    pub recorded_at: String,
}

/// The resolved label of a task's common decision event (B7).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Label {
    /// The task closed with criteria met, the promoted diff accepted and no budget ceiling breached.
    True,
    /// It closed with any of the three failing.
    False,
    /// An operator stop (`TaskClosed` or `Stop` with reason `operator`), or never reviewed.
    Censored,
    /// Still open.
    Unresolved,
}

/// A task's resolved label with the evidence that decided it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskLabel {
    pub task: TaskId,
    pub label: Label,
    /// The event ids that decided the label, in log order.
    pub evidence: Vec<EventId>,
}

/// Resolve `task_accepted_in_budget@v1` for every task in `events` per its §B7 row.
///
/// - `true` when the task closed with criteria met, the promoted diff accepted and no budget
///   ceiling breached;
/// - `false` when it closed with any of the three failing;
/// - `censored` on an operator stop (`TaskClosed` or `Stop` with reason `operator`) or when never
///   reviewed;
/// - `unresolved` while open.
///
/// Closing and budget come from the log (`TaskClosed`, `Stop { reason: "budget" }`); criteria-met
/// and the diff label come from `labels`. A record whose reviewer is not the operator is refused
/// with [`LearnError::NonOperatorLabel`] (B8: labels only after review).
pub fn resolve_labels(
    events: &[(i64, Event)],
    labels: &[LabelRecord],
    operator: &str,
) -> Result<Vec<TaskLabel>, crate::LearnError> {
    use std::collections::HashMap;

    // Refuse any label not recorded by the operator.
    for r in labels {
        if r.reviewer != operator {
            return Err(crate::LearnError::NonOperatorLabel {
                reviewer: r.reviewer.clone(),
            });
        }
    }
    let by_task: HashMap<TaskId, &LabelRecord> = labels.iter().map(|r| (r.task, r)).collect();

    // Per task: did it close, did it breach budget, did it stop on operator, and the deciding ids.
    struct Facts {
        closed: Option<EventId>,
        budget: Option<EventId>,
        operator_stop: Option<EventId>,
    }
    let mut facts: HashMap<TaskId, Facts> = HashMap::new();
    let mut order: Vec<TaskId> = Vec::new();
    for (_, e) in events {
        let f = facts.entry(e.task).or_insert_with(|| {
            order.push(e.task);
            Facts {
                closed: None,
                budget: None,
                operator_stop: None,
            }
        });
        match &e.kind {
            EventKind::TaskClosed { reason, .. } => {
                if f.closed.is_none() {
                    f.closed = Some(e.id);
                }
                if reason == "operator" && f.operator_stop.is_none() {
                    f.operator_stop = Some(e.id);
                }
            }
            EventKind::Stop { reason } => {
                if reason == "budget" && f.budget.is_none() {
                    f.budget = Some(e.id);
                }
                if reason == "operator" && f.operator_stop.is_none() {
                    f.operator_stop = Some(e.id);
                }
            }
            _ => {}
        }
    }

    let mut out = Vec::new();
    for task in order {
        let f = &facts[&task];
        let rec = by_task.get(&task);
        let mut evidence = Vec::new();

        // Censored: an operator stop, or never reviewed.
        if let Some(id) = f.operator_stop {
            evidence.push(id);
            out.push(TaskLabel {
                task,
                label: Label::Censored,
                evidence,
            });
            continue;
        }
        let reviewed = rec.and_then(|r| r.diff_label).is_some();
        if !reviewed {
            // Never reviewed: censored if closed, unresolved while open.
            let label = if f.closed.is_some() {
                Label::Censored
            } else {
                Label::Unresolved
            };
            if let Some(id) = f.closed {
                evidence.push(id);
            }
            out.push(TaskLabel {
                task,
                label,
                evidence,
            });
            continue;
        }

        // Reviewed. Unresolved while open.
        let Some(closed_id) = f.closed else {
            out.push(TaskLabel {
                task,
                label: Label::Unresolved,
                evidence,
            });
            continue;
        };
        evidence.push(closed_id);

        let rec = rec.expect("reviewed implies a record");
        let diff_ok = rec.diff_label == Some(DiffLabel::Accepted);
        let budget_ok = f.budget.is_none();
        if let Some(id) = f.budget {
            evidence.push(id);
        }
        let label = if rec.criteria_met && diff_ok && budget_ok {
            Label::True
        } else {
            Label::False
        };
        out.push(TaskLabel {
            task,
            label,
            evidence,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tsr_domain::{
        Candidate, CandidateId, Decision, Event, EventKind, Forecast, Forecaster, Hash, Horizon,
        Probability, Source, TaskId, Utility,
    };

    fn task() -> TaskId {
        TaskId(Hash::of(&[b"task-1"]))
    }

    fn candidate(kind: ActionKind, tag: &[u8]) -> Candidate {
        Candidate::new(
            kind,
            Source::Generator,
            Hash::of(&[tag]),
            Vec::new(),
            Vec::new(),
        )
        .unwrap()
    }

    fn ev(task: &TaskId, kind: EventKind, tag: &[u8]) -> Event {
        Event::new(
            *task,
            Vec::new(),
            Source::Rules,
            kind,
            "2024-01-01T00:00:00Z",
        )
        .with_tag(tag)
    }

    /// A tiny helper: `Event::new` hashes the payload, so two events with the same kind collide.
    /// We vary `recorded_at` to keep ids distinct where a test needs distinct events.
    trait WithTag {
        fn with_tag(self, tag: &[u8]) -> Self;
    }
    impl WithTag for Event {
        fn with_tag(mut self, tag: &[u8]) -> Self {
            self.recorded_at = format!("2024-01-01T00:00:00Z#{}", hex(tag));
            self
        }
    }
    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    fn forecast(cid: CandidateId, p: f64, version: u64, source: Forecaster) -> EventKind {
        EventKind::Forecast {
            forecast: Forecast::new(
                decision_event_id(),
                Horizon::ThisDispatch,
                source,
                Probability::new(p).unwrap(),
                version,
            ),
            candidate: cid,
        }
    }

    fn committed(cid: CandidateId, utility: f64) -> EventKind {
        EventKind::Committed {
            candidate: cid,
            decision: Decision::Selected {
                value: cid,
                utility: Utility {
                    p_success: utility,
                    value: 0.0,
                    cost: 0.0,
                    ev: 0.0,
                },
                propensity: 1.0,
                evidence: Vec::new(),
            },
        }
    }

    /// A gated commitment with a paired generator forecast at the commitment's index.
    fn gated_session() -> (Vec<(i64, Event)>, CandidateId) {
        let t = task();
        let c = candidate(ActionKind::ProposePatch, b"patch");
        let cid = c.id;
        let events = vec![
            (1, ev(&t, EventKind::TaskOpened { task: t }, b"open")),
            (2, ev(&t, EventKind::Proposed { candidate: c }, b"prop")),
            // index of the commitment will be 3 (0-based), so state_version = 3.
            (
                3,
                ev(&t, forecast(cid, 0.9, 3, Forecaster::Generator), b"fc"),
            ),
            (4, ev(&t, committed(cid, 0.5), b"commit")),
        ];
        (events, cid)
    }

    #[test]
    fn one_row_per_chosen_gated_commitment() {
        let (events, cid) = gated_session();
        let rows = decision_rows(&events);
        assert_eq!(rows.len(), 1);
        let r = &rows[0];
        assert_eq!(r.task, task());
        assert_eq!(r.index, 3);
        assert_eq!(r.action, ActionKind::ProposePatch);
        assert_eq!(r.commitment, events[3].1.id);
        assert_eq!(r.forecasts.len(), 1);
        assert_eq!(r.forecasts[0].raw.get(), 0.9);
        assert_eq!(r.generator_raw(), Some(0.9));
        let _ = cid;
    }

    #[test]
    fn unchosen_proposed_candidate_is_never_a_row() {
        let t = task();
        let chosen = candidate(ActionKind::ProposePatch, b"chosen");
        let unchosen = candidate(ActionKind::ProposePatch, b"unchosen");
        let events = vec![
            (1, ev(&t, EventKind::TaskOpened { task: t }, b"open")),
            (
                2,
                ev(
                    &t,
                    EventKind::Proposed {
                        candidate: chosen.clone(),
                    },
                    b"p1",
                ),
            ),
            (
                3,
                ev(
                    &t,
                    EventKind::Proposed {
                        candidate: unchosen.clone(),
                    },
                    b"p2",
                ),
            ),
            (
                4,
                ev(
                    &t,
                    forecast(chosen.id, 0.7, 3, Forecaster::Generator),
                    b"fc",
                ),
            ),
            (5, ev(&t, committed(chosen.id, 0.5), b"commit")),
        ];
        let rows = decision_rows(&events);
        assert_eq!(rows.len(), 1, "only the chosen candidate is a row");
        assert_eq!(rows[0].commitment, events[4].1.id);
    }

    #[test]
    fn abstain_is_not_a_row() {
        let t = task();
        let c = candidate(ActionKind::ProposePatch, b"p");
        let events = vec![
            (1, ev(&t, EventKind::TaskOpened { task: t }, b"open")),
            (
                2,
                ev(
                    &t,
                    EventKind::Proposed {
                        candidate: c.clone(),
                    },
                    b"prop",
                ),
            ),
            (
                3,
                ev(
                    &t,
                    EventKind::Committed {
                        candidate: c.id,
                        decision: Decision::Abstain {
                            reason: tsr_domain::AbstainReason::NoEligibleCandidate,
                            next: tsr_domain::SafeNext::Inspect,
                        },
                    },
                    b"abstain",
                ),
            ),
        ];
        assert!(decision_rows(&events).is_empty());
    }

    #[test]
    fn non_gated_commitment_is_not_a_row() {
        let t = task();
        let c = candidate(ActionKind::Inspect, b"inspect");
        let events = vec![
            (1, ev(&t, EventKind::TaskOpened { task: t }, b"open")),
            (
                2,
                ev(
                    &t,
                    EventKind::Proposed {
                        candidate: c.clone(),
                    },
                    b"prop",
                ),
            ),
            (3, ev(&t, committed(c.id, 0.5), b"commit")),
        ];
        assert!(decision_rows(&events).is_empty());
    }

    #[test]
    fn a_forecast_separated_by_a_non_forecast_event_is_not_paired() {
        let t = task();
        let c = candidate(ActionKind::ProposePatch, b"p");
        let events = vec![
            (1, ev(&t, EventKind::TaskOpened { task: t }, b"open")),
            (
                2,
                ev(
                    &t,
                    EventKind::Proposed {
                        candidate: c.clone(),
                    },
                    b"prop",
                ),
            ),
            // The forecast is at index 2; the commitment is at index 4. A non-Forecast event (an
            // approval) sits between them, so the forecast does not pair (F5: only Forecasts may
            // lie between a forecast and its commitment).
            (
                3,
                ev(&t, forecast(c.id, 0.9, 3, Forecaster::Generator), b"fc"),
            ),
            (
                4,
                ev(
                    &t,
                    EventKind::Approved {
                        candidate: c.id,
                        scope: "task".into(),
                        expires_at: "2024-01-02T00:00:00Z".into(),
                    },
                    b"approve",
                ),
            ),
            (5, ev(&t, committed(c.id, 0.5), b"commit")),
        ];
        let rows = decision_rows(&events);
        assert_eq!(rows.len(), 1);
        assert!(
            rows[0].forecasts.is_empty(),
            "a non-Forecast event between the forecast and the commitment breaks pairing"
        );
    }

    // ---- A1/A3/A11: the index is task-local, and a forecast must precede the commitment ----

    /// A1: two tasks in one session. Main's hook records a forecast's `state_version` as the number
    /// of events in the **task's own** log, so each task's commitment is paired by its task-local
    /// index. Under the old session-global index the second task's forecast (task-local 3) would be
    /// compared against its session-global index (7) and never pair.
    #[test]
    fn a1_two_tasks_in_one_session_pair_by_task_local_index() {
        let t1 = TaskId(Hash::of(&[b"task-1"]));
        let t2 = TaskId(Hash::of(&[b"task-2"]));
        let c1 = candidate(ActionKind::ProposePatch, b"p1");
        let c2 = candidate(ActionKind::RunApprovedTest, b"p2");
        let events = vec![
            // Task 1: open(0), propose(1), forecast(2), commit(3).
            (1, ev(&t1, EventKind::TaskOpened { task: t1 }, b"o1")),
            (
                2,
                ev(
                    &t1,
                    EventKind::Proposed {
                        candidate: c1.clone(),
                    },
                    b"p1",
                ),
            ),
            (
                3,
                ev(&t1, forecast(c1.id, 0.9, 3, Forecaster::Generator), b"f1"),
            ),
            (4, ev(&t1, committed(c1.id, 0.5), b"c1")),
            // Task 2: open(0), propose(1), forecast(2), commit(3) — task-local, not 4..7.
            (5, ev(&t2, EventKind::TaskOpened { task: t2 }, b"o2")),
            (
                6,
                ev(
                    &t2,
                    EventKind::Proposed {
                        candidate: c2.clone(),
                    },
                    b"p2",
                ),
            ),
            (
                7,
                ev(&t2, forecast(c2.id, 0.8, 3, Forecaster::Generator), b"f2"),
            ),
            (8, ev(&t2, committed(c2.id, 0.5), b"c2")),
        ];
        let rows = decision_rows(&events);
        assert_eq!(rows.len(), 2, "one row per task");
        assert_eq!(rows[0].task, t1);
        assert_eq!(rows[0].index, 3, "task-local index");
        assert_eq!(rows[0].generator_raw(), Some(0.9));
        assert_eq!(rows[1].task, t2);
        assert_eq!(rows[1].index, 3, "task-local index, not session-global 7");
        assert_eq!(
            rows[1].generator_raw(),
            Some(0.8),
            "the second task's forecast pairs by its task-local index"
        );
    }

    /// A3: a forecast appended **after** the commitment never pairs, even at the right version.
    #[test]
    fn a3_a_forecast_after_the_commitment_never_pairs() {
        let t = task();
        let c = candidate(ActionKind::ProposePatch, b"p");
        let events = vec![
            (1, ev(&t, EventKind::TaskOpened { task: t }, b"open")),
            (
                2,
                ev(
                    &t,
                    EventKind::Proposed {
                        candidate: c.clone(),
                    },
                    b"prop",
                ),
            ),
            (3, ev(&t, committed(c.id, 0.5), b"commit")),
            // The commitment's index is 2; this forecast carries version 2 but is recorded after it.
            (
                4,
                ev(&t, forecast(c.id, 0.9, 2, Forecaster::Generator), b"late"),
            ),
        ];
        let rows = decision_rows(&events);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].index, 2);
        assert!(
            rows[0].forecasts.is_empty(),
            "a forecast recorded after the commitment never pairs (§B9 post-execution edit)"
        );
    }

    /// A11: a forecast appended after the execute (or the close) never pairs either.
    #[test]
    fn a11_a_forecast_after_the_execute_never_pairs() {
        let t = task();
        let c = candidate(ActionKind::RunApprovedTest, b"t");
        let events = vec![
            (1, ev(&t, EventKind::TaskOpened { task: t }, b"open")),
            (
                2,
                ev(
                    &t,
                    EventKind::Proposed {
                        candidate: c.clone(),
                    },
                    b"prop",
                ),
            ),
            (3, ev(&t, committed(c.id, 0.5), b"commit")),
            (
                4,
                ev(
                    &t,
                    EventKind::Executed {
                        candidate: c.id,
                        exit_code: Some(0),
                        log_digest: None,
                        duration_ms: 1,
                    },
                    b"exec",
                ),
            ),
            (
                5,
                ev(&t, forecast(c.id, 0.9, 2, Forecaster::Generator), b"late"),
            ),
            (
                6,
                ev(
                    &t,
                    EventKind::TaskClosed {
                        task: t,
                        reason: "done".into(),
                    },
                    b"close",
                ),
            ),
        ];
        let rows = decision_rows(&events);
        assert_eq!(rows.len(), 1);
        assert!(
            rows[0].forecasts.is_empty(),
            "a forecast after the execute or the close never pairs"
        );
    }

    #[test]
    fn purity_same_events_same_rows_and_later_events_change_nothing() {
        let (events, _) = gated_session();
        let a = decision_rows(&events);
        let b = decision_rows(&events);
        assert_eq!(a, b, "same events -> same rows");

        // Append events after the commitment: the row is unchanged.
        let t = task();
        let mut later = events.clone();
        later.push((
            5,
            ev(
                &t,
                EventKind::Stop {
                    reason: "done".into(),
                },
                b"stop",
            ),
        ));
        later.push((
            6,
            ev(
                &t,
                EventKind::TaskClosed {
                    task: t,
                    reason: "done".into(),
                },
                b"closed",
            ),
        ));
        let c = decision_rows(&later);
        assert_eq!(
            a, c,
            "events after a commitment change none of its features"
        );
    }

    #[test]
    fn changing_the_placeholder_utility_changes_no_feature() {
        let t = task();
        let c = candidate(ActionKind::ProposePatch, b"p");
        let mk = |utility: f64| {
            vec![
                (1, ev(&t, EventKind::TaskOpened { task: t }, b"open")),
                (
                    2,
                    ev(
                        &t,
                        EventKind::Proposed {
                            candidate: c.clone(),
                        },
                        b"prop",
                    ),
                ),
                (
                    3,
                    ev(&t, forecast(c.id, 0.9, 3, Forecaster::Generator), b"fc"),
                ),
                (4, ev(&t, committed(c.id, utility), b"commit")),
            ]
        };
        let a = decision_rows(&mk(0.5));
        let b = decision_rows(&mk(0.99));
        // The commitment's event id hashes the decision, so it differs; the *features* do not.
        assert_eq!(a.len(), 1);
        assert_eq!(b.len(), 1);
        assert_eq!(a[0].index, b[0].index);
        assert_eq!(a[0].action, b[0].action);
        assert_eq!(a[0].forecasts, b[0].forecasts);
        assert_eq!(a[0].task, b[0].task);
        assert_eq!(a[0].generator_raw(), b[0].generator_raw());
    }

    // ---- label resolution (B7) ----

    fn label_rec(
        task: TaskId,
        criteria_met: bool,
        diff: Option<DiffLabel>,
        reviewer: &str,
    ) -> LabelRecord {
        LabelRecord {
            task,
            criteria_met,
            diff_label: diff,
            reviewer: reviewer.to_string(),
            recorded_at: "2024-01-02T00:00:00Z".into(),
        }
    }

    fn closed(t: &TaskId) -> (i64, Event) {
        (
            1,
            ev(
                t,
                EventKind::TaskClosed {
                    task: *t,
                    reason: "done".into(),
                },
                b"closed",
            ),
        )
    }

    #[test]
    fn label_true_when_all_three_hold() {
        let t = task();
        let events = vec![closed(&t)];
        let labels = vec![label_rec(t, true, Some(DiffLabel::Accepted), "operator")];
        let out = resolve_labels(&events, &labels, "operator").unwrap();
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].label, Label::True);
        assert_eq!(out[0].evidence, vec![events[0].1.id]);
    }

    #[test]
    fn label_false_when_any_of_the_three_fails() {
        let t = task();
        // criteria not met
        let out = resolve_labels(
            &[closed(&t)],
            &[label_rec(t, false, Some(DiffLabel::Accepted), "operator")],
            "operator",
        )
        .unwrap();
        assert_eq!(out[0].label, Label::False);
        // diff rejected
        let out = resolve_labels(
            &[closed(&t)],
            &[label_rec(t, true, Some(DiffLabel::Rejected), "operator")],
            "operator",
        )
        .unwrap();
        assert_eq!(out[0].label, Label::False);
        // diff disputed resolves false
        let out = resolve_labels(
            &[closed(&t)],
            &[label_rec(t, true, Some(DiffLabel::Disputed), "operator")],
            "operator",
        )
        .unwrap();
        assert_eq!(out[0].label, Label::False);
        // budget breached
        let events = vec![
            closed(&t),
            (
                2,
                ev(
                    &t,
                    EventKind::Stop {
                        reason: "budget".into(),
                    },
                    b"budget",
                ),
            ),
        ];
        let out = resolve_labels(
            &events,
            &[label_rec(t, true, Some(DiffLabel::Accepted), "operator")],
            "operator",
        )
        .unwrap();
        assert_eq!(out[0].label, Label::False);
        assert!(out[0].evidence.contains(&events[1].1.id));
    }

    #[test]
    fn label_censored_on_operator_stop_or_never_reviewed() {
        let t = task();
        // operator stop via Stop
        let events = vec![
            closed(&t),
            (
                2,
                ev(
                    &t,
                    EventKind::Stop {
                        reason: "operator".into(),
                    },
                    b"op",
                ),
            ),
        ];
        let out = resolve_labels(
            &events,
            &[label_rec(t, true, Some(DiffLabel::Accepted), "operator")],
            "operator",
        )
        .unwrap();
        assert_eq!(out[0].label, Label::Censored);
        // operator stop via TaskClosed reason
        let events = vec![(
            1,
            ev(
                &t,
                EventKind::TaskClosed {
                    task: t,
                    reason: "operator".into(),
                },
                b"closed-op",
            ),
        )];
        let out = resolve_labels(&events, &[], "operator").unwrap();
        assert_eq!(out[0].label, Label::Censored);
        // never reviewed (diff_label null) but closed
        let out = resolve_labels(
            &[closed(&t)],
            &[label_rec(t, true, None, "operator")],
            "operator",
        )
        .unwrap();
        assert_eq!(out[0].label, Label::Censored);
        // never reviewed and no record at all, but closed
        let out = resolve_labels(&[closed(&t)], &[], "operator").unwrap();
        assert_eq!(out[0].label, Label::Censored);
    }

    #[test]
    fn label_unresolved_while_open() {
        let t = task();
        let events = vec![(1, ev(&t, EventKind::TaskOpened { task: t }, b"open"))];
        // open, reviewed
        let out = resolve_labels(
            &events,
            &[label_rec(t, true, Some(DiffLabel::Accepted), "operator")],
            "operator",
        )
        .unwrap();
        assert_eq!(out[0].label, Label::Unresolved);
        // open, never reviewed
        let out = resolve_labels(&events, &[], "operator").unwrap();
        assert_eq!(out[0].label, Label::Unresolved);
    }

    #[test]
    fn non_operator_label_is_refused() {
        let t = task();
        let events = vec![closed(&t)];
        let labels = vec![label_rec(t, true, Some(DiffLabel::Accepted), "generator")];
        let err = resolve_labels(&events, &labels, "operator").unwrap_err();
        assert!(matches!(
            err,
            crate::LearnError::NonOperatorLabel { ref reviewer } if reviewer == "generator"
        ));
    }
}
