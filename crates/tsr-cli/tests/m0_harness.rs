//! M0 harness test: drive realistic Claude Code hook payloads through the *built binary* and prove
//! the committed fixture is exactly what the hook produces.
//!
//! The goal asks for a harness that drives realistic payloads through `tsr hook` and writes
//! `fixtures/m0/session.events.jsonl`. `tsr hook --selftest` is that harness; this test is its
//! reproducible form. It does not write JSON by hand — it runs the binary and compares bytes.
//!
//! Two things are checked:
//! 1. `tsr hook --selftest` is reproducible **structurally**: two runs agree on the sequence of
//!    `(seq, kind, candidate id, forecast state_version, in_sandbox, exit_code, log_digest)`.
//!    `duration_ms` is excluded **by name** — it is wall-clock, not a function of the inputs. The
//!    harness replay through the `FakeRunner` is the reference; the real selftest on a host is
//!    compared against it, and the test **skips with a printed reason** when no container runtime is
//!    reachable (`exec::Runtime::detect()` is `None`).
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
// The selftest reproducibility test (structural; skips without a container runtime)
// ---------------------------------------------------------------------------------------------

/// One event reduced to the fields that must be reproducible across two selftests.
///
/// `duration_ms` is deliberately absent: it is wall-clock, so it differs between runs and is
/// excluded **by name** (see `selftest_is_reproducible_structurally_excluding_duration_ms_by_name`).
/// Everything else — the sequence position, the kind, the candidate id, the forecast's state
/// version, whether the dispatch was in the sandbox, the exit code and the log digest — is a
/// function of the inputs and must be identical.
#[derive(Debug, PartialEq, Eq)]
struct StructuralEvent {
    seq: i64,
    kind: String,
    candidate: Option<String>,
    forecast_state_version: Option<u64>,
    in_sandbox: Option<bool>,
    exit_code: Option<i32>,
    log_digest: Option<String>,
}

impl StructuralEvent {
    fn of(seq: i64, kind: &EventKind) -> StructuralEvent {
        let mut s = StructuralEvent {
            seq,
            kind: kind_name(kind),
            candidate: None,
            forecast_state_version: None,
            in_sandbox: None,
            exit_code: None,
            log_digest: None,
        };
        match kind {
            EventKind::Proposed { candidate } => s.candidate = Some(candidate.id.0.to_string()),
            EventKind::Generate { candidate } => s.candidate = Some(candidate.0.to_string()),
            EventKind::Forecast {
                forecast,
                candidate,
            } => {
                s.candidate = Some(candidate.0.to_string());
                s.forecast_state_version = Some(forecast.state_version);
            }
            EventKind::Committed { candidate, .. } => s.candidate = Some(candidate.0.to_string()),
            EventKind::Dispatched {
                candidate,
                in_sandbox,
            } => {
                s.candidate = Some(candidate.0.to_string());
                s.in_sandbox = Some(*in_sandbox);
            }
            EventKind::Executed {
                candidate,
                exit_code,
                log_digest,
                // duration_ms is excluded by name: it is wall-clock, not a function of the inputs.
                ..
            } => {
                s.candidate = Some(candidate.0.to_string());
                s.exit_code = *exit_code;
                s.log_digest = log_digest.map(|h| h.to_string());
            }
            EventKind::Unresolved { candidate, .. } => s.candidate = Some(candidate.0.to_string()),
            _ => {}
        }
        s
    }
}

/// The structural trace of a task's events, in sequence order.
fn structural_trace(store: &Store, task: &TaskId) -> Vec<StructuralEvent> {
    store
        .by_task(task)
        .expect("by_task")
        .into_iter()
        .map(|(seq, ev)| StructuralEvent::of(seq, &ev.kind))
        .collect()
}

/// The structural trace of a fixture JSONL file, in sequence order.
fn structural_trace_of_fixture(path: &Path) -> Vec<StructuralEvent> {
    let text = std::fs::read_to_string(path).expect("read fixture");
    text.lines()
        .filter(|l| !l.trim().is_empty())
        .map(|line| {
            let v: serde_json::Value = serde_json::from_str(line).expect("fixture line is JSON");
            let seq = v["seq"].as_i64().expect("seq");
            let ev: tsr_domain::Event = serde_json::from_value(v["event"].clone()).expect("event");
            StructuralEvent::of(seq, &ev.kind)
        })
        .collect()
}

/// Run `tsr hook --selftest --out <cwd>/out` with `cwd` as the working directory, returning the
/// produced trace. The selftest writes to the named directory, never into `fixtures/m0`.
fn run_selftest(cwd: &Path) -> Vec<u8> {
    let out_dir = cwd.join("out");
    let out = Command::new(tsr_bin())
        .args(["hook", "--selftest", "--out"])
        .arg(&out_dir)
        .current_dir(cwd)
        .output()
        .expect("run tsr hook --selftest");
    assert!(
        out.status.success(),
        "tsr hook --selftest failed: status={:?} stderr={}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    std::fs::read(out_dir.join("session.events.jsonl")).expect("selftest wrote the trace")
}

/// Two selftests are reproducible when their structural traces agree — `duration_ms` is excluded
/// **by name** because it is wall-clock and cannot be reproduced, while every other field is a
/// function of the inputs.
///
/// The two runs compared are two harness replays through the `FakeRunner` (always available), and —
/// when a container runtime is reachable — the real selftest on the host, which now writes to
/// `--out <scratch>/out` and never into `fixtures/m0`. The committed fixture is deliberately *not*
/// compared here: it is the operator's real record, regenerated on a host after this repair, and
/// the frozen `tests/m0_reconstruct.rs` is what turns green on it.
#[test]
fn selftest_is_reproducible_structurally_excluding_duration_ms_by_name() {
    // Two independent harness replays through the FakeRunner: the same calls the selftest makes,
    // with the container invocation injected. They must agree structurally.
    let (store_a, task_a, _dir_a) = replay_selftest("repro-fake-a");
    let (store_b, task_b, _dir_b) = replay_selftest("repro-fake-b");
    let fake_a = structural_trace(&store_a, &task_a);
    let fake_b = structural_trace(&store_b, &task_b);
    assert_eq!(fake_a.len(), 19, "the reference trace has 19 events");
    assert_eq!(
        fake_a, fake_b,
        "two FakeRunner replays are not reproducible structurally"
    );

    // Without a container runtime the real selftest cannot run; skip that half with a reason.
    if exec::Runtime::detect().is_none() {
        eprintln!(
            "skipping the real-runner half: no container runtime reachable; \
             the two FakeRunner replays agreed structurally"
        );
        return;
    }

    // Two independent real runs in two scratch directories must agree structurally with each other
    // and with the FakeRunner replay. A wall-clock timestamp in the id input would break this.
    let _ = run_selftest(&scratch("run1"));
    let first = structural_trace_of_fixture(&scratch("run1").join("out/session.events.jsonl"));
    let _ = run_selftest(&scratch("run2"));
    let second = structural_trace_of_fixture(&scratch("run2").join("out/session.events.jsonl"));

    assert_eq!(
        first, second,
        "the harness is not reproducible: two runs produced different structural traces"
    );
    assert_eq!(
        first, fake_a,
        "the real selftest and the FakeRunner replay disagree structurally"
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

// ---------------------------------------------------------------------------------------------
// The selftest sequence, replayed through the library with a FakeRunner (no container needed)
// ---------------------------------------------------------------------------------------------

/// The argv the selftest runs in the container: `cargo test --offline -p parser`.
const SELFTEST_TEST_ARGV: [&str; 5] = ["cargo", "test", "--offline", "-p", "parser"];
/// The command string the forecast and the hook payload name for a run.
const SELFTEST_TEST_CMD: &str = "cargo test --offline -p parser";

/// A `PreToolUse` payload for a tool call, as a parsed [`HookPayload`].
fn pre(session: &str, tool: &str, input: serde_json::Value) -> HookPayload {
    HookPayload::parse(&pretooluse(session, tool, input))
}

/// A `PostToolUse` payload for a `Bash` call, as a parsed [`HookPayload`].
fn post_bash(session: &str, cmd: &str, exit_code: i32) -> HookPayload {
    HookPayload::parse(
        &serde_json::json!({
            "session_id": session,
            "hook_event_name": "PostToolUse",
            "tool_name": "Bash",
            "tool_input": { "command": cmd },
            "tool_response": { "exit_code": exit_code, "stdout": "test result: ok" },
        })
        .to_string(),
    )
}

/// Replay the selftest's exact sequence through the library, with the container invocation injected
/// as a [`exec::FakeRunner`]. Returns the store, the task and the scratch directory.
///
/// This is `hook::selftest_inner` with `exec::run` replaced by `exec::run_with(…, &FakeRunner)`: the
/// same calls, in the same order, so the resulting log is the log the selftest would write on a host
/// with a container runtime — without needing one.
fn replay_selftest(name: &str) -> (Store, TaskId, PathBuf) {
    let (store, dir) = fresh_store(name);
    let session = "m0-selftest-session";
    let task = store_path::task_for_session(session);
    let workspace = dir.clone();

    // 1. The task is opened.
    let start = HookPayload::parse(
        &serde_json::json!({
            "session_id": session,
            "hook_event_name": "SessionStart",
            "cwd": workspace.display().to_string(),
        })
        .to_string(),
    );
    hook::record_session_start(&store, &start).expect("record session start");

    // 2. Two inspections.
    for path in ["crates/tsr-cli/src/lib.rs", "crates/tsr-store/src/lib.rs"] {
        hook::record_tool(
            &store,
            &pre(session, "Read", serde_json::json!({ "file_path": path })),
        )
        .expect("record read");
    }

    // 3. A forecast before the first gated action.
    hook::record_forecast_for_action(
        &store,
        &task,
        ActionKind::RunApprovedTest,
        SELFTEST_TEST_CMD,
        0.7,
        "generator",
    )
    .expect("record forecast");

    // 4. The first run: the agent's Bash call is `tsr exec -- cargo test …`. The hook records the
    //    commitment; `tsr exec` records the dispatch and the execution; the PostToolUse reconciles
    //    nothing further.
    let exec_cmd = format!("tsr exec -- {SELFTEST_TEST_CMD}");
    let argv: Vec<String> = SELFTEST_TEST_ARGV.iter().map(|s| s.to_string()).collect();
    hook::record_tool(
        &store,
        &pre(session, "Bash", serde_json::json!({ "command": exec_cmd })),
    )
    .expect("record exec pre");
    exec::run_with(
        &store,
        &task,
        &workspace,
        &argv,
        &exec::FakeRunner::new(0, "test result: ok"),
    )
    .expect("fake exec run");
    hook::record_post_tool(&store, &post_bash(session, &exec_cmd, 0)).expect("record exec post");

    // 5. A forecast before the patch proposal, then the proposal itself (gated).
    hook::record_forecast_for_action(
        &store,
        &task,
        ActionKind::ProposePatch,
        "crates/tsr-cli/src/lib.rs",
        0.6,
        "generator",
    )
    .expect("record patch forecast");
    hook::record_tool(
        &store,
        &pre(
            session,
            "Edit",
            serde_json::json!({ "file_path": "crates/tsr-cli/src/lib.rs" }),
        ),
    )
    .expect("record edit");

    // 6. A forecast before the second gated action, then the second run the same way.
    hook::record_forecast_for_action(
        &store,
        &task,
        ActionKind::RunApprovedTest,
        SELFTEST_TEST_CMD,
        0.6,
        "generator",
    )
    .expect("record second forecast");
    hook::record_tool(
        &store,
        &pre(session, "Bash", serde_json::json!({ "command": exec_cmd })),
    )
    .expect("record second exec pre");
    exec::run_with(
        &store,
        &task,
        &workspace,
        &argv,
        &exec::FakeRunner::new(0, "test result: ok"),
    )
    .expect("second fake exec run");
    hook::record_post_tool(&store, &post_bash(session, &exec_cmd, 0))
        .expect("record second exec post");

    // 7. A stop.
    let stop = HookPayload::parse(
        &serde_json::json!({
            "session_id": session,
            "hook_event_name": "Stop",
            "stop_hook_active": false,
        })
        .to_string(),
    );
    hook::record_stop(&store, &stop).expect("record stop");

    (store, task, dir)
}

/// The selftest sequence replayed through the FakeRunner has exactly 19 events and satisfies every
/// `reconstruct::Session` invariant — in particular B5, the one the real record failed.
#[test]
fn selftest_sequence_through_the_fake_runner_has_19_events_and_holds_every_invariant() {
    let (store, task, _dir) = replay_selftest("replay-selftest");

    let ks = kinds(&store, &task);
    let tags: Vec<&str> = ks.iter().map(|(_, k)| k.as_str()).collect();
    eprintln!("replayed selftest event kinds: {tags:?}");

    assert_eq!(
        tags,
        vec![
            "task_opened",
            "proposed",
            "committed",
            "proposed",
            "committed",
            "proposed",
            "forecast",
            "committed",
            "dispatched",
            "executed",
            "proposed",
            "forecast",
            "committed",
            "proposed",
            "forecast",
            "committed",
            "dispatched",
            "executed",
            "stop",
        ],
        "the repaired selftest sequence is exactly 19 events"
    );
    assert_eq!(ks.len(), 19, "exactly 19 events");

    let session = reconstruct_task(&store, &task);
    assert_eq!(session.events.len(), 19);
    assert!(session.monotone_seq(), "sequence is monotone");
    assert!(
        session.every_parent_precedes_its_child(),
        "every parent precedes its child"
    );
    assert!(
        session.every_gated_action_has_prior_forecast_at_same_state_version(),
        "B5: every gated action has a forecast at its commitment's state version"
    );
    assert!(
        session.every_dispatch_reconciled_or_unresolved(),
        "B6: every dispatch is reconciled"
    );
    assert!(
        session.every_command_ran_in_sandbox(),
        "B4: every command ran in the sealed sandbox"
    );
}

/// No `Proposed` or `Committed` follows a `tsr exec` PostToolUse, and the second run dispatches the
/// candidate committed at its own `PreToolUse` — not a freshly minted one.
#[test]
fn no_proposal_after_a_tsr_exec_post_tool_and_the_second_run_reuses_its_commitment() {
    let (store, task, _dir) = replay_selftest("replay-no-proposal");

    let events = store.by_task(&task).expect("by_task");

    // The two `tsr exec` PostToolUse calls append nothing: the log is unchanged across each. Replay
    // the sequence and compare the log length before and after each PostToolUse.
    let exec_cmd = format!("tsr exec -- {SELFTEST_TEST_CMD}");
    let (store2, task2, _dir2) = replay_selftest("replay-no-proposal-2");
    let before = store2.by_task(&task2).expect("by_task").len();
    hook::record_post_tool(&store2, &post_bash("m0-selftest-session", &exec_cmd, 0))
        .expect("record exec post");
    let after = store2.by_task(&task2).expect("by_task").len();
    assert_eq!(
        before, after,
        "a tsr exec PostToolUse appends nothing: no Proposed/Committed after it"
    );

    assert_eq!(
        count_of(&store, &task, "proposed"),
        5,
        "two inspections + three gated actions: five proposals, none from a PostToolUse"
    );

    // The second run dispatches the candidate committed at its own PreToolUse. The two runs share a
    // candidate id (same kind + args), so the check is on the *occurrence*: the second Dispatched
    // must follow the second Committed, and the second Executed must follow that Dispatched.
    let committed: Vec<usize> = events
        .iter()
        .enumerate()
        .filter_map(|(i, (_, e))| match &e.kind {
            EventKind::Committed { .. } => Some(i),
            _ => None,
        })
        .collect();
    let dispatched: Vec<usize> = events
        .iter()
        .enumerate()
        .filter_map(|(i, (_, e))| match &e.kind {
            EventKind::Dispatched { .. } => Some(i),
            _ => None,
        })
        .collect();
    assert_eq!(committed.len(), 5, "five commitments");
    assert_eq!(dispatched.len(), 2, "two dispatches");
    // The second dispatch follows the fifth commitment (the second run's PreToolUse), not a fresh
    // proposal: no Proposed sits between them.
    assert!(
        committed[4] < dispatched[1],
        "the second dispatch follows the second run's commitment"
    );
    assert!(
        !events[committed[4] + 1..dispatched[1]]
            .iter()
            .any(|(_, e)| matches!(e.kind, EventKind::Proposed { .. })),
        "the second run mints no fresh candidate between its commitment and its dispatch"
    );
}

/// A host `Bash` (`cargo test …` without `tsr exec`) yields `Dispatched { in_sandbox: false }` at
/// `PreToolUse` and `Executed` at `PostToolUse`, with no new `Proposed`.
#[test]
fn host_bash_yields_a_host_dispatch_and_an_executed_post_tool_with_no_new_proposal() {
    let (store, _dir) = fresh_store("replay-host-bash");
    let session = "replay-host-bash";
    let task = store_path::task_for_session(session);
    let cmd = SELFTEST_TEST_CMD;

    // A forecast first, so the gated action is legitimate.
    hook::record_forecast_for_action(
        &store,
        &task,
        ActionKind::RunApprovedTest,
        cmd,
        0.7,
        "generator",
    )
    .expect("record forecast");

    hook::record_tool(
        &store,
        &pre(session, "Bash", serde_json::json!({ "command": cmd })),
    )
    .expect("record host pre");

    let ks = kinds(&store, &task);
    let tags: Vec<&str> = ks.iter().map(|(_, k)| k.as_str()).collect();
    assert_eq!(
        tags,
        vec!["proposed", "forecast", "committed", "dispatched"],
        "a host Bash run is dispatched on the host at PreToolUse"
    );
    let in_sandbox: Vec<bool> = store
        .by_task(&task)
        .expect("by_task")
        .into_iter()
        .filter_map(|(_, ev)| match ev.kind {
            EventKind::Dispatched { in_sandbox, .. } => Some(in_sandbox),
            _ => None,
        })
        .collect();
    assert_eq!(
        in_sandbox,
        vec![false],
        "the host run is not in the sandbox"
    );

    // The PostToolUse reconciles the commitment: Executed, and no new Proposed.
    hook::record_post_tool(&store, &post_bash(session, cmd, 0)).expect("record host post");
    let ks = kinds(&store, &task);
    let tags: Vec<&str> = ks.iter().map(|(_, k)| k.as_str()).collect();
    assert_eq!(
        tags,
        vec![
            "proposed",
            "forecast",
            "committed",
            "dispatched",
            "executed"
        ],
        "the PostToolUse records Executed and no new Proposed"
    );
    assert_eq!(count_of(&store, &task, "proposed"), 1, "no new Proposed");
    assert_eq!(count_of(&store, &task, "committed"), 1, "no new Committed");
}
