//! The controller against the generator on held-out rows (PRD §B7 `tsr eval calibration`; lane 06
//! item 8).
//!
//! `tsr train report <bundle> --dataset <dir> [--seed <n>] [--json]` scores, on `held_out` rows
//! with a resolved label, the controller's calibrated `p` against the generator's raw `p` for the
//! same commitment: Brier, log loss (clip named and printed) and reliability (10 equal-width bins
//! with counts) per forecaster, per family and for the bundle. The operator's raw `p` is scored
//! beside them where recorded.
//!
//! The paired difference, controller minus generator, is taken per task (the task is the unit)
//! with a 90% cluster-bootstrap interval over repository lineage (2000 resamples, seeded),
//! INCONCLUSIVE when it spans zero, and `interval n/a` with fewer than two lineage clusters.
//! Uncalibrated rows are excluded from the pair and counted.
//!
//! Every printed number goes through `tsr_domain::Proxy` with its engineering name (`brier`,
//! `log_loss`, `support`, `coverage`); a calibrated number prints as `p` with its calibrator
//! version, a raw one as `raw p`, a missing calibrator as `uncalibrated`. Families and forecasters
//! are listed in name order, never by score.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::bundle::Bundle;
use crate::dataset::{Dataset, Row, Side};
use crate::head::predict_checked;
use crate::isotonic::apply_isotonic;
use crate::support::support;
use tsr_domain::{Proxy, Support};

/// The log-loss clip: a probability is clamped to `[CLIP, 1 - CLIP]` before the log.
pub const LOG_LOSS_CLIP: f64 = 1e-12;

/// The number of reliability bins.
pub const RELIABILITY_BINS: usize = 10;

/// The bootstrap resample count.
pub const BOOTSTRAP_RESAMPLES: usize = 2000;

/// The bootstrap confidence level.
pub const BOOTSTRAP_LEVEL: f64 = 0.90;

/// One forecaster's scores over a set of rows.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scores {
    pub n: u32,
    pub brier: f64,
    pub log_loss: f64,
    /// 10 equal-width bins: `(lower, upper, count)`.
    pub reliability: Vec<(f64, f64, u32)>,
}

/// A paired difference with its cluster-bootstrap interval.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Paired {
    /// The number of tasks in the pair.
    pub tasks: u32,
    /// The mean paired difference, controller minus generator.
    pub mean: f64,
    /// The 90% interval, or `None` when there are fewer than two lineage clusters.
    pub interval: Option<(f64, f64)>,
    /// True when the interval spans zero.
    pub inconclusive: bool,
    /// The number of lineage clusters.
    pub clusters: u32,
}

/// The report: the bundle's scores, per family, per forecaster, and the paired difference.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Report {
    pub bundle_id: String,
    /// The dataset's split digest, so a report names the split it scored.
    pub split_digest: String,
    pub calibrator: String,
    pub seed: u64,
    /// The number of held_out rows with a resolved label.
    pub scored: u32,
    /// The number of held_out rows excluded (censored/unresolved).
    pub excluded: u32,
    /// The number of held_out rows excluded from the pair because they are uncalibrated.
    pub uncalibrated: u32,
    /// The controller's scores over all scored rows.
    pub controller: Scores,
    /// The generator's scores over all scored rows.
    pub generator: Scores,
    /// The operator's scores where recorded.
    pub operator: Option<Scores>,
    /// The logged controller forecasts' scores, where recorded (labeled as logged).
    pub controller_logged: Option<Scores>,
    /// Per family: `(controller, generator, operator)`.
    pub families: BTreeMap<String, FamilyScores>,
    /// Per family on the calibrate slice: coverage and controller Brier at floors 0,5,10,20,30,50.
    pub floor_sweep: BTreeMap<String, Vec<FloorPoint>>,
    /// The paired difference, overall.
    pub paired: Paired,
    /// The paired difference, per family.
    pub paired_by_family: BTreeMap<String, Paired>,
    /// The support floor and the calibrated population.
    pub support_floor: u32,
    pub population: BTreeMap<String, u32>,
}

/// One family's scores.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FamilyScores {
    pub controller: Scores,
    pub generator: Scores,
    pub operator: Option<Scores>,
}

/// One point of the floor sweep: a floor, the coverage it leaves and the controller's Brier.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FloorPoint {
    pub floor: u32,
    pub coverage: u32,
    pub brier: f64,
}

/// The floors the report sweeps (brief item 6).
pub const FLOOR_SWEEP: [u32; 6] = [0, 5, 10, 20, 30, 50];

/// The controller's calibrated `p` for a row, or `None` when the row is uncalibrated.
///
/// B1: a row whose feature length differs from the bundle's is refused rather than zipped.
pub fn calibrated_p(bundle: &Bundle, row: &Row) -> Option<f64> {
    if !bundle.population_contains(&row.family) {
        return None;
    }
    let raw = predict_checked(&bundle.head, &row.features).ok()?;
    Some(apply_isotonic(&bundle.isotonic, raw))
}

/// The controller's raw `p` for a row (the head's output, always).
///
/// B1: a row whose feature length differs from the bundle's is refused rather than zipped.
pub fn raw_p(bundle: &Bundle, row: &Row) -> f64 {
    predict_checked(&bundle.head, &row.features).unwrap_or(f64::NAN)
}

/// Score a set of `(p, y)` pairs.
pub fn score(pairs: &[(f64, f64)]) -> Scores {
    let n = pairs.len() as u32;
    if pairs.is_empty() {
        return Scores {
            n: 0,
            brier: 0.0,
            log_loss: 0.0,
            reliability: Vec::new(),
        };
    }
    let mut brier = 0.0;
    let mut log_loss = 0.0;
    let mut bins = [0u32; RELIABILITY_BINS];
    for (p, y) in pairs {
        brier += (p - y) * (p - y);
        let c = p.clamp(LOG_LOSS_CLIP, 1.0 - LOG_LOSS_CLIP);
        log_loss += -(y * c.ln() + (1.0 - y) * (1.0 - c).ln());
        let idx = ((p * RELIABILITY_BINS as f64) as usize).min(RELIABILITY_BINS - 1);
        bins[idx] += 1;
    }
    let nf = pairs.len() as f64;
    let reliability = bins
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let lo = i as f64 / RELIABILITY_BINS as f64;
            let hi = (i + 1) as f64 / RELIABILITY_BINS as f64;
            (lo, hi, *c)
        })
        .collect();
    Scores {
        n,
        brier: brier / nf,
        log_loss: log_loss / nf,
        reliability,
    }
}

/// The support of a row against the bundle's support index.
pub fn row_support(bundle: &Bundle, row: &Row) -> Support {
    support(
        &bundle.spec,
        &bundle.support_index,
        &row.family,
        &row.features,
    )
}

/// A `Proxy`-rendered number with its engineering name.
pub fn proxy(name: &str, value: f64) -> String {
    Proxy::new(name, format!("{value:.6}")).to_string()
}

/// The held_out rows with a resolved label, in canonical order.
fn scored_rows(ds: &Dataset) -> Vec<&Row> {
    ds.rows
        .iter()
        .filter(|r| r.side == Side::HeldOut && r.label.is_some())
        .collect()
}

/// B1: refuse a dataset that does not belong to the bundle.
///
/// `report` scores the bundle's head and calibrator against the dataset's rows, so a dataset with
/// another split or another feature spec is not the bundle's evaluation set: a foreign split makes
/// the bundle's own fit and calibrate rows look held-out, and a foreign feature length makes
/// `predict` zip misaligned vectors and print a plausible score. The split digest and the feature
/// spec (schema version, feature names and length) must match; the error names what differs.
pub fn check_dataset_matches_bundle(
    bundle: &Bundle,
    dataset: &Dataset,
) -> Result<(), crate::LearnError> {
    let want = &bundle.manifest.split_digest;
    let got = dataset.split.digest.to_hex();
    if want != &got {
        return Err(crate::LearnError::Invalid(format!(
            "dataset split digest {got} does not match the bundle's {want}"
        )));
    }
    if dataset.spec.version != bundle.spec.version {
        return Err(crate::LearnError::Invalid(format!(
            "dataset feature schema {} does not match the bundle's {}",
            dataset.spec.version, bundle.spec.version
        )));
    }
    if dataset.spec.names.len() != bundle.spec.names.len() {
        return Err(crate::LearnError::Invalid(format!(
            "dataset feature length {} does not match the bundle's {}",
            dataset.spec.names.len(),
            bundle.spec.names.len()
        )));
    }
    for (i, (a, b)) in dataset
        .spec
        .names
        .iter()
        .zip(bundle.spec.names.iter())
        .enumerate()
    {
        if a != b {
            return Err(crate::LearnError::Invalid(format!(
                "dataset feature {i} is {a:?}, the bundle's is {b:?}"
            )));
        }
    }
    Ok(())
}

/// Build the report, logging every held_out row read to `<dir>/holdout-access.jsonl` (B9).
///
/// The split digest covers assignments only, so this logging never changes it.
pub fn report_logged(
    bundle: &Bundle,
    dataset: &Dataset,
    seed: u64,
    dir: &std::path::Path,
) -> Result<Report, crate::LearnError> {
    // B1: refuse a dataset that does not belong to the bundle, before any row is read or logged.
    check_dataset_matches_bundle(bundle, dataset)?;
    for r in dataset.rows.iter().filter(|r| r.side == Side::HeldOut) {
        crate::probes::log_holdout_access(dir, r, "report")?;
    }
    Ok(report(bundle, dataset, seed))
}

/// Build the report: score the controller against the generator on held_out rows.
pub fn report(bundle: &Bundle, dataset: &Dataset, seed: u64) -> Report {
    let rows = scored_rows(dataset);
    let excluded = dataset
        .rows
        .iter()
        .filter(|r| r.side == Side::HeldOut && r.label.is_none())
        .count() as u32;

    // Per-row pairs.
    let mut ctrl_pairs: Vec<(f64, f64)> = Vec::new();
    let mut gen_pairs: Vec<(f64, f64)> = Vec::new();
    let mut op_pairs: Vec<(f64, f64)> = Vec::new();
    let mut logged_pairs: Vec<(f64, f64)> = Vec::new();
    let mut uncalibrated = 0u32;

    // Per family.
    let mut fam_ctrl: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    let mut fam_gen: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    let mut fam_op: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();

    // The paired difference, per task: (task, lineage, controller_brier - generator_brier).
    let mut per_task: BTreeMap<String, (String, f64, f64)> = BTreeMap::new();

    for r in &rows {
        let y = if r.label == Some(true) { 1.0 } else { 0.0 };
        let gen = r.generator_raw;
        let ctrl = calibrated_p(bundle, r);
        if ctrl.is_none() {
            uncalibrated += 1;
        }
        if let Some(g) = gen {
            gen_pairs.push((g, y));
            fam_gen.entry(r.family.clone()).or_default().push((g, y));
        }
        if let Some(c) = ctrl {
            ctrl_pairs.push((c, y));
            fam_ctrl.entry(r.family.clone()).or_default().push((c, y));
        }
        if let Some(o) = r.operator_raw {
            op_pairs.push((o, y));
            fam_op.entry(r.family.clone()).or_default().push((o, y));
        }
        if let Some(l) = r.controller_logged {
            logged_pairs.push((l, y));
        }
        // The pair: only rows with both a calibrated controller p and a generator p.
        if let (Some(c), Some(g)) = (ctrl, gen) {
            let key = r.task.0.to_hex();
            let entry = per_task
                .entry(key)
                .or_insert_with(|| (r.lineage.clone(), 0.0, 0.0));
            entry.1 += (c - y) * (c - y);
            entry.2 += (g - y) * (g - y);
        }
    }

    let controller = score(&ctrl_pairs);
    let generator = score(&gen_pairs);
    let operator = if op_pairs.is_empty() {
        None
    } else {
        Some(score(&op_pairs))
    };
    let controller_logged = if logged_pairs.is_empty() {
        None
    } else {
        Some(score(&logged_pairs))
    };

    let mut families: BTreeMap<String, FamilyScores> = BTreeMap::new();
    let mut all_fams: Vec<String> = fam_ctrl
        .keys()
        .chain(fam_gen.keys())
        .chain(fam_op.keys())
        .cloned()
        .collect();
    all_fams.sort();
    all_fams.dedup();
    for f in all_fams {
        let c = score(fam_ctrl.get(&f).map(|v| v.as_slice()).unwrap_or(&[]));
        let g = score(fam_gen.get(&f).map(|v| v.as_slice()).unwrap_or(&[]));
        let o = fam_op.get(&f).map(|v| score(v));
        families.insert(
            f,
            FamilyScores {
                controller: c,
                generator: g,
                operator: o,
            },
        );
    }

    // The paired difference: per task, controller minus generator.
    let diffs: Vec<(String, f64)> = per_task
        .values()
        .map(|(lineage, cb, gb)| (lineage.clone(), cb - gb))
        .collect();
    let paired = paired_difference(&diffs, seed);

    // Per family paired difference.
    let mut paired_by_family: BTreeMap<String, Paired> = BTreeMap::new();
    for f in families.keys() {
        let fam_diffs: Vec<(String, f64)> = rows
            .iter()
            .filter(|r| &r.family == f)
            .filter_map(|r| {
                let c = calibrated_p(bundle, r)?;
                let g = r.generator_raw?;
                let y = if r.label == Some(true) { 1.0 } else { 0.0 };
                Some((r.lineage.clone(), (c - y) * (c - y) - (g - y) * (g - y)))
            })
            .collect();
        paired_by_family.insert(f.clone(), paired_difference(&fam_diffs, seed));
    }

    // The floor sweep: per family on the calibrate slice, coverage and controller Brier at each
    // floor. Support is compared with the floor and reported; it never weights or scales anything.
    let mut floor_sweep: BTreeMap<String, Vec<FloorPoint>> = BTreeMap::new();
    let calibrate_rows: Vec<&Row> = dataset
        .rows
        .iter()
        .filter(|r| r.side == Side::Calibrate && r.label.is_some())
        .collect();
    let mut sweep_fams: Vec<String> = calibrate_rows.iter().map(|r| r.family.clone()).collect();
    sweep_fams.sort();
    sweep_fams.dedup();
    for f in sweep_fams {
        let fam_rows: Vec<&&Row> = calibrate_rows.iter().filter(|r| r.family == f).collect();
        let mut points = Vec::new();
        for floor in FLOOR_SWEEP {
            let mut pairs = Vec::new();
            for r in &fam_rows {
                let sup = row_support(bundle, r).episodes;
                if sup >= floor {
                    if let Some(c) = calibrated_p(bundle, r) {
                        let y = if r.label == Some(true) { 1.0 } else { 0.0 };
                        pairs.push((c, y));
                    }
                }
            }
            let s = score(&pairs);
            points.push(FloorPoint {
                floor,
                coverage: s.n,
                brier: s.brier,
            });
        }
        floor_sweep.insert(f, points);
    }

    Report {
        bundle_id: bundle.id.to_hex(),
        split_digest: dataset.split.digest.to_hex(),
        calibrator: bundle.manifest.calibrator.clone(),
        seed,
        scored: rows.len() as u32,
        excluded,
        uncalibrated,
        controller,
        generator,
        operator,
        controller_logged,
        families,
        floor_sweep,
        paired,
        paired_by_family,
        support_floor: bundle.manifest.support_floor,
        population: bundle.population.clone(),
    }
}

/// The paired difference with a 90% cluster-bootstrap interval over lineage.
///
/// `diffs` is `(lineage, difference)` per task. With fewer than two lineage clusters the interval
/// is `None` (`interval n/a`). The interval is INCONCLUSIVE when it spans zero.
pub fn paired_difference(diffs: &[(String, f64)], seed: u64) -> Paired {
    let tasks = diffs.len() as u32;
    if diffs.is_empty() {
        return Paired {
            tasks: 0,
            mean: 0.0,
            interval: None,
            inconclusive: false,
            clusters: 0,
        };
    }
    let mean = diffs.iter().map(|(_, d)| d).sum::<f64>() / diffs.len() as f64;

    // Group by lineage cluster.
    let mut clusters: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for (lineage, d) in diffs {
        clusters.entry(lineage.clone()).or_default().push(*d);
    }
    let n_clusters = clusters.len() as u32;
    if n_clusters < 2 {
        return Paired {
            tasks,
            mean,
            interval: None,
            inconclusive: false,
            clusters: n_clusters,
        };
    }

    // Cluster bootstrap: resample clusters with replacement, 2000 times, seeded.
    let cluster_vec: Vec<Vec<f64>> = clusters.values().cloned().collect();
    let mut rng = crate::rng::SplitMix64::new(seed);
    let mut means: Vec<f64> = Vec::with_capacity(BOOTSTRAP_RESAMPLES);
    for _ in 0..BOOTSTRAP_RESAMPLES {
        let mut sum = 0.0;
        let mut n = 0usize;
        for _ in 0..cluster_vec.len() {
            let idx = (rng.next_u64() as usize) % cluster_vec.len();
            for d in &cluster_vec[idx] {
                sum += d;
                n += 1;
            }
        }
        means.push(if n == 0 { 0.0 } else { sum / n as f64 });
    }
    means.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let lo_idx = (((1.0 - BOOTSTRAP_LEVEL) / 2.0) * means.len() as f64) as usize;
    let hi_idx = ((1.0 - (1.0 - BOOTSTRAP_LEVEL) / 2.0) * means.len() as f64) as usize;
    let lo = means[lo_idx.min(means.len() - 1)];
    let hi = means[hi_idx.min(means.len() - 1)];
    let inconclusive = lo <= 0.0 && hi >= 0.0;
    Paired {
        tasks,
        mean,
        interval: Some((lo, hi)),
        inconclusive,
        clusters: n_clusters,
    }
}

/// Render the report as text or JSON.
///
/// Every number goes through `Proxy` with its engineering name; families and forecasters are in
/// name order; a calibrated number prints as `p` with its calibrator version, a raw one as
/// `raw p`, a missing calibrator as `uncalibrated`.
pub fn render_report(r: &Report, json: bool) -> String {
    if json {
        return serde_json::to_string_pretty(r)
            .unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"));
    }
    let mut out = String::new();
    out.push_str(&format!("bundle {}\n", r.bundle_id));
    out.push_str(&format!("split_digest {}\n", r.split_digest));
    out.push_str(&format!("calibrator {}\n", r.calibrator));
    out.push_str(&format!("seed {}\n", r.seed));
    out.push_str(&format!("scored {}\n", r.scored));
    out.push_str(&format!("excluded {}\n", r.excluded));
    out.push_str(&format!("uncalibrated {}\n", r.uncalibrated));
    out.push_str(&format!("log_loss_clip {}\n", LOG_LOSS_CLIP));
    out.push_str(&format!("support_floor {}\n", r.support_floor));

    out.push_str("\nforecasters\n");
    out.push_str(&render_scores("controller", &r.controller));
    out.push_str(&render_scores("generator", &r.generator));
    if let Some(o) = &r.operator {
        out.push_str(&render_scores("operator", o));
    }
    if let Some(l) = &r.controller_logged {
        out.push_str(&render_scores("controller_logged", l));
    }

    out.push_str("\nfamilies\n");
    for (name, f) in &r.families {
        out.push_str(&format!("family {name}\n"));
        out.push_str(&render_scores("controller", &f.controller));
        out.push_str(&render_scores("generator", &f.generator));
        if let Some(o) = &f.operator {
            out.push_str(&render_scores("operator", o));
        }
    }

    out.push_str("\npaired controller minus generator\n");
    out.push_str(&render_paired("all", &r.paired));
    for (name, p) in &r.paired_by_family {
        out.push_str(&render_paired(name, p));
    }

    out.push_str("\npopulation\n");
    for (name, n) in &r.population {
        out.push_str(&format!("family {name} {}\n", proxy("support", *n as f64)));
    }

    out.push_str("\nfloor sweep (calibrate slice)\n");
    for (name, points) in &r.floor_sweep {
        out.push_str(&format!("family {name}\n"));
        for p in points {
            out.push_str(&format!(
                "  floor={} {} {}\n",
                p.floor,
                proxy("coverage", p.coverage as f64),
                proxy("brier", p.brier)
            ));
        }
    }
    out
}

fn render_scores(label: &str, s: &Scores) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "  {label} n={} {} {}\n",
        s.n,
        proxy("brier", s.brier),
        proxy("log_loss", s.log_loss)
    ));
    for (lo, hi, c) in &s.reliability {
        out.push_str(&format!(
            "    bin [{lo:.1},{hi:.1}) {}\n",
            proxy("coverage", *c as f64)
        ));
    }
    out
}

fn render_paired(label: &str, p: &Paired) -> String {
    let interval = match p.interval {
        Some((lo, hi)) => format!("[{lo:.6}, {hi:.6}]"),
        None => "interval n/a".to_string(),
    };
    let verdict = if p.interval.is_none() {
        "interval n/a"
    } else if p.inconclusive {
        "INCONCLUSIVE"
    } else {
        "conclusive"
    };
    format!(
        "  {label} tasks={} clusters={} mean={} {interval} {verdict}\n",
        p.tasks,
        p.clusters,
        proxy("brier", p.mean)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bundle::{Bundle, Manifest};
    use crate::dataset::{SplitBy, SplitManifest};
    use crate::features::{spec_for_families, FeatureVector};
    use crate::head::{predict_checked, Head};
    use crate::isotonic::Isotonic;
    use crate::support::SupportEntry;
    use std::collections::BTreeMap;
    use tsr_domain::{ActionKind, EventId, Hash, TaskId};

    fn bundle_for(families: &[&str], population: &[&str]) -> Bundle {
        let fams: Vec<String> = families.iter().map(|s| s.to_string()).collect();
        let spec = spec_for_families(&fams);
        let dim = spec.names.len();
        // A head that reads the last feature (a planted signal) and ignores the rest.
        let mut weights = vec![0.0; dim];
        weights[dim - 1] = 4.0;
        let head = Head {
            weights,
            bias: 0.0,
            mean: vec![0.0; dim],
            std: vec![1.0; dim],
            l2: 0.0,
            iters: 1,
        };
        // A step calibrator: raw below 0.75 -> 0.1, at or above -> 0.9. The head separates the
        // labels (raw 0.5 vs 0.982), so the calibrated p is close to the truth.
        let iso = Isotonic {
            breakpoints: vec![(0.75, 0.1), (1.0, 0.9)],
        };
        let mut pop = BTreeMap::new();
        for f in population {
            pop.insert(f.to_string(), 100);
        }
        Bundle {
            id: Hash::of(&[b"bundle"]),
            dir: std::path::PathBuf::from("/tmp/none"),
            manifest: Manifest {
                data_manifest: "d".into(),
                training_root: "t".into(),
                feature_schema: spec.version.clone(),
                code_commit: "unknown".into(),
                seed: 1,
                calibrator: format!(
                    "{} {}",
                    &Hash::of(&[b"bundle"]).to_hex()[..12],
                    spec.version
                ),
                support_floor: 30,
                split_digest: Hash::of(&[b"split"]).to_hex(),
                source_root: "r".into(),
                state: "draft".into(),
                id: Hash::of(&[b"bundle"]).to_hex(),
                content_digests: BTreeMap::new(),
            },
            head,
            isotonic: iso,
            spec,
            split: SplitManifest {
                by: SplitBy::Time,
                seed: 1,
                assignments: BTreeMap::new(),
                digest: Hash::of(&[b"split"]),
            },
            support_index: Vec::<SupportEntry>::new(),
            population: pop,
        }
    }

    fn row(
        b: &Bundle,
        family: &str,
        lineage: &str,
        signal: f64,
        label: bool,
        gen: f64,
        i: u64,
    ) -> Row {
        let dim = b.spec.names.len();
        let mut values = vec![0.0; dim];
        values[dim - 1] = signal;
        Row {
            id: Hash::of(&[format!("row{i}").as_bytes()]),
            task: TaskId(Hash::of(&[format!("t{i}").as_bytes()])),
            commitment: EventId(Hash::of(&[format!("c{i}").as_bytes()])),
            index: i,
            action: ActionKind::ProposePatch,
            family: family.to_string(),
            lineage: lineage.to_string(),
            side: Side::HeldOut,
            features: FeatureVector { values },
            label: Some(label),
            evidence: Vec::new(),
            generator_raw: Some(gen),
            operator_raw: None,
            controller_logged: None,
        }
    }

    fn dataset(b: &Bundle, rows: Vec<Row>) -> Dataset {
        Dataset {
            rows,
            split: b.split.clone(),
            spec: b.spec.clone(),
            data_root: Hash::of(&[b"data"]),
            training_root: Hash::of(&[b"train"]),
            source_root: Hash::of(&[b"source"]),
        }
    }

    #[test]
    fn same_seed_gives_the_same_output() {
        let b = bundle_for(&["calibrated"], &["calibrated"]);
        let mut rows = Vec::new();
        for i in 0..40u64 {
            let label = i % 2 == 0;
            let signal = if label { 1.0 } else { 0.0 };
            rows.push(row(
                &b,
                "calibrated",
                &format!("l{}", i % 4),
                signal,
                label,
                0.5,
                i,
            ));
        }
        let ds = dataset(&b, rows);
        let a = render_report(&report(&b, &ds, 7), false);
        let c = render_report(&report(&b, &ds, 7), false);
        assert_eq!(a, c, "same seed gives the same report");
    }

    #[test]
    fn controller_beats_generator_with_interval_below_zero() {
        let b = bundle_for(&["overconfident"], &["overconfident"]);
        let mut rows = Vec::new();
        for i in 0..80u64 {
            let label = i % 2 == 0;
            let signal = if label { 1.0 } else { 0.0 };
            // The generator is overconfident: it says 0.9 where the family succeeds ~0.5.
            rows.push(row(
                &b,
                "overconfident",
                &format!("l{}", i % 8),
                signal,
                label,
                0.9,
                i,
            ));
        }
        let ds = dataset(&b, rows);
        let r = report(&b, &ds, 11);
        assert!(
            r.controller.brier < r.generator.brier,
            "controller {} < generator {}",
            r.controller.brier,
            r.generator.brier
        );
        let p = &r.paired;
        assert!(p.interval.is_some(), "two or more clusters");
        let (lo, hi) = p.interval.unwrap();
        assert!(hi < 0.0, "interval [{lo}, {hi}] wholly below zero");
        assert!(!p.inconclusive);
        // The overconfident family's pair is also below zero.
        let fam = &r.paired_by_family["overconfident"];
        assert!(fam.interval.unwrap().1 < 0.0);
    }

    #[test]
    fn straddling_input_prints_inconclusive() {
        // Controller and generator are identical, so the difference is exactly zero and the
        // interval straddles zero.
        let b = bundle_for(&["calibrated"], &["calibrated"]);
        let mut rows = Vec::new();
        for i in 0..40u64 {
            let label = i % 2 == 0;
            let signal = if label { 1.0 } else { 0.0 };
            // The generator equals the controller's calibrated p (identity isotonic, same head).
            let gen =
                crate::report::calibrated_p(&b, &row(&b, "calibrated", "l", signal, label, 0.0, i))
                    .unwrap();
            rows.push(row(
                &b,
                "calibrated",
                &format!("l{}", i % 4),
                signal,
                label,
                gen,
                i,
            ));
        }
        let ds = dataset(&b, rows);
        let r = report(&b, &ds, 3);
        assert!(r.paired.inconclusive, "a zero difference straddles zero");
        let text = render_report(&r, false);
        assert!(text.contains("INCONCLUSIVE"), "{text}");
    }

    #[test]
    fn fewer_than_two_clusters_prints_interval_na() {
        let b = bundle_for(&["calibrated"], &["calibrated"]);
        let mut rows = Vec::new();
        for i in 0..20u64 {
            let label = i % 2 == 0;
            let signal = if label { 1.0 } else { 0.0 };
            rows.push(row(&b, "calibrated", "only", signal, label, 0.9, i));
        }
        let ds = dataset(&b, rows);
        let r = report(&b, &ds, 5);
        assert!(r.paired.interval.is_none());
        let text = render_report(&r, false);
        assert!(text.contains("interval n/a"), "{text}");
    }

    #[test]
    fn uncalibrated_rows_are_excluded_and_counted() {
        // The bundle's population holds only "calibrated"; an "underconfident" row is uncalibrated.
        let b = bundle_for(&["calibrated", "underconfident"], &["calibrated"]);
        let mut rows = Vec::new();
        for i in 0..20u64 {
            let label = i % 2 == 0;
            let signal = if label { 1.0 } else { 0.0 };
            rows.push(row(
                &b,
                "calibrated",
                &format!("l{}", i % 3),
                signal,
                label,
                0.9,
                i,
            ));
        }
        for i in 20..30u64 {
            let label = i % 2 == 0;
            let signal = if label { 1.0 } else { 0.0 };
            rows.push(row(
                &b,
                "underconfident",
                &format!("l{}", i % 3),
                signal,
                label,
                0.9,
                i,
            ));
        }
        let ds = dataset(&b, rows);
        let r = report(&b, &ds, 9);
        assert_eq!(
            r.uncalibrated, 10,
            "the ten underconfident rows are uncalibrated"
        );
        let text = render_report(&r, false);
        assert!(text.contains("uncalibrated 10"), "{text}");
    }

    #[test]
    fn no_forbidden_strings_and_calibrator_version_printed() {
        let b = bundle_for(&["calibrated"], &["calibrated"]);
        let mut rows = Vec::new();
        for i in 0..40u64 {
            let label = i % 2 == 0;
            let signal = if label { 1.0 } else { 0.0 };
            rows.push(row(
                &b,
                "calibrated",
                &format!("l{}", i % 4),
                signal,
                label,
                0.9,
                i,
            ));
        }
        let ds = dataset(&b, rows);
        let text = render_report(&report(&b, &ds, 2), false);
        assert!(!text.contains("percent certain"), "{text}");
        // The calibrator version is the bundle id's first 12 hex and features@v1.
        let prefix = &b.id.to_hex()[..12];
        assert!(
            text.contains(prefix),
            "calibrator prefix {prefix} in {text}"
        );
        assert!(text.contains(&b.spec.version), "feature version in {text}");
        // Every number is a Proxy with its engineering name.
        assert!(text.contains("brier="), "{text}");
        assert!(text.contains("log_loss="), "{text}");
    }

    /// B1: `report` must refuse a dataset that does not belong to the bundle. The dataset's split
    /// digest and its feature spec (schema version, feature names and length) must match the
    /// bundle's; today `report` scores whatever it is handed.
    #[test]
    fn b1_report_refuses_a_foreign_dataset() {
        let b = bundle_for(&["calibrated"], &["calibrated"]);
        let mut rows = Vec::new();
        for i in 0..20u64 {
            let label = i % 2 == 0;
            let signal = if label { 1.0 } else { 0.0 };
            rows.push(row(&b, "calibrated", "l", signal, label, 0.9, i));
        }

        // A dataset with another split digest is refused, naming the split digest.
        let mut foreign_split = dataset(&b, rows.clone());
        foreign_split.split.digest = Hash::of(&[b"another split"]);
        let err = report_logged(&b, &foreign_split, 7, std::path::Path::new("/tmp/none"))
            .expect_err("a foreign split digest is refused");
        let msg = err.to_string();
        assert!(msg.contains("split digest"), "{msg}");

        // A dataset with another feature-schema version is refused, naming the schema version.
        let mut foreign_schema = dataset(&b, rows.clone());
        foreign_schema.spec.version = "features@v2".to_string();
        let err = report_logged(&b, &foreign_schema, 7, std::path::Path::new("/tmp/none"))
            .expect_err("a foreign feature schema is refused");
        let msg = err.to_string();
        assert!(msg.contains("feature schema"), "{msg}");

        // A dataset with another feature length is refused, naming the length.
        let mut foreign_len = dataset(&b, rows.clone());
        foreign_len.spec.names.pop();
        foreign_len.spec.numeric_widths.pop();
        let err = report_logged(&b, &foreign_len, 7, std::path::Path::new("/tmp/none"))
            .expect_err("a foreign feature length is refused");
        let msg = err.to_string();
        assert!(msg.contains("feature length"), "{msg}");

        // A dataset with another feature name is refused, naming the feature.
        let mut foreign_name = dataset(&b, rows.clone());
        foreign_name.spec.names[0] = "not_a_feature".to_string();
        let err = report_logged(&b, &foreign_name, 7, std::path::Path::new("/tmp/none"))
            .expect_err("a foreign feature name is refused");
        let msg = err.to_string();
        assert!(msg.contains("not_a_feature"), "{msg}");

        // The matching dataset is accepted.
        let ok = dataset(&b, rows);
        assert!(report_logged(&b, &ok, 7, std::path::Path::new("/tmp/none")).is_ok());
    }

    /// B1: `predict` must return an error on a feature-length mismatch instead of zipping the
    /// shorter vector against the head's weights and printing a plausible number.
    #[test]
    fn b1_predict_errors_on_a_length_mismatch() {
        let b = bundle_for(&["calibrated"], &["calibrated"]);
        let dim = b.spec.names.len();
        let short = FeatureVector {
            values: vec![0.0; dim - 1],
        };
        let err = predict_checked(&b.head, &short).expect_err("a short vector is refused");
        assert!(err.to_string().contains("feature length"), "{err}");
        let long = FeatureVector {
            values: vec![0.0; dim + 1],
        };
        assert!(predict_checked(&b.head, &long).is_err());
        let exact = FeatureVector {
            values: vec![0.0; dim],
        };
        assert!(predict_checked(&b.head, &exact).is_ok());
    }

    #[test]
    fn families_and_forecasters_are_in_name_order() {
        let b = bundle_for(
            &["calibrated", "overconfident"],
            &["calibrated", "overconfident"],
        );
        let mut rows = Vec::new();
        for i in 0..40u64 {
            let label = i % 2 == 0;
            let signal = if label { 1.0 } else { 0.0 };
            let fam = if i % 2 == 0 {
                "overconfident"
            } else {
                "calibrated"
            };
            rows.push(row(&b, fam, &format!("l{}", i % 4), signal, label, 0.9, i));
        }
        let ds = dataset(&b, rows);
        let text = render_report(&report(&b, &ds, 4), false);
        let cal = text.find("family calibrated").unwrap();
        let over = text.find("family overconfident").unwrap();
        assert!(cal < over, "families in name order: {text}");
    }
}
