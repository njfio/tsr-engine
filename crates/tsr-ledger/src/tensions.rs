//! Operator-reviewed candidate/criterion impacts. No semantic inference or missing-support labels.
//! Only the trusted controller's explicit review entry point can create ReviewedTensions.
use crate::CollectiveReality;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use tsr_domain::{
    CriterionId, Event, EventKind, Hash, PreferenceId, Snapshot, Source, Tension, TensionWith,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImpactLink {
    pub preference: PreferenceId,
    pub criterion: CriterionId,
    pub against: TensionWith,
    /// Operator's concrete explanation of the conflicting impact (not a model label).
    pub note: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReviewInput {
    /// Operator explicitly declares these named criteria open for this review.
    pub open_criteria: Vec<CriterionId>,
    pub links: Vec<ImpactLink>,
}

/// Not deserializable: loading a generator's JSON cannot manufacture a reviewed value.
pub struct ReviewedTensions {
    snapshot: Snapshot,
    input: ReviewInput,
    selected: Vec<PreferenceId>,
    tensions: Vec<Tension>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TensionTrace {
    pub snapshot: Snapshot,
    pub selected: Vec<PreferenceId>,
    pub review: tsr_domain::EventId,
    pub tensions: Vec<Tension>,
    pub abstention: Option<String>,
}

/// Trusted operator boundary. Call only for an explicit operator review, never provider output.
/// The criteria declaration is an operator assertion, not an inferred fact about the program.
pub fn operator_review(
    ledger: &CollectiveReality,
    snapshot: &Snapshot,
    selected: &[PreferenceId],
    events: &[Event],
    input: ReviewInput,
) -> Result<ReviewedTensions, String> {
    if ledger.root != snapshot.root || input.open_criteria != snapshot.open_criteria {
        return Err("review must match the pinned ledger and declared criteria".into());
    }
    if input.links.len() > 256 || selected.len() > 256 || input.open_criteria.len() > 256 {
        return Err("review exceeds bounded link/criterion limits".into());
    }
    let selected_set: BTreeSet<_> = selected.iter().copied().collect();
    if selected_set.len() != selected.len()
        || selected.iter().any(|id| {
            !ledger
                .entries
                .iter()
                .any(|e| e.preference.id == *id && e.is_open())
        })
    {
        return Err("selection includes duplicate or non-admitted/open preferences".into());
    }
    let criteria: BTreeSet<_> = snapshot.open_criteria.iter().copied().collect();
    if criteria.len() != snapshot.open_criteria.len() {
        return Err("duplicate criterion".into());
    }
    let mut unique = BTreeSet::new();
    let mut tensions = Vec::new();
    for link in &input.links {
        if !selected_set.contains(&link.preference)
            || !criteria.contains(&link.criterion)
            || link.note.trim().is_empty()
            || link.note.len() > 512
        {
            return Err(
                "impact must name a selected preference, open criterion and bounded explanation"
                    .into(),
            );
        }
        let known = events.iter().filter(|e| e.task == snapshot.task).any(|e| {
            match (&link.against, &e.kind) {
                (TensionWith::Belief(id), EventKind::Proposed { candidate }) => candidate.id == *id,
                (TensionWith::Expectation(id), EventKind::Forecast { forecast, .. }) => {
                    forecast.id == *id
                }
                _ => false,
            }
        });
        if !known {
            return Err("impact target is not recorded in this task".into());
        }
        let key = serde_json::to_vec(&(link.preference, link.criterion, &link.against))
            .map_err(|e| e.to_string())?;
        if !unique.insert(key) {
            return Err("duplicate impact link".into());
        }
        tensions.push(Tension {
            preference: link.preference,
            against: link.against.clone(),
            note: link.note.clone(),
        });
    }
    Ok(ReviewedTensions {
        snapshot: snapshot.clone(),
        input,
        selected: selected.to_vec(),
        tensions,
    })
}

/// Persist outside the candidate copy, then append the operator review and derived trace together.
/// Neither evidence record changes policy authority, preference admission, rewards or outcome labels.
pub fn record(
    store: &tsr_store::Store,
    reviewed: ReviewedTensions,
    mut persist: impl FnMut(Hash, &[u8]) -> Result<(), String>,
    recorded_at: &str,
) -> Result<TensionTrace, String> {
    use tsr_domain::Observation;
    let snapshot = &reviewed.snapshot;
    let events = store.by_task(&snapshot.task).map_err(|e| e.to_string())?;
    if events.is_empty()
        || events.len() as u64 != snapshot.version
        || store.head_root().map_err(|e| e.to_string())? != snapshot.root
    {
        return Err("stale operator review".into());
    }
    let bytes = serde_json::to_vec(&(snapshot, &reviewed.input)).map_err(|e| e.to_string())?;
    let digest = Hash::of(&[&bytes]);
    persist(digest, &bytes)?;
    let review = Event::new(
        snapshot.task,
        vec![events.last().unwrap().1.id],
        Source::Operator,
        EventKind::Observed {
            observation: Observation::Evidence {
                digest,
                source: "m3/impact-review@v1".into(),
            },
        },
        recorded_at,
    );
    let trace = TensionTrace {
        snapshot: snapshot.clone(),
        selected: reviewed.selected,
        review: review.id,
        abstention: reviewed
            .tensions
            .is_empty()
            .then(|| "no operator-reviewed conflicting impacts".into()),
        tensions: reviewed.tensions,
    };
    let bytes = serde_json::to_vec(&trace).map_err(|e| e.to_string())?;
    let digest = Hash::of(&[&bytes]);
    persist(digest, &bytes)?;
    let event = Event::new(
        snapshot.task,
        vec![review.id],
        Source::Rules,
        EventKind::Observed {
            observation: Observation::Evidence {
                digest,
                source: "m3/tensions@v1".into(),
            },
        },
        recorded_at,
    );
    store
        .append_all(&[review, event])
        .map_err(|e| e.to_string())?;
    Ok(trace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tsr_domain::{ActionKind, Candidate, Preference, PreferenceStatus, Scope, TaskId};
    #[test]
    fn only_explicit_reviewed_links_are_recorded_and_missing_evidence_is_not_conflict() {
        let store = tsr_store::Store::open_in_memory().unwrap();
        let task = TaskId(Hash::of(&[b"review"]));
        let event = Event::new(
            task,
            vec![],
            Source::Operator,
            EventKind::TaskOpened { task },
            "fixture",
        );
        store.append(&event).unwrap();
        let candidate = Candidate::new(
            ActionKind::ProposePatch,
            Source::Generator,
            Hash::of(&[b"patch"]),
            vec![],
            vec![],
        )
        .unwrap();
        let proposed = Event::new(
            task,
            vec![event.id],
            Source::Generator,
            EventKind::Proposed {
                candidate: candidate.clone(),
            },
            "fixture",
        );
        store.append(&proposed).unwrap();
        let preference = Preference::new(
            "retain compatibility",
            Scope::Task(task),
            vec![],
            Source::Operator,
        )
        .unwrap();
        let root = store.head_root().unwrap();
        let ledger = CollectiveReality {
            root,
            entries: vec![crate::LedgerEntry {
                preference: preference.clone(),
                expanded_by: None,
                from_contrast: None,
                from_success: None,
                from_provisional: None,
                catalyzed_by: vec![],
                status: Some(PreferenceStatus::Open),
                history: vec![],
            }],
        };
        let criterion = CriterionId(Hash::of(&[b"compatibility"]));
        let snap = Snapshot {
            root,
            task,
            version: 2,
            open_criteria: vec![criterion],
            missing_evidence: vec!["untested".into()],
        };
        let events = vec![event, proposed];
        let empty = operator_review(
            &ledger,
            &snap,
            &[preference.id],
            &events,
            ReviewInput {
                open_criteria: vec![criterion],
                links: vec![],
            },
        )
        .unwrap();
        assert!(empty.tensions.is_empty());
        let link = ImpactLink {
            preference: preference.id,
            criterion,
            against: TensionWith::Belief(candidate.id),
            note: "removes a required public entry point".into(),
        };
        let reviewed = operator_review(
            &ledger,
            &snap,
            &[preference.id],
            &events,
            ReviewInput {
                open_criteria: vec![criterion],
                links: vec![link.clone()],
            },
        )
        .unwrap();
        let mut objects = std::collections::BTreeMap::new();
        let trace = record(
            &store,
            reviewed,
            |h, b| {
                objects.insert(h, b.to_vec());
                Ok(())
            },
            "fixture",
        )
        .unwrap();
        assert_eq!(trace.tensions.len(), 1);
        let saved = store.by_task(&task).unwrap();
        assert_eq!(saved[2].1.source, Source::Operator);
        assert_eq!(saved[3].1.parents, vec![saved[2].1.id]);
        assert_eq!(objects.len(), 2);
        assert!(record(&store, empty, |_, _| Ok(()), "fixture").is_err());
        let mut wrong = link;
        wrong.against = TensionWith::Belief(tsr_domain::CandidateId(Hash::of(&[b"unknown"])));
        assert!(operator_review(
            &ledger,
            &snap,
            &[preference.id],
            &events,
            ReviewInput {
                open_criteria: vec![criterion],
                links: vec![wrong]
            }
        )
        .is_err());
        assert!(serde_json::from_str::<ReviewInput>(
            r#"{"open_criteria":[],"links":[],"reviewed":true}"#
        )
        .is_err());
    }
}
