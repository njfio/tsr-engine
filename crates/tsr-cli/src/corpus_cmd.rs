//! `tsr corpus` — corpus tooling (PRD v0.3.1, §B9, §B11 M3).
//!
//! `mine` walks a repository's first-parent history and writes the red→green flip tasks. The suite
//! runs through the [`ExecRunner`], which invokes `tsr exec -- <argv>` in the sealed sibling
//! sandbox; nothing here runs a repository's tests on the host.

use std::path::PathBuf;
use std::process::ExitCode;

use tsr_corpus::mine::{mine, MineOptions};
use tsr_corpus::ExecRunner;

/// `tsr corpus mine <repo> --out <dir> [--since <rev>] [--suite <cmd>]`.
pub fn mine_cmd(
    repo: String,
    out: PathBuf,
    since: Option<String>,
    suite: Option<String>,
) -> ExitCode {
    let mut opts = MineOptions::default();
    if let Some(suite) = suite {
        opts.suite = suite.split_whitespace().map(str::to_string).collect();
    }
    opts.since = since;

    let runner = ExecRunner::new();
    match mine(std::path::Path::new(&repo), &out, &opts, &runner) {
        Ok(report) => {
            println!(
                "mined {}: {} commits walked, {} tasks, {} refactors, {} regressions",
                report.repo,
                report.commits_walked,
                report.tasks,
                report.skipped_no_flip,
                report.skipped_regression
            );
            println!("wrote {}", out.join("tasks.jsonl").display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("tsr: corpus mine failed: {e}");
            ExitCode::from(1)
        }
    }
}
