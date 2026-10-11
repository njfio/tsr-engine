//! `tsr corpus mine` against a tiny git repository built by the test (PRD §B9, §B11 M3).
//!
//! The repository has three commits after the root:
//! 1. `flip` — a test that fails at the parent and passes at the commit (a red→green flip);
//! 2. `refactor` — the suite is green at both ends and nothing flips;
//! 3. `regression` — the suite is green at the parent and red at the commit.
//!
//! `mine` must yield exactly the flip commit as a task, with the right flipped tests, family and
//! hidden-diff digest, and no task for the refactor or the regression.
//!
//! The suite is never run on the host: a [`SequenceRunner`] answers the two runs per commit in
//! order and writes the suite report the run would have written inside the snapshot.

use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tsr_corpus::mine::{
    cas_dir, first_parent_commits, mine, read_report, FamilyRule, MineOptions, SuiteReport,
    TestVerdict, REPORT_FILE,
};
use tsr_corpus::{digest, Limits, RunResult, Runner};

/// A runner that answers runs in call order and writes the report the suite would have written.
///
/// `mine` calls the runner twice per commit (parent, then commit) with the same `(snapshot, argv)`
/// key, so a keyed script cannot tell them apart; this answers by position instead. Each answer is
/// `(exit, report)`; the report is written to `<snapshot>/.tsr-suite-report.json` before returning,
/// which is exactly the convention `mine` reads.
struct SequenceRunner {
    answers: RefCell<Vec<(i32, SuiteReport)>>,
    calls: RefCell<Vec<(PathBuf, Vec<String>)>>,
}

impl SequenceRunner {
    fn new(answers: Vec<(i32, SuiteReport)>) -> Self {
        Self {
            answers: RefCell::new(answers),
            calls: RefCell::new(Vec::new()),
        }
    }
}

impl Runner for SequenceRunner {
    fn run(&self, snapshot: &Path, argv: &[String], _limits: &Limits) -> RunResult {
        self.calls
            .borrow_mut()
            .push((snapshot.to_path_buf(), argv.to_vec()));
        let (exit, report) = self.answers.borrow_mut().remove(0);
        let body = serde_json::to_vec(&report).expect("serializable");
        fs::write(snapshot.join(REPORT_FILE), &body).expect("write report");
        RunResult::from_output(exit, &body, b"", 1)
    }
}

fn report(passed: &[&str], failed: &[&str]) -> SuiteReport {
    let mut tests: Vec<TestVerdict> = Vec::new();
    for name in passed {
        tests.push(TestVerdict {
            name: (*name).to_string(),
            passed: true,
        });
    }
    for name in failed {
        tests.push(TestVerdict {
            name: (*name).to_string(),
            passed: false,
        });
    }
    SuiteReport { tests }
}

fn git(repo: &Path, args: &[&str]) {
    let status = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .env("GIT_AUTHOR_NAME", "fixture")
        .env("GIT_AUTHOR_EMAIL", "fixture@example.invalid")
        .env("GIT_COMMITTER_NAME", "fixture")
        .env("GIT_COMMITTER_EMAIL", "fixture@example.invalid")
        .status()
        .expect("git");
    assert!(status.success(), "git {args:?} failed");
}

fn write(repo: &Path, rel: &str, body: &str) {
    let path = repo.join(rel);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("mkdir");
    }
    fs::write(path, body).expect("write");
}

/// Build the fixture repository and return `(repo, root, flip, refactor, regression)`.
fn fixture_repo(dir: &Path) -> (PathBuf, String, String, String, String) {
    let repo = dir.join("repo");
    fs::create_dir_all(&repo).expect("mkdir repo");
    git(&repo, &["init", "-q", "-b", "main"]);

    // Root: the crate exists, the test is red (it asserts the wrong thing).
    write(
        &repo,
        "crates/demo/src/lib.rs",
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
    );
    write(
        &repo,
        "crates/demo/tests/demo.rs",
        "#[test]\nfn adds() { assert_eq!(demo::add(1, 1), 3); }\n",
    );
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "root: red test"]);
    let root = rev(&repo, "HEAD");

    // Commit 1 (flip): the test now passes. Red→green.
    write(
        &repo,
        "crates/demo/tests/demo.rs",
        "#[test]\nfn adds() { assert_eq!(demo::add(1, 1), 2); }\n",
    );
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "flip: fix the assertion"]);
    let flip = rev(&repo, "HEAD");

    // Commit 2 (refactor): rename a function; the suite is green at both ends, nothing flips.
    write(
        &repo,
        "crates/demo/src/lib.rs",
        "pub fn sum(a: i32, b: i32) -> i32 { a + b }\n",
    );
    write(
        &repo,
        "crates/demo/tests/demo.rs",
        "#[test]\nfn adds() { assert_eq!(demo::sum(1, 1), 2); }\n",
    );
    git(&repo, &["add", "-A"]);
    git(
        &repo,
        &["commit", "-q", "-m", "refactor: rename add to sum"],
    );
    let refactor = rev(&repo, "HEAD");

    // Commit 3 (regression): break the function; the suite is green at the parent, red at the
    // commit. Nothing flips to green.
    write(
        &repo,
        "crates/demo/src/lib.rs",
        "pub fn sum(a: i32, b: i32) -> i32 { a - b }\n",
    );
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "regression: break sum"]);
    let regression = rev(&repo, "HEAD");

    (repo, root, flip, refactor, regression)
}

fn rev(repo: &Path, r: &str) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", r])
        .output()
        .expect("git rev-parse");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

#[test]
fn mine_yields_exactly_the_flip_commit() {
    let tmp = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).expect("tempdir");
    let (repo, root, flip, refactor, regression) = fixture_repo(tmp.path());
    let out = tmp.path().join("out");

    // The suite at each revision, in the order mine walks them (oldest first):
    //   root:        adds FAILS
    //   flip:        adds passes
    //   refactor:    adds passes
    //   regression:  adds FAILS
    let runner = SequenceRunner::new(vec![
        (1, report(&[], &["adds"])), // parent = root
        (0, report(&["adds"], &[])), // commit = flip
        (0, report(&["adds"], &[])), // parent = flip
        (0, report(&["adds"], &[])), // commit = refactor
        (0, report(&["adds"], &[])), // parent = refactor
        (1, report(&[], &["adds"])), // commit = regression
    ]);

    let opts = MineOptions {
        suite: vec!["cargo".into(), "test".into()],
        since: None,
        family_rules: vec![FamilyRule {
            prefix: "crates/demo/".into(),
            family: "demo".into(),
        }],
        limits: Limits::default(),
    };
    let report_out = mine(&repo, &out, &opts, &runner).expect("mine");

    // Exactly one task: the flip commit.
    assert_eq!(report_out.tasks, 1, "one task");
    assert_eq!(report_out.commits_walked, 3, "three commits with a parent");
    assert_eq!(report_out.skipped_no_flip, 1, "the refactor");
    assert_eq!(report_out.skipped_regression, 1, "the regression");

    let body = fs::read_to_string(out.join("tasks.jsonl")).expect("tasks.jsonl");
    let tasks: Vec<serde_json::Value> = body
        .lines()
        .map(|l| serde_json::from_str(l).expect("task json"))
        .collect();
    assert_eq!(tasks.len(), 1);
    let task = &tasks[0];

    assert_eq!(task["parent"], serde_json::json!(root));
    assert_eq!(task["commit"], serde_json::json!(flip));
    assert_eq!(task["flipped_tests"], serde_json::json!(["adds"]));
    assert_eq!(task["family"], serde_json::json!("demo"));
    assert_eq!(
        task["lineage"],
        serde_json::json!(repo.display().to_string())
    );
    assert_eq!(
        task["id"],
        serde_json::json!(tsr_corpus::mine::task_id(
            &repo.display().to_string(),
            &root,
            &flip
        ))
    );

    // The hidden diff is the reference diff, addressed by its digest in the CAS.
    let diff = Command::new("git")
        .arg("-C")
        .arg(&repo)
        .args(["diff", &root, &flip])
        .output()
        .expect("git diff");
    let expected_diff = digest(&diff.stdout);
    assert_eq!(task["hidden_diff"], serde_json::json!(expected_diff));
    let cas_object = fs::read(cas_dir(&out).join(&expected_diff)).expect("cas object");
    assert_eq!(cas_object, diff.stdout);

    // The oracle encodes both halves.
    assert_eq!(
        task["outcome"]["name"],
        serde_json::json!("flipped_tests_pass_and_suite_no_regression")
    );
    assert!(task["outcome"]["resolves_true_when"]
        .as_str()
        .unwrap()
        .contains("full suite exits 0"));

    // No task for the refactor or the regression.
    assert!(tasks
        .iter()
        .all(|t| t["commit"] != serde_json::json!(refactor)));
    assert!(tasks
        .iter()
        .all(|t| t["commit"] != serde_json::json!(regression)));

    // The runner saw exactly two runs per commit, all against the repository snapshot.
    assert_eq!(runner.calls.borrow().len(), 6);
    assert!(runner.calls.borrow().iter().all(|(snap, _)| snap == &repo));
}

#[test]
fn first_parent_walk_is_oldest_first_and_excludes_the_root() {
    let tmp = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).expect("tempdir");
    let (repo, root, flip, refactor, regression) = fixture_repo(tmp.path());
    let pairs = first_parent_commits(&repo, None).expect("walk");
    assert_eq!(
        pairs,
        vec![
            (root.clone(), flip.clone()),
            (flip.clone(), refactor.clone()),
            (refactor.clone(), regression.clone()),
        ]
    );
}

#[test]
fn a_missing_report_is_an_error_not_an_empty_flip_set() {
    let tmp = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).expect("tempdir");
    let err = read_report(tmp.path()).expect_err("missing report must error");
    assert!(matches!(err, tsr_corpus::mine::MineError::MissingReport(_)));
}

/// Dump the fixture repo's `tasks.jsonl` for the evidence record. Run with `--nocapture`.
#[test]
fn dump_fixture_tasks_jsonl() {
    let tmp = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).expect("tempdir");
    let (repo, _root, _flip, _refactor, _regression) = fixture_repo(tmp.path());
    let out = tmp.path().join("out");
    let runner = SequenceRunner::new(vec![
        (1, report(&[], &["adds"])),
        (0, report(&["adds"], &[])),
        (0, report(&["adds"], &[])),
        (0, report(&["adds"], &[])),
        (0, report(&["adds"], &[])),
        (1, report(&[], &["adds"])),
    ]);
    let opts = MineOptions {
        suite: vec!["cargo".into(), "test".into()],
        since: None,
        family_rules: vec![FamilyRule {
            prefix: "crates/demo/".into(),
            family: "demo".into(),
        }],
        limits: Limits::default(),
    };
    mine(&repo, &out, &opts, &runner).expect("mine");
    let body = fs::read_to_string(out.join("tasks.jsonl")).expect("tasks.jsonl");
    println!("--- tasks.jsonl ---\n{body}--- end ---");
}
