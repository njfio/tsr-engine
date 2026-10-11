//! # tsr-learn — Tier 1 calibration (PRD §B8, §B11 M4)
//!
//! The calibration half of M4: the common decision event as a pure view over the log, the
//! features@v1 schema, the dataset with its split and leakage probes, the logistic head and its
//! isotonic calibrator, support, and content-addressed bundles.
//!
//! This crate depends on `tsr-domain` and `tsr-store` only — never on `tsr-cli` — and is
//! **read-only** on the log. Nothing here writes a fitted parameter to the log or the store: the
//! bundle is the one home for fitted parameters (§B8, §B10; AGENTS.md "no stored weights" governs
//! ledger strengths).
//!
//! ## The common decision event
//!
//! Every candidate is scored on one event, `task_accepted_in_budget@v1` (§B7, §B8). Its id is the
//! one main's hook mints — `OutcomeDefId(Hash::of(&[b"task_accepted_in_budget@v1"]))` — and it is
//! defined **once**, here, in [`decision_event_id`]. Lane 04's `tsr-forecast` mints
//! `blake3(name, version)` instead; the operator reconciles the two at merge.

#![forbid(unsafe_code)]

pub mod dataset;
pub mod decision;
pub mod features;
pub mod head;
pub mod isotonic;
pub mod probes;
pub mod rng;
pub mod support;
pub mod synth;

pub use decision::{
    decision_rows, resolve_labels, DecisionRow, DiffLabel, Label, LabelRecord, TaskLabel,
};

use tsr_domain::{Hash, OutcomeDefId};

/// The name of the common decision event (§B7, §B8). Every candidate forecasts it.
pub const DECISION_EVENT: &str = "task_accepted_in_budget@v1";

/// The id of the common decision event, minted exactly as `crates/tsr-cli/src/hook.rs` mints it:
/// `OutcomeDefId(Hash::of(&[DECISION_EVENT.as_bytes()]))`. Defined once, here.
pub fn decision_event_id() -> OutcomeDefId {
    OutcomeDefId(Hash::of(&[DECISION_EVENT.as_bytes()]))
}

/// Everything `tsr-learn` can refuse.
#[derive(Debug, thiserror::Error)]
pub enum LearnError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("store: {0}")]
    Store(#[from] tsr_store::StoreError),
    #[error("a label record names reviewer {reviewer:?}, who is not the operator (B8: labels only after review)")]
    NonOperatorLabel { reviewer: String },
    #[error("invalid input: {0}")]
    Invalid(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decision_event_id_matches_the_hook() {
        // hook.rs line 635: OutcomeDefId(Hash::of(&[DECISION_EVENT.as_bytes()]))
        assert_eq!(
            decision_event_id(),
            OutcomeDefId(Hash::of(&[b"task_accepted_in_budget@v1"]))
        );
        assert_eq!(DECISION_EVENT, "task_accepted_in_budget@v1");
    }
}
