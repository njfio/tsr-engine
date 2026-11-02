//! Operator-reviewed finite checks, separate from protected acceptance tests.
//! Only controller receipts from an explicitly supplied local episode are imported.
use super::*;
use anyhow::Context;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mapping {
    pub cases: BTreeSet<String>,
    pub rationale: String,
    pub limits: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bundle {
    pub version: String,
    pub snapshot: Hash,
    pub requirements_revision: EventId,
    /// Operator attestation of independently authored and reviewed check mapping.
    pub author: String,
    pub reviewer: String,
    pub generator: String,
    pub review: String,
    pub harness: BTreeMap<String, String>,
    pub probe_entry: String,
    pub cases: Vec<ProbeCase>,
    pub mapping: BTreeMap<String, Mapping>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CaseOutcome {
    PassedFiniteCase,
    Counterexample,
    Unknown,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Observation {
    pub admission: EventId,
    pub bundle_digest: Hash,
    pub source_episode_root: Hash,
    pub source_resource_root: Hash,
    pub mapping: BTreeMap<String, Mapping>,
    pub receipt: EventId,
    pub snapshot: Hash,
    pub checked_snapshot: Hash,
    pub requirements_revision: EventId,
    pub cases: BTreeMap<String, CaseOutcome>,
    /// Never Met: finite success is not universal semantic proof.
    pub findings: BTreeMap<String, Finding>,
    /// Resources charged in the check episode, not again on import.
    pub resources: serde_json::Value,
    /// Optional authenticated raw browser cases; no inferred gameplay judgment.
    /// Absent in legacy records, whose serialized identity remains unchanged.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub browser_cases: Option<Vec<ProbeObservation>>,
}
impl Engine {
    pub(super) fn supplemental_bundle(&self, id: EventId) -> Result<Bundle> {
        ensure!(
            !self.verifier_records()?.iter().any(
                |(_, r)| matches!(r,Record::SupplementalRevoked{admission,..} if *admission==id)
            ),
            "supplemental admission revoked"
        );
        self.verifier_records()?
            .into_iter()
            .find_map(|(key, r)| match r {
                Record::SupplementalAdmitted { bundle } if key == id => Some(bundle),
                _ => None,
            })
            .ok_or_else(|| anyhow::anyhow!("unknown supplemental admission"))
    }
    pub(crate) fn project_validate_supplemental(&self, b: &Bundle) -> Result<()> {
        self.supplemental_spec(b).map(|_| ())
    }
    fn supplemental_spec(&self, b: &Bundle) -> Result<ProjectSpec> {
        let current = self.project_acceptance_status()?;
        ensure!(
            matches!(
                b.version.as_str(),
                "supplemental-checks@v1" | "supplemental-cargo@v2" | "supplemental-browser@v3"
            ) && b.snapshot == current.snapshot
                && Some(b.requirements_revision) == current.requirements_revision,
            "stale supplemental source or requirements"
        );
        ensure!(
            [&b.author, &b.reviewer, &b.generator, &b.review]
                .into_iter()
                .all(|s| short(s))
                && b.author != b.reviewer
                && b.author != b.generator
                && b.reviewer != b.generator,
            "independent author/reviewer attestation required"
        );
        let requirements = self.project_acceptance_requirements_view()?;
        let ids = b
            .cases
            .iter()
            .map(|c| c.id.clone())
            .collect::<BTreeSet<_>>();
        ensure!(
            !b.mapping.is_empty()
                && b.mapping.keys().all(|k| requirements.contains_key(k))
                && b.mapping.values().all(|m| !m.cases.is_empty()
                    && m.cases.is_subset(&ids)
                    && short(&m.rationale)
                    && short(&m.limits))
                && b.mapping
                    .values()
                    .flat_map(|m| m.cases.iter().cloned())
                    .collect::<BTreeSet<_>>()
                    == ids,
            "explicit reviewed requirement mapping and limits required"
        );
        let mut spec = self.project_configuration()?;
        ensure!(
            spec.cargo == (b.version == "supplemental-cargo@v2")
                && spec.browser == (b.version == "supplemental-browser@v3"),
            "supplemental version must match adapter"
        );
        let snapshot = self.project_snapshot(&spec)?;
        ensure!(
            !b.harness.is_empty()
                && b.harness.len() <= 2
                && b.harness.keys().all(|p| !snapshot.contains_key(p)),
            "new supplemental namespace required; original files cannot be replaced"
        );
        if spec.cargo {
            let manifest: toml::Value = spec.protected["Cargo.toml"].parse()?;
            ensure!(
                spec.named_package.is_none()
                    && manifest.get("workspace").is_none()
                    && manifest.get("package").is_some()
                    && manifest["package"]
                        .get("autotests")
                        .is_none_or(|v| v.as_bool() == Some(true))
                    && b.probe_entry.is_empty(),
                "supplemental Cargo v2 requires root package with automatic integration tests"
            );
            ensure!(
                b.harness.keys().all(|p| p
                    .strip_prefix("tests/tsr_supplemental_")
                    .and_then(|s| s.strip_suffix(".rs"))
                    .is_some_and(|s| !s.is_empty()
                        && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_'))),
                "reserved supplemental integration-test namespace required"
            );
            ensure!(
                ids.len() == b.cases.len()
                    && !ids.contains("full_workspace_oracle@v1")
                    && spec.cases.iter().all(|c| !ids.contains(&c.id)),
                "supplemental test identities must be new and unique across the entire suite"
            );
        } else if spec.browser {
            ensure!(
                b.probe_entry == "browser-harness.cjs"
                    && b.harness.keys().all(|p| p.starts_with("supplemental/") && p.ends_with(".json"))
                    && b.cases.iter().all(|c| c.args.len() == 1 && b.harness.contains_key(&c.args[0]))
                    && ids.len() == b.cases.len()
                    && spec.cases.iter().all(|c| !ids.contains(&c.id)),
                "new browser probe namespace and case identities required; original verifier is immutable"
            );
        } else {
            ensure!(
                b.harness.contains_key(&b.probe_entry)
                    && b.harness.keys().all(|p| p.starts_with("supplemental/")),
                "supplemental probe namespace required"
            );
        }
        for (p, v) in &mut spec.editable {
            *v = snapshot[p].clone();
        }
        for (p, v) in &mut spec.protected {
            *v = snapshot[p].clone();
        }
        spec.protected.extend(b.harness.clone());
        spec.probe_entry = b.probe_entry.clone();
        if spec.cargo || spec.browser {
            spec.cases.extend(b.cases.clone());
        } else {
            spec.cases = b.cases.clone();
        }
        spec.validate()?;
        Ok(spec)
    }
    /// No execution permission or budget is granted. A separate episode must be admitted.
    pub fn project_supplemental_admit(&self, bundle: Bundle) -> Result<EventId> {
        self.write(|| self.project_supplemental_admit_inner(bundle))
    }
    pub(crate) fn project_supplemental_admit_inner(&self, bundle: Bundle) -> Result<EventId> {
        self.supplemental_spec(&bundle)?;
        ensure!(
            self.verifier_records()?
                .iter()
                .filter(|(_, r)| matches!(r, Record::SupplementalAdmitted { .. }))
                .count()
                < 32,
            "supplemental admission bound"
        );
        self.verifier_append(Record::SupplementalAdmitted { bundle }, Source::Operator)
    }
    pub fn project_supplemental_spec(&self, id: EventId) -> Result<ProjectSpec> {
        self.supplemental_spec(&self.supplemental_bundle(id)?)
    }
    /// Import controller-produced evidence, never a model-supplied report or raw receipt JSON.
    /// The caller must explicitly trust the local engine store as controller-owned evidence.
    pub fn project_supplemental_link(
        &self,
        id: EventId,
        other: &Engine,
        receipt: EventId,
    ) -> Result<EventId> {
        self.write(|| {
            ensure!(self.directory.canonicalize()?!=other.directory.canonicalize()?,"separate check episode required");
            let source_episode_root=other.store.head_root()?;
            let source_resource_root=other.shadow_store()?.store.head_root()?;
            let b=self.supplemental_bundle(id)?;
            let expected=self.supplemental_spec(&b)?;
            let spec=other.project_configuration()?;
            ensure!(serde_json::to_value(&expected)?==serde_json::to_value(&spec)?,"wrong check configuration or original source");
            let snapshot=other.project_snapshot(&spec)?;
            let initial=spec.editable.iter().chain(&spec.protected).map(|(k,v)|(k.clone(),v.clone())).collect::<BTreeMap<_,_>>();
            ensure!(snapshot==initial,"check source drift");
            let checked_snapshot=super::super::super::digest(&snapshot)?;
            let records=other.records()?;
            let events=other.events()?;
            let (commitment,verdict,observations)=records.iter().find_map(|(key,r)|match r {
                AdaptiveRecord::Coding{record:CodingRecord::Receipt{commitment,verdict,source_after,observations,..}}
                    if *key==receipt && *source_after==Some(checked_snapshot)=>Some((*commitment,verdict,observations)), _=>None
            }).ok_or_else(||anyhow::anyhow!("matching immutable receipt required"))?;
            let pos=events.iter().position(|(_,e)|e.id==receipt && e.source==Source::Rules)
                .ok_or_else(||anyhow::anyhow!("controller receipt provenance required"))?;
            ensure!(events.get(pos+1).is_some_and(|(_,e)|e.source==Source::Rules && matches!(e.kind,EventKind::Executed{..})),"completed execution event required");
            // Require the receipt's commitment to a source-bound verification action, not an edit.
            let action=records.iter().find_map(|(_,r)|match r {
                AdaptiveRecord::Pending{commitment: key, action,..} if *key==commitment=>Some(action),
                _=>None,
            });
            let action=action.ok_or_else(||anyhow::anyhow!("receipt commitment missing"))?;
            ensure!(other.project_verification_admitted(action)?.is_some_and(|(source,_)|source==checked_snapshot),"source-bound verification admission required");
            let candidate=events.iter().find_map(|(_,e)|match e.kind { EventKind::Committed{candidate,..} if e.id==commitment && e.source==Source::Rules=>Some(candidate),_=>None }).ok_or_else(||anyhow::anyhow!("controller commitment missing"))?;
            ensure!(matches!(events[pos+1].1.kind,EventKind::Executed{candidate: actual,..} if actual==candidate),"wrong execution candidate");
            let mut cases=BTreeMap::new();
            for c in &b.cases {
                let found=observations.iter().filter(|o|o.case_id.as_ref()==Some(&c.id)).collect::<Vec<_>>();
                ensure!(found.len()<=1,"duplicate check observation");
                let outcome=if let Some(o)=found.first() {
                    if spec.cargo {
                        let mut argv=super::super::super::cargo::command();
                        argv.extend(["--".into(),"--test-threads=1".into()]);
                        ensure!(o.argv==argv && o.expected_stdout=="named Cargo test passes", "Cargo check command/identity mismatch");
                        // Summaries alone are not proof. The source-bound controller final verdict
                        // is produced by strict completed-suite/inventory/exit/truncation validation.
                        match (verdict,o.exit,o.passed) {
                            (Verdict::Passed,0,Some(true)) | (Verdict::FailedTests,101,Some(true)) => CaseOutcome::PassedFiniteCase,
                            (Verdict::FailedTests,101,Some(false)) => CaseOutcome::Counterexample,
                            _ => CaseOutcome::Unknown,
                        }
                    } else if spec.browser {
                        let mut argv=vec!["node".to_string(),"browser-harness.cjs".to_string()];
                        argv.extend(c.args.clone());
                        if spec.browser_container_only { argv.push("--container-only".into()); }
                        ensure!(o.argv==argv && o.expected_stdout==c.expected_stdout,"browser check identity/expectation mismatch");
                        // Probe failure/unavailability is not an artifact counterexample.
                        if !o.abbreviated && o.exit==0 && o.passed==Some(true) && matches!(verdict,Verdict::Passed|Verdict::FailedTests) { CaseOutcome::PassedFiniteCase }
                        else { CaseOutcome::Unknown }
                    } else {
                    let argv=std::iter::once("./fixture-probe".to_string()).chain(c.args.clone()).collect::<Vec<_>>();
                    ensure!(o.argv==argv && o.expected_stdout==c.expected_stdout,"check identity/expectation mismatch");
                    if o.abbreviated || o.exit!=0 || !matches!(verdict,Verdict::Passed|Verdict::FailedTests) { CaseOutcome::Unknown }
                    else if o.passed==Some(true) && o.stdout.trim()==c.expected_stdout.trim() { CaseOutcome::PassedFiniteCase }
                    else if o.passed==Some(false) && o.stdout.trim()!=c.expected_stdout.trim() { CaseOutcome::Counterexample }
                    else { CaseOutcome::Unknown }
                    }
                } else { CaseOutcome::Unknown };
                cases.insert(c.id.clone(),outcome);
            }
            ensure!(observations.iter().all(|o|o.case_id.as_ref().is_some_and(|k|spec.cases.iter().any(|c| &c.id==k))),"unknown check observation");
            let findings=b.mapping.iter().map(|(k,m)|(k.clone(),if m.cases.iter().any(|id|cases[id]==CaseOutcome::Counterexample){Finding::NotMet}else{Finding::Unknown})).collect();
            let budget=other.project_budget()?;
            if spec.browser { ensure!(budget.active && !budget.stopped && !budget.exceeded && budget.unsettled_requests==0 && budget.unresolved_tool_requests==0 && budget.unknown_usage==0 && budget.unknown_compute_requests==0 && budget.unknown_external_cost_requests==0,"browser check has stopped or unresolved resources/effects"); }
            let resources=serde_json::to_value(budget)?;
            ensure!(resources["active"]==true,"prospective check resource admission required");
            let browser_cases=if spec.browser {Some(observations.iter().filter(|o|o.case_id.as_ref().is_some_and(|id|b.cases.iter().any(|c| &c.id==id))).cloned().collect())} else {None};
            let observation=Observation{admission:id,bundle_digest:Hash::of(&[&serde_json::to_vec(&b)?]),source_episode_root,source_resource_root,mapping:b.mapping.clone(),receipt,snapshot:b.snapshot,checked_snapshot,requirements_revision:b.requirements_revision,cases,findings,resources,browser_cases};
            ensure!(other.store.head_root()?==source_episode_root && other.shadow_store()?.store.head_root()?==source_resource_root
                && other.project_snapshot(&spec)?==snapshot, "check evidence changed during import");
            self.supplemental_spec(&b)?;
            for (key,r) in self.verifier_records()? {
                if let Record::SupplementalObserved{observation:old}=r {
                    if old.admission==id && old.receipt==receipt { ensure!(old.checked_snapshot==checked_snapshot,"conflicting replay");return Ok(key); }
                }
            }
            self.verifier_append(Record::SupplementalObserved{observation},Source::Rules)
        })
    }
    pub(crate) fn observed_browser_supplemental(
        &self,
        admission: EventId,
        receipt: EventId,
    ) -> Result<Observation> {
        let b = self.supplemental_bundle(admission)?;
        ensure!(
            b.version == "supplemental-browser@v3",
            "reviewed browser observation required"
        );
        self.supplemental_spec(&b)?;
        let found = self
            .supplemental_current()?
            .into_iter()
            .filter(|o| o.admission == admission && o.receipt == receipt)
            .collect::<Vec<_>>();
        ensure!(
            found.len() == 1 && found[0].browser_cases.is_some(),
            "exact current browser observation required"
        );
        Ok(found[0].clone())
    }

    pub fn project_supplemental_revoke(&self, id: EventId, reason: String) -> Result<EventId> {
        self.write(|| {
            self.supplemental_bundle(id)?;
            ensure!(short(&reason), "bounded revocation reason required");
            self.verifier_append(
                Record::SupplementalRevoked {
                    admission: id,
                    reason,
                },
                Source::Operator,
            )
        })
    }
    pub(in crate::engine::adaptive::coding::project) fn supplemental_for(
        &self,
        snapshot: Hash,
        revision: Option<EventId>,
    ) -> Result<Vec<Observation>> {
        let records = self.verifier_records()?;
        let revoked = records
            .iter()
            .filter_map(|(_, r)| match r {
                Record::SupplementalRevoked { admission, .. } => Some(*admission),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        Ok(records
            .into_iter()
            .filter_map(|(_, r)| match r {
                Record::SupplementalObserved { observation }
                    if observation.snapshot == snapshot
                        && Some(observation.requirements_revision) == revision
                        && !revoked.contains(&observation.admission) =>
                {
                    Some(observation)
                }
                _ => None,
            })
            .collect())
    }
    /// Exact chronological current coverage, including unsuccessful observations and
    /// every registered mapped browser bundle. Never select only favorable returns.
    pub(crate) fn current_supplemental_bindings(
        &self,
        key: &str,
    ) -> Result<Option<Vec<typed::SupplementalBinding>>> {
        let current = self.supplemental_current()?;
        let expected = current
            .iter()
            .filter(|o| o.browser_cases.is_some() && o.mapping.contains_key(key))
            .map(|o| typed::SupplementalBinding {
                admission: o.admission,
                receipt: o.receipt,
            })
            .collect::<Vec<_>>();
        let status = self.project_acceptance_status()?;
        for (id, record) in self.verifier_records()? {
            if let Record::SupplementalAdmitted { bundle } = record {
                if bundle.version == "supplemental-browser@v3"
                    && bundle.snapshot == status.snapshot
                    && Some(bundle.requirements_revision) == status.requirements_revision
                    && bundle.mapping.contains_key(key)
                    && !expected.iter().any(|b| b.admission == id)
                {
                    return Ok(None);
                }
            }
        }
        Ok(Some(expected))
    }
    pub(crate) fn validate_supplemental_set(
        &self,
        key: &str,
        bindings: &[typed::SupplementalBinding],
    ) -> Result<()> {
        let expected = self
            .current_supplemental_bindings(key)?
            .context("registered cumulative bundle lacks observation")?;
        ensure!(
            !expected.is_empty() && bindings == expected,
            "incomplete, reordered or conflicting cumulative observations"
        );
        for b in bindings {
            self.observed_browser_supplemental(b.admission, b.receipt)?;
        }
        Ok(())
    }
    pub(crate) fn supplemental_current(&self) -> Result<Vec<Observation>> {
        let c = self.project_acceptance_status()?;
        self.supplemental_for(c.snapshot, c.requirements_revision)
    }
    pub(super) fn supplemental_consistent(&self, report: &Report) -> Result<()> {
        for observation in self.supplemental_current()? {
            for (key, finding) in observation.findings {
                ensure!(
                    !(finding == Finding::NotMet
                        && report
                            .findings
                            .get(&key)
                            .is_some_and(|x| x.finding == Finding::Met)),
                    "semantic met contradicts current independently reviewed counterexample"
                );
            }
        }
        Ok(())
    }
    pub fn project_supplemental_view(&self) -> Result<serde_json::Value> {
        let c = self.project_acceptance_status()?;
        let records=self.verifier_records()?.into_iter().filter_map(|(id,r)|match r {
            Record::SupplementalRevoked{admission,reason}=>Some(serde_json::json!({"id":id,"revoked":admission,"reason":reason})),
            Record::SupplementalAdmitted{bundle}=>Some(serde_json::json!({"id":id,"bundle":bundle})),
            Record::SupplementalObserved{observation}=>Some(serde_json::json!({"id":id,"source_current":observation.snapshot==c.snapshot && Some(observation.requirements_revision)==c.requirements_revision,"observation":observation})),_=>None
        }).collect::<Vec<_>>();
        Ok(
            serde_json::json!({"records":records,"automatic_acceptance":false,"finite_pass_is_semantic_met":false,"current_observations":self.supplemental_current()?}),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::super::{budget, lifecycle};
    use super::*;
    use tsr_corpus::{Limits, RunResult, Runner};
    fn bundle(e: &Engine) -> Result<Bundle> {
        let c = e.project_acceptance_status()?;
        Ok(Bundle {
            version: "supplemental-checks@v1".into(),
            snapshot: c.snapshot,
            requirements_revision: c.requirements_revision.unwrap(),
            author: "separate-check-author".into(),
            reviewer: "independent-mapping-review".into(),
            generator: "proposal-model".into(),
            review: "Synthetic contract fixture; no operational review or execution".into(),
            harness: BTreeMap::from([("supplemental/check.rs".into(), "fn main() {}".into())]),
            probe_entry: "supplemental/check.rs".into(),
            cases: vec![ProbeCase {
                id: "multiply".into(),
                args: vec!["3".into(), "4".into()],
                expected_stdout: "12\n".into(),
            }],
            mapping: BTreeMap::from([(
                "subtotal".into(),
                Mapping {
                    cases: BTreeSet::from(["multiply".into()]),
                    rationale: "3 times 4 is 12".into(),
                    limits: "One finite nonoverflowing example, not universal proof".into(),
                },
            )]),
        })
    }
    struct Fake {
        stdout: &'static [u8],
        exit: i32,
    }
    impl Runner for Fake {
        fn run(&self, _: &std::path::Path, args: &[String], _: &Limits) -> RunResult {
            match args[0].as_str() {
                "true" | "rustc" => RunResult::exit_only(0),
                "./fixture-probe" => RunResult::from_output(self.exit, self.stdout, b"", 1),
                _ => panic!("no edit or real execution permitted"),
            }
        }
    }
    fn check_episode(e: &Engine, id: EventId) -> Result<(tempfile::TempDir, Engine)> {
        let d = tempfile::tempdir()?;
        let other = Engine::open(d.path())?;
        other.project_initialize(e.project_supplemental_spec(id)?)?;
        other.project_lifecycle_open(lifecycle::Admission {
            training: None,
            state_root: other.store.head_root()?,
            tool_ms_ceiling: if other.project_configuration()?.browser {
                240_000
            } else {
                60_000
            },
            reason: "Synthetic isolated check protocol".into(),
            non_tool: Some(budget::Limits {
                model_calls: 0,
                model_tokens: 0,
                wall_ms: if other.project_configuration()?.browser {
                    600_000
                } else {
                    120_000
                },
                patch_bytes: 0,
                external_cost_microusd: 0,
            }),
        })?;
        Ok((d, other))
    }
    fn execute<R: Runner>(other: &Engine, fake: &R) -> Result<EventId> {
        let s = other.project_acceptance_status()?;
        let response = other.project_admit_verification(other.store.head_root()?, s.snapshot)?;
        other.evaluate_proposal(serde_json::from_value(response)?, true)?;
        let plan = other.coding_plan()?;
        assert!(plan.edits.is_empty());
        assert_eq!(plan.source_before, plan.expected_after.unwrap());
        other.coding_execute(plan.commitment, plan.source_before, fake)?;
        Ok(other.project_acceptance_status()?.test_receipt.unwrap())
    }
    struct BrowserDouble {
        pass: bool,
    }
    impl Runner for BrowserDouble {
        fn run(&self, _: &std::path::Path, args: &[String], _: &Limits) -> RunResult {
            if args[0] == "true" || (args[0] == "node" && args[1] == "--check") {
                return RunResult::exit_only(0);
            }
            assert_eq!(args[0], "node");
            assert_eq!(args[1], "browser-harness.cjs");
            let value = serde_json::json!({"version":"browser-observation@v1","passed":self.pass,"files":{"index.html":"a".repeat(64),"app.js":"b".repeat(64)},"observations":[{"kind":"render","sha256":"c".repeat(64)},{"kind":"click","value":"#start"},{"kind":"text","selector":"#start","text":"Authored result","met":self.pass,"abbreviated":false,"original_length":15},{"kind":"render","sha256":"d".repeat(64)}],"errors":[],"error_count":0});
            let exit = if self.pass { 0 } else { 1 };
            let mut r = RunResult::from_output(exit, &serde_json::to_vec(&value).unwrap(), b"", 1);
            r.execution = Some(tsr_corpus::ExecutionDiagnostics {
                version: 1,
                termination: "runtime_exit".into(),
                runtime_exit: Some(exit),
                runtime_signal: None,
                stdout_complete: true,
                stderr_complete: true,
                worker: "authored browser double, not real control evidence".into(),
            });
            r
        }
    }

    #[test]
    fn browser_probe_registration_preserves_original_scope_and_rejects_stale_mapping() -> Result<()>
    {
        for complete in [true, false] {
            for cumulative_basis in [true, false] {
                browser_registration(complete, cumulative_basis)?;
            }
        }
        Ok(())
    }
    fn browser_registration(complete: bool, cumulative_basis: bool) -> Result<()> {
        let d = tempfile::tempdir()?;
        let e = Engine::open(d.path())?;
        let original = super::super::super::super::browser::tests::spec();
        e.project_initialize(original.clone())?;
        e.project_acceptance_requirements(BTreeMap::from([(
            "crossing".into(),
            "Authored control observation only".into(),
        )]))?;
        let state = e.project_acceptance_status()?;
        e.project_acceptance_feedback(super::super::super::Feedback {
            requirements_revision: state.requirements_revision.unwrap(),
            decision_record: state.decision_record,
            guidance: e.project_acceptance_requirements_view()?,
        })?;
        let probe = original.protected["probe.json"].clone();
        let b = Bundle {
            version: "supplemental-browser@v3".into(),
            snapshot: state.snapshot,
            requirements_revision: state.requirements_revision.unwrap(),
            author: "authored mapper".into(),
            reviewer: "authored independent reviewer".into(),
            generator: "authored application".into(),
            review: "Authored contract fixture, not live review or semantic success".into(),
            harness: BTreeMap::from([("supplemental/controls.json".into(), probe)]),
            probe_entry: "browser-harness.cjs".into(),
            cases: vec![ProbeCase {
                id: "mapped_controls".into(),
                args: vec!["supplemental/controls.json".into()],
                expected_stdout: "browser-observation@v1".into(),
            }],
            mapping: BTreeMap::from([(
                "crossing".into(),
                Mapping {
                    cases: BTreeSet::from(["mapped_controls".into()]),
                    rationale: "Observe actual advertised control".into(),
                    limits: "Finite protocol observations; not fulfillment".into(),
                },
            )]),
        };
        for mutation in 0..5 {
            let mut bad = b.clone();
            match mutation {
                0 => bad.snapshot = Hash::of(&[b"stale"]),
                1 => {
                    bad.harness
                        .insert("browser-harness.cjs".into(), "changed".into());
                }
                2 => bad.probe_entry = "supplemental/arbitrary.cjs".into(),
                3 => bad.cases[0].id = original.cases[0].id.clone(),
                _ => bad.reviewer = bad.author.clone(),
            }
            assert!(e.project_supplemental_admit(bad).is_err());
        }
        let id = e.project_supplemental_admit(b)?;
        let derived = e.project_supplemental_spec(id)?;
        assert_eq!(derived.editable, original.editable);
        for (path, text) in &original.protected {
            assert_eq!(derived.protected.get(path), Some(text));
        }
        assert_eq!(
            serde_json::to_value(&derived.cases[..original.cases.len()])?,
            serde_json::to_value(&original.cases)?
        );
        assert_eq!(
            serde_json::to_value(e.project_configuration()?)?,
            serde_json::to_value(&original)?
        );
        let reopened = Engine::open(d.path())?;
        assert_eq!(
            serde_json::to_value(reopened.project_supplemental_spec(id)?)?,
            serde_json::to_value(&derived)?
        );
        let (_od, other) = check_episode(&reopened, id)?;
        assert!(reopened
            .project_supplemental_link(id, &other, EventId(Hash::of(&[b"forged receipt"])))
            .is_err());
        assert!(
            reopened.project_supplemental_view()?["current_observations"]
                .as_array()
                .unwrap()
                .is_empty()
        );
        let receipt = execute(&other, &BrowserDouble { pass: true })?;
        let linked = reopened.project_supplemental_link(id, &other, receipt)?;
        assert_eq!(
            reopened.project_supplemental_link(id, &other, receipt)?,
            linked
        );
        let observed = reopened.project_supplemental_view()?["current_observations"].clone();
        assert_eq!(observed[0]["findings"]["crossing"], "unknown");
        assert_eq!(observed[0]["browser_cases"].as_array().unwrap().len(), 1);
        assert_eq!(
            observed[0]["browser_cases"][0]["case_id"],
            "mapped_controls"
        );
        assert!(observed[0]["browser_cases"][0]["stdout"]
            .as_str()
            .unwrap()
            .contains("Authored result"));
        let (_fd, failed) = check_episode(&reopened, id)?;
        let failed_receipt = execute(&failed, &BrowserDouble { pass: complete })?;
        reopened.project_supplemental_link(id, &failed, failed_receipt)?;
        let cumulative = vec![
            typed::SupplementalBinding {
                admission: id,
                receipt,
            },
            typed::SupplementalBinding {
                admission: id,
                receipt: failed_receipt,
            },
        ];
        reopened.validate_supplemental_set("crossing", &cumulative)?;
        for bad in [
            vec![],
            vec![cumulative[1].clone()],
            vec![cumulative[1].clone(), cumulative[0].clone()],
            vec![cumulative[0].clone(), cumulative[0].clone()],
        ] {
            assert!(reopened
                .validate_supplemental_set("crossing", &bad)
                .is_err());
        }
        let view = reopened.project_supplemental_view()?;
        assert!(view["current_observations"]
            .as_array()
            .unwrap()
            .iter()
            .all(|v| v["findings"]["crossing"] == "unknown"));
        assert_eq!(
            serde_json::to_value(reopened.project_configuration()?)?,
            serde_json::to_value(&original)?
        );
        for (observed_receipt, available) in [(receipt, complete)] {
            use super::super::super::super::criterion_return::{
                CriterionDomain, Policy, ScopePart,
            };
            let key = format!(
                "supplemental:{id}:{}",
                if cumulative_basis {
                    observed_receipt
                } else {
                    failed_receipt
                }
            );
            let requirements = reopened.project_acceptance_requirements_view()?;
            let coverage = BTreeMap::from([(
                "crossing".into(),
                BTreeSet::from([key.clone(), format!("supplemental:{id}:{failed_receipt}")]),
            )]);
            let prerequisites = BTreeMap::from([(
                "crossing".into(),
                if cumulative_basis {
                    typed::Prerequisite::ObservedSupplementalSet {
                        observations: cumulative.clone(),
                    }
                } else {
                    typed::Prerequisite::ObservedSupplemental {
                        admission: id,
                        receipt: failed_receipt,
                    }
                },
            )]);
            let policy = Policy {
                state_root: reopened.store.head_root()?,
                requirements_revision: state.requirements_revision.unwrap(),
                requirements: requirements.clone(),
                assessment_scope: Some(BTreeMap::from([(
                    "crossing".into(),
                    vec![ScopePart {
                        domain: CriterionDomain::Artifact,
                        text: requirements["crossing"].clone(),
                    }],
                )])),
                coverage: coverage.clone(),
                prerequisites: prerequisites.clone(),
            };
            let mut bad = policy.clone();
            bad.assessment_scope = None;
            assert!(reopened.project_criterion_policy(bad).is_err());
            let mut bad = policy.clone();
            bad.coverage.get_mut("crossing").unwrap().clear();
            assert!(reopened.project_criterion_policy(bad).is_err());
            let frozen_policy = policy.clone();
            let pid = reopened.project_criterion_policy(policy)?;
            let mut premature = frozen_policy;
            premature.state_root = reopened.store.head_root()?;
            if let typed::Prerequisite::ObservedSupplementalSet { observations } =
                premature.prerequisites.get_mut("crossing").unwrap()
            {
                observations.push(cumulative[0].clone());
            }
            if cumulative_basis {
                assert!(reopened
                    .project_criterion_policy_refresh(premature, pid)
                    .unwrap_err()
                    .to_string()
                    .contains("assessment"));
            }
            let a = Admission {
                version: "verifier-admission@v2".into(),
                scope: state.snapshot,
                requirements_revision: state.requirements_revision.unwrap(),
                verifier_identity: "authored independent semantic review".into(),
                generator_identity: "authored fixture".into(),
                artifact: Hash::of(&[b"authored"]),
                coverage: coverage.clone(),
                prerequisites,
                max_requests: 1,
                tokens_per_request: 1,
                timeout_ms: 1,
                expires_at: String::new(),
            };
            let request = Request {
                version: "verifier-request@v2".into(),
                admission: pid,
                ordinal: 1,
                factual_root: reopened.store.head_root()?,
                snapshot: state.snapshot,
                requirements_revision: state.requirements_revision.unwrap(),
                test_receipt: None,
                requirements,
                evidence: BTreeMap::new(),
                evidence_digest: Hash::of(&[b"authored"]),
                observed_tests: None,
                supplemental: reopened.supplemental_current()?,
            };
            let report = Report {
                findings: BTreeMap::from([(
                    "crossing".into(),
                    Assessment {
                        finding: Finding::Met,
                        evidence: BTreeSet::from([key]),
                        explanation:
                            "Authored independent judgment, never inferred from protocol passage"
                                .into(),
                        limitations: "Exact finite artifact scope only".into(),
                        basis: Some(if cumulative_basis {
                            typed::Basis::ObservedSupplementalSet {
                                observations: cumulative.clone(),
                            }
                        } else {
                            typed::Basis::ObservedSupplemental {
                                admission: id,
                                receipt: failed_receipt,
                            }
                        }),
                    },
                )]),
            };
            assert!(reopened
                .validate_typed_report(&request, &a, &report)
                .is_err()); // Generic unscoped route stays closed.
            assert_eq!(
                reopened
                    .validate_criterion_report(&request, &a, &report)
                    .is_ok(),
                available
            );
            let mut unknown = report.clone();
            unknown.findings.get_mut("crossing").unwrap().finding = Finding::Unknown;
            // Authenticated unavailable evidence may be cited without asserting a result.
            reopened.validate_criterion_report(&request, &a, &unknown)?;
            let mut negative = report.clone();
            negative.findings.get_mut("crossing").unwrap().finding = Finding::NotMet;
            assert_eq!(
                reopened
                    .validate_criterion_report(&request, &a, &negative)
                    .is_ok(),
                available
            );
            for bad_bindings in [
                vec![],
                vec![cumulative[0].clone()],
                vec![cumulative[1].clone(), cumulative[0].clone()],
                vec![typed::SupplementalBinding {
                    admission: id,
                    receipt: EventId(Hash::of(&[b"forged"])),
                }],
            ] {
                let mut bad = unknown.clone();
                bad.findings.get_mut("crossing").unwrap().basis =
                    Some(typed::Basis::ObservedSupplementalSet {
                        observations: bad_bindings,
                    });
                assert!(reopened
                    .validate_criterion_report(&request, &a, &bad)
                    .is_err());
            }
            unknown.findings.get_mut("crossing").unwrap().basis = Some(typed::Basis::Unknown);
            reopened.validate_criterion_report(&request, &a, &unknown)?;
        }
        let mut pending_bundle = reopened.supplemental_bundle(id)?;
        pending_bundle
            .review
            .push_str(" Additional registered but unobserved authored check");
        reopened.project_supplemental_admit(pending_bundle)?;
        assert!(reopened
            .validate_supplemental_set("crossing", &cumulative)
            .unwrap_err()
            .to_string()
            .contains("lacks observation"));
        reopened.project_supplemental_revoke(
            id,
            "Authored revocation must not conceal returned adverse evidence".into(),
        )?;
        assert!(reopened
            .validate_supplemental_set("crossing", &cumulative)
            .is_err());
        std::fs::write(d.path().join("coding-work/app.js"), "// changed source")?;
        assert!(reopened.project_supplemental_spec(id).is_err());
        Ok(())
    }

    #[test]
    fn supplemental_finite_results_restart_replay_and_zero_double_charge() -> Result<()> {
        for (stdout, exit, expected) in [
            (b"12\n".as_slice(), 0, Finding::Unknown),
            (b"7\n".as_slice(), 0, Finding::NotMet),
            (b"".as_slice(), 124, Finding::Unknown),
        ] {
            let (d, e, _) = super::super::tests::setup()?;
            let original = e.project_configuration()?;
            let id = e.project_supplemental_admit(bundle(&e)?)?;
            let (_od, other) = check_episode(&e, id)?;
            let before = serde_json::to_value(e.project_budget()?)?;
            let receipt = execute(&other, &Fake { stdout, exit })?;
            let observed = e.project_supplemental_link(id, &other, receipt)?;
            assert_eq!(e.project_supplemental_link(id, &other, receipt)?, observed);
            let reopened = Engine::open(d.path())?;
            let view = reopened.project_supplemental_view()?;
            assert_eq!(
                view["records"][1]["observation"]["findings"]["subtotal"],
                serde_json::to_value(expected)?
            );
            assert_eq!(view["records"][1]["source_current"], true);
            assert_eq!(
                view["records"][1]["observation"]["cases"]["multiply"],
                if exit != 0 {
                    "unknown"
                } else if stdout == b"12\n" {
                    "passed_finite_case"
                } else {
                    "counterexample"
                }
            );
            assert_eq!(before, serde_json::to_value(e.project_budget()?)?);
            assert_eq!(
                serde_json::to_value(original)?,
                serde_json::to_value(e.project_configuration()?)?
            );
            assert!(!e.project_acceptance_status()?.accepted);
            assert!(e.project_acceptance_status()?.test_receipt.is_none());
            std::fs::write(d.path().join("coding-work/src/pricing.rs"), "// changed")?;
            assert_eq!(
                e.project_supplemental_view()?["records"][1]["source_current"],
                false
            );
            assert!(e.project_supplemental_link(id, &other, receipt).is_err());
        }
        Ok(())
    }
    #[test]
    fn supplemental_mapping_scope_identity_and_missing_evidence_fail_closed() -> Result<()> {
        let (_d, e, _) = super::super::tests::setup()?;
        let b = bundle(&e)?;
        for mutation in 0..8 {
            let mut bad = b.clone();
            match mutation {
                0 => bad.reviewer = bad.author.clone(),
                1 => bad.author = bad.generator.clone(),
                2 => bad.snapshot = Hash::of(&[b"stale"]),
                3 => bad.requirements_revision = EventId(Hash::of(&[b"stale"])),
                4 => {
                    bad.mapping
                        .get_mut("subtotal")
                        .unwrap()
                        .cases
                        .insert("missing".into());
                }
                5 => {
                    bad.harness
                        .insert("src/pricing.rs".into(), "overwrite".into());
                }
                6 => {
                    bad.harness
                        .insert("supplemental/../src/pricing.rs".into(), "overwrite".into());
                }
                _ => bad.mapping.get_mut("subtotal").unwrap().limits.clear(),
            }
            assert!(
                e.project_supplemental_admit(bad).is_err(),
                "mutation {mutation}"
            );
        }
        let id = e.project_supplemental_admit(b)?;
        let (od, other) = check_episode(&e, id)?;
        assert!(e
            .project_supplemental_link(id, &other, EventId(Hash::of(&[b"invented"])))
            .is_err());
        let receipt = execute(
            &other,
            &Fake {
                stdout: b"12\n",
                exit: 0,
            },
        )?;
        std::fs::write(
            od.path().join("coding-work/supplemental/check.rs"),
            "// drift",
        )?;
        assert!(e.project_supplemental_link(id, &other, receipt).is_err());
        Ok(())
    }
    #[test]
    fn supplemental_counterexample_invalidates_frozen_request_and_vetoes_met() -> Result<()> {
        let (_d, e, a) = super::super::tests::setup()?;
        let verifier = e.project_verifier_admit(a.clone())?;
        let (_, frozen) = e.project_verifier_request(verifier)?;
        let id = e.project_supplemental_admit(bundle(&e)?)?;
        // Check admission alone is bookkeeping, not new observed evidence.
        e.verifier_fresh(&frozen)?;
        let (_od, other) = check_episode(&e, id)?;
        let receipt = execute(
            &other,
            &Fake {
                stdout: b"7\n",
                exit: 0,
            },
        )?;
        e.project_supplemental_link(id, &other, receipt)?;
        assert!(e.verifier_fresh(&frozen).is_err());
        let (_, fresh) = e.project_verifier_request(verifier)?;
        assert_eq!(fresh.supplemental.len(), 1);
        let mut report = Report {
            findings: a
                .coverage
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        Assessment {
                            finding: Finding::Met,
                            evidence: v.clone(),
                            explanation: "Synthetic semantic assertion".into(),
                            limitations: "No operational evidence".into(),
                            basis: None,
                        },
                    )
                })
                .collect(),
        };
        assert!(e.validate_verifier_report(&fresh, &a, &report).is_err());
        report.findings.get_mut("subtotal").unwrap().finding = Finding::Unknown;
        e.validate_verifier_report(&fresh, &a, &report)?;
        report.findings.get_mut("subtotal").unwrap().finding = Finding::NotMet;
        e.validate_verifier_report(&fresh, &a, &report)?;
        Ok(())
    }
    #[test]
    fn supplemental_wrong_receipt_source_and_case_binding_are_rejected() -> Result<()> {
        for tamper in 0..5 {
            let (_d, e, _) = super::super::tests::setup()?;
            let id = e.project_supplemental_admit(bundle(&e)?)?;
            let (_od, other) = check_episode(&e, id)?;
            let receipt = execute(
                &other,
                &Fake {
                    stdout: b"7\n",
                    exit: 0,
                },
            )?;
            // Additional adversarial records, never rewrite original immutable receipts.
            let mut record = other
                .records()?
                .into_iter()
                .find(|(id, _)| *id == receipt)
                .unwrap()
                .1;
            if let AdaptiveRecord::Coding {
                record:
                    CodingRecord::Receipt {
                        source_after,
                        observations,
                        commitment,
                        ..
                    },
            } = &mut record
            {
                match tamper {
                    0 => *source_after = Some(Hash::of(&[b"wrong"])),
                    1 => observations[0].argv.push("injected".into()),
                    2 => observations.push(observations[0].clone()),
                    3 => *commitment = EventId(Hash::of(&[b"invented"])),
                    _ => (),
                }
            }
            let forged = other.adaptive_record(
                &record,
                if tamper == 4 {
                    Source::Generator
                } else {
                    Source::Rules
                },
            )?;
            let executed = other
                .events()?
                .into_iter()
                .find(|(_, event)| matches!(event.kind, EventKind::Executed { .. }))
                .unwrap()
                .1
                .kind;
            other.append(executed, Source::Rules)?;
            assert!(
                e.project_supplemental_link(id, &other, forged).is_err(),
                "tamper {tamper}"
            );
        }
        Ok(())
    }
    #[test]
    fn supplemental_verification_requires_fresh_resolved_budget() -> Result<()> {
        let (_d, e, _) = super::super::tests::setup()?;
        let id = e.project_supplemental_admit(bundle(&e)?)?;
        let d = tempfile::tempdir()?;
        let other = Engine::open(d.path())?;
        other.project_initialize(e.project_supplemental_spec(id)?)?;
        let source = other.project_acceptance_status()?.snapshot;
        assert!(other
            .project_admit_verification(other.store.head_root()?, source)
            .is_err());
        other.project_lifecycle_open(lifecycle::Admission {
            training: None,
            state_root: other.store.head_root()?,
            tool_ms_ceiling: 1,
            reason: "Too small synthetic budget".into(),
            non_tool: Some(budget::Limits {
                model_calls: 0,
                model_tokens: 0,
                wall_ms: 120_000,
                patch_bytes: 0,
                external_cost_microusd: 0,
            }),
        })?;
        assert!(other
            .project_admit_verification(other.store.head_root()?, source)
            .is_err());
        Ok(())
    }
    #[test]
    fn supplemental_conflict_reopens_accepted_projection_and_revocation_preserves_history(
    ) -> Result<()> {
        for failed in [false, true] {
            let (_d, e, _) = super::super::tests::setup()?;
            let c = e.project_acceptance_status()?;
            // Synthetic pre-existing protected acceptance receipt, no real code execution.
            let receipt = e.adaptive_record(
                &AdaptiveRecord::Coding {
                    record: CodingRecord::Receipt {
                        commitment: EventId(Hash::of(&[b"synthetic"])),
                        verdict: Verdict::Passed,
                        runs: vec![],
                        source_after: Some(c.snapshot),
                        observations: vec![],
                    },
                },
                Source::Rules,
            )?;
            let review = Review {
                snapshot: c.snapshot,
                requirements_revision: c.requirements_revision.unwrap(),
                test_receipt: Some(receipt),
                decision: Decision::Accept,
                evidence: e
                    .project_acceptance_requirements_view()?
                    .keys()
                    .map(|k| {
                        (
                            k.clone(),
                            Evidence {
                                met: true,
                                evidence: "Synthetic prior independent acceptance fixture".into(),
                            },
                        )
                    })
                    .collect(),
            };
            let accepted = e.project_acceptance_review(review.clone())?;
            assert!(accepted.accepted);
            let id = e.project_supplemental_admit(bundle(&e)?)?;
            let (_od, other) = check_episode(&e, id)?;
            let receipt = execute(
                &other,
                &Fake {
                    stdout: if failed { b"7\n" } else { b"12\n" },
                    exit: 0,
                },
            )?;
            e.project_supplemental_link(id, &other, receipt)?;
            let current = e.project_acceptance_status()?;
            assert_eq!(current.accepted, !failed);
            assert_eq!(e.project_export()?["goal_accepted"], !failed);
            assert_eq!(current.decision_record, accepted.decision_record);
            if failed {
                assert_eq!(current.status, "supplemental_conflict_review_required");
                assert!(e.project_acceptance_review(review).is_err());
                assert!(e.adaptive_state()?.preferences.iter().any(|p| p.is_open()));
            }
            e.project_supplemental_revoke(
                id,
                "Synthetic mapping withdrawal; preserves receipts".into(),
            )?;
            assert!(e.project_acceptance_status()?.accepted);
            assert!(e.project_supplemental_link(id, &other, receipt).is_err());
            assert!(e.supplemental_current()?.is_empty());
            assert!(e
                .verifier_records()?
                .iter()
                .any(|(_, r)| matches!(r, Record::SupplementalObserved { .. })));
        }
        Ok(())
    }
    #[test]
    fn supplemental_typed_basis_requires_exact_finite_evidence_and_never_proves_met() -> Result<()>
    {
        for failed in [false, true] {
            let (_d, e, mut a) = super::super::tests::setup()?;
            let id = e.project_supplemental_admit(bundle(&e)?)?;
            let (_od, other) = check_episode(&e, id)?;
            let receipt = execute(
                &other,
                &Fake {
                    stdout: if failed { b"7\n" } else { b"12\n" },
                    exit: 0,
                },
            )?;
            e.project_supplemental_link(id, &other, receipt)?;
            a.version = "verifier-admission@v2".into();
            a.prerequisites = BTreeMap::from([
                (
                    "subtotal".into(),
                    typed::Prerequisite::SupplementalFinite { admission: id },
                ),
                ("discount".into(), typed::Prerequisite::StaticSemantic),
            ]);
            let admitted = e.project_verifier_admit(a.clone())?;
            let (_, mut request) = e.project_verifier_request(admitted)?;
            let mut report = Report {
                findings: a
                    .coverage
                    .iter()
                    .map(|(k, v)| {
                        (
                            k.clone(),
                            Assessment {
                                finding: Finding::Unknown,
                                evidence: v.clone(),
                                explanation: "Synthetic evidence contract".into(),
                                limitations: "Finite cases only".into(),
                                basis: Some(typed::Basis::Unknown),
                            },
                        )
                    })
                    .collect(),
            };
            let x = report.findings.get_mut("subtotal").unwrap();
            x.basis = Some(typed::Basis::SupplementalFinite {
                admission: id,
                receipt,
            });
            e.validate_verifier_report(&request, &a, &report)?;
            report.findings.get_mut("subtotal").unwrap().finding = Finding::Met;
            assert!(e.validate_verifier_report(&request, &a, &report).is_err());
            report.findings.get_mut("subtotal").unwrap().finding = Finding::NotMet;
            assert_eq!(
                e.validate_verifier_report(&request, &a, &report).is_ok(),
                failed
            );
            report.findings.get_mut("subtotal").unwrap().basis =
                Some(typed::Basis::SupplementalFinite {
                    admission: id,
                    receipt: EventId(Hash::of(&[b"fabricated"])),
                });
            assert!(e.validate_verifier_report(&request, &a, &report).is_err());
            report.findings.get_mut("subtotal").unwrap().basis =
                Some(typed::Basis::SupplementalFinite {
                    admission: id,
                    receipt,
                });
            request.supplemental[0]
                .findings
                .insert("subtotal".into(), Finding::NotMet);
            if !failed {
                assert!(e.validate_verifier_report(&request, &a, &report).is_err());
            }
        }
        Ok(())
    }
}
