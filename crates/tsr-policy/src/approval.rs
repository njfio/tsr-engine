//! Approvals, at the boundary only (PRD §B5).
//!
//! An approval binds **task, artifact digest, repository snapshot digest, scope and expiry**; any
//! change invalidates it, and it is checked again at dispatch. Inside the sandbox nothing needs
//! approval: edits are forecasted, logged and reversible by git. Promotion out, a non-allowlisted
//! command, or any network use needs one.
//!
//! ## Where the digests live
//!
//! The contract's [`EventKind::Approved`] carries only `candidate`, `scope` and `expires_at`
//! (`crates/tsr-domain/src/lib.rs:711-715`). The artifact and snapshot digests are therefore
//! encoded **inside the `scope` string**, so no `tsr-domain` change is needed:
//!
//! ```text
//! scope=<human scope>;artifact=<64 hex>;snapshot=<64 hex>
//! ```
//!
//! [`Approval::from_event`] parses that encoding; [`Approval::scope_string`] produces it. A scope
//! that does not carry both digests yields `None` — an approval that cannot name what it binds is
//! not an approval, and policy is fail-closed.
//!
//! ## Expiry
//!
//! `now` and `expires_at` are RFC 3339 / ISO 8601 UTC strings compared **lexicographically**. For
//! fixed-width UTC timestamps (`YYYY-MM-DDTHH:MM:SSZ`, optionally with a fractional part) that
//! ordering is the same as chronological ordering, which is the assumption this module documents
//! and relies on. An approval is expired when `now >= expires_at` (the boundary is inclusive: at
//! the expiry instant the approval is already gone).

use tsr_domain::{CandidateId, Event, EventKind, Hash, TaskId};

/// An operator approval, bound to everything it authorizes (PRD §B5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Approval {
    /// The task the approval was granted for.
    pub task: TaskId,
    /// The candidate (proposed action) it authorizes.
    pub candidate: CandidateId,
    /// Digest of the artifact (e.g. the diff) it authorizes.
    pub artifact_digest: Hash,
    /// Digest of the repository snapshot it was granted against.
    pub snapshot_digest: Hash,
    /// The human-readable scope, without the encoded digests.
    pub scope: String,
    /// RFC 3339 expiry instant; at or after it the approval is invalid.
    pub expires_at: String,
}

/// Why an approval did not hold at dispatch.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum ApprovalError {
    /// The approval was granted for a different task.
    #[error("approval is for a different task")]
    TaskMismatch,
    /// The approval was granted for a different candidate.
    #[error("approval is for a different candidate")]
    CandidateMismatch,
    /// The artifact changed since the approval was granted.
    #[error("artifact digest changed since approval")]
    ArtifactChanged,
    /// The repository snapshot changed since the approval was granted.
    #[error("repository snapshot digest changed since approval")]
    SnapshotChanged,
    /// The scope changed since the approval was granted.
    #[error("scope changed since approval")]
    ScopeChanged,
    /// The approval has expired.
    #[error("approval expired at {expires_at}")]
    Expired {
        /// The expiry instant the approval carried.
        expires_at: String,
    },
}

impl Approval {
    /// Re-check this approval at dispatch. Fails if task, candidate, artifact digest, snapshot
    /// digest or scope differ, or if `now >= expires_at`. **Any change invalidates it.**
    ///
    /// `now` and `expires_at` are compared as RFC 3339 UTC strings (see the module docs).
    #[allow(clippy::too_many_arguments)]
    pub fn is_valid(
        &self,
        now: &str,
        task: TaskId,
        candidate: CandidateId,
        artifact_digest: Hash,
        snapshot_digest: Hash,
        scope: &str,
    ) -> Result<(), ApprovalError> {
        if self.task != task {
            return Err(ApprovalError::TaskMismatch);
        }
        if self.candidate != candidate {
            return Err(ApprovalError::CandidateMismatch);
        }
        if self.artifact_digest != artifact_digest {
            return Err(ApprovalError::ArtifactChanged);
        }
        if self.snapshot_digest != snapshot_digest {
            return Err(ApprovalError::SnapshotChanged);
        }
        if self.scope != scope {
            return Err(ApprovalError::ScopeChanged);
        }
        if now >= self.expires_at.as_str() {
            return Err(ApprovalError::Expired {
                expires_at: self.expires_at.clone(),
            });
        }
        Ok(())
    }

    /// The `scope` string as it is stored on an [`EventKind::Approved`] event:
    /// `scope=<scope>;artifact=<hex>;snapshot=<hex>`.
    pub fn scope_string(&self) -> String {
        encode_scope(&self.scope, &self.artifact_digest, &self.snapshot_digest)
    }
}

/// Encode a scope plus the two digests into the single `scope` string an `Approved` event carries.
pub fn encode_scope(scope: &str, artifact: &Hash, snapshot: &Hash) -> String {
    format!(
        "scope={scope};artifact={};snapshot={}",
        artifact.to_hex(),
        snapshot.to_hex()
    )
}

/// Parse `scope=<scope>;artifact=<hex>;snapshot=<hex>`. Returns `None` if either digest is missing
/// or malformed. The scope itself may contain `;` — only the trailing `;artifact=` and
/// `;snapshot=` markers are structural.
pub fn decode_scope(s: &str) -> Option<(String, Hash, Hash)> {
    let rest = s.strip_prefix("scope=")?;
    let (scope, tail) = rest.split_once(";artifact=")?;
    let (artifact_hex, snapshot_hex) = tail.split_once(";snapshot=")?;
    let artifact = Hash::from_hex(artifact_hex).ok()?;
    let snapshot = Hash::from_hex(snapshot_hex).ok()?;
    Some((scope.to_string(), artifact, snapshot))
}

/// Read an [`EventKind::Approved`] event into an [`Approval`].
///
/// The event carries `candidate`, `scope` and `expires_at`; the artifact and snapshot digests are
/// parsed out of the `scope` string (see the module docs). Returns `None` for any other event kind,
/// or for an `Approved` event whose scope does not carry both digests.
pub fn from_event(e: &Event) -> Option<Approval> {
    let EventKind::Approved {
        candidate,
        scope,
        expires_at,
    } = &e.kind
    else {
        return None;
    };
    let (scope, artifact_digest, snapshot_digest) = decode_scope(scope)?;
    Some(Approval {
        task: e.task,
        candidate: *candidate,
        artifact_digest,
        snapshot_digest,
        scope,
        expires_at: expires_at.clone(),
    })
}

/// True if a [`EventKind::Denied`] event for `candidate` appears **after** its
/// [`EventKind::Approved`] event in `log` — an approval revoked between decision and dispatch.
///
/// The log is ordered by the store's sequence (PRD §B6: ordering is by `seq` and causation, not by
/// wall clock), so this is a positional scan. A `Denied` with no preceding `Approved` is not a
/// revocation of an approval (there was none to revoke) and returns `false`.
pub fn revoked_between(log: &[Event], candidate: CandidateId) -> bool {
    let mut approved = false;
    for e in log {
        match &e.kind {
            EventKind::Approved { candidate: c, .. } if *c == candidate => approved = true,
            EventKind::Denied { candidate: c, .. } if *c == candidate && approved => return true,
            _ => {}
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tsr_domain::{Event, EventKind, Source};

    fn hash(byte: u8) -> Hash {
        Hash([byte; 32])
    }

    fn task() -> TaskId {
        TaskId(hash(1))
    }

    fn candidate() -> CandidateId {
        CandidateId(hash(2))
    }

    fn approval() -> Approval {
        Approval {
            task: task(),
            candidate: candidate(),
            artifact_digest: hash(0xaa),
            snapshot_digest: hash(0xbb),
            scope: "promote diff to host checkout".to_string(),
            expires_at: "2026-01-01T00:00:00Z".to_string(),
        }
    }

    fn approved_event(a: &Approval) -> Event {
        Event::new(
            a.task,
            vec![],
            Source::Operator,
            EventKind::Approved {
                candidate: a.candidate,
                scope: a.scope_string(),
                expires_at: a.expires_at.clone(),
            },
            "2025-12-31T00:00:00Z",
        )
    }

    #[test]
    fn valid_approval_passes() {
        let a = approval();
        assert_eq!(
            a.is_valid(
                "2025-12-31T23:59:59Z",
                task(),
                candidate(),
                hash(0xaa),
                hash(0xbb),
                "promote diff to host checkout",
            ),
            Ok(())
        );
    }

    #[test]
    fn altered_artifact_digest_invalidates() {
        let a = approval();
        assert_eq!(
            a.is_valid(
                "2025-12-31T00:00:00Z",
                task(),
                candidate(),
                hash(0xcc), // artifact changed
                hash(0xbb),
                "promote diff to host checkout",
            ),
            Err(ApprovalError::ArtifactChanged)
        );
    }

    #[test]
    fn altered_snapshot_digest_invalidates() {
        let a = approval();
        assert_eq!(
            a.is_valid(
                "2025-12-31T00:00:00Z",
                task(),
                candidate(),
                hash(0xaa),
                hash(0xdd), // snapshot changed
                "promote diff to host checkout",
            ),
            Err(ApprovalError::SnapshotChanged)
        );
    }

    #[test]
    fn changed_scope_invalidates() {
        let a = approval();
        assert_eq!(
            a.is_valid(
                "2025-12-31T00:00:00Z",
                task(),
                candidate(),
                hash(0xaa),
                hash(0xbb),
                "promote diff AND run network command", // widened scope
            ),
            Err(ApprovalError::ScopeChanged)
        );
    }

    #[test]
    fn different_task_or_candidate_invalidates() {
        let a = approval();
        assert_eq!(
            a.is_valid(
                "2025-12-31T00:00:00Z",
                TaskId(hash(9)),
                candidate(),
                hash(0xaa),
                hash(0xbb),
                "promote diff to host checkout",
            ),
            Err(ApprovalError::TaskMismatch)
        );
        assert_eq!(
            a.is_valid(
                "2025-12-31T00:00:00Z",
                task(),
                CandidateId(hash(9)),
                hash(0xaa),
                hash(0xbb),
                "promote diff to host checkout",
            ),
            Err(ApprovalError::CandidateMismatch)
        );
    }

    #[test]
    fn expiry_is_inclusive() {
        let a = approval();
        // Exactly at the expiry instant: already expired.
        assert_eq!(
            a.is_valid(
                "2026-01-01T00:00:00Z",
                task(),
                candidate(),
                hash(0xaa),
                hash(0xbb),
                "promote diff to host checkout",
            ),
            Err(ApprovalError::Expired {
                expires_at: "2026-01-01T00:00:00Z".to_string()
            })
        );
        // One second later: still expired.
        assert!(matches!(
            a.is_valid(
                "2026-01-01T00:00:01Z",
                task(),
                candidate(),
                hash(0xaa),
                hash(0xbb),
                "promote diff to host checkout",
            ),
            Err(ApprovalError::Expired { .. })
        ));
    }

    #[test]
    fn scope_encoding_round_trips_through_an_event() {
        let a = approval();
        let e = approved_event(&a);
        let parsed = from_event(&e).expect("approved event must parse");
        assert_eq!(parsed, a);
    }

    #[test]
    fn from_event_rejects_a_scope_without_digests() {
        let e = Event::new(
            task(),
            vec![],
            Source::Operator,
            EventKind::Approved {
                candidate: candidate(),
                scope: "promote diff".to_string(), // no artifact/snapshot
                expires_at: "2026-01-01T00:00:00Z".to_string(),
            },
            "2025-12-31T00:00:00Z",
        );
        assert_eq!(from_event(&e), None);
    }

    #[test]
    fn from_event_ignores_other_kinds() {
        let e = Event::new(
            task(),
            vec![],
            Source::Operator,
            EventKind::Denied {
                candidate: candidate(),
                reason: "no".to_string(),
            },
            "2025-12-31T00:00:00Z",
        );
        assert_eq!(from_event(&e), None);
    }

    #[test]
    fn revoked_between_decision_and_dispatch() {
        let a = approval();
        let approved = approved_event(&a);
        let denied = Event::new(
            task(),
            vec![approved.id],
            Source::Operator,
            EventKind::Denied {
                candidate: candidate(),
                reason: "operator revoked".to_string(),
            },
            "2025-12-31T00:00:01Z",
        );

        // Approved then denied: revoked.
        assert!(revoked_between(&[approved.clone(), denied.clone()], candidate()));
        // Denied then approved: the later approval stands.
        assert!(!revoked_between(&[denied.clone(), approved.clone()], candidate()));
        // Approved only: not revoked.
        assert!(!revoked_between(&[approved.clone()], candidate()));
        // Denied only (no approval to revoke): not a revocation.
        assert!(!revoked_between(&[denied], candidate()));
        // A denial for a different candidate does not revoke this one.
        let other = Event::new(
            task(),
            vec![],
            Source::Operator,
            EventKind::Denied {
                candidate: CandidateId(hash(7)),
                reason: "other".to_string(),
            },
            "2025-12-31T00:00:02Z",
        );
        assert!(!revoked_between(&[approved, other], candidate()));
    }
}