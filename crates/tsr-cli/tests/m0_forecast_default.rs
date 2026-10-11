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

/// Run `tsr <args>` against `db` with `cwd` as the working directory, feeding `stdin` if given.
/// Returns (exit code, stdout, stderr).
///
/// The working directory matters: the hook writes `<cwd>/.tsr/session` and `tsr forecast` /
/// `tsr exec` read it, so each test runs in its own scratch directory to keep the session file
/// private to that test.
fn run_tsr_in(
    db: &PathBuf,
    cwd: &PathBuf,
    args: &[&str],
    stdin: Option<&str>,
) -> (i32, String, String) {
    let mut cmd = Command::new(tsr_bin());
    cmd.args(args)
        .current_dir(cwd)
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
    let cwd = dir.display().to_string();

    // 1. SessionStart opens the session's task and writes `<cwd>/.tsr/session`.
    let (code, _, err) = run_tsr_in(
        &db,
        &dir,
        &["hook", "--stage", "SessionStart"],
        Some(&format!(
            r#"{{"session_id":"{session}","hook_event_name":"SessionStart","cwd":"{cwd}"}}"#
        )),
    );
    assert_eq!(code, 0, "SessionStart exits 0: stderr={err}");

    // 2. The forecast enters the log first, naming the action — with NO `--task`, exactly as the
    //    DoD and README document it. It resolves the session from the file SessionStart wrote.
    let (code, forecast_out, err) = run_tsr_in(
        &db,
        &dir,
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
    let (code, _, err) = run_tsr_in(
        &db,
        &dir,
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
    let cwd = dir.display().to_string();

    let (code, _, err) = run_tsr_in(
        &db,
        &dir,
        &["hook", "--stage", "SessionStart"],
        Some(&format!(
            r#"{{"session_id":"{session}","hook_event_name":"SessionStart","cwd":"{cwd}"}}"#
        )),
    );
    assert_eq!(code, 0, "SessionStart exits 0: stderr={err}");

    let (code, _, err) = run_tsr_in(
        &db,
        &dir,
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

    let (code, _, err) = run_tsr_in(
        &db,
        &dir,
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

/// With no session known — no `--task`, no `TSR_SESSION`, no `.tsr/session` file — `tsr forecast`
/// refuses with exit 2 and records nothing. There is no "latest task" and no `hash("default")`
/// fallback: guessing is exactly the ambiguity that let a forecast land in another session's task.
#[test]
fn forecast_without_a_session_refuses_and_records_nothing() {
    let dir = scratch("no-session");
    let db = dir.join("tsr.db");

    let (code, out, err) = run_tsr_in(
        &db,
        &dir,
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
        code, 2,
        "tsr forecast with no session exits 2: stderr={err}"
    );
    assert!(out.trim().is_empty(), "it prints no forecast id");
    assert!(
        err.contains("no session"),
        "the refusal names the reason: stderr={err}"
    );

    // Nothing was recorded anywhere: the store has no events at all.
    let store = store_path::open_store_at(&db).expect("open store");
    let default_task = store_path::task_for_session("default");
    assert!(
        kinds(&store, &default_task).is_empty(),
        "no hash(\"default\") fallback task was written"
    );
}

/// The architect's two-session repro: with two sessions open, a `--task`-less forecast must land in
/// the *acting* session's task, not in whichever task the store opened most recently.
///
/// Before the fix, `tsr forecast` resolved its task with `latest_task` (the store's most recent
/// `task_opened`) while the hook resolved with `hash(session_id)`. With session B opened after A, the
/// forecast landed in B and A's log was `Proposed` + `Committed` with no `Forecast` — the M0
/// invariant defeated by an ordinary situation. Now both paths read the session file SessionStart
/// wrote, so the forecast lands in A.
#[test]
fn two_sessions_open_the_forecast_lands_in_the_acting_session() {
    let dir = scratch("two-sessions");
    let db = dir.join("tsr.db");
    let cmd = "cargo test --offline -p parser";
    let cwd = dir.display().to_string();

    // Session A opens, then session B opens — B is the store's most recent task.
    for session in ["sess-A", "sess-B"] {
        let (code, _, err) = run_tsr_in(
            &db,
            &dir,
            &["hook", "--stage", "SessionStart"],
            Some(&format!(
                r#"{{"session_id":"{session}","hook_event_name":"SessionStart","cwd":"{cwd}"}}"#
            )),
        );
        assert_eq!(code, 0, "SessionStart {session} exits 0: stderr={err}");
    }

    // The agent in session A forecasts, then acts. Two sessions share this cwd, so the session file
    // names only the last SessionStart (B); A names itself with `TSR_SESSION`, the documented way a
    // session disambiguates itself. The forecast must land in A, the acting session.
    let mut forecast = Command::new(tsr_bin());
    forecast
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
        ])
        .current_dir(&dir)
        .env("TSR_DB", &db)
        .env("TSR_SESSION", "sess-A")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let out = forecast.output().expect("run tsr forecast");
    assert_eq!(
        out.status.code(),
        Some(0),
        "tsr forecast exits 0: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );

    let (code, _, err) = run_tsr_in(
        &db,
        &dir,
        &["hook", "--stage", "PreToolUse"],
        Some(&format!(
            r#"{{"session_id":"sess-A","hook_event_name":"PreToolUse","tool_name":"Bash","tool_input":{{"command":"{cmd}"}}}}"#
        )),
    );
    assert_eq!(code, 0, "PreToolUse exits 0: stderr={err}");

    let store = store_path::open_store_at(&db).expect("open store");
    let task_a = store_path::task_for_session("sess-A");
    let task_b = store_path::task_for_session("sess-B");

    let tags_a: Vec<String> = kinds(&store, &task_a).into_iter().map(|(_, k)| k).collect();
    assert_eq!(
        tags_a,
        vec![
            "task_opened",
            "proposed",
            "forecast",
            "committed",
            "dispatched"
        ],
        "the forecast lands in the acting session A, before the commitment"
    );
    assert!(
        reconstruct_task(&store, &task_a)
            .every_gated_action_has_prior_forecast_at_same_state_version(),
        "B5 holds for the acting session"
    );

    let tags_b: Vec<String> = kinds(&store, &task_b).into_iter().map(|(_, k)| k).collect();
    assert_eq!(
        tags_b,
        vec!["task_opened"],
        "session B, which did nothing, holds only its own task_opened"
    );

    // The limitation, asserted rather than hidden: the session file names the session of the *last
    // hook stage*, so a `--task`-less forecast with no `TSR_SESSION` resolves to whatever session
    // last ran a hook — here A (its PreToolUse ran last), not B. With two sessions sharing a cwd the
    // file cannot tell them apart, which is why a session must name itself with `TSR_SESSION`; the
    // file is the single-session default. (Documented in README's agent protocol.)
    let (code, _, err) = run_tsr_in(
        &db,
        &dir,
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
    assert_eq!(code, 0, "file-only forecast exits 0: stderr={err}");
    let tags_a_after: Vec<String> = kinds(&store, &task_a).into_iter().map(|(_, k)| k).collect();
    assert_eq!(
        tags_a_after,
        vec![
            "task_opened",
            "proposed",
            "forecast",
            "committed",
            "dispatched",
            "proposed",
            "forecast"
        ],
        "with no TSR_SESSION the file names the last hook stage's session (A)"
    );
    let tags_b_after: Vec<String> = kinds(&store, &task_b).into_iter().map(|(_, k)| k).collect();
    assert_eq!(
        tags_b_after,
        vec!["task_opened"],
        "session B never acted, so it stays task_opened only"
    );
}

/// `tsr exec` resolves its task with the same session rule as the hook, so the committed-candidate
/// reuse can fire. With no session known it refuses (exit 2) instead of falling back to
/// `hash("default")`; with a session known it reaches the runtime check (exit 3 here, no runtime),
/// which proves it resolved a task rather than refusing.
#[test]
fn exec_resolves_the_session_task_and_refuses_without_one() {
    let dir = scratch("exec-session");
    let db = dir.join("tsr.db");
    let session = "exec-session-id";
    let cwd = dir.display().to_string();

    // No session yet: `tsr exec` refuses with exit 2 and records nothing.
    let (code, _, err) = run_tsr_in(&db, &dir, &["exec", "--", "cargo", "test"], None);
    assert_eq!(code, 2, "tsr exec with no session exits 2: stderr={err}");
    assert!(
        err.contains("no session"),
        "the refusal names the reason: stderr={err}"
    );

    // SessionStart writes the session file; now `tsr exec` resolves the task and reaches the runtime
    // check. No container runtime is reachable here, so it refuses with exit 3 and records nothing.
    let (code, _, err) = run_tsr_in(
        &db,
        &dir,
        &["hook", "--stage", "SessionStart"],
        Some(&format!(
            r#"{{"session_id":"{session}","hook_event_name":"SessionStart","cwd":"{cwd}"}}"#
        )),
    );
    assert_eq!(code, 0, "SessionStart exits 0: stderr={err}");

    let (code, _, err) = run_tsr_in(&db, &dir, &["exec", "--", "cargo", "test"], None);
    assert_eq!(
        code, 3,
        "tsr exec with a session reaches the runtime check: stderr={err}"
    );
    assert!(
        err.contains("no container runtime"),
        "the refusal names the runtime: stderr={err}"
    );

    // Nothing was recorded: the task holds only its task_opened.
    let store = store_path::open_store_at(&db).expect("open store");
    let task = store_path::task_for_session(session);
    let tags: Vec<String> = kinds(&store, &task).into_iter().map(|(_, k)| k).collect();
    assert_eq!(tags, vec!["task_opened"], "a refused exec records nothing");
}

/// `TSR_SESSION` names the session explicitly, so `tsr forecast` resolves the same task the hook
/// would for that session — the documented escape hatch when no session file is present.
#[test]
fn forecast_resolves_the_session_from_tsr_session_env() {
    let dir = scratch("env-session");
    let db = dir.join("tsr.db");
    let session = "env-session-id";
    let cmd = "cargo test --offline -p parser";

    let mut cmd_proc = Command::new(tsr_bin());
    cmd_proc
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
        ])
        .current_dir(&dir)
        .env("TSR_DB", &db)
        .env("TSR_SESSION", session)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let out = cmd_proc.output().expect("run tsr forecast");
    assert_eq!(
        out.status.code(),
        Some(0),
        "tsr forecast with TSR_SESSION exits 0: stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );

    let store = store_path::open_store_at(&db).expect("open store");
    let task = store_path::task_for_session(session);
    let tags: Vec<String> = kinds(&store, &task).into_iter().map(|(_, k)| k).collect();
    assert_eq!(
        tags,
        vec!["proposed", "forecast"],
        "TSR_SESSION resolves to hash(session_id)"
    );
}
