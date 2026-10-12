//! D2: the common decision event id must equal the id main's hook records.
//!
//! `crates/tsr-cli/src/hook.rs` mints the common decision event as
//! `OutcomeDefId(Hash::of(&[DECISION_EVENT.as_bytes()]))` where `DECISION_EVENT` is
//! `"task_accepted_in_budget@v1"`. `tsr-forecast`'s `common_decision_event_id()` must mint the
//! *same* id, or every forecast the hook records is never scored.
//!
//! The fixture `fixtures/m0/claude-session.events.jsonl` is a real hook-recorded session: its
//! `forecast` events carry the `event` id the hook wrote. This test reads that id and asserts it
//! equals `tsr_forecast::common_decision_event_id()`.

use std::path::PathBuf;

use tsr_domain::{Hash, OutcomeDefId};

/// The workspace root: `CARGO_MANIFEST_DIR` is `crates/tsr-forecast`.
fn workspace_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("crates/tsr-forecast has a workspace root")
        .to_path_buf()
}

/// The `event` id of the first hook-recorded `forecast` event in the fixture.
fn hook_recorded_common_event_id() -> OutcomeDefId {
    let path = workspace_root().join("fixtures/m0/claude-session.events.jsonl");
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(line)
            .unwrap_or_else(|e| panic!("parse fixture line: {e}\n{line}"));
        let kind = &value["event"]["kind"];
        if kind["kind"] == "forecast" {
            let event = kind["forecast"]["event"]
                .as_str()
                .expect("a forecast event carries an `event` id");
            return OutcomeDefId(Hash::from_hex(event).expect("a hex event id"));
        }
    }
    panic!("no forecast event in {}", path.display());
}

#[test]
fn d2_common_decision_event_id_matches_hook_recorded_event() {
    let recorded = hook_recorded_common_event_id();
    let minted = tsr_forecast::common_decision_event_id();
    assert_eq!(
        minted, recorded,
        "tsr-forecast common_decision_event_id must equal the id the hook records (hook.rs DECISION_EVENT = task_accepted_in_budget@v1)"
    );
}

#[test]
fn d2_outcome_def_id_commits_to_name_at_version() {
    // The id is blake3 of the literal `name@vN`, the same string the hook hashes.
    let expected = OutcomeDefId(Hash::of(&[b"task_accepted_in_budget@v1"]));
    assert_eq!(tsr_forecast::common_decision_event_id(), expected);
    assert_eq!(
        tsr_forecast::outcome_def_id("diff_accepted", 1),
        OutcomeDefId(Hash::of(&[b"diff_accepted@v1"]))
    );
    assert_eq!(
        tsr_forecast::outcome_def_id("test_pass", 2),
        OutcomeDefId(Hash::of(&[b"test_pass@v2"]))
    );
}
