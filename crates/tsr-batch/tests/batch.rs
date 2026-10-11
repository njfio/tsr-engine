//! Integration tests for `tsr-batch` (PRD §B9, §B11 M3).
//!
//! Every test uses the `ScriptedRunner` and the `ScriptedDriver`: nothing here runs a repository's
//! tests on the host, and nothing here needs a container runtime. The one test that would need
//! Claude Code is `#[ignore]`d with its reason.

use std::path::{Path, PathBuf};

use tsr_batch::driver::{read_session, Mode, ScriptedDriver};
use tsr_batch::invariants::{check_session_invariants, session_from_store};
use tsr_batch::results::{read_results, Outcome};
use tsr_batch::run::{run_all, run_one, BatchConfig};
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
fn flipped_fails_runner() -> ScriptedRunner {
    let mut r = ScriptedRunner::ok();
    // The flipped-tests argv carries `--tests`; the full-suite argv carries `--full-suite`.
    for task in tasks() {
        let flipped = tsr_batch::run::flipped_tests_argv(task.snapshot(), &task.failing_tests());
        r.script(task.snapshot(), &flipped, RunResult::exit_only(1));
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
    let report = driver.drive(&mut ctx, task).unwrap();
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
    let report = driver.drive(&mut ctx, task).unwrap();
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
    let report = driver.drive(&mut ctx, task).unwrap();
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
    let report = driver.drive(&mut ctx, task).unwrap();
    ctx.close("done").unwrap();

    assert!(report.denials.is_empty(), "system 1 gates nothing");
    let session = session_from_store(ctx.store(), &ctx.task()).unwrap();
    check_session_invariants(&session).expect("record session satisfies the invariants");
}

#[test]
fn batch_run_records_results_for_the_five_fixture_tasks() {
    let dir = tempfile::tempdir().unwrap();
    let config = BatchConfig::new(dir.path(), profile());
    let runner = ok_runner();
    let all = tasks();
    assert_eq!(all.len(), 5, "five fixture tasks");

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
    let runner = flipped_fails_runner();
    let task = &tasks()[0];
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
    let suite = tsr_batch::run::full_suite_argv(task.snapshot());
    runner.script(task.snapshot(), &suite, RunResult::exit_only(1));
    let dir = tempfile::tempdir().unwrap();
    let config = BatchConfig::new(dir.path(), profile());
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
#[ignore = "needs Claude Code and a container runtime; the operator runs it on a host that has both"]
fn claude_code_driver_runs_on_a_host() {
    let task = &tasks()[0];
    let dir = tempfile::tempdir().unwrap();
    let mut ctx = ctx_for(task, dir.path(), Mode::Rules);
    ctx.open_task().unwrap();
    let driver = ClaudeCodeDriver::new("tsr", "claude-sonnet-4-5");
    let err = driver.drive(&mut ctx, task).unwrap_err();
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
    let err = driver.drive(&mut ctx, task).unwrap_err();
    assert!(err.to_string().contains("no Claude Code"));
    assert_eq!(driver.name(), "claude-code");
}

#[test]
fn limits_and_run_result_are_the_corpus_contract() {
    // A guard that the corpus contract this crate builds on has not moved.
    let limits = Limits::new(1000, 4096);
    let r = RunResult::exit_only(0);
    assert!(r.succeeded());
    assert_eq!(limits.wall_clock_ms, 1000);
}
