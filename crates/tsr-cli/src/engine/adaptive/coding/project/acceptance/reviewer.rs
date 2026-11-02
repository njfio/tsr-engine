//! Quarantined verifier observations, opt-in local transport, no automatic acceptance.
use super::super::super::Record as AdaptiveRecord;
use super::*;
pub mod local;
pub mod supplemental;
pub mod typed;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Admission {
    pub version: String,
    pub scope: Hash,
    pub requirements_revision: EventId,
    pub verifier_identity: String,
    pub generator_identity: String,
    pub artifact: Hash,
    pub coverage: BTreeMap<String, BTreeSet<String>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub prerequisites: BTreeMap<String, typed::Prerequisite>,
    pub max_requests: u32,
    pub tokens_per_request: u64,
    pub timeout_ms: u64,
    pub expires_at: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub version: String,
    pub admission: EventId,
    pub ordinal: u32,
    pub factual_root: Hash,
    pub snapshot: Hash,
    pub requirements_revision: EventId,
    pub test_receipt: Option<EventId>,
    pub requirements: BTreeMap<String, String>,
    pub evidence: BTreeMap<String, String>,
    pub evidence_digest: Hash,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed_tests: Option<typed::ObservedTests>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supplemental: Vec<supplemental::Observation>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Finding {
    Met,
    NotMet,
    Unknown,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Assessment {
    pub finding: Finding,
    pub evidence: BTreeSet<String>,
    pub explanation: String,
    pub limitations: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basis: Option<typed::Basis>,
}
/// Only findings come from the verifier; identity, bindings and usage do not.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub findings: BTreeMap<String, Assessment>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Record {
    SupplementalRevoked {
        admission: EventId,
        reason: String,
    },
    SupplementalAdmitted {
        bundle: supplemental::Bundle,
    },
    SupplementalObserved {
        observation: supplemental::Observation,
    },
    LocalConfigured {
        admission: EventId,
        config: local::Config,
    },
    LocalProgress {
        request: EventId,
        progress: local::Progress,
    },
    LocalCancel {
        request: EventId,
    },
    LocalReconciled {
        request: EventId,
        reason: String,
    },
    Admitted {
        admission: Admission,
    },
    Revoked {
        admission: EventId,
        reason: String,
    },
    Requested {
        request: Request,
    },
    Started {
        request: EventId,
        identity: String,
        artifact: Hash,
    },
    RejectedReport {
        request: EventId,
        report: Report,
        reason: String,
    },
    Finished {
        request: EventId,
        response_digest: Hash,
        report: Option<Report>,
        diagnostic: String,
        tokens: Option<u64>,
        elapsed_ms: Option<u64>,
    },
    Adopted {
        report: EventId,
        decision: Decision,
        unknown: BTreeSet<String>,
    },
}
/// Trusted host adapter, not a model-supplied identity or imported receipt.
/// An implementation must enforce the supplied deadline/output/token limits and
/// isolate its invocation/context from generation. Local transport requires explicit configuration.
pub trait Verifier {
    fn identity(&self) -> (&str, Hash);
    fn evaluate(&self, request: &Request, tokens: u64, timeout_ms: u64) -> Result<Output>;
}
pub struct Output {
    pub json: String,
    pub tokens: Option<u64>,
}

impl Engine {
    pub(crate) fn verifier_has_unfinished(&self) -> Result<bool> {
        let records = self.verifier_records()?;
        Ok(records.iter().any(|(_, r)| match r {
            Record::Started { request, .. } => !records
                .iter()
                .any(|(_, f)| matches!(f,Record::Finished{request:done,..} if done==request)),
            _ => false,
        }))
    }
    fn verifier_records(&self) -> Result<Vec<(EventId, Record)>> {
        // Read the validated prefix first. A later append-only event snapshot must
        // contain every record in that prefix, including concurrent cancellations.
        let records = self.records()?;
        let events = self.events()?;
        records
            .into_iter()
            .filter_map(|(id, r)| match r {
                AdaptiveRecord::Coding {
                    record: CodingRecord::Verifier { event },
                } => Some((id, event)),
                _ => None,
            })
            .map(|(id, r)| {
                let expected = if matches!(
                    r,
                    Record::Admitted { .. }
                        | Record::SupplementalAdmitted { .. }
                        | Record::SupplementalRevoked { .. }
                        | Record::Adopted { .. }
                        | Record::Revoked { .. }
                        | Record::LocalConfigured { .. }
                        | Record::LocalCancel { .. }
                        | Record::LocalReconciled { .. }
                ) {
                    Source::Operator
                } else {
                    Source::Rules
                };
                ensure!(
                    events
                        .iter()
                        .any(|(_, e)| e.id == id && e.source == expected),
                    "untrusted verifier record provenance"
                );
                Ok((id, r))
            })
            .collect()
    }
    fn verifier_append(&self, event: Record, source: Source) -> Result<EventId> {
        self.adaptive_record(
            &AdaptiveRecord::Coding {
                record: CodingRecord::Verifier { event },
            },
            source,
        )
    }
    fn verifier_root(&self) -> Result<Hash> {
        let excluded = self
            .verifier_records()?
            .into_iter()
            .filter_map(|(id, r)| {
                (!matches!(
                    r,
                    Record::SupplementalObserved { .. } | Record::SupplementalRevoked { .. }
                ))
                .then_some(id)
            })
            .collect::<BTreeSet<_>>();
        let ids = self
            .events()?
            .into_iter()
            .map(|(_, e)| e.id)
            .filter(|id| !excluded.contains(id))
            .collect::<Vec<_>>();
        Ok(Hash::of(&[
            b"verifier-factual-prefix@v1",
            &serde_json::to_vec(&ids)?,
        ]))
    }
    fn verifier_scope(&self) -> Result<Hash> {
        Ok(Hash::of(&[&serde_json::to_vec(
            &self.project_configuration()?,
        )?]))
    }
    fn verifier_admission(&self, id: EventId) -> Result<Admission> {
        ensure!(
            !self
                .verifier_records()?
                .iter()
                .any(|(_, r)| matches!(r,Record::Revoked{admission,..} if *admission==id)),
            "verifier admission revoked"
        );
        self.verifier_records()?
            .into_iter()
            .find_map(|(i, r)| match r {
                Record::Admitted { admission } if i == id => Some(admission),
                _ => None,
            })
            .ok_or_else(|| anyhow::anyhow!("unknown verifier admission"))
    }
    fn verifier_current(&self, a: &Admission) -> Result<()> {
        ensure!(
            !self
                .project_resource_admission()?
                .is_some_and(|(_, _, _, closed)| closed),
            "closed project cannot admit new verifier activity"
        );
        let budget = self.project_budget()?;
        ensure!(
            !budget.active
                || (!budget.stopped
                    && !budget.exceeded
                    && (budget.unlimited_mode
                        || budget
                            .limits
                            .as_ref()
                            .is_some_and(|l| budget.elapsed_wall_ms < l.wall_ms))),
            "project resource admission stopped or expired"
        );
        ensure!(
            a.scope == self.verifier_scope()?
                && Some(a.requirements_revision)
                    == self.project_acceptance_status()?.requirements_revision,
            "changed verifier scope or requirements"
        );
        ensure!(
            tsr_learn::maturity::timestamp(&a.expires_at)?
                > tsr_learn::maturity::timestamp(&super::super::lifecycle::now()?)?,
            "verifier admission expired"
        );
        ensure!(
            self.adaptive_state_inner()?.pending.is_none(),
            "pending candidate execution"
        );
        let private = self.acceptance_current_aims()?;
        let entries = Ledger::new(&self.store).collective()?.entries;
        ensure!(
            private
                .values()
                .all(|id| entries.iter().any(|p| p.preference.id == *id
                    && matches!(
                        p.status,
                        Some(PreferenceStatus::Open | PreferenceStatus::Fulfilled)
                    ))),
            "withdrawn verifier requirements"
        );
        Ok(())
    }
    pub fn project_verifier_admit(&self, a: Admission) -> Result<EventId> {
        self.write(|| {
            ensure!(
                matches!(
                    a.version.as_str(),
                    "verifier-admission@v1" | "verifier-admission@v2"
                ) && short(&a.verifier_identity)
                    && short(&a.generator_identity)
                    && a.verifier_identity != a.generator_identity,
                "distinct admitted invocation identities required"
            );
            ensure!(
                (1..=8).contains(&a.max_requests)
                    && (1..=32768).contains(&a.tokens_per_request)
                    && (1..=240000).contains(&a.timeout_ms),
                "invalid verifier resource bounds"
            );
            self.verifier_current(&a)?;
            self.validate_prerequisites(&a)?;
            let now = tsr_learn::maturity::timestamp(&super::super::lifecycle::now()?)?;
            ensure!(
                tsr_learn::maturity::timestamp(&a.expires_at)? - now <= 86_400_000_000_000,
                "verifier admission maximum lifetime one day"
            );
            let required = self.project_acceptance_requirements_view()?;
            let spec = self.project_configuration()?;
            ensure!(
                !spec.editable.contains_key("test_receipt")
                    && !spec.protected.contains_key("test_receipt"),
                "reserved verifier evidence path"
            );
            let paths = spec
                .editable
                .keys()
                .chain(spec.protected.keys())
                .cloned()
                .chain(std::iter::once("test_receipt".into()))
                .collect::<BTreeSet<_>>();
            ensure!(
                a.coverage.keys().eq(required.keys())
                    && a.coverage
                        .values()
                        .all(|v| !v.is_empty() && v.is_subset(&paths)),
                "complete admitted coverage with known evidence paths required"
            );
            ensure!(
                a.coverage
                    .values()
                    .flatten()
                    .cloned()
                    .collect::<BTreeSet<_>>()
                    == paths,
                "all source and protected evidence must be covered"
            );
            self.verifier_append(Record::Admitted { admission: a }, Source::Operator)
        })
    }
    pub fn project_verifier_request(&self, admission: EventId) -> Result<(EventId, Request)> {
        self.write(||{
        let a=self.verifier_admission(admission)?;self.verifier_current(&a)?;
        let count=self.verifier_records()?.iter().filter(|(_,r)|matches!(r,Record::Requested{request} if request.admission==admission)).count() as u32;
        ensure!(count<a.max_requests,"verifier reservation budget exhausted; unknown/interrupted requests are not refunded");
        let c=self.project_acceptance_status()?;
        let mut evidence=self.project_snapshot(&self.project_configuration()?)?;
        ensure!(digest(&evidence)?==c.snapshot,"source changed while capturing verifier evidence");
        // Receipt summary is controller-derived, never a generator assertion.
        let tested_source=self.records()?.into_iter().find_map(|(id,r)|match r {
            AdaptiveRecord::Coding{record:CodingRecord::Receipt{source_after,..}} if Some(id)==c.test_receipt=>source_after,_=>None
        });
        evidence.insert("test_receipt".into(),serde_json::to_string(&serde_json::json!({"id":c.test_receipt,"current_source":c.snapshot,"tested_source":tested_source,"current_verdict":if tested_source==Some(c.snapshot){c.test_verdict.clone()}else{None},"historical_verdict":c.test_verdict}))?);
        ensure!(serde_json::to_vec(&evidence)?.len()<=131072,"verifier evidence bound exceeded");
        let observed_tests=self.verifier_observed_tests(c.test_receipt,c.snapshot)?;
        let request=Request{version:if a.version=="verifier-admission@v2" {"verifier-request@v2"}else{"verifier-request@v1"}.into(),observed_tests,supplemental:self.supplemental_current()?,admission,ordinal:count+1,factual_root:self.verifier_root()?,snapshot:c.snapshot,requirements_revision:a.requirements_revision,test_receipt:c.test_receipt,requirements:self.project_acceptance_requirements_view()?,evidence_digest:Hash::of(&[&serde_json::to_vec(&evidence)?]),evidence};
        self.verifier_fresh(&request)?;
        let id=self.verifier_append(Record::Requested{request:request.clone()},Source::Rules)?;
        Ok((id,request))
    })
    }
    fn verifier_request(&self, id: EventId) -> Result<Request> {
        self.verifier_records()?
            .into_iter()
            .find_map(|(i, r)| match r {
                Record::Requested { request } if i == id => Some(request),
                _ => None,
            })
            .ok_or_else(|| anyhow::anyhow!("unknown verifier request"))
    }
    fn verifier_fresh(&self, r: &Request) -> Result<()> {
        self.verifier_current(&self.verifier_admission(r.admission)?)?;
        let c = self.project_acceptance_status()?;
        ensure!(
            r.factual_root == self.verifier_root()?
                && r.snapshot == c.snapshot
                && Some(r.requirements_revision) == c.requirements_revision
                && r.test_receipt == c.test_receipt,
            "stale verifier evidence"
        );
        Ok(())
    }
    fn validate_verifier_report(&self, r: &Request, a: &Admission, report: &Report) -> Result<()> {
        ensure!(
            report.findings.keys().eq(r.requirements.keys()),
            "missing or unknown verifier criteria"
        );
        self.validate_typed_report(r, a, report)?;
        self.supplemental_consistent(report)?;
        for (key, x) in &report.findings {
            ensure!(
                short(&x.explanation) && short(&x.limitations),
                "bounded explanation and limitations required"
            );
            ensure!(
                x.evidence == a.coverage[key]
                    && x.evidence.iter().all(|e| r.evidence.contains_key(e)),
                "unsupported or incomplete verifier evidence"
            );
        }
        Ok(())
    }
    /// Only trusted host code can supply a runner; there is no JSON receipt import.
    pub fn project_verifier_run(&self, id: EventId, runner: &dyn Verifier) -> Result<EventId> {
        let (request,a)=self.write(||{
            let r=self.verifier_request(id)?;self.verifier_fresh(&r)?;
            let a=self.verifier_admission(r.admission)?;
            ensure!(runner.identity()==(a.verifier_identity.as_str(),a.artifact),"unadmitted verifier implementation or self-attestation");
            ensure!(!self.verifier_records()?.iter().any(|(_,x)|matches!(x,Record::Started{request,..}|Record::Finished{request,..} if *request==id)),"verifier invocation already started; reconcile unknown outcome, never retry");
            self.verifier_append(Record::Started{request:id,identity:a.verifier_identity.clone(),artifact:a.artifact},Source::Rules)?;
            Ok((r,a))
        })?;
        let start = std::time::Instant::now();
        let result = runner.evaluate(&request, a.tokens_per_request, a.timeout_ms);
        let elapsed_ms = start.elapsed().as_millis().min(u64::MAX as u128) as u64;
        let mut rejected = None;
        let (response_digest, tokens, report, diagnostic) = match result {
            Ok(out) => {
                let digest = Hash::of(&[out.json.as_bytes()]);
                let parsed = (|| {
                    ensure!(out.json.len() <= 16384, "verifier response exceeds bound");
                    ensure!(
                        out.tokens.is_some_and(|t| t <= a.tokens_per_request)
                            && elapsed_ms <= a.timeout_ms,
                        "unknown or exceeded verifier resources"
                    );
                    self.verifier_fresh(&request)?;
                    let report: Report = serde_json::from_str(&out.json)?;
                    if let Err(e) = self.validate_verifier_report(&request, &a, &report) {
                        rejected = Some(report);
                        return Err(e);
                    }
                    Ok(report)
                })();
                match parsed {
                    Ok(r) => (
                        digest,
                        out.tokens,
                        Some(r),
                        "quarantined; explicit operator adoption required".into(),
                    ),
                    Err(e) => (
                        digest,
                        out.tokens,
                        None,
                        format!("invalid verifier report: {e}"),
                    ),
                }
            }
            Err(_) => (
                Hash::of(&[b"unknown-verifier-return"]),
                None,
                None,
                "verifier failed; outcome/resources unknown; reservation retained".into(),
            ),
        };
        self.write(|| {
            let mut report = report;
            let mut diagnostic = diagnostic;
            if self
                .verifier_records()?
                .iter()
                .any(|(_, r)| matches!(r,Record::LocalCancel{request} if *request==id))
            {
                rejected = report.take().or(rejected.take());
                diagnostic = "canceled; response quarantined without adoption eligibility".into();
            }
            if report.is_some() && self.verifier_fresh(&request).is_err() {
                rejected = report.take().or(rejected.take());
                diagnostic = "state changed before report persistence; quarantined".into();
            }
            if let Some(rejected) = rejected {
                self.verifier_append(
                    Record::RejectedReport {
                        request: id,
                        report: rejected,
                        reason: diagnostic.clone(),
                    },
                    Source::Rules,
                )?;
            }
            self.verifier_append(
                Record::Finished {
                    request: id,
                    response_digest,
                    report,
                    diagnostic,
                    tokens,
                    elapsed_ms: Some(elapsed_ms),
                },
                Source::Rules,
            )
        })
    }
    pub fn project_verifier_revoke(&self, id: EventId, reason: String) -> Result<EventId> {
        self.write(|| {
            self.verifier_admission(id)?;
            ensure!(short(&reason), "bounded revocation reason required");
            self.verifier_append(
                Record::Revoked {
                    admission: id,
                    reason,
                },
                Source::Operator,
            )
        })
    }
    pub fn project_verifier_view(&self) -> Result<serde_json::Value> {
        Ok(
            serde_json::json!({"records":self.verifier_records()?,"automatic_acceptance":false,"local_transport_available":true,"test_identity":self.project_verifier_test_identity()?,"human_effort":null}),
        )
    }
    pub fn project_verifier_adopt(&self, id: EventId) -> Result<Completion> {
        self.write(|| {
            let records = self.verifier_records()?;
            ensure!(
                !records
                    .iter()
                    .any(|(_, r)| matches!(r,Record::Adopted{report,..} if *report==id)),
                "verifier report already adopted"
            );
            let (request, report) = records
                .iter()
                .find_map(|(i, r)| match r {
                    Record::Finished {
                        request,
                        report: Some(report),
                        ..
                    } if *i == id => Some((*request, report.clone())),
                    _ => None,
                })
                .ok_or_else(|| anyhow::anyhow!("no valid quarantined report"))?;
            ensure!(!records.iter().any(|(_,r)|matches!(r,Record::LocalCancel{request: canceled} if *canceled==request)), "canceled verifier report cannot be adopted");
            let r = self.verifier_request(request)?;
            self.verifier_fresh(&r)?;
            self.supplemental_consistent(&report)?;
            let mut unknown = report
                .findings
                .iter()
                .filter(|(_, x)| x.finding == Finding::Unknown)
                .map(|(k, _)| k.clone())
                .collect::<BTreeSet<_>>();
            for (_, other) in &records {
                if let Record::Finished {
                    request: other,
                    report: Some(report2),
                    ..
                } = other
                {
                    let r2 = self.verifier_request(*other)?;
                    if r2.snapshot == r.snapshot
                        && r2.requirements_revision == r.requirements_revision
                        && r2.test_receipt == r.test_receipt
                        && r2.evidence_digest == r.evidence_digest
                    {
                        for (k, v) in &report.findings {
                            if report2
                                .findings
                                .get(k)
                                .is_some_and(|x| x.finding != v.finding)
                            {
                                unknown.insert(k.clone());
                            }
                        }
                    }
                }
            }
            let decision = if !unknown.is_empty() {
                Decision::Unresolved
            } else if report.findings.values().all(|x| x.finding == Finding::Met) {
                Decision::Accept
            } else {
                Decision::Reject
            };
            self.verifier_append(
                Record::Adopted {
                    report: id,
                    decision: decision.clone(),
                    unknown: unknown.clone(),
                },
                Source::Operator,
            )?;
            let evidence = report
                .findings
                .into_iter()
                .map(|(k, x)| {
                    (
                        k.clone(),
                        Evidence {
                            met: !unknown.contains(&k) && x.finding == Finding::Met,
                            evidence: format!(
                                "Verifier report {id}; operator adopted; {}",
                                if unknown.contains(&k) {
                                    "unknown or conflicting; unresolved"
                                } else {
                                    "see immutable findings and limitations"
                                }
                            ),
                        },
                    )
                })
                .collect();
            self.acceptance_review_with_uncertainty(
                Review {
                    snapshot: r.snapshot,
                    requirements_revision: r.requirements_revision,
                    test_receipt: r.test_receipt,
                    decision,
                    evidence,
                },
                unknown,
            )
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    pub(super) fn setup() -> Result<(tempfile::TempDir, Engine, Admission)> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.project_initialize(serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/engine-invoice/project.json"
        )))?)?;
        e.project_acceptance_requirements(BTreeMap::from([
            ("subtotal".into(), "Multiply price and quantity".into()),
            (
                "discount".into(),
                "Subtract discount with floor zero".into(),
            ),
        ]))?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs()
            + 3600;
        // Convert a real future cutoff to the strict RFC3339 admission format.
        let a = Admission {
            version: "verifier-admission@v1".into(),
            scope: e.verifier_scope()?,
            requirements_revision: e
                .project_acceptance_status()?
                .requirements_revision
                .unwrap(),
            verifier_identity: "isolated-static-verifier".into(),
            generator_identity: "proposal-invocation".into(),
            artifact: Hash::of(&[b"test-only-host-adapter"]),
            prerequisites: BTreeMap::new(),
            coverage: BTreeMap::from([
                (
                    "subtotal".into(),
                    BTreeSet::from([
                        "src/pricing.rs".into(),
                        "tests/probe.rs".into(),
                        "test_receipt".into(),
                    ]),
                ),
                (
                    "discount".into(),
                    BTreeSet::from([
                        "src/discount.rs".into(),
                        "tests/probe.rs".into(),
                        "test_receipt".into(),
                    ]),
                ),
            ]),
            max_requests: 3,
            tokens_per_request: 1000,
            timeout_ms: 1000,
            expires_at: super::super::super::lifecycle::timestamp(&format!(
                "unix-ms:{}",
                now * 1000
            ))?,
        };
        Ok((dir, e, a))
    }
    struct Fake {
        identity: String,
        artifact: Hash,
        body: String,
        tokens: Option<u64>,
    }
    impl Verifier for Fake {
        fn identity(&self) -> (&str, Hash) {
            (&self.identity, self.artifact)
        }
        fn evaluate(&self, r: &Request, _: u64, _: u64) -> Result<Output> {
            assert!(!r.evidence.contains_key("generator_rationale"));
            assert_eq!(
                r.evidence_digest,
                Hash::of(&[&serde_json::to_vec(&r.evidence)?])
            );
            Ok(Output {
                json: self.body.clone(),
                tokens: self.tokens,
            })
        }
    }
    fn runner(a: &Admission, f: Finding) -> Fake {
        let report = Report {
            findings: a
                .coverage
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        Assessment {
                            finding: f.clone(),
                            evidence: v.clone(),
                            explanation: "Synthetic test finding, not actual verification".into(),
                            basis: None,
                            limitations: "Synthetic fixture only".into(),
                        },
                    )
                })
                .collect(),
        };
        Fake {
            identity: a.verifier_identity.clone(),
            artifact: a.artifact,
            body: serde_json::to_string(&report).unwrap(),
            tokens: Some(20),
        }
    }
    #[test]
    fn verifier_quarantine_adoption_restart_and_replay() -> Result<()> {
        let (dir, e, a) = setup()?;
        let id = e.project_verifier_admit(a.clone())?;
        let (request, _) = e.project_verifier_request(id)?;
        let before = e.project_acceptance_status()?;
        let report = e.project_verifier_run(request, &runner(&a, Finding::NotMet))?;
        assert_eq!(
            e.project_acceptance_status()?.decision_record,
            before.decision_record
        );
        let e = Engine::open(dir.path())?;
        assert!(e
            .project_verifier_run(request, &runner(&a, Finding::NotMet))
            .is_err());
        assert_eq!(e.project_verifier_adopt(report)?.status, "rejected");
        assert!(e.project_verifier_adopt(report).is_err());
        assert!(!e.project_acceptance_status()?.accepted);
        assert!(e
            .acceptance_current_aims()?
            .values()
            .all(|id| Ledger::new(&e.store)
                .collective()
                .unwrap()
                .entries
                .iter()
                .any(|p| p.preference.id == *id && p.is_open())));
        Ok(())
    }
    #[test]
    fn verifier_rejects_self_attestation_bad_bounds_coverage_and_provenance() -> Result<()> {
        let (_dir, e, a) = setup()?;
        for which in 0..6 {
            let mut bad = a.clone();
            match which {
                0 => bad.max_requests = 0,
                1 => bad.timeout_ms = 240001,
                2 => bad.tokens_per_request = 0,
                3 => bad.verifier_identity = bad.generator_identity.clone(),
                4 => {
                    bad.coverage.remove("discount");
                }
                _ => bad.expires_at = "2020-01-01T00:00:00Z".into(),
            }
            assert!(e.project_verifier_admit(bad).is_err());
        }
        let id = e.project_verifier_admit(a.clone())?;
        let (request, _) = e.project_verifier_request(id)?;
        let mut fake = runner(&a, Finding::Met);
        fake.identity = a.generator_identity.clone();
        assert!(e.project_verifier_run(request, &fake).is_err());
        fake.identity = a.verifier_identity.clone();
        fake.artifact = Hash::of(&[b"wrong-artifact"]);
        assert!(e.project_verifier_run(request, &fake).is_err());
        // A model cannot import or attest a receipt through the durable record boundary.
        e.verifier_append(
            Record::Started {
                request,
                identity: a.verifier_identity,
                artifact: a.artifact,
            },
            Source::Generator,
        )?;
        assert!(e.project_verifier_view().is_err());
        Ok(())
    }
    #[test]
    fn verifier_malformed_unknown_resources_and_uncovered_reports_stay_quarantined() -> Result<()> {
        for mode in 0..6 {
            let (_dir, e, a) = setup()?;
            let id = e.project_verifier_admit(a.clone())?;
            let (request, _) = e.project_verifier_request(id)?;
            let mut fake = runner(&a, Finding::Met);
            match mode {
                0 => fake.body = "not json".into(),
                1 => fake.tokens = None,
                2 => fake.tokens = Some(1001),
                _ => {
                    let mut report: serde_json::Value = serde_json::from_str(&fake.body)?;
                    match mode {
                        3 => {
                            report["identity"] = serde_json::json!(a.verifier_identity);
                        }
                        4 => {
                            report["findings"]
                                .as_object_mut()
                                .unwrap()
                                .remove("subtotal");
                        }
                        _ => {
                            report["findings"]["subtotal"]["evidence"] =
                                serde_json::json!(["made-up-source"]);
                        }
                    }
                    fake.body = serde_json::to_string(&report)?;
                }
            }
            let report = e.project_verifier_run(request, &fake)?;
            assert!(e.project_verifier_adopt(report).is_err());
            assert!(!e.project_acceptance_status()?.accepted);
            assert!(e.project_verifier_run(request, &fake).is_err());
        }
        Ok(())
    }
    #[test]
    fn verifier_stale_evidence_withdrawal_unknown_and_conflict() -> Result<()> {
        for mode in 0..5 {
            let (dir, e, a) = setup()?;
            let id = e.project_verifier_admit(a.clone())?;
            let (request, _) = e.project_verifier_request(id)?;
            let report = e.project_verifier_run(
                request,
                &runner(
                    &a,
                    if mode == 3 {
                        Finding::Unknown
                    } else {
                        Finding::NotMet
                    },
                ),
            )?;
            match mode {
                0 => {
                    let path = dir.path().join("coding-work/src/pricing.rs");
                    let old = std::fs::read_to_string(&path)?;
                    std::fs::write(path, old.replace("+ quantity", "* quantity"))?;
                }
                1 => {
                    e.project_acceptance_requirements(BTreeMap::from([(
                        "changed".into(),
                        "Different intent".into(),
                    )]))?;
                }
                2 => {
                    let id = e.acceptance_current_aims()?["subtotal"];
                    e.append(
                        EventKind::StatusSet {
                            preference: id,
                            status: PreferenceStatus::Withdrawn,
                            by: Source::Operator,
                        },
                        Source::Operator,
                    )?;
                }
                4 => {
                    let (r2, _) = e.project_verifier_request(id)?;
                    e.project_verifier_run(r2, &runner(&a, Finding::Met))?;
                }
                _ => {}
            }
            if mode < 3 {
                assert!(e.project_verifier_adopt(report).is_err());
            } else {
                assert_eq!(e.project_verifier_adopt(report)?.status, "unresolved");
            }
        }
        Ok(())
    }
    #[test]
    fn verifier_interrupted_reservations_are_not_reclaimed_or_replayed() -> Result<()> {
        let (dir, e, mut a) = setup()?;
        a.max_requests = 1;
        let id = e.project_verifier_admit(a.clone())?;
        let (request, _) = e.project_verifier_request(id)?;
        e.verifier_append(
            Record::Started {
                request,
                identity: a.verifier_identity.clone(),
                artifact: a.artifact,
            },
            Source::Rules,
        )?;
        let e = Engine::open(dir.path())?;
        assert!(e
            .project_verifier_run(request, &runner(&a, Finding::Met))
            .is_err());
        assert!(e.project_verifier_request(id).is_err());
        assert!(e.project_verifier_adopt(request).is_err());
        Ok(())
    }
    #[test]
    fn verifier_all_met_cannot_replace_protected_test_evidence() -> Result<()> {
        let (_dir, e, a) = setup()?;
        let id = e.project_verifier_admit(a.clone())?;
        let (request, _) = e.project_verifier_request(id)?;
        let report = e.project_verifier_run(request, &runner(&a, Finding::Met))?;
        assert!(e.project_verifier_adopt(report).is_err());
        assert!(!e.project_acceptance_status()?.accepted);
        assert!(!e
            .verifier_records()?
            .iter()
            .any(|(_, r)| matches!(r, Record::Adopted { .. })));
        Ok(())
    }
    #[test]
    fn verifier_unknown_does_not_create_false_contrast_and_known_findings_survive() -> Result<()> {
        for known in [Finding::NotMet, Finding::Met] {
            let (_dir, e, a) = setup()?;
            let id = e.project_verifier_admit(a.clone())?;
            let (request, _) = e.project_verifier_request(id)?;
            let mut fake = runner(&a, Finding::Unknown);
            let mut report: Report = serde_json::from_str(&fake.body)?;
            report.findings.get_mut("subtotal").unwrap().finding = known.clone();
            fake.body = serde_json::to_string(&report)?;
            let report = e.project_verifier_run(request, &fake)?;
            let result = e.project_verifier_adopt(report)?;
            assert_eq!(result.status, "unresolved");
            assert_eq!(e.acceptance_cycle()?["unknown_count"], 1);
            assert_eq!(
                e.acceptance_cycle()?["unmet_count"],
                usize::from(known == Finding::NotMet)
            );
            let contrast = e
                .events()?
                .into_iter()
                .rev()
                .find_map(|(_, e)| match e.kind {
                    EventKind::Observed {
                        observation:
                            Observation::Return {
                                contrast: Some(c), ..
                            },
                    } => Some(c),
                    _ => None,
                })
                .unwrap();
            assert!(!contrast
                .unmet
                .iter()
                .any(|(id, _)| *id == CriterionId(Hash::of(&[b"acceptance", b"discount"]))));
            assert_eq!(contrast.unmet.len(), usize::from(known == Finding::NotMet));
            let aims = e.acceptance_current_aims()?;
            let entries = Ledger::new(&e.store).collective()?.entries;
            assert!(entries
                .iter()
                .any(|x| x.preference.id == aims["discount"] && x.is_open()));
            assert!(entries.iter().any(|x| x.preference.id == aims["subtotal"]
                && x.status
                    == Some(if known == Finding::Met {
                        PreferenceStatus::Fulfilled
                    } else {
                        PreferenceStatus::Open
                    })));
        }
        Ok(())
    }
    #[test]
    fn verifier_reserved_path_and_changed_current_evidence_are_rejected() -> Result<()> {
        let (_dir, e, mut a) = setup()?;
        a.coverage
            .get_mut("discount")
            .unwrap()
            .insert("not-admitted".into());
        assert!(e.project_verifier_admit(a).is_err());
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let mut spec: super::super::super::ProjectSpec =
            serde_json::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../examples/engine-invoice/project.json"
            )))?;
        spec.protected
            .insert("test_receipt".into(), "untrusted source".into());
        e.project_initialize(spec)?;
        e.project_acceptance_requirements(BTreeMap::from([("x".into(), "criterion".into())]))?;
        let (_, _, mut a) = setup()?;
        a.scope = e.verifier_scope()?;
        a.requirements_revision = e
            .project_acceptance_status()?
            .requirements_revision
            .unwrap();
        assert!(e
            .project_verifier_admit(a)
            .unwrap_err()
            .to_string()
            .contains("reserved"));
        Ok(())
    }
    #[test]
    fn verifier_revocation_and_unknown_return_preserve_reservations() -> Result<()> {
        let (dir, e, a) = setup()?;
        let id = e.project_verifier_admit(a.clone())?;
        let (request, _) = e.project_verifier_request(id)?;
        e.project_verifier_revoke(id, "operator withdrew reviewer authority".into())?;
        let e = Engine::open(dir.path())?;
        assert!(e
            .project_verifier_run(request, &runner(&a, Finding::Met))
            .is_err());
        assert!(e.project_verifier_request(id).is_err());
        struct Failed(Fake);
        impl Verifier for Failed {
            fn identity(&self) -> (&str, Hash) {
                self.0.identity()
            }
            fn evaluate(&self, _: &Request, _: u64, _: u64) -> Result<Output> {
                anyhow::bail!("transport lost")
            }
        }
        let (_dir, e, mut a) = setup()?;
        a.max_requests = 1;
        let id = e.project_verifier_admit(a.clone())?;
        let (request, _) = e.project_verifier_request(id)?;
        let report = e.project_verifier_run(request, &Failed(runner(&a, Finding::Met)))?;
        assert!(e.project_verifier_adopt(report).is_err());
        assert!(e.project_verifier_request(id).is_err());
        assert!(e
            .project_verifier_run(request, &runner(&a, Finding::Met))
            .is_err());
        Ok(())
    }
    #[test]
    fn verifier_legacy_aliases_are_conjunctive_independent_of_key_order() -> Result<()> {
        for uncertain in [Finding::NotMet, Finding::Unknown] {
            for key in ["discount", "subtotal"] {
                let (_dir, e, a) = setup()?;
                let aims = e.acceptance_current_aims()?;
                let preference = aims["subtotal"];
                e.adaptive_record(
                    &AdaptiveRecord::Coding {
                        record: CodingRecord::AcceptanceAims {
                            revision: a.requirements_revision,
                            preferences: BTreeMap::from([
                                ("discount".into(), preference),
                                ("subtotal".into(), preference),
                            ]),
                        },
                    },
                    Source::Operator,
                )?;
                let id = e.project_verifier_admit(a.clone())?;
                let (request, _) = e.project_verifier_request(id)?;
                let mut fake = runner(&a, Finding::Met);
                let mut report: Report = serde_json::from_str(&fake.body)?;
                report.findings.get_mut(key).unwrap().finding = uncertain.clone();
                fake.body = serde_json::to_string(&report)?;
                let report = e.project_verifier_run(request, &fake)?;
                e.project_verifier_adopt(report)?;
                assert!(
                    Ledger::new(&e.store)
                        .collective()?
                        .entries
                        .iter()
                        .any(|p| p.preference.id == preference && p.is_open()),
                    "{key} {uncertain:?}"
                );
            }
        }
        Ok(())
    }
    #[test]
    fn verifier_disagreement_survives_prior_adoption_without_new_evidence() -> Result<()> {
        let (_dir, e, a) = setup()?;
        let id = e.project_verifier_admit(a.clone())?;
        let (request, _) = e.project_verifier_request(id)?;
        let rejected = e.project_verifier_run(request, &runner(&a, Finding::NotMet))?;
        e.project_verifier_adopt(rejected)?;
        let (fresh, _) = e.project_verifier_request(id)?;
        let positive = e.project_verifier_run(fresh, &runner(&a, Finding::Met))?;
        assert_eq!(e.project_verifier_adopt(positive)?.status, "unresolved");
        Ok(())
    }
    #[test]
    fn verifier_uncertainty_requires_rules_provenance_and_applicable_review() -> Result<()> {
        for mode in ["source", "requirements", "forged", "invalid-key"] {
            let (dir, e, a) = setup()?;
            let id = e.project_verifier_admit(a.clone())?;
            let (request, _) = e.project_verifier_request(id)?;
            let report = e.project_verifier_run(request, &runner(&a, Finding::Unknown))?;
            let completion = e.project_verifier_adopt(report)?;
            assert_eq!(e.acceptance_cycle()?["unknown_count"], 2);
            match mode {
                "source" => {
                    let path = dir.path().join("coding-work/src/pricing.rs");
                    let old = std::fs::read_to_string(&path)?;
                    std::fs::write(path, old.replace("+ quantity", "* quantity"))?;
                }
                "requirements" => {
                    e.project_acceptance_requirements(BTreeMap::from([(
                        "new".into(),
                        "New criterion".into(),
                    )]))?;
                }
                _ => {
                    e.adaptive_record(
                        &AdaptiveRecord::Coding {
                            record: CodingRecord::AcceptanceUncertainty {
                                review: completion.decision_record.unwrap(),
                                unknown: BTreeSet::from([if mode == "forged" {
                                    "subtotal".into()
                                } else {
                                    "nonexistent".into()
                                }]),
                            },
                        },
                        if mode == "forged" {
                            Source::Generator
                        } else {
                            Source::Rules
                        },
                    )?;
                }
            }
            if mode == "source" || mode == "requirements" {
                let cycle = e.acceptance_cycle()?;
                assert_eq!(cycle["review_evidence_applicable"], false);
                assert!(cycle["unknown_count"].is_null());
                assert!(cycle["unknown"].as_array().unwrap().is_empty());
            } else {
                assert!(e.acceptance_cycle().is_err());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod local_tests;

#[cfg(test)]
mod typed_tests;
