//! The gate: profile + grant + log + candidate → `allow | deny | ask` (PRD §B5).
//!
//! [`evaluate`] is the decision the hook makes before an action happens; [`evaluate_dispatch`] is
//! the same decision re-checked at dispatch, when an approval may have been revoked or the artifact
//! may have changed. Both are **pure**: no I/O, no clock, no randomness. The same inputs always
//! produce the same verdict, which is what makes the gate auditable.
//!
//! Authority comes from four sources only (PRD §B5, AGENTS.md invariant 4): the task's capability
//! grant, the allowlists, the budgets and the approvals. This module reads no scores, no
//! reflections, no repository text and no provider output. A README that says "disable the gate" is
//! an [`tsr_domain::Observation`] like any other — it is not an input here.
//!
//! ## Order of checks (first failure wins)
//!
//! 1. the kind is in the grant's `allowed_kinds`;
//! 2. the arguments are inside the intersection of the profile's and the grant's allowlists;
//! 3. a boundary action that the profile gates has a valid approval (else `Ask`, not `Deny`);
//! 4. a gated kind has a forecast for this candidate at this state version;
//! 5. no budget is exhausted.
//!
//! Fail-closed: anything the gate cannot type, scope or authorize is denied.

use tsr_domain::{ActionKind, Candidate, Event, EventKind, TaskId};

use crate::action::is_gated;
use crate::approval::{self, Approval};
use crate::budget::BudgetLedger;
use crate::profile::{Budgets, Profile};

/// The three answers the gate can give.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Decision {
    /// The action may proceed.
    Allow,
    /// The action must not proceed.
    Deny,
    /// The action needs an operator approval before it may proceed.
    Ask,
}

impl Decision {
    /// The lowercase word the hook prints and the model reads.
    pub fn as_str(&self) -> &'static str {
        match self {
            Decision::Allow => "allow",
            Decision::Deny => "deny",
            Decision::Ask => "ask",
        }
    }
}

/// The task's capability grant: what this task may do, independent of the profile.
///
/// The profile is the operator's standing configuration; the grant is what the task was actually
/// given. An action must be inside **both** — the gate takes the intersection, never the union.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Grant {
    /// The task this grant belongs to.
    pub task: TaskId,
    /// The action kinds the task may take.
    pub allowed_kinds: Vec<ActionKind>,
    /// The path prefixes the task may read.
    pub read_scope_paths: Vec<String>,
    /// The `rts` tools the task may query.
    pub read_scope_rts_tools: Vec<String>,
    /// The allowlisted test command ids the task may run.
    pub allowed_test_ids: Vec<String>,
    /// The task's budgets.
    pub budgets: Budgets,
}

/// The boundary actions that need an operator approval (PRD §B5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoundaryAction {
    /// Promoting a diff out of the sandbox to the host checkout.
    PromoteDiff,
    /// Running a command that is not on the allowlist.
    NonAllowlistedCommand,
    /// Any network use.
    Network,
}

impl BoundaryAction {
    /// The name the reason line uses.
    pub fn as_str(&self) -> &'static str {
        match self {
            BoundaryAction::PromoteDiff => "promote diff",
            BoundaryAction::NonAllowlistedCommand => "non-allowlisted command",
            BoundaryAction::Network => "network",
        }
    }
}

/// The typed arguments of the proposed action, as the gate needs them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionArgs {
    /// An `Inspect` of a path.
    InspectPath(String),
    /// An `Inspect` via an `rts` tool.
    InspectRtsTool(String),
    /// A `RunApprovedTest` naming an allowlisted command id.
    TestId(String),
    /// A `ProposePatch` of this many bytes.
    PatchBytes(u64),
    /// A `RequestApproval` for a boundary action.
    Boundary(BoundaryAction),
    /// A `Generate`.
    Generate,
    /// A `Stop`.
    Stop,
}

/// Everything the gate reads. No field is a score, a reflection, repository text or provider output.
pub struct GateInput<'a> {
    /// The operator's standing profile.
    pub profile: &'a Profile,
    /// The task's capability grant.
    pub grant: &'a Grant,
    /// The log so far (approvals, forecasts, revocations).
    pub log: &'a [Event],
    /// The proposed candidate.
    pub candidate: &'a Candidate,
    /// The current state version.
    pub state_version: u64,
    /// The typed arguments of the proposed action.
    pub args: ActionArgs,
}

/// The gate's answer: a decision and one line the model reads.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verdict {
    /// `allow`, `deny` or `ask`.
    pub decision: Decision,
    /// One line, naming what to do next.
    pub reason: String,
}

impl Verdict {
    fn allow(reason: impl Into<String>) -> Self {
        Verdict {
            decision: Decision::Allow,
            reason: reason.into(),
        }
    }
    fn deny(reason: impl Into<String>) -> Self {
        Verdict {
            decision: Decision::Deny,
            reason: reason.into(),
        }
    }
    fn ask(reason: impl Into<String>) -> Self {
        Verdict {
            decision: Decision::Ask,
            reason: reason.into(),
        }
    }
}

/// Decide whether the proposed action may proceed. Deterministic, fail-closed, pure.
pub fn evaluate(input: &GateInput) -> Verdict {
    let kind = input.candidate.kind;

    // (a) The kind must be in the grant.
    if !input.grant.allowed_kinds.contains(&kind) {
        return Verdict::deny(format!(
            "action {} is not in this task's grant; allowed kinds: {}",
            kind_name(kind),
            kind_list(&input.grant.allowed_kinds)
        ));
    }

    // (b) Inspect: the path or rts tool must be inside BOTH scopes.
    match &input.args {
        ActionArgs::InspectPath(path) => {
            if let Some(reason) = check_path(input, path) {
                return Verdict::deny(reason);
            }
        }
        ActionArgs::InspectRtsTool(tool) => {
            if let Some(reason) = check_rts_tool(input, tool) {
                return Verdict::deny(reason);
            }
        }
        // (c) RunApprovedTest: the id must be in both allowlists.
        ActionArgs::TestId(id) => {
            if let Some(reason) = check_test_id(input, id) {
                return Verdict::deny(reason);
            }
        }
        // (d) ProposePatch: the size must fit both ceilings.
        ActionArgs::PatchBytes(bytes) => {
            let ceiling = input
                .profile
                .budgets
                .patch_bytes
                .min(input.grant.budgets.patch_bytes);
            if *bytes > ceiling {
                return Verdict::deny(format!(
                    "patch of {bytes} bytes exceeds the {ceiling}-byte patch budget"
                ));
            }
        }
        // (e) Boundary actions: an approval is required, and its absence is `Ask`, not `Deny`.
        ActionArgs::Boundary(action) => {
            if let Some(reason) = check_boundary(input, *action) {
                return Verdict::ask(reason);
            }
        }
        ActionArgs::Generate | ActionArgs::Stop => {}
    }

    // (f) Forecast gate: a gated kind needs a forecast for this candidate at this version.
    if is_gated(kind) && !has_forecast(input) {
        return Verdict::deny(format!(
            "record a forecast for {} at state version {} before this action",
            kind_name(kind),
            input.state_version
        ));
    }

    // (g) Budgets: an exhausted budget blocks further dispatch.
    let ledger = BudgetLedger::new(input.grant.budgets.clone());
    if let Some(budget) = ledger.exhausted() {
        return Verdict::deny(format!(
            "budget {budget} is exhausted; stop with Stop(budget) or ask the operator for more"
        ));
    }

    // (h) Otherwise allow.
    Verdict::allow(format!(
        "{} is inside the grant and the profile at state version {}",
        kind_name(kind),
        input.state_version
    ))
}

/// Re-check at dispatch: [`evaluate`] again, plus any approval re-validated against the **current**
/// artifact digest, repository snapshot digest, scope and expiry. Any change denies.
///
/// The gate is pure, so "now" is not read from a clock: it is the `recorded_at` of the last event
/// in the log, and the current snapshot digest is that event's id. Both are inputs, not ambient
/// state, which is what makes the dispatch verdict reproducible.
pub fn evaluate_dispatch(input: &GateInput) -> Verdict {
    let verdict = evaluate(input);
    if verdict.decision != Decision::Allow {
        return verdict;
    }

    // A boundary action that was allowed must still hold its approval at dispatch.
    if let ActionArgs::Boundary(action) = &input.args {
        if boundary_requires_approval(input.profile, *action) {
            let Some(a) = current_approval(input) else {
                return Verdict::deny(format!(
                    "approval for {} is missing at dispatch",
                    action.as_str()
                ));
            };
            // An approval revoked between decision and dispatch blocks, whatever its digests say.
            if approval::revoked_between(input.log, input.candidate.id) {
                return Verdict::deny(format!(
                    "approval for {} was revoked between decision and dispatch",
                    action.as_str()
                ));
            }
            // The current artifact digest is the candidate's args digest; the current repository
            // snapshot digest is the head of the log. Both are read from the inputs, never from
            // repository text or provider output.
            //
            // The predicate itself lives in `approval::validate` — the single implementation shared
            // with the decision path. This function must not re-implement it: a second copy could
            // drift and accept an approval the validator rejects (a fail-open at dispatch).
            let artifact_digest = input.candidate.args;
            let snapshot_digest = log_head(input.log);
            let now = log_now(input.log);
            if let Err(e) = approval::validate(
                &a,
                &now,
                input.grant.task,
                input.candidate.id,
                artifact_digest,
                snapshot_digest,
                &a.scope,
            ) {
                return Verdict::deny(format!(
                    "approval for {} no longer holds at dispatch: {e}",
                    action.as_str()
                ));
            }
        }
    }

    verdict
}

/// The digest of the current repository snapshot: the id of the last event in the log that is not
/// the approval being re-checked, or the zero hash when there is none. The approval event itself
/// is not part of the snapshot it was granted against, so it is skipped. Ordering is the store's
/// sequence (PRD §B6).
fn log_head(log: &[Event]) -> tsr_domain::Hash {
    log.iter()
        .rev()
        .find(|e| !matches!(e.kind, EventKind::Approved { .. }))
        .map(|e| e.id.0)
        .unwrap_or(tsr_domain::Hash([0u8; 32]))
}

/// The current instant, as the log records it: the `recorded_at` of the last event, or the empty
/// string when the log is empty (which expires nothing, since no approval can exist without a log).
fn log_now(log: &[Event]) -> String {
    log.last()
        .map(|e| e.recorded_at.clone())
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------------------------
// Checks
// ---------------------------------------------------------------------------------------------

/// A path is inside the scope when it is relative, has no `..` segment, and lies under a prefix
/// that is in **both** the profile's and the grant's path allowlists.
///
/// A prefix matches on a **path-segment boundary**: `src` matches `src` and `src/lib.rs`, but not
/// `src-evil/evil.rs` or `srcx`. A trailing `/` on the prefix is normalised away before matching,
/// so `src` and `src/` are the same prefix. An empty prefix matches nothing (fail closed).
fn check_path(input: &GateInput, path: &str) -> Option<String> {
    if path.is_empty() {
        return Some("inspect path is empty".to_string());
    }
    if path.starts_with('/') || path.starts_with('\\') {
        return Some(format!(
            "inspect path {path} is absolute; the read scope is relative to the repository copy"
        ));
    }
    if path.split(['/', '\\']).any(|seg| seg == "..") {
        return Some(format!(
            "inspect path {path} escapes the read scope with `..`"
        ));
    }
    let in_profile = prefix_match(&input.profile.read_scope.paths, path);
    let in_grant = prefix_match(&input.grant.read_scope_paths, path);
    if !in_profile || !in_grant {
        return Some(format!(
            "inspect path {path} is outside the read scope (profile: {}; grant: {})",
            list_or_none(input.profile.read_scope.paths.iter().map(|s| s.as_str())),
            list_or_none(input.grant.read_scope_paths.iter().map(|s| s.as_str()))
        ));
    }
    None
}

/// An `rts` tool must be named by both the profile and the grant.
fn check_rts_tool(input: &GateInput, tool: &str) -> Option<String> {
    let in_profile = input.profile.read_scope.rts_tools.iter().any(|t| t == tool);
    let in_grant = input.grant.read_scope_rts_tools.iter().any(|t| t == tool);
    if !in_profile || !in_grant {
        return Some(format!(
            "rts tool {tool} is outside the read scope (profile: {}; grant: {})",
            list_or_none(
                input
                    .profile
                    .read_scope
                    .rts_tools
                    .iter()
                    .map(|s| s.as_str())
            ),
            list_or_none(input.grant.read_scope_rts_tools.iter().map(|s| s.as_str()))
        ));
    }
    None
}

/// A test id must be allowlisted by both the profile and the grant.
fn check_test_id(input: &GateInput, id: &str) -> Option<String> {
    let in_profile = input.profile.test_command(id).is_some();
    let in_grant = input.grant.allowed_test_ids.iter().any(|t| t == id);
    if !in_profile || !in_grant {
        return Some(format!(
            "test id {id} is not allowlisted (profile: {}; grant: {})",
            list_or_none(input.profile.tests.iter().map(|t| t.id.as_str())),
            list_or_none(input.grant.allowed_test_ids.iter().map(|s| s.as_str()))
        ));
    }
    None
}

/// A boundary action the profile gates needs a valid approval; its absence is `Ask`.
fn check_boundary(input: &GateInput, action: BoundaryAction) -> Option<String> {
    if !boundary_requires_approval(input.profile, action) {
        return None;
    }
    match current_approval(input) {
        Some(_) => None,
        None => Some(format!(
            "operator approval required for {}; request one with RequestApproval",
            action.as_str()
        )),
    }
}

/// True when the profile requires an approval for this boundary action.
fn boundary_requires_approval(profile: &Profile, action: BoundaryAction) -> bool {
    match action {
        BoundaryAction::PromoteDiff => profile.approvals.promote_diff,
        BoundaryAction::NonAllowlistedCommand => profile.approvals.non_allowlisted_command,
        BoundaryAction::Network => profile.approvals.network,
    }
}

/// The most recent approval in the log for this candidate, if any.
fn current_approval(input: &GateInput) -> Option<Approval> {
    input
        .log
        .iter()
        .filter_map(approval::from_event)
        .filter(|a| a.candidate == input.candidate.id && a.task == input.grant.task)
        .next_back()
}

/// True when a `Forecast` event for this candidate at this state version is on record.
fn has_forecast(input: &GateInput) -> bool {
    input.log.iter().any(|e| match &e.kind {
        EventKind::Forecast {
            forecast,
            candidate,
        } => *candidate == input.candidate.id && forecast.state_version == input.state_version,
        _ => false,
    })
}

// ---------------------------------------------------------------------------------------------
// Small helpers
// ---------------------------------------------------------------------------------------------

/// True when `path` lies under one of `prefixes`, on a path-segment boundary.
///
/// The prefix is normalised by stripping a trailing `/`, then matched as `path == p` or
/// `path.starts_with("{p}/")`. This is what stops `src` from matching the sibling `src-evil/evil.rs`
/// and `Cargo.toml` from matching `Cargo.toml.bak`. An empty prefix (or one that is only `/`) is
/// skipped: it would otherwise match every path, which is a total scope bypass. Fail closed.
fn prefix_match(prefixes: &[String], path: &str) -> bool {
    prefixes.iter().any(|p| {
        let p = p.trim_end_matches('/');
        if p.is_empty() {
            return false;
        }
        path == p || path.starts_with(&format!("{p}/"))
    })
}

/// The engineering name of an action kind.
fn kind_name(kind: ActionKind) -> &'static str {
    match kind {
        ActionKind::Inspect => "Inspect",
        ActionKind::RunApprovedTest => "RunApprovedTest",
        ActionKind::Generate => "Generate",
        ActionKind::ProposePatch => "ProposePatch",
        ActionKind::RequestApproval => "RequestApproval",
        ActionKind::Stop => "Stop",
    }
}

/// A comma-joined list of action kinds.
fn kind_list(kinds: &[ActionKind]) -> String {
    if kinds.is_empty() {
        return "none".to_string();
    }
    kinds
        .iter()
        .map(|k| kind_name(*k))
        .collect::<Vec<_>>()
        .join(", ")
}

/// A comma-joined list, or `none` when empty.
fn list_or_none<'a, I: Iterator<Item = &'a str>>(items: I) -> String {
    let v: Vec<&str> = items.collect();
    if v.is_empty() {
        "none".to_string()
    } else {
        v.join(", ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::{ApprovalPolicy, ReadScope, TestCommand};
    use tsr_domain::{
        CandidateId, EventId, Forecast, ForecastId, Forecaster, Hash, Horizon, OutcomeDefId,
        Probability, Source, Support,
    };

    fn hash(byte: u8) -> Hash {
        Hash([byte; 32])
    }

    fn task() -> TaskId {
        TaskId(hash(1))
    }

    fn profile() -> Profile {
        Profile {
            version: 1,
            name: "test".to_string(),
            read_scope: ReadScope {
                paths: vec!["src/".to_string(), "tests/".to_string()],
                rts_tools: vec!["search".to_string(), "callers".to_string()],
            },
            tests: vec![TestCommand {
                id: "cargo-test".to_string(),
                argv: vec!["cargo".to_string(), "test".to_string()],
            }],
            budgets: Budgets {
                generator_tokens: 1000,
                tool_minutes: 10,
                wall_clock_secs: 600,
                generator_calls: 4,
                patch_bytes: 4096,
            },
            approvals: ApprovalPolicy {
                promote_diff: true,
                non_allowlisted_command: true,
                network: true,
            },
        }
    }

    fn grant() -> Grant {
        Grant {
            task: task(),
            allowed_kinds: vec![
                ActionKind::Inspect,
                ActionKind::RunApprovedTest,
                ActionKind::Generate,
                ActionKind::ProposePatch,
                ActionKind::RequestApproval,
                ActionKind::Stop,
            ],
            read_scope_paths: vec!["src/".to_string()],
            read_scope_rts_tools: vec!["search".to_string()],
            allowed_test_ids: vec!["cargo-test".to_string()],
            budgets: Budgets {
                generator_tokens: 1000,
                tool_minutes: 10,
                wall_clock_secs: 600,
                generator_calls: 4,
                patch_bytes: 2048,
            },
        }
    }

    fn candidate(kind: ActionKind) -> Candidate {
        Candidate::new(kind, Source::Generator, hash(7), vec![], vec![]).expect("candidate")
    }

    fn forecast_event(c: &Candidate, version: u64) -> Event {
        Event::new(
            task(),
            vec![],
            Source::Rules,
            EventKind::Forecast {
                forecast: Forecast {
                    id: ForecastId(hash(9)),
                    event: OutcomeDefId(hash(10)),
                    horizon: Horizon::ThisDispatch,
                    source: Forecaster::Controller,
                    raw: Probability::new(0.5).unwrap(),
                    calibrated: None,
                    support: Support { episodes: 1 },
                    state_version: version,
                },
                candidate: c.id,
            },
            "2025-12-31T00:00:00Z",
        )
    }

    fn input<'a>(
        profile: &'a Profile,
        grant: &'a Grant,
        log: &'a [Event],
        candidate: &'a Candidate,
        args: ActionArgs,
    ) -> GateInput<'a> {
        GateInput {
            profile,
            grant,
            log,
            candidate,
            state_version: 1,
            args,
        }
    }

    #[test]
    fn kind_outside_the_grant_is_denied() {
        let p = profile();
        let mut g = grant();
        g.allowed_kinds = vec![ActionKind::Inspect];
        let c = candidate(ActionKind::Generate);
        let v = evaluate(&input(&p, &g, &[], &c, ActionArgs::Generate));
        assert_eq!(v.decision, Decision::Deny);
        assert!(
            v.reason.contains("not in this task's grant"),
            "{}",
            v.reason
        );
    }

    #[test]
    fn inspect_path_inside_both_scopes_is_allowed() {
        let p = profile();
        let g = grant();
        let c = candidate(ActionKind::Inspect);
        let v = evaluate(&input(
            &p,
            &g,
            &[],
            &c,
            ActionArgs::InspectPath("src/parser.rs".to_string()),
        ));
        assert_eq!(v.decision, Decision::Allow, "{}", v.reason);
    }

    #[test]
    fn inspect_path_outside_the_grant_scope_is_denied() {
        let p = profile();
        let g = grant();
        let c = candidate(ActionKind::Inspect);
        // `tests/` is in the profile but not in the grant: the intersection denies it.
        let v = evaluate(&input(
            &p,
            &g,
            &[],
            &c,
            ActionArgs::InspectPath("tests/foo.rs".to_string()),
        ));
        assert_eq!(v.decision, Decision::Deny);
        assert!(v.reason.contains("outside the read scope"), "{}", v.reason);
    }

    #[test]
    fn inspect_path_escaping_with_dotdot_is_denied() {
        let p = profile();
        let g = grant();
        let c = candidate(ActionKind::Inspect);
        let v = evaluate(&input(
            &p,
            &g,
            &[],
            &c,
            ActionArgs::InspectPath("src/../../etc/passwd".to_string()),
        ));
        assert_eq!(v.decision, Decision::Deny);
        assert!(v.reason.contains("escapes"), "{}", v.reason);
    }

    #[test]
    fn absolute_inspect_path_is_denied() {
        let p = profile();
        let g = grant();
        let c = candidate(ActionKind::Inspect);
        let v = evaluate(&input(
            &p,
            &g,
            &[],
            &c,
            ActionArgs::InspectPath("/etc/passwd".to_string()),
        ));
        assert_eq!(v.decision, Decision::Deny);
        assert!(v.reason.contains("absolute"), "{}", v.reason);
    }

    #[test]
    fn rts_tool_outside_the_grant_is_denied() {
        let p = profile();
        let g = grant();
        let c = candidate(ActionKind::Inspect);
        // `callers` is in the profile but not the grant.
        let v = evaluate(&input(
            &p,
            &g,
            &[],
            &c,
            ActionArgs::InspectRtsTool("callers".to_string()),
        ));
        assert_eq!(v.decision, Decision::Deny);
        assert!(v.reason.contains("outside the read scope"), "{}", v.reason);
    }

    #[test]
    fn allowlisted_test_id_is_allowed_with_a_forecast() {
        let p = profile();
        let g = grant();
        let c = candidate(ActionKind::RunApprovedTest);
        let log = vec![forecast_event(&c, 1)];
        let v = evaluate(&input(
            &p,
            &g,
            &log,
            &c,
            ActionArgs::TestId("cargo-test".to_string()),
        ));
        assert_eq!(v.decision, Decision::Allow, "{}", v.reason);
    }

    #[test]
    fn test_id_outside_the_allowlist_is_denied() {
        let p = profile();
        let g = grant();
        let c = candidate(ActionKind::RunApprovedTest);
        let log = vec![forecast_event(&c, 1)];
        let v = evaluate(&input(
            &p,
            &g,
            &log,
            &c,
            ActionArgs::TestId("rm-rf".to_string()),
        ));
        assert_eq!(v.decision, Decision::Deny);
        assert!(v.reason.contains("not allowlisted"), "{}", v.reason);
    }

    #[test]
    fn patch_over_the_smaller_ceiling_is_denied() {
        let p = profile();
        let g = grant(); // grant patch_bytes = 2048, profile = 4096
        let c = candidate(ActionKind::ProposePatch);
        let log = vec![forecast_event(&c, 1)];
        let v = evaluate(&input(&p, &g, &log, &c, ActionArgs::PatchBytes(3000)));
        assert_eq!(v.decision, Decision::Deny);
        assert!(v.reason.contains("2048-byte patch budget"), "{}", v.reason);
    }

    #[test]
    fn patch_within_both_ceilings_is_allowed() {
        let p = profile();
        let g = grant();
        let c = candidate(ActionKind::ProposePatch);
        let log = vec![forecast_event(&c, 1)];
        let v = evaluate(&input(&p, &g, &log, &c, ActionArgs::PatchBytes(1024)));
        assert_eq!(v.decision, Decision::Allow, "{}", v.reason);
    }

    #[test]
    fn boundary_without_an_approval_asks() {
        let p = profile();
        let g = grant();
        let c = candidate(ActionKind::RequestApproval);
        let log = vec![forecast_event(&c, 1)];
        let v = evaluate(&input(
            &p,
            &g,
            &log,
            &c,
            ActionArgs::Boundary(BoundaryAction::PromoteDiff),
        ));
        assert_eq!(v.decision, Decision::Ask, "{}", v.reason);
        assert!(
            v.reason.contains("operator approval required"),
            "{}",
            v.reason
        );
    }

    #[test]
    fn gated_action_without_a_forecast_is_denied_naming_what_to_record() {
        let p = profile();
        let g = grant();
        let c = candidate(ActionKind::RunApprovedTest);
        let v = evaluate(&input(
            &p,
            &g,
            &[],
            &c,
            ActionArgs::TestId("cargo-test".to_string()),
        ));
        assert_eq!(v.decision, Decision::Deny);
        assert!(
            v.reason
                .contains("record a forecast for RunApprovedTest at state version 1"),
            "{}",
            v.reason
        );
    }

    #[test]
    fn forecast_at_a_different_version_does_not_satisfy_the_gate() {
        let p = profile();
        let g = grant();
        let c = candidate(ActionKind::RunApprovedTest);
        let log = vec![forecast_event(&c, 2)]; // forecast at version 2, gate at version 1
        let v = evaluate(&input(
            &p,
            &g,
            &log,
            &c,
            ActionArgs::TestId("cargo-test".to_string()),
        ));
        assert_eq!(v.decision, Decision::Deny);
        assert!(v.reason.contains("record a forecast"), "{}", v.reason);
    }

    #[test]
    fn exhausted_budget_blocks_dispatch() {
        let p = profile();
        let mut g = grant();
        g.budgets.generator_calls = 0; // exhausted from the start
        let c = candidate(ActionKind::Generate);
        let v = evaluate(&input(&p, &g, &[], &c, ActionArgs::Generate));
        assert_eq!(v.decision, Decision::Deny);
        assert!(v.reason.contains("generator_calls"), "{}", v.reason);
    }

    #[test]
    fn generate_and_stop_are_allowed_when_in_the_grant() {
        let p = profile();
        let g = grant();
        let gen = candidate(ActionKind::Generate);
        assert_eq!(
            evaluate(&input(&p, &g, &[], &gen, ActionArgs::Generate)).decision,
            Decision::Allow
        );
        let stop = candidate(ActionKind::Stop);
        assert_eq!(
            evaluate(&input(&p, &g, &[], &stop, ActionArgs::Stop)).decision,
            Decision::Allow
        );
    }

    #[test]
    fn decision_as_str_is_the_lowercase_word() {
        assert_eq!(Decision::Allow.as_str(), "allow");
        assert_eq!(Decision::Deny.as_str(), "deny");
        assert_eq!(Decision::Ask.as_str(), "ask");
    }

    #[test]
    fn dispatch_denies_when_the_approval_is_missing() {
        let p = profile();
        let g = grant();
        let c = candidate(ActionKind::RequestApproval);
        let log = vec![forecast_event(&c, 1)];
        let v = evaluate_dispatch(&input(
            &p,
            &g,
            &log,
            &c,
            ActionArgs::Boundary(BoundaryAction::PromoteDiff),
        ));
        // evaluate() already returns Ask; dispatch keeps it non-allow.
        assert_ne!(v.decision, Decision::Allow);
    }

    #[test]
    fn dispatch_denies_when_the_snapshot_changed_since_approval() {
        let p = profile();
        let g = grant();
        let c = candidate(ActionKind::RequestApproval);
        let forecast = forecast_event(&c, 1);
        // The approval binds a snapshot that is NOT the current log head.
        let approval = Approval {
            task: task(),
            candidate: c.id,
            artifact_digest: c.args,
            snapshot_digest: hash(0xbb),
            scope: "promote diff".to_string(),
            expires_at: "2099-01-01T00:00:00Z".to_string(),
        };
        let approved = Event::new(
            task(),
            vec![],
            Source::Operator,
            EventKind::Approved {
                candidate: c.id,
                scope: approval.scope_string(),
                expires_at: approval.expires_at.clone(),
            },
            "2025-12-31T00:00:00Z",
        );
        let log = vec![forecast, approved];
        let v = evaluate_dispatch(&input(
            &p,
            &g,
            &log,
            &c,
            ActionArgs::Boundary(BoundaryAction::PromoteDiff),
        ));
        assert_eq!(v.decision, Decision::Deny, "{}", v.reason);
        assert!(v.reason.contains("no longer holds"), "{}", v.reason);
    }

    #[test]
    fn dispatch_denies_when_the_approval_was_revoked() {
        let p = profile();
        let g = grant();
        let c = candidate(ActionKind::RequestApproval);
        let forecast = forecast_event(&c, 1);
        let approval = Approval {
            task: task(),
            candidate: c.id,
            artifact_digest: c.args,
            snapshot_digest: forecast.id.0,
            scope: "promote diff".to_string(),
            expires_at: "2099-01-01T00:00:00Z".to_string(),
        };
        let approved = Event::new(
            task(),
            vec![],
            Source::Operator,
            EventKind::Approved {
                candidate: c.id,
                scope: approval.scope_string(),
                expires_at: approval.expires_at.clone(),
            },
            "2025-12-31T00:00:00Z",
        );
        let denied = Event::new(
            task(),
            vec![],
            Source::Operator,
            EventKind::Denied {
                candidate: c.id,
                reason: "operator revoked".to_string(),
            },
            "2025-12-31T00:00:01Z",
        );
        let log = vec![forecast, approved, denied];
        let v = evaluate_dispatch(&input(
            &p,
            &g,
            &log,
            &c,
            ActionArgs::Boundary(BoundaryAction::PromoteDiff),
        ));
        assert_eq!(v.decision, Decision::Deny, "{}", v.reason);
        assert!(v.reason.contains("revoked"), "{}", v.reason);
    }

    #[test]
    fn dispatch_allows_when_the_approval_still_holds() {
        let p = profile();
        let g = grant();
        let c = candidate(ActionKind::RequestApproval);
        // The approval binds the current artifact (candidate args) and the current snapshot head,
        // which is the last non-approval event in the log (here, the forecast).
        let forecast = forecast_event(&c, 1);
        let approval = Approval {
            task: task(),
            candidate: c.id,
            artifact_digest: c.args,
            snapshot_digest: forecast.id.0,
            scope: "promote diff".to_string(),
            expires_at: "2099-01-01T00:00:00Z".to_string(),
        };
        let approved = Event::new(
            task(),
            vec![],
            Source::Operator,
            EventKind::Approved {
                candidate: c.id,
                scope: approval.scope_string(),
                expires_at: approval.expires_at.clone(),
            },
            "2025-12-31T00:00:00Z",
        );
        let log = vec![forecast, approved];
        let v = evaluate_dispatch(&input(
            &p,
            &g,
            &log,
            &c,
            ActionArgs::Boundary(BoundaryAction::PromoteDiff),
        ));
        assert_eq!(v.decision, Decision::Allow, "{}", v.reason);
    }

    #[test]
    fn candidate_id_is_used_for_the_forecast_match() {
        // A forecast for a different candidate does not satisfy the gate.
        let p = profile();
        let g = grant();
        let c = candidate(ActionKind::RunApprovedTest);
        let other = Candidate::new(
            ActionKind::RunApprovedTest,
            Source::Generator,
            hash(8),
            vec![],
            vec![],
        )
        .unwrap();
        let log = vec![forecast_event(&other, 1)];
        let v = evaluate(&input(
            &p,
            &g,
            &log,
            &c,
            ActionArgs::TestId("cargo-test".to_string()),
        ));
        assert_eq!(v.decision, Decision::Deny);
    }

    #[test]
    fn unused_imports_are_referenced() {
        // Keep the imports honest: these types are part of the frozen surface.
        let _ = CandidateId(hash(0));
        let _ = EventId(hash(0));
    }

    /// D1: the dispatch re-check must go through the single approval predicate
    /// (`approval::validate`), not a second hand-written copy. This pins the seam: an approval the
    /// validator rejects (here, an unparseable expiry) must also be denied at dispatch.
    #[test]
    fn dispatch_recheck_uses_the_single_approval_predicate() {
        let p = profile();
        let g = grant();
        let c = candidate(ActionKind::RequestApproval);
        let forecast = forecast_event(&c, 1);
        let snapshot_digest = forecast.id.0;
        let approval = Approval {
            task: task(),
            candidate: c.id,
            artifact_digest: c.args,
            snapshot_digest,
            scope: "promote diff".to_string(),
            // A malformed expiry: `Approval::is_valid` treats it as expired, and dispatch must too.
            expires_at: "zzz".to_string(),
        };
        let approved = Event::new(
            task(),
            vec![],
            Source::Operator,
            EventKind::Approved {
                candidate: c.id,
                scope: approval.scope_string(),
                expires_at: approval.expires_at.clone(),
            },
            "2025-12-31T00:00:00Z",
        );
        let log = vec![forecast, approved];

        // The validator rejects it...
        assert!(approval::validate(
            &approval,
            "2025-12-31T00:00:00Z",
            task(),
            c.id,
            c.args,
            snapshot_digest,
            "promote diff",
        )
        .is_err());

        // ...and dispatch denies it, with the same predicate's reason.
        let v = evaluate_dispatch(&input(
            &p,
            &g,
            &log,
            &c,
            ActionArgs::Boundary(BoundaryAction::PromoteDiff),
        ));
        assert_eq!(v.decision, Decision::Deny, "{}", v.reason);
        assert!(v.reason.contains("no longer holds"), "{}", v.reason);
    }
}
