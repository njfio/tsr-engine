//! Reviewed information detour. Access is not task success or evidence of assertion truth.
use super::*;
use crate::engine::qualitative::retrieval::{Manifest, RetrievedRecord};
use anyhow::Context;
use serde_json::{json, Value};
const SOURCE: &str = "project-information@v1";
fn default_local_context_bytes() -> usize {
    32768
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Admission {
    pub state_root: Hash,
    pub manifest: Manifest,
    pub max_reads: u8,
    pub max_read_ms: u64,
    pub exploration_allowed: bool,
    #[serde(default = "default_local_context_bytes")]
    pub local_context_bytes: usize,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub binding: Hash,
    pub ids: Vec<String>,
    pub parents: Vec<PreferenceId>,
    pub question: String,
    pub relevance: String,
    pub uncertainty: String,
    pub access_probability: f64,
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Entry {
    Config {
        admission: Admission,
        scope: Hash,
        requirements: EventId,
    },
    Prepared {
        choice: Choice,
        snapshot: Hash,
        parents: Hash,
        review: cycle::Review,
    },
    Attempt {
        prepared: EventId,
        resource: String,
        binding: Hash,
        forecast: Forecast,
    },
    Receipt {
        prepared: EventId,
        attempt: EventId,
        resource: String,
        binding: Hash,
        elapsed_ms: u64,
        records: Vec<RetrievedRecord>,
    },
}
fn h<T: Serialize>(v: &T) -> Result<Hash> {
    Ok(Hash::of(&[&serde_json::to_vec(v)?]))
}
impl Engine {
    fn information_entries(&self) -> Result<Vec<(EventId, Entry)>> {
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
                ensure!(
                    Hash::of(&[&bytes]) == *digest,
                    "information evidence changed"
                );
                let entry: Entry = serde_json::from_slice(&bytes)?;
                let origin = if matches!(entry, Entry::Receipt { .. }) {
                    Source::Rules
                } else {
                    Source::Operator
                };
                ensure!(e.source == origin, "information provenance mismatch");
                out.push((e.id, entry));
            }
        }
        let mut configuration = None;
        let mut prepared = BTreeMap::new();
        let mut attempts = BTreeMap::new();
        let mut completed = BTreeSet::new();
        for (id, e) in &out {
            match e {
                Entry::Config { admission, .. } => {
                    ensure!(
                        configuration.is_none(),
                        "duplicate information configuration"
                    );
                    configuration = Some(admission);
                }
                Entry::Prepared { choice, review, .. } => {
                    let a = configuration.context("information before configuration")?;
                    ensure!(
                        prepared.len() < a.max_reads as usize
                            && review.decision == "approve"
                            && review.reviewer != review.generator,
                        "invalid information preparation/review"
                    );
                    prepared.insert(*id, choice);
                }
                Entry::Attempt { prepared: p, .. } => {
                    ensure!(
                        prepared.contains_key(p) && !attempts.values().any(|(prior, _)| prior == p),
                        "duplicate/foreign information intent"
                    );
                    attempts.insert(*id, (*p, e));
                }
                Entry::Receipt {
                    prepared: p,
                    attempt,
                    resource,
                    binding,
                    records,
                    ..
                } => {
                    let (prior, a) = attempts
                        .get(attempt)
                        .context("foreign information attempt")?;
                    let Entry::Attempt {
                        resource: r,
                        binding: b,
                        ..
                    } = a
                    else {
                        unreachable!()
                    };
                    ensure!(
                        prior == p && r == resource && b == binding && completed.insert(*p),
                        "information receipt identity/replay mismatch"
                    );
                    let choice = prepared[p];
                    let m = &configuration.context("information corpus")?.manifest;
                    ensure!(
                        records.iter().map(|r| r.id.clone()).collect::<Vec<_>>() == choice.ids,
                        "information selected IDs changed"
                    );
                    let mut total = 0;
                    for r in records {
                        let declared = m
                            .records
                            .iter()
                            .find(|d| d.id == r.id)
                            .context("foreign information record")?;
                        total += r.bytes.len();
                        ensure!(
                            r.digest == declared.digest
                                && Hash::of(&[&r.bytes]) == r.digest
                                && r.attribution == declared.attribution
                                && r.uncertainty == declared.uncertainty
                                && r.bytes.len() <= 2048
                                && std::str::from_utf8(&r.bytes).is_ok(),
                            "information receipt bytes/provenance changed"
                        );
                    }
                    ensure!(total <= 4096, "information receipt byte bound");
                }
            }
        }
        Ok(out)
    }
    fn information_append(&self, entry: Entry) -> Result<EventId> {
        let source = if matches!(entry, Entry::Receipt { .. }) {
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
            source,
        )
    }
    fn information_config(&self) -> Result<Option<Admission>> {
        let mut found = None;
        for (_, e) in self.information_entries()? {
            if let Entry::Config {
                admission,
                scope,
                requirements,
            } = e
            {
                ensure!(found.is_none(), "duplicate information admission");
                ensure!(
                    scope == h(&self.project_configuration()?)?
                        && Some(requirements)
                            == self.project_acceptance_status()?.requirements_revision,
                    "information authority changed"
                );
                found = Some(admission);
            }
        }
        Ok(found)
    }
    fn information_parents(&self) -> Result<Value> {
        // Published active meanings only, not private acceptance text.
        let cycle = self.acceptance_cycle()?;
        let aims = self.acceptance_current_aims()?;
        let active = self
            .adaptive_state()?
            .preferences
            .into_iter()
            .filter(|p| p.is_open())
            .map(|p| p.preference.id)
            .collect::<BTreeSet<_>>();
        let base = cycle["guidance"]
            .as_object()
            .into_iter()
            .flatten()
            .filter_map(|(key, text)| {
                aims.get(key)
                    .filter(|id| active.contains(id))
                    .map(|id| json!({"id":id,"meaning":text,"requirement":key}))
            })
            .collect::<Vec<_>>();
        Ok(if let Some(public) = self.project_public_desires(&base)? {
            public["active"].clone()
        } else {
            json!(base)
        })
    }
    pub fn project_information_configure(&self, admission: Admission) -> Result<Value> {
        self.write(|| {
            ensure!(
                self.store.head_root()? == admission.state_root
                    && self.information_config()?.is_none(),
                "stale/duplicate information admission"
            );
            ensure!(
                self.adaptive_state()?.pending.is_none()
                    && !self.project_acceptance_status()?.accepted,
                "pending/accepted task"
            );
            ensure!(
                admission.exploration_allowed
                    && (1..=3).contains(&admission.max_reads)
                    && matches!(admission.local_context_bytes, 32768 | 131072)
                    && (1..=5000).contains(&admission.max_read_ms),
                "bounded explicit exploration admission required"
            );
            let m = &admission.manifest;
            ensure!(
                m.root.is_absolute()
                    && fs::canonicalize(&m.root)? == m.root
                    && (1..=16).contains(&m.records.len()),
                "canonical bounded corpus"
            );
            let mut ids = BTreeSet::new();
            let mut paths = BTreeSet::new();
            for r in &m.records {
                ensure!(
                    short(&r.id)
                        && short(&r.attribution)
                        && short(&r.uncertainty)
                        && valid_path(&r.path)
                        && ids.insert(&r.id)
                        && paths.insert(&r.path),
                    "bounded unique public records"
                );
            }
            let status = self.project_acceptance_status()?;
            self.information_append(Entry::Config {
                admission,
                scope: h(&self.project_configuration()?)?,
                requirements: status
                    .requirements_revision
                    .context("registered requirements required")?,
            })?;
            self.project_information_context()
        })
    }
    pub(crate) fn project_information_pending(&self) -> Result<Option<(EventId, bool)>> {
        let entries = self.information_entries()?;
        for (id, e) in &entries {
            if matches!(e, Entry::Prepared { .. })
                && !entries
                    .iter()
                    .any(|(_, r)| matches!(r,Entry::Receipt{prepared,..} if prepared==id))
            {
                return Ok(Some((
                    *id,
                    entries
                        .iter()
                        .any(|(_, r)| matches!(r,Entry::Attempt{prepared,..} if prepared==id)),
                )));
            }
        }
        Ok(None)
    }
    pub fn project_information_context(&self) -> Result<Value> {
        let Some(a) = self.information_config()? else {
            return Ok(Value::Null);
        };
        let entries = self.information_entries()?;
        let status = self.project_acceptance_status()?;
        let parents = self.information_parents()?;
        let receipts = self.project_information_receipts()?;
        let body = json!({"version":"project-information@v1","admission":h(&a)?,"local_context_bytes":a.local_context_bytes,"corpus":h(&a.manifest)?,"source_snapshot":status.snapshot,"requirements_revision":status.requirements_revision,"parents":parents,"index":a.manifest.records.iter().map(|r|json!({"id":r.id,"digest":r.digest,"attribution":r.attribution,"uncertainty":r.uncertainty})).collect::<Vec<_>>(),"receipts":receipts,"remaining_reads":a.max_reads as usize-entries.iter().filter(|(_,e)|matches!(e,Entry::Prepared{..})).count().min(a.max_reads as usize),"policy":"Optional independently reviewed uncertainty detour. No user preference, assertion truth, task fulfillment or mixed-action utility is inferred from access. No prescribed record/order; abstention valid."});
        let ids = a
            .manifest
            .records
            .iter()
            .map(|r| r.id.clone())
            .collect::<Vec<_>>();
        let parents = body["parents"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["id"].clone())
            .collect::<Vec<_>>();
        let binding = h(&body)?;
        let text = json!({"type":"string","minLength":1,"maxLength":512});
        let schema = json!({"type":"object","additionalProperties":false,"required":["binding","ids","parents","question","relevance","uncertainty","access_probability"],"properties":{
            "binding":{"const":binding},"ids":{"type":"array","minItems":1,"maxItems":4,"uniqueItems":true,"items":{"enum":ids}},"parents":{"type":"array","minItems":1,"maxItems":8,"uniqueItems":true,"items":{"enum":parents}},"question":text,"relevance":text,"uncertainty":text,"access_probability":{"type":"number","minimum":0,"maximum":1}}});
        Ok(
            json!({"binding":binding,"context":body,"choice_schema":schema,"text_limit":"Each text field is at most 512 UTF-8 bytes, not characters. access_probability forecasts validated byte access only, not task benefit. Use only when a concrete uncertainty and plausible source relevance justify a reversible read; no invented user preference."}),
        )
    }
    pub(crate) fn validate_project_information(&self, c: &Choice) -> Result<()> {
        let view = self.project_information_context()?;
        ensure!(
            !view.is_null()
                && view["binding"] == json!(c.binding)
                && view["context"]["remaining_reads"].as_u64().unwrap_or(0) > 0,
            "stale/unadmitted information choice"
        );
        ensure!(
            self.project_information_pending()?.is_none()
                && !self.project_objective_pending()?
                && self.adaptive_state()?.pending.is_none(),
            "pending action or unknown information effect"
        );
        ensure!(
            (1..=4).contains(&c.ids.len())
                && c.ids.iter().collect::<BTreeSet<_>>().len() == c.ids.len()
                && c.ids.iter().all(|id| view["context"]["index"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|r| r["id"] == *id)),
            "unadmitted record IDs"
        );
        ensure!(
            !c.parents.is_empty()
                && c.parents.len() <= 8
                && c.parents.iter().collect::<BTreeSet<_>>().len() == c.parents.len()
                && c.parents.iter().all(|id| view["context"]["parents"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|p| p["id"] == json!(id))),
            "stale/withdrawn information parents"
        );
        ensure!(
            [&c.question, &c.relevance, &c.uncertainty]
                .iter()
                .all(|s| short(s))
                && c.access_probability.is_finite()
                && (0.0..=1.0).contains(&c.access_probability),
            "bounded inquiry and access forecast required"
        );
        Ok(())
    }
    pub(crate) fn prepare_project_information(
        &self,
        c: Choice,
        review: cycle::Review,
    ) -> Result<EventId> {
        self.validate_project_information(&c)?;
        self.information_append(Entry::Prepared {
            choice: c,
            snapshot: self.project_acceptance_status()?.snapshot,
            parents: h(&self.information_parents()?)?,
            review,
        })
    }
    pub(crate) fn project_information_receipts(&self) -> Result<Vec<Value>> {
        let entries = self.information_entries()?;
        let mut out = vec![];
        for (id, e) in &entries {
            if let Entry::Receipt {
                prepared,
                attempt,
                records,
                ..
            } = e
            {
                let p = entries
                    .iter()
                    .find(|(id, _)| id == prepared)
                    .context("missing information preparation")?;
                let Entry::Prepared {
                    choice,
                    snapshot,
                    parents,
                    ..
                } = &p.1
                else {
                    anyhow::bail!("wrong preparation")
                };
                out.push(json!({"receipt":id,"attempt":attempt,"prepared":prepared,"source_snapshot":snapshot,"parents_binding":parents,"choice":choice,"records":records.iter().map(|r|json!({"id":r.id,"digest":r.digest,"text":String::from_utf8_lossy(&r.bytes),"attribution":r.attribution,"uncertainty":r.uncertainty})).collect::<Vec<_>>(),"meaning":"Authenticated access/content identity. Contents are attributed assertions, not established truth, task outcomes or fulfillment."}));
            }
        }
        Ok(out)
    }
    pub(crate) fn project_information_measurement(
        &self,
        resource: &str,
        binding: Hash,
    ) -> Result<Option<u64>> {
        Ok(self
            .information_entries()?
            .iter()
            .find_map(|(_, e)| match e {
                Entry::Receipt {
                    resource: r,
                    binding: b,
                    elapsed_ms,
                    ..
                } if r == resource && *b == binding => Some(*elapsed_ms),
                _ => None,
            }))
    }
    pub fn project_information_resume(&self, binding: Hash) -> Result<Value> {
        let current = self.project_cycle_request()?;
        ensure!(
            current["branch"] == "information_resume" && current["binding"] == json!(binding),
            "stale/non-information cycle"
        );
        let (prepared, unknown) = self
            .project_information_pending()?
            .context("no prepared information action")?;
        ensure!(!unknown, "unknown information effect; no retry");
        let entries = self.information_entries()?;
        let Entry::Prepared {
            choice,
            snapshot,
            parents,
            ..
        } = entries
            .iter()
            .find(|(id, _)| *id == prepared)
            .context("preparation")?
            .1
            .clone()
        else {
            anyhow::bail!("preparation kind")
        };
        ensure!(
            snapshot == self.project_acceptance_status()?.snapshot
                && parents == h(&self.information_parents()?)?,
            "source/parents changed before information dispatch"
        );
        let a = self
            .information_config()?
            .context("information authority")?;
        let resource = self
            .reserve_project_preparation(binding, a.max_read_ms)?
            .context("active resource admission required")?;
        let forecast = Forecast::new(
            OutcomeDefId(Hash::of(&[b"validated_information_receipt@v1"])),
            Horizon::ThisDispatch,
            Forecaster::Generator,
            Probability::new(choice.access_probability)?,
            self.events()?.len() as u64,
        );
        let attempt = self.write(|| {
            ensure!(
                self.project_information_pending()? == Some((prepared, false))
                    && snapshot == self.project_acceptance_status()?.snapshot
                    && parents == h(&self.information_parents()?)?,
                "source/parents changed before information intent"
            );
            self.information_append(Entry::Attempt {
                prepared,
                resource: resource.clone(),
                binding,
                forecast: forecast.clone(),
            })
        })?;
        let dispatch_root = self.store.head_root()?;
        let start = std::time::Instant::now();
        let mut records = vec![];
        for id in &choice.ids {
            let r = a
                .manifest
                .records
                .iter()
                .find(|r| &r.id == id)
                .context("record not admitted")?;
            let bytes = tsr_batch::broker_v2::read_bounded_record(&a.manifest.root, &r.path)?;
            ensure!(Hash::of(&[&bytes]) == r.digest, "corpus bytes changed");
            ensure!(
                bytes.len() <= 2048
                    && std::str::from_utf8(&bytes).is_ok()
                    && records
                        .iter()
                        .map(|r: &RetrievedRecord| r.bytes.len())
                        .sum::<usize>()
                        + bytes.len()
                        <= 4096,
                "public UTF-8 information bound"
            );
            records.push(RetrievedRecord {
                id: r.id.clone(),
                digest: r.digest,
                bytes,
                attribution: r.attribution.clone(),
                uncertainty: r.uncertainty.clone(),
            });
        }
        let elapsed_ms = start.elapsed().as_millis().try_into()?;
        self.write(||{
            ensure!(self.store.head_root()?==dispatch_root && self.project_information_pending()?==Some((prepared,true)) && snapshot==self.project_acceptance_status()?.snapshot && parents==h(&self.information_parents()?)?,"information dispatch changed");
            let receipt=self.information_append(Entry::Receipt{prepared,attempt,resource,binding,elapsed_ms,records})?;
            self.append(EventKind::Observed{observation:Observation::Return{surprise:vec![Surprise{forecast:forecast.id,residual:forecast.raw.get()-1.0}],contrast:None}},Source::Rules)?;
            Ok(json!({"receipt":receipt,"access_complete":true,"task_fulfillment_inferred":false,"next":self.project_cycle_request()?}))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::adaptive::provider::{Response, Suggestion, PROTOCOL};
    fn setup() -> Result<(tempfile::TempDir, Engine)> {
        let d = tempfile::tempdir()?;
        let e = Engine::open(d.path())?;
        e.project_initialize(serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/engine-invoice/project.json"
        )))?)?;
        e.project_acceptance_requirements(BTreeMap::from([
            ("correctness".into(), "Preserve invoice behavior".into()),
            ("preservation".into(), "Preserve boundaries".into()),
        ]))?;
        e.project_lifecycle_open(lifecycle::Admission {
            state_root: e.store.head_root()?,
            training: None,
            tool_ms_ceiling: 100000,
            reason: "Authored local information fixture".into(),
            non_tool: Some(budget::Limits {
                model_calls: 12,
                model_tokens: 10000,
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
            guidance: BTreeMap::from([
                ("correctness".into(), "Preserve invoice behavior".into()),
                ("preservation".into(), "Preserve boundaries".into()),
            ]),
        })?;
        let root = d.path().join("public-records");
        fs::create_dir(&root)?;
        fs::write(
            root.join("a"),
            "Authored fictional assertion, not a program result.",
        )?;
        e.project_information_configure(Admission {
            state_root: e.store.head_root()?,
            manifest: Manifest {
                root: root.canonicalize()?,
                records: vec![crate::engine::qualitative::retrieval::Record {
                    id: "a".into(),
                    path: "a".into(),
                    digest: Hash::of(&[&fs::read(root.join("a"))?]),
                    attribution: "Local authored fixture".into(),
                    uncertainty: "Assertion not validated".into(),
                }],
            },
            local_context_bytes: 32768,
            max_reads: 1,
            max_read_ms: 1000,
            exploration_allowed: true,
        })?;
        Ok((d, e))
    }
    fn choice(e: &Engine) -> Result<Choice> {
        let c = e.project_information_context()?;
        Ok(Choice {
            binding: serde_json::from_value(c["binding"].clone())?,
            ids: vec!["a".into()],
            parents: c["context"]["parents"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| serde_json::from_value(p["id"].clone()))
                .collect::<std::result::Result<_, _>>()?,
            question: "Could the authored condition clarify an uncertainty?".into(),
            relevance: "May inform both original public aims; no guaranteed benefit".into(),
            uncertainty: "Actual program behavior is not established".into(),
            access_probability: 0.5,
        })
    }
    fn prepare(e: &Engine, c: Choice) -> Result<Value> {
        let response = Response {
            protocol: PROTOCOL.into(),
            state_root: e.store.head_root()?,
            attention_binding: e.proposal_attention_binding()?,
            proposals: vec![Suggestion::Information { choice: c }],
            pursuit_expectations: Default::default(),
            common_event_forecasts: BTreeMap::new(),
        };
        let v = e.project_cycle_preview(response.clone())?;
        let b = serde_json::from_value(v["cycle"]["binding"].clone())?;
        e.project_cycle_advance(cycle::Advance {
            binding: b,
            review: cycle::Review {
                binding: b,
                response_digest: serde_json::from_value(v["response_digest"].clone())?,
                state_root: response.state_root,
                source_snapshot: serde_json::from_value(v["cycle"]["source_snapshot"].clone())?,
                reviewer: "authored-independent-fixture".into(),
                generator: "authored-proposal-fixture".into(),
                decision: "approve".into(),
            },
            response,
        })
    }
    #[test]
    fn reviewed_read_is_not_task_success_or_failure_and_restart_is_inert() -> Result<()> {
        let (d, e) = setup()?;
        let before = e.adaptive_state()?;
        let status = e.project_acceptance_status()?;
        let c = choice(&e)?;
        let v = prepare(&e, c.clone())?;
        assert_eq!(v["next"]["branch"], "information_resume");
        assert!(!v["review_required"].as_bool().unwrap());
        assert!(e.validate_project_information(&c).is_err());
        let binding = serde_json::from_value(v["next"]["binding"].clone())?;
        let r = e.project_information_resume(binding)?;
        assert_eq!(r["access_complete"], true);
        assert_eq!(r["task_fulfillment_inferred"], false);
        assert_eq!(e.project_acceptance_status()?.accepted, status.accepted);
        let after = e.adaptive_state()?;
        assert_eq!(before.completed, after.completed);
        assert_eq!(before.unmet_pressure, after.unmet_pressure);
        assert_eq!(e.project_budget()?.unresolved_tool_requests, 0);
        let roots = e.store.head_root()?;
        let restarted = Engine::open(d.path())?;
        assert_eq!(
            restarted.project_information_context()?,
            e.project_information_context()?
        );
        assert!(restarted.project_information_resume(binding).is_err());
        assert_eq!(restarted.store.head_root()?, roots);
        assert!(e.events()?.iter().any(|(_, v)| matches!(
            &v.kind,
            EventKind::Observed {
                observation: Observation::Return { contrast: None, .. }
            }
        )));
        assert_eq!(
            e.project_relational_context()?["inputs"]["information_returns"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        Ok(())
    }
    #[test]
    fn altered_bytes_and_symlink_escape_remain_unknown_without_retry() -> Result<()> {
        for symlink in [false, true] {
            let (d, e) = setup()?;
            let v = prepare(&e, choice(&e)?)?;
            let p = d.path().join("public-records/a");
            if symlink {
                fs::remove_file(&p)?;
                std::os::unix::fs::symlink(d.path().join("outside"), &p)?;
                fs::write(d.path().join("outside"), "outside")?;
            } else {
                fs::write(&p, "changed")?;
            }
            let binding = serde_json::from_value(v["next"]["binding"].clone())?;
            assert!(e.project_information_resume(binding).is_err());
            assert_eq!(e.project_cycle_request()?["branch"], "reconcile");
            let root = e.store.head_root()?;
            assert!(e.project_information_resume(binding).is_err());
            assert_eq!(root, e.store.head_root()?);
            assert!(e.project_information_receipts()?.is_empty());
            assert_eq!(e.project_budget()?.unresolved_tool_requests, 1);
        }
        Ok(())
    }
    #[test]
    fn stale_cross_root_wrong_id_and_unauthorized_choices_rejected() -> Result<()> {
        let (_d, e) = setup()?;
        let (_d2, e2) = setup()?;
        let c = choice(&e)?;
        assert!(e2.validate_project_information(&c).is_err());
        let mut bad = c.clone();
        bad.ids = vec!["../outside".into()];
        assert!(e.validate_project_information(&bad).is_err());
        bad = c.clone();
        bad.binding = Hash::of(&[b"stale"]);
        assert!(e.validate_project_information(&bad).is_err());
        bad = c.clone();
        bad.parents.clear();
        assert!(e.validate_project_information(&bad).is_err());
        assert!(serde_json::from_value::<Choice>(json!({"binding":c.binding,"ids":c.ids,"parents":c.parents,"question":c.question,"relevance":c.relevance,"uncertainty":c.uncertainty,"access_probability":0.5,"task_accepted":true})).is_err());
        Ok(())
    }
}
