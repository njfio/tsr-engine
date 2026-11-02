//! Optional generated wanting from authenticated current contributors.
//! No success, Contrast, human fulfillment or execution authority is inferred.
use crate::{Ledger, LedgerError};
use tsr_domain::*;
pub const REVIEW_SOURCE: &str = "ledger-provisional-preference-review@v1";

pub fn canonical(x: &ProvisionalExpansion) -> Result<Preference, LedgerError> {
    let p = Preference::from_admitted_text(
        x.preference.text.clone(),
        x.preference.scope.clone(),
        x.preference.parents.clone(),
        Source::Generator,
    )?;
    if p != x.preference || p.parents.is_empty() || x.rationale.trim().is_empty() {
        return Err(LedgerError::InvalidReview(
            "provisional identity/ancestry/rationale",
        ));
    }
    Ok(p)
}
pub fn basis(
    events: &[(i64, Event)],
    task: TaskId,
    x: &ProvisionalExpansion,
) -> Result<(), LedgerError> {
    canonical(x)?;
    if x.preference.scope != Scope::Task(task) {
        return Err(LedgerError::InvalidReview("provisional task scope"));
    }
    if !events
        .iter()
        .any(|(_, e)| e.id == x.proposal && e.task == task && e.source == Source::Generator)
        || !events
            .iter()
            .any(|(_, e)| e.id == x.requirements_revision && e.task == task)
    {
        return Err(LedgerError::InvalidReview(
            "provisional source context missing",
        ));
    }
    Ok(())
}
pub fn validate(
    events: &[(i64, Event)],
    task: TaskId,
    r: &ProvisionalPreferenceReview,
    evidence: EventId,
) -> Result<Preference, LedgerError> {
    let bad = LedgerError::InvalidReview;
    if r.decision != "approve"
        || r.reviewer.trim().is_empty()
        || r.generator.trim().is_empty()
        || r.reviewer.trim() == r.generator.trim()
    {
        return Err(bad("independent provisional review required"));
    }
    let find = |id| {
        events
            .iter()
            .position(|(_, e)| e.id == id && e.task == task)
            .ok_or(bad("missing provisional review reference"))
    };
    let xp = find(r.expansion)?;
    let ep = find(evidence)?;
    if find(r.proposal)? >= xp || xp >= ep {
        return Err(bad("provisional review causal order"));
    }
    let event = &events[xp].1;
    let EventKind::ProvisionalExpanded { expansion: x } = &event.kind else {
        return Err(bad("wrong provisional expansion kind"));
    };
    basis(&events[..xp], task, x)?;
    if event.source != Source::Generator
        || x.preference.id != r.preference
        || x.proposal != r.proposal
        || x.source_snapshot != r.source_snapshot
        || x.requirements_revision != r.requirements_revision
        || x.context_binding != r.context_binding
        || x.contributors_digest != r.contributors_digest
    {
        return Err(bad("provisional review binding"));
    }
    let e = &events[ep].1;
    let digest = Hash::of(&[&serde_json::to_vec(r).expect("serializable review")]);
    if e.source != Source::Council
        || !matches!(&e.kind, EventKind::Observed { observation:Observation::Evidence { source,digest:d } } if source == REVIEW_SOURCE && *d == digest)
    {
        return Err(bad("provisional review evidence"));
    }
    canonical(x)
}
pub fn eligible_parent(entry: &crate::LedgerEntry, task: TaskId) -> bool {
    entry.preference.scope == Scope::Task(task)
        && matches!(
            entry.status,
            Some(PreferenceStatus::Open | PreferenceStatus::Fulfilled)
        )
}
impl Ledger<'_> {
    pub fn propose_provisional(
        &self,
        task: &TaskId,
        parents: Vec<EventId>,
        expansion: ProvisionalExpansion,
        at: impl Into<String>,
    ) -> Result<EventId, LedgerError> {
        basis(&self.store().all()?, *task, &expansion)?;
        let view = self.collective()?;
        if expansion
            .preference
            .parents
            .iter()
            .any(|id| !view.get(id).is_some_and(|p| eligible_parent(p, *task)))
        {
            return Err(LedgerError::InvalidReview(
                "inactive or foreign provisional contributor",
            ));
        }
        let event = Event::new(
            *task,
            parents,
            Source::Generator,
            EventKind::ProvisionalExpanded { expansion },
            at,
        );
        self.store().append(&event)?;
        Ok(event.id)
    }
    pub fn admit_provisional(
        &self,
        task: &TaskId,
        head: Hash,
        review: ProvisionalPreferenceReview,
        evidence: EventId,
        at: impl Into<String>,
    ) -> Result<EventId, LedgerError> {
        if self.store().head_root()? != head {
            return Err(LedgerError::InvalidReview("stale provisional admission"));
        }
        let p = validate(&self.store().all()?, *task, &review, evidence)?;
        let view = self.collective()?;
        if p.parents
            .iter()
            .any(|id| !view.get(id).is_some_and(|p| eligible_parent(p, *task)))
            || view.get(&p.id).is_none_or(|e| {
                e.status.is_some() || e.preference != p || e.expanded_by != Some(review.expansion)
            })
        {
            return Err(LedgerError::InvalidReview(
                "duplicate or unknown provisional lineage",
            ));
        }
        let event = Event::new(
            *task,
            vec![evidence],
            Source::Council,
            EventKind::ReviewedProvisionalPreferenceAdmission {
                review: Box::new(review),
                evidence,
            },
            at,
        );
        self.store().append(&event)?;
        Ok(event.id)
    }
}
