//! Reviewed Contrast -> new preference identity. No new criteria or action authority.
//! PRD B2.1/B6/B8, checked-in interpretation; not an independent reading of the book.
use super::*;
use anyhow::Context;
use serde_json::{json, Value};
use tsr_domain::{Contrast, ContrastId};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub binding: Hash,
    pub aspiration: EventId,
    pub contrast: ContrastId,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Admission {
    pub choice: Choice,
    pub preference: Preference,
    pub roots: Vec<PreferenceId>,
    pub contributors: Vec<Value>,
    pub snapshot: Hash,
    pub requirements_revision: EventId,
    pub proposal: EventId,
    pub review: cycle::Review,
}
/// A derived preference, the preferences it depends on, and its supporting records.
type DerivedCandidate = (Preference, Vec<PreferenceId>, Vec<Value>);

impl Engine {
    /// Only the current authenticated semantic return's explicitly unmet public
    /// criteria can cross. Unknown findings and surprise cannot supply Contrast.
    pub(crate) fn project_current_contrast(&self) -> Result<Option<Contrast>> {
        if self.criterion_policy()?.is_some() {
            return self.criterion_current_contrast();
        }
        let status = self.project_acceptance_status()?;
        let view = self.acceptance_cycle()?;
        if view["review_evidence_applicable"] != true {
            return Ok(None);
        }
        let Some(review) = status.decision_record else {
            return Ok(None);
        };
        self.require_project_bridge_source(review, Source::Operator)?;
        let published = view["guidance"].as_object().context("published guidance")?;
        let unmet: BTreeSet<CriterionId> = view["unmet"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|v| v.as_str())
            .filter(|k| published.contains_key(*k))
            .map(|k| CriterionId(Hash::of(&[b"acceptance", k.as_bytes()])))
            .collect();
        if unmet.is_empty() {
            return Ok(None);
        }
        let found: Vec<_> = self
            .events()?
            .into_iter()
            .filter_map(|(_, e)| match e.kind {
                EventKind::Observed {
                    observation:
                        Observation::Return {
                            contrast: Some(c), ..
                        },
                } if e.source == Source::Operator
                    && c.from_return == review
                    && !c.unmet.is_empty()
                    && c.unmet.iter().all(|(id, _)| unmet.contains(id)) =>
                {
                    Some(c)
                }
                _ => None,
            })
            .collect();
        ensure!(found.len() <= 1, "ambiguous current Contrast");
        Ok(found.into_iter().next())
    }
    /// Historical records persist. Operational eligibility follows contributors,
    /// never an invented fulfillment predicate or Operator Open event.
    pub(crate) fn project_generated_preferences(&self, parents: &[Value]) -> Result<Vec<Value>> {
        let mut active: BTreeSet<PreferenceId> = parents
            .iter()
            .map(|p| serde_json::from_value(p["id"].clone()))
            .collect::<std::result::Result<_, _>>()?;
        let events = self.events()?;
        let ledger = Ledger::new(&self.store).collective()?;
        let mut result = vec![];
        for (id, r) in self.records()? {
            let (
                preference,
                roots,
                contributors,
                snapshot,
                requirements_revision,
                proposal,
                review,
                aspiration,
                basis,
            ) = match r {
                Record::Coding {
                    record: CodingRecord::GeneratedPreferenceAdmission { admission: a },
                } => {
                    ensure!(events.iter().any(|(_,event)|event.source==Source::Generator && matches!(&event.kind,EventKind::Expanded{expansion:x} if x.produced==a.preference.id && x.from_contrast==a.choice.contrast && x.text==a.preference.text && x.parents==a.preference.parents && x.catalyzed_by==a.preference.parents && x.scope==a.preference.scope && x.origin==Source::Generator)),"generated preference expansion provenance");
                    (
                        a.preference,
                        a.roots,
                        a.contributors,
                        a.snapshot,
                        a.requirements_revision,
                        a.proposal,
                        a.review,
                        Some(a.choice.aspiration),
                        json!({"contrast":a.choice.contrast}),
                    )
                }
                Record::Coding {
                    record: CodingRecord::SuccessPreferenceAdmission { admission: a },
                } => {
                    ensure!(events.iter().any(|(_,event)|event.source==Source::Generator && matches!(&event.kind,EventKind::SuccessExpanded{expansion:x} if x.preference==a.preference && x.success==a.choice.success)),"success preference expansion provenance");
                    tsr_ledger::success::basis(&events, Self::task(), a.choice.success)?;
                    (
                        a.preference,
                        a.roots,
                        a.contributors,
                        a.snapshot,
                        a.requirements_revision,
                        a.proposal,
                        a.review,
                        Some(a.choice.aspiration),
                        json!({"success":a.choice.success,"success_scope":"independently assessed artifact criteria only; no human fulfillment"}),
                    )
                }
                Record::Coding {
                    record: CodingRecord::ProvisionalPreferenceAdmission { admission: a },
                } => {
                    let contributors_digest = Hash::of(&[&serde_json::to_vec(&a.contributors)?]);
                    ensure!(events.iter().any(|(_,event)|event.source==Source::Generator && matches!(&event.kind,EventKind::ProvisionalExpanded{expansion:x} if x.preference==a.preference && x.proposal==a.proposal && x.context_binding==a.choice.binding && x.source_snapshot==a.snapshot && x.requirements_revision==a.requirements_revision && x.contributors_digest==contributors_digest && x.rationale==a.choice.rationale)),"provisional preference expansion provenance");
                    (
                        a.preference,
                        a.roots,
                        a.contributors,
                        a.snapshot,
                        a.requirements_revision,
                        a.proposal,
                        a.review,
                        None,
                        json!({"provisional":{"proposal":a.proposal,"context_binding":a.choice.binding,"contributors_digest":contributors_digest,"rationale":a.choice.rationale,"fulfillment":"unknown","attribution":"generated wanting, not user testimony or empirical success"}}),
                    )
                }
                _ => continue,
            };
            self.require_project_bridge_source(id, Source::Council)?;
            let canonical = Preference::from_admitted_text(
                preference.text.clone(),
                preference.scope.clone(),
                preference.parents.clone(),
                Source::Generator,
            )?;
            ensure!(
                canonical == preference && browser::narrative(&canonical.text),
                "generated preference content identity"
            );
            let eligible = ledger.get(&preference.id).is_some_and(|entry| {
                matches!(
                    entry.status,
                    Some(PreferenceStatus::Open | PreferenceStatus::Fulfilled)
                )
            }) && preference.parents.iter().all(|p| active.contains(p))
                && roots.iter().all(|p| active.contains(p));
            if eligible {
                active.insert(preference.id);
            }
            let mut value = json!({"admission":id,"id":preference.id,"meaning":preference.text,"parents":preference.parents,"original_contributors":roots,"contributors":contributors,"aspiration":aspiration,"snapshot":snapshot,"requirements_revision":requirements_revision,"proposal":proposal,"reviewer":review.reviewer,"generator":review.generator,"review":review,"eligible":eligible,"fulfillment":"unknown","grants_authority":false});
            value
                .as_object_mut()
                .unwrap()
                .extend(basis.as_object().unwrap().clone());
            result.push(value);
        }
        Ok(result)
    }
    pub(crate) fn contrast_preference_parts(
        &self,
        c: &Choice,
    ) -> Result<(Preference, Vec<PreferenceId>, Vec<Value>)> {
        ensure!(
            self.project_configuration()?.browser,
            "contrast preference requires browser profile"
        );
        let ctx = self.project_relational_context()?;
        ensure!(
            ctx["binding"] == json!(c.binding),
            "stale contrast preference binding"
        );
        let contrast = self
            .project_current_contrast()?
            .context("no current known unmet Contrast")?;
        ensure!(contrast.id == c.contrast, "foreign/stale Contrast");
        let (preference, roots, contributors) =
            self.derived_preference_parts(&ctx, c.aspiration)?;
        ensure!(
            self.contrast_covers_contributors(&contrast, &roots)?,
            "Contrast unrelated to contributing desires"
        );
        Ok((preference, roots, contributors))
    }
    pub(crate) fn contrast_covers_contributors(
        &self,
        contrast: &Contrast,
        roots: &[PreferenceId],
    ) -> Result<bool> {
        Ok(self.acceptance_current_aims()?.iter().any(|(key, id)| {
            roots.contains(id)
                && contrast.unmet.iter().any(|(criterion, _)| {
                    *criterion == CriterionId(Hash::of(&[b"acceptance", key.as_bytes()]))
                })
        }))
    }
    pub(crate) fn derived_preference_parts(
        &self,
        ctx: &Value,
        aspiration_id: EventId,
    ) -> Result<(Preference, Vec<PreferenceId>, Vec<Value>)> {
        self.derived_preference_candidate(ctx, aspiration_id)?
            .context("insufficient ancestry or duplicate generated preference")
    }
    pub(crate) fn derived_preference_candidate(
        &self,
        ctx: &Value,
        aspiration_id: EventId,
    ) -> Result<Option<DerivedCandidate>> {
        let aspiration = ctx["guidance"]
            .as_array()
            .context("guidance")?
            .iter()
            .find(|r| r["admission"] == json!(aspiration_id))
            .context("aspiration not currently reviewed and applicable")?;
        let p: crate::engine::deliberation::relational::Proposal =
            serde_json::from_value(aspiration["proposal"].clone())?;
        let mut parents = p.parents.clone();
        parents.sort();
        parents.dedup();
        let inputs = &ctx["inputs"];
        let originals = inputs
            .get("acceptance_parents")
            .unwrap_or(&inputs["parents"]);
        let original_ids: BTreeSet<PreferenceId> = originals
            .as_array()
            .context("original parents")?
            .iter()
            .map(|p| serde_json::from_value(p["id"].clone()))
            .collect::<std::result::Result<_, _>>()?;
        let mut roots = BTreeSet::new();
        let mut contributors = vec![];
        for parent in &parents {
            let entry = inputs["parents"]
                .as_array()
                .unwrap()
                .iter()
                .find(|p| p["id"] == json!(parent))
                .context("inactive contributor")?;
            // Freeze the exact meaning and immutable reference used, never an
            // expanded ancestry tree. Full records remain in append-only history.
            contributors.push(json!({
                "id":parent,"meaning":entry["meaning"],"revision":entry["revision"],
                "admission":entry["admission"],"requirement":entry["requirement"],
                "parents":entry.get("parents").cloned().unwrap_or(json!([])),
                "original_contributors":entry.get("original_contributors").cloned().unwrap_or(json!([parent]))
            }));
            if original_ids.contains(parent) {
                roots.insert(*parent);
            } else {
                roots.extend(serde_json::from_value::<Vec<PreferenceId>>(
                    entry["original_contributors"].clone(),
                )?);
            }
        }
        ensure!(
            roots.is_subset(&original_ids),
            "foreign original contributors"
        );
        if roots.len() < 2 {
            return Ok(None);
        }
        ensure!(
            browser::narrative(&p.aspiration),
            "empty or oversized aspiration"
        );
        let preference = Preference::from_admitted_text(
            p.aspiration,
            Scope::Task(Self::task()),
            parents,
            Source::Generator,
        )?;
        let roots: Vec<_> = roots.into_iter().collect();
        let norm = |s: &str| {
            s.split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase()
        };
        if Ledger::new(&self.store)
            .collective()?
            .entries
            .iter()
            .any(|e| e.preference.id == preference.id)
        {
            return Ok(None);
        }
        for old in inputs["generated_preferences"]
            .as_array()
            .into_iter()
            .flatten()
        {
            if old["original_contributors"] == json!(roots)
                && norm(old["meaning"].as_str().context("generated meaning")?)
                    == norm(&preference.text)
            {
                return Ok(None);
            }
        }
        Ok(Some((preference, roots, contributors)))
    }
    pub(crate) fn validate_contrast_preference(&self, c: &Choice) -> Result<()> {
        self.contrast_preference_parts(c).map(|_| ())
    }
    pub(crate) fn admit_contrast_preference(
        &self,
        c: &Choice,
        proposal: EventId,
        review: cycle::Review,
    ) -> Result<EventId> {
        let (preference, roots, contributors) = self.contrast_preference_parts(c)?;
        let status = self.project_acceptance_status()?;
        let ledger = Ledger::new(&self.store);
        let recorded_at = format!(
            "unix-ms:{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis()
        );
        let history = self.events()?;
        let expansion = ledger.propose_generated(
            &Self::task(),
            history.last().map(|(_, e)| vec![e.id]).unwrap_or_default(),
            Expansion {
                from_contrast: c.contrast,
                produced: preference.id,
                catalyzed_by: preference.parents.clone(),
                text: preference.text.clone(),
                scope: preference.scope.clone(),
                parents: preference.parents.clone(),
                origin: Source::Generator,
            },
            &recorded_at,
        )?;
        let requirements_revision = status
            .requirements_revision
            .context("requirements revision")?;
        let attestation = tsr_domain::PreferenceAdmissionReview {
            expansion,
            preference: preference.id,
            contrast: c.contrast,
            from_return: self
                .project_current_contrast()?
                .context("current Contrast")?
                .from_return,
            proposal,
            source_snapshot: status.snapshot,
            requirements_revision,
            request_binding: review.binding,
            response_digest: review.response_digest,
            reviewer: review.reviewer.clone(),
            generator: review.generator.clone(),
            decision: review.decision.clone(),
        };
        let record = self.adaptive_record(
            &Record::Coding {
                record: CodingRecord::GeneratedPreferenceAdmission {
                    admission: Box::new(Admission {
                        choice: c.clone(),
                        preference,
                        roots,
                        contributors,
                        snapshot: status.snapshot,
                        requirements_revision,
                        proposal,
                        review,
                    }),
                },
            },
            Source::Council,
        )?;
        let digest = self.blob(&serde_json::to_vec(&attestation)?)?;
        let evidence = self.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    source: tsr_ledger::reviewed::REVIEW_SOURCE.into(),
                    digest,
                },
            },
            Source::Council,
        )?;
        let admission_at = format!(
            "unix-ms:{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis()
        );
        ledger.admit_reviewed(
            &Self::task(),
            self.store.head_root()?,
            attestation,
            evidence,
            &admission_at,
        )?;
        Ok(record)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::adaptive::provider::{Response, Suggestion, PROTOCOL};
    fn setup(unknown: bool) -> Result<(tempfile::TempDir, Engine, Choice)> {
        let wants = BTreeMap::from([
            ("orientation".into(), "Keep navigation dependable".into()),
            (
                "discovery".into(),
                "Make unfamiliar perspectives accessible".into(),
            ),
        ]);
        setup_wants(unknown, wants)
    }
    fn setup_wants(
        unknown: bool,
        wants: BTreeMap<String, String>,
    ) -> Result<(tempfile::TempDir, Engine, Choice)> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(browser::tests::spec())?;
        e.project_acceptance_requirements(wants.clone())?;
        e.project_lifecycle_open(lifecycle::Admission {
            state_root: e.store.head_root()?,
            training: None,
            tool_ms_ceiling: 100_000,
            reason: "Authored offline expansion fixture; no provider/browser effects".into(),
            non_tool: Some(budget::Limits {
                model_calls: 10,
                model_tokens: 10000,
                wall_ms: 600000,
                patch_bytes: 10000,
                external_cost_microusd: 0,
            }),
        })?;
        let current = e.project_acceptance_status()?;
        e.project_acceptance_review(acceptance::Review {snapshot:current.snapshot,requirements_revision:current.requirements_revision.unwrap(),test_receipt:current.test_receipt,decision:if unknown {acceptance::Decision::Unresolved}else{acceptance::Decision::Reject},evidence:wants.keys().map(|k|(k.clone(),acceptance::Evidence {met:false,evidence:"Authored judgment only: the current fixture does not yet provide this condition; no live observation".into()})).collect()})?;
        let current = e.project_acceptance_status()?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: current.requirements_revision.unwrap(),
            decision_record: current.decision_record,
            guidance: wants,
        })?;
        e.project_resource_unpriced_admit(e.store.head_root()?)?;
        let ctx = e.project_relational_context()?;
        let p = crate::engine::deliberation::relational::Proposal {
            parents: serde_json::from_value(
                ctx["inputs"]["parents"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|x| x["id"].clone())
                    .collect::<Value>(),
            )?,
            relationship:
                "Authored hypothesis: orientation can make unfamiliar material approachable".into(),
            aspiration: format!(
                "Make unfamiliar routes approachable while keeping orientation recoverable.{}",
                " Authored narrative without any fulfillment assertion.".repeat(12)
            ),
            discriminator: "Observe whether unfamiliar paths retain a recoverable return route"
                .into(),
            revises: None,
            catalysts: vec![],
            development: None,
        };
        let binding = serde_json::from_value(ctx["binding"].clone())?;
        e.evaluate_proposal(
            Response {
                protocol: PROTOCOL.into(),
                state_root: e.store.head_root()?,
                attention_binding: e.proposal_attention_binding()?,
                proposals: vec![Suggestion::Relational {
                    binding,
                    proposal: p,
                }],
                pursuit_expectations: Default::default(),
                common_event_forecasts: Default::default(),
            },
            true,
        )?;
        let proposal = e.project_latest_proposal()?.unwrap();
        let aspiration = e.project_relational_admit(e.store.head_root()?, proposal, 0)?;
        let ctx = e.project_relational_context()?;
        let choice = Choice {
            binding: serde_json::from_value(ctx["binding"].clone())?,
            aspiration,
            contrast: e
                .project_current_contrast()?
                .map(|x| x.id)
                .unwrap_or(ContrastId(Hash::of(&[b"unknown is not contrast"]))),
        };
        Ok((dir, e, choice))
    }
    fn input(e: &Engine, c: Choice) -> Result<cycle::Advance> {
        let ctx = e.project_cycle_request()?;
        let response = Response {
            protocol: PROTOCOL.into(),
            state_root: e.store.head_root()?,
            attention_binding: e.proposal_attention_binding()?,
            proposals: vec![Suggestion::ContrastPreference { choice: c }],
            pursuit_expectations: Default::default(),
            common_event_forecasts: Default::default(),
        };
        let binding = serde_json::from_value(ctx["binding"].clone())?;
        Ok(cycle::Advance {
            binding,
            response: response.clone(),
            review: cycle::Review {
                binding,
                response_digest: Hash::of(&[&serde_json::to_vec(&response)?]),
                state_root: response.state_root,
                source_snapshot: serde_json::from_value(ctx["source_snapshot"].clone())?,
                reviewer: "authored independent semantic reviewer".into(),
                generator: "authored generator".into(),
                decision: "approve".into(),
            },
        })
    }
    #[test]
    fn reviewed_identity_later_composition_restart_and_no_fulfillment() -> Result<()> {
        let (dir, e, c) = setup(false)?;
        let old = e.project_relational_context()?;
        let requirements = e.acceptance_current_aims()?;
        let advance = input(&e, c)?;
        let result = e.project_cycle_preference_advance(advance.clone())?;
        assert_eq!(result["preference_admitted"], true);
        assert_eq!(result["meaning_admitted"], false);
        assert_eq!(e.project_meaning_advance_receipt(advance.clone())?, result);
        let now = e.project_relational_context()?;
        let derived = &now["inputs"]["generated_preferences"][0];
        assert_eq!(derived["fulfillment"], "unknown");
        assert_eq!(derived["eligible"], true);
        assert!(derived["meaning"].as_str().unwrap().len() > 512);
        assert_eq!(
            now["inputs"]["acceptance_parents"],
            old["inputs"]["parents"]
        );
        assert_eq!(e.acceptance_current_aims()?, requirements);
        let id = serde_json::from_value(derived["id"].clone())?;
        let ledger = Ledger::new(&e.store).collective()?;
        let entry = ledger.get(&id).expect("reviewed native ledger identity");
        assert!(entry.is_open());
        assert_eq!(entry.preference.origin, Source::Generator);
        assert_eq!(
            entry.history.last(),
            Some(&(PreferenceStatus::Open, Source::Council))
        );
        assert_eq!(entry.preference.text, derived["meaning"].as_str().unwrap());
        let record = serde_json::from_value(derived["admission"].clone())?;
        e.require_project_bridge_source(record, Source::Council)?;
        let mut bad_meaning = meaning(&e)?;
        if let Suggestion::PublicDesire { proposal } = &mut bad_meaning.response.proposals[0] {
            proposal
                .parents
                .push(PreferenceId(Hash::of(&[b"unknown contributor"])));
            assert!(e.validate_public_desire(proposal).is_err());
        }
        assert!(e.project_cycle_preference_advance(advance).is_err());
        let r: crate::engine::deliberation::relational::Proposal =
            serde_json::from_value(now["history"][0]["proposal"].clone())?;
        let mut later = r;
        later.parents.push(id);
        later.aspiration="Authored later possibility draws on the derived contributor without changing obligations".into();
        e.validate_project_relational(serde_json::from_value(now["binding"].clone())?, &later)?;
        e.evaluate_proposal(
            Response {
                protocol: PROTOCOL.into(),
                state_root: e.store.head_root()?,
                attention_binding: e.proposal_attention_binding()?,
                proposals: vec![Suggestion::Relational {
                    binding: serde_json::from_value(now["binding"].clone())?,
                    proposal: later,
                }],
                pursuit_expectations: Default::default(),
                common_event_forecasts: Default::default(),
            },
            true,
        )?;
        e.project_relational_admit(
            e.store.head_root()?,
            e.project_latest_proposal()?.unwrap(),
            0,
        )?;
        assert_eq!(
            e.project_relational_context()?["history"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        // Authored environment transition, not engine-written game code or a live receipt.
        fs::write(
            e.directory.join("coding-work/app.js"),
            "// Authored temporary-state transition: unfamiliar route remains unverified\n",
        )?;
        let changed = e.project_acceptance_status()?;
        e.project_acceptance_review(acceptance::Review {
            snapshot: changed.snapshot,
            requirements_revision: changed.requirements_revision.unwrap(),
            test_receipt: changed.test_receipt,
            decision: acceptance::Decision::Unresolved,
            evidence: BTreeMap::from([
                (
                    "orientation".into(),
                    acceptance::Evidence {
                        met: false,
                        evidence: "Authored new-source assessment: evidence insufficient".into(),
                    },
                ),
                (
                    "discovery".into(),
                    acceptance::Evidence {
                        met: false,
                        evidence: "Authored new-source assessment: evidence insufficient".into(),
                    },
                ),
            ]),
        })?;
        assert!(e.project_current_contrast()?.is_none());
        let now = e.project_relational_context()?;
        assert_eq!(now["inputs"]["generated_preferences"][0]["id"], json!(id));
        assert_eq!(
            now["inputs"]["generated_preferences"][0]["fulfillment"],
            "unknown"
        );
        assert!(now["guidance"].as_array().unwrap().is_empty());
        assert_eq!(
            now["inputs"]["semantic_return"]["unknown"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        drop(e);
        let reopened = Engine::open(dir.path())?;
        assert_eq!(reopened.project_relational_context()?, now);
        assert_eq!(reopened.acceptance_current_aims()?, requirements);
        Ok(())
    }
    #[test]
    fn unknown_stale_rejection_and_authority_injection_do_not_admit() -> Result<()> {
        let (_d, e, c) = setup(true)?;
        assert!(e.project_current_contrast()?.is_none());
        assert!(e.validate_contrast_preference(&c).is_err());
        let (_d, e, c) = setup(false)?;
        let root = e.store.head_root()?;
        let mut wrong = c.clone();
        wrong.contrast = ContrastId(Hash::of(&[b"foreign"]));
        assert!(e.validate_contrast_preference(&wrong).is_err());
        let mut a = input(&e, c.clone())?;
        a.review.decision = "reject".into();
        assert!(e.project_cycle_preference_advance(a).is_err());
        let mut a = input(&e, c.clone())?;
        a.review.reviewer = a.review.generator.clone();
        assert!(e.project_cycle_preference_advance(a).is_err());
        let mut json = serde_json::to_value(&c)?;
        json["fulfills_parents"] = json!(true);
        assert!(serde_json::from_value::<Choice>(json).is_err());
        assert_eq!(e.store.head_root()?, root);
        let path = e.directory.join("coding-work/app.js");
        let original = fs::read(&path)?;
        fs::write(&path, b"// authored source change\n")?;
        assert!(e.validate_contrast_preference(&c).is_err());
        fs::write(path, original)?;
        assert_eq!(e.store.head_root()?, root);
        Ok(())
    }
    fn meaning(e: &Engine) -> Result<cycle::Advance> {
        let ctx = e.project_relational_context()?;
        let parent = ctx["inputs"]["parents"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p.get("original_contributors").is_none())
            .unwrap();
        let c = Choice {
            binding: serde_json::from_value(ctx["binding"].clone())?,
            aspiration: EventId(Hash::of(&[b"unused"])),
            contrast: ContrastId(Hash::of(&[b"unused"])),
        };
        let mut a = input(e, c)?;
        a.response.proposals = vec![Suggestion::PublicDesire {
            proposal: desire::Proposal {
                binding: serde_json::from_value(ctx["binding"].clone())?,
                identity: serde_json::from_value(parent["id"].clone())?,
                revises: serde_json::from_value(parent["revision"].clone()).unwrap_or(None),
                parents: vec![serde_json::from_value(parent["id"].clone())?],
                meaning: format!(
                    "{} Authored clarified scope remains answerable to evidence.",
                    parent["meaning"].as_str().unwrap()
                ),
                change: desire::Change::Clarification,
                basis: desire::Basis::Unknown,
                generator: a.review.generator.clone(),
            },
        }];
        a.review.response_digest = Hash::of(&[&serde_json::to_vec(&a.response)?]);
        Ok(a)
    }
    #[test]
    fn root_child_meaning_dependencies_do_not_erase_eligible_history() -> Result<()> {
        let (_dir, e, c) = setup(false)?;
        e.project_cycle_preference_advance(input(&e, c)?)?;
        let before = e.project_relational_context()?;
        let child =
            serde_json::from_value(before["inputs"]["generated_preferences"][0]["id"].clone())?;
        let mut a = meaning(&e)?;
        let root = if let Suggestion::PublicDesire { proposal } = &mut a.response.proposals[0] {
            proposal.parents.push(child);
            proposal.identity
        } else {
            unreachable!()
        };
        a.review.response_digest = Hash::of(&[&serde_json::to_vec(&a.response)?]);
        e.project_cycle_meaning_advance(a)?;
        let after = e.project_relational_context()?;
        assert_eq!(after["inputs"]["parents"].as_array().unwrap().len(), 3);
        assert_eq!(
            after["inputs"]["generated_preferences"][0]["eligible"],
            true
        );
        let parent = after["inputs"]["parents"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == json!(root))
            .unwrap();
        e.project_desire_status(desire::Status {
            state_root: e.store.head_root()?,
            identity: root,
            revision: serde_json::from_value(parent["revision"].clone())?,
            withdrawn: true,
        })?;
        let stopped = e.project_relational_context()?;
        assert_eq!(
            stopped["inputs"]["generated_preferences"][0]["eligible"],
            false
        );
        assert_eq!(stopped["inputs"]["parents"].as_array().unwrap().len(), 1);
        assert_eq!(
            stopped["inputs"]["acceptance_parents"],
            before["inputs"]["acceptance_parents"]
        );
        Ok(())
    }
    #[test]
    fn eight_parent_child_meaning_is_representable() -> Result<()> {
        let (_dir, e, c) = setup_wants(
            false,
            (0..8)
                .map(|i| {
                    (
                        format!("want_{i}"),
                        format!("Authored independent want {i}"),
                    )
                })
                .collect(),
        )?;
        e.project_cycle_preference_advance(input(&e, c)?)?;
        let ctx = e.project_relational_context()?;
        let child = &ctx["inputs"]["generated_preferences"][0];
        let mut a = meaning(&e)?;
        if let Suggestion::PublicDesire { proposal } = &mut a.response.proposals[0] {
            proposal.identity = serde_json::from_value(child["id"].clone())?;
            proposal.parents = serde_json::from_value(child["parents"].clone())?;
            proposal.parents.push(proposal.identity);
            assert_eq!(proposal.parents.len(), 9);
            e.validate_public_desire(proposal)?;
        }
        a.review.response_digest = Hash::of(&[&serde_json::to_vec(&a.response)?]);
        e.project_cycle_meaning_advance(a)?;
        assert_eq!(
            e.project_relational_context()?["inputs"]["parents"]
                .as_array()
                .unwrap()
                .len(),
            9
        );
        Ok(())
    }
    #[test]
    fn generated_identity_can_develop_meaning_without_replacing_origin() -> Result<()> {
        let (dir, e, c) = setup(false)?;
        e.project_cycle_preference_advance(input(&e, c)?)?;
        let before = e.project_relational_context()?;
        let origin = before["inputs"]["generated_preferences"][0].clone();
        let requirements = e.acceptance_current_aims()?;
        let mut a = meaning(&e)?;
        if let Suggestion::PublicDesire { proposal } = &mut a.response.proposals[0] {
            proposal.identity = serde_json::from_value(origin["id"].clone())?;
            proposal.parents = serde_json::from_value(origin["parents"].clone())?;
            proposal.parents.push(proposal.identity);
            proposal.meaning = "Authored generated wanting now emphasizes recoverable exploration; fulfillment is still unknown. ".repeat(50);
            let mut missing = proposal.clone();
            missing.parents = vec![proposal.identity];
            assert!(e.validate_public_desire(&missing).is_err());
            e.validate_public_desire(proposal)?;
        }
        a.review.response_digest = Hash::of(&[&serde_json::to_vec(&a.response)?]);
        let result = e.project_cycle_meaning_advance(a.clone())?;
        assert_eq!(result["meaning_admitted"], true);
        assert_eq!(e.project_meaning_advance_receipt(a.clone())?, result);
        assert!(e.project_cycle_meaning_advance(a).is_err());
        let after = e.project_relational_context()?;
        assert_eq!(after["inputs"]["generated_preferences"][0], origin);
        assert_eq!(
            after["inputs"]["acceptance_parents"],
            before["inputs"]["acceptance_parents"]
        );
        assert_eq!(e.acceptance_current_aims()?, requirements);
        let current = after["inputs"]["parents"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["id"] == origin["id"])
            .unwrap();
        assert_ne!(current["meaning"], origin["meaning"]);
        assert_eq!(current["generated_origin"], origin);
        assert_eq!(current["preference_parents"], origin["parents"]);
        assert_eq!(current["fulfillment"], "unknown");
        let id = serde_json::from_value(origin["id"].clone())?;
        assert_eq!(
            Ledger::new(&e.store)
                .collective()?
                .get(&id)
                .unwrap()
                .preference
                .text,
            origin["meaning"].as_str().unwrap()
        );
        drop(e);
        let restarted = Engine::open(dir.path())?;
        assert_eq!(restarted.project_relational_context()?, after);
        Ok(())
    }
    #[test]
    fn mixed_meaning_preference_receipts_are_exact_and_provenance_safe() -> Result<()> {
        let (_dir, e, _c) = setup(false)?;
        let a = meaning(&e)?;
        e.project_cycle_meaning_advance(a)?;
        let ctx = e.project_relational_context()?;
        let mut p: crate::engine::deliberation::relational::Proposal =
            serde_json::from_value(ctx["history"][0]["proposal"].clone())?;
        p.relationship
            .push_str(" Authored fresh relation after clarification.");
        e.evaluate_proposal(
            Response {
                protocol: PROTOCOL.into(),
                state_root: e.store.head_root()?,
                attention_binding: e.proposal_attention_binding()?,
                proposals: vec![Suggestion::Relational {
                    binding: serde_json::from_value(ctx["binding"].clone())?,
                    proposal: p,
                }],
                pursuit_expectations: Default::default(),
                common_event_forecasts: Default::default(),
            },
            true,
        )?;
        let aspiration = e.project_relational_admit(
            e.store.head_root()?,
            e.project_latest_proposal()?.unwrap(),
            0,
        )?;
        let ctx = e.project_relational_context()?;
        let a = input(
            &e,
            Choice {
                binding: serde_json::from_value(ctx["binding"].clone())?,
                aspiration,
                contrast: e.project_current_contrast()?.unwrap().id,
            },
        )?;
        let result = e.project_cycle_preference_advance(a.clone())?;
        assert_eq!(e.project_meaning_advance_receipt(a)?, result);
        let a = meaning(&e)?;
        let result = e.project_cycle_meaning_advance(a.clone())?;
        assert_eq!(e.project_meaning_advance_receipt(a)?, result);
        Ok(())
    }
    #[test]
    fn duplicate_permuted_contributors_and_pending_effect_block() -> Result<()> {
        let (_dir, e, c) = setup(false)?;
        e.project_cycle_preference_advance(input(&e, c)?)?;
        let ctx = e.project_relational_context()?;
        let mut p: crate::engine::deliberation::relational::Proposal =
            serde_json::from_value(ctx["history"][0]["proposal"].clone())?;
        p.parents.reverse();
        p.relationship
            .push_str(" Authored duplicate identity permutation probe.");
        e.evaluate_proposal(
            Response {
                protocol: PROTOCOL.into(),
                state_root: e.store.head_root()?,
                attention_binding: e.proposal_attention_binding()?,
                proposals: vec![Suggestion::Relational {
                    binding: serde_json::from_value(ctx["binding"].clone())?,
                    proposal: p,
                }],
                pursuit_expectations: Default::default(),
                common_event_forecasts: Default::default(),
            },
            true,
        )?;
        let aspiration = e.project_relational_admit(
            e.store.head_root()?,
            e.project_latest_proposal()?.unwrap(),
            0,
        )?;
        let ctx = e.project_relational_context()?;
        let c = Choice {
            binding: serde_json::from_value(ctx["binding"].clone())?,
            aspiration,
            contrast: e.project_current_contrast()?.unwrap().id,
        };
        assert!(e.validate_contrast_preference(&c).is_err());
        let (_dir, e, c) = setup(false)?;
        let a = input(&e, c)?;
        e.project_resource_reserve_unpriced(budget::UnpricedReserve {
            request_id: "authored unresolved request".into(),
            state_root: e.store.head_root()?,
            envelope: "authored".into(),
            model: "authored".into(),
            tokens_upper: 10,
        })?;
        let root = e.store.head_root()?;
        assert!(e.project_cycle_preference_advance(a).is_err());
        assert_eq!(e.store.head_root()?, root);
        assert_eq!(e.project_cycle_request()?["branch"], "reconcile");
        Ok(())
    }
    #[test]
    fn shared_ancestor_composition_keeps_flat_exact_lineage() -> Result<()> {
        let (dir, e, c) = setup(false)?;
        e.project_cycle_preference_advance(input(&e, c)?)?;
        let ctx = e.project_relational_context()?;
        let first = ctx["inputs"]["generated_preferences"][0].clone();
        let mut p: crate::engine::deliberation::relational::Proposal =
            serde_json::from_value(ctx["history"][0]["proposal"].clone())?;
        p.parents.push(serde_json::from_value(first["id"].clone())?);
        p.relationship =
            "Authored second-generation composition with shared original contributors".into();
        p.aspiration =
            "Make the route back understandable while exploring unfamiliar branches".into();
        e.evaluate_proposal(
            Response {
                protocol: PROTOCOL.into(),
                state_root: e.store.head_root()?,
                attention_binding: e.proposal_attention_binding()?,
                proposals: vec![Suggestion::Relational {
                    binding: serde_json::from_value(ctx["binding"].clone())?,
                    proposal: p,
                }],
                pursuit_expectations: Default::default(),
                common_event_forecasts: Default::default(),
            },
            true,
        )?;
        let aspiration = e.project_relational_admit(
            e.store.head_root()?,
            e.project_latest_proposal()?.unwrap(),
            0,
        )?;
        let ctx = e.project_relational_context()?;
        let c = Choice {
            binding: serde_json::from_value(ctx["binding"].clone())?,
            aspiration,
            contrast: e.project_current_contrast()?.unwrap().id,
        };
        e.project_cycle_preference_advance(input(&e, c)?)?;
        let ctx = e.project_relational_context()?;
        let second = &ctx["inputs"]["generated_preferences"][1];
        assert_eq!(
            second["original_contributors"],
            first["original_contributors"]
        );
        let snapshots = second["contributors"].as_array().unwrap();
        assert_eq!(snapshots.len(), 3);
        assert!(snapshots
            .iter()
            .all(|x| x.get("contributors").is_none() && x.get("review").is_none()));
        let reference = snapshots.iter().find(|x| x["id"] == first["id"]).unwrap();
        assert_eq!(reference["admission"], first["admission"]);
        assert_eq!(reference["meaning"], first["meaning"]);
        assert_eq!(ctx["inputs"]["generated_preferences"][0], first);
        drop(e);
        let e = Engine::open(dir.path())?;
        assert_eq!(e.project_relational_context()?, ctx);
        let first_id: PreferenceId = serde_json::from_value(first["id"].clone())?;
        let second_id: PreferenceId = serde_json::from_value(second["id"].clone())?;
        for (status, available) in [
            (PreferenceStatus::Fulfilled, true),
            (PreferenceStatus::Withdrawn, false),
            (PreferenceStatus::Open, true),
            (PreferenceStatus::OutOfScope, false),
            (PreferenceStatus::Open, true),
            (PreferenceStatus::Superseded(second_id), false),
            (PreferenceStatus::Open, true),
            (PreferenceStatus::Overlapping(second_id), false),
        ] {
            e.append(
                EventKind::StatusSet {
                    preference: first_id,
                    status: status.clone(),
                    by: Source::Operator,
                },
                Source::Operator,
            )?;
            let updated = e.project_relational_context()?;
            let nodes = updated["inputs"]["generated_preferences"]
                .as_array()
                .unwrap();
            assert_eq!(nodes.len(), 2);
            assert!(nodes.iter().all(|p| p["eligible"] == json!(available)));
            assert!(nodes.iter().all(|p| p["fulfillment"] == "unknown"));
            assert_eq!(
                updated["inputs"]["generated_preference_statuses"][0]["status"],
                tsr_ledger::status_name(&status)
            );
            assert_eq!(nodes[0]["contributors"], first["contributors"]);
            let reopened = Engine::open(dir.path())?;
            assert_eq!(reopened.project_relational_context()?, updated);
        }
        Ok(())
    }
    #[test]
    fn legacy_oversized_operator_expansion_stays_ineligible() -> Result<()> {
        let (_dir, e, c) = setup(false)?;
        let text = "Authored legacy invalid operator text ".repeat(30);
        assert!(Preference::new(
            text.clone(),
            Scope::Task(Engine::task()),
            vec![],
            Source::Operator
        )
        .is_err());
        let p = Preference::from_admitted_text(
            text,
            Scope::Task(Engine::task()),
            vec![],
            Source::Operator,
        )?;
        e.append(
            EventKind::Expanded {
                expansion: Expansion {
                    from_contrast: c.contrast,
                    produced: p.id,
                    catalyzed_by: vec![],
                    text: p.text,
                    scope: p.scope,
                    parents: vec![],
                    origin: Source::Operator,
                },
            },
            Source::Operator,
        )?;
        assert!(Ledger::new(&e.store).collective()?.get(&p.id).is_none());
        assert!(e.project_relational_context()?["inputs"]
            .get("generated_preferences")
            .is_none());
        Ok(())
    }
    #[test]
    fn interrupted_admission_rolls_back_and_stop_blocks() -> Result<()> {
        let (_d, e, c) = setup(false)?;
        let root = e.store.head_root()?;
        let a = input(&e, c.clone())?;
        let error = e.write(|| -> Result<()> {
            e.project_cycle_preference_advance(a)?;
            anyhow::bail!("authored interruption before outer commit")
        });
        assert!(error.is_err());
        assert_eq!(root, e.store.head_root()?);
        assert!(e.project_relational_context()?["inputs"]
            .get("generated_preferences")
            .is_none());
        e.project_lifecycle_close_advisory("Authored Stop before admission".into())?;
        assert!(e.project_cycle_preference_advance(input(&e, c)?).is_err());
        Ok(())
    }
}
