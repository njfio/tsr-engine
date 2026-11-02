//! Qualitative pursuit expectations, frozen before action and revisable only from
//! attributed returns plus separate review. Neither probabilities nor desire utility.
use super::*;
use crate::engine::adaptive::provider::{Response, Suggestion};
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Expectation {
    pub parents: Vec<PreferenceId>,
    pub anticipated: String,
    pub grounds: String,
    pub uncertainty: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frozen {
    pub proposal: EventId,
    pub index: usize,
    pub commitment: EventId,
    pub action: String,
    pub source_before: Hash,
    pub expectation: Expectation,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Conclusion {
    Supported,
    Contradicted,
    Unknown,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Reflection {
    pub binding: Hash,
    pub expectation: EventId,
    pub receipt: EventId,
    pub source_after: Hash,
    pub conclusion: Conclusion,
    pub observed: String,
    /// None retains the frozen interpretation; no novelty is required.
    pub revised_expectation: Option<String>,
    pub uncertainty: String,
    /// What remains wanted, distinct from whether the prediction was accurate.
    pub contrast: String,
    pub possibilities: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub state_root: Hash,
    pub proposal: EventId,
    pub reviewer: String,
    pub generator: String,
    pub note: String,
    pub decision: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Apply {
    pub state_root: Hash,
    pub reflection: Reflection,
    pub reviewer: String,
    pub generator: String,
    pub note: String,
    pub decision: String,
}
impl Engine {
    pub(crate) fn validate_pursuit_expectations(&self, response: &Response) -> Result<()> {
        if response.pursuit_expectations.is_empty() {
            return Ok(());
        }
        let config = self.project_configuration()?;
        ensure!(
            config.browser,
            "pursuit expectation profile not enabled for this project"
        );
        let context = self.project_relational_context()?;
        let parents = context["inputs"]["parents"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("public desire parents missing"))?;
        for (key, x) in &response.pursuit_expectations {
            let index = key.parse::<usize>()?;
            ensure!(
                index.to_string() == *key && index < response.proposals.len(),
                "pursuit expectation index"
            );
            ensure!(
                matches!(
                    response.proposals[index],
                    Suggestion::EditSet { .. }
                        | Suggestion::ExactEdit { .. }
                        | Suggestion::Existing { .. }
                ),
                "expectation requires executable possibility"
            );
            let distinct: BTreeSet<_> = x.parents.iter().collect();
            ensure!(
                !distinct.is_empty()
                    && distinct.len() == x.parents.len()
                    && x.parents
                        .iter()
                        .all(|id| parents.iter().any(|p| p["id"] == json!(id))),
                "expectation contributor lineage"
            );
            for text in [&x.anticipated, &x.grounds, &x.uncertainty] {
                ensure!(
                    browser::narrative(text),
                    "nonblank pursuit expectation text"
                );
            }
        }
        Ok(())
    }
    pub(crate) fn freeze_pursuit_expectation(
        &self,
        response: &Response,
        proposal: EventId,
        pending: &Pending,
    ) -> Result<()> {
        let mut selected = None;
        for (key, x) in &response.pursuit_expectations {
            let index = key.parse::<usize>()?;
            let action = match &response.proposals[index] {
                Suggestion::EditSet { patches, .. } => self.validate_patch_set(patches)?,
                Suggestion::ExactEdit { patch, .. } => patch.id()?,
                Suggestion::Existing { action, .. } => action.clone(),
                _ => anyhow::bail!("nonexecutable expectation"),
            };
            if action == pending.action {
                ensure!(selected.is_none(), "ambiguous selected expectation");
                selected = Some((index, x));
            }
        }
        if let Some((index, expectation)) = selected {
            let source_before = digest(&self.project_snapshot(&self.project_configuration()?)?)?;
            self.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::PursuitFrozen {
                        frozen: Frozen {
                            proposal,
                            index,
                            commitment: pending.commitment,
                            action: pending.action.clone(),
                            source_before,
                            expectation: expectation.clone(),
                        },
                    },
                },
                Source::Rules,
            )?;
        }
        Ok(())
    }
    pub fn project_pursuit_context(&self) -> Result<Value> {
        let records = self.records()?;
        let mut history = vec![];
        for (id, r) in &records {
            let Record::Coding {
                record: CodingRecord::PursuitFrozen { frozen },
            } = r
            else {
                continue;
            };
            self.require_project_bridge_source(*id, Source::Rules)?;
            let proposal = records
                .iter()
                .find_map(|(pid, p)| match p {
                    Record::ProposalReceived { response } if *pid == frozen.proposal => {
                        Some(response)
                    }
                    _ => None,
                })
                .ok_or_else(|| anyhow::anyhow!("frozen expectation proposal missing"))?;
            self.require_project_bridge_source(frozen.proposal, Source::Generator)?;
            ensure!(
                serde_json::to_value(proposal.pursuit_expectations.get(&frozen.index.to_string()))?
                    == serde_json::to_value(Some(&frozen.expectation))?,
                "frozen expectation content mismatch"
            );
            let action = match proposal.proposals.get(frozen.index) {
                Some(Suggestion::EditSet { patches, .. }) => {
                    let mut sorted = patches.clone();
                    sorted.sort_by(|a, b| a.path.cmp(&b.path));
                    format!(
                        "patch-{}",
                        Hash::of(&[b"scoped-edit-set@v1", &serde_json::to_vec(&sorted)?])
                    )
                }
                Some(Suggestion::ExactEdit { patch, .. }) => patch.id()?,
                Some(Suggestion::Existing { action, .. }) => action.clone(),
                _ => anyhow::bail!("frozen expectation candidate missing"),
            };
            ensure!(action==frozen.action && records.iter().any(|(_,r)|matches!(r,Record::Pending{action,commitment,..} if *commitment==frozen.commitment && *action==frozen.action)),"frozen expectation commitment mismatch");
            for (_, r) in &records {
                if let Record::Coding {
                    record: CodingRecord::Intent { plan },
                } = r
                {
                    if plan.commitment == frozen.commitment {
                        ensure!(
                            plan.action == frozen.action
                                && plan.source_before == frozen.source_before,
                            "frozen expectation execution source mismatch"
                        );
                    }
                }
            }
            let returned=records.iter().find_map(|(rid,r)|match r{Record::Coding{record:CodingRecord::Receipt{commitment,verdict,source_after,observations,..}} if *commitment==frozen.commitment=>Some(json!({"receipt":rid,"source_after":source_after,"verdict":verdict,"observations":observations})),_=>None});
            if let Some(value) = &returned {
                let receipt: EventId = serde_json::from_value(value["receipt"].clone())?;
                self.require_project_bridge_source(receipt, Source::Rules)?;
                let intents: Vec<_> = records
                    .iter()
                    .filter_map(|(id, r)| match r {
                        Record::Coding {
                            record: CodingRecord::Intent { plan },
                        } if plan.commitment == frozen.commitment => Some((*id, plan)),
                        _ => None,
                    })
                    .collect();
                ensure!(
                    intents.len() == 1,
                    "expectation return requires one execution intent"
                );
                self.require_project_bridge_source(intents[0].0, Source::Rules)?;
                ensure!(
                    intents[0].1.source_before == frozen.source_before
                        && intents[0].1.action == frozen.action,
                    "expectation receipt lineage mismatch"
                );
            }
            let mut reflections = vec![];
            for (rid, r) in &records {
                if let Record::Coding {
                    record:
                        CodingRecord::PursuitReflectionAdmitted { review }
                        | CodingRecord::PursuitReflectionReviewedV2 { review },
                } = r
                {
                    let p = records
                        .iter()
                        .find_map(|(pid, p)| match p {
                            Record::Coding {
                                record: CodingRecord::PursuitReflectionReceived { reflection },
                            } if *pid == review.proposal => Some(reflection),
                            _ => None,
                        })
                        .ok_or_else(|| anyhow::anyhow!("reflection proposal missing"))?;
                    if p.expectation == *id {
                        let council = matches!(
                            r,
                            Record::Coding {
                                record: CodingRecord::PursuitReflectionReviewedV2 { .. }
                            }
                        );
                        self.require_project_bridge_source(
                            *rid,
                            if council {
                                Source::Council
                            } else {
                                Source::Operator
                            },
                        )?;
                        self.require_project_bridge_source(review.proposal, Source::Generator)?;
                        ensure!(
                            review.reviewer != review.generator
                                && matches!(
                                    review.decision.as_str(),
                                    "accept" | "reject" | "unknown" | "abstain"
                                ),
                            "reflection review attribution"
                        );
                        ensure!(
                            returned
                                .as_ref()
                                .is_some_and(|v| v["receipt"] == json!(p.receipt)
                                    && v["source_after"] == json!(p.source_after)),
                            "reflection receipt/source mismatch"
                        );
                        let mut item = json!({"admission":rid,"proposal":review.proposal,"reflection":p,"reviewer":review.reviewer,"generator":review.generator,"decision":review.decision,"review_note":review.note});
                        if council {
                            item["review_source"] = json!("council");
                            item["review_version"] = json!("independent-pursuit-review@v2");
                        }
                        reflections.push(item);
                    }
                }
            }
            history.push(json!({"id":id,"frozen":frozen,"return":returned,"reviewed_reflections":reflections.iter().filter(|v|v["decision"]=="accept").collect::<Vec<_>>(),"reflection_reviews":reflections}));
        }
        Ok(
            json!({"version":"qualitative-pursuit@v1","history":history,"limits":"Frozen generated expectations and independently reviewed interpretations; not calibrated probabilities, desire utilities, user acceptance, or proof of causal learning. Contrast remains distinct; contributors and their history persist."}),
        )
    }
    pub fn project_pursuit_reflection_request(&self) -> Result<Value> {
        let context = self.project_pursuit_context()?;
        let snapshot = digest(&self.project_snapshot(&self.project_configuration()?)?)?;
        let pending = context["history"]
            .as_array()
            .unwrap()
            .iter()
            .rev()
            .find(|h| {
                !h["return"].is_null()
                    && h["return"]["source_after"] == json!(snapshot)
                    && h["reflection_reviews"]
                        .as_array()
                        .is_some_and(|a| a.is_empty())
            });
        let state_root = self.store.head_root()?;
        let binding = Hash::of(&[&serde_json::to_vec(
            &json!({"state_root":state_root,"snapshot":snapshot,"pending":pending}),
        )?]);
        Ok(
            json!({"binding":binding,"state_root":state_root,"snapshot":snapshot,"pending":pending,"history":context}),
        )
    }
    pub fn project_pursuit_reflection_apply(&self, input: Apply) -> Result<Value> {
        self.write(|| {
            ensure!(self.store.head_root()?==input.state_root && self.adaptive_state()?.pending.is_none(),"stale reflection or pending action");
            ensure!(input.generator!=input.reviewer && short(&input.generator)&&short(&input.reviewer)&&browser::narrative(&input.note)&&matches!(input.decision.as_str(),"accept"|"reject"|"unknown"|"abstain"),"independent reflection assessment required");
            self.validate_pursuit_reflection(&input.reflection)?;
            let proposal=self.adaptive_record(&Record::Coding{record:CodingRecord::PursuitReflectionReceived{reflection:input.reflection}},Source::Generator)?;
            let review=Review{state_root:input.state_root,proposal,reviewer:input.reviewer,generator:input.generator,note:input.note,decision:input.decision};
            let admission=self.adaptive_record(&Record::Coding{record:CodingRecord::PursuitReflectionReviewedV2{review}},Source::Council)?;
            Ok(json!({"proposal":proposal,"assessment":admission,"history":self.project_pursuit_context()?}))
        })
    }
    fn validate_pursuit_reflection(&self, reflection: &Reflection) -> Result<()> {
        ensure!(
            self.adaptive_state()?.pending.is_none(),
            "reflection cannot retroactively inform a pending action"
        );
        let request = self.project_pursuit_reflection_request()?;
        ensure!(
            request["binding"] == json!(reflection.binding)
                && request["pending"]["id"] == json!(reflection.expectation)
                && request["pending"]["return"]["receipt"] == json!(reflection.receipt)
                && request["snapshot"] == json!(reflection.source_after),
            "stale/cross-source pursuit reflection"
        );
        for text in [
            &reflection.observed,
            &reflection.uncertainty,
            &reflection.contrast,
        ] {
            ensure!(browser::narrative(text), "nonblank reflection text");
        }
        if let Some(text) = &reflection.revised_expectation {
            ensure!(browser::narrative(text), "nonblank revised expectation");
        }
        ensure!(
            reflection.possibilities.len() <= 4
                && reflection
                    .possibilities
                    .iter()
                    .all(|p| browser::narrative(p)),
            "reflection possibilities"
        );
        Ok(())
    }
    pub fn project_pursuit_reflection_propose(&self, reflection: Reflection) -> Result<EventId> {
        self.write(|| {
            ensure!(
                self.adaptive_state()?.pending.is_none(),
                "reflection cannot retroactively inform a pending action"
            );
            let request = self.project_pursuit_reflection_request()?;
            ensure!(
                request["binding"] == json!(reflection.binding)
                    && request["pending"]["id"] == json!(reflection.expectation)
                    && request["pending"]["return"]["receipt"] == json!(reflection.receipt)
                    && request["snapshot"] == json!(reflection.source_after),
                "stale/cross-source pursuit reflection"
            );
            for text in [
                &reflection.observed,
                &reflection.uncertainty,
                &reflection.contrast,
            ] {
                ensure!(browser::narrative(text), "nonblank reflection text");
            }
            if let Some(text) = &reflection.revised_expectation {
                ensure!(browser::narrative(text), "nonblank revised expectation");
            }
            ensure!(
                reflection.possibilities.len() <= 4
                    && reflection
                        .possibilities
                        .iter()
                        .all(|p| browser::narrative(p)),
                "reflection possibilities"
            );
            self.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::PursuitReflectionReceived { reflection },
                },
                Source::Generator,
            )
        })
    }
    pub fn project_pursuit_reflection_admit(&self, review: Review) -> Result<EventId> {
        self.write(|| {
            ensure!(
                self.adaptive_state()?.pending.is_none(),
                "reflection cannot retroactively inform a pending action"
            );
            ensure!(
                self.store.head_root()? == review.state_root
                    && review.generator != review.reviewer
                    && short(&review.generator)
                    && short(&review.reviewer)
                    && browser::narrative(&review.note),
                "independent source-bound reflection review required"
            );
            ensure!(
                matches!(
                    review.decision.as_str(),
                    "accept" | "reject" | "unknown" | "abstain"
                ),
                "reflection decision"
            );
            let records = self.records()?;
            let p = records
                .iter()
                .find_map(|(id, r)| match r {
                    Record::Coding {
                        record: CodingRecord::PursuitReflectionReceived { reflection },
                    } if *id == review.proposal => Some(reflection),
                    _ => None,
                })
                .ok_or_else(|| anyhow::anyhow!("reflection proposal missing"))?;
            self.require_project_bridge_source(review.proposal, Source::Generator)?;
            let now = self.project_pursuit_reflection_request()?;
            ensure!(
                now["pending"]["id"] == json!(p.expectation)
                    && now["pending"]["return"]["receipt"] == json!(p.receipt)
                    && now["snapshot"] == json!(p.source_after),
                "reflection return changed or already reviewed"
            );
            self.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::PursuitReflectionReviewedV2 { review },
                },
                Source::Council,
            )
        })
    }
}
