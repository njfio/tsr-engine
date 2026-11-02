//! Explicit diagnostic expectation, never a learned controller probability or execution grant.
use super::*;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Admission {
    pub state_root: Hash,
    pub snapshot: Hash,
    pub commitment: EventId,
    pub action: String,
    pub probability: f64,
    pub evaluator: String,
}
impl Engine {
    fn evaluator_source(&self, id: EventId, source: Source) -> Result<()> {
        ensure!(
            self.events()?
                .iter()
                .any(|(_, e)| e.id == id && e.source == source),
            "evaluator evidence provenance mismatch"
        );
        Ok(())
    }
    fn validate_evaluator_record(
        &self,
        id: EventId,
        admission: &Admission,
        forecast: &Forecast,
    ) -> Result<()> {
        self.evaluator_source(id, Source::Operator)?;
        ensure!(
            forecast.source
                == Forecaster::External(format!("evaluator-authored:{}", admission.evaluator))
                && forecast.raw.get() == admission.probability
                && forecast.calibrated.is_none(),
            "evaluator forecast attribution mismatch"
        );
        let events = self.events()?;
        let candidate = events
            .iter()
            .find_map(|(_, e)| match e.kind {
                EventKind::Committed { candidate, .. }
                    if e.id == admission.commitment && e.source == Source::Rules =>
                {
                    Some(candidate)
                }
                _ => None,
            })
            .ok_or_else(|| anyhow::anyhow!("evaluator native commitment missing"))?;
        let position=events.iter().position(|(_,e)| matches!(&e.kind, EventKind::Forecast{forecast:f,candidate:c} if f==forecast && *c==candidate) && e.source==Source::Operator).ok_or_else(|| anyhow::anyhow!("evaluator native forecast missing"))?;
        ensure!(
            forecast.state_version == position as u64
                && events.get(position + 1).is_some_and(|(_, e)| e.id == id),
            "evaluator forecast sequence mismatch"
        );
        let all = self.store.all()?;
        let sequence = events[position].0;
        let prefix: Vec<String> = all
            .iter()
            .filter(|(seq, _)| *seq < sequence)
            .map(|(_, e)| e.id.to_string())
            .collect();
        ensure!(
            Hash::of(&prefix.iter().map(|s| s.as_bytes()).collect::<Vec<_>>())
                == admission.state_root,
            "evaluator admission root mismatch"
        );
        ensure!(self.records()?.iter().any(|(_,r)|matches!(r,Record::Pending{commitment,action,..} if *commitment==admission.commitment && *action==admission.action)),"evaluator action mismatch");
        Ok(())
    }
    fn validate_evaluator_return(
        &self,
        admission: &Admission,
        returned: EventId,
        receipt: EventId,
        success: bool,
    ) -> Result<()> {
        let records = self.records()?;
        let intent = records
            .iter()
            .find_map(|(id, r)| match r {
                Record::Coding {
                    record: CodingRecord::Intent { plan },
                } if plan.commitment == admission.commitment
                    && plan.action == admission.action
                    && plan.source_before == admission.snapshot =>
                {
                    Some(*id)
                }
                _ => None,
            })
            .ok_or_else(|| anyhow::anyhow!("evaluator execution binding mismatch"))?;
        self.evaluator_source(intent, Source::Rules)?;
        self.evaluator_source(receipt, Source::Rules)?;
        self.evaluator_source(returned, Source::Rules)?;
        ensure!(records.iter().any(|(id,r)|*id==receipt && matches!(r,Record::Coding{record:CodingRecord::Receipt{commitment,verdict,..}} if *commitment==admission.commitment && ((*verdict==Verdict::Passed)||(!success && *verdict==Verdict::FailedTests)))),"evaluator receipt outcome mismatch");
        let criteria = records
            .iter()
            .find_map(|(_, r)| match r {
                Record::Pending {
                    commitment,
                    action,
                    criteria,
                    ..
                } if *commitment == admission.commitment && *action == admission.action => {
                    Some(criteria)
                }
                _ => None,
            })
            .ok_or_else(|| anyhow::anyhow!("evaluator pending criteria missing"))?;
        ensure!(records.iter().any(|(id,r)|*id==returned && matches!(r,Record::Returned{commitment,action,success:s,input} if *commitment==admission.commitment && *action==admission.action && *s==success && CriteriaAdapter.success(criteria,input)==success && input.met.iter().all(|c|criteria.contains(c)))),"evaluator return identity mismatch");
        Ok(())
    }
    pub fn project_evaluator_forecast(&self, admission: Admission) -> Result<EventId> {
        self.write(|| {
            ensure!(self.store.head_root()? == admission.state_root, "stale evaluator state");
            ensure!(short(&admission.evaluator), "bounded evaluator attribution required");
            let probability = Probability::new(admission.probability)?;
            let pending = self.adaptive_state_inner()?.pending.ok_or_else(|| anyhow::anyhow!("native commitment required"))?;
            ensure!(pending.commitment == admission.commitment && pending.action == admission.action, "evaluator action/commitment mismatch");
            self.evaluator_source(admission.commitment, Source::Rules)?;
            let spec = self.project_configuration()?;
            ensure!(digest(&self.project_snapshot(&spec)?)? == admission.snapshot, "stale evaluator snapshot");
            ensure!(!self.records()?.iter().any(|(_, r)| matches!(r, Record::Coding { record: CodingRecord::Intent { plan } } if plan.commitment == admission.commitment) || matches!(r, Record::Coding { record: CodingRecord::EvaluatorForecast { admission:a, .. } } if a.commitment == admission.commitment)), "execution attempted or evaluator forecast already frozen");
            let candidate = self.events()?.iter().find_map(|(_, e)| match e.kind { EventKind::Committed { candidate, .. } if e.id == admission.commitment => Some(candidate), _ => None }).ok_or_else(|| anyhow::anyhow!("native commitment missing"))?;
            let forecast = Forecast::new(OutcomeDefId(Hash::of(&[b"configured_criteria_met@v1"])), Horizon::TaskClose, Forecaster::External(format!("evaluator-authored:{}", admission.evaluator)), probability, self.events()?.len() as u64);
            self.append(EventKind::Forecast { forecast:forecast.clone(), candidate }, Source::Operator)?;
            self.adaptive_record(&Record::Coding { record:CodingRecord::EvaluatorForecast { admission, forecast } }, Source::Operator)
        })
    }
    pub fn project_evaluator_forecasts(&self) -> Result<serde_json::Value> {
        let mut rows = vec![];
        for (id, r) in self.records()? {
            match r {
                Record::Coding {
                    record:
                        CodingRecord::EvaluatorForecast {
                            admission,
                            forecast,
                        },
                } => {
                    self.validate_evaluator_record(id, &admission, &forecast)?;
                    rows.push(
                        serde_json::json!({"record":id,"admission":admission,"forecast":forecast}),
                    );
                }
                Record::Coding {
                    record:
                        CodingRecord::EvaluatorResolution {
                            admission,
                            commitment,
                            returned,
                            receipt,
                            forecast,
                            success,
                        },
                } => {
                    self.evaluator_source(id, Source::Rules)?;
                    let (a, f) = self
                        .records()?
                        .into_iter()
                        .find_map(|(key, r)| match r {
                            Record::Coding {
                                record:
                                    CodingRecord::EvaluatorForecast {
                                        admission: a,
                                        forecast: f,
                                    },
                            } if key == admission => Some((a, f)),
                            _ => None,
                        })
                        .ok_or_else(|| anyhow::anyhow!("evaluator resolution admission missing"))?;
                    self.validate_evaluator_record(admission, &a, &f)?;
                    ensure!(
                        a.commitment == commitment && f == forecast,
                        "evaluator resolution forecast mismatch"
                    );
                    self.validate_evaluator_return(&a, returned, receipt, success)?;
                    rows.push(serde_json::json!({"record":id,"admission_record":admission,"commitment":commitment,"returned":returned,"receipt":receipt,"forecast":forecast,"observed_success":success,"residual":forecast.raw.get()-if success {1.0}else{0.0}}));
                }
                _ => {}
            }
        }
        Ok(
            serde_json::json!({"version":"project-evaluator-forecast@v1","records":rows,"limits":"Evaluator-authored diagnostic only; controller prediction and selection unchanged; no learned calibration or execution authority"}),
        )
    }
    pub(crate) fn project_resolve_evaluator(
        &self,
        pending: &Pending,
        returned: EventId,
        success: bool,
    ) -> Result<Vec<Surprise>> {
        let records = self.records()?;
        let mut result = vec![];
        for (id, r) in &records {
            let Record::Coding {
                record:
                    CodingRecord::EvaluatorForecast {
                        admission,
                        forecast,
                    },
            } = r
            else {
                continue;
            };
            if admission.commitment != pending.commitment {
                continue;
            }
            self.validate_evaluator_record(*id, admission, forecast)?;
            ensure!(
                admission.action == pending.action,
                "evaluator return action mismatch"
            );
            ensure!(!records.iter().any(|(_, r)| matches!(r, Record::Coding {record:CodingRecord::EvaluatorResolution { admission:a,.. }} if a == id)), "evaluator return already resolved");
            let Some(receipt) = records.iter().rev().find_map(|(id, r)| match r {
                Record::Coding {
                    record:
                        CodingRecord::Receipt {
                            commitment,
                            verdict,
                            ..
                        },
                } if *commitment == pending.commitment
                    && matches!(verdict, Verdict::Passed | Verdict::FailedTests) =>
                {
                    Some(*id)
                }
                _ => None,
            }) else {
                continue;
            };
            self.validate_evaluator_return(admission, returned, receipt, success)?;
            self.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::EvaluatorResolution {
                        admission: *id,
                        commitment: pending.commitment,
                        returned,
                        receipt,
                        forecast: forecast.clone(),
                        success,
                    },
                },
                Source::Rules,
            )?;
            result.push(Surprise {
                forecast: forecast.id,
                residual: forecast.raw.get() - if success { 1.0 } else { 0.0 },
            });
        }
        Ok(result)
    }
}
