//! Controller facts and typed claim prerequisites, never natural-language truth checking.
use super::*;
use anyhow::Context;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Prerequisite {
    ObservedSupplementalSet {
        observations: Vec<SupplementalBinding>,
    },
    SupplementalFinite {
        admission: EventId,
    },
    ObservedSupplemental {
        admission: EventId,
        receipt: EventId,
    },
    StaticSemantic,
    ObservedTests {
        test_identity: Hash,
    },
    Mixed {
        test_identity: Hash,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    Passed,
    Failed,
    Unknown,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ObservedTests {
    pub receipt: EventId,
    pub snapshot: Hash,
    pub test_identity: Hash,
    pub outcome: Outcome,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Basis {
    ObservedSupplementalSet {
        observations: Vec<SupplementalBinding>,
    },
    SupplementalFinite {
        admission: EventId,
        receipt: EventId,
    },
    Unknown,
    ObservedSupplemental {
        admission: EventId,
        receipt: EventId,
    },
    StaticSemantic,
    ObservedTests {
        observation: ObservedTests,
    },
    Mixed {
        observation: ObservedTests,
        static_finding: Finding,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SupplementalBinding {
    pub admission: EventId,
    pub receipt: EventId,
}
impl Prerequisite {
    pub(crate) fn supplemental_bindings(&self) -> Vec<SupplementalBinding> {
        match self {
            Self::ObservedSupplemental { admission, receipt } => vec![SupplementalBinding {
                admission: *admission,
                receipt: *receipt,
            }],
            Self::ObservedSupplementalSet { observations } => observations.clone(),
            _ => Vec::new(),
        }
    }
}
impl Engine {
    pub fn project_verifier_test_identity(&self) -> Result<Hash> {
        suite_identity(&self.project_configuration()?)
    }

    pub(super) fn validate_prerequisites(&self, a: &Admission) -> Result<()> {
        self.validate_prerequisites_scoped(a, false)
    }
    fn validate_prerequisites_scoped(&self, a: &Admission, scoped: bool) -> Result<()> {
        if a.version == "verifier-admission@v1" {
            ensure!(
                a.prerequisites.is_empty(),
                "legacy admission cannot claim typed guarantees"
            );
            return Ok(());
        }
        ensure!(
            a.prerequisites.keys().eq(a.coverage.keys()),
            "typed prerequisites required for every criterion"
        );
        let identity = self.project_verifier_test_identity()?;
        for (key, p) in &a.prerequisites {
            if matches!(
                p,
                Prerequisite::ObservedSupplemental { .. }
                    | Prerequisite::ObservedSupplementalSet { .. }
            ) {
                ensure!(
                    scoped,
                    "observed supplemental requires explicit criterion assessment scope"
                );
                let (_, published) = self
                    .criterion_policy()?
                    .context("observed supplemental requires scoped criterion publication")?;
                ensure!(
                    published.assessment_scope.is_some()
                        && published.requirements_revision == a.requirements_revision
                        && published.coverage.get(key) == a.coverage.get(key)
                        && published
                            .prerequisites
                            .get(key)
                            .is_some_and(|expected| serde_json::to_value(expected).ok()
                                == serde_json::to_value(p).ok()),
                    "observed supplemental differs from published scoped criterion"
                );
                if let Prerequisite::ObservedSupplementalSet { observations } = p {
                    self.validate_supplemental_set(key, observations)?;
                }
                for binding in p.supplemental_bindings() {
                    let observation =
                        self.observed_browser_supplemental(binding.admission, binding.receipt)?;
                    ensure!(
                        observation.snapshot == a.scope && observation.mapping.contains_key(key),
                        "unmapped observed supplemental scope"
                    );
                }
            }
            if let Prerequisite::SupplementalFinite { admission } = p {
                let bundle = self.supplemental_bundle(*admission)?;
                ensure!(
                    bundle.requirements_revision == a.requirements_revision
                        && bundle.mapping.contains_key(key),
                    "unreviewed supplemental requirement mapping"
                );
                self.project_supplemental_spec(*admission)?;
            }
            if let Prerequisite::ObservedTests { test_identity }
            | Prerequisite::Mixed { test_identity } = p
            {
                ensure!(*test_identity == identity, "unadmitted test identity");
            }
        }
        Ok(())
    }
    pub(crate) fn verifier_observed_tests(
        &self,
        id: Option<EventId>,
        snapshot: Hash,
    ) -> Result<Option<ObservedTests>> {
        let Some(id) = id else { return Ok(None) };
        let records = self.records()?;
        let events = self.events()?;
        ensure!(
            events
                .iter()
                .any(|(_, e)| e.id == id && e.source == Source::Rules),
            "observed test receipt provenance required"
        );
        let Some((source, outcome)) = records.into_iter().find_map(|(key, r)| match r {
            AdaptiveRecord::Coding {
                record:
                    CodingRecord::Receipt {
                        source_after,
                        verdict,
                        ..
                    },
            } if key == id => Some((
                source_after,
                match verdict {
                    Verdict::Passed => Outcome::Passed,
                    Verdict::FailedTests => Outcome::Failed,
                    _ => Outcome::Unknown,
                },
            )),
            _ => None,
        }) else {
            anyhow::bail!("immutable test receipt missing")
        };
        // Old source evidence remains in history, never promoted to the current snapshot.
        if source != Some(snapshot) {
            return Ok(None);
        }
        Ok(Some(ObservedTests {
            receipt: id,
            snapshot,
            test_identity: self.project_verifier_test_identity()?,
            outcome,
        }))
    }
    pub(crate) fn validate_typed_report(
        &self,
        r: &Request,
        a: &Admission,
        report: &Report,
    ) -> Result<()> {
        self.validate_typed_report_scoped(r, a, report, false)
    }
    pub(crate) fn validate_criterion_report(
        &self,
        r: &Request,
        a: &Admission,
        report: &Report,
    ) -> Result<()> {
        self.validate_typed_report_scoped(r, a, report, true)
    }
    fn validate_typed_report_scoped(
        &self,
        r: &Request,
        a: &Admission,
        report: &Report,
        scoped: bool,
    ) -> Result<()> {
        if a.version == "verifier-admission@v1" {
            ensure!(
                report.findings.values().all(|x| x.basis.is_none()),
                "legacy report cannot claim typed authority"
            );
            return Ok(());
        }
        ensure!(r.version == "verifier-request@v2", "typed request required");
        self.validate_prerequisites_scoped(a, scoped)?;
        ensure!(
            serde_json::to_value(&r.supplemental)?
                == serde_json::to_value(self.supplemental_current()?)?,
            "changed supplemental evidence binding"
        );
        ensure!(
            r.observed_tests == self.verifier_observed_tests(r.test_receipt, r.snapshot)?,
            "changed observed receipt binding"
        );
        for (key, x) in &report.findings {
            let basis = x
                .basis
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("explicit typed basis required"))?;
            let policy = &a.prerequisites[key];
            if matches!(basis, Basis::Unknown) {
                ensure!(
                    x.finding == Finding::Unknown,
                    "unknown evidence cannot establish success or failure"
                );
                continue;
            }
            if let (
                Prerequisite::SupplementalFinite {
                    admission: expected,
                },
                Basis::SupplementalFinite { admission, receipt },
            ) = (policy, basis)
            {
                ensure!(expected == admission, "unadmitted supplemental evidence");
                let observation = r
                    .supplemental
                    .iter()
                    .find(|o| o.admission == *admission && o.receipt == *receipt)
                    .ok_or_else(|| anyhow::anyhow!("missing supplemental observation"))?;
                ensure!(
                    observation.snapshot == r.snapshot
                        && observation.requirements_revision == r.requirements_revision,
                    "stale supplemental observation"
                );
                ensure!(
                    x.finding == Finding::Unknown
                        || (x.finding == Finding::NotMet
                            && observation.findings.get(key) == Some(&Finding::NotMet)),
                    "finite evidence cannot prove semantic success or invent a counterexample"
                );
                continue;
            }
            if let (
                Prerequisite::ObservedSupplementalSet {
                    observations: expected,
                },
                Basis::ObservedSupplementalSet { observations },
            ) = (policy, basis)
            {
                ensure!(
                    observations == expected,
                    "cumulative observed supplemental binding mismatch"
                );
                self.validate_supplemental_set(key, observations)?;
                for binding in observations {
                    let o =
                        self.observed_browser_supplemental(binding.admission, binding.receipt)?;
                    ensure!(
                        o.snapshot == r.snapshot
                            && o.requirements_revision == r.requirements_revision,
                        "stale cumulative observation"
                    );
                    let mapped = o
                        .mapping
                        .get(key)
                        .context("unmapped cumulative criterion")?;
                    // An authenticated set may be consulted to conclude Unknown.
                    // Non-unknown claims still require complete protocol observations.
                    ensure!(
                        x.finding == Finding::Unknown
                            || mapped.cases.iter().all(|c| o.cases.get(c)
                                == Some(&super::supplemental::CaseOutcome::PassedFiniteCase)),
                        "incomplete cumulative protocol requires unknown"
                    );
                }
                continue;
            }
            if let (
                Prerequisite::ObservedSupplemental {
                    admission: expected,
                    receipt: expected_receipt,
                },
                Basis::ObservedSupplemental { admission, receipt },
            ) = (policy, basis)
            {
                ensure!(
                    admission == expected && receipt == expected_receipt,
                    "observed supplemental binding mismatch"
                );
                let o = self.observed_browser_supplemental(*admission, *receipt)?;
                ensure!(
                    o.snapshot == r.snapshot && o.requirements_revision == r.requirements_revision,
                    "stale observed supplemental scope"
                );
                let mapped = o.mapping.get(key).context("unmapped observed criterion")?;
                ensure!(
                    x.finding == Finding::Unknown
                        || mapped.cases.iter().all(|c| o.cases.get(c)
                            == Some(&super::supplemental::CaseOutcome::PassedFiniteCase)),
                    "incomplete protocol observations cannot establish semantic success or failure"
                );
                continue;
            }
            if matches!(
                (policy, basis),
                (Prerequisite::StaticSemantic, Basis::StaticSemantic)
            ) {
                continue;
            }
            let (expected, observed) = match (policy, basis) {
                (
                    Prerequisite::ObservedTests { test_identity },
                    Basis::ObservedTests { observation },
                )
                | (Prerequisite::Mixed { test_identity }, Basis::Mixed { observation, .. }) => {
                    (test_identity, observation)
                }
                _ => anyhow::bail!("finding basis does not satisfy admitted prerequisite"),
            };
            ensure!(
                Some(observed) == r.observed_tests.as_ref()
                    && observed.test_identity == *expected
                    && observed.snapshot == r.snapshot
                    && Some(observed.receipt) == r.test_receipt,
                "unsupported receipt/source/test/outcome citation"
            );
            match x.finding {
                Finding::Met => ensure!(
                    observed.outcome == Outcome::Passed
                        && !matches!(basis,Basis::Mixed{static_finding,..} if *static_finding!=Finding::Met),
                    "observed success requires matching passed receipt"
                ),
                Finding::NotMet => ensure!(
                    observed.outcome == Outcome::Failed
                        || matches!(
                            basis,
                            Basis::Mixed {
                                static_finding: Finding::NotMet,
                                ..
                            }
                        ),
                    "observed failure requires matching failed receipt"
                ),
                Finding::Unknown => {}
            }
        }
        Ok(())
    }
}
pub(super) fn basis_schema(policy: &Prerequisite) -> serde_json::Value {
    let variant = |kind: &str| serde_json::json!({"type":"object","additionalProperties":false,"required":["kind"],"properties":{"kind":{"type":"string","enum":[kind]}}});
    let mut variants = vec![variant("unknown")];
    if let Prerequisite::SupplementalFinite { admission } = policy {
        let v = serde_json::json!({"type":"object","additionalProperties":false,"required":["kind","admission","receipt"],"properties":{"kind":{"type":"string","enum":["supplemental_finite"]},"admission":{"type":"string","enum":[admission]},"receipt":{"type":"string"}}});
        variants.push(v);
        return serde_json::json!({"oneOf":variants});
    }
    if let Prerequisite::ObservedSupplemental { admission, receipt } = policy {
        variants.push(serde_json::json!({"type":"object","additionalProperties":false,"required":["kind","admission","receipt"],"properties":{"kind":{"type":"string","enum":["observed_supplemental"]},"admission":{"type":"string","enum":[admission]},"receipt":{"type":"string","enum":[receipt]}}}));
        return serde_json::json!({"oneOf":variants});
    }
    if let Prerequisite::ObservedSupplementalSet { observations } = policy {
        variants.push(serde_json::json!({"type":"object","additionalProperties":false,"required":["kind","observations"],"properties":{"kind":{"type":"string","enum":["observed_supplemental_set"]},"observations":{"const":observations}}}));
        return serde_json::json!({"oneOf":variants});
    }
    let kind = match policy {
        Prerequisite::SupplementalFinite { .. }
        | Prerequisite::ObservedSupplemental { .. }
        | Prerequisite::ObservedSupplementalSet { .. } => unreachable!("handled above"),
        Prerequisite::StaticSemantic => "static_semantic",
        Prerequisite::ObservedTests { .. } => "observed_tests",
        Prerequisite::Mixed { .. } => "mixed",
    };
    let mut v = variant(kind);
    if kind != "static_semantic" {
        v["required"] = serde_json::json!(["kind", "observation"]);
        v["properties"]["observation"] = serde_json::json!({"type":"object","additionalProperties":false,"required":["receipt","snapshot","test_identity","outcome"],"properties":{"receipt":{"type":"string"},"snapshot":{"type":"string"},"test_identity":{"type":"string"},"outcome":{"type":"string","enum":["passed","failed","unknown"]}}});
    }
    if kind == "mixed" {
        v["required"]
            .as_array_mut()
            .unwrap()
            .push(serde_json::json!("static_finding"));
        v["properties"]["static_finding"] =
            serde_json::json!({"type":"string","enum":["met","not_met","unknown"]});
    }
    variants.push(v);
    serde_json::json!({"oneOf":variants})
}

pub(super) fn suite_identity(spec: &ProjectSpec) -> Result<Hash> {
    Ok(Hash::of(&[
        b"verifier-test-suite@v1",
        &serde_json::to_vec(&serde_json::json!({
            "cargo":spec.cargo,"named_package":spec.named_package,
            "full_workspace_oracle":spec.full_workspace_oracle,"probe_entry":spec.probe_entry,
            "cases":spec.cases,"protected":spec.protected,"image":spec.image,
            "user":spec.user,"timeout_ms":spec.timeout_ms
        }))?,
    ]))
}
