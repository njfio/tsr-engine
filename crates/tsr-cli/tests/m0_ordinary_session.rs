//! M0 r2: an *ordinary* Claude Code session records a log that reconstruct accepts.
//!
//! The real-session fixture (`fixtures/m0/claude-session.events.jsonl`) was recorded in a restricted
//! session: every Bash call was a `tsr exec -- …` and every other tool call was a `Read`. An ordinary
//! session is messier, and the recorder must hold up over it:
//!
//! * a Bash `ls` and a `TodoWrite` are the model's own turn — they must be recorded in a shape
//!   `reconstruct` accepts (a `Committed` whose candidate has no prior `Proposed` is rejected);
//! * `RUST_BACKTRACE=1 tsr exec -- cargo test` is a test run even though the command does not start
//!   with the literal `cargo test`;
//! * `tsr exec -- sh -c 'a; b'` must keep the forecast the hook recorded for the same command: the
//!   hook sees the command *string*, `tsr exec` sees the argv after the shell split it, and the two
//!   must digest the arguments identically.
//!
//! The test drives the built `tsr` binary through `tsr hook` with a payload sequence of an ordinary
//! session, exports the task with `tsr task export`, and asserts `reconstruct::from_events` accepts
//! the log and the six invariants `m0_real_session.rs` checks hold over it.
//!
//! No container runtime is needed: the `tsr exec` calls run with a fake `docker` on `PATH` whose
//! `info` succeeds and whose `run` exits 0, exactly as `m0_honest_sandbox.rs` does.

use std::path::{Path, PathBuf};
use std::process::Command;

use tsr_cli::reconstruct;

/// The `tsr` binary cargo built for this test.
fn tsr_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_tsr"))
}

/// A scratch directory under the target dir (never `/tmp`, which this host prunes).
fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/m0-ordinary")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

/// Write a fake `docker` script into `dir/bin` and return the directory to prepend to `PATH`.
///
/// The script answers `info` with success and `run` with `run_exit`: it never starts a container, so
/// the test needs no runtime. `--version` also succeeds (the probe asks the daemon, not the binary).
fn fake_docker(dir: &Path, run_exit: i32) -> PathBuf {
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin).expect("fake bin dir");
    let script = format!(
        r#"#!/bin/sh
case "$1" in
  --version) exit 0 ;;
  info) exit 0 ;;
  run) echo "fake docker run"; exit {run_exit} ;;
  *) exit 0 ;;
esac
"#,
        run_exit = run_exit,
    );
    let path = bin.join("docker");
    std::fs::write(&path, script).expect("write fake docker");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .expect("chmod fake docker");
    }
    bin
}

/// A `PATH` with the fake runtime's directory first and the real `PATH` after it.
fn path_with(bin: &Path) -> String {
    let real = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", bin.display(), real)
}

/// The environment every `tsr` invocation in this test shares: the scratch store and the session.
struct Env<'a> {
    cwd: &'a Path,
    db: &'a Path,
    session: &'a str,
    bin: &'a Path,
}

impl Env<'_> {
    fn command(&self, program: &str) -> Command {
        let mut cmd = Command::new(program);
        cmd.current_dir(self.cwd)
            .env("PATH", path_with(self.bin))
            .env("TSR_DB", self.db)
            .env("TSR_SESSION", self.session);
        cmd
    }

    /// Pipe one hook payload into `tsr hook <stage>`.
    fn hook(&self, stage: &str, payload: &str) -> std::process::Output {
        let mut cmd = self.command(tsr_bin().to_str().expect("utf8 path").as_ref());
        cmd.arg("hook").arg("--stage").arg(stage);
        cmd.stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        let mut child = cmd.spawn().expect("spawn tsr hook");
        {
            use std::io::Write;
            child
                .stdin
                .as_mut()
                .expect("stdin")
                .write_all(payload.as_bytes())
                .expect("write payload");
        }
        child.wait_with_output().expect("tsr hook output")
    }

    /// Run `tsr <args…>` and return the output.
    fn tsr(&self, args: &[&str]) -> std::process::Output {
        let mut cmd = self.command(tsr_bin().to_str().expect("utf8 path").as_ref());
        cmd.args(args);
        cmd.output().expect("run tsr")
    }

    /// Export the session's task trace as a string.
    fn export(&self, session: &str) -> String {
        let task = task_for(session);
        let out = self.tsr(&["task", "export", &task]);
        assert!(
            out.status.success(),
            "tsr task export exits 0: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).expect("utf8 trace")
    }
}

/// The candidate kind of the last `proposed` event in an exported trace, if any.
fn last_proposed_kind(trace: &str) -> Option<String> {
    trace
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str::<serde_json::Value>(l).expect("an event line"))
        .filter(|v| v["event"]["kind"]["kind"] == "proposed")
        .next_back()
        .map(|v| {
            v["event"]["kind"]["candidate"]["kind"]
                .as_str()
                .expect("a candidate kind")
                .to_string()
        })
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

/// A PostToolUse payload for a tool call, with a Bash-shaped response.
fn posttooluse(session: &str, tool: &str, input: serde_json::Value, exit_code: i32) -> String {
    serde_json::json!({
        "session_id": session,
        "hook_event_name": "PostToolUse",
        "tool_name": tool,
        "tool_input": input,
        "tool_response": { "stdout": "ok", "stderr": "", "exit_code": exit_code },
        "cwd": "/workspace",
    })
    .to_string()
}

/// The task id for a session, as the hook computes it.
fn task_for(session: &str) -> String {
    tsr_cli::store_path::task_for_session(session).to_string()
}

/// Drive the whole ordinary session and return the exported trace path.
fn record_ordinary_session(dir: &Path) -> PathBuf {
    let bin = fake_docker(dir, 0);
    let db = dir.join(".tsr/tsr.db");
    let session = "m0-ordinary-session";
    let env = Env {
        cwd: dir,
        db: &db,
        session,
        bin: &bin,
    };

    // 1. SessionStart opens the task.
    let start = serde_json::json!({
        "session_id": session,
        "hook_event_name": "SessionStart",
        "cwd": dir.to_str().expect("utf8 cwd"),
    })
    .to_string();
    let out = env.hook("SessionStart", &start);
    assert!(out.status.success(), "SessionStart exits 0: {out:?}");

    // 2. A Read: an Inspect, recorded Proposed → Committed.
    let read = serde_json::json!({ "file_path": "src/lib.rs" });
    let out = env.hook("PreToolUse", &pretooluse(session, "Read", read.clone()));
    assert!(out.status.success(), "Read PreToolUse exits 0: {out:?}");
    let out = env.hook("PostToolUse", &posttooluse(session, "Read", read, 0));
    assert!(out.status.success(), "Read PostToolUse exits 0: {out:?}");

    // 3. A Bash `ls`: the model's own turn (Generate), not a test run.
    let ls = serde_json::json!({ "command": "ls" });
    let out = env.hook("PreToolUse", &pretooluse(session, "Bash", ls.clone()));
    assert!(out.status.success(), "Bash ls PreToolUse exits 0: {out:?}");
    let out = env.hook("PostToolUse", &posttooluse(session, "Bash", ls, 0));
    assert!(out.status.success(), "Bash ls PostToolUse exits 0: {out:?}");

    // 4. A TodoWrite: the model's own turn.
    let todo =
        serde_json::json!({ "todos": [{ "content": "run the tests", "status": "pending" }] });
    let out = env.hook(
        "PreToolUse",
        &pretooluse(session, "TodoWrite", todo.clone()),
    );
    assert!(
        out.status.success(),
        "TodoWrite PreToolUse exits 0: {out:?}"
    );
    let out = env.hook("PostToolUse", &posttooluse(session, "TodoWrite", todo, 0));
    assert!(
        out.status.success(),
        "TodoWrite PostToolUse exits 0: {out:?}"
    );

    // 5. A forecast for the test-run candidate, recorded by `tsr forecast` itself. The command the
    //    hook will see is `RUST_BACKTRACE=1 tsr exec -- cargo test`; the inner command is
    //    `cargo test`, which is what both the forecast and `tsr exec` digest.
    let out = env.tsr(&[
        "forecast",
        "--kind",
        "run_approved_test",
        "--args",
        "cargo test",
        "--p",
        "0.9",
        "--source",
        "generator",
    ]);
    assert!(
        out.status.success(),
        "tsr forecast exits 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // 6. `RUST_BACKTRACE=1 tsr exec -- cargo test`: a test run whose command does not start with the
    //    literal `cargo test`. The hook records the commitment; `tsr exec` records the dispatch.
    let exec_cmd = "RUST_BACKTRACE=1 tsr exec -- cargo test";
    let exec_input = serde_json::json!({ "command": exec_cmd });
    let out = env.hook(
        "PreToolUse",
        &pretooluse(session, "Bash", exec_input.clone()),
    );
    assert!(out.status.success(), "exec PreToolUse exits 0: {out:?}");
    // The hook itself must classify the prefixed command as a test run: a `VAR=value` prefix must not
    // defeat `is_tsr_exec_command`. Assert on the hook's own event, not on the forecast's proposal.
    let after_hook = env.export(session);
    let hook_proposed = last_proposed_kind(&after_hook);
    assert_eq!(
        hook_proposed.as_deref(),
        Some("run_approved_test"),
        "the hook's PreToolUse for {exec_cmd:?} proposes a run_approved_test, not a generate"
    );
    let out = env.tsr(&["exec", "--", "cargo", "test"]);
    assert!(
        out.status.success(),
        "tsr exec -- cargo test exits 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = env.hook("PostToolUse", &posttooluse(session, "Bash", exec_input, 0));
    assert!(out.status.success(), "exec PostToolUse exits 0: {out:?}");

    // 7. A `sh -c '…; …'` exec with its own forecast. The hook sees the command string; `tsr exec`
    //    sees the argv. Both must digest `sh -c 'a; b'` identically, or the forecast is orphaned.
    let inner = "sh -c 'a; b'";
    let out = env.tsr(&[
        "forecast",
        "--kind",
        "run_approved_test",
        "--args",
        inner,
        "--p",
        "0.8",
        "--source",
        "generator",
    ]);
    assert!(
        out.status.success(),
        "tsr forecast (sh -c) exits 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let sh_cmd = format!("tsr exec -- {inner}");
    let sh_input = serde_json::json!({ "command": sh_cmd });
    let out = env.hook("PreToolUse", &pretooluse(session, "Bash", sh_input.clone()));
    assert!(out.status.success(), "sh -c PreToolUse exits 0: {out:?}");
    let out = env.tsr(&["exec", "--", "sh", "-c", "a; b"]);
    assert!(
        out.status.success(),
        "tsr exec -- sh -c 'a; b' exits 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let out = env.hook("PostToolUse", &posttooluse(session, "Bash", sh_input, 0));
    assert!(out.status.success(), "sh -c PostToolUse exits 0: {out:?}");

    // 8. Stop.
    let stop = serde_json::json!({
        "session_id": session,
        "hook_event_name": "Stop",
        "cwd": dir.to_str().expect("utf8 cwd"),
    })
    .to_string();
    let out = env.hook("Stop", &stop);
    assert!(out.status.success(), "Stop exits 0: {out:?}");

    // Export the task's trace.
    let task = task_for(session);
    let out = env.tsr(&["task", "export", &task]);
    assert!(
        out.status.success(),
        "tsr task export exits 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let trace = dir.join("events.jsonl");
    std::fs::write(&trace, &out.stdout).expect("write trace");
    trace
}

/// The whole ordinary session reconstructs and keeps every M0 invariant.
#[test]
fn an_ordinary_session_reconstructs_and_keeps_every_rule() {
    let dir = scratch("ordinary");
    let trace = record_ordinary_session(&dir);

    let session = reconstruct::from_events(&trace).expect("reconstructs from the log alone");
    assert!(!session.events.is_empty(), "a session has events");
    assert!(session.monotone_seq(), "B6: sequence is monotone");
    assert!(
        session.every_parent_precedes_its_child(),
        "B6: causality is finite and prior"
    );
    assert!(
        session.every_gated_action_has_prior_forecast_at_same_state_version(),
        "B5: every gated action has a forecast at its state version"
    );
    assert!(
        session.every_dispatch_reconciled_or_unresolved(),
        "B6: every dispatch is reconciled or explicitly unresolved"
    );
    assert!(
        session.every_command_ran_in_sandbox(),
        "B4: no command ran outside the sandbox"
    );
}

/// The ordinary session exercises the whole path: a test run recorded as a test run (not a
/// `Generate`), a sandboxed dispatch, and a stop.
#[test]
fn an_ordinary_session_exercises_the_whole_path() {
    let dir = scratch("ordinary-path");
    let trace = record_ordinary_session(&dir);
    let kinds: Vec<serde_json::Value> = std::fs::read_to_string(&trace)
        .expect("the session's log")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let v: serde_json::Value = serde_json::from_str(l).expect("an event line");
            v["event"]["kind"].clone()
        })
        .collect();
    let count = |tag: &str| kinds.iter().filter(|k| k["kind"] == tag).count();
    let proposed_test = kinds
        .iter()
        .filter(|k| k["kind"] == "proposed" && k["candidate"]["kind"] == "run_approved_test")
        .count();
    let sandboxed = kinds
        .iter()
        .filter(|k| k["kind"] == "dispatched" && k["in_sandbox"] == true)
        .count();
    let host = kinds
        .iter()
        .filter(|k| k["kind"] == "dispatched" && k["in_sandbox"] != true)
        .count();
    assert_eq!(count("task_opened"), 1, "one task opened");
    assert!(
        proposed_test >= 2,
        "both test runs were proposed as test runs (got {proposed_test})"
    );
    assert!(count("forecast") >= 2, "both gated actions were forecast");
    assert!(
        sandboxed >= 2,
        "both commands were dispatched into the sandbox"
    );
    assert_eq!(host, 0, "no command was dispatched on the host");
    assert_eq!(count("stop"), 1, "the session stopped");
}

/// The exit-2 refusal of a store equal to the workspace must not leave an empty `tsr.db` behind
/// (review item 9): the refusal is decided before the store is opened.
#[test]
fn store_is_workspace_refusal_leaves_no_empty_db() {
    let dir = scratch("store-is-workspace");
    let bin = fake_docker(&dir, 0);
    // TSR_DB is the workspace itself: `<dir>/tsr.db` has parent `<dir>`.
    let db = dir.join("tsr.db");
    let env = Env {
        cwd: &dir,
        db: &db,
        session: "store-is-workspace",
        bin: &bin,
    };
    let out = env.tsr(&["exec", "--", "cargo", "test"]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "a store equal to the workspace is refused with exit 2: {out:?}"
    );
    assert!(
        !db.exists(),
        "the refusal must not create an empty tsr.db at {}",
        db.display()
    );
}
