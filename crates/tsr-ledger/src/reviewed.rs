//! Shared writer/replay validation for independently reviewed generated preferences.
use crate::LedgerError;
use tsr_domain::{
    Event, EventId, EventKind, Hash, Observation, Preference, PreferenceAdmissionReview, Source,
    TaskId,
};

pub fn return_source(source: Source) -> bool {
    matches!(source, Source::Rules | Source::Operator | Source::Council)
}

pub const REVIEW_SOURCE: &str = "ledger-preference-review@v1";

/// Only an already authenticated controller may write Council evidence. This validator
/// proves exact association, not the truth of a review or a model's claimed identity.
/// The controller remains responsible for authenticated independent transport and scope.
pub fn validate(
    prefix: &[(i64, Event)],
    task: TaskId,
    review: &PreferenceAdmissionReview,
    evidence: EventId,
) -> Result<Preference, LedgerError> {
    let bad = LedgerError::InvalidReview;
    if review.decision != "approve"
        || review.reviewer.trim().is_empty()
        || review.generator.trim().is_empty()
        || review.reviewer.trim() == review.generator.trim()
    {
        return Err(bad("independent approval required"));
    }
    let find = |id: EventId| {
        prefix
            .iter()
            .find(|(_, e)| e.id == id && e.task == task)
            .map(|(_, e)| e)
            .ok_or(bad("missing, future or cross-task reference"))
    };
    let expansion_event = find(review.expansion)?;
    let EventKind::Expanded { expansion: x } = &expansion_event.kind else {
        return Err(bad("wrong expansion kind"));
    };
    if expansion_event.source != Source::Generator
        || x.origin != Source::Generator
        || x.produced != review.preference
        || x.from_contrast != review.contrast
    {
        return Err(bad("expansion identity or provenance mismatch"));
    }
    let pref = Preference::from_admitted_text(
        x.text.clone(),
        x.scope.clone(),
        x.parents.clone(),
        x.origin,
    )?;
    if pref.id != x.produced {
        return Err(bad("noncanonical expansion"));
    }
    let return_event = find(review.from_return)?;
    if !return_source(return_event.source) {
        return Err(bad("untrusted return"));
    }
    let contrast = prefix
        .iter()
        .find_map(|(_, e)| match &e.kind {
            EventKind::Observed {
                observation:
                    Observation::Return {
                        contrast: Some(c), ..
                    },
            } if e.task == task && return_source(e.source) && c.id == review.contrast => Some(c),
            _ => None,
        })
        .ok_or(bad("missing authentic Contrast"))?;
    if contrast.from_return != review.from_return
        || contrast.unmet.is_empty()
        || contrast.id
            != crate::filter::contrast_id(
                &contrast.from_return,
                &contrast.unmet,
                &contrast.evidence,
            )
    {
        return Err(bad("Contrast identity or unmet conditions mismatch"));
    }
    for id in &contrast.evidence {
        find(*id)?;
    }
    find(review.proposal)?;
    find(review.requirements_revision)?;
    let position = |id: EventId| {
        prefix
            .iter()
            .position(|(_, e)| e.id == id)
            .expect("validated reference")
    };
    find(evidence)?;
    let evidence_position = position(evidence);
    if [
        review.expansion,
        review.proposal,
        review.requirements_revision,
        review.from_return,
    ]
    .iter()
    .any(|id| position(*id) >= evidence_position)
    {
        return Err(bad("review evidence precedes its subject"));
    }
    let contrast_position = prefix
        .iter()
        .position(|(_, e)| {
            matches!(&e.kind,
        EventKind::Observed{observation:Observation::Return{contrast:Some(c),..}}
        if e.task==task && return_source(e.source) && c==contrast)
        })
        .expect("validated Contrast");
    if contrast_position >= position(review.expansion)
        || position(review.from_return) >= contrast_position
        || contrast
            .evidence
            .iter()
            .any(|id| position(*id) >= contrast_position)
    {
        return Err(bad("return/evidence must precede Contrast, then expansion"));
    }
    let e = find(evidence)?;
    let digest = Hash::of(&[&serde_json::to_vec(review).expect("review serializable")]);
    if e.source != Source::Council
        || !matches!(&e.kind,
        EventKind::Observed{observation:Observation::Evidence{source,digest:d}}
        if source==REVIEW_SOURCE && *d==digest)
    {
        return Err(bad("review evidence digest or provenance mismatch"));
    }
    Ok(pref)
}
