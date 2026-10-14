//! M0 r5: the agent names the candidate; nothing is matched.
//!
//! Operator decision: three rounds tried to make the hook's view of a Bash command and `tsr exec`'s
//! argv agree (r2 digests, r3 shell emulation, r4/r4b pattern matching); each fixed the named
//! spellings and broke others. So nothing is matched. The agent names the candidate:
//!
//! * `tsr forecast --kind <k> --args <text> --p <p>` prints the candidate id alone on stdout and
//!   records the candidate as proposed with that forecast at the current state version;
//! * `tsr exec --candidate <64-hex> -- <argv…>` binds the run to that candidate — it must be a
//!   candidate of the session's task that has a forecast and has not been dispatched since that
//!   newest forecast (one forecast authorizes one run); exec records `Dispatched` and `Executed` for
//!   it and runs argv as today. An unknown, unforecast, not-committed-after-its-forecast or
//!   already-dispatched-since-that-forecast id exits 2 naming the reason and runs nothing;
//! * the hook, at PreToolUse for a Bash command, tokenizes the command with the grammar-only parser
//!   and, if it finds a `tsr exec` invocation carrying `--candidate <64-hex>`, commits that candidate
//!   and records no new proposal. No pattern, no sidecar, no matcher;
//! * without `--candidate`, the hook records the Bash call as it does for any non-gated tool and
//!   `tsr exec` mints its own candidate with no forecast and prints one stderr line saying so.
//!
//! Each spelling below is driven through `tsr hook` and `tsr exec` with a real `bash -c` running the
//! command after PreToolUse, exactly as a session would. No container runtime is needed: a fake
//! `docker` on `PATH` answers `info` with success and `run` with exit 0.

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
        .join("../../target/m0-candidate")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

/// Write a fake `docker` script into `dir/bin` and return the directory to prepend to `PATH`.
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

/// Write a fake `docker` that appends one line to `dir/runs.log` for every `run` it is asked to do,
/// so a test can count container runs. `info` succeeds (a daemon answers).
fn fake_docker_counting(dir: &Path) -> PathBuf {
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin).expect("fake bin dir");
    let log = dir.join("runs.log");
    let script = format!(
        r#"#!/bin/sh
case "$1" in
  --version) exit 0 ;;
  info) exit 0 ;;
  run) echo "run" >> "{log}"; echo "fake docker run"; exit 0 ;;
  *) exit 0 ;;
esac
"#,
        log = log.display(),
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

/// How many container runs the counting fake runtime recorded.
fn container_runs(dir: &Path) -> usize {
    std::fs::read_to_string(dir.join("runs.log"))
        .map(|s| s.lines().filter(|l| !l.trim().is_empty()).count())
        .unwrap_or(0)
}

/// A `PATH` with the built `tsr` binary's directory first, then the fake runtime's directory, then
/// the real `PATH`. The `tsr` directory must come first so the `bash -c` that runs the command
/// string finds *this* build's `tsr`, not one installed on the host (defect 4: hermetic).
fn path_with(bin: &Path) -> String {
    let real = std::env::var("PATH").unwrap_or_default();
    let tsr_dir = tsr_bin()
        .parent()
        .expect("the tsr binary has a parent directory")
        .to_path_buf();
    format!("{}:{}:{}", tsr_dir.display(), bin.display(), real)
}

/// Quote one argv word for `bash -c` with single quotes, so the shell hands `tsr exec` exactly this
/// word (a `;`, a space or a `$` inside it stays literal).
fn shell_quote(word: &str) -> String {
    format!("'{}'", word.replace('\'', "'\\''"))
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

/// Open a session and return the environment plus the fake runtime dir.
fn open_session(dir: &Path, session: &str) -> (PathBuf, PathBuf) {
    let bin = fake_docker(dir, 0);
    let db = dir.join(".tsr/tsr.db");
    let env = Env {
        cwd: dir,
        db: &db,
        session,
        bin: &bin,
    };
    let start = serde_json::json!({
        "session_id": session,
        "hook_event_name": "SessionStart",
        "cwd": dir.to_str().expect("utf8 cwd"),
    })
    .to_string();
    let out = env.hook("SessionStart", &start);
    assert!(out.status.success(), "SessionStart exits 0: {out:?}");
    (bin, db)
}

/// The candidate id `tsr forecast` printed: alone on stdout, 64 hex.
fn forecast(env: &Env, args: &str, p: &str) -> String {
    let out = env.tsr(&[
        "forecast",
        "--kind",
        "run_approved_test",
        "--args",
        args,
        "--p",
        p,
        "--source",
        "generator",
    ]);
    assert!(
        out.status.success(),
        "tsr forecast --args {args:?} exits 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8(out.stdout).expect("utf8 stdout");
    let id = stdout.trim();
    assert_eq!(
        id.len(),
        64,
        "tsr forecast prints the candidate id alone on stdout, got {stdout:?}"
    );
    assert!(
        id.chars().all(|c| c.is_ascii_hexdigit()),
        "the candidate id is 64 hex, got {id:?}"
    );
    id.to_string()
}

/// The `kind` tags of every event in an exported trace.
fn kinds(trace: &str) -> Vec<serde_json::Value> {
    trace
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let v: serde_json::Value = serde_json::from_str(l).expect("an event line");
            v["event"]["kind"].clone()
        })
        .collect()
}

/// Count events of a tag.
fn count(kinds: &[serde_json::Value], tag: &str) -> usize {
    kinds.iter().filter(|k| k["kind"] == tag).count()
}

/// Count `proposed` events whose candidate kind is `run_approved_test`.
fn count_proposed_test(kinds: &[serde_json::Value]) -> usize {
    kinds
        .iter()
        .filter(|k| k["kind"] == "proposed" && k["candidate"]["kind"] == "run_approved_test")
        .count()
}

/// One spelling: forecast, then the Bash command `tsr exec --candidate <id> -- …`, run for real
/// through `bash -c` after PreToolUse. Asserts one proposal, one forecast, one commit, one dispatch,
/// and a log reconstruct accepts.
fn one_spelling(dir: &Path, name: &str, inner: &str, argv: &[&str]) {
    let session = format!("m0-candidate-{name}");
    let (bin, db) = open_session(dir, &session);
    let env = Env {
        cwd: dir,
        db: &db,
        session: &session,
        bin: &bin,
    };

    let id = forecast(&env, inner, "0.9");

    // The Bash command the agent runs: `tsr exec --candidate <id> -- <argv…>`. The argv is the
    // shell's expansion of `inner`, which for several cases differs from the forecast's `--args`
    // text (e.g. `cat <<< "hi"` forecasts `cat <<< "hi"` but runs `cat`): the run binds by id, not
    // by text (defect 4).
    let exec_argv = argv
        .iter()
        .map(|a| shell_quote(a))
        .collect::<Vec<_>>()
        .join(" ");
    let command = format!("tsr exec --candidate {id} -- {exec_argv}");
    let input = serde_json::json!({ "command": command });
    let out = env.hook("PreToolUse", &pretooluse(&session, "Bash", input.clone()));
    assert!(
        out.status.success(),
        "{name}: PreToolUse exits 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Run the command for real, as the shell would, with the fake runtime on PATH.
    let mut bash = env.command("bash");
    bash.arg("-c").arg(&command);
    let out = bash.output().expect("run bash -c");
    assert!(
        out.status.success(),
        "{name}: `{command}` exits 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = env.hook("PostToolUse", &posttooluse(&session, "Bash", input, 0));
    assert!(
        out.status.success(),
        "{name}: PostToolUse exits 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let trace = env.export(&session);
    let ks = kinds(&trace);
    assert_eq!(
        count_proposed_test(&ks),
        1,
        "{name}: exactly one run_approved_test proposal (got {})",
        count_proposed_test(&ks)
    );
    assert_eq!(
        count(&ks, "forecast"),
        1,
        "{name}: exactly one forecast (got {})",
        count(&ks, "forecast")
    );
    assert_eq!(
        count(&ks, "committed"),
        1,
        "{name}: exactly one commit (got {})",
        count(&ks, "committed")
    );
    assert_eq!(
        count(&ks, "dispatched"),
        1,
        "{name}: exactly one dispatch (got {})",
        count(&ks, "dispatched")
    );

    // The log reconstruct accepts.
    let trace_path = dir.join(format!("{name}.jsonl"));
    std::fs::write(&trace_path, &trace).expect("write trace");
    let session = reconstruct::from_events(&trace_path)
        .unwrap_or_else(|e| panic!("{name}: reconstruct accepts the log: {e}"));
    assert!(
        session.every_gated_action_has_prior_forecast_at_same_state_version(),
        "{name}: B5: every gated action has a forecast at its state version"
    );
    assert!(
        session.every_dispatch_reconciled_or_unresolved(),
        "{name}: B6: every dispatch is reconciled or unresolved"
    );
    assert!(
        session.every_command_ran_in_sandbox(),
        "{name}: B4: no command ran outside the sandbox"
    );
}

/// Every spelling the operator named, each through the full protocol.
#[test]
fn every_spelling_records_one_proposal_one_forecast_one_commit_one_dispatch() {
    let dir = scratch("spellings");
    // (name, inner command string, argv after `--`)
    let cases: &[(&str, &str, &[&str])] = &[
        ("pipe", "cargo test 2>&1 | tail -20", &["cargo", "test"]),
        ("redirect", "cargo test > out.txt", &["cargo", "test"]),
        ("dollar-home", "cat \"$HOME/x\"", &["cat", "$HOME/x"]),
        ("tilde", "ls ~", &["ls", "~"]),
        ("glob", "ls *.toml", &["ls", "*.toml"]),
        ("sh-semi", "sh -c 'a; b'", &["sh", "-c", "a; b"]),
        ("sh-newline", "sh -c \"a\nb\"", &["sh", "-c", "a\nb"]),
        (
            "git-commit-newlines",
            "git commit -m \"t\n\nbody\"",
            &["git", "commit", "-m", "t\n\nbody"],
        ),
        ("herestring", "cat <<< \"hi\"", &["cat"]),
        ("bracket", "ls [ab].toml", &["ls", "[ab].toml"]),
        ("quoted-star", "echo '*'", &["echo", "*"]),
        (
            "find-exec",
            "find . -exec echo {} \\;",
            &["find", ".", "-exec", "echo", "{}", ";"],
        ),
    ];
    for (name, inner, argv) in cases {
        one_spelling(&dir, name, inner, argv);
    }
}

/// `time` / `timeout 60` / `nice -n 5` before `tsr exec` must not defeat the hook's recognition of
/// the `--candidate` (review item 8a).
#[test]
fn prefixes_before_tsr_exec_do_not_defeat_the_candidate() {
    let dir = scratch("prefixes");
    for (name, prefix) in [
        ("time", "time "),
        ("timeout", "timeout 60 "),
        ("nice", "nice -n 5 "),
    ] {
        let session = format!("m0-candidate-prefix-{name}");
        let (bin, db) = open_session(&dir, &session);
        let env = Env {
            cwd: &dir,
            db: &db,
            session: &session,
            bin: &bin,
        };
        let id = forecast(&env, "cargo test", "0.9");
        let command = format!("{prefix}tsr exec --candidate {id} -- cargo test");
        let input = serde_json::json!({ "command": command });
        let out = env.hook("PreToolUse", &pretooluse(&session, "Bash", input.clone()));
        assert!(
            out.status.success(),
            "{name}: PreToolUse exits 0: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let mut bash = env.command("bash");
        bash.arg("-c").arg(&command);
        let out = bash.output().expect("run bash -c");
        assert!(
            out.status.success(),
            "{name}: `{command}` exits 0: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let out = env.hook("PostToolUse", &posttooluse(&session, "Bash", input, 0));
        assert!(
            out.status.success(),
            "{name}: PostToolUse exits 0: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let trace = env.export(&session);
        let ks = kinds(&trace);
        assert_eq!(
            count_proposed_test(&ks),
            1,
            "{name}: exactly one run_approved_test proposal"
        );
        assert_eq!(count(&ks, "forecast"), 1, "{name}: exactly one forecast");
        assert_eq!(count(&ks, "committed"), 1, "{name}: exactly one commit");
        assert_eq!(count(&ks, "dispatched"), 1, "{name}: exactly one dispatch");
    }
}

/// An unknown or already-dispatched `--candidate` exits 2 and runs nothing.
#[test]
fn unknown_or_dispatched_candidate_exits_2_and_runs_nothing() {
    let dir = scratch("refusals");
    let session = "m0-candidate-refusals";
    let (bin, db) = open_session(&dir, session);
    let env = Env {
        cwd: &dir,
        db: &db,
        session,
        bin: &bin,
    };

    // Unknown: a well-formed 64-hex id that is not a candidate of this task.
    let unknown = "0".repeat(64);
    let out = env.tsr(&["exec", "--candidate", &unknown, "--", "cargo", "test"]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "an unknown --candidate exits 2: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("not a candidate"),
        "the refusal names the reason: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // A candidate with no forecast: mint one by running without --candidate, then name it.
    let out = env.tsr(&["exec", "--", "cargo", "test"]);
    assert!(out.status.success(), "a bare exec exits 0: {out:?}");
    let trace = env.export(session);
    let ks = kinds(&trace);
    let unforecast = ks
        .iter()
        .find(|k| k["kind"] == "proposed")
        .and_then(|k| k["candidate"]["id"].as_str())
        .expect("a proposed candidate id")
        .to_string();
    let out = env.tsr(&["exec", "--candidate", &unforecast, "--", "cargo", "test"]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "an unforecast --candidate exits 2: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("no forecast"),
        "the refusal names the reason: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // Already dispatched: forecast, commit through the hook, dispatch once, then dispatch again.
    // The command differs from the bare exec above so the forecast mints a *fresh* candidate (the
    // bare exec's candidate is already dispatched).
    let id = forecast(&env, "cargo build", "0.9");
    let command = format!("tsr exec --candidate {id} -- cargo build");
    let input = serde_json::json!({ "command": command });
    let out = env.hook("PreToolUse", &pretooluse(session, "Bash", input.clone()));
    assert!(out.status.success(), "PreToolUse exits 0: {out:?}");
    let out = env.tsr(&["exec", "--candidate", &id, "--", "cargo", "build"]);
    assert!(out.status.success(), "the first dispatch exits 0: {out:?}");
    let out = env.tsr(&["exec", "--candidate", &id, "--", "cargo", "build"]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "an already-dispatched --candidate exits 2: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("already dispatched"),
        "the refusal names the reason: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Without `--candidate`, `tsr exec` prints one stderr line and records a candidate with no forecast.
#[test]
fn exec_without_candidate_says_so_and_records_no_forecast() {
    let dir = scratch("no-candidate");
    let session = "m0-candidate-none";
    let (bin, db) = open_session(&dir, session);
    let env = Env {
        cwd: &dir,
        db: &db,
        session,
        bin: &bin,
    };

    let out = env.tsr(&["exec", "--", "cargo", "test"]);
    assert!(out.status.success(), "a bare exec exits 0: {out:?}");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("no forecast") && stderr.contains("--candidate"),
        "one stderr line says the run has no forecast: {stderr:?}"
    );

    let trace = env.export(session);
    let ks = kinds(&trace);
    assert_eq!(
        count_proposed_test(&ks),
        1,
        "the bare exec records one candidate"
    );
    assert_eq!(
        count(&ks, "forecast"),
        0,
        "the bare exec records no forecast"
    );
    assert_eq!(count(&ks, "dispatched"), 1, "the bare exec dispatches once");
}

/// One forecast authorizes one run (operator correction, r5c re-arm): forecast once, then run the
/// *same* `tsr exec --candidate <id>` command three times through the hook and a real `bash -c`.
/// The first run dispatches; the second and third are refused (exit 2, nothing runs), because the
/// candidate's newest forecast no longer comes after its newest dispatch. Exactly one container run.
#[test]
fn one_forecast_authorizes_one_run_three_attempts_one_container_run() {
    let dir = scratch("one-forecast-one-run");
    let session = "m0-candidate-one-forecast";
    let bin = fake_docker_counting(&dir);
    let db = dir.join(".tsr/tsr.db");
    let env = Env {
        cwd: &dir,
        db: &db,
        session,
        bin: &bin,
    };
    let start = serde_json::json!({
        "session_id": session,
        "hook_event_name": "SessionStart",
        "cwd": dir.to_str().expect("utf8 cwd"),
    })
    .to_string();
    assert!(env.hook("SessionStart", &start).status.success());

    let id = forecast(&env, "cargo test", "0.9");
    let command = format!("tsr exec --candidate {id} -- cargo test");
    let input = serde_json::json!({ "command": command });

    // Attempt 1: the hook commits, the command runs, the container runs once.
    let out = env.hook("PreToolUse", &pretooluse(session, "Bash", input.clone()));
    assert!(out.status.success(), "PreToolUse 1 exits 0: {out:?}");
    let mut bash = env.command("bash");
    bash.arg("-c").arg(&command);
    let out = bash.output().expect("run bash -c");
    assert!(
        out.status.success(),
        "attempt 1 `{command}` exits 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(env
        .hook(
            "PostToolUse",
            &posttooluse(session, "Bash", input.clone(), 0)
        )
        .status
        .success());
    assert_eq!(container_runs(&dir), 1, "attempt 1 runs the container once");

    // Attempts 2 and 3: the same command, the same candidate id. The hook refuses to commit (one
    // stderr line) and `tsr exec` refuses with exit 2, running nothing.
    for attempt in 2..=3 {
        let out = env.hook("PreToolUse", &pretooluse(session, "Bash", input.clone()));
        assert!(
            out.status.success(),
            "attempt {attempt}: PreToolUse exits 0: {out:?}"
        );
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("not committing") && stderr.contains("already dispatched"),
            "attempt {attempt}: the hook prints one line why it does not commit: {stderr:?}"
        );
        let mut bash = env.command("bash");
        bash.arg("-c").arg(&command);
        let out = bash.output().expect("run bash -c");
        assert_eq!(
            out.status.code(),
            Some(2),
            "attempt {attempt}: `{command}` exits 2: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            String::from_utf8_lossy(&out.stderr).contains("already dispatched"),
            "attempt {attempt}: the refusal names the reason: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    assert_eq!(
        container_runs(&dir),
        1,
        "three attempts, one forecast: exactly one container run"
    );

    let trace = env.export(session);
    let ks = kinds(&trace);
    assert_eq!(count(&ks, "forecast"), 1, "exactly one forecast");
    assert_eq!(count(&ks, "dispatched"), 1, "exactly one dispatch");
    assert_eq!(count(&ks, "executed"), 1, "exactly one execution");
}

/// Forecast, run, forecast again, run again: the second forecast re-arms the same candidate id, so
/// both runs are accepted and `reconstruct` accepts the log. This is `fixtures/m0`'s shape: one
/// candidate run twice, each time after a fresh forecast.
#[test]
fn forecast_run_forecast_run_two_runs_both_accepted() {
    let dir = scratch("rearm");
    let session = "m0-candidate-rearm";
    let bin = fake_docker_counting(&dir);
    let db = dir.join(".tsr/tsr.db");
    let env = Env {
        cwd: &dir,
        db: &db,
        session,
        bin: &bin,
    };
    let start = serde_json::json!({
        "session_id": session,
        "hook_event_name": "SessionStart",
        "cwd": dir.to_str().expect("utf8 cwd"),
    })
    .to_string();
    assert!(env.hook("SessionStart", &start).status.success());

    let mut ids = Vec::new();
    for round in 1..=2 {
        let id = forecast(&env, "cargo test", "0.9");
        ids.push(id.clone());
        let command = format!("tsr exec --candidate {id} -- cargo test");
        let input = serde_json::json!({ "command": command });
        let out = env.hook("PreToolUse", &pretooluse(session, "Bash", input.clone()));
        assert!(
            out.status.success(),
            "round {round}: PreToolUse exits 0: {out:?}"
        );
        let mut bash = env.command("bash");
        bash.arg("-c").arg(&command);
        let out = bash.output().expect("run bash -c");
        assert!(
            out.status.success(),
            "round {round}: `{command}` exits 0: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(env
            .hook("PostToolUse", &posttooluse(session, "Bash", input, 0))
            .status
            .success());
    }
    assert_eq!(
        ids[0], ids[1],
        "the same command mints the same candidate id"
    );
    assert_eq!(container_runs(&dir), 2, "two forecasts, two runs");

    let trace = env.export(session);
    let ks = kinds(&trace);
    assert_eq!(count(&ks, "forecast"), 2, "two forecasts");
    assert_eq!(count(&ks, "dispatched"), 2, "two dispatches");
    assert_eq!(count(&ks, "executed"), 2, "two executions");

    let trace_path = dir.join("rearm.jsonl");
    std::fs::write(&trace_path, &trace).expect("write trace");
    let session = reconstruct::from_events(&trace_path)
        .unwrap_or_else(|e| panic!("reconstruct accepts the log: {e}"));
    assert!(
        session.every_gated_action_has_prior_forecast_at_same_state_version(),
        "B5: every gated action has a forecast at its state version"
    );
    assert!(
        session.every_dispatch_reconciled_or_unresolved(),
        "B6: every dispatch is reconciled or unresolved"
    );
    assert!(
        session.every_command_ran_in_sandbox(),
        "B4: no command ran outside the sandbox"
    );
}

/// A here-string `<<<` on its own line before `tsr exec --candidate …` (not joined by `&&`) must not
/// swallow the `tsr exec` command: the hook still sees the candidate and commits it. This is the
/// d611b5e shell.rs defect — a `<<<` read as a here-document would consume the rest of the line.
#[test]
fn here_string_on_its_own_line_before_tsr_exec_is_seen() {
    let dir = scratch("herestring-own-line");
    let session = "m0-candidate-herestring-line";
    let bin = fake_docker_counting(&dir);
    let db = dir.join(".tsr/tsr.db");
    let env = Env {
        cwd: &dir,
        db: &db,
        session,
        bin: &bin,
    };
    let start = serde_json::json!({
        "session_id": session,
        "hook_event_name": "SessionStart",
        "cwd": dir.to_str().expect("utf8 cwd"),
    })
    .to_string();
    assert!(env.hook("SessionStart", &start).status.success());

    let id = forecast(&env, "cargo test", "0.9");
    // The here-string is on its own line, and the `tsr exec` follows on the next line — no `&&`.
    let command = format!("cat <<< \"hi\"\ntsr exec --candidate {id} -- cargo test");
    let input = serde_json::json!({ "command": command });
    let out = env.hook("PreToolUse", &pretooluse(session, "Bash", input.clone()));
    assert!(out.status.success(), "PreToolUse exits 0: {out:?}");
    assert!(
        !String::from_utf8_lossy(&out.stderr).contains("not committing"),
        "the hook commits the candidate the here-string precedes: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let mut bash = env.command("bash");
    bash.arg("-c").arg(&command);
    let out = bash.output().expect("run bash -c");
    assert!(
        out.status.success(),
        "`{command}` exits 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(env
        .hook("PostToolUse", &posttooluse(session, "Bash", input, 0))
        .status
        .success());

    let trace = env.export(session);
    let ks = kinds(&trace);
    assert_eq!(count(&ks, "committed"), 1, "the candidate is committed");
    assert_eq!(count(&ks, "dispatched"), 1, "the candidate is dispatched");
    assert_eq!(container_runs(&dir), 1, "the container runs once");
}

/// The brief's spellings sent to the hook as the shell text the agent types — pipes, redirects,
/// globs, `~`, `$HOME`, `<<<`, `[ab]`, `'*'`, `find … {} \;` — not as pre-expanded argv. The hook
/// must still find the `tsr exec --candidate <id>` inside the text and commit the candidate.
#[test]
fn spellings_sent_as_shell_text_are_recognised() {
    let dir = scratch("shell-text");
    // (name, the shell text the agent types, with `{id}` for the candidate)
    let cases: &[(&str, &str)] = &[
        (
            "pipe",
            "tsr exec --candidate {id} -- cargo test 2>&1 | tail -20",
        ),
        (
            "redirect",
            "tsr exec --candidate {id} -- cargo test > out.txt",
        ),
        (
            "dollar-home",
            "tsr exec --candidate {id} -- cat \"$HOME/x\"",
        ),
        ("tilde", "tsr exec --candidate {id} -- ls ~"),
        ("glob", "tsr exec --candidate {id} -- ls *.toml"),
        ("herestring", "tsr exec --candidate {id} -- cat <<< \"hi\""),
        ("bracket", "tsr exec --candidate {id} -- ls [ab].toml"),
        ("quoted-star", "tsr exec --candidate {id} -- echo '*'"),
        (
            "find-exec",
            "tsr exec --candidate {id} -- find . -exec echo {} \\;",
        ),
    ];
    for (name, template) in cases {
        let session = format!("m0-candidate-shelltext-{name}");
        let bin = fake_docker_counting(&dir);
        let db = dir.join(".tsr/tsr.db");
        let env = Env {
            cwd: &dir,
            db: &db,
            session: &session,
            bin: &bin,
        };
        let start = serde_json::json!({
            "session_id": session,
            "hook_event_name": "SessionStart",
            "cwd": dir.to_str().expect("utf8 cwd"),
        })
        .to_string();
        assert!(env.hook("SessionStart", &start).status.success());

        let id = forecast(&env, "cargo test", "0.9");
        let command = template.replace("{id}", &id);
        let input = serde_json::json!({ "command": command });
        let out = env.hook("PreToolUse", &pretooluse(&session, "Bash", input.clone()));
        assert!(
            out.status.success(),
            "{name}: PreToolUse exits 0: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            !String::from_utf8_lossy(&out.stderr).contains("not committing"),
            "{name}: the hook commits the candidate in the shell text: {}",
            String::from_utf8_lossy(&out.stderr)
        );

        let trace = env.export(&session);
        let ks = kinds(&trace);
        assert_eq!(
            count(&ks, "committed"),
            1,
            "{name}: the candidate is committed"
        );
        // The commit must be the *named* candidate, not the ordinary non-gated fallback: a missed
        // `--candidate` still yields exactly one commit (of a different candidate), so counting
        // commits alone cannot tell recognition from a miss (r5c review defect 2). The committed
        // candidate's id must equal the forecast's id.
        let committed_ids: Vec<&str> = ks
            .iter()
            .filter(|k| k["kind"] == "committed")
            .filter_map(|k| k["candidate"].as_str())
            .collect();
        assert_eq!(
            committed_ids,
            vec![id.as_str()],
            "{name}: the committed candidate is the named one (forecast {id})"
        );
        // And the hook must not have refused the named candidate: a refusal line means the commit
        // above came from the ordinary fallback, not from recognising `--candidate`.
        assert!(
            !String::from_utf8_lossy(&out.stderr).contains("not committing"),
            "{name}: the hook did not refuse the named candidate: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}

/// No input panics the hook: random strings through PreToolUse for a Bash command. Bounded to
/// finish in under 60 s (defect 4): each iteration spawns a `tsr hook` process, so the count is
/// chosen to stay well inside the bound, and the whole loop is timed.
#[test]
fn no_random_command_panics_the_hook() {
    let dir = scratch("fuzz");
    let session = "m0-candidate-fuzz";
    let (bin, db) = open_session(&dir, session);
    let env = Env {
        cwd: &dir,
        db: &db,
        session,
        bin: &bin,
    };

    // A small xorshift PRNG: deterministic, no dependency.
    let mut state: u64 = 0x9e3779b97f4a7c15;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let alphabet: Vec<char> = "abc tsr exec --candidate '\"\\;|&<>(){}[]*?$`~\n\t#!"
        .chars()
        .collect();
    let started = std::time::Instant::now();
    for i in 0..500 {
        let len = (next() % 40) as usize;
        let command: String = (0..len)
            .map(|_| alphabet[(next() as usize) % alphabet.len()])
            .collect();
        let input = serde_json::json!({ "command": command });
        let out = env.hook("PreToolUse", &pretooluse(session, "Bash", input));
        // The hook exits 0 on every input: a panic would be a non-zero status (101) and a Rust
        // panic message on stderr.
        assert_eq!(
            out.status.code(),
            Some(0),
            "iteration {i}: the hook exits 0 on {command:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(
            !String::from_utf8_lossy(&out.stderr).contains("panicked"),
            "iteration {i}: the hook does not panic on {command:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
    let elapsed = started.elapsed();
    assert!(
        elapsed < std::time::Duration::from_secs(60),
        "the fuzz loop finishes in under 60 s (took {elapsed:?})"
    );
}

/// Defect 1 (race): one forecast authorizes one run *across processes*. Eight `tsr exec --candidate
/// <id>` processes are started at once after a single forecast; exactly one must dispatch and run the
/// container, the other seven must exit 2 having run nothing. Repeated 20 times, because the defect
/// was a read followed by a later append and only showed up in 3 of 18 trials.
///
/// The check and the `Dispatched` append are one critical section under an exclusive advisory lock on
/// a lock file beside the store, so the outcome is deterministic, not merely likely.
#[test]
fn eight_concurrent_execs_after_one_forecast_dispatch_exactly_one() {
    const TRIALS: usize = 20;
    const CONCURRENCY: usize = 8;
    let root = scratch("race");
    for trial in 0..TRIALS {
        let dir = root.join(format!("trial-{trial}"));
        std::fs::create_dir_all(&dir).expect("trial dir");
        let session = format!("m0-candidate-race-{trial}");
        let bin = fake_docker_counting(&dir);
        let db = dir.join(".tsr/tsr.db");
        let env = Env {
            cwd: &dir,
            db: &db,
            session: &session,
            bin: &bin,
        };
        let start = serde_json::json!({
            "session_id": session,
            "hook_event_name": "SessionStart",
            "cwd": dir.to_str().expect("utf8 cwd"),
        })
        .to_string();
        assert!(env.hook("SessionStart", &start).status.success());

        // One forecast. The hook commits the named candidate (as a session would), then the eight
        // `tsr exec` processes race for the single dispatch.
        let id = forecast(&env, "cargo test", "0.9");
        let command = format!("tsr exec --candidate {id} -- cargo test");
        let input = serde_json::json!({ "command": command });
        let out = env.hook("PreToolUse", &pretooluse(&session, "Bash", input));
        assert!(
            out.status.success(),
            "trial {trial}: PreToolUse exits 0: {}",
            String::from_utf8_lossy(&out.stderr)
        );

        // Start all eight at once, then collect. A plain spawn loop staggers the children by the
        // cost of `fork`/`exec` (~5ms each), which is longer than the check+append critical section
        // (~1ms), so they never actually overlap and the test would pass even with the lock
        // removed. Gate them instead: each child signals readiness, then blocks until the parent
        // writes `go`, so all eight enter `tsr exec` within microseconds of each other and the lock
        // is the only thing that can make exactly one win.
        let gate = dir.join("gate");
        std::fs::create_dir_all(&gate).expect("gate dir");
        let go = gate.join("go");
        let wrapper = gate.join("gated-tsr.sh");
        std::fs::write(
            &wrapper,
            format!(
                "#!/bin/sh\n\
                 : > \"{gate}/ready.$$\"\n\
                 while [ ! -e \"{go}\" ]; do sleep 0.001; done\n\
                 exec \"{tsr}\" \"$@\"\n",
                gate = gate.display(),
                go = go.display(),
                tsr = tsr_bin().display(),
            ),
        )
        .expect("write gate wrapper");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&wrapper, std::fs::Permissions::from_mode(0o755))
                .expect("chmod gate wrapper");
        }
        let mut children: Vec<std::process::Child> = Vec::with_capacity(CONCURRENCY);
        for _ in 0..CONCURRENCY {
            let mut cmd = env.command(wrapper.to_str().expect("utf8 path"));
            cmd.args(["exec", "--candidate", &id, "--", "cargo", "test"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::piped());
            children.push(cmd.spawn().expect("spawn tsr exec"));
        }
        // Wait until every child is parked at the gate, then release them together.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            let ready = std::fs::read_dir(&gate)
                .expect("read gate")
                .filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().starts_with("ready."))
                .count();
            if ready >= CONCURRENCY {
                break;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "trial {trial}: only {ready} of {CONCURRENCY} children reached the gate"
            );
            std::thread::sleep(std::time::Duration::from_millis(1));
        }
        std::fs::write(&go, b"go").expect("release the gate");
        let mut dispatched = 0usize;
        let mut refused = 0usize;
        for child in children {
            let out = child.wait_with_output().expect("tsr exec output");
            match out.status.code() {
                Some(0) => dispatched += 1,
                Some(2) => {
                    refused += 1;
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    assert!(
                        stderr.contains("already dispatched"),
                        "trial {trial}: a refused run names the reason: {stderr:?}"
                    );
                }
                other => panic!(
                    "trial {trial}: unexpected exit {other:?}: {}",
                    String::from_utf8_lossy(&out.stderr)
                ),
            }
        }
        assert_eq!(
            dispatched, 1,
            "trial {trial}: exactly one of {CONCURRENCY} concurrent runs dispatches (got {dispatched})"
        );
        assert_eq!(
            refused,
            CONCURRENCY - 1,
            "trial {trial}: the other {} runs exit 2 (got {refused})",
            CONCURRENCY - 1
        );
        assert_eq!(
            container_runs(&dir),
            1,
            "trial {trial}: exactly one container run"
        );

        let trace = env.export(&session);
        let ks = kinds(&trace);
        assert_eq!(
            count(&ks, "dispatched"),
            1,
            "trial {trial}: exactly one dispatch in the log"
        );
        assert_eq!(
            count(&ks, "executed"),
            1,
            "trial {trial}: exactly one execution in the log"
        );
    }
}

/// Defect 3: the "committed after its forecast" half of exec's rule. A forecast that follows the last
/// commit re-arms the candidate id, but the commitment is now *older* than the forecast, so `tsr exec
/// --candidate <id>` called directly must refuse with exit 2 and run nothing.
#[test]
fn exec_refuses_a_candidate_whose_forecast_follows_its_last_commit() {
    let dir = scratch("committed-after-forecast");
    let session = "m0-candidate-committed-after-forecast";
    let bin = fake_docker_counting(&dir);
    let db = dir.join(".tsr/tsr.db");
    let env = Env {
        cwd: &dir,
        db: &db,
        session,
        bin: &bin,
    };
    let start = serde_json::json!({
        "session_id": session,
        "hook_event_name": "SessionStart",
        "cwd": dir.to_str().expect("utf8 cwd"),
    })
    .to_string();
    assert!(env.hook("SessionStart", &start).status.success());

    // Forecast, then the hook commits the named candidate.
    let id = forecast(&env, "cargo test", "0.9");
    let command = format!("tsr exec --candidate {id} -- cargo test");
    let input = serde_json::json!({ "command": command });
    let out = env.hook("PreToolUse", &pretooluse(session, "Bash", input));
    assert!(
        out.status.success(),
        "PreToolUse exits 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    // A second forecast for the same command, after the commit. The candidate id is unchanged, but
    // its newest forecast now comes after its newest commit.
    let id2 = forecast(&env, "cargo test", "0.9");
    assert_eq!(id, id2, "the same command forecasts the same candidate id");

    // Called directly (no hook): the commitment is older than the newest forecast, so exec refuses.
    let out = env.tsr(&["exec", "--candidate", &id, "--", "cargo", "test"]);
    assert_eq!(
        out.status.code(),
        Some(2),
        "a candidate whose forecast follows its last commit exits 2: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("has not been committed after its forecast"),
        "the refusal names the reason: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert_eq!(
        container_runs(&dir),
        0,
        "nothing runs when the candidate is refused"
    );

    let trace = env.export(session);
    let ks = kinds(&trace);
    assert_eq!(count(&ks, "dispatched"), 0, "nothing is dispatched");
    assert_eq!(count(&ks, "executed"), 0, "nothing is executed");
}

/// Defect 5: a malformed `--candidate xyz` is a *named* run the hook cannot vouch for. It must be
/// recorded as an ordinary non-gated call with one stderr line — never as a gated test run with no
/// forecast.
#[test]
fn malformed_candidate_is_recorded_as_an_ordinary_call_not_a_gated_test_run() {
    let dir = scratch("malformed-candidate");
    let session = "m0-candidate-malformed";
    let bin = fake_docker_counting(&dir);
    let db = dir.join(".tsr/tsr.db");
    let env = Env {
        cwd: &dir,
        db: &db,
        session,
        bin: &bin,
    };
    let start = serde_json::json!({
        "session_id": session,
        "hook_event_name": "SessionStart",
        "cwd": dir.to_str().expect("utf8 cwd"),
    })
    .to_string();
    assert!(env.hook("SessionStart", &start).status.success());

    // No forecast at all: the malformed value must not become a gated test run.
    let command = "tsr exec --candidate xyz -- cargo test";
    let input = serde_json::json!({ "command": command });
    let out = env.hook("PreToolUse", &pretooluse(session, "Bash", input));
    assert!(
        out.status.success(),
        "PreToolUse exits 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("not a 64-hex candidate id"),
        "the hook prints one line saying the --candidate value is malformed: {stderr:?}"
    );

    let trace = env.export(session);
    let ks = kinds(&trace);
    assert_eq!(
        count_proposed_test(&ks),
        0,
        "a malformed --candidate mints no run_approved_test candidate"
    );
    assert_eq!(
        count(&ks, "dispatched"),
        0,
        "a malformed --candidate is never dispatched as a test run"
    );
    // It is recorded as an ordinary non-gated call: a `Generate` turn, proposed and committed.
    assert_eq!(
        count(&ks, "committed"),
        1,
        "the call is still recorded (as an ordinary non-gated call)"
    );
}
