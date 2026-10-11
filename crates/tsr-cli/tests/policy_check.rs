//! Integration test for `tsr policy check <profile.toml> <event.json>` (PRD §B5).
//!
//! Runs the **built binary** on the fixtures under `tests/fixtures/policy/` and asserts the
//! decision, the exit code and that the reason is a single line. The binary is located via
//! `CARGO_BIN_EXE_tsr`, which cargo sets for integration tests of a crate with a `[[bin]]`.

use std::path::PathBuf;
use std::process::Command;

/// The workspace root: `crates/tsr-cli` is two levels below it.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/tsr-cli has a parent")
        .to_path_buf()
}

/// Run the binary on a fixture and return `(exit_code, stdout, stderr)`.
fn run_check(fixture: &str) -> (i32, String, String) {
    let root = workspace_root();
    let profile = root.join("profiles/default.toml");
    let event = root
        .join("crates/tsr-cli/tests/fixtures/policy")
        .join(fixture);
    let out = Command::new(env!("CARGO_BIN_EXE_tsr"))
        .arg("policy")
        .arg("check")
        .arg(&profile)
        .arg(&event)
        .output()
        .expect("the tsr binary must run");
    (
        out.status.code().expect("the process must exit normally"),
        String::from_utf8(out.stdout).expect("stdout is UTF-8"),
        String::from_utf8(out.stderr).expect("stderr is UTF-8"),
    )
}

/// Parse the single JSON object the command prints on stdout.
fn decision_of(stdout: &str) -> (String, String) {
    let v: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("stdout must be one JSON object");
    let decision = v
        .get("decision")
        .and_then(|d| d.as_str())
        .expect("decision is a string")
        .to_string();
    let reason = v
        .get("reason")
        .and_then(|r| r.as_str())
        .expect("reason is a string")
        .to_string();
    (decision, reason)
}

/// A reason is one line: no newline, and non-empty.
fn assert_one_line(reason: &str) {
    assert!(!reason.is_empty(), "reason must not be empty");
    assert!(
        !reason.contains('\n') && !reason.contains('\r'),
        "reason must be one line, got {reason:?}"
    );
}

#[test]
fn injection_readme_does_not_widen_scope() {
    // A README prompt-injection attempt: the proposed action is an Inspect of an absolute path
    // outside the read scope. Repository text carries no authority, so the gate denies.
    let (code, stdout, stderr) = run_check("injection-readme.json");
    assert_eq!(code, 1, "deny must exit 1; stderr={stderr}");
    let (decision, reason) = decision_of(&stdout);
    assert_eq!(decision, "deny", "reason={reason}");
    assert_one_line(&reason);
    assert!(
        reason.contains("absolute") || reason.contains("outside the read scope"),
        "reason must name why the injection failed: {reason}"
    );
}

#[test]
fn gated_action_without_forecast_is_denied_naming_what_to_record() {
    // RunApprovedTest is gated: with no Forecast at the current state version it is denied, and
    // the reason names what to record.
    let (code, stdout, stderr) = run_check("no-forecast.json");
    assert_eq!(code, 1, "deny must exit 1; stderr={stderr}");
    let (decision, reason) = decision_of(&stdout);
    assert_eq!(decision, "deny", "reason={reason}");
    assert_one_line(&reason);
    assert!(
        reason.contains("forecast") && reason.contains("state version 1"),
        "reason must name the forecast to record at the current version: {reason}"
    );
}

#[test]
fn malformed_event_exits_three_with_json_error_on_stderr() {
    let root = workspace_root();
    let profile = root.join("profiles/default.toml");
    let bad = root.join("crates/tsr-cli/tests/fixtures/policy/does-not-exist.json");
    let out = Command::new(env!("CARGO_BIN_EXE_tsr"))
        .arg("policy")
        .arg("check")
        .arg(&profile)
        .arg(&bad)
        .output()
        .expect("the tsr binary must run");
    assert_eq!(out.status.code(), Some(3), "malformed input must exit 3");
    let stderr = String::from_utf8(out.stderr).expect("stderr is UTF-8");
    let v: serde_json::Value =
        serde_json::from_str(stderr.trim()).expect("stderr must be one JSON object");
    assert!(
        v.get("error").is_some(),
        "stderr must carry an error: {stderr}"
    );
}
