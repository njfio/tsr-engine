//! `tsr exec` — the sealed executor (PRD §B4, path B).
//!
//! The command runs **only** inside a sibling container: Podman or Docker, `--network none`, a
//! read-only root, the workspace as the only writable mount, and cgroup limits. The agent's argv is
//! never handed to the host shell — the only `Command` this module builds is the container runtime
//! invocation, and the argv is passed as arguments to it, never interpolated into a shell string.
//!
//! With no container runtime reachable the command refuses with exit code 3 and records **nothing**
//! (B4: "no runtime → refuse, never fall back to the host"). The refusal is decided in
//! [`Runtime::detect`] and acted on in [`run`] before any event is appended.

use std::path::{Path, PathBuf};
use std::process::Command;
use tsr_domain::{
    ActionKind, Candidate, CandidateId, Decision, Event, EventKind, Hash, Source, TaskId, Utility,
};
use tsr_store::Store;

/// The exit code for "no container runtime reachable" (B4).
pub const EXIT_NO_RUNTIME: i32 = 3;

/// A reachable container runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Runtime {
    Podman,
    Docker,
}

impl Runtime {
    /// The binary name.
    pub fn binary(self) -> &'static str {
        match self {
            Runtime::Podman => "podman",
            Runtime::Docker => "docker",
        }
    }

    /// Detect a reachable runtime: Podman first, then Docker. A runtime counts as reachable only if
    /// its binary exists on `PATH` and `--version` succeeds — a binary that cannot run is no runtime.
    pub fn detect() -> Option<Runtime> {
        [Runtime::Podman, Runtime::Docker]
            .into_iter()
            .find(|&rt| which(rt.binary()).is_some() && probe(rt))
    }
}

/// Find an executable on `PATH` without spawning a shell.
fn which(bin: &str) -> Option<PathBuf> {
    let path = std::env::var_os("PATH")?;
    for dir in std::env::split_paths(&path) {
        let candidate = dir.join(bin);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Probe a runtime with `--version`; a non-zero exit or a spawn failure means "not reachable".
fn probe(rt: Runtime) -> bool {
    Command::new(rt.binary())
        .arg("--version")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// The sealed-container invocation for a runtime and an argv. Pure: it builds the argument vector
/// and never runs anything, so it can be asserted on without a runtime present.
///
/// The container is a *sibling*: the workspace is bind-mounted at the same path, the root is
/// read-only, the network is off, and the cgroup limits are fixed.
pub fn container_argv(rt: Runtime, workspace: &Path, argv: &[String]) -> Vec<String> {
    let ws = workspace.display().to_string();
    let mut v: Vec<String> = vec![
        rt.binary().to_string(),
        "run".to_string(),
        "--rm".to_string(),
        "--network".to_string(),
        "none".to_string(),
        "--read-only".to_string(),
        "--tmpfs".to_string(),
        "/tmp:rw,noexec,nosuid,size=64m".to_string(),
        "--mount".to_string(),
        format!("type=bind,src={ws},dst={ws}"),
        "--workdir".to_string(),
        ws,
        "--cpus".to_string(),
        "2".to_string(),
        "--memory".to_string(),
        "2g".to_string(),
        "--pids-limit".to_string(),
        "512".to_string(),
        "--security-opt".to_string(),
        "no-new-privileges".to_string(),
        "--cap-drop".to_string(),
        "ALL".to_string(),
        "docker.io/library/debian:stable-slim".to_string(),
    ];
    v.extend(argv.iter().cloned());
    v
}

/// The outcome of a dispatch.
#[derive(Debug, Clone)]
pub struct ExecOutcome {
    pub exit_code: i32,
    pub log_digest: Option<Hash>,
    pub duration_ms: u64,
}

/// Run `argv` in the sealed container and record the dispatch lifecycle.
///
/// Returns `Err(EXIT_NO_RUNTIME)`-shaped refusal via [`ExecRefusal`] when no runtime is reachable:
/// in that case **nothing is recorded**. Otherwise it records `Dispatched { in_sandbox: true }`
/// then `Executed` (or `Unresolved` if the runtime itself failed to produce an exit code).
pub fn run(
    store: &Store,
    task: &TaskId,
    workspace: &Path,
    argv: &[String],
) -> Result<ExecOutcome, ExecRefusal> {
    let rt = match Runtime::detect() {
        Some(rt) => rt,
        None => return Err(ExecRefusal::NoRuntime),
    };

    // Record the dispatch only once a runtime is known to exist.
    let candidate = mint_candidate(store, task, argv)?;
    let cid = candidate.id;
    let proposed = append(
        store,
        task,
        Source::Rules,
        EventKind::Proposed { candidate },
    )?;
    let committed = append(
        store,
        task,
        Source::Rules,
        EventKind::Committed {
            candidate: cid,
            decision: Decision::Selected {
                value: cid,
                utility: Utility {
                    p_success: 0.5,
                    value: 0.0,
                    cost: 0.0,
                    ev: 0.0,
                },
                propensity: 1.0,
                evidence: vec![proposed],
            },
        },
    )?;
    let _ = committed;
    let dispatched = append(
        store,
        task,
        Source::Rules,
        EventKind::Dispatched {
            candidate: cid,
            in_sandbox: true,
        },
    )?;
    let _ = dispatched;

    let started = std::time::Instant::now();
    let args = container_argv(rt, workspace, argv);
    let output = Command::new(&args[0]).args(&args[1..]).output();
    let duration_ms = started.elapsed().as_millis() as u64;

    match output {
        Ok(out) => {
            let exit_code = out.status.code().unwrap_or(-1);
            let log_digest = Hash::of(&[&out.stdout, &out.stderr]);
            append(
                store,
                task,
                Source::Rules,
                EventKind::Executed {
                    candidate: cid,
                    exit_code: Some(exit_code),
                    log_digest: Some(log_digest),
                    duration_ms,
                },
            )?;
            Ok(ExecOutcome {
                exit_code,
                log_digest: Some(log_digest),
                duration_ms,
            })
        }
        Err(e) => {
            append(
                store,
                task,
                Source::Rules,
                EventKind::Unresolved {
                    candidate: cid,
                    reason: format!("container runtime failed to start: {e}"),
                },
            )?;
            Ok(ExecOutcome {
                exit_code: -1,
                log_digest: None,
                duration_ms,
            })
        }
    }
}

/// Why an execution was refused.
#[derive(Debug, thiserror::Error)]
pub enum ExecRefusal {
    #[error("no container runtime reachable (podman/docker): refusing to run on the host")]
    NoRuntime,
}

/// Mint the candidate for an argv: its identity commits to the argv and the action kind.
fn mint_candidate(store: &Store, task: &TaskId, argv: &[String]) -> Result<Candidate, ExecRefusal> {
    let args = Hash::of(&[argv.join("\u{0}").as_bytes()]);
    let parents = last_parent(store, task);
    Candidate::new(
        ActionKind::RunApprovedTest,
        Source::Rules,
        args,
        parents,
        vec![],
    )
    .map_err(|_| ExecRefusal::NoRuntime)
}

fn last_parent(store: &Store, task: &TaskId) -> Vec<tsr_domain::EventId> {
    store
        .by_task(task)
        .ok()
        .and_then(|v| v.into_iter().next_back().map(|(_, e)| vec![e.id]))
        .unwrap_or_default()
}

fn append(
    store: &Store,
    task: &TaskId,
    source: Source,
    kind: EventKind,
) -> Result<tsr_domain::EventId, ExecRefusal> {
    let parents = last_parent(store, task);
    let ev = Event::new(*task, parents, source, kind, crate::hook::now_rfc3339());
    let id = ev.id;
    store.append(&ev).map_err(|_| ExecRefusal::NoRuntime)?;
    Ok(id)
}

/// The candidate id an argv would produce (used by tests).
pub fn candidate_for(argv: &[String]) -> CandidateId {
    let args = Hash::of(&[argv.join("\u{0}").as_bytes()]);
    Candidate::new(
        ActionKind::RunApprovedTest,
        Source::Rules,
        args,
        vec![],
        vec![],
    )
    .expect("no conflicts")
    .id
}
