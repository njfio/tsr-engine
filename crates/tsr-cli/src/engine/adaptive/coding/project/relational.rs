//! Project bridge for qualitative hypotheses and independently reviewed experiments.
//! Generated proposals stay in ProposalReceived; only operator admission publishes guidance.
use super::*;
use crate::engine::adaptive::provider::Suggestion;
use crate::engine::deliberation::relational::Proposal;
use acceptance::reviewer::supplemental::{Bundle, Mapping};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Experiment {
    pub binding: Hash,
    pub snapshot: Hash,
    pub requirements_revision: EventId,
    pub harness: BTreeMap<String, String>,
    pub cases: Vec<ProbeCase>,
    pub mapping: BTreeMap<String, Mapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExperimentReview {
    pub author: String,
    pub reviewer: String,
    pub generator: String,
    pub review: String,
}

impl Engine {
    pub fn project_latest_proposal(&self) -> Result<Option<EventId>> {
        Ok(self
            .records()?
            .into_iter()
            .rev()
            .find_map(|(id, r)| matches!(r, Record::ProposalReceived { .. }).then_some(id)))
    }
    pub(crate) fn require_project_bridge_source(&self, id: EventId, source: Source) -> Result<()> {
        ensure!(
            self.events()?
                .iter()
                .any(|(_, e)| e.id == id && e.source == source),
            "project bridge provenance mismatch"
        );
        Ok(())
    }
    fn project_relational_inputs(&self) -> Result<serde_json::Value> {
        let current = self.project_acceptance_status()?;
        let cycle = self.acceptance_cycle()?;
        let aims = self.acceptance_current_aims()?;
        // Never expose the private requirements/review. Only published intent is eligible.
        let parents: Vec<_> = cycle["guidance"]
            .as_object()
            .into_iter()
            .flatten()
            .filter_map(|(key, text)| {
                aims.get(key)
                    .map(|id| serde_json::json!({"id":id,"requirement":key,"meaning":text}))
            })
            .collect();
        let mut returns = vec![];
        for (id, r) in self.records()?.into_iter().rev() {
            if let Record::Coding {
                record:
                    CodingRecord::Receipt {
                        commitment,
                        verdict,
                        source_after,
                        observations,
                        ..
                    },
            } = r
            {
                if source_after == Some(current.snapshot) {
                    self.require_project_bridge_source(id, Source::Rules)?;
                    if returns.len() < 2 {
                        returns.push(serde_json::json!({"receipt":id,"snapshot":source_after,"commitment":commitment,"verdict":verdict,"observations":observations}));
                    }
                }
            }
        }
        let allowed: BTreeSet<_> = parents
            .iter()
            .filter_map(|p| p["requirement"].as_str())
            .collect();
        // A semantic return can contradict passing tests without changing source or a
        // tool receipt. Bind its identity and only already-published dimensions; private
        // review prose, unpublished criteria and aggregate counts never enter guidance.
        if let Some(review) = current.decision_record {
            self.require_project_bridge_source(review, Source::Operator)?;
        }
        let visible_dimensions = |key: &str| {
            cycle[key]
                .as_array()
                .into_iter()
                .flatten()
                .filter_map(|value| value.as_str())
                .filter(|id| allowed.contains(id))
                .map(str::to_owned)
                .collect::<Vec<_>>()
        };
        let semantic_return = serde_json::json!({
            "review":current.decision_record,
            "applicable":cycle["review_evidence_applicable"],
            "unmet":visible_dimensions("unmet"),
            "unknown":visible_dimensions("unknown"),
            "meaning":"Independent semantic judgment, not a new tool observation or action permission; private review text is not exported"
        });
        let supplemental_view = self.project_supplemental_view()?;
        let origins = self.project_experiment_origins(&supplemental_view)?;
        let supplemental = supplemental_view["current_observations"].as_array().into_iter().flatten().filter_map(|o| {
            let mapped:BTreeMap<_,_> = o["mapping"].as_object().into_iter().flatten()
                .filter(|(k,_)|allowed.contains(k.as_str())).map(|(k,v)|(k.clone(),v["cases"].clone())).collect();
            if mapped.is_empty() { return None; }
            let ids:BTreeSet<_> = mapped.values().flat_map(|v|v.as_array().into_iter().flatten()).filter_map(|v|v.as_str()).collect();
            let cases:BTreeMap<_,_> = o["cases"].as_object().into_iter().flatten().filter(|(k,_)|ids.contains(k.as_str())).map(|(k,v)|(k.clone(),v.clone())).collect();
            let findings:BTreeMap<_,_> = o["findings"].as_object().into_iter().flatten().filter(|(k,_)|mapped.contains_key(*k)).map(|(k,v)|(k.clone(),v.clone())).collect();
            let mut projected=serde_json::json!({"generated_origin":origins.get(&o["admission"].to_string()),"admission":o["admission"],"receipt":o["receipt"],"snapshot":o["snapshot"],"checked_snapshot":o["checked_snapshot"],"source_episode_root":o["source_episode_root"],"bundle_digest":o["bundle_digest"],"mapped_cases":mapped,"cases":cases,"findings":findings,"limits":"Only finite check outcomes; no universal proof or private review text"});
            if let Some(raw)=o["browser_cases"].as_array() {
                projected["browser_cases"]=serde_json::json!(raw.iter().filter(|c|c["case_id"].as_str().is_some_and(|id|ids.contains(id))).collect::<Vec<_>>());
            }
            Some(projected)
        }).collect::<Vec<_>>();
        let mut inputs = serde_json::json!({"version":"project-relational@v1","snapshot":current.snapshot,"requirements_revision":current.requirements_revision,"parents":parents,"returns":returns,"supplemental":supplemental});
        // Preserve the existing no-review wire shape and its historical bindings.
        if current.decision_record.is_some() {
            inputs["version"] = serde_json::json!("project-relational@v2");
            inputs["semantic_return"] = semantic_return;
        }
        let initial_public = self.project_public_desires(&parents)?;
        let eligible_roots: BTreeSet<_> = Ledger::new(&self.store)
            .collective()?
            .entries
            .iter()
            .filter(|e| {
                e.preference.origin == Source::Operator
                    && e.preference.scope == Scope::Task(Self::task())
                    && matches!(
                        e.status,
                        Some(PreferenceStatus::Open | PreferenceStatus::Fulfilled)
                    )
            })
            .map(|e| e.preference.id)
            .collect();
        let retained_roots: Vec<_> = initial_public
            .as_ref()
            .map(|p| {
                p["current"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|p| {
                        p.get("original_contributors").is_none()
                            && serde_json::from_value::<PreferenceId>(p["id"].clone())
                                .is_ok_and(|id| eligible_roots.contains(&id))
                    })
                    .cloned()
                    .collect()
            })
            .unwrap_or_else(|| {
                parents
                    .iter()
                    .filter(|p| {
                        serde_json::from_value::<PreferenceId>(p["id"].clone())
                            .is_ok_and(|id| eligible_roots.contains(&id))
                    })
                    .cloned()
                    .collect()
            });
        let mut generated = if self.project_configuration()?.browser {
            self.project_generated_preferences(&retained_roots)?
        } else {
            vec![]
        };
        let mut combined = retained_roots;
        combined.extend(generated.iter().filter(|p| p["eligible"] == true).cloned());
        // Retained root wanting survives a new publication. Fold the complete
        // root/child dependency graph, never a root-only active intermediate.
        let public = if generated.is_empty() {
            initial_public
        } else {
            self.project_public_desires(&combined)?
        };
        if let Some(public) = public {
            inputs["version"] = serde_json::json!("project-relational@v3");
            inputs["acceptance_parents"] = serde_json::json!(parents);
            inputs["parents"] = public["active"].clone();
            inputs["public_desires"] = public;
            let ids: BTreeSet<PreferenceId> = inputs["parents"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| serde_json::from_value(p["id"].clone()))
                .collect::<std::result::Result<_, _>>()?;
            for p in &mut generated {
                let id: PreferenceId = serde_json::from_value(p["id"].clone())?;
                p["eligible"] = serde_json::json!(p["eligible"] == true && ids.contains(&id));
            }
        } else {
            inputs["parents"] = serde_json::json!(combined);
        }
        if self.project_configuration()?.browser {
            if !generated.is_empty() {
                if inputs.get("acceptance_parents").is_none() {
                    inputs["acceptance_parents"] = serde_json::json!(parents);
                }
                let ledger = Ledger::new(&self.store).collective()?;
                inputs["generated_preference_statuses"] = serde_json::json!(generated
                    .iter()
                    .map(|p| {
                        let id: PreferenceId = serde_json::from_value(p["id"].clone())
                            .expect("native generated identity");
                        let status = ledger
                            .get(&id)
                            .and_then(|entry| entry.status.as_ref())
                            .map(tsr_ledger::status_name)
                            .unwrap_or_else(|| "not_admitted".into());
                        serde_json::json!({"id":id,"status":status})
                    })
                    .collect::<Vec<_>>());
                inputs["generated_preferences"] = serde_json::json!(generated);
            }
            if let Some((id, basis)) = self.project_current_success()? {
                inputs["current_success"] = serde_json::json!({"id":id,"basis":basis,"semantics":"Scoped artifact success only; human fulfillment and new preference fulfillment remain unknown; no authority"});
            }
            if let Some(contrast) = self.project_current_contrast()? {
                inputs["current_contrast"] = serde_json::json!({"id":contrast.id,"review":contrast.from_return,"semantics":"Current authenticated unmet published criteria; not surprise, unknown or a new permission"});
            }
        }
        let information = self.project_information_receipts()?;
        if !information.is_empty() {
            inputs["information_returns"] = serde_json::json!(information);
        }
        let service = self.project_objective_receipts()?;
        if !service.is_empty() {
            inputs["service_returns"] = serde_json::json!(service);
        }
        Ok(inputs)
    }
    // Export only identities after joining generated content to the exact reviewed bundle.
    // Independently authored supplemental checks legitimately have no generated origin.
    fn project_experiment_origins(
        &self,
        supplemental: &serde_json::Value,
    ) -> Result<BTreeMap<String, serde_json::Value>> {
        let mut origins = BTreeMap::new();
        for (id, record) in self.records()? {
            let Record::Coding {
                record:
                    CodingRecord::ExperimentOrigin {
                        proposal,
                        index,
                        admission,
                        binding,
                    },
            } = record
            else {
                continue;
            };
            self.require_project_bridge_source(id, Source::Operator)?;
            self.require_project_bridge_source(admission, Source::Operator)?;
            let Suggestion::Experiment { experiment } =
                self.project_relational_proposal(proposal, index)?
            else {
                anyhow::bail!("experiment origin proposal kind mismatch");
            };
            let bundle = supplemental["records"]
                .as_array()
                .into_iter()
                .flatten()
                .find(|r| r["id"] == serde_json::json!(admission) && r.get("bundle").is_some())
                .ok_or_else(|| anyhow::anyhow!("experiment origin admission missing"))?;
            let bundle: Bundle = serde_json::from_value(bundle["bundle"].clone())?;
            ensure!(
                binding == experiment.binding
                    && bundle.version == "supplemental-cargo@v2"
                    && bundle.snapshot == experiment.snapshot
                    && bundle.requirements_revision == experiment.requirements_revision
                    && bundle.harness == experiment.harness
                    && serde_json::to_value(&bundle.cases)?
                        == serde_json::to_value(&experiment.cases)?
                    && serde_json::to_value(&bundle.mapping)?
                        == serde_json::to_value(&experiment.mapping)?
                    && bundle.probe_entry.is_empty(),
                "experiment origin content/admission mismatch"
            );
            ensure!(
                origins
                    .insert(
                        serde_json::json!(admission).to_string(),
                        serde_json::json!({
                            "record":id,"proposal":proposal,"index":index,"binding":binding,
                            "bundle_digest":Hash::of(&[&serde_json::to_vec(&bundle)?])
                        })
                    )
                    .is_none(),
                "duplicate experiment origin"
            );
        }
        for (id, record) in self.records()? {
            let Record::Coding {
                record:
                    CodingRecord::ExperimentReadmission {
                        previous,
                        admission,
                        state_root,
                        snapshot,
                    },
            } = record
            else {
                continue;
            };
            self.require_project_bridge_source(id, Source::Operator)?;
            self.require_project_bridge_source(admission, Source::Operator)?;
            let events = self.store.all()?;
            let position = events
                .iter()
                .position(|(_, e)| e.id == admission)
                .ok_or_else(|| anyhow::anyhow!("readmission event missing"))?;
            let prefix: Vec<String> = events[..position]
                .iter()
                .map(|(_, e)| e.id.to_string())
                .collect();
            ensure!(
                Hash::of(&prefix.iter().map(|s| s.as_bytes()).collect::<Vec<_>>()) == state_root
                    && events.get(position + 1).is_some_and(|(_, e)| e.id == id),
                "readmission root/sequence mismatch"
            );
            let origin = origins
                .get(&serde_json::json!(previous).to_string())
                .ok_or_else(|| anyhow::anyhow!("readmission generated origin missing"))?
                .clone();
            let get = |key: EventId| -> Result<Bundle> {
                serde_json::from_value(
                    supplemental["records"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .find(|r| r["id"] == serde_json::json!(key) && r.get("bundle").is_some())
                        .ok_or_else(|| anyhow::anyhow!("readmission bundle missing"))?["bundle"]
                        .clone(),
                )
                .map_err(Into::into)
            };
            let old = get(previous)?;
            let new = get(admission)?;
            ensure!(
                old.version == new.version
                    && old.requirements_revision == new.requirements_revision
                    && old.harness == new.harness
                    && old.probe_entry == new.probe_entry
                    && serde_json::to_value(&old.cases)? == serde_json::to_value(&new.cases)?
                    && serde_json::to_value(&old.mapping)? == serde_json::to_value(&new.mapping)?
                    && new.snapshot == snapshot,
                "readmission changed generated experiment"
            );
            let mut projected = origin;
            projected["readmission_record"] = serde_json::json!(id);
            projected["previous_admission"] = serde_json::json!(previous);
            projected["readmission_state_root"] = serde_json::json!(state_root);
            projected["bundle_digest"] = serde_json::json!(Hash::of(&[&serde_json::to_vec(&new)?]));
            ensure!(
                origins
                    .insert(serde_json::json!(admission).to_string(), projected)
                    .is_none(),
                "duplicate readmission origin"
            );
        }
        Ok(origins)
    }
    pub(crate) fn project_relational_proposal(
        &self,
        id: EventId,
        index: usize,
    ) -> Result<Suggestion> {
        self.require_project_bridge_source(id, Source::Generator)?;
        self.records()?
            .into_iter()
            .find_map(|(key, r)| match r {
                Record::ProposalReceived { response } if key == id => {
                    response.proposals.get(index).cloned()
                }
                _ => None,
            })
            .ok_or_else(|| anyhow::anyhow!("missing generated proposal/index"))
    }
    fn project_relational_admissions(&self) -> Result<Vec<(EventId, Hash, Proposal)>> {
        let mut out = vec![];
        for (id, r) in self.records()? {
            if let Record::Coding {
                record:
                    CodingRecord::RelationalAdmission {
                        proposal,
                        index,
                        binding,
                    },
            } = r
            {
                self.require_project_bridge_source(id, Source::Operator)?;
                let Suggestion::Relational { proposal: p, .. } =
                    self.project_relational_proposal(proposal, index)?
                else {
                    anyhow::bail!("wrong relational proposal kind");
                };
                out.push((id, binding, p));
            }
        }
        Ok(out)
    }
    fn project_relational_withdrawn(&self, id: EventId) -> Result<bool> {
        let mut withdrawn = false;
        for (key, r) in self.records()? {
            if let Record::Coding {
                record:
                    CodingRecord::RelationalStatus {
                        admission,
                        withdrawn: w,
                    },
            } = r
            {
                self.require_project_bridge_source(key, Source::Operator)?;
                if admission == id {
                    withdrawn = w;
                }
            }
        }
        Ok(withdrawn)
    }
    pub fn project_relational_context(&self) -> Result<serde_json::Value> {
        let inputs = self.project_relational_inputs()?;
        let binding = Hash::of(&[
            b"project-relational-inputs@v1",
            &serde_json::to_vec(&inputs)?,
        ]);
        let browser_profile = self.project_configuration()?.browser;
        let active: BTreeSet<PreferenceId> = inputs["parents"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| serde_json::from_value(p["id"].clone()))
            .collect::<std::result::Result<_, _>>()?;
        let mut catalysts = vec![];
        let mut history = vec![];
        let mut guidance = vec![];
        let admitted = self.project_relational_admissions()?;
        for (id, b, p) in &admitted {
            let withdrawn = self.project_relational_withdrawn(*id)?;
            let superseded = admitted
                .iter()
                .any(|(_, _, next)| next.revises == Some(id.0));
            let current = *b == binding && !withdrawn && !superseded;
            let entry = serde_json::json!({"admission":id,"binding":b,"proposal":p,"withdrawn":withdrawn,"superseded":superseded,"current":current});
            if browser_profile
                && !withdrawn
                && !superseded
                && p.parents.iter().all(|p| active.contains(p))
            {
                catalysts.push(
                    serde_json::json!({"admission":id,"parents":p.parents,"historical":!current}),
                );
            }
            if current {
                guidance.push(entry.clone());
            }
            history.push(entry);
        }
        let mut context = serde_json::json!({"binding":binding,"inputs":inputs,"guidance":guidance,"history":history,"catalyst_references":catalysts,
            "narrative_capacity":if self.project_configuration()?.browser {serde_json::json!({"profile":"browser-aggregate-text@v1","aggregate_response_bytes":browser::RESPONSE_BYTES,"per_field":"nonblank; no additional field-length cap"})} else {serde_json::json!({"profile":"legacy-short-text@v1","field_bytes":512,"relational_bytes":4096})},
            "semantics":"Qualitative hypotheses, not requirements, measured fulfillment, calibrated confidence or action permission. Returns are finite adapter observations. Develop or retain meaning explicitly; distinguish what returns reveal from what remains unknown. Historical guidance requires fresh source-bound review before reuse.",
            "proposal_contract":{"parents":"2..8 distinct active ids from inputs.parents; retain original meanings and reviewed development (acceptance mappings are separate in v3)","relationship":"bounded relational meaning","aspiration":"what is newly wanted, not a score","discriminator":"what could distinguish this hypothesis","revises":"null for initial aspiration; otherwise exact latest nonwithdrawn, nonsuperseded admission hash from history, only after binding changes","catalysts":if browser_profile {"optional distinct admissions from catalyst_references, at most 4; preserve every cited contributor in parents. Historical inspiration is not current truth, satisfaction, new identity or authority; new composition still requires independent review"} else {"empty in legacy project profile"},"development":"initial: null or absent, never an object. Revision: required object; original_revision is exact origin admission hash obtained by following revises in history; prior_revision equals revises (latest ancestor admission hash). These are 64-character identity references, never prose. Retain the ancestor parent set. Include retained_meaning, return_reveals, unresolved, proposed_development (null/absent means retain prior relationship, aspiration and discriminator)"},
            "experiment_contract":{"binding":"copy this binding","snapshot":"copy inputs.snapshot","requirements_revision":"copy inputs.requirements_revision","harness":"1..2 new tests/tsr_supplemental_*.rs files; no original file replacement","cases":"new uniquely named Cargo tests, empty args/expected_stdout","mapping":"published requirement id -> cases, rationale, finite limits"},
            "public_desire_contract":{"kind":"public_desire","proposal":"binding, identity of active parent, revises exact revision or null for legacy parent, parents including identity, meaning under narrative_capacity (legacy <=512 bytes; browser aggregate response capacity), change clarification|development, basis {kind:unknown}|{kind:observation,receipt}|{kind:judgment,review}, generator attribution","semantics":"Separate reviewed public meaning; no change to acceptance criteria or authority. Clarification/paraphrase is not evidence of emergence. Unknown is not negative. Proposals remain unadmitted until separate semantic review and project desire-admit or explicit atomic project cycle meaning-advance."},"grants_authority":false,"fulfills_parents":false});
        if browser_profile {
            let aims = self.acceptance_current_aims()?;
            let current_contrast = self.project_current_contrast()?;
            let mut contrast = vec![];
            let mut success = vec![];
            for entry in context["guidance"].as_array().unwrap() {
                let id: EventId = serde_json::from_value(entry["admission"].clone())?;
                if let Some((_, roots, _)) = self.derived_preference_candidate(&context, id)? {
                    if let Some(c) = current_contrast.as_ref() {
                        if self.contrast_covers_contributors(c, &roots)? {
                            contrast.push(id);
                        }
                    }
                    if let Some(criteria) =
                        context["inputs"]["current_success"]["basis"]["criteria"].as_array()
                    {
                        if roots.iter().all(|root| {
                            aims.iter().any(|(key, parent)| {
                                parent == root && criteria.contains(&serde_json::json!(key))
                            })
                        }) {
                            success.push(id);
                        }
                    }
                }
            }
            context["identity_choices"] = serde_json::json!({"contrast_preference":contrast,"success_preference":success,"authority":"Current structural candidates only; independent semantic review still required. An empty list is not an assertion that development is unwanted."});
        }
        Ok(context)
    }
    pub(crate) fn validate_project_relational(&self, binding: Hash, p: &Proposal) -> Result<()> {
        let inputs = self.project_relational_inputs()?;
        ensure!(
            Hash::of(&[
                b"project-relational-inputs@v1",
                &serde_json::to_vec(&inputs)?
            ]) == binding,
            "stale project relational evidence"
        );
        let allowed: BTreeSet<PreferenceId> = inputs["parents"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| serde_json::from_value(p["id"].clone()))
            .collect::<std::result::Result<_, _>>()?;
        let parents: BTreeSet<_> = p.parents.iter().copied().collect();
        ensure!(
            parents.len() >= 2
                && parents.len() <= 8
                && parents.len() == p.parents.len()
                && parents.is_subset(&allowed),
            "distinct active public parent identities required"
        );
        let browser_profile = self.project_configuration()?.browser;
        let narrative = |s: &str| {
            if browser_profile {
                browser::narrative(s)
            } else {
                short(s)
            }
        };
        ensure!(
            serde_json::to_vec(p)?.len()
                <= if browser_profile {
                    browser::RESPONSE_BYTES
                } else {
                    4096
                },
            "relational aggregate memory bound"
        );
        ensure!(
            [&p.relationship, &p.aspiration, &p.discriminator]
                .iter()
                .all(|s| narrative(s)),
            "bounded relational meaning"
        );
        let history = self.project_relational_admissions()?;
        ensure!(
            p.catalysts.is_empty() || browser_profile,
            "project profile has no catalyst authority"
        );
        let unique: BTreeSet<_> = p.catalysts.iter().copied().collect();
        ensure!(
            unique.len() == p.catalysts.len() && unique.len() <= 4,
            "distinct bounded catalysts required"
        );
        for catalyst in &p.catalysts {
            let (id, _, prior) = history
                .iter()
                .find(|(id, _, _)| id.0 == *catalyst)
                .ok_or_else(|| anyhow::anyhow!("unknown or foreign relational catalyst"))?;
            ensure!(
                !self.project_relational_withdrawn(*id)?
                    && !history
                        .iter()
                        .any(|(_, _, next)| next.revises == Some(*catalyst)),
                "inactive or superseded relational catalyst"
            );
            ensure!(
                prior.parents.iter().all(|p| parents.contains(p)),
                "catalyst contributing parents must remain present"
            );
        }
        if let Some(prior) = p.revises {
            let (id, old_binding, old) = history
                .iter()
                .find(|(id, _, _)| id.0 == prior)
                .ok_or_else(|| anyhow::anyhow!("unknown relational ancestor"))?;
            ensure!(
                !self.project_relational_withdrawn(*id)?
                    && !history.iter().any(|(_, _, p)| p.revises == Some(prior))
                    && old.parents.iter().copied().collect::<BTreeSet<_>>() == parents,
                "stale/withdrawn or changed parent lineage"
            );
            ensure!(
                *old_binding != binding,
                "revision requires changed observed context, not a repeated narrative"
            );
            let d = p
                .development
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("separated return development required"))?;
            let mut origin = *id;
            let mut cursor = old;
            while let Some(previous) = cursor.revises {
                let (key, _, ancestor) = history
                    .iter()
                    .find(|(key, _, _)| key.0 == previous)
                    .ok_or_else(|| anyhow::anyhow!("broken lineage"))?;
                origin = *key;
                cursor = ancestor;
            }
            ensure!(
                d.original_revision == origin.0
                    && d.prior_revision == prior
                    && [&d.retained_meaning, &d.return_reveals, &d.unresolved]
                        .iter()
                        .all(|s| narrative(s))
                    && d.proposed_development.as_ref().is_none_or(|s| narrative(s)),
                "invalid separated development provenance/text"
            );
            ensure!(
                d.proposed_development.is_some()
                    || (p.relationship == old.relationship
                        && p.aspiration == old.aspiration
                        && p.discriminator == old.discriminator),
                "no new insight must retain meaning; record observations separately"
            );
        } else {
            ensure!(p.development.is_none(), "development requires ancestor");
            ensure!(
                !history.iter().any(|(_, _, old)| old
                    .parents
                    .iter()
                    .copied()
                    .collect::<BTreeSet<_>>()
                    == parents
                    && old.relationship == p.relationship
                    && old.aspiration == p.aspiration
                    && old.discriminator == p.discriminator),
                "duplicate relational hypothesis; use lineage"
            );
        }
        if !browser_profile {
            ensure!(history.len() < 64, "project relational history bound");
        } // Browser history is preserved; actual request/resource limits remain authoritative.
        Ok(())
    }
    /// Explicit review/admission; does not admit any execution or mark a parent satisfied.
    pub fn project_relational_admit(
        &self,
        root: Hash,
        proposal: EventId,
        index: usize,
    ) -> Result<EventId> {
        self.write(|| self.project_relational_admit_unlocked(root, proposal, index))
    }
    pub(crate) fn project_relational_admit_unlocked(
        &self,
        root: Hash,
        proposal: EventId,
        index: usize,
    ) -> Result<EventId> {
        ensure!(
            root == self.store.head_root()?,
            "stale relational admission"
        );
        let Suggestion::Relational {
            binding,
            proposal: p,
        } = self.project_relational_proposal(proposal, index)?
        else {
            anyhow::bail!("wrong proposal kind");
        };
        self.validate_project_relational(binding, &p)?;
        self.adaptive_record(
            &Record::Coding {
                record: CodingRecord::RelationalAdmission {
                    proposal,
                    index,
                    binding,
                },
            },
            Source::Operator,
        )
    }
    pub fn project_relational_status(
        &self,
        root: Hash,
        admission: EventId,
        withdrawn: bool,
    ) -> Result<()> {
        self.write(|| {
            ensure!(
                root == self.store.head_root()?
                    && self
                        .project_relational_admissions()?
                        .iter()
                        .any(|(id, _, _)| *id == admission),
                "stale or unknown relational status"
            );
            if self.project_relational_withdrawn(admission)? != withdrawn {
                self.adaptive_record(
                    &Record::Coding {
                        record: CodingRecord::RelationalStatus {
                            admission,
                            withdrawn,
                        },
                    },
                    Source::Operator,
                )?;
            }
            Ok(())
        })
    }
    pub(crate) fn validate_project_experiment(&self, x: &Experiment) -> Result<()> {
        let inputs = self.project_relational_inputs()?;
        ensure!(
            Hash::of(&[
                b"project-relational-inputs@v1",
                &serde_json::to_vec(&inputs)?
            ]) == x.binding,
            "stale experiment context"
        );
        let allowed: BTreeSet<_> = inputs
            .get("acceptance_parents")
            .unwrap_or(&inputs["parents"])
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|p| p["requirement"].as_str())
            .collect();
        ensure!(
            x.mapping.keys().all(|k| allowed.contains(k.as_str())),
            "experiment requires published requirement mapping"
        );
        // Validate structure using the same contract as operator admission, never persist an admission.
        self.project_validate_supplemental(&Bundle {
            version: "supplemental-cargo@v2".into(),
            snapshot: x.snapshot,
            requirements_revision: x.requirements_revision,
            author: "unreviewed-generator".into(),
            reviewer: "validation-placeholder-not-review".into(),
            generator: "transport-role".into(),
            review: "Shape validation only; not admission".into(),
            harness: x.harness.clone(),
            probe_entry: String::new(),
            cases: x.cases.clone(),
            mapping: x.mapping.clone(),
        })?;
        Ok(())
    }
    /// Fresh review and source admission of unchanged generated checks, never execution authority.
    pub fn project_experiment_readmit(
        &self,
        root: Hash,
        previous: EventId,
        snapshot: Hash,
        attestation: ExperimentReview,
    ) -> Result<EventId> {
        self.write(|| {
            ensure!(root==self.store.head_root()?,"stale experiment readmission");
            ensure!(self.adaptive_state_inner()?.pending.is_none(),"pending action prevents experiment readmission");
            let view=self.project_supplemental_view()?;
            ensure!(!view["records"].as_array().into_iter().flatten().any(|r|r["revoked"]==serde_json::json!(previous)),"previous experiment revoked");
            let origins=self.project_experiment_origins(&view)?;
            ensure!(origins.contains_key(&serde_json::json!(previous).to_string()),"reviewed generated origin required");
            let mut ancestor=previous;
            loop {
                ensure!(!view["records"].as_array().into_iter().flatten().any(|r|r["revoked"]==serde_json::json!(ancestor)),"experiment lineage revoked");
                let provenance=origins.get(&serde_json::json!(ancestor).to_string()).ok_or_else(||anyhow::anyhow!("experiment lineage missing"))?;
                if provenance["previous_admission"].is_null(){break;}
                ancestor=serde_json::from_value(provenance["previous_admission"].clone())?;
            }
            self.require_project_bridge_source(previous,Source::Operator)?;
            let old:Bundle=serde_json::from_value(view["records"].as_array().into_iter().flatten().find(|r|r["id"]==serde_json::json!(previous) && r.get("bundle").is_some()).ok_or_else(||anyhow::anyhow!("previous experiment missing"))?["bundle"].clone())?;
            let current=self.project_acceptance_status()?;
            ensure!(snapshot==current.snapshot && snapshot!=old.snapshot && Some(old.requirements_revision)==current.requirements_revision,"changed source and unchanged requirements required");
            ensure!(!self.records()?.iter().any(|(_,r)|matches!(r,Record::Coding{record:CodingRecord::ExperimentReadmission{previous:p,snapshot:s,..}} if *p==previous && *s==snapshot)),"experiment already readmitted to this source");
            let admission=self.project_supplemental_admit_inner(Bundle{snapshot,author:attestation.author,reviewer:attestation.reviewer,generator:attestation.generator,review:attestation.review,..old})?;
            self.adaptive_record(&Record::Coding{record:CodingRecord::ExperimentReadmission{previous,admission,state_root:root,snapshot}},Source::Operator)?;
            Ok(admission)
        })
    }
    pub fn project_experiment_admit(
        &self,
        root: Hash,
        proposal: EventId,
        index: usize,
        attestation: ExperimentReview,
    ) -> Result<EventId> {
        self.write(|| {
            ensure!(
                root == self.store.head_root()?,
                "stale experiment admission"
            );
            let Suggestion::Experiment { experiment: x } =
                self.project_relational_proposal(proposal, index)?
            else {
                anyhow::bail!("wrong experiment proposal kind");
            };
            self.validate_project_experiment(&x)?;
            // This method uses the unlocked validator/append under the same transaction.
            let admission = self.project_supplemental_admit_inner(Bundle {
                version: "supplemental-cargo@v2".into(),
                snapshot: x.snapshot,
                requirements_revision: x.requirements_revision,
                author: attestation.author,
                reviewer: attestation.reviewer,
                generator: attestation.generator,
                review: attestation.review,
                harness: x.harness,
                probe_entry: String::new(),
                cases: x.cases,
                mapping: x.mapping,
            })?;
            self.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::ExperimentOrigin {
                        proposal,
                        index,
                        admission,
                        binding: x.binding,
                    },
                },
                Source::Operator,
            )?;
            Ok(admission)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::adaptive::provider::{Response, PROTOCOL};
    // Authored protocol fixtures only: no model generation or real Cargo execution.
    fn setup() -> Result<(tempfile::TempDir, Engine)> {
        setup_wall(120_000)
    }
    fn setup_wall(wall_ms: u64) -> Result<(tempfile::TempDir, Engine)> {
        let d = tempfile::tempdir()?;
        let e = Engine::open(d.path())?;
        let spec: ProjectSpec = serde_json::from_value(serde_json::json!({
            "goal":"Handle empty input; preserve nonempty behavior",
            "image":format!("sha256:{}","1".repeat(64)),"user":"65534:65534","cargo":true,
            "editable":{"src/lib.rs":"pub fn parse(s: &str) -> bool { !s.is_empty() }\n"},
            "protected":{"Cargo.toml":"[package]\nname=\"parser\"\nversion=\"0.1.0\"\nedition=\"2021\"\n", "Cargo.lock":"version = 4\n[[package]]\nname=\"parser\"\nversion=\"0.1.0\"\n", "tests/original.rs":"#[test] fn empty() {}\n#[test] fn nonempty() {}\n"},
            "cases":[{"id":"empty","args":[],"expected_stdout":""},{"id":"nonempty","args":[],"expected_stdout":""}],"timeout_ms":2000
        }))?;
        e.project_initialize(spec)?;
        e.project_acceptance_requirements(BTreeMap::from([
            ("first".into(), "Empty input handled".into()),
            ("second".into(), "Existing behavior unchanged".into()),
        ]))?;
        open_wall(&e, wall_ms)?;
        let c = e.project_acceptance_status()?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: c.requirements_revision.unwrap(),
            decision_record: None,
            guidance: BTreeMap::from([
                ("first".into(), "Empty input handled".into()),
                ("second".into(), "Existing behavior unchanged".into()),
            ]),
        })?;
        Ok((d, e))
    }
    fn open(e: &Engine) -> Result<()> {
        open_wall(e, 120_000)
    }
    fn open_wall(e: &Engine, wall_ms: u64) -> Result<()> {
        e.project_lifecycle_open(lifecycle::Admission {
            state_root: e.store.head_root()?,
            training: None,
            tool_ms_ceiling: 100_000,
            reason: "Authored protocol fixture only".into(),
            non_tool: Some(budget::Limits {
                model_calls: 0,
                model_tokens: 0,
                wall_ms,
                patch_bytes: 0,
                external_cost_microusd: 0,
            }),
        })?;
        Ok(())
    }
    fn propose(e: &Engine, p: Suggestion) -> Result<EventId> {
        let root = e.store.head_root()?;
        let result = e.evaluate_proposal(
            Response {
                protocol: PROTOCOL.into(),
                state_root: root,
                attention_binding: e.proposal_attention_binding()?,
                proposals: vec![p],
                pursuit_expectations: Default::default(),
                common_event_forecasts: Default::default(),
            },
            true,
        )?;
        assert!(!result.committed);
        assert!(e.adaptive_state()?.pending.is_none());
        Ok(e.records()?
            .iter()
            .rev()
            .find_map(|(id, r)| matches!(r, Record::ProposalReceived { .. }).then_some(*id))
            .unwrap())
    }
    fn hypothesis(e: &Engine) -> Result<(Hash, Proposal)> {
        let ctx = e.project_relational_context()?;
        let parents = ctx["inputs"]["parents"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| serde_json::from_value(p["id"].clone()).unwrap())
            .collect();
        Ok((
            serde_json::from_value(ctx["binding"].clone())?,
            Proposal {
                parents,
                relationship: "Fixture relational hypothesis, not evidence of emergence".into(),
                aspiration: "Investigate compatible boundary behavior".into(),
                discriminator: "Observe a separately reviewed second-aim check".into(),
                revises: None,
                catalysts: vec![],
                development: None,
            },
        ))
    }
    #[test]
    fn browser_history_keeps_development_available_after_legacy_cutoff() -> Result<()> {
        // Capacity boundary fixture, not 130 simulated full deliberations. Each
        // authored generator output is quarantined as an actual native proposal;
        // the unchanged admission API validates and appends every admission.
        // Ordinary independent review/advance is exercised by the browser suite.
        fn quarantine(e: &Engine, binding: Hash, proposal: Proposal) -> Result<EventId> {
            e.validate_project_relational(binding, &proposal)?;
            e.adaptive_record(
                &Record::ProposalReceived {
                    response: Response {
                        protocol: PROTOCOL.into(),
                        state_root: e.store.head_root()?,
                        attention_binding: e.proposal_attention_binding()?,
                        proposals: vec![Suggestion::Relational { binding, proposal }],
                        pursuit_expectations: Default::default(),
                        common_event_forecasts: Default::default(),
                    },
                },
                Source::Generator,
            )
        }
        for browser in [true, false] {
            let (dir, e) = if browser {
                let dir = tempfile::tempdir()?;
                let e = Engine::open(dir.path())?;
                e.project_initialize(super::browser::tests::spec())?;
                let wants = BTreeMap::from([
                    ("controls".into(), "Usable controls".into()),
                    ("clarity".into(), "Understandable status".into()),
                ]);
                e.project_acceptance_requirements(wants.clone())?;
                open_wall(&e, 600_000)?;
                let status = e.project_acceptance_status()?;
                e.project_acceptance_feedback(acceptance::Feedback {
                    requirements_revision: status.requirements_revision.unwrap(),
                    decision_record: None,
                    guidance: wants,
                })?;
                (dir, e)
            } else {
                setup_wall(600_000)?
            };
            e.project_resource_unpriced_admit(e.store.head_root()?)?;
            let aims = e.acceptance_current_aims()?;
            for index in 0..64 {
                let (binding, mut proposal) = hypothesis(&e)?;
                proposal.aspiration =
                    format!("Authored capacity fixture {index}; not spontaneous development");
                let record = quarantine(&e, binding, proposal)?;
                e.project_relational_admit(e.store.head_root()?, record, 0)?;
            }
            let (binding, mut proposal) = hypothesis(&e)?;
            proposal.aspiration = "Authored next capacity fixture".into();
            let before = e.store.head_root()?;
            if browser {
                e.validate_project_relational(binding, &proposal)?;
                assert!(e
                    .validate_project_relational(Hash::of(&[b"stale"]), &proposal)
                    .is_err());
                let mut duplicate = proposal.clone();
                duplicate.aspiration =
                    "Authored capacity fixture 0; not spontaneous development".into();
                assert!(e.validate_project_relational(binding, &duplicate).is_err());
                let mut oversized = proposal.clone();
                oversized.relationship = "x".repeat(131073);
                assert!(e.validate_project_relational(binding, &oversized).is_err());
                assert_eq!(before, e.store.head_root()?);
                let record = quarantine(&e, binding, proposal)?;
                e.project_relational_admit(e.store.head_root()?, record, 0)?;
                assert_eq!(
                    e.project_relational_context()?["history"]
                        .as_array()
                        .unwrap()
                        .len(),
                    65
                );
            } else {
                assert!(e.validate_project_relational(binding, &proposal).is_err());
                assert_eq!(before, e.store.head_root()?);
            }
            assert_eq!(e.acceptance_current_aims()?, aims);
            assert!(!e.project_acceptance_status()?.accepted);
            assert!(e.adaptive_state()?.pending.is_none());
            let context = e.project_relational_context()?;
            drop(e);
            assert_eq!(
                Engine::open(dir.path())?.project_relational_context()?,
                context
            );
        }
        Ok(())
    }
    struct CargoFixture {
        failed: bool,
        broken: bool,
        extra: bool,
    }
    impl Runner for CargoFixture {
        fn run(&self, _: &Path, args: &[String], _: &Limits) -> RunResult {
            assert!(
                matches!(args[0].as_str(), "true" | "cargo"),
                "verification cannot edit"
            );
            if args[0] == "true" || args.iter().any(|s| s == "--no-run") {
                return RunResult::exit_only(if self.broken { 101 } else { 0 });
            }
            let mut names = vec!["empty", "nonempty"];
            if self.extra {
                names.push("boundary");
            }
            let mut text = format!("running {} tests\n", names.len());
            for name in &names {
                text.push_str(&format!(
                    "test {name} ... {}\n",
                    if self.failed && *name == names[names.len() - 1] {
                        "FAILED"
                    } else {
                        "ok"
                    }
                ));
            }
            let failed = usize::from(self.failed);
            text.push_str(&format!("test result: {}. {} passed; {failed} failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n",if self.failed{"FAILED"}else{"ok"},names.len()-failed));
            RunResult::from_output(if self.failed { 101 } else { 0 }, text.as_bytes(), b"", 1)
        }
    }
    fn verify(e: &Engine, runner: &dyn Runner) -> Result<EventId> {
        let c = e.project_acceptance_status()?;
        let response = e.project_admit_verification(e.store.head_root()?, c.snapshot)?;
        let result = e.evaluate_proposal(serde_json::from_value(response)?, true)?;
        assert!(result.committed);
        let plan = e.coding_plan()?;
        assert!(plan.edits.is_empty());
        e.coding_execute(plan.commitment, plan.source_before, runner)?;
        Ok(e.project_acceptance_status()?.test_receipt.unwrap())
    }
    fn evaluator_pending(e: &Engine) -> Result<evaluator::Admission> {
        let c = e.project_acceptance_status()?;
        let response = e.project_admit_verification(e.store.head_root()?, c.snapshot)?;
        assert!(
            e.evaluate_proposal(serde_json::from_value(response)?, true)?
                .committed
        );
        let pending = e.adaptive_state()?.pending.unwrap();
        Ok(evaluator::Admission {
            state_root: e.store.head_root()?,
            snapshot: c.snapshot,
            commitment: pending.commitment,
            action: pending.action,
            probability: 0.1,
            evaluator: "evaluator-authored deterministic fixture".into(),
        })
    }
    #[test]
    fn b91_evaluator_forecast_binds_native_return_and_survives_restart() -> Result<()> {
        let (d, e) = setup()?;
        let admission = evaluator_pending(&e)?;
        let controller = e.adaptive_state()?.pending.unwrap().forecast;
        let id = e.project_evaluator_forecast(admission.clone())?;
        assert!(e.project_evaluator_forecast(admission.clone()).is_err());
        drop(e);
        let e = Engine::open(d.path())?;
        assert_eq!(
            e.adaptive_state()?.pending.unwrap().forecast.id,
            controller.id
        );
        let plan = e.coding_plan()?;
        e.coding_execute(
            plan.commitment,
            plan.source_before,
            &CargoFixture {
                failed: true,
                broken: false,
                extra: false,
            },
        )?;
        let view = e.project_evaluator_forecasts()?;
        let rows = view["records"].as_array().unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[1]["admission_record"], serde_json::to_value(id)?);
        assert_eq!(
            rows[1]["commitment"],
            serde_json::to_value(admission.commitment)?
        );
        assert_eq!(rows[1]["residual"], 0.1);
        assert_eq!(rows[1]["observed_success"], false);
        assert!(rows[0]["forecast"]["calibrated"].is_null());
        assert_eq!(e.acceptance_current_aims()?.len(), 2);
        let pending = Pending {
            action: admission.action,
            commitment: admission.commitment,
            forecast: controller,
            criteria: vec![],
        };
        assert!(e
            .project_resolve_evaluator(&pending, EventId(Hash::of(&[b"wrong-return"])), false)
            .is_err());
        Ok(())
    }
    #[test]
    fn b91_evaluator_forecast_rejects_stale_identity_and_forged_origin() -> Result<()> {
        let (_d, e) = setup()?;
        let admission = evaluator_pending(&e)?;
        for kind in 0..4 {
            let mut bad = admission.clone();
            match kind {
                0 => bad.state_root = Hash::of(&[b"stale"]),
                1 => bad.snapshot = Hash::of(&[b"stale"]),
                2 => bad.action = "wrong-action".into(),
                _ => bad.commitment = EventId(Hash::of(&[b"foreign"])),
            }
            let before = e.store.head_root()?;
            assert!(e.project_evaluator_forecast(bad).is_err());
            assert_eq!(e.store.head_root()?, before);
        }
        let forecast = e.adaptive_state()?.pending.unwrap().forecast;
        e.adaptive_record(
            &Record::Coding {
                record: CodingRecord::EvaluatorForecast {
                    admission,
                    forecast,
                },
            },
            Source::Generator,
        )?;
        assert!(e.project_evaluator_forecasts().is_err());
        Ok(())
    }
    #[test]
    fn b91_unknown_execution_does_not_resolve_evaluator_forecast() -> Result<()> {
        let (_d, e) = setup()?;
        let admission = evaluator_pending(&e)?;
        e.project_evaluator_forecast(admission)?;
        let plan = e.coding_plan()?;
        e.coding_execute(
            plan.commitment,
            plan.source_before,
            &CargoFixture {
                failed: false,
                broken: true,
                extra: false,
            },
        )?;
        assert_eq!(
            e.project_evaluator_forecasts()?["records"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        Ok(())
    }
    #[test]
    fn b91_evaluator_rejects_wrong_return_before_resolution_and_forged_resolution() -> Result<()> {
        let (_d, e) = setup()?;
        let admission = evaluator_pending(&e)?;
        let admitted = e.project_evaluator_forecast(admission.clone())?;
        let pending = e.adaptive_state()?.pending.unwrap();
        let plan = e.coding_plan()?;
        // Deliberately authored adversarial records; no runner invocation.
        e.adaptive_record(
            &Record::Coding {
                record: CodingRecord::Intent { plan },
            },
            Source::Rules,
        )?;
        let receipt = e.adaptive_record(
            &Record::Coding {
                record: CodingRecord::Receipt {
                    commitment: pending.commitment,
                    verdict: Verdict::FailedTests,
                    runs: vec![],
                    source_after: Some(admission.snapshot),
                    observations: vec![],
                },
            },
            Source::Rules,
        )?;
        let wrong = e.adaptive_record(
            &Record::Returned {
                commitment: pending.commitment,
                action: "foreign-action".into(),
                success: false,
                input: ResultInput {
                    met: vec![],
                    evidence: "Fixture malformed return".into(),
                    proposed_refinement: None,
                },
            },
            Source::Rules,
        )?;
        let before = e.store.head_root()?;
        assert!(e.project_resolve_evaluator(&pending, wrong, false).is_err());
        assert_eq!(before, e.store.head_root()?);
        let mismatched = e.adaptive_record(
            &Record::Returned {
                commitment: pending.commitment,
                action: pending.action.clone(),
                success: true,
                input: ResultInput {
                    met: vec![],
                    evidence: "Fixture contradictory return".into(),
                    proposed_refinement: None,
                },
            },
            Source::Rules,
        )?;
        let before = e.store.head_root()?;
        assert!(e
            .project_resolve_evaluator(&pending, mismatched, true)
            .is_err());
        assert_eq!(before, e.store.head_root()?);
        let forecast = e
            .records()?
            .into_iter()
            .find_map(|(id, r)| match r {
                Record::Coding {
                    record: CodingRecord::EvaluatorForecast { forecast, .. },
                } if id == admitted => Some(forecast),
                _ => None,
            })
            .unwrap();
        e.adaptive_record(
            &Record::Coding {
                record: CodingRecord::EvaluatorResolution {
                    admission: admitted,
                    commitment: pending.commitment,
                    returned: wrong,
                    receipt,
                    forecast,
                    success: false,
                },
            },
            Source::Rules,
        )?;
        assert!(e.project_evaluator_forecasts().is_err());
        Ok(())
    }
    #[test]
    fn b91_generated_experiment_readmission_preserves_bytes_and_requires_fresh_review() -> Result<()>
    {
        let (d, e) = setup()?;
        let x = experiment(&e)?;
        let proposal = propose(
            &e,
            Suggestion::Experiment {
                experiment: x.clone(),
            },
        )?;
        let review = || ExperimentReview {
            author: "fixture-author".into(),
            reviewer: "independent-fixture-reviewer".into(),
            generator: "fixture-generator".into(),
            review: "Explicit source-specific fixture review, not model evidence".into(),
        };
        let old = e.project_experiment_admit(e.store.head_root()?, proposal, 0, review())?;
        assert!(e
            .project_experiment_readmit(e.store.head_root()?, old, x.snapshot, review())
            .is_err());
        // Controlled fixture source transition only; no target command or model call.
        fs::write(
            d.path().join("coding-work/src/lib.rs"),
            "pub fn parse(s: &str) -> bool { s.is_empty() }\n",
        )?;
        let snapshot = e.project_acceptance_status()?.snapshot;
        assert!(e
            .project_experiment_readmit(Hash::of(&[b"stale"]), old, snapshot, review())
            .is_err());
        assert!(e
            .project_experiment_readmit(
                e.store.head_root()?,
                EventId(Hash::of(&[b"foreign"])),
                snapshot,
                review()
            )
            .is_err());
        assert!(e
            .project_experiment_readmit(e.store.head_root()?, old, x.snapshot, review())
            .is_err());
        let new = e.project_experiment_readmit(e.store.head_root()?, old, snapshot, review())?;
        assert_ne!(old, new);
        assert!(e
            .project_experiment_readmit(e.store.head_root()?, old, snapshot, review())
            .is_err());
        let view = e.project_supplemental_view()?;
        let origins = e.project_experiment_origins(&view)?;
        assert_eq!(
            origins[&serde_json::json!(new).to_string()]["proposal"],
            serde_json::json!(proposal)
        );
        let bundles: Vec<Bundle> = view["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|r| r.get("bundle"))
            .map(|b| serde_json::from_value(b.clone()).unwrap())
            .collect();
        assert_eq!(bundles.len(), 2);
        assert_eq!(bundles[0].harness, bundles[1].harness);
        assert_eq!(
            serde_json::to_value(&bundles[0].mapping)?,
            serde_json::to_value(&bundles[1].mapping)?
        );
        assert_eq!(bundles[0].snapshot, x.snapshot);
        assert_eq!(bundles[1].snapshot, snapshot);
        drop(e);
        let e = Engine::open(d.path())?;
        assert_eq!(
            e.project_experiment_origins(&e.project_supplemental_view()?)?
                .len(),
            2
        );
        e.project_supplemental_revoke(new, "Fixture withdrawal".into())?;
        fs::write(
            d.path().join("coding-work/src/lib.rs"),
            "pub fn parse(_: &str) -> bool { true }\n",
        )?;
        assert!(e
            .project_experiment_readmit(
                e.store.head_root()?,
                new,
                e.project_acceptance_status()?.snapshot,
                review()
            )
            .is_err());
        Ok(())
    }
    fn experiment(e: &Engine) -> Result<Experiment> {
        let c = e.project_acceptance_status()?;
        Ok(Experiment {
            binding: serde_json::from_value(e.project_relational_context()?["binding"].clone())?,
            snapshot: c.snapshot,
            requirements_revision: c.requirements_revision.unwrap(),
            harness: BTreeMap::from([(
                "tests/tsr_supplemental_boundary.rs".into(),
                "#[test] fn boundary() { assert!(parser::parse(\"x\")); }\n".into(),
            )]),
            cases: vec![ProbeCase {
                id: "boundary".into(),
                args: vec![],
                expected_stdout: String::new(),
            }],
            mapping: BTreeMap::from([(
                "second".into(),
                Mapping {
                    cases: BTreeSet::from(["boundary".into()]),
                    rationale: "Authored fixture: finite second-aim example".into(),
                    limits: "Finite check, not universal equivalence".into(),
                },
            )]),
        })
    }
    // Explicit authored fixture exporter / read-only validator. No real runner or provider.
    #[test]
    #[ignore = "explicit semantic shadow fixture preparation; zero real target executions"]
    fn semantic_shadow_fixture() -> Result<()> {
        fn copy_tree(from: &Path, to: &Path) -> Result<()> {
            fs::create_dir(to)?;
            for item in fs::read_dir(from)? {
                let item = item?;
                let kind = item.file_type()?;
                ensure!(!kind.is_symlink(), "fixture symlink");
                if kind.is_dir() {
                    copy_tree(&item.path(), &to.join(item.file_name()))?;
                } else {
                    fs::copy(item.path(), to.join(item.file_name()))?;
                }
            }
            Ok(())
        }
        let root = std::path::PathBuf::from(std::env::var("TSR_SEMANTIC_SHADOW_ROOT")?);
        let mode = std::env::var("TSR_SEMANTIC_SHADOW_MODE")?;
        if mode == "prepare" || mode == "prepare_next" {
            ensure!(!root.exists(), "fresh fixture root required");
            fs::create_dir_all(&root)?;
            let (directory, e) = setup_wall(3_600_000)?;
            verify(
                &e,
                &CargoFixture {
                    failed: false,
                    broken: false,
                    extra: false,
                },
            )?;
            let (binding, p) = hypothesis(&e)?;
            let proposal = propose(
                &e,
                Suggestion::Relational {
                    binding,
                    proposal: p,
                },
            )?;
            e.project_relational_admit(e.store.head_root()?, proposal, 0)?;
            drop(e);
            for (label, decision) in [
                ("reject", acceptance::Decision::Reject),
                ("unknown", acceptance::Decision::Unresolved),
            ] {
                let path = root.join(label);
                copy_tree(directory.path(), &path)?;
                let e = Engine::open(&path)?;
                let current = e.project_acceptance_status()?;
                e.project_acceptance_review(acceptance::Review {
                    snapshot: current.snapshot,
                    requirements_revision: current.requirements_revision.unwrap(),
                    test_receipt: current.test_receipt,
                    decision,
                    evidence: BTreeMap::from([
                        (
                            "first".into(),
                            acceptance::Evidence {
                                met: true,
                                evidence: "PRIVATE fixture reviewer evidence".into(),
                            },
                        ),
                        (
                            "second".into(),
                            acceptance::Evidence {
                                met: false,
                                evidence: "PRIVATE fixture reviewer evidence".into(),
                            },
                        ),
                    ]),
                })?;
                if mode == "prepare_next" && label == "reject" {
                    let context = e.project_relational_context()?;
                    let origin: EventId =
                        serde_json::from_value(context["history"][0]["admission"].clone())?;
                    let mut revised: Proposal =
                        serde_json::from_value(context["history"][0]["proposal"].clone())?;
                    revised.revises = Some(origin.0);
                    revised.relationship = "Boundary handling and preservation must be justified together; neither stands in for the other.".into();
                    revised.aspiration = "Develop compatible boundary behavior while keeping both parent meanings answerable to evidence.".into();
                    revised.discriminator = "Distinguish evidence for each parent instead of treating finite passes as joint fulfillment.".into();
                    revised.development = Some(crate::engine::deliberation::relational::Development {
                        original_revision: origin.0, prior_revision: origin.0,
                        retained_meaning: "Both published parent meanings remain wanted.".into(),
                        return_reveals: "Authored independent review judges the second requirement unmet; finite outcomes are simulated.".into(),
                        unresolved: "The reason for the judgment and real-world correctness remain unknown.".into(),
                        proposed_development: Some("Keep the relationship accountable to distinct evidence for both parents, without declaring fulfillment.".into()),
                    });
                    let proposal = propose(
                        &e,
                        Suggestion::Relational {
                            binding: serde_json::from_value(context["binding"].clone())?,
                            proposal: revised,
                        },
                    )?;
                    e.project_relational_admit(e.store.head_root()?, proposal, 0)?;
                    let control = Response {
                        protocol: PROTOCOL.into(),
                        state_root: e.store.head_root()?,
                        attention_binding: e.proposal_attention_binding()?,
                        proposals: vec![Suggestion::Experiment {
                            experiment: experiment(&e)?,
                        }],
                        pursuit_expectations: Default::default(),
                        common_event_forecasts: Default::default(),
                    };
                    fs::write(
                        root.join("next-control.json"),
                        serde_json::to_vec(&control)?,
                    )?;
                }
                let request = e.proposal_request()?;
                fs::write(
                    root.join(format!("{label}.request.json")),
                    serde_json::to_vec(&request)?,
                )?;
            }
        } else if mode == "validate" || mode == "validate_next" {
            let label = std::env::var("TSR_SEMANTIC_SHADOW_ARM")?;
            ensure!(
                matches!(label.as_str(), "reject" | "unknown"),
                "fixture arm"
            );
            let e = Engine::open(&root.join(&label))?;
            let before = e.store.head_root()?;
            let body = fs::read(std::env::var("TSR_SEMANTIC_SHADOW_RESPONSE")?)?;
            ensure!(body.len() <= 32_768, "fixture response bound");
            let response: Response = serde_json::from_slice(&body)?;
            if mode == "validate" {
                ensure!(
                    response.proposals.len() == 1
                        && matches!(response.proposals[0], Suggestion::Relational { .. }),
                    "revision only"
                );
            } else {
                ensure!(
                    label == "reject"
                        && response.proposals.iter().all(|p| matches!(
                            p,
                            Suggestion::Existing { .. }
                                | Suggestion::ExactEdit { .. }
                                | Suggestion::Experiment { .. }
                        )),
                    "next-proposal kinds only"
                );
            }
            let result = e.evaluate_proposal(response, false)?;
            ensure!(
                !result.committed && e.store.head_root()? == before,
                "readonly validation changed history"
            );
            let output = std::path::PathBuf::from(std::env::var("TSR_SEMANTIC_SHADOW_OUTPUT")?);
            ensure!(
                output
                    .parent()
                    .unwrap()
                    .canonicalize()?
                    .starts_with(root.canonicalize()?),
                "fixture output scope"
            );
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(output)?;
            std::io::Write::write_all(&mut file, &serde_json::to_vec(&result)?)?;
        } else {
            anyhow::bail!("unknown fixture mode");
        }
        Ok(())
    }
    #[test]
    fn relational_reconsiders_independent_semantic_return_without_new_execution() -> Result<()> {
        let (d, e) = setup()?;
        verify(
            &e,
            &CargoFixture {
                failed: false,
                broken: false,
                extra: false,
            },
        )?;
        let (binding, p) = hypothesis(&e)?;
        let proposal = propose(
            &e,
            Suggestion::Relational {
                binding,
                proposal: p.clone(),
            },
        )?;
        let admission = e.project_relational_admit(e.store.head_root()?, proposal, 0)?;
        let before = e.project_relational_context()?;
        assert_eq!(before["inputs"]["version"], "project-relational@v1");
        assert!(before["inputs"].get("semantic_return").is_none());
        assert_eq!(
            before["inputs"]
                .as_object()
                .unwrap()
                .keys()
                .map(String::as_str)
                .collect::<BTreeSet<_>>(),
            BTreeSet::from([
                "version",
                "snapshot",
                "requirements_revision",
                "parents",
                "returns",
                "supplemental"
            ])
        );
        for parent in before["inputs"]["parents"].as_array().unwrap() {
            assert_eq!(
                parent
                    .as_object()
                    .unwrap()
                    .keys()
                    .map(String::as_str)
                    .collect::<BTreeSet<_>>(),
                BTreeSet::from(["id", "requirement", "meaning"])
            );
        }
        let second = e.acceptance_current_aims()?["second"];
        e.append(
            EventKind::StatusSet {
                preference: second,
                status: PreferenceStatus::Withdrawn,
                by: Source::Operator,
            },
            Source::Operator,
        )?;
        let withdrawn = e.project_relational_context()?;
        assert_ne!(before["binding"], withdrawn["binding"]);
        assert_eq!(withdrawn["inputs"]["parents"].as_array().unwrap().len(), 1);
        e.append(
            EventKind::StatusSet {
                preference: second,
                status: PreferenceStatus::Open,
                by: Source::Operator,
            },
            Source::Operator,
        )?;
        assert_eq!(
            before,
            Engine::open(d.path())?.project_relational_context()?
        );
        let current = e.project_acceptance_status()?;
        e.project_acceptance_review(acceptance::Review {
            snapshot: current.snapshot,
            requirements_revision: current.requirements_revision.unwrap(),
            test_receipt: current.test_receipt,
            decision: acceptance::Decision::Reject,
            evidence: BTreeMap::from([
                (
                    "first".into(),
                    acceptance::Evidence {
                        met: true,
                        evidence: "PRIVATE semantic reviewer first evidence".into(),
                    },
                ),
                (
                    "second".into(),
                    acceptance::Evidence {
                        met: false,
                        evidence: "PRIVATE semantic reviewer contrary evidence".into(),
                    },
                ),
            ]),
        })?;
        let restarted = Engine::open(d.path())?;
        let after = restarted.project_relational_context()?;
        assert_eq!(before["inputs"]["snapshot"], after["inputs"]["snapshot"]);
        assert_eq!(before["inputs"]["returns"], after["inputs"]["returns"]);
        assert_ne!(
            before["binding"], after["binding"],
            "independent semantic return must invalidate the old relational evidence binding"
        );
        assert!(after["guidance"].as_array().unwrap().is_empty());
        assert_eq!(
            after["inputs"]["semantic_return"]["unmet"],
            serde_json::json!(["second"])
        );
        assert_eq!(
            after["inputs"]["semantic_return"]["unknown"],
            serde_json::json!([])
        );
        assert_eq!(after["inputs"]["semantic_return"]["applicable"], true);
        assert_eq!(
            after,
            restarted.proposal_request()?.coding.unwrap()["relational"]
        );
        assert!(!serde_json::to_string(&after)?.contains("PRIVATE"));
        let mut revised = p;
        revised.revises = Some(admission.0);
        revised.development = Some(crate::engine::deliberation::relational::Development {
            original_revision: admission.0,
            prior_revision: admission.0,
            retained_meaning: revised.aspiration.clone(),
            return_reveals:
                "Independent fixture review rejects the second aim despite finite passing tests"
                    .into(),
            unresolved: "No new source or execution evidence".into(),
            proposed_development: None,
        });
        assert!(restarted
            .validate_project_relational(binding, &revised)
            .is_err());
        restarted.validate_project_relational(
            serde_json::from_value(after["binding"].clone())?,
            &revised,
        )?;
        // Authored revision enters the ordinary next-proposal context after admission,
        // without inventing evidence, action authority, or an aspiration score.
        let decision_options = |engine: &Engine| -> Result<serde_json::Value> {
            let mut options = serde_json::to_value(engine.adaptive_state()?.options)?;
            for option in options.as_array_mut().unwrap() {
                if let Some(cost) = option.get_mut("resource_cost") {
                    // New guidance events advance the cost audit prefix, not estimates.
                    assert!(cost["source_prefix"].is_string());
                    cost.as_object_mut().unwrap().remove("source_prefix");
                }
            }
            Ok(options)
        };
        let fixed_options = decision_options(&restarted)?;
        let fixed_expectations = serde_json::to_value(restarted.acceptance_kernel_inputs(None)?)?;
        let candidate = propose(
            &restarted,
            Suggestion::Relational {
                binding: serde_json::from_value(after["binding"].clone())?,
                proposal: revised.clone(),
            },
        )?;
        let new_admission =
            restarted.project_relational_admit(restarted.store.head_root()?, candidate, 0)?;
        let request = Engine::open(d.path())?.proposal_request()?;
        let guidance = &request.coding.as_ref().unwrap()["relational"]["guidance"];
        assert_eq!(guidance.as_array().unwrap().len(), 1);
        assert_eq!(guidance[0]["admission"], serde_json::json!(new_admission));
        assert_eq!(guidance[0]["proposal"], serde_json::to_value(&revised)?);
        assert_eq!(fixed_options, decision_options(&restarted)?);
        assert_eq!(
            fixed_expectations,
            serde_json::to_value(restarted.acceptance_kernel_inputs(None)?)?
        );
        assert!(request.pending.is_none());
        assert!(!restarted.project_acceptance_status()?.accepted);
        let current = restarted.project_acceptance_status()?;
        restarted.project_acceptance_review(acceptance::Review {
            snapshot: current.snapshot,
            requirements_revision: current.requirements_revision.unwrap(),
            test_receipt: current.test_receipt,
            decision: acceptance::Decision::Unresolved,
            evidence: BTreeMap::from([
                (
                    "first".into(),
                    acceptance::Evidence {
                        met: true,
                        evidence: "PRIVATE first stays met".into(),
                    },
                ),
                (
                    "second".into(),
                    acceptance::Evidence {
                        met: false,
                        evidence: "PRIVATE second now unknown, not a failure".into(),
                    },
                ),
            ]),
        })?;
        let unknown = Engine::open(d.path())?.project_relational_context()?;
        assert_ne!(after["binding"], unknown["binding"]);
        assert!(unknown["guidance"].as_array().unwrap().is_empty());
        assert_eq!(unknown["history"].as_array().unwrap().len(), 2);
        assert_eq!(
            unknown["inputs"]["semantic_return"]["unmet"],
            serde_json::json!([])
        );
        assert_eq!(
            unknown["inputs"]["semantic_return"]["unknown"],
            serde_json::json!(["second"])
        );
        assert!(!serde_json::to_string(&unknown)?.contains("PRIVATE"));
        let second = restarted.acceptance_current_aims()?["second"];
        restarted.append(
            EventKind::StatusSet {
                preference: second,
                status: PreferenceStatus::Withdrawn,
                by: Source::Operator,
            },
            Source::Operator,
        )?;
        let withdrawn = Engine::open(d.path())?.project_relational_context()?;
        assert_eq!(withdrawn["inputs"]["parents"].as_array().unwrap().len(), 1);
        assert!(withdrawn["inputs"]["semantic_return"]["unknown"]
            .as_array()
            .unwrap()
            .is_empty());
        assert!(restarted
            .validate_project_relational(
                serde_json::from_value(withdrawn["binding"].clone())?,
                &revised
            )
            .is_err());
        restarted.append(
            EventKind::StatusSet {
                preference: second,
                status: PreferenceStatus::Open,
                by: Source::Operator,
            },
            Source::Operator,
        )?;
        assert_eq!(
            unknown,
            Engine::open(d.path())?.project_relational_context()?
        );
        Ok(())
    }
    #[test]
    fn relational_semantic_projection_filters_private_dimensions_and_checks_provenance(
    ) -> Result<()> {
        let (d, e) = setup()?;
        e.project_acceptance_requirements(BTreeMap::from([
            ("first".into(), "Empty input handled".into()),
            ("second".into(), "Existing behavior unchanged".into()),
            (
                "PRIVATE_DIMENSION".into(),
                "PRIVATE requirement text".into(),
            ),
        ]))?;
        let current = e.project_acceptance_status()?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: current.requirements_revision.unwrap(),
            decision_record: None,
            guidance: BTreeMap::from([
                ("first".into(), "Empty input handled".into()),
                ("second".into(), "Existing behavior unchanged".into()),
            ]),
        })?;
        verify(
            &e,
            &CargoFixture {
                failed: false,
                broken: false,
                extra: false,
            },
        )?;
        let current = e.project_acceptance_status()?;
        let mut review = acceptance::Review {
            snapshot: current.snapshot,
            requirements_revision: current.requirements_revision.unwrap(),
            test_receipt: current.test_receipt,
            decision: acceptance::Decision::Reject,
            evidence: BTreeMap::from([
                (
                    "first".into(),
                    acceptance::Evidence {
                        met: true,
                        evidence: "PRIVATE first evidence".into(),
                    },
                ),
                (
                    "second".into(),
                    acceptance::Evidence {
                        met: true,
                        evidence: "PRIVATE second evidence".into(),
                    },
                ),
                (
                    "PRIVATE_DIMENSION".into(),
                    acceptance::Evidence {
                        met: false,
                        evidence: "PRIVATE third evidence".into(),
                    },
                ),
            ]),
        };
        e.project_acceptance_review(review.clone())?;
        let rejected = e.project_relational_context()?;
        assert!(!serde_json::to_string(&rejected)?.contains("PRIVATE"));
        assert_eq!(
            rejected["inputs"]["semantic_return"]["unmet"],
            serde_json::json!([])
        );
        review.decision = acceptance::Decision::Accept;
        review.evidence.get_mut("PRIVATE_DIMENSION").unwrap().met = true;
        e.project_acceptance_review(review.clone())?;
        let accepted = Engine::open(d.path())?.project_relational_context()?;
        assert_ne!(rejected["binding"], accepted["binding"]);
        assert!(e.project_acceptance_status()?.accepted);
        assert!(!serde_json::to_string(&accepted)?.contains("PRIVATE"));
        assert_eq!(accepted["inputs"]["semantic_return"]["applicable"], true);
        fs::write(
            d.path().join("coding-work/src/lib.rs"),
            "pub fn parse(_: &str) -> bool { false }\n",
        )?;
        let stale = Engine::open(d.path())?.project_relational_context()?;
        assert_eq!(stale["inputs"]["semantic_return"]["applicable"], false);
        assert_ne!(accepted["binding"], stale["binding"]);
        assert!(stale["inputs"]["returns"].as_array().unwrap().is_empty());
        review.decision = acceptance::Decision::Unresolved;
        e.adaptive_record(
            &Record::Coding {
                record: CodingRecord::AcceptanceReview { review },
            },
            Source::Generator,
        )?;
        assert!(Engine::open(d.path())?
            .project_relational_context()
            .is_err());
        Ok(())
    }
    #[test]
    fn browser_catalysts_reuse_reviewed_aspirations_without_new_authority() -> Result<()> {
        // Authored semantics and operator review only; no live emergence claim.
        let d = tempfile::tempdir()?;
        let e = Engine::open(d.path())?;
        e.project_initialize(browser::tests::spec())?;
        let meanings = BTreeMap::from([
            ("orientation".into(), "Find useful information".into()),
            (
                "discovery".into(),
                "Encounter unfamiliar perspectives".into(),
            ),
        ]);
        e.project_acceptance_requirements(meanings.clone())?;
        open(&e)?;
        let status = e.project_acceptance_status()?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: status.requirements_revision.unwrap(),
            decision_record: None,
            guidance: meanings,
        })?;
        let config = e.project_configuration()?;
        let (binding, first) = hypothesis(&e)?;
        let proposal = propose(
            &e,
            Suggestion::Relational {
                binding,
                proposal: first.clone(),
            },
        )?;
        assert!(e.project_relational_context()?["catalyst_references"]
            .as_array()
            .unwrap()
            .is_empty());
        let admitted = e.project_relational_admit(e.store.head_root()?, proposal, 0)?;
        let mut next = first.clone();
        next.relationship = "Authored further composition using the prior shared aspiration".into();
        next.aspiration = "Investigate a new possibility while preserving both contributors".into();
        next.catalysts = vec![admitted.0];
        let status = e.project_acceptance_status()?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: status.requirements_revision.unwrap(),
            decision_record: None,
            guidance: BTreeMap::from([
                (
                    "orientation".into(),
                    "Authored clarification: find useful information without losing context".into(),
                ),
                (
                    "discovery".into(),
                    "Encounter unfamiliar perspectives".into(),
                ),
            ]),
        })?;
        assert!(e.validate_project_relational(binding, &next).is_err());
        let context = e.project_relational_context()?;
        let binding = serde_json::from_value(context["binding"].clone())?;
        assert_eq!(
            context["catalyst_references"][0]["admission"],
            serde_json::json!(admitted)
        );
        assert_eq!(context["catalyst_references"][0]["historical"], true);
        assert!(context["guidance"].as_array().unwrap().is_empty());
        e.validate_project_relational(binding, &next)?;
        for refs in [vec![Hash::of(&[b"foreign"])], vec![admitted.0, admitted.0]] {
            let mut bad = next.clone();
            bad.catalysts = refs;
            assert!(e.validate_project_relational(binding, &bad).is_err());
        }
        let mut bad = next.clone();
        bad.parents.pop();
        assert!(e.validate_project_relational(binding, &bad).is_err());
        let generated = propose(
            &e,
            Suggestion::Relational {
                binding,
                proposal: next.clone(),
            },
        )?;
        let composed = e.project_relational_admit(e.store.head_root()?, generated, 0)?;
        drop(e);
        let e = Engine::open(d.path())?;
        let request = e.proposal_request()?;
        let relation = &request.coding.as_ref().unwrap()["relational"];
        assert_eq!(relation["history"].as_array().unwrap().len(), 2);
        assert_eq!(
            relation["history"][1]["proposal"]["catalysts"],
            serde_json::json!([admitted.0])
        );
        assert_eq!(
            relation["history"][1]["proposal"]["parents"],
            serde_json::json!(first.parents)
        );
        assert_eq!(
            relation["history"][1]["admission"],
            serde_json::json!(composed)
        );
        assert_eq!(
            serde_json::to_value(e.project_configuration()?)?,
            serde_json::to_value(config)?
        );
        assert!(e.project_acceptance_status()?.decision_record.is_none());
        // Administrative suppression blocks future reuse; it does not delete parents/history.
        e.project_relational_status(e.store.head_root()?, admitted, true)?;
        assert!(e.validate_project_relational(binding, &next).is_err());
        assert_eq!(
            e.project_relational_context()?["history"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            e.project_relational_context()?["inputs"]["parents"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        Ok(())
    }
    #[test]
    fn b91_project_bridges_return_review_restart_and_separate_authority() -> Result<()> {
        let (d, e) = setup()?;
        verify(
            &e,
            &CargoFixture {
                failed: true,
                broken: false,
                extra: false,
            },
        )?;
        let before = e.project_configuration()?;
        let (binding, p) = hypothesis(&e)?;
        let generated = propose(
            &e,
            Suggestion::Relational {
                binding,
                proposal: p.clone(),
            },
        )?;
        assert!(e.project_relational_context()?["guidance"]
            .as_array()
            .unwrap()
            .is_empty());
        let admission = e.project_relational_admit(e.store.head_root()?, generated, 0)?;
        assert!(e
            .project_relational_admit(e.store.head_root()?, generated, 0)
            .is_err());
        let restarted = Engine::open(d.path())?;
        assert_eq!(
            restarted.proposal_request()?.coding.unwrap()["relational"]["guidance"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        let x = experiment(&restarted)?;
        let proposal = propose(&restarted, Suggestion::Experiment { experiment: x })?;
        assert!(restarted.project_supplemental_view()?["records"]
            .as_array()
            .unwrap()
            .is_empty());
        let id = restarted.project_experiment_admit(
            restarted.store.head_root()?,
            proposal,
            0,
            ExperimentReview {
                author: "check author".into(),
                reviewer: "mapping reviewer".into(),
                generator: "generator role".into(),
                review: "Fixture review only".into(),
            },
        )?;
        let checks = tempfile::tempdir()?;
        let other = Engine::open(checks.path())?;
        other.project_initialize(restarted.project_supplemental_spec(id)?)?;
        assert!(other
            .project_admit_verification(
                other.store.head_root()?,
                other.project_acceptance_status()?.snapshot
            )
            .is_err());
        open(&other)?;
        let receipt = verify(
            &other,
            &CargoFixture {
                failed: true,
                broken: false,
                extra: true,
            },
        )?;
        let imported = restarted.project_supplemental_link(id, &other, receipt)?;
        assert_eq!(
            restarted.project_supplemental_link(id, &other, receipt)?,
            imported
        );
        let next = Engine::open(d.path())?.proposal_request()?;
        let ctx = &next.coding.as_ref().unwrap()["relational"];
        assert!(ctx["guidance"].as_array().unwrap().is_empty()); // old hypothesis awaits fresh return review
        assert_eq!(
            ctx["inputs"]["supplemental"][0]["findings"]["second"],
            "not_met"
        );
        let origin = &ctx["inputs"]["supplemental"][0]["generated_origin"];
        assert_eq!(origin["proposal"], serde_json::to_value(proposal)?);
        assert_eq!(origin["index"], 0);
        assert_eq!(
            origin["bundle_digest"],
            ctx["inputs"]["supplemental"][0]["bundle_digest"]
        );
        assert!(!serde_json::to_string(origin)?.contains("Fixture review only"));
        assert!(!restarted.project_acceptance_status()?.accepted);
        assert_eq!(
            serde_json::to_value(before)?,
            serde_json::to_value(restarted.project_configuration()?)?
        );
        let entries = restarted.adaptive_state()?.preferences;
        for aim in restarted.acceptance_current_aims()?.values() {
            assert!(entries
                .iter()
                .find(|e| e.preference.id == *aim)
                .unwrap()
                .is_open());
        }
        let mut revised = p.clone();
        revised.revises = Some(admission.0);
        revised.development = Some(crate::engine::deliberation::relational::Development {
            original_revision: admission.0,
            prior_revision: admission.0,
            retained_meaning: p.aspiration.clone(),
            return_reveals: "Fixture check found a second-aim counterexample".into(),
            unresolved: "No universal equivalence or successful patch established".into(),
            proposed_development: None,
        });
        let id = propose(
            &restarted,
            Suggestion::Relational {
                binding: serde_json::from_value(ctx["binding"].clone())?,
                proposal: revised,
            },
        )?;
        restarted.project_relational_admit(restarted.store.head_root()?, id, 0)?;
        assert_eq!(
            restarted.proposal_request()?.coding.unwrap()["relational"]["guidance"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert!(restarted.adaptive_state()?.pending.is_none()); // no automatic patch/action
        Ok(())
    }
    #[test]
    fn b91_generated_experiment_origin_rejects_forged_source_and_content() -> Result<()> {
        for source in [Source::Generator, Source::Operator] {
            let (_d, e) = setup()?;
            let x = experiment(&e)?;
            let binding = x.binding;
            let proposal = propose(
                &e,
                Suggestion::Experiment {
                    experiment: x.clone(),
                },
            )?;
            // An independently admitted bundle is not automatically a generated experiment.
            let mut harness = x.harness;
            if source == Source::Operator {
                harness.values_mut().next().unwrap().push_str("// changed");
            }
            let admission = e.project_supplemental_admit(Bundle {
                version: "supplemental-cargo@v2".into(),
                snapshot: x.snapshot,
                requirements_revision: x.requirements_revision,
                author: "fixture author".into(),
                reviewer: "fixture reviewer".into(),
                generator: "fixture generator".into(),
                review: "Private review must not enter projection".into(),
                harness,
                probe_entry: String::new(),
                cases: x.cases,
                mapping: x.mapping,
            })?;
            assert!(e
                .project_experiment_origins(&e.project_supplemental_view()?)?
                .is_empty());
            e.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::ExperimentOrigin {
                        proposal,
                        index: 0,
                        admission,
                        binding,
                    },
                },
                source,
            )?;
            assert!(e.project_relational_context().is_err());
        }
        Ok(())
    }
    #[test]
    fn b91_project_bridge_stale_private_withdrawal_and_invalid_experiment() -> Result<()> {
        let (d, e) = setup()?;
        let (binding, p) = hypothesis(&e)?;
        let mut bad = p.clone();
        bad.parents[1] = bad.parents[0];
        assert!(e.validate_project_relational(binding, &bad).is_err());
        bad = p.clone();
        bad.revises = Some(Hash::of(&[b"future"]));
        assert!(e.validate_project_relational(binding, &bad).is_err());
        let generated = propose(
            &e,
            Suggestion::Relational {
                binding,
                proposal: p,
            },
        )?;
        let id = e.project_relational_admit(e.store.head_root()?, generated, 0)?;
        e.project_relational_status(e.store.head_root()?, id, true)?;
        let root = e.store.head_root()?;
        e.project_relational_status(root, id, true)?;
        assert_eq!(root, e.store.head_root()?);
        assert!(e.project_relational_context()?["guidance"]
            .as_array()
            .unwrap()
            .is_empty());
        e.project_relational_status(root, id, false)?;
        let x = experiment(&e)?;
        for mode in 0..6 {
            let mut bad = x.clone();
            match mode {
                0 => bad.snapshot = Hash::of(&[b"stale"]),
                1 => bad.harness = BTreeMap::from([("Cargo.toml".into(), "overwrite".into())]),
                2 => bad.cases[0].id = "nonempty".into(),
                3 => bad.binding = Hash::of(&[b"stale"]),
                4 => bad.mapping.get_mut("second").unwrap().cases.clear(),
                _ => {
                    bad.harness =
                        BTreeMap::from([("tests/tsr_supplemental_../x.rs".into(), "bad".into())])
                }
            }
            assert!(e.validate_project_experiment(&bad).is_err(), "mode {mode}");
        }
        fs::write(
            d.path().join("coding-work/src/lib.rs"),
            "pub fn parse(_: &str) -> bool { false }\n",
        )?;
        assert!(e.validate_project_experiment(&x).is_err());
        assert!(e.project_relational_context()?["guidance"]
            .as_array()
            .unwrap()
            .is_empty());
        Ok(())
    }
    #[test]
    fn b91_cargo_experiment_unknown_and_finite_pass_are_not_semantic_success() -> Result<()> {
        for broken in [false, true] {
            let (_d, e) = setup()?;
            let id = propose(
                &e,
                Suggestion::Experiment {
                    experiment: experiment(&e)?,
                },
            )?;
            let admission = e.project_experiment_admit(
                e.store.head_root()?,
                id,
                0,
                ExperimentReview {
                    author: "author".into(),
                    reviewer: "reviewer".into(),
                    generator: "generator".into(),
                    review: "Fixture mapping".into(),
                },
            )?;
            let d = tempfile::tempdir()?;
            let other = Engine::open(d.path())?;
            other.project_initialize(e.project_supplemental_spec(admission)?)?;
            open(&other)?;
            let receipt = verify(
                &other,
                &CargoFixture {
                    failed: false,
                    broken,
                    extra: true,
                },
            )?;
            e.project_supplemental_link(admission, &other, receipt)?;
            let view = e.project_supplemental_view()?;
            assert_eq!(
                view["current_observations"][0]["findings"]["second"],
                "unknown"
            );
            assert_eq!(
                view["current_observations"][0]["cases"]["boundary"],
                if broken {
                    "unknown"
                } else {
                    "passed_finite_case"
                }
            );
            assert!(!e.project_acceptance_status()?.accepted);
        }
        Ok(())
    }
    #[test]
    fn native_return_projection_preserves_authenticated_source_epoch() -> Result<()> {
        // Authored receipts only; no model call, container or game execution.
        let (d, e) = setup()?;
        let snapshot = e.project_acceptance_status()?.snapshot;
        let mut current_receipt = None;
        for source_after in [
            None,
            Some(Hash::of(&[b"authored other source"])),
            Some(snapshot),
        ] {
            let id = e.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::Receipt {
                        commitment: EventId(Hash::of(&[b"authored commitment"])),
                        verdict: Verdict::Passed,
                        runs: vec![],
                        source_after,
                        observations: vec![],
                    },
                },
                Source::Rules,
            )?;
            if source_after == Some(snapshot) {
                current_receipt = Some(id);
            }
        }
        let context = e.project_relational_context()?;
        let returns = context["inputs"]["returns"].as_array().unwrap();
        assert_eq!(returns.len(), 1);
        assert_eq!(
            returns[0]["receipt"],
            serde_json::json!(current_receipt.unwrap())
        );
        assert_eq!(returns[0]["snapshot"], serde_json::json!(snapshot));
        assert_eq!(
            Engine::open(d.path())?.project_relational_context()?,
            context
        );
        fs::write(
            d.path().join("coding-work/src/lib.rs"),
            "pub fn parse(_: &str) -> bool { false }\n",
        )?;
        let changed = Engine::open(d.path())?.project_relational_context()?;
        assert!(changed["inputs"]["returns"].as_array().unwrap().is_empty());
        Ok(())
    }
    #[test]
    fn b91_generated_records_cannot_self_admit_withdraw_or_forge_return() -> Result<()> {
        for mode in 0..4 {
            let (_d, e) = setup()?;
            let (binding, p) = hypothesis(&e)?;
            let id = propose(
                &e,
                Suggestion::Relational {
                    binding,
                    proposal: p,
                },
            )?;
            if mode == 0 {
                e.adaptive_record(
                    &Record::Coding {
                        record: CodingRecord::RelationalAdmission {
                            proposal: id,
                            index: 0,
                            binding,
                        },
                    },
                    Source::Generator,
                )?;
            } else if mode == 1 {
                let admission = e.project_relational_admit(e.store.head_root()?, id, 0)?;
                e.adaptive_record(
                    &Record::Coding {
                        record: CodingRecord::RelationalStatus {
                            admission,
                            withdrawn: false,
                        },
                    },
                    Source::Generator,
                )?;
            } else if mode == 2 {
                e.adaptive_record(
                    &Record::Coding {
                        record: CodingRecord::Receipt {
                            commitment: id,
                            verdict: Verdict::Passed,
                            runs: vec![],
                            source_after: Some(e.project_acceptance_status()?.snapshot),
                            observations: vec![],
                        },
                    },
                    Source::Generator,
                )?;
            } else {
                let proposal = e
                    .records()?
                    .into_iter()
                    .find(|(key, _)| *key == id)
                    .unwrap()
                    .1;
                let forged = e.adaptive_record(&proposal, Source::Operator)?;
                assert!(e
                    .project_relational_admit(e.store.head_root()?, forged, 0)
                    .is_err());
                continue;
            }
            assert!(e.project_relational_context().is_err(), "mode {mode}");
        }
        Ok(())
    }
    #[test]
    fn b91_private_supplemental_mapping_is_not_proposal_context() -> Result<()> {
        let (_d, e) = setup()?;
        e.project_acceptance_requirements(BTreeMap::from([
            ("first".into(), "Empty input handled".into()),
            ("second".into(), "Existing behavior unchanged".into()),
            (
                "PRIVATE_CRITERION_SENTINEL".into(),
                "PRIVATE_MEANING_SENTINEL".into(),
            ),
        ]))?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: e
                .project_acceptance_status()?
                .requirements_revision
                .unwrap(),
            decision_record: None,
            guidance: BTreeMap::from([
                ("first".into(), "Empty input handled".into()),
                ("second".into(), "Existing behavior unchanged".into()),
            ]),
        })?;
        let mut x = experiment(&e)?;
        let mut mapping = x.mapping.remove("second").unwrap();
        mapping.rationale = "PRIVATE_REVIEW_SENTINEL".into();
        x.mapping
            .insert("PRIVATE_CRITERION_SENTINEL".into(), mapping);
        assert!(e.validate_project_experiment(&x).is_err());
        let id = e.project_supplemental_admit(Bundle {
            version: "supplemental-cargo@v2".into(),
            snapshot: x.snapshot,
            requirements_revision: x.requirements_revision,
            author: "author".into(),
            reviewer: "reviewer".into(),
            generator: "generator".into(),
            review: "Private reviewed check".into(),
            harness: x.harness,
            probe_entry: String::new(),
            cases: x.cases,
            mapping: x.mapping,
        })?;
        let d = tempfile::tempdir()?;
        let other = Engine::open(d.path())?;
        other.project_initialize(e.project_supplemental_spec(id)?)?;
        open(&other)?;
        let receipt = verify(
            &other,
            &CargoFixture {
                failed: true,
                broken: false,
                extra: true,
            },
        )?;
        e.project_supplemental_link(id, &other, receipt)?;
        assert!(serde_json::to_string(&e.project_supplemental_view()?)?
            .contains("PRIVATE_CRITERION_SENTINEL"));
        let context = e.proposal_request()?;
        assert!(!serde_json::to_string(&context)?.contains("PRIVATE_"));
        assert!(
            context.coding.unwrap()["relational"]["inputs"]["supplemental"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        Ok(())
    }
    #[test]
    fn b91_cargo_duplicate_names_or_missing_inventory_cannot_establish_return() -> Result<()> {
        let cases = vec![ProbeCase {
            id: "boundary".into(),
            args: vec![],
            expected_stdout: String::new(),
        }];
        let output=b"running 2 tests\ntest boundary ... ok\ntest boundary ... ok\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s\n";
        assert_eq!(
            cargo::classify_bounded(&RunResult::from_output(0, output, b"", 1), &cases, 4096).0,
            Verdict::Unavailable
        );
        let (_d, e) = setup()?;
        let id = propose(
            &e,
            Suggestion::Experiment {
                experiment: experiment(&e)?,
            },
        )?;
        let admission = e.project_experiment_admit(
            e.store.head_root()?,
            id,
            0,
            ExperimentReview {
                author: "author".into(),
                reviewer: "reviewer".into(),
                generator: "generator".into(),
                review: "Fixture mapping".into(),
            },
        )?;
        let d = tempfile::tempdir()?;
        let other = Engine::open(d.path())?;
        other.project_initialize(e.project_supplemental_spec(admission)?)?;
        open(&other)?;
        let receipt = verify(
            &other,
            &CargoFixture {
                failed: false,
                broken: false,
                extra: false,
            },
        )?;
        e.project_supplemental_link(admission, &other, receipt)?;
        assert_eq!(
            e.project_supplemental_view()?["current_observations"][0]["cases"]["boundary"],
            "unknown"
        );
        Ok(())
    }
}
