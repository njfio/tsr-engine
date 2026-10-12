//! The ledger's writes: two event kinds, nothing else (PRD §B2.1, §B6, §B8).
//!
//! An expansion is appended as `EventKind::Expanded { expansion }`; a status as
//! `EventKind::StatusSet { preference, status, by }`. Nothing is stored but events: the ledger is
//! a fold over the log, and every write here is an append.

use tsr_domain::{
    ContrastId, Event, EventId, EventKind, Expansion, Preference, PreferenceId, PreferenceStatus,
    Scope, Source, TaskId,
};
use tsr_store::Store;

use crate::active::{duplicate_in_scope, fold, CollectiveReality};
use crate::error::LedgerError;

/// The ledger over one store. Holds no state of its own: every read is a fold over the log.
pub struct Ledger<'a> {
    store: &'a Store,
}

impl<'a> Ledger<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self { store }
    }

    pub fn store(&self) -> &'a Store {
        self.store
    }

    /// Fold the whole log. Collective Reality, with the store's `head_root()` at the fold.
    pub fn collective(&self) -> Result<CollectiveReality, LedgerError> {
        let events = self.store.all()?;
        let root = self.store.head_root()?;
        Ok(fold(&events, root))
    }

    /// Fold one task's events. The root is still the whole log's head: one hash for the whole
    /// culmination at any decision (PRD §B6).
    pub fn collective_for_task(&self, task: &TaskId) -> Result<CollectiveReality, LedgerError> {
        let events = self.store.by_task(task)?;
        let root = self.store.head_root()?;
        Ok(fold(&events, root))
    }

    /// The active view: statuses applied, admission by source.
    pub fn active(&self) -> Result<crate::active::VibrationalReality, LedgerError> {
        Ok(self.collective()?.active())
    }

    /// The active view over one task's events.
    pub fn active_for_task(
        &self,
        task: &TaskId,
    ) -> Result<crate::active::VibrationalReality, LedgerError> {
        Ok(self.collective_for_task(task)?.active())
    }

    /// Append an expansion: contrast crossed as expanded wanting.
    ///
    /// The produced preference is content-addressed by blake3(text, scope, parents) through
    /// [`Preference::new`]. The event carries the preference itself, so the fold reads the node
    /// from the log and never re-derives it. Returns the produced preference's id.
    #[allow(clippy::too_many_arguments)]
    pub fn expand(
        &self,
        task: &TaskId,
        parents: Vec<EventId>,
        from_contrast: ContrastId,
        text: impl Into<String>,
        scope: Scope,
        preference_parents: Vec<PreferenceId>,
        catalyzed_by: Vec<PreferenceId>,
        origin: Source,
        recorded_at: impl Into<String>,
    ) -> Result<PreferenceId, LedgerError> {
        // The Filter's write rule: only a Contrast the log holds, with at least one unmet
        // criterion, may expand (PRD §B2.1). An expansion that names anything else is refused
        // (exit 2), so the Filter is the only path from a return to a preference.
        let contrast = self.contrast_in_log(&from_contrast)?;
        if contrast.unmet.is_empty() {
            return Err(LedgerError::ContrastWithoutUnmet(from_contrast));
        }
        let preference = Preference::new(text, scope, preference_parents, origin)?;
        let expansion = Expansion {
            from_contrast,
            produced: preference.id,
            catalyzed_by,
            text: preference.text.clone(),
            scope: preference.scope.clone(),
            parents: preference.parents.clone(),
            origin: preference.origin,
        };
        // `produced` must be the content address of (text, scope, parents): refuse otherwise.
        let expected = Preference::new(
            expansion.text.clone(),
            expansion.scope.clone(),
            expansion.parents.clone(),
            expansion.origin,
        )?;
        if expected.id != expansion.produced {
            return Err(LedgerError::ProducedMismatch {
                produced: expansion.produced,
                expected: expected.id,
            });
        }
        let kind = EventKind::Expanded {
            expansion: expansion.clone(),
        };
        let ev = Event::new(*task, parents, origin, kind, recorded_at);
        self.store.append(&ev)?;
        Ok(expansion.produced)
    }

    /// Record a Contrast as an `Observed` event, so a later [`Ledger::expand`] may name it.
    ///
    /// The Filter's write rule (PRD §B2.1) admits an expansion only from a Contrast the log
    /// holds; this is how a contrast enters the log. The contrast's `id` must be the content
    /// address of its own fields (D12): a record whose id is not its content address is refused
    /// with exit 2, so the same contrast is always the same id. Returns the appended event's id.
    pub fn record_contrast(
        &self,
        task: &TaskId,
        parents: Vec<EventId>,
        contrast: tsr_domain::Contrast,
        recorded_at: impl Into<String>,
    ) -> Result<EventId, LedgerError> {
        // The id must be the content address of (from_return, unmet, evidence), the same address
        // `Filter::contrast_from_criteria` mints. Refuse otherwise (exit 2).
        let expected =
            crate::filter::contrast_id(&contrast.from_return, &contrast.unmet, &contrast.evidence);
        if expected != contrast.id {
            return Err(LedgerError::ContrastIdMismatch {
                id: contrast.id,
                expected,
            });
        }
        let kind = EventKind::Observed {
            observation: tsr_domain::Observation::Return {
                surprise: Vec::new(),
                contrast: Some(contrast),
            },
        };
        let ev = Event::new(*task, parents, Source::Rules, kind, recorded_at);
        self.store.append(&ev)?;
        Ok(ev.id)
    }

    /// The Contrast the log holds under `id`, or a refusal (exit 2) when it holds none.
    ///
    /// A Contrast travels inside an `Observed` event as `Observation::Return { contrast }`; the
    /// log is the only store, so the fold reads it back from there.
    fn contrast_in_log(&self, id: &ContrastId) -> Result<tsr_domain::Contrast, LedgerError> {
        for (_, ev) in self.store.all()? {
            if let EventKind::Observed {
                observation:
                    tsr_domain::Observation::Return {
                        contrast: Some(c), ..
                    },
            } = &ev.kind
            {
                if c.id == *id {
                    // R8: the content-address check must hold for a Contrast appended directly as
                    // an `Observed` event too, not only for one written through `record_contrast`.
                    // A record whose id is not the content address of its own fields is refused.
                    let expected =
                        crate::filter::contrast_id(&c.from_return, &c.unmet, &c.evidence);
                    if expected != c.id {
                        return Err(LedgerError::ContrastIdMismatch { id: c.id, expected });
                    }
                    return Ok(c.clone());
                }
            }
        }
        Err(LedgerError::UnknownContrast(*id))
    }

    /// Append a status. `withdrawn` and `out_of_scope` are operator-only: refused with exit 2 when
    /// `by` is not `Source::Operator` (PRD §B11 M2 item 1).
    pub fn set_status(
        &self,
        task: &TaskId,
        parents: Vec<EventId>,
        preference: PreferenceId,
        status: PreferenceStatus,
        by: Source,
        recorded_at: impl Into<String>,
    ) -> Result<EventId, LedgerError> {
        if is_operator_only(&status) && by != Source::Operator {
            return Err(LedgerError::OperatorOnly {
                preference,
                status: status_name(&status),
                by,
            });
        }
        let view = self.collective()?;
        let Some(entry) = view.get(&preference) else {
            return Err(LedgerError::UnknownPreference(preference));
        };
        // Admission by source (PRD §B2.1): a status other than `Open` applies only to a preference
        // already in the active view. `Open` is the admission act itself, so it is always recorded
        // (a non-operator `Open` is recorded and ignored for admission by the fold).
        if status != PreferenceStatus::Open && !entry.is_active() {
            return Err(LedgerError::NotActive(preference));
        }
        let kind = EventKind::StatusSet {
            preference,
            status,
            by,
        };
        let ev = Event::new(*task, parents, by, kind, recorded_at);
        self.store.append(&ev)?;
        Ok(ev.id)
    }

    /// The mechanical status for a merge: `Overlapping(id)` when a preference already in the
    /// structure carries the same normalized text in the same scope, `None` otherwise.
    pub fn overlapping_with(
        &self,
        text: &str,
        scope: &Scope,
    ) -> Result<Option<PreferenceId>, LedgerError> {
        let view = self.collective()?;
        Ok(duplicate_in_scope(&view, text, scope))
    }

    /// Record the mechanical `Overlapping(id)` status for a preference whose text duplicates an
    /// earlier one in the same scope. Refuses when there is no duplicate.
    pub fn mark_overlapping(
        &self,
        task: &TaskId,
        parents: Vec<EventId>,
        preference: PreferenceId,
        recorded_at: impl Into<String>,
    ) -> Result<Option<EventId>, LedgerError> {
        let view = self.collective()?;
        let Some(entry) = view.get(&preference) else {
            return Err(LedgerError::UnknownPreference(preference));
        };
        let Some(other) =
            duplicate_in_scope(&view, &entry.preference.text, &entry.preference.scope)
        else {
            return Ok(None);
        };
        if other == preference {
            return Ok(None);
        }
        let id = self.set_status(
            task,
            parents,
            preference,
            PreferenceStatus::Overlapping(other),
            Source::Rules,
            recorded_at,
        )?;
        Ok(Some(id))
    }

    /// Record the mechanical `Fulfilled` status: the preference's criterion closed because a later
    /// return met it. `by` is `Source::Rules` — a mechanical status, not an operator's.
    pub fn mark_fulfilled(
        &self,
        task: &TaskId,
        parents: Vec<EventId>,
        preference: PreferenceId,
        recorded_at: impl Into<String>,
    ) -> Result<EventId, LedgerError> {
        self.set_status(
            task,
            parents,
            preference,
            PreferenceStatus::Fulfilled,
            Source::Rules,
            recorded_at,
        )
    }
}

/// `withdrawn` and `out_of_scope` are operator-only (PRD §B11 M2 item 1).
pub fn is_operator_only(status: &PreferenceStatus) -> bool {
    matches!(
        status,
        PreferenceStatus::Withdrawn | PreferenceStatus::OutOfScope
    )
}

/// The engineering name of a status, for refusals and for the CLI.
pub fn status_name(status: &PreferenceStatus) -> String {
    match status {
        PreferenceStatus::Open => "open".to_string(),
        PreferenceStatus::Fulfilled => "fulfilled".to_string(),
        PreferenceStatus::Superseded(_) => "superseded".to_string(),
        PreferenceStatus::Withdrawn => "withdrawn".to_string(),
        PreferenceStatus::Overlapping(_) => "overlapping".to_string(),
        PreferenceStatus::OutOfScope => "out_of_scope".to_string(),
    }
}
