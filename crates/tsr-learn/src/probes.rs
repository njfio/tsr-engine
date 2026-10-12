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

/// The result of running the probes: the leaks found (empty means clean) and the refusals -- a
/// probe that could not examine the data it was given (a row whose task has no events, or a probe
/// that examined no rows, or fewer rows than the dataset's held-out slice). A refusal is not a
/// clean pass: the caller must exit 2 naming the probe and what is missing.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct ProbeReport {
    pub leaks: Vec<Leak>,
    #[serde(default)]
    pub refusals: Vec<String>,
}

impl ProbeReport {
    pub fn is_clean(&self) -> bool {
        self.leaks.is_empty() && self.refusals.is_empty()
    }
}

/// An extractor: a function from the pre-action events, the index row, the action and the
/// generator's raw probability on the common event to a feature vector. It receives **no label** —
/// by type, so a label cannot reach a feature.
pub type Extractor<'a> =
    &'a dyn Fn(&[(i64, Event)], &TaskIndexRow, ActionKind, Option<f64>) -> FeatureVector;

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
    let mut leaks = Vec::new();
    let mut refusals = Vec::new();
    let held_out = ds.rows.iter().filter(|r| r.side == Side::HeldOut).count();
    let fit_rows = ds.rows.iter().filter(|r| r.side == Side::Fit).count();
    let calibrate_rows = ds.rows.iter().filter(|r| r.side == Side::Calibrate).count();
    let mut examined = 0usize;
    let mut examined_fit = 0usize;
    let mut examined_calibrate = 0usize;
    let mut examined_held_out = 0usize;
    let mut missing_event = 0usize;
    let mut first_missing_event: Option<String> = None;
    let mut missing_commitment = 0usize;
    let mut first_missing_commitment: Option<String> = None;
    for row in &ds.rows {
        let Some(index_row) = index_of.get(&row.task) else {
            continue;
        };
        // F9/B3: never index into data the probe did not check. A row whose task has no events in
        // the log is a refusal naming the file, not a panic.
        let Some(task_events) = by_task.get(&row.task) else {
            missing_event += 1;
            if first_missing_event.is_none() {
                first_missing_event = Some(format!("task {} (row {})", row.task, row.id.to_hex()));
            }
            continue;
        };
        let before: Vec<(i64, Event)> = task_events
            .iter()
            .take_while(|(_, e)| e.id != row.commitment)
            .cloned()
            .collect();
        // The generator's raw p is recomputed from the task's log truncated before the
        // commitment, by the same rule `decision_rows` uses -- so a leak in the pairing itself is
        // caught, not just a leak in the features. `decision_rows` needs the commitment event to
        // form a row, so we pass the task's log up to and including it; the pairing is task-local,
        // so events after the commitment cannot change the result.
        // F9/B3: never index into data the probe did not check. A row whose commitment event is
        // absent from the log is a refusal naming the file and what is missing, not a panic.
        let Some(commitment) = task_events
            .iter()
            .find(|(_, e)| e.id == row.commitment)
            .cloned()
        else {
            missing_commitment += 1;
            if first_missing_commitment.is_none() {
                first_missing_commitment = Some(format!(
                    "commitment {} for task {} (row {})",
                    row.commitment,
                    row.task,
                    row.id.to_hex()
                ));
            }
            continue;
        };
        let upto: Vec<(i64, Event)> = task_events
            .iter()
            .take_while(|(_, e)| e.id != row.commitment)
            .cloned()
            .chain(std::iter::once(commitment))
            .collect();
        examined += 1;
        match row.side {
            Side::Fit => examined_fit += 1,
            Side::Calibrate => examined_calibrate += 1,
            Side::HeldOut => examined_held_out += 1,
        }
        let raw = crate::decision::decision_rows(&upto)
            .into_iter()
            .find(|dr| dr.commitment == row.commitment)
            .and_then(|dr| dr.generator_raw());
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
    // F9/B3 (d): the refusal names the first offending row and the count, not one line per row (a
    // cut events log produced a single 271 KB stderr line).
    if missing_event > 0 {
        refusals.push(format!(
            "probe post_action_events refused: events.jsonl holds no event for {missing_event} \
             indexed rows, the first being {}",
            first_missing_event.unwrap_or_default()
        ));
    }
    if missing_commitment > 0 {
        refusals.push(format!(
            "probe post_action_events refused: events.jsonl holds no commitment event for \
             {missing_commitment} indexed rows, the first being {}",
            first_missing_commitment.unwrap_or_default()
        ));
    }
    // F9/B3: a probe that examines no rows, or fewer rows than the dataset's held-out slice, is
    // refused naming the probe and the counts -- never skipped, never a silent clean pass. An
    // absent index (no `tasks.jsonl`) is not a probe that examined nothing: the CLI refuses a
    // missing or empty index before it reaches here, so the refusal is gated on a present index.
    //
    // F9/B3 (b): a count is not coverage. A `tasks.jsonl` cut to the fit-side tasks examines 696
    // rows and none of the 215 held-out, and a held-out-only or calibrate-only index also passes a
    // bare count. The probe must cover every split side: at least one row of fit, calibrate and
    // held-out, and every held-out row.
    if !index.is_empty() {
        if examined == 0 {
            refusals.push(format!(
                "probe post_action_events refused: it examined 0 of {} rows (the task index matches \
                 no row), so it checked nothing",
                ds.rows.len()
            ));
        } else {
            // A side with no rows in the dataset is not a coverage gap here: an empty fit or
            // calibrate slice is refused by `fit` itself (F3/R2), naming the slice. Coverage is
            // required of every side that has rows, and of every held-out row.
            let mut missing: Vec<String> = Vec::new();
            if fit_rows > 0 && examined_fit == 0 {
                missing.push(format!("fit (0 of {fit_rows})"));
            }
            if calibrate_rows > 0 && examined_calibrate == 0 {
                missing.push(format!("calibrate (0 of {calibrate_rows})"));
            }
            if examined_held_out < held_out {
                missing.push(format!("held_out ({examined_held_out} of {held_out})"));
            }
            if !missing.is_empty() {
                refusals.push(format!(
                    "probe post_action_events refused: it examined {examined} of {} rows but did \
                     not cover every split side: {}",
                    ds.rows.len(),
                    missing.join(", ")
                ));
            }
        }
    }
    ProbeReport { leaks, refusals }
}

/// Probe (b): a single feature that separates the fit slice's labels perfectly by a threshold
/// fails, named.
///
/// A feature separates perfectly when, over the fit rows with a resolved label, the largest value
/// of one label is strictly below the smallest value of the other -- a threshold that puts every
/// `true` row on one side and every `false` row on the other. Each label may take many values; the
/// old singleton rule missed a multi-valued separator. The feature is named in the leak.
pub fn probe_label_reach(ds: &Dataset, spec: &FeatureSpec) -> ProbeReport {
    let fit: Vec<&Row> = ds
        .rows
        .iter()
        .filter(|r| r.side == Side::Fit && r.label.is_some())
        .collect();
    let mut leaks = Vec::new();
    if fit.is_empty() {
        return ProbeReport {
            leaks,
            refusals: Vec::new(),
        };
    }
    for (i, name) in spec.names.iter().enumerate() {
        let mut true_vals: Vec<f64> = Vec::new();
        let mut false_vals: Vec<f64> = Vec::new();
        for r in &fit {
            let v = r.features.values.get(i).copied().unwrap_or(0.0);
            if r.label == Some(true) {
                true_vals.push(v);
            } else {
                false_vals.push(v);
            }
        }
        if true_vals.is_empty() || false_vals.is_empty() {
            continue;
        }
        let t_max = true_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let t_min = true_vals.iter().cloned().fold(f64::INFINITY, f64::min);
        let f_max = false_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let f_min = false_vals.iter().cloned().fold(f64::INFINITY, f64::min);
        // A threshold separates perfectly when one label's largest value is strictly below the
        // other label's smallest value.
        let separated = t_max < f_min || f_max < t_min;
        if separated {
            let (lo, hi) = if t_max < f_min {
                (t_max, f_min)
            } else {
                (f_max, t_min)
            };
            leaks.push(Leak {
                probe: Probe::LabelReach,
                row: format!("feature:{name}"),
                detail: format!(
                    "feature {name:?} separates the fit slice's labels by a threshold \
                     (largest of one label {lo}, smallest of the other {hi})"
                ),
            });
        }
    }
    ProbeReport {
        leaks,
        refusals: Vec::new(),
    }
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
            lineage_sides
                .entry(r.lineage.clone())
                .or_default()
                .insert(r.side);
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

    // Under time, the split rule (dataset.rs make_split) orders the units by their earliest
    // timestamp and cuts them into three contiguous blocks: fit, then calibrate, then held_out.
    // The probe checks *contiguity*: order every task by its timestamp and require the side
    // sequence to be non-decreasing (all Fit, then all Calibrate, then all HeldOut). A pairwise
    // extremes check is not enough — a forged manifest that moves the earliest held_out task onto
    // the fit side leaves every remaining held_out task later than the new max(fit/calibrate), and
    // the moved task later than every calibrate task, so no pair is out of order; only the gap in
    // the ordered sequence reveals the forge.
    if ds.split.by == SplitBy::Time {
        let rank = |s: Side| match s {
            Side::Fit => 0u8,
            Side::Calibrate => 1,
            Side::HeldOut => 2,
        };
        // F8/R5: the unit is the task merged with its clone group and its equal-timestamp peers,
        // ordered by the unit's earliest timestamp (dataset.rs make_split). A clone group whose
        // members have different timestamps goes to the side of its earliest member, so the probe
        // must order by the unit's earliest timestamp, not by each task's own timestamp.
        let mut parent: BTreeMap<String, String> = BTreeMap::new();
        for r in &ds.rows {
            let key = format!("task:{}", r.task);
            parent.entry(key.clone()).or_insert(key);
        }
        fn find(parent: &mut BTreeMap<String, String>, x: &str) -> String {
            let mut root = x.to_string();
            while parent[&root] != root {
                root = parent[&root].clone();
            }
            let mut cur = x.to_string();
            while parent[&cur] != cur {
                let next = parent[&cur].clone();
                parent.insert(cur.clone(), root.clone());
                cur = next;
            }
            root
        }
        let merge = |parent: &mut BTreeMap<String, String>, members: &[String]| {
            let mut it = members.iter();
            if let Some(first) = it.next() {
                let first_root = find(parent, first);
                for other in it {
                    let other_root = find(parent, other);
                    if other_root != first_root {
                        parent.insert(other_root, first_root.clone());
                    }
                }
            }
        };
        // Merge a clone group's tasks.
        let mut group_tasks: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for r in &ds.rows {
            if let Some(g) = index_of.get(&r.task).and_then(|i| i.clone_group.clone()) {
                group_tasks
                    .entry(g)
                    .or_default()
                    .push(format!("task:{}", r.task));
            }
        }
        for members in group_tasks.values() {
            merge(&mut parent, members);
        }
        // Merge tasks that share a timestamp.
        let mut ts_tasks: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for r in &ds.rows {
            if let Some(i) = index_of.get(&r.task) {
                ts_tasks
                    .entry(i.timestamp.clone())
                    .or_default()
                    .push(format!("task:{}", r.task));
            }
        }
        for members in ts_tasks.values() {
            merge(&mut parent, members);
        }
        // One entry per unit: its earliest timestamp and its side.
        let mut unit_key: BTreeMap<String, String> = BTreeMap::new();
        let mut unit_side: BTreeMap<String, Side> = BTreeMap::new();
        for r in &ds.rows {
            let Some(i) = index_of.get(&r.task) else {
                continue;
            };
            let unit = find(&mut parent, &format!("task:{}", r.task));
            unit_key
                .entry(unit.clone())
                .and_modify(|k| {
                    if i.timestamp < *k {
                        *k = i.timestamp.clone();
                    }
                })
                .or_insert_with(|| i.timestamp.clone());
            unit_side.insert(unit, r.side);
        }
        let mut ordered: Vec<(String, Side)> = unit_key
            .into_iter()
            .map(|(unit, ts)| {
                let side = unit_side[&unit];
                (ts, side)
            })
            .collect();
        ordered.sort_by(|a, b| a.0.cmp(&b.0));
        for w in ordered.windows(2) {
            let (ts_a, side_a) = &w[0];
            let (ts_b, side_b) = &w[1];
            if rank(*side_b) < rank(*side_a) {
                leaks.push(Leak {
                    probe: Probe::Boundary,
                    row: format!("time:{ts_b}"),
                    detail: format!(
                        "the time split is not contiguous: a {side_b:?} task ({ts_b}) follows a \
                         {side_a:?} task ({ts_a})"
                    ),
                });
            }
        }
    }

    ProbeReport {
        leaks,
        refusals: Vec::new(),
    }
}

/// Probe (c), the split manifest: read `split.json` and check it against the rows' sides, then
/// recompute the split digest from its assignments.
///
/// A forged `split.json` alone -- the rows untouched -- is caught here: the manifest's assignment
/// for a task must equal the side its rows carry, and the stored digest must equal the digest
/// recomputed from the rule, the seed and the assignments.
pub fn probe_split_manifest(ds: &Dataset, split_json: &Path) -> ProbeReport {
    let mut leaks = Vec::new();
    let text = match std::fs::read_to_string(split_json) {
        Ok(t) => t,
        Err(e) => {
            leaks.push(Leak {
                probe: Probe::Boundary,
                row: "split.json".to_string(),
                detail: format!("cannot read {}: {e}", split_json.display()),
            });
            return ProbeReport {
                leaks,
                refusals: Vec::new(),
            };
        }
    };
    let manifest: crate::dataset::SplitManifest = match serde_json::from_str(&text) {
        Ok(m) => m,
        Err(e) => {
            leaks.push(Leak {
                probe: Probe::Boundary,
                row: "split.json".to_string(),
                detail: format!("{} is not a split manifest: {e}", split_json.display()),
            });
            return ProbeReport {
                leaks,
                refusals: Vec::new(),
            };
        }
    };

    // The manifest's assignment for every task must equal the side its rows carry.
    let mut row_sides: BTreeMap<TaskId, BTreeSet<Side>> = BTreeMap::new();
    for r in &ds.rows {
        row_sides.entry(r.task).or_default().insert(r.side);
    }
    for (task, sides) in &row_sides {
        let key = task.to_string();
        match manifest.assignments.get(&key) {
            Some(side) => {
                if sides.len() > 1 || !sides.contains(side) {
                    leaks.push(Leak {
                        probe: Probe::Boundary,
                        row: key.clone(),
                        detail: format!(
                            "split.json assigns task {key} to {side:?}, but its rows are on {sides:?}"
                        ),
                    });
                }
            }
            None => leaks.push(Leak {
                probe: Probe::Boundary,
                row: key.clone(),
                detail: format!("split.json has no assignment for task {key}"),
            }),
        }
    }
    // Every assignment must name a task the rows carry.
    for task in manifest.assignments.keys() {
        if !row_sides.keys().any(|t| &t.to_string() == task) {
            leaks.push(Leak {
                probe: Probe::Boundary,
                row: task.clone(),
                detail: format!("split.json assigns task {task}, which has no rows"),
            });
        }
    }

    // The stored digest must equal the digest recomputed from the rule, the seed and the
    // assignments.
    let recomputed =
        crate::dataset::split_digest(manifest.by, manifest.seed, &manifest.assignments);
    if recomputed != manifest.digest {
        leaks.push(Leak {
            probe: Probe::Boundary,
            row: "split.json".to_string(),
            detail: format!(
                "split.json digest {} does not match the digest recomputed from its assignments {}",
                manifest.digest.to_hex(),
                recomputed.to_hex()
            ),
        });
    }
    ProbeReport {
        leaks,
        refusals: Vec::new(),
    }
}

/// Run all three probes over a dataset.
///
/// When `split_json` names the dataset's `split.json`, probe (c) also checks the manifest against
/// the rows' sides and recomputes the split digest.
pub fn run_all(
    ds: &Dataset,
    events: &[(i64, Event)],
    index: &[TaskIndexRow],
    spec: &FeatureSpec,
    extractor: Extractor<'_>,
    split_json: Option<&Path>,
) -> ProbeReport {
    let mut leaks = Vec::new();
    let mut refusals = Vec::new();
    let a = probe_post_action(ds, events, index, spec, extractor);
    leaks.extend(a.leaks);
    refusals.extend(a.refusals);
    leaks.extend(probe_label_reach(ds, spec).leaks);
    leaks.extend(probe_boundary(ds, index).leaks);
    if let Some(path) = split_json {
        leaks.extend(probe_split_manifest(ds, path).leaks);
    }
    ProbeReport { leaks, refusals }
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
    use crate::dataset::{build_dataset, SplitConfig, SplitFractions};
    use crate::features::spec_for_families;
    use tsr_domain::{
        Candidate, Decision, EventKind, Forecast, Forecaster, Hash, Horizon, Probability, Source,
        Utility,
    };

    fn task(tag: &[u8]) -> TaskId {
        TaskId(Hash::of(&[tag]))
    }

    fn ev(task: TaskId, kind: EventKind, tag: &[u8]) -> Event {
        let mut e = Event::new(
            task,
            Vec::new(),
            Source::Rules,
            kind,
            "2024-01-01T00:00:00Z",
        );
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
            SplitConfig {
                by: SplitBy::Time,
                fractions: SplitFractions {
                    calibrate: 0.25,
                    holdout: 0.25,
                },
                seed: 7,
            },
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
        let report = run_all(&ds, &events, &index, &ds.spec, &extractor, None);
        assert!(report.is_clean(), "{:?}", report.leaks);
    }

    #[test]
    fn probe_a_catches_an_extractor_reading_the_decisions_own_executed_exit_code() {
        let (ds, events, index) = fixture();
        let families: Vec<String> = Vec::new();
        // A leaky extractor: it reads the decision's own Executed exit code, which is a
        // post-action event. It appends the exit code as an extra value.
        let leaky =
            |before: &[(i64, Event)], row: &TaskIndexRow, action: ActionKind, raw: Option<f64>| {
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
        // Forge: move the earliest held_out task onto the fit side. This is not a two-sides
        // forge (all of the task's rows move together); it is a time-boundary forge: the
        // remaining held_out tasks are still later than the new max(fit/calibrate) timestamp,
        // and the moved task is later than every calibrate task, so no pair is out of order —
        // only the contiguity check (non-decreasing side rank in timestamp order) catches it.
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

    /// A4: probe (b) must flag a feature that separates the fit slice's labels perfectly by a
    /// *threshold* -- the largest value of one label below the smallest of the other -- even when
    /// each label takes many values. The old rule (`true_vals.len()==1 && false_vals.len()==1`)
    /// misses this plant.
    #[test]
    fn a4_probe_b_catches_a_multi_valued_threshold_separator() {
        let (mut ds, _events, _index) = fixture();
        // Plant a threshold separator into feature 0: every `false` row gets a value in [0, 0.4],
        // every `true` row a value in [0.6, 1.0]. Each label takes many values, so the old
        // singleton rule does not fire; the threshold rule must.
        let mut n = 0u64;
        for r in ds.rows.iter_mut() {
            if let Some(l) = r.label {
                let v = if l {
                    0.6 + (n % 5) as f64 * 0.1
                } else {
                    (n % 5) as f64 * 0.1
                };
                r.features.values[0] = v;
                n += 1;
            }
        }
        let report = probe_label_reach(&ds, &ds.spec);
        assert!(
            !report.is_clean(),
            "A4: a multi-valued threshold separator must be caught"
        );
        assert_eq!(report.leaks[0].probe, Probe::LabelReach);
        assert!(
            report.leaks[0].row.starts_with("feature:"),
            "A7: the leak names the feature: {:?}",
            report.leaks[0]
        );
    }

    /// A4: a feature that does *not* separate by threshold (the labels overlap) must pass.
    #[test]
    fn a4_probe_b_passes_an_overlapping_feature() {
        let (mut ds, _events, _index) = fixture();
        // Feature 0 takes the same values for both labels: no threshold separates them.
        let mut n = 0u64;
        for r in ds.rows.iter_mut() {
            if r.label.is_some() {
                r.features.values[0] = (n % 5) as f64 * 0.1;
                n += 1;
            }
        }
        let report = probe_label_reach(&ds, &ds.spec);
        assert!(
            report.is_clean(),
            "A4: an overlapping feature must not be flagged: {:?}",
            report.leaks
        );
    }

    /// A6: probe (c) must read `split.json` and flag a manifest whose assignments disagree with
    /// the rows' sides -- a forged split.json alone (the rows are untouched) must be caught.
    #[test]
    fn a6_probe_c_catches_a_forged_split_json() {
        let (ds, _events, _index) = fixture();
        let dir = std::env::temp_dir().join(format!("tsr-learn-split-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        // Forge: move every held_out task onto the fit side in split.json only.
        let mut forged = ds.split.clone();
        for side in forged.assignments.values_mut() {
            if *side == Side::HeldOut {
                *side = Side::Fit;
            }
        }
        std::fs::write(
            dir.join("split.json"),
            serde_json::to_string_pretty(&forged).unwrap(),
        )
        .unwrap();
        let report = probe_split_manifest(&ds, &dir.join("split.json"));
        assert!(!report.is_clean(), "A6: a forged split.json must be caught");
        assert_eq!(report.leaks[0].probe, Probe::Boundary);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A6: probe (c) must recompute the split digest and flag a split.json whose stored digest
    /// does not match its assignments.
    #[test]
    fn a6_probe_c_recomputes_the_split_digest() {
        let (ds, _events, _index) = fixture();
        let dir = std::env::temp_dir().join(format!("tsr-learn-digest-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut forged = ds.split.clone();
        forged.digest = Hash::of(&[b"forged"]);
        std::fs::write(
            dir.join("split.json"),
            serde_json::to_string_pretty(&forged).unwrap(),
        )
        .unwrap();
        let report = probe_split_manifest(&ds, &dir.join("split.json"));
        assert!(
            !report.is_clean(),
            "A6: a split.json with a wrong digest must be caught"
        );
        assert_eq!(report.leaks[0].probe, Probe::Boundary);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A6: a split.json that agrees with the rows and carries the right digest passes.
    #[test]
    fn a6_probe_c_passes_a_faithful_split_json() {
        let (ds, _events, _index) = fixture();
        let dir = std::env::temp_dir().join(format!("tsr-learn-ok-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("split.json"),
            serde_json::to_string_pretty(&ds.split).unwrap(),
        )
        .unwrap();
        let report = probe_split_manifest(&ds, &dir.join("split.json"));
        assert!(report.is_clean(), "{:?}", report.leaks);
        let _ = std::fs::remove_dir_all(&dir);
    }

    /// A7: a caught probe names the probe, the row id and (for (b)) the feature.
    #[test]
    fn a7_a_caught_probe_names_the_probe_row_and_feature() {
        let (mut ds, _events, _index) = fixture();
        for r in ds.rows.iter_mut() {
            if let Some(l) = r.label {
                r.features.values[0] = if l { 1.0 } else { 0.0 };
            }
        }
        let report = probe_label_reach(&ds, &ds.spec);
        assert!(!report.is_clean());
        let leak = &report.leaks[0];
        assert_eq!(leak.probe, Probe::LabelReach);
        assert!(
            leak.row.starts_with("feature:"),
            "A7: the leak names the feature: {leak:?}"
        );
        assert!(
            leak.detail.contains("feature"),
            "A7: the detail names the feature: {leak:?}"
        );
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
