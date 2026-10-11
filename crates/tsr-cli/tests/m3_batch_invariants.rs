//! The batch runner's recorded sessions satisfy `tsr_cli::reconstruct::Session`'s invariants.
//!
//! `tsr-batch` cannot depend on `tsr-cli` (the CLI depends on the batch crate), so the invariant
//! check lives here, in the crate that owns `reconstruct::Session`. This test drives the five
//! fixture tasks through the scripted driver in both systems, then rebuilds each session from its
//! store and asserts every invariant the PRD names (B4, B5, B6).

use std::path::{Path, PathBuf};

use tsr_batch::driver::{read_session, Mode, ScriptedDriver};
use tsr_batch::run::{run_all, BatchConfig};
use tsr_batch::task::read_tasks;
use tsr_cli::reconstruct::Session;
use tsr_corpus::ScriptedRunner;
use tsr_domain::Hash;
use tsr_policy::profile::Profile;
use tsr_store::Store;

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn fixtures() -> PathBuf {
    workspace().join("fixtures/m3/batch")
}

fn profile() -> Profile {
    Profile::load(&workspace().join("profiles/default.toml")).unwrap()
}

/// Rebuild a session from a store's log for a task, the way `reconstruct::from_events` will.
fn session_from_store(store: &Store, task: &tsr_domain::TaskId) -> Session {
    Session {
        events: store.by_task(task).unwrap(),
    }
}

fn check(session: &Session, label: &str) {
    assert!(session.monotone_seq(), "{label}: sequence is not monotone");
    assert!(
        session.every_parent_precedes_its_child(),
        "{label}: a parent does not precede its child"
    );
    assert!(
        session.every_gated_action_has_prior_forecast_at_same_state_version(),
        "{label}: a gated action has no prior forecast at the same state version"
    );
    assert!(
        session.every_dispatch_reconciled_or_unresolved(),
        "{label}: a dispatch was neither executed nor unresolved"
    );
    assert!(
        session.every_command_ran_in_sandbox(),
        "{label}: a command ran outside the sealed sandbox"
    );
}

fn run_system(mode: Mode) -> (tempfile::TempDir, Vec<tsr_batch::TaskResult>) {
    let tasks = read_tasks(&fixtures().join("tasks.jsonl")).unwrap();
    let dir = tempfile::tempdir().unwrap();
    let config = BatchConfig::new(dir.path(), profile());
    let runner = ScriptedRunner::ok();
    let results = run_all(
        &tasks,
        mode,
        |task| {
            let path =
                fixtures()
                    .join("sessions")
                    .join(format!("{}-{}.json", task.id, mode.system()));
            let session = read_session(&path).unwrap();
            Box::new(ScriptedDriver::new(mode, session))
        },
        &runner,
        &config,
    )
    .unwrap();
    (dir, results)
}

#[test]
fn scripted_runs_satisfy_reconstruct_invariants() {
    for mode in [Mode::Record, Mode::Rules] {
        let (dir, results) = run_system(mode);
        assert_eq!(results.len(), 5, "five fixture tasks");
        for r in &results {
            let store_path = dir
                .path()
                .join("state")
                .join(&r.task)
                .join(format!("{}.sqlite", mode.system()));
            let store = Store::open(&store_path).unwrap();
            let task_id = tsr_domain::TaskId(Hash::of(&[r.task.as_bytes()]));
            let session = session_from_store(&store, &task_id);
            assert!(!session.events.is_empty(), "{}: empty session", r.task);
            check(&session, &format!("{}/{}", mode.system(), r.task));
        }
    }
}

#[test]
fn rules_mode_records_a_denial_with_its_reason() {
    // The missing-forecast session: a gated `RunApprovedTest` with no forecast at the current state
    // version is denied, and the denial carries the reason that says what to record.
    let tasks = read_tasks(&fixtures().join("tasks.jsonl")).unwrap();
    let task = &tasks[0];
    let dir = tempfile::tempdir().unwrap();
    let config = BatchConfig::new(dir.path(), profile());
    let runner = ScriptedRunner::ok();
    let session = read_session(&fixtures().join("session-missing-forecast.json")).unwrap();
    let driver = ScriptedDriver::new(Mode::Rules, session);
    let r = tsr_batch::run::run_one(task, Mode::Rules, &driver, &runner, &config).unwrap();

    assert_eq!(r.denials.len(), 1, "exactly one denial: {:?}", r.denials);
    let reason = &r.denials[0].reason;
    assert!(
        reason.contains("record a forecast for RunApprovedTest"),
        "the denial says what to record, got: {reason}"
    );
    assert!(
        reason.contains("state version"),
        "the denial names the state version, got: {reason}"
    );

    // The denied action did not happen, and the session still satisfies the invariants.
    let store_path = dir.path().join("state").join(&task.id).join("rules.sqlite");
    let store = Store::open(&store_path).unwrap();
    let task_id = tsr_domain::TaskId(Hash::of(&[task.id.as_bytes()]));
    let session = session_from_store(&store, &task_id);
    check(&session, "rules/denial");
}
