//! Shared operator policy and durable selection evidence for both adapters.
use super::*;
use tsr_domain::selection::{Policy, Trace};
const POLICY: &str = "operator-decision-policy@v1";
const AUDIT: &str = "selection-learning-intervention@v1";
const TRACE: &str = "bounded-selection-trace@v1";
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Update {
    pub state_root: Hash,
    pub policy: Policy,
    pub reason: String,
}
impl Engine {
    pub fn decision_policy(&self) -> Result<Option<Policy>> {
        let mut current = None;
        for (_, event) in self.events()? {
            if let EventKind::Observed {
                observation: Observation::Evidence { digest, source },
            } = event.kind
            {
                if source != POLICY {
                    continue;
                }
                ensure!(
                    event.source == Source::Operator,
                    "decision policy requires operator provenance"
                );
                let bytes = fs::read(self.directory.join("cas").join(digest.to_hex()))?;
                ensure!(
                    Hash::of(&[&bytes]) == digest,
                    "changed decision policy evidence"
                );
                let update: Update = serde_json::from_slice(&bytes)?;
                update.policy.validate().map_err(anyhow::Error::msg)?;
                current = Some(update.policy);
            }
        }
        Ok(current)
    }
    pub fn update_decision_policy(&self, update: Update) -> Result<Option<Policy>> {
        self.write(|| {
            ensure!(
                update.state_root == self.store.head_root()?,
                "stale decision policy update"
            );
            update.policy.validate().map_err(anyhow::Error::msg)?;
            ensure!(
                !update.reason.trim().is_empty() && update.reason.len() <= 512,
                "bounded policy reason required"
            );
            let digest = self.blob(&serde_json::to_vec(&update)?)?;
            self.append(
                EventKind::Observed {
                    observation: Observation::Evidence {
                        digest,
                        source: POLICY.into(),
                    },
                },
                Source::Operator,
            )?;
            self.decision_policy()
        })
    }
    pub(super) fn select_with_policy(
        &self,
        state: Hash,
        candidates: &[(String, f64)],
    ) -> Result<Option<Trace>> {
        self.decision_policy()?
            .map(|policy| {
                let trace = tsr_domain::selection::select(&policy, state, candidates)
                    .map_err(anyhow::Error::msg)?;
                let digest = self.blob(&serde_json::to_vec(&trace)?)?;
                self.append(
                    EventKind::Observed {
                        observation: Observation::Evidence {
                            digest,
                            source: TRACE.into(),
                        },
                    },
                    Source::Rules,
                )?;
                Ok(trace)
            })
            .transpose()
    }
    pub(super) fn record_selection_intervention(
        &self,
        trace: &Trace,
        intervention: &str,
        scores: &[(String, f64)],
        eligible: &[String],
    ) -> Result<()> {
        let comparison = tsr_domain::selection::counterfactual(trace, scores, eligible)
            .map_err(anyhow::Error::msg)?;
        let value = serde_json::json!({"version":AUDIT,"factual_trace_digest":Hash::of(&[&serde_json::to_vec(trace)?]),
            "factual_trace":trace,"intervention":intervention,"comparison":comparison});
        let digest = self.blob(&serde_json::to_vec(&value)?)?;
        self.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    digest,
                    source: AUDIT.into(),
                },
            },
            Source::Rules,
        )?;
        Ok(())
    }
    pub fn decision_policy_view(&self) -> Result<serde_json::Value> {
        let mut traces = vec![];
        let mut interventions = vec![];
        let mut trace_values = std::collections::BTreeMap::new();
        for (_, event) in self.events()? {
            if let EventKind::Observed {
                observation: Observation::Evidence { digest, source },
            } = event.kind
            {
                if source == AUDIT {
                    ensure!(
                        event.source == Source::Rules,
                        "selection intervention requires controller provenance"
                    );
                    let bytes = fs::read(self.directory.join("cas").join(digest.to_hex()))?;
                    ensure!(
                        Hash::of(&[&bytes]) == digest,
                        "changed selection intervention"
                    );
                    let value: serde_json::Value = serde_json::from_slice(&bytes)?;
                    let reference = value["factual_trace_digest"]
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("missing factual trace reference"))?;
                    let factual = trace_values.get(reference).ok_or_else(|| {
                        anyhow::anyhow!("intervention references no preceding controller trace")
                    })?;
                    ensure!(
                        &value["factual_trace"] == factual,
                        "intervention factual trace mismatch"
                    );
                    if !value["comparison"]["factual"].is_null() {
                        ensure!(
                            &value["comparison"]["factual"] == factual,
                            "comparison factual trace mismatch"
                        );
                    }
                    interventions.push(value);
                }
                if source == TRACE {
                    ensure!(
                        event.source == Source::Rules,
                        "selection trace requires controller provenance"
                    );
                    let bytes = fs::read(self.directory.join("cas").join(digest.to_hex()))?;
                    ensure!(Hash::of(&[&bytes]) == digest, "changed selection trace");
                    trace_values.insert(
                        digest.to_hex(),
                        serde_json::from_slice::<serde_json::Value>(&bytes)?,
                    );
                    traces.push(serde_json::from_slice::<Trace>(&bytes)?);
                }
            }
        }
        Ok(
            serde_json::json!({"state_root":self.store.head_root()?,"policy":self.decision_policy()?,"legacy_if_unconfigured":true,"traces":traces,"learning_interventions":interventions,"calibration":"engineering proxy scores; no validated Tier 2 claim"}),
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn operator_policy_restart_abstention_and_provenance() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        assert!(e.decision_policy()?.is_none());
        let policy = Policy {
            margin: 0.1,
            epsilon: 0.0,
            seed: 19,
        };
        e.update_decision_policy(Update {
            state_root: e.store.head_root()?,
            policy: policy.clone(),
            reason: "Bounded selection control".into(),
        })?;
        assert_eq!(
            Engine::open(dir.path())?.decision_policy()?,
            Some(policy.clone())
        );
        let offered = vec![("a".into(), 1.0), ("b".into(), 0.95)];
        let result = e
            .select_with_policy(e.store.head_root()?, &offered)?
            .unwrap();
        assert!(result.selected.is_none());
        assert_eq!(
            e.decision_policy_view()?["traces"][0]["reason"],
            "near_tie_abstention"
        );
        assert_eq!(
            e.decision_policy_view()?,
            Engine::open(dir.path())?.decision_policy_view()?
        );
        let update = Update {
            state_root: e.store.head_root()?,
            policy,
            reason: "untrusted".into(),
        };
        let digest = e.blob(&serde_json::to_vec(&update)?)?;
        e.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    digest,
                    source: POLICY.into(),
                },
            },
            Source::Generator,
        )?;
        assert!(e.decision_policy().is_err());
        Ok(())
    }
    #[test]
    fn generated_trace_cannot_impersonate_controller() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let trace = tsr_domain::selection::select(
            &Policy {
                margin: 0.0,
                epsilon: 0.0,
                seed: 0,
            },
            e.store.head_root()?,
            &[],
        )
        .unwrap();
        let digest = e.blob(&serde_json::to_vec(&trace)?)?;
        e.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    digest,
                    source: TRACE.into(),
                },
            },
            Source::Generator,
        )?;
        assert!(e.decision_policy_view().is_err());
        Ok(())
    }
    #[test]
    fn intervention_must_reference_recorded_matching_trace() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let value = serde_json::json!({"factual_trace_digest":Hash::of(&[b"nonexistent"]),"factual_trace":{}});
        let digest = e.blob(&serde_json::to_vec(&value)?)?;
        e.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    digest,
                    source: AUDIT.into(),
                },
            },
            Source::Rules,
        )?;
        assert!(e.decision_policy_view().is_err());
        Ok(())
    }
}
