//! Local operator attention projection, not an action/admission or a new goal.
use super::super::super::shared_decision::{criterion_observation, source_bound_result};
use super::*;
use tsr_ledger::selection::{select, SelectionContext};
impl Engine {
    pub fn project_attention(&self) -> Result<serde_json::Value> {
        let root = self.store.head_root()?;
        let side_root = || -> Result<Option<Hash>> {
            if self.directory.join("shadow").exists() {
                Ok(Some(self.shadow_store()?.store.head_root()?))
            } else {
                Ok(None)
            }
        };
        let resource_root = side_root()?;
        let state = self.adaptive_state_inner()?;
        let completion = self.project_acceptance_status()?;
        let records = self.records()?;
        let aims = self.acceptance_current_aims()?;
        let semantic = self.acceptance_kernel_inputs(None)?;
        let latest = source_bound_result(&records, completion.snapshot);
        let admitted_criteria = records
            .iter()
            .find_map(|(_, r)| match r {
                Record::Config { spec } => Some(spec.criteria.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let mut mapped = BTreeMap::new();
        for (_, record) in &records {
            if let Record::Want {
                preference,
                criterion,
                ..
            } = record
            {
                if admitted_criteria.contains(criterion) || state.criteria.contains(criterion) {
                    mapped.insert(*preference, criterion.clone());
                }
            }
        }
        let mut observations = BTreeMap::new();
        for (id, name) in &mapped {
            observations.insert(
                *id,
                latest.and_then(|(required, result)| criterion_observation(required, result, name)),
            );
        }
        for (o, _, _) in &semantic {
            observations.insert(o.id, o.observed);
        }
        let mut blocked = BTreeMap::new();
        for (receipt, r) in &records {
            if let Record::Coding {
                record:
                    CodingRecord::Receipt {
                        commitment,
                        verdict,
                        source_after,
                        ..
                    },
            } = r
            {
                if let Some(action) = records.iter().find_map(|(_, r)| match r {
                    Record::Pending {
                        commitment: c,
                        action,
                        ..
                    } if c == commitment => Some(action),
                    _ => None,
                }) {
                    if *source_after == Some(completion.snapshot)
                        && matches!(verdict, Verdict::FailedTests)
                    {
                        blocked.insert(action.clone(), *receipt);
                    } else {
                        blocked.remove(action);
                    }
                }
            }
        }
        let mut ledger = Ledger::new(&self.store).collective()?;
        ledger.entries = state.preferences.clone(); // includes source-bound reopening, never revives withdrawal
        let objective_parents = self.project_objective_active_parents()?;
        let mut rows = vec![];
        for entry in &mut ledger.entries {
            let id = entry.preference.id;
            let known = mapped.contains_key(&id) || aims.values().any(|p| *p == id);
            let scope = matches!(entry.preference.scope,Scope::Task(t) if t==Self::task());
            let observed = observations.get(&id).copied().flatten();
            let objective_pending = objective_parents.contains(&id);
            let exclusion = if !entry.is_open() && !objective_pending {
                Some("not_open_or_not_admitted")
            } else if !scope || !known {
                Some("outside_current_admitted_goal_contract")
            } else if observed == Some(1.0) && !objective_pending {
                Some("currently_observed_satisfied")
            } else {
                None
            };
            let options = state
                .options
                .iter()
                .filter(|o| {
                    mapped.get(&id).is_some_and(|name| o.serves.contains(name))
                        && !blocked.contains_key(&o.action)
                })
                .collect::<Vec<_>>();
            rows.push(serde_json::json!({"preference":id,"parents":entry.preference.parents,"status":entry.status,"observed":observed,"exclusion":exclusion,"registered_routes_requiring_fresh_validation":options,"semantic_support":semantic.iter().filter(|(o,_,_)|o.id==id).map(|(_,p,e)|serde_json::json!({"probability_proxy":p,"evidence":e,"calibration":"unvalidated reviewed-outcome fold; not confidence or permission"})).collect::<Vec<_>>()}));
            // Keep eligible unfinished obligations open in this transient selection view.
            // None means inactive to the shared selector; history and permission remain unchanged.
            if objective_pending && exclusion.is_none() {
                entry.status = Some(PreferenceStatus::Open);
            }
            if exclusion.is_some() {
                entry.status = Some(PreferenceStatus::OutOfScope);
            }
        }
        let snapshot = Snapshot {
            task: Self::task(),
            root,
            version: self.events()?.len() as u64,
            open_criteria: vec![],
            missing_evidence: vec![],
        };
        let tune = tsr_domain::Tune {
            intuition_max: 1,
            quiet_route: 0.0,
            ..Default::default()
        };
        let context = SelectionContext {
            family: "admitted-project".into(),
            repository: "current-scope-only".into(),
            minimum_depth: 1,
        };
        let selection =
            select(&ledger, &snapshot, &tune, &context, 0).map_err(anyhow::Error::msg)?;
        let admission = self.project_resource_admission()?;
        let resources = self.project_budget()?;
        let capacity = serde_json::json!({"model_calls_available":resources.limits.as_ref().map(|l|resources.unlimited_mode || resources.model_calls<l.model_calls),"model_tokens_available":resources.limits.as_ref().map(|l|resources.unlimited_mode || resources.tokens_charged<l.model_tokens),"tool_time_available":admission.as_ref().map(|(_,a,_,_)|resources.unlimited_mode || resources.tool_ms_charged<a.tool_ms_ceiling),"meaning":"Resource-specific capacity only; zero model allowance does not block admitted verification. Fresh action gate still required."});
        let closed = admission.as_ref().is_some_and(|(_, _, _, closed)| *closed);
        let mut blockers = vec![];
        if state.pending.is_some() {
            blockers.push("pending_commitment");
        }
        if closed {
            blockers.push("closed_episode_requires_fresh_admission");
        }
        if !resources.active {
            blockers.push("prospective_resource_admission_required");
        }
        if resources.exceeded || resources.stopped {
            blockers.push("resource_budget_exhausted_or_stopped");
        }
        if resources.unsettled_requests > 0 || resources.unresolved_tool_requests > 0 {
            blockers.push("unresolved_resource_reservations");
        }
        if completion.accepted && objective_parents.is_empty() {
            blockers.push("accepted_task_requires_fresh_goal_and_action_admission");
        }
        let selected = if state.pending.is_some() {
            None
        } else {
            selection.selected.first().copied()
        };
        ensure!(
            root == self.store.head_root()?
                && resource_root == side_root()?
                && completion.snapshot
                    == digest(&self.project_snapshot(&self.project_configuration()?)?)?,
            "attention inputs changed; read again"
        );
        Ok(
            serde_json::json!({"version":"admitted-next-attention@v1","state_root":root,"resource_root":resource_root,"source_snapshot":completion.snapshot,"task_accepted":completion.accepted,"episode_closed":closed,"selected_preference":selected,"selection":selection,"desires":rows,"blocked_exact_actions":blocked,"continuation_blockers":blockers,"resources":resources,"resource_capacity":capacity,"dispatch_authorized":false,"next_step":if selected.is_none(){"no_eligible_next_attention"}else if !blockers.is_empty(){"review_attention_and_obtain_fresh_admission"}else{"fresh_proposal_and_independent_action_gate_required"},"acceptance_context":state.acceptance_cycle,"conflict_status":"no new conflict inferred; existing reviewed evidence and standing wants retained","meaning":"Read-only attention among existing admitted residual desires, using existing lineage selector. Accepted task is not overall desire satisfaction. No inference, execution, new goal or permission. Unknown observations remain unknown. Failed exact actions do not prohibit new repairs; future commitments must freeze their own current evidence."}),
        )
    }
}

impl Engine {
    /// Explicit provider allowlist. Never serialize the operator attention report.
    pub(crate) fn proposal_attention(&self) -> Result<Option<serde_json::Value>> {
        if self.coding_config()?.and_then(|c| c.project).is_none() {
            return Ok(None);
        }
        let report = self.project_attention()?;
        let private = self.acceptance_private_aims()?;
        let selected: Option<PreferenceId> =
            serde_json::from_value(report["selected_preference"].clone())?;
        let scope = Hash::of(&[&serde_json::to_vec(&self.project_configuration()?)?]);
        // Resource usage changes during the very request being answered. Bind immutable
        // admission, not its mutable accounting head; recheck current capacity at apply.
        let admission = self.project_resource_admission()?;
        let binding = Hash::of(&[&serde_json::to_vec(&(
            "proposal-attention@v1",
            &report["state_root"],
            &report["source_snapshot"],
            scope,
            selected,
            &admission,
        ))?]);
        let focus = if let Some(id) = selected {
            if private.contains(&id) {
                let aims = self.acceptance_current_aims()?;
                let guidance = report["acceptance_context"]["guidance"].as_object();
                let key = aims.iter().find(|(_, p)| **p == id).map(|(key, _)| key);
                if let Some(key) = key.filter(|key| guidance.is_some_and(|g| g.contains_key(*key)))
                {
                    serde_json::json!({"kind":"published_requirement","key":key})
                } else {
                    serde_json::json!({"kind":"operator_guidance_required"})
                }
            } else {
                let criterion = self.records()?.into_iter().find_map(|(_, r)| match r {
                    Record::Want {
                        preference,
                        criterion,
                        ..
                    } if preference == id => Some(criterion),
                    _ => None,
                });
                let text = Ledger::new(&self.store)
                    .collective()?
                    .entries
                    .into_iter()
                    .find(|p| p.preference.id == id)
                    .map(|p| p.preference.text);
                serde_json::json!({"kind":"admitted_preference","preference":id,"criterion":criterion,"text":text})
            }
        } else {
            serde_json::json!({"kind":"no_eligible_attention"})
        };
        Ok(Some(
            serde_json::json!({"version":"proposal-attention@v1","binding":binding,"state_root":report["state_root"],"source_snapshot":report["source_snapshot"],"scope":scope,"focus":focus,"dispatch_authorized":false}),
        ))
    }
    pub(crate) fn proposal_attention_binding(&self) -> Result<Option<Hash>> {
        self.proposal_attention()?
            .map(|v| serde_json::from_value(v["binding"].clone()).map_err(Into::into))
            .transpose()
    }
    pub(crate) fn validate_proposal_attention(&self, supplied: Option<Hash>) -> Result<()> {
        let expected = self.proposal_attention_binding()?;
        ensure!(
            supplied == expected,
            "missing, stale or forged proposal attention binding; request again"
        );
        if expected.is_none() {
            return Ok(());
        }
        let report = self.project_attention()?;
        ensure!(
            !report["selected_preference"].is_null(),
            "no eligible residual attention or pending commitment"
        );
        ensure!(
            report["episode_closed"] != true,
            "closed episode requires fresh admission"
        );
        let budget = self.project_budget()?;
        ensure!(
            !budget.stopped
                && !budget.exceeded
                && budget.unsettled_requests == 0
                && budget.unresolved_tool_requests == 0,
            "blocked resource authority; reconcile before proposal application"
        );
        if let Some((_, admission, _, _)) = self.project_resource_admission()? {
            ensure!(
                budget.unlimited_mode || budget.tool_ms_charged < admission.tool_ms_ceiling,
                "tool allowance exhausted"
            );
        }
        Ok(())
    }
}
