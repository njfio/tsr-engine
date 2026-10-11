//! The dataset, its split and its manifests (PRD §B8, §B9; lane 06 item 4).
//!
//! A row is one commitment of a gated action with a resolved label. Its id is
//! `tsr_domain::Hash::of` over the commitment's event id, the feature vector and the resolved label
//! with its evidence ids. The **data root** hashes every row id in canonical order; the **training
//! root** hashes the fit and calibrate rows.
//!
//! The split is made **before any fitting** and has three sides — `fit` (the logistic head),
//! `calibrate` (isotonic, the support table, the population) and `held_out` (reports only). `--by
//! time` is forward-time by the index timestamp; `--by repo` assigns whole lineages by a seeded
//! shuffle. A task, and a clone group when the index names one, never crosses a boundary. Family
//! rides on every row for per-family scoring, support and the population, but is **not** a split
//! rule: the PRD's holdouts are by time and by repository (§B9).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use tsr_domain::{ActionKind, Event, EventId, Hash, TaskId};

use crate::decision::{decision_rows, resolve_labels, Label, LabelRecord, TaskLabel};
use crate::features::{extract, spec_for_families, FeatureSpec, FeatureVector};
use crate::rng::SplitMix64;
use crate::LearnError;

/// A task index row: the task's family, lineage, timestamp and optional clone group and tool-time
/// ceiling. Named like lane 05's results so the merge-time adapter is trivial.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TaskIndexRow {
    pub task: TaskId,
    pub family: String,
    pub lineage: String,
    pub timestamp: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_group: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_ms_ceiling: Option<u64>,
}

/// Which side of the split a task is on.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Fit,
    Calibrate,
    HeldOut,
}

/// The split rule.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitBy {
    Time,
    Repo,
}

/// One dataset row: a commitment with its features and its resolved label.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Row {
    /// The row id: `Hash::of(commitment event id, feature vector, resolved label + evidence ids)`.
    pub id: Hash,
    pub task: TaskId,
    pub commitment: EventId,
    pub index: u64,
    pub action: ActionKind,
    pub family: String,
    pub lineage: String,
    pub side: Side,
    pub features: FeatureVector,
    /// The resolved label, or `None` for a censored/unresolved row (excluded from training and
    /// scoring, counted in every report).
    pub label: Option<bool>,
    /// The label's evidence ids.
    pub evidence: Vec<EventId>,
    /// The generator's raw probability on the common event, if the generator forecast this row.
    pub generator_raw: Option<f64>,
    /// The operator's raw probability on the common event, if the operator forecast this row.
    pub operator_raw: Option<f64>,
    /// A controller forecast already logged on the common event, if any (scored separately).
    #[serde(default)]
    pub controller_logged: Option<f64>,
}

/// The split manifest: the assignment of every task to a side, plus the rule and seed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SplitManifest {
    pub by: SplitBy,
    pub seed: u64,
    /// Task id (hex) -> side, in canonical (sorted) order.
    pub assignments: BTreeMap<String, Side>,
    /// The split digest: covers assignments only, so logging a held_out read never changes it.
    pub digest: Hash,
}

/// The dataset: the rows, the split manifest and the two roots.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Dataset {
    pub rows: Vec<Row>,
    pub split: SplitManifest,
    pub spec: FeatureSpec,
    /// The data root: every row id, in canonical order.
    pub data_root: Hash,
    /// The training root: the fit and calibrate rows.
    pub training_root: Hash,
    /// The source root: the store's head_root or the events file's digest.
    pub source_root: Hash,
}

/// The split fractions: the calibrate and held-out shares; the fit side takes the remainder.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplitFractions {
    pub calibrate: f64,
    pub holdout: f64,
}

/// The split configuration: the rule, the fractions and the seed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SplitConfig {
    pub by: SplitBy,
    pub fractions: SplitFractions,
    pub seed: u64,
}

/// Assign every task to a side, before any fitting.
///
/// `--by time`: forward-time by the index timestamp — the earliest tasks are fit, the next
/// calibrate, the latest held_out. `--by repo`: whole lineages are assigned by a seeded shuffle.
/// A task, and a clone group when the index names one, never crosses a boundary.
pub fn make_split(
    index: &[TaskIndexRow],
    by: SplitBy,
    fractions: SplitFractions,
    seed: u64,
) -> SplitManifest {
    // Group tasks into atomic units: a clone group is one unit; a task with no clone group is its
    // own unit. A unit never crosses a boundary.
    let mut unit_of: BTreeMap<String, String> = BTreeMap::new();
    for row in index {
        let unit = row
            .clone_group
            .clone()
            .unwrap_or_else(|| format!("task:{}", row.task));
        unit_of.insert(row.task.to_string(), unit);
    }

    // The unit's sort key: under time, the earliest timestamp in the unit; under repo, the lineage.
    let mut unit_key: BTreeMap<String, String> = BTreeMap::new();
    let mut unit_lineage: BTreeMap<String, String> = BTreeMap::new();
    for row in index {
        let unit = &unit_of[&row.task.to_string()];
        let key = match by {
            SplitBy::Time => row.timestamp.clone(),
            SplitBy::Repo => row.lineage.clone(),
        };
        unit_key
            .entry(unit.clone())
            .and_modify(|k| {
                if key < *k {
                    *k = key.clone();
                }
            })
            .or_insert(key);
        unit_lineage.insert(unit.clone(), row.lineage.clone());
    }

    // Order the units. Under time: by the unit's earliest timestamp. Under repo: by a seeded
    // shuffle of the distinct lineages, then by lineage name for determinism within a lineage.
    let mut units: Vec<String> = unit_key.keys().cloned().collect();
    match by {
        SplitBy::Time => units.sort_by(|a, b| unit_key[a].cmp(&unit_key[b]).then(a.cmp(b))),
        SplitBy::Repo => {
            let mut lineages: Vec<String> = unit_lineage.values().cloned().collect();
            lineages.sort();
            lineages.dedup();
            let mut rng = SplitMix64::new(seed);
            rng.shuffle(&mut lineages);
            let rank: BTreeMap<String, usize> = lineages
                .iter()
                .enumerate()
                .map(|(i, l)| (l.clone(), i))
                .collect();
            units.sort_by(|a, b| {
                rank[&unit_lineage[a]]
                    .cmp(&rank[&unit_lineage[b]])
                    .then(a.cmp(b))
            });
        }
    }

    // Cut the ordered units into three sides by the fractions.
    let n = units.len();
    let n_cal = ((n as f64) * fractions.calibrate).round() as usize;
    let n_hold = ((n as f64) * fractions.holdout).round() as usize;
    let n_cal = n_cal.min(n);
    let n_hold = n_hold.min(n - n_cal);
    let n_fit = n - n_cal - n_hold;

    let mut unit_side: BTreeMap<String, Side> = BTreeMap::new();
    for (i, unit) in units.iter().enumerate() {
        let side = if i < n_fit {
            Side::Fit
        } else if i < n_fit + n_cal {
            Side::Calibrate
        } else {
            Side::HeldOut
        };
        unit_side.insert(unit.clone(), side);
    }

    let mut assignments: BTreeMap<String, Side> = BTreeMap::new();
    for row in index {
        let unit = &unit_of[&row.task.to_string()];
        assignments.insert(row.task.to_string(), unit_side[unit]);
    }

    let digest = split_digest(by, seed, &assignments);
    SplitManifest {
        by,
        seed,
        assignments,
        digest,
    }
}

/// The split digest: covers the rule, the seed and the assignments only.
pub fn split_digest(by: SplitBy, seed: u64, assignments: &BTreeMap<String, Side>) -> Hash {
    let mut parts: Vec<Vec<u8>> = Vec::new();
    parts.push(match by {
        SplitBy::Time => b"time".to_vec(),
        SplitBy::Repo => b"repo".to_vec(),
    });
    parts.push(seed.to_le_bytes().to_vec());
    for (task, side) in assignments {
        parts.push(task.as_bytes().to_vec());
        parts.push(match side {
            Side::Fit => b"fit".to_vec(),
            Side::Calibrate => b"calibrate".to_vec(),
            Side::HeldOut => b"held_out".to_vec(),
        });
    }
    let refs: Vec<&[u8]> = parts.iter().map(|p| p.as_slice()).collect();
    Hash::of(&refs)
}

/// The row id: `Hash::of(commitment event id, feature vector, resolved label + evidence ids)`.
pub fn row_id(
    commitment: &EventId,
    features: &FeatureVector,
    label: Option<bool>,
    evidence: &[EventId],
) -> Hash {
    let mut parts: Vec<Vec<u8>> = Vec::new();
    parts.push(commitment.0 .0.to_vec());
    for v in &features.values {
        parts.push(v.to_le_bytes().to_vec());
    }
    parts.push(match label {
        Some(true) => b"true".to_vec(),
        Some(false) => b"false".to_vec(),
        None => b"none".to_vec(),
    });
    for e in evidence {
        parts.push(e.0 .0.to_vec());
    }
    let refs: Vec<&[u8]> = parts.iter().map(|p| p.as_slice()).collect();
    Hash::of(&refs)
}

/// The data root: every row id, in canonical (sorted) order.
pub fn data_root(rows: &[Row]) -> Hash {
    let mut ids: Vec<Hash> = rows.iter().map(|r| r.id).collect();
    ids.sort();
    let parts: Vec<&[u8]> = ids.iter().map(|h| h.0.as_slice()).collect();
    Hash::of(&parts)
}

/// The training root: the fit and calibrate rows, in canonical order.
pub fn training_root(rows: &[Row]) -> Hash {
    let mut ids: Vec<Hash> = rows
        .iter()
        .filter(|r| r.side != Side::HeldOut)
        .map(|r| r.id)
        .collect();
    ids.sort();
    let parts: Vec<&[u8]> = ids.iter().map(|h| h.0.as_slice()).collect();
    Hash::of(&parts)
}

/// Build the dataset from the events, the task index and the labels.
///
/// The split is made **before any fitting**. Rows are built for every commitment of a gated action
/// whose task has an index row; a task with no index row is excluded (and counted by the caller).
/// A row whose label is censored or unresolved carries `label: None` and is excluded from training
/// and scoring.
pub fn build_dataset(
    events: &[(i64, Event)],
    index: &[TaskIndexRow],
    labels: &[LabelRecord],
    operator: &str,
    split_cfg: SplitConfig,
    source_root: Hash,
) -> Result<Dataset, LearnError> {
    let SplitConfig {
        by,
        fractions,
        seed,
    } = split_cfg;
    let task_labels = resolve_labels(events, labels, operator)?;
    let label_of: BTreeMap<TaskId, &TaskLabel> = task_labels.iter().map(|l| (l.task, l)).collect();
    let index_of: BTreeMap<TaskId, &TaskIndexRow> = index.iter().map(|r| (r.task, r)).collect();

    // The fit slice's families, in name order, define the one-hot. The split is made first, so the
    // families are those of the fit side.
    let split = make_split(index, by, fractions, seed);
    let mut fit_families: BTreeSet<String> = BTreeSet::new();
    for row in index {
        if split.assignments.get(&row.task.to_string()) == Some(&Side::Fit) {
            fit_families.insert(row.family.clone());
        }
    }
    let families: Vec<String> = fit_families.into_iter().collect();
    let spec = spec_for_families(&families);

    // Per task, the events in log order, so a commitment's "strictly before" slice is available.
    let mut by_task: BTreeMap<TaskId, Vec<(i64, Event)>> = BTreeMap::new();
    for (seq, e) in events {
        by_task.entry(e.task).or_default().push((*seq, e.clone()));
    }

    let rows = decision_rows(events);
    let mut out: Vec<Row> = Vec::new();
    for dr in rows {
        let Some(index_row) = index_of.get(&dr.task) else {
            continue; // no index row: excluded (item 2)
        };
        let side = split
            .assignments
            .get(&dr.task.to_string())
            .copied()
            .unwrap_or(Side::HeldOut);
        let task_events = &by_task[&dr.task];
        // The events strictly before the commitment, in log order.
        let before: Vec<(i64, Event)> = task_events
            .iter()
            .take_while(|(_, e)| e.id != dr.commitment)
            .cloned()
            .collect();
        let generator_raw = dr.generator_raw();
        let operator_raw = dr.operator_raw();
        let controller_logged = dr.controller_logged();
        let fv = extract(
            &spec,
            &before,
            index_row,
            dr.action,
            &families,
            generator_raw,
        );
        let tl = label_of.get(&dr.task);
        let (label, evidence) = match tl {
            Some(t) => match t.label {
                Label::True => (Some(true), t.evidence.clone()),
                Label::False => (Some(false), t.evidence.clone()),
                Label::Censored | Label::Unresolved => (None, t.evidence.clone()),
            },
            None => (None, Vec::new()),
        };
        let id = row_id(&dr.commitment, &fv, label, &evidence);
        out.push(Row {
            id,
            task: dr.task,
            commitment: dr.commitment,
            index: dr.index,
            action: dr.action,
            family: index_row.family.clone(),
            lineage: index_row.lineage.clone(),
            side,
            features: fv,
            label,
            evidence,
            generator_raw,
            operator_raw,
            controller_logged,
        });
    }

    // Canonical order: by row id.
    out.sort_by(|a, b| a.id.cmp(&b.id));

    let data_root = data_root(&out);
    let training_root = training_root(&out);
    Ok(Dataset {
        rows: out,
        split,
        spec,
        data_root,
        training_root,
        source_root,
    })
}

/// Read a task index from JSONL.
pub fn read_task_index(path: &Path) -> Result<Vec<TaskIndexRow>, LearnError> {
    let text = std::fs::read_to_string(path)?;
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let row: TaskIndexRow = serde_json::from_str(line)
            .map_err(|e| LearnError::Invalid(format!("tasks.jsonl line {}: {e}", i + 1)))?;
        out.push(row);
    }
    Ok(out)
}

/// Read a labels input from JSONL.
pub fn read_labels(path: &Path) -> Result<Vec<LabelRecord>, LearnError> {
    let text = std::fs::read_to_string(path)?;
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let row: LabelRecord = serde_json::from_str(line)
            .map_err(|e| LearnError::Invalid(format!("labels.jsonl line {}: {e}", i + 1)))?;
        out.push(row);
    }
    Ok(out)
}

/// Write the dataset to `dir`: `rows.jsonl`, `split.json`, `data-manifest.json`.
pub fn write_dataset(ds: &Dataset, dir: &Path) -> Result<(), LearnError> {
    std::fs::create_dir_all(dir)?;
    let mut rows = String::new();
    for r in &ds.rows {
        rows.push_str(&serde_json::to_string(r)?);
        rows.push('\n');
    }
    std::fs::write(dir.join("rows.jsonl"), rows)?;
    std::fs::write(
        dir.join("split.json"),
        serde_json::to_string_pretty(&ds.split)?,
    )?;
    let manifest = serde_json::json!({
        "feature_schema": ds.spec.version,
        "data_root": ds.data_root.to_hex(),
        "training_root": ds.training_root.to_hex(),
        "source_root": ds.source_root.to_hex(),
        "split_digest": ds.split.digest.to_hex(),
        "rows": ds.rows.len(),
        "fit": ds.rows.iter().filter(|r| r.side == Side::Fit).count(),
        "calibrate": ds.rows.iter().filter(|r| r.side == Side::Calibrate).count(),
        "held_out": ds.rows.iter().filter(|r| r.side == Side::HeldOut).count(),
        "resolved": ds.rows.iter().filter(|r| r.label.is_some()).count(),
    });
    std::fs::write(
        dir.join("data-manifest.json"),
        serde_json::to_string_pretty(&manifest)?,
    )?;
    std::fs::write(
        dir.join("features.json"),
        serde_json::to_string_pretty(&ds.spec)?,
    )?;
    // B9: every read of a held_out row appends a line to `<dir>/holdout-access.jsonl`. Building the
    // dataset reads the held_out rows to assign their side, so the build logs them too. The split
    // digest covers assignments only, so this logging never changes it.
    for r in &ds.rows {
        if r.side == Side::HeldOut {
            crate::probes::log_holdout_access(dir, r, "dataset")?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decision::DiffLabel;
    use tsr_domain::{
        Candidate, Decision, EventKind, Forecast, Forecaster, Horizon, Probability, Source, Utility,
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

    /// A gated session: open, propose, forecast at the commitment's index, commit, close.
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

    fn label(t: TaskId, accepted: bool) -> LabelRecord {
        LabelRecord {
            task: t,
            criteria_met: accepted,
            diff_label: Some(if accepted {
                DiffLabel::Accepted
            } else {
                DiffLabel::Rejected
            }),
            reviewer: "operator".to_string(),
            recorded_at: "2024-01-02T00:00:00Z".to_string(),
        }
    }

    fn build(seed: u64, by: SplitBy) -> Dataset {
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
        build_dataset(
            &events,
            &index,
            &labels,
            "operator",
            SplitConfig {
                by,
                fractions: SplitFractions {
                    calibrate: 0.25,
                    holdout: 0.25,
                },
                seed,
            },
            Hash::of(&[b"src"]),
        )
        .unwrap()
    }

    #[test]
    fn dataset_twice_gives_the_same_roots_and_split_digest() {
        let a = build(7, SplitBy::Time);
        let b = build(7, SplitBy::Time);
        assert_eq!(a.data_root, b.data_root);
        assert_eq!(a.training_root, b.training_root);
        assert_eq!(a.split.digest, b.split.digest);
        assert_eq!(a.rows.len(), b.rows.len());
    }

    #[test]
    fn different_seed_gives_a_different_split_digest() {
        let a = build(7, SplitBy::Repo);
        let b = build(8, SplitBy::Repo);
        assert_ne!(a.split.digest, b.split.digest);
    }

    #[test]
    fn a_task_never_crosses_a_boundary() {
        let ds = build(7, SplitBy::Time);
        let mut sides: BTreeMap<TaskId, BTreeSet<Side>> = BTreeMap::new();
        for r in &ds.rows {
            sides.entry(r.task).or_default().insert(r.side);
        }
        for (t, s) in &sides {
            assert_eq!(s.len(), 1, "task {t} on {} sides", s.len());
        }
    }

    #[test]
    fn a_clone_group_never_crosses_a_boundary() {
        let mut events = Vec::new();
        let mut index = Vec::new();
        let mut labels = Vec::new();
        for i in 0..8u8 {
            let t = task(&[100 + i]);
            events.extend(session(t, &[100 + i], 0.9));
            let mut row = index_row(
                t,
                "fam",
                "repo-a",
                &format!("2024-01-{:02}T00:00:00Z", i + 1),
            );
            row.clone_group = Some("clone-1".to_string());
            index.push(row);
            labels.push(label(t, true));
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
        let mut sides: BTreeSet<Side> = BTreeSet::new();
        for r in &ds.rows {
            sides.insert(r.side);
        }
        assert_eq!(sides.len(), 1, "a clone group must stay on one side");
    }

    #[test]
    fn under_time_held_out_is_later_than_train() {
        let ds = build(7, SplitBy::Time);
        let mut real_index = Vec::new();
        for i in 0..12u8 {
            let t = task(&[i]);
            let family = if i % 2 == 0 { "over" } else { "cal" };
            let lineage = if i < 6 { "repo-a" } else { "repo-b" };
            let ts = format!("2024-01-{:02}T00:00:00Z", i + 1);
            real_index.push(index_row(t, family, lineage, &ts));
        }
        let report = crate::probes::probe_boundary(&ds, &real_index);
        assert!(report.is_clean(), "{:?}", report.leaks);
    }

    #[test]
    fn a_task_with_no_index_row_is_excluded() {
        let mut events = Vec::new();
        let mut index = Vec::new();
        let mut labels = Vec::new();
        for i in 0..4u8 {
            let t = task(&[i]);
            events.extend(session(t, &[i], 0.9));
            if i != 0 {
                index.push(index_row(
                    t,
                    "fam",
                    "repo-a",
                    &format!("2024-01-0{}T00:00:00Z", i + 1),
                ));
            }
            labels.push(label(t, true));
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
        assert!(ds.rows.iter().all(|r| r.task != task(&[0])));
    }

    #[test]
    fn a_non_operator_label_is_refused() {
        let t = task(&[1]);
        let events = session(t, &[1], 0.9);
        let index = vec![index_row(t, "fam", "repo-a", "2024-01-01T00:00:00Z")];
        let mut l = label(t, true);
        l.reviewer = "someone-else".to_string();
        let err = build_dataset(
            &events,
            &index,
            &[l],
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
        .unwrap_err();
        assert!(matches!(err, LearnError::NonOperatorLabel { .. }));
    }
}
