//! Operator-configured, observational domain. Suggestions never execute external effects.
use super::*;
mod shared_decision;
use std::collections::{BTreeMap, BTreeSet};
const SOURCE: &str = "configured-cycle@v1";
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub id: String,
    pub description: String,
    pub serves: Vec<String>,
    pub cost: f64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prior {
    pub action: String,
    pub success: bool,
    pub evidence: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    pub goal: String,
    pub criteria: Vec<String>,
    pub actions: Vec<Action>,
    #[serde(default)]
    pub prior: Vec<Prior>,
    #[serde(default)]
    pub initial_commitment: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Refinement {
    pub criterion: String,
    pub text: String,
    pub actions: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultInput {
    pub met: Vec<String>,
    pub evidence: String,
    #[serde(default)]
    pub proposed_refinement: Option<Refinement>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "fact", rename_all = "snake_case")]
pub(super) enum Record {
    Coding {
        record: coding::CodingRecord,
    },
    Censored {
        commitment: EventId,
        reason: String,
    },
    AttentionConsumed {
        attention: serde_json::Value,
    },
    ProposalReceived {
        response: provider::Response,
    },
    Config {
        spec: Spec,
    },
    Pending {
        action: String,
        commitment: EventId,
        forecast: Forecast,
        criteria: Vec<String>,
    },
    Returned {
        commitment: EventId,
        action: String,
        success: bool,
        input: ResultInput,
    },
    Want {
        preference: PreferenceId,
        criterion: String,
        actions: Vec<String>,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pending {
    pub action: String,
    pub commitment: EventId,
    pub forecast: Forecast,
    pub criteria: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionView {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resource_cost: Option<coding::project::cost::Estimate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commitment_supported: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decision: Option<tsr_domain::decision::Assessment>,
    pub action: String,
    pub prediction: Proxy<f64>,
    pub value: usize,
    pub score: Proxy<f64>,
    pub evidence: Vec<EventId>,
    pub serves: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance_cycle: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal_completion: Option<coding::project::acceptance::Completion>,
    pub goal: String,
    pub criteria: Vec<String>,
    pub completed: usize,
    pub last_commitment: Option<String>,
    pub pending: Option<Pending>,
    pub options: Vec<OptionView>,
    pub preferences: Vec<tsr_ledger::LedgerEntry>,
    pub unmet_pressure: Proxy<f64>,
}
/// Adapter defines outcome semantics; providers only offer configured action IDs.
pub trait DomainAdapter {
    fn success(&self, required: &[String], observed: &ResultInput) -> bool;
}
pub struct CriteriaAdapter;
impl DomainAdapter for CriteriaAdapter {
    fn success(&self, required: &[String], observed: &ResultInput) -> bool {
        required.iter().all(|c| observed.met.contains(c))
    }
}
pub trait ProposalProvider {
    fn common_event_forecast(&self, _: &State, _: &str) -> Option<f64> {
        None
    }
    fn propose(&self, state: &State) -> Vec<String>;
}
pub struct DeterministicProvider;
impl ProposalProvider for DeterministicProvider {
    fn propose(&self, state: &State) -> Vec<String> {
        let mut options = state.options.clone();
        options.sort_by(|a, b| {
            b.score
                .value
                .total_cmp(&a.score.value)
                .then_with(|| {
                    let previous = state.last_commitment.as_deref();
                    let a_previous = previous == Some(a.action.as_str());
                    let b_previous = previous == Some(b.action.as_str());
                    if state.unmet_pressure.value > 0.0 {
                        a_previous.cmp(&b_previous)
                    } else {
                        b_previous.cmp(&a_previous)
                    }
                })
                .then_with(|| a.action.cmp(&b.action))
        });
        options.into_iter().map(|o| o.action).collect()
    }
}
fn short(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 512
}
impl Engine {
    fn adaptive_record(&self, record: &Record, source: Source) -> Result<EventId> {
        let digest = self.blob(&serde_json::to_vec(record)?)?;
        self.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    digest,
                    source: SOURCE.into(),
                },
            },
            source,
        )
    }
    fn records(&self) -> Result<Vec<(EventId, Record)>> {
        let mut records = vec![];
        let mut previous = None;
        for (_, e) in self.events()? {
            ensure!(
                e.schema_version == Event::SCHEMA_VERSION && e.task == Self::task(),
                "schema/task mismatch"
            );
            ensure!(
                e.parents == previous.into_iter().collect::<Vec<_>>(),
                "broken event chain"
            );
            previous = Some(e.id);
            ensure!(
                Event::new(
                    e.task,
                    e.parents.clone(),
                    e.source,
                    e.kind.clone(),
                    e.recorded_at.clone()
                )
                .id == e.id,
                "event identity mismatch"
            );
            if let EventKind::Observed {
                observation: Observation::Evidence { digest, source },
            } = &e.kind
            {
                if source == SOURCE {
                    let bytes = fs::read(self.directory.join("cas").join(digest.to_hex()))?;
                    ensure!(Hash::of(&[&bytes]) == *digest, "changed evidence");
                    // Cache only deserialization, never evidence/authority checks.
                    // Fresh bytes above must still match their content identity.
                    let cached = self.record_cache.borrow().get(digest).cloned();
                    let record = match cached {
                        Some(record) => record,
                        None => {
                            let record: Record = serde_json::from_slice(&bytes)?;
                            self.record_cache
                                .borrow_mut()
                                .insert(*digest, record.clone());
                            record
                        }
                    };
                    records.push((e.id, record));
                }
            }
        }
        Ok(records)
    }
    pub fn configure(&self, spec: Spec) -> Result<State> {
        ensure!(
            short(&spec.goal) && !spec.criteria.is_empty() && spec.criteria.len() <= 16,
            "goal/criteria bounds"
        );
        let criteria: BTreeSet<_> = spec.criteria.iter().collect();
        ensure!(
            criteria.len() == spec.criteria.len() && spec.criteria.iter().all(|s| short(s)),
            "unique short criteria required"
        );
        ensure!(
            !spec.actions.is_empty() && spec.actions.len() <= 16,
            "1..16 actions required"
        );
        let ids: BTreeSet<_> = spec.actions.iter().map(|a| a.id.as_str()).collect();
        ensure!(ids.len() == spec.actions.len(), "duplicate action");
        for a in &spec.actions {
            ensure!(
                short(&a.id)
                    && short(&a.description)
                    && a.cost.is_finite()
                    && (0.0..=1.0).contains(&a.cost)
                    && !a.serves.is_empty()
                    && a.serves.iter().all(|c| criteria.contains(c)),
                "invalid action"
            );
        }
        ensure!(
            spec.prior.len() <= 100
                && spec
                    .prior
                    .iter()
                    .all(|p| ids.contains(p.action.as_str()) && short(&p.evidence)),
            "invalid prior evidence"
        );
        ensure!(
            spec.initial_commitment
                .as_ref()
                .is_none_or(|a| ids.contains(a.as_str())),
            "unknown initial commitment"
        );
        self.write(|| {
            ensure!(self.events()?.is_empty(), "directory already initialized");
            self.append(
                EventKind::TaskOpened { task: Self::task() },
                Source::Operator,
            )?;
            let evidence =
                self.adaptive_record(&Record::Config { spec: spec.clone() }, Source::Operator)?;
            for c in &spec.criteria {
                self.adaptive_want(
                    c,
                    c,
                    spec.actions
                        .iter()
                        .filter(|a| a.serves.contains(c))
                        .map(|a| a.id.clone())
                        .collect(),
                    evidence,
                    Source::Operator,
                )?;
            }
            self.adaptive_state_inner()
        })
    }
    fn adaptive_want(
        &self,
        criterion: &str,
        text: &str,
        actions: Vec<String>,
        evidence: EventId,
        origin: Source,
    ) -> Result<()> {
        let (contrast, parents) = if origin == Source::Generator {
            let contrast = self
                .events()?
                .into_iter()
                .rev()
                .find_map(|(_, e)| match e.kind {
                    EventKind::Observed {
                        observation:
                            Observation::Return {
                                contrast: Some(c), ..
                            },
                    } if c.from_return == evidence && !c.unmet.is_empty() => Some(c),
                    _ => None,
                })
                .ok_or_else(|| anyhow::anyhow!("refinement needs recorded contrast"))?;
            let parents = Ledger::new(&self.store)
                .collective()?
                .entries
                .iter()
                .filter(|p| p.is_open())
                .map(|p| p.preference.id)
                .collect();
            (contrast, parents)
        } else {
            let contrast = Filter::contrast_from_criteria(
                evidence,
                vec![(
                    CriterionId(Hash::of(&[criterion.as_bytes()])),
                    UnmetKind::WrongBehaviour,
                )],
                vec![evidence],
            );
            self.append(
                EventKind::Observed {
                    observation: Observation::Return {
                        surprise: vec![],
                        contrast: Some(contrast.clone()),
                    },
                },
                origin,
            )?;
            (contrast, vec![])
        };
        self.expand(&contrast, text, parents.clone(), origin)?;
        let preference = Preference::new(text, Scope::Task(Self::task()), parents, origin)?.id;
        self.adaptive_record(
            &Record::Want {
                preference,
                criterion: criterion.into(),
                actions,
            },
            origin,
        )?;
        Ok(())
    }
    pub fn adaptive_state(&self) -> Result<State> {
        let own = self.store.connection().is_autocommit();
        if own {
            self.store.connection().execute_batch("BEGIN")?;
        }
        let result = self.adaptive_state_inner();
        if own {
            self.store.connection().execute_batch("ROLLBACK")?;
        }
        result
    }
    fn effective_actions(&self, spec: &Spec, records: &[(EventId, Record)]) -> Result<Vec<Action>> {
        if self.coding_config()?.is_some_and(|c| c.generated) {
            for (_, record) in records {
                if let Record::Coding {
                    record: coding::CodingRecord::Verification { action, .. },
                } = record
                {
                    self.project_verification_admitted(&action.id)?;
                }
            }
            Ok(records
                .iter()
                .filter_map(|(_, r)| match r {
                    Record::Coding {
                        record:
                            coding::CodingRecord::Patch { action, .. }
                            | coding::CodingRecord::PatchSet { action, .. }
                            | coding::CodingRecord::Verification { action, .. },
                    } => Some(action.clone()),
                    _ => None,
                })
                .collect())
        } else {
            Ok(spec.actions.clone())
        }
    }
    fn adaptive_state_inner(&self) -> Result<State> {
        let records = self.records()?;
        let (config_id, spec) = records
            .iter()
            .find_map(|(id, r)| {
                if let Record::Config { spec } = r {
                    Some((*id, spec))
                } else {
                    None
                }
            })
            .ok_or_else(|| anyhow::anyhow!("run configure first"))?;
        let mut preferences = Ledger::new(&self.store).collective()?.entries;
        let open: BTreeSet<_> = preferences
            .iter()
            .filter(|p| p.is_open())
            .map(|p| p.preference.id)
            .collect();
        let criteria: Vec<String> = records
            .iter()
            .filter_map(|(_, r)| match r {
                Record::Want {
                    preference,
                    criterion,
                    ..
                } if open.contains(preference) => Some(criterion.clone()),
                _ => None,
            })
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let original: Vec<String> = spec
            .criteria
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect();
        let actions = self.effective_actions(spec, &records)?;
        let mut support: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let generated = self.coding_config()?.is_some_and(|c| c.generated);
        for a in &actions {
            if generated {
                support.insert(
                    a.id.clone(),
                    a.serves
                        .iter()
                        .filter(|c| criteria.contains(c))
                        .cloned()
                        .collect(),
                );
            }
        }
        let mut counts: BTreeMap<String, (u32, u32, Vec<EventId>)> = actions
            .iter()
            .map(|a| {
                let mut evidence = vec![config_id];
                if generated {
                    evidence.extend(records.iter().filter_map(|(id, r)| match r {
                        Record::Coding {
                            record:
                                coding::CodingRecord::Patch { action, .. }
                                | coding::CodingRecord::PatchSet { action, .. }
                                | coding::CodingRecord::Verification { action, .. },
                        } if action.id == a.id => Some(*id),
                        _ => None,
                    }));
                }
                (a.id.clone(), (0, 0, evidence))
            })
            .collect();
        for p in spec.prior.iter().filter(|_| criteria == original) {
            let n = counts.get_mut(&p.action).unwrap();
            if p.success {
                n.0 += 1
            } else {
                n.1 += 1
            }
        }
        let mut pending = None;
        let mut completed = 0;
        let mut failures = 0;
        let mut last = spec.initial_commitment.clone();
        for (id, r) in records.iter() {
            match r {
                Record::Want {
                    preference,
                    criterion,
                    actions,
                } if open.contains(preference) => {
                    for a in actions {
                        support
                            .entry(a.clone())
                            .or_default()
                            .insert(criterion.clone());
                    }
                }
                Record::Pending {
                    action,
                    commitment,
                    forecast,
                    criteria,
                } => {
                    ensure!(pending.is_none(), "unresolved commitment");
                    last = Some(action.clone());
                    pending = Some(Pending {
                        action: action.clone(),
                        commitment: *commitment,
                        forecast: forecast.clone(),
                        criteria: criteria.clone(),
                    });
                }
                Record::Censored { commitment, .. } => {
                    ensure!(
                        pending
                            .as_ref()
                            .is_some_and(|p| p.commitment == *commitment),
                        "unmatched censored return"
                    );
                    pending = None;
                    completed += 1;
                }
                Record::Returned {
                    commitment,
                    action,
                    success,
                    ..
                } => {
                    ensure!(
                        pending
                            .as_ref()
                            .is_some_and(|p| p.commitment == *commitment && p.action == *action),
                        "unmatched return"
                    );
                    let same_criteria = pending.as_ref().is_some_and(|p| p.criteria == criteria);
                    pending = None;
                    completed += 1;
                    if !*success {
                        failures += 1;
                    }
                    if !same_criteria {
                        continue;
                    }
                    let n = counts
                        .get_mut(action)
                        .ok_or_else(|| anyhow::anyhow!("unknown action"))?;
                    if *success {
                        n.0 += 1
                    } else {
                        n.1 += 1
                    }
                    n.2.push(*id);
                }
                _ => {}
            }
        }
        let mut options: Vec<OptionView> = actions
            .iter()
            .map(|a| {
                let (ok, bad, evidence) = &counts[&a.id];
                let p = (1.0 + f64::from(*ok)) / (2.0 + f64::from(ok + bad));
                let serves = support
                    .get(&a.id)
                    .cloned()
                    .unwrap_or_default()
                    .into_iter()
                    .collect::<Vec<_>>();
                let value = serves.len();
                OptionView {
                    resource_cost: None,
                    commitment_supported: None,
                    decision: None,
                    action: a.id.clone(),
                    prediction: Proxy::new("empirical_success", p),
                    score: Proxy::new("ev", p * value as f64 - a.cost),
                    value,
                    evidence: evidence.clone(),
                    serves,
                }
            })
            .collect();
        let goal_completion = if self.coding_config()?.is_some_and(|c| c.project.is_some()) {
            Some(self.project_acceptance_status()?)
        } else {
            None
        };
        // Read-only projection: a fulfilled aim is no longer closed when its source binding is stale.
        // The next explicit repair appends the corresponding reopening event.
        if goal_completion.as_ref().is_some_and(|c| !c.accepted) {
            let aims = self.acceptance_current_aims()?;
            for entry in &mut preferences {
                if aims.values().any(|id| *id == entry.preference.id)
                    && entry.status == Some(PreferenceStatus::Fulfilled)
                {
                    entry.status = Some(PreferenceStatus::Open);
                }
            }
        }
        let acceptance_cycle = if goal_completion.is_some() {
            Some(self.acceptance_cycle()?)
        } else {
            None
        };
        if goal_completion.is_some() {
            self.project_rank_options(spec, &records, &criteria, &mut options, last.as_deref())?;
        }
        let semantic_pressure = acceptance_cycle
            .as_ref()
            .and_then(|c| c["unmet_count"].as_u64())
            .unwrap_or(0) as f64;
        Ok(State {
            acceptance_cycle,
            goal_completion,
            goal: spec.goal.clone(),
            criteria,
            completed,
            last_commitment: last,
            pending,
            options,
            preferences,
            unmet_pressure: Proxy::new(
                "unmet_pressure",
                (failures as f64 + semantic_pressure) / (1.0 + failures as f64 + semantic_pressure),
            ),
        })
    }
    pub fn next(&self, provider: &dyn ProposalProvider) -> Result<State> {
        self.write(|| self.next_unlocked(provider))
    }
    fn next_unlocked(&self, provider: &dyn ProposalProvider) -> Result<State> {
        self.acceptance_reopen_aims()?;
        let state = self.adaptive_state_inner()?;
        ensure!(
            state.pending.is_none(),
            "observe the pending commitment before requesting next"
        );
        let selection_root = self.store.head_root()?;
        let shadow_events = self.events()?;
        let offered = provider.propose(&state);
        ensure!(
            !offered.is_empty() && offered.len() <= 16,
            "provider bounds"
        );
        let mut seen = BTreeSet::new();
        let mut best = None;
        let mut eligible = vec![];
        for id in offered {
            ensure!(seen.insert(id.clone()), "duplicate provider action");
            let option = state
                .options
                .iter()
                .find(|o| o.action == id)
                .ok_or_else(|| anyhow::anyhow!("provider offered unknown action"))?;
            if option.value == 0 {
                continue;
            }
            if let Some(patches) = self.patches_for(&id)? {
                if self.validate_patch_set(&patches).is_err() {
                    continue;
                }
            }
            let verification = self.project_verification_for(&id)?;
            let origin = if verification {
                Source::Operator
            } else {
                Source::Intuition
            };
            let args = self.blob(&serde_json::to_vec(&id)?)?;
            let mut candidate = Candidate::new(
                if verification {
                    ActionKind::RunApprovedTest
                } else if self.coding_config()?.is_some() {
                    ActionKind::ProposePatch
                } else {
                    ActionKind::Generate
                },
                origin,
                args,
                option.evidence.clone(),
                vec![],
            )?;
            candidate.support = self
                .records()?
                .into_iter()
                .filter_map(|(_, r)| match r {
                    Record::Want {
                        preference,
                        actions,
                        ..
                    } if (actions.contains(&id)
                        || (actions.iter().any(|a| a == "scope")
                            && (id.starts_with("patch-") || verification)))
                        && state
                            .preferences
                            .iter()
                            .any(|p| p.preference.id == preference && p.is_open()) =>
                    {
                        Some(preference)
                    }
                    _ => None,
                })
                .collect();
            candidate
                .support
                .extend(self.acceptance_active_aims()?.into_values());
            candidate.support.sort();
            candidate.support.dedup();
            self.append(
                EventKind::Proposed {
                    candidate: candidate.clone(),
                },
                origin,
            )?;
            self.record_shadow(
                selection_root,
                state
                    .goal_completion
                    .as_ref()
                    .map(|c| c.snapshot)
                    .unwrap_or(selection_root),
                &candidate,
                &shadow_events,
                if state.goal_completion.is_some() {
                    "bounded-project@v1"
                } else {
                    "adaptive@v1"
                },
            );
            eligible.push((candidate.clone(), option));
            if best
                .as_ref()
                .is_none_or(|(_, o): &(Candidate, &OptionView)| option.score.value > o.score.value)
            {
                best = Some((candidate, option));
            }
        }
        let tier2 = self.tier2_select(
            selection_root,
            if state.goal_completion.is_some() {
                "bounded-project@v1"
            } else {
                "adaptive@v1"
            },
            &shadow_events,
            &eligible
                .iter()
                .filter(|(_, o)| o.score.value > 0.0)
                .map(|(c, o)| super::tier2::Choice {
                    id: &o.action,
                    candidate: c,
                    assessment: o.decision.as_ref(),
                })
                .collect::<Vec<_>>(),
        )?;
        let policy_trace = if let Some(applied) = &tier2 {
            Some(applied.trace.clone())
        } else {
            self.select_with_policy(
                selection_root,
                &eligible
                    .iter()
                    .filter(|(_, o)| o.score.value > 0.0)
                    .map(|(_, o)| (o.action.clone(), o.score.value))
                    .collect::<Vec<_>>(),
            )?
        };
        if let Some(trace) = policy_trace.as_ref().filter(|_| tier2.is_none()) {
            let control = self.success_prior_options(
                &state,
                &eligible
                    .iter()
                    .map(|(_, o)| o.action.clone())
                    .collect::<Vec<_>>(),
            )?;
            let scores = control
                .iter()
                .map(|o| (o.action.clone(), o.score.value))
                .collect::<Vec<_>>();
            let support = control
                .iter()
                .filter(|o| o.score.value > 0.0)
                .map(|o| o.action.clone())
                .collect::<Vec<_>>();
            self.record_selection_intervention(trace,"adaptive/project: reset empirical dispatch-success probability to prior 0.5; preserve observed returns, semantic review/projection learning, commitment, costs, desires and validated offered actions",&scores,&support)?;
        }
        let (candidate, option) = if let Some(trace) = &policy_trace {
            let Some(id) = trace.selected.as_ref() else {
                return self.adaptive_state_inner();
            };
            eligible
                .into_iter()
                .find(|(_, o)| &o.action == id)
                .ok_or_else(|| anyhow::anyhow!("selected action absent"))?
        } else {
            best.ok_or_else(|| anyhow::anyhow!("no proposal serves an admitted criterion"))?
        };
        self.commit_adaptive_selection(
            provider,
            &state,
            candidate,
            option,
            policy_trace.as_ref(),
            tier2.as_ref(),
        )
    }
    fn commit_adaptive_selection(
        &self,
        provider: &dyn ProposalProvider,
        state: &State,
        candidate: Candidate,
        option: &OptionView,
        policy_trace: Option<&tsr_domain::selection::Trace>,
        tier2: Option<&super::tier2::Applied>,
    ) -> Result<State> {
        let mut commitment_evidence = option.evidence.clone();
        if let Some(applied) = tier2 {
            if self.coding_config()?.is_some_and(|c| c.project.is_some()) {
                self.project_tier2_action_budget(&option.action)?;
            }
            ensure!(
                applied.trace.selected.as_deref() == Some(option.action.as_str())
                    && applied.probabilities.contains_key(&option.action)
                    && applied.scores.contains_key(&option.action),
                "Tier2 selection handoff mismatch"
            );
            commitment_evidence.push(
                applied
                    .receipt
                    .ok_or_else(|| anyhow::anyhow!("missing Tier2 gate receipt"))?,
            );
        }
        let tier2_probability = tier2
            .as_ref()
            .and_then(|a| a.probabilities.get(&option.action))
            .copied();
        let forecast = Forecast::new(
            OutcomeDefId(Hash::of(&[b"configured_criteria_met@v1"])),
            Horizon::TaskClose,
            Forecaster::Controller,
            Probability::new(option.prediction.value)?,
            self.events()?.len() as u64,
        );
        self.append(
            EventKind::Forecast {
                forecast: forecast.clone(),
                candidate: candidate.id,
            },
            Source::Rules,
        )?;
        let acceptance_forecast = self.acceptance_freeze(candidate.id)?;
        for (who, p) in [
            (
                Forecaster::Generator,
                provider.common_event_forecast(state, &option.action),
            ),
            (Forecaster::Controller, tier2_probability),
        ] {
            if let Some(p) = p {
                self.append(
                    EventKind::Forecast {
                        forecast: Forecast::new(
                            tsr_learn::decision_event_id(),
                            Horizon::TaskClose,
                            who.clone(),
                            Probability::new(p)?,
                            self.events()?.len() as u64,
                        ),
                        candidate: candidate.id,
                    },
                    if who == Forecaster::Generator {
                        Source::Generator
                    } else {
                        Source::Rules
                    },
                )?;
            }
        }
        let commitment = self.append(
            EventKind::Committed {
                candidate: candidate.id,
                decision: Decision::Selected {
                    value: candidate.id,
                    utility: Utility {
                        p_success: tier2_probability.unwrap_or(option.prediction.value),
                        value: option
                            .decision
                            .as_ref()
                            .map(|d| d.gain)
                            .unwrap_or(option.value as f64),
                        cost: option
                            .decision
                            .as_ref()
                            .map(|d| d.effort + d.continuity_cost)
                            .unwrap_or(
                                option.prediction.value * option.value as f64 - option.score.value,
                            ),
                        ev: tier2
                            .as_ref()
                            .and_then(|a| a.scores.get(&option.action))
                            .copied()
                            .unwrap_or(option.score.value),
                    },
                    propensity: policy_trace
                        .as_ref()
                        .and_then(|t| t.propensity)
                        .unwrap_or(1.0),
                    evidence: commitment_evidence,
                },
            },
            Source::Rules,
        )?;
        if let Some((revision, forecast)) = acceptance_forecast {
            self.adaptive_record(
                &Record::Coding {
                    record: coding::CodingRecord::AcceptanceExpectation {
                        revision,
                        commitment,
                        forecast,
                    },
                },
                Source::Rules,
            )?;
        }
        self.adaptive_record(
            &Record::Pending {
                action: option.action.clone(),
                commitment,
                forecast,
                criteria: state.criteria.clone(),
            },
            Source::Rules,
        )?;
        self.adaptive_state_inner()
    }
    pub fn observe(&self, commitment: EventId, input: ResultInput) -> Result<State> {
        ensure!(
            self.coding_config()?.is_none(),
            "coding outcomes must come from coding-run receipts"
        );
        self.write(|| self.observe_unlocked(commitment, input, Source::Operator))
    }
    fn observe_unlocked(
        &self,
        commitment: EventId,
        input: ResultInput,
        source: Source,
    ) -> Result<State> {
        let state = self.adaptive_state_inner()?;
        let p = state
            .pending
            .ok_or_else(|| anyhow::anyhow!("no pending commitment"))?;
        ensure!(p.commitment == commitment, "stale or foreign commitment");
        ensure!(
            short(&input.evidence),
            "supply bounded observation evidence"
        );
        ensure!(
            input.met.len() <= 16 && input.met.iter().all(|c| p.criteria.contains(c)),
            "met must name frozen criteria"
        );
        let success = CriteriaAdapter.success(&p.criteria, &input);
        let returned = self.adaptive_record(
            &Record::Returned {
                commitment,
                action: p.action.clone(),
                success,
                input: input.clone(),
            },
            source,
        )?;
        let mut surprise = vec![Surprise {
            forecast: p.forecast.id,
            residual: p.forecast.raw.get() - if success { 1.0 } else { 0.0 },
        }];
        surprise.extend(self.project_resolve_evaluator(&p, returned, success)?);
        let unmet = p
            .criteria
            .iter()
            .filter(|c| !input.met.contains(c))
            .map(|c| {
                (
                    CriterionId(Hash::of(&[c.as_bytes()])),
                    UnmetKind::WrongBehaviour,
                )
            })
            .collect();
        let contrast = Filter::contrast_from_criteria(returned, unmet, vec![returned]);
        self.append(
            EventKind::Observed {
                observation: Observation::Return {
                    surprise,
                    contrast: Some(contrast),
                },
            },
            Source::Rules,
        )?;
        if let Some(r) = input.proposed_refinement {
            ensure!(!success, "refinement requires unmet criteria");
            ensure!(
                short(&r.criterion)
                    && short(&r.text)
                    && !r.actions.is_empty()
                    && r.actions.len() <= 16
                    && r.actions
                        .iter()
                        .all(|a| state.options.iter().any(|o| o.action == *a)),
                "invalid refinement"
            );
            // New names only: a proposal cannot remap an already admitted criterion.
            ensure!(
                !self.records()?.iter().any(
                    |(_, f)| matches!(f,Record::Want{criterion,..} if *criterion==r.criterion)
                ),
                "criterion already exists"
            );
            ensure!(
                !state
                    .preferences
                    .iter()
                    .any(|p| p.preference.text == r.text),
                "preference already exists"
            );
            self.adaptive_want(
                &r.criterion,
                &r.text,
                r.actions,
                returned,
                Source::Generator,
            )?;
        }
        self.adaptive_state_inner()
    }
    pub fn adaptive_admit(&self, id: PreferenceId) -> Result<State> {
        self.write(|| {
            let state = self.adaptive_state_inner()?;
            let preference = state
                .preferences
                .iter()
                .find(|p| p.preference.id == id)
                .ok_or_else(|| anyhow::anyhow!("unknown preference"))?;
            ensure!(
                preference.is_open()
                    || state.preferences.iter().filter(|p| p.is_open()).count() < 16,
                "active criterion limit reached"
            );
            self.append(
                EventKind::StatusSet {
                    preference: id,
                    status: PreferenceStatus::Open,
                    by: Source::Operator,
                },
                Source::Operator,
            )?;
            self.adaptive_state_inner()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn spec() -> Spec {
        Spec {
            goal: "Publish a useful report".into(),
            criteria: vec!["readable".into()],
            actions: vec![
                Action {
                    id: "draft".into(),
                    description: "Write a short draft".into(),
                    serves: vec!["readable".into()],
                    cost: 0.05,
                },
                Action {
                    id: "outline".into(),
                    description: "Outline before drafting".into(),
                    serves: vec!["readable".into()],
                    cost: 0.15,
                },
            ],
            prior: vec![],
            initial_commitment: None,
        }
    }
    fn input(success: bool) -> ResultInput {
        ResultInput {
            met: if success {
                vec!["readable".into()]
            } else {
                vec![]
            },
            evidence: "Operator inspected the resulting report".into(),
            proposed_refinement: None,
        }
    }
    #[test]
    fn tier2_validated_score_handoff_freezes_forecast_propensity_and_receipt() -> Result<()> {
        // Test only the post-gate handoff with synthetic scores. No model is enabled,
        // no fixture receives operational eligibility, and no action executes.
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.configure(spec())?;
        let state = e.adaptive_state()?;
        let scores = vec![("draft".into(), 0.4), ("outline".into(), 0.4)];
        let policy = tsr_domain::selection::Policy {
            margin: 0.1,
            epsilon: 1.0,
            seed: 4,
        };
        let trace = tsr_domain::selection::select(&policy, e.store.head_root()?, &scores)
            .map_err(anyhow::Error::msg)?;
        let option = state
            .options
            .iter()
            .find(|o| Some(&o.action) == trace.selected.as_ref())
            .unwrap();
        let candidate = Candidate::new(
            ActionKind::Generate,
            Source::Intuition,
            Hash::of(&[option.action.as_bytes()]),
            vec![],
            vec![],
        )?;
        let receipt = e.tier2_audit(
            serde_json::json!({"status":"synthetic_handoff_test_only","trace":trace}),
        )?;
        let applied = super::super::tier2::Applied {
            trace: trace.clone(),
            scores: scores.into_iter().collect(),
            probabilities: std::collections::BTreeMap::from([
                ("draft".into(), 0.7),
                ("outline".into(), 0.8),
            ]),
            receipt: Some(receipt),
        };
        let p = applied.probabilities[&option.action];
        let result = e.write(|| {
            e.commit_adaptive_selection(
                &DeterministicProvider,
                &state,
                candidate,
                option,
                Some(&trace),
                Some(&applied),
            )
        })?;
        let commitment = result.pending.unwrap().commitment;
        let events = e.events()?;
        let selected = events.iter().find(|(_, e)| e.id == commitment).unwrap();
        let EventKind::Committed {
            decision:
                Decision::Selected {
                    utility,
                    propensity,
                    evidence,
                    ..
                },
            ..
        } = &selected.1.kind
        else {
            panic!("commitment")
        };
        assert_eq!(utility.p_success, p);
        assert_eq!(utility.ev, 0.4);
        assert_eq!(Some(*propensity), trace.propensity);
        assert!(evidence.contains(&receipt));
        assert!(events.iter().any(|(_,e)|matches!(&e.kind,EventKind::Forecast{forecast,..} if forecast.event==tsr_learn::decision_event_id()&&forecast.raw.get()==p)));
        assert_eq!(
            Engine::open(dir.path())?
                .adaptive_state()?
                .pending
                .unwrap()
                .commitment,
            commitment
        );
        Ok(())
    }
    #[test]
    fn outcome_counterfactual_and_restart_change_next_without_schedule() -> Result<()> {
        for (success, expected) in [(true, "draft"), (false, "outline")] {
            let dir = tempfile::tempdir()?;
            let e = Engine::open(dir.path())?;
            e.configure(spec())?;
            let pending = e.next(&DeterministicProvider)?.pending.unwrap();
            assert_eq!(pending.action, "draft");
            assert!(e.next(&DeterministicProvider).is_err());
            drop(e);
            let e = Engine::open(dir.path())?;
            assert_eq!(
                e.adaptive_state()?.pending.unwrap().commitment,
                pending.commitment
            );
            e.observe(pending.commitment, input(success))?;
            assert!(e.observe(pending.commitment, input(success)).is_err());
            assert_eq!(
                e.next(&DeterministicProvider)?.pending.unwrap().action,
                expected
            );
        }
        Ok(())
    }
    #[test]
    fn expected_failure_preserves_desire_and_separates_contrast() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let mut s = spec();
        s.actions.truncate(1);
        s.prior = (0..9)
            .map(|_| Prior {
                action: "draft".into(),
                success: false,
                evidence: "Observed prior failed attempt".into(),
            })
            .collect();
        e.configure(s)?;
        let pending = e.next(&DeterministicProvider)?.pending.unwrap();
        assert!(pending.forecast.raw.get() < 0.1);
        let after = e.observe(pending.commitment, input(false))?;
        assert!(after.preferences.iter().any(|p| p.is_open()));
        assert!(after.unmet_pressure.value > 0.0);
        assert!(after.options[0].prediction.value < pending.forecast.raw.get());
        assert!(e.events()?.iter().any(|(_, event)| matches!(&event.kind,
            EventKind::Observed { observation: Observation::Return { surprise, contrast: Some(_) } }
            if surprise.iter().any(|s| s.residual > 0.0 && s.residual < 0.1))));
        Ok(())
    }
    #[test]
    fn multiple_desires_change_selected_candidate() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let mut s = spec();
        s.criteria.push("complete".into());
        s.actions[1].serves.push("complete".into());
        e.configure(s)?;
        assert_eq!(
            e.next(&DeterministicProvider)?.pending.unwrap().action,
            "outline"
        );
        Ok(())
    }
    #[test]
    fn initial_commitment_changes_equal_value_choice_without_cycle_schedule() -> Result<()> {
        for initial in ["draft", "outline"] {
            let dir = tempfile::tempdir()?;
            let e = Engine::open(dir.path())?;
            let mut s = spec();
            s.actions[1].cost = s.actions[0].cost;
            s.initial_commitment = Some(initial.into());
            e.configure(s)?;
            assert_eq!(e.adaptive_state()?.completed, 0);
            assert_eq!(
                e.next(&DeterministicProvider)?.pending.unwrap().action,
                initial
            );
        }
        Ok(())
    }
    #[test]
    fn initial_evidence_changes_first_choice_not_cycle_number() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let mut s = spec();
        s.initial_commitment = Some("draft".into());
        s.prior = vec![Prior {
            action: "draft".into(),
            success: false,
            evidence: "Previous draft was unreadable".into(),
        }];
        e.configure(s)?;
        assert_eq!(
            e.next(&DeterministicProvider)?.pending.unwrap().action,
            "outline"
        );
        Ok(())
    }
    #[test]
    fn pending_refinement_only_affects_ranking_after_explicit_admission() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.configure(spec())?;
        let p = e.next(&DeterministicProvider)?.pending.unwrap();
        let mut r = input(false);
        r.proposed_refinement = Some(Refinement {
            criterion: "concise".into(),
            text: "Keep the report concise".into(),
            actions: vec!["draft".into()],
        });
        let state = e.observe(p.commitment, r)?;
        let pending = state.preferences.iter().find(|p| !p.is_active()).unwrap();
        assert_eq!(DeterministicProvider.propose(&state)[0], "outline");
        let admitted = e.adaptive_admit(pending.preference.id)?;
        assert!(
            admitted.options.iter().all(|o| o.prediction.value == 0.5),
            "old-criterion outcomes must not train a changed goal"
        );
        assert_eq!(DeterministicProvider.propose(&admitted)[0], "draft");
        assert_eq!(
            e.next(&DeterministicProvider)?.pending.unwrap().action,
            "draft"
        );
        Ok(())
    }
    #[test]
    fn learned_success_intervention_changes_ranking_with_same_draw() -> Result<()> {
        use crate::engine::decision_policy::Update;
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.configure(spec())?;
        let first = e.next(&DeterministicProvider)?.pending.unwrap();
        e.observe(first.commitment, input(false))?;
        e.update_decision_policy(Update {
            state_root: e.store.head_root()?,
            policy: tsr_domain::selection::Policy {
                margin: 1.0,
                epsilon: 0.5,
                seed: 7,
            },
            reason: "Controlled audit".into(),
        })?;
        e.next(&DeterministicProvider)?;
        let view = e.decision_policy_view()?;
        let c = &view["learning_interventions"][0]["comparison"];
        assert_eq!(c["ranking_changed"], true);
        assert_eq!(c["support_changed"], false);
        assert_eq!(c["factual"]["draw"], c["fixed_support"]["draw"]);
        assert_ne!(c["factual"]["scores"], c["fixed_support"]["scores"]);
        assert_eq!(view, Engine::open(dir.path())?.decision_policy_view()?);
        Ok(())
    }
    #[test]
    fn shared_policy_applies_to_adaptive_candidates_and_restart() -> Result<()> {
        use crate::engine::decision_policy::Update;
        use tsr_domain::selection::Policy;
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let mut configured = spec();
        configured.actions[1].cost = configured.actions[0].cost;
        configured.actions.push(Action {
            id: "costly".into(),
            description: "Bounded but nonpositive".into(),
            serves: vec!["readable".into()],
            cost: 1.0,
        });
        e.configure(configured)?;
        e.update_decision_policy(Update {
            state_root: e.store.head_root()?,
            policy: Policy {
                margin: 0.0,
                epsilon: 0.0,
                seed: 1,
            },
            reason: "Exact tie abstention".into(),
        })?;
        assert!(e.next(&DeterministicProvider)?.pending.is_none());
        assert!(Engine::open(dir.path())?
            .adaptive_state()?
            .pending
            .is_none());
        assert_eq!(
            e.decision_policy_view()?["traces"][0]["reason"],
            "near_tie_abstention"
        );
        assert_eq!(
            e.decision_policy_view()?["traces"][0]["scores"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        e.update_decision_policy(Update {
            state_root: e.store.head_root()?,
            policy: Policy {
                margin: 0.0,
                epsilon: 1.0,
                seed: 1,
            },
            reason: "Opt in".into(),
        })?;
        assert!(e.next(&DeterministicProvider)?.pending.is_some());
        let probabilities = e
            .events()?
            .into_iter()
            .filter_map(|(_, e)| match e.kind {
                EventKind::Committed {
                    decision: Decision::Selected { propensity, .. },
                    ..
                } => Some(propensity),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(probabilities, vec![0.5]);
        let audits = e.decision_policy_view()?;
        assert_eq!(
            audits["learning_interventions"].as_array().unwrap().len(),
            2
        );
        assert_eq!(
            audits["learning_interventions"][1]["comparison"]["factual"]["propensity"],
            0.5
        );
        assert_eq!(audits, Engine::open(dir.path())?.decision_policy_view()?);
        Ok(())
    }
    #[test]
    fn shadow_diagnostics_preserve_adaptive_choice_and_policy() -> Result<()> {
        let a = tempfile::tempdir()?;
        let b = tempfile::tempdir()?;
        let e = Engine::open(a.path())?;
        e.configure(spec())?;
        let other = Engine::open(b.path())?;
        for (_, event) in e.events()? {
            other.store.append(&event)?;
        }
        for entry in fs::read_dir(a.path().join("cas"))? {
            let entry = entry?;
            fs::copy(entry.path(), b.path().join("cas").join(entry.file_name()))?;
        }
        let root = e.store.head_root()?;
        other.configure_shadow(crate::engine::shadow::Update {
            state_root: root,
            reason: "noninterference".into(),
            config: tsr_learn::shadow::Config {
                bundle_directory: b.path().join("missing"),
                bundle_id: Hash::of(&[b"missing"]),
                dataset_directory: None,
                event: tsr_learn::DECISION_EVENT.into(),
                family: "fixture".into(),
                lineage: "fixture".into(),
                training_as_of: None,
            },
        })?;
        assert_eq!(other.store.head_root()?, root);
        let x = e.next(&DeterministicProvider)?;
        let y = other.next(&DeterministicProvider)?;
        assert_eq!(
            x.pending.as_ref().unwrap().action,
            y.pending.as_ref().unwrap().action
        );
        assert_eq!(
            e.decision_policy_view()?["traces"],
            other.decision_policy_view()?["traces"]
        );
        assert!(other.shadow_view()?["records"].as_array().unwrap().len() > 1);
        Ok(())
    }
    #[test]
    fn decoded_record_cache_rechecks_missing_and_changed_cas() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.configure(spec())?;
        let original = serde_json::to_value(e.records()?)?;
        let digest = *e.record_cache.borrow().keys().next().unwrap();
        let path = dir.path().join("cas").join(digest.to_hex());
        let bytes = fs::read(&path)?;
        fs::write(&path, b"{}")?;
        assert!(e.records().is_err());
        fs::write(&path, &bytes)?;
        assert_eq!(serde_json::to_value(e.records()?)?, original);
        fs::remove_file(&path)?;
        assert!(e.records().is_err());
        Ok(())
    }
    #[test]
    fn decoded_record_cache_tracks_appends_and_rollback_without_stale_state() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.configure(spec())?;
        let before = serde_json::to_value(e.records()?)?;
        let failed: Result<()> = e.write(|| {
            e.adaptive_record(
                &Record::AttentionConsumed {
                    attention: serde_json::json!({"authored":"rolled back"}),
                },
                Source::Rules,
            )?;
            e.records()?;
            anyhow::bail!("authored rollback")
        });
        assert!(failed.is_err());
        assert_eq!(serde_json::to_value(e.records()?)?, before);
        let other = Engine::open(dir.path())?;
        other.adaptive_record(
            &Record::AttentionConsumed {
                attention: serde_json::json!({"authored":"external append"}),
            },
            Source::Rules,
        )?;
        assert_eq!(e.records()?.len(), before.as_array().unwrap().len() + 1);
        e.adaptive_record(
            &Record::AttentionConsumed {
                attention: serde_json::json!({"authored":"same connection"}),
            },
            Source::Rules,
        )?;
        let cached = serde_json::to_value(e.records()?)?;
        e.record_cache.borrow_mut().clear();
        assert_eq!(serde_json::to_value(e.records()?)?, cached);
        assert_eq!(serde_json::to_value(other.records()?)?, cached);
        Ok(())
    }
    #[test]
    fn decoded_record_cache_does_not_hide_external_broken_chain() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.configure(spec())?;
        e.records()?;
        let digest = *e.record_cache.borrow().keys().next().unwrap();
        let other = Engine::open(dir.path())?;
        other.store.append(&Event::new(
            Engine::task(),
            vec![],
            Source::Rules,
            EventKind::Observed {
                observation: Observation::Evidence {
                    source: SOURCE.into(),
                    digest,
                },
            },
            "authored corruption",
        ))?;
        assert!(e.records().is_err());
        Ok(())
    }
    #[test]
    fn invalid_observation_rolls_back_and_provider_cannot_invent_actions() -> Result<()> {
        struct Bad;
        impl ProposalProvider for Bad {
            fn propose(&self, _: &State) -> Vec<String> {
                vec!["execute arbitrary command".into()]
            }
        }
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.configure(spec())?;
        let root = e.store.head_root()?;
        assert!(e.next(&Bad).is_err());
        assert_eq!(e.store.head_root()?, root);
        let p = e.next(&DeterministicProvider)?.pending.unwrap();
        let root = e.store.head_root()?;
        let mut r = input(true);
        r.met.push("undeclared".into());
        assert!(e.observe(p.commitment, r).is_err());
        assert_eq!(e.store.head_root()?, root);
        Ok(())
    }
}

pub mod provider;

pub mod coding;
