//! M0's exit evidence (PRD B11): "one real session on a disposable copy of rts is fully reconstructed
//! from the log; every command ran in the sandbox".
//!
//! `fixtures/m0/claude-session.events.jsonl` is the log of a real Claude Code session (not the
//! selftest) on a disposable clone of rts: it added a unit test to rts-core and ran the crate's tests
//! through `tsr exec`, with `tsr hook` registered for SessionStart, PreToolUse, PostToolUse and Stop.
//! The operator exported it with `tsr task export`; how it was run is in docs/m0-evidence.md ("The
//! real Claude Code session"). Nothing here regenerates it.

use serde_json::Value;
use std::path::PathBuf;
use tsr_cli::reconstruct;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/m0/claude-session.events.jsonl")
}

fn kinds() -> Vec<Value> {
    std::fs::read_to_string(fixture())
        .expect("the real session's log")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| {
            let v: Value = serde_json::from_str(l).expect("an event line");
            v["event"]["kind"].clone()
        })
        .collect()
}

#[test]
fn the_real_session_reconstructs_and_keeps_every_rule() {
    let session = reconstruct::from_events(&fixture()).expect("reconstructs from the log alone");
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

/// The frozen test passes on a log with nothing in it but a task opening. The real session must
/// show the milestone's whole path: a forecast patch, a forecast test run dispatched into the sealed
/// sandbox and executed there, and a stop.
#[test]
fn the_real_session_exercises_the_whole_path() {
    let kinds = kinds();
    let count = |tag: &str| kinds.iter().filter(|k| k["kind"] == tag).count();
    let proposed_patch = kinds
        .iter()
        .filter(|k| k["kind"] == "proposed" && k["candidate"]["kind"] == "propose_patch")
        .count();
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
    let passed = kinds
        .iter()
        .filter(|k| k["kind"] == "executed" && k["exit_code"] == 0)
        .count();
    assert_eq!(count("task_opened"), 1, "one task opened");
    assert!(proposed_patch >= 1, "a patch was proposed");
    assert!(proposed_test >= 1, "a test run was proposed");
    assert!(count("forecast") >= 2, "both gated actions were forecast");
    assert!(sandboxed >= 1, "a command was dispatched into the sandbox");
    assert_eq!(host, 0, "no command was dispatched on the host");
    assert!(passed >= 1, "the sandboxed test run executed and passed");
    assert_eq!(count("stop"), 1, "the session stopped");
}
