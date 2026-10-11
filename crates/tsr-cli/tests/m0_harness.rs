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
//!    into every event id, so the harness pins the clock (`SELFTEST_RECORDED_AT`).
//! 2. A real PreToolUse payload piped on stdin is recorded and the process exits 0 — the hook
//!    records and never blocks (M0).

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

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
fn a_real_pretooluse_payload_is_recorded_and_never_blocks() {
    let dir = scratch("payload");
    let db = dir.join("tsr.db");

    // A realistic Claude Code PreToolUse payload for a Read.
    let payload = r#"{"session_id":"m0-harness-session","hook_event_name":"PreToolUse","tool_name":"Read","tool_input":{"file_path":"crates/tsr-cli/src/lib.rs"},"cwd":"/workspace"}"#;

    let mut child = Command::new(tsr_bin())
        .args(["hook", "--stage", "PreToolUse"])
        .env("TSR_DB", &db)
        .current_dir(&dir)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn tsr hook");
    {
        use std::io::Write;
        child
            .stdin
            .as_mut()
            .expect("stdin")
            .write_all(payload.as_bytes())
            .expect("write payload");
    }
    let out = child.wait_with_output().expect("wait for tsr hook");

    assert!(
        out.status.success(),
        "the hook must exit 0 and never block: status={:?} stderr={}",
        out.status,
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        db.exists(),
        "the hook recorded the payload into the store at {}",
        db.display()
    );
}
