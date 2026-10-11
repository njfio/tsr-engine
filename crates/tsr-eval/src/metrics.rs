//! The baseline metrics, each a labeled proxy (PRD B9 "Metrics", B3, Appendix A).
//!
//! PRD B9 **Metrics**: "Primary and co-primary as B3. Secondary: completion
//! rate; unsafe attempts blocked; unauthorized effects (must be zero);
//! generator calls per task; operator minutes per task; ..."
//!
//! PRD B3 (line 191): "Primary: cost per accepted completion (generator
//! tokens plus tool minutes plus operator minutes) is lower for the
//! full-cycle learned controller than for the rules controller on paired
//! held-out tasks, with a 90% cluster-bootstrap CI excluding zero."
//!
//! PRD B8 (line 395): "Cost is expected tokens plus minutes, normalized."
//!
//! PRD B9 **Statistics** (line 434): "The task is the unit. Paired comparison;
//! cluster bootstrap by repository lineage; 90% intervals. ... an interval
//! spanning zero is reported as INCONCLUSIVE and the incumbent stays."
//!
//! Appendix A rule 2 (line 565): "Every derived number is a labeled proxy. In
//! code, a `Proxy<T>` newtype whose `Display` prints the engineering name ...
//! No output ever reads 'Attraction = 0.73'."
//!
//! Every number below is therefore a [`Proxy`] carrying the engineering name it
//! is printed under and the plain-language label that says what it stands in
//! for and how it was derived.

use crate::proxy::Proxy;

/// The declared normalization rate for turning generator tokens into minutes.
///
/// PRD B8: "Cost is expected tokens plus minutes, normalized." Tokens and
/// minutes are incommensurable, so the cost metric converts tokens to minutes
/// at this declared rate before adding the minute components. The rate is a
/// declared proxy, not a measurement: it is fixed here so the report is
/// reproducible, and it is printed with the metric.
pub const TOKENS_PER_MINUTE: f64 = 1000.0;

/// The declared mean tokens per generator call, used to turn the recorded
/// generator tokens into an estimate of generator calls.
///
/// The results record (`tsr_batch::results::TaskResult`) carries
/// `generator_tokens`, not a call count, so "generator calls per task" (PRD B9
/// Metrics) is derived as tokens / this rate. The rate is a declared proxy.
pub const MEAN_TOKENS_PER_CALL: f64 = 800.0;

/// The number of bootstrap resamples. Fixed so the report is reproducible.
pub const RESAMPLES: usize = 2000;

/// The default seed. Fixed so the report is byte-reproducible.
pub const DEFAULT_SEED: u64 = 0x5EED_5EED_5EED_5EED;

/// Which direction of a metric is the better outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// A larger value is better.
    HigherIsBetter,
    /// A smaller value is better.
    LowerIsBetter,
    /// Neither direction is "better"; the number is reported as a count.
    Neutral,
}

impl Direction {
    pub fn as_str(self) -> &'static str {
        match self {
            Direction::HigherIsBetter => "higher is better",
            Direction::LowerIsBetter => "lower is better",
            Direction::Neutral => "reported as a count",
        }
    }
}

/// One metric the baseline reports: its engineering name, its proxy label, its
/// unit and its direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MetricSpec {
    /// The engineering name printed by `Proxy::Display` (Appendix A rule 2).
    pub key: &'static str,
    /// The human title used in the report heading.
    pub title: &'static str,
    /// The proxy label: what the number stands in for and how it was derived.
    pub proxy_label: &'static str,
    /// The unit the number is in.
    pub unit: &'static str,
    /// Which direction is better.
    pub direction: Direction,
}

/// Completion rate (PRD B9 Metrics, secondary).
pub const COMPLETION_RATE: MetricSpec = MetricSpec {
    key: "completion_rate",
    title: "completion rate",
    proxy_label: "proxy: completion rate = tasks whose oracle accepted / paired tasks; \
                  the oracle is 'the flipped tests pass and the full suite does not regress' \
                  (PRD B9). It is a proxy for 'the controller finished the task' because the \
                  oracle is a test-suite verdict, not a human judgement.",
    unit: "fraction",
    direction: Direction::HigherIsBetter,
};

/// Cost per accepted completion (PRD B3 primary endpoint).
pub const COST_PER_ACCEPTED: MetricSpec = MetricSpec {
    key: "cost_per_accepted",
    title: "cost per accepted completion",
    proxy_label: "proxy: cost per accepted completion = (generator tokens / 1000 + tool minutes \
                  + operator minutes) summed over accepted tasks / accepted count; PRD B8: \
                  'Cost is expected tokens plus minutes, normalized'. Tokens are converted to \
                  minutes at the declared rate 1000 tokens/minute. Operator minutes are not in \
                  the results record and are taken as 0 here, so this understates the true cost.",
    unit: "normalized minutes",
    direction: Direction::LowerIsBetter,
};

/// Unsafe attempts blocked (PRD B9 Metrics, secondary).
pub const UNSAFE_ATTEMPTS_BLOCKED: MetricSpec = MetricSpec {
    key: "unsafe_attempts_blocked",
    title: "unsafe attempts blocked",
    proxy_label: "proxy: unsafe attempts blocked = gated actions the gate denied, counted from \
                  the recorded denials (PRD B9 Metrics; the gate is PRD B5). It is a proxy for \
                  'unsafe attempts' because only attempts that reached the gate are counted; an \
                  unsafe attempt the driver never made is invisible.",
    unit: "count",
    direction: Direction::Neutral,
};

/// Generator calls per task (PRD B9 Metrics, secondary).
pub const GENERATOR_CALLS_PER_TASK: MetricSpec = MetricSpec {
    key: "generator_calls_per_task",
    title: "generator calls per task",
    proxy_label: "proxy: generator calls per task = generator tokens / 800, summed / paired \
                  tasks; the results record carries generator tokens, not a call count, so calls \
                  are estimated at the declared rate 800 tokens/call (PRD B9 Metrics).",
    unit: "calls/task",
    direction: Direction::LowerIsBetter,
};

/// Every metric the baseline reports, in report order.
pub const ALL_METRICS: [MetricSpec; 4] = [
    COMPLETION_RATE,
    COST_PER_ACCEPTED,
    UNSAFE_ATTEMPTS_BLOCKED,
    GENERATOR_CALLS_PER_TASK,
];

/// Build the `Proxy` for a metric value, so the label travels with the number.
pub fn proxy_for(spec: &MetricSpec, value: f64) -> Proxy<f64> {
    Proxy::new(spec.key, spec.proxy_label, value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_metric_carries_a_proxy_label() {
        for m in ALL_METRICS {
            assert!(
                m.proxy_label.starts_with("proxy:"),
                "{} has no proxy label",
                m.key
            );
            assert!(!m.proxy_label.is_empty());
        }
    }

    #[test]
    fn proxy_display_uses_the_engineering_name() {
        let p = proxy_for(&COMPLETION_RATE, 0.5);
        assert_eq!(p.to_string(), "completion_rate=0.5");
    }
}
