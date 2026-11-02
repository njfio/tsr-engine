//! Optional success-derived wanting. Additive provenance; never fabricated Contrast.
use crate::{Ledger, LedgerError};
use tsr_domain::*;
pub const REVIEW_SOURCE: &str = "ledger-success-preference-review@v1";
pub fn basis(
    events: &[(i64, Event)],
    task: TaskId,
    id: EventId,
) -> Result<ScopedSuccess, LedgerError> {
    let bad = LedgerError::InvalidReview;
    let (position, (_, event)) = events
        .iter()
        .enumerate()
        .find(|(_, (_, e))| e.id == id && e.task == task)
        .ok_or(bad("missing success"))?;
    let EventKind::ScopedSuccess { success } = &event.kind else {
        return Err(bad("wrong success kind"));
    };
    if event.source != Source::Council
        || success.criteria.is_empty()
        || success.criteria.iter().any(|k| k.trim().is_empty())
        || success
            .criteria
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != success.criteria.len()
    {
        return Err(bad("success scope/provenance"));
    }
    // Council is the trusted controller attestation boundary. The project
    // verifies the exact all-met assessment; this generic fold verifies its
    // causal provenance and never infers success from an arbitrary receipt.
    let assessment = events[..position]
        .iter()
        .position(|(_, e)| {
            e.task == task && e.id == success.assessment && e.source == Source::Council
        })
        .ok_or(bad("success assessment provenance"))?;
    for id in [
        success.receipt,
        success.publication,
        success.requirements_revision,
    ] {
        if !events[..assessment]
            .iter()
            .any(|(_, e)| e.task == task && e.id == id)
        {
            return Err(bad("success subject must precede assessment"));
        }
    }
    Ok(success.clone())
}
pub fn canonical(x: &SuccessExpansion) -> Result<Preference, LedgerError> {
    let p = Preference::from_admitted_text(
        x.preference.text.clone(),
        x.preference.scope.clone(),
        x.preference.parents.clone(),
        Source::Generator,
    )?;
    if p != x.preference || p.parents.len() < 2 {
        return Err(LedgerError::InvalidReview(
            "success expansion identity/lineage",
        ));
    }
    Ok(p)
}
pub fn validate(
    events: &[(i64, Event)],
    task: TaskId,
    r: &SuccessPreferenceReview,
    evidence: EventId,
) -> Result<Preference, LedgerError> {
    let bad = LedgerError::InvalidReview;
    if r.decision != "approve"
        || r.reviewer.trim().is_empty()
        || r.generator.trim().is_empty()
        || r.reviewer.trim() == r.generator.trim()
    {
        return Err(bad("independent success review required"));
    }
    let find = |id| {
        events
            .iter()
            .position(|(_, e)| e.id == id && e.task == task)
            .ok_or(bad("missing success review reference"))
    };
    let x_pos = find(r.expansion)?;
    let e_pos = find(evidence)?;
    let s_pos = find(r.success)?;
    if s_pos >= x_pos || x_pos >= e_pos || find(r.proposal)? >= e_pos {
        return Err(bad("success review causal order"));
    }
    let x_event = &events[x_pos].1;
    let EventKind::SuccessExpanded { expansion: x } = &x_event.kind else {
        return Err(bad("wrong success expansion kind"));
    };
    let b = basis(&events[..x_pos], task, r.success)?;
    if x_event.source != Source::Generator
        || x.success != r.success
        || x.preference.id != r.preference
        || b.source_snapshot != r.source_snapshot
        || b.requirements_revision != r.requirements_revision
    {
        return Err(bad("success review binding"));
    }
    let p = canonical(x)?;
    let e = &events[e_pos].1;
    let digest = Hash::of(&[&serde_json::to_vec(r).expect("serializable review")]);
    if e.source != Source::Council
        || !matches!(&e.kind,EventKind::Observed{observation:Observation::Evidence{source,digest:d}} if source==REVIEW_SOURCE&&*d==digest)
    {
        return Err(bad("success review evidence"));
    }
    Ok(p)
}
impl Ledger<'_> {
    pub fn propose_success(
        &self,
        task: &TaskId,
        parents: Vec<EventId>,
        expansion: SuccessExpansion,
        at: impl Into<String>,
    ) -> Result<EventId, LedgerError> {
        basis(&self.store().all()?, *task, expansion.success)?;
        canonical(&expansion)?;
        let e = Event::new(
            *task,
            parents,
            Source::Generator,
            EventKind::SuccessExpanded { expansion },
            at,
        );
        self.store().append(&e)?;
        Ok(e.id)
    }
    pub fn admit_success(
        &self,
        task: &TaskId,
        head: Hash,
        review: SuccessPreferenceReview,
        evidence: EventId,
        at: impl Into<String>,
    ) -> Result<EventId, LedgerError> {
        if self.store().head_root()? != head {
            return Err(LedgerError::InvalidReview("stale success admission"));
        }
        let p = validate(&self.store().all()?, *task, &review, evidence)?;
        let view = self.collective()?;
        if p.parents.iter().any(|id| view.get(id).is_none())
            || view.get(&p.id).is_some_and(|e| {
                e.status.is_some() || e.preference != p || e.expanded_by != Some(review.expansion)
            })
        {
            return Err(LedgerError::InvalidReview(
                "duplicate or unknown success lineage",
            ));
        }
        let e = Event::new(
            *task,
            vec![evidence],
            Source::Council,
            EventKind::ReviewedSuccessPreferenceAdmission {
                review: Box::new(review),
                evidence,
            },
            at,
        );
        self.store().append(&e)?;
        Ok(e.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (TaskId, Vec<(i64, Event)>, EventId) {
        let task = TaskId(Hash::of(&[b"authored task"]));
        let mut events = vec![];
        for n in 0..4 {
            let e = Event::new(
                task,
                vec![],
                if n == 3 {
                    Source::Council
                } else {
                    Source::Rules
                },
                EventKind::Observed {
                    observation: Observation::Evidence {
                        source: format!("authored {n}"),
                        digest: Hash::of(&[&[n]]),
                    },
                },
                "authored",
            );
            events.push((n as i64, e));
        }
        let b = ScopedSuccess {
            receipt: events[0].1.id,
            publication: events[1].1.id,
            requirements_revision: events[2].1.id,
            assessment: events[3].1.id,
            source_snapshot: Hash::of(&[b"source"]),
            scope_digest: Hash::of(&[b"scope"]),
            criteria: vec!["orientation".into(), "discovery".into()],
        };
        let e = Event::new(
            task,
            vec![],
            Source::Council,
            EventKind::ScopedSuccess { success: b },
            "authored",
        );
        let id = e.id;
        events.push((4, e));
        (task, events, id)
    }
    #[test]
    fn success_subjects_must_precede_council_assessment() {
        let (task, events, id) = fixture();
        assert!(basis(&events, task, id).is_ok());
        for subject in 0..3 {
            let mut bad = events.clone();
            bad.swap(subject, 3);
            assert!(basis(&bad, task, id).is_err());
        }
        let mut bad = events.clone();
        bad[3].1.source = Source::Generator;
        assert!(basis(&bad, task, id).is_err());
        let mut bad = events.clone();
        bad[0].1.task = TaskId(Hash::of(&[b"foreign"]));
        assert!(basis(&bad, task, id).is_err());
        let mut bad = events.clone();
        bad[4].1.source = Source::Generator;
        assert!(basis(&bad, task, id).is_err());
        let mut bad = events;
        let EventKind::ScopedSuccess { success } = &mut bad[4].1.kind else {
            unreachable!()
        };
        success.criteria.push(success.criteria[0].clone());
        assert!(basis(&bad, task, id).is_err());
    }
}
