//! Pre-action resource estimates, never reservations or measured outcomes.
use super::*;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Estimate {
    pub version: String,
    pub status: String,
    pub expected_tool_ms: f64,
    pub future_model_tokens: u64,
    pub tool_budget_ms: u64,
    pub token_budget: u64,
    pub normalized_cost: f64,
    pub samples: Vec<EventId>,
    pub compatible_incomplete: usize,
    pub sunk_tokens_charged: u64,
    pub sunk_tool_ms_charged: u64,
    pub source_prefix: Hash,
    pub interpretation: String,
}
fn normalized(tool: f64, tokens: u64, tool_budget: u64, token_budget: u64) -> Result<f64> {
    ensure!(
        tool.is_finite() && tool >= 0.0 && tool_budget > 0 && (tokens == 0 || token_budget > 0),
        "invalid resource normalization"
    );
    Ok(tool / tool_budget as f64
        + if tokens == 0 {
            0.0
        } else {
            tokens as f64 / token_budget as f64
        })
}
impl Engine {
    pub(crate) fn project_cost_estimate(&self, action: &str) -> Result<Option<Estimate>> {
        // Disabled aggregate ceilings cannot supply meaningful normalization units.
        // None retains the existing declared-action effort fallback, not zero/free cost.
        if self.project_budget()?.unlimited_mode {
            return Ok(None);
        }
        let Some((_, admission, _, _)) = self.project_resource_admission()? else {
            return Ok(None);
        };
        let Some(limits) = admission.non_tool else {
            return Ok(None);
        };
        let patches = match self.patches_for(action)? {
            Some(patches) => patches,
            None if self.project_verification_admitted(action)?.is_some() => vec![],
            None => return Ok(None),
        };
        let spec = self.project_configuration()?;
        let paths = patches
            .iter()
            .map(|p| p.path.clone())
            .collect::<BTreeSet<_>>();
        let protected = Hash::of(&[&serde_json::to_vec(&spec.protected)?]);
        let records = self.records()?;
        let events = self.events()?;
        let mut samples = vec![];
        let mut durations = vec![];
        let mut incomplete = 0;
        for (_, record) in &records {
            let Record::Coding {
                record: CodingRecord::Intent { plan },
            } = record
            else {
                continue;
            };
            if plan.image != spec.image
                || plan.user != spec.user
                || plan.timeout_ms != spec.timeout_ms
                || plan.protected_tests != protected
                || plan.commands.len() != execution_command_count(&spec)
                || serde_json::to_value(&plan.cases)? != serde_json::to_value(&spec.cases)?
                || plan
                    .edits
                    .iter()
                    .map(|e| e.path.clone())
                    .collect::<BTreeSet<_>>()
                    != paths
            {
                continue;
            }
            let receipt = records.iter().find_map(|(id, r)| match r {
                Record::Coding {
                    record:
                        CodingRecord::Receipt {
                            commitment,
                            verdict,
                            runs,
                            ..
                        },
                } if *commitment == plan.commitment => Some((*id, verdict, runs.len())),
                _ => None,
            });
            let candidate = events.iter().find_map(|(_, e)| match e.kind {
                EventKind::Committed { candidate, .. } if e.id == plan.commitment => {
                    Some(candidate)
                }
                _ => None,
            });
            let measured = events.iter().find_map(|(_, e)| match e.kind {
                EventKind::Executed {
                    candidate: c,
                    duration_ms,
                    ..
                } if Some(c) == candidate && e.source == Source::Rules => Some(duration_ms),
                _ => None,
            });
            match (receipt, measured) {
                (Some((id, Verdict::Passed | Verdict::FailedTests, n)), Some(ms))
                    if n == plan.commands.len() =>
                {
                    samples.push(id);
                    durations.push(ms as f64);
                }
                _ => incomplete += 1,
            }
        }
        let needs_preparation = !self.dependencies_ready(&spec)?;
        let upper = spec
            .timeout_ms
            .checked_mul((execution_command_count(&spec) + usize::from(needs_preparation)) as u64)
            .ok_or_else(|| anyhow::anyhow!("cost bound overflow"))?;
        let supported = !durations.is_empty() && incomplete == 0 && !needs_preparation;
        let expected = if supported {
            durations.iter().sum::<f64>() / durations.len() as f64
        } else {
            upper as f64
        };
        let spent = self.project_budget()?;
        Ok(Some(Estimate{version:"budget-normalized-action-cost@v1".into(),status:if supported {"empirical_complete_path_set"}else{"reservation_bound_fallback"}.into(),expected_tool_ms:expected,future_model_tokens:0,
            tool_budget_ms:admission.tool_ms_ceiling,token_budget:limits.model_tokens,
            normalized_cost:normalized(expected,0,admission.tool_ms_ceiling,limits.model_tokens)?,samples,compatible_incomplete:incomplete,
            sunk_tokens_charged:spent.tokens_charged,sunk_tool_ms_charged:spent.tool_ms_charged,source_prefix:Hash::of(&[&serde_json::to_vec(&events)?]),
            interpretation:"Expected future scoped execution and verification only; this action calls no model. Already generated proposals and prepared dependencies are sunk, retained in cumulative budgets, not charged per alternative. Local inference is not free. Complete same-path-set/scope histories estimate tool time, not calibrated cost; incomplete history uses reservation bounds. Units: ms/tool ceiling + tokens/token ceiling; no learned utility scale.".into()}))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_budget_fractions_preserve_units_and_unknowns() {
        assert_eq!(normalized(30000.0, 500, 60000, 1000).unwrap(), 1.0);
        assert_eq!(normalized(30000.0, 0, 60000, 1000).unwrap(), 0.5);
        assert!(normalized(f64::NAN, 0, 60000, 1000).is_err());
        assert!(normalized(1.0, 0, 0, 1000).is_err());
        // Same desire/probability: measured cost alone reverses ranking; replay unchanged.
        let score = |a, b| {
            if 0.8 - normalized(a, 0, 1000, 1000).unwrap()
                > 0.8 - normalized(b, 0, 1000, 1000).unwrap()
            {
                "a"
            } else {
                "b"
            }
        };
        assert_eq!(score(100.0, 500.0), "a");
        assert_eq!(score(500.0, 100.0), "b");
        assert_eq!(score(100.0, 500.0), "a");
    }
}
