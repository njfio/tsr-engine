//! The hook tool-name → [`ActionKind`] mapping (PRD §B4, §B5).
//!
//! The hook sees a tool name (`Read`, `Bash`, `Edit`, …) and must decide which of the six typed
//! actions it is. That mapping is a **pure function of the tool name alone**: it reads no
//! arguments, no repository text, no provider output and no scores. The arguments are checked
//! later, in the gate — a `Bash` call is `RunApprovedTest` whatever its argv, and the gate is what
//! refuses an argv that is not an allowlisted command id.
//!
//! `Wait` is a task **state**, not an action (AGENTS.md invariant 11, PRD §B5): it maps to `None`,
//! as does every tool name the vocabulary does not name. An unknown tool is not silently treated
//! as `Inspect` — policy is fail-closed, and the caller denies what it cannot type.

use tsr_domain::ActionKind;

/// Map a hook tool name to the typed action it is, or `None` if the vocabulary does not name it.
///
/// | tool name | action |
/// | --- | --- |
/// | `Read`, `Grep`, `Glob`, `rts` | [`ActionKind::Inspect`] |
/// | `Bash` | [`ActionKind::RunApprovedTest`] (the argv is checked in the gate) |
/// | `Task`, `agent-generate` | [`ActionKind::Generate`] |
/// | `Edit`, `Write` | [`ActionKind::ProposePatch`] |
/// | `RequestApproval` | [`ActionKind::RequestApproval`] |
/// | `Stop` | [`ActionKind::Stop`] |
/// | anything else, including `Wait` | `None` |
///
/// The match is exact and case-sensitive: a tool named `read` is not `Read`, and an unknown name
/// is `None` rather than a guess.
pub fn action_kind_for_tool(tool: &str) -> Option<ActionKind> {
    match tool {
        // Inspect: reads against a read-only view of the copy (PRD §B4).
        "Read" | "Grep" | "Glob" | "rts" => Some(ActionKind::Inspect),
        // A shell call is a test/build call; the gate checks the argv against the allowlist.
        "Bash" => Some(ActionKind::RunApprovedTest),
        // The model's own turn, and the council behind Generate(alternatives).
        "Task" | "agent-generate" => Some(ActionKind::Generate),
        // Edits are applied inside the sandbox and are reversible by git.
        "Edit" | "Write" => Some(ActionKind::ProposePatch),
        // The boundary: promote diff, non-allowlisted command, network.
        "RequestApproval" => Some(ActionKind::RequestApproval),
        // The Stop hook closes the task.
        "Stop" => Some(ActionKind::Stop),
        // `Wait` is a state, not an action; unknown tools are not typed.
        _ => None,
    }
}

/// True for the three actions that cannot run until a forecast for the current state version is on
/// record (PRD §B5, forecast-gated execution): `RunApprovedTest`, `ProposePatch`,
/// `RequestApproval`.
pub fn is_gated(kind: ActionKind) -> bool {
    matches!(
        kind,
        ActionKind::RunApprovedTest | ActionKind::ProposePatch | ActionKind::RequestApproval
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inspect_tools_map_to_inspect() {
        for tool in ["Read", "Grep", "Glob", "rts"] {
            assert_eq!(
                action_kind_for_tool(tool),
                Some(ActionKind::Inspect),
                "{tool} must be Inspect"
            );
        }
    }

    #[test]
    fn bash_maps_to_run_approved_test() {
        // The tool name alone decides the kind; the argv is the gate's business.
        assert_eq!(
            action_kind_for_tool("Bash"),
            Some(ActionKind::RunApprovedTest)
        );
    }

    #[test]
    fn generate_tools_map_to_generate() {
        assert_eq!(action_kind_for_tool("Task"), Some(ActionKind::Generate));
        assert_eq!(
            action_kind_for_tool("agent-generate"),
            Some(ActionKind::Generate)
        );
    }

    #[test]
    fn edit_tools_map_to_propose_patch() {
        assert_eq!(action_kind_for_tool("Edit"), Some(ActionKind::ProposePatch));
        assert_eq!(
            action_kind_for_tool("Write"),
            Some(ActionKind::ProposePatch)
        );
    }

    #[test]
    fn approval_and_stop_hooks_map_to_their_kinds() {
        assert_eq!(
            action_kind_for_tool("RequestApproval"),
            Some(ActionKind::RequestApproval)
        );
        assert_eq!(action_kind_for_tool("Stop"), Some(ActionKind::Stop));
    }

    #[test]
    fn wait_is_a_state_not_an_action() {
        // AGENTS.md invariant 11: the vocabulary is the six ActionKinds.
        assert_eq!(action_kind_for_tool("Wait"), None);
    }

    #[test]
    fn unknown_tools_are_not_typed() {
        for tool in [
            "",
            "wait",
            "read",
            "BASH",
            "MultiEdit",
            "NotebookEdit",
            "WebFetch",
        ] {
            assert_eq!(action_kind_for_tool(tool), None, "{tool:?} must be None");
        }
    }

    #[test]
    fn exactly_the_three_gated_kinds_are_gated() {
        assert!(is_gated(ActionKind::RunApprovedTest));
        assert!(is_gated(ActionKind::ProposePatch));
        assert!(is_gated(ActionKind::RequestApproval));
        assert!(!is_gated(ActionKind::Inspect));
        assert!(!is_gated(ActionKind::Generate));
        assert!(!is_gated(ActionKind::Stop));
    }
}
