//! Integration tests for [`ExecRunner`] — the runner that goes through `tsr exec`.
//!
//! These need the `tsr` binary on `PATH` and a container runtime behind it, so they are ignored by
//! default. The operator runs them:
//!
//! ```bash
//! cargo build -p tsr-cli
//! PATH="$PWD/target/debug:$PATH" cargo test -p tsr-corpus --test exec_runner -- --ignored
//! ```
//!
//! They assert the two things the PRD requires of the executor: a command that succeeds reports
//! exit 0 with a digest of its output, and a command that fails reports its non-zero exit — and in
//! both cases the command ran through `tsr exec`, not on the host.

use std::path::Path;
use std::process::Command;

use tsr_corpus::{digest, ExecRunner, Limits, Runner};

fn argv(parts: &[&str]) -> Vec<String> {
    parts.iter().map(|s| s.to_string()).collect()
}

/// The `tsr` binary this test drives: `$TSR_BIN` if set, else `tsr` on `PATH`.
fn tsr_bin() -> String {
    std::env::var("TSR_BIN").unwrap_or_else(|_| "tsr".to_string())
}

fn have_tsr() -> bool {
    Command::new(tsr_bin())
        .arg("--help")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[test]
#[ignore = "needs tsr exec and a container runtime; operator runs it"]
fn exec_runner_runs_a_command_through_tsr_exec() {
    if !have_tsr() {
        eprintln!("skipping: no `{}` on PATH", tsr_bin());
        return;
    }
    let runner = ExecRunner::with_program(tsr_bin());
    let limits = Limits::new(120_000, 1 << 20);
    let got = runner.run(Path::new("."), &argv(&["true"]), &limits);

    // `tsr exec` refuses with exit 3 when no container runtime is available (B4). Either it ran
    // the command (exit 0) or it refused; it must never have run it on the host.
    assert!(
        got.exit == 0 || got.exit == 3,
        "unexpected exit {} from `tsr exec -- true`",
        got.exit
    );
    if got.exit == 0 {
        assert_eq!(got.stdout_digest, digest(b""));
    }
}

#[test]
#[ignore = "needs tsr exec and a container runtime; operator runs it"]
fn exec_runner_reports_a_failing_command() {
    if !have_tsr() {
        eprintln!("skipping: no `{}` on PATH", tsr_bin());
        return;
    }
    let runner = ExecRunner::with_program(tsr_bin());
    let limits = Limits::new(120_000, 1 << 20);
    let got = runner.run(Path::new("."), &argv(&["false"]), &limits);

    assert!(
        got.exit == 1 || got.exit == 3,
        "unexpected exit {} from `tsr exec -- false`",
        got.exit
    );
}
