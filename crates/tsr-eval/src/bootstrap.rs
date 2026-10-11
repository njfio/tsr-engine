//! Cluster bootstrap over repository lineage (PRD B9: "intervals by cluster
//! bootstrap over repository lineage").
//!
//! The unit of resampling is the *lineage* (the repository), not the task: a
//! repository contributes several tasks, and those tasks are not independent,
//! so resampling tasks would understate the interval. We draw lineages with
//! replacement, take every task of each drawn lineage, and recompute the
//! statistic on that multiset. The 90% interval is the percentile interval of
//! the bootstrap distribution (5th and 95th percentiles).
//!
//! The PRNG is a hand-rolled SplitMix64 so the report is byte-reproducible
//! from a fixed seed and the crate pulls in no `rand` dependency.

/// A deterministic SplitMix64 generator. Same seed -> same stream, always.
#[derive(Debug, Clone)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// The next 64-bit value.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A uniform index in `0..n` (n > 0), by rejection-free modulo of a 64-bit
    /// draw. The bias is negligible for the small n here and the stream stays
    /// deterministic.
    pub fn below(&mut self, n: usize) -> usize {
        debug_assert!(n > 0);
        (self.next_u64() % n as u64) as usize
    }
}

/// One cluster: a lineage and the per-task paired differences it contributes.
#[derive(Debug, Clone)]
pub struct Cluster {
    /// The lineage (repository) name.
    pub lineage: String,
    /// The paired per-task differences for this lineage, in task-id order.
    pub diffs: Vec<f64>,
}

/// One lineage's contribution to a ratio-of-sums statistic: the numerator
/// (sum of the per-task costs) and the denominator (count of accepted tasks)
/// under each of the two systems.
///
/// A ratio-of-sums metric is `sum(numerator) / sum(denominator)` over the
/// resampled lineages, per system. The paired difference is the difference of
/// the two systems' ratios. This is *not* the mean of per-task differences: a
/// task that was not accepted contributes to neither the numerator nor the
/// denominator, so the two statistics differ whenever the accepted counts
/// differ between systems.
#[derive(Debug, Clone)]
pub struct RatioCluster {
    /// The lineage (repository) name.
    pub lineage: String,
    /// Per-task `(record numerator, record denominator, rules numerator,
    /// rules denominator)` for this lineage, in task-id order.
    pub tasks: Vec<RatioTask>,
}

/// One task's contribution to a ratio-of-sums statistic under both systems.
#[derive(Debug, Clone, Copy)]
pub struct RatioTask {
    /// The record system's numerator contribution (0 if not accepted).
    pub record_num: f64,
    /// The record system's denominator contribution (1 if accepted, else 0).
    pub record_den: f64,
    /// The rules system's numerator contribution (0 if not accepted).
    pub rules_num: f64,
    /// The rules system's denominator contribution (1 if accepted, else 0).
    pub rules_den: f64,
}

/// The ratio of sums over a set of tasks: `sum(num) / sum(den)`, or `None`
/// when the denominator is zero (no accepted task — the metric is undefined,
/// never 0).
fn ratio_of_sums(
    tasks: &[RatioTask],
    num: impl Fn(&RatioTask) -> f64,
    den: impl Fn(&RatioTask) -> f64,
) -> Option<f64> {
    let mut n = 0.0;
    let mut d = 0.0;
    for t in tasks {
        n += num(t);
        d += den(t);
    }
    if d == 0.0 {
        None
    } else {
        Some(n / d)
    }
}

/// The result of a ratio-of-sums cluster bootstrap: the two systems' ratios,
/// the paired difference, and the 90% interval over lineage.
#[derive(Debug, Clone, PartialEq)]
pub struct RatioInterval {
    /// The record system's ratio of sums on the observed sample, or `None`
    /// when it has no accepted task (undefined).
    pub record: Option<f64>,
    /// The rules system's ratio of sums on the observed sample, or `None`.
    pub rules: Option<f64>,
    /// The paired difference (rules - record) with its interval. `None` when
    /// either system's ratio is undefined.
    pub diff: Option<Interval>,
    /// The number of lineages resampled.
    pub clusters: usize,
    /// The number of paired tasks behind the point estimate.
    pub tasks: usize,
}

/// Cluster bootstrap of a ratio-of-sums paired difference over lineages.
///
/// Each resample draws `clusters.len()` lineages with replacement, pools their
/// tasks, and recomputes *both* systems' ratios of sums on the pooled multiset;
/// the resample statistic is the difference of those two ratios. The 90%
/// interval is the 5th and 95th percentiles of that distribution. A resample
/// in which either system has no accepted task is skipped (the ratio is
/// undefined there).
pub fn ratio_cluster_bootstrap(
    clusters: &[RatioCluster],
    seed: u64,
    resamples: usize,
) -> RatioInterval {
    let all: Vec<RatioTask> = clusters
        .iter()
        .flat_map(|c| c.tasks.iter().copied())
        .collect();
    let record = ratio_of_sums(&all, |t| t.record_num, |t| t.record_den);
    let rules = ratio_of_sums(&all, |t| t.rules_num, |t| t.rules_den);
    let point = match (record, rules) {
        (Some(r), Some(u)) => Some(u - r),
        _ => None,
    };
    let mut out = RatioInterval {
        record,
        rules,
        diff: None,
        clusters: clusters.len(),
        tasks: all.len(),
    };

    let point = match point {
        Some(p) => p,
        None => return out,
    };

    if clusters.is_empty() || all.is_empty() || resamples == 0 {
        out.diff = Some(Interval {
            point,
            lo: point,
            hi: point,
            resamples,
            clusters: clusters.len(),
            tasks: all.len(),
        });
        return out;
    }

    let mut rng = SplitMix64::new(seed);
    let mut stats = Vec::with_capacity(resamples);
    for _ in 0..resamples {
        let mut pooled: Vec<RatioTask> = Vec::new();
        for _ in 0..clusters.len() {
            let c = &clusters[rng.below(clusters.len())];
            pooled.extend_from_slice(&c.tasks);
        }
        let r = ratio_of_sums(&pooled, |t| t.record_num, |t| t.record_den);
        let u = ratio_of_sums(&pooled, |t| t.rules_num, |t| t.rules_den);
        if let (Some(r), Some(u)) = (r, u) {
            stats.push(u - r);
        }
    }

    if stats.is_empty() {
        out.diff = Some(Interval {
            point,
            lo: point,
            hi: point,
            resamples,
            clusters: clusters.len(),
            tasks: all.len(),
        });
        return out;
    }

    let lo = quantile(&mut stats, 0.05);
    let hi = quantile(&mut stats, 0.95);
    out.diff = Some(Interval {
        point,
        lo,
        hi,
        resamples,
        clusters: clusters.len(),
        tasks: all.len(),
    });
    out
}

/// The result of a cluster bootstrap.
#[derive(Debug, Clone, PartialEq)]
pub struct Interval {
    /// The point estimate on the observed sample.
    pub point: f64,
    /// The lower bound of the interval (5th percentile).
    pub lo: f64,
    /// The upper bound of the interval (95th percentile).
    pub hi: f64,
    /// The number of bootstrap resamples.
    pub resamples: usize,
    /// The number of lineages resampled.
    pub clusters: usize,
    /// The number of paired tasks behind the point estimate.
    pub tasks: usize,
}

impl Interval {
    /// Whether the interval spans zero, i.e. the difference is not
    /// distinguishable from no difference at this confidence.
    pub fn spans_zero(&self) -> bool {
        self.lo <= 0.0 && self.hi >= 0.0
    }

    /// Whether the interval is defined at all (D5).
    ///
    /// The cluster bootstrap resamples *lineages*. With fewer than two
    /// lineages every resample draws the same single lineage, so the bootstrap
    /// distribution is one point and the interval has zero width — a
    /// degenerate interval that must not be read as "clear". An interval is
    /// defined only when there are at least two lineages to resample, at least
    /// one resample, and at least one paired task behind the point estimate.
    pub fn is_defined(&self) -> bool {
        self.clusters >= 2 && self.resamples > 0 && self.tasks > 0
    }
}

/// The mean of a slice, or 0.0 for an empty slice.
fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.iter().sum::<f64>() / xs.len() as f64
}

/// The `q`-quantile (0..=1) of `xs` by linear interpolation between order
/// statistics. `xs` is sorted in place.
fn quantile(xs: &mut [f64], q: f64) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    if xs.len() == 1 {
        return xs[0];
    }
    let pos = q * (xs.len() - 1) as f64;
    let lo = pos.floor() as usize;
    let hi = pos.ceil() as usize;
    if lo == hi {
        return xs[lo];
    }
    let frac = pos - lo as f64;
    xs[lo] + (xs[hi] - xs[lo]) * frac
}

/// Cluster bootstrap of the mean paired difference over lineages.
///
/// `clusters` is the per-lineage list of paired per-task differences. We draw
/// `clusters.len()` lineages with replacement `resamples` times, pool their
/// task differences, and take the mean. The 90% interval is the 5th and 95th
/// percentiles of that distribution.
pub fn cluster_bootstrap(clusters: &[Cluster], seed: u64, resamples: usize) -> Interval {
    let all: Vec<f64> = clusters
        .iter()
        .flat_map(|c| c.diffs.iter().copied())
        .collect();
    let point = mean(&all);

    if clusters.is_empty() || all.is_empty() || resamples == 0 {
        return Interval {
            point,
            lo: point,
            hi: point,
            resamples,
            clusters: clusters.len(),
            tasks: all.len(),
        };
    }

    let mut rng = SplitMix64::new(seed);
    let mut stats = Vec::with_capacity(resamples);
    for _ in 0..resamples {
        let mut pooled: Vec<f64> = Vec::new();
        for _ in 0..clusters.len() {
            let c = &clusters[rng.below(clusters.len())];
            pooled.extend_from_slice(&c.diffs);
        }
        stats.push(mean(&pooled));
    }

    let lo = quantile(&mut stats, 0.05);
    let hi = quantile(&mut stats, 0.95);
    Interval {
        point,
        lo,
        hi,
        resamples,
        clusters: clusters.len(),
        tasks: all.len(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitmix_is_deterministic() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..8 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        let mut c = SplitMix64::new(43);
        assert_ne!(SplitMix64::new(42).next_u64(), c.next_u64());
    }

    #[test]
    fn interval_spans_zero_when_the_difference_is_noise() {
        // Two lineages pull opposite ways; the pooled mean is ~0 and the
        // interval must straddle it.
        let clusters = vec![
            Cluster {
                lineage: "a".into(),
                diffs: vec![1.0, 1.0, 1.0],
            },
            Cluster {
                lineage: "b".into(),
                diffs: vec![-1.0, -1.0, -1.0],
            },
        ];
        let iv = cluster_bootstrap(&clusters, 7, 2000);
        assert!(iv.spans_zero(), "expected span of zero, got {iv:?}");
    }

    #[test]
    fn interval_excludes_zero_when_every_lineage_agrees() {
        let clusters = vec![
            Cluster {
                lineage: "a".into(),
                diffs: vec![-2.0, -2.0],
            },
            Cluster {
                lineage: "b".into(),
                diffs: vec![-3.0, -3.0],
            },
            Cluster {
                lineage: "c".into(),
                diffs: vec![-2.5, -2.5],
            },
        ];
        let iv = cluster_bootstrap(&clusters, 7, 2000);
        assert!(!iv.spans_zero(), "expected a clear result, got {iv:?}");
        assert!(iv.hi < 0.0);
    }

    #[test]
    fn same_seed_same_interval() {
        let clusters = vec![
            Cluster {
                lineage: "a".into(),
                diffs: vec![0.5, -0.2, 0.1],
            },
            Cluster {
                lineage: "b".into(),
                diffs: vec![-0.4, 0.3],
            },
        ];
        let x = cluster_bootstrap(&clusters, 99, 500);
        let y = cluster_bootstrap(&clusters, 99, 500);
        assert_eq!(x, y);
    }
}
