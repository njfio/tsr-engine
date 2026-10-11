//! The three leakage probes (PRD §B9; lane 06 item 4).
//!
//! Each probe has a **planted-leak test** it catches and a **clean-fixture test** it passes:
//!
//! - **(a) post-action events** — every row's features are recomputed from the log truncated before
//!   the commitment, and any difference fails. Planted: an extractor reading the decision's own
//!   `Executed` exit code.
//! - **(b) label reach** — extractors receive no label, by type, and a single feature that perfectly
//!   separates the fit slice's labels fails, named. Planted: the label copied into a feature.
//! - **(c) the boundary** — no task, clone group or (under repo) lineage on two sides, and under
//!   time every held_out task is later than every fit and calibrate task. Planted: a forged split
//!   manifest.
//!
//! A caught probe exits 2 naming the probe and the row. Every read of a held_out row appends a line
//! to `<dir>/holdout-access.jsonl` (B9); the split digest covers assignments only, so logging never
//! changes it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use tsr_domain::{ActionKind, Event, EventId, TaskId};

use crate::dataset::{Dataset, Row, Side, SplitBy, TaskIndexRow};
use crate::features::{extract, FeatureSpec, FeatureVector};
use crate::LearnError;

/// Which probe caught a leak.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Probe {
    /// (a) post-action events.
    PostAction,
    /// (b) label reach.
    LabelReach,
    /// (c) the boundary.
    Boundary,
}

impl Probe {
    pub fn name(self) -> &'static str {
        match self {
            Probe::PostAction => "post_action_events",
            Probe::LabelReach => "label_reach",
            Probe::Boundary => "boundary",
        }
    }
}

/// A caught leak: the probe and the row it names.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Leak {
    pub probe: Probe,
    /// The row the leak was seen on (its id, hex), or a task id for a boundary leak.
    pub row: String,
    pub detail: String,
}

impl std::fmt::Display for Leak {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "leak probe {} caught on row {}: {}",
            self.probe.name(),
            self.row,
            self.detail
        )
    }
}

/// The result of running the probes: the leaks found (empty means clean).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProbeReport {
    pub leaks: Vec<Leak>,
}

impl ProbeReport {
    pub fn is_clean(&self) -> bool {
        self.leaks.is_empty()
    }
}

/// An extractor: a function from the pre-action events, the index row, the action and the
/// generator's raw probability on the common event to a feature vector. It receives **no label** —
/// by type, so a label cannot reach a feature.
pub type Extractor<'a> = &'a dyn Fn(
    &[(i64, Event)],
    &TaskIndexRow,
    ActionKind,
    Option<f64>,
) -> FeatureVector;

/// Probe (a): recompute every row's features from the log truncated before the commitment and fail
/// on any difference.
///
/// `events` is the full log; `index` the task index; `spec` the feature schema; `extract` the
/// extractor under test. A row whose recomputed features differ from the stored ones is a leak.
pub fn probe_post_action(
    ds: &Dataset,
    events: &[(i64, Event)],
    index: &[TaskIndexRow],
    spec: &FeatureSpec,
    extractor: Extractor<'_>,
) -> ProbeReport {
    let index_of: BTreeMap<TaskId, &TaskIndexRow> = index.iter().map(|r| (r.task, r)).collect();
    let mut by_task: BTreeMap<TaskId, Vec<(i64, Event)>> = BTreeMap::new();
    for (seq, e) in events {
        by_task.entry(e.task).or_default().push((*seq, e.clone()));
    }
    // The generator's raw p per commitment, by the same rule `decision_rows` uses.
    let generator_raw: BTreeMap<EventId, Option<f64>> = crate::decision::decision_rows(events)
        .into_iter()
        .map(|dr| (dr.commitment, dr.generator_raw()))
        .collect();

    let mut leaks = Vec::new();
    for row in &ds.rows {
        let Some(index_row) = index_of.get(&row.task) else {
            continue;
        };
        let task_events = &by_task[&row.task];
        let before: Vec<(i64, Event)> = task_events
            .iter()
            .take_while(|(_, e)| e.id != row.commitment)
            .cloned()
            .collect();
        let raw = generator_raw.get(&row.commitment).copied().flatten();
        let recomputed = extractor(&before, index_row, row.action, raw);
        if recomputed.values != row.features.values {
            leaks.push(Leak {
                probe: Probe::PostAction,
                row: row.id.to_hex(),
                detail: format!(
                    "features recomputed from the log truncated before the commitment differ \
                     (stored {} values, recomputed {} values)",
                    row.features.values.len(),
                    recomputed.values.len()
                ),
            });
        }
    }
    let _ = spec;
    ProbeReport { leaks }
}

/// Probe (b): a single feature that perfectly separates the fit slice's labels fails, named.
///
/// A feature separates perfectly when, over the fit rows with a resolved label, every row with
/// label `true` has one value and every row with label `false` has a different value. The feature
/// is named in the leak.
pub fn probe_label_reach(ds: &Dataset, spec: &FeatureSpec) -> ProbeReport {
    let fit: Vec<&Row> = ds
        .rows
        .iter()
        .filter(|r| r.side == Side::Fit && r.label.is_some())
        .collect();
    let mut leaks = Vec::new();
    if fit.is_empty() {
        return ProbeReport { leaks };
    }
    for (i, name) in spec.names.iter().enumerate() {
        let mut true_vals: BTreeSet<u64> = BTreeSet::new();
        let mut false_vals: BTreeSet<u64> = BTreeSet::new();
        for r in &fit {
            let v = r.features.values.get(i).copied().unwrap_or(0.0);
            let bits = v.to_bits();
            if r.label == Some(true) {
                true_vals.insert(bits);
            } else {
                false_vals.insert(bits);
            }
        }
        // Perfect separation: the two value sets are disjoint and each is a singleton.
        if true_vals.len() == 1
            && false_vals.len() == 1
            && true_vals.is_disjoint(&false_vals)
        {
            leaks.push(Leak {
                probe: Probe::LabelReach,
                row: format!("feature:{name}"),
                detail: format!(
                    "feature {name:?} perfectly separates the fit slice's labels \
                     (true = {}, false = {})",
                    f64::from_bits(*true_vals.iter().next().unwrap()),
                    f64::from_bits(*false_vals.iter().next().unwrap())
                ),
            });
        }
    }
    ProbeReport { leaks }
}

/// Probe (c): no task, clone group or (under repo) lineage on two sides, and under time every
/// held_out task is later than every fit and calibrate task.
pub fn probe_boundary(ds: &Dataset, index: &[TaskIndexRow]) -> ProbeReport {
    let index_of: BTreeMap<TaskId, &TaskIndexRow> = index.iter().map(|r| (r.task, r)).collect();
    let mut leaks = Vec::new();

    // No task on two sides.
    let mut task_sides: BTreeMap<TaskId, BTreeSet<Side>> = BTreeMap::new();
    for r in &ds.rows {
        task_sides.entry(r.task).or_default().insert(r.side);
    }
    for (task, sides) in &task_sides {
        if sides.len() > 1 {
            leaks.push(Leak {
                probe: Probe::Boundary,
                row: task.to_string(),
                detail: format!("task {task} appears on {} sides", sides.len()),
            });
        }
    }

    // No clone group on two sides.
    let mut group_sides: BTreeMap<String, BTreeSet<Side>> = BTreeMap::new();
    for r in &ds.rows {
        if let Some(g) = index_of.get(&r.task).and_then(|i| i.clone_group.clone()) {
            group_sides.entry(g).or_default().insert(r.side);
        }
    }
    for (g, sides) in &group_sides {
        if sides.len() > 1 {
            leaks.push(Leak {
                probe: Probe::Boundary,
                row: format!("clone_group:{g}"),
                detail: format!("clone group {g} appears on {} sides", sides.len()),
            });
        }
    }

    // Under repo, no lineage on two sides.
    if ds.split.by == SplitBy::Repo {
        let mut lineage_sides: BTreeMap<String, BTreeSet<Side>> = BTreeMap::new();
        for r in &ds.rows {
            lineage_sides.entry(r.lineage.clone()).or_default().insert(r.side);
        }
        for (l, sides) in &lineage_sides {
            if sides.len() > 1 {
                leaks.push(Leak {
                    probe: Probe::Boundary,
                    row: format!("lineage:{l}"),
                    detail: format!("lineage {l} appears on {} sides", sides.len()),
                });
            }
        }
    }

    // Under time, every held_out task is later than every fit and calibrate task.
    if ds.split.by == SplitBy::Time {
        let mut max_train: Option<String> = None;
        let mut min_hold: Option<String> = None;
        for r in &ds.rows {
            let Some(i) = index_of.get(&r.task) else {
                continue;
            };
            match r.side {
                Side::Fit | Side::Calibrate => {
                    max_train = Some(match &max_train {
                        Some(m) if *m >= i.timestamp => m.clone(),
                        _ => i.timestamp.clone(),
                    });
                }
                Side::HeldOut => {
                    min_hold = Some(match &min_hold {
                        Some(m) if *m <= i.timestamp => m.clone(),
                        _ => i.timestamp.clone(),
                    });
                }
            }
        }
        if let (Some(mt), Some(mh)) = (max_train, min_hold) {
            if mh <= mt {
                leaks.push(Leak {
                    probe: Probe::Boundary,
                    row: format!("time:{mh}"),
                    detail: format!(
                        "a held_out task ({mh}) is not later than every fit/calibrate task ({mt})"
                    ),
                });
            }
        }
    }

    ProbeReport { leaks }
}

/// Run all three probes over a dataset.
pub fn run_all(
    ds: &Dataset,
    events: &[(i64, Event)],
    index: &[TaskIndexRow],
    spec: &FeatureSpec,
    extractor: Extractor<'_>,
) -> ProbeReport {
    let mut leaks = Vec::new();
    leaks.extend(probe_post_action(ds, events, index, spec, extractor).leaks);
    leaks.extend(probe_label_reach(ds, spec).leaks);
    leaks.extend(probe_boundary(ds, index).leaks);
    ProbeReport { leaks }
}

/// The default extractor: the features@v1 extractor, which reads only pre-action events.
pub fn default_extractor<'a>(
    spec: &'a FeatureSpec,
    families: &'a [String],
) -> impl Fn(&[(i64, Event)], &TaskIndexRow, ActionKind, Option<f64>) -> FeatureVector + 'a {
    move |before, index_row, action, generator_raw| {
        extract(spec, before, index_row, action, families, generator_raw)
    }
}

/// Append a line to `<dir>/holdout-access.jsonl` for every read of a held_out row (B9).
///
/// The split digest covers assignments only, so this logging never changes it.
pub fn log_holdout_access(dir: &Path, row: &Row, purpose: &str) -> Result<(), LearnError> {
    std::fs::create_dir_all(dir)?;
    let line = serde_json::json!({
        "row": row.id.to_hex(),
        "task": row.task.to_string(),
        "side": "held_out",
        "purpose": purpose,
    });
    let mut f = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join("holdout-access.jsonl"))?;
    use std::io::Write;
    writeln!(f, "{line}")?;
    Ok(())
}

/// The held_out rows of a dataset, logging each read.
pub fn held_out_rows<'a>(
    ds: &'a Dataset,
    dir: &Path,
    purpose: &str,
) -> Result<Vec<&'a Row>, LearnError> {
    let mut out = Vec::new();
    for r in &ds.rows {
        if r.side == Side::HeldOut {
            log_holdout_access(dir, r, purpose)?;
            out.push(r);
        }
    }
    Ok(out)
}

/// A commitment's event id, for a probe that needs to name a row by its commitment.
pub fn commitment_of(row: &Row) -> EventId {
    row.commitment
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::{build_dataset, SplitFractions};
    use crate::features::spec_for_families;
    use tsr_domain::{
        Candidate, Decision, EventKind, Forecast, Forecaster, Hash, Horizon, Probability, Source,
        Utility,
    };

    fn task(tag: &[u8]) -> TaskId {
        TaskId(Hash::of(&[tag]))
    }

    fn ev(task: TaskId, kind: EventKind, tag: &[u8]) -> Event {
        let mut e = Event::new(task, Vec::new(), Source::Rules, kind, "2024-01-01T00:00:00Z");
        e.recorded_at = format!("2024-01-01T00:00:00Z#{}", hex(tag));
        e
    }

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    fn session(t: TaskId, tag: &[u8], p: f64) -> Vec<(i64, Event)> {
        let c = Candidate::new(
            ActionKind::ProposePatch,
            Source::Generator,
            Hash::of(&[tag]),
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
        let cid = c.id;
        vec![
            (1, ev(t, EventKind::TaskOpened { task: t }, tag)),
            (2, ev(t, EventKind::Proposed { candidate: c }, tag)),
            (
                3,
                ev(
                    t,
                    EventKind::Forecast {
                        forecast: Forecast::new(
                            crate::decision_event_id(),
                            Horizon::ThisDispatch,
                            Forecaster::Generator,
                            Probability::new(p).unwrap(),
                            3,
                        ),
                        candidate: cid,
                    },
                    tag,
                ),
            ),
            (
                4,
                ev(
                    t,
                    EventKind::Committed {
                        candidate: cid,
                        decision: Decision::Selected {
                            value: cid,
                            utility: Utility {
                                p_success: 0.5,
                                value: 0.0,
                                cost: 0.0,
                                ev: 0.0,
                            },
                            propensity: 1.0,
                            evidence: Vec::new(),
                        },
                    },
                    tag,
                ),
            ),
            (
                5,
                ev(
                    t,
                    EventKind::TaskClosed {
                        task: t,
                        reason: "done".to_string(),
                    },
                    tag,
                ),
            ),
        ]
    }

    fn index_row(t: TaskId, family: &str, lineage: &str, ts: &str) -> TaskIndexRow {
        TaskIndexRow {
            task: t,
            family: family.to_string(),
            lineage: lineage.to_string(),
            timestamp: ts.to_string(),
            clone_group: None,
            tool_ms_ceiling: Some(1000),
        }
    }

    fn label(t: TaskId, accepted: bool) -> crate::decision::LabelRecord {
        crate::decision::LabelRecord {
            task: t,
            criteria_met: accepted,
            diff_label: Some(if accepted {
                crate::decision::DiffLabel::Accepted
            } else {
                crate::decision::DiffLabel::Rejected
            }),
            reviewer: "operator".to_string(),
            recorded_at: "2024-01-02T00:00:00Z".to_string(),
        }
    }

    fn fixture() -> (Dataset, Vec<(i64, Event)>, Vec<TaskIndexRow>) {
        let mut events = Vec::new();
        let mut index = Vec::new();
        let mut labels = Vec::new();
        for i in 0..12u8 {
            let t = task(&[i]);
            events.extend(session(t, &[i], 0.9));
            let family = if i % 2 == 0 { "over" } else { "cal" };
            let lineage = if i < 6 { "repo-a" } else { "repo-b" };
            let ts = format!("2024-01-{:02}T00:00:00Z", i + 1);
            index.push(index_row(t, family, lineage, &ts));
            labels.push(label(t, i % 3 != 0));
        }
        let ds = build_dataset(
            &events,
            &index,
            &labels,
            "operator",
            SplitBy::Time,
            SplitFractions {
                calibrate: 0.25,
                holdout: 0.25,
            },
            7,
            Hash::of(&[b"src"]),
        )
        .unwrap();
        (ds, events, index)
    }

    #[test]
    fn clean_fixture_passes_all_three_probes() {
        let (ds, events, index) = fixture();
        let families = crate::features::families_from_spec(&ds.spec);
        let extractor = default_extractor(&ds.spec, &families);
        let report = run_all(&ds, &events, &index, &ds.spec, &extractor);
        assert!(report.is_clean(), "{:?}", report.leaks);
    }

    #[test]
    fn probe_a_catches_an_extractor_reading_the_decisions_own_executed_exit_code() {
        let (ds, events, index) = fixture();
        let families: Vec<String> = Vec::new();
        // A leaky extractor: it reads the decision's own Executed exit code, which is a
        // post-action event. It appends the exit code as an extra value.
        let leaky = |before: &[(i64, Event)], row: &TaskIndexRow, action: ActionKind, raw: Option<f64>| {
            let mut fv = extract(&ds.spec, before, row, action, &families, raw);
            // Read the *whole* log's Executed for this task — a post-action read.
            let mut extra = 0.0;
            for (_, e) in &events {
                if e.task == row.task {
                    if let EventKind::Executed { exit_code, .. } = &e.kind {
                        extra += exit_code.unwrap_or(0) as f64;
                    }
                }
            }
            fv.values.push(extra);
            fv
        };
        let report = probe_post_action(&ds, &events, &index, &ds.spec, &leaky);
        assert!(!report.is_clean());
        assert_eq!(report.leaks[0].probe, Probe::PostAction);
    }

    #[test]
    fn probe_b_catches_a_label_copied_into_a_feature() {
        let (mut ds, _events, _index) = fixture();
        // Plant the label into a feature: a perfect separator.
        for r in ds.rows.iter_mut() {
            if let Some(l) = r.label {
                r.features.values[0] = if l { 1.0 } else { 0.0 };
            }
        }
        let report = probe_label_reach(&ds, &ds.spec);
        assert!(!report.is_clean());
        assert_eq!(report.leaks[0].probe, Probe::LabelReach);
        assert!(report.leaks[0].row.starts_with("feature:"));
    }

    #[test]
    fn probe_c_catches_a_forged_split_manifest() {
        let (mut ds, _events, index) = fixture();
        // Forge: move one held_out task onto the fit side, so a task is on two sides.
        let held: Vec<TaskId> = ds
            .rows
            .iter()
            .filter(|r| r.side == Side::HeldOut)
            .map(|r| r.task)
            .collect();
        assert!(!held.is_empty());
        for r in ds.rows.iter_mut() {
            if r.task == held[0] {
                r.side = Side::Fit;
            }
        }
        let report = probe_boundary(&ds, &index);
        assert!(!report.is_clean());
        assert_eq!(report.leaks[0].probe, Probe::Boundary);
    }

    #[test]
    fn held_out_reads_are_logged_and_do_not_change_the_split_digest() {
        let (ds, _events, _index) = fixture();
        let dir = std::env::temp_dir().join(format!("tsr-learn-probe-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let before = ds.split.digest;
        let rows = held_out_rows(&ds, &dir, "report").unwrap();
        assert!(!rows.is_empty());
        let log = std::fs::read_to_string(dir.join("holdout-access.jsonl")).unwrap();
        assert_eq!(log.lines().count(), rows.len());
        assert_eq!(ds.split.digest, before);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn spec_for_families_is_used_by_the_default_extractor() {
        let spec = spec_for_families(&["a".to_string()]);
        let families = vec!["a".to_string()];
        let extractor = default_extractor(&spec, &families);
        let row = index_row(task(&[1]), "a", "repo-a", "2024-01-01T00:00:00Z");
        let fv = extractor(&[], &row, ActionKind::Inspect, None);
        assert_eq!(fv.values.len(), spec.names.len());
    }
}
