//! Explicit operator-reviewed disposal of an interrupted read-only observation tail.
//! Never synthesizes a test result, measured duration, acceptance, or permission to replay.
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Attestation {
    pub native_pid: u32,
    pub outer_pid: u32,
    pub observed_at: String,
    pub processes_absent: bool,
    pub containers_absent: bool,
    pub container_prefix: String,
    pub artifacts: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub version: String,
    pub directory: String,
    pub state_root: Hash,
    pub commitment: EventId,
    pub intent: EventId,
    pub source_after: Hash,
    pub configuration: Hash,
    pub plan: Hash,
    pub receipts_digest: Hash,
    pub attestation: Attestation,
    pub artifact_hashes: BTreeMap<String, Hash>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub decision: String,
    pub evidence: Hash,
    pub reviewer: String,
    pub report: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Disposition {
    pub evidence: Evidence,
    pub review: Review,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Prepare {
    pub commitment: EventId,
    pub attestation: Attestation,
}

fn prefix(plan: &Plan, bytes: &[u8], evidence: &Path, native_pid: u32) -> Result<Vec<String>> {
    // This narrowly supported suffix has no writable workspace or network. Other
    // runtimes/commands require their own reconciliation, never this disposition.
    ensure!(
        plan.commands.len() == 3
            && plan.commands[1] == strings(&["node", "--check", "browser-harness.cjs"])
            && plan.commands[2]
                == strings(&[
                    "node",
                    "browser-harness.cjs",
                    "probe.json",
                    "--container-only"
                ]),
        "unsupported partial execution suffix"
    );
    let lines: Vec<_> = std::str::from_utf8(bytes)?.lines().collect();
    ensure!(
        lines.len() == 2,
        "exact successful writable prefix and syntax receipt required"
    );
    let mut ids = vec![];
    for (i, line) in lines.iter().enumerate() {
        let value: serde_json::Value = serde_json::from_str(line)?;
        ensure!(
            value["snapshot"] == serde_json::json!(plan.source_before)
                && value["argv"] == serde_json::json!(plan.commands[i])
                && value["preparation_id"].is_null(),
            "receipt prefix/source mismatch"
        );
        let run: RunResult = serde_json::from_value(value["result"].clone())?;
        let execution = run
            .execution
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("runtime diagnostics missing"))?;
        ensure!(
            run.exit == 0
                && execution.version == 1
                && execution.termination == "runtime_exit"
                && execution.runtime_exit == Some(0)
                && execution.runtime_signal.is_none()
                && execution.stdout_complete
                && execution.stderr_complete,
            "prefix did not complete successfully"
        );
        ensure!(
            execution
                .worker
                .starts_with(&format!("tsr-m3-{native_pid}-{i}-"))
                && execution
                    .worker
                    .rsplit('-')
                    .next()
                    .is_some_and(|n| n.parse::<u64>().is_ok()),
            "receipt invocation mismatch"
        );
        let id = tsr_corpus::digest(line.as_bytes());
        ensure!(
            fs::read(evidence.join("cas").join(&id))? == line.as_bytes(),
            "receipt CAS mismatch"
        );
        for digest in [&run.stdout_digest, &run.stderr_digest] {
            ensure!(
                digest.len() == 64 && digest.bytes().all(|c| c.is_ascii_hexdigit()),
                "invalid stream identity"
            );
            ensure!(
                tsr_corpus::digest(&fs::read(evidence.join("cas").join(digest))?) == *digest,
                "stream CAS mismatch"
            );
        }
        ids.push(id);
    }
    Ok(ids)
}
impl Engine {
    pub fn project_partial_execution_evidence(&self, input: Prepare) -> Result<serde_json::Value> {
        let (evidence, _) = self.partial_evidence(input)?;
        Ok(
            serde_json::json!({"digest":Hash::of(&[&serde_json::to_vec(&evidence)?]),"evidence":evidence}),
        )
    }
    fn partial_evidence(&self, input: Prepare) -> Result<(Evidence, Vec<String>)> {
        let a = &input.attestation;
        ensure!(
            a.native_pid > 1
                && a.outer_pid > 1
                && a.native_pid != a.outer_pid
                && a.processes_absent
                && a.containers_absent
                && !a.observed_at.is_empty()
                && a.container_prefix == format!("tsr-m3-{}-", a.native_pid)
                && a.artifacts.len() >= 3,
            "explicit operator quiescence attestation required"
        );
        let mut artifact_hashes = BTreeMap::new();
        for path in &a.artifacts {
            let p = Path::new(path);
            ensure!(p.is_absolute() && !p.is_symlink(), "host evidence path");
            ensure!(
                artifact_hashes
                    .insert(path.clone(), Hash::of(&[&fs::read(p)?]))
                    .is_none(),
                "duplicate host artifact"
            );
        }
        let spec = self.project_configuration()?;
        ensure!(
            spec.browser
                && spec.browser_container_only
                && !spec.cargo
                && spec.named_package.is_none(),
            "only sealed browser read-only tail supported"
        );
        let records = self.records()?;
        ensure!(!records.iter().any(|(_,r)|matches!(r,Record::Coding{record:CodingRecord::PartialDisposition{input:p,..}} if p.evidence.commitment==input.commitment)),"partial disposition already recorded");
        let (intent, plan) = records
            .iter()
            .find_map(|(id, r)| match r {
                Record::Coding {
                    record: CodingRecord::Intent { plan },
                } if plan.commitment == input.commitment => Some((*id, plan)),
                _ => None,
            })
            .ok_or_else(|| anyhow::anyhow!("execution intent missing"))?;
        self.require_project_bridge_source(intent, Source::Rules)?;
        ensure!(
            plan.generated
                && plan.image == spec.image
                && plan.user == spec.user
                && plan.timeout_ms == spec.timeout_ms
                && plan.protected_tests == Hash::of(&[&serde_json::to_vec(&spec.protected)?]),
            "execution configuration mismatch"
        );
        ensure!(
            self.adaptive_state_inner()?
                .pending
                .is_some_and(|p| p.commitment == input.commitment),
            "pending commitment mismatch"
        );
        let source_after = digest(&self.project_snapshot(&spec)?)?;
        ensure!(
            plan.expected_after == Some(source_after),
            "partial source/protected files changed"
        );
        let dir = self
            .directory
            .join("coding-receipts")
            .join(input.commitment.to_string());
        let bytes = fs::read(dir.join("runs.jsonl"))?;
        let receipts = prefix(plan, &bytes, &dir, a.native_pid)?;
        let evidence = Evidence {
            version: "partial-readonly-tail-proof@v1".into(),
            directory: self.directory.canonicalize()?.to_string_lossy().into(),
            state_root: self.store.head_root()?,
            commitment: input.commitment,
            intent,
            source_after,
            configuration: Hash::of(&[&serde_json::to_vec(&spec)?]),
            plan: Hash::of(&[&serde_json::to_vec(plan)?]),
            receipts_digest: Hash::of(&[&bytes]),
            attestation: input.attestation,
            artifact_hashes,
        };
        Ok((evidence, receipts))
    }
    pub fn project_dispose_partial_execution(
        &self,
        input: Disposition,
    ) -> Result<serde_json::Value> {
        self.write(|| {
            let (current,receipts)=self.partial_evidence(Prepare{commitment:input.evidence.commitment,attestation:input.evidence.attestation.clone()})?;
            ensure!(current==input.evidence,"stale partial disposition evidence");
            ensure!(input.review.decision=="partial_observation_unavailable"
                && input.review.evidence==Hash::of(&[&serde_json::to_vec(&current)?])
                && short(&input.review.reviewer) && !input.review.report.trim().is_empty(),"independent disposition review binding required");
            let id=self.adaptive_record(&Record::Coding{record:CodingRecord::PartialDisposition{input:Box::new(input.clone()),receipts:receipts.clone()}},Source::Operator)?;
            self.adaptive_record(&Record::Censored{commitment:current.commitment,reason:format!("Reviewed partial execution {id}: mutating prefix completed; read-only observation unavailable. Full reservation retained; no success/failure learning or replay.")},Source::Rules)?;
            Ok(serde_json::json!({"disposition":id,"commitment":current.commitment,"source_after":current.source_after,"receipts":receipts,"evidence":input.evidence,"review":input.review,"observation":"unavailable","duration":"unmeasured_full_reservation_charged","replayed":false}))
        })
    }
    pub fn project_partial_execution_dispositions(&self) -> Result<Vec<serde_json::Value>> {
        let mut result = vec![];
        for (id, r) in self.records()? {
            if let Record::Coding {
                record: CodingRecord::PartialDisposition { input, receipts },
            } = r
            {
                self.require_project_bridge_source(id, Source::Operator)?;
                result.push(serde_json::json!({"disposition":id,"commitment":input.evidence.commitment,"source_after":input.evidence.source_after,"receipts":receipts,"evidence":input.evidence,"review":input.review,"observation":"unavailable","duration":"unmeasured_full_reservation_charged","replayed":false}));
            }
        }
        Ok(result)
    }
    pub(super) fn disposed_partial_commitments(&self) -> Result<BTreeSet<EventId>> {
        let mut result = BTreeSet::new();
        for (id, r) in self.records()? {
            if let Record::Coding {
                record: CodingRecord::PartialDisposition { input, .. },
            } = r
            {
                self.require_project_bridge_source(id, Source::Operator)?;
                ensure!(
                    result.insert(input.evidence.commitment),
                    "duplicate partial disposition"
                );
            }
        }
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn authored() -> Result<(tempfile::TempDir, Plan, Vec<u8>)> {
        let d = tempfile::tempdir()?;
        let plan = Plan {
            commitment: serde_json::from_value(serde_json::json!("01".repeat(32)))?,
            action: "authored".into(),
            source_before: Hash::of(&[b"before"]),
            replacement: String::new(),
            image: "pinned".into(),
            user: "501:20".into(),
            commands: vec![
                strings(&["sh", "-c", "authored scoped edit"]),
                strings(&["node", "--check", "browser-harness.cjs"]),
                strings(&[
                    "node",
                    "browser-harness.cjs",
                    "probe.json",
                    "--container-only",
                ]),
            ],
            timeout_ms: 100,
            protected_tests: Hash::of(&[b"protected"]),
            generated: true,
            edits: vec![],
            cases: vec![],
            expected_after: Some(Hash::of(&[b"after"])),
        };
        for argv in &plan.commands[..2] {
            let mut r = RunResult::from_output(0, b"", b"", 1);
            r.execution = Some(tsr_corpus::ExecutionDiagnostics {
                version: 1,
                termination: "runtime_exit".into(),
                runtime_exit: Some(0),
                runtime_signal: None,
                stdout_complete: true,
                stderr_complete: true,
                worker: format!(
                    "tsr-m3-42-{}-123",
                    if argv == &plan.commands[0] { 0 } else { 1 }
                ),
            });
            tsr_corpus::persist_run(d.path(), &plan.source_before.to_string(), argv, &r, None)
                .map_err(anyhow::Error::msg)?;
        }
        let bytes = fs::read(d.path().join("runs.jsonl"))?;
        Ok((d, plan, bytes))
    }
    #[test]
    fn partial_prefix_is_not_a_success_receipt() -> Result<()> {
        let (d, p, b) = authored()?;
        assert_eq!(prefix(&p, &b, d.path(), 42)?.len(), 2);
        let mut unsafe_plan = p.clone();
        unsafe_plan.commands[2] = strings(&["sh", "-c", "touch source"]);
        assert!(prefix(&unsafe_plan, &b, d.path(), 42).is_err());
        let first = std::str::from_utf8(&b)?.lines().next().unwrap();
        assert!(prefix(&p, first.as_bytes(), d.path(), 42).is_err());
        let mut cross = p.clone();
        cross.source_before = Hash::of(&[b"different"]);
        assert!(prefix(&cross, &b, d.path(), 42).is_err());
        let mut lines: Vec<serde_json::Value> = std::str::from_utf8(&b)?
            .lines()
            .map(serde_json::from_str)
            .collect::<std::result::Result<_, _>>()?;
        lines[0]["result"]["exit"] = serde_json::json!(1);
        let changed = lines
            .iter()
            .map(|v| serde_json::to_string(v).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        assert!(prefix(&p, changed.as_bytes(), d.path(), 42).is_err());
        fs::write(
            d.path().join("cas").join(tsr_corpus::digest(b"")),
            b"changed",
        )?;
        assert!(prefix(&p, &b, d.path(), 42).is_err());
        Ok(())
    }
    #[test]
    fn authored_partial_disposition_preserves_history_and_charges() -> Result<()> {
        use crate::engine::adaptive::provider::{Response, Suggestion, PROTOCOL};
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let mut spec = browser::tests::spec();
        spec.browser_container_only = true;
        e.project_initialize(spec.clone())?;
        e.project_acceptance_requirements(BTreeMap::from([(
            "playability".into(),
            "Authored criterion".into(),
        )]))?;
        e.project_lifecycle_open(lifecycle::Admission {
            state_root: e.store.head_root()?,
            training: None,
            tool_ms_ceiling: 100000,
            reason: "Authored partial recovery fixture".into(),
            non_tool: Some(budget::Limits {
                model_calls: 10,
                model_tokens: 10000,
                wall_ms: 600000,
                patch_bytes: 10000,
                external_cost_microusd: 0,
            }),
        })?;
        let before = e.project_acceptance_status()?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: before.requirements_revision.unwrap(),
            decision_record: None,
            guidance: BTreeMap::from([("playability".into(), "Authored legibility aim".into())]),
        })?;
        let snapshot = e.project_snapshot(&spec)?;
        let patch = ExactPatch {
            path: "index.html".into(),
            source_digest: Hash::of(&[snapshot["index.html"].as_bytes()]),
            inspection_handle: e.project_handle(&spec, &snapshot, "index.html")?,
            old_text: "Start".into(),
            new_text: "Play".into(),
        };
        let response = Response {
            protocol: PROTOCOL.into(),
            state_root: e.store.head_root()?,
            attention_binding: e.proposal_attention_binding()?,
            pursuit_expectations: Default::default(),
            common_event_forecasts: Default::default(),
            proposals: vec![Suggestion::EditSet {
                patches: vec![patch],
                rationale: "Authored test proposal".into(),
            }],
        };
        let preview = e.project_cycle_preview(response.clone())?;
        let cy = &preview["cycle"];
        e.project_cycle_advance(cycle::Advance {
            binding: serde_json::from_value(cy["binding"].clone())?,
            review: cycle::Review {
                binding: serde_json::from_value(cy["binding"].clone())?,
                response_digest: serde_json::from_value(preview["response_digest"].clone())?,
                state_root: response.state_root,
                source_snapshot: serde_json::from_value(cy["source_snapshot"].clone())?,
                reviewer: "authored independent review".into(),
                generator: "authored fixture".into(),
                decision: "approve".into(),
            },
            response,
        })?;
        let plan = e.coding_plan()?;
        e.write(|| {
            e.reserve_project_execution(&plan)?;
            e.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::Intent { plan: plan.clone() },
                },
                Source::Rules,
            )?;
            Ok(())
        })?;
        for edit in &plan.edits {
            fs::write(
                dir.path().join("coding-work").join(&edit.path),
                &edit.replacement,
            )?;
        }
        let receipts = dir
            .path()
            .join("coding-receipts")
            .join(plan.commitment.to_string());
        for (i, argv) in plan.commands[..2].iter().enumerate() {
            let mut run = RunResult::from_output(0, b"", b"", 1);
            run.execution = Some(tsr_corpus::ExecutionDiagnostics {
                version: 1,
                termination: "runtime_exit".into(),
                runtime_exit: Some(0),
                runtime_signal: None,
                stdout_complete: true,
                stderr_complete: true,
                worker: format!("tsr-m3-42-{i}-123"),
            });
            tsr_corpus::persist_run(&receipts, &plan.source_before.to_string(), argv, &run, None)
                .map_err(anyhow::Error::msg)?;
        }
        let mut artifacts = vec![];
        for n in 0..3 {
            let p = dir.path().join(format!("authored-proof-{n}"));
            fs::write(&p, b"Authored process evidence; no actual process executed")?;
            artifacts.push(p.to_string_lossy().into());
        }
        let prepare = Prepare {
            commitment: plan.commitment,
            attestation: Attestation {
                native_pid: 42,
                outer_pid: 43,
                observed_at: "authored fixture".into(),
                processes_absent: true,
                containers_absent: true,
                container_prefix: "tsr-m3-42-".into(),
                artifacts,
            },
        };
        let proof = e.project_partial_execution_evidence(prepare.clone())?;
        let input = Disposition {
            evidence: serde_json::from_value(proof["evidence"].clone())?,
            review: Review {
                decision: "partial_observation_unavailable".into(),
                evidence: serde_json::from_value(proof["digest"].clone())?,
                reviewer: "authored separate reviewer".into(),
                report: "Authored review of partial read-only-tail disposal, not test success"
                    .into(),
            },
        };
        let root = e.store.head_root()?;
        let reserved = e.project_budget()?.tool_ms_charged;
        assert_eq!(e.project_budget()?.unknown_usage, 1);
        let mut bad = input.clone();
        bad.review.evidence = Hash::of(&[b"wrong"]);
        assert!(e.project_dispose_partial_execution(bad).is_err());
        assert_eq!(e.store.head_root()?, root);
        let mut bad = prepare.clone();
        bad.attestation.native_pid = 99;
        bad.attestation.container_prefix = "tsr-m3-99-".into();
        assert!(e.project_partial_execution_evidence(bad).is_err());
        let source = dir.path().join("coding-work/index.html");
        let saved = fs::read(&source)?;
        fs::write(&source, b"changed")?;
        assert!(e.project_dispose_partial_execution(input.clone()).is_err());
        fs::write(&source, saved)?;
        let result = e.project_dispose_partial_execution(input.clone())?;
        assert_eq!(result["observation"], "unavailable");
        assert_eq!(result["replayed"], false);
        assert!(e.adaptive_state_inner()?.pending.is_none());
        assert_eq!(e.project_budget()?.unknown_usage, 0);
        assert_eq!(e.project_budget()?.tool_ms_charged, reserved);
        assert!(!e.project_acceptance_status()?.accepted);
        assert!(!e.records()?.iter().any(
            |(_, r)| matches!(r,Record::Returned{commitment,..} if *commitment==plan.commitment)
        ));
        assert!(!e.records()?.iter().any(|(_,r)|matches!(r,Record::Coding{record:CodingRecord::Receipt{commitment,..}} if *commitment==plan.commitment)));
        let restarted = Engine::open(dir.path())?;
        assert_eq!(restarted.project_budget()?.tool_ms_charged, reserved);
        assert!(restarted.project_dispose_partial_execution(input).is_err());
        Ok(())
    }
}
