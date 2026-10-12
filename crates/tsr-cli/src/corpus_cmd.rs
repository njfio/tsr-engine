//! `tsr corpus` — corpus tooling (PRD v0.3.1, §B9, §B11 M3).
//!
//! `mine` walks a repository's first-parent history and writes the red→green flip tasks. The suite
//! runs through the [`ExecRunner`], which invokes `tsr exec -- <argv>` in the sealed sibling
//! sandbox; nothing here runs a repository's tests on the host.

use std::path::PathBuf;
use std::process::ExitCode;

use tsr_corpus::mine::{mine, MineOptions};
use tsr_corpus::mutants::{mutants, MutantsOptions};
use tsr_corpus::ExecRunner;
use tsr_corpus::Runner;

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

/// `tsr corpus mutants <repo> --out <dir>`.
///
/// Drives `cargo mutants --list --json` and the kill run through the [`ExecRunner`], which invokes
/// `tsr exec -- <argv>` in the sealed sibling sandbox; nothing here runs a repository's tests on
/// the host.
pub fn mutants_cmd(repo: String, out: PathBuf) -> ExitCode {
    let runner = ExecRunner::new();
    match mutants(
        std::path::Path::new(&repo),
        &out,
        &MutantsOptions::default(),
        &runner,
    ) {
        Ok(report) => {
            println!(
                "mutants {}: {} listed, {} run, {} tasks, {} not caught",
                report.repo,
                report.mutants_listed,
                report.mutants_run,
                report.tasks,
                report.skipped_not_caught
            );
            if report.skipped_unsafe_descriptor > 0 {
                println!(
                    "warning: {} caught mutant(s) had an untrusted descriptor file (absolute or \
                     containing `..`); their source line was not read and the reversal diff falls \
                     back to the descriptor text",
                    report.skipped_unsafe_descriptor
                );
            }
            println!("wrote {}", out.join("tasks.jsonl").display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("tsr: corpus mutants failed: {e}");
            ExitCode::from(1)
        }
    }
}

/// `tsr corpus materialize --snapshot <sha> --out <dest> [--repo <dir>]`.
///
/// Materialises a snapshot into a disposable copy. The batch runner provisions the copy it drives a
/// session in by asking the sealed sandbox to run this command, so the argv it builds names a real
/// subcommand (D4). The checkout is a `git worktree` detached at the snapshot; `git` reads history
/// and writes the working tree, it executes nothing from the repository.
pub fn materialize_cmd(repo: Option<&std::path::Path>, snapshot: &str, out: &std::path::Path) -> ExitCode {
    let repo = repo.unwrap_or_else(|| std::path::Path::new("."));
    match tsr_corpus::mine::materialize(repo, snapshot, out) {
        Ok(dest) => {
            println!("materialized {snapshot} at {}", dest.display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("tsr: corpus materialize failed: {e}");
            ExitCode::from(1)
        }
    }
}

/// `tsr corpus oracle --snapshot <dir> --tests <csv> | --full-suite [--suite <cmd>]`.
///
/// Runs the oracle in a materialised copy and reports pass/fail: the named tests (the flipped tests)
/// or the full suite (the regression check). The batch runner runs the oracle by asking the sealed
/// sandbox to run this command, so the argv it builds names a real subcommand (D4). The suite runs
/// through the [`ExecRunner`], which invokes `tsr exec -- <argv>` in the sealed sibling sandbox;
/// nothing here runs a repository's tests on the host.
pub fn oracle_cmd(
    snapshot: &std::path::Path,
    tests: Option<&str>,
    full_suite: bool,
    suite: Option<&str>,
) -> ExitCode {
    let suite_argv: Vec<String> = suite
        .map(|s| s.split_whitespace().map(str::to_string).collect())
        .unwrap_or_else(|| vec!["cargo".to_string(), "test".to_string()]);
    let mut argv = suite_argv;
    if let Some(tests) = tests {
        for t in tests.split(',').filter(|t| !t.is_empty()) {
            argv.push(t.to_string());
        }
    } else if !full_suite {
        eprintln!("tsr: corpus oracle needs --tests <csv> or --full-suite");
        return ExitCode::from(2);
    }

    let runner = ExecRunner::new();
    let result = runner.run(snapshot, &argv, &tsr_corpus::Limits::new(600_000, 1 << 20));
    if result.succeeded() {
        println!("oracle: pass (exit 0)");
        ExitCode::SUCCESS
    } else {
        println!("oracle: fail (exit {})", result.exit);
        ExitCode::from(1)
    }
}

/// `tsr corpus split <dir> --by time|repo --holdout <fraction> --seed <n>`.
///
/// Reads `tasks.jsonl` and the CAS under `dir`, groups near-clones, assigns sides under the rule,
/// runs the three leakage probes, and writes `<dir>/split.json`. A caught probe is a refusal: the
/// manifest is written with the verdicts and the command exits non-zero.
pub fn split_cmd(dir: PathBuf, by: String, holdout: f64, seed: u64) -> ExitCode {
    let rule = match tsr_corpus::split::SplitRule::parse(&by) {
        Ok(rule) => rule,
        Err(e) => {
            eprintln!("tsr: corpus split failed: {e}");
            return ExitCode::from(2);
        }
    };
    let opts = tsr_corpus::split::SplitOptions {
        rule,
        holdout,
        seed,
        command: format!(
            "tsr corpus split {} --by {by} --holdout {holdout} --seed {seed}",
            dir.display()
        ),
        ..Default::default()
    };
    match tsr_corpus::split::split(&dir, &opts) {
        Ok(manifest) => {
            println!(
                "split by {}: {} train, {} holdout, {} clone groups, {} holdout accesses",
                manifest.rule,
                manifest.train.len(),
                manifest.holdout.len(),
                manifest.clone_groups.len(),
                manifest.holdout_access.len()
            );
            println!("wrote {}", dir.join("split.json").display());
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("tsr: corpus split failed: {e}");
            ExitCode::from(1)
        }
    }
}
