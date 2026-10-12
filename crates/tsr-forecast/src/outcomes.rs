//! The named events of §B7 as data: horizon, required evidence, censoring rule and
//! intervention rule, each with a versioned [`OutcomeDefId`].
//!
//! Nothing here is fitted. These are the definitions a forecast is made against and a
//! return is resolved against; the calibrator that turns them into probabilities is M4.

use serde::{Deserialize, Serialize};
use tsr_domain::{Hash, Horizon, OutcomeDef, OutcomeDefId};

/// The canonical name of the common decision event (B7, B8): every candidate is scored on
/// this one event so unlike actions are compared on like terms.
pub const COMMON_DECISION_EVENT: &str = "task_accepted_in_budget";

/// The version of the common decision event. `task_accepted_in_budget@v1`.
pub const COMMON_DECISION_EVENT_VERSION: u32 = 1;

/// The four Intuition stages (B7, v0.3.1). Each is its own named event with its own horizon.
pub const INTUITION_STAGES: [&str; 4] = [
    "intuition_offered",
    "intuition_selected",
    "intuition_executed",
    "intuition_observed",
];

/// Mint a versioned [`OutcomeDefId`] from a name and version: `blake3("name@vN")`.
///
/// The id commits to the name and the version, so `test_pass@v1` and `test_pass@v2` are
/// different ids and a forecast made against one is never silently resolved against the other.
///
/// The hashed bytes are the literal `name@vN` string — the same bytes `tsr hook` hashes for the
/// common decision event (`hook.rs`, `DECISION_EVENT = "task_accepted_in_budget@v1"`). A
/// forecast the hook records is therefore scored against the id this function mints (D2).
pub fn outcome_def_id(name: &str, version: u32) -> OutcomeDefId {
    OutcomeDefId(Hash::of(&[format!("{name}@v{version}").as_bytes()]))
}

/// The canonical id of the common decision event, `task_accepted_in_budget@v1`.
pub fn common_decision_event_id() -> OutcomeDefId {
    outcome_def_id(COMMON_DECISION_EVENT, COMMON_DECISION_EVENT_VERSION)
}

/// The canonical id of a named event at the current version, or `None` if the name is not one
/// of the §B7 events.
pub fn named_event_id(name: &str) -> Option<OutcomeDefId> {
    let version = match name {
        "test_pass" | "diff_accepted" => 1,
        "task_accepted_in_budget" => COMMON_DECISION_EVENT_VERSION,
        n if INTUITION_STAGES.contains(&n) => 1,
        _ => return None,
    };
    Some(outcome_def_id(name, version))
}

/// The required evidence for a named event: what must be present in the log for the event to
/// be resolvable at all. A return that lacks it is `unresolved`, never `false`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RequiredEvidence {
    /// An `Executed` event for the dispatch, with an exit code.
    ExecutedExitCode,
    /// An operator outcome record labelling the promoted diff.
    OperatorLabel,
    /// A `TaskClosed` event for the task.
    TaskClosed,
    /// A `Proposed` event carrying an Intuition candidate.
    IntuitionCandidate,
    /// A `Committed` decision naming the Intuition candidate.
    IntuitionCommitted,
    /// An `Executed` event for the Intuition candidate.
    IntuitionExecuted,
    /// The common event resolved for the Intuition candidate's episode.
    CommonEventResolved,
}

impl RequiredEvidence {
    /// A short engineering name for the evidence, for the Mirror and for diagnostics.
    pub fn name(self) -> &'static str {
        match self {
            RequiredEvidence::ExecutedExitCode => "executed_exit_code",
            RequiredEvidence::OperatorLabel => "operator_label",
            RequiredEvidence::TaskClosed => "task_closed",
            RequiredEvidence::IntuitionCandidate => "intuition_candidate",
            RequiredEvidence::IntuitionCommitted => "intuition_committed",
            RequiredEvidence::IntuitionExecuted => "intuition_executed",
            RequiredEvidence::CommonEventResolved => "common_event_resolved",
        }
    }
}

/// The censoring rule: when an event that never resolved is recorded as censored rather than
/// left unresolved. Censoring is a fact about the record, not a value of the outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CensoringRule {
    /// Timeout or cancellation of the dispatch censors the forecast.
    TimeoutOrCancellation,
    /// Never reviewed by the horizon censors the forecast.
    NeverReviewed,
    /// An operator stop for external reasons censors the forecast.
    OperatorStopExternal,
    /// An offer not selected is recorded with what was selected instead; it is not censored.
    OfferNotSelected,
}

impl CensoringRule {
    pub fn name(self) -> &'static str {
        match self {
            CensoringRule::TimeoutOrCancellation => "timeout_or_cancellation",
            CensoringRule::NeverReviewed => "never_reviewed",
            CensoringRule::OperatorStopExternal => "operator_stop_external",
            CensoringRule::OfferNotSelected => "offer_not_selected",
        }
    }
}

/// The intervention rule: what an operator edit between forecast and resolution does to the
/// forecast. The rule is data so the Mirror can print it beside the number.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterventionRule {
    /// An operator edit between forecast and run censors the generator's forecast; the
    /// controller's is kept with a flag.
    EditCensorsGeneratorKeepsController,
    /// An operator-modified diff resolves as `disputed`.
    ModifiedDiffResolvesDisputed,
    /// The one event every candidate is scored on; the action-specific events are features.
    CommonDecisionEvent,
    /// An operator who takes the intuition by hand records `selected` with `by: Operator`.
    OperatorTakesByHand,
}

impl InterventionRule {
    pub fn name(self) -> &'static str {
        match self {
            InterventionRule::EditCensorsGeneratorKeepsController => {
                "edit_censors_generator_keeps_controller"
            }
            InterventionRule::ModifiedDiffResolvesDisputed => "modified_diff_resolves_disputed",
            InterventionRule::CommonDecisionEvent => "common_decision_event",
            InterventionRule::OperatorTakesByHand => "operator_takes_by_hand",
        }
    }
}

/// A named event of §B7 with its rules as data, plus the machine-readable evidence the
/// resolver needs. This is the crate's own richer view; [`NamedEvent::def`] projects it onto
/// the domain's [`OutcomeDef`] so the contract type is the one on the wire.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NamedEvent {
    pub id: OutcomeDefId,
    pub name: String,
    pub version: u32,
    pub horizon: Horizon,
    pub resolves_true_when: String,
    pub censored_when: String,
    pub intervention_rule: String,
    /// Machine-readable form of `resolves_true_when`'s evidence requirement.
    pub required_evidence: Vec<RequiredEvidence>,
    /// Machine-readable form of `censored_when`.
    pub censoring: CensoringRule,
    /// Machine-readable form of `intervention_rule`.
    pub intervention: InterventionRule,
    /// True for the common decision event every candidate is scored on (B8).
    pub is_common_decision_event: bool,
}

impl NamedEvent {
    /// Project onto the domain contract type. The strings are the same ones the PRD's table
    /// carries, so a reader of the log sees the rule, not a code.
    pub fn def(&self) -> OutcomeDef {
        OutcomeDef {
            id: self.id,
            name: self.name.clone(),
            version: self.version,
            horizon: self.horizon.clone(),
            resolves_true_when: self.resolves_true_when.clone(),
            censored_when: self.censored_when.clone(),
            intervention_rule: self.intervention_rule.clone(),
        }
    }
}

/// The §B7 named events, v1. The order is the PRD's table order.
pub fn named_events() -> Vec<NamedEvent> {
    vec![
        NamedEvent {
            id: outcome_def_id("test_pass", 1),
            name: "test_pass".into(),
            version: 1,
            horizon: Horizon::ThisDispatch,
            resolves_true_when: "the approved command exits 0 on the matching snapshot digest"
                .into(),
            censored_when: "timeout or cancellation".into(),
            intervention_rule:
                "an operator edit between forecast and run censors the generator's forecast; \
                 the controller's is kept with a flag"
                    .into(),
            required_evidence: vec![RequiredEvidence::ExecutedExitCode],
            censoring: CensoringRule::TimeoutOrCancellation,
            intervention: InterventionRule::EditCensorsGeneratorKeepsController,
            is_common_decision_event: false,
        },
        NamedEvent {
            id: outcome_def_id("diff_accepted", 1),
            name: "diff_accepted".into(),
            version: 1,
            horizon: Horizon::TaskCloseDays(7),
            resolves_true_when: "the operator labels the promoted diff `accepted` at review".into(),
            censored_when: "never reviewed".into(),
            intervention_rule: "an operator-modified diff resolves as `disputed`".into(),
            required_evidence: vec![RequiredEvidence::OperatorLabel],
            censoring: CensoringRule::NeverReviewed,
            intervention: InterventionRule::ModifiedDiffResolvesDisputed,
            is_common_decision_event: false,
        },
        NamedEvent {
            id: common_decision_event_id(),
            name: COMMON_DECISION_EVENT.into(),
            version: COMMON_DECISION_EVENT_VERSION,
            horizon: Horizon::TaskCloseDays(7),
            resolves_true_when:
                "the task closes with criteria met, the promoted diff accepted, inside every \
                 budget ceiling"
                    .into(),
            censored_when: "operator stop for external reasons; never reviewed".into(),
            intervention_rule:
                "the one event every candidate is scored on in B8; the two above are features \
                 and calibration diagnostics"
                    .into(),
            required_evidence: vec![
                RequiredEvidence::TaskClosed,
                RequiredEvidence::OperatorLabel,
            ],
            censoring: CensoringRule::OperatorStopExternal,
            intervention: InterventionRule::CommonDecisionEvent,
            is_common_decision_event: true,
        },
        NamedEvent {
            id: outcome_def_id("intuition_offered", 1),
            name: "intuition_offered".into(),
            version: 1,
            horizon: Horizon::ThisDispatch,
            resolves_true_when: "composed and admitted".into(),
            censored_when: "an offer not selected is recorded with what was selected instead"
                .into(),
            intervention_rule: "an operator who takes the intuition by hand records `selected` \
                 with `by: Operator`"
                .into(),
            required_evidence: vec![RequiredEvidence::IntuitionCandidate],
            censoring: CensoringRule::OfferNotSelected,
            intervention: InterventionRule::OperatorTakesByHand,
            is_common_decision_event: false,
        },
        NamedEvent {
            id: outcome_def_id("intuition_selected", 1),
            name: "intuition_selected".into(),
            version: 1,
            horizon: Horizon::ThisDispatch,
            resolves_true_when: "chosen by the policy".into(),
            censored_when: "an offer not selected is recorded with what was selected instead"
                .into(),
            intervention_rule: "an operator who takes the intuition by hand records `selected` \
                 with `by: Operator`"
                .into(),
            required_evidence: vec![RequiredEvidence::IntuitionCommitted],
            censoring: CensoringRule::OfferNotSelected,
            intervention: InterventionRule::OperatorTakesByHand,
            is_common_decision_event: false,
        },
        NamedEvent {
            id: outcome_def_id("intuition_executed", 1),
            name: "intuition_executed".into(),
            version: 1,
            horizon: Horizon::ThisDispatch,
            resolves_true_when: "the selected intuition step ran".into(),
            censored_when: "timeout or cancellation".into(),
            intervention_rule: "an operator who takes the intuition by hand records `selected` \
                 with `by: Operator`"
                .into(),
            required_evidence: vec![RequiredEvidence::IntuitionExecuted],
            censoring: CensoringRule::TimeoutOrCancellation,
            intervention: InterventionRule::OperatorTakesByHand,
            is_common_decision_event: false,
        },
        NamedEvent {
            id: outcome_def_id("intuition_observed", 1),
            name: "intuition_observed".into(),
            version: 1,
            horizon: Horizon::TaskCloseDays(7),
            resolves_true_when: "its common event resolved".into(),
            censored_when: "never reviewed".into(),
            intervention_rule: "an operator who takes the intuition by hand records `selected` \
                 with `by: Operator`"
                .into(),
            required_evidence: vec![RequiredEvidence::CommonEventResolved],
            censoring: CensoringRule::NeverReviewed,
            intervention: InterventionRule::OperatorTakesByHand,
            is_common_decision_event: false,
        },
    ]
}

/// Look up a named event by its versioned id.
pub fn named_event_by_id(id: &OutcomeDefId) -> Option<NamedEvent> {
    named_events().into_iter().find(|e| &e.id == id)
}

/// Look up a named event by name at the current version.
pub fn named_event_by_name(name: &str) -> Option<NamedEvent> {
    named_event_id(name).and_then(|id| named_event_by_id(&id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_b7_event_is_defined_with_all_four_rules() {
        let events = named_events();
        assert_eq!(
            events.len(),
            7,
            "test_pass, diff_accepted, common, four stages"
        );
        for e in &events {
            assert!(!e.resolves_true_when.is_empty(), "{}", e.name);
            assert!(!e.censored_when.is_empty(), "{}", e.name);
            assert!(!e.intervention_rule.is_empty(), "{}", e.name);
            assert!(!e.required_evidence.is_empty(), "{}", e.name);
            assert_eq!(e.id, outcome_def_id(&e.name, e.version));
        }
        let names: Vec<&str> = events.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"test_pass"));
        assert!(names.contains(&"diff_accepted"));
        assert!(names.contains(&"task_accepted_in_budget"));
        for stage in INTUITION_STAGES {
            assert!(names.contains(&stage), "missing {stage}");
        }
    }

    #[test]
    fn ids_are_versioned_so_v1_and_v2_are_different_events() {
        assert_ne!(
            outcome_def_id("test_pass", 1),
            outcome_def_id("test_pass", 2)
        );
        assert_eq!(
            named_event_id("test_pass"),
            Some(outcome_def_id("test_pass", 1))
        );
        assert_eq!(named_event_id("nonsense"), None);
    }

    #[test]
    fn exactly_one_common_decision_event_and_it_is_the_v1_id() {
        let events = named_events();
        let common: Vec<&NamedEvent> = events
            .iter()
            .filter(|e| e.is_common_decision_event)
            .collect();
        assert_eq!(common.len(), 1);
        assert_eq!(common[0].id, common_decision_event_id());
        assert_eq!(common[0].name, COMMON_DECISION_EVENT);
        assert_eq!(common[0].version, 1);
    }

    #[test]
    fn def_projects_onto_the_domain_contract_type() {
        let e = named_event_by_name("diff_accepted").unwrap();
        let d: OutcomeDef = e.def();
        assert_eq!(d.id, e.id);
        assert_eq!(d.horizon, Horizon::TaskCloseDays(7));
        assert_eq!(d.name, "diff_accepted");
    }
}
