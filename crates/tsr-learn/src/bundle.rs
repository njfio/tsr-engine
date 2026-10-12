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

/// The result of applying a bundle to a row (B7, B10; lane 06 item 8).
///
/// `raw_p` is the head's output, always. `calibrated` is `Some` only for a row whose family is in
/// the bundle's calibrated population — the calibrator is the bundle id and the population is the
/// family's fit+calibrate resolved count — and `None` (`uncalibrated`) otherwise, never a
/// fabricated number. `support` is the same-family fit+calibrate episode count within feature range.
#[derive(Clone, Debug, PartialEq)]
pub struct Applied {
    /// The controller's raw probability: the head's output, always.
    pub raw_p: f64,
    /// The calibrated probability, only for a family in the population.
    pub calibrated: Option<tsr_domain::Calibrated>,
    /// The same-family episodes within feature range.
    pub support: tsr_domain::Support,
}

impl Bundle {
    /// Whether a family is in the calibrated population.
    pub fn population_contains(&self, family: &str) -> bool {
        self.population.contains_key(family)
    }

    /// Apply the bundle to a row (B7, B10): the raw `p`, a `Calibrated` only for a family in the
    /// population, and the support.
    ///
    /// The raw `p` is the head's output, always. `Calibrated { p, calibrator, population }` is
    /// constructed only when the row's family is in the bundle's population: `calibrator` is the
    /// bundle id's first 12 hex and the feature-schema version (the manifest's `calibrator`), and
    /// `population` names the family with its fit+calibrate resolved count. Every other row is
    /// `uncalibrated` — `calibrated` is `None`, never a fabricated number. A row whose feature
    /// length differs from the head's is refused rather than zipped.
    pub fn apply(&self, row: &Row) -> Result<Applied, LearnError> {
        let raw = crate::head::predict_checked(&self.head, &self.spec, &row.features)?;
        let support =
            crate::support::support(&self.spec, &self.support_index, &row.family, &row.features);
        let calibrated = match self.population.get(&row.family) {
            Some(_) => {
                let p = crate::isotonic::apply_isotonic(&self.isotonic, raw);
                let probability = tsr_domain::Probability::new(p).map_err(|e| {
                    LearnError::Invalid(format!("calibrated p {p} is not a probability: {e}"))
                })?;
                Some(tsr_domain::Calibrated {
                    p: probability,
                    calibrator: self.manifest.calibrator.clone(),
                    // B14: the population names every family in it, not only the row's.
                    population: self
                        .population
                        .iter()
                        .map(|(f, n)| format!("{f}={n}"))
                        .collect::<Vec<_>>()
                        .join(","),
                })
            }
            None => None,
        };
        Ok(Applied {
            raw_p: raw,
            calibrated,
            support,
        })
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

    // B2: recompute every row id, both roots and the split digest from `rows.jsonl` and
    // `split.json`, and refuse when `data-manifest.json` disagrees. The manifest's roots are
    // otherwise copied verbatim, so flipping a label in `rows.jsonl` would fit happily with the
    // old training root.
    for r in &rows {
        let recomputed = crate::dataset::row_id_of(r);
        if recomputed != r.id {
            // F14: name the row by the id that appears in rows.jsonl, not by a hash of its
            // commitment (which appears in no file).
            return Err(LearnError::Invalid(format!(
                "rows.jsonl: row {} has id {}, recomputed {}",
                r.id.to_hex(),
                r.id.to_hex(),
                recomputed.to_hex()
            )));
        }
    }
    let data_root = crate::dataset::data_root(&rows);
    let training_root = crate::dataset::training_root(&rows);
    let split_digest = crate::dataset::split_digest(split.by, split.seed, &split.assignments);

    let stored_data_root = Hash::from_hex(
        manifest["data_root"]
            .as_str()
            .ok_or_else(|| LearnError::Invalid("data-manifest.json has no data_root".into()))?,
    )
    .map_err(|e| LearnError::Invalid(format!("data_root: {e}")))?;
    if stored_data_root != data_root {
        return Err(LearnError::Invalid(format!(
            "data-manifest.json: data_root {} disagrees with the recomputed {}",
            stored_data_root.to_hex(),
            data_root.to_hex()
        )));
    }
    let stored_training_root =
        Hash::from_hex(manifest["training_root"].as_str().ok_or_else(|| {
            LearnError::Invalid("data-manifest.json has no training_root".into())
        })?)
        .map_err(|e| LearnError::Invalid(format!("training_root: {e}")))?;
    if stored_training_root != training_root {
        return Err(LearnError::Invalid(format!(
            "data-manifest.json: training_root {} disagrees with the recomputed {}",
            stored_training_root.to_hex(),
            training_root.to_hex()
        )));
    }
    let stored_split_digest = Hash::from_hex(
        manifest["split_digest"]
            .as_str()
            .ok_or_else(|| LearnError::Invalid("data-manifest.json has no split_digest".into()))?,
    )
    .map_err(|e| LearnError::Invalid(format!("split_digest: {e}")))?;
    if stored_split_digest != split_digest {
        return Err(LearnError::Invalid(format!(
            "data-manifest.json: split_digest {} disagrees with the recomputed {}",
            stored_split_digest.to_hex(),
            split_digest.to_hex()
        )));
    }

    // B15: the standardization statistics live in `features.json` (the spec). Recompute them from
    // the fit slice's labelled rows and refuse a spec whose declared statistics disagree, so a
    // tampered `features.json` cannot silently change how every row is standardized.
    let mut spec = spec;
    let fit_labelled: Vec<Row> = rows
        .iter()
        .filter(|r| r.side == Side::Fit && r.label.is_some())
        .cloned()
        .collect();
    let (mean, std) = crate::head::standardize_stats(&fit_labelled, spec.names.len());
    if !spec.standardize_mean.is_empty() && spec.standardize_mean != mean {
        return Err(LearnError::Invalid(
            "features.json: standardize_mean disagrees with the fit slice's recomputed mean".into(),
        ));
    }
    if !spec.standardize_std.is_empty() && spec.standardize_std != std {
        return Err(LearnError::Invalid(
            "features.json: standardize_std disagrees with the fit slice's recomputed std".into(),
        ));
    }
    spec.standardize_mean = mean;
    spec.standardize_std = std;
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
        // A8: the count is written by `dataset` into `data-manifest.json`; `load_dataset` reads the
        // rows, not the index, so it carries the stored count when present.
        unindexed_tasks: manifest["unindexed_tasks"].as_u64().unwrap_or(0) as usize,
    })
}

/// B15 test hook: the support index a dataset would produce, so a test can assert it holds only
/// labelled fit/calibrate rows.
#[cfg(test)]
pub fn support_index_for_test(ds: &Dataset) -> Vec<SupportEntry> {
    support_index(ds)
}

/// The support index: each fit- and calibrate-slice row's family and feature vector, no labels.
///
/// B15: only rows with a **resolved label** count — an unlabelled row is not a resolved episode, so
/// it must not inflate support. (The index carries no label; the filter is applied here.)
fn support_index(ds: &Dataset) -> Vec<SupportEntry> {
    ds.rows
        .iter()
        .filter(|r| (r.side == Side::Fit || r.side == Side::Calibrate) && r.label.is_some())
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

    // B9: a bundle is written once. Refuse a non-empty `--out` so a second `fit` cannot overwrite
    // a bundle that has already been verified.
    if out_dir.exists() {
        let non_empty = std::fs::read_dir(out_dir)?.next().is_some();
        if non_empty {
            return Err(LearnError::Invalid(format!(
                "output directory {} exists and is not empty; a bundle is written once",
                out_dir.display()
            )));
        }
    }

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
    // B3: the probe pass reads every row, held_out included; log each held_out read.
    for r in &ds.rows {
        if r.side == crate::dataset::Side::HeldOut {
            crate::probes::log_holdout_access(dataset_dir, r, "fit")?;
        }
    }
    let families = crate::features::families_from_spec(&ds.spec);
    let extractor = crate::probes::default_extractor(&ds.spec, &families);
    let split_json = dataset_dir.join("split.json");
    let probe_report = crate::probes::run_all(
        &ds,
        &events,
        &index,
        &ds.spec,
        &extractor,
        Some(&split_json),
    );
    if !probe_report.is_clean() {
        // A7: name the row (and, for probe (b), the feature) — not just the probe. The `Leak`
        // Display carries the probe, the row id (or `feature:<name>`) and the detail.
        let caught = probe_report
            .leaks
            .iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>()
            .join("; ");
        return Err(LearnError::Invalid(format!(
            "probe caught a leak: {caught}"
        )));
    }

    // F3/R2: an empty fit or calibrate slice must be refused, naming the slice. An empty fit slice
    // fits a zero-weight head (raw p 0.5 for every row); an empty calibrate slice fits an empty
    // isotonic, which acts as the identity, so `apply` labels raw output `Calibrated`. The single
    // mature-count minimum above does not catch either.
    let fit_labelled = ds
        .rows
        .iter()
        .filter(|r| r.side == Side::Fit && r.label.is_some())
        .count();
    if fit_labelled == 0 {
        return Err(LearnError::Invalid(
            "the fit slice holds no row with a resolved label; refusing to fit an empty fit slice"
                .into(),
        ));
    }
    let cal_labelled = ds
        .rows
        .iter()
        .filter(|r| r.side == Side::Calibrate && r.label.is_some())
        .count();
    if cal_labelled == 0 {
        return Err(LearnError::Invalid(
            "the calibrate slice holds no row with a resolved label; refusing to fit an empty \
             calibrate slice"
                .into(),
        ));
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
        .map(|r| crate::head::predict(&head, &ds.spec, &r.features))
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

    // F6/B8: name the file that is missing, rather than an io error with no file name.
    let read = |name: &str| -> Result<String, LearnError> {
        std::fs::read_to_string(dir.join(name)).map_err(|e| {
            LearnError::Invalid(format!(
                "bundle file {name:?} is missing or unreadable: {e}"
            ))
        })
    };

    // Recompute every content file's digest and compare with the stored digest BEFORE parsing the
    // files, so a one-byte change is refused naming the file rather than surfacing as a JSON parse
    // error. `verify_bundle` runs this same function, so it names the file too.
    let mut content_digests: Vec<(String, Hash)> = Vec::new();
    for name in CONTENT_FILES {
        let bytes = std::fs::read(dir.join(name)).map_err(|e| {
            LearnError::Invalid(format!(
                "bundle file {name:?} is missing or unreadable: {e}"
            ))
        })?;
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

    let head: Head = serde_json::from_str(&read("head.json")?)?;
    let isotonic: Isotonic = serde_json::from_str(&read("isotonic.json")?)?;
    let spec: FeatureSpec = serde_json::from_str(&read("features.json")?)?;
    let split: SplitManifest = serde_json::from_str(&read("split.json")?)?;
    let support_file: SupportFile = serde_json::from_str(&read("support.json")?)?;

    let id = bundle_id(&content_digests, &manifest);
    if manifest.id != id.to_hex() {
        return Err(LearnError::Invalid(format!(
            "bundle id {} does not match the recomputed id {}",
            manifest.id,
            id.to_hex()
        )));
    }

    // F6/B8: the calibrator is derived (the recomputed id's first 12 hex and the feature-schema
    // version) and the state is `draft`. `load_bundle` is used by verify, report and apply alike,
    // so a forged calibrator must be refused here too, not only in `verify_bundle`.
    let expected_calibrator = format!("{} {}", &id.to_hex()[..12], manifest.feature_schema);
    if manifest.calibrator != expected_calibrator {
        return Err(LearnError::Invalid(format!(
            "manifest.toml: calibrator {:?} disagrees with the derived {:?}",
            manifest.calibrator, expected_calibrator
        )));
    }
    if manifest.state != "draft" {
        return Err(LearnError::Invalid(format!(
            "manifest.toml: state {:?} is not \"draft\"",
            manifest.state
        )));
    }

    // F6/B8: the manifest must be the canonical rendering of its own fields. Any byte edit —
    // whitespace, a comment, a reordered key — is refused, naming manifest.toml, so a manifest
    // that merely parses is never trusted.
    let canonical = toml::to_string_pretty(&manifest)
        .map_err(|e| LearnError::Invalid(format!("manifest.toml: {e}")))?;
    if canonical != manifest_text {
        return Err(LearnError::Invalid(
            "manifest.toml: not the canonical rendering of its fields".to_string(),
        ));
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

/// Verify a bundle: run the one loader every command runs, then optionally rebuild.
///
/// F6/B8: `verify`, `report` and `apply` all run `load_bundle`, so every manifest or content
/// tamper `verify` refuses, `report` and `apply` refuse too — one function, one set of checks.
/// With `rebuild`, retrain from the dataset directory and compare ids.
pub fn verify_bundle(dir: &Path, rebuild: Option<&Path>) -> Result<(), VerifyError> {
    // P1: `code_commit` is recorded provenance, not a verified property — no signature backs it,
    // so anyone who can edit the manifest can edit it consistently. It is never compared with the
    // environment here; `verify` and `report` print it labelled `code_commit (recorded, not
    // verified)`. `load_bundle` still refuses a manifest whose canonical rendering changed.
    let bundle = load_bundle(dir).map_err(VerifyError::Learn)?;

    if let Some(dataset_dir) = rebuild {
        // B9: build in a temporary directory, compare ids, and remove it, so the bundle directory
        // is unchanged and no `rebuild` subdirectory is left behind.
        let tmp = std::env::temp_dir().join(format!(
            "tsr-verify-rebuild-{}-{}",
            std::process::id(),
            bundle.id.to_hex()
        ));
        let _ = std::fs::remove_dir_all(&tmp);
        let rebuilt = write_bundle(dataset_dir, &tmp, bundle.manifest.support_floor);
        let _ = std::fs::remove_dir_all(&tmp);
        let rebuilt = rebuilt.map_err(VerifyError::Learn)?;
        if rebuilt != bundle.id {
            return Err(VerifyError::IdMismatch {
                found: rebuilt.to_hex(),
                expected: bundle.id.to_hex(),
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
        let commitment = EventId(Hash::of(&[format!("c{i}").as_bytes()]));
        let features = FeatureVector { values };
        let evidence: Vec<EventId> = Vec::new();
        let mut r = Row {
            id: Hash::of(&[b"placeholder"]),
            task: TaskId(Hash::of(&[format!("t{i}").as_bytes()])),
            commitment,
            index: i,
            action: ActionKind::ProposePatch,
            family: family.to_string(),
            lineage: format!("l{}", i % 3),
            side,
            features,
            label,
            evidence,
            generator_raw: None,
            operator_raw: None,
            controller_logged: None,
        };
        // B2/F7: the id is the real `row_id`, so `load_dataset`'s recompute agrees on a clean row.
        r.id = crate::dataset::row_id_of(&r);
        r
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
            let mut r = row("calibrated", v, Some(i % 2 == 0), side, i);
            // Keep each lineage on one side, so the boundary probe does not fire on this clean
            // fixture (a lineage on two sides is a real leak).
            r.lineage = if side == Side::Fit { "l0" } else { "l1" }.to_string();
            // F7: the id covers the lineage, so recompute it after the edit.
            r.id = crate::dataset::row_id_of(&r);
            rows.push(r);
        }
        // Every task's side must appear in split.json, or the split-manifest probe fires.
        let mut assignments = std::collections::BTreeMap::new();
        for r in &rows {
            assignments.insert(r.task.to_string(), r.side);
        }
        let data_root = crate::dataset::data_root(&rows);
        let training_root = crate::dataset::training_root(&rows);
        let digest = crate::dataset::split_digest(SplitBy::Time, 7, &assignments);
        Dataset {
            rows,
            split: SplitManifest {
                by: SplitBy::Time,
                seed: 7,
                assignments,
                digest,
            },
            spec,
            data_root,
            training_root,
            source_root: Hash::of(&[b"source"]),
            unindexed_tasks: 0,
        }
    }

    /// B7, B10: `apply` returns the raw p always, a `Calibrated` only for a family in the population,
    /// and the support. A family outside the population is `uncalibrated` — `calibrated` is `None`,
    /// never a fabricated number.
    #[test]
    fn b7_apply_calibrates_only_a_family_in_the_population() {
        let ds = dataset();
        let ddir = tmpdir("ds-apply");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let out = tmpdir("out-apply");
        let id = write_bundle(&ddir, &out, 30).unwrap();
        let b = load_bundle(&out).unwrap();
        assert!(
            b.population_contains("calibrated"),
            "the fixture's family is in the population"
        );

        // A row of the calibrated family: raw p, a Calibrated with the bundle's calibrator, support.
        let mut v = vec![0.0; b.spec.names.len()];
        v[0] = 1.0;
        v[b.spec.names.len() - 1] = 2.0;
        let in_pop = row("calibrated", v.clone(), Some(true), Side::HeldOut, 900);
        let applied = b.apply(&in_pop).unwrap();
        let raw = crate::head::predict_checked(&b.head, &b.spec, &in_pop.features).unwrap();
        assert_eq!(applied.raw_p, raw, "raw p is the head's output, always");
        let cal = applied
            .calibrated
            .as_ref()
            .expect("a family in the population is calibrated");
        assert_eq!(cal.calibrator, b.manifest.calibrator);
        assert!(
            cal.population.starts_with("calibrated="),
            "the population names the family and its count, got {}",
            cal.population
        );
        assert!(
            applied.support.episodes > 0,
            "the fixture has same-family rows"
        );

        // A row of a family outside the population: uncalibrated, never a fabricated number.
        let out_pop = row("other", v, Some(true), Side::HeldOut, 901);
        let applied = b.apply(&out_pop).unwrap();
        assert!(
            applied.calibrated.is_none(),
            "a family outside the population is uncalibrated"
        );
        assert_eq!(
            applied.raw_p,
            crate::head::predict_checked(&b.head, &b.spec, &out_pop.features).unwrap()
        );
        assert_eq!(applied.support.episodes, 0, "no same-family rows");
        let _ = id;
    }

    /// B14: the `Calibrated.population` names every family in the bundle's population, not only
    /// the row's own family.
    #[test]
    fn b14_population_names_every_family_in_it() {
        let ds = dataset();
        let ddir = tmpdir("ds-pop");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let out = tmpdir("out-pop");
        write_bundle(&ddir, &out, 30).unwrap();
        let mut b = load_bundle(&out).unwrap();
        // Add a second family to the population; the row's own family is still `calibrated`.
        b.population.insert("other".to_string(), 42);
        let mut v = vec![0.0; b.spec.names.len()];
        v[0] = 1.0;
        let r = row("calibrated", v, Some(true), Side::HeldOut, 903);
        let applied = b.apply(&r).unwrap();
        let cal = applied.calibrated.as_ref().expect("calibrated family");
        assert!(
            cal.population.contains("calibrated=") && cal.population.contains("other=42"),
            "B14: the population names every family in it, got {}",
            cal.population
        );
    }

    /// B7: a row whose feature length differs from the head's is refused, not zipped.
    #[test]
    fn b7_apply_refuses_a_feature_length_mismatch() {
        let ds = dataset();
        let ddir = tmpdir("ds-apply-len");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let out = tmpdir("out-apply-len");
        write_bundle(&ddir, &out, 30).unwrap();
        let b = load_bundle(&out).unwrap();
        let short = row("calibrated", vec![1.0], Some(true), Side::HeldOut, 902);
        assert!(
            b.apply(&short).is_err(),
            "a feature-length mismatch is an error, not a zip"
        );
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
        // F6/B8: `verify_bundle` runs `load_bundle`, so a one-byte change is refused naming the
        // file (as a `LearnError`), not surfaced as a JSON parse error.
        let err = verify_bundle(&out, None).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("head.json"),
            "expected a digest mismatch naming head.json, got {msg}"
        );
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

    /// A7: a probe caught in `fit` names the row, and for probe (b) the feature — not just the
    /// probe name. Before the fix, `write_bundle` joined only `l.probe.name()`.
    #[test]
    fn a7_a_caught_probe_in_fit_names_the_row_and_feature() {
        let spec = spec_for_families(&["calibrated".to_string()]);
        let dim = spec.names.len();
        let mut rows = Vec::new();
        for i in 0..160u64 {
            let label = i % 2 == 0;
            let mut v = vec![0.0; dim];
            // The label copied into feature 0: a perfect threshold separator.
            v[0] = if label { 1.0 } else { 0.0 };
            let side = if i < 120 { Side::Fit } else { Side::Calibrate };
            let mut r = row("calibrated", v, Some(label), side, i);
            r.lineage = if side == Side::Fit { "l0" } else { "l1" }.to_string();
            r.id = crate::dataset::row_id_of(&r);
            rows.push(r);
        }
        let mut assignments = std::collections::BTreeMap::new();
        for r in &rows {
            assignments.insert(r.task.to_string(), r.side);
        }
        let data_root = crate::dataset::data_root(&rows);
        let training_root = crate::dataset::training_root(&rows);
        let digest = crate::dataset::split_digest(SplitBy::Time, 7, &assignments);
        let ds = Dataset {
            rows,
            split: SplitManifest {
                by: SplitBy::Time,
                seed: 7,
                assignments,
                digest,
            },
            spec,
            data_root,
            training_root,
            source_root: Hash::of(&[b"a7"]),
            unindexed_tasks: 0,
        };
        let ddir = tmpdir("ds-a7");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let out = tmpdir("out-a7");
        let err = write_bundle(&ddir, &out, 30).unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("label_reach"),
            "A7: the error names the probe: {msg}"
        );
        assert!(
            msg.contains("feature:"),
            "A7: the error names the feature: {msg}"
        );
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

    /// B2: flipping every fit row's label in `rows.jsonl` changes the row ids, so the data root and
    /// the training root no longer match `data-manifest.json`. `load_dataset` must recompute them
    /// from the rows and refuse, naming the file that disagrees — today it copies the manifest's
    /// roots and fits happily with the old training root.
    #[test]
    fn b2_a_flipped_label_is_refused_naming_rows_jsonl() {
        let ds = dataset();
        let ddir = tmpdir("b2-flip");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let text = std::fs::read_to_string(ddir.join("rows.jsonl")).unwrap();
        let mut out = String::new();
        for line in text.lines() {
            let mut v: serde_json::Value = serde_json::from_str(line).unwrap();
            if v["side"] == "fit" {
                let l = v["label"].as_bool().unwrap();
                v["label"] = serde_json::Value::Bool(!l);
            }
            out.push_str(&serde_json::to_string(&v).unwrap());
            out.push('\n');
        }
        std::fs::write(ddir.join("rows.jsonl"), out).unwrap();
        let err = match load_dataset(&ddir) {
            Ok(_) => panic!("B2: a flipped label must be refused naming rows.jsonl"),
            Err(e) => e.to_string(),
        };
        assert!(
            err.contains("rows.jsonl"),
            "B2: a flipped label must be refused naming rows.jsonl, got: {err}"
        );
    }

    /// B2: a forged `training_root` in `data-manifest.json` must be refused, naming the file.
    #[test]
    fn b2_a_forged_training_root_is_refused_naming_the_manifest() {
        let ds = dataset();
        let ddir = tmpdir("b2-root");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let path = ddir.join("data-manifest.json");
        let mut v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        v["training_root"] = serde_json::Value::String(Hash::of(&[b"forged"]).to_hex());
        std::fs::write(&path, serde_json::to_string_pretty(&v).unwrap()).unwrap();
        let err = match load_dataset(&ddir) {
            Ok(_) => panic!("B2: a forged training_root must be refused naming data-manifest.json"),
            Err(e) => e.to_string(),
        };
        assert!(
            err.contains("data-manifest.json") && err.contains("training_root"),
            "B2: a forged training_root must be refused naming data-manifest.json, got: {err}"
        );
    }

    /// B2: a forged `split_digest` in `data-manifest.json` must be refused, naming the file.
    #[test]
    fn b2_a_forged_split_digest_is_refused_naming_the_manifest() {
        let ds = dataset();
        let ddir = tmpdir("b2-split");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let path = ddir.join("data-manifest.json");
        let mut v: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        v["split_digest"] = serde_json::Value::String(Hash::of(&[b"forged"]).to_hex());
        std::fs::write(&path, serde_json::to_string_pretty(&v).unwrap()).unwrap();
        let err = match load_dataset(&ddir) {
            Ok(_) => panic!("B2: a forged split_digest must be refused naming data-manifest.json"),
            Err(e) => e.to_string(),
        };
        assert!(
            err.contains("data-manifest.json") && err.contains("split_digest"),
            "B2: a forged split_digest must be refused naming data-manifest.json, got: {err}"
        );
    }

    /// B8: a forged `calibrator` in `manifest.toml` must make `verify` refuse, naming manifest.toml.
    /// Today the id does not cover the calibrator field, so the forgery verifies.
    #[test]
    fn b8_a_forged_calibrator_is_refused_naming_manifest_toml() {
        let ds = dataset();
        let ddir = tmpdir("b8-cal");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let out = tmpdir("b8-cal-out");
        write_bundle(&ddir, &out, 30).unwrap();
        let path = out.join("manifest.toml");
        let text = std::fs::read_to_string(&path).unwrap();
        let forged: String = text
            .lines()
            .map(|l| {
                if l.trim_start().starts_with("calibrator =") {
                    "calibrator = \"deadbeefcafe features@v1\"".to_string()
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&path, forged).unwrap();
        let err = match verify_bundle(&out, None) {
            Ok(_) => panic!("B8: a forged calibrator must be refused naming manifest.toml"),
            Err(e) => e.to_string(),
        };
        assert!(
            err.contains("manifest.toml"),
            "B8: a forged calibrator must be refused naming manifest.toml, got: {err}"
        );
    }

    /// B8: a forged `state` in `manifest.toml` must make `verify` refuse, naming manifest.toml.
    #[test]
    fn b8_a_forged_state_is_refused_naming_manifest_toml() {
        let ds = dataset();
        let ddir = tmpdir("b8-state");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let out = tmpdir("b8-state-out");
        write_bundle(&ddir, &out, 30).unwrap();
        let path = out.join("manifest.toml");
        let text = std::fs::read_to_string(&path).unwrap();
        let forged: String = text
            .lines()
            .map(|l| {
                if l.trim_start().starts_with("state =") {
                    "state = \"released\"".to_string()
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&path, forged).unwrap();
        let err = match verify_bundle(&out, None) {
            Ok(_) => panic!("B8: a forged state must be refused naming manifest.toml"),
            Err(e) => e.to_string(),
        };
        assert!(
            err.contains("manifest.toml"),
            "B8: a forged state must be refused naming manifest.toml, got: {err}"
        );
    }

    /// F6/B8: `load_bundle` (used by report and apply alike) must refuse a forged calibrator,
    /// naming manifest.toml. Before the fix only `verify_bundle` checked it, so `apply` printed the
    /// forged version on every p.
    #[test]
    fn f6_load_bundle_refuses_a_forged_calibrator() {
        let ds = dataset();
        let ddir = tmpdir("f6-cal-ds");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let out = tmpdir("f6-cal-out");
        write_bundle(&ddir, &out, 30).unwrap();
        let path = out.join("manifest.toml");
        let text = std::fs::read_to_string(&path).unwrap();
        let forged: String = text
            .lines()
            .map(|l| {
                if l.trim_start().starts_with("calibrator =") {
                    "calibrator = \"000000000000 features@v9\"".to_string()
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&path, forged).unwrap();
        let err = match load_bundle(&out) {
            Ok(_) => panic!("F6: load_bundle must refuse a forged calibrator"),
            Err(e) => e.to_string(),
        };
        assert!(
            err.contains("manifest.toml") && err.contains("calibrator"),
            "F6: load_bundle must refuse a forged calibrator naming manifest.toml, got: {err}"
        );
    }

    /// F6/B8: `load_bundle` must refuse a state other than `draft`.
    #[test]
    fn f6_load_bundle_refuses_a_forged_state() {
        let ds = dataset();
        let ddir = tmpdir("f6-state-ds");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let out = tmpdir("f6-state-out");
        write_bundle(&ddir, &out, 30).unwrap();
        let path = out.join("manifest.toml");
        let text = std::fs::read_to_string(&path).unwrap();
        let forged: String = text
            .lines()
            .map(|l| {
                if l.trim_start().starts_with("state =") {
                    "state = \"released\"".to_string()
                } else {
                    l.to_string()
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        std::fs::write(&path, forged).unwrap();
        let err = match load_bundle(&out) {
            Ok(_) => panic!("F6: load_bundle must refuse a forged state"),
            Err(e) => e.to_string(),
        };
        assert!(
            err.contains("manifest.toml") && err.contains("state"),
            "F6: load_bundle must refuse a forged state naming manifest.toml, got: {err}"
        );
    }

    /// F6/B8: any byte edit to manifest.toml — a comment, whitespace, a reordered key — must be
    /// refused, because the manifest must be the canonical rendering of its own fields.
    #[test]
    fn f6_a_comment_in_the_manifest_is_refused() {
        let ds = dataset();
        let ddir = tmpdir("f6-comment-ds");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let out = tmpdir("f6-comment-out");
        write_bundle(&ddir, &out, 30).unwrap();
        let path = out.join("manifest.toml");
        let text = std::fs::read_to_string(&path).unwrap();
        std::fs::write(&path, format!("# a comment\n{text}")).unwrap();
        let err = match load_bundle(&out) {
            Ok(_) => panic!("F6: a comment in manifest.toml must be refused"),
            Err(e) => e.to_string(),
        };
        assert!(
            err.contains("manifest.toml") && err.contains("canonical"),
            "F6: a comment must be refused as non-canonical, got: {err}"
        );
    }

    /// P1: `code_commit` is recorded provenance, not a verified property. An untouched bundle must
    /// verify whatever `TSR_CODE_COMMIT` says — unset, equal or different — because no signature
    /// backs the field and anyone who can edit the manifest can edit it consistently.
    #[test]
    fn p1_verify_ignores_the_code_commit_environment() {
        let ds = dataset();
        let ddir = tmpdir("p1-commit-ds");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let out = tmpdir("p1-commit-out");
        write_bundle(&ddir, &out, 30).unwrap();

        // Unset: the recorded commit is whatever the fitter's environment said.
        std::env::remove_var("TSR_CODE_COMMIT");
        assert!(
            verify_bundle(&out, None).is_ok(),
            "P1: an untouched bundle verifies with TSR_CODE_COMMIT unset"
        );

        // Equal to the recorded value.
        let recorded = load_bundle(&out).unwrap().manifest.code_commit.clone();
        std::env::set_var("TSR_CODE_COMMIT", &recorded);
        assert!(
            verify_bundle(&out, None).is_ok(),
            "P1: an untouched bundle verifies with TSR_CODE_COMMIT equal to the recorded value"
        );

        // Different from the recorded value: still verifies.
        std::env::set_var("TSR_CODE_COMMIT", "abc123");
        assert!(
            verify_bundle(&out, None).is_ok(),
            "P1: an untouched bundle verifies from another commit"
        );
        std::env::remove_var("TSR_CODE_COMMIT");
    }

    /// F6/B8: `load_bundle` is the one function verify, report and apply run, so every manifest or
    /// content tamper `verify` refuses, `load_bundle` refuses too. A `code_commit` edit is refused
    /// only when it changes the canonical rendering (P1: it is recorded, not verified).
    #[test]
    fn f6_load_bundle_refuses_every_tamper_verify_refuses() {
        let ds = dataset();
        let ddir = tmpdir("f6-shared-ds");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();

        // Each tamper is applied to a fresh bundle; both `verify_bundle` and `load_bundle` must
        // refuse it (exit 2 in the CLI, which runs `load_bundle` for report and apply).
        type Tamper = (&'static str, Box<dyn Fn(&Path)>);
        let tampers: Vec<Tamper> = vec![
            (
                "a comment",
                Box::new(|dir: &Path| {
                    let p = dir.join("manifest.toml");
                    let text = std::fs::read_to_string(&p).unwrap();
                    std::fs::write(&p, format!("# a comment\n{text}")).unwrap();
                }),
            ),
            (
                "a whitespace edit",
                Box::new(|dir: &Path| {
                    let p = dir.join("manifest.toml");
                    let text = std::fs::read_to_string(&p).unwrap();
                    std::fs::write(&p, text.replace("state = ", "state  = ")).unwrap();
                }),
            ),
            (
                "a forged calibrator",
                Box::new(|dir: &Path| {
                    let p = dir.join("manifest.toml");
                    let text = std::fs::read_to_string(&p).unwrap();
                    let forged: String = text
                        .lines()
                        .map(|l| {
                            if l.trim_start().starts_with("calibrator =") {
                                "calibrator = \"000000000000 features@v9\"".to_string()
                            } else {
                                l.to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    std::fs::write(&p, forged).unwrap();
                }),
            ),
            (
                "state=\"active\"",
                Box::new(|dir: &Path| {
                    let p = dir.join("manifest.toml");
                    let text = std::fs::read_to_string(&p).unwrap();
                    let forged: String = text
                        .lines()
                        .map(|l| {
                            if l.trim_start().starts_with("state =") {
                                "state = \"active\"".to_string()
                            } else {
                                l.to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    std::fs::write(&p, forged).unwrap();
                }),
            ),
            (
                "a missing head.json",
                Box::new(|dir: &Path| {
                    std::fs::remove_file(dir.join("head.json")).unwrap();
                }),
            ),
            (
                "a one-byte isotonic.json change",
                Box::new(|dir: &Path| {
                    let p = dir.join("isotonic.json");
                    let mut bytes = std::fs::read(&p).unwrap();
                    let last = bytes.len() - 2;
                    bytes[last] ^= 0x01;
                    std::fs::write(&p, &bytes).unwrap();
                }),
            ),
            (
                // P1: an edited `code_commit` line is refused only because it changes the
                // canonical rendering, not because it is compared with the environment.
                "an edited code_commit line",
                Box::new(|dir: &Path| {
                    let p = dir.join("manifest.toml");
                    let text = std::fs::read_to_string(&p).unwrap();
                    let forged: String = text
                        .lines()
                        .map(|l| {
                            if l.trim_start().starts_with("code_commit =") {
                                "code_commit = \"deadbeef\"".to_string()
                            } else {
                                l.to_string()
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    std::fs::write(&p, forged).unwrap();
                }),
            ),
        ];

        for (name, tamper) in tampers {
            let out = tmpdir(&format!("f6-shared-{}", name.replace(' ', "-")));
            write_bundle(&ddir, &out, 30).unwrap();
            tamper(&out);
            assert!(
                verify_bundle(&out, None).is_err(),
                "F6/B8: verify must refuse {name}"
            );
            assert!(
                load_bundle(&out).is_err(),
                "F6/B8: load_bundle (report and apply) must refuse {name} too"
            );
        }
    }

    /// F6/B8: a deleted content file must be named, not reported as a bare io error.
    #[test]
    fn f6_a_missing_file_is_named() {
        let ds = dataset();
        let ddir = tmpdir("f6-missing-ds");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let out = tmpdir("f6-missing-out");
        write_bundle(&ddir, &out, 30).unwrap();
        std::fs::remove_file(out.join("head.json")).unwrap();
        let err = match load_bundle(&out) {
            Ok(_) => panic!("F6: a deleted head.json must be refused"),
            Err(e) => e.to_string(),
        };
        assert!(
            err.contains("head.json"),
            "F6: a missing file must be named, got: {err}"
        );
    }

    /// F7: editing a row's `family` in rows.jsonl must be refused, naming the row by its id.
    #[test]
    fn f7_a_family_edit_is_refused_naming_the_row() {
        let ds = dataset();
        let ddir = tmpdir("f7-family-ds");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let text = std::fs::read_to_string(ddir.join("rows.jsonl")).unwrap();
        let mut out = String::new();
        let mut first_id = String::new();
        for line in text.lines() {
            let mut v: serde_json::Value = serde_json::from_str(line).unwrap();
            if first_id.is_empty() {
                first_id = v["id"].as_str().unwrap().to_string();
                v["family"] = serde_json::Value::String("forged-family".to_string());
                // The id is left as it was: the edit is exactly what the id must catch.
            }
            out.push_str(&serde_json::to_string(&v).unwrap());
            out.push('\n');
        }
        std::fs::write(ddir.join("rows.jsonl"), out).unwrap();
        let err = match load_dataset(&ddir) {
            Ok(_) => panic!("F7: a family edit must be refused"),
            Err(e) => e.to_string(),
        };
        assert!(
            err.contains("rows.jsonl") && err.contains(&first_id),
            "F7: a family edit must be refused naming the row id {first_id}, got: {err}"
        );
    }

    /// F7: editing a held-out row's `generator_raw` must be refused — before the fix it flipped the
    /// report's verdict with no id mismatch.
    #[test]
    fn f7_a_generator_raw_edit_is_refused() {
        let ds = dataset();
        let ddir = tmpdir("f7-gen-ds");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let text = std::fs::read_to_string(ddir.join("rows.jsonl")).unwrap();
        let mut out = String::new();
        for line in text.lines() {
            let mut v: serde_json::Value = serde_json::from_str(line).unwrap();
            if v["generator_raw"].is_null() {
                v["generator_raw"] = serde_json::json!(0.99);
                // The id is left as it was: the edit is exactly what the id must catch.
            }
            out.push_str(&serde_json::to_string(&v).unwrap());
            out.push('\n');
        }
        std::fs::write(ddir.join("rows.jsonl"), out).unwrap();
        let err = match load_dataset(&ddir) {
            Ok(_) => panic!("F7: a generator_raw edit must be refused"),
            Err(e) => e.to_string(),
        };
        assert!(
            err.contains("rows.jsonl"),
            "F7: a generator_raw edit must be refused naming rows.jsonl, got: {err}"
        );
    }

    /// B9: a bundle is written once. `write_bundle` must refuse a non-empty `--out`.
    #[test]
    fn b9_a_non_empty_out_is_refused() {
        let ds = dataset();
        let ddir = tmpdir("b9-ds");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let out = tmpdir("b9-out");
        write_bundle(&ddir, &out, 30).unwrap();
        let err = match write_bundle(&ddir, &out, 30) {
            Ok(_) => panic!("B9: a non-empty --out must be refused"),
            Err(e) => e.to_string(),
        };
        assert!(
            err.contains("exists") || err.contains("not empty") || err.contains("written once"),
            "B9: a non-empty --out must be refused, got: {err}"
        );
    }

    /// B9: `verify --rebuild` builds in a temporary directory and removes it, so the bundle
    /// directory is unchanged and no `rebuild` subdirectory is left behind.
    #[test]
    fn b9_rebuild_leaves_no_temporary_directory() {
        let ds = dataset();
        let ddir = tmpdir("b9-rb-ds");
        crate::dataset::write_dataset(&ds, &ddir).unwrap();
        let out = tmpdir("b9-rb-out");
        write_bundle(&ddir, &out, 30).unwrap();
        verify_bundle(&out, Some(&ddir)).unwrap();
        assert!(
            !out.join("rebuild").exists(),
            "B9: verify --rebuild must not leave a rebuild directory in the bundle"
        );
    }
}
