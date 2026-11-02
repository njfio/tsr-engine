//! Finite service objectives. Artifact acceptance is neither round outcome nor objective acceptance.
//! The immutable protected service entry owns domain validation; the native layer owns order,
//! receipts, provenance and budgets. No provider, scenario policy or automatic action is here.
use super::*;
use anyhow::Context;
use serde_json::{json, Value};
const SOURCE: &str = "project-finite-objective@v1";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Condition {
    pub id: String,
    pub input: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Admission {
    pub state_root: Hash,
    pub conditions: Vec<Condition>,
    pub service_entry: String,
    pub max_output_bytes: usize,
    pub timeout_ms: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub binding: Hash,
    pub rationale: String,
    pub success_probability: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub state_root: Hash,
    pub binding: Hash,
    pub receipt: EventId,
    pub decision: String,
    pub reviewer: String,
    pub generator: String,
    pub evidence: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FinalReview {
    pub state_root: Hash,
    pub binding: Hash,
    pub decision: String,
    pub reviewer: String,
    pub generator: String,
    pub evidence: String,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Entry {
    Config {
        admission: Admission,
        scope: Hash,
        requirements: EventId,
        parents: Vec<PreferenceId>,
    },
    Prepared {
        choice: Choice,
        round: usize,
        snapshot: Hash,
        previous: String,
        guidance: Hash,
        review: cycle::Review,
    },
    Retired {
        prepared: EventId,
        reason: String,
    },
    Attempt {
        prepared: EventId,
        binding: Hash,
        resource: String,
        forecast: Forecast,
    },
    Receipt {
        prepared: EventId,
        attempt: EventId,
        resource: String,
        binding: Hash,
        elapsed_ms: u64,
        runs: Vec<String>,
        output: Option<String>,
        exit: i32,
        complete: bool,
    },
    Reviewed {
        review: Review,
    },
    Final {
        review: FinalReview,
    },
}
fn h<T: Serialize>(v: &T) -> Result<Hash> {
    Ok(Hash::of(&[&serde_json::to_vec(v)?]))
}
impl Engine {
    fn objective_entries(&self) -> Result<Vec<(EventId, Entry)>> {
        let mut out = vec![];
        for (_, e) in self.events()? {
            if let EventKind::Observed {
                observation: Observation::Evidence { source, digest },
            } = &e.kind
            {
                if source != SOURCE {
                    continue;
                }
                let bytes = fs::read(self.directory.join("cas").join(digest.to_hex()))?;
                ensure!(Hash::of(&[&bytes]) == *digest, "objective evidence changed");
                let entry: Entry = serde_json::from_slice(&bytes)?;
                let origin = if matches!(entry, Entry::Receipt { .. }) {
                    Source::Rules
                } else {
                    Source::Operator
                };
                ensure!(e.source == origin, "objective evidence provenance");
                out.push((e.id, entry));
            }
        }
        Ok(out)
    }
    fn objective_append(&self, entry: Entry) -> Result<EventId> {
        let origin = if matches!(entry, Entry::Receipt { .. }) {
            Source::Rules
        } else {
            Source::Operator
        };
        let digest = self.blob(&serde_json::to_vec(&entry)?)?;
        self.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    source: SOURCE.into(),
                    digest,
                },
            },
            origin,
        )
    }
    pub(crate) fn objective_config(&self) -> Result<Option<(EventId, Admission)>> {
        let entries = self.objective_entries()?;
        let configs: Vec<_> = entries
            .iter()
            .filter_map(|(id, e)| {
                if let Entry::Config {
                    admission,
                    scope,
                    requirements,
                    ..
                } = e
                {
                    Some((*id, admission, scope, requirements))
                } else {
                    None
                }
            })
            .collect();
        ensure!(configs.len() <= 1, "duplicate objective configuration");
        if let Some((id, a, scope, requirements)) = configs.first() {
            ensure!(
                **scope == h(&self.project_configuration()?)?
                    && Some(**requirements)
                        == self.project_acceptance_status()?.requirements_revision,
                "objective scope/criteria drift"
            );
            return Ok(Some((*id, (*a).clone())));
        }
        Ok(None)
    }
    pub fn project_objective_configure(&self, a: Admission) -> Result<Value> {
        self.write(|| {
            ensure!(
                a.state_root == self.store.head_root()? && self.objective_config()?.is_none(),
                "stale/duplicate objective admission"
            );
            ensure!(
                self.adaptive_state_inner()?.pending.is_none()
                    && self.project_information_pending()?.is_none(),
                "pending action prevents objective admission"
            );
            let spec = self.project_configuration()?;
            ensure!(
                !spec.browser
                    && !spec.cargo
                    && spec.named_package.is_none()
                    && spec.protected.contains_key(&a.service_entry)
                    && a.service_entry.ends_with(".rs")
                    && valid_path(&a.service_entry),
                "protected non-Cargo Rust service entry required"
            );
            ensure!(
                (1..=8).contains(&a.conditions.len())
                    && (1..=2048).contains(&a.max_output_bytes)
                    && (1..=30000).contains(&a.timeout_ms),
                "finite objective bounds"
            );
            let mut ids = BTreeSet::new();
            for c in &a.conditions {
                ensure!(
                    short(&c.id)
                        && ids.insert(c.id.clone())
                        && !c.input.is_empty()
                        && c.input.len() <= 2048
                        && !c.input.contains('\0'),
                    "bounded unique condition data"
                );
            }
            let requirements = self
                .project_acceptance_status()?
                .requirements_revision
                .context("objective requirements missing")?;
            let parents = self
                .acceptance_current_aims()?
                .into_values()
                .collect::<Vec<_>>();
            ensure!(
                !parents.is_empty(),
                "objective needs existing parent meanings"
            );
            self.objective_append(Entry::Config {
                admission: a,
                scope: h(&spec)?,
                requirements,
                parents,
            })?;
            self.project_objective_context()
        })
    }
    pub(crate) fn project_objective_active_parents(&self) -> Result<BTreeSet<PreferenceId>> {
        let entries = self.objective_entries()?;
        if entries
            .iter()
            .any(|(_, e)| matches!(e, Entry::Final { .. }))
        {
            return Ok(BTreeSet::new());
        }
        let parents = entries.iter().find_map(|(_, e)| {
            if let Entry::Config { parents, .. } = e {
                Some(parents)
            } else {
                None
            }
        });
        let state = self.adaptive_state_inner()?;
        let relation = self.project_relational_context()?;
        let active = relation["inputs"]["parents"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let mut eligible = BTreeSet::new();
        for id in parents.into_iter().flatten() {
            let Some(p) = state.preferences.iter().find(|p| p.preference.id == *id) else {
                continue;
            };
            if matches!(
                p.status,
                Some(PreferenceStatus::Withdrawn | PreferenceStatus::OutOfScope)
            ) {
                continue;
            }
            // Contributor roots are not discarded when reviewed meaning develops. A bare
            // ledger supersession is not itself evidence of reviewed culmination.
            let represented = active.iter().any(|meaning| {
                meaning["id"] == json!(id)
                    && (!matches!(p.status, Some(PreferenceStatus::Superseded(_)))
                        || !meaning["revision"].is_null()
                        || relation["history"]
                            .as_array()
                            .into_iter()
                            .flatten()
                            .filter(|g| g["withdrawn"] != true && g["superseded"] != true)
                            .any(|g| {
                                g["proposal"]["parents"]
                                    .as_array()
                                    .is_some_and(|p| p.contains(&json!(id)))
                            }))
            });
            if represented {
                eligible.insert(*id);
            }
        }
        Ok(eligible)
    }
    fn objective_guidance_binding(&self) -> Result<Hash> {
        let c = self.project_relational_context()?;
        h(
            &json!({"parents":c["inputs"]["parents"],"public_desires":c["inputs"]["public_desires"],"guidance":c["guidance"],"active_contributors":self.project_objective_active_parents()?}),
        )
    }
    pub fn project_objective_context(&self) -> Result<Value> {
        let Some((admission, a)) = self.objective_config()? else {
            return Ok(Value::Null);
        };
        let entries = self.objective_entries()?;
        let mut reviewed = vec![];
        let mut previous = String::new();
        let mut pending = None;
        let mut final_review = None;
        for (id, e) in &entries {
            match e {
                Entry::Prepared { round, .. } => {
                    ensure!(
                        pending.is_none() && *round == reviewed.len(),
                        "duplicate/out-of-order objective preparation"
                    );
                    pending = Some((*id, false, None));
                }
                Entry::Retired { prepared, .. } => {
                    let p = pending.as_ref().context("retirement without preparation")?;
                    ensure!(
                        p.0 == *prepared && !p.1,
                        "attempted service cannot be retired"
                    );
                    pending = None;
                }
                Entry::Attempt { prepared, .. } => {
                    let p = pending
                        .as_mut()
                        .context("objective attempt without preparation")?;
                    ensure!(
                        p.0 == *prepared && !p.1,
                        "duplicate/foreign objective attempt"
                    );
                    p.1 = true;
                }
                Entry::Receipt {
                    prepared,
                    attempt,
                    output,
                    ..
                } => {
                    let p = pending
                        .as_mut()
                        .context("objective receipt without preparation")?;
                    ensure!(
                        p.0 == *prepared
                            && p.1
                            && p.2.is_none()
                            && entries.iter().any(|(i, x)| i == attempt
                                && matches!(x,Entry::Attempt{prepared:q,..} if q==prepared)),
                        "duplicate/foreign objective receipt"
                    );
                    p.2 = Some((*id, output.clone()));
                }
                Entry::Reviewed { review } => {
                    let p = pending
                        .as_ref()
                        .context("objective review without pending round")?;
                    let (receipt, output) =
                        p.2.as_ref().context("objective review without receipt")?;
                    ensure!(*receipt == review.receipt, "foreign objective review");
                    if let Some(output) = output {
                        previous = output.clone()
                    };
                    reviewed.push(json!({"receipt":receipt,"review_record":id,"decision":review.decision,"evidence":review.evidence,"reviewer":review.reviewer}));
                    pending = None;
                }
                Entry::Final { review } => {
                    ensure!(final_review.is_none(), "duplicate objective final review");
                    final_review = Some(review);
                }
                _ => {}
            }
        }
        ensure!(
            reviewed.len() <= a.conditions.len(),
            "objective round overflow"
        );
        let receipts = self.project_objective_receipts_inner(&entries)?;
        let next = if let Some((_, attempted, receipt)) = &pending {
            if receipt.is_some() {
                "round_review"
            } else if *attempted {
                "reconcile"
            } else {
                "service_resume"
            }
        } else if final_review.is_some() {
            "stop"
        } else if reviewed.iter().any(|v| v["decision"] == "unknown") {
            "unresolved"
        } else if reviewed.len() == a.conditions.len() {
            "objective_review"
        } else if self.project_objective_active_parents()?.is_empty() {
            "inactive"
        } else {
            "propose"
        };
        let current = if next == "propose" {
            a.conditions.get(reviewed.len()).cloned()
        } else {
            None
        };
        let body = json!({"version":"finite-objective@v1","admission":admission,"objective":h(&a)?,"schedule":a.conditions.iter().map(|c|json!({"id":c.id,"input_digest":h(&c.input).unwrap()})).collect::<Vec<_>>(),"horizon":a.conditions.len(),"round":reviewed.len(),"active_parents":self.project_objective_active_parents()?,"guidance_binding":self.objective_guidance_binding()?,"current_condition":current,"previous_output":previous,"source_snapshot":self.project_acceptance_status()?.snapshot,"requirements_revision":self.project_acceptance_status()?.requirements_revision,"receipts":receipts,"round_reviews":reviewed,"next":next,"final_review":final_review,"meaning":"Finite service outcomes, not artifact acceptance or TSR culmination. Future condition bodies withheld. Actual stdout is prior-output data, not proof of correctness. Independent round and objective reviews are distinct. No automatic action or renewed budget."});
        Ok(
            json!({"binding":h(&body)?,"context":body,"choice_schema":{"type":"object","additionalProperties":false,"required":["binding","rationale","success_probability"],"properties":{"binding":{"const":h(&body)?},"rationale":{"type":"string","minLength":1,"maxLength":512},"success_probability":{"type":"number","minimum":0,"maximum":1}}},"limits":"rationale <=512 UTF-8 bytes; forecast is protected service invocation exit-zero, not qualitative fulfillment; service uses current condition and exact prior actual stdout only"}),
        )
    }
    fn project_objective_receipts_inner(&self, entries: &[(EventId, Entry)]) -> Result<Vec<Value>> {
        let mut out = vec![];
        let config = entries.iter().find_map(|(_, e)| {
            if let Entry::Config { admission, .. } = e {
                Some(admission)
            } else {
                None
            }
        });
        for (id, e) in entries {
            if let Entry::Receipt {
                prepared,
                attempt,
                runs,
                output,
                exit,
                complete,
                ..
            } = e
            {
                let Entry::Prepared {
                    round,
                    snapshot,
                    previous,
                    choice,
                    ..
                } = &entries
                    .iter()
                    .find(|(i, _)| i == prepared)
                    .context("missing objective preparation")?
                    .1
                else {
                    bail!("bad objective preparation")
                };
                out.push(json!({"receipt":id,"prepared":prepared,"attempt":attempt,"round":round,"condition":config.context("missing objective configuration")?.conditions[*round],"source_snapshot":snapshot,"previous_output":previous,"previous_output_digest":h(previous)?,"output":output,"exit":exit,"complete":complete,"runner_receipts":runs,"choice":choice,"meaning":"Actual bounded program stdout/exit; domain evidence from frozen protected entry, independent interpretation still required"}));
            }
        }
        Ok(out)
    }
    pub(crate) fn project_objective_receipts(&self) -> Result<Vec<Value>> {
        self.project_objective_receipts_inner(&self.objective_entries()?)
    }
    pub(crate) fn validate_project_service(&self, c: &Choice) -> Result<()> {
        let v = self.project_objective_context()?;
        ensure!(
            v["binding"] == json!(c.binding) && v["context"]["next"] == "propose",
            "stale/unavailable objective service"
        );
        ensure!(
            short(&c.rationale)
                && c.success_probability.is_finite()
                && (0.0..=1.0).contains(&c.success_probability),
            "bounded service rationale/forecast"
        );
        ensure!(
            self.adaptive_state_inner()?.pending.is_none()
                && self.project_information_pending()?.is_none(),
            "pending action prevents service"
        );
        Ok(())
    }
    pub(crate) fn prepare_project_service(
        &self,
        c: Choice,
        review: cycle::Review,
    ) -> Result<EventId> {
        self.validate_project_service(&c)?;
        let v = self.project_objective_context()?;
        self.objective_append(Entry::Prepared {
            choice: c,
            round: v["context"]["round"].as_u64().unwrap() as usize,
            snapshot: self.project_acceptance_status()?.snapshot,
            previous: v["context"]["previous_output"].as_str().unwrap().into(),
            guidance: self.objective_guidance_binding()?,
            review,
        })
    }
    pub(crate) fn project_objective_pending(&self) -> Result<bool> {
        let entries = self.objective_entries()?;
        Ok(entries
            .iter()
            .filter(|(_, e)| matches!(e, Entry::Prepared { .. }))
            .count()
            > entries
                .iter()
                .filter(|(_, e)| matches!(e, Entry::Reviewed { .. } | Entry::Retired { .. }))
                .count())
    }
    pub(crate) fn project_objective_measurement(
        &self,
        resource: &str,
        binding: Hash,
    ) -> Result<Option<u64>> {
        Ok(self.objective_entries()?.iter().find_map(|(_, e)| match e {
            Entry::Receipt {
                resource: r,
                binding: b,
                elapsed_ms,
                ..
            } if r == resource && *b == binding => Some(*elapsed_ms),
            _ => None,
        }))
    }
    /// Retire only a known-unattempted stale preparation. Original review remains in history.
    /// This creates no replacement action: ordinary proposal and independent review are required.
    pub fn project_objective_retire_stale(&self, state_root: Hash) -> Result<Value> {
        self.write(|| {
            ensure!(
                state_root == self.store.head_root()?,
                "stale retirement root"
            );
            let context = self.project_objective_context()?;
            ensure!(
                context["context"]["next"] == "service_resume",
                "only unattempted service may retire"
            );
            let entries = self.objective_entries()?;
            let (prepared, entry) = entries
                .iter()
                .rev()
                .find(|(_, e)| matches!(e, Entry::Prepared { .. }))
                .context("missing preparation")?;
            let Entry::Prepared {
                snapshot, guidance, ..
            } = entry
            else {
                unreachable!()
            };
            ensure!(
                *snapshot != self.project_acceptance_status()?.snapshot
                    || *guidance != self.objective_guidance_binding()?,
                "current review cannot be retired as stale"
            );
            self.objective_append(Entry::Retired {
                prepared: *prepared,
                reason: "Reviewed source or guidance developed; new choice and review required"
                    .into(),
            })?;
            self.project_objective_context()
        })
    }
    pub fn project_objective_resume(&self, binding: Hash) -> Result<Value> {
        let spec = self.project_configuration()?;
        struct Staged {
            compile: SealedRunner,
            service: SealedRunner,
        }
        impl Runner for Staged {
            fn run(&self, copy: &Path, argv: &[String], limits: &Limits) -> RunResult {
                if argv.first().is_some_and(|x| x == "./fixture-probe") {
                    self.service.run(copy, argv, limits)
                } else {
                    self.compile.run(copy, argv, limits)
                }
            }
        }
        let runner = Staged {
            compile: SealedRunner::new(Runtime::Docker, spec.image.clone(), spec.user.clone())
                .map_err(anyhow::Error::msg)?,
            service: SealedRunner::new(Runtime::Docker, spec.image, spec.user)
                .map_err(anyhow::Error::msg)?
                .with_read_only_workspace(),
        };
        self.project_objective_execute(binding, &runner)
    }
    fn project_objective_execute(&self, binding: Hash, runner: &dyn Runner) -> Result<Value> {
        let view = self.project_cycle_request()?;
        ensure!(
            view["branch"] == "service_resume" && view["binding"] == json!(binding),
            "stale/non-service cycle"
        );
        let (_, a) = self.objective_config()?.context("objective missing")?;
        let entries = self.objective_entries()?;
        let (prepared, entry) = entries
            .iter()
            .rev()
            .find(|(_, e)| matches!(e, Entry::Prepared { .. }))
            .context("service preparation missing")?;
        let Entry::Prepared {
            round,
            snapshot,
            previous,
            choice,
            guidance,
            ..
        } = entry
        else {
            unreachable!()
        };
        ensure!(
            *snapshot == self.project_acceptance_status()?.snapshot
                && *guidance == self.objective_guidance_binding()?
                && !self.project_objective_active_parents()?.is_empty(),
            "source or reviewed guidance changed before service; fresh review required"
        );
        let (resource, forecast, attempt, root) = self.write(|| {
            ensure!(
                self.store.head_root()?
                    == serde_json::from_value::<Hash>(view["state_root"].clone())?
                    && *snapshot == self.project_acceptance_status()?.snapshot
                    && *guidance == self.objective_guidance_binding()?,
                "reviewed service changed before durable intent"
            );
            ensure!(
                self.project_objective_context()?["context"]["next"] == "service_resume",
                "service already attempted"
            );
            let resource = self
                .reserve_project_preparation_inner(binding, a.timeout_ms * 2)?
                .context("service needs existing resource admission")?;
            let forecast = Forecast::new(
                OutcomeDefId(Hash::of(&[b"protected_service_exit_zero@v1"])),
                Horizon::ThisDispatch,
                Forecaster::Generator,
                Probability::new(choice.success_probability)?,
                self.events()?.len() as u64,
            );
            let attempt = self.objective_append(Entry::Attempt {
                prepared: *prepared,
                binding,
                resource: resource.clone(),
                forecast: forecast.clone(),
            })?;
            Ok((resource, forecast, attempt, self.store.head_root()?))
        })?;
        let work = self.directory.join("coding-work").canonicalize()?;
        let evidence = self
            .directory
            .join("objective-receipts")
            .join(attempt.to_string());
        fs::create_dir_all(&evidence)?;
        let commands = [
            strings(&["rustc", &a.service_entry, "-o", "fixture-probe"]),
            vec![
                "./fixture-probe".into(),
                a.conditions[*round].input.clone(),
                previous.clone(),
            ],
        ];
        let mut runs = vec![];
        let mut output = None;
        let mut exit = -1;
        let mut elapsed_ms = 0;
        let mut complete = false;
        for (i, argv) in commands.iter().enumerate() {
            let remaining = self
                .project_remaining_wall_ms()?
                .context("active objective wall budget required")?;
            let limits = Limits {
                wall_clock_ms: remaining.min(a.timeout_ms),
                max_output_bytes: a.max_output_bytes + 1,
            };
            // Meaning and parent status were validated atomically before reservation.
            // The committed root binds them; our own unresolved reservation may
            // temporarily hide rebounded guidance but cannot change those records.
            ensure!(
                self.store.head_root()? == root
                    && self.project_acceptance_status()?.snapshot == *snapshot,
                "service review changed before worker"
            );
            let run = runner.run(&work, argv, &limits);
            exit = run.exit;
            elapsed_ms += run.duration_ms;
            runs.push(
                tsr_corpus::persist_run(&evidence, &snapshot.to_string(), argv, &run, None)
                    .map_err(anyhow::Error::msg)?,
            );
            ensure!(
                run.execution
                    .as_ref()
                    .is_some_and(|d| d.termination == "runtime_exit"
                        && d.stdout_complete
                        && d.stderr_complete)
                    && run.stdout.len() <= a.max_output_bytes
                    && run.stderr.len() <= a.max_output_bytes,
                "unknown or oversized service output; do not retry"
            );
            ensure!(
                self.store.head_root()? == root
                    && self.project_acceptance_status()?.snapshot == *snapshot,
                "service source/root changed"
            );
            if i == 1 {
                output = Some(String::from_utf8(run.stdout).context("non-UTF8 service output")?);
                complete = true;
            }
            if exit != 0 {
                break;
            }
        }
        self.write(|| {
            ensure!(
                self.store.head_root()? == root
                    && self.project_acceptance_status()?.snapshot == *snapshot,
                "service changed before receipt"
            );
            let receipt = self.objective_append(Entry::Receipt {
                prepared: *prepared,
                attempt,
                resource,
                binding,
                elapsed_ms,
                runs,
                output,
                exit,
                complete,
            })?;
            self.append(
                EventKind::Observed {
                    observation: Observation::Return {
                        surprise: vec![Surprise {
                            forecast: forecast.id,
                            residual: forecast.raw.get()
                                - if complete && exit == 0 { 1.0 } else { 0.0 },
                        }],
                        contrast: None,
                    },
                },
                Source::Rules,
            )?;
            Ok(json!({"receipt":receipt,"next":self.project_cycle_request()?}))
        })
    }
    pub fn project_objective_review(&self, r: Review) -> Result<Value> {
        self.write(|| {
            let v = self.project_objective_context()?;
            ensure!(
                r.state_root == self.store.head_root()?
                    && v["binding"] == json!(r.binding)
                    && v["context"]["next"] == "round_review",
                "stale/out-of-order objective review"
            );
            let receipt = v["context"]["receipts"]
                .as_array()
                .unwrap()
                .last()
                .context("objective receipt missing")?;
            ensure!(
                receipt["receipt"] == json!(r.receipt),
                "foreign objective receipt"
            );
            ensure!(
                matches!(r.decision.as_str(), "met" | "not_met" | "unknown")
                    && [&r.reviewer, &r.generator, &r.evidence]
                        .iter()
                        .all(|x| short(x))
                    && r.reviewer != r.generator,
                "separate bounded objective review required"
            );
            ensure!(
                r.decision != "met" || (receipt["complete"] == true && receipt["exit"] == 0),
                "cannot approve failed/incomplete service"
            );
            self.objective_append(Entry::Reviewed { review: r })?;
            self.project_objective_context()
        })
    }
    pub fn project_objective_accept(&self, r: FinalReview) -> Result<Value> {
        self.write(|| {
            let v = self.project_objective_context()?;
            ensure!(
                r.state_root == self.store.head_root()?
                    && v["binding"] == json!(r.binding)
                    && matches!(
                        v["context"]["next"].as_str(),
                        Some("objective_review" | "unresolved" | "propose" | "inactive")
                    ),
                "stale/pending objective acceptance"
            );
            ensure!(
                matches!(r.decision.as_str(), "accept" | "reject" | "abstain")
                    && [&r.reviewer, &r.generator, &r.evidence]
                        .iter()
                        .all(|x| short(x))
                    && r.reviewer != r.generator,
                "separate bounded objective acceptance required"
            );
            if r.decision == "accept" {
                ensure!(
                    v["context"]["next"] == "objective_review"
                        && v["context"]["round_reviews"]
                            .as_array()
                            .unwrap()
                            .iter()
                            .all(|x| x["decision"] == "met"),
                    "incomplete or unmet objective cannot be accepted"
                );
            }
            self.objective_append(Entry::Final { review: r })?;
            self.project_objective_context()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::adaptive::provider::{Response, Suggestion, PROTOCOL};
    use std::sync::Mutex;
    fn setup() -> Result<(tempfile::TempDir, Engine)> {
        let d = tempfile::tempdir()?;
        let e = Engine::open(d.path())?;
        e.project_initialize(ProjectSpec {
            browser_container_only: false,
            browser: false,
            full_workspace_oracle: false,
            goal: "Authored generic finite-service fixture".into(),
            image: format!("sha256:{}", "a".repeat(64)),
            user: "501:20".into(),
            editable: BTreeMap::from([("lib.rs".into(), "pub fn value()->u32{1}\n".into())]),
            protected: BTreeMap::from([(
                "service.rs".into(),
                "include!(\"lib.rs\"); fn main(){println!(\"{}\",value());}\n".into(),
            )]),
            edit_windows: BTreeMap::new(),
            supporting_spans: BTreeMap::new(),
            cargo: false,
            named_package: None,
            probe_entry: "service.rs".into(),
            cases: vec![ProbeCase {
                id: "smoke".into(),
                args: vec![],
                expected_stdout: "1\n".into(),
            }],
            timeout_ms: 1000,
        })?;
        let requirements = BTreeMap::from([
            (
                "continuity".into(),
                "Keep the original continuity obligation".into(),
            ),
            (
                "discovery".into(),
                "Keep the original discovery obligation".into(),
            ),
        ]);
        e.project_acceptance_requirements(requirements.clone())?;
        e.project_lifecycle_open(lifecycle::Admission {
            state_root: e.store.head_root()?,
            training: None,
            tool_ms_ceiling: 100000,
            reason: "Authored fixture; no real worker or provider".into(),
            non_tool: Some(budget::Limits {
                model_calls: 40,
                model_tokens: 100000,
                wall_ms: 600000,
                patch_bytes: 10000,
                external_cost_microusd: 0,
            }),
        })?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: e
                .project_acceptance_status()?
                .requirements_revision
                .unwrap(),
            decision_record: None,
            guidance: requirements,
        })?;
        e.project_objective_configure(Admission {
            state_root: e.store.head_root()?,
            conditions: vec![
                Condition {
                    id: "first".into(),
                    input: "authored-first".into(),
                },
                Condition {
                    id: "second".into(),
                    input: "authored-second".into(),
                },
                Condition {
                    id: "third".into(),
                    input: "authored-third".into(),
                },
            ],
            service_entry: "service.rs".into(),
            max_output_bytes: 256,
            timeout_ms: 1000,
        })?;
        Ok((d, e))
    }
    fn prepare(e: &Engine) -> Result<Hash> {
        let c = e.project_objective_context()?;
        let choice = Choice {
            binding: serde_json::from_value(c["binding"].clone())?,
            rationale: "Authored independent test choice; no model behavior claim".into(),
            success_probability: 0.6,
        };
        let response = Response {
            protocol: PROTOCOL.into(),
            state_root: e.store.head_root()?,
            attention_binding: e.proposal_attention_binding()?,
            proposals: vec![Suggestion::Service { choice }],
            pursuit_expectations: Default::default(),
            common_event_forecasts: BTreeMap::new(),
        };
        let v = e.project_cycle_preview(response.clone())?;
        let binding = serde_json::from_value(v["cycle"]["binding"].clone())?;
        let next = e.project_cycle_advance(cycle::Advance {
            binding,
            review: cycle::Review {
                binding,
                response_digest: serde_json::from_value(v["response_digest"].clone())?,
                state_root: response.state_root,
                source_snapshot: serde_json::from_value(v["cycle"]["source_snapshot"].clone())?,
                reviewer: "authored-independent-review".into(),
                generator: "authored-proposal".into(),
                decision: "approve".into(),
            },
            response,
        })?;
        assert_eq!(next["next"]["branch"], "service_resume");
        Ok(serde_json::from_value(next["next"]["binding"].clone())?)
    }
    struct AuthoredRunner {
        seen: Mutex<Vec<Vec<String>>>,
        output: String,
        exit: i32,
        unknown: bool,
    }
    impl Runner for AuthoredRunner {
        fn run(&self, copy: &Path, args: &[String], _: &Limits) -> RunResult {
            self.seen.lock().unwrap().push(args.to_vec());
            let service = args[0] == "./fixture-probe";
            if args[0] == "sh" {
                for pair in args[4..].chunks_exact(2) {
                    fs::write(copy.join(&pair[0]), &pair[1]).unwrap();
                }
            }
            let mut result = RunResult::from_output(
                if service { self.exit } else { 0 },
                if service { self.output.as_bytes() } else { b"" },
                b"",
                1,
            );
            result.execution = Some(tsr_corpus::ExecutionDiagnostics {
                version: 1,
                termination: if self.unknown {
                    "wall_timeout"
                } else {
                    "runtime_exit"
                }
                .into(),
                runtime_exit: Some(result.exit),
                runtime_signal: None,
                stdout_complete: true,
                stderr_complete: true,
                worker: "authored-local-stub-no-target-execution".into(),
            });
            result
        }
    }
    fn runner(output: &str) -> AuthoredRunner {
        AuthoredRunner {
            seen: Mutex::new(vec![]),
            output: output.into(),
            exit: 0,
            unknown: false,
        }
    }
    fn review(e: &Engine, decision: &str) -> Result<Review> {
        let v = e.project_objective_context()?;
        Ok(Review {
            state_root: e.store.head_root()?,
            binding: serde_json::from_value(v["binding"].clone())?,
            receipt: serde_json::from_value(
                v["context"]["receipts"].as_array().unwrap().last().unwrap()["receipt"].clone(),
            )?,
            decision: decision.into(),
            reviewer: "authored-round-review".into(),
            generator: "authored-proposal".into(),
            evidence:
                "Authored runner observations and separately authored assessment; wiring only"
                    .into(),
        })
    }
    fn final_review(e: &Engine, decision: &str) -> Result<FinalReview> {
        Ok(FinalReview {
            state_root: e.store.head_root()?,
            binding: serde_json::from_value(e.project_objective_context()?["binding"].clone())?,
            decision: decision.into(),
            reviewer: "authored-final-review".into(),
            generator: "authored-proposal".into(),
            evidence: "Authored fixture final judgment, not live semantic evidence".into(),
        })
    }
    #[test]
    fn fixture_worker() -> Result<()> {
        let Some(path) = std::env::var_os("TSR_OBJECTIVE_FIXTURE_DIRECTORY") else {
            return Ok(());
        };
        let directory = PathBuf::from(path).canonicalize()?;
        let parent = directory.parent().context("fixture parent")?;
        ensure!(
            fs::read_to_string(parent.join("fixture-only.marker"))?
                == "objective-native-fixture-only@v1\n",
            "explicit objective fixture marker"
        );
        let output =
            PathBuf::from(std::env::var_os("TSR_OBJECTIVE_FIXTURE_OUTPUT").context("output")?);
        ensure!(
            output.parent().context("output parent")?.canonicalize()? == parent,
            "fixture output boundary"
        );
        let text = std::env::var("TSR_OBJECTIVE_FIXTURE_STDOUT")?;
        ensure!(text.len() <= 2048, "authored output bound");
        let engine = Engine::open(&directory)?;
        let v = engine.project_cycle_request()?;
        let r = runner(&text);
        let binding = serde_json::from_value(v["binding"].clone())?;
        let result = if v["branch"] == "resume" {
            let plan = engine.project_cycle_plan(binding)?;
            serde_json::to_value(engine.coding_execute(plan.commitment, plan.source_before, &r)?)?
        } else {
            engine.project_objective_execute(binding, &r)?
        };
        fs::write(
            output,
            serde_json::to_vec(
                &json!({"target_commands_executed":0,"authored_fixture":true,"result":result}),
            )?,
        )?;
        Ok(())
    }
    #[test]
    fn multiple_rounds_use_actual_output_and_restart_without_budget_reset() -> Result<()> {
        let (d, e) = setup()?;
        let requirements = e.project_acceptance_status()?.requirements_revision;
        assert!(e
            .project_objective_accept(final_review(&e, "accept")?)
            .is_err());
        let mut previous = "".to_string();
        for (index, output) in ["one\n", "two\n", "three\n"].iter().enumerate() {
            let e = Engine::open(d.path())?;
            let before = e.project_objective_context()?;
            assert_eq!(before["context"]["round"], index);
            assert_eq!(before["context"]["previous_output"], previous);
            assert!(!serde_json::to_string(&before)?.contains(if index == 0 {
                "authored-second"
            } else if index == 1 {
                "authored-third"
            } else {
                "not-a-condition"
            }));
            let b = prepare(&e)?;
            let r = runner(output);
            e.project_objective_execute(b, &r)?;
            assert_eq!(r.seen.lock().unwrap()[1][2], previous);
            let review = review(&e, "met")?;
            e.project_objective_review(review.clone())?;
            assert!(e.project_objective_review(review).is_err());
            assert!(e.project_objective_execute(b, &r).is_err());
            assert_eq!(r.seen.lock().unwrap().len(), 2);
            previous = output.to_string();
            assert_eq!(e.project_budget()?.tool_ms_charged, (index as u64 + 1) * 2);
            assert_eq!(
                e.project_acceptance_status()?.requirements_revision,
                requirements
            );
        }
        let e = Engine::open(d.path())?;
        assert_eq!(
            e.project_objective_context()?["context"]["next"],
            "objective_review"
        );
        e.project_objective_accept(final_review(&e, "accept")?)?;
        assert_eq!(e.project_cycle_request()?["branch"], "stop");
        assert_eq!(
            Engine::open(d.path())?.project_objective_context()?,
            e.project_objective_context()?
        );
        assert_eq!(e.project_budget()?.tool_ms_charged, 6);
        Ok(())
    }
    #[test]
    fn serialized_patch_smoke_keeps_pending_contributors_selectable() -> Result<()> {
        for pass in [true, false] {
            let (d, e) = setup()?;
            let spec = e.project_configuration()?;
            let snap = e.project_snapshot(&spec)?;
            let response = Response {
                protocol: PROTOCOL.into(),
                state_root: e.store.head_root()?,
                attention_binding: e.proposal_attention_binding()?,
                pursuit_expectations: Default::default(),
                common_event_forecasts: BTreeMap::new(),
                proposals: vec![Suggestion::ExactEdit {
                    patch: ExactPatch {
                        path: "lib.rs".into(),
                        source_digest: Hash::of(&[snap["lib.rs"].as_bytes()]),
                        inspection_handle: e.project_handle(&spec, &snap, "lib.rs")?,
                        old_text: snap["lib.rs"].clone(),
                        new_text: "pub fn value()->u32{2}\n".into(),
                    },
                    rationale: "Authored transition fixture".into(),
                }],
            };
            let response: Response = serde_json::from_slice(&serde_json::to_vec(&response)?)?;
            let v = e.project_cycle_preview(response.clone())?;
            let binding = serde_json::from_value(v["cycle"]["binding"].clone())?;
            let a = cycle::Advance {
                binding,
                review: cycle::Review {
                    binding,
                    response_digest: serde_json::from_value(v["response_digest"].clone())?,
                    state_root: response.state_root,
                    source_snapshot: serde_json::from_value(v["cycle"]["source_snapshot"].clone())?,
                    reviewer: "authored-review".into(),
                    generator: "authored-generator".into(),
                    decision: "approve".into(),
                },
                response,
            };
            let next =
                e.project_cycle_advance(serde_json::from_slice(&serde_json::to_vec(&a)?)?)?;
            let b = serde_json::from_value(next["next"]["binding"].clone())?;
            let plan = e.project_cycle_plan(b)?;
            let r = runner(if pass { "1\n" } else { "wrong\n" });
            let executed = e.coding_execute(plan.commitment, plan.source_before, &r)?;
            assert!(matches!(
                (&executed.verdict, pass),
                (Verdict::Passed, true) | (Verdict::FailedTests, false)
            ));
            let e = Engine::open(d.path())?;
            let before = e.store.head_root()?;
            let attention = e.project_attention()?;
            assert!(!attention["selected_preference"].is_null());
            if pass {
                assert!(e
                    .project_objective_active_parents()?
                    .contains(&serde_json::from_value(
                        attention["selected_preference"].clone()
                    )?));
            }
            assert_eq!(e.project_cycle_request()?["branch"], "propose");
            assert_eq!(before, e.store.head_root()?);
            assert!(e.project_cycle_plan(b).is_err());
            assert_eq!(e.project_objective_context()?["context"]["round"], 0);
            let b = prepare(&e)?;
            let service = runner("authored-return");
            e.project_objective_execute(b, &service)?;
            assert_eq!(
                e.project_cycle_request()?["branch"],
                "objective_review_required"
            );
            e.project_objective_review(review(&e, "met")?)?;
            assert_eq!(e.project_cycle_request()?["branch"], "propose");
        }
        Ok(())
    }
    #[test]
    fn known_failure_unknown_review_and_final_abstention_are_distinct() -> Result<()> {
        let (d, e) = setup()?;
        assert!(e
            .project_lifecycle_close_inconclusive("authored attempt".into())
            .is_err());
        assert!(e
            .project_lifecycle_close_advisory("authored attempt".into())
            .is_err());
        assert!(e.project_lifecycle_close().is_err());
        let b = prepare(&e)?;
        let mut r = runner("failed output");
        r.exit = 1;
        e.project_objective_execute(b, &r)?;
        assert_eq!(
            e.project_cycle_request()?["branch"],
            "objective_review_required"
        );
        assert!(e.project_objective_review(review(&e, "met")?).is_err());
        e.project_objective_review(review(&e, "not_met")?)?;
        assert_eq!(
            Engine::open(d.path())?.project_cycle_request()?["branch"],
            "propose"
        );
        let b = prepare(&e)?;
        e.project_objective_execute(b, &runner("unknown interpretation"))?;
        e.project_objective_review(review(&e, "unknown")?)?;
        assert_eq!(
            Engine::open(d.path())?.project_cycle_request()?["branch"],
            "objective_review_required"
        );
        assert!(prepare(&e).is_err());
        let (_d, e) = setup()?;
        for _ in 0..3 {
            let b = prepare(&e)?;
            e.project_objective_execute(b, &runner("known"))?;
            e.project_objective_review(review(&e, "met")?)?;
        }
        e.project_objective_accept(final_review(&e, "abstain")?)?;
        assert_eq!(e.project_cycle_request()?["branch"], "stop");
        assert!(e.project_objective_active_parents()?.is_empty());
        Ok(())
    }
    #[test]
    fn changed_state_during_worker_blocks_remaining_invocation() -> Result<()> {
        let (_d, e) = setup()?;
        let b = prepare(&e)?;
        struct Drift<'a> {
            engine: &'a Engine,
            seen: Mutex<usize>,
        }
        impl Runner for Drift<'_> {
            fn run(&self, _: &Path, _: &[String], _: &Limits) -> RunResult {
                *self.seen.lock().unwrap() += 1;
                self.engine
                    .append(
                        EventKind::Observed {
                            observation: Observation::Evidence {
                                digest: Hash::of(&[b"authored concurrent state change"]),
                                source: "authored fixture".into(),
                            },
                        },
                        Source::Operator,
                    )
                    .unwrap();
                runner("").run(
                    Path::new("."),
                    &["rustc".into()],
                    &Limits {
                        wall_clock_ms: 1000,
                        max_output_bytes: 256,
                    },
                )
            }
        }
        let r = Drift {
            engine: &e,
            seen: Mutex::new(0),
        };
        assert!(e.project_objective_execute(b, &r).is_err());
        assert_eq!(*r.seen.lock().unwrap(), 1);
        assert_eq!(e.project_cycle_request()?["branch"], "reconcile");
        assert!(e.project_objective_execute(b, &r).is_err());
        assert_eq!(*r.seen.lock().unwrap(), 1);
        Ok(())
    }
    #[test]
    fn stale_cross_root_wrong_receipt_and_immutable_criteria() -> Result<()> {
        let (_d, e) = setup()?;
        let (_other, foreign) = setup()?;
        let v = e.project_objective_context()?;
        let c = Choice {
            binding: serde_json::from_value(v["binding"].clone())?,
            rationale: "authored".into(),
            success_probability: 0.5,
        };
        assert!(foreign.validate_project_service(&c).is_err());
        assert!(e
            .project_acceptance_requirements(BTreeMap::from([("other".into(), "changed".into())]))
            .is_err());
        let b = prepare(&e)?;
        let r = runner("one");
        e.project_objective_execute(b, &r)?;
        let review = review(&e, "met")?;
        let mut wrong = review.clone();
        wrong.receipt = EventId(Hash::of(&[b"foreign"]));
        assert!(e.project_objective_review(wrong).is_err());
        let mut wrong = review.clone();
        wrong.state_root = Hash::of(&[b"stale"]);
        assert!(e.project_objective_review(wrong).is_err());
        e.project_objective_review(review)?;
        assert!(e.validate_project_service(&c).is_err());
        Ok(())
    }
    #[test]
    fn unknown_effect_survives_restart_and_no_reexecution() -> Result<()> {
        let (d, e) = setup()?;
        let b = prepare(&e)?;
        let mut r = runner("?");
        r.unknown = true;
        assert!(e.project_objective_execute(b, &r).is_err());
        let reopened = Engine::open(d.path())?;
        assert_eq!(reopened.project_cycle_request()?["branch"], "reconcile");
        assert!(reopened.project_objective_execute(b, &r).is_err());
        assert_eq!(r.seen.lock().unwrap().len(), 1);
        assert_eq!(reopened.project_budget()?.unresolved_tool_requests, 1);
        Ok(())
    }
    #[test]
    fn reviewed_joint_relation_preserves_contributors_and_requires_fresh_choice() -> Result<()> {
        let (_d, e) = setup()?;
        let original = e.project_relational_context()?["inputs"]["parents"].clone();
        let requirements = e.project_acceptance_status()?.requirements_revision;
        prepare(&e)?;
        let c = e.project_relational_context()?;
        let proposal = crate::engine::deliberation::relational::Proposal {
            parents: original
                .as_array()
                .unwrap()
                .iter()
                .map(|p| serde_json::from_value(p["id"].clone()).unwrap())
                .collect(),
            relationship: "Authored joint means can preserve both original contributor meanings"
                .into(),
            aspiration: "Retain both original wanted conditions".into(),
            discriminator: "Actual finite outcomes, not a prescribed semantic result".into(),
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
                    binding: serde_json::from_value(c["binding"].clone())?,
                    proposal,
                }],
                pursuit_expectations: Default::default(),
                common_event_forecasts: Default::default(),
            },
            true,
        )?;
        let record = e.project_latest_proposal()?.unwrap();
        e.project_relational_admit(e.store.head_root()?, record, 0)?;
        assert_eq!(
            e.project_relational_context()?["inputs"]["parents"],
            original
        );
        assert_eq!(e.project_objective_active_parents()?.len(), 2);
        let r = runner("not-called");
        let binding = serde_json::from_value(e.project_cycle_request()?["binding"].clone())?;
        assert!(e.project_objective_execute(binding, &r).is_err());
        assert!(r.seen.lock().unwrap().is_empty());
        e.project_objective_retire_stale(e.store.head_root()?)?;
        let b = prepare(&e)?;
        e.project_objective_execute(b, &runner("one"))?;
        assert_eq!(
            e.project_acceptance_status()?.requirements_revision,
            requirements
        );
        Ok(())
    }
    #[test]
    fn changed_guidance_requires_fresh_review_without_erasing_history() -> Result<()> {
        let (_d, e) = setup()?;
        prepare(&e)?;
        let parents = e.project_objective_active_parents()?;
        let id = *parents.iter().next().unwrap();
        e.project_desire_status(desire::Status {
            state_root: e.store.head_root()?,
            identity: id,
            revision: None,
            withdrawn: true,
        })?;
        let fresh = e.project_cycle_request()?;
        let r = runner("must-not-run");
        assert!(e
            .project_objective_execute(serde_json::from_value(fresh["binding"].clone())?, &r)
            .is_err());
        assert!(r.seen.lock().unwrap().is_empty());
        assert_eq!(e.project_budget()?.tool_ms_charged, 0);
        e.project_objective_retire_stale(e.store.head_root()?)?;
        assert_eq!(e.project_objective_context()?["context"]["round"], 0);
        assert!(!e.project_objective_pending()?);
        assert!(e
            .objective_entries()?
            .iter()
            .any(|(_, x)| matches!(x, Entry::Prepared { .. })));
        Ok(())
    }
    #[test]
    fn known_failed_output_is_preserved_and_never_accepted_or_rewritten() -> Result<()> {
        let (_d, e) = setup()?;
        let b = prepare(&e)?;
        let mut r = runner("actual-invalid-output\n");
        r.exit = 1;
        e.project_objective_execute(b, &r)?;
        assert!(e.project_objective_review(review(&e, "met")?).is_err());
        e.project_objective_review(review(&e, "not_met")?)?;
        assert_eq!(
            e.project_objective_context()?["context"]["previous_output"],
            "actual-invalid-output\n"
        );
        assert!(e
            .project_objective_accept(final_review(&e, "accept")?)
            .is_err());
        e.project_objective_accept(final_review(&e, "abstain")?)?;
        assert_eq!(e.project_cycle_request()?["branch"], "stop");
        Ok(())
    }
}
