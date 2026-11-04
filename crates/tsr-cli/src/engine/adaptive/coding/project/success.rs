//! Independently reviewed scoped success may catalyze optional new wanting.
use super::*;
use anyhow::Context;
use serde_json::{json, Value};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub binding: Hash,
    pub aspiration: EventId,
    pub success: EventId,
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
impl Engine {
    pub(crate) fn project_current_success(
        &self,
    ) -> Result<Option<(EventId, tsr_domain::ScopedSuccess)>> {
        if self.criterion_policy()?.is_none() {
            return Ok(None);
        }
        let ctx = self.project_criterion_context()?;
        let Some(id) = ctx["current"].get("id") else {
            return Ok(None);
        };
        let id: EventId = serde_json::from_value(id.clone())?;
        let a: criterion_return::Apply =
            serde_json::from_value(ctx["current"]["assessment"].clone())?;
        if a.decision == "abstain"
            || a.findings.is_empty()
            || a.findings
                .values()
                .all(|f| f.finding != acceptance::reviewer::Finding::Met)
            || a.frame.assessment_scope.is_none()
        {
            return Ok(None);
        }
        let expected = tsr_domain::ScopedSuccess {
            assessment: id,
            receipt: a.frame.receipt,
            publication: a.frame.publication,
            requirements_revision: a.frame.requirements_revision,
            source_snapshot: a.frame.snapshot,
            scope_digest: Hash::of(&[&serde_json::to_vec(&a.frame)?]),
            criteria: a
                .findings
                .iter()
                .filter(|(_, f)| f.finding == acceptance::reviewer::Finding::Met)
                .map(|(k, _)| k.clone())
                .collect(),
        };
        let events = self.events()?;
        let found: Vec<_> = events
            .iter()
            .filter_map(|(_, e)| match &e.kind {
                EventKind::ScopedSuccess { success }
                    if e.source == Source::Council && success == &expected =>
                {
                    Some((e.id, success.clone()))
                }
                _ => None,
            })
            .collect();
        ensure!(found.len() <= 1, "ambiguous scoped success");
        if let Some((id, _)) = found.first() {
            tsr_ledger::success::basis(&events, Self::task(), *id)?;
        }
        Ok(found.into_iter().next())
    }
    pub(crate) fn success_preference_parts(
        &self,
        c: &Choice,
    ) -> Result<(Preference, Vec<PreferenceId>, Vec<Value>)> {
        ensure!(
            self.project_configuration()?.browser,
            "success preference browser profile"
        );
        let ctx = self.project_relational_context()?;
        ensure!(
            ctx["binding"] == json!(c.binding),
            "stale success preference binding"
        );
        let (id, b) = self
            .project_current_success()?
            .context("no current scoped success")?;
        ensure!(id == c.success, "foreign or stale success");
        let (p, roots, contributors) = self.derived_preference_parts(&ctx, c.aspiration)?;
        let aims = self.acceptance_current_aims()?;
        ensure!(
            roots.iter().all(|id| aims
                .iter()
                .any(|(key, parent)| parent == id && b.criteria.contains(key))),
            "success must cover every original contributor"
        );
        Ok((p, roots, contributors))
    }
    pub(crate) fn validate_success_preference(&self, c: &Choice) -> Result<()> {
        self.success_preference_parts(c).map(|_| ())
    }
    pub(crate) fn admit_success_preference(
        &self,
        c: &Choice,
        proposal: EventId,
        review: cycle::Review,
    ) -> Result<EventId> {
        let (preference, roots, contributors) = self.success_preference_parts(c)?;
        let (_, basis) = self.project_current_success()?.context("current success")?;
        let ledger = Ledger::new(&self.store);
        let at = format!(
            "unix-ms:{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis()
        );
        let history = self.events()?;
        let expansion = ledger.propose_success(
            &Self::task(),
            history.last().map(|(_, e)| vec![e.id]).unwrap_or_default(),
            tsr_domain::SuccessExpansion {
                success: c.success,
                preference: preference.clone(),
            },
            &at,
        )?;
        let attestation = tsr_domain::SuccessPreferenceReview {
            expansion,
            success: c.success,
            preference: preference.id,
            proposal,
            source_snapshot: basis.source_snapshot,
            requirements_revision: basis.requirements_revision,
            request_binding: review.binding,
            response_digest: review.response_digest,
            reviewer: review.reviewer.clone(),
            generator: review.generator.clone(),
            decision: review.decision.clone(),
        };
        let record = self.adaptive_record(
            &Record::Coding {
                record: CodingRecord::SuccessPreferenceAdmission {
                    admission: Box::new(Admission {
                        choice: c.clone(),
                        preference,
                        roots,
                        contributors,
                        snapshot: basis.source_snapshot,
                        requirements_revision: basis.requirements_revision,
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
                    source: tsr_ledger::success::REVIEW_SOURCE.into(),
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
        ledger.admit_success(
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
    use acceptance::reviewer::Finding;
    fn setup(finding: Finding) -> Result<(tempfile::TempDir, Engine, Choice)> {
        setup_scoped(finding, None)
    }
    fn setup_scoped(
        finding: Finding,
        unrelated: Option<Finding>,
    ) -> Result<(tempfile::TempDir, Engine, Choice)> {
        let (d, e) = criterion_return::tests::setup_with(if unrelated.is_some() {
            "success_extra"
        } else {
            "success"
        })?;
        let source = e.project_acceptance_status()?.snapshot;
        criterion_return::tests::receipt(&e, Verdict::Passed, source, Source::Rules)?;
        let mut assessment = criterion_return::tests::input(&e, finding)?;
        if let Some(unrelated) = unrelated {
            let f = assessment.findings.get_mut("unrelated").unwrap();
            f.finding = unrelated.clone();
            f.basis = Some(if unrelated == Finding::Unknown {
                acceptance::reviewer::typed::Basis::Unknown
            } else {
                acceptance::reviewer::typed::Basis::StaticSemantic
            });
            assessment.decision = if assessment
                .findings
                .values()
                .any(|f| f.finding == Finding::NotMet)
            {
                "reject"
            } else if assessment
                .findings
                .values()
                .any(|f| f.finding == Finding::Unknown)
            {
                "unknown"
            } else {
                "accept"
            }
            .into();
        }
        e.project_criterion_apply(assessment)?;
        e.project_resource_unpriced_admit(e.store.head_root()?)?;
        let ctx = e.project_relational_context()?;
        let p = crate::engine::deliberation::relational::Proposal {
            parents: serde_json::from_value(json!(ctx["inputs"]["parents"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|p| p["requirement"] != "unrelated"
                    && p["meaning"] != "Preserve a separate unresolved obligation")
                .map(|p| p["id"].clone())
                .collect::<Vec<_>>()))?,
            relationship:
                "Authored possibility: dependable orientation supports unfamiliar exploration"
                    .into(),
            aspiration: "Cultivate recoverable exploration across unfamiliar perspectives".into(),
            discriminator: "Observe exploration and whether orientation remains recoverable".into(),
            revises: None,
            catalysts: vec![],
            development: None,
        };
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
        let cycle = e.project_cycle_request()?;
        if cycle["branch"] == "review_required" {
            e.project_cycle_continue(cycle::Continuation {
                binding: serde_json::from_value(cycle["binding"].clone())?,
                message_id: "authored-success-continuation".into(),
                reason: "Authored session permits an optional next choice after scoped results"
                    .into(),
            })?;
        }
        let ctx = e.project_relational_context()?;
        let choice = Choice {
            binding: serde_json::from_value(ctx["binding"].clone())?,
            aspiration,
            success: e
                .project_current_success()?
                .map(|x| x.0)
                .unwrap_or(EventId(Hash::of(&[b"unknown is not success"]))),
        };
        Ok((d, e, choice))
    }
    fn input(e: &Engine, c: Choice) -> Result<cycle::Advance> {
        let ctx = e.project_cycle_request()?;
        let binding = serde_json::from_value(ctx["binding"].clone())?;
        let response = Response {
            protocol: PROTOCOL.into(),
            state_root: e.store.head_root()?,
            attention_binding: e.proposal_attention_binding()?,
            proposals: vec![Suggestion::SuccessPreference { choice: c }],
            pursuit_expectations: Default::default(),
            common_event_forecasts: Default::default(),
        };
        Ok(cycle::Advance {
            binding,
            response: response.clone(),
            review: cycle::Review {
                binding,
                response_digest: Hash::of(&[&serde_json::to_vec(&response)?]),
                state_root: response.state_root,
                source_snapshot: serde_json::from_value(ctx["source_snapshot"].clone())?,
                reviewer: "authored independent success reviewer".into(),
                generator: "authored success generator".into(),
                decision: "approve".into(),
            },
        })
    }
    #[test]
    fn success_identity_is_native_reviewed_composable_and_restart_safe() -> Result<()> {
        let (d, e, c) = setup(Finding::Met)?;
        let old = e.project_relational_context()?;
        let aims = e.acceptance_current_aims()?;
        let config = e.project_configuration()?;
        assert!(e.project_current_contrast()?.is_none());
        let advance = input(&e, c.clone())?;
        let head = e.store.head_root()?;
        assert!(e.project_cycle_advance(advance.clone()).is_err());
        assert_eq!(head, e.store.head_root()?);
        let mut rejected = advance.clone();
        rejected.review.decision = "reject".into();
        assert!(e.project_cycle_preference_advance(rejected).is_err());
        assert_eq!(head, e.store.head_root()?);
        let result = e.project_cycle_preference_advance(advance.clone())?;
        assert_eq!(result["preference_admitted"], true);
        assert_eq!(result["executed"], false);
        assert_eq!(e.project_meaning_advance_receipt(advance.clone())?, result);
        let now = e.project_relational_context()?;
        let g = &now["inputs"]["generated_preferences"][0];
        assert_eq!(g["eligible"], true);
        assert_eq!(g["fulfillment"], "unknown");
        assert_eq!(g["grants_authority"], false);
        assert_eq!(g["success"], json!(c.success));
        assert!(g.get("contrast").is_none());
        assert_eq!(
            now["inputs"]["acceptance_parents"],
            old["inputs"]["parents"]
        );
        assert_eq!(e.acceptance_current_aims()?, aims);
        assert_eq!(
            serde_json::to_value(e.project_configuration()?)?,
            serde_json::to_value(config)?
        );
        let id: PreferenceId = serde_json::from_value(g["id"].clone())?;
        let ledger = Ledger::new(&e.store).collective()?;
        let entry = ledger.get(&id).unwrap();
        assert!(entry.is_open());
        let snapshot = tsr_domain::Snapshot {
            task: Engine::task(),
            version: 0,
            root: ledger.root,
            open_criteria: vec![],
            missing_evidence: vec![],
        };
        let tune = tsr_domain::Tune {
            flow_horizon: 0,
            intuition_max: 100,
            quiet_route: 0.0,
            admitted_sources: vec![Source::Operator, Source::Generator],
            ..Default::default()
        };
        let selected = tsr_ledger::selection::select(
            &ledger,
            &snapshot,
            &tune,
            &tsr_ledger::selection::SelectionContext {
                family: "authored".into(),
                repository: "authored".into(),
                minimum_depth: 1,
            },
            0,
        )
        .map_err(anyhow::Error::msg)?;
        assert!(selected.selected.contains(&id));

        assert_eq!(entry.from_success, Some(c.success));
        assert_eq!(entry.from_contrast, None);
        assert_eq!(
            entry.history.last(),
            Some(&(PreferenceStatus::Open, Source::Council))
        );
        let mut later: crate::engine::deliberation::relational::Proposal =
            serde_json::from_value(now["history"][0]["proposal"].clone())?;
        later.parents.push(id);
        later.aspiration =
            "Authored later choice incorporates the successor without replacing parents".into();
        e.validate_project_relational(serde_json::from_value(now["binding"].clone())?, &later)?;
        let before = e.store.head_root()?;
        assert!(e.project_cycle_preference_advance(advance).is_err());
        assert_eq!(before, e.store.head_root()?);
        drop(e);
        let fresh = Engine::open(d.path())?;
        assert_eq!(fresh.project_relational_context()?, now);
        assert_eq!(fresh.store.head_root()?, before);
        Ok(())
    }
    #[test]
    fn scoped_success_preserves_unrelated_open_obligations() -> Result<()> {
        for unrelated in [Finding::Unknown, Finding::NotMet] {
            let (d, e, c) = setup_scoped(Finding::Met, Some(unrelated.clone()))?;
            let choices = e.project_relational_context()?;
            assert_eq!(
                choices["identity_choices"]["success_preference"],
                json!([c.aspiration])
            );
            assert_eq!(
                choices["identity_choices"]["contrast_preference"],
                json!([])
            );
            let before = e.project_criterion_context()?;
            let aims = e.acceptance_current_aims()?;
            let config = serde_json::to_value(e.project_configuration()?)?;
            let (_, basis) = e.project_current_success()?.unwrap();
            assert_eq!(basis.criteria, vec!["discovery", "orientation"]);
            assert_eq!(
                before["current"]["assessment"]["findings"]["unrelated"]["finding"],
                json!(unrelated)
            );
            e.project_cycle_preference_advance(input(&e, c)?)?;
            assert_eq!(e.project_criterion_context()?["current"], before["current"]);
            assert_eq!(e.acceptance_current_aims()?, aims);
            assert_eq!(serde_json::to_value(e.project_configuration()?)?, config);
            let ledger = Ledger::new(&e.store).collective()?;
            assert!(ledger.get(&aims["unrelated"]).unwrap().is_open());
            let end = e.project_relational_context()?;
            let child = &end["inputs"]["generated_preferences"][0];
            assert_eq!(child["eligible"], true);
            assert_eq!(child["original_contributors"].as_array().unwrap().len(), 2);
            assert!(!child["original_contributors"]
                .as_array()
                .unwrap()
                .contains(&json!(aims["unrelated"])));
            drop(e);
            assert_eq!(Engine::open(d.path())?.project_relational_context()?, end);
        }
        // An unrelated success cannot cover unknown/unmet actual contributors.
        for relevant in [Finding::Unknown, Finding::NotMet] {
            let (_d, e, c) = setup_scoped(relevant, Some(Finding::Met))?;
            assert_eq!(
                e.project_current_success()?.unwrap().1.criteria,
                vec!["unrelated"]
            );
            let head = e.store.head_root()?;
            assert!(e.validate_success_preference(&c).is_err());
            assert_eq!(
                e.project_relational_context()?["identity_choices"]["success_preference"],
                json!([])
            );
            assert_eq!(e.store.head_root()?, head);
        }
        Ok(())
    }
    #[test]
    fn mixed_assessment_abstention_does_not_attest_success() -> Result<()> {
        let (_d, e) = criterion_return::tests::setup_with("success_extra")?;
        let source = e.project_acceptance_status()?.snapshot;
        criterion_return::tests::receipt(&e, Verdict::Passed, source, Source::Rules)?;
        let mut a = criterion_return::tests::input(&e, Finding::Met)?;
        a.decision = "abstain".into();
        let f = a.findings.get_mut("unrelated").unwrap();
        f.finding = Finding::Unknown;
        f.basis = Some(acceptance::reviewer::typed::Basis::Unknown);
        e.project_criterion_apply(a)?;
        assert!(e.project_current_success()?.is_none());
        assert!(e.project_current_contrast()?.is_none());
        assert!(
            e.project_relational_context()?["inputs"]["generated_preferences"]
                .as_array()
                .is_none_or(|v| v.is_empty())
        );
        Ok(())
    }
    #[test]
    fn uncertainty_failure_and_no_proposal_do_not_generate_wanting() -> Result<()> {
        for finding in [Finding::Unknown, Finding::NotMet] {
            let (_d, e, c) = setup(finding)?;
            assert!(e.project_current_success()?.is_none());
            let head = e.store.head_root()?;
            assert!(e.validate_success_preference(&c).is_err());
            assert_eq!(head, e.store.head_root()?);
        }
        let (_d, e, _) = setup(Finding::Met)?;
        assert!(e.project_current_success()?.is_some());
        assert!(
            e.project_relational_context()?["inputs"]["generated_preferences"]
                .as_array()
                .is_none_or(|x| x.is_empty())
        );
        Ok(())
    }
    #[test]
    fn forged_stale_and_same_source_new_publication_are_rejected() -> Result<()> {
        let (_d, e, c) = setup(Finding::Met)?;
        let head = e.store.head_root()?;
        let mut forged = c.clone();
        forged.success = EventId(Hash::of(&[b"forged"]));
        assert!(e.validate_success_preference(&forged).is_err());
        forged = c.clone();
        forged.binding = Hash::of(&[b"stale"]);
        assert!(e.validate_success_preference(&forged).is_err());
        assert_eq!(head, e.store.head_root()?);
        let st = e.project_acceptance_status()?;
        let source = st.snapshot;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: st.requirements_revision.unwrap(),
            decision_record: st.decision_record,
            guidance: BTreeMap::from([(
                "orientation".into(),
                "Authored later condition; old success is historical".into(),
            )]),
        })?;
        assert_eq!(e.project_acceptance_status()?.snapshot, source);
        assert!(e.project_current_success()?.is_none());
        assert!(e.validate_success_preference(&c).is_err());
        let (_d, e, c) = setup(Finding::Met)?;
        fs::write(
            e.directory.join("coding-work/app.js"),
            "// authored newer source; not the assessed version\n",
        )?;
        assert!(e.project_current_success()?.is_none());
        assert!(e.validate_success_preference(&c).is_err());
        Ok(())
    }
    #[test]
    fn interrupted_admission_rolls_back_stop_and_unknown_effects_block() -> Result<()> {
        let (_d, e, c) = setup(Finding::Met)?;
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
        e.project_lifecycle_close_advisory("Authored Stop before success admission".into())?;
        assert!(e.project_cycle_preference_advance(input(&e, c)?).is_err());
        let (_d, e, c) = setup(Finding::Met)?;
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
        assert_eq!(root, e.store.head_root()?);
        assert_eq!(e.project_cycle_request()?["branch"], "reconcile");
        Ok(())
    }

    #[test]
    fn mixed_success_and_contrast_lineage_survives_replay_and_satisfied_parent() -> Result<()> {
        let (d, e, c) = setup(Finding::Met)?;
        e.project_cycle_preference_advance(input(&e, c)?)?;
        let first = e.project_relational_context()?["inputs"]["generated_preferences"][0].clone();
        let first_id: PreferenceId = serde_json::from_value(first["id"].clone())?;
        // Explicit authored status fixture: admission itself never marks fulfillment.
        e.append(
            EventKind::StatusSet {
                preference: first_id,
                status: PreferenceStatus::Fulfilled,
                by: Source::Operator,
            },
            Source::Operator,
        )?;
        assert_eq!(
            e.project_relational_context()?["inputs"]["generated_preferences"][0]["eligible"],
            true
        );
        let st = e.project_acceptance_status()?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: st.requirements_revision.unwrap(),
            decision_record: st.decision_record,
            guidance: BTreeMap::from([
                (
                    "orientation".into(),
                    "Authored new condition with an independently assessed unmet criterion".into(),
                ),
                (
                    "discovery".into(),
                    "Make unfamiliar perspectives accessible".into(),
                ),
            ]),
        })?;
        e.project_criterion_apply(criterion_return::tests::input(&e, Finding::NotMet)?)?;
        assert!(e.project_current_success()?.is_none());
        let contrast = e.project_current_contrast()?.unwrap().id;
        let ctx = e.project_relational_context()?;
        let mut p: crate::engine::deliberation::relational::Proposal =
            serde_json::from_value(ctx["history"][0]["proposal"].clone())?;
        // The other original root participates only through the child lineage.
        // Requiring it again as a direct parent would block recursive composition.
        p.parents = vec![p.parents[0], first_id];
        p.aspiration =
            "Authored next wanting: keep recovery understandable while exploring another branch"
                .into();
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
        let ctx = e.project_cycle_request()?;
        if ctx["branch"] == "review_required" {
            e.project_cycle_continue(cycle::Continuation {
                binding: serde_json::from_value(ctx["binding"].clone())?,
                message_id: "authored-mixed-continuation".into(),
                reason: "Authored next decision under unchanged authority".into(),
            })?;
        }
        let ctx = e.project_cycle_request()?;
        let r = e.project_relational_context()?;
        let response = Response {
            protocol: PROTOCOL.into(),
            state_root: e.store.head_root()?,
            attention_binding: e.proposal_attention_binding()?,
            proposals: vec![Suggestion::ContrastPreference {
                choice: super::super::expansion::Choice {
                    binding: serde_json::from_value(r["binding"].clone())?,
                    aspiration,
                    contrast,
                },
            }],
            pursuit_expectations: Default::default(),
            common_event_forecasts: Default::default(),
        };
        let binding = serde_json::from_value(ctx["binding"].clone())?;
        e.project_cycle_preference_advance(cycle::Advance {
            binding,
            response: response.clone(),
            review: cycle::Review {
                binding,
                response_digest: Hash::of(&[&serde_json::to_vec(&response)?]),
                state_root: response.state_root,
                source_snapshot: serde_json::from_value(ctx["source_snapshot"].clone())?,
                reviewer: "authored mixed independent reviewer".into(),
                generator: "authored mixed generator".into(),
                decision: "approve".into(),
            },
        })?;
        let end = e.project_relational_context()?;
        let nodes = end["inputs"]["generated_preferences"].as_array().unwrap();
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0], first);
        assert_eq!(
            nodes[1]["original_contributors"],
            first["original_contributors"]
        );
        assert!(nodes[1]["parents"]
            .as_array()
            .unwrap()
            .contains(&json!(first_id)));
        assert_eq!(nodes[1]["parents"].as_array().unwrap().len(), 2);
        assert_eq!(
            nodes[1]["original_contributors"].as_array().unwrap().len(),
            2
        );
        assert_eq!(nodes[1]["contrast"], json!(contrast));
        assert_eq!(nodes[1]["eligible"], true);
        let ledger = Ledger::new(&e.store).collective()?;
        assert_eq!(
            ledger.get(&first_id).unwrap().status,
            Some(PreferenceStatus::Fulfilled)
        );
        drop(e);
        assert_eq!(Engine::open(d.path())?.project_relational_context()?, end);
        Ok(())
    }
}
