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
#[serde(deny_unknown_fields)]
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
    /// A8: the number of distinct tasks that produced a decision row but have no row in the task
    /// index, so their rows were dropped. Counted in `dataset`'s stdout, `data-manifest.json` and
    /// the report, so a truncated `--tasks` is visible rather than silent.
    #[serde(default)]
    pub unindexed_tasks: usize,
    /// F8: the number of distinct tasks that have an index row but no gated commitment, so they
    /// produce no row and are dropped from the split. Counted in `dataset`'s stdout,
    /// `data-manifest.json` and the report, exactly as `unindexed_tasks` is.
    #[serde(default)]
    pub uncommitted_tasks: usize,
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
///
/// The atomic unit is the thing that must never cross a boundary:
/// - under `--by repo` the unit is the whole **lineage**, merged with every lineage a clone group
///   touches (the union of task, clone group and lineage). A clone group whose members sit in
///   several lineages therefore merges those lineages into one unit, so no lineage and no clone
///   group can land on two sides.
/// - under `--by time` the unit is the **task merged with its clone group**, and tasks with equal
///   timestamps go to the same side (a tie is one unit, so mutants of one snapshot stay together).
pub fn make_split(
    index: &[TaskIndexRow],
    by: SplitBy,
    fractions: SplitFractions,
    seed: u64,
) -> SplitManifest {
    // The unit of each task, as a canonical key. Under repo the unit is the lineage, but a clone
    // group that spans lineages merges them: union-find over (task, clone group, lineage).
    let mut unit_of: BTreeMap<String, String> = BTreeMap::new();
    match by {
        SplitBy::Repo => {
            // Union-find over the lineage names, merging every lineage a clone group touches.
            let mut parent: BTreeMap<String, String> = BTreeMap::new();
            for row in index {
                parent
                    .entry(row.lineage.clone())
                    .or_insert_with(|| row.lineage.clone());
            }
            fn find(parent: &mut BTreeMap<String, String>, x: &str) -> String {
                let mut root = x.to_string();
                while parent[&root] != root {
                    root = parent[&root].clone();
                }
                // Path compression.
                let mut cur = x.to_string();
                while parent[&cur] != cur {
                    let next = parent[&cur].clone();
                    parent.insert(cur.clone(), root.clone());
                    cur = next;
                }
                root
            }
            // Every clone group merges the lineages of its members.
            let mut group_lineages: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
            for row in index {
                if let Some(g) = &row.clone_group {
                    group_lineages
                        .entry(g.clone())
                        .or_default()
                        .insert(row.lineage.clone());
                }
            }
            for lineages in group_lineages.values() {
                let mut it = lineages.iter();
                if let Some(first) = it.next() {
                    let first_root = find(&mut parent, first);
                    for other in it {
                        let other_root = find(&mut parent, other);
                        if other_root != first_root {
                            parent.insert(other_root, first_root.clone());
                        }
                    }
                }
            }
            for row in index {
                let root = find(&mut parent, &row.lineage);
                unit_of.insert(row.task.to_string(), format!("lineage:{root}"));
            }
        }
        SplitBy::Time => {
            // The unit is the task merged with its clone group; tasks with equal timestamps are one
            // unit too, so a tie never crosses a boundary.
            let mut parent: BTreeMap<String, String> = BTreeMap::new();
            for row in index {
                let key = format!("task:{}", row.task);
                parent.entry(key.clone()).or_insert(key);
            }
            // Merge a clone group's tasks.
            let mut group_tasks: BTreeMap<String, Vec<String>> = BTreeMap::new();
            for row in index {
                if let Some(g) = &row.clone_group {
                    group_tasks
                        .entry(g.clone())
                        .or_default()
                        .push(format!("task:{}", row.task));
                }
            }
            // Merge tasks that share a timestamp.
            let mut ts_tasks: BTreeMap<String, Vec<String>> = BTreeMap::new();
            for row in index {
                ts_tasks
                    .entry(row.timestamp.clone())
                    .or_default()
                    .push(format!("task:{}", row.task));
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
            for members in group_tasks.values() {
                merge(&mut parent, members);
            }
            for members in ts_tasks.values() {
                merge(&mut parent, members);
            }
            for row in index {
                let key = format!("task:{}", row.task);
                let root = find(&mut parent, &key);
                unit_of.insert(row.task.to_string(), format!("unit:{root}"));
            }
        }
    }

    // The unit's sort key: under time, the earliest timestamp in the unit; under repo, the unit's
    // canonical name (the lineage root).
    let mut unit_key: BTreeMap<String, String> = BTreeMap::new();
    for row in index {
        let unit = &unit_of[&row.task.to_string()];
        let key = match by {
            SplitBy::Time => row.timestamp.clone(),
            SplitBy::Repo => unit.clone(),
        };
        unit_key
            .entry(unit.clone())
            .and_modify(|k| {
                if key < *k {
                    *k = key.clone();
                }
            })
            .or_insert(key);
    }

    // Order the units. Under time: by the unit's earliest timestamp. Under repo: by a seeded
    // shuffle of the distinct units, then by unit name for determinism.
    let mut units: Vec<String> = unit_key.keys().cloned().collect();
    match by {
        SplitBy::Time => units.sort_by(|a, b| unit_key[a].cmp(&unit_key[b]).then(a.cmp(b))),
        SplitBy::Repo => {
            // Deterministic seeded shuffle of the distinct units: sort first, then shuffle, so the
            // result does not depend on the order the units were discovered.
            units.sort();
            let mut rng = SplitMix64::new(seed);
            rng.shuffle(&mut units);
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

/// The row id: `Hash::of` over **every field a row carries** except the id itself.
///
/// F7: the id used to cover only the commitment, the features, the label and the evidence, so
/// `family`, `lineage`, `side`, `generator_raw`, `operator_raw` and `controller_logged` were
/// trusted verbatim — editing a row's `family` changed the population and editing a held-out
/// `generator_raw` flipped the report's verdict, with no id mismatch. The id now covers them all,
/// so `load_dataset` recomputes it and refuses any edit, naming the row.
pub fn row_id(r: &Row) -> Hash {
    let mut parts: Vec<Vec<u8>> = vec![
        r.task.0 .0.to_vec(),
        r.commitment.0 .0.to_vec(),
        r.index.to_le_bytes().to_vec(),
        serde_json::to_string(&r.action)
            .unwrap_or_default()
            .into_bytes(),
        r.family.as_bytes().to_vec(),
        r.lineage.as_bytes().to_vec(),
        match r.side {
            Side::Fit => b"fit".to_vec(),
            Side::Calibrate => b"calibrate".to_vec(),
            Side::HeldOut => b"held_out".to_vec(),
        },
    ];
    for v in &r.features.values {
        parts.push(v.to_le_bytes().to_vec());
    }
    parts.push(match r.label {
        Some(true) => b"true".to_vec(),
        Some(false) => b"false".to_vec(),
        None => b"none".to_vec(),
    });
    for e in &r.evidence {
        parts.push(e.0 .0.to_vec());
    }
    for v in [r.generator_raw, r.operator_raw, r.controller_logged] {
        parts.push(match v {
            Some(x) => x.to_le_bytes().to_vec(),
            None => b"none".to_vec(),
        });
    }
    let refs: Vec<&[u8]> = parts.iter().map(|p| p.as_slice()).collect();
    Hash::of(&refs)
}

/// F7: recompute a row's id from the row itself.
pub fn row_id_of(r: &Row) -> Hash {
    row_id(r)
}

/// D3: the counts digest -- a hash over the seven count fields `data-manifest.json` carries, so an
/// in-range edit of `unindexed_tasks` or `uncommitted_tasks` (which are not derivable from the
/// bundle's files) is refused. It is stored in the manifest and recomputed by `load_dataset`; it is
/// deliberately **not** part of `bundle_id`, so the acceptance corpus id is unchanged.
pub fn counts_digest(
    rows: u64,
    fit: u64,
    calibrate: u64,
    held_out: u64,
    resolved: u64,
    unindexed_tasks: u64,
    uncommitted_tasks: u64,
) -> Hash {
    let parts: Vec<Vec<u8>> = [
        rows,
        fit,
        calibrate,
        held_out,
        resolved,
        unindexed_tasks,
        uncommitted_tasks,
    ]
    .iter()
    .map(|n| n.to_string().into_bytes())
    .collect();
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
    // F8: a task with no gated commitment produces no row, so it must not be assigned a side --
    // otherwise `fit` refuses the tool's own output ("split.json assigns task X, which has no
    // rows"). Count such tasks and continue.
    let committed: BTreeSet<TaskId> = decision_rows(events).into_iter().map(|r| r.task).collect();
    // F8: an index row for a task with no gated commitment produces no row and is dropped from the
    // split. Count those tasks so the drop is visible rather than silent.
    let uncommitted: BTreeSet<TaskId> = index
        .iter()
        .map(|r| r.task)
        .filter(|t| !committed.contains(t))
        .collect();
    let split_index: Vec<TaskIndexRow> = index
        .iter()
        .filter(|r| committed.contains(&r.task))
        .cloned()
        .collect();
    let split = make_split(&split_index, by, fractions, seed);
    // F9/B3 (b): a share that leaves any side empty is refused, naming the side. An empty side is
    // not a split: `fit` would skip it (probes.rs) and write a bundle with no held-out rows, and an
    // empty calibrate slice fits an identity calibrator. Refuse at the source, with the counts.
    {
        let count = |side: Side| split.assignments.values().filter(|s| **s == side).count();
        for (side, name) in [
            (Side::Fit, "fit"),
            (Side::Calibrate, "calibrate"),
            (Side::HeldOut, "held_out"),
        ] {
            let n = count(side);
            if n == 0 {
                return Err(LearnError::Invalid(format!(
                    "the split leaves the {name} side empty ({n} of {} tasks); refusing a share \
                     that leaves any side empty",
                    split.assignments.len()
                )));
            }
        }
    }
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
    // A8: count the distinct tasks whose decision rows have no index row, so a truncated `--tasks`
    // is reported rather than silently dropped.
    let mut unindexed: BTreeSet<TaskId> = BTreeSet::new();
    for dr in rows {
        let Some(index_row) = index_of.get(&dr.task) else {
            unindexed.insert(dr.task);
            continue; // no index row: excluded, and counted (A8)
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
        let row = Row {
            id: Hash::of(&[b"placeholder"]),
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
        };
        let id = row_id_of(&row);
        out.push(Row { id, ..row });
    }

    // Canonical order: by row id.
    out.sort_by(|a, b| a.id.cmp(&b.id));

    // B15: the standardization statistics are part of the feature spec (the declared schema), not
    // of the fitted head. Compute them from the fit slice's labelled rows and store them in the
    // spec, so `features.json` carries them and `load_dataset` can recompute and check them.
    let mut spec = spec;
    let fit_labelled: Vec<Row> = out
        .iter()
        .filter(|r| r.side == Side::Fit && r.label.is_some())
        .cloned()
        .collect();
    let (mean, std) = crate::head::standardize_stats(&fit_labelled, spec.names.len());
    spec.standardize_mean = mean;
    spec.standardize_std = std;

    let data_root = data_root(&out);
    let training_root = training_root(&out);
    Ok(Dataset {
        rows: out,
        split,
        spec,
        data_root,
        training_root,
        source_root,
        unindexed_tasks: unindexed.len(),
        uncommitted_tasks: uncommitted.len(),
    })
}

/// F9/B3: the one loader every input file `dataset` and `fit` read goes through.
///
/// Each non-blank line is parsed into the record type the file holds. A line that is not a record
/// of that type -- `{}`, `null`, a truncated line, a line missing a field -- is refused naming the
/// file, the line and the missing or wrong field, so no caller ever sees a half-read input and no
/// probe ever indexes into data it did not check.
pub fn read_records<T>(path: &Path, what: &str) -> Result<Vec<T>, LearnError>
where
    T: serde::de::DeserializeOwned,
{
    read_records_impl(path, what, false)
}

/// D7: like `read_records`, but a repeated `task` id is refused naming the file, the line and the id.
/// `tasks.jsonl` and `labels.jsonl` hold one record per task, so a repeat is bad input; `rows.jsonl`
/// legitimately repeats a task id across sides, so it uses the plain loader.
pub fn read_records_unique<T>(path: &Path, what: &str) -> Result<Vec<T>, LearnError>
where
    T: serde::de::DeserializeOwned,
{
    read_records_impl(path, what, true)
}

fn read_records_impl<T>(path: &Path, what: &str, unique_ids: bool) -> Result<Vec<T>, LearnError>
where
    T: serde::de::DeserializeOwned,
{
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    // F9/B3: a missing or non-UTF-8 file is bad input, refused naming the file -- not a bare `io:`.
    let text = match std::fs::read(path) {
        Ok(bytes) => String::from_utf8(bytes)
            .map_err(|_| LearnError::Invalid(format!("{name}: the file is not valid UTF-8")))?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(LearnError::Invalid(format!(
                "{name}: the file is missing from the dataset directory {}",
                path.parent()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default()
            )))
        }
        Err(e) => return Err(LearnError::Invalid(format!("{name}: cannot read: {e}"))),
    };
    let mut out = Vec::new();
    // D7: a duplicate task id in `tasks.jsonl` (or a duplicate label record in `labels.jsonl`) is
    // bad input -- refused naming the file, the line and the id, never silently deduplicated or
    // double-counted. Both record types carry a `task` field, so the id is read from the parsed
    // value before the typed parse.
    let mut seen: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(line)
            .map_err(|e| LearnError::Invalid(format!("{name} line {}: {e}", i + 1)))?;
        if unique_ids {
            if let Some(id) = value.get("task").and_then(|v| v.as_str()) {
                if let Some(first) = seen.get(id) {
                    return Err(LearnError::Invalid(format!(
                        "{name} line {}: duplicate {what} id {id:?} (first at line {first})",
                        i + 1
                    )));
                }
                seen.insert(id.to_string(), i + 1);
            }
        }
        let record: T = serde_json::from_value(value).map_err(|e| {
            LearnError::Invalid(format!("{name} line {}: not a {what} record: {e}", i + 1))
        })?;
        out.push(record);
    }
    if out.is_empty() {
        return Err(LearnError::Invalid(format!(
            "{name} line 1: no {what} record (the file is empty or holds only blank lines)"
        )));
    }
    Ok(out)
}

/// Read an `events.jsonl` file through the same loader discipline: each line is a bare `Event` or
/// a `{"seq": n, "event": …}` wrapper, and a line that is not an event record is refused naming the
/// file, the line and the missing or wrong field.
pub fn read_events_records(path: &Path) -> Result<Vec<(i64, tsr_domain::Event)>, LearnError> {
    let name = path
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.display().to_string());
    // F9/B3: a missing or non-UTF-8 file is bad input, refused naming the file -- not a bare `io:`.
    let text = match std::fs::read(path) {
        Ok(bytes) => String::from_utf8(bytes)
            .map_err(|_| LearnError::Invalid(format!("{name}: the file is not valid UTF-8")))?,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Err(LearnError::Invalid(format!(
                "{name}: the file is missing from the dataset directory {}",
                path.parent()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default()
            )))
        }
        Err(e) => return Err(LearnError::Invalid(format!("{name}: cannot read: {e}"))),
    };
    let mut out: Vec<(i64, tsr_domain::Event)> = Vec::new();
    for (i, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(line)
            .map_err(|e| LearnError::Invalid(format!("{name} line {}: {e}", i + 1)))?;
        let (seq, event_value) = match value.get("event") {
            Some(ev) => {
                // F9/B3 (c): a wrapper's `seq` is a field like any other. A wrong-typed (`"x"`,
                // 1.5, null) or absent `seq` must be refused, never silently replaced by the line
                // position.
                let seq = match value.get("seq") {
                    Some(s) => s.as_i64().ok_or_else(|| {
                        LearnError::Invalid(format!(
                            "{name} line {}: `seq` must be an integer, found {s}",
                            i + 1
                        ))
                    })?,
                    None => {
                        return Err(LearnError::Invalid(format!(
                            "{name} line {}: the events wrapper has no `seq` field",
                            i + 1
                        )))
                    }
                };
                (Some(seq), ev.clone())
            }
            None => (None, value),
        };
        let event: tsr_domain::Event = serde_json::from_value(event_value).map_err(|e| {
            LearnError::Invalid(format!("{name} line {}: not an event record: {e}", i + 1))
        })?;
        let seq = seq.unwrap_or_else(|| out.len() as i64 + 1);
        out.push((seq, event));
    }
    if out.is_empty() {
        return Err(LearnError::Invalid(format!(
            "{name} line 1: no event record (the file is empty or holds only blank lines)"
        )));
    }
    out.sort_by_key(|(seq, _)| *seq);
    Ok(out)
}

/// Read a task index from JSONL through the one loader.
pub fn read_task_index(path: &Path) -> Result<Vec<TaskIndexRow>, LearnError> {
    read_records_unique(path, "task index")
}

/// Read a labels input from JSONL through the one loader.
pub fn read_labels(path: &Path) -> Result<Vec<LabelRecord>, LearnError> {
    read_records_unique(path, "label")
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
        "unindexed_tasks": ds.unindexed_tasks,
        "uncommitted_tasks": ds.uncommitted_tasks,
        "counts_digest": counts_digest(
            ds.rows.len() as u64,
            ds.rows.iter().filter(|r| r.side == Side::Fit).count() as u64,
            ds.rows.iter().filter(|r| r.side == Side::Calibrate).count() as u64,
            ds.rows.iter().filter(|r| r.side == Side::HeldOut).count() as u64,
            ds.rows.iter().filter(|r| r.label.is_some()).count() as u64,
            ds.unindexed_tasks as u64,
            ds.uncommitted_tasks as u64,
        )
        .to_hex(),
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
            // F9/B3 (b): three lineages, so a repo split can give all three sides a non-empty
            // share (a split that leaves a side empty is now refused).
            let lineage = match i / 4 {
                0 => "repo-a",
                1 => "repo-b",
                _ => "repo-c",
            };
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
        // F9/B3 (b): three clone groups, so the split can give all three sides a non-empty share
        // (a split that leaves a side empty is now refused) while each group still stays together.
        for i in 0..12u8 {
            let t = task(&[100 + i]);
            events.extend(session(t, &[100 + i], 0.9));
            let mut row = index_row(
                t,
                "fam",
                "repo-a",
                &format!("2024-01-{:02}T00:00:00Z", i + 1),
            );
            row.clone_group = Some(format!("clone-{}", i / 4));
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
        // Each clone group must stay on one side; with three groups the dataset as a whole spans
        // three sides, so check per group rather than globally. The clone group lives on the index
        // row, so map each dataset row's task back to its group.
        let group_of: BTreeMap<TaskId, String> = index
            .iter()
            .map(|r| (r.task, r.clone_group.clone().unwrap_or_default()))
            .collect();
        let mut sides: BTreeMap<String, BTreeSet<Side>> = BTreeMap::new();
        for r in &ds.rows {
            let group = group_of.get(&r.task).cloned().unwrap_or_default();
            sides.entry(group).or_default().insert(r.side);
        }
        assert_eq!(sides.len(), 3, "three clone groups");
        for (group, s) in &sides {
            assert_eq!(s.len(), 1, "clone group {group} must stay on one side");
        }
    }

    #[test]
    fn under_time_held_out_is_later_than_train() {
        let ds = build(7, SplitBy::Time);
        let mut real_index = Vec::new();
        for i in 0..12u8 {
            let t = task(&[i]);
            let family = if i % 2 == 0 { "over" } else { "cal" };
            // F9/B3 (b): three lineages, so a repo split can give all three sides a non-empty
            // share (a split that leaves a side empty is now refused).
            let lineage = match i / 4 {
                0 => "repo-a",
                1 => "repo-b",
                _ => "repo-c",
            };
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

    /// A2: under `--by repo` the unit is the whole lineage, merged with every lineage a clone group
    /// touches. On the fixture every clone group spans several lineages, so the union is one unit
    /// and the split cannot leak a lineage across a boundary. Before the fix, `make_split` cut at a
    /// unit count after ordering by lineage, so all 12 lineages landed on two or more sides and
    /// `fit` refused with 12 boundary leaks.
    #[test]
    fn a2_repo_split_keeps_every_lineage_on_one_side() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/m4");
        let index = read_task_index(&path.join("tasks.jsonl")).expect("tasks.jsonl is readable");
        assert!(!index.is_empty(), "the fixture has tasks");
        for seed in 1..=8u64 {
            let split = make_split(
                &index,
                SplitBy::Repo,
                SplitFractions {
                    calibrate: 0.2,
                    holdout: 0.2,
                },
                seed,
            );
            // Every lineage is on exactly one side.
            let mut lineage_sides: BTreeMap<String, BTreeSet<Side>> = BTreeMap::new();
            for row in &index {
                lineage_sides
                    .entry(row.lineage.clone())
                    .or_default()
                    .insert(split.assignments[&row.task.to_string()]);
            }
            for (lineage, sides) in &lineage_sides {
                assert_eq!(
                    sides.len(),
                    1,
                    "A2: seed {seed}: lineage {lineage} is on {} sides",
                    sides.len()
                );
            }
            // Every clone group is on exactly one side.
            let mut group_sides: BTreeMap<String, BTreeSet<Side>> = BTreeMap::new();
            for row in &index {
                if let Some(g) = &row.clone_group {
                    group_sides
                        .entry(g.clone())
                        .or_default()
                        .insert(split.assignments[&row.task.to_string()]);
                }
            }
            for (g, sides) in &group_sides {
                assert_eq!(
                    sides.len(),
                    1,
                    "A2: seed {seed}: clone group {g} is on {} sides",
                    sides.len()
                );
            }
        }
    }

    /// A2: the fixture under `--by repo` for seeds 1 through 8 gives a dataset that `fit` accepts —
    /// the boundary probe is clean.
    #[test]
    fn a2_repo_split_on_the_fixture_passes_the_boundary_probe() {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/m4");
        let index = read_task_index(&path.join("tasks.jsonl")).expect("tasks.jsonl is readable");
        for seed in 1..=8u64 {
            let split = make_split(
                &index,
                SplitBy::Repo,
                SplitFractions {
                    calibrate: 0.2,
                    holdout: 0.2,
                },
                seed,
            );
            // Build a dataset whose rows carry the split's sides, then run the boundary probe.
            let ds = Dataset {
                rows: index
                    .iter()
                    .map(|r| Row {
                        id: Hash::of(&[r.task.to_string().as_bytes()]),
                        task: r.task,
                        commitment: EventId(Hash::of(&[b"c"])),
                        index: 0,
                        action: ActionKind::ProposePatch,
                        family: r.family.clone(),
                        lineage: r.lineage.clone(),
                        side: split.assignments[&r.task.to_string()],
                        features: FeatureVector { values: Vec::new() },
                        label: Some(true),
                        evidence: Vec::new(),
                        generator_raw: None,
                        operator_raw: None,
                        controller_logged: None,
                    })
                    .collect(),
                split,
                spec: spec_for_families(&["cal".to_string()]),
                data_root: Hash::of(&[b"d"]),
                training_root: Hash::of(&[b"t"]),
                source_root: Hash::of(&[b"s"]),
                unindexed_tasks: 0,
                uncommitted_tasks: 0,
            };
            let report = crate::probes::probe_boundary(&ds, &index);
            assert!(
                report.is_clean(),
                "A2: seed {seed}: fit refuses the tool's own --by repo split: {:?}",
                report.leaks
            );
        }
    }

    /// A5: under `--by time` the unit is the task merged with its clone group, and tasks with equal
    /// timestamps go to the same side. Before the fix, ties were cut and `fit` refused the tool's
    /// own split for ties of 2-5 tasks (mutants of one snapshot share a timestamp).
    #[test]
    fn a5_tied_timestamps_go_to_the_same_side() {
        let mut events = Vec::new();
        let mut index = Vec::new();
        let mut labels = Vec::new();
        // 10 tasks in 4 tied groups of 3,3,3,1, each group a distinct timestamp. With fractions
        // 0.25/0.25 the cut falls inside the second group (nfit=4), so the old code split a tie.
        let sizes = [3usize, 3, 3, 1];
        let mut i = 0u8;
        for (g, size) in sizes.iter().enumerate() {
            for _ in 0..*size {
                let t = task(&[200 + i]);
                events.extend(session(t, &[200 + i], 0.9));
                let ts = format!("2024-01-{:02}T00:00:00Z", g + 1);
                index.push(index_row(t, "fam", "repo-a", &ts));
                labels.push(label(t, true));
                i += 1;
            }
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
        // Every task sharing a timestamp is on the same side.
        let mut ts_sides: BTreeMap<String, BTreeSet<Side>> = BTreeMap::new();
        for r in &ds.rows {
            let ts = index
                .iter()
                .find(|i| i.task == r.task)
                .expect("index row")
                .timestamp
                .clone();
            ts_sides.entry(ts).or_default().insert(r.side);
        }
        for (ts, sides) in &ts_sides {
            assert_eq!(
                sides.len(),
                1,
                "A5: tasks tied at {ts} are on {} sides",
                sides.len()
            );
        }
        let report = crate::probes::probe_boundary(&ds, &index);
        assert!(
            report.is_clean(),
            "A5: fit refuses the tool's own --by time split for tied timestamps: {:?}",
            report.leaks
        );
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
    /// B15: the standardization statistics live in the feature spec (`features.json`), not in the
    /// head, and the learning rate is declared there too. On the pre-fix code `spec.standardize_mean`
    /// was empty and the head carried `mean`/`std`; this test fails there.
    #[test]
    fn b15_standardization_stats_live_in_the_spec() {
        let ds = build(7, SplitBy::Time);
        let dim = ds.spec.names.len();
        assert_eq!(
            ds.spec.standardize_mean.len(),
            dim,
            "B15: features.json must carry a per-feature mean"
        );
        assert_eq!(
            ds.spec.standardize_std.len(),
            dim,
            "B15: features.json must carry a per-feature std"
        );
        assert!(
            ds.spec.learning_rate > 0.0,
            "B15: features.json must declare the learning rate"
        );
        // The declared statistics are exactly the fit slice's labelled rows' statistics.
        let fit_labelled: Vec<Row> = ds
            .rows
            .iter()
            .filter(|r| r.side == Side::Fit && r.label.is_some())
            .cloned()
            .collect();
        let (mean, std) = crate::head::standardize_stats(&fit_labelled, dim);
        assert_eq!(ds.spec.standardize_mean, mean);
        assert_eq!(ds.spec.standardize_std, std);
    }

    /// B15: support counts only fit- and calibrate-slice rows with a resolved label. An unlabelled
    /// row must not inflate the support index.
    #[test]
    fn b15_support_index_counts_only_labelled_rows() {
        let ds = build(7, SplitBy::Time);
        let index = crate::bundle::support_index_for_test(&ds);
        let labelled = ds
            .rows
            .iter()
            .filter(|r| (r.side == Side::Fit || r.side == Side::Calibrate) && r.label.is_some())
            .count();
        assert_eq!(
            index.len(),
            labelled,
            "B15: the support index must hold exactly the labelled fit/calibrate rows"
        );
    }
}
