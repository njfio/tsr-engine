//! Export saved pre-action rows. No reconstruction of a forecast after its return.
use super::*;
impl Engine {
    pub fn project_eligibility_view(&self) -> Result<serde_json::Value> {
        let p = self.project_lifecycle_view()?;
        let input = self.project_training_evidence(&lifecycle::now()?)?;
        Ok(
            serde_json::json!({"closure":tsr_learn::project_eligibility::closure_digest(&serde_json::to_value(&p)?)?,"admission":p.admission.training,"eligibility":tsr_learn::project_eligibility::evaluate(&input.artifacts[0],&input.eligibility[0],&lifecycle::now()?)?}),
        )
    }
    pub fn project_training_evidence(&self, as_of: &str) -> Result<tsr_learn::schema::Input> {
        let a = self.project_dataset(as_of)?;
        let projection = self.project_lifecycle_view()?;
        let evidence = self.project_feature_evidence(&self.events()?, as_of)?;
        let proof = tsr_learn::project_eligibility::Proof {
            task: projection.task,
            evidence,
        };
        // Validate even when the result is explicitly withheld.
        tsr_learn::project_eligibility::evaluate(&a, &proof, as_of)?;
        Ok(tsr_learn::schema::Input {
            artifacts: vec![a],
            eligibility: vec![proof],
        })
    }
    pub fn project_eligibility_review(
        &self,
        review: tsr_learn::project_eligibility::Review,
    ) -> Result<serde_json::Value> {
        self.write(|| {
            let p = self.project_lifecycle_view()?;
            ensure!(
                p.closed && p.admission.training.is_some(),
                "prospective training admission and closure required"
            );
            ensure!(
                tsr_learn::project_eligibility::closure_digest(&serde_json::to_value(&p)?)?
                    == review.closure,
                "stale closure review"
            );
            ensure!(
                review.reviewer == p.admission.training.as_ref().unwrap().reviewer
                    && short(&review.independence)
                    && review.complete_instrumented_coverage,
                "named independent review and explicit coverage required"
            );
            let side = self.shadow_store()?;
            side.write(|| {
                for (_, e) in side.events()? {
                    if let EventKind::Observed {
                        observation: Observation::Evidence { source, digest },
                    } = e.kind
                    {
                        if source == tsr_learn::project_eligibility::SOURCE {
                            let bytes = fs::read(side.directory.join("cas").join(digest.to_hex()))?;
                            ensure!(
                                Hash::of(&[&bytes]) == digest
                                    && serde_json::from_slice::<serde_json::Value>(&bytes)?
                                        == serde_json::to_value(&review)?,
                                "eligibility review already recorded; no silent replacement"
                            );
                            return Ok(());
                        }
                    }
                }
                let digest = side.blob(&serde_json::to_vec(&review)?)?;
                side.append(
                    EventKind::Observed {
                        observation: Observation::Evidence {
                            source: tsr_learn::project_eligibility::SOURCE.into(),
                            digest,
                        },
                    },
                    Source::Operator,
                )?;
                Ok(())
            })?;
            let input = self.project_training_evidence(&lifecycle::now()?)?;
            Ok(serde_json::to_value(
                tsr_learn::project_eligibility::evaluate(
                    &input.artifacts[0],
                    &input.eligibility[0],
                    &lifecycle::now()?,
                )?,
            )?)
        })
    }
    pub fn project_dataset(&self, as_of: &str) -> Result<tsr_learn::project_receipts::Artifact> {
        let cutoff = tsr_learn::maturity::timestamp(as_of)?;
        ensure!(
            cutoff <= tsr_learn::maturity::timestamp(&lifecycle::now()?)?,
            "future project dataset cutoff"
        );
        let events = self.events()?;
        let side = if self.directory.join("shadow").exists() {
            self.shadow_store()?.events()?
        } else {
            vec![]
        };
        let mut rows = vec![];
        for (_, event) in &side {
            if let EventKind::Observed {
                observation: Observation::Evidence { source, digest },
            } = &event.kind
            {
                if source != "common-event-shadow-receipt@v1" {
                    continue;
                }
                if tsr_learn::maturity::timestamp(&lifecycle::timestamp(&event.recorded_at)?)?
                    > cutoff
                {
                    continue;
                }
                let bytes = fs::read(self.directory.join("shadow/cas").join(digest.to_hex()))?;
                ensure!(
                    event.source == Source::Rules && Hash::of(&[&bytes]) == *digest,
                    "invalid prediction provenance"
                );
                let value: serde_json::Value = serde_json::from_slice(&bytes)?;
                if value["adapter"] != "bounded-project@v1" {
                    continue;
                }
                let at = value["prediction_as_of"]
                    .as_str()
                    .ok_or_else(|| anyhow::anyhow!("missing pre-action cutoff"))?;
                if tsr_learn::maturity::timestamp(at)? > cutoff {
                    continue;
                }
                // Missing legacy features stay visible as an explicit export failure, never a reconstructed forecast.
                let features: tsr_learn::project_receipts::Features =
                    serde_json::from_value(value["project_observation_features"].clone())?;
                let count = value["prefix_events"]
                    .as_u64()
                    .ok_or_else(|| anyhow::anyhow!("missing original prefix length"))?
                    as usize;
                ensure!(count <= events.len(), "prediction prefix exceeds source");
                let mut evidence = self.project_feature_evidence(&events[..count], at)?;
                let side_count = value["project_side_prefix_events"]
                    .as_u64()
                    .ok_or_else(|| anyhow::anyhow!("legacy prediction lacks causal side prefix"))?
                    as usize;
                ensure!(
                    side_count <= evidence.side_events.len(),
                    "prediction side prefix exceeds source"
                );
                evidence.side_events.truncate(side_count);
                let used: BTreeSet<_> = evidence
                    .events
                    .iter()
                    .chain(&evidence.side_events)
                    .filter_map(|(_, e)| match &e.kind {
                        EventKind::Observed {
                            observation: Observation::Evidence { digest, .. },
                        } => Some(*digest),
                        _ => None,
                    })
                    .collect();
                evidence.blobs.retain(|digest, _| used.contains(digest));
                rows.push(tsr_learn::project_receipts::Row {
                    prediction: event.id,
                    prediction_event: event.clone(),
                    prediction_blob: String::from_utf8(bytes)?,
                    candidate: serde_json::from_value(value["candidate"].clone())?,
                    state: serde_json::from_value(value["state_root"].clone())?,
                    source_version: serde_json::from_value(value["source_version"].clone())?,
                    family: value["declared_family"].as_str().unwrap_or("").into(),
                    lineage: value["declared_lineage"].as_str().unwrap_or("").into(),
                    evidence,
                    features,
                });
            }
        }
        let lifecycle = match self.project_lifecycle_view() {
            Ok(p) => serde_json::to_value(p)?,
            Err(error) => {
                serde_json::json!({"status":"unavailable","reason":error.to_string(),"label":null})
            }
        };
        Ok(tsr_learn::project_receipts::Artifact::build(
            as_of.into(),
            Hash::of(&[&serde_json::to_vec(&(&events, &side))?]),
            rows,
            lifecycle,
        )?)
    }
}
