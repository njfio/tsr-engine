//! Shadow-only pre-action application; never a decision/activation authority.
use crate::{bundle, dataset, features, maturity, LearnError};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeSet, path::PathBuf};
use tsr_domain::*;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub bundle_directory: PathBuf,
    pub bundle_id: Hash,
    pub dataset_directory: Option<PathBuf>,
    pub event: String,
    pub family: String,
    pub lineage: String,
    pub training_as_of: Option<String>,
}
pub struct Snapshot<'a> {
    pub tool_ms_ceiling: Option<u64>,
    pub state: Hash,
    pub source_version: Hash,
    pub candidate: &'a Candidate,
    pub events: &'a [(i64, Event)],
    pub as_of: &'a str,
    /// Controller adapter identity, not generator/operator attestation.
    pub adapter: &'a str,
}
fn invalid(s: &str) -> LearnError {
    LearnError::Invalid(s.into())
}
/// Copy explicitly named artifacts using bounded reads, then let existing loaders
/// validate the private snapshot. Replacement cannot enlarge subsequent reads.
fn bounded_snapshot(
    dir: &std::path::Path,
    names: &[&str],
) -> Result<tempfile::TempDir, LearnError> {
    use std::io::Read;
    let snapshot = tempfile::tempdir()?;
    let mut total = 0;
    for name in names {
        let path = dir.join(name);
        let metadata = std::fs::symlink_metadata(&path)?;
        if !metadata.is_file() || metadata.file_type().is_symlink() {
            return Err(invalid("shadow artifact symlink"));
        }
        let file = std::fs::File::open(path)?;
        if !file.metadata()?.is_file() {
            return Err(invalid("shadow artifact must be regular"));
        }
        let mut bytes = Vec::new();
        file.take(16 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
        total += bytes.len();
        if bytes.len() > 16 * 1024 * 1024 || total > 64 * 1024 * 1024 {
            return Err(invalid("shadow artifacts exceed byte budget"));
        }
        std::fs::write(snapshot.path().join(name), bytes)?;
    }
    Ok(snapshot)
}
pub fn evaluate(config: &Config, snapshot: Snapshot<'_>) -> Result<serde_json::Value, LearnError> {
    evaluate_receipts(config, snapshot, None)
}
pub fn evaluate_receipts(
    config: &Config,
    snapshot: Snapshot<'_>,
    receipt: Option<&crate::project_receipts::Evidence>,
) -> Result<serde_json::Value, LearnError> {
    if config.event != crate::DECISION_EVENT {
        return Err(invalid("shadow event mismatch"));
    }
    maturity::timestamp(snapshot.as_of)?;
    if snapshot.events.len() > 100_000 {
        return Err(invalid("shadow prefix too large"));
    }
    if snapshot.events.iter().any(|(_,e)|matches!(e.kind,EventKind::Committed{candidate,..}|EventKind::Executed{candidate,..} if candidate==snapshot.candidate.id)) {return Err(invalid("candidate already committed or executed"))}
    let frozen = bounded_snapshot(
        &config.bundle_directory,
        &[
            "manifest.toml",
            "head.json",
            "isotonic.json",
            "features.json",
            "split.json",
            "support.json",
        ],
    )?;
    let model = bundle::load_bundle(frozen.path())?;
    evaluate_model_receipts(config, snapshot, &model, receipt, None)
}
/// Fresh pinned evaluation; a saved shadow receipt never authorizes selection.
pub fn evaluate_activation(
    config: &Config,
    snapshot: Snapshot<'_>,
    receipt: &crate::project_receipts::Evidence,
    report_digest: Hash,
    clone_group: &str,
    original_task: Hash,
) -> Result<serde_json::Value, LearnError> {
    if config.event != crate::DECISION_EVENT
        || snapshot.adapter != "project-receipt-common-event@v1"
    {
        return Err(invalid(
            "activation requires matching project common event adapter",
        ));
    }
    if snapshot.events.len()>100_000 || snapshot.events.iter().any(|(_,e)|matches!(e.kind,EventKind::Committed{candidate,..}|EventKind::Executed{candidate,..} if candidate==snapshot.candidate.id)) {return Err(invalid("invalid pre-action activation prefix"));}
    let frozen = bounded_snapshot(
        &config.bundle_directory,
        &[
            "manifest.toml",
            "head.json",
            "isotonic.json",
            "features.json",
            "split.json",
            "support.json",
        ],
    )?;
    let model = bundle::load_bundle(frozen.path())?;
    evaluate_model_receipts(
        config,
        snapshot,
        &model,
        Some(receipt),
        Some((report_digest, clone_group.to_owned(), original_task)),
    )
}
#[cfg(test)]
fn evaluate_model(
    config: &Config,
    snapshot: Snapshot<'_>,
    model: &bundle::Bundle,
) -> Result<serde_json::Value, LearnError> {
    evaluate_model_receipts(config, snapshot, model, None, None)
}
fn evaluate_model_receipts(
    config: &Config,
    snapshot: Snapshot<'_>,
    model: &bundle::Bundle,
    receipt: Option<&crate::project_receipts::Evidence>,
    activation: Option<(Hash, String, Hash)>,
) -> Result<serde_json::Value, LearnError> {
    let cutoff = maturity::timestamp(snapshot.as_of)?;
    if model.spec.names.len() > 128 || model.support_index.len() > 5000 {
        return Err(invalid("shadow model work bound"));
    }
    if model.id != config.bundle_id {
        return Err(invalid("shadow bundle pin mismatch"));
    }
    let mut result = serde_json::json!({"status":"unavailable","event":crate::DECISION_EVENT,"event_id":crate::decision_event_id(),
        "candidate":snapshot.candidate.id,"action":snapshot.candidate.kind,"state_root":snapshot.state,"source_version":snapshot.source_version,
        "prefix_digest":Hash::of(&[&serde_json::to_vec(snapshot.events)?]),"prefix_events":snapshot.events.len(),"prediction_as_of":snapshot.as_of,
        "bundle_id":model.id,"training_root":model.manifest.training_root,"feature_schema":model.spec.version,
        "maturity_digest":model.manifest.maturity_digest,"training_as_of":config.training_as_of,"adapter":snapshot.adapter,
        "declared_family":config.family,"declared_lineage":config.lineage,"selection_influence":false,"calibration_quality":"not established by maturity or artifact integrity","label":null});
    // Engine adapters currently lack the native budget/close lifecycle and execution
    // features. Do not reinterpret progress as task acceptance or missing counts as zero.
    let project = snapshot.adapter == "project-receipt-common-event@v1";
    if (snapshot.adapter != "native-common-event@v1" && !project)
        || !crate::decision::is_gated(snapshot.candidate.kind)
    {
        result["reason"] = "unsupported_common_event_lifecycle_or_action".into();
        return Ok(result);
    }
    let task = snapshot
        .events
        .first()
        .map(|(_, e)| e.task)
        .ok_or_else(|| invalid("missing task prefix"))?;
    if snapshot.events.iter().any(|(_, e)| e.task != task)
        || !snapshot
            .events
            .iter()
            .any(|(_, e)| matches!(e.kind, EventKind::TaskOpened { .. }))
        || snapshot
            .events
            .iter()
            .any(|(_, e)| matches!(e.kind, EventKind::TaskClosed { .. }))
    {
        return Err(invalid("non-live native task prefix"));
    }
    let visible = maturity::available_events(snapshot.events, snapshot.as_of)?;
    if visible != snapshot.events {
        return Err(invalid("prefix includes evidence after as_of"));
    }
    let index = dataset::TaskIndexRow {
        task,
        family: config.family.clone(),
        lineage: config.lineage.clone(),
        timestamp: snapshot.as_of.into(),
        clone_group: None,
        tool_ms_ceiling: snapshot.tool_ms_ceiling,
    };
    let vector = if project {
        let evidence =
            receipt.ok_or_else(|| invalid("missing pre-action project receipt evidence"))?;
        if evidence.as_of != snapshot.as_of {
            return Err(invalid("project receipt cutoff mismatch"));
        }
        crate::schema::receipt_vector(&model.spec, evidence, &index, snapshot.candidate.kind)?
    } else {
        crate::schema::extract(
            &model.spec,
            snapshot.events,
            &index,
            snapshot.candidate.kind,
            None,
            None,
        )?
    };
    result["features"] = serde_json::to_value(&vector)?;
    result["feature_digest"] = serde_json::to_value(Hash::of(&[&serde_json::to_vec(&vector)?]))?;
    result["generator_common_event_forecast"] = serde_json::Value::Null;
    let applied = model.apply_features(&config.family, &vector)?;
    if !project {
        result["raw_probability"] = applied.raw_p.into();
    }
    result["matching_rows"] = applied.support.episodes.into();
    result["calibrated_probability"] = serde_json::Value::Null;
    result["feature_limit"]="features@v1 is history/action/family based, not patch-content sensitive; family/lineage are declared metadata".into();
    let (Some(expected), Some(dir), Some(as_of)) = (
        model.manifest.maturity_digest,
        config.dataset_directory.as_ref(),
        config.training_as_of.as_ref(),
    ) else {
        result["reason"] = "legacy_or_missing_maturity_provenance".into();
        return Ok(result);
    };
    if maturity::timestamp(as_of)? > cutoff {
        return Err(invalid("training as_of is later than prediction"));
    }
    let mut names = vec![
        "data-manifest.json",
        "rows.jsonl",
        "features.json",
        "split.json",
        "events.jsonl",
        "tasks.jsonl",
        "labels.jsonl",
        "maturity.json",
    ];
    if project {
        names.push("feature-evidence.json");
    }
    let frozen = bounded_snapshot(dir, &names)?;
    let dir = frozen.path();
    let ds = bundle::load_dataset(dir)?;
    if ds.rows.len() > 5000 {
        return Err(invalid("shadow dataset row bound"));
    }
    if maturity::validate(dir, &ds)? != expected
        || ds.training_root.to_hex() != model.manifest.training_root
        || ds.data_root.to_hex() != model.manifest.data_manifest
    {
        return Err(invalid(
            "bundle/dataset maturity or training binding mismatch",
        ));
    }
    let proof: maturity::Maturity =
        serde_json::from_slice(&std::fs::read(dir.join("maturity.json"))?)?;
    if proof.as_of != *as_of || proof.event != config.event {
        return Err(invalid("maturity as_of/event mismatch"));
    }
    if ds
        .rows
        .iter()
        .any(|row| row.task == task || row.lineage == config.lineage)
    {
        return Err(invalid(
            "query task/lineage overlaps training/evaluation dataset",
        ));
    }
    let (total, family, within) = distinct_support(&ds, &model.spec, &config.family, &vector);
    result["distinct_mature_training_tasks"] = total.into();
    result["distinct_family_tasks"] = family.into();
    result["distinct_matching_tasks"] = within.into();
    result["maturity_policy"] = proof.policy.clone().into();
    let supported =
        total >= 150 && family >= 30 && within >= model.manifest.support_floor.max(30) as usize;
    result["reason"] = if supported {
        "mature_support_checked_calibration_quality_unverified"
    } else {
        "insufficient_distinct_mature_support"
    }
    .into();
    result["status"] = "shadow_diagnostic_only".into();
    // Numerical calibrator output remains a diagnostic, never a quality/activation claim.
    if !project || supported {
        result["raw_probability"] = applied.raw_p.into();
        result["calibrator_diagnostic"] = serde_json::to_value(&applied.calibrated)?;
    }
    if let Some((expected_report, query_clone, query_original)) = activation {
        if !project || model.spec.version != crate::project_receipts::SCHEMA || !supported {
            return Err(invalid("activation schema or mature support gate"));
        }
        let input = crate::maturity::load_schema_input(dir, &proof)?
            .ok_or_else(|| invalid("activation requires prospective provenance"))?;
        let labels = input.labels(as_of)?;
        let eligible: BTreeSet<_> = labels.iter().map(|l| l.task).collect();
        if ds
            .rows
            .iter()
            .any(|r| r.label.is_some() && !eligible.contains(&r.task))
        {
            return Err(invalid("resolved row lacks operational eligibility"));
        }
        let index = crate::dataset::read_task_index(&dir.join("tasks.jsonl"))?;
        if index
            .iter()
            .any(|r| r.clone_group.as_deref() == Some(query_clone.as_str()))
        {
            return Err(invalid("query clone overlaps training/evaluation dataset"));
        }
        for proof in &input.eligibility {
            let artifact = input
                .artifacts
                .iter()
                .find(|a| {
                    a.export_time_lifecycle_inventory["task"]
                        == serde_json::to_value(proof.task).unwrap()
                })
                .ok_or_else(|| invalid("missing eligibility artifact"))?;
            if crate::project_eligibility::evaluate(artifact, proof, as_of)?.original_task
                == Some(query_original)
            {
                return Err(invalid(
                    "query original task overlaps training/evaluation dataset",
                ));
            }
        }
        let events = crate::bundle::read_events(dir)?;
        let available = maturity::available_events(&events, as_of)?;
        let extractor = crate::probes::schema_extractor(&ds.spec, Some(&input));
        if !crate::probes::run_all(
            &ds,
            &available,
            &index,
            &ds.spec,
            &extractor,
            Some(&dir.join("split.json")),
        )
        .is_clean()
        {
            return Err(invalid("activation leakage validation failed"));
        }
        crate::report::check_dataset_matches_bundle(model, &ds)?;
        let report = crate::report::report(model, &ds, 0);
        let digest = Hash::of(&[&serde_json::to_vec(&report)?]);
        if digest != expected_report {
            return Err(invalid("calibration validation report pin mismatch"));
        }
        validate_activation_report(&report, &config.family)?;
        let calibrated = applied
            .calibrated
            .as_ref()
            .ok_or_else(|| invalid("missing calibrated probability"))?;
        result["status"] = "activation_eligible".into();
        result["calibrated_probability"] = serde_json::to_value(calibrated.p)?;
        result["validation_report_digest"] = serde_json::to_value(digest)?;
        result["calibration_quality"] =
            "heldout paired improvement checked; no guarantee for future outcomes".into();
    }
    Ok(result)
}

/// Conservative v1 activation rule; not causal superiority or future reliability.
fn validate_activation_report(
    report: &crate::report::Report,
    family: &str,
) -> Result<(), LearnError> {
    for paired in [Some(&report.paired), report.paired_by_family.get(family)] {
        let p = paired.ok_or_else(|| invalid("missing family calibration validation"))?;
        if p.clusters < 2
            || p.tasks < 2
            || p.inconclusive
            || !p.mean.is_some_and(|v| v.is_finite() && v < 0.0)
            || !p
                .interval
                .is_some_and(|(lo, hi)| lo.is_finite() && hi.is_finite() && lo <= hi && hi < 0.0)
        {
            return Err(invalid(
                "heldout paired calibration improvement not established",
            ));
        }
    }
    Ok(())
}

fn distinct_support(
    ds: &dataset::Dataset,
    spec: &features::FeatureSpec,
    query_family: &str,
    vector: &features::FeatureVector,
) -> (usize, usize, usize) {
    let mut total = BTreeSet::new();
    let mut family = BTreeSet::new();
    let mut within = BTreeSet::new();
    for row in &ds.rows {
        if row.label.is_none() || row.side == dataset::Side::HeldOut {
            continue;
        }
        total.insert(row.task);
        if row.family == query_family {
            family.insert(row.task);
            let entry = crate::support::SupportEntry {
                family: row.family.clone(),
                features: row.features.clone(),
            };
            if crate::support::support(spec, &[entry], query_family, vector).episodes == 1 {
                within.insert(row.task);
            }
        }
    }
    (total.len(), family.len(), within.len())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    fn fixture() -> (Config, bundle::Bundle, Candidate, Vec<(i64, Event)>) {
        let id = Hash::of(&[b"synthetic-shadow-model"]);
        let mut spec = features::spec_for_families(&["fixture".into()]);
        spec.standardize_mean = vec![0.0; spec.names.len()];
        spec.standardize_std = vec![1.0; spec.names.len()];
        let split = dataset::SplitManifest {
            by: dataset::SplitBy::Time,
            seed: 0,
            assignments: BTreeMap::new(),
            digest: id,
        };
        let manifest = bundle::Manifest {
            maturity_digest: None,
            data_manifest: id.to_hex(),
            training_root: id.to_hex(),
            feature_schema: spec.version.clone(),
            code_commit: "fixture".into(),
            seed: 0,
            calibrator: "fixture".into(),
            support_floor: 30,
            split_digest: id.to_hex(),
            source_root: id.to_hex(),
            state: "draft".into(),
            id: id.to_hex(),
            content_digests: BTreeMap::new(),
        };
        let model = bundle::Bundle {
            id,
            dir: PathBuf::new(),
            manifest,
            head: crate::head::Head {
                weights: vec![0.0; spec.names.len()],
                bias: 0.0,
                l2: 0.0,
                iters: 0,
            },
            isotonic: crate::isotonic::Isotonic {
                breakpoints: vec![(0.0, 0.7)],
            },
            spec,
            split,
            support_index: vec![],
            population: BTreeMap::from([("fixture".into(), 300)]),
        };
        let config = Config {
            bundle_directory: PathBuf::new(),
            bundle_id: id,
            dataset_directory: None,
            event: crate::DECISION_EVENT.into(),
            family: "fixture".into(),
            lineage: "fixture".into(),
            training_as_of: None,
        };
        let candidate = Candidate::new(
            ActionKind::ProposePatch,
            Source::Intuition,
            id,
            vec![],
            vec![],
        )
        .unwrap();
        let task = TaskId(id);
        let event = Event::new(
            task,
            vec![],
            Source::Operator,
            EventKind::TaskOpened { task },
            "2026-01-01T00:00:00Z",
        );
        (config, model, candidate, vec![(1, event)])
    }
    #[test]
    fn legacy_numeric_diagnostic_is_not_calibration_or_adapter_compatibility() {
        let (config, model, candidate, events) = fixture();
        let query = |adapter| Snapshot {
            tool_ms_ceiling: None,
            state: model.id,
            source_version: model.id,
            candidate: &candidate,
            events: &events,
            as_of: "2026-02-01T00:00:00Z",
            adapter,
        };
        let p = evaluate_model(&config, query("native-common-event@v1"), &model).unwrap();
        assert_eq!(p["raw_probability"], 0.5); // independent sigmoid(0) check
        assert!(p["calibrated_probability"].is_null());
        assert_eq!(p["reason"], "legacy_or_missing_maturity_provenance");
        let features: features::FeatureVector =
            serde_json::from_value(p["features"].clone()).unwrap();
        assert_eq!(features.get(&model.spec, "generator_missing"), Some(1.0));
        for adapter in ["notebook@v1", "bounded-project@v1"] {
            let p = evaluate_model(&config, query(adapter), &model).unwrap();
            assert_eq!(p["reason"], "unsupported_common_event_lifecycle_or_action");
            assert!(p.get("raw_probability").is_none());
        }
        let mut mismatch = config.clone();
        mismatch.bundle_id = Hash::of(&[b"wrong"]);
        assert!(evaluate_model(&mismatch, query("native-common-event@v1"), &model).is_err());
        assert!(evaluate_model(
            &config,
            Snapshot {
                as_of: "2025-01-01T00:00:00Z",
                ..query("native-common-event@v1")
            },
            &model
        )
        .is_err());
    }
    #[test]
    fn repeated_rows_are_not_distinct_mature_support() {
        let (config, model, _, events) = fixture();
        let index = dataset::TaskIndexRow {
            task: events[0].1.task,
            family: config.family.clone(),
            lineage: config.lineage.clone(),
            timestamp: "2026-01-01T00:00:00Z".into(),
            clone_group: None,
            tool_ms_ceiling: None,
        };
        let vector = features::extract(
            &model.spec,
            &events,
            &index,
            ActionKind::ProposePatch,
            &["fixture".into()],
            None,
        );
        let row = dataset::Row {
            id: model.id,
            task: index.task,
            commitment: events[0].1.id,
            index: 1,
            action: ActionKind::ProposePatch,
            family: config.family,
            lineage: config.lineage,
            side: dataset::Side::Fit,
            features: vector.clone(),
            label: Some(true),
            evidence: vec![],
            generator_raw: None,
            operator_raw: None,
            controller_logged: None,
        };
        let ds = dataset::Dataset {
            rows: vec![row; 200],
            split: model.split.clone(),
            spec: model.spec.clone(),
            data_root: model.id,
            training_root: model.id,
            source_root: model.id,
            unindexed_tasks: 0,
            uncommitted_tasks: 0,
        };
        assert_eq!(
            distinct_support(&ds, &model.spec, "fixture", &vector),
            (1, 1, 1)
        );
    }
    #[test]
    fn event_mismatch_and_already_returned_candidate_rejected_before_artifact_read() {
        let (mut config, model, candidate, mut events) = fixture();
        config.event = "notebook-progress@v1".into();
        assert!(evaluate(
            &config,
            Snapshot {
                tool_ms_ceiling: None,
                state: model.id,
                source_version: model.id,
                candidate: &candidate,
                events: &events,
                as_of: "2026-02-01T00:00:00Z",
                adapter: "native-common-event@v1"
            }
        )
        .unwrap_err()
        .to_string()
        .contains("event mismatch"));
        config.event = crate::DECISION_EVENT.into();
        events.push((
            2,
            Event::new(
                events[0].1.task,
                vec![events[0].1.id],
                Source::Rules,
                EventKind::Executed {
                    candidate: candidate.id,
                    exit_code: Some(0),
                    log_digest: None,
                    duration_ms: 1,
                },
                "2026-01-02T00:00:00Z",
            ),
        ));
        assert!(evaluate(
            &config,
            Snapshot {
                tool_ms_ceiling: None,
                state: model.id,
                source_version: model.id,
                candidate: &candidate,
                events: &events,
                as_of: "2026-02-01T00:00:00Z",
                adapter: "native-common-event@v1"
            }
        )
        .unwrap_err()
        .to_string()
        .contains("already committed"));
    }
    #[test]
    fn artifact_snapshot_is_bounded_and_survives_source_replacement() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("test"), b"original").unwrap();
        std::fs::create_dir(dir.path().join("nonregular")).unwrap();
        assert!(bounded_snapshot(dir.path(), &["nonregular"]).is_err());
        let snapshot = bounded_snapshot(dir.path(), &["test"]).unwrap();
        std::fs::write(dir.path().join("test"), b"replaced").unwrap();
        assert_eq!(
            std::fs::read(snapshot.path().join("test")).unwrap(),
            b"original"
        );
        std::fs::File::create(dir.path().join("test"))
            .unwrap()
            .set_len(16 * 1024 * 1024 + 1)
            .unwrap();
        assert!(bounded_snapshot(dir.path(), &["test"]).is_err());
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(dir.path().join("test"), dir.path().join("alias")).unwrap();
            assert!(bounded_snapshot(dir.path(), &["alias"]).is_err());
        }
    }
    #[test]
    fn real_artifact_pipeline_binds_maturity_and_rejects_overlap_and_tampering() {
        let dir = tempfile::tempdir().unwrap();
        let data = dir.path().join("data");
        let out = dir.path().join("bundle");
        let fixtures = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/m4");
        let events = dataset::read_events_records(&fixtures.join("events.jsonl")).unwrap();
        let index = dataset::read_task_index(&fixtures.join("tasks.jsonl")).unwrap();
        let labels = dataset::read_labels(&fixtures.join("labels.jsonl")).unwrap();
        let split = dataset::SplitConfig {
            by: dataset::SplitBy::Time,
            fractions: dataset::SplitFractions {
                calibrate: 0.25,
                holdout: 0.25,
            },
            seed: 7,
        };
        let (ds, proof) = maturity::build(
            &events,
            &index,
            &labels,
            "operator",
            split,
            Hash::of(&[b"synthetic"]),
            "2028-01-01T00:00:00Z",
        )
        .unwrap();
        maturity::write(&ds, &proof, &events, &index, &labels, &data).unwrap();
        let id = bundle::write_bundle(&data, &out, 30).unwrap();
        // Pure gate tests do not promote this synthetic dataset to operational evidence.
        let loaded = bundle::load_bundle(&out).unwrap();
        let mut validation = crate::report::report(&loaded, &ds, 0);
        assert!(validate_activation_report(&validation, "missing").is_err());
        let good = crate::report::Paired {
            tasks: 30,
            clusters: 3,
            mean: Some(-0.1),
            interval: Some((-0.2, -0.01)),
            inconclusive: false,
        };
        validation.paired = good.clone();
        validation
            .paired_by_family
            .insert("fixture".into(), good.clone());
        assert!(validate_activation_report(&validation, "fixture").is_ok());
        for bad in [
            crate::report::Paired {
                mean: None,
                ..good.clone()
            },
            crate::report::Paired {
                inconclusive: true,
                ..good.clone()
            },
            crate::report::Paired {
                interval: Some((-0.1, 0.1)),
                ..good.clone()
            },
            crate::report::Paired {
                clusters: 1,
                ..good
            },
        ] {
            validation.paired_by_family.insert("fixture".into(), bad);
            assert!(validate_activation_report(&validation, "fixture").is_err());
        }
        let (mut config, _, candidate, prefix) = fixture();
        config.bundle_directory = out;
        config.bundle_id = id;
        config.dataset_directory = Some(data.clone());
        config.training_as_of = Some("2028-01-01T00:00:00Z".into());
        config.lineage = "unseen-query".into();
        let query = || Snapshot {
            tool_ms_ceiling: None,
            state: id,
            source_version: id,
            candidate: &candidate,
            events: &prefix,
            as_of: "2028-02-01T00:00:00Z",
            adapter: "native-common-event@v1",
        };
        // A native synthetic fixture model cannot cross the operational project gate.
        let empty = crate::project_receipts::Evidence {
            events: vec![],
            side_events: vec![],
            blobs: BTreeMap::new(),
            as_of: "2028-02-01T00:00:00Z".into(),
        };
        assert!(evaluate_activation(&config, query(), &empty, id, "unseen-clone", id).is_err());
        let result = evaluate(&config, query()).unwrap();
        assert_eq!(result["status"], "shadow_diagnostic_only");
        assert!(result["calibrated_probability"].is_null());
        assert_eq!(result["label"], serde_json::Value::Null);
        assert_eq!(evaluate(&config, query()).unwrap(), result); // no ambient clock in evaluation
        let mut bad = config.clone();
        bad.lineage = ds.rows[0].lineage.clone();
        assert!(evaluate(&bad, query())
            .unwrap_err()
            .to_string()
            .contains("overlaps"));
        bad = config.clone();
        bad.training_as_of = Some("2028-01-02T00:00:00Z".into());
        assert!(evaluate(&bad, query()).is_err());
        std::fs::write(data.join("maturity.json"), b"{}").unwrap();
        assert!(evaluate(&config, query()).is_err());
    }
}
