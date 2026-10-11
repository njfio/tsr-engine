//! M0 exit test (PRD §B11): one real session, reconstructed from the log alone.
//!
//! This test is RED by design in the skeleton. The M0 lane is done when it is green against a
//! fixture recorded through `tsr hook` — a real Claude Code session on a disposable copy of rts, or
//! the hook's own harness fed with realistic hook payloads — saved at `fixtures/m0/session.events.jsonl`.
//!
//! Do not weaken the invariants to make it pass. Each one is a line of the PRD:
//! - B6: sequence is monotone; every parent precedes its child.
//! - B5: every gated action has a forecast at the same state version, recorded before commitment.
//! - B6: every dispatch is reconciled or explicitly unresolved.
//! - B4: every command ran in the sealed sibling sandbox.

use std::path::PathBuf;
use tsr_cli::reconstruct;

fn fixture() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/m0/session.events.jsonl")
}

#[test]
fn m0_session_reconstructs_from_the_log_alone() {
    let path = fixture();
    assert!(
        path.exists(),
        "M0 fixture missing at {}: record a session through `tsr hook` and save its events.jsonl here",
        path.display()
    );

    let session =
        reconstruct::from_events(&path).expect("the session reconstructs from events.jsonl alone");

    assert!(!session.events.is_empty(), "a session has events");
    assert!(session.monotone_seq(), "B6: sequence is monotone");
    assert!(
        session.every_parent_precedes_its_child(),
        "B6: causality is finite and prior"
    );
    assert!(
        session.every_gated_action_has_prior_forecast_at_same_state_version(),
        "B5: a gated action is denied until a forecast exists for the current state version"
    );
    assert!(
        session.every_dispatch_reconciled_or_unresolved(),
        "B6: every dispatch is reconciled, or explicitly unresolved — never silent"
    );
    assert!(
        session.every_command_ran_in_sandbox(),
        "B4: no command runs outside the sealed sandbox"
    );
}
