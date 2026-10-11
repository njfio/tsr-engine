//! Bundles: the one home for fitted parameters (PRD §B8, §B10; lane 06 item 7).
//!
//! `tsr train fit <dataset-dir> --out models/<name> [--support-floor <n>]` writes `manifest.toml`
//! with §B8's fields beside `head.json`, `isotonic.json`, `features.json`, `split.json` and
//! `support.json`, and prints the bundle id. The id is `tsr_domain::Hash::of` over each content
//! file's digest and the manifest's content fields — **not** the code commit, a timestamp or the
//! evaluation report — so the same data and seed give the same id.
//!
//! Coefficients and breakpoints live only in the bundle; nothing here writes a fitted parameter to
//! the log or the store.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::dataset::{Dataset, Row, Side, SplitManifest};
use crate::features::FeatureSpec;
use crate::head::{fit_head, Head, DEFAULT_ITERS, DEFAULT_L2};
use crate::isotonic::{fit_isotonic, Isotonic};
use crate::support::SupportEntry;
use crate::LearnError;
use tsr_domain::Hash;

/// The content files of a bundle, in canonical order. The id covers these digests.
pub const CONTENT_FILES: &[&str] = &[
    "head.json",
    "isotonic.json",
    "features.json",
    "split.json",
    "support.json",
];

/// The bundle manifest: §B8's fields plus the split digest and the source root.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    /// The data-manifest digest: the data and training roots.
    pub data_manifest: String,
    pub training_root: String,
    /// The feature-schema version.
    pub feature_schema: String,
    /// The code commit from `TSR_CODE_COMMIT`, or `"unknown"`.
    pub code_commit: String,
    pub seed: u64,
    /// The calibrator: the bundle id's first 12 hex and the feature-schema version.
    pub calibrator: String,
    /// The thresholds: the support floor.
    pub support_floor: u32,
    /// The split digest (assignments only).
    pub split_digest: String,
    /// The source root: the store's `head_root` or the events file's digest.
    pub source_root: String,
    /// The bundle state: written once, as `draft`.
    pub state: String,
    /// The bundle id, stored so a tampered content file is detectable. Not covered by the id.
    #[serde(default)]
    pub id: String,
    /// Each content file's digest, stored so verify can name the file that differs.
    #[serde(default)]
    pub content_digests: BTreeMap<String, String>,
}

/// A loaded bundle: the id, its directory, the manifest and every content file.
#[derive(Clone, Debug, PartialEq)]
pub struct Bundle {
    pub id: Hash,
    pub dir: PathBuf,
    pub manifest: Manifest,
    pub head: Head,
    pub isotonic: Isotonic,
    pub spec: FeatureSpec,
    pub split: SplitManifest,
    pub support_index: Vec<SupportEntry>,
    /// The calibrated population: the families with their fit+calibrate resolved counts.
    pub population: BTreeMap<String, u32>,
}

impl Bundle {
    /// Whether a family is in the calibrated population.
    pub fn population_contains(&self, family: &str) -> bool {
        self.population.contains_key(family)
    }
}

/// The error `verify_bundle` returns: it names the file whose digest differs.
#[derive(Debug, thiserror::Error)]
pub enum VerifyError {
    #[error("bundle file {file:?} has digest {found}, expected {expected}")]
    DigestMismatch {
        file: String,
        found: String,
        expected: String,
    },
    #[error("bundle id {found} does not match the recomputed id {expected}")]
    IdMismatch { found: String, expected: String },
    #[error("bundle: {0}")]
    Learn(#[from] LearnError),
}

impl From<std::io::Error> for VerifyError {
    fn from(e: std::io::Error) -> Self {
        VerifyError::Learn(LearnError::Io(e))
    }
}

/// The digest of a file's bytes.
fn file_digest(bytes: &[u8]) -> Hash {
    Hash::of(&[bytes])
}

/// The bundle id: `Hash::of` over each content file's digest and the manifest's content fields.
///
/// The code commit, a timestamp and the evaluation report are deliberately excluded, so the same
/// data and seed give the same id.
fn bundle_id(content_digests: &[(String, Hash)], manifest: &Manifest) -> Hash {
    let mut parts: Vec<Vec<u8>> = Vec::new();
    for (name, digest) in content_digests {
        parts.push(name.as_bytes().to_vec());
        parts.push(digest.to_hex().into_bytes());
    }
    // The manifest's content fields, in a fixed order.
    parts.push(manifest.data_manifest.clone().into_bytes());
    parts.push(manifest.training_root.clone().into_bytes());
    parts.push(manifest.feature_schema.clone().into_bytes());
    parts.push(manifest.seed.to_string().into_bytes());
    parts.push(manifest.support_floor.to_string().into_bytes());
    parts.push(manifest.split_digest.clone().into_bytes());
    parts.push(manifest.source_root.clone().into_bytes());
    let refs: Vec<&[u8]> = parts.iter().map(|p| p.as_slice()).collect();
    Hash::of(&refs)
}

/// Read the events file (`events.jsonl`, the `tsr task export` line format) into `(seq, Event)`.
///
/// A missing file reads as no events (the dataset directory may omit it).
pub fn read_events(dir: &Path) -> Result<Vec<(i64, tsr_domain::Event)>, LearnError> {
    let path = dir.join("events.jsonl");
    if !path.exists() {
        return Ok(Vec::new());
    }
    let text = std::fs::read_to_string(path)?;
    let mut out = Vec::new();
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let v: serde_json::Value = serde_json::from_str(line)?;
        let seq = v["seq"]
            .as_i64()
            .ok_or_else(|| LearnError::Invalid("events.jsonl line has no seq".into()))?;
        let event: tsr_domain::Event = serde_json::from_value(v["event"].clone())?;
        out.push((seq, event));
    }
    Ok(out)
}

/// Load a dataset directory written by `write_dataset`.
///
/// Reads `rows.jsonl`, `split.json`, `features.json` and `data-manifest.json`.
pub fn load_dataset(dir: &Path) -> Result<Dataset, LearnError> {
    let rows_text = std::fs::read_to_string(dir.join("rows.jsonl"))?;
    let mut rows: Vec<Row> = Vec::new();
    for line in rows_text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        rows.push(serde_json::from_str(line)?);
    }
    let split: SplitManifest =
        serde_json::from_str(&std::fs::read_to_string(dir.join("split.json"))?)?;
    let spec: FeatureSpec =
        serde_json::from_str(&std::fs::read_to_string(dir.join("features.json"))?)?;
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("data-manifest.json"))?)?;
    let data_root = Hash::from_hex(
        manifest["data_root"]
            .as_str()
            .ok_or_else(|| LearnError::Invalid("data-manifest.json has no data_root".into()))?,
    )
    .map_err(|e| LearnError::Invalid(format!("data_root: {e}")))?;
    let training_root =
        Hash::from_hex(manifest["training_root"].as_str().ok_or_else(|| {
            LearnError::Invalid("data-manifest.json has no training_root".into())
        })?)
        .map_err(|e| LearnError::Invalid(format!("training_root: {e}")))?;
    let source_root = Hash::from_hex(
        manifest["source_root"]
            .as_str()
            .ok_or_else(|| LearnError::Invalid("data-manifest.json has no source_root".into()))?,
    )
    .map_err(|e| LearnError::Invalid(format!("source_root: {e}")))?;
    Ok(Dataset {
        rows,
        split,
        spec,
        data_root,
        training_root,
        source_root,
    })
}

/// The support index: each fit- and calibrate-slice row's family and feature vector, no labels.
fn support_index(ds: &Dataset) -> Vec<SupportEntry> {
    ds.rows
        .iter()
        .filter(|r| r.side == Side::Fit || r.side == Side::Calibrate)
        .map(|r| SupportEntry {
            family: r.family.clone(),
            features: r.features.clone(),
        })
        .collect()
}

/// The families present in the fit and calibrate slices, in name order.
/// The minimum number of mature episodes (tasks with a resolved label) fit and calibrate must hold
/// together, per §B8 ("150 mature episodes per event").
pub const MIN_MATURE_TASKS: usize = 150;

/// The minimum number of mature episodes a family needs to enter the calibrated population, per
/// §B8 ("≥ 30 per family").
pub const MIN_FAMILY_TASKS: usize = 30;

/// The calibrated population: each family's count of *distinct tasks* with a resolved label on the
/// fit and calibrate slices, keeping only families that reach `MIN_FAMILY_TASKS`. A family below
/// the floor stays outside the population, so its rows are reported `uncalibrated`.
fn population_families(ds: &Dataset) -> BTreeMap<String, u32> {
    let mut tasks: BTreeMap<String, std::collections::BTreeSet<tsr_domain::TaskId>> =
        BTreeMap::new();
    for r in &ds.rows {
        if (r.side == Side::Fit || r.side == Side::Calibrate) && r.label.is_some() {
            tasks.entry(r.family.clone()).or_default().insert(r.task);
        }
    }
    tasks
        .into_iter()
        .filter(|(_, t)| t.len() >= MIN_FAMILY_TASKS)
        .map(|(f, t)| (f, t.len() as u32))
        .collect()
}

/// The number of distinct tasks with a resolved label on the fit and calibrate slices.
fn mature_tasks(ds: &Dataset) -> usize {
    ds.rows
        .iter()
        .filter(|r| (r.side == Side::Fit || r.side == Side::Calibrate) && r.label.is_some())
        .map(|r| r.task)
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

/// The code commit from `TSR_CODE_COMMIT`, or `"unknown"`.
fn code_commit() -> String {
    std::env::var("TSR_CODE_COMMIT").unwrap_or_else(|_| "unknown".to_string())
}

/// Write a bundle: fit the head and calibrator, re-run every probe, and write the content files.
///
/// Returns the bundle id. The bundle is written once, as `draft`. `fit` re-runs every probe and
/// refuses when one fails.
pub fn write_bundle(
    dataset_dir: &Path,
    out_dir: &Path,
    support_floor: u32,
) -> Result<Hash, LearnError> {
    let ds = load_dataset(dataset_dir)?;

    // §B8's minimum data: refuse with the counts when fit and calibrate hold fewer than 150 tasks
    // with resolved labels.
    let mature = mature_tasks(&ds);
    if mature < MIN_MATURE_TASKS {
        return Err(LearnError::Invalid(format!(
            "too few mature episodes: {mature} tasks with a resolved label on fit+calibrate, \
             need {MIN_MATURE_TASKS}"
        )));
    }

    // Re-run every probe over the dataset; refuse when one fails.
    let index = {
        let p = dataset_dir.join("tasks.jsonl");
        if p.exists() {
            crate::dataset::read_task_index(&p)?
        } else {
            Vec::new()
        }
    };
    let events = read_events(dataset_dir)?;
    let families = crate::features::families_from_spec(&ds.spec);
    let extractor = crate::probes::default_extractor(&ds.spec, &families);
    let probe_report = crate::probes::run_all(&ds, &events, &index, &ds.spec, &extractor);
    if !probe_report.is_clean() {
        let caught = probe_report
            .leaks
            .iter()
            .map(|l| l.probe.name())
            .collect::<Vec<_>>()
            .join(", ");
        return Err(LearnError::Invalid(format!(
            "probe caught a leak: {caught}"
        )));
    }

    // Fit the head on the fit slice, then isotonic over the head's outputs on the calibrate slice.
    let fit_rows: Vec<Row> = ds
        .rows
        .iter()
        .filter(|r| r.side == Side::Fit)
        .cloned()
        .collect();
    let head = fit_head(&fit_rows, &ds.spec, DEFAULT_L2, DEFAULT_ITERS);
    let cal_rows: Vec<Row> = ds
        .rows
        .iter()
        .filter(|r| r.side == Side::Calibrate && r.label.is_some())
        .cloned()
        .collect();
    let xs: Vec<f64> = cal_rows
        .iter()
        .map(|r| crate::head::predict(&head, &r.features))
        .collect();
    let ys: Vec<f64> = cal_rows
        .iter()
        .map(|r| if r.label == Some(true) { 1.0 } else { 0.0 })
        .collect();
    let isotonic = fit_isotonic(&xs, &ys);

    let support_index = support_index(&ds);
    let population = population_families(&ds);

    std::fs::create_dir_all(out_dir)?;

    // Content files, in canonical order.
    let head_bytes = serde_json::to_vec_pretty(&head)?;
    let iso_bytes = serde_json::to_vec_pretty(&isotonic)?;
    let feat_bytes = serde_json::to_vec_pretty(&ds.spec)?;
    let split_bytes = serde_json::to_vec_pretty(&ds.split)?;
    let support_bytes = serde_json::to_vec_pretty(&SupportFile {
        floor: support_floor,
        population,
        index: support_index,
    })?;

    let contents: [(&str, &[u8]); 5] = [
        ("head.json", &head_bytes),
        ("isotonic.json", &iso_bytes),
        ("features.json", &feat_bytes),
        ("split.json", &split_bytes),
        ("support.json", &support_bytes),
    ];
    let mut content_digests: Vec<(String, Hash)> = Vec::new();
    for (name, bytes) in contents.iter() {
        std::fs::write(out_dir.join(name), bytes)?;
        content_digests.push((name.to_string(), file_digest(bytes)));
    }

    // The manifest. The calibrator is the bundle id's first 12 hex and the feature-schema version,
    // so it is filled after the id is known — but the id does not cover the calibrator field.
    let mut manifest = Manifest {
        data_manifest: ds.data_root.to_hex(),
        training_root: ds.training_root.to_hex(),
        feature_schema: ds.spec.version.clone(),
        code_commit: code_commit(),
        seed: ds.split.seed,
        calibrator: String::new(),
        support_floor,
        split_digest: ds.split.digest.to_hex(),
        source_root: ds.source_root.to_hex(),
        state: "draft".to_string(),
        id: String::new(),
        content_digests: BTreeMap::new(),
    };
    let id = bundle_id(&content_digests, &manifest);
    manifest.calibrator = format!("{} {}", &id.to_hex()[..12], ds.spec.version);
    manifest.id = id.to_hex();
    manifest.content_digests = content_digests
        .iter()
        .map(|(n, h)| (n.clone(), h.to_hex()))
        .collect();

    let manifest_toml = toml::to_string_pretty(&manifest)
        .map_err(|e| LearnError::Invalid(format!("manifest: {e}")))?;
    std::fs::write(out_dir.join("manifest.toml"), manifest_toml)?;

    Ok(id)
}

/// The `support.json` content: the floor, the calibrated population and the support index.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SupportFile {
    pub floor: u32,
    /// The families in the calibrated population with their fit+calibrate resolved counts.
    pub population: BTreeMap<String, u32>,
    /// Each training row's family and feature vector, no labels.
    pub index: Vec<SupportEntry>,
}

/// Load a bundle: recompute every digest and refuse a mismatch.
pub fn load_bundle(dir: &Path) -> Result<Bundle, LearnError> {
    let manifest_text = std::fs::read_to_string(dir.join("manifest.toml"))?;
    let manifest: Manifest = toml::from_str(&manifest_text)
        .map_err(|e| LearnError::Invalid(format!("manifest.toml: {e}")))?;

    // Fail closed: a manifest that omits its id or any content digest is refused, never treated
    // as "no mismatch". `id` and `content_digests` are `#[serde(default)]`, so a stripped manifest
    // parses; without this check `load_bundle` would perform no digest and no id check at all.
    // Checked before any content file is read, so the refusal does not depend on their state.
    if manifest.id.is_empty() {
        return Err(LearnError::Invalid(
            "manifest.toml: missing bundle id".to_string(),
        ));
    }
    for name in CONTENT_FILES {
        if !manifest.content_digests.contains_key(*name) {
            return Err(LearnError::Invalid(format!(
                "manifest.toml: missing content digest for {name:?}"
            )));
        }
    }

    let head: Head = serde_json::from_str(&std::fs::read_to_string(dir.join("head.json"))?)?;
    let isotonic: Isotonic =
        serde_json::from_str(&std::fs::read_to_string(dir.join("isotonic.json"))?)?;
    let spec: FeatureSpec =
        serde_json::from_str(&std::fs::read_to_string(dir.join("features.json"))?)?;
    let split: SplitManifest =
        serde_json::from_str(&std::fs::read_to_string(dir.join("split.json"))?)?;
    let support_file: SupportFile =
        serde_json::from_str(&std::fs::read_to_string(dir.join("support.json"))?)?;

    // Recompute the id from the content files and the manifest's content fields, and refuse a
    // mismatch against the stored id.
    let mut content_digests: Vec<(String, Hash)> = Vec::new();
    for name in CONTENT_FILES {
        let bytes = std::fs::read(dir.join(name))?;
        let found = file_digest(&bytes);
        let expected = manifest
            .content_digests
            .get(*name)
            .expect("presence checked above");
        if &found.to_hex() != expected {
            return Err(LearnError::Invalid(format!(
                "bundle file {name:?} has digest {}, expected {expected}",
                found.to_hex()
            )));
        }
        content_digests.push((name.to_string(), found));
    }
    let id = bundle_id(&content_digests, &manifest);
    if manifest.id != id.to_hex() {
        return Err(LearnError::Invalid(format!(
            "bundle id {} does not match the recomputed id {}",
            manifest.id,
            id.to_hex()
        )));
    }

    Ok(Bundle {
        id,
        dir: dir.to_path_buf(),
        manifest,
        head,
        isotonic,
        spec,
        split,
        support_index: support_file.index,
        population: support_file.population,
    })
}

/// Verify a bundle: recompute every digest and refuse a mismatch, naming the file.
///
/// With `rebuild`, retrain from the dataset directory and compare ids.
pub fn verify_bundle(dir: &Path, rebuild: Option<&Path>) -> Result<(), VerifyError> {
    let manifest_text = std::fs::read_to_string(dir.join("manifest.toml"))?;
    let manifest: Manifest = toml::from_str(&manifest_text)
        .map_err(|e| LearnError::Invalid(format!("manifest.toml: {e}")))?;

    // Fail closed: a manifest that omits its id or any content digest is refused, never treated
    // as "no mismatch". `id` and `content_digests` are `#[serde(default)]`, so a stripped manifest
    // parses; without this check `verify_bundle` would perform no digest and no id check at all.
    if manifest.id.is_empty() {
        return Err(VerifyError::Learn(LearnError::Invalid(
            "manifest.toml: missing bundle id".to_string(),
        )));
    }
    for name in CONTENT_FILES {
        if !manifest.content_digests.contains_key(*name) {
            return Err(VerifyError::Learn(LearnError::Invalid(format!(
                "manifest.toml: missing content digest for {name:?}"
            ))));
        }
    }

    // Recompute every content file's digest and compare with the stored digest, naming the file.
    let mut content_digests: Vec<(String, Hash)> = Vec::new();
    for name in CONTENT_FILES {
        let bytes = std::fs::read(dir.join(name))?;
        let found = file_digest(&bytes);
        let expected = manifest
            .content_digests
            .get(*name)
            .expect("presence checked above");
        if &found.to_hex() != expected {
            return Err(VerifyError::DigestMismatch {
                file: (*name).to_string(),
                found: found.to_hex(),
                expected: expected.clone(),
            });
        }
        content_digests.push((name.to_string(), found));
    }
    let recomputed = bundle_id(&content_digests, &manifest);

    // The stored id must equal the recomputed id.
    if manifest.id != recomputed.to_hex() {
        return Err(VerifyError::IdMismatch {
            found: manifest.id.clone(),
            expected: recomputed.to_hex(),
        });
    }

    if let Some(dataset_dir) = rebuild {
        let rebuilt = write_bundle(dataset_dir, &dir.join("rebuild"), manifest.support_floor)
            .map_err(VerifyError::Learn)?;
        if rebuilt != recomputed {
            return Err(VerifyError::IdMismatch {
                found: rebuilt.to_hex(),
                expected: recomputed.to_hex(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::{SplitBy, SplitManifest};
    use crate::features::{spec_for_families, FeatureVector};
    use tsr_domain::{ActionKind, EventId, TaskId};

    fn tmpdir(tag: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!("tsr-learn-bundle-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&p);
        std::fs::create_dir_all(&p).unwrap();
        p
    }

    fn row(family: &str, values: Vec<f64>, label: Option<bool>, side: Side, i: u64) -> Row {
        Row {
            id: Hash::of(&[format!("row{i}").as_bytes()]),
            task: TaskId(Hash::of(&[format!("t{i}").as_bytes()])),
            commitment: EventId(Hash::of(&[format!("c{i}").as_bytes()])),
            index: i,
            action: ActionKind::ProposePatch,
            family: family.to_string(),
            lineage: format!("l{}", i % 3),
            side,
            features: FeatureVector { values },
            label,
            evidence: Vec::new(),
            generator_raw: None,
            operator_raw: None,
            controller_logged: None,
        }
    }

    fn dataset() -> Dataset {
        let spec = spec_for_families(&["calibrated".to_string()]);
        let dim = spec.names.len();
        let mut rows = Vec::new();
        // §B8's minimum data: 150 mature tasks, ≥ 30 in the family. 160 distinct tasks, 120 fit and
        // 40 calibrate, all `calibrated`, so `write_bundle` does not refuse.
        for i in 0..160u64 {
            let mut v = vec![0.0; dim];
            v[0] = 1.0;
            v[dim - 1] = (i % 5) as f64;
            let side = if i < 120 { Side::Fit } else { Side::Calibrate };
            rows.push(row("calibrated", v, Some(i % 2 == 0), side, i));
        }
        Dataset {
            rows,
            split: SplitManifest {
                by: SplitBy::Time,
                seed: 7,
                assignments: std::collections::BTreeMap::new(),
                digest: Hash::of(&[b"split"]),
            },
            spec,
            data_root: Hash::of(&[b"data"]),
            training_root: Hash::of(&[b"train"]),
            source_root: Hash::of(&[b"source"]),
        }
    }

    #[test]
    fn write_twice_gives_the_same_id() {
        let ds = dataset();
        let ddir = tmpdir("ds");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let a = write_bundle(&ddir, &tmpdir("a"), 30).unwrap();
        let b = write_bundle(&ddir, &tmpdir("b"), 30).unwrap();
        assert_eq!(a, b, "same data and seed give the same bundle id");
    }

    #[test]
    fn load_recomputes_the_id_and_verify_passes() {
        let ds = dataset();
        let ddir = tmpdir("ds2");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let out = tmpdir("out2");
        let id = write_bundle(&ddir, &out, 30).unwrap();
        let b = load_bundle(&out).unwrap();
        assert_eq!(b.id, id);
        assert_eq!(b.manifest.state, "draft");
        assert_eq!(b.manifest.support_floor, 30);
        assert!(b.manifest.calibrator.starts_with(&id.to_hex()[..12]));
        verify_bundle(&out, None).unwrap();
    }

    #[test]
    fn a_one_byte_change_makes_verify_fail_naming_the_file() {
        let ds = dataset();
        let ddir = tmpdir("ds3");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let out = tmpdir("out3");
        write_bundle(&ddir, &out, 30).unwrap();
        // Tamper with head.json.
        let mut bytes = std::fs::read(out.join("head.json")).unwrap();
        bytes[0] ^= 0x01;
        std::fs::write(out.join("head.json"), &bytes).unwrap();
        let err = verify_bundle(&out, None).unwrap_err();
        match err {
            VerifyError::DigestMismatch { file, .. } => assert_eq!(file, "head.json"),
            other => panic!("expected a digest mismatch naming head.json, got {other:?}"),
        }
    }

    #[test]
    fn verify_rebuild_gives_the_same_id() {
        let ds = dataset();
        let ddir = tmpdir("ds4");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let out = tmpdir("out4");
        write_bundle(&ddir, &out, 30).unwrap();
        verify_bundle(&out, Some(&ddir)).unwrap();
    }

    #[test]
    fn support_floor_is_recorded() {
        let ds = dataset();
        let ddir = tmpdir("ds5");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let out = tmpdir("out5");
        write_bundle(&ddir, &out, 12).unwrap();
        let b = load_bundle(&out).unwrap();
        assert_eq!(b.manifest.support_floor, 12);
    }

    /// Strip the `id` line and the `[content_digests]` table from a manifest, as an attacker
    /// would to make `load_bundle` skip every digest and id check.
    fn strip_digest_fields(dir: &Path) {
        let text = std::fs::read_to_string(dir.join("manifest.toml")).unwrap();
        let mut kept: Vec<&str> = Vec::new();
        let mut in_table = false;
        for line in text.lines() {
            if line.trim_start().starts_with('[') {
                in_table = true;
            }
            if in_table || line.trim_start().starts_with("id =") {
                continue;
            }
            kept.push(line);
        }
        std::fs::write(dir.join("manifest.toml"), kept.join("\n")).unwrap();
    }

    #[test]
    fn a_manifest_without_digest_fields_is_refused() {
        let ds = dataset();
        let ddir = tmpdir("ds6");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let out = tmpdir("out6");
        write_bundle(&ddir, &out, 30).unwrap();
        // Tamper head.json so a fail-open would be observable, then strip the fields that would
        // have caught it.
        let mut bytes = std::fs::read(out.join("head.json")).unwrap();
        bytes[0] ^= 0x01;
        std::fs::write(out.join("head.json"), &bytes).unwrap();
        strip_digest_fields(&out);

        let err = load_bundle(&out).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("missing bundle id") || msg.contains("missing content digest"),
            "load_bundle must fail closed on a stripped manifest, got: {msg}"
        );

        let err = verify_bundle(&out, None).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("missing bundle id") || msg.contains("missing content digest"),
            "verify_bundle must fail closed on a stripped manifest, got: {msg}"
        );
    }

    #[test]
    fn a_manifest_with_only_the_id_stripped_is_refused() {
        let ds = dataset();
        let ddir = tmpdir("ds7");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let out = tmpdir("out7");
        write_bundle(&ddir, &out, 30).unwrap();
        let text = std::fs::read_to_string(out.join("manifest.toml")).unwrap();
        let kept: Vec<&str> = text
            .lines()
            .filter(|l| !l.trim_start().starts_with("id ="))
            .collect();
        std::fs::write(out.join("manifest.toml"), kept.join("\n")).unwrap();
        let msg = load_bundle(&out).unwrap_err().to_string();
        assert!(msg.contains("missing bundle id"), "got: {msg}");
    }

    #[test]
    fn a_manifest_with_one_digest_missing_is_refused() {
        let ds = dataset();
        let ddir = tmpdir("ds8");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let out = tmpdir("out8");
        write_bundle(&ddir, &out, 30).unwrap();
        let text = std::fs::read_to_string(out.join("manifest.toml")).unwrap();
        let kept: Vec<&str> = text
            .lines()
            .filter(|l| !l.trim_start().starts_with("\"head.json\""))
            .collect();
        std::fs::write(out.join("manifest.toml"), kept.join("\n")).unwrap();
        let msg = load_bundle(&out).unwrap_err().to_string();
        assert!(
            msg.contains("missing content digest") && msg.contains("head.json"),
            "got: {msg}"
        );
    }
}
