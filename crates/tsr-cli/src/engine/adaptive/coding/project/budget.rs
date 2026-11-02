//! Prospective accounting. Reservations survive unknown outcomes and never grant authority.
use super::*;
use anyhow::Context;
const SOURCE: &str = "project-resource-accounting@v1";
pub use tsr_learn::project_eligibility::ResourceLimits as Limits;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Reserve {
    pub request_id: String,
    pub state_root: Hash,
    pub envelope: String,
    pub model: String,
    pub local: bool,
    pub tokens_upper: u64,
    pub external_cost_upper: u64,
}
/// Explicitly unpriced external call: bounded calls/tokens/time, no invented dollar ceiling.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct UnpricedReserve {
    pub request_id: String,
    pub state_root: Hash,
    pub envelope: String,
    pub model: String,
    pub tokens_upper: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Settle {
    pub request_id: String,
    pub prompt_tokens: Option<u64>,
    pub output_tokens: Option<u64>,
    pub duration_ns: Option<u64>,
    pub external_cost_microusd: Option<u64>,
    pub outcome: String,
}
/// Operator-supplied exact failed-zero proof, independently verified by the transport.
/// This records accounting only; it cannot authorize another request or action.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct FailedZero {
    pub state_root: Hash,
    pub prior: Settle,
    pub proof: Hash,
    pub review: Hash,
}
/// Trusted transport attestation of a retained reservation never dispatched.
/// Not a measured provider response or authority for a later request.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct NotDispatched {
    pub state_root: Hash,
    pub reservation: UnpricedReserve,
    pub evidence: Hash,
    pub review: Hash,
}
fn failed_zero_usage(input: &FailedZero) -> Result<Settle> {
    ensure!(
        input.prior.outcome == "unknown"
            && input.prior.prompt_tokens.is_none()
            && input.prior.output_tokens.is_none()
            && input.prior.external_cost_microusd.is_none(),
        "only exact unknown failed request may be reconciled"
    );
    Ok(Settle {
        prompt_tokens: Some(0),
        output_tokens: Some(0),
        external_cost_microusd: Some(0),
        outcome: "measured".into(),
        ..input.prior.clone()
    })
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum Record {
    /// Explicit operator policy: no aggregate resource ceilings; never broadens action authority.
    UnlimitedPolicy {
        state_root: Hash,
        policy_sha256: String,
    },
    /// Operator selects honest unknown-cost accounting; this is not provider authority.
    UnpricedPolicy {
        state_root: Hash,
    },
    ReserveUnpriced {
        request: UnpricedReserve,
    },
    Auxiliary {
        request_id: String,
        binding: Hash,
        tool_ms_upper: u64,
    },
    Reserve {
        request: Reserve,
    },
    Settle {
        usage: Settle,
    },
    FailedZero {
        proof: FailedZero,
    },
    NotDispatched {
        proof: NotDispatched,
    },
    Execution {
        commitment: EventId,
        patch_bytes: u64,
        tool_ms_upper: u64,
    },
}
#[derive(Clone, Serialize, Deserialize)]
struct Entry {
    episode: TaskId,
    at: String,
    record: Record,
}
#[derive(Debug, Serialize)]
pub struct View {
    pub active: bool,
    pub episode: Option<TaskId>,
    pub limits: Option<Limits>,
    pub model_calls: u64,
    pub tokens_charged: u64,
    pub measured_prompt_tokens: u64,
    pub measured_output_tokens: u64,
    /// Known charges plus legacy finite reservations; excludes unpriced unknown charges.
    pub external_cost_charged_microusd: u64,
    pub external_cost_total_microusd: Option<u64>,
    pub measured_external_cost_microusd: u64,
    pub unknown_external_cost_requests: u64,
    pub unpriced_mode: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub unlimited_mode: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unlimited_policy_sha256: Option<String>,
    pub measured_local_duration_ns: u64,
    pub unknown_compute_requests: u64,
    pub unknown_usage: u64,
    /// Reconciled partial effects retain full reservations; duration was not measured.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub unmeasured_disposed_executions: u64,
    #[serde(default, skip_serializing_if = "is_zero")]
    pub known_not_dispatched_requests: u64,
    pub unsettled_requests: u64,
    pub unresolved_tool_requests: u64,
    pub patch_bytes_charged: u64,
    pub tool_ms_charged: u64,
    pub elapsed_wall_ms: u64,
    pub exceeded: bool,
    pub stopped: bool,
    pub provenance: String,
}
fn is_zero(value: &u64) -> bool {
    *value == 0
}
fn valid_policy_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn add(a: &mut u64, b: u64) -> Result<()> {
    *a = a
        .checked_add(b)
        .ok_or_else(|| anyhow::anyhow!("resource accounting overflow"))?;
    Ok(())
}
impl Engine {
    fn resource_entries(&self) -> Result<Vec<Entry>> {
        let side = self.shadow_store()?;
        let mut entries = vec![];
        for (_, e) in side.events()? {
            if let EventKind::Observed {
                observation: Observation::Evidence { source, digest },
            } = e.kind
            {
                if source != SOURCE {
                    continue;
                }
                let bytes = fs::read(side.directory.join("cas").join(digest.to_hex()))?;
                ensure!(Hash::of(&[&bytes]) == digest, "resource receipt changed");
                let entry: Entry = serde_json::from_slice(&bytes)?;
                let expected = if matches!(
                    entry.record,
                    Record::UnpricedPolicy { .. }
                        | Record::UnlimitedPolicy { .. }
                        | Record::FailedZero { .. }
                ) {
                    Source::Operator
                } else {
                    Source::Rules
                };
                ensure!(e.source == expected, "invalid resource provenance");
                entries.push(entry);
            }
        }
        Ok(entries)
    }
    fn resource_record(&self, episode: TaskId, record: Record) -> Result<()> {
        let side = self.shadow_store()?;
        side.write(|| {
            let source = if matches!(
                record,
                Record::UnpricedPolicy { .. }
                    | Record::UnlimitedPolicy { .. }
                    | Record::FailedZero { .. }
            ) {
                Source::Operator
            } else {
                Source::Rules
            };
            let entry = Entry {
                episode,
                at: lifecycle::now()?,
                record,
            };
            let digest = side.blob(&serde_json::to_vec(&entry)?)?;
            side.append(
                EventKind::Observed {
                    observation: Observation::Evidence {
                        source: SOURCE.into(),
                        digest,
                    },
                },
                source,
            )?;
            Ok(())
        })
    }
    pub fn project_budget(&self) -> Result<View> {
        let admission = self.project_resource_admission()?;
        let mut v=View{active:false,episode:None,limits:None,model_calls:0,tokens_charged:0,measured_prompt_tokens:0,measured_output_tokens:0,external_cost_charged_microusd:0,external_cost_total_microusd:Some(0),measured_external_cost_microusd:0,unknown_external_cost_requests:0,unpriced_mode:false,unlimited_mode:false,unlimited_policy_sha256:None,measured_local_duration_ns:0,unknown_compute_requests:0,unknown_usage:0,unmeasured_disposed_executions:0,known_not_dispatched_requests:0,unsettled_requests:0,unresolved_tool_requests:0,patch_bytes_charged:0,tool_ms_charged:0,elapsed_wall_ms:0,exceeded:false,stopped:false,provenance:"charged is an accounting quantity of known cost plus legacy finite reservations; measured cost is a subtotal and external_cost_total_microusd is null unless all bills are known; unpriced mode has no monetary cap and is not training-eligible; unknown outcomes retain reservations; local external charge is distinct from resource use".into()};
        let Some((episode, admission, at, closed)) = admission else {
            return Ok(v);
        };
        let Some(limits) = admission.non_tool else {
            return Ok(v);
        };
        v.active = true;
        v.episode = Some(episode);
        v.limits = Some(limits.clone());
        v.stopped = closed;
        let start = tsr_learn::maturity::timestamp(&at)?;
        let cutoff = if closed {
            self.project_lifecycle_view()?
                .events
                .last()
                .unwrap()
                .1
                .recorded_at
                .clone()
        } else {
            lifecycle::now()?
        };
        let now = tsr_learn::maturity::timestamp(&cutoff)?;
        ensure!(now >= start, "resource clock before admission");
        v.elapsed_wall_ms = ((now - start) / 1_000_000).try_into()?;
        let mut undispatched: BTreeMap<String, NotDispatched> = BTreeMap::new();
        let mut reserved = BTreeMap::new();
        let mut settled = BTreeMap::new();
        let mut unpriced = BTreeSet::new();
        let mut executions = BTreeSet::new();
        let disposed = self.disposed_partial_commitments()?;
        let mut auxiliary = BTreeSet::new();
        let mut prior = start;
        for entry in self.resource_entries()? {
            ensure!(entry.episode == episode, "resource episode mismatch");
            let t = tsr_learn::maturity::timestamp(&entry.at)?;
            ensure!(t >= prior && t <= now, "resource chronology mismatch");
            prior = t;
            match entry.record {
                Record::UnlimitedPolicy {
                    state_root,
                    policy_sha256,
                } => {
                    ensure!(
                        !v.unlimited_mode && admission.training.is_none(),
                        "invalid unlimited policy chronology"
                    );
                    ensure!(
                        valid_policy_sha256(&policy_sha256),
                        "invalid unlimited policy identity"
                    );
                    ensure!(
                        self.store.head_root()? == state_root
                            || self.resource_root_is_prefix(state_root)?,
                        "unlimited policy source state missing"
                    );
                    v.unlimited_mode = true;
                    v.unlimited_policy_sha256 = Some(policy_sha256);
                }
                Record::UnpricedPolicy { state_root } => {
                    ensure!(
                        !v.unpriced_mode && reserved.is_empty() && admission.training.is_none(),
                        "invalid unpriced accounting policy chronology"
                    );
                    ensure!(
                        self.store.head_root()? == state_root
                            || self.resource_root_is_prefix(state_root)?,
                        "unpriced policy source state missing"
                    );
                    v.unpriced_mode = true;
                }
                Record::ReserveUnpriced { request } => {
                    ensure!(
                        (v.unpriced_mode || v.unlimited_mode)
                            && unpriced.insert(request.request_id.clone()),
                        "unadmitted/duplicate unpriced request"
                    );
                    let request = Reserve {
                        request_id: request.request_id,
                        state_root: request.state_root,
                        envelope: request.envelope,
                        model: request.model,
                        tokens_upper: request.tokens_upper,
                        local: false,
                        external_cost_upper: 0,
                    };
                    ensure!(
                        reserved
                            .insert(request.request_id.clone(), request)
                            .is_none(),
                        "duplicate resource reservation"
                    );
                }

                Record::Auxiliary {
                    request_id,
                    binding,
                    tool_ms_upper,
                } => {
                    ensure!(
                        auxiliary.insert(request_id.clone()),
                        "duplicate auxiliary reservation"
                    );
                    let measured = self.records()?.iter().find_map(|(_, r)| match r {
                        super::super::super::Record::Coding {
                            record:
                                CodingRecord::Dependencies {
                                    resource_request: Some(id),
                                    binding: b,
                                    measured_tool_ms,
                                    ..
                                },
                        } if *id == request_id && *b == binding => *measured_tool_ms,
                        _ => None,
                    });
                    let measured = measured
                        .or(self.project_information_measurement(&request_id, binding)?)
                        .or(self.project_objective_measurement(&request_id, binding)?);
                    add(&mut v.tool_ms_charged, measured.unwrap_or(tool_ms_upper))?;
                    if measured.is_none() {
                        v.unknown_usage += 1;
                        v.unresolved_tool_requests += 1;
                    }
                    if measured.is_some_and(|n| n > tool_ms_upper) {
                        v.exceeded = true;
                    }
                }
                Record::Reserve { request } => {
                    ensure!(
                        reserved
                            .insert(request.request_id.clone(), request)
                            .is_none(),
                        "duplicate resource reservation"
                    );
                }
                Record::Settle { usage } => {
                    ensure!(
                        reserved.contains_key(&usage.request_id)
                            && !undispatched.contains_key(&usage.request_id)
                            && settled.insert(usage.request_id.clone(), usage).is_none(),
                        "unbound/duplicate resource settlement"
                    );
                }
                Record::NotDispatched { proof } => {
                    let r = reserved
                        .get(&proof.reservation.request_id)
                        .context("unreserved no-dispatch proof")?;
                    ensure!(
                        unpriced.contains(&proof.reservation.request_id)
                            && r.state_root == proof.reservation.state_root
                            && r.envelope == proof.reservation.envelope
                            && r.model == proof.reservation.model
                            && r.tokens_upper == proof.reservation.tokens_upper
                            && !settled.contains_key(&proof.reservation.request_id)
                            && self.resource_root_is_prefix(proof.state_root)?,
                        "no-dispatch provenance mismatch"
                    );
                    ensure!(
                        undispatched
                            .insert(proof.reservation.request_id.clone(), proof)
                            .is_none(),
                        "duplicate no-dispatch disposition"
                    );
                }
                Record::FailedZero { proof } => {
                    ensure!(
                        settled.get(&proof.prior.request_id) == Some(&proof.prior),
                        "reconciliation prior mismatch"
                    );
                    ensure!(
                        self.store.head_root()? == proof.state_root
                            || self.resource_root_is_prefix(proof.state_root)?,
                        "reconciliation state missing"
                    );
                    settled.insert(proof.prior.request_id.clone(), failed_zero_usage(&proof)?);
                }
                Record::Execution {
                    commitment,
                    patch_bytes,
                    tool_ms_upper,
                } => {
                    ensure!(
                        executions.insert(commitment),
                        "duplicate execution reservation"
                    );
                    add(&mut v.patch_bytes_charged, patch_bytes)?;
                    // Tool reservations remain charged if no independently observed execution exists.
                    let events = self.events()?;
                    let candidate = events.iter().find_map(|(_, e)| match e.kind {
                        EventKind::Committed { candidate, .. } if e.id == commitment => {
                            Some(candidate)
                        }
                        _ => None,
                    });
                    let duration = events.iter().find_map(|(_, e)| match e.kind {
                        EventKind::Executed {
                            candidate: c,
                            duration_ms,
                            ..
                        } if Some(c) == candidate && e.source == Source::Rules => Some(duration_ms),
                        _ => None,
                    });
                    add(&mut v.tool_ms_charged, duration.unwrap_or(tool_ms_upper))?;
                    if duration.is_none() && disposed.contains(&commitment) {
                        v.unmeasured_disposed_executions += 1;
                    }
                    if duration.is_none() && !disposed.contains(&commitment) {
                        v.unknown_usage += 1;
                    }
                    if duration.is_some_and(|n| n > tool_ms_upper) {
                        v.exceeded = true;
                    }
                }
            }
        }
        for (id, r) in reserved {
            if undispatched.contains_key(&id) {
                v.model_calls += 1;
                v.known_not_dispatched_requests += 1;
                add(&mut v.tokens_charged, r.tokens_upper)?;
                continue; // Reservation remains counted; no provider usage was measured.
            }
            v.model_calls += 1;
            let s = settled.get(&id);
            let measured = s.and_then(|s| s.prompt_tokens.zip(s.output_tokens));
            let tokens = match measured {
                Some((p, o)) => {
                    add(&mut v.measured_prompt_tokens, p)?;
                    add(&mut v.measured_output_tokens, o)?;
                    p.checked_add(o)
                        .ok_or_else(|| anyhow::anyhow!("token overflow"))?
                }
                None => r.tokens_upper,
            };
            let reported_cost = s.and_then(|s| s.external_cost_microusd);
            if let Some(cost) = reported_cost {
                add(&mut v.measured_external_cost_microusd, cost)?;
            }
            let unknown_cost = reported_cost.is_none();
            if unknown_cost {
                v.unknown_external_cost_requests += 1;
            }
            let cost = if unpriced.contains(&id) {
                // The numeric field sums only known charges. None below is the total,
                // never a zero-charge assertion for an unknown provider outcome.
                reported_cost.unwrap_or(0)
            } else {
                s.filter(|s| s.outcome == "measured")
                    .and_then(|s| s.external_cost_microusd)
                    .unwrap_or(r.external_cost_upper)
            };
            add(&mut v.tokens_charged, tokens)?;
            add(&mut v.external_cost_charged_microusd, cost)?;
            if let Some(ns) = s.and_then(|s| s.duration_ns) {
                if r.local {
                    add(&mut v.measured_local_duration_ns, ns)?;
                }
            } else {
                v.unknown_compute_requests += 1;
            }
            if s.is_none() {
                v.unsettled_requests += 1;
            }
            if measured.is_none() || s.and_then(|s| s.external_cost_microusd).is_none() {
                v.unknown_usage += 1;
            }
            if tokens > r.tokens_upper || (!unpriced.contains(&id) && cost > r.external_cost_upper)
            {
                v.exceeded = true;
            }
        }
        v.exceeded |= !v.unlimited_mode
            && (v.model_calls > limits.model_calls
                || v.tokens_charged > limits.model_tokens
                || v.patch_bytes_charged > limits.patch_bytes
                || v.elapsed_wall_ms > limits.wall_ms
                || (!v.unpriced_mode
                    && v.external_cost_charged_microusd > limits.external_cost_microusd)
                || v.tool_ms_charged > admission.tool_ms_ceiling);
        v.external_cost_total_microusd = if v.unknown_external_cost_requests == 0 {
            Some(v.measured_external_cost_microusd)
        } else {
            None
        };
        v.stopped |= v.exceeded
            || ((v.unpriced_mode || v.unlimited_mode)
                && (v.unknown_usage > 0
                    || v.unsettled_requests > 0
                    || v.unresolved_tool_requests > 0));
        Ok(v)
    }
    /// Explicit native policy admission, separate from all action/source permissions.
    /// Existing counters and per-request bounds remain intact; never reopens a closure.
    pub fn project_resource_unlimited_admit(
        &self,
        state_root: Hash,
        policy_sha256: String,
    ) -> Result<View> {
        self.write(|| {
            ensure!(
                valid_policy_sha256(&policy_sha256),
                "lowercase SHA256 policy identity required"
            );
            for entry in self.resource_entries()? {
                if let Record::UnlimitedPolicy {
                    state_root: old,
                    policy_sha256: policy,
                } = entry.record
                {
                    ensure!(
                        old == state_root && policy == policy_sha256,
                        "unlimited policy admission identity reused"
                    );
                    return self.project_budget();
                }
            }
            ensure!(
                self.store.head_root()? == state_root,
                "stale unlimited policy state"
            );
            let (_, admission, _, closed) = self
                .project_resource_admission()?
                .ok_or_else(|| anyhow::anyhow!("native resource admission required"))?;
            ensure!(
                !closed && admission.training.is_none(),
                "closed or training episode cannot become unlimited"
            );
            ensure!(
                self.adaptive_state_inner()?.pending.is_none()
                    && self.project_information_pending()?.is_none()
                    && !self.verifier_has_unfinished()?,
                "pending work prevents unlimited admission"
            );
            let used = self.project_budget()?;
            ensure!(
                used.active
                    && used.unsettled_requests == 0
                    && used.unresolved_tool_requests == 0
                    && used.unknown_usage == 0
                    && used.unknown_compute_requests == 0
                    && used.unknown_external_cost_requests == 0,
                "unknown resources prevent unlimited admission"
            );
            self.resource_record(
                used.episode.unwrap(),
                Record::UnlimitedPolicy {
                    state_root,
                    policy_sha256,
                },
            )?;
            self.project_budget()
        })
    }
    /// Legacy finite-cost/local verifier lookup only. Unpriced requests have a distinct
    /// typed record and cannot be mistaken for a zero-cost finite reservation.
    pub(crate) fn project_resource_lookup(
        &self,
        id: &str,
    ) -> Result<Option<(Reserve, Option<Settle>)>> {
        let entries = self.resource_entries()?;
        let request = entries.iter().find_map(|e| match &e.record {
            Record::Reserve { request } if request.request_id == id => Some(request.clone()),
            _ => None,
        });
        Ok(request.map(|r| {
            let usage = entries.iter().find_map(|e| match &e.record {
                Record::Settle { usage } if usage.request_id == id => Some(usage.clone()),
                _ => None,
            });
            (r, usage)
        }))
    }
    fn resource_root_is_prefix(&self, root: Hash) -> Result<bool> {
        let mut ids = Vec::new();
        for (_, event) in self.store.all()? {
            ids.push(event.id.to_string());
            if Hash::of(&ids.iter().map(|s| s.as_bytes()).collect::<Vec<_>>()) == root {
                return Ok(true);
            }
        }
        Ok(false)
    }
    /// Explicit prospective mode for a bounded trial without a monetary cap. Never grants inference permission.
    pub fn project_resource_unpriced_admit(&self, state_root: Hash) -> Result<View> {
        self.write(|| {
            for entry in self.resource_entries()? {
                if let Record::UnpricedPolicy { state_root: old } = entry.record {
                    ensure!(
                        old == state_root,
                        "unpriced policy admission identity reused"
                    );
                    return self.project_budget();
                }
            }
            ensure!(
                self.store.head_root()? == state_root,
                "stale unpriced policy state"
            );
            let v = self.project_budget()?;
            ensure!(
                v.active && !v.stopped && v.model_calls == 0,
                "unpriced mode requires a fresh active model budget"
            );
            let (_, admission, _, _) = self.project_resource_admission()?.unwrap();
            ensure!(
                admission.training.is_none(),
                "unpriced accounting is not training-eligible evidence"
            );
            if !v.unpriced_mode {
                self.resource_record(v.episode.unwrap(), Record::UnpricedPolicy { state_root })?;
            }
            self.project_budget()
        })
    }
    pub fn project_resource_reserve_unpriced(&self, request: UnpricedReserve) -> Result<View> {
        self.write(|| {
            for e in self.resource_entries()? {
                match e.record {
                    Record::ReserveUnpriced { request: r }
                        if r.request_id == request.request_id =>
                    {
                        ensure!(r == request, "reservation identity reused");
                        return self.project_budget();
                    }
                    Record::Reserve { request: r } if r.request_id == request.request_id => {
                        anyhow::bail!("reservation identity reused across pricing modes")
                    }
                    _ => {}
                }
            }
            let v = self.project_budget()?;
            ensure!(
                v.active
                    && (v.unpriced_mode || v.unlimited_mode)
                    && !v.stopped
                    && v.unsettled_requests == 0
                    && v.unresolved_tool_requests == 0
                    && v.unknown_usage == 0,
                "unpriced budget inactive/stopped or unknown; reconcile before dispatch"
            );
            ensure!(
                self.store.head_root()? == request.state_root
                    && short(&request.request_id)
                    && short(&request.model)
                    && short(&request.envelope)
                    && request.tokens_upper > 0,
                "invalid/stale resource reservation"
            );
            let l = v.limits.as_ref().unwrap();
            ensure!(
                v.unlimited_mode
                    || (v.model_calls < l.model_calls
                        && v.tokens_charged
                            .checked_add(request.tokens_upper)
                            .is_some_and(|n| n <= l.model_tokens)),
                "model budget exhausted"
            );
            self.resource_record(v.episode.unwrap(), Record::ReserveUnpriced { request })?;
            self.project_budget()
        })
    }
    pub fn project_resource_reserve(&self, request: Reserve) -> Result<View> {
        self.write(|| {
            for e in self.resource_entries()? {if let Record::Reserve{request:r}=e.record{if r.request_id==request.request_id{ensure!(r==request,"reservation identity reused");return self.project_budget()}}}
            ensure!(!self.resource_entries()?.iter().any(|e|matches!(&e.record,Record::ReserveUnpriced{request:r} if r.request_id==request.request_id)),"reservation identity reused across pricing modes");
            let v=self.project_budget()?;ensure!(!v.unpriced_mode,"use explicit unpriced reservation in unpriced mode");ensure!(v.active && !v.stopped && v.unsettled_requests==0 && v.unresolved_tool_requests==0,"budget inactive/stopped or unresolved reservation; reconcile before another dispatch");
            ensure!(self.store.head_root()?==request.state_root && short(&request.request_id) && short(&request.model) && short(&request.envelope) && request.tokens_upper>0,"invalid/stale resource reservation");
            ensure!(!request.local || request.external_cost_upper==0,"local external charge must be zero, not resource usage");
            let l=v.limits.as_ref().unwrap();ensure!(v.unlimited_mode || (v.model_calls<l.model_calls && v.tokens_charged.checked_add(request.tokens_upper).is_some_and(|n|n<=l.model_tokens) && v.external_cost_charged_microusd.checked_add(request.external_cost_upper).is_some_and(|n|n<=l.external_cost_microusd)),"model budget exhausted");
            self.resource_record(v.episode.unwrap(),Record::Reserve{request})?;self.project_budget()
        })
    }
    pub fn project_resource_reconcile_failed_zero(&self, proof: FailedZero) -> Result<View> {
        self.write(|| {
            failed_zero_usage(&proof)?;
            ensure!(self.store.head_root()? == proof.state_root, "stale reconciliation source");
            let entries = self.resource_entries()?;
            for entry in &entries {
                if let Record::FailedZero { proof: old } = &entry.record {
                    if old.prior.request_id == proof.prior.request_id {
                        ensure!(old == &proof, "reconciliation cannot be rewritten");
                        return self.project_budget();
                    }
                }
            }
            ensure!(entries.iter().any(|entry| matches!(&entry.record, Record::Settle { usage } if usage == &proof.prior)), "exact original settlement required");
            let budget = self.project_budget()?;
            ensure!(budget.active && !budget.exceeded && !self.project_resource_admission()?.is_some_and(|(_,_,_,closed)|closed), "inactive reconciliation episode");
            self.resource_record(budget.episode.unwrap(), Record::FailedZero { proof })?;
            self.project_budget()
        })
    }
    pub fn project_resource_not_dispatched(&self, proof: NotDispatched) -> Result<View> {
        self.write(|| {
            let entries = self.resource_entries()?;
            for e in &entries {
                if let Record::NotDispatched { proof: old } = &e.record {
                    if old.reservation.request_id == proof.reservation.request_id {
                        ensure!(old == &proof, "no-dispatch proof cannot be replaced");
                        return self.project_budget();
                    }
                }
            }
            ensure!(!self.project_resource_admission()?.is_some_and(|(_, _, _, closed)| closed), "closed episode cannot accept new disposition");
            ensure!(self.store.head_root()? == proof.state_root, "stale no-dispatch state");
            ensure!(entries.iter().any(|e| matches!(&e.record, Record::ReserveUnpriced {request} if request == &proof.reservation)), "exact unpriced reservation required");
            ensure!(!entries.iter().any(|e| matches!(&e.record, Record::Settle {usage} if usage.request_id == proof.reservation.request_id)), "settled/unknown effect cannot be disposed as undispatched");
            ensure!(proof.evidence != Hash::of(&[]) && proof.review != Hash::of(&[]), "no-dispatch evidence/review required");
            let view = self.project_budget()?;
            self.resource_record(view.episode.context("no episode")?, Record::NotDispatched {proof})?;
            self.project_budget()
        })
    }
    pub fn project_resource_settle(&self, usage: Settle) -> Result<View> {
        self.write(|| {
            ensure!(
                matches!(usage.outcome.as_str(), "measured" | "unknown" | "not_sent"),
                "invalid resource outcome"
            );
            ensure!(
                usage.prompt_tokens.is_some() == usage.output_tokens.is_some(),
                "partial token measurement remains unknown"
            );
            ensure!(
                usage.outcome == "measured"
                    || (usage.prompt_tokens.is_none() && usage.output_tokens.is_none()),
                "unknown/not-sent reservations are not refunded"
            );
            let entries = self.resource_entries()?;
            ensure!(!entries.iter().any(|e| matches!(&e.record,Record::NotDispatched{proof} if proof.reservation.request_id == usage.request_id)), "disposed request cannot receive a later settlement");
            let request = entries
                .iter()
                .find_map(|e| match &e.record {
                    Record::Reserve { request } if request.request_id == usage.request_id => {
                        Some(request)
                    }
                    Record::ReserveUnpriced { request } if request.request_id == usage.request_id => None,
                    _ => None,
                });
            let is_unpriced = entries.iter().any(|e| matches!(&e.record, Record::ReserveUnpriced {request} if request.request_id == usage.request_id));
            ensure!(request.is_some() || is_unpriced, "unreserved request");
            ensure!(
                request.is_none_or(|r| !r.local) || usage.external_cost_microusd.is_none_or(|c| c == 0),
                "invalid local external charge"
            );
            for e in &entries {
                if let Record::Settle { usage: s } = &e.record {
                    if s.request_id == usage.request_id {
                        ensure!(*s == usage, "settlement cannot be rewritten");
                        return self.project_budget();
                    }
                }
            }
            let v = self.project_budget()?;
            ensure!(
                !self
                    .project_resource_admission()?
                    .is_some_and(|(_, _, _, closed)| closed),
                "closed episode cannot accept new usage"
            );
            self.resource_record(
                v.episode.ok_or_else(|| anyhow::anyhow!("no episode"))?,
                Record::Settle { usage },
            )?;
            self.project_budget()
        })
    }
    pub(crate) fn reserve_project_preparation(
        &self,
        binding: Hash,
        upper: u64,
    ) -> Result<Option<String>> {
        self.write(|| self.reserve_project_preparation_inner(binding, upper))
    }
    pub(crate) fn reserve_project_preparation_inner(
        &self,
        binding: Hash,
        upper: u64,
    ) -> Result<Option<String>> {
        let v = self.project_budget()?;
        if !v.active {
            return Ok(None);
        }
        let (_, a, _, _) = self.project_resource_admission()?.unwrap();
        ensure!(!v.stopped && v.unresolved_tool_requests==0 && v.unsettled_requests==0,"preparation budget stopped or unresolved; reconcile receipts, never retry uncertain execution");
        ensure!(
            v.unlimited_mode
                || (v
                    .tool_ms_charged
                    .checked_add(upper)
                    .is_some_and(|n| n <= a.tool_ms_ceiling)
                    && v.elapsed_wall_ms
                        .checked_add(upper)
                        .is_some_and(|n| n <= v.limits.as_ref().unwrap().wall_ms)),
            "preparation tool/wall budget exhausted"
        );
        let request_id = format!("preparation-{}-{}", binding, self.resource_entries()?.len());
        self.resource_record(
            v.episode.unwrap(),
            Record::Auxiliary {
                request_id: request_id.clone(),
                binding,
                tool_ms_upper: upper,
            },
        )?;
        Ok(Some(request_id))
    }

    /// Read-only precommit check; execution still makes its own durable reservation.
    pub(crate) fn project_tier2_action_budget(&self, action: &str) -> Result<()> {
        let spec = self.project_configuration()?;
        ensure!(
            self.dependencies_ready(&spec)?,
            "project dependencies not ready"
        );
        let patches = if self.project_verification_for(action)? {
            vec![]
        } else {
            let patches = self
                .patches_for(action)?
                .ok_or_else(|| anyhow::anyhow!("no admitted patch action"))?;
            self.validate_patch_set(&patches)?;
            patches
        };
        let bytes = patches.iter().try_fold(0u64, |n, p| -> Result<u64> {
            n.checked_add(self.validate_patch(p)?.len() as u64)
                .ok_or_else(|| anyhow::anyhow!("patch bound overflow"))
        })?;
        let tool = spec
            .timeout_ms
            .checked_mul(execution_command_count(&spec) as u64)
            .ok_or_else(|| anyhow::anyhow!("tool bound overflow"))?;
        let v = self.project_budget()?;
        let (_, admission, _, _) = self
            .project_resource_admission()?
            .ok_or_else(|| anyhow::anyhow!("missing resource admission"))?;
        let limits = v
            .limits
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("missing resource limits"))?;
        ensure!(
            v.active
                && !v.exceeded
                && !v.stopped
                && v.unknown_usage == 0
                && v.unknown_compute_requests == 0
                && v.unsettled_requests == 0,
            "resource coverage unavailable"
        );
        ensure!(
            v.unlimited_mode
                || (v
                    .patch_bytes_charged
                    .checked_add(bytes)
                    .is_some_and(|n| n <= limits.patch_bytes)
                    && v.tool_ms_charged
                        .checked_add(tool)
                        .is_some_and(|n| n <= admission.tool_ms_ceiling)
                    && v.elapsed_wall_ms
                        .checked_add(tool)
                        .is_some_and(|n| n <= limits.wall_ms)),
            "candidate exceeds remaining patch/tool/wall budget"
        );
        Ok(())
    }
    pub(crate) fn reserve_project_execution(&self, plan: &Plan) -> Result<()> {
        let v = self.project_budget()?;
        if !v.active {
            return Ok(());
        }
        ensure!(
            !v.stopped && v.unsettled_requests == 0 && v.unresolved_tool_requests == 0,
            "resource budget stopped or generation unresolved"
        );
        let bytes = plan.edits.iter().try_fold(0u64, |n, e| {
            n.checked_add(e.replacement.len() as u64)
                .ok_or_else(|| anyhow::anyhow!("patch size overflow"))
        })?;
        let tool = plan
            .timeout_ms
            .checked_mul(plan.commands.len() as u64)
            .ok_or_else(|| anyhow::anyhow!("tool reservation overflow"))?;
        for e in self.resource_entries()? {
            if let Record::Execution {
                commitment,
                patch_bytes,
                tool_ms_upper,
            } = e.record
            {
                if commitment == plan.commitment {
                    ensure!(
                        patch_bytes == bytes && tool_ms_upper == tool,
                        "changed execution reservation"
                    );
                    return Ok(());
                }
            }
        }
        let (_, a, _, _) = self.project_resource_admission()?.unwrap();
        let limits = v.limits.as_ref().unwrap();
        ensure!(
            v.unlimited_mode
                || (v
                    .patch_bytes_charged
                    .checked_add(bytes)
                    .is_some_and(|n| n <= limits.patch_bytes)
                    && v.tool_ms_charged
                        .checked_add(tool)
                        .is_some_and(|n| n <= a.tool_ms_ceiling)),
            "patch/tool budget exhausted"
        );
        self.resource_record(
            v.episode.unwrap(),
            Record::Execution {
                commitment: plan.commitment,
                patch_bytes: bytes,
                tool_ms_upper: tool,
            },
        )
    }
    pub(crate) fn project_remaining_wall_ms(&self) -> Result<Option<u64>> {
        let v = self.project_budget()?;
        if v.unlimited_mode {
            return Ok(None);
        }
        Ok(v.limits
            .map(|l| l.wall_ms.saturating_sub(v.elapsed_wall_ms)))
    }
}

#[cfg(test)]
mod unpriced_tests {
    use super::*;
    fn setup() -> Result<(tempfile::TempDir, Engine)> {
        let d = tempfile::tempdir()?;
        let e = Engine::open(d.path())?;
        e.project_initialize(serde_json::from_value(serde_json::json!({
            "goal":"Authored accounting fixture, not a provider or task outcome", "image":"sha256:cff6dcceb8b9d203a818b7ff104b89a6aaa353f8576850ca4f58903539417945", "user":"501:20",
            "editable":{"src/lib.rs":"pub fn value() -> u32 { 1 }"},
            "protected":{"Cargo.toml":"[package]\nname=\"fixture\"\nversion=\"0.1.0\"\nedition=\"2021\"\n", "Cargo.lock":"version = 3\n\n[[package]]\nname = \"fixture\"\nversion = \"0.1.0\"\n", "tests/check.rs":"#[test] fn check() { assert!(true); }"},
            "cargo":true,"cases":[{"id":"check","args":[],"expected_stdout":""}],"timeout_ms":20000
        }))?)?;
        e.project_lifecycle_open(lifecycle::Admission {
            training: None,
            state_root: e.store.head_root()?,
            tool_ms_ceiling: 60000,
            reason: "Deterministic accounting fixture only".into(),
            non_tool: Some(Limits {
                model_calls: 2,
                model_tokens: 100,
                wall_ms: 600000,
                patch_bytes: 100,
                external_cost_microusd: 0,
            }),
        })?;
        Ok((d, e))
    }
    fn request(e: &Engine, id: &str, tokens: u64) -> Result<UnpricedReserve> {
        Ok(UnpricedReserve {
            request_id: id.into(),
            state_root: e.store.head_root()?,
            envelope: "fixture-envelope".into(),
            model: "fixture-provider".into(),
            tokens_upper: tokens,
        })
    }
    fn usage(id: &str, cost: Option<u64>) -> Settle {
        Settle {
            request_id: id.into(),
            prompt_tokens: Some(10),
            output_tokens: Some(5),
            duration_ns: Some(1000),
            external_cost_microusd: cost,
            outcome: "measured".into(),
        }
    }
    #[test]
    fn proven_not_dispatched_retains_reservation_and_rejects_unknown() -> Result<()> {
        let (d, e) = setup()?;
        e.project_resource_unpriced_admit(e.store.head_root()?)?;
        let r = request(&e, "not-dispatched", 50)?;
        e.project_resource_reserve_unpriced(r.clone())?;
        let p = NotDispatched {
            state_root: e.store.head_root()?,
            reservation: r.clone(),
            evidence: Hash::of(&[b"authored transport proof"]),
            review: Hash::of(&[b"reviewed validator"]),
        };
        let mut bad = p.clone();
        bad.reservation.envelope = "foreign".into();
        assert!(e.project_resource_not_dispatched(bad).is_err());
        let mut bad = p.clone();
        bad.state_root = Hash::of(&[b"stale"]);
        assert!(e.project_resource_not_dispatched(bad).is_err());
        let v = e.project_resource_not_dispatched(p.clone())?;
        assert_eq!(v.model_calls, 1);
        assert_eq!(v.tokens_charged, 50);
        assert_eq!(v.known_not_dispatched_requests, 1);
        assert_eq!(v.unsettled_requests, 0);
        assert_eq!(v.measured_prompt_tokens, 0);
        assert_eq!(v.unknown_usage, 0);
        assert!(!v.stopped);
        assert!(e
            .project_resource_settle(usage("not-dispatched", Some(0)))
            .is_err());
        assert_eq!(
            e.project_resource_not_dispatched(p)?
                .known_not_dispatched_requests,
            1
        );
        drop(e);
        let e = Engine::open(d.path())?;
        assert_eq!(e.project_budget()?.known_not_dispatched_requests, 1);
        let r = request(&e, "unknown", 50)?;
        e.project_resource_reserve_unpriced(r.clone())?;
        e.project_resource_settle(Settle {
            request_id: r.request_id.clone(),
            prompt_tokens: None,
            output_tokens: None,
            duration_ns: None,
            external_cost_microusd: None,
            outcome: "unknown".into(),
        })?;
        assert!(e
            .project_resource_not_dispatched(NotDispatched {
                state_root: e.store.head_root()?,
                reservation: r,
                evidence: Hash::of(&[b"proof"]),
                review: Hash::of(&[b"review"])
            })
            .is_err());
        assert!(e.project_budget()?.stopped);
        let (_closed_dir, e) = setup()?;
        e.project_resource_unpriced_admit(e.store.head_root()?)?;
        let r = request(&e, "closed-undispatched", 50)?;
        e.project_resource_reserve_unpriced(r.clone())?;
        e.project_resource_settle(usage("closed-undispatched", Some(0)))?;
        e.project_lifecycle_close_advisory("Authored closed lifecycle control".into())?;
        assert!(e
            .project_resource_not_dispatched(NotDispatched {
                state_root: e.store.head_root()?,
                reservation: r,
                evidence: Hash::of(&[b"proof"]),
                review: Hash::of(&[b"review"])
            })
            .unwrap_err()
            .to_string()
            .contains("closed episode"));
        Ok(())
    }
    #[test]
    fn unlimited_policy_preserves_legacy_identity_and_counts_across_restart() -> Result<()> {
        let (dir, e) = setup()?;
        let original = e.project_resource_admission()?;
        let original_bytes = serde_json::to_vec(&original)?;
        let legacy = serde_json::to_value(e.project_budget()?)?;
        assert!(legacy.get("unlimited_mode").is_none());
        assert!(legacy.get("unlimited_policy_sha256").is_none());
        let root = e.store.head_root()?;
        let policy = "ab".repeat(32);
        let view = e.project_resource_unlimited_admit(root, policy.clone())?;
        assert!(view.unlimited_mode && !view.stopped);
        assert_eq!(view.unlimited_policy_sha256, Some(policy.clone()));
        assert_eq!(
            serde_json::to_vec(&e.project_resource_admission()?)?,
            original_bytes
        );
        assert_eq!(e.store.head_root()?, root);
        for n in 0..3 {
            let id = format!("unlimited-{n}");
            e.project_resource_reserve_unpriced(request(&e, &id, 150)?)?;
            let view = e.project_resource_settle(Settle {
                prompt_tokens: Some(60),
                output_tokens: Some(40),
                ..usage(&id, Some(123))
            })?;
            assert!(!view.exceeded && !view.stopped);
        }
        let e = Engine::open(dir.path())?;
        let count = e.resource_entries()?.len();
        let view = e.project_resource_unlimited_admit(root, policy.clone())?;
        assert_eq!(e.resource_entries()?.len(), count);
        assert_eq!(view.model_calls, 3);
        assert_eq!(view.tokens_charged, 300);
        assert_eq!(view.external_cost_total_microusd, Some(369));
        assert_eq!(view.limits.unwrap().model_calls, 2); // original finite admission was not rewritten
        assert!(e
            .project_resource_unlimited_admit(root, "cd".repeat(32))
            .is_err());
        assert_eq!(e.project_remaining_wall_ms()?, None);
        // A cumulative numeric extension cannot silently disable explicit unlimited policy.
        let previous = e.project_resource_admission()?.unwrap().1;
        let view = e.project_resource_extend(lifecycle::ResourceExtension {
            state_root: root,
            previous,
            limits: Limits {
                model_calls: 0,
                model_tokens: 0,
                wall_ms: 0,
                patch_bytes: 0,
                external_cost_microusd: 0,
            },
            tool_ms_ceiling: 0,
            reason: "Authored unchanged unlimited policy fixture".into(),
        })?;
        assert!(view.unlimited_mode && !view.stopped);
        e.project_resource_reserve_unpriced(request(&e, "after-extension", 50)?)?;
        e.project_resource_settle(usage("after-extension", Some(1)))?;
        Ok(())
    }
    #[test]
    fn unlimited_policy_does_not_erase_historical_priced_request_overrun() -> Result<()> {
        let (dir, e) = setup()?;
        let root = e.store.head_root()?;
        e.project_resource_reserve(Reserve {
            request_id: "priced-overrun".into(),
            state_root: root,
            envelope: "fixture".into(),
            model: "fixture".into(),
            local: false,
            tokens_upper: 50,
            external_cost_upper: 0,
        })?;
        let before = e.project_resource_settle(usage("priced-overrun", Some(1)))?;
        assert!(before.exceeded && before.stopped);
        let after = e.project_resource_unlimited_admit(root, "ab".repeat(32))?;
        assert!(after.unlimited_mode && after.exceeded && after.stopped);
        assert_eq!(after.external_cost_total_microusd, Some(1));
        let e = Engine::open(dir.path())?;
        assert!(e.project_budget()?.stopped);
        assert!(e
            .project_resource_reserve_unpriced(request(&e, "must-not-dispatch", 50)?)
            .is_err());
        assert_eq!(e.project_budget()?.model_calls, 1);
        Ok(())
    }
    #[test]
    fn unlimited_policy_preserves_unknown_and_per_request_capacity_stops() -> Result<()> {
        for unknown in [true, false] {
            let (dir, e) = setup()?;
            e.project_resource_unlimited_admit(e.store.head_root()?, "01".repeat(32))?;
            e.project_resource_reserve_unpriced(request(&e, "bounded-request", 10)?)?;
            let view = if unknown {
                e.project_resource_settle(Settle {
                    prompt_tokens: None,
                    output_tokens: None,
                    outcome: "unknown".into(),
                    ..usage("bounded-request", None)
                })?
            } else {
                e.project_resource_settle(usage("bounded-request", Some(999)))?
            }; // 15 > per-request capacity10
            assert!(view.stopped);
            assert_eq!(view.exceeded, !unknown);
            if unknown {
                assert_eq!(view.tokens_charged, 10);
                assert_eq!(view.external_cost_total_microusd, None);
            }
            let e = Engine::open(dir.path())?;
            assert!(e
                .project_resource_reserve_unpriced(request(&e, "must-not-dispatch", 10)?)
                .is_err());
            assert_eq!(e.project_budget()?.model_calls, 1);
        }
        Ok(())
    }
    #[test]
    fn unlimited_policy_rejects_stale_forged_unknown_and_closed_admissions() -> Result<()> {
        let (_dir, e) = setup()?;
        let root = e.store.head_root()?;
        assert!(e
            .project_resource_unlimited_admit(Hash::of(&[b"stale"]), "ab".repeat(32))
            .is_err());
        assert!(e
            .project_resource_unlimited_admit(root, "not-a-policy".into())
            .is_err());
        e.project_resource_reserve(Reserve {
            request_id: "pending".into(),
            state_root: root,
            envelope: "fixture".into(),
            model: "fixture".into(),
            local: true,
            tokens_upper: 10,
            external_cost_upper: 0,
        })?;
        assert!(e
            .project_resource_unlimited_admit(root, "ab".repeat(32))
            .is_err());
        e.project_resource_settle(Settle {
            prompt_tokens: Some(1),
            output_tokens: Some(1),
            ..usage("pending", Some(0))
        })?;
        e.project_lifecycle_close_advisory(
            "Fixture accounting only, no execution or acceptance".into(),
        )?;
        assert!(e
            .project_resource_unlimited_admit(root, "ab".repeat(32))
            .is_err());
        assert!(e.project_budget()?.stopped);
        let (_dir, e) = setup()?;
        let entry = Entry {
            episode: e.project_budget()?.episode.unwrap(),
            at: lifecycle::now()?,
            record: Record::UnlimitedPolicy {
                state_root: e.store.head_root()?,
                policy_sha256: "ab".repeat(32),
            },
        };
        let side = e.shadow_store()?;
        side.write(|| {
            let digest = side.blob(&serde_json::to_vec(&entry)?)?;
            side.append(
                EventKind::Observed {
                    observation: Observation::Evidence {
                        source: SOURCE.into(),
                        digest,
                    },
                },
                Source::Generator,
            )?;
            Ok(())
        })?;
        assert!(e.project_budget().is_err());
        Ok(())
    }
    #[test]
    fn unlimited_removes_obsolete_cost_normalization_without_removing_candidates() -> Result<()> {
        let (_dir, e) = setup()?;
        e.project_resource_unlimited_admit(e.store.head_root()?, "ef".repeat(32))?;
        let snapshot = e.project_acceptance_status()?.snapshot;
        let response = e.project_admit_verification(e.store.head_root()?, snapshot)?;
        let result = e.evaluate_proposal(serde_json::from_value(response)?, true)?;
        assert!(result.committed);
        let plan = e.coding_plan()?;
        assert_eq!(plan.timeout_ms, 20000);
        assert_eq!(plan.commands.len(), 3);
        assert!(e.project_cost_estimate(&plan.action)?.is_none());
        e.project_tier2_action_budget(&plan.action)?;
        assert!(e
            .project_resource_unlimited_admit(e.store.head_root()?, "ab".repeat(32))
            .is_err());
        Ok(())
    }
    #[test]
    fn unpriced_accounting_is_explicit_measured_and_restart_bound() -> Result<()> {
        let (d, e) = setup()?;
        let first = request(&e, "one", 50)?;
        assert!(e.project_resource_reserve_unpriced(first.clone()).is_err());
        assert!(e
            .project_resource_unpriced_admit(Hash::of(&[b"stale"]))
            .is_err());
        let root = e.store.head_root()?;
        e.project_resource_unpriced_admit(root)?;
        let v = e.project_resource_reserve_unpriced(first.clone())?;
        assert!(v.unpriced_mode && v.stopped);
        assert_eq!(v.external_cost_total_microusd, None);
        assert_eq!(v.unknown_external_cost_requests, 1);
        assert_eq!(v.model_calls, 1);
        assert_eq!(v.tokens_charged, 50);
        assert_eq!(e.store.head_root()?, root); // prospective side accounting preserves request binding
        drop(e);
        let e = Engine::open(d.path())?;
        assert_eq!(e.project_resource_unpriced_admit(root)?.model_calls, 1);
        assert_eq!(e.project_resource_reserve_unpriced(first)?.model_calls, 1);
        assert!(e
            .project_resource_reserve_unpriced(request(&e, "two", 50)?)
            .is_err());
        let v = e.project_resource_settle(usage("one", Some(123)))?;
        assert_eq!(v.external_cost_total_microusd, Some(123));
        assert_eq!(v.external_cost_charged_microusd, 123);
        assert!(!v.stopped && !v.exceeded); // zero legacy monetary field is not a fabricated cap
        assert_eq!(v.tokens_charged, 15);
        assert!(e.project_resource_settle(usage("one", Some(0))).is_err());
        e.project_resource_reserve_unpriced(request(&e, "two", 50)?)?;
        let v = e.project_resource_settle(usage("two", Some(234)))?;
        assert_eq!(v.external_cost_total_microusd, Some(357));
        assert!(e
            .project_resource_reserve_unpriced(request(&e, "three", 1)?)
            .is_err());
        Ok(())
    }
    #[test]
    fn unpriced_unknown_and_not_sent_retain_reservations() -> Result<()> {
        for outcome in ["unknown", "not_sent"] {
            let (d, e) = setup()?;
            e.project_resource_unpriced_admit(e.store.head_root()?)?;
            e.project_resource_reserve_unpriced(request(&e, "one", 50)?)?;
            let usage = Settle {
                request_id: "one".into(),
                prompt_tokens: None,
                output_tokens: None,
                duration_ns: None,
                external_cost_microusd: None,
                outcome: outcome.into(),
            };
            e.project_resource_settle(usage.clone())?;
            drop(e);
            let e = Engine::open(d.path())?;
            let v = e.project_resource_settle(usage)?;
            assert!(v.stopped);
            assert_eq!(v.tokens_charged, 50);
            assert_eq!(v.model_calls, 1);
            assert_eq!(v.external_cost_total_microusd, None);
            assert!(e
                .project_resource_reserve_unpriced(request(&e, "two", 1)?)
                .is_err());
        }
        Ok(())
    }
    #[test]
    fn unpriced_missing_cost_and_token_overrun_stop_without_false_zero() -> Result<()> {
        for cost in [None, Some(999)] {
            let (_d, e) = setup()?;
            e.project_resource_unpriced_admit(e.store.head_root()?)?;
            e.project_resource_reserve_unpriced(request(&e, "one", 10)?)?;
            let v = e.project_resource_settle(usage("one", cost))?;
            assert!(v.exceeded && v.stopped);
            assert_eq!(v.tokens_charged, 15);
            assert_eq!(v.external_cost_total_microusd, cost);
        }
        Ok(())
    }
    #[test]
    fn unpriced_unknown_execution_keeps_known_bill_and_blocks_further_work() -> Result<()> {
        let (_d, e) = setup()?;
        e.project_resource_unpriced_admit(e.store.head_root()?)?;
        e.project_resource_reserve_unpriced(request(&e, "one", 50)?)?;
        let v = e.project_resource_settle(Settle {
            request_id: "one".into(),
            prompt_tokens: None,
            output_tokens: None,
            duration_ns: None,
            external_cost_microusd: Some(123),
            outcome: "unknown".into(),
        })?;
        assert_eq!(v.external_cost_total_microusd, Some(123));
        assert_eq!(v.external_cost_charged_microusd, 123);
        assert_eq!(v.tokens_charged, 50);
        assert!(v.stopped && v.unknown_usage > 0);
        assert!(e
            .project_resource_reserve_unpriced(request(&e, "two", 1)?)
            .is_err());
        Ok(())
    }
    #[test]
    fn unpriced_policy_rejects_forged_source_and_foreign_root() -> Result<()> {
        for source in [Source::Rules, Source::Operator] {
            let (_d, e) = setup()?;
            let side = e.shadow_store()?;
            let entry = Entry {
                episode: e.project_budget()?.episode.unwrap(),
                at: lifecycle::now()?,
                record: Record::UnpricedPolicy {
                    state_root: if source == Source::Rules {
                        e.store.head_root()?
                    } else {
                        Hash::of(&[b"foreign-root"])
                    },
                },
            };
            let digest = side.blob(&serde_json::to_vec(&entry)?)?;
            side.append(
                EventKind::Observed {
                    observation: Observation::Evidence {
                        source: SOURCE.into(),
                        digest,
                    },
                },
                source,
            )?;
            assert!(e.project_budget().is_err());
        }
        Ok(())
    }
    #[test]
    fn finite_cost_contract_remains_enforced_and_cannot_be_reclassified() -> Result<()> {
        let (_d, e) = setup()?;
        let r = Reserve {
            request_id: "finite".into(),
            state_root: e.store.head_root()?,
            envelope: "fixture".into(),
            model: "fixture".into(),
            local: false,
            tokens_upper: 50,
            external_cost_upper: 1,
        };
        assert!(e.project_resource_reserve(r.clone()).is_err()); // original zero monetary admission
        let r = Reserve {
            external_cost_upper: 0,
            ..r
        };
        e.project_resource_reserve(r)?;
        assert!(e
            .project_resource_unpriced_admit(e.store.head_root()?)
            .is_err());
        let v = e.project_resource_settle(usage("finite", Some(1)))?;
        assert!(v.exceeded && v.stopped);
        assert!(!v.unpriced_mode);
        Ok(())
    }
    #[test]
    fn failed_zero_is_append_only_identity_bound_and_restart_safe() -> Result<()> {
        let (dir, e) = setup()?;
        e.project_resource_unpriced_admit(e.store.head_root()?)?;
        e.project_resource_reserve_unpriced(request(&e, "failed", 50)?)?;
        let prior = Settle {
            request_id: "failed".into(),
            prompt_tokens: None,
            output_tokens: None,
            duration_ns: Some(1000),
            external_cost_microusd: None,
            outcome: "unknown".into(),
        };
        e.project_resource_settle(prior.clone())?;
        let root = e.store.head_root()?;
        let admission = e.project_resource_admission()?;
        let proof = FailedZero {
            state_root: root,
            prior,
            proof: Hash::of(&[b"authored exact provider proof"]),
            review: Hash::of(&[b"authored independent review"]),
        };
        let mut wrong = proof.clone();
        wrong.prior.request_id = "other".into();
        assert!(e.project_resource_reconcile_failed_zero(wrong).is_err());
        let view = e.project_resource_reconcile_failed_zero(proof.clone())?;
        assert!(
            !view.stopped
                && view.model_calls == 1
                && view.unknown_usage == 0
                && view.measured_external_cost_microusd == 0
        );
        assert_eq!(e.store.head_root()?, root);
        assert_eq!(e.project_resource_admission()?, admission);
        let n = e.resource_entries()?.len();
        e.project_resource_reconcile_failed_zero(proof.clone())?;
        assert_eq!(n, e.resource_entries()?.len());
        let mut wrong = proof.clone();
        wrong.review = Hash::of(&[b"substituted"]);
        assert!(e.project_resource_reconcile_failed_zero(wrong).is_err());
        let fresh = Engine::open(dir.path())?;
        assert!(!fresh.project_budget()?.stopped);
        assert_eq!(fresh.project_budget()?.model_calls, 1);
        assert!(fresh
            .project_resource_settle(failed_zero_usage(&proof)?)
            .is_err());
        Ok(())
    }
}
