//! M0 harness test: drive realistic Claude Code hook payloads through the *built binary* and prove
//! the committed fixture is exactly what the hook produces.
//!
//! The goal asks for a harness that drives realistic payloads through `tsr hook` and writes
//! `fixtures/m0/session.events.jsonl`. `tsr hook --selftest` is that harness; this test is its
//! reproducible form. It does not write JSON by hand — it runs the binary and compares bytes.
//!
//! Two things are checked:
//! 1. `tsr hook --selftest` run in a scratch directory reproduces the committed fixture
//!    byte-for-byte. This is the reproducibility the milestone requires: `recorded_at` is hashed
//!    into every event id, so the harness pins the clock (`SELFTEST_RECORDED_AT`). The selftest
//!    runs a real container, so this test **skips with a printed reason** when no container runtime
//!    is reachable (`exec::Runtime::detect()` is `None`).
//! 2. A real PreToolUse payload piped on stdin is recorded and the process exits 0 — the hook
//!    records and never blocks (M0).
//!
//! The honest-record tests (item 5 of the M0 repair) drive real payloads through the hook and
//! assert what the log says: a forecast recorded *before* a gated action lines up with the
//! commitment's state version; a gated action with no forecast fails the invariant; a host `Bash`
//! run is recorded `in_sandbox: false`; a `tsr exec -- …` call is not dispatched by the hook.

use std::path::{Path, PathBuf};
use std::process::Command;

use tsr_cli::hook::{self, HookPayload};
use tsr_cli::{exec, reconstruct, store_path};
use tsr_domain::{ActionKind, EventKind, TaskId};
use tsr_store::Store;

/// The `tsr` binary cargo built for this test.
fn tsr_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_tsr"))
}

/// The committed fixture, relative to the workspace root.
fn committed_fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/m0/session.events.jsonl")
}

/// A scratch directory under the target dir (never `/tmp`, which this host prunes).
fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/m0-harness")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

/// Open a fresh store in a scratch directory, returning it and the directory.
fn fresh_store(name: &str) -> (Store, PathBuf) {
    let dir = scratch(name);
    let db = dir.join("tsr.db");
    let store = store_path::open_store_at(&db).expect("open store");
    (store, dir)
}

/// A PreToolUse payload for a tool call, as JSON (the shape Claude Code sends).
fn pretooluse(session: &str, tool: &str, input: serde_json::Value) -> String {
    serde_json::json!({
        "session_id": session,
        "hook_event_name": "PreToolUse",
        "tool_name": tool,
        "tool_input": input,
        "cwd": "/workspace",
    })
    .to_string()
}

/// The (seq, kind) pairs of a task, in sequence order.
fn kinds(store: &Store, task: &TaskId) -> Vec<(i64, String)> {
    store
        .by_task(task)
        .expect("by_task")
        .into_iter()
        .map(|(seq, ev)| (seq, kind_name(&ev.kind)))
        .collect()
}

/// The `kind` tag of an event kind, as serde writes it.
fn kind_name(kind: &EventKind) -> String {
    let v = serde_json::to_value(kind).expect("serialize kind");
    v["kind"].as_str().expect("kind tag").to_string()
}

/// The index (0-based) of the first event whose kind tag is `name`.
fn index_of(store: &Store, task: &TaskId, name: &str) -> usize {
    kinds(store, task)
        .iter()
        .position(|(_, k)| k == name)
        .unwrap_or_else(|| panic!("no {name} event in the task"))
}

/// The number of events whose kind tag is `name`.
fn count_of(store: &Store, task: &TaskId, name: &str) -> usize {
    kinds(store, task).iter().filter(|(_, k)| k == name).count()
}

/// The `state_version` of the first `Forecast` event in the task.
fn first_forecast_state_version(store: &Store, task: &TaskId) -> u64 {
    for (_, ev) in store.by_task(task).expect("by_task") {
        if let EventKind::Forecast { forecast, .. } = &ev.kind {
            return forecast.state_version;
        }
    }
    panic!("no Forecast event in the task");
}

/// Reconstruct a session from the task's exported trace and run the M0 invariants.
fn reconstruct_task(store: &Store, task: &TaskId) -> reconstruct::Session {
    let jsonl = tsr_cli::task::export(store, task).expect("export");
    let dir = scratch(&format!("reconstruct-{task}"));
    let path = dir.join("events.jsonl");
    std::fs::write(&path, jsonl).expect("write trace");
    reconstruct::from_events(&path).expect("reconstruct from events.jsonl alone")
}

// ---------------------------------------------------------------------------------------------
// The selftest reproducibility test (skips without a container runtime)
// ---------------------------------------------------------------------------------------------

/// Run `tsr hook --selftest` with `cwd` as the working directory, returning the produced fixture.
fn run_selftest(cwd: &Path) -> Vec<u8> {
    let out = Command::new(tsr_bin())
        .args(["hook", "--selftest"])
        .current_dir(cwd)
        .output()
        .expect("run tsr hook --selftest");
    assert!(
        out.status.success(),
        "tsr hook --selftest failed: status={:?} stderr={}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    std::fs::read(cwd.join("fixtures/m0/session.events.jsonl")).expect("selftest wrote the fixture")
}

#[test]
fn selftest_reproduces_the_committed_fixture_byte_for_byte() {
    // The selftest runs a real command in the sealed container, so it can only reproduce the
    // fixture where a container runtime is reachable. Without one, skip with a printed reason
    // rather than fail: the operator regenerates the fixture on a host with Docker.
    if exec::Runtime::detect().is_none() {
        eprintln!("skipping: no container runtime reachable");
        return;
    }

    let committed = std::fs::read(committed_fixture()).expect("committed fixture exists");
    assert!(!committed.is_empty(), "the committed fixture is not empty");

    // Two independent runs in two scratch directories must agree with each other and with the
    // committed file. A wall-clock timestamp in the id input would break this.
    let first = run_selftest(&scratch("run1"));
    let second = run_selftest(&scratch("run2"));

    assert_eq!(
        first, second,
        "the harness is not reproducible: two runs produced different bytes"
    );
    assert_eq!(
        first, committed,
        "the committed fixture is stale: regenerate it with `tsr hook --selftest`"
    );
}

#[test]
fn selftest_without_a_runtime_exits_3_and_writes_nothing() {
    if exec::Runtime::detect().is_some() {
        eprintln!("skipping: a container runtime is reachable, so the refusal path is not taken");
        return;
    }
    let dir = scratch("selftest-refusal");
    let out = Command::new(tsr_bin())
        .args(["hook", "--selftest"])
        .current_dir(&dir)
        .output()
        .expect("run tsr hook --selftest");
    assert_eq!(
        out.status.code(),
        Some(exec::EXIT_NO_RUNTIME),
        "no runtime → exit 3: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !dir.join("fixtures/m0/session.events.jsonl").exists(),
        "the selftest must not fabricate a fixture without a runtime"
    );
}

// ---------------------------------------------------------------------------------------------
// Honest-record tests: the log says only what happened (M0 repair, item 5)
// ---------------------------------------------------------------------------------------------

/// A forecast recorded before a gated tool call lines up with the commitment's state version.
///
/// `tsr forecast --kind run_approved_test --args "cargo test --offline -p parser" --p 0.7
/// --source generator` records `Proposed` + `Forecast`; the following PreToolUse Bash for the same
/// command records `Committed` only (the candidate was already proposed). The forecast's
/// `state_version` must equal the 0-based index of the `Committed` event, and there must be exactly
/// one `Forecast` — the hook never mints a second one.
#[test]
fn forecast_then_tool_call_yields_proposed_forecast_committed_and_no_second_forecast() {
    let (store, _dir) = fresh_store("forecast-then-call");
    let session = "honest-forecast";
    let task = store_path::task_for_session(session);
    let cmd = "cargo test --offline -p parser";

    // 1. The forecast enters the log first, naming the action.
    hook::record_forecast_for_action(
        &store,
        &task,
        ActionKind::RunApprovedTest,
        cmd,
        0.7,
        "generator",
    )
    .expect("record forecast");

    // 2. The tool call follows.
    let payload = pretooluse(session, "Bash", serde_json::json!({ "command": cmd }));
    hook::run(&store, "PreToolUse", &payload).expect("record tool call");

    let ks = kinds(&store, &task);
    let tags: Vec<&str> = ks.iter().map(|(_, k)| k.as_str()).collect();
    assert_eq!(
        tags,
        vec!["proposed", "forecast", "committed", "dispatched"],
        "forecast-then-tool-call records Proposed, Forecast, Committed, then the host dispatch"
    );

    let committed_index = index_of(&store, &task, "committed");
    assert_eq!(
        first_forecast_state_version(&store, &task),
        committed_index as u64,
        "the forecast's state_version equals the index of the Committed event"
    );
    assert_eq!(
        count_of(&store, &task, "forecast"),
        1,
        "the hook mints no second Forecast"
    );

    // The session satisfies the B5 invariant: a forecast exists at the commitment's version.
    let session = reconstruct_task(&store, &task);
    assert!(
        session.every_gated_action_has_prior_forecast_at_same_state_version(),
        "a forecast recorded before the gated action satisfies B5"
    );
}

/// A gated tool call with no prior forecast records `Proposed` + `Committed` and fails B5.
#[test]
fn gated_tool_call_without_a_forecast_fails_the_b5_invariant() {
    let (store, _dir) = fresh_store("no-forecast");
    let session = "honest-no-forecast";
    let task = store_path::task_for_session(session);

    let payload = pretooluse(
        session,
        "Bash",
        serde_json::json!({ "command": "cargo test --offline -p parser" }),
    );
    hook::run(&store, "PreToolUse", &payload).expect("record tool call");

    let ks = kinds(&store, &task);
    let tags: Vec<&str> = ks.iter().map(|(_, k)| k.as_str()).collect();
    assert_eq!(
        tags,
        vec!["proposed", "committed", "dispatched"],
        "no prior forecast → Proposed + Committed + the host dispatch, and no forecast"
    );
    assert_eq!(
        count_of(&store, &task, "forecast"),
        0,
        "the hook never constructs a Forecast"
    );

    let session = reconstruct_task(&store, &task);
    assert!(
        !session.every_gated_action_has_prior_forecast_at_same_state_version(),
        "a gated action with no prior forecast must fail B5 — that is the truth"
    );
}

/// A raw `cargo test` Bash run is a host run: `Dispatched { in_sandbox: false }`, and B4 fails.
#[test]
fn raw_cargo_test_bash_is_dispatched_on_the_host_and_fails_b4() {
    let (store, _dir) = fresh_store("host-run");
    let session = "honest-host-run";
    let task = store_path::task_for_session(session);

    let payload = pretooluse(
        session,
        "Bash",
        serde_json::json!({ "command": "cargo test --offline -p parser" }),
    );
    hook::run(&store, "PreToolUse", &payload).expect("record tool call");

    let dispatched: Vec<bool> = store
        .by_task(&task)
        .expect("by_task")
        .into_iter()
        .filter_map(|(_, ev)| match ev.kind {
            EventKind::Dispatched { in_sandbox, .. } => Some(in_sandbox),
            _ => None,
        })
        .collect();
    assert_eq!(
        dispatched,
        vec![false],
        "a raw Bash run is observed on the host: Dispatched {{ in_sandbox: false }}"
    );

    let session = reconstruct_task(&store, &task);
    assert!(
        !session.every_command_ran_in_sandbox(),
        "a host run must fail B4 — the hook cannot claim the sandbox"
    );
}

/// A `tsr exec -- …` Bash call is dispatched by `tsr exec`, not by the hook.
#[test]
fn tsr_exec_bash_is_committed_and_not_dispatched_by_the_hook() {
    let (store, _dir) = fresh_store("exec-call");
    let session = "honest-exec-call";
    let task = store_path::task_for_session(session);

    let payload = pretooluse(
        session,
        "Bash",
        serde_json::json!({ "command": "tsr exec -- cargo test --offline -p parser" }),
    );
    hook::run(&store, "PreToolUse", &payload).expect("record tool call");

    let ks = kinds(&store, &task);
    let tags: Vec<&str> = ks.iter().map(|(_, k)| k.as_str()).collect();
    assert_eq!(
        tags,
        vec!["proposed", "committed"],
        "a `tsr exec -- …` call is committed by the hook and dispatched by tsr exec"
    );
    assert_eq!(
        count_of(&store, &task, "dispatched"),
        0,
        "the hook records no Dispatched for a `tsr exec -- …` call"
    );
}

/// A `tsr exec -- …` call with a path prefix on `tsr` is still recognised as an exec invocation.
#[test]
fn tsr_exec_with_a_path_prefix_is_recognised() {
    let (store, _dir) = fresh_store("exec-prefix");
    let session = "honest-exec-prefix";
    let task = store_path::task_for_session(session);

    let payload = pretooluse(
        session,
        "Bash",
        serde_json::json!({ "command": "./target/debug/tsr exec -- cargo test --offline -p parser" }),
    );
    hook::run(&store, "PreToolUse", &payload).expect("record tool call");

    assert_eq!(
        count_of(&store, &task, "dispatched"),
        0,
        "a path-prefixed `tsr exec` is not dispatched by the hook"
    );
}

/// `tsr forecast --kind … --args …` and the hook compute the same candidate id for the same call.
#[test]
fn forecast_cli_and_hook_agree_on_the_candidate_id() {
    let (store, dir) = fresh_store("candidate-id");
    let session = "honest-candidate-id";
    let cmd = "cargo test --offline -p parser";

    // The hook's candidate for the tool call.
    let payload = HookPayload::parse(&pretooluse(
        session,
        "Bash",
        serde_json::json!({ "command": cmd }),
    ));
    let hook_candidate = hook::candidate_for(&payload).expect("hook candidate");

    // The CLI's candidate: run `tsr forecast` against the same store and read the Proposed event.
    let db = dir.join("tsr.db");
    let out = Command::new(tsr_bin())
        .args([
            "forecast",
            "--kind",
            "run_approved_test",
            "--args",
            cmd,
            "--p",
            "0.7",
            "--source",
            "generator",
            "--task",
            session,
        ])
        .env("TSR_DB", &db)
        .output()
        .expect("run tsr forecast");
    assert!(
        out.status.success(),
        "tsr forecast succeeds: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );

    // The CLI resolves `--task <name>` the same way `tsr task` does; mirror that resolution.
    let cli_task = store_path::resolve_task(&store, session);
    let cli_candidate = store
        .by_task(&cli_task)
        .expect("by_task")
        .into_iter()
        .find_map(|(_, ev)| match ev.kind {
            EventKind::Proposed { candidate } => Some(candidate.id),
            _ => None,
        })
        .expect("tsr forecast recorded a Proposed candidate");

    assert_eq!(
        hook_candidate, cli_candidate,
        "`tsr forecast --kind … --args …` and the hook mint the same candidate id"
    );
    assert_eq!(
        hook::candidate_for_action(ActionKind::RunApprovedTest, cmd).expect("candidate_for_action"),
        cli_candidate,
        "the shared digest function mints the same candidate id from both paths"
    );
}

/// A `tsr exec -- …` Bash call commits to the *inner* command, so a forecast naming the bare
/// command matches it: the hook records `Committed` only and B5 holds.
///
/// Regression: the hook used to digest the full `tsr exec -- cargo test …` string, minting a
/// candidate the forecast had never named — a second `Proposed` + `Committed` with no prior
/// forecast at its state version.
#[test]
fn forecast_of_the_bare_command_matches_a_tsr_exec_bash_call() {
    let (store, _dir) = fresh_store("exec-forecast");
    let session = "honest-exec-forecast";
    let task = store_path::task_for_session(session);
    let cmd = "cargo test --offline -p parser";

    hook::record_forecast_for_action(
        &store,
        &task,
        ActionKind::RunApprovedTest,
        cmd,
        0.7,
        "generator",
    )
    .expect("record forecast");

    let payload = pretooluse(
        session,
        "Bash",
        serde_json::json!({ "command": format!("tsr exec -- {cmd}") }),
    );
    hook::run(&store, "PreToolUse", &payload).expect("record tool call");

    let ks = kinds(&store, &task);
    let tags: Vec<&str> = ks.iter().map(|(_, k)| k.as_str()).collect();
    assert_eq!(
        tags,
        vec!["proposed", "forecast", "committed"],
        "the forecast names the inner command, so the exec call commits without a second proposal"
    );
    assert_eq!(
        count_of(&store, &task, "proposed"),
        1,
        "the hook mints no second Proposed for a `tsr exec -- …` call"
    );
    assert_eq!(
        first_forecast_state_version(&store, &task),
        index_of(&store, &task, "committed") as u64,
        "the forecast's state_version equals the index of the Committed event"
    );
    assert!(
        reconstruct_task(&store, &task)
            .every_gated_action_has_prior_forecast_at_same_state_version(),
        "a forecast naming the inner command satisfies B5 for a `tsr exec -- …` call"
    );
}

/// A second run of the same command is proposed again, so it is pending again: the hook records
/// `Committed` only, at the state version the second forecast named.
#[test]
fn a_second_run_of_the_same_command_commits_without_a_second_proposal() {
    let (store, _dir) = fresh_store("second-run");
    let session = "honest-second-run";
    let task = store_path::task_for_session(session);
    let cmd = "cargo test --offline -p parser";

    for _ in 0..2 {
        hook::record_forecast_for_action(
            &store,
            &task,
            ActionKind::RunApprovedTest,
            cmd,
            0.6,
            "generator",
        )
        .expect("record forecast");
        let payload = pretooluse(
            session,
            "Bash",
            serde_json::json!({ "command": format!("tsr exec -- {cmd}") }),
        );
        hook::run(&store, "PreToolUse", &payload).expect("record tool call");
    }

    let ks = kinds(&store, &task);
    let tags: Vec<&str> = ks.iter().map(|(_, k)| k.as_str()).collect();
    assert_eq!(
        tags,
        vec![
            "proposed",
            "forecast",
            "committed",
            "proposed",
            "forecast",
            "committed"
        ],
        "each run is proposed once and committed once"
    );
    assert_eq!(count_of(&store, &task, "proposed"), 2);
    assert_eq!(count_of(&store, &task, "committed"), 2);
    assert!(
        reconstruct_task(&store, &task)
            .every_gated_action_has_prior_forecast_at_same_state_version(),
        "both commitments have a prior forecast at their state version"
    );
}
