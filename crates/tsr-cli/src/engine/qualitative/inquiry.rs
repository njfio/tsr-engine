//! Reviewed information intentions use the existing attention kernel and possibility memory.
//! Attention orders support identities, not semantic importance or permission to inspect.
use super::*;
use tsr_ledger::selection::{select, SelectionContext};
use tsr_ledger::{CollectiveReality, LedgerEntry};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    UserSupplied,
    ModelProposed,
    EngineExploration,
}
/// Operator policy only; does not admit a read or grant a provider budget.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExplorationPolicy {
    pub state_root: Hash,
    pub enabled: bool,
    pub max_records: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Exploration {
    pub policy: EventId,
    pub considered_records: Vec<String>,
    pub possible_information_benefit: String,
    pub why_no_user_priority_needed: String,
}
/// Proposed answer source, reviewed with the entire inquiry; not semantic proof.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum AnswerSource {
    CorpusAssertions { corpus: Hash, rationale: String },
    UserPreference,
    UserAuthority,
    ExternalEvidence,
    Mixed,
    Unknown,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum InquiryRoute {
    CorpusCandidate { corpus: Hash },
    Waiting { reason: String },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Inquiry {
    pub state_root: Hash,
    pub attention: Hash,
    pub parents: Vec<PreferenceId>,
    pub question: String,
    pub uncertainty: String,
    pub attribution: String,
    pub origin: Origin,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub answer_source: Option<AnswerSource>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exploration: Option<Exploration>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InformationChoice {
    pub inquiry: EventId,
    pub request: retrieval::Request,
    /// Pre-read relevance hypothesis, never evidence that the content supports an answer.
    pub relevance: String,
}
impl Engine {
    fn exploration_policy(&self) -> Result<Option<(EventId, ExplorationPolicy)>> {
        Ok(self
            .qualitative_entries()?
            .into_iter()
            .filter_map(|(id, e)| match e {
                Entry::ExplorationPolicy { policy } => Some((id, policy)),
                _ => None,
            })
            .next_back())
    }
    pub fn qualitative_set_exploration_policy(&self, policy: ExplorationPolicy) -> Result<EventId> {
        self.write(|| {
            ensure!(policy.state_root == self.store.head_root()?, "stale policy");
            self.qualitative_state()?;
            ensure!(
                (1..=4).contains(&policy.max_records),
                "bounded existing reader policy"
            );
            self.qualitative_append(Entry::ExplorationPolicy { policy }, Source::Operator)
        })
    }
    fn validate_exploration(&self, inquiry: &Inquiry) -> Result<()> {
        if inquiry.origin != Origin::EngineExploration {
            ensure!(
                inquiry.exploration.is_none(),
                "exploration requires explicit engine attribution"
            );
            return Ok(());
        }
        let e = inquiry
            .exploration
            .as_ref()
            .context("engine exploration requires attributed benefit and policy")?;
        let (id, p) = self
            .exploration_policy()?
            .context("exploration policy is off")?;
        ensure!(
            p.enabled && id == e.policy,
            "disabled or stale exploration policy"
        );
        ensure!(
            bounded(&e.possible_information_benefit) && bounded(&e.why_no_user_priority_needed),
            "explicit plausible benefit required; abstain without benefit"
        );
        let (m, _) = self.retrieval_state()?;
        let m = m.context("no bounded source")?;
        let corpus = Hash::of(&[&serde_json::to_vec(&m)?]);
        ensure!(
            matches!(&inquiry.answer_source,Some(AnswerSource::CorpusAssertions{corpus:c,..}) if *c==corpus),
            "exploration cannot resolve user authority/preference or unsupported sources"
        );
        ensure!(
            (1..=32).contains(&e.considered_records.len())
                && e.considered_records.iter().collect::<BTreeSet<_>>().len()
                    == e.considered_records.len()
                && e.considered_records
                    .iter()
                    .all(|id| m.records.iter().any(|r| &r.id == id)),
            "consider only admitted record options"
        );
        Ok(())
    }
    pub(super) fn validate_exploration_read(&self, id: EventId, selected: &[String]) -> Result<()> {
        let inquiry = self.current_inquiry(id)?;
        if let Some(e) = inquiry.exploration {
            let (_, p) = self.exploration_policy()?.context("missing policy")?;
            ensure!(
                selected.len() <= p.max_records
                    && selected.iter().all(|id| e.considered_records.contains(id)),
                "read exceeds reviewed exploration options/bound"
            );
        }
        Ok(())
    }
    /// Prospective view. Legacy v1/v2 exports remain frozen.
    pub fn qualitative_question_request(&self) -> Result<Value> {
        let mut q = self.qualitative_request_v2()?;
        let s = self.qualitative_state()?;
        let root = self.store.head_root()?;
        let active = s.active();
        let ledger = CollectiveReality {
            root,
            entries: s
                .current
                .iter()
                .map(|(id, p)| LedgerEntry {
                    preference: p.original.clone(),
                    expanded_by: None,
                    from_contrast: None,
                    from_success: None,
                    from_provisional: None,
                    catalyzed_by: p.parents.clone(),
                    status: Some(if active.contains(id) {
                        PreferenceStatus::Open
                    } else {
                        PreferenceStatus::Withdrawn
                    }),
                    history: vec![],
                })
                .collect(),
        };
        let snapshot = tsr_domain::Snapshot {
            task: Self::task(),
            root,
            version: self.events()?.len() as u64,
            open_criteria: vec![],
            missing_evidence: vec![],
        };
        // Same native kernel; expose all eligible identities, no fabricated semantic priority.
        let tune = tsr_domain::Tune {
            intuition_max: 16,
            quiet_route: 0.0,
            ..Default::default()
        };
        let trace = select(
            &ledger,
            &snapshot,
            &tune,
            &SelectionContext {
                family: "public-meaning".into(),
                repository: "current-scope-only".into(),
                minimum_depth: 1,
            },
            0,
        )
        .map_err(anyhow::Error::msg)?;
        q["native_attention"] = json!(trace);
        q["exploration_policy"] = match self.exploration_policy()? {
            Some((id, p)) => {
                json!({"policy":id,"enabled":p.enabled,"max_records":p.max_records,"role":"Permits proposing bounded corpus exploration, not user preference or action authority"})
            }
            None => json!({"enabled":false,"max_records":0}),
        };
        q["inquiry_binding"] = json!(Hash::of(&[&serde_json::to_vec(&q)?]));
        let mut inquiries = vec![];
        for (id, entry) in self.qualitative_entries()? {
            if let Entry::InquiryProposed { inquiry, .. } = entry {
                inquiries.push(json!({"proposal":id,"inquiry":inquiry,"applicable":self.current_inquiry(id).is_ok()}));
            }
        }
        q["information_questions"] = json!(inquiries);
        q["information_routes"] = json!(self.qualitative_entries()?.iter().filter(|(_,e)|matches!(e,Entry::InquiryProposed{..})).map(|(id,_)|json!({"inquiry":id,"route":self.qualitative_inquiry_route(*id).unwrap_or_else(|_|InquiryRoute::Waiting{reason:"unreviewed_or_stale_inquiry".into()})})).collect::<Vec<_>>());
        q["information_contract"] = json!({
            "meaning":"Question is an attributed unresolved intention, not a new desire or authorization. Native attention is lineage/scope ordering, not topic importance. User focus is supplied only by the local operator; ordinary model inquiries use model_proposed; policy-bound self-directed exploration uses engine_exploration, never user_supplied. Review must assess whether this origin is appropriate, not just trust the label.",
            "read":"Use retrieve-for-question with an admitted current inquiry and a pre-read relevance hypothesis. Reading proves content access only; reflection separately assesses support. Abstention remains legitimate.",
            "answer_source":"Declare corpus_assertions with exact corpus and bounded rationale only for questions about what those records assert, not verified world truth. User preference/authority, external evidence, mixed, unknown or omitted source waits unresolved. Independent admission reviews source suitability as well as relevance; labels are fallible judgments, never authority.",
            "exploration":"Only with enabled explicit policy, engine_exploration may propose a bounded corpus question among plausible options without inventing user priority. Include policy ID, considered_records, possible_information_benefit and why_no_user_priority_needed. Independent review must assess benefit and source suitability. User-only preferences, authority and consequential decisions still wait. Abstain when benefit is unsupported; no forced exploration.",
            "executions_authorized":false
        });
        Ok(q)
    }
    pub fn qualitative_propose_inquiry(&self, inquiry: Inquiry) -> Result<EventId> {
        ensure!(
            matches!(
                inquiry.origin,
                Origin::ModelProposed | Origin::EngineExploration
            ),
            "model path cannot claim user focus"
        );
        self.record_inquiry(inquiry)
    }
    /// Local operator input only; never exposed as a generator response route.
    pub fn qualitative_user_inquiry(&self, inquiry: Inquiry) -> Result<EventId> {
        ensure!(
            inquiry.origin == Origin::UserSupplied,
            "operator input must identify user origin"
        );
        self.record_inquiry(inquiry)
    }
    fn record_inquiry(&self, inquiry: Inquiry) -> Result<EventId> {
        self.write(|| {
            let q = self.qualitative_question_request()?;
            ensure!(
                q["phase"] == "propose",
                "review, stale evidence or stop required"
            );
            ensure!(
                inquiry.state_root == self.store.head_root()?
                    && json!(inquiry.attention) == q["inquiry_binding"],
                "stale inquiry context"
            );
            let selected: Vec<PreferenceId> =
                serde_json::from_value(q["native_attention"]["selected"].clone())?;
            validate_parents(&inquiry.parents, &selected.into_iter().collect(), 1)?;
            ensure!(
                [
                    &inquiry.question,
                    &inquiry.uncertainty,
                    &inquiry.attribution
                ]
                .iter()
                .all(|s| bounded(s)),
                "bounded question, uncertainty and attribution required"
            );
            ensure!(
                self.qualitative_state()?.proposals.len() < 128,
                "qualitative proposal capacity"
            );
            let submission = Submission {
                basis: serde_json::from_value(q["basis"].clone())?,
                generator: inquiry.attribution.clone(),
                response: Response {
                    protocol: PROTOCOL.into(),
                    state_root: inquiry.state_root,
                    attention_binding: serde_json::from_value(q["attention_binding"].clone())?,
                    pursuit_expectations: Default::default(),
                    common_event_forecasts: BTreeMap::new(),
                    proposals: vec![Suggestion::NewCandidate {
                        description: inquiry.question.clone(),
                        serves: inquiry
                            .parents
                            .iter()
                            .map(|id| {
                                serde_json::to_value(id)
                                    .unwrap()
                                    .as_str()
                                    .unwrap()
                                    .to_owned()
                            })
                            .collect(),
                        rationale: inquiry.uncertainty.clone(),
                    }],
                },
            };
            if let Some(AnswerSource::CorpusAssertions { corpus, rationale }) =
                &inquiry.answer_source
            {
                ensure!(
                    json!(corpus) == q["retrieval_contract"]["corpus"] && bounded(rationale),
                    "unavailable corpus or missing source rationale"
                );
            }
            self.validate_exploration(&inquiry)?;
            self.qualitative_validate(&submission, None)?;
            let source = if inquiry.origin != Origin::UserSupplied {
                Source::Generator
            } else {
                Source::Operator
            };
            self.qualitative_append(
                Entry::InquiryProposed {
                    parent_context: self.qualitative_state()?.parent_context(
                        &self
                            .qualitative_state()?
                            .active()
                            .into_iter()
                            .collect::<Vec<_>>(),
                    ),
                    inquiry,
                    submission,
                },
                source,
            )
        })
    }
    pub(super) fn current_inquiry(&self, id: EventId) -> Result<Inquiry> {
        let s = self.qualitative_state()?;
        let entries = self.qualitative_entries()?;
        let pos = entries
            .iter()
            .position(|(event, _)| *event == id)
            .context("unknown inquiry")?;
        let Entry::InquiryProposed {
            inquiry,
            parent_context,
            ..
        } = &entries[pos].1
        else {
            bail!("not an inquiry")
        };
        let memory = s
            .possibilities
            .iter()
            .find(|(_, proposal, _, _, _)| *proposal == id)
            .context("inquiry not independently admitted")?;
        validate_parents(&inquiry.parents, &s.active(), 1)?;
        ensure!(
            *parent_context == s.parent_context(&s.active().into_iter().collect::<Vec<_>>()),
            "attention parent context changed"
        );
        ensure!(
            memory.3 == s.parent_context(&inquiry.parents),
            "stale inquiry parents"
        );
        ensure!(
            !entries[pos + 1..]
                .iter()
                .any(|(_, e)| matches!(e, Entry::Status { .. })),
            "status changed since inquiry; history retained"
        );
        self.validate_exploration(inquiry)?;
        Ok(inquiry.clone())
    }
    /// Reconstructed from reviewed persistent metadata; never calls a user or reader.
    pub fn qualitative_inquiry_route(&self, id: EventId) -> Result<InquiryRoute> {
        let inquiry = self.current_inquiry(id)?;
        let reason = match inquiry.answer_source {
            Some(AnswerSource::CorpusAssertions { corpus, .. }) => {
                let (manifest, _) = self.retrieval_state()?;
                if let Some(manifest) = manifest {
                    if Hash::of(&[&serde_json::to_vec(&manifest)?]) == corpus {
                        return Ok(InquiryRoute::CorpusCandidate { corpus });
                    }
                }
                "unavailable_corpus"
            }
            Some(AnswerSource::UserPreference) => "awaiting_user_preference",
            Some(AnswerSource::UserAuthority) => "awaiting_user_authority",
            Some(AnswerSource::ExternalEvidence) => "unsupported_evidence_source",
            Some(AnswerSource::Mixed) => "mixed_sources_unresolved",
            Some(AnswerSource::Unknown) | None => "answer_source_unknown",
        };
        Ok(InquiryRoute::Waiting {
            reason: reason.into(),
        })
    }
    pub fn qualitative_retrieve_for_question(&self, choice: InformationChoice) -> Result<EventId> {
        ensure!(bounded(&choice.relevance), "pre-read relevance required");
        // All state checks are repeated inside the reader's write lock.
        self.current_inquiry(choice.inquiry)?;
        self.qualitative_retrieve_bound(
            choice.request,
            Some(choice.inquiry),
            Some(choice.relevance),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> Result<(tempfile::TempDir, Engine)> {
        let d = tempfile::tempdir()?;
        let e = Engine::open(&d.path().join("state"))?;
        e.qualitative_init(Bootstrap {
            parents: vec![Parent {
                label: "a".into(),
                meaning: "Understand competing explanations".into(),
                attribution: "authored local fixture".into(),
            }],
            records: vec![],
            constraints: vec![],
            acceptance_references: vec![Hash::of(&[b"immutable"])],
        })?;
        let root = d.path().join("corpus");
        fs::create_dir(&root)?;
        fs::write(root.join("a.txt"), b"Fictional assertion; truth unknown")?;
        fs::write(
            root.join("b.txt"),
            b"Another fictional assertion; truth unknown",
        )?;
        e.qualitative_configure_retrieval(
            e.store.head_root()?,
            retrieval::Manifest {
                root: fs::canonicalize(root)?,
                records: vec![
                    retrieval::Record {
                        id: "a".into(),
                        path: "a.txt".into(),
                        digest: Hash::of(&[b"Fictional assertion; truth unknown"]),
                        attribution: "fiction".into(),
                        uncertainty: "unverified".into(),
                    },
                    retrieval::Record {
                        id: "b".into(),
                        path: "b.txt".into(),
                        digest: Hash::of(&[b"Another fictional assertion; truth unknown"]),
                        attribution: "fiction".into(),
                        uncertainty: "unverified".into(),
                    },
                ],
            },
        )?;
        Ok((d, e))
    }
    fn input(e: &Engine, origin: Origin) -> Result<Inquiry> {
        let q = e.qualitative_question_request()?;
        Ok(Inquiry {
            state_root: serde_json::from_value(q["state_root"].clone())?,
            attention: serde_json::from_value(q["inquiry_binding"].clone())?,
            parents: serde_json::from_value(q["native_attention"]["selected"].clone())?,
            question: "Which explanations are actually asserted?".into(),
            uncertainty: "Index does not establish content or truth".into(),
            attribution: "fixture author; not live model".into(),
            origin,
            exploration: None,
            answer_source: Some(AnswerSource::CorpusAssertions {
                corpus: serde_json::from_value(q["retrieval_contract"]["corpus"].clone())?,
                rationale: "Authored question concerns assertions, not world truth".into(),
            }),
        })
    }
    fn review(e: &Engine, id: EventId, decision: Decision) -> Result<()> {
        e.qualitative_review(Review {
            state_root: e.store.head_root()?,
            proposal: id,
            reviewer: "independent fixture review".into(),
            decision,
            reason: "Relevance only; no truth or action authority".into(),
        })?;
        Ok(())
    }
    fn choice(e: &Engine, id: EventId) -> Result<InformationChoice> {
        let q = e.qualitative_question_request()?;
        Ok(InformationChoice {
            inquiry: id,
            request: retrieval::Request {
                state_root: e.store.head_root()?,
                corpus: serde_json::from_value(q["retrieval_contract"]["corpus"].clone())?,
                ids: vec!["a".into()],
            },
            relevance: "May disclose an attributed explanation; not proof".into(),
        })
    }
    #[test]
    fn shared_attention_question_receipt_reflection_restart() -> Result<()> {
        let (d, e) = setup()?;
        let before = e.qualitative_question_request()?;
        assert_eq!(before, e.qualitative_question_request()?);
        assert_eq!(
            before["native_attention"]["kernel"],
            "lineage-half-retain@v1"
        );
        let i = input(&e, Origin::ModelProposed)?;
        let id = e.qualitative_propose_inquiry(i)?;
        assert!(e
            .qualitative_retrieve_for_question(choice(&e, id)?)
            .is_err());
        review(&e, id, Decision::Admit)?;
        let c = choice(&e, id)?;
        assert!(e.qualitative_retrieve(c.request.clone()).is_err());
        let receipt = e.qualitative_retrieve_for_question(c.clone())?;
        assert!(e.qualitative_retrieve_for_question(c).is_err());
        let q = e.qualitative_question_request()?;
        assert_eq!(q["basis"]["receipt"], json!(receipt));
        assert_eq!(
            q["retrieval_questions"][0]["inquiry"]["origin"],
            "model_proposed"
        );
        assert_eq!(q["retrieval_observations"][0]["inquiry"], json!(id));
        assert!(q["retrieval_observations"][0]["pre_read_relevance"].is_string());
        assert_eq!(
            e.public_meaning_snapshot()?["context"]["retrieval_questions"],
            q["retrieval_questions"]
        );
        assert_eq!(q["parents"], before["parents"]);
        assert_eq!(q["authority"], before["authority"]);
        // Authored reflection demonstrates the normal admission route, not model behavior.
        let sub = Submission {
            basis: serde_json::from_value(q["basis"].clone())?,
            generator: "fixture reflection".into(),
            response: Response {
                protocol: PROTOCOL.into(),
                state_root: e.store.head_root()?,
                attention_binding: serde_json::from_value(q["attention_binding"].clone())?,
                pursuit_expectations: Default::default(),
                common_event_forecasts: BTreeMap::new(),
                proposals: vec![Suggestion::NewCandidate {
                    description: "Retain uncertainty about assertion truth".into(),
                    serves: input(&e, Origin::ModelProposed)?
                        .parents
                        .iter()
                        .map(|p| {
                            serde_json::to_value(p)
                                .unwrap()
                                .as_str()
                                .unwrap()
                                .to_owned()
                        })
                        .collect(),
                    rationale: "Access observed; truth unverified".into(),
                }],
            },
        };
        let p = e.qualitative_submit_possibility(sub)?;
        review(&e, p, Decision::Admit)?;
        let final_q = e.qualitative_question_request()?;
        drop(e);
        let e = Engine::open(&d.path().join("state"))?;
        assert_eq!(final_q, e.qualitative_question_request()?);
        // Continuation wiring only: authored inquiry, not generated adaptive behavior.
        // The ordinary native entry point consumes retained context without a new bootstrap.
        assert_eq!(final_q["possibility_memory"].as_array().unwrap().len(), 2);
        let stale = input(&e, Origin::ModelProposed)?;
        let mut later = stale.clone();
        later.question = "Which recorded assumptions remain unverified?".into();
        later.uncertainty = "Prior access established assertions, not their truth".into();
        let next_id = e.qualitative_propose_inquiry(later)?;
        assert!(e.qualitative_propose_inquiry(stale).is_err());
        let pending = e.qualitative_question_request()?;
        assert_eq!(pending["phase"], "review");
        assert_eq!(pending["authority"], final_q["authority"]);
        assert_eq!(pending["parents"], final_q["parents"]);
        assert_eq!(
            pending["retrieval_observations"],
            final_q["retrieval_observations"]
        );
        review(&e, next_id, Decision::Admit)?;
        let continued = e.qualitative_question_request()?;
        assert_eq!(continued["possibility_memory"].as_array().unwrap().len(), 3);
        assert_eq!(
            continued["retrieval_observations"],
            final_q["retrieval_observations"]
        );
        assert_eq!(continued["parents"], final_q["parents"]);
        assert_eq!(continued["authority"], final_q["authority"]);
        assert_eq!(continued, e.qualitative_question_request()?);
        drop(e);
        let e = Engine::open(&d.path().join("state"))?;
        assert_eq!(continued, e.qualitative_question_request()?);
        Ok(())
    }
    #[test]
    fn inquiry_negative_context_review_withdrawal_and_origin() -> Result<()> {
        let (_d, e) = setup()?;
        let (_d2, e2) = setup()?;
        let mut i = input(&e, Origin::UserSupplied)?;
        i.attention = Hash::of(&[b"forged"]);
        assert!(e.qualitative_propose_inquiry(i).is_err());
        let i = input(&e, Origin::UserSupplied)?;
        assert!(e.qualitative_propose_inquiry(i.clone()).is_err());
        let id = e.qualitative_user_inquiry(i.clone())?;
        assert!(e.qualitative_propose_inquiry(i.clone()).is_err());
        review(&e, id, Decision::Unknown)?;
        assert!(e
            .qualitative_retrieve_for_question(choice(&e, id)?)
            .is_err());
        assert!(e2.current_inquiry(id).is_err());
        let id = e.qualitative_user_inquiry(input(&e, Origin::UserSupplied)?)?;
        review(&e, id, Decision::Admit)?;
        assert_eq!(e.current_inquiry(id)?.origin, Origin::UserSupplied);
        e.qualitative_status(Status {
            state_root: e.store.head_root()?,
            identity: i.parents[0],
            revision: None,
            withdrawn: true,
        })?;
        assert!(e.current_inquiry(id).is_err());
        assert!(
            e.qualitative_question_request()?["native_attention"]["selected"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        e.qualitative_status(Status {
            state_root: e.store.head_root()?,
            identity: i.parents[0],
            revision: None,
            withdrawn: false,
        })?;
        assert!(e.current_inquiry(id).is_err()); // reactivation cannot revive stale focus
        assert!(e.qualitative_request_v2()?["possibility_memory"]
            .as_array()
            .unwrap()
            .is_empty());
        Ok(())
    }
    #[test]
    fn changed_parent_meaning_invalidates_reviewed_question() -> Result<()> {
        use adaptive::coding::project::desire::{Change, Proposal};
        let (_d, e) = setup()?;
        let i = input(&e, Origin::ModelProposed)?;
        let id = e.qualitative_propose_inquiry(i.clone())?;
        review(&e, id, Decision::Admit)?;
        let q = e.qualitative_request_v2()?;
        let binding: Hash = serde_json::from_value(q["attention_binding"].clone())?;
        let sub = Submission {
            basis: serde_json::from_value(q["basis"].clone())?,
            generator: "authored development".into(),
            response: Response {
                protocol: PROTOCOL.into(),
                state_root: e.store.head_root()?,
                attention_binding: Some(binding),
                pursuit_expectations: Default::default(),
                common_event_forecasts: BTreeMap::new(),
                proposals: vec![Suggestion::PublicDesire {
                    proposal: Proposal {
                        binding,
                        identity: i.parents[0],
                        revises: None,
                        parents: i.parents.clone(),
                        meaning:
                            "Understand competing explanations while preserving their limitations"
                                .into(),
                        change: Change::Development,
                        basis: DesireBasis::Unknown,
                        generator: "authored development".into(),
                    },
                }],
            },
        };
        let p = e.qualitative_submit(sub)?;
        review(&e, p, Decision::Admit)?;
        assert!(e.current_inquiry(id).is_err());
        assert!(e.qualitative_question_request()?["possibility_memory"]
            .as_array()
            .unwrap()
            .is_empty());
        assert!(e
            .qualitative_retrieve_for_question(choice(&e, id)?)
            .is_err());
        Ok(())
    }
    #[test]
    fn answer_sources_wait_without_reads_and_survive_restart() -> Result<()> {
        let cases = vec![
            (
                Some(AnswerSource::UserPreference),
                "Which topic do you prefer?",
                "awaiting_user_preference",
            ),
            (
                Some(AnswerSource::UserAuthority),
                "May I change an external account?",
                "awaiting_user_authority",
            ),
            (
                Some(AnswerSource::ExternalEvidence),
                "Is the fictional assertion true in the world?",
                "unsupported_evidence_source",
            ),
            (
                Some(AnswerSource::Mixed),
                "What is asserted and which do you prefer?",
                "mixed_sources_unresolved",
            ),
            (
                Some(AnswerSource::Unknown),
                "What information would settle this?",
                "answer_source_unknown",
            ),
            (
                None,
                "Legacy question with no declared source",
                "answer_source_unknown",
            ),
        ];
        for (source, question, reason) in cases {
            let (d, e) = setup()?;
            let mut i = input(&e, Origin::ModelProposed)?;
            i.answer_source = source;
            i.question = question.into();
            let id = e.qualitative_propose_inquiry(i)?;
            review(&e, id, Decision::Admit)?;
            let expected = InquiryRoute::Waiting {
                reason: reason.into(),
            };
            assert_eq!(e.qualitative_inquiry_route(id)?, expected);
            let root = e.store.head_root()?;
            assert!(e
                .qualitative_retrieve_for_question(choice(&e, id)?)
                .is_err());
            assert_eq!(root, e.store.head_root()?);
            assert!(e.qualitative_request_v2()?["retrieval_observations"]
                .as_array()
                .unwrap()
                .is_empty());
            drop(e);
            let e = Engine::open(&d.path().join("state"))?;
            assert_eq!(e.qualitative_inquiry_route(id)?, expected);
        }
        Ok(())
    }
    #[test]
    fn answer_source_corpus_binding_and_review_are_required() -> Result<()> {
        let (_d, e) = setup()?;
        let mut i = input(&e, Origin::ModelProposed)?;
        i.answer_source = Some(AnswerSource::CorpusAssertions {
            corpus: Hash::of(&[b"wrong corpus"]),
            rationale: "authored".into(),
        });
        assert!(e.qualitative_propose_inquiry(i).is_err());
        let id = e.qualitative_propose_inquiry(input(&e, Origin::ModelProposed)?)?;
        assert!(e.qualitative_inquiry_route(id).is_err());
        review(&e, id, Decision::Reject)?;
        assert!(e.qualitative_inquiry_route(id).is_err());
        assert!(e
            .qualitative_retrieve_for_question(choice(&e, id)?)
            .is_err());
        Ok(())
    }
    fn explore(e: &Engine, policy: EventId) -> Result<Inquiry> {
        let mut i = input(e, Origin::EngineExploration)?;
        i.exploration = Some(Exploration {
            policy,
            considered_records: vec!["a".into(), "b".into()],
            possible_information_benefit:
                "Either attributed account may clarify what explanations are present".into(),
            why_no_user_priority_needed:
                "This is a bounded information sample, not a claim about user preference".into(),
        });
        Ok(i)
    }
    #[test]
    fn bounded_exploration_policy_off_no_benefit_and_user_authority() -> Result<()> {
        let (_d, e) = setup()?;
        assert_eq!(
            e.qualitative_question_request()?["exploration_policy"]["enabled"],
            false
        );
        assert!(e
            .qualitative_propose_inquiry(explore(
                &e,
                serde_json::from_value(json!(Hash::of(&[b"not policy"])))?
            )?)
            .is_err());
        let policy = e.qualitative_set_exploration_policy(ExplorationPolicy {
            state_root: e.store.head_root()?,
            enabled: true,
            max_records: 1,
        })?;
        let mut i = explore(&e, policy)?;
        i.exploration
            .as_mut()
            .unwrap()
            .possible_information_benefit
            .clear();
        assert!(e.qualitative_propose_inquiry(i).is_err());
        let mut i = explore(&e, policy)?;
        i.answer_source = Some(AnswerSource::UserAuthority);
        assert!(e.qualitative_propose_inquiry(i).is_err());
        let mut i = explore(&e, policy)?;
        i.answer_source = Some(AnswerSource::UserPreference);
        assert!(e.qualitative_propose_inquiry(i).is_err());
        let id = e.qualitative_propose_inquiry(explore(&e, policy)?)?;
        review(&e, id, Decision::Unknown)?; // nonempty benefit text isn't a proven benefit
        assert!(e
            .qualitative_retrieve_for_question(choice(&e, id)?)
            .is_err());
        let proposed = explore(&e, policy)?;
        let parent = proposed.parents[0];
        let id = e.qualitative_propose_inquiry(proposed)?;
        review(&e, id, Decision::Admit)?;
        e.qualitative_status(Status {
            state_root: e.store.head_root()?,
            identity: parent,
            revision: None,
            withdrawn: true,
        })?;
        assert!(e.qualitative_inquiry_route(id).is_err());
        assert!(e
            .qualitative_retrieve_for_question(choice(&e, id)?)
            .is_err());

        Ok(())
    }
    #[test]
    fn equally_plausible_exploration_preserves_choice_bounds_and_restart() -> Result<()> {
        let (d, e) = setup()?;
        let before = e.qualitative_request_v2()?;
        let p = ExplorationPolicy {
            state_root: e.store.head_root()?,
            enabled: true,
            max_records: 1,
        };
        let policy = e.qualitative_set_exploration_policy(p.clone())?;
        assert!(e.qualitative_set_exploration_policy(p).is_err());
        let id = e.qualitative_propose_inquiry(explore(&e, policy)?)?;
        review(&e, id, Decision::Admit)?;
        let q = e.qualitative_question_request()?;
        assert_eq!(q, e.qualitative_question_request()?);
        assert_eq!(q["parents"], before["parents"]);
        assert_eq!(q["authority"], before["authority"]);
        let mut c = choice(&e, id)?;
        c.request.ids = vec!["a".into(), "b".into()];
        assert!(e.qualitative_retrieve_for_question(c).is_err());
        // Neither option is prescribed by code; this authored test chooses the second.
        let mut c = choice(&e, id)?;
        c.request.ids = vec!["b".into()];
        let receipt = e.qualitative_retrieve_for_question(c.clone())?;
        assert!(e.qualitative_retrieve_for_question(c).is_err());
        let saved = e.qualitative_question_request()?;
        drop(e);
        let e = Engine::open(&d.path().join("state"))?;
        assert_eq!(saved, e.qualitative_question_request()?);
        assert_eq!(saved["basis"]["receipt"], json!(receipt));
        e.qualitative_set_exploration_policy(ExplorationPolicy {
            state_root: e.store.head_root()?,
            enabled: false,
            max_records: 1,
        })?;
        assert!(e.qualitative_inquiry_route(id).is_err());
        assert!(e
            .qualitative_retrieve_for_question(choice(&e, id)?)
            .is_err());
        assert!(e.qualitative_request_v2()?["possibility_memory"]
            .as_array()
            .unwrap()
            .is_empty());
        Ok(())
    }
    #[test]
    fn information_forecast_receipt_residual_restart_and_unknown() -> Result<()> {
        let (d, e) = setup()?;
        let id = e.qualitative_user_inquiry(input(&e, Origin::UserSupplied)?)?;
        review(&e, id, Decision::Admit)?;
        let before = e.qualitative_question_request()?;
        let c = choice(&e, id)?;
        let receipt = e.qualitative_forecasted_retrieve(super::information::Input {
            choice: c.clone(),
            probability: 0.2,
        })?;
        let after = e.qualitative_question_request()?;
        assert_eq!(before["parents"], after["parents"]);
        assert_eq!(before["authority"], after["authority"]);
        assert_eq!(after["information_expectation"]["resolved"], 1);
        assert_eq!(
            e.public_meaning_context()?["information_expectation"],
            after["information_expectation"]
        );
        assert_eq!(
            after["information_expectation"]["history"][0]["residual"],
            -0.8
        );
        assert_eq!(
            after["information_expectation"]["history"][0]["receipt"],
            json!(receipt)
        );
        assert!(e
            .qualitative_forecasted_retrieve(super::information::Input {
                choice: c,
                probability: 0.8
            })
            .is_err());
        let restart = Engine::open(&d.path().join("state"))?;
        assert_eq!(restart.qualitative_question_request()?, after);
        let second = choice(&restart, id)?;
        fs::write(d.path().join("corpus/a.txt"), b"altered")?;
        assert!(restart
            .qualitative_forecasted_retrieve(super::information::Input {
                choice: second,
                probability: 0.9
            })
            .is_err());
        let view = restart.qualitative_information_expectation()?;
        assert_eq!(view["unresolved"], 1);
        assert_eq!(view["history"][1]["residual"], Value::Null);
        assert!(restart
            .qualitative_retrieve_for_question(choice(&restart, id)?)
            .is_err());
        let attempt: EventId = serde_json::from_value(view["history"][1]["attempt"].clone())?;
        assert!(restart
            .qualitative_resolve_information(attempt, receipt)
            .is_err());
        Ok(())
    }
    #[test]
    fn information_crash_receipt_settlement_and_cross_root() -> Result<()> {
        let (d, e) = setup()?;
        let id = e.qualitative_user_inquiry(input(&e, Origin::UserSupplied)?)?;
        review(&e, id, Decision::Admit)?;
        let c = choice(&e, id)?;
        let (_, foreign) = setup()?;
        assert!(foreign
            .qualitative_forecasted_retrieve(super::information::Input {
                choice: c.clone(),
                probability: 0.5
            })
            .is_err());
        assert!(e
            .qualitative_forecasted_retrieve(super::information::Input {
                choice: c.clone(),
                probability: 2.0
            })
            .is_err());
        let root = e.store.head_root()?;
        for variant in 0..4 {
            let mut bad = c.clone();
            match variant {
                0 => bad.relevance = "".into(),
                1 => bad.relevance = "x".repeat(513),
                2 => bad.request.ids = vec!["missing".into()],
                _ => bad.request.corpus = Hash::of(&[b"foreign corpus"]),
            }
            assert!(e
                .qualitative_forecasted_retrieve(super::information::Input {
                    choice: bad,
                    probability: 0.5
                })
                .is_err());
            assert_eq!(root, e.store.head_root()?);
        }
        // Fixture injects the exact durable pre-I/O boundary; no model or campaign work.
        let attempt = e.write(|| {
            e.qualitative_append(
                Entry::InformationAttempt {
                    attempt: super::information::Attempt {
                        choice: c.clone(),
                        forecast: Forecast::new(
                            OutcomeDefId(Hash::of(&[b"validated_information_receipt@v1"])),
                            Horizon::ThisDispatch,
                            Forecaster::Operator,
                            Probability::new(0.8)?,
                            e.events()?.len() as u64,
                        ),
                    },
                },
                Source::Operator,
            )
        })?;
        assert_ne!(root, e.store.head_root()?);
        let restart = Engine::open(&d.path().join("state"))?;
        assert!(restart
            .qualitative_retrieve_for_question(choice(&restart, id)?)
            .is_err());
        let mut req = c.request.clone();
        req.state_root = restart.store.head_root()?;
        let receipt = restart.qualitative_retrieve_with_attempt(
            req,
            Some(id),
            Some(c.relevance),
            Some(attempt),
        )?;
        // Receipt persisted but process stopped before settlement: resolve without rereading.
        let resumed = Engine::open(&d.path().join("state"))?;
        assert_eq!(
            resumed.qualitative_information_expectation()?["unresolved"],
            1
        );
        assert!(resumed
            .qualitative_resolve_information(attempt, EventId(Hash::of(&[b"wrong"])))
            .is_err());
        let settled = resumed.qualitative_resolve_information(attempt, receipt)?;
        assert_eq!(settled["resolved"], 1);
        assert!(resumed
            .qualitative_resolve_information(attempt, receipt)
            .is_err());
        let new_choice = choice(&resumed, id)?;
        resumed.qualitative_forecasted_retrieve(super::information::Input {
            choice: new_choice,
            probability: 0.9,
        })?;
        assert_eq!(
            resumed.qualitative_information_expectation()?["resolved"],
            2
        );
        Ok(())
    }
}
