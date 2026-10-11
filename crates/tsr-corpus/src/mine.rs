//! # `tsr corpus mine` — mined history as tasks (PRD v0.3.1, §B9, §B11 M3)
//!
//! For each commit on the repository's **first-parent** history, run the suite at the parent and at
//! the commit, and keep the tests that flipped red→green. A commit whose suite run is green at the
//! parent and green at the commit is a refactor (nothing flipped); a commit that is green at the
//! parent and red at the commit is a regression (nothing flipped *to* green). Neither is a task.
//!
//! Reading history is the one place this crate shells out to `git` directly: `git` reads the
//! repository's history and executes nothing from it. Every *suite* run goes through the
//! [`Runner`], so in production it is `tsr exec` in the sealed sibling sandbox (§B4) and in tests it
//! is the deterministic [`ScriptedRunner`]. Nothing here runs a repository's tests on the host.
//!
//! ## The suite report convention
//!
//! [`RunResult`] carries only exit codes and output digests — no raw output — so the *names* of the
//! tests that flipped cannot come from the [`Runner`]'s return value. Instead the suite invocation
//! itself writes a machine-readable report to a file **inside the snapshot**, and `mine` reads that
//! file after the run:
//!
//! - the report path is `<snapshot>/.tsr-suite-report.json`;
//! - the report is produced by the argv `mine` passes (the reporter is part of the suite argv, so
//!   `ExecRunner`'s `tsr exec -- <argv>` carries it into the sandbox), never by a host-side shell;
//! - the format is `{"tests": [{"name": "...", "passed": true|false}, ...]}`;
//! - a **missing** report is an error, never a silently empty flip set.
//!
//! The red/green gate stays on the [`Runner`]'s exit code; the report only names which tests
//! flipped. The agent is meant to see a shallow clone at the parent only; the reference diff
//! (parent → commit) is the hidden oracle and lives in the CAS under `--out`, addressed by its
//! blake3 digest. The task's [`OutcomeDef`] encodes both halves of the oracle: the flipped tests
//! pass **and** the full suite does not regress.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use tsr_domain::{Hash, Horizon, OutcomeDef, OutcomeDefId};

use crate::{digest, Limits, Runner};

/// The default suite command when `--suite` is not given.
pub const DEFAULT_SUITE: &str = "cargo test";

/// The report file the suite invocation writes inside the snapshot.
pub const REPORT_FILE: &str = ".tsr-suite-report.json";

/// The oracle's name, versioned like every other outcome definition (B7).
pub const ORACLE_NAME: &str = "flipped_tests_pass_and_suite_no_regression";
pub const ORACLE_VERSION: u32 = 1;

/// One test's verdict in a suite report.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestVerdict {
    pub name: String,
    pub passed: bool,
}

/// The suite report a run writes inside the snapshot.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SuiteReport {
    pub tests: Vec<TestVerdict>,
}

impl SuiteReport {
    /// The names that passed.
    pub fn passed(&self) -> BTreeSet<String> {
        self.tests
            .iter()
            .filter(|t| t.passed)
            .map(|t| t.name.clone())
            .collect()
    }

    /// The names that failed.
    pub fn failed(&self) -> BTreeSet<String> {
        self.tests
            .iter()
            .filter(|t| !t.passed)
            .map(|t| t.name.clone())
            .collect()
    }
}

/// One mined task: a parent snapshot, the tests that flipped red→green at the commit, and the
/// hidden reference diff that produced the flip.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MinedTask {
    /// blake3 of `(repo, parent, commit)` — stable across runs of the same history.
    pub id: String,
    /// The repository the task was mined from. Also the task's lineage (B9).
    pub repo: String,
    /// The parent commit the agent starts from (the shallow clone's tip).
    pub parent: String,
    /// The commit whose diff is the hidden reference.
    pub commit: String,
    /// Digest of the parent snapshot: the suite run's stdout digest at the parent.
    pub parent_snapshot: String,
    /// The tests that failed at the parent and passed at the commit, sorted.
    pub flipped_tests: Vec<String>,
    /// CAS digest of the reference diff (`git diff parent commit`).
    pub hidden_diff: String,
    /// Family: the crate the touched paths belong to, or a declared mapping.
    pub family: String,
    /// Lineage: the repository (B9).
    pub lineage: String,
    /// The commit's timestamp, ISO-8601 (B9: every task carries a timestamp).
    pub timestamp: String,
    /// The oracle, encoded as an outcome definition.
    pub outcome: OutcomeDef,
}

/// The result of a mine run.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MineReport {
    pub repo: String,
    pub suite: Vec<String>,
    /// Commits walked (first-parent, excluding the root).
    pub commits_walked: usize,
    /// Tasks written.
    pub tasks: usize,
    /// Commits skipped because the suite was red at the commit (a regression, not a flip).
    pub skipped_regression: usize,
    /// Commits skipped because nothing flipped (a refactor).
    pub skipped_no_flip: usize,
}

/// A declared family mapping: a path prefix to a family name. The first matching prefix wins.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FamilyRule {
    pub prefix: String,
    pub family: String,
}

/// How to mine.
#[derive(Clone, Debug)]
pub struct MineOptions {
    /// The suite command, split into argv.
    pub suite: Vec<String>,
    /// Only walk commits reachable from this revision (default: `HEAD`).
    pub since: Option<String>,
    /// Declared family mappings, checked before the crate-name heuristic.
    pub family_rules: Vec<FamilyRule>,
    /// Run limits handed to the [`Runner`].
    pub limits: Limits,
}

impl Default for MineOptions {
    fn default() -> Self {
        Self {
            suite: DEFAULT_SUITE
                .split_whitespace()
                .map(str::to_string)
                .collect(),
            since: None,
            family_rules: Vec::new(),
            limits: Limits::default(),
        }
    }
}

/// The oracle: the flipped tests pass and the full suite does not regress.
pub fn oracle() -> OutcomeDef {
    let name = ORACLE_NAME.to_string();
    let version = ORACLE_VERSION;
    let resolves_true_when =
        "every flipped test passes at the commit AND the full suite exits 0 at the commit"
            .to_string();
    let censored_when =
        "the suite could not be run at the parent or at the commit (no sandbox, timeout)"
            .to_string();
    let intervention_rule =
        "the agent sees a shallow clone at the parent only; the reference diff stays hidden"
            .to_string();
    let payload = serde_json::to_vec(&(
        &name,
        version,
        &resolves_true_when,
        &censored_when,
        &intervention_rule,
    ))
    .expect("serializable");
    OutcomeDef {
        id: OutcomeDefId(Hash::of(&[&payload])),
        name,
        version,
        horizon: Horizon::TaskClose,
        resolves_true_when,
        censored_when,
        intervention_rule,
    }
}

/// The task id: blake3 of `(repo, parent, commit)`.
pub fn task_id(repo: &str, parent: &str, commit: &str) -> String {
    let payload = serde_json::to_vec(&(repo, parent, commit)).expect("serializable");
    digest(&payload)
}

/// Mine `repo` into `out`, writing `tasks.jsonl` and the CAS.
pub fn mine(
    repo: &Path,
    out: &Path,
    opts: &MineOptions,
    runner: &dyn Runner,
) -> Result<MineReport, MineError> {
    let repo_str = repo.display().to_string();
    let commits = first_parent_commits(repo, opts.since.as_deref())?;
    fs::create_dir_all(out).map_err(MineError::Io)?;
    let cas = out.join("cas");
    fs::create_dir_all(&cas).map_err(MineError::Io)?;
    // The materialised checkouts live under the output directory, one per revision.
    let snapshots = out.join("snapshots");
    fs::create_dir_all(&snapshots).map_err(MineError::Io)?;

    let mut report = MineReport {
        repo: repo_str.clone(),
        suite: opts.suite.clone(),
        commits_walked: commits.len(),
        tasks: 0,
        skipped_regression: 0,
        skipped_no_flip: 0,
    };
    let mut lines: Vec<String> = Vec::new();

    for (parent, commit) in &commits {
        // Materialise the parent and the commit as separate checkouts under the output directory,
        // so the suite at each revision runs against the tree it claims (D4). Running both in the
        // repository itself would run the same tree twice and make the two runs indistinguishable.
        let parent_snap = materialize(repo, parent, &snapshots.join(parent))?;
        let commit_snap = materialize(repo, commit, &snapshots.join(commit))?;

        // The suite at the parent and at the commit, through the Runner, each in its own checkout.
        // The report file is read from the snapshot after each run; a missing report is an error.
        let _parent_run = runner.run(&parent_snap, &opts.suite, &opts.limits);
        let parent_report = read_report(&parent_snap)?;
        let commit_run = runner.run(&commit_snap, &opts.suite, &opts.limits);
        let commit_report = read_report(&commit_snap)?;

        // The flip is read from the reports: a test that failed at the parent and passed at the
        // commit. The commit must be green (the oracle's second half); a commit that is red is a
        // regression, not a flip.
        let flipped: Vec<String> = parent_report
            .failed()
            .intersection(&commit_report.passed())
            .cloned()
            .collect();

        if !commit_run.succeeded() {
            // Red at the commit: a regression, not a red→green flip.
            report.skipped_regression += 1;
            continue;
        }
        if flipped.is_empty() {
            // Green at both ends and nothing flipped: a refactor.
            report.skipped_no_flip += 1;
            continue;
        }

        let diff = git(repo, &["diff", parent, commit])?;
        let hidden_diff = digest(diff.as_bytes());
        write_cas(&cas, &hidden_diff, diff.as_bytes())?;

        let touched = touched_paths(repo, parent, commit)?;
        let family = family_of(&touched, &opts.family_rules);
        let timestamp = git(repo, &["show", "-s", "--format=%cI", commit])?
            .trim()
            .to_string();

        let task = MinedTask {
            id: task_id(&repo_str, parent, commit),
            repo: repo_str.clone(),
            parent: parent.clone(),
            commit: commit.clone(),
            parent_snapshot: tree_id(repo, parent)?,
            flipped_tests: flipped,
            hidden_diff,
            family,
            lineage: repo_str.clone(),
            timestamp,
            outcome: oracle(),
        };
        lines.push(serde_json::to_string(&task).map_err(MineError::Json)?);
        report.tasks += 1;
    }

    let mut body = lines.join("\n");
    if !body.is_empty() {
        body.push('\n');
    }
    fs::write(out.join("tasks.jsonl"), body).map_err(MineError::Io)?;
    Ok(report)
}

/// Read the suite report the run wrote inside the snapshot. A missing report is an error.
pub fn read_report(snapshot: &Path) -> Result<SuiteReport, MineError> {
    let path = snapshot.join(REPORT_FILE);
    let bytes = fs::read(&path).map_err(|e| {
        MineError::MissingReport(format!(
            "no suite report at {}: {e} — the suite argv must write {REPORT_FILE} inside the snapshot",
            path.display()
        ))
    })?;
    serde_json::from_slice(&bytes).map_err(MineError::Json)
}

/// The first-parent history as `(parent, commit)` pairs, oldest first, excluding the root commit
/// (which has no parent). `since` limits the walk to commits reachable from that revision.
pub fn first_parent_commits(
    repo: &Path,
    since: Option<&str>,
) -> Result<Vec<(String, String)>, MineError> {
    let rev = since.unwrap_or("HEAD");
    let out = git(repo, &["rev-list", "--first-parent", rev])?;
    let commits: Vec<String> = out
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect();
    // `rev-list` is newest-first; reverse to oldest-first so a task's parent precedes it.
    let mut pairs = Vec::new();
    for pair in commits.windows(2) {
        let commit = &pair[0];
        let parent = &pair[1];
        pairs.push((parent.clone(), commit.clone()));
    }
    pairs.reverse();
    Ok(pairs)
}

/// The paths a commit touched, relative to the repository root.
pub fn touched_paths(repo: &Path, parent: &str, commit: &str) -> Result<Vec<String>, MineError> {
    let out = git(repo, &["diff", "--name-only", parent, commit])?;
    Ok(out
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_string)
        .collect())
}

/// The family of a task: the first declared rule whose prefix matches a touched path, else the
/// crate name of the first touched path under `crates/<name>/`, else `"root"`.
pub fn family_of(touched: &[String], rules: &[FamilyRule]) -> String {
    for rule in rules {
        if touched.iter().any(|p| p.starts_with(&rule.prefix)) {
            return rule.family.clone();
        }
    }
    for path in touched {
        if let Some(rest) = path.strip_prefix("crates/") {
            if let Some((name, _)) = rest.split_once('/') {
                return name.to_string();
            }
        }
    }
    "root".to_string()
}

/// Write a CAS object, addressed by its digest. Idempotent: the same digest is the same bytes.
pub(crate) fn write_cas(cas: &Path, digest_hex: &str, bytes: &[u8]) -> Result<(), MineError> {
    let path = cas.join(digest_hex);
    if path.exists() {
        return Ok(());
    }
    fs::write(path, bytes).map_err(MineError::Io)
}

/// Materialise `sha` as a checkout at `dest` and return `dest`.
///
/// The checkout is a `git worktree` of the repository, detached at `sha`, so the suite run against
/// it sees exactly the tree that revision names (D4). `git` reads history and writes the working
/// tree; it executes nothing from the repository. An existing checkout at `dest` is reused.
pub fn materialize(repo: &Path, sha: &str, dest: &Path) -> Result<PathBuf, MineError> {
    if dest.exists() {
        return Ok(dest.to_path_buf());
    }
    if let Some(parent) = dest.parent() {
        fs::create_dir_all(parent).map_err(MineError::Io)?;
    }
    let dest_str = dest.display().to_string();
    git(repo, &["worktree", "add", "--detach", &dest_str, sha])?;
    Ok(dest.to_path_buf())
}

/// The tree id of a commit: `git rev-parse <sha>^{tree}`. This is what a task's `parent_snapshot`
/// records — the parent commit's tree, not the digest of a suite run's stdout (D4).
pub fn tree_id(repo: &Path, sha: &str) -> Result<String, MineError> {
    Ok(git(repo, &["rev-parse", &format!("{sha}^{{tree}}")])?
        .trim()
        .to_string())
}

/// Run `git -C <repo> <args>` and return stdout as UTF-8. `git` reads history; it executes nothing
/// from the repository.
fn git(repo: &Path, args: &[&str]) -> Result<String, MineError> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .map_err(|e| MineError::Git(format!("could not run git: {e}")))?;
    if !output.status.success() {
        return Err(MineError::Git(format!(
            "git {} failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Errors from a mine run.
#[derive(Debug, thiserror::Error)]
pub enum MineError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("git: {0}")]
    Git(String),
    #[error("missing suite report: {0}")]
    MissingReport(String),
}

/// The path of the CAS directory under an output directory.
pub fn cas_dir(out: &Path) -> PathBuf {
    out.join("cas")
}
