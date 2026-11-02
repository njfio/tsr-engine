//! Explicit, versioned opt-in; calibrated scores never create action authority.
use super::*;
use std::collections::BTreeMap;
use tsr_domain::selection::{Policy, Trace};
pub const VERSION: &str = "supported-common-event-selection@v1";
const CONFIG: &str = "operator-tier2-policy@v1";
const AUDIT: &str = "tier2-gate-receipt@v1";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub version: String,
    pub prediction: tsr_learn::shadow::Config,
    pub validation_report: Hash,
    pub selection: Policy,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Update {
    pub state_root: Hash,
    /// None is explicit rollback. Historical records remain intact.
    pub enabled: Option<Config>,
    pub reason: String,
}
pub(super) struct Choice<'a> {
    pub id: &'a str,
    pub candidate: &'a Candidate,
    pub assessment: Option<&'a tsr_domain::decision::Assessment>,
}
pub(super) struct Applied {
    pub trace: Trace,
    pub probabilities: BTreeMap<String, f64>,
    pub scores: BTreeMap<String, f64>,
    pub receipt: Option<EventId>,
}
fn validate(config: &Config) -> Result<()> {
    ensure!(
        config.version == VERSION,
        "unsupported Tier2 policy version"
    );
    config.selection.validate().map_err(anyhow::Error::msg)?;
    ensure!(
        config.prediction.event == tsr_learn::DECISION_EVENT,
        "Tier2 event mismatch"
    );
    ensure!(
        config.prediction.dataset_directory.is_some() && config.prediction.training_as_of.is_some(),
        "Tier2 requires pinned training evidence and cutoff"
    );
    ensure!(
        !config.prediction.family.trim().is_empty()
            && config.prediction.family.len() <= 128
            && !config.prediction.lineage.trim().is_empty()
            && config.prediction.lineage.len() <= 128,
        "bounded family/lineage required"
    );
    Ok(())
}
fn rescore(assessment: &tsr_domain::decision::Assessment, p: f64) -> Result<f64> {
    ensure!(
        assessment.kernel == tsr_domain::decision::KERNEL && assessment.unresolved.is_empty(),
        "Tier2 requires known conserved obligations"
    );
    ensure!(
        p.is_finite() && (0.0..=1.0).contains(&p),
        "invalid calibrated probability"
    );
    ensure!(
        [
            assessment.gain,
            assessment.effort,
            assessment.continuity_cost,
            assessment.score
        ]
        .iter()
        .all(|v| v.is_finite())
            && assessment.gain > 0.0
            && assessment.score > 0.0
            && assessment.effort >= 0.0
            && assessment.continuity_cost >= 0.0,
        "Tier2 cannot admit a rejected or nonproductive action"
    );
    // Gain already includes conserved standing losses; never add them back.
    // Keep the factual continuity charge rather than reducing it to favour switching.
    Ok(p * assessment.gain - assessment.effort - assessment.continuity_cost)
}
impl Engine {
    fn tier2_records(&self) -> Result<Vec<(String, serde_json::Value)>> {
        let mut records = vec![];
        for (_, e) in self.events()? {
            if let EventKind::Observed {
                observation: Observation::Evidence { source, digest },
            } = e.kind
            {
                if source != CONFIG && source != AUDIT {
                    continue;
                }
                ensure!(
                    e.source
                        == if source == CONFIG {
                            Source::Operator
                        } else {
                            Source::Rules
                        },
                    "invalid Tier2 provenance"
                );
                let bytes = fs::read(self.directory.join("cas").join(digest.to_hex()))?;
                ensure!(Hash::of(&[&bytes]) == digest, "changed Tier2 record");
                records.push((source, serde_json::from_slice(&bytes)?));
            }
        }
        Ok(records)
    }
    fn tier2_config(&self) -> Result<Option<Config>> {
        let mut config = None;
        for (source, value) in self.tier2_records()? {
            if source == CONFIG {
                let update: Update = serde_json::from_value(value)?;
                if let Some(c) = &update.enabled {
                    validate(c)?;
                }
                config = update.enabled;
            }
        }
        Ok(config)
    }
    pub fn update_tier2(&self, update: Update) -> Result<serde_json::Value> {
        self.write(|| {
            ensure!(
                update.state_root == self.store.head_root()?,
                "stale Tier2 update"
            );
            ensure!(
                !update.reason.trim().is_empty() && update.reason.len() <= 512,
                "bounded operator reason required"
            );
            if let Some(config) = &update.enabled {
                validate(config)?;
                ensure!(
                    tsr_learn::maturity::timestamp(
                        config.prediction.training_as_of.as_ref().unwrap()
                    )? <= tsr_learn::maturity::timestamp(&crate::hook::now_rfc3339())?,
                    "future Tier2 training cutoff"
                );
            }
            let digest = self.blob(&serde_json::to_vec(&update)?)?;
            self.append(
                EventKind::Observed {
                    observation: Observation::Evidence {
                        source: CONFIG.into(),
                        digest,
                    },
                },
                Source::Operator,
            )?;
            self.tier2_view()
        })
    }
    pub fn tier2_view(&self) -> Result<serde_json::Value> {
        Ok(
            serde_json::json!({"state_root":self.store.head_root()?,"policy":self.tier2_config()?,"records":self.tier2_records()?,"opt_in_is_not_eligibility":true,"disabled_default":"historical shadow/fallback","authority":"unchanged adapter and execution checks"}),
        )
    }
    pub(super) fn tier2_audit(&self, value: serde_json::Value) -> Result<EventId> {
        let digest = self.blob(&serde_json::to_vec(&value)?)?;
        self.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    source: AUDIT.into(),
                    digest,
                },
            },
            Source::Rules,
        )
    }
    pub(super) fn tier2_select(
        &self,
        state: Hash,
        adapter: &str,
        events: &[(i64, Event)],
        choices: &[Choice<'_>],
    ) -> Result<Option<Applied>> {
        let Some(config) = self.tier2_config()? else {
            return Ok(None);
        };
        let result = self.tier2_evaluate(&config, state, adapter, events, choices);
        match result {
            Ok((mut applied, receipts)) => {
                let receipt=self.tier2_audit(serde_json::json!({"version":VERSION,"state_root":state,"adapter":adapter,"status":"applied","policy":config,"predictions":receipts,"trace":applied.trace,"scores":applied.scores,"authority_expanded":false}))?;
                applied.receipt = Some(receipt);
                Ok(Some(applied))
            }
            Err(error) => {
                self.tier2_audit(serde_json::json!({"version":VERSION,"state_root":state,"adapter":adapter,"status":"fallback","reason":error.to_string(),"policy":config,"authority_expanded":false}))?;
                Ok(None)
            }
        }
    }
    fn tier2_evaluate(
        &self,
        config: &Config,
        state: Hash,
        adapter: &str,
        events: &[(i64, Event)],
        choices: &[Choice<'_>],
    ) -> Result<(Applied, Vec<serde_json::Value>)> {
        ensure!(
            adapter == "bounded-project@v1",
            "adapter has no compatible operational common event"
        );
        ensure!(
            !choices.is_empty() && choices.len() <= 16,
            "bounded admissible alternatives required"
        );
        let life = self.project_lifecycle_prefix(events)?;
        let admission = life
            .admission
            .training
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing prospective episode admission"))?;
        ensure!(
            admission.origin == tsr_learn::project_eligibility::Origin::Operational
                && !life.closed
                && life.incomplete_executions == 0,
            "current episode is not operational and ready"
        );
        ensure!(
            admission.family == config.prediction.family
                && admission.lineage == config.prediction.lineage,
            "query admission applicability mismatch"
        );
        let budget = self.project_budget()?;
        ensure!(
            budget.active
                && !budget.exceeded
                && !budget.stopped
                && budget.unknown_usage == 0
                && budget.unknown_compute_requests == 0
                && budget.unsettled_requests == 0,
            "current resource coverage unavailable or exhausted"
        );
        let as_of = super::adaptive::coding::project::lifecycle::now()?;
        let source = self.project_acceptance_status()?.snapshot;
        let evidence = self.project_feature_evidence(events, &as_of)?;
        let mut probabilities = BTreeMap::new();
        let mut scores = BTreeMap::new();
        let mut receipts = vec![];
        for choice in choices {
            self.project_tier2_action_budget(choice.id)?;
            ensure!(
                !probabilities.contains_key(choice.id),
                "duplicate alternative identity"
            );
            let assessment = choice
                .assessment
                .ok_or_else(|| anyhow::anyhow!("missing shared obligation assessment"))?;
            let prediction = tsr_learn::shadow::evaluate_activation(
                &config.prediction,
                tsr_learn::shadow::Snapshot {
                    tool_ms_ceiling: Some(life.admission.tool_ms_ceiling),
                    state,
                    source_version: source,
                    candidate: choice.candidate,
                    events: &life.events,
                    as_of: &as_of,
                    adapter: "project-receipt-common-event@v1",
                },
                &evidence,
                config.validation_report,
                &admission.clone_group,
                admission.original_task,
            )?;
            ensure!(
                prediction["status"] == "activation_eligible",
                "prediction did not pass activation gates"
            );
            let p = prediction["calibrated_probability"]
                .as_f64()
                .ok_or_else(|| anyhow::anyhow!("missing calibrated probability"))?;
            scores.insert(choice.id.to_owned(), rescore(assessment, p)?);
            probabilities.insert(choice.id.to_owned(), p);
            receipts.push(prediction);
        }
        ensure!(
            self.project_acceptance_status()?.snapshot == source,
            "project source changed during Tier2 validation"
        );
        let viable = scores
            .iter()
            .filter(|(_, s)| **s > 0.0)
            .map(|(id, s)| (id.clone(), *s))
            .collect::<Vec<_>>();
        let trace = tsr_domain::selection::select(&config.selection, state, &viable)
            .map_err(anyhow::Error::msg)?;
        if let Some(id) = &trace.selected {
            self.project_tier2_action_budget(id)?;
        }
        Ok((
            Applied {
                trace,
                probabilities,
                scores,
                receipt: None,
            },
            receipts,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn config(path: &Path) -> Config {
        Config {
            version: VERSION.into(),
            prediction: tsr_learn::shadow::Config {
                bundle_directory: path.join("absent"),
                bundle_id: Hash::of(&[b"absent"]),
                dataset_directory: Some(path.join("absent-data")),
                event: tsr_learn::DECISION_EVENT.into(),
                family: "fixture".into(),
                lineage: "fixture".into(),
                training_as_of: Some("2026-01-01T00:00:00Z".into()),
            },
            validation_report: Hash::of(&[b"not-validation"]),
            selection: Policy {
                margin: 0.1,
                epsilon: 0.0,
                seed: 3,
            },
        }
    }
    #[test]
    fn disabled_fallback_restart_and_explicit_rollback_preserve_authority() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let root = e.store.head_root()?;
        for adapter in ["notebook@v1", "adaptive@v1", "bounded-project@v1"] {
            assert!(e.tier2_select(root, adapter, &[], &[])?.is_none());
        }
        assert_eq!(root, e.store.head_root()?);
        assert!(!dir.path().join("shadow").exists());
        e.update_tier2(Update {
            state_root: root,
            enabled: Some(config(dir.path())),
            reason: "synthetic gate test, no activation".into(),
        })?;
        for adapter in ["notebook@v1", "adaptive@v1", "bounded-project@v1"] {
            assert!(e
                .tier2_select(e.store.head_root()?, adapter, &e.events()?, &[])?
                .is_none());
        }
        let view = e.tier2_view()?;
        assert_eq!(view, Engine::open(dir.path())?.tier2_view()?);
        assert!(view["records"]
            .as_array()
            .unwrap()
            .iter()
            .skip(1)
            .all(|v| v[1]["status"] == "fallback"));
        e.update_tier2(Update {
            state_root: e.store.head_root()?,
            enabled: None,
            reason: "rollback retains evidence".into(),
        })?;
        let root = e.store.head_root()?;
        assert!(e
            .tier2_select(root, "notebook@v1", &e.events()?, &[])?
            .is_none());
        assert_eq!(root, e.store.head_root()?);
        assert!(Engine::open(dir.path())?.tier2_config()?.is_none());
        let mut bad = config(dir.path());
        bad.version = "future".into();
        assert!(e
            .update_tier2(Update {
                state_root: root,
                enabled: Some(bad),
                reason: "invalid".into()
            })
            .is_err());
        assert_eq!(root, e.store.head_root()?);
        assert!(e
            .update_tier2(Update {
                state_root: Hash::of(&[b"stale"]),
                enabled: None,
                reason: "stale".into()
            })
            .is_err());
        Ok(())
    }
    #[test]
    fn generated_policy_cannot_enable_or_disable_controller() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let update = Update {
            state_root: e.store.head_root()?,
            enabled: Some(config(dir.path())),
            reason: "forged".into(),
        };
        let digest = e.blob(&serde_json::to_vec(&update)?)?;
        e.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    source: CONFIG.into(),
                    digest,
                },
            },
            Source::Generator,
        )?;
        assert!(e.tier2_config().is_err());
        Ok(())
    }
    #[test]
    fn probability_changes_ranking_without_restoring_standing_loss() -> Result<()> {
        let a = tsr_domain::decision::Assessment {
            kernel: tsr_domain::decision::KERNEL.into(),
            score: 0.5,
            gain: 0.8,
            standing_loss: 0.2,
            continuity_cost: 0.04,
            projected: BTreeMap::new(),
            unresolved: vec![],
            observation: BTreeMap::new(),
            selected: vec![],
            probability: 0.8,
            effort: 0.1,
            uncertainty_policy: "fixture".into(),
        };
        assert!((rescore(&a, 0.8)? - 0.5).abs() < 1e-12);
        assert!(rescore(&a, 0.1)? < 0.0);
        assert_eq!(rescore(&a, 0.8)?, rescore(&a, 0.8)?);
        for p in [f64::NAN, -0.1, 1.1] {
            assert!(rescore(&a, p).is_err());
        }
        let mut rejected = a.clone();
        rejected.score = -0.1;
        assert!(rescore(&rejected, 1.0).is_err());
        rejected = a;
        rejected.kernel = "unknown".into();
        assert!(rescore(&rejected, 1.0).is_err());
        Ok(())
    }
}
