//! D6: the Tier 0 controller behind the `Driver` trait for the rules system (PRD §B9).
//!
//! PRD §B9, the Tier 0 row: *"inspect until the failing assertion and its call site are read;
//! test; propose; stop on budget"*. PRD §B5 adds: *"on exhaustion the task stops with
//! `Stop(budget)`"*, and a gate denial ends the attempt — it does not continue.
//!
//! Before this fix, `ScriptedDriver::drive` in `Mode::Rules` replayed a fixture's steps through
//! the gate and, on a denial, `continue`d to the next step (driver.rs:335). Nothing enforced
//! read-before-test, nothing stopped on budget, and a denial did not end the attempt. These tests
//! exercise the three rules with a scripted repository: a `ScriptedRepo` names the failing
//! assertion, its call site, and the allowlisted test id that runs the failing test, and a
//! `Tier0Controller` decides each step.
//!
//! Every test here uses the `ScriptedDriver` and a scripted repository: nothing runs a
//! repository's tests on the host.

use std::path::{Path, PathBuf};

use tsr_batch::driver::{
    read_session, Mode, ScriptedDriver, ScriptedRepo, ScriptedSession, ScriptedStep,
    Tier0Controller, Tier0Error,
};
use tsr_batch::session::SessionCtx;
use tsr_batch::task::{read_tasks, CorpusTask};
use tsr_batch::Driver;
use tsr_domain::{ActionKind, Hash, TaskId};
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

fn ctx_for(task: &CorpusTask, dir: &Path, mode: Mode) -> SessionCtx {
    let store = Store::open(&dir.join(format!("{}.sqlite", mode.system()))).expect("store opens");
    let grant = Grant {
        task: TaskId(Hash::of(&[task.id.as_bytes()])),
        allowed_kinds: vec![
            ActionKind::Inspect,
            ActionKind::RunApprovedTest,
            ActionKind::Generate,
            ActionKind::ProposePatch,
            ActionKind::RequestApproval,
            ActionKind::Stop,
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

/// A scripted repository: the failing assertion, its call site, and the allowlisted test id that
/// runs the failing test. `cargo-test-parser` is the id the default profile allowlists.
fn scripted_repo() -> ScriptedRepo {
    ScriptedRepo::new("src/parser.rs", "src/lib.rs", "cargo-test-parser")
}

/// A session that reads the failing assertion and its call site, then runs the test, then proposes.
fn read_then_test_then_propose() -> ScriptedSession {
    ScriptedSession {
        name: "tier0-read-test-propose".to_string(),
        steps: vec![
            ScriptedStep::inspect("src/parser.rs", 0.9),
            ScriptedStep::inspect("src/lib.rs", 0.9),
            ScriptedStep::run_test("cargo-test-parser", 0.4, 101),
            ScriptedStep::propose_patch(128, 0.7),
            ScriptedStep::run_test("cargo-test-parser", 0.85, 0),
            ScriptedStep::stop(),
        ],
    }
}

/// D6 (a) read-before-test: the controller refuses to run the failing test until the failing
/// assertion and its call site have both been read.
///
/// The defect: rules mode replayed the test step with no read-before-test enforcement, so a
/// session that ran the test first was accepted. The controller must refuse the test step and
/// name the unread path.
#[test]
fn d6_read_before_test_is_enforced() {
    let repo = scripted_repo();
    let mut controller = Tier0Controller::new(repo, 60);

    // The test step arrives before either path has been read: refused.
    let err = controller
        .check(&ScriptedStep::run_test("cargo-test-parser", 0.4, 101))
        .expect_err("D6: the test must not run before the assertion and its call site are read");
    match err {
        Tier0Error::ReadBeforeTest { unread, .. } => {
            assert!(
                unread.contains(&"src/parser.rs".to_string()),
                "the refusal names the unread assertion, got {unread:?}"
            );
            assert!(
                unread.contains(&"src/lib.rs".to_string()),
                "the refusal names the unread call site, got {unread:?}"
            );
        }
        other => panic!("expected ReadBeforeTest, got {other:?}"),
    }

    // Read the assertion only: still refused, and the call site is the one named.
    controller
        .check(&ScriptedStep::inspect("src/parser.rs", 0.9))
        .expect("inspecting the assertion is allowed");
    let err = controller
        .check(&ScriptedStep::run_test("cargo-test-parser", 0.4, 101))
        .expect_err("D6: the call site is still unread");
    match err {
        Tier0Error::ReadBeforeTest { unread, .. } => {
            assert_eq!(unread, vec!["src/lib.rs".to_string()]);
        }
        other => panic!("expected ReadBeforeTest, got {other:?}"),
    }

    // Read the call site too: the test is now allowed.
    controller
        .check(&ScriptedStep::inspect("src/lib.rs", 0.9))
        .expect("inspecting the call site is allowed");
    controller
        .check(&ScriptedStep::run_test("cargo-test-parser", 0.4, 101))
        .expect("D6: with both paths read, the test is allowed");
}

/// D6 (b) budget stop: the controller stops when the budget is exhausted, with `Stop(budget)`.
///
/// The defect: rules mode had no budget stop; it replayed every step regardless of the budget.
#[test]
fn d6_budget_stop_ends_the_attempt() {
    let repo = scripted_repo();
    // A budget of 1 tool-minute: the first test step spends it, the second must stop.
    let mut controller = Tier0Controller::new(repo, 1);
    controller
        .check(&ScriptedStep::inspect("src/parser.rs", 0.9))
        .unwrap();
    controller
        .check(&ScriptedStep::inspect("src/lib.rs", 0.9))
        .unwrap();
    controller
        .check(&ScriptedStep::run_test("cargo-test-parser", 0.4, 101))
        .expect("the first test fits the budget");

    // The budget is now exhausted: the next step must stop with Stop(budget), not run.
    let err = controller
        .check(&ScriptedStep::run_test("cargo-test-parser", 0.85, 0))
        .expect_err("D6: the budget is exhausted");
    match err {
        Tier0Error::BudgetExhausted { spent, budget } => {
            assert_eq!(budget, 1);
            assert!(spent >= budget, "spent {spent} >= budget {budget}");
        }
        other => panic!("expected BudgetExhausted, got {other:?}"),
    }
    assert!(
        controller.stopped_on_budget(),
        "D6: the controller records that it stopped on budget"
    );
}

/// D6 (c) stop on denial: a gate denial ends the attempt; the controller does not continue.
///
/// The defect: on a denial, `drive` pushed the denial and `continue`d to the next step
/// (driver.rs:335), so a denied action did not end the attempt.
#[test]
fn d6_a_denial_ends_the_attempt() {
    let task = &tasks()[2]; // the kamn store task
    let dir = tempfile::tempdir().unwrap();
    // The fixture's test step omits its forecast, so the gate denies it. The controller's failing
    // test id is a *different* allowlisted id, so read-before-test does not fire first: the step
    // reaches the gate, and the denial must end the attempt.
    let repo = ScriptedRepo::new("src/store.rs", "src/store.rs", "cargo-test-workspace");
    let session = read_session(&fixtures().join("session-missing-forecast.json")).unwrap();
    let driver = ScriptedDriver::new(Mode::Rules, session).with_tier0(repo, 60);
    let mut ctx = ctx_for(task, dir.path(), Mode::Rules);
    ctx.open_task().unwrap();
    let report = driver.drive(&mut ctx, task).unwrap();
    ctx.close("done").unwrap();

    assert_eq!(report.denials.len(), 1, "exactly one denial");
    assert!(
        report.ended_on_denial,
        "D6: a denial ends the attempt; the driver must not continue to the next step"
    );
    // The steps after the denied one did not run: the attempt ended at the denial.
    assert!(
        !report.stopped,
        "the attempt ended on the denial, not on a later Stop step"
    );
}

/// D6: a well-formed Tier 0 session (read, test, propose, test, stop) is not denied and reaches Stop.
#[test]
fn d6_a_well_formed_tier0_session_reaches_stop() {
    let task = &tasks()[0];
    let dir = tempfile::tempdir().unwrap();
    let driver = ScriptedDriver::new(Mode::Rules, read_then_test_then_propose())
        .with_tier0(scripted_repo(), 60);
    let mut ctx = ctx_for(task, dir.path(), Mode::Rules);
    ctx.open_task().unwrap();
    let report = driver.drive(&mut ctx, task).unwrap();
    ctx.close("done").unwrap();

    assert!(report.denials.is_empty(), "no denial: {:?}", report.denials);
    assert!(report.stopped, "the session reached Stop");
    assert!(!report.ended_on_denial);
}

/// D6: the controller is behind the `Driver` trait — the rules driver consults it, and the
/// record driver does not (system 1 gates nothing and runs no Tier 0 policy).
#[test]
fn d6_the_controller_is_behind_the_driver_trait() {
    let task = &tasks()[0];
    let dir = tempfile::tempdir().unwrap();

    // A session that runs the failing test before reading the assertion: a Tier 0 violation.
    let session = ScriptedSession {
        name: "tier0-violating".to_string(),
        steps: vec![
            ScriptedStep::run_test("cargo-test-parser", 0.4, 101),
            ScriptedStep::stop(),
        ],
    };

    // Record mode: the same session is replayed unchanged.
    let record = ScriptedDriver::new(Mode::Record, session.clone());
    let mut ctx = ctx_for(task, dir.path(), Mode::Record);
    ctx.open_task().unwrap();
    let report = record.drive(&mut ctx, task).unwrap();
    ctx.close("done").unwrap();
    assert!(
        report.denials.is_empty(),
        "system 1 gates nothing and runs no Tier 0 policy"
    );

    // Rules mode with the controller: the same session is refused at the test step. D6 (F2): the
    // refusal ends *this* task — the driver records `Stop(blocked)` and returns Ok with the report
    // marked, so the batch continues with the next task — rather than returning Err and aborting
    // the whole batch.
    let rules = ScriptedDriver::new(Mode::Rules, session).with_tier0(scripted_repo(), 60);
    let mut ctx = ctx_for(task, dir.path(), Mode::Rules);
    ctx.open_task().unwrap();
    let report = rules
        .drive(&mut ctx, task)
        .expect("D6 (F2): a read-before-test refusal ends the task, it does not abort the batch");
    assert!(
        report.stopped_on_tier0,
        "D6 (F2): the report marks the Tier 0 refusal"
    );
    assert!(
        report.tier0_reason.is_some(),
        "D6 (F2): the refusal's reason is recorded"
    );
    assert!(
        report.denials.is_empty(),
        "the refusal is a Tier 0 stop, not a gate denial"
    );
}

/// D6 (F1): the step that would *cross* the budget is refused, not the step after it.
///
/// The defect: `check` tested `spent >= budget` before adding the step's minutes, so the crossing
/// step was committed and only a later step was refused. A session whose overspending step was
/// last was therefore accepted at 101 tool-minutes against a budget of 30, with 0 denials.
#[test]
fn d6_budget_stop_refuses_the_crossing_step() {
    let repo = scripted_repo();
    // A budget of 1 tool-minute. The first test step spends exactly 1 (allowed: 0 + 1 > 1 is
    // false). The *next* step is the crossing step and must be refused, even though it is last.
    let mut controller = Tier0Controller::new(repo, 1);
    controller
        .check(&ScriptedStep::inspect("src/parser.rs", 0.9))
        .unwrap();
    controller
        .check(&ScriptedStep::inspect("src/lib.rs", 0.9))
        .unwrap();
    controller
        .check(&ScriptedStep::run_test("cargo-test-parser", 0.4, 101))
        .expect("the first test fits the budget exactly");

    // The crossing step: 1 + 1 > 1, so it is refused before it is committed.
    let err = controller
        .check(&ScriptedStep::run_test("cargo-test-parser", 0.85, 0))
        .expect_err("D6 (F1): the step that would cross the budget must be refused");
    match err {
        Tier0Error::BudgetExhausted { spent, budget } => {
            assert_eq!(budget, 1);
            assert_eq!(spent, 1, "the crossing step was not committed");
        }
        other => panic!("expected BudgetExhausted, got {other:?}"),
    }
    assert!(
        controller.stopped_on_budget(),
        "D6 (F1): the controller records that it stopped on budget"
    );
    assert_eq!(
        controller.spent(),
        1,
        "D6 (F1): the refused step's minutes were not added to the spend"
    );
}

/// D6: the controller's state is observable, so a test can assert the three rules directly.
#[test]
fn d6_controller_state_is_observable() {
    let mut controller = Tier0Controller::new(scripted_repo(), 60);
    assert!(!controller.read_assertion());
    assert!(!controller.read_call_site());
    assert!(!controller.ran_test());
    controller
        .check(&ScriptedStep::inspect("src/parser.rs", 0.9))
        .unwrap();
    assert!(controller.read_assertion());
    assert!(!controller.read_call_site());
    controller
        .check(&ScriptedStep::inspect("src/lib.rs", 0.9))
        .unwrap();
    assert!(controller.read_call_site());
    controller
        .check(&ScriptedStep::run_test("cargo-test-parser", 0.4, 101))
        .unwrap();
    assert!(controller.ran_test());
}

/// D6: a Tier 0 session satisfies the reconstruct invariants when driven through the controller.
#[test]
fn d6_tier0_session_satisfies_reconstruct_invariants() {
    use tsr_batch::invariants::{check_session_invariants, session_from_store};
    let task = &tasks()[0];
    let dir = tempfile::tempdir().unwrap();
    let driver = ScriptedDriver::new(Mode::Rules, read_then_test_then_propose())
        .with_tier0(scripted_repo(), 60);
    let mut ctx = ctx_for(task, dir.path(), Mode::Rules);
    ctx.open_task().unwrap();
    driver.drive(&mut ctx, task).unwrap();
    ctx.close("done").unwrap();
    let session = session_from_store(ctx.store(), &ctx.task()).unwrap();
    check_session_invariants(&session).expect("a Tier 0 session satisfies the invariants");
}

/// A guard that the fixture manifest the tests read is present and non-empty.
#[test]
fn d6_fixture_manifest_is_present() {
    let all = tasks();
    assert!(!all.is_empty(), "fixtures/m3/batch/tasks.jsonl has tasks");
}
