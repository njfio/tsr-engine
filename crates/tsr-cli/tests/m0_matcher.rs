//! M0 r4: the hook *matches* the command instead of emulating the shell.
//!
//! The hook parses the Bash command with shell grammar only and records a *pattern* for the
//! `tsr exec` simple command; `tsr exec` reuses the hook's pending candidate when exactly one
//! pattern matches its argv. These tests drive the built `tsr` binary through `tsr hook` and
//! `tsr exec` with a real `bash -c` for the command, as the r3 review did.

use std::path::{Path, PathBuf};
use std::process::Command;

use tsr_cli::reconstruct;

fn tsr_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_tsr"))
}

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/m0-matcher")
        .join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

fn fake_docker(dir: &Path) -> PathBuf {
    let bin = dir.join("bin");
    std::fs::create_dir_all(&bin).expect("fake bin dir");
    let script = "#!/bin/sh\ncase \"$1\" in\n  --version) exit 0 ;;\n  info) exit 0 ;;\n  run) echo \"fake docker run\"; exit 0 ;;\n  *) exit 0 ;;\nesac\n";
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

fn path_with(bin: &Path) -> String {
    let real = std::env::var("PATH").unwrap_or_default();
    format!("{}:{}", bin.display(), real)
}

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

    fn tsr(&self, args: &[&str]) -> std::process::Output {
        let mut cmd = self.command(tsr_bin().to_str().expect("utf8 path").as_ref());
        cmd.args(args);
        cmd.output().expect("run tsr")
    }

    fn export(&self, session: &str) -> String {
        let task = tsr_cli::store_path::task_for_session(session).to_string();
        let out = self.tsr(&["task", "export", &task]);
        assert!(
            out.status.success(),
            "tsr task export exits 0: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).expect("utf8 trace")
    }
}

fn pretooluse(session: &str, cwd: &Path, tool: &str, input: serde_json::Value) -> String {
    serde_json::json!({
        "session_id": session,
        "hook_event_name": "PreToolUse",
        "tool_name": tool,
        "tool_input": input,
        "cwd": cwd.to_str().expect("utf8 cwd"),
    })
    .to_string()
}

fn posttooluse(
    session: &str,
    cwd: &Path,
    tool: &str,
    input: serde_json::Value,
    exit_code: i32,
) -> String {
    serde_json::json!({
        "session_id": session,
        "hook_event_name": "PostToolUse",
        "tool_name": tool,
        "tool_input": input,
        "tool_response": { "stdout": "ok", "stderr": "", "exit_code": exit_code },
        "cwd": cwd.to_str().expect("utf8 cwd"),
    })
    .to_string()
}

fn count_proposed_and_forecast(trace: &str) -> (usize, usize) {
    let mut proposed = 0;
    let mut forecast = 0;
    for line in trace.lines().filter(|l| !l.trim().is_empty()) {
        let v: serde_json::Value = serde_json::from_str(line).expect("an event line");
        match v["event"]["kind"]["kind"].as_str() {
            Some("proposed") => proposed += 1,
            Some("forecast") => forecast += 1,
            _ => {}
        }
    }
    (proposed, forecast)
}

fn last_proposed_kind(trace: &str) -> Option<String> {
    let mut kind = None;
    for line in trace.lines().filter(|l| !l.trim().is_empty()) {
        let v: serde_json::Value = serde_json::from_str(line).expect("an event line");
        if v["event"]["kind"]["kind"] == "proposed" {
            kind = v["event"]["kind"]["candidate"]["kind"]
                .as_str()
                .map(str::to_string);
        }
    }
    kind
}

/// Drive one typed spelling end to end and return the exported trace.
fn record_typed_spelling(
    dir: &Path,
    session: &str,
    bash_cmd: &str,
    forecast_args: &str,
    exec_argv: &[&str],
) -> PathBuf {
    let bin = fake_docker(dir);
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

    let out = env.tsr(&[
        "forecast",
        "--kind",
        "run_approved_test",
        "--args",
        forecast_args,
        "--p",
        "0.9",
        "--source",
        "generator",
    ]);
    assert!(
        out.status.success(),
        "tsr forecast --args {forecast_args:?} exits 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let input = serde_json::json!({ "command": bash_cmd });
    let out = env.hook(
        "PreToolUse",
        &pretooluse(session, dir, "Bash", input.clone()),
    );
    assert!(
        out.status.success(),
        "PreToolUse for {bash_cmd:?} exits 0: {out:?}"
    );

    let mut exec_args = vec!["exec", "--"];
    exec_args.extend_from_slice(exec_argv);
    let out = env.tsr(&exec_args);
    assert!(
        out.status.success(),
        "tsr exec -- {exec_argv:?} exits 0: {}",
        String::from_utf8_lossy(&out.stderr)
    );

    let out = env.hook("PostToolUse", &posttooluse(session, dir, "Bash", input, 0));
    assert!(
        out.status.success(),
        "PostToolUse for {bash_cmd:?} exits 0: {out:?}"
    );

    let stop = serde_json::json!({
        "session_id": session,
        "hook_event_name": "Stop",
        "cwd": dir.to_str().expect("utf8 cwd"),
    })
    .to_string();
    let out = env.hook("Stop", &stop);
    assert!(out.status.success(), "Stop exits 0: {out:?}");

    let text = env.export(session);
    let trace = dir.join("events.jsonl");
    std::fs::write(&trace, &text).expect("write trace");
    trace
}

fn assert_one_proposal_one_forecast_and_reconstructs(trace: &Path, what: &str) {
    let text = std::fs::read_to_string(trace).expect("the session's log");
    let (proposed, forecast) = count_proposed_and_forecast(&text);
    if proposed != 1 {
        eprintln!("=== TRACE {what} ===\n{text}");
    }
    assert_eq!(proposed, 1, "{what}: exactly one proposal (got {proposed})");
    assert_eq!(forecast, 1, "{what}: exactly one forecast (got {forecast})");
    let session = reconstruct::from_events(trace).expect("reconstruct accepts the log");
    assert!(
        session.every_gated_action_has_prior_forecast_at_same_state_version(),
        "{what}: B5 holds"
    );
    assert!(
        session.every_dispatch_reconciled_or_unresolved(),
        "{what}: B6 holds"
    );
    assert!(session.every_command_ran_in_sandbox(), "{what}: B4 holds");
}

// ---------------------------------------------------------------------------------------------
// M0 r4: the hook matches the command instead of emulating the shell (r3 review defects 1–10).
// ---------------------------------------------------------------------------------------------

/// The r3 review's defect spellings must each give one proposal, one forecast and a log that
/// reconstruct accepts. The hook no longer evaluates expansions; it records a pattern and
/// `tsr exec` matches its argv against it.
#[test]
fn the_r3_review_defect_spellings_each_give_one_candidate() {
    // (defect 1) a quoted glob must NOT be expanded: `find . -name "*.toml"` with a Cargo.toml
    // present. The hook records the literal `*.toml`; exec receives the literal `*.toml`.
    let dir = scratch("r4-quoted-glob");
    std::fs::write(dir.join("Cargo.toml"), "[package]\n").expect("write Cargo.toml");
    let trace = record_typed_spelling(
        &dir,
        "r4-quoted-glob",
        "tsr exec -- find . -name \"*.toml\"",
        "find . -name *.toml",
        &["find", ".", "-name", "*.toml"],
    );
    assert_one_proposal_one_forecast_and_reconstructs(&trace, "quoted-glob");

    // (defect 2) `{}` must not split a word: `find . -exec echo {} \;`.
    let dir = scratch("r4-brace");
    let trace = record_typed_spelling(
        &dir,
        "r4-brace",
        "tsr exec -- find . -exec echo {} \\;",
        "find . -exec echo {} ;",
        &["find", ".", "-exec", "echo", "{}", ";"],
    );
    assert_one_proposal_one_forecast_and_reconstructs(&trace, "brace");

    // (defect 2) `HEAD@{1}` must not split: `git show HEAD@{1}`.
    let dir = scratch("r4-head");
    let trace = record_typed_spelling(
        &dir,
        "r4-head",
        "tsr exec -- git show HEAD@{1}",
        "git show HEAD@{1}",
        &["git", "show", "HEAD@{1}"],
    );
    assert_one_proposal_one_forecast_and_reconstructs(&trace, "head");

    // (defect 2) `xargs -I{} echo {}`.
    let dir = scratch("r4-xargs");
    let trace = record_typed_spelling(
        &dir,
        "r4-xargs",
        "tsr exec -- xargs -I{} echo {}",
        "xargs -I{} echo {}",
        &["xargs", "-I{}", "echo", "{}"],
    );
    assert_one_proposal_one_forecast_and_reconstructs(&trace, "xargs");

    // (defect 3) `$(…)` must not be mangled: `echo $(date)`.
    let dir = scratch("r4-cmdsub");
    let trace = record_typed_spelling(
        &dir,
        "r4-cmdsub",
        "tsr exec -- echo $(date)",
        "echo 2024-01-01",
        &["echo", "2024-01-01"],
    );
    assert_one_proposal_one_forecast_and_reconstructs(&trace, "cmdsub");

    // (defect 3) process substitution `<(…)`/`>(…)` is one unit: `diff <(echo a) <(echo b)`.
    let dir = scratch("r4-procsub");
    let trace = record_typed_spelling(
        &dir,
        "r4-procsub",
        "tsr exec -- diff <(echo a) <(echo b)",
        "diff /dev/fd/63 /dev/fd/62",
        &["diff", "/dev/fd/63", "/dev/fd/62"],
    );
    assert_one_proposal_one_forecast_and_reconstructs(&trace, "procsub");

    // (defect 4) backslash-newline continuation: `cargo test \<newline>--release`.
    let dir = scratch("r4-continuation");
    let trace = record_typed_spelling(
        &dir,
        "r4-continuation",
        "tsr exec -- cargo test \\\n--release",
        "cargo test --release",
        &["cargo", "test", "--release"],
    );
    assert_one_proposal_one_forecast_and_reconstructs(&trace, "continuation");

    // (defect 5) a `#` comment: `cargo test # run tests`.
    let dir = scratch("r4-comment");
    let trace = record_typed_spelling(
        &dir,
        "r4-comment",
        "tsr exec -- cargo test # run tests",
        "cargo test",
        &["cargo", "test"],
    );
    assert_one_proposal_one_forecast_and_reconstructs(&trace, "comment");

    // (defect 6) an empty expansion must not be kept as a word: `echo $UNSET x`.
    let dir = scratch("r4-empty");
    let trace = record_typed_spelling(
        &dir,
        "r4-empty",
        "tsr exec -- echo $UNSET x",
        "echo x",
        &["echo", "x"],
    );
    assert_one_proposal_one_forecast_and_reconstructs(&trace, "empty");

    // (defect 7) `ls *` with a dotfile present: the dotfile is not globbed.
    let dir = scratch("r4-dotfile");
    std::fs::write(dir.join(".hidden"), "x").expect("write dotfile");
    std::fs::write(dir.join("visible"), "x").expect("write visible");
    let trace = record_typed_spelling(
        &dir,
        "r4-dotfile",
        "tsr exec -- ls *",
        "ls visible",
        &["ls", "visible"],
    );
    assert_one_proposal_one_forecast_and_reconstructs(&trace, "dotfile");

    // (defect 8) a directory wildcard: `ls crates/*/Cargo.toml`.
    let dir = scratch("r4-dirglob");
    std::fs::create_dir_all(dir.join("crates/a")).expect("mkdir crates/a");
    std::fs::write(dir.join("crates/a/Cargo.toml"), "[package]\n").expect("write toml");
    let trace = record_typed_spelling(
        &dir,
        "r4-dirglob",
        "tsr exec -- ls crates/*/Cargo.toml",
        "ls crates/a/Cargo.toml",
        &["ls", "crates/a/Cargo.toml"],
    );
    assert_one_proposal_one_forecast_and_reconstructs(&trace, "dirglob");
}

/// A here-document body is data, not commands: an apostrophe in the body before `tsr exec` must
/// not confuse the parser (defect 5).
#[test]
fn a_heredoc_body_is_data_not_commands() {
    let dir = scratch("r4-heredoc");
    let trace = record_typed_spelling(
        &dir,
        "r4-heredoc",
        "cat <<'EOF'\nit's a body\nEOF\ntsr exec -- cargo test",
        "cargo test",
        &["cargo", "test"],
    );
    assert_one_proposal_one_forecast_and_reconstructs(&trace, "heredoc");
}

/// Prefixes before `tsr exec` are skipped: `time`, `nice`, `timeout -s KILL 60`, `nohup`.
#[test]
fn prefixes_before_tsr_exec_are_skipped() {
    for (name, bash_cmd) in [
        ("time", "time tsr exec -- cargo test"),
        ("nice", "nice tsr exec -- cargo test"),
        ("nice-n", "nice -n 5 tsr exec -- cargo test"),
        ("timeout-s", "timeout -s KILL 60 tsr exec -- cargo test"),
        ("nohup", "nohup tsr exec -- cargo test"),
    ] {
        let dir = scratch(&format!("r4-{name}"));
        let trace = record_typed_spelling(
            &dir,
            &format!("r4-{name}"),
            bash_cmd,
            "cargo test",
            &["cargo", "test"],
        );
        assert_one_proposal_one_forecast_and_reconstructs(&trace, name);
    }
}

/// Malformed input never panics: a trailing `env -u` is a plain word list and exits 0 (defect 3).
#[test]
fn a_trailing_env_u_does_not_panic() {
    let dir = scratch("r4-env-u");
    let bin = fake_docker(&dir);
    let db = dir.join(".tsr/tsr.db");
    let env = Env {
        cwd: &dir,
        db: &db,
        session: "r4-env-u",
        bin: &bin,
    };
    let start = serde_json::json!({
        "session_id": "r4-env-u",
        "hook_event_name": "SessionStart",
        "cwd": dir.to_str().expect("utf8 cwd"),
    })
    .to_string();
    let out = env.hook("SessionStart", &start);
    assert!(out.status.success(), "SessionStart exits 0: {out:?}");

    for cmd in ["env -u", "nice -n", "env -C"] {
        let input = serde_json::json!({ "command": cmd });
        let out = env.hook("PreToolUse", &pretooluse("r4-env-u", &dir, "Bash", input));
        assert!(
            out.status.success(),
            "PreToolUse for {cmd:?} exits 0 (no panic): {out:?}"
        );
    }
}

/// The test-run spellings with a timeout record `run_approved_test`.
#[test]
fn timeout_test_run_spellings_record_run_approved_test() {
    for (name, cmd) in [
        ("timeout-s-kill", "timeout -s KILL 60 cargo test"),
        ("timeout-k", "timeout -k 5 60 cargo test"),
    ] {
        let dir = scratch(&format!("r4-{name}"));
        let bin = fake_docker(&dir);
        let db = dir.join(".tsr/tsr.db");
        let session = format!("r4-{name}");
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
        let out = env.hook("SessionStart", &start);
        assert!(out.status.success(), "SessionStart exits 0: {out:?}");

        let input = serde_json::json!({ "command": cmd });
        let out = env.hook("PreToolUse", &pretooluse(&session, &dir, "Bash", input));
        assert!(
            out.status.success(),
            "PreToolUse for {cmd:?} exits 0: {out:?}"
        );

        let text = env.export(&session);
        assert_eq!(
            last_proposed_kind(&text).as_deref(),
            Some("run_approved_test"),
            "`{cmd}` records run_approved_test"
        );
    }
}

/// The matcher is linear: ten `*a` patterns against a 40-character name return in under a second
/// (defect 10, the exponential glob matcher).
#[test]
fn the_matcher_is_linear() {
    let dir = scratch("r4-linear");
    let bin = fake_docker(&dir);
    let db = dir.join(".tsr/tsr.db");
    let env = Env {
        cwd: &dir,
        db: &db,
        session: "r4-linear",
        bin: &bin,
    };
    let start = serde_json::json!({
        "session_id": "r4-linear",
        "hook_event_name": "SessionStart",
        "cwd": dir.to_str().expect("utf8 cwd"),
    })
    .to_string();
    let out = env.hook("SessionStart", &start);
    assert!(out.status.success(), "SessionStart exits 0: {out:?}");

    let name = "a".repeat(40);
    let pattern = "*a".repeat(10);
    let cmd = format!("tsr exec -- echo {pattern}");
    let input = serde_json::json!({ "command": cmd });
    let started = std::time::Instant::now();
    let out = env.hook("PreToolUse", &pretooluse("r4-linear", &dir, "Bash", input));
    let elapsed = started.elapsed();
    assert!(out.status.success(), "PreToolUse exits 0: {out:?}");
    assert!(
        elapsed < std::time::Duration::from_secs(1),
        "ten `*a` against a 40-char name must return in under a second (took {elapsed:?})"
    );
    let _ = name;
}
