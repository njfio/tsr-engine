//! Regression test for the DoD repro: `tsr forecast` **without** `--task` must write into the same
//! task the hook uses.
//!
//! The verifier rejected the M0 honest-record repair because `tsr forecast` with no `--task`
//! recorded into task `hash("default")` while the hook recorded into task `hash(session_id)`. The
//! DoD command (`specs/lanes/01b-honest-record.md:90-93`, `README.md:36`) passes no `--task`, so the
//! session task ended up with `Proposed` + `Committed` and **no** `Forecast`, making
//! `every_gated_action_has_prior_forecast_at_same_state_version()` false — the exact M0 failure the
//! repair exists to prevent.
//!
//! This test reproduces the verifier's exact sequence through the *built binary* (real CLI, real
//! hook payloads on stdin, real store) and asserts the session task holds
//! `Proposed`, `Forecast` (state_version == the 0-based index of `Committed`), `Committed` — and
//! nothing else — and that the B5 invariant holds.

use std::path::PathBuf;
use std::process::{Command, Stdio};

use tsr_cli::{reconstruct, store_path};
use tsr_domain::{EventKind, TaskId};
use tsr_store::Store;

/// The `tsr` binary cargo built for this test.
fn tsr_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_tsr"))
}

/// A scratch directory under the target dir (never `/tmp`, which this host prunes).
fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/m0-forecast-default")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

/// Run `tsr <args>` against `db`, feeding `stdin` if given. Returns (exit code, stdout, stderr).
fn run_tsr(db: &PathBuf, args: &[&str], stdin: Option<&str>) -> (i32, String, String) {
    let mut cmd = Command::new(tsr_bin());
    cmd.args(args)
        .env("TSR_DB", db)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if stdin.is_some() {
        cmd.stdin(Stdio::piped());
    } else {
        cmd.stdin(Stdio::null());
    }
    let mut child = cmd.spawn().expect("spawn tsr");
    if let Some(input) = stdin {
        use std::io::Write;
        child
            .stdin
            .as_mut()
            .expect("stdin")
            .write_all(input.as_bytes())
            .expect("write stdin");
    }
    let out = child.wait_with_output().expect("wait tsr");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The `kind` tag of an event kind, as serde writes it.
fn kind_name(kind: &EventKind) -> String {
    let v = serde_json::to_value(kind).expect("serialize kind");
    v["kind"].as_str().expect("kind tag").to_string()
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

/// Reconstruct a session from the task's exported trace and run the M0 invariants.
fn reconstruct_task(store: &Store, task: &TaskId) -> reconstruct::Session {
    let jsonl = tsr_cli::task::export(store, task).expect("export");
    let dir = scratch(&format!("reconstruct-{task}"));
    let path = dir.join("events.jsonl");
    std::fs::write(&path, jsonl).expect("write trace");
    reconstruct::from_events(&path).expect("reconstruct from events.jsonl alone")
}

/// The verifier's exact DoD sequence, driven through the real binary with no `--task`:
/// SessionStart → `tsr forecast` (no `--task`) → PreToolUse Bash for the same command.
///
/// The forecast must land in the session's task, so the task holds exactly
/// `Proposed`, `Forecast`, `Committed`, `Dispatched` — with the forecast's `state_version` equal to
/// the 0-based index of the `Committed` event — and B5 must hold.
#[test]
fn forecast_without_task_records_into_the_hook_session_task() {
    let dir = scratch("dod-repro");
    let db = dir.join("tsr.db");
    let session = "dod-session";
    let cmd = "cargo test --offline -p parser";

    // 1. SessionStart opens the session's task.
    let (code, _, err) = run_tsr(
        &db,
        &["hook", "--stage", "SessionStart"],
        Some(&format!(
            r#"{{"session_id":"{session}","hook_event_name":"SessionStart","cwd":"/work"}}"#
        )),
    );
    assert_eq!(code, 0, "SessionStart exits 0: stderr={err}");

    // 2. The forecast enters the log first, naming the action — with NO `--task`, exactly as the
    //    DoD and README document it.
    let (code, forecast_out, err) = run_tsr(
        &db,
        &[
            "forecast",
            "--kind",
            "run_approved_test",
            "--args",
            cmd,
            "--p",
            "0.7",
            "--source",
            "generator",
        ],
        None,
    );
    assert_eq!(code, 0, "tsr forecast (no --task) exits 0: stderr={err}");
    assert!(
        !forecast_out.trim().is_empty(),
        "tsr forecast prints the forecast id"
    );

    // 3. The gated tool call follows.
    let (code, _, err) = run_tsr(
        &db,
        &["hook", "--stage", "PreToolUse"],
        Some(&format!(
            r#"{{"session_id":"{session}","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{{"command":"{cmd}"}}}}"#
        )),
    );
    assert_eq!(code, 0, "PreToolUse exits 0: stderr={err}");

    // The session's task is the one the hook names from the payload session_id.
    let store = store_path::open_store_at(&db).expect("open store");
    let task = store_path::task_for_session(session);

    let ks = kinds(&store, &task);
    let tags: Vec<&str> = ks.iter().map(|(_, k)| k.as_str()).collect();
    assert_eq!(
        tags,
        vec![
            "task_opened",
            "proposed",
            "forecast",
            "committed",
            "dispatched"
        ],
        "the session task holds Proposed, Forecast, Committed and the host dispatch — and nothing \
         else; a forecast in a different task is the rejected defect"
    );

    // The forecast's state_version equals the 0-based index of the Committed event.
    let committed_index = ks
        .iter()
        .position(|(_, k)| k == "committed")
        .expect("a Committed event") as u64;
    let forecast_state_version = store
        .by_task(&task)
        .expect("by_task")
        .into_iter()
        .find_map(|(_, ev)| match ev.kind {
            EventKind::Forecast { forecast, .. } => Some(forecast.state_version),
            _ => None,
        })
        .expect("a Forecast event in the session task");
    assert_eq!(
        forecast_state_version, committed_index,
        "the forecast's state_version equals the index of the Committed event"
    );

    // Exactly one Forecast: the hook never mints a second one.
    assert_eq!(
        tags.iter().filter(|k| **k == "forecast").count(),
        1,
        "exactly one Forecast"
    );

    // The session satisfies the B5 invariant — the whole point of the repair.
    let session = reconstruct_task(&store, &task);
    assert!(
        session.every_gated_action_has_prior_forecast_at_same_state_version(),
        "a forecast recorded before the gated action satisfies B5"
    );
}

/// The same sequence with an explicit `--task <session_id>` still works (the documented alternative
/// must not regress).
#[test]
fn forecast_with_an_explicit_task_still_records_into_that_task() {
    let dir = scratch("explicit-task");
    let db = dir.join("tsr.db");
    let session = "explicit-session";
    let cmd = "cargo test --offline -p parser";

    let (code, _, err) = run_tsr(
        &db,
        &["hook", "--stage", "SessionStart"],
        Some(&format!(
            r#"{{"session_id":"{session}","hook_event_name":"SessionStart","cwd":"/work"}}"#
        )),
    );
    assert_eq!(code, 0, "SessionStart exits 0: stderr={err}");

    let (code, _, err) = run_tsr(
        &db,
        &[
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
        ],
        None,
    );
    assert_eq!(code, 0, "tsr forecast --task exits 0: stderr={err}");

    let (code, _, err) = run_tsr(
        &db,
        &["hook", "--stage", "PreToolUse"],
        Some(&format!(
            r#"{{"session_id":"{session}","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{{"command":"{cmd}"}}}}"#
        )),
    );
    assert_eq!(code, 0, "PreToolUse exits 0: stderr={err}");

    let store = store_path::open_store_at(&db).expect("open store");
    let task = store_path::task_for_session(session);
    let tags: Vec<String> = kinds(&store, &task).into_iter().map(|(_, k)| k).collect();
    assert_eq!(
        tags,
        vec![
            "task_opened",
            "proposed",
            "forecast",
            "committed",
            "dispatched"
        ],
        "an explicit --task <session_id> records into the same task the hook uses"
    );
    assert!(
        reconstruct_task(&store, &task)
            .every_gated_action_has_prior_forecast_at_same_state_version(),
        "B5 holds with an explicit --task"
    );
}

/// With no task in the store at all, `tsr forecast` still records (into the `hash("default")`
/// fallback) rather than erroring — the fallback is only for an empty store.
#[test]
fn forecast_without_task_on_an_empty_store_still_records() {
    let dir = scratch("empty-store");
    let db = dir.join("tsr.db");

    let (code, out, err) = run_tsr(
        &db,
        &[
            "forecast",
            "--kind",
            "run_approved_test",
            "--args",
            "cargo test --offline -p parser",
            "--p",
            "0.7",
            "--source",
            "generator",
        ],
        None,
    );
    assert_eq!(
        code, 0,
        "tsr forecast on an empty store exits 0: stderr={err}"
    );
    assert!(!out.trim().is_empty(), "it prints the forecast id");

    let store = store_path::open_store_at(&db).expect("open store");
    let task = store_path::task_for_session("default");
    let tags: Vec<String> = kinds(&store, &task).into_iter().map(|(_, k)| k).collect();
    assert_eq!(
        tags,
        vec!["proposed", "forecast"],
        "an empty store falls back to hash(\"default\")"
    );
}
