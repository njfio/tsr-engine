//! Qualitative, generated relational hypotheses. Never scored obligations or authority.
use super::*;
use anyhow::anyhow;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Development {
    pub original_revision: Hash,
    pub prior_revision: Hash,
    pub retained_meaning: String,
    pub return_reveals: String,
    pub unresolved: String,
    pub proposed_development: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Proposal {
    pub parents: Vec<PreferenceId>,
    pub relationship: String,
    pub aspiration: String,
    pub discriminator: String,
    pub revises: Option<Hash>,
    #[serde(default)]
    pub catalysts: Vec<Hash>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub development: Option<Development>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Submission {
    pub state_root: Hash,
    pub return_commitment: EventId,
    pub observation_revision: Hash,
    pub evidence_key: Hash,
    pub proposals: Vec<Proposal>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StatusUpdate {
    pub state_root: Hash,
    pub identity: Hash,
    pub withdrawn: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Revision {
    pub identity: Hash,
    pub revision: Hash,
    pub proposal: Proposal,
    pub return_commitment: EventId,
    pub observation_revision: Hash,
    pub evidence_key: Hash,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Entry {
    Considered {
        evidence_key: Hash,
        revisions: Vec<Revision>,
    },
    Status {
        identity: Hash,
        withdrawn: bool,
    },
}
fn history(records: &[Record]) -> (Vec<Revision>, BTreeMap<Hash, bool>, BTreeSet<Hash>) {
    let mut revisions = vec![];
    let mut status = BTreeMap::new();
    let mut considered = BTreeSet::new();
    for r in records {
        if let Record::Relational { entry } = r {
            match entry {
                Entry::Considered {
                    evidence_key,
                    revisions: batch,
                } => {
                    considered.insert(*evidence_key);
                    revisions.extend(batch.clone());
                }
                Entry::Status {
                    identity,
                    withdrawn,
                } => {
                    status.insert(*identity, *withdrawn);
                }
            }
        }
    }
    (revisions, status, considered)
}
impl Engine {
    pub(super) fn relational_view(
        &self,
        spec: &Spec,
        ids: &[PreferenceId],
        turns: &[Turn],
        active: &[usize],
    ) -> Result<serde_json::Value> {
        let (revisions, status, considered) = history(&self.deliberation_records()?);
        let revision = observation_revision(spec);
        let originals: Vec<_> = active
            .iter()
            .copied()
            .filter(|i| original(spec, *i) == *i)
            .map(|i| ids[i])
            .collect();
        let last = turns.last();
        let key = Hash::of(&[&serde_json::to_vec(&(
            last.map(|t| t.commitment),
            revision,
            &originals,
            &status,
        ))?]);
        let observed_return = last.map(|t| serde_json::json!({
            "commitment":t.commitment,"observation_revision":t.observation_revision,
            "chosen":t.offered.get(t.chosen).map(|a| &a.direction.action),
            "before":ids.iter().zip(&t.before).map(|(id,v)|(id.to_string(),v)).collect::<BTreeMap<_,_>>(),
            "after":ids.iter().zip(&t.after).map(|(id,v)|(id.to_string(),v)).collect::<BTreeMap<_,_>>(),
            "contrast":t.contrast.iter().filter_map(|i|ids.get(*i)).collect::<Vec<_>>(),
            "observed_progress":t.observed_progress,
            "measurement":"Adapter predicates only; no semantic discriminator result"
        }));
        let mut latest = BTreeMap::new();
        for r in &revisions {
            latest.insert(r.identity, r);
        }
        let has_contrast = last.is_some_and(|t| t.contrast.iter().any(|i| active.contains(i)));
        // An observed return can close every mechanical predicate without answering
        // a qualitative hypothesis. Review is not new contrast-backed expansion.
        let reviewable: Vec<_> = latest
            .values()
            .filter(|r| {
                !status.get(&r.identity).copied().unwrap_or(false)
                    && r.observation_revision == revision
                    && r.proposal.parents.iter().all(|p| originals.contains(p))
                    && last.is_some_and(|t| t.commitment != r.return_commitment)
                    && turns.iter().any(|t| t.commitment == r.return_commitment)
            })
            .map(|r| r.revision)
            .collect();
        let due = last.is_some_and(|t| t.observation_revision == Some(revision))
            && (has_contrast || !reviewable.is_empty())
            && originals.len() >= 2
            && !considered.contains(&key);
        let mut guidance = vec![];
        let mut existing = vec![];
        for r in latest.values() {
            let withdrawn = status.get(&r.identity).copied().unwrap_or(false);
            let applicable = !withdrawn
                && last.is_some_and(|t| t.commitment == r.return_commitment)
                && r.observation_revision == revision
                && r.proposal.parents.iter().all(|p| originals.contains(p));
            let subsequent: Vec<_> = turns
                .iter()
                .skip_while(|t| t.commitment != r.return_commitment)
                .skip(1)
                .collect();
            let regressed = subsequent.iter().any(|t| {
                t.observation_revision == Some(revision)
                    && r.proposal.parents.iter().any(|p| {
                        ids.iter().position(|id| id == p).is_some_and(|i| {
                            t.before
                                .get(i)
                                .zip(t.after.get(i))
                                .is_some_and(|(b, a)| a < b)
                        })
                    })
            });
            let origin = revisions
                .iter()
                .find(|x| x.identity == r.identity)
                .expect("lineage origin");
            let origin_view = serde_json::json!({"revision":origin.revision,"parents":origin.proposal.parents,
                "relationship":origin.proposal.relationship,"aspiration":origin.proposal.aspiration,
                "discriminator":origin.proposal.discriminator,"role":"historical generated meaning, revisable and not authority"});
            let value = serde_json::json!({"identity":r.identity,"revision":r.revision,"origin":origin_view,"development_status":if r.proposal.development.is_some(){"separated_claims_semantically_unverified"}else{"legacy_unseparated_semantically_unverified"},"proposal":r.proposal,"return_commitment":r.return_commitment,"observation_revision":r.observation_revision,"withdrawn":withdrawn,"applicable":applicable,"source":"generated_hypothesis_structurally_checked_only","record_provenance":"Rules receipt of generated proposal; not operator admission","semantic_status":"unknown","discriminator_tested":"unknown","feedback":if regressed {"parent_regression"}else if !subsequent.is_empty(){"needs_review"}else{"awaiting_return"},"catalyst_scope":"historical hypotheses only; references grant no current validity or authority","subsequent_returns":subsequent.iter().map(|t|t.commitment).collect::<Vec<_>>()});
            if applicable {
                guidance.push(value.clone());
            }
            existing.push(value);
        }
        let text_schema = serde_json::json!({"type":"string","minLength":1,"maxLength":512});
        let reference_ids: Vec<_> = latest
            .values()
            .filter(|r| !status.get(&r.identity).copied().unwrap_or(false))
            .map(|r| r.revision)
            .collect();
        let catalyst_ids: Vec<_> = latest
            .values()
            .filter(|r| {
                !status.get(&r.identity).copied().unwrap_or(false)
                    && r.observation_revision == revision
                    && r.proposal.parents.iter().all(|p| originals.contains(p))
            })
            .map(|r| r.revision)
            .collect();
        let ref_schema = if !has_contrast {
            if reviewable.is_empty() {
                serde_json::json!({"type":"null"})
            } else {
                serde_json::json!({"type":"string","enum":reviewable})
            }
        } else if reference_ids.is_empty() {
            serde_json::json!({"type":"null"})
        } else {
            serde_json::json!({"anyOf":[{"type":"null"},{"type":"string","enum":reference_ids}]})
        };
        let development_schema = serde_json::json!({"anyOf":[{"type":"null"},{"type":"object","additionalProperties":false,
            "required":["original_revision","prior_revision","retained_meaning","return_reveals","unresolved","proposed_development"],
            "properties":{"original_revision":{"type":"string","enum":revisions.iter().filter(|r|r.proposal.revises.is_none()).map(|r|r.revision).collect::<Vec<_>>()},
                "prior_revision":{"type":"string","enum":reference_ids},"retained_meaning":text_schema,"return_reveals":text_schema,"unresolved":text_schema,
                "proposed_development":{"anyOf":[{"type":"null"},text_schema.clone()]}}}]});
        let mut response_schema = serde_json::json!({"type":"object","additionalProperties":false,"required":["state_root","return_commitment","observation_revision","evidence_key","proposals"],"properties":{
        "state_root":{"type":"string","enum":[self.store.head_root()?]},"return_commitment":{"type":"string","enum":[last.map(|t|t.commitment)]},"observation_revision":{"type":"string","enum":[revision]},"evidence_key":{"type":"string","enum":[key]},
        "proposals":{"type":"array","maxItems":2,"items":{"type":"object","additionalProperties":false,"required":["parents","relationship","aspiration","discriminator","revises","catalysts","development"],"properties":{
            "parents":{"type":"array","minItems":2,"maxItems":16,"uniqueItems":true,"items":{"type":"string","enum":originals}},"relationship":text_schema,"aspiration":text_schema,"discriminator":text_schema,"revises":ref_schema,
            "development":development_schema,"catalysts":{"type":"array","maxItems":if catalyst_ids.is_empty(){0}else{4},"uniqueItems":true,"items":{"type":"string","enum":catalyst_ids}}
        }}}}});
        // Model-facing branches express new versus revised meaning explicitly.
        // Historical Proposal deserialization remains additive and compatible.
        let base = response_schema["properties"]["proposals"]["items"].clone();
        let mut variants = vec![];
        if has_contrast {
            let mut new = base.clone();
            new["properties"]["revises"] = serde_json::json!({"type":"null"});
            new["properties"]["development"] = serde_json::json!({"type":"null"});
            variants.push(new);
        }
        let allowed = if has_contrast {
            &reference_ids
        } else {
            &reviewable
        };
        if !allowed.is_empty() {
            let mut revised = base.clone();
            revised["properties"]["revises"] = serde_json::json!({"type":"string","enum":allowed});
            revised["properties"]["development"] = development_schema["anyOf"][1].clone();
            variants.push(revised);
        }
        if variants.is_empty() {
            response_schema["properties"]["proposals"]["maxItems"] = serde_json::json!(0);
        } else {
            response_schema["properties"]["proposals"]["items"] =
                serde_json::json!({"anyOf":variants});
        }
        Ok(
            serde_json::json!({"review_only":!has_contrast,"reviewable_revisions":reviewable,"observed_return":observed_return,"response_schema":response_schema,"due":due,"state_root":self.store.head_root()?,"return_commitment":last.map(|t|t.commitment),"observation_revision":revision,"evidence_key":key,"active_originals":originals,"existing":existing,"guidance":guidance,"history":revisions,"meaning":"Qualitative generated hypotheses; semantic novelty and fulfillment unknown. No independent weight, acceptance, or execution authority."}),
        )
    }
    pub fn submit_relational(&self, mut response: Submission) -> Result<serde_json::Value> {
        self.write(|| {
            ensure!(
                response.state_root == self.store.head_root()?,
                "stale relational response"
            );
            let request = self.deliberation_request()?;
            let view = &request["relational_request"];
            ensure!(
                view["due"] == true,
                "no unreviewed current contrast or existing hypothesis return"
            );
            ensure!(
                serde_json::to_value(response.return_commitment)? == view["return_commitment"]
                    && serde_json::to_value(response.observation_revision)?
                        == view["observation_revision"]
                    && serde_json::to_value(response.evidence_key)? == view["evidence_key"],
                "stale relational evidence"
            );
            ensure!(
                response.proposals.len() <= 2,
                "at most two relational proposals"
            );
            let (spec, ids, _, _) = self.deliberation_state()?;
            let active: Vec<PreferenceId> =
                serde_json::from_value(view["active_originals"].clone())?;
            let (all, status, _) = history(&self.deliberation_records()?);
            ensure!(
                all.len() + response.proposals.len() <= 128,
                "relational history capacity exhausted"
            );
            let mut batch = vec![];
            let mut signatures = BTreeSet::new();
            let mut revised = BTreeSet::new();
            for p in &mut response.proposals {
                if view["review_only"] == true {
                    ensure!(
                        p.revises.is_some_and(|id| view["reviewable_revisions"]
                            .as_array()
                            .is_some_and(|ids| ids.contains(&serde_json::json!(id)))),
                        "without contrast only existing return-bound hypotheses may be revised"
                    );
                }
                ensure!(
                    [&p.relationship, &p.aspiration, &p.discriminator]
                        .iter()
                        .all(|s| bounded(s)),
                    "bounded nonempty relational text required"
                );
                ensure!(
                    p.parents.len() >= 2 && p.parents.len() <= 16,
                    "bounded multi-parent relation"
                );
                let mut canonical = BTreeSet::new();
                for id in &p.parents {
                    let i = ids
                        .iter()
                        .position(|v| v == id)
                        .ok_or_else(|| anyhow!("unknown relational parent"))?;
                    canonical.insert(ids[original(&spec, i)]);
                }
                ensure!(
                    canonical.len() >= 2 && canonical.iter().all(|p| active.contains(p)),
                    "distinct active original parents required"
                );
                p.parents = canonical.into_iter().collect();
                ensure!(p.catalysts.len() <= 4, "bounded catalysts");
                p.catalysts.sort();
                p.catalysts.dedup();
                for id in &p.catalysts {
                    ensure!(
                        all.iter().any(|r| r.revision == *id
                            && all
                                .iter()
                                .rev()
                                .find(|x| x.identity == r.identity)
                                .map(|x| x.revision)
                                == Some(*id)
                            && r.observation_revision == response.observation_revision
                            && r.proposal.parents.iter().all(|p| active.contains(p))
                            && !status.get(&r.identity).copied().unwrap_or(false)),
                        "unknown or withdrawn catalyst; no forward cycles"
                    );
                }
                let signature = serde_json::to_string(&(
                    &p.parents,
                    p.relationship.trim(),
                    p.aspiration.trim(),
                    p.discriminator.trim(),
                ))?;
                ensure!(
                    signatures.insert(signature.clone())
                        && !all.iter().any(|r| !p.revises.is_some_and(|id| all
                            .iter()
                            .any(|old| old.revision == id && old.identity == r.identity))
                            && serde_json::to_string(&(
                                &r.proposal.parents,
                                r.proposal.relationship.trim(),
                                r.proposal.aspiration.trim(),
                                r.proposal.discriminator.trim()
                            ))
                            .ok()
                            .as_ref()
                                == Some(&signature)),
                    "duplicate relational hypothesis"
                );
                let prior = if let Some(id) = p.revises {
                    let r = all
                        .iter()
                        .find(|r| r.revision == id)
                        .ok_or_else(|| anyhow!("missing revision ancestor"))?;
                    ensure!(
                        all.iter()
                            .rev()
                            .find(|x| x.identity == r.identity)
                            .map(|x| x.revision)
                            == Some(id)
                            && r.proposal.parents == p.parents
                            && !status.get(&r.identity).copied().unwrap_or(false)
                            && revised.insert(r.identity),
                        "stale, withdrawn or cross-lineage revision"
                    );
                    Some(r.identity)
                } else {
                    None
                };
                if let Some(development) = &p.development {
                    let ancestor = all.iter().find(|r| Some(r.revision) == p.revises)
                        .ok_or_else(|| anyhow!("separated development requires revision ancestor"))?;
                    let origin = all.iter().find(|r| r.identity == ancestor.identity)
                        .ok_or_else(|| anyhow!("missing relational origin"))?;
                    ensure!(development.original_revision == origin.revision
                        && development.prior_revision == ancestor.revision, "development lineage mismatch");
                    ensure!([&development.retained_meaning,&development.return_reveals,&development.unresolved]
                        .iter().all(|s|bounded(s)) && development.proposed_development.as_ref().is_none_or(|s|bounded(s)),
                        "bounded separated development text required");
                    if development.proposed_development.is_none() {
                        ensure!(p.relationship == ancestor.proposal.relationship && p.aspiration == ancestor.proposal.aspiration
                            && p.discriminator == ancestor.proposal.discriminator,
                            "no new development must retain prior meaning fields; record return/uncertainty separately");
                    }
                }
                let hash = Hash::of(&[&serde_json::to_vec(&(response.evidence_key, &p))?]);
                batch.push(Revision {
                    identity: prior.unwrap_or(hash),
                    revision: hash,
                    proposal: p.clone(),
                    return_commitment: response.return_commitment,
                    observation_revision: response.observation_revision,
                    evidence_key: response.evidence_key,
                });
            }
            self.deliberation_record(&Record::Relational {
                entry: Entry::Considered {
                    evidence_key: response.evidence_key,
                    revisions: batch,
                },
            })?;
            self.deliberation_request()
        })
    }
    pub fn relational_status(&self, update: StatusUpdate) -> Result<serde_json::Value> {
        self.write(|| {
            ensure!(
                update.state_root == self.store.head_root()?,
                "stale relational status"
            );
            let (revisions, statuses, _) = history(&self.deliberation_records()?);
            ensure!(
                revisions.iter().any(|r| r.identity == update.identity),
                "unknown relational identity"
            );
            // Missing status means active. An unchanged effective status is not new evidence.
            // Validate the root above even for a no-op, so stale requests remain rejected.
            if statuses.get(&update.identity).copied().unwrap_or(false) == update.withdrawn {
                return self.deliberation_request();
            }
            self.deliberation_record(&Record::Relational {
                entry: Entry::Status {
                    identity: update.identity,
                    withdrawn: update.withdrawn,
                },
            })?;
            self.deliberation_request()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> Result<(tempfile::TempDir, Engine)> {
        let dir = tempfile::tempdir()?;
        let engine = Engine::open(dir.path())?;
        engine.deliberation_initialize(Spec {
            archive: None,
            wants: vec![
                Want {
                    text: "Understand".into(),
                    aim: Aim::Understand { topic: "x".into() },
                    parents: vec![],
                },
                Want {
                    text: "Account".into(),
                    aim: Aim::Account { topic: "x".into() },
                    parents: vec![],
                },
            ],
            notes: vec![
                Note {
                    id: "a".into(),
                    topic: "x".into(),
                    text: "alpha".into(),
                },
                Note {
                    id: "b".into(),
                    topic: "x".into(),
                    text: "beta".into(),
                },
            ],
            max_cycles: 8,
            attention_limit: 2,
        })?;
        returned(&engine)?;
        Ok((dir, engine))
    }
    fn returned(e: &Engine) -> Result<()> {
        let request = e.deliberation_request()?;
        let known: Vec<String> = serde_json::from_value(request["notebook"]["known"].clone())?;
        let notes = if known.is_empty() {
            vec!["a".to_string()]
        } else {
            vec!["a".to_string(), "b".to_string()]
        };
        // One authentic controller-dispatched inspection, with alternative grouping.
        let d = |notes: Vec<String>| Direction {
            deferred_observation: None,
            continuation: None,
            purpose: "Read admitted evidence".into(),
            supports: vec![0],
            anticipated: vec![1.0, 0.0],
            action: Move::Inspect { notes },
        };
        e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![d(notes), d(vec!["b".into()])],
        })?;
        Ok(())
    }
    fn response(e: &Engine) -> Result<Submission> {
        let q = e.deliberation_request()?;
        let v = &q["relational_request"];
        Ok(Submission {
            state_root: e.store.head_root()?,
            return_commitment: serde_json::from_value(v["return_commitment"].clone())?,
            observation_revision: serde_json::from_value(v["observation_revision"].clone())?,
            evidence_key: serde_json::from_value(v["evidence_key"].clone())?,
            proposals: vec![Proposal {
                parents: serde_json::from_value(v["active_originals"].clone())?,
                relationship:
                    "Understanding informs the account; the account exposes gaps in understanding"
                        .into(),
                aspiration: "Make explanatory gaps visible while relating the evidence".into(),
                discriminator:
                    "Compare a later account with the evidence and retain unknown interpretation"
                        .into(),
                revises: None,
                catalysts: vec![],
                development: None,
            }],
        })
    }
    #[test]
    fn separated_development_preserves_history_and_does_not_certify_meaning() -> Result<()> {
        for mode in ["no_insight", "richer_claim", "measurement_only_claim"] {
            let (dir, e) = setup()?;
            let original = response(&e)?;
            let legacy = serde_json::to_value(&original.proposals[0])?;
            assert!(
                legacy.get("development").is_none(),
                "legacy identity serialization unchanged"
            );
            let first = e.submit_relational(original.clone())?;
            let old: Hash =
                serde_json::from_value(first["relational_guidance"][0]["revision"].clone())?;
            returned(&e)?;
            let before = e.deliberation_request()?;
            let mut next = response(&e)?;
            let p = &mut next.proposals[0];
            p.revises = Some(old);
            p.development = Some(Development {
                original_revision: old,
                prior_revision: old,
                retained_meaning:
                    "FIXTURE: understanding and explanation remain related; meaning is unverified"
                        .into(),
                return_reveals: "FIXTURE: another bounded inspection occurred, not semantic proof"
                    .into(),
                unresolved: "FIXTURE: contrary interpretations remain possible".into(),
                proposed_development: None,
            });
            if mode != "no_insight" {
                p.aspiration = if mode == "richer_claim" {
                    "FIXTURE: expose how an explanation might invite revision of understanding"
                } else {
                    "FIXTURE: maximize the number of inspected notes"
                }
                .into();
                p.development.as_mut().unwrap().proposed_development = Some(
                    "FIXTURE: proposed interpretation; quality not mechanically established".into(),
                );
            }
            let after = e.submit_relational(next)?;
            assert_eq!(
                after["relational_request"]["history"][0],
                first["relational_request"]["history"][0]
            );
            assert_eq!(
                after["relational_guidance"][0]["origin"]["revision"],
                serde_json::json!(old)
            );
            assert_eq!(
                after["relational_guidance"][0]["semantic_status"],
                "unknown"
            );
            assert_eq!(
                after["relational_guidance"][0]["discriminator_tested"],
                "unknown"
            );
            for key in [
                "standing_desires",
                "authority",
                "cycles",
                "observation",
                "archive",
            ] {
                assert_eq!(before[key], after[key]);
            }
            assert_eq!(Engine::open(dir.path())?.deliberation_request()?, after);
        }
        Ok(())
    }
    #[test]
    fn null_development_cannot_silently_replace_meaning_or_misbind_lineage() -> Result<()> {
        let (_dir, e) = setup()?;
        let first = e.submit_relational(response(&e)?)?;
        let old: Hash =
            serde_json::from_value(first["relational_guidance"][0]["revision"].clone())?;
        returned(&e)?;
        let before = e.deliberation_request()?;
        let mut next = response(&e)?;
        let p = &mut next.proposals[0];
        p.revises = Some(old);
        p.development = Some(Development {
            original_revision: old,
            prior_revision: old,
            retained_meaning: "FIXTURE: prior relationship retained".into(),
            return_reveals: "FIXTURE: inspection counted".into(),
            unresolved: "FIXTURE: meaning unknown".into(),
            proposed_development: None,
        });
        let valid = next.clone();
        next.proposals[0].aspiration = "FIXTURE: inspection fraction is one".into();
        assert!(e.submit_relational(next).is_err());
        assert_eq!(e.deliberation_request()?, before);
        let mut wrong = valid.clone();
        wrong.proposals[0]
            .development
            .as_mut()
            .unwrap()
            .original_revision = Hash::of(&[b"other-origin"]);
        assert!(e.submit_relational(wrong).is_err());
        assert_eq!(e.deliberation_request()?, before);
        e.submit_relational(valid)?;
        Ok(())
    }
    #[test]
    fn relational_loop_preserves_wants_and_exposes_return_revision_history() -> Result<()> {
        let (dir, e) = setup()?;
        let before = e.deliberation_request()?;
        assert_eq!(before["relational_request"]["due"], true);
        let observed = &before["relational_request"]["observed_return"];
        assert_eq!(
            observed["commitment"],
            before["relational_request"]["return_commitment"]
        );
        assert_eq!(observed["chosen"]["operation"], "inspect");
        assert!(observed["before"].is_object());
        assert!(!observed["contrast"].as_array().unwrap().is_empty());
        let offered = response(&e)?;
        let after = e.submit_relational(offered.clone())?;
        for key in [
            "standing_desires",
            "obligation_ids",
            "observation",
            "notebook",
            "authority",
            "cycles",
        ] {
            assert_eq!(before[key], after[key], "{key}");
        }
        assert_eq!(after["relational_request"]["due"], false);
        assert_eq!(after["relational_guidance"].as_array().unwrap().len(), 1);
        assert!(e.submit_relational(offered).is_err());
        let old: Hash =
            serde_json::from_value(after["relational_guidance"][0]["revision"].clone())?;
        returned(&e)?;
        let current = e.deliberation_request()?;
        assert_eq!(
            current["relational_request"]["existing"][0]["feedback"],
            "needs_review"
        );
        assert_eq!(
            current["relational_request"]["existing"][0]["discriminator_tested"],
            "unknown"
        );
        let mut revise = response(&e)?;
        revise.proposals[0].revises = Some(old);
        revise.proposals[0].aspiration =
            "Use newly inspected evidence to distinguish remaining explanatory gaps".into();
        let revised = e.submit_relational(revise)?;
        assert_eq!(
            revised["relational_request"]["history"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            revised["relational_guidance"][0]["identity"],
            after["relational_guidance"][0]["identity"]
        );
        assert_ne!(
            revised["relational_guidance"][0]["revision"],
            after["relational_guidance"][0]["revision"]
        );
        assert_eq!(Engine::open(dir.path())?.deliberation_request()?, revised);
        Ok(())
    }
    #[test]
    fn successful_return_reviews_existing_hypothesis_once_without_expansion() -> Result<()> {
        for empty in [false, true] {
            let (dir, e) = setup()?;
            returned(&e)?;
            let prior = e.submit_relational(response(&e)?)?;
            let old = serde_json::from_value(prior["relational_guidance"][0]["revision"].clone())?;
            let direction = |claims: Vec<String>, anticipated| Direction {
                purpose: "Fixture: measure inclusion, not semantic fulfillment".into(),
                supports: vec![1],
                anticipated: vec![1.0, anticipated],
                action: Move::Compose {
                    title: "Fixture account".into(),
                    claims,
                },
                continuation: None,
                deferred_observation: None,
            };
            e.deliberate(Offer {
                state_root: e.store.head_root()?,
                directions: vec![
                    direction(vec!["a".into(), "b".into()], 1.0),
                    direction(vec!["a".into()], 0.5),
                ],
            })?;
            let reopened = Engine::open(dir.path())?;
            let q = reopened.deliberation_request()?;
            assert_eq!(q["status"], "satisfied");
            assert_eq!(
                q["relational_request"]["observed_return"]["contrast"],
                serde_json::json!([])
            );
            assert_eq!(q["relational_request"]["due"], true);
            assert_eq!(q["relational_request"]["review_only"], true);
            assert_eq!(
                q["relational_request"]["existing"][0]["discriminator_tested"],
                "unknown"
            );
            assert!(q["relational_guidance"].as_array().unwrap().is_empty());
            let root = reopened.store.head_root()?;
            assert!(reopened.submit_relational(response(&reopened)?).is_err());
            assert_eq!(reopened.store.head_root()?, root);
            let mut review = response(&reopened)?;
            review.proposals[0].revises = Some(old);
            if empty {
                review.proposals.clear();
            }
            let after = reopened.submit_relational(review.clone())?;
            assert_eq!(after["relational_request"]["due"], false);
            assert_eq!(after["status"], "satisfied");
            for key in [
                "standing_desires",
                "authority",
                "notebook",
                "observation",
                "cycles",
            ] {
                assert_eq!(q[key], after[key], "{key}");
            }
            assert!(reopened.submit_relational(review).is_err());
            assert_eq!(Engine::open(dir.path())?.deliberation_request()?, after);
        }
        Ok(())
    }

    #[test]
    fn relational_duplicates_cycles_withdrawal_and_empty_review_are_bounded() -> Result<()> {
        let (_dir, e) = setup()?;
        let root = e.store.head_root()?;
        let mut bad = response(&e)?;
        bad.proposals[0].parents = vec![bad.proposals[0].parents[0]; 2];
        assert!(e.submit_relational(bad).is_err());
        assert_eq!(e.store.head_root()?, root);
        let mut missing = response(&e)?;
        missing.proposals[0].revises = Some(Hash::of(&[b"absent revision"]));
        assert!(e.submit_relational(missing).is_err());
        let mut bad = response(&e)?;
        bad.proposals[0].catalysts = vec![Hash::of(&[b"future"])];
        assert!(e.submit_relational(bad).is_err());
        let mut bad = response(&e)?;
        bad.proposals.push(bad.proposals[0].clone());
        assert!(e.submit_relational(bad).is_err());
        assert_eq!(e.store.head_root()?, root);
        let view = e.submit_relational(response(&e)?)?;
        let identity = serde_json::from_value(view["relational_guidance"][0]["identity"].clone())?;
        let withdrawn = e.relational_status(StatusUpdate {
            state_root: e.store.head_root()?,
            identity,
            withdrawn: true,
        })?;
        assert!(withdrawn["relational_guidance"]
            .as_array()
            .unwrap()
            .is_empty());
        let mut empty = response(&e)?;
        empty.proposals.clear();
        let considered = e.submit_relational(empty)?;
        assert_eq!(considered["relational_request"]["due"], false);
        let mut stale = response(&e)?;
        stale.state_root = root;
        assert!(e.submit_relational(stale).is_err());
        Ok(())
    }
    #[test]
    fn relational_aliases_stale_observation_and_parent_withdrawal() -> Result<()> {
        let (_dir, e) = setup()?;
        let (_, ids, _, turns) = e.deliberation_state()?;
        let alias = Preference::new(
            "Observe a subset",
            Scope::Task(Engine::task()),
            vec![ids[0]],
            Source::Generator,
        )?
        .id;
        e.deliberation_record(&Record::Refined {
            from: turns.last().unwrap().commitment,
            admitted: vec![(
                Want {
                    text: "Observe a subset".into(),
                    aim: Aim::InspectNotes {
                        notes: vec!["a".into()],
                    },
                    parents: vec![0],
                },
                alias,
            )],
            rejected: vec![],
        })?;
        let mut duplicate = response(&e)?;
        duplicate.proposals[0].parents = vec![ids[0], alias];
        let root = e.store.head_root()?;
        assert!(e.submit_relational(duplicate).is_err());
        assert_eq!(e.store.head_root()?, root);
        let mut valid = response(&e)?;
        valid.proposals[0].parents = vec![alias, ids[1]];
        let view = e.submit_relational(valid)?;
        let canonical: Vec<PreferenceId> =
            serde_json::from_value(view["relational_guidance"][0]["proposal"]["parents"].clone())?;
        assert_eq!(
            canonical.into_iter().collect::<BTreeSet<_>>(),
            ids.into_iter().collect()
        );
        let before = view["standing_desires"].clone();
        let changed = e.update_deliberation_observation(ObservationUpdate {
            state_root: e.store.head_root()?,
            reason: "Observed corrected admitted note".into(),
            notes: vec![Note {
                id: "a".into(),
                topic: "x".into(),
                text: "new alpha".into(),
            }],
        })?;
        assert!(changed["relational_guidance"]
            .as_array()
            .unwrap()
            .is_empty());
        assert_eq!(changed["standing_desires"], before);
        assert_eq!(changed["relational_request"]["due"], false);
        let (_, ids, _, _) = e.deliberation_state()?;
        e.update_desire_status(DesireStatusUpdate {
            state_root: e.store.head_root()?,
            preference: ids[0],
            status: PreferenceStatus::Withdrawn,
        })?;
        assert!(e.deliberation_request()?["relational_guidance"]
            .as_array()
            .unwrap()
            .is_empty());
        Ok(())
    }
    #[test]
    fn relational_parent_regression_is_not_semantic_disproof() -> Result<()> {
        let (_dir, e) = setup()?;
        let admitted = e.submit_relational(response(&e)?)?;
        let revision: Hash =
            serde_json::from_value(admitted["relational_guidance"][0]["revision"].clone())?;
        // Explicit synthetic observed-return fixture: arithmetic predicate loss,
        // not an invented semantic discriminator result or model execution.
        let mut turn = e.deliberation_state()?.3.last().unwrap().clone();
        turn.commitment = EventId(Hash::of(&[b"controlled-relational-parent-loss"]));
        turn.before = vec![1.0, 1.0];
        turn.after = vec![1.0, 0.0];
        turn.contrast = vec![1];
        e.deliberation_record(&Record::Returned {
            turn: Box::new(turn),
        })?;
        let view = e.deliberation_request()?;
        assert!(view["relational_guidance"].as_array().unwrap().is_empty());
        assert_eq!(
            view["relational_request"]["existing"][0]["feedback"],
            "parent_regression"
        );
        assert_eq!(
            view["relational_request"]["existing"][0]["semantic_status"],
            "unknown"
        );
        let mut revision_response = response(&e)?;
        revision_response.proposals[0].revises = Some(revision);
        revision_response.proposals[0].development = Some(Development {
            original_revision: revision,
            prior_revision: revision,
            retained_meaning:
                "FIXTURE: prior interpretation is questioned, not protected from contradiction"
                    .into(),
            return_reveals: "FIXTURE: the observed account predicate regressed".into(),
            unresolved: "FIXTURE: predicate loss is not semantic disproof".into(),
            proposed_development: Some(
                "FIXTURE: reconsider whether the earlier account exposed gaps sufficiently".into(),
            ),
        });
        revision_response.proposals[0].discriminator =
            "Reassess the account loss without claiming semantic contradiction".into();
        let revised = e.submit_relational(revision_response)?;
        assert_eq!(
            revised["relational_guidance"][0]["return_commitment"],
            view["relational_request"]["return_commitment"]
        );
        assert_eq!(
            revised["relational_request"]["history"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        Ok(())
    }
    #[test]
    fn relational_status_noops_preserve_evidence_and_restart() -> Result<()> {
        let (dir, e) = setup()?;
        let initial = e.submit_relational(response(&e)?)?;
        let identity =
            serde_json::from_value(initial["relational_guidance"][0]["identity"].clone())?;
        let same = e.relational_status(StatusUpdate {
            state_root: e.store.head_root()?,
            identity,
            withdrawn: false,
        })?;
        assert_eq!(
            same, initial,
            "implicit active no-op must not reopen consideration"
        );
        let withdrawn = e.relational_status(StatusUpdate {
            state_root: e.store.head_root()?,
            identity,
            withdrawn: true,
        })?;
        let reopened = Engine::open(dir.path())?;
        assert_eq!(
            reopened.relational_status(StatusUpdate {
                state_root: reopened.store.head_root()?,
                identity,
                withdrawn: true
            })?,
            withdrawn
        );
        assert!(reopened
            .relational_status(StatusUpdate {
                state_root: serde_json::from_value(initial["state_root"].clone())?,
                identity,
                withdrawn: true
            })
            .is_err());
        let restored = reopened.relational_status(StatusUpdate {
            state_root: reopened.store.head_root()?,
            identity,
            withdrawn: false,
        })?;
        assert_eq!(
            reopened.relational_status(StatusUpdate {
                state_root: reopened.store.head_root()?,
                identity,
                withdrawn: false
            })?,
            restored
        );
        Ok(())
    }
    #[test]
    fn relational_batch_is_atomic_and_old_revision_stays_stale_after_restart() -> Result<()> {
        let (dir, e) = setup()?;
        let before = e.deliberation_request()?;
        let mut bad = response(&e)?;
        let mut second = bad.proposals[0].clone();
        second.aspiration = "different proposal".into();
        second.parents = vec![second.parents[0]];
        bad.proposals.push(second);
        assert!(e.submit_relational(bad).is_err());
        assert_eq!(e.deliberation_request()?, before);
        let first = e.submit_relational(response(&e)?)?;
        let old = serde_json::from_value(first["relational_guidance"][0]["revision"].clone())?;
        returned(&e)?;
        let mut revision = response(&e)?;
        revision.proposals[0].revises = Some(old);
        let second = e.submit_relational(revision)?;
        let latest = serde_json::from_value(second["relational_guidance"][0]["revision"].clone())?;
        let reopened = Engine::open(dir.path())?;
        // Operator withdrawal/restoration yields a fresh contrast consideration without inventing a return.
        let identity =
            serde_json::from_value(second["relational_guidance"][0]["identity"].clone())?;
        reopened.relational_status(StatusUpdate {
            state_root: reopened.store.head_root()?,
            identity,
            withdrawn: true,
        })?;
        reopened.relational_status(StatusUpdate {
            state_root: reopened.store.head_root()?,
            identity,
            withdrawn: false,
        })?;
        let mut stale = response(&reopened)?;
        stale.proposals[0].revises = Some(old);
        let root = reopened.store.head_root()?;
        assert!(reopened.submit_relational(stale).is_err());
        assert_eq!(reopened.store.head_root()?, root);
        let mut valid = response(&reopened)?;
        valid.proposals[0].revises = Some(latest);
        assert_eq!(
            reopened.submit_relational(valid)?["relational_request"]["history"]
                .as_array()
                .unwrap()
                .len(),
            3
        );
        Ok(())
    }
    /// Explicit fixture construction only. No execution, operational labels or inference.
    #[test]
    #[ignore = "offline synthetic benchmark fixture builder; requires explicit paths"]
    fn build_synthetic_relational_benchmark_fixture() -> Result<()> {
        let spec_path = std::env::var("TSR_RELATIONAL_BENCH_SPEC")?;
        let destination = std::path::PathBuf::from(std::env::var("TSR_RELATIONAL_BENCH_RUN")?);
        ensure!(
            !destination.exists(),
            "fresh synthetic destination required"
        );
        let spec: Spec = serde_json::from_slice(&std::fs::read(spec_path)?)?;
        ensure!(
            spec.wants.len() == 2
                && spec
                    .wants
                    .iter()
                    .all(|w| w.text.starts_with("SYNTHETIC BENCHMARK:")),
            "explicit synthetic desire labels required"
        );
        let e = Engine::open(&destination)?;
        e.deliberation_initialize(spec.clone())?;
        let mut book = Notebook::default();
        book.known.extend(spec.notes.iter().map(|n| n.id.clone()));
        let turn: Turn = serde_json::from_value(serde_json::json!({
            "observation_revision":observation_revision(&spec),
            "offered":[{"direction":{"purpose":"SYNTHETIC fictional inspect; never executed", "supports":[0,1],"anticipated":[0.0,0.0],"action":{"operation":"inspect","notes":["synthetic-fictional-inspect-NEVER-EXECUTED"]}},"prediction":{"name":"synthetic","value":0.0},"projected":[0.0,0.0],"score":{"name":"synthetic","value":0.0},"continuity_cost":0.0}],"chosen":0,"selected":[],"before":[0.0,0.0],"after":[0.0,0.0],
            "observed_progress":false,"notebook":book,
            "commitment":EventId(Hash::of(&[b"synthetic-unexecuted-benchmark-return", &serde_json::to_vec(&spec)?])),
            "residual":0.0,"contrast":[0,1]
        }))?;
        e.deliberation_record(&Record::Returned {
            turn: Box::new(turn),
        })?;
        let q = e.deliberation_request()?;
        ensure!(
            q["relational_request"]["due"] == true,
            "fixture must expose relational request"
        );
        ensure!(
            q["relational_request"]["observed_return"]["chosen"]["notes"][0]
                == "synthetic-fictional-inspect-NEVER-EXECUTED",
            "fictional entry must disclose no execution"
        );
        std::fs::write(
            destination.join("SYNTHETIC-NOT-OPERATIONAL.json"),
            serde_json::to_vec_pretty(&serde_json::json!({"synthetic":true,"actions_executed":0,
                "semantic_measurements":false,"training_eligible":false,"spec":spec,"initial_root":q["state_root"]}))?,
        )?;
        std::fs::write(
            destination.join("benchmark-initial-request.json"),
            serde_json::to_vec_pretty(&q)?,
        )?;
        Ok(())
    }
    #[test]
    fn relational_status_rejects_generated_provenance() -> Result<()> {
        let (_dir, e) = setup()?;
        let view = e.submit_relational(response(&e)?)?;
        let identity = serde_json::from_value(view["relational_guidance"][0]["identity"].clone())?;
        let record = Record::Relational {
            entry: Entry::Status {
                identity,
                withdrawn: true,
            },
        };
        let digest = e.blob(&serde_json::to_vec(&record)?)?;
        e.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    digest,
                    source: SOURCE.into(),
                },
            },
            Source::Generator,
        )?;
        assert!(e
            .deliberation_request()
            .unwrap_err()
            .to_string()
            .contains("relational provenance"));
        Ok(())
    }
}
