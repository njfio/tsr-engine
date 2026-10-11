//! # tsr-ledger — the Culmination ledger as views over the event log
//!
//! Nothing is stored but events (PRD §B2.1, §B6, §B8). The ledger is a fold over the append-only
//! log: an expansion is appended as `EventKind::Expanded { expansion }`, a status as
//! `EventKind::StatusSet { preference, status, by }`, and every read recomputes the view from the
//! hashed history. No weight is stored anywhere.
//!
//! Two views of one log:
//!
//! - **Collective Reality** ([`CollectiveReality`]) — the whole log folded: every preference ever
//!   expanded, with its lineage and its status history.
//! - **Vibrational Reality** ([`VibrationalReality`]) — the active view: the same structure with
//!   statuses applied in sequence order, and admission by source.
//!
//! Admission by source: a preference whose origin is [`Source::Operator`] — including a rejection
//! reason — is open on arrival; one whose origin is [`Source::Generator`] is in the structure but
//! absent from the active view until an Operator `StatusSet(Open)` exists for it.
//!
//! Mechanical statuses: `fulfilled` when the preference's criterion closes (a later return meets
//! it), `overlapping(id)` when a merge finds a duplicate by normalized text within the same scope.
//! `withdrawn` and `out_of_scope` are operator-only and are refused with exit 2 when `by` is not
//! [`Source::Operator`].
//!
//! The produced preference is content-addressed by blake3(text, scope, parents) through
//! [`tsr_domain::Preference::new`]; this crate never re-implements the hash.

#![forbid(unsafe_code)]

pub mod active;
pub mod error;
pub mod filter;
pub mod write;

pub use active::{
    deduplicated_open, duplicate_in_scope, fold, normalize_text, scope_key, CollectiveReality,
    LedgerEntry, VibrationalReality,
};
pub use error::LedgerError;
pub use filter::{dimension_name, dimension_text, Filter};
pub use write::{is_operator_only, status_name, Ledger};

/// The ledger root hash at a fold: the store's `head_root()`, one hash for the whole culmination
/// at any decision (PRD §B6). A convenience for callers that hold a store and nothing else.
pub fn ledger_root(store: &tsr_store::Store) -> Result<tsr_domain::Hash, LedgerError> {
    Ok(store.head_root()?)
}
