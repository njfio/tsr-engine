//! Reviewed public meaning, separate from task criteria and execution authority.
use super::*;
use crate::engine::adaptive::provider::Suggestion;
use anyhow::Context;
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Change {
    Clarification,
    Development,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Basis {
    Unknown,
    Observation { receipt: EventId },
    Judgment { review: EventId },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Proposal {
    pub binding: Hash,
    pub identity: PreferenceId,
    pub revises: Option<EventId>,
    pub parents: Vec<PreferenceId>,
    pub meaning: String,
    pub change: Change,
    pub basis: Basis,
    pub generator: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub state_root: Hash,
    pub proposal: EventId,
    pub index: usize,
    pub reviewer: String,
    pub note: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Status {
    pub state_root: Hash,
    pub identity: PreferenceId,
    pub revision: Option<EventId>,
    pub withdrawn: bool,
}
fn origin(p: &Value) -> Value {
    let mut result = p.clone();
    let original = p.get("original_meaning").unwrap_or(&p["meaning"]).clone();
    result["meaning"] = original.clone();
    result["original_meaning"] = original;
    result["revision"] = Value::Null;
    result["withdrawn"] = json!(false);
    if p.get("original_contributors").is_some() {
        result["preference_parents"] = p.get("preference_parents").unwrap_or(&p["parents"]).clone();
        result["generated_origin"] = p.get("generated_origin").unwrap_or(p).clone();
    }
    result["parents"] = if p.get("original_contributors").is_some() {
        result["preference_parents"].clone()
    } else {
        json!([])
    };
    result
}
impl Engine {
    pub(crate) fn project_public_desires(&self, base: &[Value]) -> Result<Option<Value>> {
        let mut current: BTreeMap<PreferenceId, Value> = base
            .iter()
            .map(|p| Ok((serde_json::from_value(p["id"].clone())?, origin(p))))
            .collect::<Result<_>>()?;
        let mut history = vec![];
        for (id, r) in self.records()? {
            match r {
                Record::Coding {
                    record:
                        CodingRecord::PublicDesireAdmission {
                            proposal,
                            index,
                            reviewer,
                            note,
                            original_parent,
                            parent_origins,
                        },
                } => {
                    self.require_project_bridge_source(id, Source::Operator)?;
                    let Suggestion::PublicDesire { proposal: p } =
                        self.project_relational_proposal(proposal, index)?
                    else {
                        anyhow::bail!("public desire origin kind");
                    };
                    let retained_origins = parent_origins.clone();
                    for parent in parent_origins {
                        let identity: PreferenceId = serde_json::from_value(parent["id"].clone())?;
                        current.entry(identity).or_insert(parent);
                    }
                    if p.revises.is_none() {
                        current.insert(p.identity, original_parent.clone());
                    }
                    let old = current.entry(p.identity).or_insert(original_parent.clone());
                    ensure!(
                        old["revision"] == json!(p.revises),
                        "broken public desire lineage"
                    );
                    let mut next = old.clone();
                    next.as_object_mut().unwrap().extend(json!({"id":p.identity,"meaning":p.meaning,"original_meaning":original_parent["original_meaning"],"requirement":old["requirement"],"revision":id,"parents":p.parents,"withdrawn":false,"generator":p.generator,"reviewer":reviewer,"proposal":proposal,"change":p.change,"basis":p.basis}).as_object().unwrap().clone());
                    history.push(json!({"revision":id,"previous":old,"next":next,"parent_origins":retained_origins,"review_note_digest":Hash::of(&[note.as_bytes()])}));
                    current.insert(p.identity, next);
                }
                Record::Coding {
                    record:
                        CodingRecord::PublicDesireStatus {
                            update,
                            original_parent,
                        },
                } => {
                    self.require_project_bridge_source(id, Source::Operator)?;
                    let old = current.entry(update.identity).or_insert(original_parent);
                    ensure!(
                        old["revision"] == json!(update.revision),
                        "public desire status stale history"
                    );
                    old["withdrawn"] = json!(update.withdrawn);
                    history.push(json!({"status_record":id,"identity":update.identity,"revision":update.revision,"withdrawn":update.withdrawn}));
                }
                _ => {}
            }
        }
        if history.is_empty() {
            return Ok(None);
        }
        // Least fixed point of ineligibility: a withdrawn ancestor removes all descendants.
        let base_ids: BTreeSet<PreferenceId> = base
            .iter()
            .map(|p| serde_json::from_value(p["id"].clone()))
            .collect::<std::result::Result<_, _>>()?;
        // Publishing later guidance does not delete independently retained root
        // wanting. Generated identities, however, still require current native
        // eligibility; historical overlay records cannot resurrect them.
        let retained_roots: BTreeSet<_> = Ledger::new(&self.store)
            .collective()?
            .entries
            .iter()
            .filter(|e| {
                e.preference.origin == Source::Operator
                    && e.preference.scope == Scope::Task(Self::task())
                    && matches!(
                        e.status,
                        Some(PreferenceStatus::Open | PreferenceStatus::Fulfilled)
                    )
            })
            .map(|e| e.preference.id)
            .collect();
        let mut active_ids: BTreeSet<_> = current
            .iter()
            .filter(|(id, p)| {
                (if p.get("original_contributors").is_some() {
                    base_ids.contains(id)
                } else {
                    retained_roots.contains(id)
                }) && p["withdrawn"] != true
            })
            .map(|(id, _)| *id)
            .collect();
        loop {
            let remove: Vec<_> = active_ids
                .iter()
                .filter(|id| {
                    current[id]["parents"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .any(|p| {
                            serde_json::from_value::<PreferenceId>(p.clone())
                                .map_or(true, |parent| !active_ids.contains(&parent))
                        })
                })
                .copied()
                .collect();
            if remove.is_empty() {
                break;
            }
            for id in remove {
                active_ids.remove(&id);
            }
        }
        let active: Vec<_> = current
            .iter()
            .filter(|(id, _)| active_ids.contains(id))
            .map(|(_, p)| p.clone())
            .collect();
        for entry in &mut history {
            if entry.get("next").is_some() {
                let latest = current
                    .values()
                    .find(|p| p["id"] == entry["next"]["id"])
                    .unwrap();
                entry["superseded"] = json!(entry["revision"] != latest["revision"]);
                entry["withdrawn"] = latest["withdrawn"].clone();
            }
        }
        Ok(Some(
            json!({"version":"public-meaning-history@v2","active":active,"current":current.values().collect::<Vec<_>>(),"history":history,"authority":"qualitative public meaning only; immutable acceptance obligations remain independently assessed"}),
        ))
    }
    pub(crate) fn validate_public_desire(&self, p: &Proposal) -> Result<()> {
        let ctx = self.project_relational_context()?;
        ensure!(
            ctx["binding"] == json!(p.binding),
            "stale public desire binding"
        );
        let browser_profile = self.project_configuration()?.browser;
        ensure!(
            (if browser_profile {
                browser::narrative(&p.meaning)
            } else {
                short(&p.meaning)
            }) && short(&p.generator),
            "bounded public desire text/attribution"
        );
        let parents = ctx["inputs"]["parents"]
            .as_array()
            .context("public parents")?;
        let old = parents
            .iter()
            .find(|x| x["id"] == json!(p.identity))
            .context("inactive/unknown public desire")?;
        ensure!(
            old.get("revision").unwrap_or(&Value::Null) == &json!(p.revises),
            "stale public desire revision"
        );
        ensure!(
            old["meaning"].as_str() != Some(p.meaning.trim()),
            "unchanged meaning is not development"
        );
        let unique: BTreeSet<_> = p.parents.iter().copied().collect();
        ensure!(
            !unique.is_empty()
                && unique.len() == p.parents.len()
                && unique.len() <= if browser_profile { parents.len() } else { 8 }
                && unique.contains(&p.identity),
            "public desire ancestry must include original identity"
        );
        ensure!(
            unique
                .iter()
                .all(|id| parents.iter().any(|x| x["id"] == json!(id))),
            "inactive/unknown relationship parent"
        );
        if p.revises.is_none() && old.get("original_contributors").is_some() {
            let ancestry: Vec<PreferenceId> = serde_json::from_value(
                old.get("preference_parents")
                    .unwrap_or(&old["parents"])
                    .clone(),
            )?;
            ensure!(
                ancestry.iter().all(|id| unique.contains(id)),
                "derived meaning must retain immutable contributing ancestry"
            );
        }
        if p.revises.is_some() {
            let prior: BTreeSet<PreferenceId> = serde_json::from_value(old["parents"].clone())?;
            ensure!(
                prior == unique,
                "revision cannot replace parent relationships"
            );
        }
        match p.basis {
            Basis::Unknown => {}
            Basis::Observation { receipt } => ensure!(
                ctx["inputs"]["returns"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .any(|r| r["receipt"] == json!(receipt)),
                "unknown/stale observation receipt"
            ),
            Basis::Judgment { review } => ensure!(
                ctx["inputs"]["semantic_return"]["applicable"] == true
                    && ctx["inputs"]["semantic_return"]["review"] == json!(review),
                "unknown/stale attributed judgment"
            ),
        }
        ensure!(
            browser_profile
                || ctx["inputs"]["public_desires"]["history"]
                    .as_array()
                    .map_or(0, Vec::len)
                    < 64,
            "public desire history capacity"
        );
        Ok(())
    }
    pub fn project_desire_admit(&self, r: Review) -> Result<EventId> {
        self.write(|| self.project_desire_admit_unlocked(r))
    }
    pub(crate) fn project_desire_admit_unlocked(&self, r: Review) -> Result<EventId> {
        ensure!(
            r.state_root == self.store.head_root()?,
            "stale public desire review"
        );
        ensure!(
            self.adaptive_state_inner()?.pending.is_none(),
            "pending action prevents desire admission"
        );
        let Suggestion::PublicDesire { proposal: p } =
            self.project_relational_proposal(r.proposal, r.index)?
        else {
            anyhow::bail!("wrong public desire proposal kind")
        };
        self.validate_public_desire(&p)?;
        ensure!(
            short(&r.reviewer) && short(&r.note) && r.reviewer != p.generator,
            "distinct review attribution required"
        );
        let ctx = self.project_relational_context()?;
        let old = ctx["inputs"]["parents"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["id"] == json!(p.identity))
            .unwrap();
        let original_parent = origin(old);
        let parent_origins = ctx["inputs"]["parents"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|x| p.parents.iter().any(|id| x["id"] == json!(id)))
            .map(origin)
            .collect();
        self.adaptive_record(
            &Record::Coding {
                record: CodingRecord::PublicDesireAdmission {
                    proposal: r.proposal,
                    index: r.index,
                    reviewer: r.reviewer,
                    note: r.note,
                    original_parent,
                    parent_origins,
                },
            },
            Source::Operator,
        )
    }
    pub fn project_desire_status(&self, u: Status) -> Result<()> {
        self.write(|| {
            ensure!(
                u.state_root == self.store.head_root()?,
                "stale public desire status"
            );
            ensure!(
                self.adaptive_state_inner()?.pending.is_none(),
                "pending action prevents desire status"
            );
            let ctx = self.project_relational_context()?;
            let current = ctx["inputs"]["public_desires"]
                .get("current")
                .unwrap_or(&ctx["inputs"]["parents"])
                .as_array()
                .context("public desires")?;
            let p = current
                .iter()
                .find(|p| p["id"] == json!(u.identity))
                .context("unknown public desire")?;
            ensure!(
                p["revision"] == json!(u.revision),
                "superseded public desire revision"
            );
            if p["withdrawn"].as_bool().unwrap_or(false) != u.withdrawn {
                self.adaptive_record(
                    &Record::Coding {
                        record: CodingRecord::PublicDesireStatus {
                            original_parent: origin(p),
                            update: u,
                        },
                    },
                    Source::Operator,
                )?;
            }
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::adaptive::provider::{Response, PROTOCOL};
    fn setup() -> Result<(tempfile::TempDir, Engine)> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/engine-invoice/project.json"
        )))?)?;
        e.project_acceptance_requirements(BTreeMap::from([
            (
                "correctness".into(),
                "Preserve the admitted invoice behavior".into(),
            ),
            (
                "preservation".into(),
                "Retain protected boundary behavior".into(),
            ),
        ]))?;
        e.project_lifecycle_open(lifecycle::Admission {
            state_root: e.store.head_root()?,
            training: None,
            tool_ms_ceiling: 100_000,
            reason: "Authored offline cycle fixture".into(),
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
            decision_record: None,
            guidance: BTreeMap::from([
                (
                    "correctness".into(),
                    "Preserve the admitted invoice behavior".into(),
                ),
                (
                    "preservation".into(),
                    "Retain protected boundary behavior".into(),
                ),
            ]),
        })?;
        Ok((dir, e))
    }
    fn offer(e: &Engine, text: &str, change: Change) -> Result<Proposal> {
        let c = e.project_relational_context()?;
        let parents = c["inputs"]["parents"].as_array().unwrap();
        Ok(Proposal {
            binding: serde_json::from_value(c["binding"].clone())?,
            identity: serde_json::from_value(parents[0]["id"].clone())?,
            revises: serde_json::from_value(parents[0]["revision"].clone())?,
            parents: parents
                .iter()
                .map(|p| serde_json::from_value(p["id"].clone()).unwrap())
                .collect(),
            meaning: text.into(),
            change,
            basis: Basis::Unknown,
            generator: "authored-fixture-generator".into(),
        })
    }
    fn submit(e: &Engine, p: Proposal) -> Result<EventId> {
        let r = Response {
            protocol: PROTOCOL.into(),
            state_root: e.store.head_root()?,
            attention_binding: e.proposal_attention_binding()?,
            proposals: vec![Suggestion::PublicDesire { proposal: p }],
            pursuit_expectations: Default::default(),
            common_event_forecasts: Default::default(),
        };
        let preview = e.project_cycle_preview(r.clone())?;
        let cy = &preview["cycle"];
        let result = e.project_cycle_advance(cycle::Advance {
            binding: serde_json::from_value(cy["binding"].clone())?,
            review: cycle::Review {
                binding: serde_json::from_value(cy["binding"].clone())?,
                response_digest: serde_json::from_value(preview["response_digest"].clone())?,
                state_root: r.state_root,
                source_snapshot: serde_json::from_value(cy["source_snapshot"].clone())?,
                reviewer: "authored-shape-review".into(),
                generator: "authored-fixture-generator".into(),
                decision: "approve".into(),
            },
            response: r,
        })?;
        assert_eq!(result["review_required"], true);
        assert_eq!(result["evaluation"]["committed"], false);
        Ok(serde_json::from_value(result["proposal_record"].clone())?)
    }
    fn review(e: &Engine, id: EventId) -> Result<EventId> {
        e.project_desire_admit(Review {
            state_root: e.store.head_root()?,
            proposal: id,
            index: 0,
            reviewer: "independent-authored-fixture-review".into(),
            note: "Private test review text must not be projected".into(),
        })
    }
    #[test]
    fn native_meaning_development_keeps_acceptance_and_restart_history() -> Result<()> {
        let (dir, e) = setup()?;
        let before = e.project_relational_context()?;
        let config = serde_json::to_value(e.project_configuration()?)?;
        let acceptance = serde_json::to_value(e.project_acceptance_status()?)?;
        let p=offer(&e,"Authored fixture: understanding how boundary behavior supports dependable use, beyond a passing check",Change::Development)?;
        let id = submit(&e, p.clone())?;
        assert_eq!(e.project_relational_context()?["inputs"], before["inputs"]);
        let admitted = review(&e, id)?;
        let current = e.project_relational_context()?;
        assert_eq!(current["inputs"]["version"], "project-relational@v3");
        assert_eq!(
            current["inputs"]["acceptance_parents"],
            before["inputs"]["parents"]
        );
        assert_eq!(current["inputs"]["parents"][0]["meaning"], p.meaning);
        assert_eq!(
            current["inputs"]["parents"][0]["original_meaning"],
            before["inputs"]["parents"][0]["meaning"]
        );
        assert_eq!(current["inputs"]["parents"][0]["revision"], json!(admitted));
        assert!(!current.to_string().contains("Private test review text"));
        assert_eq!(config, serde_json::to_value(e.project_configuration()?)?);
        let after = serde_json::to_value(e.project_acceptance_status()?)?;
        for key in [
            "snapshot",
            "requirements_revision",
            "requirements",
            "decision_record",
            "accepted",
        ] {
            assert_eq!(acceptance[key], after[key], "acceptance field {key}");
        }
        let request = e.project_cycle_request()?;
        assert_eq!(
            request["request"]["coding"]["relational"]["inputs"],
            current["inputs"]
        );
        assert!(e.adaptive_state()?.pending.is_none());
        drop(e);
        let reopened = Engine::open(dir.path())?;
        assert_eq!(current, reopened.project_relational_context()?);
        Ok(())
    }
    #[test]
    fn clarification_and_unknown_do_not_claim_observed_change_or_emergence() -> Result<()> {
        let (_d, e) = setup()?;
        let p = offer(
            &e,
            "Authored paraphrase: keep the agreed invoice behavior intact",
            Change::Clarification,
        )?;
        let id = submit(&e, p)?;
        review(&e, id)?;
        let ctx = e.project_relational_context()?;
        assert_eq!(ctx["inputs"]["parents"][0]["change"], "clarification");
        assert_eq!(
            ctx["inputs"]["parents"][0]["basis"],
            json!({"kind":"unknown"})
        );
        assert_eq!(ctx["inputs"]["returns"], json!([]));
        assert_eq!(ctx["guidance"], json!([]));
        let root = e.store.head_root()?;
        assert_eq!(ctx, e.project_relational_context()?);
        assert_eq!(root, e.store.head_root()?);
        let mut same = offer(&e, "placeholder", Change::Development)?;
        same.meaning = ctx["inputs"]["parents"][0]["meaning"]
            .as_str()
            .unwrap()
            .into();
        assert!(e.validate_public_desire(&same).is_err());
        Ok(())
    }
    #[test]
    fn rejects_stale_lineage_forged_authority_and_unknown_as_observation() -> Result<()> {
        let (_d, e) = setup()?;
        let original = offer(&e, "Authored candidate meaning", Change::Development)?;
        let mut invalid = original.clone();
        invalid.revises = Some(EventId(Hash::of(&[b"fake"])));
        assert!(e.validate_public_desire(&invalid).is_err());
        invalid = original.clone();
        invalid.meaning = "x".repeat(513);
        assert!(e.validate_public_desire(&invalid).is_err());
        invalid = original.clone();
        invalid.basis = Basis::Observation {
            receipt: EventId(Hash::of(&[b"fake"])),
        };
        assert!(e.validate_public_desire(&invalid).is_err());
        invalid = original.clone();
        invalid.basis = Basis::Judgment {
            review: EventId(Hash::of(&[b"fake"])),
        };
        assert!(e.validate_public_desire(&invalid).is_err());
        let mut forged = serde_json::to_value(&original)?;
        forged["execution_authorized"] = json!(true);
        assert!(serde_json::from_value::<Proposal>(forged).is_err());
        let id = submit(&e, original.clone())?;
        let bad = Review {
            state_root: e.store.head_root()?,
            proposal: id,
            index: 0,
            reviewer: original.generator.clone(),
            note: "self review".into(),
        };
        assert!(e.project_desire_admit(bad).is_err());
        review(&e, id)?;
        assert!(e.validate_public_desire(&original).is_err());
        assert!(review(&e, id).is_err());
        Ok(())
    }
    #[test]
    fn withdrawal_keeps_immutable_obligations_and_removes_relational_guidance() -> Result<()> {
        let (_d, e) = setup()?;
        let p = offer(&e, "Authored evolving public meaning", Change::Development)?;
        let identity = p.identity;
        let id = submit(&e, p)?;
        let revision = review(&e, id)?;
        let c = e.project_relational_context()?;
        let relation = crate::engine::deliberation::relational::Proposal {
            parents: c["inputs"]["parents"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| serde_json::from_value(p["id"].clone()).unwrap())
                .collect(),
            relationship: "Authored relation over reviewed public meanings".into(),
            aspiration: "Authored possibility, not generated emergence".into(),
            discriminator: "Unmeasured fixture discriminator".into(),
            revises: None,
            catalysts: vec![],
            development: None,
        };
        let response = Response {
            protocol: PROTOCOL.into(),
            state_root: e.store.head_root()?,
            attention_binding: e.proposal_attention_binding()?,
            proposals: vec![Suggestion::Relational {
                binding: serde_json::from_value(c["binding"].clone())?,
                proposal: relation,
            }],
            pursuit_expectations: Default::default(),
            common_event_forecasts: Default::default(),
        };
        e.evaluate_proposal(response, true)?;
        let proposal = e.project_latest_proposal()?.unwrap();
        e.project_relational_admit(e.store.head_root()?, proposal, 0)?;
        let before = e.project_relational_context()?;
        assert_eq!(before["guidance"].as_array().unwrap().len(), 1);
        e.project_desire_status(Status {
            state_root: e.store.head_root()?,
            identity,
            revision: Some(revision),
            withdrawn: true,
        })?;
        let after = e.project_relational_context()?;
        assert_eq!(after["guidance"], json!([]));
        assert_eq!(
            after["history"][0]["proposal"],
            before["history"][0]["proposal"]
        );
        assert!(!after["inputs"]["parents"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["id"] == json!(identity)));
        assert_eq!(
            before["inputs"]["acceptance_parents"],
            after["inputs"]["acceptance_parents"]
        );
        assert_eq!(
            after["inputs"]["public_desires"]["history"][0]["revision"],
            json!(revision)
        );
        let root = e.store.head_root()?;
        e.project_desire_status(Status {
            state_root: root,
            identity,
            revision: Some(revision),
            withdrawn: true,
        })?;
        assert_eq!(root, e.store.head_root()?);
        e.project_desire_status(Status {
            state_root: root,
            identity,
            revision: Some(revision),
            withdrawn: false,
        })?;
        assert_eq!(
            e.project_relational_context()?["inputs"]["parents"],
            before["inputs"]["parents"]
        );
        Ok(())
    }
    #[test]
    fn publication_changes_do_not_erase_reviewed_origins() -> Result<()> {
        let (_d, e) = setup()?;
        let p = offer(
            &e,
            "Authored independent public meaning",
            Change::Development,
        )?;
        let identity = p.identity;
        let id = submit(&e, p)?;
        review(&e, id)?;
        let before = e.project_relational_context()?;
        let status = e.project_acceptance_status()?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: status.requirements_revision.unwrap(),
            decision_record: None,
            guidance: BTreeMap::from([(
                "preservation".into(),
                "Only this criterion is republished".into(),
            )]),
        })?;
        let after = e.project_relational_context()?;
        let original = before["inputs"]["public_desires"]["history"][0].clone();
        assert_eq!(after["inputs"]["public_desires"]["history"][0], original);
        assert!(after["inputs"]["parents"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["id"] == json!(identity)
                && p["meaning"] == "Authored independent public meaning"));
        Ok(())
    }
    #[test]
    fn retained_undeveloped_parent_has_owned_origin_after_republication() -> Result<()> {
        let (dir, e) = setup()?;
        let proposal = offer(
            &e,
            "Authored A develops with undeveloped B",
            Change::Development,
        )?;
        let ids = proposal.parents.clone();
        assert!(ids.len() >= 2);
        let id = submit(&e, proposal)?;
        review(&e, id)?;
        let before = e.project_relational_context()?;
        e.project_acceptance_requirements(BTreeMap::from([(
            "new".into(),
            "New separate obligation".into(),
        )]))?;
        let status = e.project_acceptance_status()?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: status.requirements_revision.unwrap(),
            decision_record: None,
            guidance: BTreeMap::from([("new".into(), "New separate obligation".into())]),
        })?;
        let after = e.project_relational_context()?;
        let origins = after["inputs"]["public_desires"]["history"][0]["parent_origins"]
            .as_array()
            .unwrap();
        for identity in &ids {
            assert!(origins.iter().any(|p| p["id"] == json!(identity)));
            assert!(after["inputs"]["parents"]
                .as_array()
                .unwrap()
                .iter()
                .any(|p| p["id"] == json!(identity)));
        }
        assert_eq!(
            before["inputs"]["public_desires"]["history"],
            after["inputs"]["public_desires"]["history"]
        );
        assert_eq!(
            after["inputs"]["acceptance_parents"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        drop(e);
        let e = Engine::open(dir.path())?;
        assert_eq!(after, e.project_relational_context()?);
        e.project_desire_status(Status {
            state_root: e.store.head_root()?,
            identity: ids[1],
            revision: None,
            withdrawn: true,
        })?;
        let stopped = e.project_relational_context()?;
        assert!(!stopped["inputs"]["parents"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| ids.iter().any(|id| p["id"] == json!(id))));
        assert_eq!(
            stopped["inputs"]["public_desires"]["history"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        Ok(())
    }
    #[test]
    fn republication_preserves_explicit_exclusion_and_private_text_boundary() -> Result<()> {
        let (dir, e) = setup()?;
        let old = e.acceptance_current_aims()?;
        for id in old.values() {
            e.append(
                EventKind::StatusSet {
                    preference: *id,
                    status: PreferenceStatus::OutOfScope,
                    by: Source::Operator,
                },
                Source::Operator,
            )?;
        }
        let records = e.records()?;
        let requirements = records
            .iter()
            .rev()
            .find_map(|(_, r)| match r {
                Record::Coding {
                    record: CodingRecord::AcceptanceRequirements { requirements },
                } => Some(requirements.clone()),
                _ => None,
            })
            .unwrap();
        e.project_acceptance_requirements(requirements.clone())?;
        let status = e.project_acceptance_status()?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: status.requirements_revision.unwrap(),
            decision_record: None,
            guidance: requirements,
        })?;
        assert_eq!(e.acceptance_current_aims()?, old);
        assert_eq!(
            e.project_relational_context()?["inputs"]["parents"],
            json!([])
        );
        e.project_acceptance_requirements(BTreeMap::from([
            (
                "secret".into(),
                "Authored unpublished private text sentinel".into(),
            ),
            ("visible".into(), "Published nonprivate criterion".into()),
        ]))?;
        let status = e.project_acceptance_status()?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: status.requirements_revision.unwrap(),
            decision_record: None,
            guidance: BTreeMap::from([("visible".into(), "Published nonprivate criterion".into())]),
        })?;
        let context = e.project_relational_context()?;
        assert!(!serde_json::to_string(&context)?
            .contains("Authored unpublished private text sentinel"));
        drop(e);
        assert_eq!(
            Engine::open(dir.path())?.project_relational_context()?,
            context
        );
        Ok(())
    }
    #[test]
    fn transitive_withdrawal_and_legacy_status_preserve_history() -> Result<()> {
        let (_d, e) = setup()?;
        e.project_acceptance_requirements(BTreeMap::from([
            ("a".into(), "Original A".into()),
            ("b".into(), "Original B".into()),
            ("c".into(), "Original C".into()),
        ]))?;
        let status = e.project_acceptance_status()?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: status.requirements_revision.unwrap(),
            decision_record: None,
            guidance: BTreeMap::from([
                ("a".into(), "Original A".into()),
                ("b".into(), "Original B".into()),
                ("c".into(), "Original C".into()),
            ]),
        })?;
        let ctx = e.project_relational_context()?;
        let ids: Vec<PreferenceId> = ctx["inputs"]["parents"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| serde_json::from_value(p["id"].clone()).unwrap())
            .collect();
        let mut a = offer(&e, "Authored A informed by B", Change::Development)?;
        a.identity = ids[0];
        a.parents = vec![ids[0], ids[1]];
        let id = submit(&e, a)?;
        review(&e, id)?;
        let mut b = offer(&e, "Authored B informed by C", Change::Development)?;
        b.identity = ids[1];
        b.revises = None;
        b.parents = vec![ids[1], ids[2]];
        let id = submit(&e, b)?;
        review(&e, id)?;
        e.project_desire_status(Status {
            state_root: e.store.head_root()?,
            identity: ids[2],
            revision: None,
            withdrawn: true,
        })?;
        let after = e.project_relational_context()?;
        assert_eq!(after["inputs"]["parents"], json!([]));
        assert_eq!(
            after["inputs"]["acceptance_parents"]
                .as_array()
                .unwrap()
                .len(),
            3
        );
        assert_eq!(
            after["inputs"]["public_desires"]["current"]
                .as_array()
                .unwrap()
                .len(),
            3
        );
        Ok(())
    }
    #[test]
    fn requirements_revision_after_admission_preserves_public_history_on_restart() -> Result<()> {
        let (dir, e) = setup()?;
        let p = offer(
            &e,
            "Authored reviewed meaning survives task requirement changes",
            Change::Development,
        )?;
        let identity = p.identity;
        let id = submit(&e, p)?;
        review(&e, id)?;
        let before = e.project_relational_context()?;
        e.project_acceptance_requirements(BTreeMap::from([(
            "new-obligation".into(),
            "New explicitly supplied acceptance criterion".into(),
        )]))?;
        let status = e.project_acceptance_status()?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: status.requirements_revision.unwrap(),
            decision_record: None,
            guidance: BTreeMap::from([(
                "new-obligation".into(),
                "New public acceptance guidance".into(),
            )]),
        })?;
        let after = e.project_relational_context()?;
        assert_eq!(
            after["inputs"]["public_desires"]["history"],
            before["inputs"]["public_desires"]["history"]
        );
        assert!(after["inputs"]["parents"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["id"] == json!(identity)));
        assert_eq!(
            after["inputs"]["acceptance_parents"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_ne!(
            after["inputs"]["requirements_revision"],
            before["inputs"]["requirements_revision"]
        );
        drop(e);
        let e = Engine::open(dir.path())?;
        assert_eq!(after, e.project_relational_context()?);
        Ok(())
    }
    #[test]
    fn revisions_preserve_original_meaning_and_mark_superseded_history() -> Result<()> {
        let (_d, e) = setup()?;
        let first = offer(&e, "Authored first developed meaning", Change::Development)?;
        let identity = first.identity;
        let id = submit(&e, first)?;
        let r1 = review(&e, id)?;
        let old = e.project_relational_context()?;
        let entry = old["inputs"]["parents"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x["id"] == json!(identity))
            .unwrap();
        let mut next = offer(&e, "Authored second developed meaning", Change::Development)?;
        next.identity = identity;
        next.revises = Some(r1);
        next.parents = serde_json::from_value(entry["parents"].clone())?;
        let id = submit(&e, next)?;
        let r2 = review(&e, id)?;
        let ctx = e.project_relational_context()?;
        let h = ctx["inputs"]["public_desires"]["history"]
            .as_array()
            .unwrap();
        assert_eq!(h[0]["superseded"], true);
        assert_eq!(h[0]["next"]["meaning"], "Authored first developed meaning");
        assert_eq!(h[1]["revision"], json!(r2));
        assert_eq!(h[1]["superseded"], false);
        assert_eq!(
            h[0]["next"]["original_meaning"],
            h[1]["next"]["original_meaning"]
        );
        assert!(e
            .project_desire_status(Status {
                state_root: e.store.head_root()?,
                identity,
                revision: Some(r1),
                withdrawn: true
            })
            .is_err());
        Ok(())
    }
}
