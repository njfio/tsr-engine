//! D6: `tsr task run --mode rules` attaches the Tier 0 controller, and a budget stop ends *that*
//! task with `Stop(budget)` and `TaskClosed` while the batch continues with the next task.
//!
//! Pre-fix, `batch_cmd.rs` built `ScriptedDriver::new(mode, session)` without `with_tier0`, so a
//! rules-mode session that ran the failing test with no inspect and spent 100 tool-minutes against
//! the profile's 30 was accepted with 0 denials. And when the controller *was* attached, a budget
//! stop returned `Err` from `drive`, which `run_one` propagated, aborting the whole batch with no
//! `Stop(budget)`, no `TaskClosed`, and no other task run.
//!
//! These tests drive the built `tsr` binary with `--scripted-runner` (the oracle is a canned
//! success) so the only thing under test is the Tier 0 controller and the batch's continuation.

use std::path::{Path, PathBuf};
use std::process::Command;

use tsr_domain::{EventKind, Hash, TaskId};
use tsr_store::Store;

fn workspace() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn tsr_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_tsr"))
}

/// A mined task JSON object with the given id, family and flipped test.
fn task_json(id: &str, family: &str, flipped: &str) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "repo": "rts",
        "parent": "a1b2c3",
        "commit": "d4e5f6",
        "parent_snapshot": "0000000000000000000000000000000000000000",
        "flipped_tests": [flipped],
        "hidden_diff": "cas-diff-rts-1",
        "family": family,
        "lineage": "rts",
        "timestamp": "2026-01-01T00:00:00Z",
        "outcome": {
            "id": "0000000000000000000000000000000000000000000000000000000000000001",
            "name": "flip@v1",
            "version": 1,
            "horizon": "task_close",
            "resolves_true_when": "the flipped tests pass and the full suite does not regress",
            "censored_when": "the suite could not run",
            "intervention_rule": "the hidden diff stays hidden"
        }
    })
}

/// A rules session that inspects the failing assertion and its call site, runs the failing test,
/// then spends 100 tool-minutes in one step — past the profile's 30 — and tries to stop.
///
/// The controller refuses the step that would cross the budget (D6, F1), so the 100-minute step is
/// the one refused with `BudgetExhausted` and the following `stop` step never runs.
fn overspending_session() -> serde_json::Value {
    serde_json::json!({
        "name": "d6-overspend",
        "steps": [
            {
                "kind": "inspect",
                "args": { "arg": "inspect_path", "path": "src/parser.rs" },
                "forecast": 0.9, "forecast_event": "test_pass", "dispatch": false,
                "exit_code": null, "duration_ms": 0, "generator_tokens": 0,
                "tool_minutes": 0, "wall_clock_ms": 0
            },
            {
                "kind": "inspect",
                "args": { "arg": "inspect_path", "path": "tests/parser.rs" },
                "forecast": 0.9, "forecast_event": "test_pass", "dispatch": false,
                "exit_code": null, "duration_ms": 0, "generator_tokens": 0,
                "tool_minutes": 0, "wall_clock_ms": 0
            },
            {
                "kind": "run_approved_test",
                "args": { "arg": "test_id", "id": "cargo-test-parser" },
                "forecast": 0.4, "forecast_event": "test_pass", "dispatch": true,
                "exit_code": 101, "duration_ms": 1200, "generator_tokens": 0,
                "tool_minutes": 1, "wall_clock_ms": 1200
            },
            {
                "kind": "inspect",
                "args": { "arg": "inspect_path", "path": "src/parser.rs" },
                "forecast": 0.9, "forecast_event": "test_pass", "dispatch": false,
                "exit_code": null, "duration_ms": 0, "generator_tokens": 0,
                "tool_minutes": 100, "wall_clock_ms": 0
            },
            {
                "kind": "stop",
                "args": { "arg": "stop" },
                "forecast": null, "forecast_event": null, "dispatch": false,
                "exit_code": null, "duration_ms": 0, "generator_tokens": 0,
                "tool_minutes": 0, "wall_clock_ms": 0
            }
        ]
    })
}

/// A rules session whose *last* step is the one that crosses the budget (D6, F1).
///
/// The defect: the controller checked `spent >= budget` before adding the step's minutes, so the
/// crossing step was committed and only a later step was refused. With no later step, the session
/// was accepted at 101 tool-minutes against the profile's 30, with 0 denials.
fn overspending_last_session() -> serde_json::Value {
    serde_json::json!({
        "name": "d6-overspend-last",
        "steps": [
            {
                "kind": "inspect",
                "args": { "arg": "inspect_path", "path": "src/parser.rs" },
                "forecast": 0.9, "forecast_event": "test_pass", "dispatch": false,
                "exit_code": null, "duration_ms": 0, "generator_tokens": 0,
                "tool_minutes": 0, "wall_clock_ms": 0
            },
            {
                "kind": "inspect",
                "args": { "arg": "inspect_path", "path": "tests/parser.rs" },
                "forecast": 0.9, "forecast_event": "test_pass", "dispatch": false,
                "exit_code": null, "duration_ms": 0, "generator_tokens": 0,
                "tool_minutes": 0, "wall_clock_ms": 0
            },
            {
                "kind": "run_approved_test",
                "args": { "arg": "test_id", "id": "cargo-test-parser" },
                "forecast": 0.4, "forecast_event": "test_pass", "dispatch": true,
                "exit_code": 101, "duration_ms": 1200, "generator_tokens": 0,
                "tool_minutes": 1, "wall_clock_ms": 1200
            },
            {
                "kind": "inspect",
                "args": { "arg": "inspect_path", "path": "src/parser.rs" },
                "forecast": 0.9, "forecast_event": "test_pass", "dispatch": false,
                "exit_code": null, "duration_ms": 0, "generator_tokens": 0,
                "tool_minutes": 100, "wall_clock_ms": 0
            }
        ]
    })
}

/// A rules session that runs the failing test with no inspect: a read-before-test refusal (D6, F2).
fn read_before_test_violation_session() -> serde_json::Value {
    serde_json::json!({
        "name": "d6-read-before-test",
        "steps": [
            {
                "kind": "run_approved_test",
                "args": { "arg": "test_id", "id": "cargo-test-parser" },
                "forecast": 0.4, "forecast_event": "test_pass", "dispatch": true,
                "exit_code": 101, "duration_ms": 1200, "generator_tokens": 0,
                "tool_minutes": 1, "wall_clock_ms": 1200
            }
        ]
    })
}

/// A well-formed rules session that inspects the failing assertion and its call site for the
/// given module, runs the failing test, then stays inside the budget and stops cleanly.
fn within_budget_session(module: &str) -> serde_json::Value {
    serde_json::json!({
        "name": "d6-within-budget",
        "steps": [
            {
                "kind": "inspect",
                "args": { "arg": "inspect_path", "path": format!("src/{module}.rs") },
                "forecast": 0.9, "forecast_event": "test_pass", "dispatch": false,
                "exit_code": null, "duration_ms": 0, "generator_tokens": 0,
                "tool_minutes": 0, "wall_clock_ms": 0
            },
            {
                "kind": "inspect",
                "args": { "arg": "inspect_path", "path": format!("tests/{module}.rs") },
                "forecast": 0.9, "forecast_event": "test_pass", "dispatch": false,
                "exit_code": null, "duration_ms": 0, "generator_tokens": 0,
                "tool_minutes": 0, "wall_clock_ms": 0
            },
            {
                "kind": "run_approved_test",
                "args": { "arg": "test_id", "id": "cargo-test-parser" },
                "forecast": 0.4, "forecast_event": "test_pass", "dispatch": true,
                "exit_code": 101, "duration_ms": 1200, "generator_tokens": 0,
                "tool_minutes": 1, "wall_clock_ms": 1200
            },
            {
                "kind": "propose_patch",
                "args": { "arg": "patch_bytes", "bytes": 128 },
                "forecast": 0.7, "forecast_event": "diff_accepted", "dispatch": false,
                "exit_code": null, "duration_ms": 0, "generator_tokens": 0,
                "tool_minutes": 0, "wall_clock_ms": 0
            },
            {
                "kind": "run_approved_test",
                "args": { "arg": "test_id", "id": "cargo-test-parser" },
                "forecast": 0.85, "forecast_event": "test_pass", "dispatch": true,
                "exit_code": 0, "duration_ms": 1100, "generator_tokens": 0,
                "tool_minutes": 1, "wall_clock_ms": 1100
            },
            {
                "kind": "stop",
                "args": { "arg": "stop" },
                "forecast": null, "forecast_event": null, "dispatch": false,
                "exit_code": null, "duration_ms": 0, "generator_tokens": 0,
                "tool_minutes": 0, "wall_clock_ms": 0
            }
        ]
    })
}

/// The events of a task's store, read back through `tsr_store`.
fn events_of(out: &Path, task_id: &str) -> Vec<EventKind> {
    let store_path = out.join("state").join(task_id).join("rules.sqlite");
    let store = Store::open(&store_path)
        .unwrap_or_else(|e| panic!("opening {}: {e}", store_path.display()));
    let id = TaskId(Hash::of(&[task_id.as_bytes()]));
    store
        .by_task(&id)
        .expect("by_task")
        .into_iter()
        .map(|(_, e)| e.kind)
        .collect()
}

/// Whether the events contain a `Stop` with the given reason.
fn has_stop(events: &[EventKind], reason: &str) -> bool {
    events
        .iter()
        .any(|k| matches!(k, EventKind::Stop { reason: r } if r == reason))
}

/// Whether the events contain a `TaskClosed`.
fn has_task_closed(events: &[EventKind]) -> bool {
    events
        .iter()
        .any(|k| matches!(k, EventKind::TaskClosed { .. }))
}

/// D6(a): a rules-mode session that spends past the profile's budget is stopped with
/// `Stop(budget)` and `TaskClosed`, and is not accepted.
#[test]
fn d6_rules_budget_stop_ends_the_task() {
    let ws = workspace();
    let scratch = ws.join("target/d6-budget-stop");
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    let manifest = scratch.join("tasks.jsonl");
    std::fs::write(
        &manifest,
        format!(
            "{}\n",
            task_json("d6-budget", "parser", "parser::tests::handles_empty_input")
        ),
    )
    .unwrap();
    std::fs::write(
        scratch.join("session-rules.json"),
        serde_json::to_string_pretty(&overspending_session()).unwrap(),
    )
    .unwrap();
    let out = scratch.join("out");
    let profile = ws.join("profiles/default.toml");

    let output = Command::new(tsr_bin())
        .args([
            "task",
            "run",
            "--mode",
            "rules",
            "--tasks",
            manifest.to_str().unwrap(),
            "--driver",
            "scripted",
            "--out",
            out.to_str().unwrap(),
            "--profile",
            profile.to_str().unwrap(),
            "--scripted-runner",
        ])
        .output()
        .expect("run tsr");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(0),
        "D6: a budget stop must not abort the batch; got {:?}\nstdout: {stdout}\nstderr: {stderr}",
        output.status.code()
    );

    let events = events_of(&out, "d6-budget");
    assert!(
        has_stop(&events, "budget"),
        "D6: the task must end with Stop(budget); events: {events:#?}"
    );
    assert!(
        has_task_closed(&events),
        "D6: the task must be closed with TaskClosed; events: {events:#?}"
    );

    // The task must not be accepted: the budget stop happened before the oracle could pass it.
    let results = std::fs::read_to_string(out.join("results-rules.jsonl")).unwrap();
    assert!(
        !results.contains("\"accepted\""),
        "D6: a budget-stopped task must not be accepted; results: {results}"
    );
}

/// D6(b): after a budget stop ends one task, the batch continues with the next task.
#[test]
fn d6_batch_continues_after_a_budget_stop() {
    let ws = workspace();
    let scratch = ws.join("target/d6-batch-continues");
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    let manifest = scratch.join("tasks.jsonl");
    std::fs::write(
        &manifest,
        format!(
            "{}\n{}\n",
            task_json("d6-first", "parser", "parser::tests::handles_empty_input"),
            task_json("d6-second", "lexer", "lexer::tests::tokenizes_unicode"),
        ),
    )
    .unwrap();
    // The first task overspends; the second stays inside the budget.
    std::fs::create_dir_all(scratch.join("sessions")).unwrap();
    std::fs::write(
        scratch.join("sessions/d6-first-rules.json"),
        serde_json::to_string_pretty(&overspending_session()).unwrap(),
    )
    .unwrap();
    std::fs::write(
        scratch.join("sessions/d6-second-rules.json"),
        serde_json::to_string_pretty(&within_budget_session("lexer")).unwrap(),
    )
    .unwrap();
    let out = scratch.join("out");
    let profile = ws.join("profiles/default.toml");

    let output = Command::new(tsr_bin())
        .args([
            "task",
            "run",
            "--mode",
            "rules",
            "--tasks",
            manifest.to_str().unwrap(),
            "--driver",
            "scripted",
            "--out",
            out.to_str().unwrap(),
            "--profile",
            profile.to_str().unwrap(),
            "--scripted-runner",
        ])
        .output()
        .expect("run tsr");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(0),
        "D6: the batch must continue after a budget stop; got {:?}\nstdout: {stdout}\nstderr: {stderr}",
        output.status.code()
    );

    // The first task stopped on budget and was closed.
    let first = events_of(&out, "d6-first");
    assert!(
        has_stop(&first, "budget"),
        "D6: the first task must end with Stop(budget); events: {first:#?}"
    );
    assert!(
        has_task_closed(&first),
        "D6: the first task must be closed; events: {first:#?}"
    );

    // The second task ran to completion: it reached its own Stop and was closed.
    let second = events_of(&out, "d6-second");
    assert!(
        has_task_closed(&second),
        "D6: the second task must have run and closed; events: {second:#?}"
    );
    assert!(
        !has_stop(&second, "budget"),
        "D6: the second task stayed inside the budget and must not stop on budget; events: {second:#?}"
    );

    // Both tasks appear in the results file.
    let results = std::fs::read_to_string(out.join("results-rules.jsonl")).unwrap();
    assert!(
        results.contains("d6-first") && results.contains("d6-second"),
        "D6: both tasks must be recorded; results: {results}"
    );
}

/// D6 (F1): a rules-mode session whose *last* step crosses the budget is stopped with
/// `Stop(budget)` and `TaskClosed`, and is not accepted.
///
/// The defect: the controller checked `spent >= budget` before adding the step's minutes, so the
/// crossing step was committed and only a later step was refused. With no later step, the session
/// was accepted at 101 tool-minutes against the profile's 30, with 0 denials.
#[test]
fn d6_rules_budget_stop_refuses_the_crossing_step() {
    let ws = workspace();
    let scratch = ws.join("target/d6-budget-crossing");
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    let manifest = scratch.join("tasks.jsonl");
    std::fs::write(
        &manifest,
        format!(
            "{}\n",
            task_json(
                "d6-crossing",
                "parser",
                "parser::tests::handles_empty_input"
            )
        ),
    )
    .unwrap();
    std::fs::write(
        scratch.join("session-rules.json"),
        serde_json::to_string_pretty(&overspending_last_session()).unwrap(),
    )
    .unwrap();
    let out = scratch.join("out");
    let profile = ws.join("profiles/default.toml");

    let output = Command::new(tsr_bin())
        .args([
            "task",
            "run",
            "--mode",
            "rules",
            "--tasks",
            manifest.to_str().unwrap(),
            "--driver",
            "scripted",
            "--out",
            out.to_str().unwrap(),
            "--profile",
            profile.to_str().unwrap(),
            "--scripted-runner",
        ])
        .output()
        .expect("run tsr");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(0),
        "D6 (F1): a budget stop must not abort the batch; got {:?}\nstdout: {stdout}\nstderr: {stderr}",
        output.status.code()
    );

    let events = events_of(&out, "d6-crossing");
    assert!(
        has_stop(&events, "budget"),
        "D6 (F1): the task must end with Stop(budget); events: {events:#?}"
    );
    assert!(
        has_task_closed(&events),
        "D6 (F1): the task must be closed with TaskClosed; events: {events:#?}"
    );

    let results = std::fs::read_to_string(out.join("results-rules.jsonl")).unwrap();
    assert!(
        !results.contains("\"accepted\""),
        "D6 (F1): a budget-stopped task must not be accepted; results: {results}"
    );
}

/// D6 (F2): a read-before-test refusal ends *that* task and the batch continues with the next.
///
/// The defect: the refusal returned `Err` from `drive`, which `run_one` propagated, aborting the
/// whole batch with no results file and no second task run.
#[test]
fn d6_batch_continues_after_a_read_before_test_refusal() {
    let ws = workspace();
    let scratch = ws.join("target/d6-read-before-test");
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    let manifest = scratch.join("tasks.jsonl");
    std::fs::write(
        &manifest,
        format!(
            "{}\n{}\n",
            task_json("d6-bad", "parser", "parser::tests::handles_empty_input"),
            task_json("d6-good", "lexer", "lexer::tests::tokenizes_unicode"),
        ),
    )
    .unwrap();
    std::fs::create_dir_all(scratch.join("sessions")).unwrap();
    std::fs::write(
        scratch.join("sessions/d6-bad-rules.json"),
        serde_json::to_string_pretty(&read_before_test_violation_session()).unwrap(),
    )
    .unwrap();
    std::fs::write(
        scratch.join("sessions/d6-good-rules.json"),
        serde_json::to_string_pretty(&within_budget_session("lexer")).unwrap(),
    )
    .unwrap();
    let out = scratch.join("out");
    let profile = ws.join("profiles/default.toml");

    let output = Command::new(tsr_bin())
        .args([
            "task",
            "run",
            "--mode",
            "rules",
            "--tasks",
            manifest.to_str().unwrap(),
            "--driver",
            "scripted",
            "--out",
            out.to_str().unwrap(),
            "--profile",
            profile.to_str().unwrap(),
            "--scripted-runner",
        ])
        .output()
        .expect("run tsr");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(0),
        "D6 (F2): a read-before-test refusal must not abort the batch; got {:?}\nstdout: {stdout}\nstderr: {stderr}",
        output.status.code()
    );

    // The first task ended on the Tier 0 refusal and was closed.
    let first = events_of(&out, "d6-bad");
    assert!(
        has_stop(&first, "blocked"),
        "D6 (F2): the first task must end with Stop(blocked); events: {first:#?}"
    );
    assert!(
        has_task_closed(&first),
        "D6 (F2): the first task must be closed; events: {first:#?}"
    );

    // The second task ran to completion.
    let second = events_of(&out, "d6-good");
    assert!(
        has_task_closed(&second),
        "D6 (F2): the second task must have run and closed; events: {second:#?}"
    );

    // Both tasks appear in the results file.
    let results = std::fs::read_to_string(out.join("results-rules.jsonl")).unwrap();
    assert!(
        results.contains("d6-bad") && results.contains("d6-good"),
        "D6 (F2): both tasks must be recorded; results: {results}"
    );
}

/// A rules session that inspects two files that are **not** the task's oracle test files, then
/// runs the failing test. On the pre-fix code the controller took the "failing assertion" and its
/// "call site" from the session's own first two inspects, so this session satisfied
/// read-before-test and ran the test. After the fix the files to have been read come from the task
/// (`parser::tests::handles_empty_input` → `src/parser.rs`, `tests/parser.rs`), so this session is
/// denied.
fn unrelated_inspect_session() -> serde_json::Value {
    serde_json::json!({
        "name": "n4-unrelated-inspect",
        "steps": [
            {
                "kind": "inspect",
                "args": { "arg": "inspect_path", "path": "src/unrelated.rs" },
                "forecast": 0.9, "forecast_event": "test_pass", "dispatch": false,
                "exit_code": null, "duration_ms": 0, "generator_tokens": 0,
                "tool_minutes": 0, "wall_clock_ms": 0
            },
            {
                "kind": "inspect",
                "args": { "arg": "inspect_path", "path": "tests/unrelated.rs" },
                "forecast": 0.9, "forecast_event": "test_pass", "dispatch": false,
                "exit_code": null, "duration_ms": 0, "generator_tokens": 0,
                "tool_minutes": 0, "wall_clock_ms": 0
            },
            {
                "kind": "run_approved_test",
                "args": { "arg": "test_id", "id": "cargo-test-parser" },
                "forecast": 0.4, "forecast_event": "test_pass", "dispatch": true,
                "exit_code": 101, "duration_ms": 1200, "generator_tokens": 0,
                "tool_minutes": 1, "wall_clock_ms": 1200
            }
        ]
    })
}

/// N4: the read-before-test rule reads the **task**, not the session.
///
/// The defect: `batch_cmd.rs` `scripted_repo_for` took the failing assertion and its call site from
/// the session's own first two `inspect` steps, so any session that inspected any two files before
/// its first test passed read-before-test. The files to have been read must come from the task: the
/// files that define the task's failing tests. A session that inspects unrelated files and then
/// runs the failing test is denied.
#[test]
fn n4_read_before_test_reads_the_task_not_the_session() {
    let ws = workspace();
    let scratch = ws.join("target/n4-read-before-test");
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).unwrap();
    let manifest = scratch.join("tasks.jsonl");
    std::fs::write(
        &manifest,
        format!(
            "{}\n",
            task_json("n4-task", "parser", "parser::tests::handles_empty_input")
        ),
    )
    .unwrap();
    std::fs::write(
        scratch.join("session-rules.json"),
        serde_json::to_string_pretty(&unrelated_inspect_session()).unwrap(),
    )
    .unwrap();
    let out = scratch.join("out");
    let profile = ws.join("profiles/default.toml");

    let output = Command::new(tsr_bin())
        .args([
            "task",
            "run",
            "--mode",
            "rules",
            "--tasks",
            manifest.to_str().unwrap(),
            "--driver",
            "scripted",
            "--out",
            out.to_str().unwrap(),
            "--profile",
            profile.to_str().unwrap(),
            "--scripted-runner",
        ])
        .output()
        .expect("run tsr");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.code(),
        Some(0),
        "N4: a read-before-test refusal must not abort the batch; got {:?}\nstdout: {stdout}\nstderr: {stderr}",
        output.status.code()
    );

    // The task must end on the Tier 0 refusal: the session inspected files the task never named.
    let events = events_of(&out, "n4-task");
    assert!(
        has_stop(&events, "blocked"),
        "N4: a session that inspects unrelated files must be denied read-before-test; events: {events:#?}"
    );
    assert!(
        has_task_closed(&events),
        "N4: the denied task must be closed; events: {events:#?}"
    );

    let results = std::fs::read_to_string(out.join("results-rules.jsonl")).unwrap();
    assert!(
        !results.contains("\"accepted\""),
        "N4: a session that never read the task's oracle test files must not be accepted; results: {results}"
    );
}
