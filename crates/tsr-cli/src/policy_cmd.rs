//! `tsr policy check <profile.toml> <event.json>` — the operator's one-shot gate (PRD §B5).
//!
//! This is a thin, deterministic shell around [`tsr_policy::gate::evaluate`]: it loads a RATCHET
//! protection profile and an **event envelope** (a JSON object describing the proposed action, the
//! task's grant and the log so far), builds a [`GateInput`] and prints the verdict as one JSON
//! object on stdout:
//!
//! ```json
//! {"decision":"allow","reason":"Inspect is inside the grant and the profile at state version 1"}
//! ```
//!
//! Exit codes (documented in `docs/ratchet-profile.md`):
//!
//! | code | meaning |
//! | --- | --- |
//! | 0 | `allow` |
//! | 1 | `deny` |
//! | 2 | `ask` |
//! | 3 | malformed profile or event (a JSON error object is printed to stderr) |
//!
//! The command reads **no** scores, reflections, repository text or provider output. Everything it
//! reads is in the two files it is handed: the profile is authority, the envelope is the task's
//! grant, the log so far and the proposed candidate.

use std::path::Path;
use std::process::ExitCode;

use serde::Deserialize;
use tsr_domain::{ActionKind, Candidate, CandidateId, Event, Hash, Source, TaskId};
use tsr_policy::gate::{self, ActionArgs, BoundaryAction, Decision, GateInput, Grant};
use tsr_policy::profile::{Budgets, Profile};

/// The event envelope: the JSON object `tsr policy check` reads as its second argument.
///
/// ```json
/// {
///   "task": "<64 hex>",
///   "candidate": "<64 hex>",
///   "kind": "inspect|run_approved_test|generate|propose_patch|request_approval|stop",
///   "args": { ... },
///   "state_version": 1,
///   "grant": { ... },
///   "log": [ Event, ... ]
/// }
/// ```
///
/// `kind` and `args` are mapped to a [`Candidate`] and an [`ActionArgs`] by [`Envelope::into_parts`].
/// The candidate's `args` digest is the blake3 of the canonical JSON of the `args` object, so an
/// approval that binds an artifact digest binds exactly the arguments the model proposed.
#[derive(Debug, Deserialize)]
struct Envelope {
    /// The task this action belongs to (64 hex characters).
    task: String,
    /// The proposed candidate's id (64 hex characters).
    candidate: String,
    /// The action kind, as the snake_case name of [`ActionKind`].
    kind: String,
    /// The typed arguments of the action; shape depends on `kind`.
    #[serde(default)]
    args: serde_json::Value,
    /// The current state version.
    state_version: u64,
    /// The task's capability grant.
    grant: GrantJson,
    /// The log so far (approvals, forecasts, revocations).
    #[serde(default)]
    log: Vec<Event>,
}

/// The task's capability grant, as JSON. Mirrors [`Grant`] with hex ids and snake_case kinds.
#[derive(Debug, Deserialize)]
struct GrantJson {
    /// The task id (64 hex characters).
    task: String,
    /// The action kinds the task may take.
    allowed_kinds: Vec<String>,
    /// The path prefixes the task may read.
    #[serde(default)]
    read_scope_paths: Vec<String>,
    /// The `rts` tools the task may query.
    #[serde(default)]
    read_scope_rts_tools: Vec<String>,
    /// The allowlisted test command ids the task may run.
    #[serde(default)]
    allowed_test_ids: Vec<String>,
    /// The task's budgets.
    budgets: Budgets,
}

/// Why an envelope could not be turned into a [`GateInput`].
#[derive(Debug, thiserror::Error)]
enum EnvelopeError {
    /// A hex id was not 64 hex characters.
    #[error("invalid {what}: {source}")]
    Id {
        /// Which id failed (`task` or `candidate`).
        what: &'static str,
        /// The underlying contract error.
        source: tsr_domain::ContractError,
    },
    /// The `kind` was not one of the six action kinds.
    #[error("unknown action kind {0:?}; expected one of inspect, run_approved_test, generate, propose_patch, request_approval, stop")]
    Kind(String),
    /// The `args` object did not match the shape the `kind` requires.
    #[error("args for {kind}: {message}")]
    Args {
        /// The action kind whose args were malformed.
        kind: &'static str,
        /// What was wrong.
        message: String,
    },
}

impl Envelope {
    /// Parse the envelope from a JSON string.
    fn from_json_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    /// Map `kind` + `args` to the typed [`ActionKind`] and [`ActionArgs`] the gate reads.
    fn action(&self) -> Result<(ActionKind, ActionArgs), EnvelopeError> {
        let kind = match self.kind.as_str() {
            "inspect" => ActionKind::Inspect,
            "run_approved_test" => ActionKind::RunApprovedTest,
            "generate" => ActionKind::Generate,
            "propose_patch" => ActionKind::ProposePatch,
            "request_approval" => ActionKind::RequestApproval,
            "stop" => ActionKind::Stop,
            other => return Err(EnvelopeError::Kind(other.to_string())),
        };

        let args = match kind {
            ActionKind::Inspect => {
                // `{"path": "src/lib.rs"}` or `{"rts_tool": "search"}`.
                if let Some(path) = self.args.get("path").and_then(|v| v.as_str()) {
                    ActionArgs::InspectPath(path.to_string())
                } else if let Some(tool) = self.args.get("rts_tool").and_then(|v| v.as_str()) {
                    ActionArgs::InspectRtsTool(tool.to_string())
                } else {
                    return Err(EnvelopeError::Args {
                        kind: "inspect",
                        message: "expected {\"path\": \"...\"} or {\"rts_tool\": \"...\"}"
                            .to_string(),
                    });
                }
            }
            ActionKind::RunApprovedTest => {
                let id = self
                    .args
                    .get("test_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| EnvelopeError::Args {
                        kind: "run_approved_test",
                        message: "expected {\"test_id\": \"...\"}".to_string(),
                    })?;
                ActionArgs::TestId(id.to_string())
            }
            ActionKind::ProposePatch => {
                let bytes = self
                    .args
                    .get("patch_bytes")
                    .and_then(|v| v.as_u64())
                    .ok_or_else(|| EnvelopeError::Args {
                        kind: "propose_patch",
                        message: "expected {\"patch_bytes\": <u64>}".to_string(),
                    })?;
                ActionArgs::PatchBytes(bytes)
            }
            ActionKind::RequestApproval => {
                let name = self
                    .args
                    .get("boundary")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| EnvelopeError::Args {
                        kind: "request_approval",
                        message: "expected {\"boundary\": \"promote_diff|non_allowlisted_command|network\"}"
                            .to_string(),
                    })?;
                let boundary = match name {
                    "promote_diff" => BoundaryAction::PromoteDiff,
                    "non_allowlisted_command" => BoundaryAction::NonAllowlistedCommand,
                    "network" => BoundaryAction::Network,
                    other => {
                        return Err(EnvelopeError::Args {
                            kind: "request_approval",
                            message: format!("unknown boundary {other:?}"),
                        })
                    }
                };
                ActionArgs::Boundary(boundary)
            }
            ActionKind::Generate => ActionArgs::Generate,
            ActionKind::Stop => ActionArgs::Stop,
        };
        Ok((kind, args))
    }

    /// Build the [`Candidate`] the gate reads. Its `args` digest is the blake3 of the canonical
    /// JSON of the `args` object, so an approval binding an artifact digest binds these arguments.
    fn candidate(&self, kind: ActionKind) -> Result<Candidate, EnvelopeError> {
        let id =
            CandidateId(
                Hash::from_hex(&self.candidate).map_err(|source| EnvelopeError::Id {
                    what: "candidate",
                    source,
                })?,
            );
        let args_bytes = serde_json::to_vec(&self.args).expect("serde_json::Value is serializable");
        let args = Hash::of(&[&args_bytes]);
        Ok(Candidate {
            id,
            kind,
            source: Source::Generator,
            args,
            parents: Vec::new(),
            conflicts_with: Vec::new(),
            support: Vec::new(),
            forecasts: Vec::new(),
        })
    }

    /// Build the task's [`Grant`] from the envelope's `grant` object.
    fn grant(&self) -> Result<Grant, EnvelopeError> {
        let task =
            TaskId(
                Hash::from_hex(&self.grant.task).map_err(|source| EnvelopeError::Id {
                    what: "grant.task",
                    source,
                })?,
            );
        let mut allowed_kinds = Vec::with_capacity(self.grant.allowed_kinds.len());
        for k in &self.grant.allowed_kinds {
            allowed_kinds.push(match k.as_str() {
                "inspect" => ActionKind::Inspect,
                "run_approved_test" => ActionKind::RunApprovedTest,
                "generate" => ActionKind::Generate,
                "propose_patch" => ActionKind::ProposePatch,
                "request_approval" => ActionKind::RequestApproval,
                "stop" => ActionKind::Stop,
                other => return Err(EnvelopeError::Kind(other.to_string())),
            });
        }
        Ok(Grant {
            task,
            allowed_kinds,
            read_scope_paths: self.grant.read_scope_paths.clone(),
            read_scope_rts_tools: self.grant.read_scope_rts_tools.clone(),
            allowed_test_ids: self.grant.allowed_test_ids.clone(),
            budgets: self.grant.budgets.clone(),
        })
    }

    /// The task id of the envelope itself (the `task` field).
    fn task(&self) -> Result<TaskId, EnvelopeError> {
        Ok(TaskId(Hash::from_hex(&self.task).map_err(|source| {
            EnvelopeError::Id {
                what: "task",
                source,
            }
        })?))
    }
}

/// Run `tsr policy check <profile.toml> <event.json>`.
pub fn run(profile_path: &str, event_path: &str) -> ExitCode {
    let profile = match Profile::load(Path::new(profile_path)) {
        Ok(p) => p,
        Err(e) => return malformed("profile", profile_path, &e.to_string()),
    };

    let text = match std::fs::read_to_string(event_path) {
        Ok(t) => t,
        Err(e) => return malformed("event", event_path, &e.to_string()),
    };

    let envelope = match Envelope::from_json_str(&text) {
        Ok(env) => env,
        Err(e) => return malformed("event", event_path, &e.to_string()),
    };

    let (kind, args) = match envelope.action() {
        Ok(pair) => pair,
        Err(e) => return malformed("event", event_path, &e.to_string()),
    };
    let candidate = match envelope.candidate(kind) {
        Ok(c) => c,
        Err(e) => return malformed("event", event_path, &e.to_string()),
    };
    let grant = match envelope.grant() {
        Ok(g) => g,
        Err(e) => return malformed("event", event_path, &e.to_string()),
    };
    // The envelope's own `task` must agree with the grant's task; a mismatch is malformed input.
    match envelope.task() {
        Ok(t) if t == grant.task => {}
        Ok(_) => {
            return malformed(
                "event",
                event_path,
                "envelope `task` does not match `grant.task`",
            )
        }
        Err(e) => return malformed("event", event_path, &e.to_string()),
    }

    let input = GateInput {
        profile: &profile,
        grant: &grant,
        log: &envelope.log,
        candidate: &candidate,
        state_version: envelope.state_version,
        args,
    };
    let verdict = gate::evaluate(&input);

    // One JSON object on stdout: the decision and the one-line reason the model reads.
    let out = serde_json::json!({
        "decision": verdict.decision.as_str(),
        "reason": verdict.reason,
    });
    println!("{out}");

    match verdict.decision {
        Decision::Allow => ExitCode::from(0),
        Decision::Deny => ExitCode::from(1),
        Decision::Ask => ExitCode::from(2),
    }
}

/// Print a JSON error object to stderr and return exit code 3 (malformed profile or event).
fn malformed(what: &str, path: &str, message: &str) -> ExitCode {
    let err = serde_json::json!({
        "error": format!("malformed {what}"),
        "path": path,
        "message": message,
    });
    eprintln!("{err}");
    ExitCode::from(3)
}
