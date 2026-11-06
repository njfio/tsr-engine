//! Adapter-neutral public meaning. Initial context is never an empirical return.
//! Uses the native event/CAS store, Preference identities and proposal vocabulary.
use super::*;
use adaptive::coding::project::desire::Basis as DesireBasis;
use adaptive::provider::{Response, Suggestion, PROTOCOL};
use anyhow::Context;
use serde_json::{json, Value};
use std::collections::{BTreeMap, BTreeSet};
pub mod contract;
pub mod information;
pub mod inquiry;
pub mod retrieval;
const SOURCE: &str = "qualitative-context@v1";
pub(super) const MAX_TEXT_BYTES: usize = 512;
fn bounded(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= MAX_TEXT_BYTES
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parent {
    pub label: String,
    pub meaning: String,
    pub attribution: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VisibleRecord {
    pub label: String,
    pub content: String,
    pub attribution: String,
    pub uncertainty: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bootstrap {
    pub parents: Vec<Parent>,
    pub records: Vec<VisibleRecord>,
    pub constraints: Vec<String>,
    /// References only; this module never evaluates or changes acceptance.
    pub acceptance_references: Vec<Hash>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Basis {
    InitialContext { context: EventId },
    RetrievalObservation { context: EventId, receipt: EventId },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Submission {
    pub basis: Basis,
    pub generator: String,
    pub response: Response,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Admit,
    Reject,
    Unknown,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub state_root: Hash,
    pub proposal: EventId,
    pub reviewer: String,
    pub decision: Decision,
    pub reason: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Status {
    pub state_root: Hash,
    pub identity: PreferenceId,
    pub revision: Option<EventId>,
    pub withdrawn: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "record", rename_all = "snake_case", deny_unknown_fields)]
// Entries are parsed from the log and dropped soon after. Boxing the large variant would only add
// indirection to the provenance check that matches on an inquiry's origin.
#[allow(clippy::large_enum_variant)]
enum Entry {
    Bootstrap {
        input: Bootstrap,
        parents: Vec<Preference>,
    },
    Proposed {
        submission: Submission,
    },
    ProposedPossibility {
        submission: Submission,
    },
    InquiryProposed {
        inquiry: inquiry::Inquiry,
        parent_context: Hash,
        submission: Submission,
    },
    Reviewed {
        review: Review,
    },
    ExplorationPolicy {
        policy: inquiry::ExplorationPolicy,
    },
    RetrievalConfigured {
        manifest: retrieval::Manifest,
    },
    Retrieved {
        receipt: retrieval::Receipt,
    },
    InformationAttempt {
        attempt: information::Attempt,
    },
    InformationResolved {
        attempt: EventId,
        receipt: EventId,
    },
    Status {
        update: Status,
    },
}
#[derive(Clone)]
struct Current {
    original: Preference,
    meaning: String,
    revision: Option<EventId>,
    parents: Vec<PreferenceId>,
}
struct State {
    context: EventId,
    input: Bootstrap,
    current: BTreeMap<PreferenceId, Current>,
    proposals: BTreeMap<EventId, Submission>,
    reviewed: BTreeSet<EventId>,
    withdrawn: BTreeSet<PreferenceId>,
    guidance: Vec<(EventId, deliberation::relational::Proposal, Hash)>,
    possibilities: Vec<(EventId, EventId, Submission, Hash, String)>,
    history: Vec<Value>,
}
impl State {
    fn parent_context(&self, ids: &[PreferenceId]) -> Hash {
        let unique: BTreeSet<_> = ids.iter().copied().collect();
        let parts: Vec<_> = unique
            .iter()
            .map(|id| {
                let p = &self.current[id];
                (
                    id,
                    &p.meaning,
                    p.revision,
                    p.parents.iter().copied().collect::<BTreeSet<_>>(),
                )
            })
            .collect();
        Hash::of(&[&serde_json::to_vec(&parts).expect("serializable parent view")])
    }
    fn active(&self) -> BTreeSet<PreferenceId> {
        let mut withdrawn = self.withdrawn.clone();
        loop {
            let before = withdrawn.len();
            for (id, p) in &self.current {
                if p.parents.iter().any(|x| x != id && withdrawn.contains(x)) {
                    withdrawn.insert(*id);
                }
            }
            if before == withdrawn.len() {
                break;
            }
        }
        self.current
            .keys()
            .filter(|id| !withdrawn.contains(id))
            .copied()
            .collect()
    }
}
impl Engine {
    fn qualitative_append(&self, entry: Entry, source: Source) -> Result<EventId> {
        let digest = self.blob(&serde_json::to_vec(&entry)?)?;
        self.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    digest,
                    source: SOURCE.into(),
                },
            },
            source,
        )
    }
    fn qualitative_entries(&self) -> Result<Vec<(EventId, Entry)>> {
        let mut out = vec![];
        let mut previous = None;
        for (_, event) in self.events()? {
            ensure!(
                event.parents == previous.into_iter().collect::<Vec<_>>(),
                "broken native chain"
            );
            previous = Some(event.id);
            ensure!(
                Event::new(
                    event.task,
                    event.parents.clone(),
                    event.source,
                    event.kind.clone(),
                    event.recorded_at.clone()
                )
                .id == event.id,
                "event identity"
            );
            if let EventKind::Observed {
                observation: Observation::Evidence { digest, source },
            } = &event.kind
            {
                if source == SOURCE {
                    let bytes = fs::read(self.directory.join("cas").join(digest.to_hex()))?;
                    ensure!(Hash::of(&[&bytes]) == *digest, "context CAS changed");
                    let entry: Entry = serde_json::from_slice(&bytes)?;
                    let expected = if matches!(
                        entry,
                        Entry::Proposed { .. }
                            | Entry::ProposedPossibility { .. }
                            | Entry::InquiryProposed {
                                inquiry: inquiry::Inquiry {
                                    origin: inquiry::Origin::ModelProposed
                                        | inquiry::Origin::EngineExploration,
                                    ..
                                },
                                ..
                            }
                    ) {
                        Source::Generator
                    } else {
                        Source::Operator
                    };
                    let expected = if matches!(
                        entry,
                        Entry::Retrieved { .. } | Entry::InformationResolved { .. }
                    ) {
                        Source::Rules
                    } else {
                        expected
                    };
                    ensure!(event.source == expected, "qualitative source provenance");
                    out.push((event.id, entry));
                }
            }
        }
        Ok(out)
    }
    fn qualitative_state(&self) -> Result<State> {
        let entries = self.qualitative_entries()?;
        let (context, first) = entries.first().context("qualitative bootstrap required")?;
        let Entry::Bootstrap { input, parents } = first else {
            bail!("bootstrap first")
        };
        let mut s = State {
            context: *context,
            input: input.clone(),
            current: parents
                .iter()
                .map(|p| {
                    (
                        p.id,
                        Current {
                            original: p.clone(),
                            meaning: p.text.clone(),
                            revision: None,
                            parents: vec![],
                        },
                    )
                })
                .collect(),
            proposals: BTreeMap::new(),
            reviewed: BTreeSet::new(),
            withdrawn: BTreeSet::new(),
            guidance: vec![],
            possibilities: vec![],
            history: vec![],
        };
        for (id, e) in entries.into_iter().skip(1) {
            match e {
                Entry::Bootstrap { .. } => bail!("duplicate bootstrap"),
                Entry::RetrievalConfigured { .. }
                | Entry::Retrieved { .. }
                | Entry::ExplorationPolicy { .. }
                | Entry::InformationAttempt { .. }
                | Entry::InformationResolved { .. } => {}
                Entry::Proposed { submission }
                | Entry::ProposedPossibility { submission }
                | Entry::InquiryProposed { submission, .. } => {
                    s.proposals.insert(id, submission);
                }
                Entry::Reviewed { review } => {
                    let sub = s
                        .proposals
                        .get(&review.proposal)
                        .context("missing generated proposal")?;
                    ensure!(s.reviewed.insert(review.proposal), "duplicate review");
                    if review.decision == Decision::Admit {
                        match &sub.response.proposals[0] {
                            Suggestion::PublicDesire { proposal: p } => {
                                let old =
                                    s.current.get_mut(&p.identity).context("missing parent")?;
                                old.meaning = p.meaning.clone();
                                old.revision = Some(id);
                                old.parents = p.parents.clone();
                            }
                            Suggestion::Relational { proposal, .. } => s.guidance.push((
                                id,
                                proposal.clone(),
                                s.parent_context(&proposal.parents),
                            )),
                            Suggestion::NewCandidate { serves, .. } => {
                                let parents: Vec<PreferenceId> = serves
                                    .iter()
                                    .map(|v| serde_json::from_value(json!(v)))
                                    .collect::<std::result::Result<_, _>>()?;
                                s.possibilities.push((
                                    id,
                                    review.proposal,
                                    sub.clone(),
                                    s.parent_context(&parents),
                                    review.reviewer.clone(),
                                ));
                            }
                            _ => bail!("nonqualitative admission"),
                        }
                    }
                    // Private review prose is retained in CAS, excluded from public projection.
                    s.history.push(json!({"revision":id,"proposal":review.proposal,"decision":review.decision,"basis":sub.basis,"generated":sub.response.proposals}));
                }
                Entry::Status { update } => {
                    if update.withdrawn {
                        s.withdrawn.insert(update.identity);
                    } else {
                        s.withdrawn.remove(&update.identity);
                    }
                }
            }
        }
        Ok(s)
    }
    pub fn qualitative_init(&self, input: Bootstrap) -> Result<Value> {
        self.write(|| {
            ensure!(
                self.events()?.is_empty(),
                "bootstrap requires an isolated empty native store"
            );
            ensure!(
                (1..=16).contains(&input.parents.len())
                    && input.records.len() <= 64
                    && input.constraints.len() <= 16
                    && input.acceptance_references.len() <= 16,
                "context bounds"
            );
            let mut labels = BTreeSet::new();
            let mut identities = BTreeSet::new();
            let mut parents = vec![];
            for p in &input.parents {
                ensure!(
                    bounded(&p.label)
                        && bounded(&p.meaning)
                        && bounded(&p.attribution)
                        && labels.insert(p.label.clone()),
                    "parent content/label"
                );
                let pref = Preference::new(
                    p.meaning.clone(),
                    Scope::Task(Self::task()),
                    vec![],
                    Source::Operator,
                )?;
                ensure!(identities.insert(pref.id), "duplicate parent meaning");
                parents.push(pref);
            }
            labels.clear();
            for r in &input.records {
                ensure!(
                    [&r.label, &r.content, &r.attribution, &r.uncertainty]
                        .iter()
                        .all(|s| bounded(s))
                        && labels.insert(r.label.clone()),
                    "visible record bounds"
                );
            }
            ensure!(
                input.constraints.iter().all(|s| bounded(s)),
                "constraint bounds"
            );
            self.append(
                EventKind::TaskOpened { task: Self::task() },
                Source::Operator,
            )?;
            self.qualitative_append(Entry::Bootstrap { input, parents }, Source::Operator)?;
            self.qualitative_request()
        })
    }
    pub fn qualitative_request(&self) -> Result<Value> {
        let s = self.qualitative_state()?;
        let active = s.active();
        let root = self.store.head_root()?;
        let head = self.events()?.last().map(|x| x.1.id);
        let pending:Vec<_>=s.proposals.iter().filter(|(id,_)|!s.reviewed.contains(id)).map(|(id,sub)|json!({"proposal":id,"current":head==Some(*id),"generator":sub.generator,"basis":sub.basis,"response":sub.response})).collect();
        let review_due = pending.iter().any(|p| p["current"] == true);
        let superseded: BTreeSet<Hash> = s
            .guidance
            .iter()
            .filter_map(|(_, p, _)| p.revises)
            .collect();
        let guidance:Vec<_>=s.guidance.iter().filter(|(id,p,pin)|!superseded.contains(&id.0)&&p.parents.iter().all(|x|active.contains(x))&&s.parent_context(&p.parents)==*pin).map(|(id,p,_)|json!({"revision":id,"proposal":p,"basis":s.history.iter().find(|h| h["revision"]==json!(id)).map(|h|h["basis"].clone()).unwrap_or(json!(Basis::InitialContext{context:s.context})),"semantic_status":"reviewed interpretation, not observed truth"})).collect();
        let binding = Hash::of(&[&serde_json::to_vec(&(root, s.context, &active))?]);
        Ok(
            json!({"protocol":PROTOCOL,"state_root":root,"attention_binding":binding,
          "phase":if active.is_empty(){"stop"}else if review_due {"review"} else {"propose"},"pending_reviews":pending,"basis":Basis::InitialContext{context:s.context},
          "parents":s.current.iter().map(|(id,p)|json!({"id":id,"original_meaning":p.original.text,"meaning":p.meaning,"revision":p.revision,"parents":p.parents,"active":active.contains(id)})).collect::<Vec<_>>(),
          "source_parents":s.input.parents,"visible_records":s.input.records,"constraints":s.input.constraints,
          "acceptance_references":s.input.acceptance_references,"acceptance_status":"not evaluated here",
          "observed_returns":[],"guidance":guidance,"history":s.history,
          "response_contract":{
            "submission_fields":["basis","generator","response"],
            "basis":Basis::InitialContext{context:s.context},
            "response_fields":{"protocol":PROTOCOL,"state_root":root,"attention_binding":binding,"proposals":"0..4 native Suggestion objects; zero means abstain","common_event_forecasts":"omit; no acceptance forecasts here"},
            "allowed_kinds":["public_desire","relational","new_candidate"],
            "public_desire_fields":{"binding":binding,"identity":"active native parent ID","revises":"current parent revision or null","parents":"distinct active IDs including identity; retain prior ancestry","meaning":"bounded proposed meaning","change":"clarification or development","basis":{"kind":"unknown"},"generator":"same as submission generator"},
            "relational_fields":{"binding":binding,"proposal":{"parents":"at least two distinct active parent IDs","relationship":"bounded relationship text","aspiration":"bounded newly wanted condition","discriminator":"bounded unresolved discriminator, not a passing claim","revises":"latest relation revision or null","catalysts":[],"development":"omit; no empirical return exists"}},
            "new_candidate_fields":{"description":"bounded possibility","serves":"active parent ID strings","rationale":"bounded reason"},
            "text_bytes_max":512,"admission":"one public_desire or relational only; new_candidate and abstention remain preview-only"},
          "authority":{"executions":0,"acceptance_changes":false,"automatic_admission":false},
          "instructions":"Use the native Response protocol. PublicDesire and Relational are quarantined pending distinct independent review. NewCandidate is preview-only. Initial context is attributed visible information, never an action return. No required novelty; abstention may return zero proposals. No utility weights or measurable fulfillment are inferred. PublicDesire basis must be unknown; relational catalysts/development must be empty/absent in this initial-context profile."}),
        )
    }
    fn qualitative_validate(&self, sub: &Submission, original_root: Option<Hash>) -> Result<()> {
        let s = self.qualitative_state()?;
        let q = self.qualitative_request()?;
        let r = &sub.response;
        ensure!(
            bounded(&sub.generator) && self.valid_qualitative_basis(&sub.basis, original_root)?,
            "initial-context provenance"
        );
        let expected = original_root.unwrap_or(self.store.head_root()?);
        ensure!(
            r.protocol == PROTOCOL && r.state_root == expected,
            "stale response/protocol"
        );
        if original_root.is_none() {
            ensure!(
                serde_json::to_value(r.attention_binding)? == q["attention_binding"],
                "stale attention"
            );
        }
        ensure!(
            r.common_event_forecasts.is_empty() && r.proposals.len() <= 4,
            "no acceptance forecasts; bounded alternatives"
        );
        let active = s.active();
        ensure!(
            !active.is_empty() || r.proposals.is_empty(),
            "withdrawn parents cannot guide"
        );
        for p in &r.proposals {
            match p {
                Suggestion::PublicDesire { proposal: p } => {
                    let old = s
                        .current
                        .get(&p.identity)
                        .context("unknown public identity")?;
                    ensure!(
                        active.contains(&p.identity)
                            && p.revises == old.revision
                            && p.binding == r.attention_binding.context("binding")?,
                        "stale desire lineage"
                    );
                    ensure!(
                        matches!(p.basis, DesireBasis::Unknown)
                            && p.generator == sub.generator
                            && bounded(&p.meaning)
                            && p.meaning.trim() != old.meaning,
                        "unknown basis and changed bounded meaning required"
                    );
                    validate_parents(&p.parents, &active, 1)?;
                    ensure!(
                        p.parents.contains(&p.identity)
                            && old.parents.iter().all(|id| p.parents.contains(id)),
                        "retain participating parents"
                    );
                }
                Suggestion::Relational {
                    binding,
                    proposal: p,
                } => {
                    ensure!(Some(*binding) == r.attention_binding, "relational binding");
                    validate_parents(&p.parents, &active, 2)?;
                    ensure!(
                        [&p.relationship, &p.aspiration, &p.discriminator]
                            .iter()
                            .all(|x| bounded(x))
                            && p.catalysts.is_empty()
                            && p.development.is_none(),
                        "initial context cannot claim return development"
                    );
                    ensure!(
                        !s.guidance.iter().any(|(_, old, pin)| *pin
                            == s.parent_context(&p.parents)
                            && same_parents(&old.parents, &p.parents)
                            && old.relationship == p.relationship
                            && old.aspiration == p.aspiration
                            && old.discriminator == p.discriminator),
                        "duplicate relational meaning"
                    );
                    if let Some(prior) = p.revises {
                        let (_, old, old_context) = s
                            .guidance
                            .iter()
                            .find(|(id, _, _)| id.0 == prior)
                            .context("unknown revision")?;
                        ensure!(
                            !s.guidance.iter().any(|(_, x, _)| x.revises == Some(prior))
                                && old.parents.iter().all(|id| p.parents.contains(id)),
                            "stale/erased lineage"
                        );
                        ensure!(
                            *old_context != s.parent_context(&p.parents)
                                || old.relationship != p.relationship
                                || old.aspiration != p.aspiration
                                || old.discriminator != p.discriminator
                                || !same_parents(&old.parents, &p.parents),
                            "unchanged relation"
                        );
                    }
                }
                Suggestion::NewCandidate {
                    description,
                    serves,
                    rationale,
                } => {
                    ensure!(
                        bounded(description)
                            && bounded(rationale)
                            && !serves.is_empty()
                            && serves
                                .iter()
                                .all(|id| active.iter().any(|p| p.to_string() == *id)),
                        "candidate public parent references"
                    );
                }
                _ => bail!("no executable or empirical proposal in qualitative context"),
            }
        }
        Ok(())
    }
    /// Prospective v2 local view. V1 export and its stopping contract are unchanged.
    pub fn qualitative_request_v2(&self) -> Result<Value> {
        let mut q = self.qualitative_request()?;
        let s = self.qualitative_state()?;
        let active = s.active();
        let inquiry_ids: BTreeSet<_> = self
            .qualitative_entries()?
            .iter()
            .filter_map(|(id, e)| matches!(e, Entry::InquiryProposed { .. }).then_some(*id))
            .collect();
        let mut reviewed_questions = vec![];
        for id in &inquiry_ids {
            if let Ok(inquiry) = self.current_inquiry(*id) {
                reviewed_questions.push(json!({"proposal":id,"inquiry":inquiry}));
            }
        }
        if !inquiry_ids.is_empty() {
            q["reviewed_information_questions"] = json!(reviewed_questions);
        }
        let mut memory = vec![];
        for (revision, proposal, sub, pin, reviewer) in &s.possibilities {
            if inquiry_ids.contains(proposal) && self.current_inquiry(*proposal).is_err() {
                continue;
            }
            if let Suggestion::NewCandidate { serves, .. } = &sub.response.proposals[0] {
                let parents: Vec<PreferenceId> = serves
                    .iter()
                    .map(|v| serde_json::from_value(json!(v)))
                    .collect::<std::result::Result<_, _>>()?;
                if parents.iter().all(|p| active.contains(p)) && *pin == s.parent_context(&parents)
                {
                    memory.push(json!({"revision":revision,"proposal":proposal,"candidate":sub.response.proposals[0],"basis":sub.basis,"generator":sub.generator,"reviewer":reviewer,"status":"reviewed possibility, not changed wanting or verified outcome"}));
                }
            }
        }
        q["context_contract"] = json!("qualitative-context-view@v2");
        q["possibility_memory"] = json!(memory);
        q["feedback_limit"] = json!("No action or information-return adapter is configured here; initial context is not empirical return.");
        q["response_contract"]["admission"] = json!("Meaning proposals use submit; ordinary possibilities use submit-possibility. Both require independent review; neither grants execution.");
        q["response_contract"]["relational_fields"]["proposal"]["aspiration"] = json!("bounded wanted condition; may retain the prior condition verbatim while relationship changes");
        q["instructions"] = json!("Distinguish a useful possibility from changed wanting. A reviewed relationship may develop while retaining its wanted condition. Unchanged meaning, development and abstention are equally legitimate; propose what the context supports, without manufacturing or suppressing novelty. Reviews must assess parent contribution, uncertainty and support, not just field changes. Initial context is not empirical return. No execution authority.");
        self.add_retrieval_context(&mut q)?;
        self.add_information_expectation(&mut q)?;
        Ok(q)
    }
    /// Explicit opt-in path; legacy submit still rejects ordinary possibilities.
    pub fn qualitative_submit_possibility(&self, sub: Submission) -> Result<EventId> {
        self.write(|| {
            self.qualitative_validate(&sub, None)?;
            ensure!(
                self.qualitative_request()?["phase"] == "propose",
                "current state requires review or stop"
            );
            let s = self.qualitative_state()?;
            ensure!(s.proposals.len() < 128, "qualitative proposal capacity");
            ensure!(
                sub.response.proposals.len() == 1
                    && matches!(sub.response.proposals[0], Suggestion::NewCandidate { .. }),
                "only ordinary possibility in this path"
            );
            let candidate = serde_json::to_value(&sub.response.proposals[0])?;
            ensure!(
                !self.qualitative_request_v2()?["possibility_memory"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|p| p["candidate"] == candidate),
                "duplicate reviewed possibility"
            );
            self.qualitative_append(
                Entry::ProposedPossibility { submission: sub },
                Source::Generator,
            )
        })
    }
    pub fn qualitative_preview(&self, sub: Submission) -> Result<Value> {
        self.qualitative_validate(&sub, None)?;
        Ok(
            json!({"valid":true,"executions":0,"admitted":false,"proposal_digest":Hash::of(&[&serde_json::to_vec(&sub)?])}),
        )
    }
    pub fn qualitative_submit(&self, sub: Submission) -> Result<EventId> {
        self.write(|| {
            self.qualitative_validate(&sub, None)?;
            ensure!(
                self.qualitative_request()?["phase"] == "propose",
                "current state requires review or stop"
            );
            ensure!(
                self.qualitative_state()?.proposals.len() < 128,
                "qualitative proposal capacity"
            );
            ensure!(
                sub.response.proposals.len() == 1
                    && matches!(
                        sub.response.proposals[0],
                        Suggestion::PublicDesire { .. } | Suggestion::Relational { .. }
                    ),
                "only one qualitative candidate may await admission"
            );
            self.qualitative_append(Entry::Proposed { submission: sub }, Source::Generator)
        })
    }
    pub fn qualitative_review(&self, review: Review) -> Result<Value> {
        self.write(|| {
            ensure!(review.state_root == self.store.head_root()?, "stale review");
            let s = self.qualitative_state()?;
            let sub = s
                .proposals
                .get(&review.proposal)
                .context("unknown proposal")?;
            ensure!(
                !s.reviewed.contains(&review.proposal)
                    && bounded(&review.reviewer)
                    && bounded(&review.reason)
                    && review.reviewer != sub.generator,
                "distinct review/attribution required"
            );
            // Exactly the quarantined prefix must still be current: intervening state invalidates review.
            ensure!(
                self.events()?.last().map(|x| x.1.id) == Some(review.proposal),
                "context changed after proposal"
            );
            self.qualitative_validate(sub, Some(sub.response.state_root))?;
            self.qualitative_append(Entry::Reviewed { review }, Source::Operator)?;
            self.qualitative_request()
        })
    }
    pub fn qualitative_status(&self, update: Status) -> Result<Value> {
        self.write(|| {
            ensure!(update.state_root == self.store.head_root()?, "stale status");
            let s = self.qualitative_state()?;
            let p = s
                .current
                .get(&update.identity)
                .context("unknown identity")?;
            ensure!(p.revision == update.revision, "stale status revision");
            if s.withdrawn.contains(&update.identity) != update.withdrawn {
                self.qualitative_append(Entry::Status { update }, Source::Operator)?;
            }
            self.qualitative_request()
        })
    }
}
fn same_parents(a: &[PreferenceId], b: &[PreferenceId]) -> bool {
    a.iter().copied().collect::<BTreeSet<_>>() == b.iter().copied().collect::<BTreeSet<_>>()
}
fn validate_parents(
    parents: &[PreferenceId],
    active: &BTreeSet<PreferenceId>,
    min: usize,
) -> Result<()> {
    ensure!(
        (min..=16).contains(&parents.len())
            && parents.iter().copied().collect::<BTreeSet<_>>().len() == parents.len()
            && parents.iter().all(|p| active.contains(p)),
        "inactive/duplicate parent lineage"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use adaptive::coding::project::desire::{Change, Proposal as PublicProposal};
    fn input() -> Bootstrap {
        Bootstrap {
            parents: vec![
                Parent {
                    label: "a".into(),
                    meaning: "Find a dependable beginning".into(),
                    attribution: "authored test, not a human observation".into(),
                },
                Parent {
                    label: "b".into(),
                    meaning: "Keep disagreement discoverable".into(),
                    attribution: "authored test".into(),
                },
                Parent {
                    label: "c".into(),
                    meaning: "Keep original provenance".into(),
                    attribution: "authored test".into(),
                },
            ],
            records: vec![VisibleRecord {
                label: "r1".into(),
                content: "A fictional assertion".into(),
                attribution: "evaluator fiction".into(),
                uncertainty: "world truth unknown".into(),
            }],
            constraints: vec!["Do not erase dissent".into()],
            acceptance_references: vec![Hash::of(&[b"immutable external acceptance reference"])],
        }
    }
    fn start() -> Result<(tempfile::TempDir, Engine)> {
        let d = tempfile::tempdir()?;
        let e = Engine::open(d.path())?;
        e.qualitative_init(input())?;
        Ok((d, e))
    }
    fn ids(q: &Value) -> Vec<PreferenceId> {
        q["parents"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| serde_json::from_value(p["id"].clone()).unwrap())
            .collect()
    }
    fn sub(q: &Value, p: Suggestion) -> Submission {
        Submission {
            basis: serde_json::from_value(q["basis"].clone()).unwrap(),
            generator: "fixture-generator".into(),
            response: Response {
                protocol: PROTOCOL.into(),
                state_root: serde_json::from_value(q["state_root"].clone()).unwrap(),
                attention_binding: serde_json::from_value(q["attention_binding"].clone()).unwrap(),
                proposals: vec![p],
                pursuit_expectations: Default::default(),
                common_event_forecasts: BTreeMap::new(),
            },
        }
    }
    fn relation(q: &Value) -> Submission {
        sub(
            q,
            Suggestion::Relational {
                binding: serde_json::from_value(q["attention_binding"].clone()).unwrap(),
                proposal: deliberation::relational::Proposal {
                    parents: ids(q)[..2].to_vec(),
                    relationship: "Authored fixture relationship; not model emergence".into(),
                    aspiration: "Authored fixture wanted condition".into(),
                    discriminator: "No observation yet".into(),
                    revises: None,
                    catalysts: vec![],
                    development: None,
                },
            },
        )
    }
    fn review(e: &Engine, p: EventId, decision: Decision) -> Result<Value> {
        e.qualitative_review(Review {
            state_root: e.store.head_root()?,
            proposal: p,
            reviewer: "independent-fixture-reviewer".into(),
            decision,
            reason: "Authored wiring control; not semantic validation".into(),
        })
    }
    #[test]
    fn reviewed_possibility_is_memory_not_wanting_and_survives_restart() -> Result<()> {
        let (d, e) = start()?;
        let q = e.qualitative_request()?;
        let make = |q: &Value| {
            sub(
                q,
                Suggestion::NewCandidate {
                    description: "Authored possible route; untested".into(),
                    serves: ids(q)[..2].iter().map(ToString::to_string).collect(),
                    rationale: "Both parents contribute; usefulness unknown".into(),
                },
            )
        };
        assert!(e.qualitative_submit(make(&q)).is_err());
        let p = e.qualitative_submit_possibility(make(&q))?;
        assert!(e.qualitative_request_v2()?["possibility_memory"]
            .as_array()
            .unwrap()
            .is_empty());
        review(&e, p, Decision::Admit)?;
        let after = e.qualitative_request_v2()?;
        assert_eq!(after["parents"], q["parents"]);
        assert_eq!(after["guidance"], q["guidance"]);
        assert_eq!(after["authority"], q["authority"]);
        assert_eq!(after["acceptance_references"], q["acceptance_references"]);
        assert_eq!(after["observed_returns"], json!([]));
        assert_eq!(after["possibility_memory"].as_array().unwrap().len(), 1);
        assert!(e.qualitative_submit_possibility(make(&after)).is_err());
        assert_eq!(
            e.public_meaning_context()?["possibility_memory"],
            after["possibility_memory"]
        );
        drop(e);
        let e = Engine::open(d.path())?;
        assert_eq!(e.qualitative_request_v2()?, after);
        let id = ids(&after)[0];
        e.qualitative_status(Status {
            state_root: e.store.head_root()?,
            identity: id,
            revision: None,
            withdrawn: true,
        })?;
        let withdrawn = e.qualitative_request_v2()?;
        assert!(withdrawn["possibility_memory"]
            .as_array()
            .unwrap()
            .is_empty());
        assert_eq!(withdrawn["history"], after["history"]);
        assert!(e.public_meaning_context().is_err());
        e.qualitative_status(Status {
            state_root: e.store.head_root()?,
            identity: id,
            revision: None,
            withdrawn: false,
        })?;
        assert_eq!(
            e.qualitative_request_v2()?["possibility_memory"],
            after["possibility_memory"]
        );
        Ok(())
    }
    #[test]
    fn relationship_can_change_without_new_wanting_and_rejection_keeps_history() -> Result<()> {
        let (_d, e) = start()?;
        let q = e.qualitative_request()?;
        let p = e.qualitative_submit(relation(&q))?;
        review(&e, p, Decision::Admit)?;
        let q = e.qualitative_request()?;
        let prior = q["guidance"][0].clone();
        let mut revised = relation(&q);
        if let Suggestion::Relational { proposal, .. } = &mut revised.response.proposals[0] {
            proposal.revises = Some(serde_json::from_value(prior["revision"].clone())?);
            proposal.relationship =
                "Authored changed interaction, preserving wanted condition; untested".into();
        }
        let p = e.qualitative_submit(revised)?;
        review(&e, p, Decision::Admit)?;
        let changed = e.qualitative_request_v2()?;
        assert_eq!(
            changed["guidance"][0]["proposal"]["aspiration"],
            prior["proposal"]["aspiration"]
        );
        assert_eq!(changed["parents"], q["parents"]);
        assert_eq!(e.qualitative_request_v2()?, changed);
        let mut bad = relation(&changed);
        if let Suggestion::Relational { proposal, .. } = &mut bad.response.proposals[0] {
            proposal.aspiration = "Unsupported authored novelty".into();
        }
        let p = e.qualitative_submit(bad)?;
        review(&e, p, Decision::Reject)?;
        let rejected = e.qualitative_request_v2()?;
        assert_eq!(rejected["guidance"], changed["guidance"]);
        assert!(
            rejected["history"].as_array().unwrap().len()
                > changed["history"].as_array().unwrap().len()
        );
        let mut abstain = relation(&rejected);
        abstain.response.proposals.clear();
        let root = e.store.head_root()?;
        e.qualitative_preview(abstain)?;
        assert_eq!(e.store.head_root()?, root);
        Ok(())
    }
    #[test]
    fn possibility_unknown_and_stale_review_never_admit() -> Result<()> {
        let (_d, e) = start()?;
        let q = e.qualitative_request()?;
        let p = e.qualitative_submit_possibility(sub(
            &q,
            Suggestion::NewCandidate {
                description: "Untested possibility".into(),
                serves: vec![ids(&q)[0].to_string()],
                rationale: "Unknown support".into(),
            },
        ))?;
        let r = Review {
            state_root: e.store.head_root()?,
            proposal: p,
            reviewer: "independent-fixture".into(),
            decision: Decision::Admit,
            reason: "Stale authored review".into(),
        };
        review(&e, p, Decision::Unknown)?;
        assert!(e.qualitative_review(r).is_err());
        assert!(e.qualitative_request_v2()?["possibility_memory"]
            .as_array()
            .unwrap()
            .is_empty());
        assert_eq!(e.qualitative_request_v2()?["parents"], q["parents"]);
        Ok(())
    }
    fn develop(
        e: &Engine,
        identity: PreferenceId,
        parents: Vec<PreferenceId>,
        meaning: &str,
    ) -> Result<Value> {
        let q = e.qualitative_request()?;
        let p = PublicProposal {
            binding: serde_json::from_value(q["attention_binding"].clone())?,
            identity,
            revises: None,
            parents,
            meaning: meaning.into(),
            change: Change::Development,
            basis: DesireBasis::Unknown,
            generator: "fixture-generator".into(),
        };
        let id = e.qualitative_submit(sub(&q, Suggestion::PublicDesire { proposal: p }))?;
        review(e, id, Decision::Admit)
    }
    #[test]
    fn initial_admission_restart_and_next_candidate_without_return() -> Result<()> {
        let (d, e) = start()?;
        let q = e.qualitative_request()?;
        assert_eq!(q["observed_returns"], json!([]));
        assert_eq!(q["guidance"], json!([]));
        let p = e.qualitative_submit(relation(&q))?;
        assert_eq!(e.qualitative_request()?["guidance"], json!([]));
        let after = review(&e, p, Decision::Admit)?;
        assert_eq!(after["guidance"].as_array().unwrap().len(), 1);
        assert_eq!(q["acceptance_references"], after["acceptance_references"]);
        assert_eq!(q["parents"], after["parents"]);
        assert_eq!(after["observed_returns"], json!([]));
        let restarted = Engine::open(d.path())?;
        assert_eq!(after, restarted.qualitative_request()?);
        let candidate = sub(
            &after,
            Suggestion::NewCandidate {
                description: "Authored fixture possibility".into(),
                serves: vec![ids(&after)[0].to_string()],
                rationale: "References persisted context, not an observation".into(),
            },
        );
        assert!(restarted.qualitative_preview(candidate.clone()).is_ok());
        assert!(restarted.qualitative_submit(candidate).is_err());
        assert_eq!(after, restarted.qualitative_request()?);
        assert!(e.events()?.iter().all(|(_, e)| !matches!(
            e.kind,
            EventKind::Executed { .. }
                | EventKind::Committed { .. }
                | EventKind::Observed {
                    observation: Observation::Return { .. }
                }
        )));
        Ok(())
    }
    #[test]
    fn forged_empirical_basis_authority_and_stale_context_rejected() -> Result<()> {
        let (_d, e) = start()?;
        let q = e.qualitative_request()?;
        let original = relation(&q);
        let mut v = serde_json::to_value(&original)?;
        v["basis"] = json!({"kind":"observed_return","receipt":q["state_root"]});
        assert!(serde_json::from_value::<Submission>(v).is_err());
        let mut bad = original.clone();
        bad.basis = Basis::InitialContext {
            context: EventId(Hash::of(&[b"forged"])),
        };
        assert!(e.qualitative_submit(bad).is_err());
        let mut v = serde_json::to_value(&original)?;
        v["authority"] = json!({"execute":true});
        assert!(serde_json::from_value::<Submission>(v).is_err());
        let id = e.qualitative_submit(original.clone())?;
        assert!(e.qualitative_submit(original).is_err());
        let root = e.store.head_root()?;
        assert!(e
            .qualitative_review(Review {
                state_root: root,
                proposal: id,
                reviewer: "fixture-generator".into(),
                decision: Decision::Admit,
                reason: "self review".into()
            })
            .is_err());
        let q = e.qualitative_request()?;
        e.qualitative_status(Status {
            state_root: root,
            identity: ids(&q)[2],
            revision: None,
            withdrawn: true,
        })?;
        assert!(review(&e, id, Decision::Admit).is_err());
        Ok(())
    }
    #[test]
    fn public_development_and_transitive_withdrawal_keep_originals() -> Result<()> {
        let (_d, e) = start()?;
        let initial = e.qualitative_request()?;
        let id = ids(&initial);
        develop(&e, id[0], vec![id[0], id[1]], "Authored changed meaning A")?;
        let q = develop(&e, id[2], vec![id[2], id[0]], "Authored changed meaning C")?;
        assert_eq!(q["acceptance_references"], initial["acceptance_references"]);
        assert_ne!(q["parents"], initial["parents"]);
        assert_eq!(q["authority"], initial["authority"]);
        let after = e.qualitative_status(Status {
            state_root: e.store.head_root()?,
            identity: id[1],
            revision: None,
            withdrawn: true,
        })?;
        assert_eq!(after["phase"], "stop");
        assert!(after["parents"]
            .as_array()
            .unwrap()
            .iter()
            .all(|p| p["active"] == false));
        assert_eq!(after["history"], q["history"]);
        let restored = e.qualitative_status(Status {
            state_root: e.store.head_root()?,
            identity: id[1],
            revision: None,
            withdrawn: false,
        })?;
        assert_eq!(restored["phase"], "propose");
        assert_eq!(restored["history"], q["history"]);
        Ok(())
    }
    #[test]
    fn unchanged_replay_unknown_rejection_and_noop_status() -> Result<()> {
        let (_d, e) = start()?;
        let q = e.qualitative_request()?;
        assert_eq!(q, e.qualitative_request()?);
        let noop = e.qualitative_status(Status {
            state_root: e.store.head_root()?,
            identity: ids(&q)[0],
            revision: None,
            withdrawn: false,
        })?;
        assert_eq!(q, noop);
        let id = e.qualitative_submit(relation(&q))?;
        let after = review(&e, id, Decision::Unknown)?;
        assert_eq!(after["guidance"], json!([]));
        assert_eq!(after["parents"], q["parents"]);
        assert!(review(&e, id, Decision::Admit).is_err());
        let mut abstain = relation(&after);
        abstain.response.proposals.clear();
        assert!(e.qualitative_preview(abstain.clone()).is_ok());
        assert!(e.qualitative_submit(abstain).is_err());
        assert!(e.qualitative_init(input()).is_err());
        Ok(())
    }
    #[test]
    fn duplicate_relation_and_fake_development_rejected() -> Result<()> {
        let (_d, e) = start()?;
        let q = e.qualitative_request()?;
        let id = e.qualitative_submit(relation(&q))?;
        let after = review(&e, id, Decision::Admit)?;
        assert!(e.qualitative_submit(relation(&after)).is_err());
        let mut fake = relation(&after);
        if let Suggestion::Relational { proposal, .. } = &mut fake.response.proposals[0] {
            proposal.relationship = "different".into();
            proposal.catalysts.push(Hash::of(&[b"fake return"]));
        }
        assert!(e.qualitative_submit(fake).is_err());
        Ok(())
    }
    #[test]
    fn stale_guidance_and_full_event_head_require_new_review() -> Result<()> {
        let (_d, e) = start()?;
        let q = e.qualitative_request()?;
        let id = e.qualitative_submit(relation(&q))?;
        let admitted = review(&e, id, Decision::Admit)?;
        let relation_revision: Hash =
            serde_json::from_value(admitted["guidance"][0]["revision"].clone())?;
        let parent_ids = ids(&q);
        let developed = develop(
            &e,
            parent_ids[0],
            vec![parent_ids[0], parent_ids[1]],
            "Authored changed participating meaning",
        )?;
        assert_eq!(developed["guidance"], json!([]));
        assert_eq!(developed["history"].as_array().unwrap().len(), 2);
        let mut reconsider = relation(&developed);
        if let Suggestion::Relational { proposal, .. } = &mut reconsider.response.proposals[0] {
            proposal.revises = Some(relation_revision);
        }
        let p = e.qualitative_submit(reconsider)?;
        let restored = review(&e, p, Decision::Admit)?;
        assert_eq!(restored["guidance"].as_array().unwrap().len(), 1);
        let mut next = relation(&restored);
        if let Suggestion::Relational { proposal, .. } = &mut next.response.proposals[0] {
            proposal.relationship = "Another authored relation".into();
        }
        let p = e.qualitative_submit(next)?;
        e.write(|| {
            e.append(
                EventKind::TaskOpened {
                    task: Engine::task(),
                },
                Source::Operator,
            )
        })?;
        assert!(review(&e, p, Decision::Admit).is_err());
        Ok(())
    }
    #[test]
    fn reordered_parents_are_not_development() -> Result<()> {
        let (_d, e) = start()?;
        let q = e.qualitative_request()?;
        let p = e.qualitative_submit(relation(&q))?;
        let after = review(&e, p, Decision::Admit)?;
        let mut alias = relation(&after);
        if let Suggestion::Relational { proposal, .. } = &mut alias.response.proposals[0] {
            proposal.parents.reverse();
        }
        assert!(e.qualitative_submit(alias).is_err());
        Ok(())
    }
    #[test]
    fn last_capacity_slot_can_be_reviewed() -> Result<()> {
        let (_d, e) = start()?;
        // Authored quarantine history, no model generation and no admission.
        for _ in 0..128 {
            let q = e.qualitative_request()?;
            let p = e.qualitative_submit(relation(&q))?;
            review(&e, p, Decision::Reject)?;
        }
        let q = e.qualitative_request()?;
        assert!(e.qualitative_preview(relation(&q)).is_ok());
        assert!(e.qualitative_submit(relation(&q)).is_err());
        assert_eq!(q["guidance"], json!([]));
        Ok(())
    }
}
