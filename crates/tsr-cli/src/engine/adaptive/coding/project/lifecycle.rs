//! Versioned, prospective native-event projection. Never modifies factual decisions.
use super::*;
use tsr_learn::decision::LabelRecord;
const SOURCE: &str = "project-common-event-lifecycle@v1";
pub use tsr_learn::project_eligibility::LifecycleAdmission as Admission;
#[derive(Clone, Debug, Serialize, Deserialize)]
struct Open {
    admission: Admission,
    prefix: Vec<EventId>,
    snapshot: Hash,
    at: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "phase", rename_all = "snake_case")]
enum Lifecycle {
    Open { open: Box<Open> },
    Closed { projection: Box<Projection> },
    ResourceExtension { request: Box<ResourceExtension> },
}
/// Explicit cumulative resource authority for continuing a settled project. Never resets usage.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ResourceExtension {
    pub state_root: Hash,
    pub previous: Admission,
    pub limits: budget::Limits,
    pub tool_ms_ceiling: u64,
    pub reason: String,
}
fn effective_admission(records: &[Lifecycle], initial: &Admission) -> Result<Admission> {
    let mut admission = initial.clone();
    for record in records.iter().skip(1) {
        if let Lifecycle::ResourceExtension { request } = record {
            ensure!(
                request.previous == admission,
                "resource extension lineage changed"
            );
            admission.non_tool = Some(request.limits.clone());
            admission.tool_ms_ceiling = request.tool_ms_ceiling;
        }
    }
    Ok(admission)
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Projection {
    pub version: String,
    pub task: TaskId,
    pub admission: Admission,
    pub opening_snapshot: Hash,
    pub factual_prefix: Hash,
    pub events: Vec<(i64, Event)>,
    pub mapping: Vec<(EventId, EventId, String)>,
    pub consumed_tool_ms: u64,
    pub incomplete_executions: usize,
    pub label: Option<LabelRecord>,
    pub reviewed_accepted: Option<bool>,
    pub native_feature_compatible: bool,
    pub native_ineligibility: String,
    pub review: Option<EventId>,
    pub source_snapshot: Option<Hash>,
    pub closed: bool,
    pub eligibility: String,
    #[serde(default)]
    pub resource_accounting: Option<serde_json::Value>,
}
fn append(p: &mut Projection, kind: EventKind, source: Source, at: &str) -> EventId {
    let parents = p.events.last().map(|(_, e)| vec![e.id]).unwrap_or_default();
    let e = Event::new(p.task, parents, source, kind, at);
    let id = e.id;
    p.events.push((p.events.len() as i64 + 1, e));
    id
}
/// Exact millisecond conversion; legacy evidence is never rewritten in place.
pub(crate) fn timestamp(stamp: &str) -> Result<String> {
    if let Some(ms) = stamp.strip_prefix("unix-ms:") {
        let ms: u64 = ms.parse()?;
        ensure!(ms <= 253_402_300_799_999, "timestamp out of RFC3339 range");
        let secs = ms / 1000;
        let z = (secs / 86400) as i64 + 719468;
        let era = z / 146097;
        let doe = (z - era * 146097) as u64;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let year = yoe as i64 + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = doy - (153 * mp + 2) / 5 + 1;
        let month = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = year + i64::from(month <= 2);
        let rem = secs % 86400;
        return Ok(format!(
            "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
            rem / 3600,
            rem % 3600 / 60,
            rem % 60,
            ms % 1000
        ));
    }
    tsr_learn::maturity::timestamp(stamp)?;
    Ok(stamp.into())
}
pub(crate) fn now() -> Result<String> {
    timestamp(&format!(
        "unix-ms:{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_millis()
    ))
}
impl Engine {
    fn lifecycle_records(&self) -> Result<Vec<Lifecycle>> {
        let side = self.shadow_store()?;
        let mut records = vec![];
        for (_, e) in side.events()? {
            if let EventKind::Observed {
                observation: Observation::Evidence { source, digest },
            } = e.kind
            {
                if source != SOURCE {
                    continue;
                }
                ensure!(
                    e.source == Source::Operator,
                    "invalid lifecycle admission/close provenance"
                );
                let bytes = fs::read(side.directory.join("cas").join(digest.to_hex()))?;
                ensure!(Hash::of(&[&bytes]) == digest, "changed lifecycle record");
                records.push(serde_json::from_slice(&bytes)?);
            }
        }
        Ok(records)
    }
    fn lifecycle_record(&self, record: &Lifecycle) -> Result<()> {
        let side = self.shadow_store()?;
        side.write(|| {
            let digest = side.blob(&serde_json::to_vec(record)?)?;
            side.append(
                EventKind::Observed {
                    observation: Observation::Evidence {
                        source: SOURCE.into(),
                        digest,
                    },
                },
                Source::Operator,
            )?;
            Ok(())
        })
    }
    pub(crate) fn project_resource_admission(
        &self,
    ) -> Result<Option<(TaskId, Admission, String, bool)>> {
        if !self.directory.join("shadow").exists() {
            return Ok(None);
        }
        let records = self.lifecycle_records()?;
        let Some(Lifecycle::Open { open }) = records.first() else {
            return Ok(None);
        };
        let task = TaskId(Hash::of(&[SOURCE.as_bytes(), &serde_json::to_vec(open)?]));
        Ok(Some((
            task,
            effective_admission(&records, &open.admission)?,
            open.at.clone(),
            matches!(records.last(), Some(Lifecycle::Closed { .. })),
        )))
    }
    pub fn project_resource_extend(&self, request: ResourceExtension) -> Result<budget::View> {
        self.write(|| {
            let records = self.lifecycle_records()?;
            ensure!(!matches!(records.last(), Some(Lifecycle::Closed { .. })), "closed episode cannot be reopened");
            if matches!(records.last(), Some(Lifecycle::ResourceExtension { request: prior }) if **prior == request) {
                return self.project_budget();
            }
            let (_, current, _, closed) = self.project_resource_admission()?.ok_or_else(|| anyhow::anyhow!("resource admission required"))?;
            ensure!(!closed && current == request.previous && request.state_root == self.store.head_root()?, "stale resource extension");
            ensure!(current.training.is_none() && short(&request.reason), "bounded non-training extension required");
            ensure!(self.adaptive_state_inner()?.pending.is_none() && self.project_information_pending()?.is_none() && !self.verifier_has_unfinished()?, "pending work prevents resource extension");
            let used = self.project_budget()?;
            ensure!(used.active && used.unsettled_requests == 0 && used.unresolved_tool_requests == 0 && used.unknown_usage == 0 && used.unknown_compute_requests == 0 && used.unknown_external_cost_requests == 0, "unknown resources prevent extension");
            ensure!(used.unlimited_mode || (request.limits.model_calls >= used.model_calls && request.limits.model_tokens >= used.tokens_charged && request.limits.patch_bytes >= used.patch_bytes_charged && request.limits.external_cost_microusd >= used.external_cost_charged_microusd && request.limits.wall_ms > used.elapsed_wall_ms && request.tool_ms_ceiling >= used.tool_ms_charged), "extension below consumed resources");
            ensure!(used.unlimited_mode || (request.tool_ms_ceiling <= 86_400_000 && request.limits.wall_ms - used.elapsed_wall_ms <= 86_400_000), "bounded prospective extension required");
            self.lifecycle_record(&Lifecycle::ResourceExtension { request: Box::new(request) })?;
            self.project_budget()
        })
    }
    pub fn project_lifecycle_open(&self, admission: Admission) -> Result<Projection> {
        self.write(|| {
            let records = self.lifecycle_records()?;
            if let Some(Lifecycle::Open { open }) = records.first() {
                ensure!(
                    open.admission == admission,
                    "episode already admitted; budget cannot be reset"
                );
                return self.project_lifecycle_view();
            }
            ensure!(
                admission.state_root == self.store.head_root()?,
                "stale lifecycle admission"
            );
            ensure!(
                admission.tool_ms_ceiling > 0
                    && admission.tool_ms_ceiling <= 86_400_000
                    && short(&admission.reason),
                "bounded explicit tool budget and reason required"
            );
            ensure!(
                self.adaptive_state_inner()?.pending.is_none()
                    && self.project_information_pending()?.is_none(),
                "cannot admit an episode during execution"
            );
            if let Some(limits) = &admission.non_tool {
                ensure!(
                    limits.wall_ms > 0,
                    "positive wall budget required; zero model/patch budgets forbid those resources"
                );
            }
            if let Some(training) = &admission.training {
                training.validate()?;
            }
            let spec = self.project_configuration()?;
            let snapshot = digest(&self.project_snapshot(&spec)?)?;
            let open = Open {
                admission,
                prefix: self.events()?.iter().map(|(_, e)| e.id).collect(),
                snapshot,
                at: now()?,
            };
            self.lifecycle_record(&Lifecycle::Open {
                open: Box::new(open),
            })?;
            self.project_lifecycle_view()
        })
    }
    pub fn project_lifecycle_view(&self) -> Result<Projection> {
        self.project_lifecycle_prefix(&self.events()?)
    }
    pub(crate) fn project_lifecycle_prefix(&self, events: &[(i64, Event)]) -> Result<Projection> {
        let records = self.lifecycle_records()?;
        if let Some(Lifecycle::Closed { projection }) = records.last() {
            return Ok(*projection.clone());
        }
        let Some(Lifecycle::Open { open }) = records.first() else {
            bail!("no prospective lifecycle admission")
        };
        ensure!(
            records
                .iter()
                .skip(1)
                .all(|r| matches!(r, Lifecycle::ResourceExtension { .. })),
            "invalid lifecycle history"
        );
        ensure!(
            events.len() >= open.prefix.len()
                && events
                    .iter()
                    .take(open.prefix.len())
                    .map(|(_, e)| e.id)
                    .eq(open.prefix.iter().copied()),
            "lifecycle source prefix mismatch"
        );
        let task = TaskId(Hash::of(&[SOURCE.as_bytes(), &serde_json::to_vec(open)?]));
        let mut p=Projection{resource_accounting:None,version:tsr_learn::project_eligibility::FORECAST_PROJECTION.into(),task,admission:effective_admission(&records, &open.admission)?,opening_snapshot:open.snapshot,
            factual_prefix:Hash::of(&[&serde_json::to_vec(events)?]),events:vec![],mapping:vec![],consumed_tool_ms:0,incomplete_executions:0,
            label:None,reviewed_accepted:None,native_feature_compatible:false,
            native_ineligibility:"project-receipt-features@v1 requires a matching validated model; legacy/uninstrumented resource coverage remains unknown; candidate parents retain external original-event references".into(),review:None,source_snapshot:None,closed:false,eligibility:"open; tool-time-only accounting; independence and maturity unestablished".into()};
        append(
            &mut p,
            EventKind::TaskOpened { task },
            Source::Operator,
            &open.at,
        );
        let mut previous = tsr_learn::maturity::timestamp(&open.at)?;
        let mut proposed = BTreeSet::new();
        let mut committed = BTreeSet::new();
        let mut executed = BTreeSet::new();
        for (_, e) in events.iter().skip(open.prefix.len()) {
            let at = timestamp(&e.recorded_at)?;
            let t = tsr_learn::maturity::timestamp(&at)?;
            ensure!(t >= previous, "invalid lifecycle chronology");
            previous = t;
            let include = match &e.kind {
                EventKind::Proposed { candidate } => {
                    proposed.insert(candidate.id);
                    true
                }
                EventKind::Forecast {
                    forecast,
                    candidate,
                } if forecast.event == tsr_learn::decision_event_id() => {
                    ensure!(
                        proposed.contains(candidate)
                            && !committed.contains(candidate)
                            && matches!(
                                (e.source, forecast.source.clone()),
                                (Source::Generator, tsr_domain::Forecaster::Generator)
                                    | (Source::Rules, tsr_domain::Forecaster::Controller)
                            ),
                        "invalid common-event forecast provenance/chronology"
                    );
                    true
                }
                EventKind::Committed {
                    candidate,
                    decision: tsr_domain::Decision::Selected { value, .. },
                } => {
                    ensure!(
                        candidate == value
                            && proposed.contains(candidate)
                            && committed.insert(*candidate),
                        "invalid lifecycle commitment"
                    );
                    true
                }
                EventKind::Executed {
                    candidate,
                    duration_ms,
                    ..
                } => {
                    ensure!(
                        e.source == Source::Rules
                            && committed.contains(candidate)
                            && executed.insert(*candidate),
                        "invalid or repeated execution observation"
                    );
                    p.consumed_tool_ms = p
                        .consumed_tool_ms
                        .checked_add(*duration_ms)
                        .ok_or_else(|| anyhow::anyhow!("tool accounting overflow"))?;
                    true
                }
                _ => false,
            };
            if include {
                let id = append(&mut p, e.kind.clone(), e.source, &at);
                p.mapping.push((e.id, id, e.recorded_at.clone()));
            }
        }
        p.incomplete_executions = committed.difference(&executed).count();
        Ok(p)
    }
    /// Terminal accounting only: verification occurred, but no acceptance judgment is made.
    pub fn project_lifecycle_close_inconclusive(&self, reason: String) -> Result<Projection> {
        self.write(|| {
            ensure!(short(&reason), "bounded inconclusive closure reason required");
            let objective=self.project_objective_context()?;
            ensure!(objective.is_null()||objective["context"]["next"]=="stop","finite objective remains open; explicit objective closure required");
            let mut p = self.project_lifecycle_view()?;
            if p.closed {
                ensure!(p.events.iter().any(|(_, e)| matches!(&e.kind, EventKind::TaskClosed { reason, .. } if reason.starts_with("verification_inconclusive@v1:"))), "existing closure cannot be reclassified");
                return Ok(p);
            }
            ensure!(self.adaptive_state_inner()?.pending.is_none() && self.project_information_pending()?.is_none() && p.incomplete_executions == 0 && !self.verifier_has_unfinished()?, "unfinished execution or verifier prevents inconclusive closure");
            let resources = self.project_budget()?;
            ensure!(resources.active && resources.unsettled_requests == 0 && resources.unresolved_tool_requests == 0 && resources.patch_bytes_charged == 0, "resolved zero-edit resource accounting required");
            let records = self.records()?;
            let events = self.events()?;
            let spec = self.project_configuration()?;
            let snapshot = digest(&self.project_snapshot(&spec)?)?;
            ensure!(snapshot == p.opening_snapshot, "verification-only source changed since admission");
            let configuration = Hash::of(&[&serde_json::to_vec(&spec)?]);
            let mut count = 0;
            for (original, _, _) in &p.mapping {
                let event = events.iter().find(|(_, e)| e.id == *original).ok_or_else(|| anyhow::anyhow!("missing mapped event"))?;
                if let EventKind::Committed { candidate, .. } = event.1.kind {
                    ensure!(event.1.source == Source::Rules && events.iter().any(|(_, e)| matches!(&e.kind, EventKind::Proposed { candidate: c } if c.id == candidate && c.kind == tsr_domain::ActionKind::RunApprovedTest)), "verification candidate required");
                    let action = records.iter().find_map(|(id, r)| match r { Record::Pending { commitment, action, .. } if *commitment == *original && events.iter().any(|(_, e)| e.id == *id && e.source == Source::Rules) => Some(action), _ => None }).ok_or_else(|| anyhow::anyhow!("verification commitment binding missing"))?;
                    ensure!(self.project_verification_admitted(action)? == Some((snapshot, configuration)), "source-bound operator verification required");
                    let plan = records.iter().find_map(|(id, r)| match r { Record::Coding { record: CodingRecord::Intent { plan } } if plan.commitment == *original && events.iter().any(|(_, e)| e.id == *id && e.source == Source::Rules) => Some(plan), _ => None }).ok_or_else(|| anyhow::anyhow!("execution intent missing"))?;
                    ensure!(plan.edits.is_empty() && plan.source_before == snapshot && plan.expected_after == Some(snapshot), "verification intent changed source");
                    ensure!(records.iter().any(|(id, r)| events.iter().any(|(_, e)| e.id == *id && e.source == Source::Rules) && matches!(r, Record::Coding { record: CodingRecord::Receipt { commitment, source_after, .. } } if *commitment == *original && *source_after == Some(snapshot))), "source-bound returned receipt required");
                    count += 1;
                }
            }
            ensure!(count > 0, "executed verification required; use advisory closure for no execution");
            let at = now()?;
            ensure!(tsr_learn::maturity::timestamp(&at)? >= tsr_learn::maturity::timestamp(&p.events.last().unwrap().1.recorded_at)?, "invalid inconclusive close chronology");
            if (!resources.unlimited_mode && p.consumed_tool_ms > p.admission.tool_ms_ceiling) || resources.exceeded {
                append(&mut p, EventKind::Stop { reason: "budget".into() }, Source::Rules, &at);
            }
            let task = p.task;
            append(&mut p, EventKind::TaskClosed { task, reason: format!("verification_inconclusive@v1:{reason}") }, Source::Operator, &at);
            p.resource_accounting = Some(serde_json::to_value(resources)?);
            p.reviewed_accepted = None; p.review = None; p.label = None; p.closed = true;
            p.source_snapshot = Some(snapshot);
            p.native_feature_compatible = false;
            p.native_ineligibility = "verification-only inconclusive closure has no acceptance judgment".into();
            p.eligibility = "verification-only inconclusive; no outcome label or maturity eligibility".into();
            ensure!(digest(&self.project_snapshot(&spec)?)? == snapshot, "source changed during closure");
            self.lifecycle_record(&Lifecycle::Closed { projection: Box::new(p.clone()) })?;
            Ok(p)
        })
    }
    pub fn project_lifecycle_close_advisory(&self, reason: String) -> Result<Projection> {
        self.write(||{
            ensure!(short(&reason),"bounded advisory closure reason required");
            let objective=self.project_objective_context()?; ensure!(objective.is_null()||objective["context"]["next"]=="stop","finite objective remains open; explicit objective closure required");
            let mut p=self.project_lifecycle_view()?;
            if p.closed {
                ensure!(p.events.iter().any(|(_,e)|matches!(&e.kind,EventKind::TaskClosed{reason,..} if reason.starts_with("advisory_only:"))),"existing reviewed closure cannot become advisory");
                return Ok(p)
            }
            ensure!(self.adaptive_state_inner()?.pending.is_none() && self.project_information_pending()?.is_none() && p.incomplete_executions==0 && !self.verifier_has_unfinished()?,"unfinished execution or verifier invocation prevents advisory closure");
            let resources=self.project_budget()?;
            ensure!(resources.unsettled_requests==0 && resources.unresolved_tool_requests==0,"unsettled resources prevent advisory closure");
            ensure!(!p.events.iter().any(|(_,e)|matches!(e.kind,EventKind::Executed{..})),"advisory-only closure cannot relabel executed project episode");
            let at=now()?;
            ensure!(tsr_learn::maturity::timestamp(&at)? >= tsr_learn::maturity::timestamp(&p.events.last().unwrap().1.recorded_at)?,"invalid advisory close chronology");
            let task=p.task;
            append(&mut p,EventKind::TaskClosed{task,reason:format!("advisory_only:{reason}")},Source::Operator,&at);
            p.resource_accounting=Some(serde_json::to_value(resources)?);
            p.reviewed_accepted=None;p.review=None;p.label=None;p.closed=true;
            p.eligibility="advisory-only closed; neither accepted nor rejected; no outcome label or maturity eligibility".into();
            p.native_feature_compatible=false;
            p.native_ineligibility="advisory-only closure has no reviewed execution outcome".into();
            self.lifecycle_record(&Lifecycle::Closed{projection:Box::new(p.clone())})?;Ok(p)
        })
    }
    pub fn project_lifecycle_close(&self) -> Result<Projection> {
        self.write(|| {
            let objective=self.project_objective_context()?; ensure!(objective.is_null()||objective["context"]["next"]=="stop","finite objective remains open; explicit objective closure required");
            let mut p=self.project_lifecycle_view()?;
            if p.closed {return Ok(p)} // successful close persists atomically; retries don't duplicate
            ensure!(self.adaptive_state_inner()?.pending.is_none() && self.project_information_pending()?.is_none() && p.incomplete_executions==0,"unresolved execution prevents reviewed closure");
            let resources=self.project_budget()?;ensure!(resources.unsettled_requests==0 && resources.unresolved_tool_requests==0,"unresolved generation/preparation reservation prevents closure");
            let status=self.project_acceptance_status()?;
            ensure!(matches!(status.status.as_str(),"accepted"|"rejected"),"current independent accepted/rejected review required");
            let review=status.decision_record.ok_or_else(||anyhow::anyhow!("review required"))?;
            let events=self.events()?;
            let original=events.iter().find(|(_,e)|e.id==review).ok_or_else(||anyhow::anyhow!("missing review event"))?;
            ensure!(original.1.source==Source::Operator,"independent review provenance required");
            let review_at=timestamp(&original.1.recorded_at)?;let at=now()?;
            let last=p.events.last().unwrap().1.recorded_at.clone();
            ensure!(tsr_learn::maturity::timestamp(&at)? >= tsr_learn::maturity::timestamp(&last)? && tsr_learn::maturity::timestamp(&review_at)?<=tsr_learn::maturity::timestamp(&at)?,"invalid close chronology");
            // No acceptance from a review predating admission, or a legacy pre-admission receipt.
            let open=self.lifecycle_records()?.into_iter().find_map(|r|match r{Lifecycle::Open{open}=>Some(open),_=>None}).unwrap();
            ensure!(!open.prefix.contains(&review) && status.test_receipt.is_some_and(|r|!open.prefix.contains(&r)),"review and test receipt must belong to admitted episode");
            ensure!(p.mapping.iter().any(|(_,id,_)|p.events.iter().any(|(_,e)|e.id==*id && matches!(e.kind,EventKind::Executed{..}))),"observed execution required");
            if (!resources.unlimited_mode && p.consumed_tool_ms>p.admission.tool_ms_ceiling) || resources.exceeded {append(&mut p,EventKind::Stop{reason:"budget".into()},Source::Rules,&at);}
            let task=p.task;append(&mut p,EventKind::TaskClosed{task,reason:"reviewed".into()},Source::Operator,&at);
            // Reviewed acceptance is independent evidence, but not a common-event label:
            // unmeasured token/call/wallclock/patch budgets must not become satisfied.
            if resources.active {p.resource_accounting=Some(serde_json::to_value(&resources)?);}
            p.reviewed_accepted=Some(status.accepted);
            p.label=None;
            p.review=Some(review);p.source_snapshot=Some(status.snapshot);p.closed=true;
            p.eligibility="closed; not mature until close plus seven days; tool-time-only accounting; independent-task eligibility unestablished".into();
            self.lifecycle_record(&Lifecycle::Closed{projection:Box::new(p.clone())})?;Ok(p)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct Fake;
    impl tsr_corpus::Runner for Fake {
        fn run(
            &self,
            _: &std::path::Path,
            argv: &[String],
            _: &tsr_corpus::Limits,
        ) -> tsr_corpus::RunResult {
            match argv[0].as_str() {
                "true" | "rustc" => tsr_corpus::RunResult::from_output(0, b"", b"", 1),
                "./fixture-probe" => tsr_corpus::RunResult::from_output(0, b"synthetic\n", b"", 2),
                _ => panic!("no actual command execution permitted"),
            }
        }
    }
    fn verification_fixture() -> Result<(tempfile::TempDir, Engine)> {
        let d = tempfile::tempdir()?;
        let e = Engine::open(d.path())?;
        let spec: ProjectSpec = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../examples/engine-invoice/project.json"
        )))?;
        e.project_initialize(spec)?;
        e.project_lifecycle_open(Admission {
            training: None,
            state_root: e.store.head_root()?,
            tool_ms_ceiling: 200_000,
            reason: "Synthetic closure contract".into(),
            non_tool: Some(super::super::budget::Limits {
                model_calls: 1,
                model_tokens: 100,
                wall_ms: 300_000,
                patch_bytes: 0,
                external_cost_microusd: 0,
            }),
        })?;
        Ok((d, e))
    }
    fn verify(e: &Engine) -> Result<()> {
        let snapshot = e.project_acceptance_status()?.snapshot;
        let offer = e.project_admit_verification(e.store.head_root()?, snapshot)?;
        e.evaluate_proposal(serde_json::from_value(offer)?, true)?;
        let plan = e.coding_plan()?;
        e.coding_execute(plan.commitment, plan.source_before, &Fake)?;
        Ok(())
    }
    #[test]
    fn resource_extension_refuses_closed_and_pending_and_keeps_history() -> Result<()> {
        let (_d, e) = verification_fixture()?;
        let before = e.project_lifecycle_view()?;
        let request = ResourceExtension {
            state_root: e.store.head_root()?,
            previous: before.admission.clone(),
            limits: budget::Limits {
                model_calls: 3,
                model_tokens: 300,
                wall_ms: 600_000,
                patch_bytes: 1000,
                external_cost_microusd: 0,
            },
            tool_ms_ceiling: 300_000,
            reason: "Authored continuation; no actual effects".into(),
        };
        e.project_resource_extend(request.clone())?;
        let records = e.lifecycle_records()?.len();
        e.project_resource_extend(request.clone())?;
        assert_eq!(records, e.lifecycle_records()?.len());
        assert_eq!(before.task, e.project_lifecycle_view()?.task);
        assert_eq!(before.mapping, e.project_lifecycle_view()?.mapping);
        e.project_lifecycle_close_advisory("Authored no-execution close".into())?;
        assert!(e.project_resource_extend(request).is_err());
        let (_d, e) = verification_fixture()?;
        let previous = e.project_lifecycle_view()?.admission;
        let offer = e.project_admit_verification(
            e.store.head_root()?,
            e.project_acceptance_status()?.snapshot,
        )?;
        e.evaluate_proposal(serde_json::from_value(offer)?, true)?;
        assert!(e
            .project_resource_extend(ResourceExtension {
                state_root: e.store.head_root()?,
                previous,
                limits: budget::Limits {
                    model_calls: 3,
                    model_tokens: 300,
                    wall_ms: 600_000,
                    patch_bytes: 1000,
                    external_cost_microusd: 0
                },
                tool_ms_ceiling: 300_000,
                reason: "Authored pending rejection".into()
            })
            .is_err());
        Ok(())
    }
    #[test]
    fn inconclusive_configuration_serialization_matches_existing_scope_contract() -> Result<()> {
        let (_d, e) = verification_fixture()?;
        let mut spec = e.project_configuration()?;
        for mode in 0..2 {
            if mode == 1 {
                spec.cargo = true;
                spec.named_package = Some("pkg".into());
                spec.full_workspace_oracle = true;
                spec.edit_windows
                    .insert("src/pricing.rs".into(), EditWindow { start: 0, end: 10 });
                spec.supporting_spans.insert(
                    "src/pricing.rs".into(),
                    vec![EditWindow { start: 10, end: 20 }],
                );
            }
            assert_eq!(
                Hash::of(&[&serde_json::to_vec(&spec)?]),
                tsr_learn::project_eligibility::verification_configuration_digest(
                    &serde_json::to_value(&spec)?
                )?
            );
        }
        Ok(())
    }
    #[test]
    fn inconclusive_verification_preserves_costs_unknowns_and_restart_without_label() -> Result<()>
    {
        let (d, e) = verification_fixture()?;
        verify(&e)?;
        // A settled unknown model outcome remains charged; closure is not a refund.
        e.project_resource_reserve(super::super::budget::Reserve {
            request_id: "unknown".into(),
            state_root: e.store.head_root()?,
            envelope: "fixture".into(),
            model: "fixture".into(),
            local: true,
            tokens_upper: 50,
            external_cost_upper: 0,
        })?;
        assert!(e
            .project_lifecycle_close_inconclusive("pending accounting".into())
            .is_err());
        e.project_resource_settle(super::super::budget::Settle {
            request_id: "unknown".into(),
            prompt_tokens: None,
            output_tokens: None,
            duration_ns: None,
            external_cost_microusd: Some(0),
            outcome: "unknown".into(),
        })?;
        let root = e.store.head_root()?;
        let before = e.project_budget()?;
        let p = e.project_lifecycle_close_inconclusive(
            "finite observations, no acceptance review".into(),
        )?;
        assert_eq!(e.store.head_root()?, root);
        assert!(
            p.closed && p.label.is_none() && p.review.is_none() && p.reviewed_accepted.is_none()
        );
        assert_eq!(e.project_budget()?.tool_ms_charged, before.tool_ms_charged);
        assert_eq!(e.project_budget()?.tokens_charged, 50);
        assert_eq!(e.project_budget()?.unknown_usage, 1);
        assert!(e.project_budget()?.stopped);
        assert!(e
            .project_lifecycle_close_advisory("cannot reclassify".into())
            .is_err());
        let e = Engine::open(d.path())?;
        assert_eq!(
            serde_json::to_value(&p)?,
            serde_json::to_value(e.project_lifecycle_close_inconclusive("retry".into())?)?
        );
        assert_eq!(
            e.project_eligibility_view()?["eligibility"]["status"],
            "verification_inconclusive_no_acceptance_label"
        );
        assert!(e
            .project_admit_verification(e.store.head_root()?, p.opening_snapshot)
            .is_err());
        Ok(())
    }
    #[test]
    fn inconclusive_verification_refuses_empty_pending_and_source_drift() -> Result<()> {
        let (d, e) = verification_fixture()?;
        assert!(e
            .project_lifecycle_close_inconclusive("no execution".into())
            .is_err());
        let snapshot = e.project_acceptance_status()?.snapshot;
        let offer = e.project_admit_verification(e.store.head_root()?, snapshot)?;
        e.evaluate_proposal(serde_json::from_value(offer)?, true)?;
        assert!(e
            .project_lifecycle_close_inconclusive("pending".into())
            .is_err());
        let plan = e.coding_plan()?;
        e.coding_execute(plan.commitment, plan.source_before, &Fake)?;
        std::fs::write(d.path().join("coding-work/src/pricing.rs"), "changed")?;
        assert!(e
            .project_lifecycle_close_inconclusive("changed source".into())
            .is_err());
        assert!(!e.project_lifecycle_view()?.closed);
        Ok(())
    }
    #[test]
    fn timestamp_conversion_preserves_instant_and_rejects_invalid_input() {
        for (input, output) in [
            ("unix-ms:0", "1970-01-01T00:00:00.000Z"),
            ("unix-ms:1709251200123", "2024-03-01T00:00:00.123Z"),
        ] {
            assert_eq!(timestamp(input).unwrap(), output);
        }
        assert!(timestamp("unix-ms:253402300800000").is_err());
        assert!(timestamp("unix-ms:-1").is_err());
        assert!(timestamp("2026-02-30T00:00:00Z").is_err());
        assert_eq!(
            tsr_learn::maturity::timestamp(&timestamp("2026-01-01T01:00:00+01:00").unwrap())
                .unwrap(),
            tsr_learn::maturity::timestamp("2026-01-01T00:00:00Z").unwrap()
        );
    }
}
