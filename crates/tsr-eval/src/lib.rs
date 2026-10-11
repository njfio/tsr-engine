//! `tsr eval baseline` — the first numbers with intervals (PRD B9, B10,
//! Appendix A).
//!
//! This crate reads the per-task results `tsr task run` wrote for both systems
//! (system 1 record, system 2 rules), pairs them by task, and reports the
//! metrics PRD B9 names with 90% cluster-bootstrap intervals over repository
//! lineage. An interval that spans zero is reported as INCONCLUSIVE and the
//! incumbent stays (PRD B9 Statistics).
//!
//! Every number is a labeled proxy (Appendix A rule 2): see [`proxy::Proxy`]
//! and [`metrics`].

pub mod baseline;
pub mod bootstrap;
pub mod metrics;
pub mod proxy;
pub mod results;

pub use baseline::{baseline, MetricResult, PairBy, Report};
pub use bootstrap::{cluster_bootstrap, Cluster, Interval, SplitMix64};
pub use metrics::{
    MetricSpec, ALL_METRICS, COMPLETION_RATE, COST_PER_ACCEPTED, DEFAULT_SEED,
    GENERATOR_CALLS_PER_TASK, MEAN_TOKENS_PER_CALL, RESAMPLES, TOKENS_PER_MINUTE,
    UNSAFE_ATTEMPTS_BLOCKED,
};
pub use proxy::Proxy;
pub use results::{is_accepted, ResultsDir, SystemResults};

use std::path::PathBuf;

/// What can go wrong reading results or computing the baseline.
#[derive(Debug, thiserror::Error)]
pub enum EvalError {
    /// The results directory has no `results.jsonl`.
    #[error("no results.jsonl in {0}")]
    MissingResults(PathBuf),
    /// A file could not be read or parsed.
    #[error("reading {path}: {detail}")]
    Io {
        /// The file that failed.
        path: PathBuf,
        /// The underlying error.
        detail: String,
    },
    /// A result named a system other than `record` or `rules`.
    #[error("unknown system {system:?} in {path} (expected record or rules)")]
    UnknownSystem {
        /// The system name found.
        system: String,
        /// The file it was found in.
        path: PathBuf,
    },
    /// A system reported the same task twice.
    #[error("duplicate task {task:?} for system {system:?} in {path}")]
    DuplicateTask {
        /// The system.
        system: String,
        /// The task id.
        task: String,
        /// The file.
        path: PathBuf,
    },
    /// `--pair-by` named a rule this build does not define.
    #[error("unknown --pair-by {0:?} (only `task` is defined)")]
    UnknownPairBy(String),
}

/// The default number of bootstrap resamples.
pub const DEFAULT_RESAMPLES: usize = RESAMPLES;

/// Run the baseline over a results directory with the default seed and
/// resample count.
pub fn baseline_default(dir: &std::path::Path) -> Result<Report, EvalError> {
    baseline(dir, PairBy::Task, DEFAULT_SEED, DEFAULT_RESAMPLES)
}
