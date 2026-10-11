//! `tsr eval baseline` — the CLI arm (PRD B9, B10, Appendix A).
//!
//! Reads the per-task results for both systems from `<results-dir>/results.jsonl`,
//! pairs them by task, and prints the metrics with 90% cluster-bootstrap
//! intervals over repository lineage. An interval spanning zero prints
//! INCONCLUSIVE (PRD B9 Statistics).

use std::path::Path;
use std::process::ExitCode;

use tsr_eval::{baseline, PairBy};

/// Run `tsr eval baseline <results-dir> [--pair-by task] [--seed <n>]`.
pub fn baseline_cmd(results_dir: &Path, pair_by: &str, seed: u64) -> ExitCode {
    let pair_by = match PairBy::parse(pair_by) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("tsr eval baseline: {e}");
            return ExitCode::from(2);
        }
    };

    match baseline(results_dir, pair_by, seed, tsr_eval::DEFAULT_RESAMPLES) {
        Ok(report) => {
            print!("{}", report.render());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("tsr eval baseline: {e}");
            ExitCode::from(2)
        }
    }
}
