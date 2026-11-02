//! Reproducible common-event eligibility at a declared cutoff (PRD B7/B8).
//! Wall clocks establish availability, never event ordering. Legacy datasets have no
//! maturity proof; their rows remain readable but cannot enter a new production fit.
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::{Deserialize, Serialize};
use tsr_domain::{Event, EventKind, Hash, TaskId};

use crate::dataset::{self, Dataset, SplitConfig, TaskIndexRow};
use crate::decision::{Label, LabelRecord};
use crate::LearnError;

pub const POLICY: &str = "task-close-plus-seven-days@v1";
const WEEK: i128 = 604_800_000_000_000;

fn invalid(message: impl Into<String>) -> LearnError {
    LearnError::Invalid(message.into())
}

/// Strict RFC3339 (nanosecond precision), with explicit known offset. Leap seconds
/// and unknown -00:00 offsets are refused rather than guessed. No ambient clock.
pub fn timestamp(text: &str) -> Result<i128, LearnError> {
    let fail = || invalid(format!("invalid RFC3339 timestamp {text:?}"));
    let b = text.as_bytes();
    if !text.is_ascii()
        || b.len() < 20
        || b[4] != b'-'
        || b[7] != b'-'
        || !matches!(b[10], b'T' | b't')
        || b[13] != b':'
        || b[16] != b':'
    {
        return Err(fail());
    }
    let number = |start: usize, end: usize| -> Result<i64, LearnError> {
        let s = &text[start..end];
        if !s.bytes().all(|c| c.is_ascii_digit()) {
            return Err(fail());
        }
        s.parse().map_err(|_| fail())
    };
    let (year, month, day) = (number(0, 4)?, number(5, 7)?, number(8, 10)?);
    let (hour, minute, second) = (number(11, 13)?, number(14, 16)?, number(17, 19)?);
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days = match month {
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => 0,
    };
    if year == 0 || day < 1 || day > days || hour > 23 || minute > 59 || second > 59 {
        return Err(fail());
    }
    let mut pos = 19;
    let mut nanos = 0i128;
    if b[pos] == b'.' {
        pos += 1;
        let start = pos;
        while pos < b.len() && b[pos].is_ascii_digit() {
            pos += 1;
        }
        let len = pos - start;
        if len == 0 || len > 9 {
            return Err(fail());
        }
        nanos =
            text[start..pos].parse::<i128>().map_err(|_| fail())? * 10i128.pow((9 - len) as u32);
    }
    let offset = match b.get(pos) {
        Some(b'Z' | b'z') if pos + 1 == b.len() => 0,
        Some(b'+' | b'-') if pos + 6 == b.len() && b[pos + 3] == b':' => {
            let h = number(pos + 1, pos + 3)?;
            let m = number(pos + 4, pos + 6)?;
            if h > 23 || m > 59 || (&text[pos..] == "-00:00") {
                return Err(fail());
            }
            (h * 3600 + m * 60) * if b[pos] == b'-' { -1 } else { 1 }
        }
        _ => return Err(fail()),
    };
    // Proleptic Gregorian civil date to days since Unix epoch.
    let y = year - i64::from(month <= 2);
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = month + if month > 2 { -3 } else { 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let unix_days = era * 146097 + doe - 719468;
    Ok(
        i128::from(unix_days * 86400 + hour * 3600 + minute * 60 + second - offset) * 1_000_000_000
            + nanos,
    )
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Eligibility {
    pub task: TaskId,
    pub status: String,
    pub close_event: Option<tsr_domain::EventId>,
    pub closed_at: Option<String>,
    pub review_recorded_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Maturity {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature_evidence_digest: Option<Hash>,
    pub policy: String,
    pub event: String,
    pub as_of: String,
    pub operator: String,
    pub split: SplitConfig,
    pub events_digest: Hash,
    pub tasks_digest: Hash,
    pub labels_digest: Hash,
    pub eligibility: Vec<Eligibility>,
}

/// The single validated event view used for features, outcome resolution and probes.
/// Sequence remains causal order. A closed task cannot acquire new decisions.
pub fn available_events(
    events: &[(i64, Event)],
    as_of: &str,
) -> Result<Vec<(i64, Event)>, LearnError> {
    let cutoff = timestamp(as_of)?;
    let mut available = Vec::new();
    let mut closed = BTreeSet::new();
    let mut previous = None;
    for (seq, event) in events {
        if previous.is_some_and(|last| *seq <= last) {
            return Err(invalid(
                "events.jsonl: sequence numbers must be unique and strictly increasing",
            ));
        }
        previous = Some(*seq);
        let at = timestamp(&event.recorded_at)?;
        let reconstructed = Event::new(
            event.task,
            event.parents.clone(),
            event.source,
            event.kind.clone(),
            &event.recorded_at,
        );
        if reconstructed.id != event.id || event.schema_version != Event::SCHEMA_VERSION {
            return Err(invalid(format!(
                "event {} identity/schema mismatch",
                event.id
            )));
        }
        if at > cutoff {
            continue;
        }
        if matches!(event.kind, EventKind::Committed { .. }) && closed.contains(&event.task) {
            return Err(invalid(format!(
                "task {} has a commitment after closure in event sequence",
                event.task
            )));
        }
        if matches!(event.kind, EventKind::TaskClosed { .. }) {
            closed.insert(event.task);
        }
        available.push((*seq, event.clone()));
    }
    Ok(available)
}

/// Build a new snapshot, retaining every input in provenance while excluding future
/// evidence from features and outcomes. A late review does not reopen this fixed
/// horizon's censored outcome. A new policy would be required for that semantics.
#[allow(clippy::too_many_arguments)]
pub fn build(
    events: &[(i64, Event)],
    index: &[TaskIndexRow],
    labels: &[LabelRecord],
    operator: &str,
    cfg: SplitConfig,
    source_root: Hash,
    as_of: &str,
) -> Result<(Dataset, Maturity), LearnError> {
    build_schema(
        events,
        index,
        labels,
        operator,
        cfg,
        source_root,
        as_of,
        None,
    )
}
#[allow(clippy::too_many_arguments)]
pub fn build_schema(
    events: &[(i64, Event)],
    index: &[TaskIndexRow],
    labels: &[LabelRecord],
    operator: &str,
    cfg: SplitConfig,
    source_root: Hash,
    as_of: &str,
    input: Option<&crate::schema::Input>,
) -> Result<(Dataset, Maturity), LearnError> {
    if let Some(input) = input {
        if input.digest()? != source_root {
            return Err(invalid("project source root/feature evidence mismatch"));
        }
    }
    if let Some(input) = input {
        if input.labels(as_of)? != labels {
            return Err(invalid(
                "project labels disagree with reconstructed prospective eligibility",
            ));
        }
    }
    let cutoff = timestamp(as_of)?;
    if operator.trim().is_empty() {
        return Err(invalid("maturity operator must be named"));
    }
    let mut seen = BTreeSet::new();
    for label in labels {
        if !seen.insert(label.task) {
            return Err(invalid(
                "duplicate task review; provide one explicitly reconciled review per task",
            ));
        }
        if label.reviewer != operator {
            return Err(LearnError::NonOperatorLabel {
                reviewer: label.reviewer.clone(),
            });
        }
        timestamp(&label.recorded_at)?;
    }
    let available = available_events(events, as_of)?;
    let mut closes = BTreeMap::new();
    for (_, event) in &available {
        if matches!(event.kind, EventKind::TaskClosed { .. }) {
            closes
                .entry(event.task)
                .or_insert((event, timestamp(&event.recorded_at)?));
        }
    }
    let selected: Vec<_> = labels
        .iter()
        .filter(|r| {
            closes.get(&r.task).is_some_and(|(_, closed)| {
                let at = timestamp(&r.recorded_at).expect("validated timestamp");
                cutoff >= closed + WEEK && at <= closed + WEEK && at <= cutoff
            })
        })
        .cloned()
        .collect();
    // Once the horizon has elapsed, later stop/budget events must not relabel its outcome.
    let outcome_events: Vec<_> = available
        .iter()
        .filter(|(_, e)| {
            closes.get(&e.task).is_none_or(|(_, close)| {
                timestamp(&e.recorded_at).expect("validated") <= close + WEEK
            })
        })
        .cloned()
        .collect();
    let resolved = crate::decision::resolve_labels(&outcome_events, &selected, operator)?;
    // Build features from the as-of view; override labels from the bounded outcome view.
    let mut ds = dataset::build_dataset_schema(
        &available,
        index,
        &selected,
        operator,
        cfg,
        source_root,
        input,
    )?;
    let labels_by_task: BTreeMap<_, _> = resolved.iter().map(|r| (r.task, r)).collect();
    for row in &mut ds.rows {
        if let Some(result) = labels_by_task.get(&row.task) {
            row.label = match result.label {
                Label::True => Some(true),
                Label::False => Some(false),
                _ => None,
            };
            row.evidence = result.evidence.clone();
            row.id = dataset::row_id_of(row);
        }
    }
    ds.rows.sort_by_key(|r| r.id);
    ds.data_root = dataset::data_root(&ds.rows);
    ds.training_root = dataset::training_root(&ds.rows);
    let fit: Vec<_> = ds
        .rows
        .iter()
        .filter(|r| r.side == dataset::Side::Fit && r.label.is_some())
        .cloned()
        .collect();
    (ds.spec.standardize_mean, ds.spec.standardize_std) =
        crate::head::standardize_stats(&fit, ds.spec.names.len());
    let eligibility = resolved
        .iter()
        .map(|r| {
            let close = closes.get(&r.task);
            let status = match close {
                None => "open",
                Some((_, closed)) if cutoff < closed + WEEK => "pending_horizon",
                Some(_) if matches!(r.label, Label::True | Label::False) => "mature_reviewed",
                Some(_) => "censored",
            };
            Eligibility {
                task: r.task,
                status: status.into(),
                close_event: close.map(|(e, _)| e.id),
                closed_at: close.map(|(e, _)| e.recorded_at.clone()),
                review_recorded_at: labels
                    .iter()
                    .find(|l| l.task == r.task)
                    .map(|l| l.recorded_at.clone()),
            }
        })
        .collect();
    let proof = Maturity {
        feature_evidence_digest: input.map(|i| i.digest()).transpose()?,
        policy: POLICY.into(),
        event: crate::DECISION_EVENT.into(),
        as_of: as_of.into(),
        operator: operator.into(),
        split: cfg,
        events_digest: digest(events)?,
        tasks_digest: digest(index)?,
        labels_digest: digest(labels)?,
        eligibility,
    };
    Ok((ds, proof))
}

fn digest<T: Serialize + ?Sized>(value: &T) -> Result<Hash, LearnError> {
    Ok(Hash::of(&[&serde_json::to_vec(value)?]))
}

/// Persist evidence separately from the candidate workspace. Call on a new directory.
pub fn write(
    ds: &Dataset,
    proof: &Maturity,
    events: &[(i64, Event)],
    index: &[TaskIndexRow],
    labels: &[LabelRecord],
    dir: &Path,
) -> Result<(), LearnError> {
    write_schema(ds, proof, events, index, labels, dir, None)
}
#[allow(clippy::too_many_arguments)]
pub fn write_schema(
    ds: &Dataset,
    proof: &Maturity,
    events: &[(i64, Event)],
    index: &[TaskIndexRow],
    labels: &[LabelRecord],
    dir: &Path,
    input: Option<&crate::schema::Input>,
) -> Result<(), LearnError> {
    let (rebuilt, expected) = build_schema(
        events,
        index,
        labels,
        &proof.operator,
        proof.split,
        ds.source_root,
        &proof.as_of,
        input,
    )?;
    if &rebuilt != ds || &expected != proof {
        return Err(invalid("maturity evidence disagrees with dataset"));
    }
    if dir.exists() && std::fs::read_dir(dir)?.next().is_some() {
        return Err(invalid(
            "maturity dataset must be written to a new or empty directory",
        ));
    }
    dataset::write_dataset(ds, dir)?;
    if let Some(input) = input {
        std::fs::write(
            dir.join("feature-evidence.json"),
            serde_json::to_vec_pretty(input)?,
        )?;
    }
    write_source_evidence(events, index, labels, dir)?;
    let bytes = serde_json::to_vec_pretty(proof)?;
    std::fs::write(dir.join("maturity.json"), &bytes)?;
    let path = dir.join("data-manifest.json");
    let mut manifest: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
    manifest["maturity_digest"] = serde_json::json!(Hash::of(&[&bytes]));
    std::fs::write(path, serde_json::to_vec_pretty(&manifest)?)?;
    Ok(())
}

/// Preserve legacy inspection/probe behavior without claiming an availability cutoff.
/// This produces no maturity proof: production fitting always refuses it.
pub fn write_diagnostic_snapshot(
    ds: &Dataset,
    events: &[(i64, Event)],
    index: &[TaskIndexRow],
    labels: &[LabelRecord],
    dir: &Path,
) -> Result<(), LearnError> {
    if dir.exists() && std::fs::read_dir(dir)?.next().is_some() {
        return Err(invalid(
            "diagnostic snapshot requires a new or empty directory",
        ));
    }
    dataset::write_dataset(ds, dir)?;
    write_source_evidence(events, index, labels, dir)?;
    std::fs::write(
        dir.join("snapshot-status.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "version":"diagnostic-evidence-snapshot@v1", "as_of":null,
            "maturity":"unknown", "fit_eligible":false,
            "source_root":ds.source_root, "events_digest":digest(events)?,
            "index_digest":digest(index)?, "labels_digest":digest(labels)?
        }))?,
    )?;
    Ok(())
}
fn write_source_evidence(
    events: &[(i64, Event)],
    index: &[TaskIndexRow],
    labels: &[LabelRecord],
    dir: &Path,
) -> Result<(), LearnError> {
    fn lines<T: Serialize>(path: &Path, rows: &[T]) -> Result<(), LearnError> {
        let mut text = String::new();
        for row in rows {
            text.push_str(&serde_json::to_string(row)?);
            text.push('\n');
        }
        std::fs::write(path, text)?;
        Ok(())
    }
    let records: Vec<_> = events
        .iter()
        .map(|(seq, event)| serde_json::json!({"seq":seq,"event":event}))
        .collect();
    lines(&dir.join("events.jsonl"), &records)?;
    lines(&dir.join("tasks.jsonl"), index)?;
    lines(&dir.join("labels.jsonl"), labels)?;
    Ok(())
}

/// Recompute, don't trust a resolved-label count or a claimed maturity flag.
pub fn binding(dir: &Path) -> Result<Hash, LearnError> {
    let manifest: crate::bundle::DataManifest =
        serde_json::from_slice(&std::fs::read(dir.join("data-manifest.json"))?)?;
    let expected=manifest.maturity_digest.ok_or_else(||invalid("legacy dataset: maturity unknown; rebuild from original evidence with explicit --as-of before fitting"))?;
    let bytes = std::fs::read(dir.join("maturity.json"))
        .map_err(|e| invalid(format!("maturity.json: {e}")))?;
    if Hash::of(&[&bytes]) != expected {
        return Err(invalid("maturity.json digest mismatch"));
    }
    Ok(expected)
}

pub fn load_schema_input(
    dir: &Path,
    proof: &Maturity,
) -> Result<Option<crate::schema::Input>, LearnError> {
    match proof.feature_evidence_digest {
        None => Ok(None),
        Some(expected) => {
            let input = crate::schema::Input::load(dir)?;
            if input.digest()? != expected {
                return Err(invalid("feature evidence digest mismatch"));
            }
            Ok(Some(input))
        }
    }
}
pub fn validate(dir: &Path, ds: &Dataset) -> Result<Hash, LearnError> {
    let expected = binding(dir)?;
    let bytes = std::fs::read(dir.join("maturity.json"))?;
    let proof: Maturity =
        serde_json::from_slice(&bytes).map_err(|e| invalid(format!("maturity.json: {e}")))?;
    let events = dataset::read_events_records(&dir.join("events.jsonl"))?;
    let index = dataset::read_task_index(&dir.join("tasks.jsonl"))?;
    // The writer represents an explicitly unlabeled snapshot with a zero-byte file.
    // Its empty label set is still bound by the maturity proof.
    let label_path = dir.join("labels.jsonl");
    let labels = if std::fs::metadata(&label_path)?.len() == 0 {
        Vec::new()
    } else {
        dataset::read_labels(&label_path)?
    };
    let input = load_schema_input(dir, &proof)?;
    let (rebuilt, recomputed) = build_schema(
        &events,
        &index,
        &labels,
        &proof.operator,
        proof.split,
        ds.source_root,
        &proof.as_of,
        input.as_ref(),
    )?;
    if proof != recomputed || ds != &rebuilt {
        return Err(invalid(
            "maturity provenance or rows disagree with recomputed as-of evidence",
        ));
    }
    Ok(expected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::{Side, SplitBy, SplitFractions};
    use crate::decision::DiffLabel;
    use tsr_domain::{ActionKind, Candidate, Decision, Source, Utility};

    type Inputs = (Vec<(i64, Event)>, Vec<TaskIndexRow>, Vec<LabelRecord>);
    fn inputs() -> Inputs {
        let mut events = Vec::new();
        let mut index = Vec::new();
        let mut labels = Vec::new();
        for i in 1..=4 {
            let task = TaskId(Hash::of(&[format!("task{i}").as_bytes()]));
            let candidate = Candidate::new(
                ActionKind::ProposePatch,
                Source::Generator,
                Hash::of(&[b"patch"]),
                vec![],
                vec![],
            )
            .unwrap();
            let cid = candidate.id;
            let kinds = vec![
                EventKind::TaskOpened { task },
                EventKind::Proposed { candidate },
                EventKind::Committed {
                    candidate: cid,
                    decision: Decision::Selected {
                        value: cid,
                        utility: Utility {
                            p_success: 0.5,
                            value: 0.,
                            cost: 0.,
                            ev: 0.,
                        },
                        propensity: 1.,
                        evidence: vec![],
                    },
                },
                EventKind::TaskClosed {
                    task,
                    reason: "done".into(),
                },
            ];
            for (step, kind) in kinds.into_iter().enumerate() {
                let at = if step == 3 {
                    "2026-01-02T00:00:00.123Z".into()
                } else {
                    format!("2026-01-01T00:00:0{step}Z")
                };
                let event = Event::new(task, vec![], Source::Rules, kind, at);
                events.push((events.len() as i64 + 1, event));
            }
            index.push(TaskIndexRow {
                task,
                family: "repair".into(),
                lineage: format!("repo{i}"),
                timestamp: format!("2026-01-01T0{i}:00:00Z"),
                clone_group: None,
                tool_ms_ceiling: None,
            });
            labels.push(LabelRecord {
                task,
                criteria_met: true,
                diff_label: Some(DiffLabel::Accepted),
                reviewer: "operator".into(),
                recorded_at: "2026-01-02T01:00:00Z".into(),
            });
        }
        (events, index, labels)
    }
    fn snapshot(input: &Inputs, at: &str) -> (Dataset, Maturity) {
        build(
            &input.0,
            &input.1,
            &input.2,
            "operator",
            SplitConfig {
                by: SplitBy::Time,
                fractions: SplitFractions {
                    calibrate: 0.25,
                    holdout: 0.25,
                },
                seed: 1,
            },
            Hash::of(&[b"source"]),
            at,
        )
        .unwrap()
    }
    const AT: &str = "2026-01-09T00:00:00.123Z";

    #[test]
    fn diagnostic_snapshot_is_reproducible_and_cannot_claim_maturity() {
        let input = inputs();
        let (_, proof) = snapshot(&input, AT);
        let ds = dataset::build_dataset(
            &input.0,
            &input.1,
            &input.2,
            "operator",
            proof.split,
            Hash::of(&[b"source"]),
        )
        .unwrap();
        let first = tempfile::tempdir().unwrap();
        let second = tempfile::tempdir().unwrap();
        for dir in [first.path(), second.path()] {
            write_diagnostic_snapshot(&ds, &input.0, &input.1, &input.2, dir).unwrap();
            assert_eq!(crate::bundle::load_dataset(dir).unwrap(), ds);
            assert!(!dir.join("maturity.json").exists());
            let out = dir.join("model");
            let error = crate::bundle::write_bundle(dir, &out, 30)
                .unwrap_err()
                .to_string();
            assert!(error.contains("maturity unknown"), "{error}");
            assert!(!out.exists());
        }
        for name in [
            "rows.jsonl",
            "events.jsonl",
            "tasks.jsonl",
            "labels.jsonl",
            "data-manifest.json",
            "snapshot-status.json",
        ] {
            assert_eq!(
                std::fs::read(first.path().join(name)).unwrap(),
                std::fs::read(second.path().join(name)).unwrap(),
                "{name}"
            );
        }
        // The diagnostic status is explanatory, never a fitting authority.
        std::fs::write(
            first.path().join("snapshot-status.json"),
            br#"{"maturity":"mature","fit_eligible":true}"#,
        )
        .unwrap();
        assert!(binding(first.path())
            .unwrap_err()
            .to_string()
            .contains("maturity unknown"));
        assert!(
            write_diagnostic_snapshot(&ds, &input.0, &input.1, &input.2, first.path()).is_err()
        );
    }

    #[test]
    fn clocks_offsets_leap_dates_and_fraction_boundaries() {
        assert_eq!(timestamp("1970-01-01T00:00:00Z").unwrap(), 0);
        assert_eq!(
            timestamp(AT).unwrap(),
            timestamp("2026-01-08T19:00:00.123-05:00").unwrap()
        );
        assert_eq!(
            timestamp("2024-03-01T00:00:00Z").unwrap() - timestamp("2024-02-28T00:00:00Z").unwrap(),
            172_800_000_000_000
        );
        for bad in [
            "",
            "2026-02-29T00:00:00Z",
            "2026-01-01T00:00:60Z",
            "2026-01-01T00:00:00Z#x",
            "2026-01-01T00:00:00-00:00",
            "2026-01-01T00:00:00.1234567890Z",
            "2026-01-01T00:00:00+24:00",
        ] {
            assert!(timestamp(bad).is_err(), "{bad}");
        }
    }
    #[test]
    fn closed_reviewed_is_pending_until_exact_horizon_and_restart_is_stable() {
        let input = inputs();
        let (pending, p) = snapshot(&input, "2026-01-09T00:00:00.122999999Z");
        assert!(pending.rows.iter().all(|r| r.label.is_none()));
        assert!(p.eligibility.iter().all(|e| e.status == "pending_horizon"));
        let (ds, proof) = snapshot(&input, AT);
        assert!(ds.rows.iter().all(|r| r.label == Some(true)));
        assert_eq!((ds.clone(), proof.clone()), snapshot(&input, AT));
        let dir = tempfile::tempdir().unwrap();
        write(&ds, &proof, &input.0, &input.1, &input.2, dir.path()).unwrap();
        let loaded = crate::bundle::load_dataset(dir.path()).unwrap();
        assert_eq!(loaded, ds);
        validate(dir.path(), &loaded).unwrap();
        let (offset, _) = snapshot(&input, "2026-01-08T19:00:00.123-05:00");
        assert_eq!(offset.rows, ds.rows);
        assert_eq!(
            ds.rows
                .iter()
                .filter(|r| r.side != Side::HeldOut && r.label.is_some())
                .map(|r| r.task)
                .collect::<BTreeSet<_>>()
                .len(),
            3
        );
    }
    #[test]
    fn future_late_missing_and_operator_stopped_reviews_do_not_mature() {
        let mut input = inputs();
        input.2[0].recorded_at = "2026-01-10T00:00:00Z".into();
        input.2[1].diff_label = None;
        let task = input.2[2].task;
        input.0.push((
            99,
            Event::new(
                task,
                vec![],
                Source::Rules,
                EventKind::Stop {
                    reason: "operator".into(),
                },
                "2026-01-03T00:00:00Z",
            ),
        ));
        let (ds, p) = snapshot(&input, "2026-01-20T00:00:00Z");
        assert_eq!(ds.rows.iter().filter(|r| r.label.is_some()).count(), 1);
        assert_eq!(
            p.eligibility
                .iter()
                .filter(|r| r.status == "censored")
                .count(),
            3
        );
        let mut open = inputs();
        open.0
            .retain(|(_, e)| !matches!(e.kind, EventKind::TaskClosed { .. }));
        let (ds, p) = snapshot(&open, AT);
        assert!(ds.rows.iter().all(|r| r.label.is_none()));
        assert!(p.eligibility.iter().all(|r| r.status == "open"));
    }
    #[test]
    fn future_returns_are_excluded_and_late_budget_cannot_relabel_horizon() {
        let mut input = inputs();
        let control = snapshot(&input, AT).0;
        input.0.push((
            99,
            Event::new(
                input.2[0].task,
                vec![],
                Source::Rules,
                EventKind::Stop {
                    reason: "budget".into(),
                },
                "2026-01-10T00:00:00Z",
            ),
        ));
        assert_eq!(control, snapshot(&input, AT).0);
        assert_eq!(
            control.rows,
            snapshot(&input, "2026-01-20T00:00:00Z").0.rows
        );
    }
    #[test]
    fn fit_refuses_legacy_and_premature_relabel_even_with_new_row_roots() {
        let input = inputs();
        let (ds, proof) = snapshot(&input, AT);
        let legacy = tempfile::tempdir().unwrap();
        dataset::write_dataset(&ds, legacy.path()).unwrap();
        assert_eq!(crate::bundle::load_dataset(legacy.path()).unwrap(), ds);
        let out = tempfile::tempdir().unwrap();
        assert!(crate::bundle::write_bundle(legacy.path(), out.path(), 30)
            .unwrap_err()
            .to_string()
            .contains("maturity unknown"));
        let dir = tempfile::tempdir().unwrap();
        write(&ds, &proof, &input.0, &input.1, &input.2, dir.path()).unwrap();
        let path = dir.path().join("maturity.json");
        let mut forged = proof;
        forged.as_of = "2026-01-08T00:00:00Z".into();
        let bytes = serde_json::to_vec_pretty(&forged).unwrap();
        std::fs::write(path, &bytes).unwrap();
        let path = dir.path().join("data-manifest.json");
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        manifest["maturity_digest"] = serde_json::json!(Hash::of(&[&bytes]));
        std::fs::write(path, serde_json::to_vec(&manifest).unwrap()).unwrap();
        assert!(crate::bundle::write_bundle(dir.path(), out.path(), 30)
            .unwrap_err()
            .to_string()
            .contains("recomputed as-of evidence"));
    }
    #[test]
    fn invalid_clocks_identity_duplicate_and_policy_are_refused() {
        let mut input = inputs();
        input.2.push(input.2[0].clone());
        let cfg = SplitConfig {
            by: SplitBy::Time,
            fractions: SplitFractions {
                calibrate: 0.25,
                holdout: 0.25,
            },
            seed: 1,
        };
        assert!(build(
            &input.0,
            &input.1,
            &input.2,
            "operator",
            cfg,
            Hash::of(&[]),
            AT
        )
        .is_err());
        input = inputs();
        input.0[0].1.recorded_at = "2026-01-01T00:00:10Z".into();
        assert!(build(
            &input.0,
            &input.1,
            &input.2,
            "operator",
            cfg,
            Hash::of(&[]),
            AT
        )
        .unwrap_err()
        .to_string()
        .contains("identity"));
        input = inputs();
        input.2[0].recorded_at = "unknown".into();
        assert!(build(
            &input.0,
            &input.1,
            &input.2,
            "operator",
            cfg,
            Hash::of(&[]),
            AT
        )
        .is_err());
        let input = inputs();
        let (ds, mut proof) = snapshot(&input, AT);
        proof.policy = "invented".into();
        assert!(write(
            &ds,
            &proof,
            &input.0,
            &input.1,
            &input.2,
            tempfile::tempdir().unwrap().path()
        )
        .is_err());
    }
    #[test]
    fn future_precommitment_observation_uses_same_view_in_probes() {
        let mut input = inputs();
        let baseline = snapshot(&input, AT).0;
        let candidate = Candidate::new(
            ActionKind::Inspect,
            Source::Generator,
            Hash::of(&[b"future"]),
            vec![],
            vec![],
        )
        .unwrap();
        let event = Event::new(
            input.2[0].task,
            vec![],
            Source::Rules,
            EventKind::Proposed { candidate },
            "2026-02-01T00:00:00Z",
        );
        input.0.insert(1, (100, event));
        for (i, (seq, _)) in input.0.iter_mut().enumerate() {
            *seq = i as i64 + 1;
        }
        let (ds, _) = snapshot(&input, AT);
        assert_eq!(ds, baseline);
        let available = available_events(&input.0, AT).unwrap();
        let families = crate::features::families_from_spec(&ds.spec);
        let extractor = crate::probes::default_extractor(&ds.spec, &families);
        let report = crate::probes::run_all(&ds, &available, &input.1, &ds.spec, &extractor, None);
        assert!(report.is_clean(), "{report:?}");
    }

    #[test]
    fn future_closure_is_open_and_post_close_commitment_is_refused() {
        let mut input = inputs();
        let (_, closure) = &mut input.0[3];
        *closure = Event::new(
            closure.task,
            vec![],
            closure.source,
            closure.kind.clone(),
            "2026-02-01T00:00:00Z",
        );
        let (ds, proof) = snapshot(&input, AT);
        assert!(ds
            .rows
            .iter()
            .find(|r| r.task == input.2[0].task)
            .unwrap()
            .label
            .is_none());
        assert_eq!(
            proof
                .eligibility
                .iter()
                .find(|e| e.task == input.2[0].task)
                .unwrap()
                .status,
            "open"
        );
        let mut input = inputs();
        let commitment = &input.0[2].1;
        input.0.push((
            100,
            Event::new(
                commitment.task,
                vec![],
                commitment.source,
                commitment.kind.clone(),
                "2026-01-03T00:00:00Z",
            ),
        ));
        assert!(available_events(&input.0, AT)
            .unwrap_err()
            .to_string()
            .contains("after closure"));
    }
    #[test]
    fn duplicate_or_reordered_sequence_is_not_chronology_evidence() {
        let mut input = inputs();
        input.0[1].0 = input.0[0].0;
        assert!(available_events(&input.0, AT)
            .unwrap_err()
            .to_string()
            .contains("unique"));
        let mut input = inputs();
        input.0.swap(0, 1);
        assert!(available_events(&input.0, AT)
            .unwrap_err()
            .to_string()
            .contains("strictly increasing"));
    }
}
