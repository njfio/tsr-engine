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
//! The default Cargo suite is parsed from the Runner's captured stdout, outside the
//! candidate workspace. Missing test evidence and duplicate identities are errors.
//! Custom suites retain the explicit `.tsr-suite-report.json` adapter contract;
//! those reports are untrusted observations, never independent acceptance evidence.
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preparation_id: Option<String>,
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
    /// The repository paths of the files that define this task's oracle tests (D3, item 2): the test
    /// files the commit's diff touched, and the files whose `#[test]` functions changed. The oracle
    /// restores these from the snapshot before it judges the copy, so a session cannot pass by
    /// deleting, editing or renaming its test.
    #[serde(default)]
    pub oracle_test_files: Vec<String>,
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

        runner
            .prepare(repo, parent, &parent_snap, out)
            .map_err(MineError::MissingReport)?;
        runner
            .prepare(repo, commit, &commit_snap, out)
            .map_err(MineError::MissingReport)?;

        // The suite at the parent and at the commit, through the Runner, each in its own checkout.
        // The report file is read from the snapshot after each run; a missing report is an error.
        let parent_run = runner.run(&parent_snap, &opts.suite, &opts.limits);
        crate::persist_run(
            out,
            parent,
            &opts.suite,
            &parent_run,
            runner.preparation_id(),
        )
        .map_err(MineError::MissingReport)?;
        let default_cargo = opts.suite == vec!["cargo".to_string(), "test".to_string()];
        let parent_report = if default_cargo {
            require_cargo_completion(&parent_run)?;
            cargo_report(&parent_run.stdout)?
        } else {
            read_report(&parent_snap)?
        };
        let commit_run = runner.run(&commit_snap, &opts.suite, &opts.limits);
        crate::persist_run(
            out,
            commit,
            &opts.suite,
            &commit_run,
            runner.preparation_id(),
        )
        .map_err(MineError::MissingReport)?;
        let commit_report = if default_cargo {
            require_cargo_completion(&commit_run)?;
            cargo_report(&commit_run.stdout)?
        } else {
            read_report(&commit_snap)?
        };

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
            preparation_id: runner.preparation_id().map(str::to_string),
            id: crate::preparation::task_id(
                task_id(
                    runner.repository_identity().unwrap_or(&repo_str),
                    parent,
                    commit,
                ),
                runner.preparation_id(),
            ),
            repo: repo_str.clone(),
            parent: parent.clone(),
            commit: commit.clone(),
            parent_snapshot: tree_id(repo, parent)?,
            oracle_test_files: oracle_test_files(repo, parent, commit, &touched, &flipped)?,
            flipped_tests: flipped,
            hidden_diff,
            family,
            lineage: runner
                .repository_identity()
                .unwrap_or(&repo_str)
                .to_string(),
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

fn require_cargo_completion(run: &crate::RunResult) -> Result<(), MineError> {
    // Cargo test reports ordinary assertion failures with 101. Worker timeout,
    // truncation and launch failures are not evidence that the parent tests failed.
    if !matches!(run.exit, 0 | 101) {
        return Err(MineError::MissingReport(format!(
            "Cargo execution incomplete (exit {})",
            run.exit
        )));
    }
    Ok(())
}

/// Parse captured libtest output without reading candidate-controlled report files.
/// Unsupported/truncated output is refused; duplicate names across targets need an
/// explicitly target-qualified adapter before they can identify an oracle uniquely.
pub fn cargo_report(stdout: &[u8]) -> Result<SuiteReport, MineError> {
    let text = std::str::from_utf8(stdout)
        .map_err(|_| MineError::MissingReport("non-UTF8 Cargo output".into()))?;
    let mut tests = Vec::new();
    let mut seen = BTreeSet::new();
    let mut summary = false;
    for line in text.lines().map(str::trim) {
        if line.starts_with("test result:") {
            summary = true;
        }
        let Some(line) = line.strip_prefix("test ") else {
            continue;
        };
        let Some((name, status)) = line.rsplit_once(" ... ") else {
            continue;
        };
        let passed = match status {
            "ok" => true,
            "FAILED" => false,
            _ => continue,
        };
        if name.is_empty() || !seen.insert(name.to_string()) {
            return Err(MineError::MissingReport(
                "ambiguous Cargo test identity".into(),
            ));
        }
        tests.push(TestVerdict {
            name: name.into(),
            passed,
        });
    }
    if tests.is_empty() || !summary {
        return Err(MineError::MissingReport(
            "Cargo output lacks completed test evidence".into(),
        ));
    }
    Ok(SuiteReport { tests })
}

#[cfg(test)]
mod captured_report_tests {
    use super::*;
    #[test]
    fn worker_failure_cannot_be_mined_as_a_red_test() {
        for exit in [124, 125, 127, -1] {
            let run = crate::RunResult::from_output(
                exit,
                b"test a ... FAILED\ntest result: FAILED.\n",
                b"",
                0,
            );
            assert!(require_cargo_completion(&run).is_err());
        }
        assert!(require_cargo_completion(&crate::RunResult::exit_only(101)).is_ok());
    }
    #[test]
    fn red_green_names_come_from_captured_output() {
        let red = cargo_report(b"test tests::check ... FAILED\ntest result: FAILED.\n").unwrap();
        let green = cargo_report(b"test tests::check ... ok\ntest result: ok.\n").unwrap();
        assert_eq!(red.failed().intersection(&green.passed()).count(), 1);
    }
    #[test]
    fn incomplete_ambiguous_and_empty_reports_are_refused() {
        for text in [
            "",
            "test result: ok. 0 passed",
            "test a ... ok",
            "test a ... ok\ntest a ... ok\ntest result: ok.",
        ] {
            assert!(cargo_report(text.as_bytes()).is_err());
        }
    }
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

/// The repository paths of the files that define a task's oracle tests (D3, item 2).
///
/// A mined task's oracle is the flipped test, so the files that define it are the test files the
/// commit's diff touched — a path under a `tests/` directory, a `tests.rs`, or a file whose name
/// ends in `_test.rs`/`_tests.rs` — together with any touched file whose diff adds or changes a
/// `#[test]` function. The list is sorted and deduplicated; a task with none is refused at load.
pub fn oracle_test_files(
    repo: &Path,
    parent: &str,
    commit: &str,
    touched: &[String],
    flipped_tests: &[String],
) -> Result<Vec<String>, MineError> {
    let mut files: Vec<String> = Vec::new();
    // (a) The files that hold the flipped tests. The flip commit often changes only the source, so
    //     the test file is not in the diff; it is found by the test's function name in the parent
    //     tree, which is the tree the copy is provisioned from.
    for test in flipped_tests {
        let Some(name) = test.rsplit("::").next() else {
            continue;
        };
        if name.is_empty() {
            continue;
        }
        let needle = format!("fn {name}");
        let out = git(
            repo,
            &["grep", "-l", "-F", "-e", &needle, parent, "--", "*.rs"],
        )?;
        for line in out.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            // `git grep <rev>` prefixes each hit with `<rev>:`, which is stripped here.
            let path = line.split_once(':').map(|(_, p)| p).unwrap_or(line);
            files.push(path.to_string());
        }
    }
    // (b) The test files the commit's diff touched, and any touched file whose diff adds or changes
    //     a `#[test]` function.
    for path in touched {
        if is_test_path(path) || diff_has_test(repo, parent, commit, path)? {
            files.push(path.clone());
        }
    }
    files.sort();
    files.dedup();
    Ok(files)
}

/// Whether a repository path names a test file by convention.
fn is_test_path(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    path.contains("/tests/")
        || path.starts_with("tests/")
        || name == "tests.rs"
        || name.ends_with("_test.rs")
        || name.ends_with("_tests.rs")
}

/// Whether the diff of `path` between `parent` and `commit` adds or changes a `#[test]` function.
///
/// The diff is read with `git diff -U0` and only its added lines are inspected: a `#[test]` that was
/// already there and untouched is not a change to the oracle.
fn diff_has_test(repo: &Path, parent: &str, commit: &str, path: &str) -> Result<bool, MineError> {
    let diff = git(repo, &["diff", "-U0", parent, commit, "--", path])?;
    Ok(diff
        .lines()
        .any(|l| l.starts_with('+') && !l.starts_with("+++") && l.contains("#[test]")))
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
pub fn write_cas(cas: &Path, digest_hex: &str, bytes: &[u8]) -> Result<(), MineError> {
    use std::io::Write;
    if digest(bytes) != digest_hex {
        return Err(MineError::Git("CAS write digest mismatch".into()));
    }
    let path = cas.join(digest_hex);
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
    {
        Ok(mut file) => {
            file.write_all(bytes)?;
            file.sync_all()?;
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            if read_cas(cas, digest_hex)? != bytes {
                return Err(MineError::Git("CAS collision".into()));
            }
            Ok(())
        }
        Err(e) => Err(MineError::Io(e)),
    }
}

/// Read and verify a complete content address; reject nonregular objects before reading.
pub fn read_cas(cas: &Path, digest_hex: &str) -> Result<Vec<u8>, MineError> {
    if digest_hex.len() != 64 || !digest_hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(MineError::Git(
            "unsafe CAS digest: expected 64 hex characters".into(),
        ));
    }
    let path = cas.join(digest_hex);
    if !fs::symlink_metadata(&path)?.is_file() {
        return Err(MineError::Git("nonregular CAS object".into()));
    }
    let bytes = fs::read(path)?;
    if digest(&bytes) != digest_hex {
        return Err(MineError::Git("CAS content digest mismatch".into()));
    }
    Ok(bytes)
}

/// Materialise `sha` as a checkout at `dest` and return `dest`.
///
/// `sha` is a git revision — a commit, a tag, a ref, **or a tree id** (D4). A task's
/// `parent_snapshot` records the parent commit's *tree* id (`git rev-parse <sha>^{tree}`), which
/// `git worktree add` refuses ("is a tree, not a commit"). `git archive <tree-ish>` accepts a tree
/// id, so the copy is written by extracting the archive into a fresh directory: the suite run
/// against it sees exactly the tree that revision names. `git` reads history and writes the working
/// tree; it executes nothing from the repository.
///
/// **The result is exactly the snapshot's tree** (D4, R1). A pre-existing `dest` is *replaced*, not
/// reused: an empty directory, or a directory a previous run's session edited, is not the snapshot,
/// and returning it would make the disposable copy neither fresh nor the snapshot (PRD §B10: the
/// disposable copy is the only writable mount). The old directory is moved aside and removed only
/// after the new tree has been extracted, so a failed extraction leaves the previous copy intact
/// rather than a half-written one. The call is therefore idempotent in the sense that matters: the
/// same `(repo, sha, dest)` always ends with `dest` holding the snapshot's tree.
pub fn materialize(repo: &Path, sha: &str, dest: &Path) -> Result<PathBuf, MineError> {
    // R1: a pre-existing `dest` is not "already materialised". Move it aside so the extraction below
    // writes a fresh copy; it is removed only once the new tree is in place.
    let stale = if dest.exists() {
        let stale = stale_path(dest);
        let _ = fs::remove_dir_all(&stale);
        fs::rename(dest, &stale).map_err(MineError::Io)?;
        Some(stale)
    } else {
        None
    };
    match extract(repo, sha, dest) {
        Ok(()) => {
            if let Some(stale) = stale {
                let _ = fs::remove_dir_all(&stale);
            }
            Ok(dest.to_path_buf())
        }
        Err(e) => {
            // The extraction failed: put the previous copy back so `dest` is not left half-written.
            let _ = fs::remove_dir_all(dest);
            if let Some(stale) = stale {
                let _ = fs::rename(&stale, dest);
            }
            Err(e)
        }
    }
}

/// The sibling path a pre-existing `dest` is moved to while a fresh copy is extracted.
fn stale_path(dest: &Path) -> PathBuf {
    let name = dest
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    dest.with_file_name(format!(".{name}.stale"))
}

/// Extract `sha`'s tree into `dest` (which must not exist yet).
fn extract(repo: &Path, sha: &str, dest: &Path) -> Result<(), MineError> {
    fs::create_dir_all(dest).map_err(MineError::Io)?;
    // `git archive <tree-ish>` accepts a tree id as well as a commit; `git worktree add` does not.
    // The archive is piped into `tar -x` in `dest`, so the copy is a plain directory with no
    // worktree registration in the operator's repository (R5).
    let mut archive = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["archive", "--format=tar", sha])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| MineError::Git(format!("could not run git archive: {e}")))?;
    let mut tar = Command::new("tar")
        .arg("-x")
        .arg("-C")
        .arg(dest)
        .stdin(archive.stdout.take().expect("piped stdout"))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| MineError::Git(format!("could not run tar: {e}")))?;
    let tar_status = tar.wait().map_err(MineError::Io)?;
    let archive_status = archive.wait().map_err(MineError::Io)?;
    if !tar_status.success() || !archive_status.success() {
        return Err(MineError::Git(format!(
            "materialising {sha} into {} failed (git archive {archive_status}, tar {tar_status})",
            dest.display()
        )));
    }
    Ok(())
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

#[cfg(test)]
mod cas_integrity_tests {
    use super::*;
    #[test]
    fn corrupt_existing_objects_and_symlinks_are_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let bytes = b"bounded mutation fixture";
        let digest = digest(bytes);
        write_cas(dir.path(), &digest, bytes).unwrap();
        assert_eq!(read_cas(dir.path(), &digest).unwrap(), bytes);
        fs::write(dir.path().join(&digest), b"different").unwrap();
        assert!(read_cas(dir.path(), &digest).is_err());
        assert!(write_cas(dir.path(), &digest, bytes).is_err());
        assert!(read_cas(dir.path(), "abcd").is_err());
        #[cfg(unix)]
        {
            fs::remove_file(dir.path().join(&digest)).unwrap();
            std::os::unix::fs::symlink("missing", dir.path().join(&digest)).unwrap();
            assert!(write_cas(dir.path(), &digest, bytes).is_err());
            assert!(read_cas(dir.path(), &digest).is_err());
        }
    }
}
