//! Prospective, operator-attested project eligibility. Never derives labels from tests alone.
use crate::{
    decision::{DiffLabel, LabelRecord},
    project_receipts::{Artifact, Evidence},
    LearnError,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use tsr_domain::{EventKind, Hash, Observation, Source, TaskId};
pub fn closure_digest(value: &serde_json::Value) -> Result<Hash, LearnError> {
    Ok(Hash::of(&[&serde_json::to_vec(value)?]))
}
pub const FORECAST_PROJECTION: &str = "project-common-event-lifecycle@v2";
pub const SOURCE: &str = "project-eligibility-review@v1";
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceLimits {
    pub model_calls: u64,
    pub model_tokens: u64,
    pub wall_ms: u64,
    pub patch_bytes: u64,
    pub external_cost_microusd: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LifecycleAdmission {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub training: Option<Admission>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub non_tool: Option<ResourceLimits>,
    pub state_root: Hash,
    pub tool_ms_ceiling: u64,
    pub reason: String,
}
#[derive(Serialize, Deserialize)]
struct Open {
    admission: LifecycleAdmission,
    prefix: Vec<tsr_domain::EventId>,
    snapshot: Hash,
    at: String,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Origin {
    Operational,
    Synthetic,
    Replay,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Admission {
    pub origin: Origin,
    pub original_task: Hash,
    pub family: String,
    pub lineage: String,
    pub clone_group: String,
    pub reviewer: String,
    pub no_inference: bool,
}
impl Admission {
    pub fn validate(&self) -> Result<(), LearnError> {
        for s in [
            &self.family,
            &self.lineage,
            &self.clone_group,
            &self.reviewer,
        ] {
            if s.trim().is_empty() || s.len() > 128 {
                return Err(invalid(
                    "bounded original identity/reviewer metadata required",
                ));
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub closure: Hash,
    pub reviewer: String,
    pub independence: String,
    pub complete_instrumented_coverage: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Proof {
    pub task: TaskId,
    pub evidence: Evidence,
}
#[derive(Debug, Serialize)]
pub struct Eligibility {
    pub task: TaskId,
    pub status: String,
    pub label: Option<LabelRecord>,
    pub original_task: Option<Hash>,
}
fn invalid(s: &str) -> LearnError {
    LearnError::Invalid(s.into())
}
fn u(v: &serde_json::Value, k: &str) -> Result<u64, LearnError> {
    v[k].as_u64()
        .ok_or_else(|| invalid("missing resource measurement"))
}
fn plus(a: &mut u64, b: u64) -> Result<(), LearnError> {
    *a = a
        .checked_add(b)
        .ok_or_else(|| invalid("resource measurement overflow"))?;
    Ok(())
}
/// Compatibility serialization for existing ProjectSpec verification hashes.
/// Field order and omission rules are pinned by a cross-crate equivalence test.
pub fn verification_configuration_digest(value: &serde_json::Value) -> Result<Hash, LearnError> {
    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Window {
        start: usize,
        end: usize,
    }
    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Case {
        id: String,
        args: Vec<String>,
        expected_stdout: String,
    }
    fn is_false(v: &bool) -> bool {
        !*v
    }
    #[derive(Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Scope {
        #[serde(default, skip_serializing_if = "is_false")]
        full_workspace_oracle: bool,
        goal: String,
        image: String,
        user: String,
        editable: BTreeMap<String, String>,
        protected: BTreeMap<String, String>,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        edit_windows: BTreeMap<String, Window>,
        #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
        supporting_spans: BTreeMap<String, Vec<Window>>,
        #[serde(default)]
        cargo: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        named_package: Option<String>,
        #[serde(default)]
        probe_entry: String,
        cases: Vec<Case>,
        timeout_ms: u64,
    }
    let scope: Scope = serde_json::from_value(value.clone())?;
    Ok(Hash::of(&[&serde_json::to_vec(&scope)?]))
}
fn verify_inconclusive_execution(
    raw: &Evidence,
    prefix: usize,
    closed: &serde_json::Value,
) -> Result<(), LearnError> {
    use crate::project_receipts::blob;
    let mut records = vec![];
    for (index, (_, e)) in raw.events.iter().enumerate() {
        if let EventKind::Observed {
            observation: Observation::Evidence { source, digest },
        } = &e.kind
        {
            if source == "configured-cycle@v1" {
                records.push((index, e, blob(raw, *digest)?));
            }
        }
    }
    let configuration = records
        .iter()
        .find(|(_, _, v)| v["fact"] == "coding" && v["record"]["phase"] == "config")
        .ok_or_else(|| invalid("missing original project configuration"))?;
    if configuration.1.source != Source::Operator || configuration.0 >= prefix {
        return Err(invalid(
            "project configuration must precede prospective admission",
        ));
    }
    let configuration_hash =
        verification_configuration_digest(&configuration.2["record"]["config"]["project"])?;
    let snapshot = &closed["opening_snapshot"];
    if closed["source_snapshot"] != *snapshot
        || !closed["label"].is_null()
        || !closed["review"].is_null()
        || !closed["reviewed_accepted"].is_null()
    {
        return Err(invalid(
            "inconclusive closure cannot assert source change/review/label",
        ));
    }
    let totals = &closed["resource_accounting"];
    if totals["active"] != true
        || u(totals, "patch_bytes_charged")? != 0
        || u(totals, "unsettled_requests")? != 0
        || u(totals, "unresolved_tool_requests")? != 0
    {
        return Err(invalid(
            "inconclusive closure requires resolved zero-edit accounting",
        ));
    }
    let mut count = 0;
    for (index, (_, event)) in raw.events.iter().enumerate().skip(prefix) {
        if let EventKind::Committed { candidate, .. } = event.kind {
            let commitment = serde_json::to_value(event.id)?;
            if event.source != Source::Rules || !raw.events[..index].iter().any(|(_,e)| matches!(&e.kind,EventKind::Proposed{candidate:c} if c.id==candidate && c.kind==tsr_domain::ActionKind::RunApprovedTest)) {
                return Err(invalid("inconclusive closure requires verification candidates"));
            }
            let pending = records
                .iter()
                .find(|(_, e, v)| {
                    e.source == Source::Rules
                        && v["fact"] == "pending"
                        && v["commitment"] == commitment
                })
                .ok_or_else(|| invalid("missing verification pending binding"))?;
            let verification = records
                .iter()
                .find(|(i, e, v)| {
                    *i >= prefix
                        && *i < index
                        && e.source == Source::Operator
                        && v["fact"] == "coding"
                        && v["record"]["phase"] == "verification"
                        && v["record"]["action"]["id"] == pending.2["action"]
                        && v["record"]["snapshot"] == *snapshot
                })
                .ok_or_else(|| invalid("missing source-bound verification admission"))?;
            let intent = records
                .iter()
                .find(|(i, e, v)| {
                    *i > index
                        && e.source == Source::Rules
                        && v["fact"] == "coding"
                        && v["record"]["phase"] == "intent"
                        && v["record"]["plan"]["commitment"] == commitment
                })
                .ok_or_else(|| invalid("missing verification intent"))?;
            let plan = &intent.2["record"]["plan"];
            if plan["edits"] != serde_json::json!([])
                || plan["source_before"] != *snapshot
                || plan["expected_after"] != *snapshot
                || verification.2["record"]["configuration"]
                    != serde_json::to_value(configuration_hash)?
            {
                return Err(invalid("inconclusive verification changed source"));
            }
            if !records.iter().any(|(i, e, v)| {
                *i > intent.0
                    && e.source == Source::Rules
                    && v["fact"] == "coding"
                    && v["record"]["phase"] == "receipt"
                    && v["record"]["commitment"] == commitment
                    && v["record"]["source_after"] == *snapshot
            }) {
                return Err(invalid("missing source-bound verification return"));
            }
            count += 1;
        }
    }
    if count == 0 {
        return Err(invalid("inconclusive verification requires execution"));
    }
    Ok(())
}
fn verify_projection(
    open: &serde_json::Value,
    closed: &serde_json::Value,
    raw: &Evidence,
    closed_record_at: &str,
) -> Result<(), LearnError> {
    use crate::project_receipts::{clock, rfc3339};
    if !matches!(
        closed["version"].as_str(),
        Some("project-common-event-lifecycle@v1") | Some(FORECAST_PROJECTION)
    ) {
        return Err(invalid("unsupported lifecycle projection version"));
    }
    let open: Open = serde_json::from_value(open.clone())?;
    let task = TaskId(Hash::of(&[
        b"project-common-event-lifecycle@v1",
        &serde_json::to_vec(&open)?,
    ]));
    if closed["task"] != serde_json::to_value(task)? {
        return Err(invalid("episode identity differs from admission"));
    }
    let mut projected = vec![];
    let append = |events: &mut Vec<(i64, tsr_domain::Event)>, kind, source, at: &str| {
        let parents = events.last().map(|(_, e)| vec![e.id]).unwrap_or_default();
        let e = tsr_domain::Event::new(task, parents, source, kind, at);
        let id = e.id;
        events.push((events.len() as i64 + 1, e));
        id
    };
    append(
        &mut projected,
        EventKind::TaskOpened { task },
        Source::Operator,
        &open.at,
    );
    if raw.events.len() < open.prefix.len()
        || raw
            .events
            .iter()
            .take(open.prefix.len())
            .map(|(_, e)| e.id)
            .ne(open.prefix.iter().copied())
    {
        return Err(invalid("admission prefix mismatch"));
    }
    let mut mapping = vec![];
    let mut proposed = BTreeSet::new();
    let mut committed = BTreeSet::new();
    let mut executed = BTreeSet::new();
    let mut tool = 0;
    let mut previous = clock(&open.at)?;
    for (_, e) in raw.events.iter().skip(open.prefix.len()) {
        let at = rfc3339(&e.recorded_at)?;
        let t = clock(&at)?;
        if t < previous {
            return Err(invalid("lifecycle chronology mismatch"));
        }
        previous = t;
        let include = match &e.kind {
            EventKind::Proposed { candidate } => {
                proposed.insert(candidate.id);
                true
            }
            EventKind::Forecast {
                forecast,
                candidate,
            } if closed["version"] == FORECAST_PROJECTION
                && forecast.event == crate::decision_event_id() =>
            {
                if !proposed.contains(candidate)
                    || committed.contains(candidate)
                    || !matches!(
                        (e.source, forecast.source.clone()),
                        (Source::Generator, tsr_domain::Forecaster::Generator)
                            | (Source::Rules, tsr_domain::Forecaster::Controller)
                    )
                {
                    return Err(invalid(
                        "invalid common-event forecast provenance/chronology",
                    ));
                }
                true
            }
            EventKind::Committed {
                candidate,
                decision: tsr_domain::Decision::Selected { value, .. },
            } => {
                if candidate != value
                    || !proposed.contains(candidate)
                    || !committed.insert(*candidate)
                {
                    return Err(invalid("invalid lifecycle commitment"));
                }
                true
            }
            EventKind::Executed {
                candidate,
                duration_ms,
                ..
            } => {
                if e.source != Source::Rules
                    || !committed.contains(candidate)
                    || !executed.insert(*candidate)
                {
                    return Err(invalid("invalid lifecycle execution"));
                }
                plus(&mut tool, *duration_ms)?;
                true
            }
            _ => false,
        };
        if include {
            let id = append(&mut projected, e.kind.clone(), e.source, &at);
            mapping.push((e.id, id, e.recorded_at.clone()));
        }
    }
    if closed["closed"] != true || committed != executed {
        return Err(invalid("unresolved commitment prevents eligible closure"));
    }
    let stored: Vec<(i64, tsr_domain::Event)> = serde_json::from_value(closed["events"].clone())?;
    let terminal = stored.last().ok_or_else(|| invalid("missing closure"))?;
    let t = clock(&terminal.1.recorded_at)?;
    if t < previous || t > clock(closed_record_at)? {
        return Err(invalid("derived closure chronology mismatch"));
    }
    if stored
        .iter()
        .skip(projected.len())
        .any(|(_, e)| matches!(&e.kind,EventKind::Stop{reason} if reason=="budget"))
    {
        append(
            &mut projected,
            EventKind::Stop {
                reason: "budget".into(),
            },
            Source::Rules,
            &terminal.1.recorded_at,
        );
    }
    let close_reason = match &terminal.1.kind {
        EventKind::TaskClosed { reason, .. } if reason == "reviewed" => reason.clone(),
        EventKind::TaskClosed { reason, .. } if reason.starts_with("advisory_only:") => {
            if !executed.is_empty()
                || !closed["label"].is_null()
                || !closed["review"].is_null()
                || !closed["reviewed_accepted"].is_null()
            {
                return Err(invalid(
                    "advisory closure cannot assert execution/review/label",
                ));
            }
            reason.clone()
        }
        EventKind::TaskClosed { reason, .. }
            if reason.starts_with("verification_inconclusive@v1:") =>
        {
            verify_inconclusive_execution(raw, open.prefix.len(), closed)?;
            reason.clone()
        }
        _ => return Err(invalid("unknown closure semantics")),
    };
    append(
        &mut projected,
        EventKind::TaskClosed {
            task,
            reason: close_reason,
        },
        Source::Operator,
        &terminal.1.recorded_at,
    );
    if serde_json::to_value(&mapping)? != closed["mapping"]
        || stored != projected
        || u(closed, "consumed_tool_ms")? != tool
        || u(closed, "incomplete_executions")? != committed.difference(&executed).count() as u64
        || closed["admission"] != serde_json::to_value(&open.admission)?
        || closed["opening_snapshot"] != serde_json::to_value(open.snapshot)?
    {
        return Err(invalid("derived lifecycle differs from original evidence"));
    }
    Ok(())
}
/// Reconstruct against raw CAS and side events. Metadata assertions establish declared
/// provenance, not cryptographic authentication of an independent human reviewer.
pub fn evaluate(a: &Artifact, p: &Proof, as_of: &str) -> Result<Eligibility, LearnError> {
    use crate::project_receipts::{blob, check_events, clock};
    a.validate()?;
    check_events(&p.evidence.events)?;
    check_events(&p.evidence.side_events)?;
    let cutoff = crate::maturity::timestamp(as_of)?;
    let withheld = |reason: &str, original_task| Eligibility {
        task: p.task,
        status: reason.into(),
        label: None,
        original_task,
    };
    let projection = &a.export_time_lifecycle_inventory;
    if projection["task"] != serde_json::to_value(p.task)? {
        return Err(invalid("eligibility task mismatch"));
    }
    let mut open = None;
    let mut closed = None;
    let mut closed_record_at = None;
    let mut attestation = None;
    let mut resources = vec![];
    for (_, e) in &p.evidence.side_events {
        if clock(&e.recorded_at)? > cutoff {
            continue;
        }
        if let EventKind::Observed {
            observation: Observation::Evidence { source, digest },
        } = &e.kind
        {
            if !matches!(
                source.as_str(),
                "project-common-event-lifecycle@v1" | "project-resource-accounting@v1" | SOURCE
            ) {
                continue;
            }
            let v = blob(&p.evidence, *digest)?;
            match source.as_str() {
                "project-common-event-lifecycle@v1" => {
                    if e.source != Source::Operator {
                        return Err(invalid("lifecycle admission source"));
                    }
                    match v["phase"].as_str() {
                        Some("open") => {
                            if open.replace(v["open"].clone()).is_some() {
                                return Err(invalid("repeated admission"));
                            }
                        }
                        Some("closed") => {
                            closed_record_at = Some(e.recorded_at.clone());
                            if closed.replace(v["projection"].clone()).is_some() {
                                return Err(invalid("repeated closure"));
                            }
                        }
                        _ => return Err(invalid("invalid lifecycle record")),
                    }
                }
                SOURCE => {
                    if e.source != Source::Operator {
                        return Err(invalid("eligibility review source"));
                    }
                    if attestation
                        .replace((serde_json::from_value::<Review>(v)?, e.recorded_at.clone()))
                        .is_some()
                    {
                        return Err(invalid("ambiguous eligibility reviews"));
                    }
                }
                _ => {
                    if e.source != Source::Rules || v["episode"] != serde_json::to_value(p.task)? {
                        return Err(invalid("resource source/episode mismatch"));
                    }
                    resources.push(v);
                }
            }
        }
    }
    let Some(open) = open else {
        return Ok(withheld("missing_prospective_admission", None));
    };
    if let Some(c) = &closed {
        let events: Vec<(i64, tsr_domain::Event)> = serde_json::from_value(c["events"].clone())?;
        if events.last().is_some_and(|(_, e)| matches!(&e.kind, EventKind::TaskClosed { reason, .. } if reason.starts_with("verification_inconclusive@v1:"))) {
            if c != projection { return Err(invalid("inconclusive projection changed")); }
            verify_projection(&open, c, &p.evidence, closed_record_at.as_deref().ok_or_else(|| invalid("missing inconclusive closure time"))?)?;
            return Ok(withheld("verification_inconclusive_no_acceptance_label", None));
        }
        if events.last().is_some_and(|(_,e)|matches!(&e.kind,EventKind::TaskClosed{reason,..} if reason.starts_with("advisory_only:"))) {
            if c!=projection {return Err(invalid("advisory projection changed"));}
            verify_projection(&open,c,&p.evidence,closed_record_at.as_deref().ok_or_else(||invalid("missing advisory closure time"))?)?;
            return Ok(withheld("advisory_only_no_execution_label",None));
        }
    }
    let Some(metadata) = open["admission"].get("training").filter(|v| !v.is_null()) else {
        return Ok(withheld("legacy_origin_unknown", None));
    };
    let admission: Admission = serde_json::from_value(metadata.clone())?;
    admission.validate()?;
    let identity = Some(admission.original_task);
    if a.rows
        .iter()
        .any(|r| r.family != admission.family || r.lineage != admission.lineage)
    {
        return Err(invalid(
            "prediction metadata contradicts prospective admission",
        ));
    }
    let Some(closed) = closed else {
        return Ok(withheld("not_closed_at_cutoff", identity));
    };
    if &closed != projection {
        return Err(invalid("closed projection changed"));
    }
    verify_projection(
        &open,
        &closed,
        &p.evidence,
        closed_record_at
            .as_deref()
            .ok_or_else(|| invalid("missing closure receipt time"))?,
    )?;
    for row in &a.rows {
        let n = row.evidence.events.len();
        if n > p.evidence.events.len() || row.evidence.events != p.evidence.events[..n] {
            return Err(invalid("receipt is not a prefix of its episode"));
        }
        // Shadow receipts include unchosen alternatives. They remain evidence,
        // not executed training examples or negative outcomes.
        if let Some(original) = p.evidence.events.iter().position(|(_, e)| {
            matches!(e.kind, EventKind::Committed { candidate, .. } if candidate == row.candidate)
        }) {
            if n > original
                || clock(&row.prediction_event.recorded_at)?
                    > clock(&p.evidence.events[original].1.recorded_at)?
                || clock(&row.evidence.as_of)?
                    > clock(&p.evidence.events[original].1.recorded_at)?
            {
                return Err(invalid("receipt is not a pre-action prefix of its episode"));
            }
        }
    }

    let Some((review, at)) = attestation else {
        return Ok(withheld("missing_independent_coverage_review", identity));
    };
    if review.closure != Hash::of(&[&serde_json::to_vec(&closed)?])
        || review.reviewer != admission.reviewer
        || review.independence.trim().is_empty()
        || review.independence.len() > 512
        || !review.complete_instrumented_coverage
    {
        return Err(invalid("eligibility review binding/coverage invalid"));
    }
    let events: Vec<(i64, tsr_domain::Event)> = serde_json::from_value(closed["events"].clone())?;
    let close = events
        .last()
        .filter(|(_, e)| matches!(e.kind, EventKind::TaskClosed { .. }))
        .ok_or_else(|| invalid("missing terminal close"))?;
    let close_at = clock(&close.1.recorded_at)?;
    let review_at = clock(&at)?;
    if review_at < close_at {
        return Err(invalid("coverage review predates closure"));
    }
    if review_at > close_at + 604_800_000_000_000 {
        return Ok(withheld("review_outside_fixed_horizon", identity));
    }
    let open_at = clock(
        open["at"]
            .as_str()
            .ok_or_else(|| invalid("missing admission time"))?,
    )?;
    let prefix: Vec<tsr_domain::EventId> = serde_json::from_value(open["prefix"].clone())?;
    if p.evidence.events.len() < prefix.len()
        || p.evidence
            .events
            .iter()
            .take(prefix.len())
            .map(|(_, e)| e.id)
            .ne(prefix.iter().copied())
    {
        return Err(invalid("prospective main prefix mismatch"));
    }
    if Hash::of(&[&serde_json::to_vec(&p.evidence.events)?])
        != serde_json::from_value::<Hash>(closed["factual_prefix"].clone())?
    {
        return Err(invalid("closed factual evidence mismatch"));
    }
    let totals = &closed["resource_accounting"];
    if totals["active"] != true || totals["episode"] != serde_json::to_value(p.task)? {
        return Ok(withheld("no_prospective_resource_coverage", identity));
    }
    for k in [
        "unknown_usage",
        "unknown_compute_requests",
        "unsettled_requests",
    ] {
        if u(totals, k)? != 0 {
            return Ok(withheld("incomplete_resource_measurement", identity));
        }
    }
    let mut reserved = BTreeMap::new();
    let mut settled = BTreeMap::new();
    let mut executions = BTreeMap::new();
    let mut auxiliary = BTreeMap::new();
    let mut last = open_at;
    for v in resources {
        let t = clock(
            v["at"]
                .as_str()
                .ok_or_else(|| invalid("resource timestamp"))?,
        )?;
        if t < last || t > close_at {
            return Err(invalid("resource chronology outside episode"));
        }
        last = t;
        let r = &v["record"];
        match r["kind"].as_str() {
            Some("reserve") => {
                let req = &r["request"];
                let id = req["request_id"]
                    .as_str()
                    .ok_or_else(|| invalid("request identity"))?;
                if reserved.insert(id.to_string(), req.clone()).is_some() {
                    return Err(invalid("duplicate resource request"));
                }
            }
            Some("settle") => {
                let usage = &r["usage"];
                let id = usage["request_id"]
                    .as_str()
                    .ok_or_else(|| invalid("settlement identity"))?;
                if !reserved.contains_key(id)
                    || settled.insert(id.to_string(), usage.clone()).is_some()
                {
                    return Err(invalid("unbound resource settlement"));
                }
            }
            Some("auxiliary") => {
                let id = r["request_id"]
                    .as_str()
                    .ok_or_else(|| invalid("auxiliary identity missing"))?;
                if auxiliary.insert(id.to_string(), (r.clone(), t)).is_some() {
                    return Err(invalid("duplicate auxiliary reservation"));
                }
            }
            Some("execution") => {
                let id: tsr_domain::EventId = serde_json::from_value(r["commitment"].clone())?;
                if executions.insert(id, r.clone()).is_some() {
                    return Err(invalid("duplicate execution reservation"));
                }
            }
            _ => return Err(invalid("unknown resource record")),
        }
    }
    let mut prompt = 0;
    let mut output = 0;
    let mut cost = 0;
    let mut per_request_breach = false;
    for (id, request) in &reserved {
        let Some(s) = settled.get(id) else {
            return Ok(withheld("unresolved_model_request", identity));
        };
        if s["outcome"] != "measured" {
            return Ok(withheld("unknown_model_usage", identity));
        }
        plus(&mut prompt, u(s, "prompt_tokens")?)?;
        plus(&mut output, u(s, "output_tokens")?)?;
        plus(&mut cost, u(s, "external_cost_microusd")?)?;
        u(s, "duration_ns")?;
        let tokens = u(s, "prompt_tokens")?
            .checked_add(u(s, "output_tokens")?)
            .ok_or_else(|| invalid("token overflow"))?;
        per_request_breach |= tokens > u(request, "tokens_upper")?
            || u(s, "external_cost_microusd")? > u(request, "external_cost_upper")?;
        if request["local"] == true && u(s, "external_cost_microusd")? != 0 {
            return Err(invalid("local external charge mismatch"));
        }
    }
    if admission.no_inference && !reserved.is_empty() {
        return Err(invalid("prospective no-inference declaration contradicted"));
    }
    if !admission.no_inference && reserved.is_empty() {
        return Ok(withheld("model_measurement_absent", identity));
    }
    let mut plans = BTreeMap::new();
    for (_, e) in p.evidence.events.iter().skip(prefix.len()) {
        if let EventKind::Observed {
            observation: Observation::Evidence { source, digest },
        } = &e.kind
        {
            if source == "configured-cycle@v1" {
                let v = blob(&p.evidence, *digest)?;
                if v["fact"] == "coding" && v["record"]["phase"] == "dependencies" {
                    let request = v["record"]["resource_request"].as_str();
                    if request.is_none_or(|id| !auxiliary.contains_key(id)) {
                        return Ok(withheld("uninstrumented_preparation", identity));
                    }
                }
                if v["fact"] == "coding" && v["record"]["phase"] == "intent" {
                    if e.source != Source::Rules {
                        return Err(invalid("intent provenance"));
                    }
                    let plan = &v["record"]["plan"];
                    let id: tsr_domain::EventId =
                        serde_json::from_value(plan["commitment"].clone())?;
                    if plans.insert(id, plan.clone()).is_some() {
                        return Err(invalid("duplicate execution intent"));
                    }
                }
            }
        }
    }
    let mut measured_tool = 0;
    for (request, (reservation, reserved_at)) in &auxiliary {
        let mut found = None;
        for (_, event) in p.evidence.events.iter().skip(prefix.len()) {
            if let EventKind::Observed {
                observation: Observation::Evidence { source, digest },
            } = &event.kind
            {
                if source != "configured-cycle@v1" {
                    continue;
                }
                let value = blob(&p.evidence, *digest)?;
                let r = &value["record"];
                if value["fact"] == "coding"
                    && r["phase"] == "dependencies"
                    && r["resource_request"].as_str() == Some(request)
                {
                    if event.source != Source::Rules
                        || clock(&event.recorded_at)? < *reserved_at
                        || r["binding"] != reservation["binding"]
                        || found.is_some()
                    {
                        return Err(invalid("auxiliary observation binding/chronology"));
                    }
                    found = Some(u(r, "measured_tool_ms")?);
                }
            }
        }
        let Some(ms) = found else {
            return Ok(withheld("unresolved_preparation", identity));
        };
        per_request_breach |= ms > u(reservation, "tool_ms_upper")?;
        plus(&mut measured_tool, ms)?;
    }
    let mut patch_bytes = 0;
    let mut seen = BTreeSet::new();
    for (_, e) in p.evidence.events.iter().skip(prefix.len()) {
        if clock(&e.recorded_at)? < open_at {
            return Err(invalid("postadmission chronology"));
        }
        if let EventKind::Executed {
            candidate,
            duration_ms,
            ..
        } = e.kind
        {
            let commitment = p
                .evidence
                .events
                .iter()
                .find_map(|(_, e)| match e.kind {
                    EventKind::Committed { candidate: c, .. } if c == candidate => Some(e.id),
                    _ => None,
                })
                .ok_or_else(|| invalid("execution missing commitment"))?;
            let Some(r) = executions.get(&commitment) else {
                return Ok(withheld("uninstrumented_execution", identity));
            };
            if e.source != Source::Rules || !seen.insert(commitment) {
                return Err(invalid("execution provenance/duplicate"));
            }
            let plan = plans
                .get(&commitment)
                .ok_or_else(|| invalid("missing reserved execution plan"))?;
            let mut bytes = 0;
            for edit in plan["edits"]
                .as_array()
                .ok_or_else(|| invalid("missing measured patch edits"))?
            {
                plus(
                    &mut bytes,
                    edit["replacement"]
                        .as_str()
                        .ok_or_else(|| invalid("invalid patch edit"))?
                        .len() as u64,
                )?;
            }
            if bytes != u(r, "patch_bytes")? {
                return Err(invalid("patch byte reservation/intent mismatch"));
            }
            per_request_breach |= duration_ms > u(r, "tool_ms_upper")?;
            plus(&mut measured_tool, duration_ms)?;
            plus(&mut patch_bytes, bytes)?;
        }
    }
    if seen.len() != executions.len() || seen.is_empty() {
        return Ok(withheld("unresolved_execution", identity));
    }
    if u(totals, "model_calls")? != reserved.len() as u64
        || u(totals, "measured_prompt_tokens")? != prompt
        || u(totals, "measured_output_tokens")? != output
        || u(totals, "tokens_charged")?
            != prompt
                .checked_add(output)
                .ok_or_else(|| invalid("token overflow"))?
        || u(totals, "external_cost_charged_microusd")? != cost
        || u(totals, "patch_bytes_charged")? != patch_bytes
        || u(totals, "tool_ms_charged")? != measured_tool
    {
        return Err(invalid("resource snapshot/reconstructed usage mismatch"));
    }
    let limits = &open["admission"]["non_tool"];
    if &totals["limits"] != limits {
        return Err(invalid("resource limits differ from admission"));
    }
    let wall = ((close_at - open_at) / 1_000_000) as u64;
    if u(totals, "elapsed_wall_ms")? > wall {
        return Err(invalid("resource elapsed time exceeds closure"));
    }
    let breached = per_request_breach
        || reserved.len() as u64 > u(limits, "model_calls")?
        || prompt
            .checked_add(output)
            .ok_or_else(|| invalid("token overflow"))?
            > u(limits, "model_tokens")?
        || cost > u(limits, "external_cost_microusd")?
        || patch_bytes > u(limits, "patch_bytes")?
        || wall > u(limits, "wall_ms")?
        || measured_tool > u(&open["admission"], "tool_ms_ceiling")?;
    let stopped = events
        .iter()
        .any(|(_, e)| matches!(&e.kind,EventKind::Stop{reason} if reason=="budget"));
    if breached != stopped {
        return Ok(withheld("budget_closure_reconciliation_required", identity));
    }
    let review_id: tsr_domain::EventId = serde_json::from_value(closed["review"].clone())?;
    let event = p
        .evidence
        .events
        .iter()
        .find(|(_, e)| e.id == review_id)
        .ok_or_else(|| invalid("missing independent acceptance review"))?;
    if event.1.source != Source::Operator || prefix.contains(&review_id) {
        return Err(invalid("acceptance review not independent/prospective"));
    }
    let digest = match &event.1.kind {
        EventKind::Observed {
            observation: Observation::Evidence { digest, .. },
        } => *digest,
        _ => return Err(invalid("review event type")),
    };
    let record = blob(&p.evidence, digest)?;
    let actual = &record["record"]["review"];
    if record["record"]["phase"] != "acceptance_review"
        || actual["snapshot"] != closed["source_snapshot"]
    {
        return Err(invalid("review/source binding"));
    }
    let test: tsr_domain::EventId = serde_json::from_value(actual["test_receipt"].clone())?;
    let requirements: tsr_domain::EventId =
        serde_json::from_value(actual["requirements_revision"].clone())?;
    if prefix.contains(&test) {
        return Err(invalid("test receipt predates admission"));
    }
    let record_for = |id| -> Result<serde_json::Value, LearnError> {
        let (_, event) = p
            .evidence
            .events
            .iter()
            .find(|(_, e)| e.id == id)
            .ok_or_else(|| invalid("review evidence missing"))?;
        let digest = match &event.kind {
            EventKind::Observed {
                observation: Observation::Evidence { digest, .. },
            } => *digest,
            _ => return Err(invalid("review evidence type")),
        };
        blob(&p.evidence, digest)
    };
    let mut latest = BTreeMap::new();
    for (_, event) in &p.evidence.events {
        if let EventKind::Observed {
            observation: Observation::Evidence { source, digest },
        } = &event.kind
        {
            if source != "configured-cycle@v1" {
                continue;
            }
            let record = blob(&p.evidence, *digest)?;
            if record["fact"] != "coding" {
                continue;
            }
            if let Some(phase) = record["record"]["phase"].as_str() {
                if matches!(
                    phase,
                    "acceptance_review" | "acceptance_requirements" | "receipt"
                ) {
                    latest.insert(phase.to_owned(), (event.id, event.source));
                }
            }
        }
    }
    for (phase, id, source) in [
        ("acceptance_review", review_id, Source::Operator),
        ("acceptance_requirements", requirements, Source::Operator),
        ("receipt", test, Source::Rules),
    ] {
        if latest.get(phase) != Some(&(id, source)) {
            return Err(invalid("stale or wrong-source acceptance evidence"));
        }
    }
    let test_record = record_for(test)?;
    let test_commitment: tsr_domain::EventId =
        serde_json::from_value(test_record["record"]["commitment"].clone())?;
    if !seen.contains(&test_commitment) {
        return Err(invalid("test receipt is not bound to measured execution"));
    }
    if test_record["record"]["phase"] != "receipt"
        || test_record["record"]["source_after"] != actual["snapshot"]
    {
        return Err(invalid("test/review snapshot mismatch"));
    }
    let requirements_record = record_for(requirements)?;
    let required = requirements_record["record"]["requirements"]
        .as_object()
        .ok_or_else(|| invalid("requirements evidence missing"))?;
    let reviewed = actual["evidence"]
        .as_object()
        .ok_or_else(|| invalid("semantic evidence missing"))?;
    if required.is_empty()
        || required.keys().collect::<BTreeSet<_>>() != reviewed.keys().collect::<BTreeSet<_>>()
    {
        return Err(invalid("semantic requirements coverage mismatch"));
    }
    let accepted = actual["decision"] == "accept";
    if !accepted && actual["decision"] != "reject" {
        return Err(invalid("unresolved acceptance review"));
    }
    if accepted
        && (test_record["record"]["verdict"] != "passed"
            || reviewed.values().any(|v| v["met"] != true))
    {
        return Err(invalid(
            "accepted outcome contradicts observed/reviewed evidence",
        ));
    }
    if closed["reviewed_accepted"] != accepted {
        return Err(invalid("review result mismatch"));
    }
    if admission.origin != Origin::Operational {
        return Ok(withheld("synthetic_or_replay_excluded", identity));
    }
    if cutoff < close_at + 604_800_000_000_000 {
        return Ok(withheld("pending_seven_day_horizon", identity));
    }
    Ok(Eligibility {
        task: p.task,
        status: "operator_attested_mature_reviewed".into(),
        original_task: identity,
        label: Some(LabelRecord {
            task: p.task,
            criteria_met: accepted,
            diff_label: Some(if accepted {
                DiffLabel::Accepted
            } else {
                DiffLabel::Rejected
            }),
            reviewer: review.reviewer,
            recorded_at: crate::project_receipts::rfc3339(&at)?,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tsr_domain::{ActionKind, Candidate, Decision, Event, Utility};
    #[test]
    fn inconclusive_reconstruction_binds_configuration_source_origin_and_no_label() {
        fn add(raw: &mut Evidence, source: Source, kind: EventKind) -> tsr_domain::EventId {
            let event = Event::new(
                TaskId(Hash::of(&[b"fixture"])),
                raw.events
                    .last()
                    .map(|(_, e)| vec![e.id])
                    .unwrap_or_default(),
                source,
                kind,
                "2026-01-01T00:00:00Z",
            );
            let id = event.id;
            raw.events.push((raw.events.len() as i64 + 1, event));
            id
        }
        fn record(raw: &mut Evidence, source: Source, value: serde_json::Value) {
            let bytes = serde_json::to_string(&value).unwrap();
            let digest = Hash::of(&[bytes.as_bytes()]);
            raw.blobs.insert(digest, bytes);
            add(
                raw,
                source,
                EventKind::Observed {
                    observation: Observation::Evidence {
                        source: "configured-cycle@v1".into(),
                        digest,
                    },
                },
            );
        }
        for mode in 0..6 {
            let spec: serde_json::Value = serde_json::from_str(include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../examples/engine-invoice/project.json"
            )))
            .unwrap();
            let configuration = verification_configuration_digest(&spec).unwrap();
            let snapshot = Hash::of(&[b"snapshot"]);
            let mut raw = Evidence {
                events: vec![],
                side_events: vec![],
                blobs: BTreeMap::new(),
                as_of: "2026-01-01T00:00:01Z".into(),
            };
            record(
                &mut raw,
                Source::Operator,
                serde_json::json!({"fact":"coding","record":{"phase":"config","config":{"project":spec}}}),
            );
            record(
                &mut raw,
                if mode == 3 {
                    Source::Generator
                } else {
                    Source::Operator
                },
                serde_json::json!({"fact":"coding","record":{"phase":"verification","action":{"id":"verify"},"snapshot":snapshot,"configuration":if mode==1 {Hash::of(&[b"wrong-config"])}else{configuration}}}),
            );
            let c = Candidate::new(
                ActionKind::RunApprovedTest,
                Source::Operator,
                Hash::of(&[b"verify"]),
                vec![],
                vec![],
            )
            .unwrap();
            add(
                &mut raw,
                Source::Rules,
                EventKind::Proposed {
                    candidate: c.clone(),
                },
            );
            let commitment = add(
                &mut raw,
                Source::Rules,
                EventKind::Committed {
                    candidate: c.id,
                    decision: Decision::Selected {
                        value: c.id,
                        utility: Utility {
                            p_success: 0.5,
                            value: 0.,
                            cost: 0.,
                            ev: 0.,
                        },
                        propensity: 1.,
                        evidence: vec![],
                    },
                },
            );
            record(
                &mut raw,
                Source::Rules,
                serde_json::json!({"fact":"pending","commitment":commitment,"action":"verify"}),
            );
            record(
                &mut raw,
                Source::Rules,
                serde_json::json!({"fact":"coding","record":{"phase":"intent","plan":{"commitment":commitment,"edits":if mode==5 {serde_json::json!([{"path":"x"}])}else{serde_json::json!([])},"source_before":snapshot,"expected_after":snapshot}}}),
            );
            record(
                &mut raw,
                Source::Rules,
                serde_json::json!({"fact":"coding","record":{"phase":"receipt","commitment":commitment,"source_after":if mode==2 {Hash::of(&[b"wrong-source"])}else{snapshot}}}),
            );
            let mut closed = serde_json::json!({"opening_snapshot":snapshot,"source_snapshot":snapshot,"label":null,"review":null,"reviewed_accepted":null,"resource_accounting":{"active":true,"patch_bytes_charged":0,"unsettled_requests":0,"unresolved_tool_requests":0}});
            if mode == 4 {
                closed["reviewed_accepted"] = serde_json::json!(true);
            }
            let result = verify_inconclusive_execution(&raw, 1, &closed);
            assert_eq!(result.is_ok(), mode == 0, "mode {mode}: {result:?}");
        }
    }
    #[test]
    fn unresolved_commitment_cannot_be_hidden_by_reported_count() {
        let admission = LifecycleAdmission {
            training: None,
            non_tool: None,
            state_root: Hash::of(&[b"state"]),
            tool_ms_ceiling: 100,
            reason: "synthetic negative control".into(),
        };
        let open = Open {
            admission: admission.clone(),
            prefix: vec![],
            snapshot: Hash::of(&[b"snapshot"]),
            at: "2026-01-01T00:00:00Z".into(),
        };
        let task = TaskId(Hash::of(&[
            b"project-common-event-lifecycle@v1",
            &serde_json::to_vec(&open).unwrap(),
        ]));
        let candidate = Candidate::new(
            ActionKind::ProposePatch,
            Source::Generator,
            Hash::of(&[b"patch"]),
            vec![],
            vec![],
        )
        .unwrap();
        let kinds = [
            EventKind::Proposed {
                candidate: candidate.clone(),
            },
            EventKind::Committed {
                candidate: candidate.id,
                decision: Decision::Selected {
                    value: candidate.id,
                    utility: Utility {
                        p_success: 0.5,
                        value: 0.,
                        cost: 0.,
                        ev: 0.,
                    },
                    propensity: 1.,
                    evidence: vec![],
                },
            },
        ];
        let original_task = TaskId(Hash::of(&[b"original"]));
        let mut raw = vec![];
        let mut projected = vec![];
        let mut mapping = vec![];
        let start = Event::new(
            task,
            vec![],
            Source::Operator,
            EventKind::TaskOpened { task },
            &open.at,
        );
        projected.push((1, start));
        for kind in kinds {
            let e = Event::new(
                original_task,
                raw.last()
                    .map(|e: &(i64, Event)| vec![e.1.id])
                    .unwrap_or_default(),
                Source::Rules,
                kind.clone(),
                &open.at,
            );
            let derived = Event::new(
                task,
                vec![projected.last().unwrap().1.id],
                Source::Rules,
                kind,
                &open.at,
            );
            mapping.push((e.id, derived.id, e.recorded_at.clone()));
            raw.push((raw.len() as i64 + 1, e));
            projected.push((projected.len() as i64 + 1, derived));
        }
        let close = Event::new(
            task,
            vec![projected.last().unwrap().1.id],
            Source::Operator,
            EventKind::TaskClosed {
                task,
                reason: "reviewed".into(),
            },
            "2026-01-01T00:00:01Z",
        );
        projected.push((4, close));
        let closed = serde_json::json!({"version":"project-common-event-lifecycle@v1","task":task,"closed":true,"events":projected,"mapping":mapping,"consumed_tool_ms":0,"incomplete_executions":1,"admission":admission,"opening_snapshot":open.snapshot});
        let evidence = Evidence {
            events: raw,
            side_events: vec![],
            blobs: BTreeMap::new(),
            as_of: "2026-01-01T00:00:02Z".into(),
        };
        let error = verify_projection(
            &serde_json::to_value(open).unwrap(),
            &closed,
            &evidence,
            "2026-01-01T00:00:02Z",
        )
        .unwrap_err();
        assert!(error.to_string().contains("unresolved commitment"));
    }
}
