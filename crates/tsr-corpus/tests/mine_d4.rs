//! D4: `tsr corpus mine` must check out what it claims (PRD §B9, §B11 M3).
//!
//! The defect (D4): `mine` ran the suite at the parent and at the commit with the **same** repo
//! path and argv, so the two runs were indistinguishable, and it set `parent_snapshot` to the
//! digest of the suite's stdout rather than to the parent commit's tree id. A task that claims to
//! start from the parent snapshot must actually be handed a checkout of the parent.
//!
//! This test builds a two-commit repository in a tempdir and drives `mine` with a [`GitRunner`]
//! that executes `git` for real (git only, no cargo): for every run it records the snapshot it was
//! handed and asks `git -C <snapshot> rev-parse HEAD` what revision that snapshot actually is. The
//! suite report the run would have written is written into the snapshot, exactly as the real suite
//! argv does.
//!
//! On the current (buggy) code the two runs share the repository path, so the parent run's
//! snapshot resolves to the commit (HEAD of the repo), not to the parent — the test fails. After
//! the fix the parent run's snapshot is a worktree of the parent commit and `parent_snapshot` is
//! the parent commit's tree id.

use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use tsr_corpus::mine::{mine, MineOptions, SuiteReport, TestVerdict, REPORT_FILE};
use tsr_corpus::{Limits, RunResult, Runner};

/// A [`Runner`] that executes `git` for real against the snapshot it is handed.
///
/// It records `(snapshot, argv, resolved_tree)` for every call, where `resolved_tree` is the tree
/// id of the snapshot's *content* (the snapshot is a plain directory from `git archive`, with no
/// `.git`, so `git rev-parse HEAD` inside it would walk up to the enclosing repository and lie —
/// N1). It then writes the suite report the run would have written into the snapshot and returns
/// the scripted exit code.
struct GitRunner {
    /// The exit code and report to answer with, in call order.
    answers: RefCell<Vec<(i32, SuiteReport)>>,
    /// Every call: the snapshot, the argv, and the tree id the snapshot's content hashes to.
    calls: RefCell<Vec<(PathBuf, Vec<String>, String)>>,
}

impl GitRunner {
    fn new(answers: Vec<(i32, SuiteReport)>) -> Self {
        Self {
            answers: RefCell::new(answers),
            calls: RefCell::new(Vec::new()),
        }
    }
}

impl Runner for GitRunner {
    fn run(&self, snapshot: &Path, argv: &[String], _limits: &Limits) -> RunResult {
        // Ask git what tree this snapshot's content is. A snapshot that is not a checkout of the
        // revision `mine` claims is the defect this test names. The snapshot has no `.git` (it is
        // extracted from `git archive`), so we hash its content with a throwaway index rather than
        // `git rev-parse HEAD`, which would resolve against the enclosing repository (N1).
        let resolved = tree_of(snapshot);
        self.calls
            .borrow_mut()
            .push((snapshot.to_path_buf(), argv.to_vec(), resolved));

        let (exit, report) = self.answers.borrow_mut().remove(0);
        let body = serde_json::to_vec(&report).expect("serializable");
        fs::write(snapshot.join(REPORT_FILE), &body).expect("write report");
        RunResult::from_output(exit, &body, b"", 1)
    }
}

/// The tree id of a plain directory's content (no `.git` required): initialise a throwaway
/// repository in it, stage everything, and write the tree. The tree id is content-addressed, so it
/// equals the source commit's tree id when the content is identical.
fn tree_of(dir: &Path) -> String {
    let status = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["init", "-q"])
        .status()
        .expect("git init");
    assert!(status.success(), "git init in snapshot failed");
    let status = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["add", "-A"])
        .status()
        .expect("git add");
    assert!(status.success(), "git add in snapshot failed");
    let out = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["write-tree"])
        .output()
        .expect("git write-tree");
    assert!(out.status.success(), "git write-tree in snapshot failed");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
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

fn rev(repo: &Path, r: &str) -> String {
    let out = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", r])
        .output()
        .expect("git rev-parse");
    assert!(out.status.success(), "git rev-parse {r} failed");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// A two-commit repository: the root has a red test, the second commit makes it green.
/// Returns `(repo, root, flip)`.
fn two_commit_repo(dir: &Path) -> (PathBuf, String, String) {
    let repo = dir.join("repo");
    fs::create_dir_all(&repo).expect("mkdir repo");
    git(&repo, &["init", "-q", "-b", "main"]);

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

    write(
        &repo,
        "crates/demo/tests/demo.rs",
        "#[test]\nfn adds() { assert_eq!(demo::add(1, 1), 2); }\n",
    );
    git(&repo, &["add", "-A"]);
    git(&repo, &["commit", "-q", "-m", "flip: fix the assertion"]);
    let flip = rev(&repo, "HEAD");

    (repo, root, flip)
}

/// D4: `mine` must materialise the parent and the commit as separate checkouts under the output
/// directory, run the suite in each, and set `parent_snapshot` to the parent commit's tree id.
#[test]
fn d4_mine_materialises_parent_and_commit_separately() {
    let tmp = tempfile::tempdir_in(env!("CARGO_MANIFEST_DIR")).expect("tempdir");
    let (repo, root, flip) = two_commit_repo(tmp.path());
    let out = tmp.path().join("out");

    // The suite at the parent (root) is red; at the commit (flip) it is green.
    let runner = GitRunner::new(vec![
        (1, report(&[], &["adds"])), // parent = root
        (0, report(&["adds"], &[])), // commit = flip
    ]);

    let opts = MineOptions {
        suite: vec!["cargo".into(), "test".into()],
        since: None,
        family_rules: Vec::new(),
        limits: Limits::default(),
    };
    let report_out = mine(&repo, &out, &opts, &runner).expect("mine");
    assert_eq!(report_out.tasks, 1, "one task: the flip commit");

    // The two runs were handed two *different* snapshots, both under the output directory, and
    // each snapshot is a real checkout of the revision it stands for.
    let calls = runner.calls.borrow();
    assert_eq!(calls.len(), 2, "one run at the parent, one at the commit");
    let (parent_snap, _, parent_rev) = &calls[0];
    let (commit_snap, _, commit_rev) = &calls[1];

    assert_ne!(
        parent_snap, commit_snap,
        "the parent and the commit must be materialised separately, not run in the same repo"
    );
    assert!(
        parent_snap.starts_with(&out),
        "the parent snapshot must live under --out, got {}",
        parent_snap.display()
    );
    assert!(
        commit_snap.starts_with(&out),
        "the commit snapshot must live under --out, got {}",
        commit_snap.display()
    );
    assert_eq!(
        parent_rev,
        &rev(&repo, &format!("{root}^{{tree}}")),
        "the parent run must be handed a checkout of the parent commit's tree"
    );
    assert_eq!(
        commit_rev,
        &rev(&repo, &format!("{flip}^{{tree}}")),
        "the commit run must be handed a checkout of the commit's tree"
    );

    // parent_snapshot is the parent commit's tree id, not the digest of the suite's stdout.
    let parent_tree = rev(&repo, &format!("{root}^{{tree}}"));
    let body = fs::read_to_string(out.join("tasks.jsonl")).expect("tasks.jsonl");
    let task: serde_json::Value =
        serde_json::from_str(body.lines().next().expect("one line")).expect("task json");
    assert_eq!(
        task["parent_snapshot"],
        serde_json::json!(parent_tree),
        "parent_snapshot must be the parent commit's tree id"
    );
    assert_ne!(
        task["parent_snapshot"],
        serde_json::json!(tsr_corpus::digest(b"")),
        "parent_snapshot must not be a stdout digest"
    );
}
