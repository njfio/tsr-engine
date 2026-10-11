//! `tsr eval baseline` — the CLI arm (PRD B9, B10, Appendix A).
//!
//! Reads the per-task results for both systems, pairs them by task, and prints
//! the metrics with 90% cluster-bootstrap intervals over repository lineage. An
//! interval spanning zero prints INCONCLUSIVE (PRD B9 Statistics).
//!
//! Two shapes are accepted:
//! - `tsr eval baseline <results-dir>` — one `results.jsonl` holding both
//!   systems (the shape `tsr task run` writes when both runs share an `--out`);
//! - `tsr eval baseline --record <dir> --rules <dir>` — one directory per run,
//!   each holding that system's `results.jsonl`.

use std::path::Path;
use std::process::ExitCode;

use tsr_eval::{baseline, baseline_pair, PairBy};

/// Run `tsr eval baseline` over either one results directory or a
/// `--record`/`--rules` pair.
pub fn baseline_cmd(
    results_dir: Option<&Path>,
    record: Option<&Path>,
    rules: Option<&Path>,
    pair_by: &str,
    seed: u64,
) -> ExitCode {
    let pair_by = match PairBy::parse(pair_by) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("tsr eval baseline: {e}");
            return ExitCode::from(2);
        }
    };

    let result = match (results_dir, record, rules) {
        (Some(dir), None, None) => baseline(dir, pair_by, seed, tsr_eval::DEFAULT_RESAMPLES),
        (None, Some(rec), Some(rul)) => {
            baseline_pair(rec, rul, pair_by, seed, tsr_eval::DEFAULT_RESAMPLES)
        }
        _ => {
            eprintln!(
                "tsr eval baseline: give either <results-dir> or --record <dir> --rules <dir>"
            );
            return ExitCode::from(2);
        }
    };

    match result {
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
