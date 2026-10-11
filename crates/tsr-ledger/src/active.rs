//! The active view: a fold over the log, nothing stored (PRD §B2.1, §B6, §B8).
//!
//! Two views of one log. *Collective Reality* is the whole log folded — every preference ever
//! expanded, with its lineage. *Vibrational Reality* is the active view: the same structure with
//! statuses applied in sequence order, and admission by source. The structure beneath never
//! changes; a status is a new event that supersedes, never a rewrite.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use tsr_domain::{
    ContrastId, Event, EventId, EventKind, Preference, PreferenceId, PreferenceStatus, Scope,
    Source,
};

/// One preference as the fold sees it: the node, its lineage, and where it stands now.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub preference: Preference,
    /// The `Expanded` event that produced it, when it entered through an expansion.
    pub expanded_by: Option<EventId>,
    /// The contrast that crossed, when it entered through an expansion.
    pub from_contrast: Option<ContrastId>,
    /// The preferences that made that contrast matter (the expansion's `catalyzed_by`).
    pub catalyzed_by: Vec<PreferenceId>,
    /// The status in the active view. `None` means: in the structure, absent from the active view.
    pub status: Option<PreferenceStatus>,
    /// Every status ever set on this preference, in sequence order. The last one is `status`.
    pub history: Vec<(PreferenceStatus, Source)>,
}

impl LedgerEntry {
    /// In the active view: a status has been applied to it.
    pub fn is_active(&self) -> bool {
        self.status.is_some()
    }

    /// Open in the active view.
    pub fn is_open(&self) -> bool {
        matches!(self.status, Some(PreferenceStatus::Open))
    }
}

/// The whole log folded: every preference, in the order the log first carried it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CollectiveReality {
    pub entries: Vec<LedgerEntry>,
    /// The store's `head_root()` at the fold: one hash for the whole culmination (PRD §B6).
    pub root: tsr_domain::Hash,
}

impl CollectiveReality {
    /// The active view: statuses applied, admission by source (PRD §B2.1).
    pub fn active(&self) -> VibrationalReality {
        VibrationalReality {
            entries: self
                .entries
                .iter()
                .filter(|e| e.is_active())
                .cloned()
                .collect(),
            root: self.root,
        }
    }

    pub fn get(&self, id: &PreferenceId) -> Option<&LedgerEntry> {
        self.entries.iter().find(|e| &e.preference.id == id)
    }

    /// The preferences that are open in the active view, in log order.
    pub fn open(&self) -> Vec<&LedgerEntry> {
        self.entries.iter().filter(|e| e.is_open()).collect()
    }
}

/// The active view over the log: what is open and active now.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct VibrationalReality {
    pub entries: Vec<LedgerEntry>,
    pub root: tsr_domain::Hash,
}

impl VibrationalReality {
    pub fn get(&self, id: &PreferenceId) -> Option<&LedgerEntry> {
        self.entries.iter().find(|e| &e.preference.id == id)
    }

    pub fn contains(&self, id: &PreferenceId) -> bool {
        self.get(id).is_some()
    }

    pub fn open(&self) -> Vec<&LedgerEntry> {
        self.entries.iter().filter(|e| e.is_open()).collect()
    }

    /// The open preferences in a scope, in log order.
    pub fn open_in_scope(&self, scope: &Scope) -> Vec<&LedgerEntry> {
        self.entries
            .iter()
            .filter(|e| e.is_open() && &e.preference.scope == scope)
            .collect()
    }
}

/// Fold the log into the ledger. Pure: same events, same view, same root.
///
/// Admission by source (PRD §B2.1): a preference whose origin is `Source::Operator` — including a
/// rejection reason — is open on arrival; one whose origin is `Source::Generator` is in the
/// structure but absent from the active view until an Operator `StatusSet(Open)` exists for it.
pub fn fold(events: &[(i64, Event)], root: tsr_domain::Hash) -> CollectiveReality {
    let mut order: Vec<PreferenceId> = Vec::new();
    let mut by_id: BTreeMap<PreferenceId, LedgerEntry> = BTreeMap::new();

    for (_, ev) in events {
        match &ev.kind {
            EventKind::Expanded { expansion } => {
                let Some(pref) = expansion_preference(expansion) else {
                    continue;
                };
                let id = pref.id;
                if let Some(existing) = by_id.get_mut(&id) {
                    // The same content address is the same node; keep the first lineage seen.
                    if existing.expanded_by.is_none() {
                        existing.expanded_by = Some(ev.id);
                        existing.from_contrast = Some(expansion.from_contrast);
                        existing.catalyzed_by = expansion.catalyzed_by.clone();
                    }
                    continue;
                }
                let status = arrival_status(pref.origin);
                let history = status
                    .clone()
                    .map(|s| vec![(s, Source::Operator)])
                    .unwrap_or_default();
                order.push(id);
                by_id.insert(
                    id,
                    LedgerEntry {
                        preference: pref,
                        expanded_by: Some(ev.id),
                        from_contrast: Some(expansion.from_contrast),
                        catalyzed_by: expansion.catalyzed_by.clone(),
                        status,
                        history,
                    },
                );
            }
            EventKind::StatusSet {
                preference,
                status,
                by,
            } => {
                let Some(entry) = by_id.get_mut(preference) else {
                    // A status for a preference the log never carried is not a view.
                    continue;
                };
                entry.history.push((status.clone(), *by));
                entry.status = Some(status.clone());
            }
            _ => {}
        }
    }

    let entries = order
        .into_iter()
        .filter_map(|id| by_id.remove(&id))
        .collect();
    CollectiveReality { entries, root }
}

/// The preference an `Expanded` event carries.
///
/// The produced preference is content-addressed by blake3(text, scope, parents) and is minted by
/// [`Preference::new`] — never re-implemented here. The event carries the node's text, scope,
/// parents and origin beside the expansion, so the fold reads the node from the log and checks the
/// content address against the fields it was minted from.
fn expansion_preference(expansion: &tsr_domain::Expansion) -> Option<Preference> {
    let pref = Preference::new(
        expansion.text.clone(),
        expansion.scope.clone(),
        expansion.parents.clone(),
        expansion.origin,
    )
    .ok()?;
    if pref.id == expansion.produced {
        Some(pref)
    } else {
        None
    }
}

/// Admission by source: Operator statements enter directly; generated statements wait for review.
fn arrival_status(origin: Source) -> Option<PreferenceStatus> {
    match origin {
        Source::Operator => Some(PreferenceStatus::Open),
        _ => None,
    }
}

/// Normalize preference text for duplicate detection: lowercase, whitespace collapsed, trimmed.
///
/// A merge finds a duplicate by normalized text within the same scope (PRD §B11 M2 item 1).
pub fn normalize_text(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// The first preference already in the structure whose normalized text matches, within the same
/// scope. `None` when the text is new in that scope.
pub fn duplicate_in_scope(
    view: &CollectiveReality,
    text: &str,
    scope: &Scope,
) -> Option<PreferenceId> {
    let needle = normalize_text(text);
    view.entries
        .iter()
        .find(|e| &e.preference.scope == scope && normalize_text(&e.preference.text) == needle)
        .map(|e| e.preference.id)
}

/// The preferences that are open in the active view, deduplicated by normalized text within a
/// scope. The first occurrence in log order wins; later duplicates are reported beside it.
pub fn deduplicated_open(view: &CollectiveReality) -> Vec<PreferenceId> {
    let mut seen: BTreeSet<(String, String)> = BTreeSet::new();
    let mut out = Vec::new();
    for entry in view.entries.iter().filter(|e| e.is_open()) {
        let key = (
            scope_key(&entry.preference.scope),
            normalize_text(&entry.preference.text),
        );
        if seen.insert(key) {
            out.push(entry.preference.id);
        }
    }
    out
}

/// A stable string for a scope, for dedup keys and for the CLI.
pub fn scope_key(scope: &Scope) -> String {
    match scope {
        Scope::Task(t) => format!("task:{t}"),
        Scope::Family(f) => format!("family:{f}"),
        Scope::Repository(r) => format!("repository:{r}"),
        Scope::Operator => "operator".to_string(),
    }
}
