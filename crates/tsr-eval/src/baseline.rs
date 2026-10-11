//! `tsr eval baseline <results-dir> [--pair-by task]` — the first numbers with
//! intervals (PRD B9 Statistics, B3 primary endpoint).
//!
//! The task is the unit; the comparison is paired by task between the two
//! systems; the interval is a cluster bootstrap over repository lineage; the
//! interval is 90%; an interval spanning zero is reported as INCONCLUSIVE and
//! the incumbent stays (PRD B9 Statistics, line 434).

use std::collections::BTreeMap;
use std::path::Path;

use crate::bootstrap::{
    cluster_bootstrap, ratio_cluster_bootstrap, Cluster, Interval, RatioCluster, RatioTask,
};
use crate::metrics::{
    self, MetricSpec, ALL_METRICS, COST_PER_ACCEPTED, GENERATOR_CALLS_PER_TASK,
    MEAN_TOKENS_PER_CALL, TOKENS_PER_MINUTE, UNSAFE_ATTEMPTS_BLOCKED,
};
use crate::proxy::Proxy;
use crate::results::{is_accepted, ResultsDir};
use crate::EvalError;

/// How the two systems are paired.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PairBy {
    /// Pair by task id: only tasks present in both systems are compared.
    Task,
}

impl PairBy {
    /// Parse the `--pair-by` value. Only `task` is defined today.
    pub fn parse(s: &str) -> Result<Self, EvalError> {
        match s {
            "task" => Ok(PairBy::Task),
            other => Err(EvalError::UnknownPairBy(other.to_string())),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            PairBy::Task => "task",
        }
    }
}

/// One metric's paired result: the two systems' values, the paired difference,
/// and the 90% cluster-bootstrap interval over lineage.
///
/// A metric whose value is undefined on the sample (e.g. cost per accepted
/// completion when a system accepted no task) carries `None` for that system's
/// value and for the paired difference; the report prints `n/a`, never 0.
#[derive(Debug, Clone, PartialEq)]
pub struct MetricResult {
    /// The metric's spec (name, proxy label, unit, direction).
    pub spec: MetricSpec,
    /// The value under system 1 (record), or `None` when undefined.
    pub record: Option<f64>,
    /// The value under system 2 (rules), or `None` when undefined.
    pub rules: Option<f64>,
    /// The paired difference (rules - record) with its interval, or `None`
    /// when either system's value is undefined.
    pub diff: Option<Interval>,
}

impl MetricResult {
    /// The proxy for the paired difference, carrying the metric's label.
    /// `None` when the difference is undefined.
    pub fn diff_proxy(&self) -> Option<Proxy<f64>> {
        self.diff
            .as_ref()
            .map(|d| metrics::proxy_for(&self.spec, d.point))
    }

    /// Whether the interval spans zero, i.e. the report says INCONCLUSIVE.
    /// An undefined difference is inconclusive (nothing to conclude), and so
    /// is a degenerate interval from a single lineage (D5): one repository is
    /// not a sample of repositories, so the interval cannot exclude zero.
    pub fn inconclusive(&self) -> bool {
        match &self.diff {
            Some(d) => !d.is_defined() || d.spans_zero(),
            None => true,
        }
    }
}

/// The whole baseline report.
#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    /// The results directory the report was computed from.
    pub dir: String,
    /// The pairing rule used.
    pub pair_by: PairBy,
    /// The seed used for the bootstrap.
    pub seed: u64,
    /// The number of bootstrap resamples.
    pub resamples: usize,
    /// The number of tasks paired between the two systems.
    pub paired_tasks: usize,
    /// The number of lineages (repositories) the paired tasks span.
    pub lineages: usize,
    /// Tasks present in only one system, as `(task, systems)`.
    pub unpaired: Vec<(String, Vec<&'static str>)>,
    /// The per-metric results, in `ALL_METRICS` order.
    pub metrics: Vec<MetricResult>,
}

impl Report {
    /// The metric result for a spec, by key.
    pub fn metric(&self, key: &str) -> Option<&MetricResult> {
        self.metrics.iter().find(|m| m.spec.key == key)
    }

    /// Whether any metric's interval spans zero.
    pub fn any_inconclusive(&self) -> bool {
        self.metrics.iter().any(|m| m.inconclusive())
    }

    /// Render the report. Deterministic: same inputs and seed -> same bytes.
    pub fn render(&self) -> String {
        let mut s = String::new();
        s.push_str("tsr eval baseline\n");
        s.push_str(&format!("results-dir: {}\n", self.dir));
        s.push_str(&format!("pair-by: {}\n", self.pair_by.as_str()));
        s.push_str(&format!("seed: {}\n", self.seed));
        s.push_str(&format!("resamples: {}\n", self.resamples));
        s.push_str(&format!("paired tasks: {}\n", self.paired_tasks));
        s.push_str(&format!("lineages: {}\n", self.lineages));
        if !self.unpaired.is_empty() {
            s.push_str(&format!("unpaired tasks: {}\n", self.unpaired.len()));
            for (task, systems) in &self.unpaired {
                s.push_str(&format!("  {} (only in {})\n", task, systems.join(", ")));
            }
        }
        s.push('\n');

        for m in &self.metrics {
            s.push_str(&format!("== {} ==\n", m.spec.title));
            s.push_str(&format!("  unit: {}\n", m.spec.unit));
            s.push_str(&format!("  direction: {}\n", m.spec.direction.as_str()));
            s.push_str(&format!("  {}\n", m.spec.proxy_label));
            s.push_str(&format!("  record: {}\n", fmt_value(m.record)));
            s.push_str(&format!("  rules:  {}\n", fmt_value(m.rules)));
            match &m.diff {
                Some(d) => {
                    s.push_str(&format!("  paired diff (rules - record): {:.6}\n", d.point));
                    if d.is_defined() {
                        s.push_str(&format!(
                            "  90% interval (cluster bootstrap over lineage, {} clusters, {} tasks): [{:.6}, {:.6}]\n",
                            d.clusters, d.tasks, d.lo, d.hi
                        ));
                    } else {
                        // D5: fewer than two lineages — every resample is the
                        // same lineage, so the interval is degenerate and must
                        // not be reported as a zero-width "clear".
                        s.push_str(&format!("  90% interval n/a — {} lineage\n", d.clusters));
                    }
                    if m.inconclusive() {
                        if d.is_defined() {
                            s.push_str(
                                "  INCONCLUSIVE: the 90% interval spans zero; the incumbent stays.\n",
                            );
                        } else {
                            s.push_str(
                                "  INCONCLUSIVE: the interval is undefined (fewer than two lineages); the incumbent stays.\n",
                            );
                        }
                    } else {
                        s.push_str("  clear: the 90% interval excludes zero.\n");
                    }
                }
                None => {
                    s.push_str("  paired diff (rules - record): n/a\n");
                    s.push_str(
                        "  INCONCLUSIVE: the metric is undefined (a system has no accepted task); the incumbent stays.\n",
                    );
                }
            }
            s.push('\n');
        }

        s.push_str("-- proxy labels (Appendix A rule 2) --\n");
        for m in &self.metrics {
            s.push_str(&format!("{}: {}\n", m.spec.key, m.spec.proxy_label));
        }
        s
    }
}

/// The per-task paired values for one metric, grouped into lineages.
///
/// `value` maps a task id to the metric's value under one system. The paired
/// difference for a task is `rules - record`. Tasks are grouped by lineage so
/// the bootstrap can resample lineages.
fn paired_clusters(
    rd: &ResultsDir,
    paired: &[String],
    value: impl Fn(&tsr_batch::results::TaskResult) -> f64,
) -> Vec<Cluster> {
    let mut by_lineage: BTreeMap<String, Vec<f64>> = BTreeMap::new();
    for task in paired {
        let rec = &rd.record.by_task[task];
        let rul = &rd.rules.by_task[task];
        let d = value(rul) - value(rec);
        by_lineage.entry(rul.lineage.clone()).or_default().push(d);
    }
    by_lineage
        .into_iter()
        .map(|(lineage, diffs)| Cluster { lineage, diffs })
        .collect()
}

/// The mean of a metric over a set of tasks under one system.
fn mean_over(
    rd: &ResultsDir,
    paired: &[String],
    system: &str,
    value: impl Fn(&tsr_batch::results::TaskResult) -> f64,
) -> f64 {
    if paired.is_empty() {
        return 0.0;
    }
    let table = if system == crate::results::SYSTEM_RECORD {
        &rd.record.by_task
    } else {
        &rd.rules.by_task
    };
    let sum: f64 = paired.iter().map(|t| value(&table[t])).sum();
    sum / paired.len() as f64
}

/// The normalized cost of one result: generator tokens converted to minutes at
/// the declared rate, plus tool minutes, plus operator minutes (taken as 0 —
/// the results record does not carry them).
fn normalized_cost(r: &tsr_batch::results::TaskResult) -> f64 {
    r.generator_tokens as f64 / TOKENS_PER_MINUTE + r.tool_minutes as f64
}

/// The estimated generator calls for one result.
fn estimated_calls(r: &tsr_batch::results::TaskResult) -> f64 {
    r.generator_tokens as f64 / MEAN_TOKENS_PER_CALL
}

/// Format a metric value: six decimals, or `n/a` when undefined.
fn fmt_value(v: Option<f64>) -> String {
    match v {
        Some(x) => format!("{x:.6}"),
        None => "n/a".to_string(),
    }
}

/// The per-task ratio-of-sums contributions for one metric, grouped into
/// lineages. Each task contributes a numerator (its cost when accepted, else 0)
/// and a denominator (1 when accepted, else 0) under each system. The metric is
/// `sum(numerator) / sum(denominator)` over the resampled lineages, per system.
fn ratio_clusters(
    rd: &ResultsDir,
    paired: &[String],
    numerator: impl Fn(&tsr_batch::results::TaskResult) -> f64,
) -> Vec<RatioCluster> {
    let mut by_lineage: BTreeMap<String, Vec<RatioTask>> = BTreeMap::new();
    for task in paired {
        let rec = &rd.record.by_task[task];
        let rul = &rd.rules.by_task[task];
        let t = RatioTask {
            record_num: if is_accepted(rec) {
                numerator(rec)
            } else {
                0.0
            },
            record_den: if is_accepted(rec) { 1.0 } else { 0.0 },
            rules_num: if is_accepted(rul) {
                numerator(rul)
            } else {
                0.0
            },
            rules_den: if is_accepted(rul) { 1.0 } else { 0.0 },
        };
        by_lineage.entry(rul.lineage.clone()).or_default().push(t);
    }
    by_lineage
        .into_iter()
        .map(|(lineage, tasks)| RatioCluster { lineage, tasks })
        .collect()
}

/// Compute the baseline report for a results directory.
pub fn baseline(
    dir: &Path,
    pair_by: PairBy,
    seed: u64,
    resamples: usize,
) -> Result<Report, EvalError> {
    let rd = ResultsDir::read(dir)?;
    let paired = rd.paired_task_ids();
    let unpaired = rd.unpaired_task_ids();

    let mut metrics_out = Vec::new();

    for spec in ALL_METRICS {
        let (record, rules, diff) = match spec.key {
            k if k == crate::metrics::COMPLETION_RATE.key => {
                let v = |r: &tsr_batch::results::TaskResult| if is_accepted(r) { 1.0 } else { 0.0 };
                let clusters = paired_clusters(&rd, &paired, v);
                (
                    Some(mean_over(&rd, &paired, crate::results::SYSTEM_RECORD, v)),
                    Some(mean_over(&rd, &paired, crate::results::SYSTEM_RULES, v)),
                    Some(cluster_bootstrap(&clusters, seed, resamples)),
                )
            }
            k if k == COST_PER_ACCEPTED.key => {
                // Cost per accepted completion is ONE ratio-of-sums metric:
                // `sum(cost of accepted tasks) / count of accepted tasks`, per
                // system. The paired difference is the difference of the two
                // ratios, and the cluster bootstrap resamples lineages and
                // recomputes BOTH ratios per resample. A system with no accepted
                // task has an undefined metric ("n/a"), never 0.
                let clusters = ratio_clusters(&rd, &paired, normalized_cost);
                let ri = ratio_cluster_bootstrap(&clusters, seed, resamples);
                (ri.record, ri.rules, ri.diff)
            }
            k if k == UNSAFE_ATTEMPTS_BLOCKED.key => {
                let v = |r: &tsr_batch::results::TaskResult| r.denials.len() as f64;
                let clusters = paired_clusters(&rd, &paired, v);
                (
                    Some(mean_over(&rd, &paired, crate::results::SYSTEM_RECORD, v)),
                    Some(mean_over(&rd, &paired, crate::results::SYSTEM_RULES, v)),
                    Some(cluster_bootstrap(&clusters, seed, resamples)),
                )
            }
            k if k == GENERATOR_CALLS_PER_TASK.key => {
                let v = estimated_calls;
                let clusters = paired_clusters(&rd, &paired, v);
                (
                    Some(mean_over(&rd, &paired, crate::results::SYSTEM_RECORD, v)),
                    Some(mean_over(&rd, &paired, crate::results::SYSTEM_RULES, v)),
                    Some(cluster_bootstrap(&clusters, seed, resamples)),
                )
            }
            _ => unreachable!("metric {spec:?} has no computation"),
        };

        metrics_out.push(MetricResult {
            spec,
            record,
            rules,
            diff,
        });
    }

    Ok(Report {
        dir: dir.display().to_string(),
        pair_by,
        seed,
        resamples,
        paired_tasks: paired.len(),
        lineages: rd
            .record
            .by_task
            .values()
            .filter(|r| paired.contains(&r.task))
            .map(|r| r.lineage.clone())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        unpaired,
        metrics: metrics_out,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pair_by_parses_task_only() {
        assert_eq!(PairBy::parse("task").unwrap(), PairBy::Task);
        assert!(PairBy::parse("lineage").is_err());
    }
}
