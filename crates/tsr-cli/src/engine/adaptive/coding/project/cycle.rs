//! State-driven local scheduling. No provider, scope expansion, or automatic review.
use super::*;
use crate::engine::adaptive::provider::{Evaluation, Response, Suggestion};
use anyhow::Context;
use serde_json::{json, Value};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub binding: Hash,
    pub response_digest: Hash,
    pub state_root: Hash,
    pub source_snapshot: Hash,
    pub reviewer: String,
    pub generator: String,
    pub decision: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Advance {
    pub binding: Hash,
    pub response: Response,
    pub review: Review,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Acknowledgment {
    pub binding: Hash,
    pub evidence_key: Hash,
    pub outcome: String,
    pub rationale: String,
    pub reviewer: String,
    pub generator: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Continuation {
    pub binding: Hash,
    pub message_id: String,
    pub reason: String,
}
const CONTINUE: &str = "project-cycle-continuation@v1";
const CONTINUE_USED: &str = "project-cycle-continuation-used@v1";
const CONSIDERED: &str = "project-cycle-considered@v1";
fn cycle_hash<T: Serialize>(value: &T) -> Result<Hash> {
    Ok(Hash::of(&[&serde_json::to_vec(value)?]))
}
impl Engine {
    fn continuation_anchor(&self) -> Result<Hash> {
        self.continuation_anchor_with(&self.project_relational_context()?)
    }
    fn continuation_anchor_with(&self, relational: &Value) -> Result<Hash> {
        let status = self.project_acceptance_status()?;
        cycle_hash(
            &json!({"snapshot":status.snapshot,"requirements":status.requirements_revision,
            "review":status.decision_record,"test_receipt":status.test_receipt,"scope":cycle_hash(&self.project_configuration()?)?,
            "resource_admission":self.project_resource_admission()?,"considered":self.cycle_considered()?,
            "guidance":self.acceptance_cycle()?["guidance_provenance"],
            "parents":relational["inputs"]["parents"],
            "lineage":relational["history"]}),
        )
    }
    fn cycle_continuation(&self) -> Result<Option<EventId>> {
        self.cycle_continuation_with(&self.project_relational_context()?)
    }
    fn cycle_continuation_with(&self, relational: &Value) -> Result<Option<EventId>> {
        let mut candidates = Vec::new();
        let mut used = BTreeSet::<EventId>::new();
        for (_, event) in self.events()? {
            if let EventKind::Observed {
                observation: Observation::Evidence { source, digest },
            } = event.kind
            {
                if source != CONTINUE && source != CONTINUE_USED {
                    continue;
                }
                ensure!(event.source == Source::Operator, "continuation provenance");
                let bytes = fs::read(self.directory.join("cas").join(digest.to_hex()))?;
                ensure!(
                    Hash::of(&[&bytes]) == digest,
                    "continuation content changed"
                );
                let value: Value = serde_json::from_slice(&bytes)?;
                if source == CONTINUE {
                    candidates.push((event.id, value));
                } else {
                    used.insert(serde_json::from_value(value["continuation"].clone())?);
                }
            }
        }
        let anchor = self.continuation_anchor_with(relational)?;
        Ok(candidates.into_iter().rev().find_map(|(id, v)| {
            (!used.contains(&id) && v["anchor"] == json!(anchor)).then_some(id)
        }))
    }
    fn consume_continuation(&self, id: EventId, reason: &str) -> Result<EventId> {
        let digest = self.blob(&serde_json::to_vec(
            &json!({"continuation":id,"reason":reason,"accepted":false}),
        )?)?;
        self.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    source: CONTINUE_USED.into(),
                    digest,
                },
            },
            Source::Operator,
        )
    }
    /// An attributed operator instruction opens one reviewed decision, not new authority or wanting.
    pub fn project_cycle_continue(&self, input: Continuation) -> Result<Value> {
        self.write(||{
            let current=self.project_cycle_request_inner(true)?;
            ensure!(current["branch"]=="review_required" && current["binding"]==json!(input.binding),"stale or ineligible continuation");
            ensure!(!input.message_id.trim().is_empty() && input.message_id.len()<=256 && !input.reason.trim().is_empty() && input.reason.len()<=2048,"bounded attributed continuation required");
            let anchor=self.continuation_anchor()?;
            // An identical instruction cannot reopen the same checkpoint after consumption.
            for (_,event) in self.events()? {
                if let EventKind::Observed{observation:Observation::Evidence{source,digest}}=event.kind {
                    if source==CONTINUE {
                        let bytes=fs::read(self.directory.join("cas").join(digest.to_hex()))?;
                        ensure!(Hash::of(&[&bytes])==digest,"continuation content changed");
                        let prior:Value=serde_json::from_slice(&bytes)?;
                        ensure!(prior["anchor"]!=json!(anchor) || prior["instruction"]["message_id"]!=json!(input.message_id),"continuation already recorded");
                    }
                }
            }
            let digest=self.blob(&serde_json::to_vec(&json!({"anchor":anchor,"instruction":input,"grants_authority":false,"changes_desires":false,"accepted":false}))?)?;
            let id=self.append(EventKind::Observed{observation:Observation::Evidence{source:CONTINUE.into(),digest}},Source::Operator)?;
            Ok(json!({"continuation":id,"next":self.project_cycle_request()?}))
        })
    }
    /// Read-only receipt recovery: never creates or reopens a continuation.
    pub fn project_cycle_continuation_receipt(&self, input: Continuation) -> Result<Value> {
        let id = self
            .cycle_continuation()?
            .ok_or_else(|| anyhow::anyhow!("no active continuation receipt"))?;
        for (_, event) in self.events()? {
            if event.id != id {
                continue;
            }
            if let EventKind::Observed {
                observation: Observation::Evidence { source, digest },
            } = event.kind
            {
                ensure!(
                    source == CONTINUE && event.source == Source::Operator,
                    "continuation provenance"
                );
                let bytes = fs::read(self.directory.join("cas").join(digest.to_hex()))?;
                ensure!(
                    Hash::of(&[&bytes]) == digest,
                    "continuation receipt altered"
                );
                let value: Value = serde_json::from_slice(&bytes)?;
                ensure!(
                    value["instruction"] == serde_json::to_value(&input)?
                        && value["anchor"] == json!(self.continuation_anchor()?),
                    "continuation receipt mismatch"
                );
                return Ok(json!({"continuation":id,"next":self.project_cycle_request()?}));
            }
        }
        anyhow::bail!("continuation receipt missing")
    }
    pub fn project_cycle_close_continuation(&self, state_root: Hash, id: EventId) -> Result<Value> {
        self.write(|| {
            let resources = self.project_budget()?;
            ensure!(
                self.adaptive_state()?.pending.is_none()
                    && !resources.stopped
                    && !resources.exceeded
                    && resources.unsettled_requests == 0
                    && resources.unresolved_tool_requests == 0
                    && resources.unknown_usage == 0,
                "cannot close uncertain continuation"
            );
            ensure!(
                self.store.head_root()? == state_root && self.cycle_continuation()? == Some(id),
                "stale continuation close"
            );
            let record = self.consume_continuation(id, "terminal without approved decision")?;
            Ok(json!({"record":record,"executed":false,"next":self.project_cycle_request()?}))
        })
    }
    fn cycle_considered(&self) -> Result<BTreeSet<Hash>> {
        let mut keys = BTreeSet::new();
        for (_, event) in self.events()? {
            if let EventKind::Observed {
                observation: Observation::Evidence { source, digest },
            } = event.kind
            {
                if source != CONSIDERED {
                    continue;
                }
                ensure!(event.source == Source::Operator, "consideration provenance");
                let bytes = fs::read(self.directory.join("cas").join(digest.to_hex()))?;
                ensure!(
                    Hash::of(&[&bytes]) == digest,
                    "consideration content changed"
                );
                let value: Value = serde_json::from_slice(&bytes)?;
                keys.insert(serde_json::from_value(value["evidence_key"].clone())?);
            }
        }
        Ok(keys)
    }
    fn cycle_reconsideration(&self, context: &Value) -> Result<Option<Value>> {
        let state = self.adaptive_state()?;
        let events = self.events()?;
        let positions: BTreeMap<_, _> = events.iter().map(|(seq, e)| (e.id, *seq)).collect();
        let considered = self.cycle_considered()?;
        let inputs = &context["inputs"];
        let receipt_ids = inputs["returns"]
            .as_array()
            .into_iter()
            .flatten()
            .chain(
                inputs["information_returns"]
                    .as_array()
                    .into_iter()
                    .flatten(),
            )
            .chain(inputs["service_returns"].as_array().into_iter().flatten())
            .map(|r| serde_json::from_value::<EventId>(r["receipt"].clone()))
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let semantic_review: Option<EventId> = if inputs["semantic_return"]["applicable"] == true {
            serde_json::from_value(inputs["semantic_return"]["review"].clone())?
        } else {
            None
        };
        for item in context["history"].as_array().into_iter().flatten().rev() {
            if item["current"] == true || item["withdrawn"] == true || item["superseded"] == true {
                continue;
            }
            let lineage: EventId = serde_json::from_value(item["admission"].clone())?;
            let proposal: crate::engine::deliberation::relational::Proposal =
                serde_json::from_value(item["proposal"].clone())?;
            if !proposal.parents.iter().all(|id| {
                state.preferences.iter().any(|p| {
                    p.preference.id == *id
                        && !matches!(
                            p.status,
                            Some(PreferenceStatus::Withdrawn | PreferenceStatus::OutOfScope)
                        )
                })
            }) {
                continue;
            }
            let published: BTreeSet<PreferenceId> = inputs["parents"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|p| serde_json::from_value(p["id"].clone()))
                .collect::<std::result::Result<_, _>>()?;
            if !proposal.parents.iter().all(|id| published.contains(id)) {
                continue;
            }
            let at = *positions
                .get(&lineage)
                .context("missing aspiration provenance")?;
            if !receipt_ids
                .iter()
                .chain(semantic_review.iter())
                .any(|id| positions.get(id).is_some_and(|seq| *seq > at))
            {
                continue;
            }
            let mut question = json!({"lineage":lineage,"binding":context["binding"],"snapshot":inputs["snapshot"],"requirements_revision":inputs["requirements_revision"],"scope":cycle_hash(&self.project_configuration()?)?,"receipt_ids":receipt_ids,"semantic_review":semantic_review});
            let mut key_inputs = question.clone();
            key_inputs.as_object_mut().unwrap().remove("binding");
            let key = cycle_hash(&key_inputs)?;
            if considered.contains(&key) {
                continue;
            }
            question["evidence_key"] = json!(key);
            return Ok(Some(question));
        }
        Ok(None)
    }
    fn cycle_mark_considered(
        &self,
        question: &Value,
        outcome: &str,
        attestation: Value,
    ) -> Result<EventId> {
        let record = json!({"evidence_key":question["evidence_key"],"question":question,"outcome":outcome,"attestation":attestation,"grants_authority":false,"accepted":false});
        let digest = self.blob(&serde_json::to_vec(&record)?)?;
        self.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    source: CONSIDERED.into(),
                    digest,
                },
            },
            Source::Operator,
        )
    }
    /// A completed advisory result is attributed evidence, never task acceptance.
    pub fn project_cycle_acknowledge(&self, ack: Acknowledgment) -> Result<Value> {
        self.write(|| {
            let current=self.project_cycle_request_inner(true)?;
            ensure!(current["branch"]=="reconsider" && current["binding"]==json!(ack.binding) && current["reconsideration"]["evidence_key"]==json!(ack.evidence_key),"stale/non-advisory consideration");
            ensure!(matches!(ack.outcome.as_str(),"no_new_insight"|"abstain"|"rejected"|"unknown") && !ack.rationale.trim().is_empty() && (if self.project_configuration()?.browser { super::browser::narrative(&ack.rationale) } else { ack.rationale.len()<=512 }),"bounded advisory outcome required");
            ensure!(ack.reviewer!=ack.generator && [&ack.reviewer,&ack.generator].iter().all(|id|!id.trim().is_empty() && id.len()<=256),"separate supplied review attribution required");
            let record=self.cycle_mark_considered(&current["reconsideration"],&ack.outcome,json!(ack))?;
            Ok(json!({"considered_record":record,"outcome":ack.outcome,"executed":false,"advisory_only":true,"next":self.project_cycle_request()?}))
        })
    }
    fn cycle_resource_root(&self) -> Result<Hash> {
        let mut view = serde_json::to_value(self.project_budget()?)?;
        view.as_object_mut().unwrap().remove("elapsed_wall_ms");
        cycle_hash(&json!({"admission":self.project_resource_admission()?,"accounting":view}))
    }
    pub fn project_cycle_request(&self) -> Result<Value> {
        self.project_cycle_request_inner(false)
    }
    /// Read-only binding for consuming a supplied response, never inference authority.
    pub fn project_cycle_response_context(&self) -> Result<Value> {
        self.project_cycle_request_inner(true)
    }
    // Consuming an already supplied response is not a new inference reservation.
    // Unknown/overrun/wall/closed/authority checks below still apply unchanged.
    fn project_cycle_request_inner(&self, consuming_response: bool) -> Result<Value> {
        self.project_cycle_request_with_hook(consuming_response, || Ok(()))
    }
    fn project_cycle_request_with_hook(
        &self,
        consuming_response: bool,
        after_projection: impl FnOnce() -> Result<()>,
    ) -> Result<Value> {
        let status = self.project_status()?;
        let attention = self.project_attention()?;
        let resources = self.project_budget()?;
        let resource_root = self.cycle_resource_root()?;
        let pending = self.adaptive_state()?.pending.is_some();
        // Stack-local reuse only. Neither helper retains this projection; all
        // final integrity/source/resource checks below still read current state.
        let relational = self.project_relational_context()?;
        let reconsideration = self.cycle_reconsideration(&relational)?;
        let continuation = self.cycle_continuation_with(&relational)?;
        after_projection()?;
        let information = self.project_information_pending()?;
        let objective = self.project_objective_context()?;
        let objective_next = objective["context"]["next"].as_str();
        let ongoing = objective_next.is_some() && objective_next != Some("stop");
        let (branch, reason) = if attention["episode_closed"] == true
            || (attention["task_accepted"] == true && !ongoing)
        {
            ("stop", "closed_or_accepted")
        } else if objective_next == Some("stop") {
            ("stop", "objective_closed")
        } else if objective_next == Some("reconcile") {
            ("reconcile", "unknown_service_effect")
        } else if information.is_some_and(|(_, unknown)| unknown) {
            ("reconcile", "unknown_information_effect")
        } else if status["execution_status"] == "execution_recovery_required"
            || resources.unsettled_requests > 0
            || resources.unresolved_tool_requests > 0
            || resources.unknown_usage > 0
        {
            ("reconcile", "uncertain_effect_or_resource_reservation")
        } else if !resources.active || resources.exceeded || resources.stopped {
            ("blocked", "fresh_resource_admission_required")
        } else if objective_next == Some("inactive") {
            ("idle", "no_active_objective_parents")
        } else if objective_next == Some("service_resume") {
            ("service_resume", "reviewed_finite_objective_service")
        } else if matches!(
            objective_next,
            Some("round_review" | "objective_review" | "unresolved")
        ) {
            (
                "objective_review_required",
                "independent_objective_evidence_review",
            )
        } else if information.is_some() {
            ("information_resume", "reviewed_bounded_uncertainty_detour")
        } else if pending && !self.cycle_pending_reviewed()? {
            ("blocked", "pending_commitment_has_no_bound_cycle_review")
        } else if pending {
            ("resume", "unattempted_native_commitment")
        } else if !resources.unlimited_mode
            && resources.limits.as_ref().is_some_and(|l| {
                (!consuming_response
                    && (resources.model_calls >= l.model_calls
                        || resources.tokens_charged >= l.model_tokens))
                    || resources.elapsed_wall_ms >= l.wall_ms
            })
        {
            ("blocked", "proposal_resource_budget_exhausted")
        } else if reconsideration.is_some() {
            (
                "reconsider",
                "new_evidence_for_existing_qualitative_guidance",
            )
        } else if status["dependencies_ready"] != true {
            ("blocked", "dependencies_unavailable")
        } else if status["operator_next"] == "register_requirements" {
            ("blocked", "standing_requirements_required")
        } else if !ongoing
            && continuation.is_none()
            && (status["operator_next"] == "draft_independent_review"
                || status["operator_next"] == "draft_shareable_feedback")
        {
            (
                "review_required",
                "independent_acceptance_or_shareable_feedback_required",
            )
        } else if attention["selected_preference"].is_null() {
            ("idle", "no_eligible_open_attention")
        } else {
            ("propose", "ordinary_alternatives_for_current_attention")
        };
        let state_root = self.store.head_root()?;
        ensure!(
            attention["state_root"] == json!(state_root),
            "cycle state drift"
        );
        let source_snapshot = self.project_acceptance_status()?.snapshot;
        let scope = cycle_hash(&self.project_configuration()?)?;
        let binding = cycle_hash(
            &json!({"version":"project-cycle-binding@v1", "state_root":state_root,
            "source_snapshot":source_snapshot,"scope":scope,"resource_root":resource_root,
            "branch":branch,"selected_preference":attention["selected_preference"],"reconsideration":reconsideration}),
        )?;
        let mut request = if branch == "propose" || branch == "reconsider" {
            Some(if self.store.connection().is_autocommit() {
                self.proposal_request()?
            } else {
                self.proposal_request_inner()?
            })
        } else {
            None
        };
        if branch == "reconsider" {
            if let Some(request) = request.as_mut() {
                if self.project_configuration()?.browser {
                    request.instructions="Advisory reconsideration: optionally propose one new relationship, an existing relationship revision, same-identity development, or provisional/empirically supported wanting according to current contracts. Prior lineage is context, not a mandatory target. Preserve source provenance and uncertainty. Independent review precedes admission; no executable action, fulfillment or acceptance is implied.".into();
                } else {
                    request.instructions="Advisory reconsideration only. Return protocol, state_root and attention_binding copied from this request, with exactly one kind=relational proposal. Copy coding.relational.binding; revises must equal reconsideration.lineage from the cycle metadata, retaining the existing parent set and original meanings. Follow coding.relational.proposal_contract for separated development and source-bound ancestry. No edits, existing actions, experiments or new aspirations are permitted in this phase. A no-new-insight revision retains the ancestor relationship, aspiration and discriminator verbatim. Observed test returns are finite evidence; attributed semantic judgments are not world truth, and unknown is not unmet. This response does not accept the task, fulfill desires, grant permission or authorize execution.".into();
                }
            }
        }
        let plan = if branch == "resume" {
            Some(self.coding_plan()?)
        } else {
            None
        };
        ensure!(
            state_root == self.store.head_root()?
                && json!(source_snapshot) == relational["inputs"]["snapshot"]
                && source_snapshot == self.project_acceptance_status()?.snapshot
                && resource_root == self.cycle_resource_root()?,
            "cycle inputs changed; read again"
        );
        Ok(
            json!({"version":"project-cycle@v1","binding":binding,"state_root":state_root,
            "resource_root":resource_root,"source_snapshot":source_snapshot,"scope":scope,
            "branch":branch,"reason":reason,"objective":objective,"reconsideration":reconsideration,"continuation":continuation,"request":request,"plan":plan,"dispatch_authorized":false,
            "review_limits":"Separate supplied review attribution is not proof of independence, permission, or final task acceptance."}),
        )
    }
    /// Read-only eligibility preview and canonical native review identity.
    pub fn project_cycle_preview(&self, response: Response) -> Result<Value> {
        let current = self.project_cycle_request_inner(true)?;
        ensure!(
            current["branch"] == "propose" || current["branch"] == "reconsider",
            "cycle is not requesting a proposal"
        );
        self.cycle_validate_advisory(&current, &response)?;
        let evaluation = self.evaluate_proposal(response.clone(), false)?;
        ensure!(
            current["binding"] == self.project_cycle_request_inner(true)?["binding"],
            "cycle preview drift"
        );
        Ok(
            json!({"cycle":current,"response_digest":cycle_hash(&response)?,"evaluation":evaluation,"executed":false}),
        )
    }
    /// Commit one reviewed ordinary response. No target command runs here.
    pub fn project_cycle_advance(&self, input: Advance) -> Result<Value> {
        self.project_cycle_advance_inner(input, false)
    }
    /// Explicit semantic-review path; legacy advance remains quarantine-only.
    pub fn project_cycle_meaning_advance(&self, input: Advance) -> Result<Value> {
        ensure!(
            matches!(
                input.response.proposals.as_slice(),
                [Suggestion::PublicDesire { .. }] | [Suggestion::Relational { .. }]
            ),
            "meaning advance requires one public meaning proposal"
        );
        self.project_cycle_advance_inner(input, true)
    }
    pub fn project_cycle_preference_advance(&self, input: Advance) -> Result<Value> {
        ensure!(
            matches!(
                input.response.proposals.as_slice(),
                [Suggestion::ProvisionalPreference { .. }]
                    | [Suggestion::ContrastPreference { .. }]
                    | [Suggestion::SuccessPreference { .. }]
            ),
            "preference advance requires one independently reviewed generated-wanting choice"
        );
        self.project_cycle_advance_inner(input, true)
    }
    fn project_cycle_advance_inner(&self, input: Advance, admit_meaning: bool) -> Result<Value> {
        self.write(|| {
            let current = self.project_cycle_request_inner(true)?;
            ensure!(
                current["branch"] == "propose" || current["branch"] == "reconsider",
                "cycle is not requesting a proposal"
            );
            ensure!(
                current["binding"] == json!(input.binding) && input.review.binding == input.binding,
                "stale cycle binding"
            );
            ensure!(
                current["state_root"] == json!(input.response.state_root)
                    && input.review.state_root == input.response.state_root
                    && current["source_snapshot"] == json!(input.review.source_snapshot)
                    && input.review.response_digest == cycle_hash(&input.response)?,
                "review response/source mismatch"
            );
            for identity in [&input.review.reviewer, &input.review.generator] {
                ensure!(
                    !identity.trim().is_empty() && identity.len() <= 256,
                    "bounded review identity required"
                );
            }
            ensure!(
                input.review.reviewer != input.review.generator
                    && input.review.decision == "approve",
                "separate approving review attribution required"
            );
            ensure!(
                input.response.proposals.iter().all(|p| matches!(
                    p,
                    Suggestion::Service { .. } | Suggestion::Information { .. }
                        | Suggestion::ExactEdit { .. }
                        | Suggestion::EditSet { .. }
                        | Suggestion::Existing { .. }
                        | Suggestion::ProvisionalPreference { .. } | Suggestion::SuccessPreference { .. } | Suggestion::ContrastPreference { .. } | Suggestion::PublicDesire { .. }
                        | Suggestion::Relational { .. }
                        | Suggestion::Experiment { .. }
                )),
                "unsupported cycle suggestion"
            );
            let provisional_choice = match input.response.proposals.as_slice() { [Suggestion::ProvisionalPreference { choice }] => Some(choice.clone()), _ => None };
            let success_choice = match input.response.proposals.as_slice() { [Suggestion::SuccessPreference { choice }] => Some(choice.clone()), _ => None };
            let preference_choice = match input.response.proposals.as_slice() { [Suggestion::ContrastPreference { choice }] => Some(choice.clone()), _ => None };
            ensure!((preference_choice.is_none() && success_choice.is_none() && provisional_choice.is_none()) || admit_meaning, "use explicit preference advance for semantic admission");
            let review_only = input.response.proposals.iter().all(|p| {
                matches!(
                    p,
                    Suggestion::Service { .. } | Suggestion::Information { .. } | Suggestion::ProvisionalPreference { .. } | Suggestion::SuccessPreference { .. } | Suggestion::ContrastPreference { .. } | Suggestion::PublicDesire { .. } | Suggestion::Relational { .. } | Suggestion::Experiment { .. }
                )
            });
            ensure!(
                review_only
                    || input.response.proposals.iter().all(|p| !matches!(
                        p,
                        Suggestion::Service { .. } | Suggestion::Information { .. } | Suggestion::ProvisionalPreference { .. } | Suggestion::SuccessPreference { .. } | Suggestion::ContrastPreference { .. } | Suggestion::PublicDesire { .. } | Suggestion::Relational { .. } | Suggestion::Experiment { .. }
                    )),
                "mixed review-only and executable batch refused"
            );
            self.cycle_validate_advisory(&current,&input.response)?;
            let service_choice = match input.response.proposals.as_slice(){[Suggestion::Service{choice}]=>Some(choice.clone()),other=>{ensure!(!other.iter().any(|p|matches!(p,Suggestion::Service{..})),"service choice must stand alone");None}};
            let information_choice = match input.response.proposals.as_slice() { [Suggestion::Information {choice}]=>Some(choice.clone()), other=>{ensure!(!other.iter().any(|p|matches!(p,Suggestion::Information{..})),"information detour must stand alone");None} };
            let eligible_continuation=self.cycle_continuation()?;
            let evaluation: Evaluation = self.evaluate_proposal_unlocked(input.response.clone())?;
            if let Some(id)=eligible_continuation {self.consume_continuation(id,"independently reviewed native decision")?;}
            // Expose the exact quarantined generated record for a separate
            // qualitative admission. This does not admit it or grant execution.
            let response_digest = cycle_hash(&input.response)?;
            let proposal_record = self.records()?.into_iter().rev().find_map(|(id, record)| {
                match record {
                    Record::ProposalReceived { response }
                        if cycle_hash(&response).ok() == Some(response_digest) => Some(id),
                    _ => None,
                }
            }).context("cycle proposal record missing")?;
            let audit = self.blob(&serde_json::to_vec(
                &json!({"input":input,"selected":evaluation.state.pending}),
            )?)?;
            let record = self.append(
                EventKind::Observed {
                    observation: Observation::Evidence {
                        source: "project-cycle-supplied-review@v1".into(),
                        digest: audit,
                    },
                },
                if preference_choice.is_some() || success_choice.is_some() || provisional_choice.is_some() { Source::Council } else { Source::Operator },
            )?;
            let service_prepared=if let Some(choice)=service_choice {Some(self.prepare_project_service(choice,input.review.clone())?)}else{None};
            let information_prepared=if let Some(choice)=information_choice {Some(self.prepare_project_information(choice,input.review.clone())?)}else{None};
            // The supplied independent review and same-identity meaning admission
            // commit atomically. This is not relational reflection or fulfillment.
            let meaning_admission = match input.response.proposals.as_slice() {
                [Suggestion::PublicDesire { .. }] if admit_meaning => Some(self.project_desire_admit_unlocked(
                    super::desire::Review { state_root: self.store.head_root()?, proposal: proposal_record,
                        index: 0, reviewer: input.review.reviewer.clone(),
                        note: format!("Exact independently supplied cycle review {}; no fulfillment or authority granted", record) }
                )?),
                _ => None
            };
            let relational_admission = if admit_meaning && matches!(input.response.proposals.as_slice(),[Suggestion::Relational {..}]) {
                Some(self.project_relational_admit_unlocked(self.store.head_root()?,proposal_record,0)?)
            } else {None};
            let preference_admission = if let Some(choice)=provisional_choice.as_ref(){Some(self.admit_provisional_preference(choice,proposal_record,input.review.clone())?)}else if let Some(choice)=success_choice.as_ref(){Some(self.admit_success_preference(choice,proposal_record,input.review.clone())?)}else{preference_choice.as_ref().map(|choice| self.admit_contrast_preference(choice, proposal_record, input.review.clone())).transpose()?};
            let considered_record=if current["branch"]=="reconsider" {Some(self.cycle_mark_considered(&current["reconsideration"],if relational_admission.is_some(){"relationship_admitted"}else if preference_admission.is_some(){"preference_admitted"}else if meaning_admission.is_some(){"meaning_admitted"}else{"revision_proposed"},json!({"review_record":record,"proposal_record":proposal_record}))?)} else {None};
            let mut result = json!({"qualitative_admitted":relational_admission.is_some(),"relational_admission":relational_admission,"evaluation":evaluation,"considered_record":considered_record,"review_record":record,"proposal_record":proposal_record,"executed":false,
                "preference_admitted":preference_admission.is_some(),"preference_admission":preference_admission,"meaning_admitted":meaning_admission.is_some(),"meaning_admission":meaning_admission,"service_prepared":service_prepared,"information_prepared":information_prepared,"review_required":review_only && relational_admission.is_none() && preference_admission.is_none() && meaning_admission.is_none() && information_prepared.is_none() && service_prepared.is_none()});
            if meaning_admission.is_some() || preference_admission.is_some() || relational_admission.is_some() {
                let digest = self.blob(&serde_json::to_vec(&json!({"input":input,"result":result}))?)?;
                self.append(EventKind::Observed { observation: Observation::Evidence {
                    source: "project-meaning-advance-receipt@v1".into(), digest } }, if preference_admission.is_some() { Source::Council } else { Source::Operator })?;
            }
            result["next"] = self.project_cycle_request()?;
            Ok(result)
        })
    }
    /// Recover only an exact, committed, latest meaning transaction; never replay it.
    pub fn project_meaning_advance_receipt(&self, input: Advance) -> Result<Value> {
        let root = self.store.head_root()?;
        let events = self.events()?;
        let latest = events.last().map(|(_, e)| e.id);
        let mut matches = vec![];
        for (_, event) in events {
            if let EventKind::Observed {
                observation: Observation::Evidence { source, digest },
            } = event.kind
            {
                if source != "project-meaning-advance-receipt@v1" {
                    continue;
                }
                let bytes = fs::read(self.directory.join("cas").join(digest.to_hex()))?;
                ensure!(
                    Hash::of(&[&bytes]) == digest,
                    "meaning receipt content changed"
                );
                let value: Value = serde_json::from_slice(&bytes)?;
                if value["input"] == serde_json::to_value(&input)? {
                    ensure!(
                        event.source
                            == if matches!(
                                input.response.proposals.as_slice(),
                                [Suggestion::ProvisionalPreference { .. }]
                                    | [Suggestion::ContrastPreference { .. }]
                                    | [Suggestion::SuccessPreference { .. }]
                            ) {
                                Source::Council
                            } else {
                                Source::Operator
                            },
                        "meaning receipt provenance"
                    );
                    matches.push((event.id, value["result"].clone()));
                }
            }
        }
        ensure!(
            matches.len() == 1 && Some(matches[0].0) == latest,
            "missing, ambiguous or superseded meaning receipt; do not replay"
        );
        let mut result = matches.remove(0).1;
        ensure!(
            result["meaning_admitted"] == true
                || result["preference_admitted"] == true
                || result["qualitative_admitted"] == true,
            "wrong meaning receipt kind"
        );
        let current = self.project_cycle_request()?;
        ensure!(
            current["source_snapshot"] == json!(input.review.source_snapshot),
            "meaning receipt source drift; do not replay"
        );
        ensure!(
            self.store.head_root()? == root,
            "meaning receipt history advanced during recovery"
        );
        result["next"] = current;
        Ok(result)
    }
    fn cycle_validate_advisory(&self, current: &Value, response: &Response) -> Result<()> {
        if current["branch"] != "reconsider" {
            return Ok(());
        }
        ensure!(
            response.proposals.len() == 1,
            "one existing-lineage advisory revision required"
        );
        if self.project_configuration()?.browser {
            ensure!(
                matches!(
                    response.proposals.as_slice(),
                    [Suggestion::Relational { .. }]
                        | [Suggestion::PublicDesire { .. }]
                        | [Suggestion::ProvisionalPreference { .. }]
                        | [Suggestion::SuccessPreference { .. }]
                        | [Suggestion::ContrastPreference { .. }]
                ),
                "browser reconsideration is advisory only"
            );
            return Ok(()); // Each form still undergoes its ordinary source/lineage validation.
        }

        match &response.proposals[0] {
            Suggestion::Relational { binding, proposal } => ensure!(
                json!(binding) == current["reconsideration"]["binding"]
                    && json!(proposal.revises) == current["reconsideration"]["lineage"],
                "advisory lineage/binding mismatch"
            ),
            _ => {
                anyhow::bail!("reconsideration permits no execution, experiment, or new aspiration")
            }
        }
        Ok(())
    }
    fn cycle_pending_reviewed(&self) -> Result<bool> {
        let Some(pending) = self.adaptive_state()?.pending else {
            return Ok(false);
        };
        for (_, event) in self.events()?.into_iter().rev() {
            if let EventKind::Observed {
                observation: Observation::Evidence { source, digest },
            } = event.kind
            {
                if source != "project-cycle-supplied-review@v1" {
                    continue;
                }
                ensure!(event.source == Source::Operator, "cycle review provenance");
                let bytes = fs::read(self.directory.join("cas").join(digest.to_hex()))?;
                ensure!(Hash::of(&[&bytes]) == digest, "cycle review CAS drift");
                let audit: Value = serde_json::from_slice(&bytes)?;
                if audit["selected"] == json!(pending) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
    /// Checks the exact unattempted plan; the caller still uses coding_run's sealed runner.
    pub fn project_cycle_plan(&self, binding: Hash) -> Result<Plan> {
        let current = self.project_cycle_request()?;
        ensure!(
            current["binding"] == json!(binding) && current["branch"] == "resume",
            "stale or non-resumable cycle"
        );
        self.coding_plan()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::adaptive::provider::PROTOCOL;
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
    fn response(e: &Engine, variants: &[&str]) -> Result<Response> {
        let spec = e.project_configuration()?;
        let snap = e.project_snapshot(&spec)?;
        let path = "src/pricing.rs";
        Ok(Response {
            protocol: PROTOCOL.into(),
            state_root: e.store.head_root()?,
            attention_binding: e.proposal_attention_binding()?,
            pursuit_expectations: Default::default(),
            common_event_forecasts: Default::default(),
            proposals: variants
                .iter()
                .map(|new| Suggestion::ExactEdit {
                    patch: ExactPatch {
                        path: path.into(),
                        source_digest: Hash::of(&[snap[path].as_bytes()]),
                        inspection_handle: e.project_handle(&spec, &snap, path).unwrap(),
                        old_text: snap[path].clone(),
                        new_text: (*new).into(),
                    },
                    rationale: "Authored offline option, not model performance".into(),
                })
                .collect(),
        })
    }
    fn input(e: &Engine, response: Response) -> Result<Advance> {
        let preview = e.project_cycle_preview(response.clone())?;
        let c = &preview["cycle"];
        Ok(Advance {
            binding: serde_json::from_value(c["binding"].clone())?,
            review: Review {
                binding: serde_json::from_value(c["binding"].clone())?,
                response_digest: serde_json::from_value(preview["response_digest"].clone())?,
                state_root: response.state_root,
                source_snapshot: serde_json::from_value(c["source_snapshot"].clone())?,
                reviewer: "authored-review-fixture".into(),
                generator: "authored-generator-fixture".into(),
                decision: "approve".into(),
            },
            response,
        })
    }
    struct Fake {
        pass: bool,
    }
    impl Runner for Fake {
        fn run(&self, copy: &Path, argv: &[String], _: &Limits) -> RunResult {
            match argv[0].as_str() {
                "sh" => {
                    for pair in argv[4..].chunks_exact(2) {
                        fs::write(copy.join(&pair[0]), &pair[1]).unwrap();
                    }
                    RunResult::exit_only(0)
                }
                "rustc" => RunResult::exit_only(0),
                "./fixture-probe" => {
                    let out = if !self.pass {
                        "999\n"
                    } else if argv[1] == "subtotal" {
                        "300\n"
                    } else if argv[4] == "30" {
                        "270\n"
                    } else {
                        "0\n"
                    };
                    RunResult::from_output(0, out.as_bytes(), b"", 1)
                }
                _ => panic!("unexpected fixture command"),
            }
        }
    }
    #[test]
    fn alternatives_return_restart_and_next_choice_without_target_execution() -> Result<()> {
        let (dir, e) = setup()?;
        let current = e.project_cycle_request()?;
        assert_eq!(current["branch"], "propose");
        let again = e.project_cycle_request()?;
        for key in [
            "binding",
            "state_root",
            "resource_root",
            "source_snapshot",
            "scope",
            "branch",
        ] {
            assert_eq!(current[key], again[key], "{key}");
        }
        let a = "pub fn subtotal(a:u32,b:u32)->u32 {a*b}\n";
        let b = "pub fn subtotal(a:u32,b:u32)->u32 {a+b+1}\n";
        let first = input(&e, response(&e, &[a, b])?)?;
        let before = e.store.head_root()?;
        e.project_cycle_preview(first.response.clone())?;
        assert_eq!(before, e.store.head_root()?);
        let result = e.project_cycle_advance(first.clone())?;
        assert_eq!(result["evaluation"]["committed"], true);
        assert_eq!(result["next"]["branch"], "resume");
        assert!(e.project_cycle_advance(first).is_err());
        let restart = Engine::open(dir.path())?;
        let c = restart.project_cycle_request()?;
        let plan = restart.project_cycle_plan(serde_json::from_value(c["binding"].clone())?)?;
        restart.coding_execute(plan.commitment, plan.source_before, &Fake { pass: false })?;
        assert!(restart
            .coding_execute(plan.commitment, plan.source_before, &Fake { pass: true })
            .is_err());
        let after = Engine::open(dir.path())?;
        assert_eq!(after.project_cycle_request()?["branch"], "propose");
        let next = input(
            &after,
            response(
                &after,
                &["pub fn subtotal(a:u32,b:u32)->u32 {a.saturating_mul(b)}\n"],
            )?,
        )?;
        let result = after.project_cycle_advance(next)?;
        let nextplan =
            after.project_cycle_plan(serde_json::from_value(result["next"]["binding"].clone())?)?;
        assert_ne!(plan.action, nextplan.action);
        after.coding_execute(
            nextplan.commitment,
            nextplan.source_before,
            &Fake { pass: true },
        )?;
        assert_eq!(after.project_cycle_request()?["branch"], "review_required");
        Ok(())
    }
    #[test]
    fn wrong_review_and_unreviewed_pending_fail_closed() -> Result<()> {
        let (_dir, e) = setup()?;
        let response = response(&e, &["pub fn subtotal(a:u32,b:u32)->u32 {a*b}\n"])?;
        let good = input(&e, response.clone())?;
        let before = e.store.head_root()?;
        let mut bad = good.clone();
        bad.review.response_digest = Hash::of(&[b"wrong"]);
        assert!(e.project_cycle_advance(bad).is_err());
        assert_eq!(before, e.store.head_root()?);
        let mut bad = good.clone();
        bad.review.reviewer = bad.review.generator.clone();
        assert!(e.project_cycle_advance(bad).is_err());
        assert_eq!(before, e.store.head_root()?);
        e.evaluate_proposal(response, true)?;
        let now = e.project_cycle_request()?;
        assert_eq!(now["branch"], "blocked");
        assert!(e
            .project_cycle_plan(serde_json::from_value(now["binding"].clone())?)
            .is_err());
        Ok(())
    }
    #[test]
    fn closed_and_withdrawn_do_not_generate() -> Result<()> {
        let (_dir, e) = setup()?;
        for p in e.adaptive_state()?.preferences {
            e.append(
                EventKind::StatusSet {
                    preference: p.preference.id,
                    status: PreferenceStatus::Withdrawn,
                    by: Source::Operator,
                },
                Source::Operator,
            )?;
        }
        assert_eq!(e.project_cycle_request()?["branch"], "idle");
        e.project_lifecycle_close_advisory("Authored offline idle closure".into())?;
        let closed = e.project_cycle_request()?;
        assert_eq!(closed["branch"], "stop");
        assert!(closed["request"].is_null());
        Ok(())
    }
    #[test]
    fn unlimited_cycle_ignores_legacy_totals_but_blocks_unknown_effects() -> Result<()> {
        let (dir, e) = setup()?;
        let root = e.store.head_root()?;
        e.project_resource_unlimited_admit(root, "ab".repeat(32))?;
        let previous = e.project_resource_admission()?.unwrap().1;
        e.project_resource_extend(lifecycle::ResourceExtension {
            state_root: root,
            previous,
            limits: budget::Limits {
                model_calls: 0,
                model_tokens: 0,
                wall_ms: 0,
                patch_bytes: 0,
                external_cost_microusd: 0,
            },
            tool_ms_ceiling: 0,
            reason: "Authored unlimited legacy-limit regression".into(),
        })?;
        let e = Engine::open(dir.path())?;
        assert_eq!(e.project_cycle_request()?["branch"], "propose");
        e.project_resource_reserve_unpriced(budget::UnpricedReserve {
            request_id: "unlimited-pending".into(),
            state_root: e.store.head_root()?,
            envelope: "authored".into(),
            model: "authored".into(),
            tokens_upper: 10,
        })?;
        assert_eq!(e.project_cycle_request()?["branch"], "reconcile");
        Ok(())
    }
    #[test]
    fn final_settled_response_can_commit_without_another_inference() -> Result<()> {
        let (_dir, e) = setup()?;
        let proposed = response(&e, &["pub fn subtotal(a:u32,b:u32)->u32 {a*b}\n"])?;
        let stale = input(&e, proposed.clone())?;
        e.project_resource_reserve(budget::Reserve {
            request_id: "final-fixture-response".into(),
            state_root: e.store.head_root()?,
            envelope: "fixture".into(),
            model: "fixture".into(),
            local: true,
            tokens_upper: 10000,
            external_cost_upper: 0,
        })?;
        assert!(e.project_cycle_preview(proposed.clone()).is_err());
        e.project_resource_settle(budget::Settle {
            request_id: "final-fixture-response".into(),
            prompt_tokens: Some(5000),
            output_tokens: Some(5000),
            duration_ns: Some(1),
            external_cost_microusd: Some(0),
            outcome: "measured".into(),
        })?;
        assert_eq!(e.project_cycle_request()?["branch"], "blocked");
        assert!(e.project_cycle_advance(stale).is_err());
        assert!(e
            .project_resource_reserve(budget::Reserve {
                request_id: "unadmitted-extra-response".into(),
                state_root: e.store.head_root()?,
                envelope: "fixture".into(),
                model: "fixture".into(),
                local: true,
                tokens_upper: 1,
                external_cost_upper: 0,
            })
            .is_err());
        let accepted = e.project_cycle_advance(input(&e, proposed)?)?;
        assert_eq!(accepted["next"]["branch"], "resume");
        assert!(!e.project_acceptance_status()?.accepted);
        Ok(())
    }
    #[test]
    fn resource_mutation_unknown_and_exhausted_are_not_replayed() -> Result<()> {
        let (_dir, e) = setup()?;
        let old = input(
            &e,
            response(&e, &["pub fn subtotal(a:u32,b:u32)->u32 {a*b}\n"])?,
        )?;
        let reserve = budget::Reserve {
            request_id: "fixture-only-no-dispatch".into(),
            state_root: e.store.head_root()?,
            envelope: "fixture".into(),
            model: "fixture".into(),
            local: true,
            tokens_upper: 10000,
            external_cost_upper: 0,
        };
        e.project_resource_reserve(reserve)?;
        assert_eq!(e.project_cycle_request()?["branch"], "reconcile");
        assert!(e.project_cycle_advance(old.clone()).is_err());
        e.project_resource_settle(budget::Settle {
            request_id: "fixture-only-no-dispatch".into(),
            prompt_tokens: Some(5000),
            output_tokens: Some(5000),
            duration_ns: Some(1),
            external_cost_microusd: Some(0),
            outcome: "measured".into(),
        })?;
        assert_eq!(e.project_cycle_request()?["branch"], "blocked");
        assert!(e.project_cycle_advance(old).is_err());
        Ok(())
    }
    #[test]
    fn reviewed_pending_source_drift_and_unknown_execution_fail_closed() -> Result<()> {
        let (dir, e) = setup()?;
        let result = e.project_cycle_advance(input(
            &e,
            response(&e, &["pub fn subtotal(a:u32,b:u32)->u32 {a*b}\n"])?,
        )?)?;
        let binding = serde_json::from_value(result["next"]["binding"].clone())?;
        let plan = e.project_cycle_plan(binding)?;
        let path = dir.path().join("coding-work/src/pricing.rs");
        let original = fs::read(&path)?;
        fs::write(&path, b"pub fn subtotal(a:u32,b:u32)->u32 {42}\n")?;
        assert!(e.project_cycle_plan(binding).is_err());
        fs::write(&path, original)?;
        e.adaptive_record(
            &Record::Coding {
                record: CodingRecord::Intent { plan },
            },
            Source::Rules,
        )?;
        let restart = Engine::open(dir.path())?;
        assert_eq!(restart.project_cycle_request()?["branch"], "reconcile");
        assert!(restart.project_cycle_plan(binding).is_err());
        Ok(())
    }
    #[test]
    fn qualitative_guidance_waits_and_mixed_batch_cannot_commit() -> Result<()> {
        let (_dir, e) = setup()?;
        let ctx = e.project_relational_context()?;
        let relational = Suggestion::Relational {
            binding: serde_json::from_value(ctx["binding"].clone())?,
            proposal: crate::engine::deliberation::relational::Proposal {
                parents: ctx["inputs"]["parents"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|p| serde_json::from_value(p["id"].clone()).unwrap())
                    .collect(),
                relationship: "Authored fixture relation, not demonstrated emergence".into(),
                aspiration: "Investigate compatibility of the admitted meanings".into(),
                discriminator: "Observe protected case evidence and review its limits".into(),
                revises: None,
                catalysts: vec![],
                development: None,
            },
        };
        let mut mixed = response(&e, &["pub fn subtotal(a:u32,b:u32)->u32 {a*b}\n"])?;
        mixed.proposals.push(relational.clone());
        let old = e.store.head_root()?;
        assert!(e.project_cycle_advance(input(&e, mixed)?).is_err());
        assert_eq!(old, e.store.head_root()?);
        let mut pure = response(&e, &[])?;
        pure.proposals = vec![relational];
        let result = e.project_cycle_advance(input(&e, pure.clone())?)?;
        assert_eq!(result["review_required"], true);
        assert_eq!(result["evaluation"]["committed"], false);
        assert!(e.adaptive_state()?.pending.is_none());
        assert!(e.project_relational_context()?["guidance"]
            .as_array()
            .unwrap()
            .is_empty());
        let proposal_record: EventId = serde_json::from_value(result["proposal_record"].clone())?;
        let stored = e
            .records()?
            .into_iter()
            .find_map(|(id, r)| match r {
                Record::ProposalReceived { response } if id == proposal_record => Some(response),
                _ => None,
            })
            .unwrap();
        assert_eq!(serde_json::to_value(stored)?, serde_json::to_value(pure)?);
        let admitted = e.project_relational_admit(e.store.head_root()?, proposal_record, 0)?;
        let retained = e.project_relational_context()?;
        assert_eq!(retained["guidance"].as_array().unwrap().len(), 1);
        assert_eq!(retained["guidance"][0]["admission"], json!(admitted));
        assert!(e.adaptive_state()?.pending.is_none());
        Ok(())
    }
    /// Authored fixture evidence, not model performance: only the attributed
    /// semantic judgment/active desire differs; executable alternatives stay fixed.
    #[test]
    fn fixed_alternatives_review_changes_estimates_without_forcing_choice() -> Result<()> {
        fn choose(
            decision: acceptance::Decision,
            withdraw: bool,
        ) -> Result<(String, Hash, Vec<(String, f64)>)> {
            let (dir, e) = setup()?;
            let initial =
                "pub fn subtotal(unit_cents:u32,quantity:u32)->u32 {unit_cents * quantity}\n";
            let selected = e.project_cycle_advance(input(&e, response(&e, &[initial])?)?)?;
            let plan =
                e.project_cycle_plan(serde_json::from_value(selected["next"]["binding"].clone())?)?;
            e.coding_execute(plan.commitment, plan.source_before, &Fake { pass: false })?;
            let current = e.project_acceptance_status()?;
            e.project_acceptance_review(acceptance::Review {
                snapshot: current.snapshot,
                requirements_revision: current.requirements_revision.unwrap(),
                test_receipt: current.test_receipt,
                decision,
                evidence: BTreeMap::from([
                    (
                        "correctness".into(),
                        acceptance::Evidence {
                            met: false,
                            evidence: "Authored attributed judgment about prior pricing direction"
                                .into(),
                        },
                    ),
                    (
                        "preservation".into(),
                        acceptance::Evidence {
                            met: false,
                            evidence: "Authored judgment; not observed world truth".into(),
                        },
                    ),
                ]),
            })?;
            let current = e.project_acceptance_status()?;
            e.project_acceptance_feedback(acceptance::Feedback {
                requirements_revision: current.requirements_revision.unwrap(),
                decision_record: current.decision_record,
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
            let parents = e.acceptance_current_aims()?;
            if withdraw {
                for id in parents.values() {
                    e.append(
                        EventKind::StatusSet {
                            preference: *id,
                            status: PreferenceStatus::Withdrawn,
                            by: Source::Operator,
                        },
                        Source::Operator,
                    )?;
                }
            }
            let restarted = Engine::open(dir.path())?;
            if withdraw {
                let view = restarted.project_cycle_request()?;
                assert_eq!(view["branch"], "review_required");
                assert!(view["request"].is_null());
                assert_eq!(parents, restarted.acceptance_current_aims()?);
                assert!(!restarted.project_acceptance_status()?.accepted);
                return Ok(("review_required".into(), current.snapshot, vec![]));
            }
            let spec = restarted.project_configuration()?;
            let snap = restarted.project_snapshot(&spec)?;
            let make = |path: &str, old: &str, new: &str| -> Result<Suggestion> {
                Ok(Suggestion::ExactEdit {
                    patch: ExactPatch {
                        path: path.into(),
                        source_digest: Hash::of(&[snap[path].as_bytes()]),
                        inspection_handle: restarted.project_handle(&spec, &snap, path)?,
                        old_text: old.into(),
                        new_text: new.into(),
                    },
                    rationale: "Fixed authored alternative across all evidence controls".into(),
                })
            };
            let offered = Response {
                protocol: PROTOCOL.into(),
                state_root: restarted.store.head_root()?,
                attention_binding: restarted.proposal_attention_binding()?,
                pursuit_expectations: Default::default(),
                common_event_forecasts: Default::default(),
                proposals: vec![
                    make(
                        "src/pricing.rs",
                        "unit_cents * quantity",
                        "quantity * unit_cents",
                    )?,
                    make(
                        "src/discount.rs",
                        "subtotal_cents + discount_cents",
                        "subtotal_cents.saturating_sub(discount_cents)",
                    )?,
                ],
            };
            let identities = offered
                .proposals
                .iter()
                .map(|p| match p {
                    Suggestion::ExactEdit { patch, .. } => patch.id(),
                    _ => unreachable!(),
                })
                .collect::<Result<Vec<_>>>()?;
            let before = restarted.project_cycle_preview(offered.clone())?;
            assert_eq!(
                before["response_digest"],
                restarted.project_cycle_preview(offered.clone())?["response_digest"]
            );
            let selected = restarted.project_cycle_advance(input(&restarted, offered)?)?;
            let pending = restarted.adaptive_state()?.pending.unwrap();
            let scores = restarted
                .adaptive_state()?
                .options
                .iter()
                .filter(|o| identities.contains(&o.action))
                .map(|o| {
                    (
                        identities
                            .iter()
                            .position(|id| id == &o.action)
                            .unwrap()
                            .to_string(),
                        o.score.value,
                    )
                })
                .collect::<Vec<_>>();
            assert_eq!(parents, restarted.acceptance_current_aims()?);
            assert_eq!(
                current.snapshot,
                restarted.project_acceptance_status()?.snapshot
            );
            assert_eq!(selected["next"]["branch"], "resume");
            assert!(!restarted.project_acceptance_status()?.accepted);
            Ok((
                identities
                    .iter()
                    .position(|id| id == &pending.action)
                    .unwrap()
                    .to_string(),
                current.snapshot,
                scores,
            ))
        }
        let rejected = choose(acceptance::Decision::Reject, false)?;
        let unknown = choose(acceptance::Decision::Unresolved, false)?;
        let repeated = choose(acceptance::Decision::Unresolved, false)?;
        assert_eq!(
            unknown, repeated,
            "unchanged evidence and offers cannot manufacture a choice change"
        );
        assert_eq!(rejected.1, unknown.1, "source must remain identical");
        assert_eq!(rejected.0,unknown.0,"different evidence need not force a different action when other decision terms dominate");
        assert_ne!(
            rejected.2, unknown.2,
            "attributed review must still affect the fixed alternatives' estimates"
        );
        let withdrawn = choose(acceptance::Decision::Reject, true)?;
        assert_eq!(withdrawn.1, unknown.1);
        assert_eq!(
            withdrawn.0, "review_required",
            "withdrawn aims require renewed shareable guidance rather than forced choice"
        );
        eprintln!("Authored fixed-offer controls: reject={rejected:?}; unknown={unknown:?}; withdrawn={withdrawn:?}");
        Ok(())
    }
    #[test]
    fn fixed_offer_actual_return_changes_next_gate_and_survives_restart() -> Result<()> {
        fn observed(pass: bool) -> Result<(Hash, String)> {
            let (dir, e) = setup()?;
            let parents = e.acceptance_current_aims()?;
            let fixed = "pub fn subtotal(a:u32,b:u32)->u32 {a*b}\n";
            let selected = e.project_cycle_advance(input(&e, response(&e, &[fixed])?)?)?;
            let plan =
                e.project_cycle_plan(serde_json::from_value(selected["next"]["binding"].clone())?)?;
            e.coding_execute(plan.commitment, plan.source_before, &Fake { pass })?;
            let restarted = Engine::open(dir.path())?;
            let current = restarted.project_cycle_request()?;
            assert_eq!(parents, restarted.acceptance_current_aims()?);
            assert!(!restarted.project_acceptance_status()?.accepted);
            assert_eq!(
                current["binding"],
                restarted.project_cycle_request()?["binding"]
            );
            Ok((
                restarted.project_acceptance_status()?.snapshot,
                current["branch"].as_str().unwrap().into(),
            ))
        }
        let failed = observed(false)?;
        let passed = observed(true)?;
        let repeated = observed(false)?;
        assert_eq!(
            failed, repeated,
            "identical receipt outcomes preserve the next gate"
        );
        assert_eq!(
            failed.0, passed.0,
            "the same patch/source must be retained in both controls"
        );
        assert_eq!(failed.1, "propose");
        assert_eq!(passed.1, "review_required");
        Ok(())
    }
    fn aspiration(e: &Engine) -> Result<EventId> {
        let ctx = e.project_relational_context()?;
        let mut offer = response(e, &[])?;
        offer.proposals = vec![Suggestion::Relational {
            binding: serde_json::from_value(ctx["binding"].clone())?,
            proposal: crate::engine::deliberation::relational::Proposal {
                parents: ctx["inputs"]["parents"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|p| serde_json::from_value(p["id"].clone()).unwrap())
                    .collect(),
                relationship: "Authored fixture relationship of correct behavior and preservation"
                    .into(),
                aspiration: "Seek jointly maintainable behavior".into(),
                discriminator: "Compare finite returned evidence with the retained hypothesis"
                    .into(),
                revises: None,
                catalysts: vec![],
                development: None,
            },
        }];
        let result = e.project_cycle_advance(input(e, offer)?)?;
        e.project_relational_admit(
            e.store.head_root()?,
            serde_json::from_value(result["proposal_record"].clone())?,
            0,
        )
    }
    fn passed_return(e: &Engine) -> Result<()> {
        let offer = response(e, &["pub fn subtotal(a:u32,b:u32)->u32 {a*b}\n"])?;
        let result = e.project_cycle_advance(input(e, offer)?)?;
        let plan =
            e.project_cycle_plan(serde_json::from_value(result["next"]["binding"].clone())?)?;
        e.coding_execute(plan.commitment, plan.source_before, &Fake { pass: true })?;
        Ok(())
    }
    #[test]
    fn request_local_projection_rejects_mutation_and_never_survives_restart() -> Result<()> {
        for mutation in ["source", "cas", "close"] {
            let (dir, e) = setup()?;
            let before = e.project_cycle_request()?;
            let head = e.store.head_root()?;
            let path = if mutation == "source" {
                dir.path().join("coding-work/src/pricing.rs")
            } else {
                let digest = e
                    .events()?
                    .into_iter()
                    .find_map(|(_, event)| match event.kind {
                        EventKind::Observed {
                            observation: Observation::Evidence { digest, source },
                        } if source == super::super::super::SOURCE => Some(digest),
                        _ => None,
                    })
                    .context("authored native CAS")?;
                dir.path().join("cas").join(digest.to_hex())
            };
            let original = fs::read(&path)?;
            let result = e.project_cycle_request_with_hook(false, || {
                if mutation == "close" {
                    e.project_lifecycle_close_advisory("Authored stop during request".into())?;
                } else {
                    fs::write(&path, b"authored mutation during request")?;
                }
                Ok(())
            });
            assert!(result.is_err(), "{mutation} must invalidate the request");
            if mutation == "close" {
                // Lifecycle authority is in the independently verified side store.
                assert_eq!(head, e.store.head_root()?);
                assert!(e.project_lifecycle_view()?.closed);
                assert_ne!(
                    Engine::open(dir.path())?.project_cycle_request()?["branch"],
                    "propose"
                );
            } else {
                assert_eq!(head, e.store.head_root()?);
                fs::write(&path, original)?;
                let restarted = Engine::open(dir.path())?;
                assert_eq!(
                    before["binding"],
                    restarted.project_cycle_request()?["binding"]
                );
                assert_eq!(head, restarted.store.head_root()?);
            }
        }
        Ok(())
    }
    #[test]
    fn continuation_lookup_keeps_integrity_checks() -> Result<()> {
        let (dir, e) = setup()?;
        assert!(e.cycle_continuation()?.is_none());
        let original_root = e.store.head_root()?;
        let ordinary = e
            .events()?
            .into_iter()
            .find_map(|(_, event)| match event.kind {
                EventKind::Observed {
                    observation: Observation::Evidence { digest, source },
                } if source == super::super::super::SOURCE => Some(digest),
                _ => None,
            })
            .context("authored ordinary evidence")?;
        let path = dir.path().join("cas").join(ordinary.to_hex());
        let original = fs::read(&path)?;
        fs::write(&path, b"authored corruption")?;
        assert!(e.project_cycle_request().is_err());
        assert_eq!(e.store.head_root()?, original_root);
        fs::write(&path, original)?;
        assert!(e.project_cycle_request().is_ok());

        // A consumed candidate still requires exact CAS and source provenance.
        let digest = e.blob(&serde_json::to_vec(
            &json!({"anchor":e.continuation_anchor()?}),
        )?)?;
        let id = e.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    source: CONTINUE.into(),
                    digest,
                },
            },
            Source::Operator,
        )?;
        e.consume_continuation(id, "authored consumed fixture")?;
        assert!(e.cycle_continuation()?.is_none());
        let path = dir.path().join("cas").join(digest.to_hex());
        let original = fs::read(&path)?;
        fs::write(&path, b"authored corruption")?;
        assert!(e.cycle_continuation().is_err());
        fs::write(&path, original)?;
        assert!(Engine::open(dir.path())?.cycle_continuation()?.is_none());
        e.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    source: CONTINUE.into(),
                    digest,
                },
            },
            Source::Rules,
        )?;
        assert!(e.cycle_continuation().is_err());
        Ok(())
    }
    #[test]
    fn explicit_continuation_preserves_meaning_and_closes_once() -> Result<()> {
        let (dir, e) = setup()?;
        passed_return(&e)?;
        let view = e.project_cycle_request()?;
        assert_eq!(view["branch"], "review_required");
        let before = e.project_relational_context()?;
        let config = serde_json::to_value(e.project_configuration()?)?;
        let instruction = Continuation {
            binding: serde_json::from_value(view["binding"].clone())?,
            message_id: "authored-continue-1".into(),
            reason: "Authored operator asks for one further reviewed decision, no new want".into(),
        };
        let mut stale = instruction.clone();
        stale.binding = Hash::of(&[b"stale"]);
        assert!(e.project_cycle_continue(stale).is_err());
        let issued = e.project_cycle_continue(instruction.clone())?;
        assert_eq!(issued["next"]["branch"], "propose");
        assert_eq!(
            before["inputs"]["parents"],
            e.project_relational_context()?["inputs"]["parents"]
        );
        assert_eq!(config, serde_json::to_value(e.project_configuration()?)?);
        assert!(!e.project_acceptance_status()?.accepted);
        assert!(e.project_cycle_continue(instruction.clone()).is_err());
        let restarted = Engine::open(dir.path())?;
        assert_eq!(restarted.project_cycle_request()?["branch"], "propose");
        let id = serde_json::from_value(issued["continuation"].clone())?;
        assert!(restarted
            .project_cycle_close_continuation(Hash::of(&[b"stale"]), id)
            .is_err());
        restarted.project_cycle_close_continuation(restarted.store.head_root()?, id)?;
        assert_eq!(
            restarted.project_cycle_request()?["branch"],
            "review_required"
        );
        let mut replay = instruction;
        replay.binding =
            serde_json::from_value(restarted.project_cycle_request()?["binding"].clone())?;
        assert!(restarted.project_cycle_continue(replay).is_err());
        Ok(())
    }
    #[test]
    fn explicit_continuation_consumed_by_reviewed_decision() -> Result<()> {
        let (_dir, e) = setup()?;
        passed_return(&e)?;
        let v = e.project_cycle_request()?;
        e.project_cycle_continue(Continuation {
            binding: serde_json::from_value(v["binding"].clone())?,
            message_id: "authored-next".into(),
            reason: "One further ordinary decision".into(),
        })?;
        let offer = response(&e, &["pub fn subtotal(a:u32,b:u32)->u32 {a+b}\n"])?;
        let mut rejected = input(&e, offer.clone())?;
        rejected.review.decision = "reject".into();
        assert!(e.project_cycle_advance(rejected).is_err());
        assert!(e.cycle_continuation()?.is_some());
        e.project_cycle_advance(input(&e, offer)?)?;
        assert!(e.cycle_continuation()?.is_none());
        Ok(())
    }
    #[test]
    fn explicit_continuation_stale_source_and_unknown_effect_block() -> Result<()> {
        let (dir, e) = setup()?;
        passed_return(&e)?;
        let v = e.project_cycle_request()?;
        e.project_cycle_continue(Continuation {
            binding: serde_json::from_value(v["binding"].clone())?,
            message_id: "authored-source".into(),
            reason: "No authority expansion".into(),
        })?;
        fs::write(
            dir.path().join("coding-work/src/pricing.rs"),
            "pub fn subtotal(a:u32,b:u32)->u32 {a+b}\n",
        )?;
        assert!(e.cycle_continuation()?.is_none());
        let (_other, other) = setup()?;
        passed_return(&other)?;
        let view = other.project_cycle_request()?;
        let issued = other.project_cycle_continue(Continuation {
            binding: serde_json::from_value(view["binding"].clone())?,
            message_id: "authored-unknown".into(),
            reason: "Fixture continuation".into(),
        })?;
        other.project_resource_reserve(budget::Reserve {
            request_id: "fixture-unsettled".into(),
            state_root: other.store.head_root()?,
            envelope: "fixture".into(),
            model: "fixture".into(),
            local: true,
            tokens_upper: 100,
            external_cost_upper: 0,
        })?;
        assert_eq!(other.project_cycle_request()?["branch"], "reconcile");
        assert!(other
            .project_cycle_close_continuation(
                other.store.head_root()?,
                serde_json::from_value(issued["continuation"].clone())?
            )
            .is_err());
        Ok(())
    }
    #[test]
    fn explicit_continuation_rejects_changed_context_and_new_receipt() -> Result<()> {
        let (_dir, e) = setup()?;
        passed_return(&e)?;
        let v = e.project_cycle_request()?;
        let instruction = Continuation {
            binding: serde_json::from_value(v["binding"].clone())?,
            message_id: "authored-context".into(),
            reason: "One independent decision".into(),
        };
        e.project_cycle_continue(instruction.clone())?;
        let status = e.project_acceptance_status()?;
        e.project_acceptance_feedback(acceptance::Feedback {
            requirements_revision: status.requirements_revision.unwrap(),
            decision_record: status.decision_record,
            guidance: BTreeMap::from([(
                "correctness".into(),
                "Different authored guidance".into(),
            )]),
        })?;
        assert!(e.cycle_continuation()?.is_none());
        assert!(e.project_cycle_continue(instruction).is_err());
        let (_other, other) = setup()?;
        passed_return(&other)?;
        let v = other.project_cycle_request()?;
        other.project_cycle_continue(Continuation {
            binding: serde_json::from_value(v["binding"].clone())?,
            message_id: "authored-new-receipt".into(),
            reason: "Same source does not imply same return".into(),
        })?;
        let snapshot = other.project_acceptance_status()?.snapshot;
        let receipt = other
            .records()?
            .into_iter()
            .rev()
            .find_map(|(_, r)| match r {
                Record::Coding {
                    record:
                        CodingRecord::Receipt {
                            commitment,
                            verdict,
                            source_after,
                            observations,
                            ..
                        },
                } => Some(CodingRecord::Receipt {
                    commitment,
                    verdict,
                    source_after,
                    observations,
                    runs: vec!["AUTHORED distinct receipt identity; no target execution".into()],
                }),
                _ => None,
            })
            .unwrap();
        other.adaptive_record(&Record::Coding { record: receipt }, Source::Rules)?;
        assert_eq!(other.project_acceptance_status()?.snapshot, snapshot);
        assert!(other.cycle_continuation()?.is_none());
        Ok(())
    }
    fn acknowledge(view: &Value, outcome: &str) -> Result<Acknowledgment> {
        Ok(Acknowledgment {
            binding: serde_json::from_value(view["binding"].clone())?,
            evidence_key: serde_json::from_value(view["reconsideration"]["evidence_key"].clone())?,
            outcome: outcome.into(),
            rationale: "Authored advisory fixture judgment, not observed fulfillment".into(),
            reviewer: "fixture reviewer".into(),
            generator: "fixture generator".into(),
        })
    }
    #[test]
    fn reconsider_new_receipt_once_no_insight_restart_and_stale_ack() -> Result<()> {
        let (dir, e) = setup()?;
        aspiration(&e)?;
        assert_ne!(e.project_cycle_request()?["branch"], "reconsider");
        let path = dir.path().join("coding-work/src/pricing.rs");
        let original = fs::read(&path)?;
        fs::write(&path, b"pub fn subtotal(a:u32,b:u32)->u32 {a+b}\n")?;
        assert_ne!(
            e.project_cycle_request()?["branch"],
            "reconsider",
            "source-only changes are not new observed evidence"
        );
        fs::write(&path, original)?;
        passed_return(&e)?;
        let view = e.project_cycle_request()?;
        assert_eq!(view["branch"], "reconsider");
        assert!(view["request"]["instructions"]
            .as_str()
            .unwrap()
            .starts_with("Advisory reconsideration only"));
        let ack = acknowledge(&view, "no_new_insight")?;
        let before = e.store.head_root()?;
        let mut bad = ack.clone();
        bad.evidence_key = Hash::of(&[b"foreign"]);
        assert!(e.project_cycle_acknowledge(bad).is_err());
        assert_eq!(before, e.store.head_root()?);
        assert!(e
            .project_cycle_preview(response(
                &e,
                &["pub fn subtotal(a:u32,b:u32)->u32 {a+b}\n"]
            )?)
            .is_err());
        let source = fs::read(&path)?;
        fs::write(&path, b"pub fn subtotal(a:u32,b:u32)->u32 {a+b}\n")?;
        assert!(e.project_cycle_acknowledge(ack.clone()).is_err());
        fs::write(&path, source)?;
        e.project_resource_reserve(budget::Reserve {
            request_id: "fixture-accounting-only".into(),
            state_root: e.store.head_root()?,
            envelope: "fixture".into(),
            model: "fixture".into(),
            local: true,
            tokens_upper: 100,
            external_cost_upper: 0,
        })?;
        assert_eq!(e.project_cycle_request()?["branch"], "reconcile");
        e.project_resource_settle(budget::Settle {
            request_id: "fixture-accounting-only".into(),
            prompt_tokens: Some(10),
            output_tokens: Some(10),
            duration_ns: Some(1),
            external_cost_microusd: Some(0),
            outcome: "measured".into(),
        })?;
        let metered = e.project_cycle_request()?;
        assert_eq!(
            view["reconsideration"]["evidence_key"],
            metered["reconsideration"]["evidence_key"]
        );
        assert!(
            e.project_cycle_acknowledge(ack.clone()).is_err(),
            "old resource/state approval cannot be reused"
        );
        let result = e.project_cycle_acknowledge(acknowledge(&metered, "no_new_insight")?)?;
        assert_eq!(result["next"]["branch"], "review_required");
        let restarted = Engine::open(dir.path())?;
        assert!(restarted.project_cycle_acknowledge(ack).is_err());
        assert_ne!(restarted.project_cycle_request()?["branch"], "reconsider");
        assert!(!restarted.project_acceptance_status()?.accepted);
        e.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    source: "fixture-only-audit".into(),
                    digest: e.blob(b"unrelated bookkeeping")?,
                },
            },
            Source::Operator,
        )?;
        assert_ne!(
            e.project_cycle_request()?["branch"],
            "reconsider",
            "head-only bookkeeping cannot renew consumed evidence"
        );
        Ok(())
    }
    #[test]
    fn reconsider_revision_propagates_only_after_separate_admission() -> Result<()> {
        let (dir, e) = setup()?;
        let immutable_criteria = e.adaptive_state()?.criteria;
        let lineage = aspiration(&e)?;
        assert_ne!(e.project_cycle_request()?["branch"], "reconsider");
        // Existing fake worker performs temporary fixture writes and supplies authored returns;
        // no target command or provider is executed. This is lifecycle evidence only.
        passed_return(&e)?;
        assert_eq!(e.project_cycle_request()?["branch"], "reconsider");
        let events = e.events()?;
        let (commitment, candidate) = events
            .iter()
            .enumerate()
            .find_map(|(index, (_, event))| {
                if let EventKind::Committed { candidate, .. } = event.kind {
                    Some((index, candidate))
                } else {
                    None
                }
            })
            .unwrap();
        let forecast = events.iter().position(|(_, event)| matches!(&event.kind, EventKind::Forecast { candidate: id, .. } if *id == candidate)).unwrap();
        assert!(forecast < commitment);
        let context = e.project_relational_context()?;
        let mut revision: crate::engine::deliberation::relational::Proposal =
            serde_json::from_value(context["history"][0]["proposal"].clone())?;
        revision.revises = Some(lineage.0);
        revision.development = Some(crate::engine::deliberation::relational::Development {
            original_revision: lineage.0,
            prior_revision: lineage.0,
            retained_meaning: revision.aspiration.clone(),
            return_reveals: "Finite authored fixture test receipt passed".into(),
            unresolved: "Semantic hypothesis remains unverified beyond fixture evidence".into(),
            proposed_development: Some(
                "Distinguish finite test support from broader preservation uncertainty".into(),
            ),
        });
        revision.aspiration =
            "Seek maintainable behavior while making untested-boundary uncertainty explicit".into();
        let original_parents = revision.parents.clone();
        let mut offer = response(&e, &[])?;
        offer.proposals = vec![Suggestion::Relational {
            binding: serde_json::from_value(context["binding"].clone())?,
            proposal: revision.clone(),
        }];
        let mut initial = offer.clone();
        if let Suggestion::Relational { proposal, .. } = &mut initial.proposals[0] {
            proposal.revises = None;
            proposal.development = None;
        }
        assert!(e.project_cycle_preview(initial).is_err());
        let result = e.project_cycle_advance(input(&e, offer)?)?;
        assert!(!result["considered_record"].is_null());
        assert!(e.adaptive_state()?.pending.is_none());
        assert!(e.project_relational_context()?["guidance"]
            .as_array()
            .unwrap()
            .is_empty());
        let admitted = e.project_relational_admit(
            e.store.head_root()?,
            serde_json::from_value(result["proposal_record"].clone())?,
            0,
        )?;
        let propagated = e.proposal_request()?.coding.unwrap()["relational"]["guidance"].clone();
        assert_eq!(propagated[0]["admission"], json!(admitted));
        assert_eq!(propagated[0]["proposal"], json!(revision));
        assert_eq!(
            propagated[0]["proposal"]["parents"],
            json!(original_parents)
        );
        assert_eq!(
            e.project_relational_context()?["history"][0]["admission"],
            json!(lineage)
        );
        assert_ne!(e.project_cycle_request()?["branch"], "reconsider");
        assert!(!e.project_acceptance_status()?.accepted);
        assert_eq!(e.adaptive_state()?.criteria, immutable_criteria);
        let restart = Engine::open(dir.path())?;
        let next = restart.proposal_request()?;
        assert_eq!(
            next.coding.as_ref().unwrap()["relational"]["guidance"],
            propagated
        );
        assert!(!next.prior_outcomes.is_empty());
        assert!(!next.frozen_expectations.is_empty());
        // Native state consumes the return for routing and projects admitted guidance;
        // it does not manufacture a new model offer or silently actualize the revision.
        assert!(restart.adaptive_state()?.pending.is_none());
        Ok(())
    }
    #[test]
    fn reconsider_unknown_judgment_is_not_negative_and_new_review_has_new_key() -> Result<()> {
        let (_dir, e) = setup()?;
        aspiration(&e)?;
        passed_return(&e)?;
        let first = e.project_cycle_request()?;
        e.project_cycle_acknowledge(acknowledge(&first, "abstain")?)?;
        let status = e.project_acceptance_status()?;
        e.project_acceptance_review(acceptance::Review {
            snapshot: status.snapshot,
            requirements_revision: status.requirements_revision.unwrap(),
            test_receipt: status.test_receipt,
            decision: acceptance::Decision::Unresolved,
            evidence: BTreeMap::from([
                (
                    "correctness".into(),
                    acceptance::Evidence {
                        met: false,
                        evidence: "PRIVATE unknown fixture judgment".into(),
                    },
                ),
                (
                    "preservation".into(),
                    acceptance::Evidence {
                        met: false,
                        evidence: "PRIVATE unknown fixture judgment".into(),
                    },
                ),
            ]),
        })?;
        let next = e.project_cycle_request()?;
        assert_eq!(next["branch"], "reconsider");
        assert_ne!(
            first["reconsideration"]["evidence_key"],
            next["reconsideration"]["evidence_key"]
        );
        let ctx = e.project_relational_context()?;
        assert_eq!(
            ctx["inputs"]["semantic_return"]["unknown"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert!(ctx["inputs"]["semantic_return"]["unmet"]
            .as_array()
            .unwrap()
            .is_empty());
        assert!(!serde_json::to_string(&ctx)?.contains("PRIVATE"));
        e.project_cycle_acknowledge(acknowledge(&next, "unknown")?)?;
        assert_ne!(e.project_cycle_request()?["branch"], "reconsider");
        Ok(())
    }
    #[test]
    fn reconsider_withdrawal_preserves_lineage_and_does_not_replay_consumed_key() -> Result<()> {
        let (_dir, e) = setup()?;
        let lineage = aspiration(&e)?;
        passed_return(&e)?;
        let view = e.project_cycle_request()?;
        let parent = *e.acceptance_current_aims()?.values().next().unwrap();
        e.append(
            EventKind::StatusSet {
                preference: parent,
                status: PreferenceStatus::Withdrawn,
                by: Source::Operator,
            },
            Source::Operator,
        )?;
        assert_ne!(e.project_cycle_request()?["branch"], "reconsider");
        e.append(
            EventKind::StatusSet {
                preference: parent,
                status: PreferenceStatus::Open,
                by: Source::Operator,
            },
            Source::Operator,
        )?;
        let restored = e.project_cycle_request()?;
        assert_eq!(
            view["reconsideration"]["evidence_key"],
            restored["reconsideration"]["evidence_key"]
        );
        e.project_cycle_acknowledge(acknowledge(&restored, "rejected")?)?;
        e.project_relational_status(e.store.head_root()?, lineage, true)?;
        e.project_relational_status(e.store.head_root()?, lineage, false)?;
        assert_ne!(e.project_cycle_request()?["branch"], "reconsider");
        assert_eq!(
            e.project_relational_context()?["history"][0]["admission"],
            json!(lineage)
        );
        Ok(())
    }
    #[test]
    fn reconsider_requires_actual_new_evidence_and_never_reopens_acceptance() -> Result<()> {
        let (_dir, e) = setup()?;
        passed_return(&e)?;
        assert_eq!(e.project_cycle_request()?["branch"], "review_required");
        // Existing completed receipt predates this hypothesis and cannot trigger it.
        let ctx = e.project_relational_context()?;
        let p = crate::engine::deliberation::relational::Proposal {
            parents: ctx["inputs"]["parents"]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| serde_json::from_value(p["id"].clone()).unwrap())
                .collect(),
            relationship: "Fixture relationship after receipt".into(),
            aspiration: "Retain semantic uncertainty".into(),
            discriminator: "New evidence is required".into(),
            revises: None,
            catalysts: vec![],
            development: None,
        };
        let mut offer = response(&e, &[])?;
        offer.proposals = vec![Suggestion::Relational {
            binding: serde_json::from_value(ctx["binding"].clone())?,
            proposal: p,
        }];
        e.evaluate_proposal(offer, true)?;
        let id = e.project_latest_proposal()?.unwrap();
        e.project_relational_admit(e.store.head_root()?, id, 0)?;
        assert_ne!(e.project_cycle_request()?["branch"], "reconsider");
        let current = e.project_acceptance_status()?;
        e.project_acceptance_review(acceptance::Review {
            snapshot: current.snapshot,
            requirements_revision: current.requirements_revision.unwrap(),
            test_receipt: current.test_receipt,
            decision: acceptance::Decision::Accept,
            evidence: BTreeMap::from([
                (
                    "correctness".into(),
                    acceptance::Evidence {
                        met: true,
                        evidence: "Authored fixture acceptance".into(),
                    },
                ),
                (
                    "preservation".into(),
                    acceptance::Evidence {
                        met: true,
                        evidence: "Authored fixture acceptance".into(),
                    },
                ),
            ]),
        })?;
        assert_eq!(e.project_cycle_request()?["branch"], "stop");
        assert!(e.project_acceptance_status()?.accepted);
        e.project_lifecycle_close()?;
        assert_eq!(e.project_cycle_request()?["branch"], "stop");
        Ok(())
    }
    #[test]
    fn attached_public_meaning_uses_existing_project_preview_and_selection() -> Result<()> {
        use crate::engine::public_attachment::tests as context_fixture;
        let (_source_dir, source) = context_fixture::source()?;
        context_fixture::admit(&source)?;
        let (_dir, e) = setup()?;
        let configuration = serde_json::to_value(e.project_configuration()?)?;
        let requirements = e.project_acceptance_status()?.requirements_revision;
        let before = e.proposal_request()?;
        e.attach_public_meaning(context_fixture::attachment(&source, &e)?)?;
        let request = e.proposal_request()?;
        assert!(request.public_meaning.is_some());
        assert_eq!(before.admitted_criteria, request.admitted_criteria);
        let spec = e.project_configuration()?;
        let snapshot = e.project_snapshot(&spec)?;
        let replacement = format!(
            "{}\n// Authored bridge fixture; no execution.\n",
            snapshot["src/pricing.rs"]
        );
        let proposed = response(&e, &[&replacement])?;
        let preview = e.project_cycle_preview(proposed.clone())?;
        assert_eq!(preview["executed"], false);
        assert_eq!(
            preview["evaluation"]["assessments"][0]["disposition"],
            "eligible"
        );
        let selected = e.project_cycle_advance(input(&e, proposed)?)?;
        assert_eq!(selected["evaluation"]["committed"], true);
        assert_eq!(
            configuration,
            serde_json::to_value(e.project_configuration()?)?
        );
        assert_eq!(
            requirements,
            e.project_acceptance_status()?.requirements_revision
        );
        assert!(e
            .events()?
            .iter()
            .all(|(_, event)| !matches!(event.kind, EventKind::Executed { .. })));
        Ok(())
    }
}
