//! Refusals. A refusal is a typed value with a process exit code, so the CLI never invents one.

use tsr_domain::{PreferenceId, Source};
use tsr_store::StoreError;

/// Why a ledger write was refused.
#[derive(Debug, thiserror::Error)]
pub enum LedgerError {
    /// `withdrawn` and `out_of_scope` are operator-only (PRD §B11 M2 item 1). Exit 2.
    #[error("status {status:?} on {preference} is operator-only; `by` was {by:?}")]
    OperatorOnly {
        preference: PreferenceId,
        status: String,
        by: Source,
    },
    /// A status names a preference the log does not hold. Exit 2.
    #[error("preference {0} is not in the log")]
    UnknownPreference(PreferenceId),
    /// A status names a preference that is not in the active view. Exit 2.
    #[error("preference {0} is not in the active view")]
    NotActive(PreferenceId),
    /// An expansion whose produced preference is not the content address of its own text. Exit 2.
    #[error("expansion produced {produced} but its text, scope and parents address {expected}")]
    ProducedMismatch {
        produced: PreferenceId,
        expected: PreferenceId,
    },
    /// The domain contract refused the value. Exit 2.
    #[error("contract: {0}")]
    Contract(#[from] tsr_domain::ContractError),
    /// The store refused the append. Exit 1.
    #[error("store: {0}")]
    Store(#[from] StoreError),
}

impl LedgerError {
    /// The process exit code this refusal carries. Operator-only refusals are exit 2 (PRD §B11 M2).
    pub fn exit_code(&self) -> i32 {
        match self {
            LedgerError::Store(_) => 1,
            _ => 2,
        }
    }
}
