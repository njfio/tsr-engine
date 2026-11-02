//! Reviewed generated wanting with explicitly unknown fulfillment; no invented return.
use super::*;
use anyhow::Context;
use serde_json::{json, Value};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub binding: Hash,
    pub parents: Vec<PreferenceId>,
    pub meaning: String,
    pub rationale: String,
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
    pub(crate) fn provisional_preference_parts(
        &self,
        c: &Choice,
    ) -> Result<(Preference, Vec<PreferenceId>, Vec<Value>)> {
        ensure!(
            self.project_configuration()?.browser,
            "provisional wanting browser profile"
        );
        let ctx = self.project_relational_context()?;
        ensure!(
            ctx["binding"] == json!(c.binding),
            "stale provisional context"
        );
        ensure!(
            browser::narrative(&c.meaning) && browser::narrative(&c.rationale),
            "provisional text/rationale required"
        );
        let parents: BTreeSet<_> = c.parents.iter().copied().collect();
        ensure!(
            !parents.is_empty() && parents.len() == c.parents.len(),
            "distinct provisional contributors required"
        );
        let inputs = &ctx["inputs"];
        let originals = inputs
            .get("acceptance_parents")
            .unwrap_or(&inputs["parents"]);
        let mut original_ids: BTreeSet<PreferenceId> = originals
            .as_array()
            .context("original contributors")?
            .iter()
            .map(|p| serde_json::from_value(p["id"].clone()))
            .collect::<std::result::Result<_, _>>()?;
        for p in inputs["parents"]
            .as_array()
            .context("eligible contributors")?
            .iter()
            .filter(|p| p.get("original_contributors").is_none())
        {
            original_ids.insert(serde_json::from_value(p["id"].clone())?);
        }
        let mut roots = BTreeSet::new();
        let mut contributors = vec![];
        for id in &parents {
            let p = inputs["parents"]
                .as_array()
                .context("eligible contributors")?
                .iter()
                .find(|p| p["id"] == json!(id))
                .context("inactive/foreign provisional contributor")?;
            contributors.push(json!({"id":id,"meaning":p["meaning"],"revision":p["revision"],"admission":p["admission"],"requirement":p["requirement"],"parents":p.get("parents").cloned().unwrap_or(json!([])),"original_contributors":p.get("original_contributors").cloned().unwrap_or(json!([id]))}));
            if original_ids.contains(id) {
                roots.insert(*id);
            } else {
                roots.extend(serde_json::from_value::<Vec<PreferenceId>>(
                    p["original_contributors"].clone(),
                )?);
            }
        }
        ensure!(
            !roots.is_empty() && roots.is_subset(&original_ids),
            "provisional original ancestry"
        );
        let preference = Preference::from_admitted_text(
            c.meaning.clone(),
            Scope::Task(Self::task()),
            parents.into_iter().collect(),
            Source::Generator,
        )?;
        let norm = |s: &str| {
            s.split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .to_lowercase()
        };
        ensure!(
            !Ledger::new(&self.store)
                .collective()?
                .entries
                .iter()
                .any(|e| e.preference.id == preference.id),
            "duplicate provisional identity"
        );
        let roots: Vec<_> = roots.into_iter().collect();
        for p in inputs["generated_preferences"]
            .as_array()
            .into_iter()
            .flatten()
        {
            ensure!(
                p["original_contributors"] != json!(roots)
                    || norm(p["meaning"].as_str().context("generated origin meaning")?)
                        != norm(&preference.text),
                "duplicate generated wanting"
            );
        }
        Ok((preference, roots, contributors))
    }
    pub(crate) fn validate_provisional_preference(&self, c: &Choice) -> Result<()> {
        self.provisional_preference_parts(c).map(|_| ())
    }
    pub(crate) fn admit_provisional_preference(
        &self,
        c: &Choice,
        proposal: EventId,
        review: cycle::Review,
    ) -> Result<EventId> {
        let (preference, roots, contributors) = self.provisional_preference_parts(c)?;
        let ctx = self.project_relational_context()?;
        let snapshot = serde_json::from_value(ctx["inputs"]["snapshot"].clone())?;
        let requirements_revision =
            serde_json::from_value(ctx["inputs"]["requirements_revision"].clone())?;
        let contributors_digest = Hash::of(&[&serde_json::to_vec(&contributors)?]);
        let at = format!(
            "unix-ms:{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis()
        );
        let ledger = Ledger::new(&self.store);
        let expansion = ledger.propose_provisional(
            &Self::task(),
            self.events()?
                .last()
                .map(|(_, e)| vec![e.id])
                .unwrap_or_default(),
            tsr_domain::ProvisionalExpansion {
                preference: preference.clone(),
                proposal,
                source_snapshot: snapshot,
                requirements_revision,
                context_binding: c.binding,
                contributors_digest,
                rationale: c.rationale.clone(),
            },
            &at,
        )?;
        let attestation = tsr_domain::ProvisionalPreferenceReview {
            expansion,
            preference: preference.id,
            proposal,
            source_snapshot: snapshot,
            requirements_revision,
            context_binding: c.binding,
            contributors_digest,
            request_binding: review.binding,
            response_digest: review.response_digest,
            reviewer: review.reviewer.clone(),
            generator: review.generator.clone(),
            decision: review.decision.clone(),
        };
        let record = self.adaptive_record(
            &Record::Coding {
                record: CodingRecord::ProvisionalPreferenceAdmission {
                    admission: Box::new(Admission {
                        choice: c.clone(),
                        preference,
                        roots,
                        contributors,
                        snapshot,
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
                    source: tsr_ledger::provisional::REVIEW_SOURCE.into(),
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
        ledger.admit_provisional(
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
    fn setup() -> Result<(tempfile::TempDir, Engine)> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(browser::tests::spec())?;
        let wants = BTreeMap::from([(
            "orientation".into(),
            "Keep navigation understandable".into(),
        )]);
        e.project_acceptance_requirements(wants.clone())?;
        e.project_lifecycle_open(lifecycle::Admission {
            state_root: e.store.head_root()?,
            training: None,
            tool_ms_ceiling: 100000,
            reason: "Authored provisional mechanics, no provider effects".into(),
            non_tool: Some(budget::Limits {
                model_calls: 10,
                model_tokens: 10000,
                wall_ms: 600000,
                patch_bytes: 10000,
                external_cost_microusd: 0,
            }),
        })?;
        let status = e.project_acceptance_status()?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: status.requirements_revision.unwrap(),
            decision_record: status.decision_record,
            guidance: wants,
        })?;
        e.project_resource_unpriced_admit(e.store.head_root()?)?;
        Ok((dir, e))
    }
    fn input(e: &Engine, parent: PreferenceId, text: &str) -> Result<cycle::Advance> {
        let ctx = e.project_cycle_request()?;
        let relational = e.project_relational_context()?;
        let response=Response {protocol:PROTOCOL.into(),state_root:e.store.head_root()?,attention_binding:e.proposal_attention_binding()?,proposals:vec![Suggestion::ProvisionalPreference {choice:Choice {binding:serde_json::from_value(relational["binding"].clone())?,parents:vec![parent],meaning:text.into(),rationale:"Authored generated hypothesis from the named contributor; no observed success, user endorsement or new permission is claimed.".into()}}],pursuit_expectations:Default::default(),common_event_forecasts:Default::default()};
        let binding = serde_json::from_value(ctx["binding"].clone())?;
        Ok(cycle::Advance {
            binding,
            response: response.clone(),
            review: cycle::Review {
                binding,
                response_digest: Hash::of(&[&serde_json::to_vec(&response)?]),
                state_root: response.state_root,
                source_snapshot: serde_json::from_value(ctx["source_snapshot"].clone())?,
                reviewer: "authored independent reviewer".into(),
                generator: "authored generator".into(),
                decision: "approve".into(),
            },
        })
    }
    #[test]
    fn initial_unknown_single_root_native_identity_recursive_child_and_restart() -> Result<()> {
        let (dir, e) = setup()?;
        let before = e.project_relational_context()?;
        let aims = e.acceptance_current_aims()?;
        assert!(e.project_current_contrast()?.is_none());
        assert!(e.project_current_success()?.is_none());
        assert!(before["history"].as_array().unwrap().is_empty());
        let root = serde_json::from_value(before["inputs"]["parents"][0]["id"].clone())?;
        let a = input(
            &e,
            root,
            "Authored wanting for recoverable orientation under unfamiliar choices",
        )?;
        let head = e.store.head_root()?;
        assert!(e.project_cycle_advance(a.clone()).is_err());
        assert_eq!(head, e.store.head_root()?);
        let result = e.project_cycle_preference_advance(a.clone())?;
        assert_eq!(result["preference_admitted"], true);
        assert_eq!(result["executed"], false);
        assert_eq!(e.project_meaning_advance_receipt(a.clone())?, result);
        assert!(e.project_cycle_preference_advance(a).is_err());
        let current = e.project_relational_context()?;
        let child = &current["inputs"]["generated_preferences"][0];
        assert_eq!(child["eligible"], true);
        assert_eq!(child["fulfillment"], "unknown");
        assert_eq!(child["grants_authority"], false);
        assert!(child.get("success").is_none() && child.get("contrast").is_none());
        assert_eq!(child["original_contributors"], json!([root]));
        let id = serde_json::from_value(child["id"].clone())?;
        let ledger = Ledger::new(&e.store).collective()?;
        let native = ledger.get(&id).unwrap();
        assert!(native.is_open());
        assert_eq!(
            native.history.last(),
            Some(&(PreferenceStatus::Open, Source::Council))
        );
        assert!(native.from_provisional.is_some());
        e.project_cycle_preference_advance(input(
            &e,
            id,
            "Authored wanting for revisitable orientation while considering alternatives",
        )?)?;
        let events = e.events()?;
        for pair in events.windows(2) {
            let a = pair[0]
                .1
                .recorded_at
                .strip_prefix("unix-ms:")
                .unwrap()
                .parse::<u128>()?;
            let b = pair[1]
                .1
                .recorded_at
                .strip_prefix("unix-ms:")
                .unwrap()
                .parse::<u128>()?;
            assert!(
                b >= a,
                "provisional admission must preserve lifecycle chronology"
            );
        }
        let after = e.project_relational_context()?;
        assert_eq!(
            after["inputs"]["generated_preferences"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(after["inputs"]["parents"].as_array().unwrap().len(), 3);
        assert_eq!(e.acceptance_current_aims()?, aims);
        assert_eq!(
            after["inputs"]["acceptance_parents"],
            before["inputs"]["parents"]
        );
        drop(e);
        assert_eq!(
            Engine::open(dir.path())?.project_relational_context()?,
            after
        );
        Ok(())
    }
    #[test]
    fn native_excluded_root_and_child_do_not_resurrect_on_restart() -> Result<()> {
        let (dir, e) = setup()?;
        let ctx = e.project_relational_context()?;
        let root = serde_json::from_value(ctx["inputs"]["parents"][0]["id"].clone())?;
        e.project_cycle_preference_advance(input(
            &e,
            root,
            "Authored child before explicit exclusion",
        )?)?;
        e.append(
            EventKind::StatusSet {
                preference: root,
                status: PreferenceStatus::OutOfScope,
                by: Source::Operator,
            },
            Source::Operator,
        )?;
        let after = e.project_relational_context()?;
        assert_eq!(after["inputs"]["parents"], json!([]));
        assert_eq!(
            after["inputs"]["generated_preferences"][0]["eligible"],
            false
        );
        assert_eq!(e.acceptance_current_aims()?.len(), 1);
        assert_eq!(after["inputs"]["acceptance_parents"], json!([])); // explicitly excluded guidance is not selectable
        drop(e);
        assert_eq!(
            Engine::open(dir.path())?.project_relational_context()?,
            after
        );
        Ok(())
    }
    #[test]
    fn rejection_stale_forged_source_and_duplicate_do_not_admit() -> Result<()> {
        let (_dir, e) = setup()?;
        let ctx = e.project_relational_context()?;
        let root = serde_json::from_value(ctx["inputs"]["parents"][0]["id"].clone())?;
        let a = input(&e, root, "Authored provisional wanting remains unfulfilled")?;
        let head = e.store.head_root()?;
        for decision in ["reject", "unknown", "abstain"] {
            let mut bad = a.clone();
            bad.review.decision = decision.into();
            assert!(e.project_cycle_preference_advance(bad).is_err());
            assert_eq!(e.store.head_root()?, head);
        }
        let mut bad = a.clone();
        bad.review.reviewer = bad.review.generator.clone();
        assert!(e.project_cycle_preference_advance(bad).is_err());
        if let Suggestion::ProvisionalPreference { choice } = &a.response.proposals[0] {
            let mut bad = choice.clone();
            bad.binding = Hash::of(&[b"stale"]);
            assert!(e.validate_provisional_preference(&bad).is_err());
            bad = choice.clone();
            bad.parents = vec![PreferenceId(Hash::of(&[b"foreign"]))];
            assert!(e.validate_provisional_preference(&bad).is_err());
            bad = choice.clone();
            bad.parents.clear();
            assert!(e.validate_provisional_preference(&bad).is_err());
        }
        assert_eq!(e.store.head_root()?, head);
        e.project_cycle_preference_advance(a)?;
        let duplicate = input(&e, root, "Authored provisional wanting remains unfulfilled")?;
        assert!(e.project_cycle_preference_advance(duplicate).is_err());
        assert_eq!(
            e.project_relational_context()?["inputs"]["generated_preferences"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        Ok(())
    }
    #[test]
    fn ledger_attestation_tampering_scope_and_duplicate_admission_fail_closed() -> Result<()> {
        let (_dir, e) = setup()?;
        let ctx = e.project_relational_context()?;
        let root = serde_json::from_value(ctx["inputs"]["parents"][0]["id"].clone())?;
        e.project_cycle_preference_advance(input(&e, root, "Authored provisional ledger proof")?)?;
        let events = e.events()?;
        let (review, evidence) = events
            .iter()
            .find_map(|(_, ev)| match &ev.kind {
                EventKind::ReviewedProvisionalPreferenceAdmission { review, evidence } => {
                    Some(((**review).clone(), *evidence))
                }
                _ => None,
            })
            .unwrap();
        assert!(
            tsr_ledger::provisional::validate(&events, Engine::task(), &review, evidence).is_ok()
        );
        for field in [
            "context",
            "contributors",
            "snapshot",
            "proposal",
            "expansion",
            "reviewer",
        ] {
            let mut bad = review.clone();
            let forged = Hash::of(&[b"forged"]);
            match field {
                "context" => bad.context_binding = forged,
                "contributors" => bad.contributors_digest = forged,
                "snapshot" => bad.source_snapshot = forged,
                "proposal" => bad.proposal = EventId(forged),
                "expansion" => bad.expansion = EventId(forged),
                _ => bad.reviewer = bad.generator.clone(),
            };
            assert!(
                tsr_ledger::provisional::validate(&events, Engine::task(), &bad, evidence).is_err()
            );
        }
        assert!(tsr_ledger::provisional::validate(
            &events,
            Engine::task(),
            &review,
            review.proposal
        )
        .is_err());
        let ep = events.iter().position(|(_, e)| e.id == evidence).unwrap();
        let xp = events
            .iter()
            .position(|(_, e)| e.id == review.expansion)
            .unwrap();
        let mut bad = events.clone();
        bad[ep].1.source = Source::Generator;
        assert!(
            tsr_ledger::provisional::validate(&bad, Engine::task(), &review, evidence).is_err()
        );
        let mut bad = events.clone();
        bad.swap(ep, xp);
        assert!(
            tsr_ledger::provisional::validate(&bad, Engine::task(), &review, evidence).is_err()
        );
        let EventKind::ProvisionalExpanded { expansion } = &events[xp].1.kind else {
            unreachable!()
        };
        let mut foreign = expansion.clone();
        foreign.preference = Preference::from_admitted_text(
            foreign.preference.text.clone(),
            Scope::Task(tsr_domain::TaskId(Hash::of(&[b"other task"]))),
            foreign.preference.parents.clone(),
            Source::Generator,
        )?;
        assert!(tsr_ledger::provisional::basis(&events, Engine::task(), &foreign).is_err());
        let head = e.store.head_root()?;
        assert!(Ledger::new(&e.store)
            .admit_provisional(
                &Engine::task(),
                head,
                review,
                evidence,
                "authored duplicate"
            )
            .is_err());
        assert_eq!(head, e.store.head_root()?);
        Ok(())
    }
    #[test]
    fn interrupted_admission_stop_and_unknown_effects_never_replay() -> Result<()> {
        let (_dir, e) = setup()?;
        let ctx = e.project_relational_context()?;
        let root = serde_json::from_value(ctx["inputs"]["parents"][0]["id"].clone())?;
        let a = input(&e, root, "Authored interrupted provisional wanting")?;
        let head = e.store.head_root()?;
        let error = e.write(|| -> Result<()> {
            e.project_cycle_preference_advance(a.clone())?;
            anyhow::bail!("authored interruption before commit")
        });
        assert!(error.is_err());
        assert_eq!(head, e.store.head_root()?);
        assert!(e.project_relational_context()?["inputs"]
            .get("generated_preferences")
            .is_none());
        e.project_lifecycle_close_advisory("Authored Stop".into())?;
        assert!(e
            .project_cycle_preference_advance(input(&e, root, "Authored stopped wanting")?)
            .is_err());
        let (_dir, e) = setup()?;
        let ctx = e.project_relational_context()?;
        let root = serde_json::from_value(ctx["inputs"]["parents"][0]["id"].clone())?;
        let a = input(&e, root, "Authored blocked wanting")?;
        e.project_resource_reserve_unpriced(budget::UnpricedReserve {
            request_id: "authored unknown request".into(),
            state_root: e.store.head_root()?,
            envelope: "authored".into(),
            model: "authored".into(),
            tokens_upper: 10,
        })?;
        let head = e.store.head_root()?;
        assert!(e.project_cycle_preference_advance(a).is_err());
        assert_eq!(head, e.store.head_root()?);
        assert_eq!(e.project_cycle_request()?["branch"], "reconcile");
        Ok(())
    }
}
