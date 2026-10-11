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
use crate::head::predict;
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
    /// Per family: `(controller, generator, operator)`.
    pub families: BTreeMap<String, FamilyScores>,
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

/// The controller's calibrated `p` for a row, or `None` when the row is uncalibrated.
pub fn calibrated_p(bundle: &Bundle, row: &Row) -> Option<f64> {
    if !bundle.population_contains(&row.family) {
        return None;
    }
    let raw = predict(&bundle.head, &row.features);
    Some(apply_isotonic(&bundle.isotonic, raw))
}

/// The controller's raw `p` for a row (the head's output, always).
pub fn raw_p(bundle: &Bundle, row: &Row) -> f64 {
    predict(&bundle.head, &row.features)
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

    Report {
        bundle_id: bundle.id.to_hex(),
        calibrator: bundle.manifest.calibrator.clone(),
        seed,
        scored: rows.len() as u32,
        excluded,
        uncalibrated,
        controller,
        generator,
        operator,
        families,
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
