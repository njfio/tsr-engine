//! M0 r5: the agent names the candidate; nothing is matched.
//!
//! Operator decision: three rounds tried to make the hook's view of a Bash command and `tsr exec`'s
//! argv agree (r2 digests, r3 shell emulation, r4/r4b pattern matching); each fixed the named
//! spellings and broke others. So nothing is matched. The agent names the candidate:
//!
//! * `tsr forecast --kind <k> --args <text> --p <p>` prints the candidate id alone on stdout and
//!   records the candidate as proposed with that forecast at the current state version;
//! * `tsr exec --candidate <64-hex> -- <argv…>` binds the run to that candidate — it must be a
//!   candidate of the session's task that has a forecast and is not yet dispatched; exec records
//!   `Dispatched` and `Executed` for it and runs argv as today. An unknown, unforecast or
//!   already-dispatched id exits 2 naming the reason and runs nothing;
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

    // The Bash command the agent runs: `tsr exec --candidate <id> -- <inner>`.
    let command = format!("tsr exec --candidate {id} -- {inner}");
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

    let out = env.hook(
        "PostToolUse",
        &posttooluse(&session, "Bash", input, 0),
    );
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
        ("find-exec", "find . -exec echo {} \\;", &["find", ".", "-exec", "echo", "{}", ";"]),
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
    let id = forecast(&env, "cargo test", "0.9");
    let command = format!("tsr exec --candidate {id} -- cargo test");
    let input = serde_json::json!({ "command": command });
    let out = env.hook("PreToolUse", &pretooluse(session, "Bash", input.clone()));
    assert!(out.status.success(), "PreToolUse exits 0: {out:?}");
    let out = env.tsr(&["exec", "--candidate", &id, "--", "cargo", "test"]);
    assert!(out.status.success(), "the first dispatch exits 0: {out:?}");
    let out = env.tsr(&["exec", "--candidate", &id, "--", "cargo", "test"]);
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

/// No input panics the hook: 5,000 random strings through PreToolUse for a Bash command.
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
    let alphabet: Vec<char> = "abc tsr exec --candidate '\"\\;|&<>(){}[]*?$`~\n\t#!".chars().collect();
    for i in 0..5_000 {
        let len = (next() % 40) as usize;
        let command: String = (0..len)
            .map(|_| alphabet[(next() as usize) % alphabet.len()])
            .collect();
        let input = serde_json::json!({ "command": command });
        let out = env.hook("PreToolUse", &pretooluse(session, "Bash", input));
        assert!(
            out.status.success(),
            "iteration {i}: the hook does not panic on {command:?}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
    }
}
