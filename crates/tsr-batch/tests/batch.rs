//! Integration tests for `tsr-batch` (PRD §B9, §B11 M3).
//!
//! Every test uses the `ScriptedRunner` and the `ScriptedDriver`: nothing here runs a repository's
//! tests on the host, and nothing here needs a container runtime. The one test that would need
//! Claude Code is `#[ignore]`d with its reason.

use std::fs;
use std::path::{Path, PathBuf};

use tsr_batch::driver::{read_session, Mode, ScriptedDriver};
use tsr_batch::invariants::{check_session_invariants, session_from_store};
use tsr_batch::results::{read_results, Outcome};
use tsr_batch::run::{flipped_tests_argv, run_all, run_one, BatchConfig};
use tsr_batch::session::SessionCtx;
use tsr_batch::task::{read_tasks, CorpusTask};
use tsr_batch::{ClaudeCodeDriver, Driver, DRIVER_UNAVAILABLE_EXIT};
use tsr_corpus::{Limits, RunResult, ScriptedRunner};
use tsr_domain::{Hash, TaskId};
use tsr_policy::gate::Grant;
use tsr_policy::profile::Profile;
use tsr_store::Store;

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/m3/batch")
        .canonicalize()
        .expect("fixtures/m3/batch exists")
}

fn profile() -> Profile {
    Profile::load(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../profiles/default.toml"))
        .expect("default profile loads")
}

fn tasks() -> Vec<CorpusTask> {
    read_tasks(&fixtures().join("tasks.jsonl")).expect("tasks.jsonl reads")
}

/// A runner that answers every provision and oracle call with success.
fn ok_runner() -> ScriptedRunner {
    ScriptedRunner::ok()
}

/// A runner that fails the flipped tests (exit 1) but passes the full suite.
///
/// D3: the oracle runs in the session's copy (`dest`), so the flipped argv is scripted against
/// `dest`, not the parent snapshot.
fn flipped_fails_runner(dest: &Path) -> ScriptedRunner {
    let mut r = ScriptedRunner::ok();
    // The flipped-tests argv carries `--tests`; the full-suite argv carries `--full-suite`.
    for task in tasks() {
        let flipped = tsr_batch::run::flipped_tests_argv(dest, &task.failing_tests());
        r.script(dest, &flipped, RunResult::exit_only(1));
    }
    r
}

fn ctx_for(task: &CorpusTask, dir: &Path, mode: Mode) -> SessionCtx {
    let store = Store::open(&dir.join(format!("{}.sqlite", mode.system()))).expect("store opens");
    let grant = Grant {
        task: TaskId(Hash::of(&[task.id.as_bytes()])),
        allowed_kinds: vec![
            tsr_domain::ActionKind::Inspect,
            tsr_domain::ActionKind::RunApprovedTest,
            tsr_domain::ActionKind::Generate,
            tsr_domain::ActionKind::ProposePatch,
            tsr_domain::ActionKind::RequestApproval,
            tsr_domain::ActionKind::Stop,
        ],
        read_scope_paths: profile().read_scope.paths.clone(),
        read_scope_rts_tools: profile().read_scope.rts_tools.clone(),
        allowed_test_ids: profile().tests.iter().map(|t| t.id.clone()).collect(),
        budgets: profile().budgets.clone(),
    };
    SessionCtx::new(
        store,
        TaskId(Hash::of(&[task.id.as_bytes()])),
        profile(),
        grant,
        "2026-01-01T00:00:00Z".to_string(),
    )
}

#[test]
fn record_session_satisfies_reconstruct_invariants() {
    let task = &tasks()[0];
    let dir = tempfile::tempdir().unwrap();
    let session = read_session(&fixtures().join("session-inspect-test-propose.json")).unwrap();
    let driver = ScriptedDriver::new(Mode::Record, session);
    let mut ctx = ctx_for(task, dir.path(), Mode::Record);
    ctx.open_task().unwrap();
    let report = driver.drive(&mut ctx, task, dir.path()).unwrap();
    ctx.close("done").unwrap();
    assert!(report.stopped, "the session reached a Stop");

    let session = session_from_store(ctx.store(), &ctx.task()).unwrap();
    check_session_invariants(&session).expect("record session satisfies the invariants");
}

#[test]
fn rules_session_satisfies_reconstruct_invariants() {
    let task = &tasks()[0];
    let dir = tempfile::tempdir().unwrap();
    let session = read_session(&fixtures().join("session-inspect-test-propose.json")).unwrap();
    let driver = ScriptedDriver::new(Mode::Rules, session);
    let mut ctx = ctx_for(task, dir.path(), Mode::Rules);
    ctx.open_task().unwrap();
    let report = driver.drive(&mut ctx, task, dir.path()).unwrap();
    ctx.close("done").unwrap();
    assert!(report.stopped, "the session reached a Stop");
    assert!(
        report.denials.is_empty(),
        "a well-formed rules session is not denied: {:?}",
        report.denials
    );

    let session = session_from_store(ctx.store(), &ctx.task()).unwrap();
    check_session_invariants(&session).expect("rules session satisfies the invariants");
}

#[test]
fn rules_mode_denies_a_gated_action_without_a_forecast() {
    let task = &tasks()[2]; // kamn store task
    let dir = tempfile::tempdir().unwrap();
    let session = read_session(&fixtures().join("session-missing-forecast.json")).unwrap();
    let driver = ScriptedDriver::new(Mode::Rules, session);
    let mut ctx = ctx_for(task, dir.path(), Mode::Rules);
    ctx.open_task().unwrap();
    let report = driver.drive(&mut ctx, task, dir.path()).unwrap();
    ctx.close("done").unwrap();

    assert_eq!(
        report.denials.len(),
        1,
        "exactly the forecast-less test is denied"
    );
    let reason = &report.denials[0].reason;
    assert!(
        reason.contains("record a forecast for RunApprovedTest at state version"),
        "the denial names what to record, got: {reason}"
    );
    assert!(
        reason.contains("before this action"),
        "the denial says when, got: {reason}"
    );

    // The denied action did not happen: no Executed event for it, and the session still satisfies
    // the invariants (a denied action is not a dispatch).
    let session = session_from_store(ctx.store(), &ctx.task()).unwrap();
    check_session_invariants(&session)
        .expect("a session with a denial still satisfies the invariants");
}

#[test]
fn record_mode_gates_nothing() {
    let task = &tasks()[2];
    let dir = tempfile::tempdir().unwrap();
    let session = read_session(&fixtures().join("session-inspect-test-propose.json")).unwrap();
    let driver = ScriptedDriver::new(Mode::Record, session);
    let mut ctx = ctx_for(task, dir.path(), Mode::Record);
    ctx.open_task().unwrap();
    let report = driver.drive(&mut ctx, task, dir.path()).unwrap();
    ctx.close("done").unwrap();

    assert!(report.denials.is_empty(), "system 1 gates nothing");
    let session = session_from_store(ctx.store(), &ctx.task()).unwrap();
    check_session_invariants(&session).expect("record session satisfies the invariants");
}

#[test]
fn batch_run_records_results_for_the_five_fixture_tasks() {
    let dir = tempfile::tempdir().unwrap();
    let config = BatchConfig::new(dir.path(), profile());
    // A runner that materialises the base tree into the copy directory on a materialize call
    // (the way `tsr corpus materialize` does), so the mutant tasks' mutation diff has a real
    // tree to apply against.
    struct CopyingRunner;
    impl tsr_corpus::Runner for CopyingRunner {
        fn run(&self, _snapshot: &Path, argv: &[String], _limits: &Limits) -> RunResult {
            if let Some(i) = argv.iter().position(|a| a == "--out") {
                let out = PathBuf::from(&argv[i + 1]);
                let _ = std::fs::create_dir_all(&out);
                let _ = std::fs::write(
                    out.join("lib.rs"),
                    "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
                );
            }
            RunResult::exit_only(0)
        }
    }
    let runner = CopyingRunner;
    let all = tasks();
    assert_eq!(all.len(), 5, "five fixture tasks");

    // The mutant tasks' copies are their base snapshot with the mutation diff applied (D4); the
    // diff is read from the CAS by its hex digest, so seed the objects the fixture names.
    let cas = tsr_corpus::mine::cas_dir(dir.path());
    std::fs::create_dir_all(&cas).unwrap();
    for task in &all {
        if let Some(digest) = task.mutation_diff.as_deref() {
            std::fs::write(
                cas.join(digest),
                "--- a/lib.rs\n+++ b/lib.rs\n@@ -1 +1 @@\n-pub fn add(a: i32, b: i32) -> i32 { a + b }\n+pub fn add(a: i32, b: i32) -> i32 { a - b }\n",
            )
            .unwrap();
        }
    }

    let session = read_session(&fixtures().join("session-inspect-test-propose.json")).unwrap();
    let results = run_all(
        &all,
        Mode::Rules,
        |_t| Box::new(ScriptedDriver::new(Mode::Rules, session.clone())),
        &runner,
        &config,
    )
    .unwrap();

    assert_eq!(results.len(), 5);
    for r in &results {
        assert_eq!(r.system, "rules");
        assert_eq!(
            r.outcome,
            Outcome::Accepted,
            "the ok runner accepts every task"
        );
        assert!(!r.state_root_at_start.is_empty());
        assert!(!r.state_root_at_end.is_empty());
        assert!(r.oracle.as_ref().unwrap().accepted);
    }

    let path = dir.path().join("results.jsonl");
    tsr_batch::write_results(&path, &results).unwrap();
    let back = read_results(&path).unwrap();
    assert_eq!(back, results, "results round-trip through JSONL");
}

#[test]
fn oracle_rejects_when_the_flipped_tests_fail() {
    let dir = tempfile::tempdir().unwrap();
    let config = BatchConfig::new(dir.path(), profile());
    let task = &tasks()[0];
    let dest = dir.path().join("copies").join(&task.id).join("rules");
    let runner = flipped_fails_runner(&dest);
    let session = read_session(&fixtures().join("session-inspect-test-propose.json")).unwrap();
    let driver = ScriptedDriver::new(Mode::Rules, session);
    let result = run_one(task, Mode::Rules, &driver, &runner, &config).unwrap();
    assert_eq!(result.outcome, Outcome::Rejected);
    let oracle = result.oracle.unwrap();
    assert!(!oracle.flipped_tests_pass);
    assert!(!oracle.suite_regressed);
    assert!(!oracle.accepted);
}

#[test]
fn oracle_rejects_when_the_suite_regresses() {
    let mut runner = ScriptedRunner::ok();
    let task = &tasks()[0];
    let dir = tempfile::tempdir().unwrap();
    let config = BatchConfig::new(dir.path(), profile());
    // D3: the oracle runs in the session's copy, so the suite is scripted against `dest`.
    let dest = dir.path().join("copies").join(&task.id).join("rules");
    let suite = tsr_batch::run::full_suite_argv(&dest);
    runner.script(&dest, &suite, RunResult::exit_only(1));
    let session = read_session(&fixtures().join("session-inspect-test-propose.json")).unwrap();
    let driver = ScriptedDriver::new(Mode::Rules, session);
    let result = run_one(task, Mode::Rules, &driver, &runner, &config).unwrap();
    assert_eq!(result.outcome, Outcome::Rejected);
    let oracle = result.oracle.unwrap();
    assert!(oracle.flipped_tests_pass);
    assert!(oracle.suite_regressed);
    assert!(!oracle.accepted);
}

#[test]
fn state_root_at_start_is_the_empty_store_root() {
    let task = &tasks()[0];
    let dir = tempfile::tempdir().unwrap();
    let config = BatchConfig::new(dir.path(), profile());
    let runner = ok_runner();
    let session = read_session(&fixtures().join("session-inspect-test-propose.json")).unwrap();
    let driver = ScriptedDriver::new(Mode::Rules, session);
    let result = run_one(task, Mode::Rules, &driver, &runner, &config).unwrap();
    // A fresh store's head root is the empty-log root; two fresh stores agree.
    let other = Store::open_in_memory().unwrap();
    assert_eq!(
        result.state_root_at_start,
        other.head_root().unwrap().to_hex()
    );
    assert_ne!(result.state_root_at_start, result.state_root_at_end);
}

#[test]
fn d10_rerunning_into_an_existing_out_starts_from_an_empty_store() {
    // D10: re-running into an existing `--out` must start from empty stores.
    // Pre-fix, `Store::open` reused the leftover sqlite file, so the second
    // run's `state_root_at_start` was the first run's end root, not the empty
    // store's root.
    let task = &tasks()[0];
    let dir = tempfile::tempdir().unwrap();
    let config = BatchConfig::new(dir.path(), profile());
    let runner = ok_runner();
    let session = read_session(&fixtures().join("session-inspect-test-propose.json")).unwrap();
    let driver = ScriptedDriver::new(Mode::Rules, session);

    let first = run_one(task, Mode::Rules, &driver, &runner, &config).unwrap();
    let second = run_one(task, Mode::Rules, &driver, &runner, &config).unwrap();

    let empty = Store::open_in_memory()
        .unwrap()
        .head_root()
        .unwrap()
        .to_hex();
    assert_eq!(
        second.state_root_at_start, empty,
        "the second run must start from an empty store, not the first run's end root"
    );
    assert_ne!(
        second.state_root_at_start, first.state_root_at_end,
        "the second run must not inherit the first run's store"
    );
}

#[test]
#[ignore = "needs Claude Code and a container runtime; the operator runs it on a host that has both"]
fn claude_code_driver_runs_on_a_host() {
    let task = &tasks()[0];
    let dir = tempfile::tempdir().unwrap();
    let mut ctx = ctx_for(task, dir.path(), Mode::Rules);
    ctx.open_task().unwrap();
    let driver = ClaudeCodeDriver::new("tsr", "claude-sonnet-4-5");
    let err = driver.drive(&mut ctx, task, dir.path()).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("no Claude Code"), "got: {msg}");
    assert_eq!(DRIVER_UNAVAILABLE_EXIT, 2);
}

#[test]
fn claude_code_driver_is_a_documented_stub() {
    // The stub is available in the sandbox and returns the reason, not a panic.
    let task = &tasks()[0];
    let dir = tempfile::tempdir().unwrap();
    let mut ctx = ctx_for(task, dir.path(), Mode::Rules);
    ctx.open_task().unwrap();
    let driver = ClaudeCodeDriver::new("tsr", "claude-sonnet-4-5");
    let err = driver.drive(&mut ctx, task, dir.path()).unwrap_err();
    assert!(err.to_string().contains("no Claude Code"));
    assert_eq!(driver.name(), "claude-code");
}

/// D3: the oracle must judge the session's copy (`dest`), not the unmodified parent snapshot.
///
/// The defect: `run_one` ran the oracle with `task.snapshot()` (the parent) as the working
/// directory, so a session that leaves the flipped test failing was still accepted. The oracle's
/// argv must be run against the copy the session worked in.
#[test]
fn d3_oracle_runs_in_the_session_copy_not_the_parent_snapshot() {
    let dir = tempfile::tempdir().unwrap();
    let config = BatchConfig::new(dir.path(), profile());
    let task = &tasks()[0];
    let session = read_session(&fixtures().join("session-inspect-test-propose.json")).unwrap();
    let driver = ScriptedDriver::new(Mode::Rules, session);

    // A runner that records every snapshot it was asked to run against.
    let runner = RecordingRunner::new();
    let _ = run_one(task, Mode::Rules, &driver, &runner, &config).unwrap();

    let dest = dir.path().join("copies").join(&task.id).join("rules");
    let oracle_calls: Vec<PathBuf> = runner
        .calls()
        .into_iter()
        .filter(|(_, argv)| argv.iter().any(|a| a == "--tests" || a == "--full-suite"))
        .map(|(snapshot, _)| snapshot)
        .collect();
    assert!(
        !oracle_calls.is_empty(),
        "the oracle ran at least once (flipped tests + full suite)"
    );
    for snapshot in &oracle_calls {
        assert_eq!(
            snapshot, &dest,
            "D3: the oracle must run in the session's copy {dest:?}, not {snapshot:?}"
        );
    }
}

/// D3: a session that leaves the flipped test failing is NOT accepted; one that fixes it IS.
///
/// The oracle's verdict is read from the run in the session's copy, so the two sessions must
/// differ. Before the fix both were accepted because the oracle ran against the parent snapshot.
#[test]
fn d3_a_session_that_leaves_the_flipped_test_failing_is_not_accepted() {
    let dir = tempfile::tempdir().unwrap();
    let config = BatchConfig::new(dir.path(), profile());
    let task = &tasks()[0];
    let session = read_session(&fixtures().join("session-inspect-test-propose.json")).unwrap();
    let driver = ScriptedDriver::new(Mode::Rules, session);

    // The session's copy fails the flipped tests (exit 1) but the full suite passes.
    let mut runner = ScriptedRunner::ok();
    let dest = dir.path().join("copies").join(&task.id).join("rules");
    runner.script(
        &dest,
        &flipped_tests_argv(&dest, &task.failing_tests()),
        RunResult::exit_only(1),
    );

    let result = run_one(task, Mode::Rules, &driver, &runner, &config).unwrap();
    assert_eq!(
        result.outcome,
        Outcome::Rejected,
        "D3: a session that leaves the flipped test failing must not be accepted"
    );
    let oracle = result.oracle.unwrap();
    assert!(!oracle.flipped_tests_pass);
    assert!(!oracle.accepted);
}

/// D3: a session that fixes the flipped test IS accepted.
#[test]
fn d3_a_session_that_fixes_the_flipped_test_is_accepted() {
    let dir = tempfile::tempdir().unwrap();
    let config = BatchConfig::new(dir.path(), profile());
    let task = &tasks()[0];
    let session = read_session(&fixtures().join("session-inspect-test-propose.json")).unwrap();
    let driver = ScriptedDriver::new(Mode::Rules, session);

    // The session's copy passes the flipped tests and the full suite.
    let mut runner = ScriptedRunner::ok();
    let dest = dir.path().join("copies").join(&task.id).join("rules");
    runner.script(
        &dest,
        &flipped_tests_argv(&dest, &task.failing_tests()),
        RunResult::exit_only(0),
    );

    let result = run_one(task, Mode::Rules, &driver, &runner, &config).unwrap();
    assert_eq!(
        result.outcome,
        Outcome::Accepted,
        "D3: a session that fixes the flipped test must be accepted"
    );
    assert!(result.oracle.unwrap().accepted);
}

/// D4: a non-zero provisioning exit must be refused, not ignored.
///
/// The defect: `run_one` recorded `provision:<exit>` as Evidence but never inspected the exit, so a
/// copy that was never materialised was driven and judged anyway. Before the fix this test fails
/// because `run_one` returns `Ok`; after it, `run_one` returns `RunError::ProvisioningFailed`.
#[test]
fn d4_provisioning_failure_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let config = BatchConfig::new(dir.path(), profile());
    let task = &tasks()[0];
    let session = read_session(&fixtures().join("session-inspect-test-propose.json")).unwrap();
    let driver = ScriptedDriver::new(Mode::Rules, session);

    // The provisioning run exits non-zero: the copy was never materialised.
    let mut runner = ScriptedRunner::ok();
    let dest = dir.path().join("copies").join(&task.id).join("rules");
    let provision = tsr_batch::run::provision_argv(Path::new(&task.repo), task.snapshot(), &dest);
    runner.script(
        tsr_batch::run::copies_root(dir.path()),
        &provision,
        RunResult::exit_only(1),
    );

    let err = run_one(task, Mode::Rules, &driver, &runner, &config)
        .expect_err("a non-zero provisioning exit must be refused");
    assert!(
        matches!(
            err,
            tsr_batch::run::RunError::ProvisioningFailed { exit: 1, .. }
        ),
        "expected ProvisioningFailed, got {err:?}"
    );
}

/// A runner that records every `(snapshot, argv)` it was asked to run, and answers success.
struct RecordingRunner {
    calls: std::cell::RefCell<Vec<(PathBuf, Vec<String>)>>,
}

impl RecordingRunner {
    fn new() -> Self {
        Self {
            calls: std::cell::RefCell::new(Vec::new()),
        }
    }

    fn calls(&self) -> Vec<(PathBuf, Vec<String>)> {
        self.calls.borrow().clone()
    }
}

impl tsr_corpus::Runner for RecordingRunner {
    fn run(&self, snapshot: &Path, argv: &[String], _limits: &Limits) -> RunResult {
        self.calls
            .borrow_mut()
            .push((snapshot.to_path_buf(), argv.to_vec()));
        RunResult::exit_only(0)
    }
}

#[test]
fn limits_and_run_result_are_the_corpus_contract() {
    // A guard that the corpus contract this crate builds on has not moved.
    let limits = Limits::new(1000, 4096);
    let r = RunResult::exit_only(0);
    assert!(r.succeeded());
    assert_eq!(limits.wall_clock_ms, 1000);
}

// ---------------------------------------------------------------------------------------------
// D1: the untrusted task id must not be joined into a path (security review F1)
// ---------------------------------------------------------------------------------------------

/// Write a one-line `tasks.jsonl` whose only task carries `id`, and return the manifest path.
fn manifest_with_id(dir: &Path, id: &str) -> PathBuf {
    let mut task = tasks().into_iter().next().expect("a fixture task");
    task.id = id.to_string();
    let path = dir.join("tasks.jsonl");
    let line = serde_json::to_string(&task).expect("serializable");
    fs::write(&path, format!("{line}\n")).expect("write manifest");
    path
}

/// D1: an absolute task id must be refused where `tasks.jsonl` is parsed, not joined into
/// `out/state/<id>` (an absolute id replaces the base and escapes `--out`).
#[test]
fn d1_absolute_task_id_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = manifest_with_id(dir.path(), "/etc/tsr-escape");
    let err = read_tasks(&path).expect_err("an absolute task id must be refused");
    assert!(
        matches!(err, tsr_batch::TaskError::InvalidId { .. }),
        "expected InvalidId, got {err:?}"
    );
}

/// D1: a `../` task id must be refused where `tasks.jsonl` is parsed.
#[test]
fn d1_parent_dir_task_id_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = manifest_with_id(dir.path(), "../escape");
    let err = read_tasks(&path).expect_err("a `../` task id must be refused");
    assert!(
        matches!(err, tsr_batch::TaskError::InvalidId { .. }),
        "expected InvalidId, got {err:?}"
    );
}

/// D1: `.` and `..` are refused, and a well-formed id is accepted.
#[test]
fn d1_dot_ids_are_refused_and_a_valid_id_is_accepted() {
    let dir = tempfile::tempdir().unwrap();
    for bad in [".", "..", "a/b", "a\\b", ""] {
        let path = manifest_with_id(dir.path(), bad);
        let err = read_tasks(&path)
            .err()
            .unwrap_or_else(|| panic!("id {bad:?} must be refused"));
        assert!(
            matches!(err, tsr_batch::TaskError::InvalidId { .. }),
            "id {bad:?}: expected InvalidId, got {err:?}"
        );
    }
    // A well-formed id (the fixture's own blake3 hex) still reads.
    let good = tasks()[0].id.clone();
    let path = manifest_with_id(dir.path(), &good);
    let read = read_tasks(&path).expect("a valid id reads");
    assert_eq!(read.len(), 1);
    assert_eq!(read[0].id, good);
}

/// R2, D1: `run_one` is public, so it must re-check the task id before joining it into a path.
///
/// `read_tasks` refuses an unsafe id, but a caller can build a `CorpusTask` directly and hand it to
/// `run_one`, which joins `task.id` into `out/state/<id>` and `out/copies/<id>`. Pre-fix, an
/// absolute or `..` id reached those joins unchecked. The fix refuses it at the top of `run_one`.
#[test]
fn r2_run_one_rechecks_the_task_id() {
    let dir = tempfile::tempdir().unwrap();
    let config = BatchConfig::new(dir.path(), profile());
    let runner = ok_runner();
    let session = read_session(&fixtures().join("session-inspect-test-propose.json")).unwrap();
    let driver = ScriptedDriver::new(Mode::Rules, session);

    for bad in ["/etc/tsr-escape", "../escape", "..", "."] {
        let mut task = tasks().into_iter().next().expect("a fixture task");
        task.id = bad.to_string();
        let err = run_one(&task, Mode::Rules, &driver, &runner, &config)
            .err()
            .unwrap_or_else(|| panic!("id {bad:?} must be refused by run_one"));
        assert!(
            matches!(err, tsr_batch::RunError::InvalidTaskId { .. }),
            "id {bad:?}: expected InvalidTaskId, got {err:?}"
        );
    }

    // Nothing was created outside `--out`: the escape paths do not exist.
    assert!(!Path::new("/etc/tsr-escape").exists());
    assert!(!dir.path().parent().unwrap().join("escape").exists());
}

/// D4: provisioning runs from a batch-owned root, never from the snapshot value, and passes
/// `--repo` so `materialize` does not default to `.`.
///
/// The defect: `run_one` ran provisioning with the working directory set to `task.snapshot()` — a
/// tree id, not a directory — so every task failed `provisioning failed … exited 127`, and
/// `provision_argv` passed no `--repo`, so `materialize` defaulted to `.`.
#[test]
fn d4_provisioning_runs_from_a_batch_owned_root_with_repo() {
    let dir = tempfile::tempdir().unwrap();
    let config = BatchConfig::new(dir.path(), profile());
    let task = &tasks()[0];
    let session = read_session(&fixtures().join("session-inspect-test-propose.json")).unwrap();
    let driver = ScriptedDriver::new(Mode::Rules, session);

    let runner = RecordingRunner::new();
    let _ = run_one(task, Mode::Rules, &driver, &runner, &config).unwrap();

    let provision_calls: Vec<(PathBuf, Vec<String>)> = runner
        .calls()
        .into_iter()
        .filter(|(_, argv)| argv.iter().any(|a| a == "materialize"))
        .collect();
    assert_eq!(provision_calls.len(), 1, "exactly one provisioning run");
    let (cwd, argv) = &provision_calls[0];

    // The working directory is the batch-owned copies root, not the snapshot value.
    assert_eq!(
        cwd.as_path(),
        tsr_batch::run::copies_root(dir.path()).as_path(),
        "D4: provisioning must run from the batch-owned copies root, not the snapshot"
    );
    assert_ne!(
        cwd.as_os_str(),
        std::ffi::OsStr::new(task.snapshot()),
        "D4: the snapshot value must never be the provisioning working directory"
    );
    // `--repo` is passed, so `materialize` does not default to `.`.
    let repo_idx = argv
        .iter()
        .position(|a| a == "--repo")
        .expect("D4: provision_argv must pass --repo");
    assert_eq!(
        argv[repo_idx + 1],
        task.repo,
        "D4: --repo must name the task's repository"
    );
    // The snapshot is passed as an argument, never as a path the batch runs from.
    let snap_idx = argv.iter().position(|a| a == "--snapshot").unwrap();
    assert_eq!(argv[snap_idx + 1], task.snapshot());
}

/// R1: a snapshot value that is not a hex object id is refused before anything is mounted.
///
/// The defect: the snapshot value from `tasks.jsonl` became `ExecRunner`'s working directory, so
/// `"parent_snapshot":"/etc"` made `/etc` the sandbox's writable mount (PRD §B10: the disposable
/// copy is the only writable mount). A snapshot is accepted only as a hex object id.
#[test]
fn r1_snapshot_must_be_a_hex_object_id() {
    let dir = tempfile::tempdir().unwrap();
    let config = BatchConfig::new(dir.path(), profile());
    let session = read_session(&fixtures().join("session-inspect-test-propose.json")).unwrap();
    let driver = ScriptedDriver::new(Mode::Rules, session);

    let mut task = tasks()[0].clone();
    task.parent_snapshot = Some("/etc".to_string());

    let runner = RecordingRunner::new();
    let err = run_one(&task, Mode::Rules, &driver, &runner, &config)
        .expect_err("R1: a snapshot that is not a hex object id must be refused");
    assert!(
        matches!(err, tsr_batch::run::RunError::UnsafeSnapshot { .. }),
        "R1: expected UnsafeSnapshot, got {err:?}"
    );
    // Nothing was mounted: no provisioning run happened at all.
    assert!(
        runner.calls().is_empty(),
        "R1: nothing may be mounted for an unsafe snapshot, got {:?}",
        runner.calls()
    );
}

/// R1: a `..` snapshot is refused too — a relative path is not a hex object id.
#[test]
fn r1_dotdot_snapshot_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let config = BatchConfig::new(dir.path(), profile());
    let session = read_session(&fixtures().join("session-inspect-test-propose.json")).unwrap();
    let driver = ScriptedDriver::new(Mode::Rules, session);

    let mut task = tasks()[0].clone();
    task.parent_snapshot = Some("../../etc".to_string());

    let runner = RecordingRunner::new();
    let err = run_one(&task, Mode::Rules, &driver, &runner, &config)
        .expect_err("R1: a `..` snapshot must be refused");
    assert!(matches!(
        err,
        tsr_batch::run::RunError::UnsafeSnapshot { .. }
    ));
    assert!(runner.calls().is_empty());
}

/// D4: a mutant's copy is its base snapshot materialised with the mutant's own diff applied.
///
/// The defect: a mutant's snapshot was a descriptor hash (`mutated_snapshot`) that named nothing
/// materialisable. The copy is now the base tree with the mutation diff applied, so the copy's
/// file holds the mutated source.
#[test]
fn d4_mutant_copy_applies_the_mutation_diff() {
    use std::process::Command;

    let dir = tempfile::tempdir().unwrap();
    let repo = dir.path().join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let git = |args: &[&str]| {
        let out = Command::new("git")
            .arg("-C")
            .arg(&repo)
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "git {args:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };
    git(&["init", "-q"]);
    git(&["config", "user.email", "t@example.com"]);
    git(&["config", "user.name", "t"]);
    std::fs::write(
        repo.join("lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 { a + b }\n",
    )
    .unwrap();
    git(&["add", "."]);
    git(&["commit", "-qm", "base"]);
    let base_tree = git(&["rev-parse", "HEAD^{tree}"]);

    // The mutation diff: replace `a + b` with `a - b`.
    let mutation = "--- a/lib.rs\n+++ b/lib.rs\n@@ -1 +1 @@\n-pub fn add(a: i32, b: i32) -> i32 { a + b }\n+pub fn add(a: i32, b: i32) -> i32 { a - b }\n";
    let cas = tsr_corpus::mine::cas_dir(dir.path());
    std::fs::create_dir_all(&cas).unwrap();
    let digest = tsr_domain::Hash::of(&[mutation.as_bytes()]).to_hex();
    std::fs::write(cas.join(&digest), mutation).unwrap();

    let mut task = tasks()[0].clone();
    task.parent_snapshot = None;
    task.base_snapshot = Some(base_tree.clone());
    task.mutated_snapshot = Some("deadbeefdeadbeefdeadbeefdeadbeefdeadbeef".to_string());
    task.mutation_diff = Some(digest);
    task.repo = repo.display().to_string();

    let config = BatchConfig::new(dir.path(), profile());
    let session = read_session(&fixtures().join("session-inspect-test-propose.json")).unwrap();
    let driver = ScriptedDriver::new(Mode::Rules, session);

    // A runner that actually materialises the base tree into `dest` (the way `tsr corpus
    // materialize` does), so the mutation diff has a real tree to apply against.
    struct MaterializingRunner;
    impl tsr_corpus::Runner for MaterializingRunner {
        fn run(&self, snapshot: &Path, argv: &[String], _limits: &Limits) -> RunResult {
            if argv.iter().any(|a| a == "materialize") {
                let repo = argv[argv.iter().position(|a| a == "--repo").unwrap() + 1].clone();
                let snap = argv[argv.iter().position(|a| a == "--snapshot").unwrap() + 1].clone();
                let out = argv[argv.iter().position(|a| a == "--out").unwrap() + 1].clone();
                let _ = snapshot;
                let status = std::process::Command::new("sh")
                    .arg("-c")
                    .arg(format!(
                        "mkdir -p {out} && git -C {repo} archive --format=tar {snap} | tar -x -C {out}"
                    ))
                    .status()
                    .unwrap();
                return RunResult::exit_only(status.code().unwrap_or(1));
            }
            RunResult::exit_only(0)
        }
    }

    let result = run_one(&task, Mode::Rules, &driver, &MaterializingRunner, &config).unwrap();
    let dest = tsr_batch::run::copies_root(dir.path())
        .join(&task.id)
        .join("rules");
    let copied = std::fs::read_to_string(dest.join("lib.rs")).unwrap();
    assert!(
        copied.contains("a - b"),
        "D4: the mutant's copy must hold the mutated source, got: {copied:?}"
    );
    assert_eq!(result.outcome, Outcome::Accepted);
}

/// D4: a real runner that materialised a copy but whose mutation diff is absent from the CAS is a
/// hard error — the `dest.exists()` gate must not weaken it.
///
/// The regression fix (15c3015) skips the CAS read only when provisioning left no copy at `dest`
/// (a stand-in runner). A runner that *did* create `dest` — the shape a real `ExecRunner` always
/// has after exit 0 — must still refuse a missing digest, never drive a silently unmutated copy.
#[test]
fn d4_a_materialised_copy_with_a_missing_mutation_diff_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let config = BatchConfig::new(dir.path(), profile());
    let session = read_session(&fixtures().join("session-inspect-test-propose.json")).unwrap();
    let driver = ScriptedDriver::new(Mode::Rules, session);

    let mut task = tasks()[0].clone();
    task.parent_snapshot = None;
    task.base_snapshot = Some("d4e5f6a7b8c9d4e5f6a7b8c9d4e5f6a7b8c9d4".to_string());
    task.mutated_snapshot = Some("f6a7b8c9d0e1f6a7b8c9d0e1f6a7b8c9d0e1f6a7".to_string());
    // A digest that names nothing in the CAS.
    task.mutation_diff =
        Some("1111111111111111111111111111111111111111111111111111111111111111".to_string());

    // A runner that materialises a copy at `dest` (the way a real runner does after exit 0) but
    // leaves the CAS empty, so the diff cannot be read.
    struct MaterializingRunner;
    impl tsr_corpus::Runner for MaterializingRunner {
        fn run(&self, _snapshot: &Path, argv: &[String], _limits: &Limits) -> RunResult {
            if argv.iter().any(|a| a == "materialize") {
                let out = argv[argv.iter().position(|a| a == "--out").unwrap() + 1].clone();
                let _ = std::fs::create_dir_all(&out);
                let _ = std::fs::write(Path::new(&out).join("lib.rs"), "pub fn add() {}\n");
            }
            RunResult::exit_only(0)
        }
    }

    let err = run_one(&task, Mode::Rules, &driver, &MaterializingRunner, &config)
        .expect_err("a materialised copy with a missing mutation diff must be refused");
    assert!(
        matches!(err, tsr_batch::run::RunError::MutationDiff { .. }),
        "D4: expected MutationDiff, got {err:?}"
    );
}
