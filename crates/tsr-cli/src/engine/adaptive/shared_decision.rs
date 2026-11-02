//! Existing project execution/acceptance adapter into the shared domain kernel.
//! This is a planning fold; accepted export still requires independently bound review.
use super::*;
use tsr_domain::decision as kernel;
pub(crate) fn criterion_observation(
    required: &[String],
    result: &ResultInput,
    name: &str,
) -> Option<f64> {
    required.iter().any(|c| c == name).then(|| {
        if result.met.iter().any(|c| c == name) {
            1.0
        } else {
            0.0
        }
    })
}
pub(crate) fn source_bound_result(
    records: &[(EventId, Record)],
    snapshot: Hash,
) -> Option<(&[String], &ResultInput)> {
    let receipt = records.iter().rev().find_map(|(_, r)| match r {
        Record::Coding {
            record:
                coding::CodingRecord::Receipt {
                    commitment,
                    source_after,
                    verdict,
                    ..
                },
        } => Some((*commitment, *source_after, verdict)),
        _ => None,
    });
    receipt
        .filter(|(_, source, verdict)| {
            *source == Some(snapshot)
                && matches!(
                    verdict,
                    coding::Verdict::Passed | coding::Verdict::FailedTests
                )
        })
        .and_then(|(expected, _, _)| {
            let required = records.iter().find_map(|(_, r)| match r {
                Record::Pending {
                    commitment,
                    criteria,
                    ..
                } if *commitment == expected => Some(criteria.as_slice()),
                _ => None,
            })?;
            records.iter().rev().find_map(|(_, r)| match r {
                Record::Returned {
                    commitment, input, ..
                } if *commitment == expected => Some((required, input)),
                _ => None,
            })
        })
}
impl Engine {
    /// Recompute the declared empirical-success intervention without changing
    /// the previous-commitment lookup universe or independent semantic evidence.
    pub(super) fn success_prior_options(
        &self,
        state: &State,
        offered: &[String],
    ) -> Result<Vec<OptionView>> {
        let records = self.records()?;
        let spec = records
            .iter()
            .find_map(|(_, r)| {
                if let Record::Config { spec } = r {
                    Some(spec)
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow::anyhow!("missing config"))?;
        let actions = self.effective_actions(spec, &records)?;
        let mut control = state.options.clone();
        for option in &mut control {
            option.prediction = Proxy::new("intervention_prior_success", 0.5);
            let cost = actions
                .iter()
                .find(|a| a.id == option.action)
                .ok_or_else(|| anyhow::anyhow!("missing action"))?
                .cost;
            option.score = Proxy::new("intervention_value", 0.5 * option.value as f64 - cost);
        }
        if state.goal_completion.is_some() {
            self.project_rank_options(
                spec,
                &records,
                &state.criteria,
                &mut control,
                state.last_commitment.as_deref(),
            )?;
        }
        control.retain(|o| offered.contains(&o.action));
        Ok(control)
    }
    pub(super) fn project_rank_options(
        &self,
        spec: &Spec,
        records: &[(EventId, Record)],
        criteria: &[String],
        options: &mut [OptionView],
        last: Option<&str>,
    ) -> Result<()> {
        let completion = self.project_acceptance_status()?;
        let semantic = self.acceptance_kernel_inputs(None)?;
        let admitted = Ledger::new(&self.store)
            .collective()?
            .entries
            .into_iter()
            .filter(|entry| {
                matches!(
                    entry.status,
                    Some(PreferenceStatus::Open | PreferenceStatus::Fulfilled)
                ) && (entry.preference.origin == Source::Operator
                    || entry.history.iter().any(|(status, by)| {
                        *status == PreferenceStatus::Open && *by == Source::Operator
                    }))
            })
            .map(|entry| entry.preference.id)
            .collect::<BTreeSet<_>>();
        let ids: BTreeMap<_, _> = records
            .iter()
            .filter_map(|(_, r)| match r {
                Record::Want {
                    preference,
                    criterion,
                    ..
                } if admitted.contains(preference)
                    && (spec.criteria.contains(criterion) || criteria.contains(criterion)) =>
                {
                    Some((criterion.clone(), *preference))
                }
                _ => None,
            })
            .collect();
        let latest = source_bound_result(records, completion.snapshot);
        let mut obligations = ids
            .iter()
            .map(|(name, id)| kernel::Obligation {
                id: *id,
                refines: None,
                admitted: true,
                observed: latest
                    .and_then(|(required, result)| criterion_observation(required, result, name)),
            })
            .collect::<Vec<_>>();
        obligations.extend(semantic.iter().map(|(o, _, _)| o.clone()));
        if obligations.is_empty() {
            return Ok(());
        }
        let selected = obligations
            .iter()
            .filter(|o| o.observed != Some(1.0))
            .map(|o| o.id)
            .collect::<Vec<_>>();
        if selected.is_empty() {
            return Ok(());
        }
        let commitment = |option: &OptionView| kernel::Commitment {
            // Existing coding family is a bounded patch; supports distinguish its intended effects.
            family: "scoped-patch".into(),
            supports: option
                .serves
                .iter()
                .filter_map(|name| ids.get(name).copied())
                .chain(semantic.iter().map(|(o, _, _)| o.id))
                .collect(),
        };
        // Continuity is provisional action support, never semantic acceptance. A
        // current productive receipt may support it before review; rejection or
        // unresolved/stale review releases it without erasing learned experience.
        let returned = records.iter().rev().find_map(|(_, r)| match r {
            Record::Returned {
                commitment,
                success,
                ..
            } => Some((*commitment, *success)),
            _ => None,
        });
        let receipt = records.iter().rev().find_map(|(_, r)| match r {
            Record::Coding {
                record:
                    coding::CodingRecord::Receipt {
                        commitment,
                        source_after,
                        verdict,
                        ..
                    },
            } => Some((*commitment, *source_after, verdict)),
            _ => None,
        });
        let supported = latest.is_some()
            && receipt
                .zip(returned)
                .is_some_and(|((expected, _, _), (actual, success))| expected == actual && success)
            && matches!(completion.status.as_str(), "review_required" | "accepted");
        let previous = last
            .and_then(|id| options.iter().find(|o| o.action == id))
            .filter(|_| supported)
            .map(commitment);
        let actions = self.effective_actions(spec, records)?;
        for option in options {
            option.commitment_supported = Some(previous.is_some());
            let semantic = self.acceptance_kernel_inputs(Some(&option.action))?;
            let Some(action) = actions.iter().find(|a| a.id == option.action) else {
                continue;
            };
            let mut projections = ids
                .iter()
                .map(|(name, id)| kernel::Projection {
                    obligation: *id,
                    anticipated: if option.serves.contains(name) {
                        1.0
                    } else {
                        obligations
                            .iter()
                            .find(|o| o.id == *id)
                            .and_then(|o| o.observed)
                            .unwrap_or(0.0)
                    },
                    lower: 0.0,
                    upper: 1.0,
                })
                .collect::<Vec<_>>();
            projections.extend(semantic.iter().map(|(o, p, _)| kernel::Projection {
                obligation: o.id,
                anticipated: *p,
                lower: 0.0,
                upper: 1.0,
            }));
            let proposal = commitment(option);
            if !proposal.supports.iter().any(|id| selected.contains(id)) {
                // A residual desire can outlive all registered routes. Preserve it,
                // but do not crash the state fold or pretend an old route serves it.
                option.value = 0;
                option.score = Proxy::new("no_residual_support", 0.0);
                option.decision = None;
                continue;
            }

            // Configuration admits only local proposal scope. Actual patch validation,
            // execution authorization and export acceptance remain separate existing gates.
            let resource_cost = self.project_cost_estimate(&option.action)?;
            let effort = resource_cost
                .as_ref()
                .map(|c| c.normalized_cost)
                .unwrap_or(action.cost);
            if let Some(cost) = &resource_cost {
                option.evidence.extend(cost.samples.iter().copied());
            }
            option.resource_cost = resource_cost;
            let ranked = kernel::assess(kernel::Request {
                obligations: &obligations,
                selected: &selected,
                proposal: &proposal,
                previous: previous.as_ref(),
                projections: &projections,
                probability: option.prediction.value,
                effort,
                authorized: true,
            })
            .map_err(anyhow::Error::msg)?;
            option.score = Proxy::new(
                if option.resource_cost.is_some() {
                    "budget_normalized_obligation_value@v1"
                } else {
                    "shared_obligation_value"
                },
                ranked.score,
            );
            option
                .evidence
                .extend(semantic.iter().flat_map(|(_, _, e)| e.iter().copied()));
            option.evidence.sort();
            option.evidence.dedup();
            option.decision = Some(ranked);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn later_criterion_is_unknown_not_a_failure_of_an_earlier_execution() {
        let result = ResultInput {
            met: vec!["passed".into()],
            evidence: "independent receipt".into(),
            proposed_refinement: None,
        };
        let required = vec!["passed".into(), "failed".into()];
        assert_eq!(
            criterion_observation(&required, &result, "passed"),
            Some(1.0)
        );
        assert_eq!(
            criterion_observation(&required, &result, "failed"),
            Some(0.0)
        );
        assert_eq!(criterion_observation(&required, &result, "new"), None);
    }
}
