//! Opt-in public, independently assessed criterion returns. Never user acceptance.
use super::*;
use acceptance::reviewer::{self, typed, Assessment, Finding};
use anyhow::Context;
use serde_json::{json, Value};
use tsr_domain::Contrast;

/// Prospective operator-published domains; never inferred by a generator.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScopePart {
    pub domain: CriterionDomain,
    pub text: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CriterionDomain {
    Artifact,
    EngineInvariant,
}
fn supplemental_key(admission: EventId, receipt: EventId) -> String {
    format!("supplemental:{admission}:{receipt}")
}
fn supplemental_public(
    o: &reviewer::supplemental::Observation,
    allowed: &BTreeSet<String>,
) -> Value {
    let mapped = o
        .mapping
        .iter()
        .filter(|(k, _)| allowed.contains(*k))
        .map(|(k, m)| (k, &m.cases))
        .collect::<BTreeMap<_, _>>();
    let ids = mapped
        .values()
        .flat_map(|cases| cases.iter())
        .collect::<BTreeSet<_>>();
    let cases = o.browser_cases.as_ref().map(|raw| {
        raw.iter()
            .filter(|c| c.case_id.as_ref().is_some_and(|id| ids.contains(id)))
            .collect::<Vec<_>>()
    });
    json!({"admission":o.admission,"receipt":o.receipt,"snapshot":o.snapshot,"checked_snapshot":o.checked_snapshot,"bundle_digest":o.bundle_digest,"source_episode_root":o.source_episode_root,"mapped_cases":mapped,"browser_cases":cases,"limits":"Authenticated finite protocol observations, not automatic criterion satisfaction or human fulfillment. Independent scoped assessment required."})
}

fn validate_scope(
    requirements: &BTreeMap<String, String>,
    scope: &BTreeMap<String, Vec<ScopePart>>,
) -> Result<()> {
    ensure!(
        scope.keys().eq(requirements.keys()),
        "complete criterion scope required"
    );
    for (key, parts) in scope {
        ensure!(
            !parts.is_empty() && parts.iter().all(|p| !p.text.trim().is_empty()),
            "empty criterion scope part"
        );
        ensure!(
            parts.iter().map(|p| p.text.as_str()).collect::<String>() == requirements[key],
            "criterion scope must preserve exact original text and order"
        );
        ensure!(
            parts.iter().any(|p| p.domain == CriterionDomain::Artifact),
            "engine-only requirement is not an artifact criterion"
        );
    }
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub state_root: Hash,
    pub requirements_revision: EventId,
    /// Explicit publication, checked against the original criterion text.
    pub requirements: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assessment_scope: Option<BTreeMap<String, Vec<ScopePart>>>,
    pub coverage: BTreeMap<String, BTreeSet<String>>,
    pub prerequisites: BTreeMap<String, typed::Prerequisite>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Frame {
    pub policy: EventId,
    pub publication: EventId,
    pub snapshot: Hash,
    pub requirements_revision: EventId,
    pub receipt: EventId,
    pub requirements: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assessment_scope: Option<BTreeMap<String, Vec<ScopePart>>>,
    pub evidence: BTreeMap<String, String>,
    pub coverage: BTreeMap<String, BTreeSet<String>>,
    pub prerequisites: BTreeMap<String, typed::Prerequisite>,
    pub observed_tests: typed::ObservedTests,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Apply {
    pub state_root: Hash,
    pub binding: Hash,
    pub frame: Frame,
    pub reviewer: String,
    pub generator: String,
    pub response_digest: Hash,
    pub decision: String,
    pub findings: BTreeMap<String, Assessment>,
}
impl Engine {
    fn criterion_publication(&self) -> Result<Option<EventId>> {
        let revision = self.project_acceptance_status()?.requirements_revision;
        Ok(self
            .records()?
            .into_iter()
            .rev()
            .find_map(|(id, r)| match r {
                Record::Coding {
                    record: CodingRecord::AcceptanceFeedback { feedback },
                } if Some(feedback.requirements_revision) == revision => Some(id),
                _ => None,
            }))
    }
    pub(crate) fn criterion_policy(&self) -> Result<Option<(EventId, Policy)>> {
        let revision = self.project_acceptance_status()?.requirements_revision;
        for (id, r) in self.records()?.into_iter().rev() {
            if let Record::Coding {
                record:
                    CodingRecord::CriterionPolicy { policy }
                    | CodingRecord::CriterionPolicyRefreshed { policy, .. },
            } = r
            {
                self.require_project_bridge_source(id, Source::Operator)?;
                if Some(policy.requirements_revision) == revision {
                    return Ok(Some((id, policy)));
                }
            }
        }
        Ok(None)
    }
    pub fn project_criterion_policy(&self, policy: Policy) -> Result<EventId> {
        self.publish_criterion_policy(policy, None)
    }
    pub fn project_criterion_policy_refresh(
        &self,
        policy: Policy,
        previous: EventId,
    ) -> Result<EventId> {
        self.publish_criterion_policy(policy, Some(previous))
    }
    fn publish_criterion_policy(
        &self,
        policy: Policy,
        previous: Option<EventId>,
    ) -> Result<EventId> {
        self.write(|| {
            ensure!(
                self.store.head_root()? == policy.state_root,
                "stale criterion publication"
            );
            ensure!(
                self.project_configuration()?.browser,
                "browser criterion profile required"
            );
            ensure!(self.adaptive_state()?.pending.is_none(), "pending action");
            if let Some(previous) = previous {
                let (id, old) = self
                    .criterion_policy()?
                    .context("previous criterion policy required")?;
                ensure!(id == previous, "stale criterion policy predecessor");
                ensure!(
                    old.requirements_revision == policy.requirements_revision
                        && old.requirements == policy.requirements
                        && serde_json::to_value(&old.assessment_scope)?
                            == serde_json::to_value(&policy.assessment_scope)?,
                    "criterion refresh cannot change original criteria or scope"
                );
                ensure!(
                    old.prerequisites
                        .values()
                        .chain(policy.prerequisites.values())
                        .all(|p| matches!(p, typed::Prerequisite::ObservedSupplementalSet { .. })),
                    "refresh limited to cumulative observed evidence"
                );
                ensure!(
                    serde_json::to_value(&old.prerequisites)?
                        != serde_json::to_value(&policy.prerequisites)?,
                    "criterion refresh requires new evidence bindings"
                );
                let mut assessed = false;
                for (id, record) in self.records()? {
                    if let Record::Coding {
                        record: CodingRecord::CriterionReturn { assessment },
                    } = record
                    {
                        if assessment.frame.policy == previous {
                            self.require_project_bridge_source(id, Source::Council)?;
                            assessed = true;
                        }
                    }
                }
                ensure!(
                    assessed,
                    "previous criterion assessment must settle before refresh"
                );
                let budget = self.project_budget()?;
                ensure!(
                    !budget.stopped
                        && !budget.exceeded
                        && budget.unknown_usage == 0
                        && budget.unknown_compute_requests == 0
                        && budget.unknown_external_cost_requests == 0
                        && budget.unsettled_requests == 0
                        && budget.unresolved_tool_requests == 0,
                    "criterion refresh stopped or externally unresolved"
                );
            } else {
                ensure!(
                    self.criterion_policy()?.is_none(),
                    "criterion policy already frozen for this revision"
                );
            }
            let c = self.project_acceptance_status()?;
            ensure!(
                Some(policy.requirements_revision) == c.requirements_revision,
                "stale requirements"
            );
            let requirements = self.project_acceptance_requirements_view()?;
            let published = self.acceptance_cycle()?;
            let public = published["guidance"]
                .as_object()
                .context("public guidance required")?;
            ensure!(
                !policy.requirements.is_empty()
                    && policy
                        .requirements
                        .iter()
                        .all(|(k, v)| public.contains_key(k) && requirements.get(k) == Some(v)),
                "criterion publication must match original public scope"
            );
            ensure!(
                policy.coverage.keys().eq(policy.requirements.keys())
                    && policy.prerequisites.keys().eq(policy.requirements.keys()),
                "complete criterion policy required"
            );
            if let Some(scope) = &policy.assessment_scope {
                validate_scope(&policy.requirements, scope)?;
            }
            let spec = self.project_configuration()?;
            let mut allowed: BTreeSet<_> = spec
                .editable
                .keys()
                .cloned()
                .chain(std::iter::once("test_receipt".into()))
                .collect();
            for p in policy.prerequisites.values() {
                for binding in p.supplemental_bindings() {
                    self.observed_browser_supplemental(binding.admission, binding.receipt)?;
                    allowed.insert(supplemental_key(binding.admission, binding.receipt));
                }
            }
            for paths in policy.coverage.values() {
                ensure!(
                    !paths.is_empty() && paths.is_subset(&allowed),
                    "protected or unknown criterion evidence"
                );
            }
            let identity = self.project_verifier_test_identity()?;
            for (key, p) in &policy.prerequisites {
                match p {
                    typed::Prerequisite::StaticSemantic => ensure!(
                        policy.coverage[key].iter().any(|k| k != "test_receipt"),
                        "static source coverage required"
                    ),
                    typed::Prerequisite::ObservedTests { test_identity }
                    | typed::Prerequisite::Mixed { test_identity } => {
                        ensure!(
                            *test_identity == identity
                                && policy.coverage[key].contains("test_receipt"),
                            "exact observed test scope required"
                        );
                        if matches!(p, typed::Prerequisite::Mixed { .. }) {
                            ensure!(
                                policy.coverage[key].iter().any(|k| k != "test_receipt"),
                                "mixed source coverage required"
                            );
                        }
                    }
                    typed::Prerequisite::ObservedSupplementalSet { observations } => {
                        ensure!(
                            policy.assessment_scope.is_some(),
                            "cumulative observations require explicit artifact scope"
                        );
                        self.validate_supplemental_set(key, observations)?;
                        let keys = observations
                            .iter()
                            .map(|b| supplemental_key(b.admission, b.receipt))
                            .collect::<BTreeSet<_>>();
                        ensure!(
                            keys.is_subset(&policy.coverage[key]),
                            "complete cumulative criterion coverage required"
                        );
                    }
                    typed::Prerequisite::ObservedSupplemental { admission, receipt } => {
                        ensure!(
                            policy.assessment_scope.is_some(),
                            "observed supplemental requires explicit artifact scope"
                        );
                        let observation =
                            self.observed_browser_supplemental(*admission, *receipt)?;
                        ensure!(
                            observation.mapping.contains_key(key)
                                && policy.coverage[key]
                                    .contains(&supplemental_key(*admission, *receipt)),
                            "exact mapped supplemental criterion coverage required"
                        );
                    }
                    _ => anyhow::bail!(
                        "supplemental evidence requires its separate admitted channel"
                    ),
                }
            }
            self.adaptive_record(
                &Record::Coding {
                    record: match previous {
                        Some(previous) => {
                            CodingRecord::CriterionPolicyRefreshed { previous, policy }
                        }
                        None => CodingRecord::CriterionPolicy { policy },
                    },
                },
                Source::Operator,
            )
        })
    }
    fn criterion_frame(&self) -> Result<Option<Frame>> {
        let Some((policy_id, p)) = self.criterion_policy()? else {
            return Ok(None);
        };
        if self.adaptive_state()?.pending.is_some() || self.project_information_pending()?.is_some()
        {
            return Ok(None);
        }
        let c = self.project_acceptance_status()?;
        // A frozen set becomes historical after new source or newly registered
        // observations. Never apply it as current or silently drop new returns.
        for (key, prerequisite) in &p.prerequisites {
            if let typed::Prerequisite::ObservedSupplementalSet { observations } = prerequisite {
                if observations.is_empty()
                    || self.current_supplemental_bindings(key)?.as_ref() != Some(observations)
                {
                    return Ok(None);
                }
                self.validate_supplemental_set(key, observations)?;
            }
        }
        let Some(observed) = self.verifier_observed_tests(c.test_receipt, c.snapshot)? else {
            return Ok(None);
        };
        if observed.outcome == typed::Outcome::Unknown {
            return Ok(None);
        }
        let Some(publication) = self.criterion_publication()? else {
            return Ok(None);
        };
        self.require_project_bridge_source(publication, Source::Operator)?;
        let view = self.acceptance_cycle()?;
        let public = view["guidance"].as_object().context("published criteria")?;
        let active = self.acceptance_active_aims()?;
        if !p
            .requirements
            .keys()
            .all(|k| public.contains_key(k) && active.contains_key(k))
        {
            return Ok(None);
        }
        let spec = self.project_configuration()?;
        let snapshot = self.project_snapshot(&spec)?;
        ensure!(
            digest(&snapshot)? == c.snapshot,
            "source changed during criterion capture"
        );
        let mut evidence = BTreeMap::new();
        for key in p.coverage.values().flatten() {
            if key == "test_receipt" {
                continue;
            }
            if let Some((admission, receipt)) = p
                .prerequisites
                .values()
                .flat_map(|pr| pr.supplemental_bindings())
                .find_map(|b| {
                    if supplemental_key(b.admission, b.receipt) == *key {
                        Some((b.admission, b.receipt))
                    } else {
                        None
                    }
                })
            {
                let allowed = p
                    .prerequisites
                    .iter()
                    .filter_map(|(k, pr)| {
                        if pr
                            .supplemental_bindings()
                            .iter()
                            .any(|b| b.admission == admission && b.receipt == receipt)
                        {
                            Some(k.clone())
                        } else {
                            None
                        }
                    })
                    .collect::<BTreeSet<_>>();
                evidence.insert(
                    key.clone(),
                    serde_json::to_string(&supplemental_public(
                        &self.observed_browser_supplemental(admission, receipt)?,
                        &allowed,
                    ))?,
                );
                continue;
            }
            ensure!(
                spec.editable.contains_key(key),
                "criterion protected source excluded"
            );
            evidence.insert(
                key.clone(),
                visible_source(&spec, key, &snapshot[key])?.to_string(),
            );
        }
        evidence.insert("test_receipt".into(),serde_json::to_string(&json!({"observation":observed,"scope":"Native configured finite checks only; no player satisfaction or general semantic fulfillment. Protected checker source and command output excluded."}))?);
        Ok(Some(Frame {
            policy: policy_id,
            publication,
            snapshot: c.snapshot,
            requirements_revision: p.requirements_revision,
            receipt: observed.receipt,
            requirements: p.requirements,
            assessment_scope: p.assessment_scope,
            evidence,
            coverage: p.coverage,
            prerequisites: p.prerequisites,
            observed_tests: observed,
        }))
    }
    pub fn project_criterion_context(&self) -> Result<Value> {
        let mut history = vec![];
        for (id, r) in self.records()? {
            if let Record::Coding {
                record: CodingRecord::CriterionReturn { assessment },
            } = r
            {
                self.require_project_bridge_source(id, Source::Council)?;
                history.push(json!({"id":id,"assessment":assessment}));
            }
        }
        let frame = self.criterion_frame()?;
        // Separate, current published intent. Do not enrich historical Frames:
        // their immutable evidence and identity must continue to replay exactly.
        let published_context = if let Some(f) = &frame {
            let view = self.acceptance_cycle()?;
            let provenance = &view["guidance_provenance"];
            ensure!(
                provenance["publication"] == json!(f.publication)
                    && provenance["requirements_revision"] == json!(f.requirements_revision)
                    && provenance["current_source"] == json!(f.snapshot),
                "criterion guidance binding mismatch"
            );
            let public = view["guidance"].as_object().context("criterion guidance")?;
            let mut guidance = BTreeMap::new();
            for key in f.requirements.keys() {
                guidance.insert(
                    key.clone(),
                    public
                        .get(key)
                        .context("missing criterion guidance")?
                        .clone(),
                );
            }
            json!({"version":"criterion-published-context@v1", "guidance":guidance,
                "provenance":provenance, "meaning":"Current operator-published task context, including any explicitly attributed synthetic conditions. Not independently observed world facts, criterion findings, user acceptance, or action authority. Criteria remain the frozen published requirements."})
        } else {
            Value::Null
        };
        let current = frame
            .as_ref()
            .and_then(|f| {
                history
                    .iter()
                    .rev()
                    .find(|h| h["assessment"]["frame"] == json!(f))
            })
            .cloned();
        let root = self.store.head_root()?;
        let binding = Hash::of(&[
            b"public-criterion-return@v1",
            &serde_json::to_vec(&json!({"root":root,"frame":frame}))?,
        ]);
        Ok(
            json!({"policy":self.criterion_policy()?,"state_root":root,"binding":binding,"frame":frame,"published_context":published_context,"current":current,"history":history}),
        )
    }
    /// The transport duplicates an authenticated frame; its capacity must scale
    /// with that frame, not the old small generic acceptance-file envelope.
    /// Native apply still rechecks the complete current frame and final CAS.
    pub fn project_criterion_apply_from_file(&self, path: &std::path::Path) -> Result<EventId> {
        use std::io::Read;
        let context = self.project_criterion_context()?;
        let frame: Frame = serde_json::from_value(context["frame"].clone())?;
        // JSON escaping can expand each UTF-8 byte to six ASCII bytes. Two
        // narrative fields per finding retain the existing decoded allowance.
        let frame_bytes = serde_json::to_vec(&frame)?.len();
        let narrative_bytes = frame
            .requirements
            .len()
            .checked_mul(2 * browser::RESPONSE_BYTES)
            .context("criterion input capacity overflow")?;
        let limit = frame_bytes
            .checked_add(narrative_bytes)
            .and_then(|n| n.checked_mul(6))
            .and_then(|n| n.checked_add(16384))
            .context("criterion input capacity overflow")?;
        let mut bytes = Vec::new();
        std::fs::File::open(path)?
            .take((limit as u64) + 1)
            .read_to_end(&mut bytes)?;
        ensure!(
            bytes.len() <= limit,
            "criterion input exceeds current frame-derived transport capacity: {} > {} bytes",
            bytes.len(),
            limit
        );
        self.project_criterion_apply(serde_json::from_slice(&bytes)?)
    }
    pub fn project_criterion_apply(&self, input: Apply) -> Result<EventId> {
        self.write(|| {
            let context = self.project_criterion_context()?;
            // Exact committed response recovery, not a second assessment.
            if let Some(id) = context["current"].get("id") {
                ensure!(
                    context["current"]["assessment"] == json!(input),
                    "criterion return already assessed"
                );
                return Ok(serde_json::from_value(id.clone())?);
            }
            ensure!(
                self.store.head_root()? == input.state_root
                    && context["binding"] == json!(input.binding)
                    && context["frame"] == json!(input.frame),
                "stale criterion return binding"
            );
            let budget = self.project_budget()?;
            ensure!(
                budget.active
                    && !budget.stopped
                    && !budget.exceeded
                    && budget.unknown_usage == 0
                    && budget.unknown_compute_requests == 0
                    && budget.unsettled_requests == 0
                    && budget.unresolved_tool_requests == 0,
                "criterion assessment stopped or unresolved effects"
            );
            ensure!(
                short(&input.reviewer)
                    && short(&input.generator)
                    && input.reviewer != input.generator,
                "independent criterion reviewer required"
            );
            ensure!(
                input.findings.keys().eq(input.frame.requirements.keys()),
                "complete published criterion findings required"
            );
            ensure!(
                matches!(
                    input.decision.as_str(),
                    "accept" | "reject" | "unknown" | "abstain"
                ),
                "criterion decision"
            );
            ensure!(
                input.decision != "accept"
                    || input.findings.values().all(|f| f.finding == Finding::Met),
                "criterion acceptance consistency"
            );
            ensure!(
                input.decision != "reject"
                    || input
                        .findings
                        .values()
                        .any(|f| f.finding == Finding::NotMet),
                "criterion rejection consistency"
            );
            ensure!(
                input.decision != "unknown"
                    || input
                        .findings
                        .values()
                        .any(|f| f.finding == Finding::Unknown),
                "criterion uncertainty consistency"
            );
            let a = reviewer::Admission {
                version: "verifier-admission@v2".into(),
                scope: input.frame.snapshot,
                requirements_revision: input.frame.requirements_revision,
                verifier_identity: input.reviewer.clone(),
                generator_identity: input.generator.clone(),
                artifact: input.response_digest,
                coverage: input.frame.coverage.clone(),
                prerequisites: input.frame.prerequisites.clone(),
                max_requests: 1,
                tokens_per_request: 1,
                timeout_ms: 1,
                expires_at: String::new(),
            };
            let r = reviewer::Request {
                version: "verifier-request@v2".into(),
                admission: input.frame.policy,
                ordinal: 1,
                factual_root: input.state_root,
                snapshot: input.frame.snapshot,
                requirements_revision: input.frame.requirements_revision,
                test_receipt: Some(input.frame.receipt),
                requirements: input.frame.requirements.clone(),
                evidence: input.frame.evidence.clone(),
                evidence_digest: Hash::of(&[&serde_json::to_vec(&input.frame.evidence)?]),
                observed_tests: Some(input.frame.observed_tests.clone()),
                supplemental: self.supplemental_current()?,
            };
            self.validate_criterion_report(
                &r,
                &a,
                &reviewer::Report {
                    findings: input.findings.clone(),
                },
            )?;
            for (key, f) in &input.findings {
                ensure!(
                    f.evidence == input.frame.coverage[key]
                        && browser::narrative(&f.explanation)
                        && browser::narrative(&f.limitations),
                    "incomplete criterion evidence or explanation"
                );
            }
            let unmet = input
                .findings
                .iter()
                .filter(|(_, f)| input.decision != "abstain" && f.finding == Finding::NotMet)
                .map(|(k, _)| {
                    (
                        CriterionId(Hash::of(&[b"acceptance", k.as_bytes()])),
                        UnmetKind::WrongBehaviour,
                    )
                })
                .collect::<Vec<_>>();
            let receipt = input.frame.receipt;
            let policy = input.frame.policy;
            let success = if input.decision != "abstain"
                && input.findings.values().any(|f| f.finding == Finding::Met)
                && input.frame.assessment_scope.is_some()
            {
                Some(tsr_domain::ScopedSuccess {
                    assessment: input.frame.receipt,
                    receipt: input.frame.receipt,
                    publication: input.frame.publication,
                    requirements_revision: input.frame.requirements_revision,
                    source_snapshot: input.frame.snapshot,
                    scope_digest: Hash::of(&[&serde_json::to_vec(&input.frame)?]),
                    criteria: input
                        .findings
                        .iter()
                        .filter(|(_, f)| f.finding == Finding::Met)
                        .map(|(k, _)| k.clone())
                        .collect(),
                })
            } else {
                None
            };
            let id = self.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::CriterionReturn {
                        assessment: Box::new(input),
                    },
                },
                Source::Council,
            )?;
            if let Some(mut success) = success {
                success.assessment = id;
                self.append(EventKind::ScopedSuccess { success }, Source::Council)?;
            }
            if !unmet.is_empty() {
                let contrast = Filter::contrast_from_criteria(id, unmet, vec![policy, receipt, id]);
                self.append(
                    EventKind::Observed {
                        observation: Observation::Return {
                            surprise: vec![],
                            contrast: Some(contrast),
                        },
                    },
                    Source::Council,
                )?;
            }
            Ok(id)
        })
    }
    pub(crate) fn criterion_current_contrast(&self) -> Result<Option<Contrast>> {
        let context = self.project_criterion_context()?;
        let Some(id) = context["current"].get("id") else {
            return Ok(None);
        };
        let id: EventId = serde_json::from_value(id.clone())?;
        Ok(self.events()?.into_iter().find_map(|(_, e)| match e.kind {
            EventKind::Observed {
                observation:
                    Observation::Return {
                        contrast: Some(c), ..
                    },
            } if e.source == Source::Council && c.from_return == id => Some(c),
            _ => None,
        }))
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    fn setup() -> Result<(tempfile::TempDir, Engine)> {
        setup_with("static")
    }
    #[test]
    fn criterion_scope_preserves_original_and_never_assesses_engine_only() -> Result<()> {
        let original = "Keep navigation dependable. Preserve contributor history.";
        let requirements = BTreeMap::from([("orientation".into(), original.into())]);
        let parts = vec![
            ScopePart {
                domain: CriterionDomain::Artifact,
                text: "Keep navigation dependable. ".into(),
            },
            ScopePart {
                domain: CriterionDomain::EngineInvariant,
                text: "Preserve contributor history.".into(),
            },
        ];
        let scope = BTreeMap::from([("orientation".into(), parts.clone())]);
        validate_scope(&requirements, &scope)?;
        for bad in [
            vec![parts[0].clone()],
            vec![parts[1].clone(), parts[0].clone()],
            vec![ScopePart {
                domain: CriterionDomain::EngineInvariant,
                text: original.into(),
            }],
        ] {
            assert!(validate_scope(
                &requirements,
                &BTreeMap::from([("orientation".into(), bad)])
            )
            .is_err());
        }
        let (_d, e) = setup_with("static")?;
        let (_id, policy) = e.criterion_policy()?.unwrap();
        let legacy = serde_json::to_value(&policy)?;
        assert!(legacy.get("assessment_scope").is_none());
        let roundtrip: Policy = serde_json::from_value(legacy.clone())?;
        assert_eq!(serde_json::to_value(roundtrip)?, legacy);
        Ok(())
    }

    pub(crate) fn setup_with(kind: &str) -> Result<(tempfile::TempDir, Engine)> {
        let d = tempfile::tempdir()?;
        let e = Engine::open(d.path())?;
        e.project_initialize(browser::tests::spec())?;
        let mut wants = BTreeMap::from([
            ("orientation".into(), "Keep navigation dependable".into()),
            (
                "discovery".into(),
                "Make unfamiliar perspectives accessible".into(),
            ),
        ]);
        if kind == "success_extra" {
            wants.insert(
                "unrelated".into(),
                "Preserve a separate unresolved obligation".into(),
            );
        }
        e.project_acceptance_requirements(wants.clone())?;
        let c = e.project_acceptance_status()?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: c.requirements_revision.unwrap(),
            decision_record: c.decision_record,
            guidance: wants.clone(),
        })?;
        e.project_lifecycle_open(lifecycle::Admission {
            state_root: e.store.head_root()?,
            training: None,
            tool_ms_ceiling: 100_000,
            reason: "Authored criterion fixture, no provider calls".into(),
            non_tool: Some(budget::Limits {
                model_calls: 10,
                model_tokens: 10000,
                wall_ms: 600000,
                patch_bytes: 10000,
                external_cost_microusd: 0,
            }),
        })?;
        let paths = BTreeSet::from(["index.html".into(), "test_receipt".into()]);
        let prerequisite = match kind {
            "observed" => typed::Prerequisite::ObservedTests {
                test_identity: e.project_verifier_test_identity()?,
            },
            "mixed" => typed::Prerequisite::Mixed {
                test_identity: e.project_verifier_test_identity()?,
            },
            _ => typed::Prerequisite::StaticSemantic,
        };
        e.project_criterion_policy(Policy {
            state_root: e.store.head_root()?,
            requirements_revision: c.requirements_revision.unwrap(),
            assessment_scope: if kind == "success" || kind == "success_extra" {
                Some(
                    wants
                        .iter()
                        .map(|(k, v)| {
                            (
                                k.clone(),
                                vec![ScopePart {
                                    domain: CriterionDomain::Artifact,
                                    text: v.clone(),
                                }],
                            )
                        })
                        .collect(),
                )
            } else {
                None
            },
            coverage: wants.keys().map(|k| (k.clone(), paths.clone())).collect(),
            prerequisites: wants
                .keys()
                .map(|k| (k.clone(), prerequisite.clone()))
                .collect(),
            requirements: wants,
        })?;
        Ok((d, e))
    }
    pub(crate) fn receipt(
        e: &Engine,
        verdict: Verdict,
        source: Hash,
        origin: Source,
    ) -> Result<()> {
        // Explicit authored native receipt for validation tests, never live evidence.
        e.adaptive_record(
            &Record::Coding {
                record: CodingRecord::Receipt {
                    commitment: EventId(Hash::of(&[b"authored criterion receipt"])),
                    verdict,
                    runs: vec!["PRIVATE RUN".into()],
                    source_after: Some(source),
                    observations: vec![ProbeObservation {
                        passed: Some(true),
                        argv: vec!["PRIVATE ARG".into()],
                        expected_stdout: "PRIVATE EXPECTATION".into(),
                        exit: 0,
                        stdout: "PRIVATE OUT".into(),
                        stderr: "PRIVATE ERR".into(),
                        abbreviated: false,
                        case_id: Some("private-case".into()),
                    }],
                },
            },
            origin,
        )?;
        Ok(())
    }
    pub(crate) fn input(e: &Engine, finding: Finding) -> Result<Apply> {
        let c = e.project_criterion_context()?;
        let f: Frame = serde_json::from_value(c["frame"].clone())?;
        let findings = f
            .requirements
            .keys()
            .map(|k| {
                (
                    k.clone(),
                    Assessment {
                        finding: finding.clone(),
                        evidence: f.coverage[k].clone(),
                        explanation: "Authored independent source assessment".into(),
                        limitations: "Fixture, not actual semantics or player experience".into(),
                        basis: Some(if finding == Finding::Unknown {
                            typed::Basis::Unknown
                        } else {
                            typed::Basis::StaticSemantic
                        }),
                    },
                )
            })
            .collect();
        Ok(Apply {
            state_root: serde_json::from_value(c["state_root"].clone())?,
            binding: serde_json::from_value(c["binding"].clone())?,
            frame: f,
            reviewer: "authored independent assessor".into(),
            generator: "authored generator".into(),
            response_digest: Hash::of(&[b"authored response"]),
            decision: match finding {
                Finding::Met => "accept",
                Finding::NotMet => "reject",
                Finding::Unknown => "unknown",
            }
            .into(),
            findings,
        })
    }
    #[test]
    fn criterion_context_carries_only_current_published_condition() -> Result<()> {
        let (d, e) = setup()?;
        let publish = |text: &str| -> Result<()> {
            let c = e.project_acceptance_status()?;
            e.project_acceptance_feedback(acceptance::Feedback {
                requirements_revision: c.requirements_revision.unwrap(),
                decision_record: c.decision_record,
                guidance: BTreeMap::from([
                    ("orientation".into(), text.into()),
                    (
                        "discovery".into(),
                        "Authored synthetic reading topic".into(),
                    ),
                ]),
            })?;
            Ok(())
        };
        publish("Authored synthetic opening: No previous directions.")?;
        let source = e.project_acceptance_status()?.snapshot;
        receipt(&e, Verdict::Passed, source, Source::Rules)?;
        let first = e.project_criterion_context()?;
        assert_eq!(
            first["published_context"]["guidance"]["orientation"],
            "Authored synthetic opening: No previous directions."
        );
        assert_eq!(
            first["published_context"]["provenance"]["publication"],
            first["frame"]["publication"]
        );
        assert_eq!(
            first["published_context"]["provenance"]["current_source"],
            first["frame"]["snapshot"]
        );
        assert!(!first["published_context"].to_string().contains("storm"));
        assert!(first["frame"].get("published_context").is_none());
        let old = input(&e, Finding::Unknown)?;
        publish("Authored synthetic storm: previous directions superseded.")?;
        let second = e.project_criterion_context()?;
        assert_ne!(
            first["frame"]["publication"],
            second["frame"]["publication"]
        );
        assert_eq!(
            first["frame"]["requirements"],
            second["frame"]["requirements"]
        );
        assert_eq!(
            second["published_context"]["guidance"]["orientation"],
            "Authored synthetic storm: previous directions superseded."
        );
        assert!(!second["published_context"]
            .to_string()
            .contains("reopening"));
        assert!(e.project_criterion_apply(old).is_err());
        let id = e.project_criterion_apply(input(&e, Finding::Unknown)?)?;
        let before = e.project_criterion_context()?;
        assert_eq!(before["current"]["id"], json!(id));
        drop(e);
        assert_eq!(Engine::open(d.path())?.project_criterion_context()?, before);
        Ok(())
    }

    #[test]
    fn criterion_file_capacity_tracks_native_frame_and_preserves_cas() -> Result<()> {
        let (d, e) = setup()?;
        let source = e.project_acceptance_status()?.snapshot;
        receipt(&e, Verdict::Passed, source, Source::Rules)?;
        let mut a = input(&e, Finding::Unknown)?;
        for finding in a.findings.values_mut() {
            finding.explanation = "Authored finite uncertainty. ".repeat(800);
        }
        let path = d.path().join("large-criterion.json");
        let bytes = serde_json::to_vec(&a)?;
        assert!(bytes.len() > 16384);
        std::fs::write(&path, &bytes)?;
        let id = e.project_criterion_apply_from_file(&path)?;
        assert_eq!(e.project_criterion_apply_from_file(&path)?, id);
        let before = e.store.head_root()?;
        std::fs::write(&path, vec![b' '; 4_000_000])?;
        assert!(e
            .project_criterion_apply_from_file(&path)
            .unwrap_err()
            .to_string()
            .contains("frame-derived"));
        assert_eq!(e.store.head_root()?, before);
        a.frame.snapshot = Hash::of(&[b"stale authored frame"]);
        std::fs::write(&path, serde_json::to_vec(&a)?)?;
        assert!(e.project_criterion_apply_from_file(&path).is_err());
        assert_eq!(e.store.head_root()?, before);
        Ok(())
    }

    #[test]
    fn ordinary_criterion_return_preserves_acceptance_and_restart() -> Result<()> {
        let (d, e) = setup()?;
        assert!(e.project_criterion_context()?["frame"].is_null());
        let source = e.project_acceptance_status()?.snapshot;
        receipt(&e, Verdict::Passed, source, Source::Rules)?;
        let before = serde_json::to_value(e.project_acceptance_status()?)?;
        let parents = serde_json::to_value(Ledger::new(&e.store).collective()?.entries)?;
        let a = input(&e, Finding::NotMet)?;
        let encoded = serde_json::to_string(&a.frame)?;
        assert!(!encoded.contains("PRIVATE"));
        assert!(!encoded.contains("private-case"));
        let id = e.project_criterion_apply(a.clone())?;
        assert_eq!(e.project_criterion_apply(a)?, id);
        let c = e
            .project_current_contrast()?
            .context("known unmet Contrast")?;
        assert_eq!(c.from_return, id);
        assert_eq!(c.unmet.len(), 2);
        assert_eq!(
            serde_json::to_value(e.project_acceptance_status()?)?,
            before
        );
        assert_eq!(
            serde_json::to_value(Ledger::new(&e.store).collective()?.entries)?,
            parents
        );
        let history = e.project_criterion_context()?;
        drop(e);
        let e = Engine::open(d.path())?;
        assert_eq!(e.project_criterion_context()?, history);
        // A genuinely new receipt is a new experience. Explicit uncertainty must
        // not resurrect the earlier unmet conclusion.
        e.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    source: "authored new round".into(),
                    digest: Hash::of(&[b"new"]),
                },
            },
            Source::Rules,
        )?;
        receipt(&e, Verdict::Passed, source, Source::Rules)?;
        e.project_criterion_apply(input(&e, Finding::Unknown)?)?;
        assert!(e.project_current_contrast()?.is_none());
        assert_eq!(
            e.project_criterion_context()?["history"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        Ok(())
    }
    #[test]
    fn stale_unknown_foreign_and_wrong_basis_fail_closed() -> Result<()> {
        let (_d, e) = setup()?;
        let source = e.project_acceptance_status()?.snapshot;
        receipt(&e, Verdict::Passed, Hash::of(&[b"old"]), Source::Rules)?;
        assert!(e.project_criterion_context()?["frame"].is_null());
        receipt(&e, Verdict::Passed, source, Source::Rules)?;
        let a = input(&e, Finding::NotMet)?;
        for mode in 0..6 {
            let mut bad = a.clone();
            match mode {
                0 => bad.frame.snapshot = Hash::of(&[b"wrong"]),
                1 => bad.reviewer = bad.generator.clone(),
                2 => {
                    bad.findings.remove("discovery");
                }
                3 => bad.findings.get_mut("discovery").unwrap().basis = Some(typed::Basis::Unknown),
                4 => {
                    bad.findings
                        .get_mut("orientation")
                        .unwrap()
                        .evidence
                        .insert("protected".into());
                }
                _ => bad.frame.publication = EventId(Hash::of(&[b"wrong"])),
            }
            let root = e.store.head_root()?;
            assert!(e.project_criterion_apply(bad).is_err());
            assert_eq!(root, e.store.head_root()?);
        }
        receipt(&e, Verdict::Passed, source, Source::Generator)?;
        assert!(e.project_criterion_context().is_err());
        Ok(())
    }
    #[test]
    fn interrupted_apply_rolls_back_and_stop_blocks() -> Result<()> {
        let (_d, e) = setup()?;
        receipt(
            &e,
            Verdict::Passed,
            e.project_acceptance_status()?.snapshot,
            Source::Rules,
        )?;
        let a = input(&e, Finding::NotMet)?;
        let root = e.store.head_root()?;
        assert!(e
            .write(|| -> Result<()> {
                e.project_criterion_apply(a.clone())?;
                anyhow::bail!("authored interrupted transaction")
            })
            .is_err());
        assert_eq!(e.store.head_root()?, root);
        e.project_lifecycle_close_advisory("Authored Stop".into())?;
        assert!(e
            .project_criterion_apply(input(&e, Finding::NotMet)?)
            .is_err());
        Ok(())
    }
    #[test]
    fn source_publication_unknown_effects_and_unknown_receipt_stay_separate() -> Result<()> {
        let (d, e) = setup()?;
        let source = e.project_acceptance_status()?.snapshot;
        receipt(&e, Verdict::Unavailable, source, Source::Rules)?;
        assert!(e.project_criterion_context()?["frame"].is_null());
        receipt(&e, Verdict::Passed, source, Source::Rules)?;
        let prior = input(&e, Finding::NotMet)?;
        e.project_criterion_apply(prior.clone())?;
        let status = e.project_acceptance_status()?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: status.requirements_revision.unwrap(),
            decision_record: status.decision_record,
            guidance: BTreeMap::from([
                ("orientation".into(), "Authored republished meaning".into()),
                ("discovery".into(), "Authored retained discovery".into()),
            ]),
        })?;
        assert!(e.project_current_contrast()?.is_none());
        assert!(e.project_criterion_apply(prior).is_err());
        e.project_resource_unpriced_admit(e.store.head_root()?)?;
        e.project_resource_reserve_unpriced(budget::UnpricedReserve {
            request_id: "authored unknown model request".into(),
            state_root: e.store.head_root()?,
            envelope: "authored".into(),
            model: "authored".into(),
            tokens_upper: 10,
        })?;
        assert!(e
            .project_criterion_apply(input(&e, Finding::NotMet)?)
            .is_err());
        std::fs::write(
            d.path().join("coding-work/index.html"),
            "authored changed source",
        )?;
        assert!(e.project_criterion_context()?["frame"].is_null());
        Ok(())
    }
    #[test]
    fn observed_and_mixed_evidence_cannot_change_the_frozen_prerequisite() -> Result<()> {
        for kind in ["observed", "mixed"] {
            let (_d, e) = setup_with(kind)?;
            receipt(
                &e,
                Verdict::Passed,
                e.project_acceptance_status()?.snapshot,
                Source::Rules,
            )?;
            let mut a = input(&e, Finding::NotMet)?;
            let observed = a.frame.observed_tests.clone();
            for f in a.findings.values_mut() {
                f.basis = Some(if kind == "observed" {
                    typed::Basis::ObservedTests {
                        observation: observed.clone(),
                    }
                } else {
                    typed::Basis::Mixed {
                        observation: observed.clone(),
                        static_finding: Finding::NotMet,
                    }
                });
            }
            if kind == "observed" {
                assert!(e.project_criterion_apply(a.clone()).is_err());
                for f in a.findings.values_mut() {
                    f.finding = Finding::Met;
                }
                a.decision = "accept".into();
                e.project_criterion_apply(a)?;
                assert!(e.project_current_contrast()?.is_none());
            } else {
                e.project_criterion_apply(a)?;
                assert!(e.project_current_contrast()?.is_some());
            }
        }
        Ok(())
    }
    #[test]
    fn criterion_refusal_is_preserved_without_creating_contrast() -> Result<()> {
        let (_d, e) = setup()?;
        receipt(
            &e,
            Verdict::Passed,
            e.project_acceptance_status()?.snapshot,
            Source::Rules,
        )?;
        let mut a = input(&e, Finding::NotMet)?;
        a.decision = "abstain".into();
        e.project_criterion_apply(a)?;
        assert!(e.project_current_contrast()?.is_none());
        assert_eq!(
            e.project_criterion_context()?["current"]["assessment"]["decision"],
            "abstain"
        );
        Ok(())
    }
}
