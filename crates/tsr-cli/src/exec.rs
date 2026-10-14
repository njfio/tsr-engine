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
//!
//! The dispatch is honest about *which* candidate ran. When the hook has just committed a
//! `RunApprovedTest` candidate for this argv (the agent forecast first, then called
//! `tsr exec -- …`), `tsr exec` dispatches **that** candidate rather than minting a second one, so
//! one run is one candidate: `Proposed` → `Forecast` → `Committed` → `Dispatched` → `Executed`.
//! Only when no such committed candidate exists does it mint one as before.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use tsr_domain::{
    ActionKind, Candidate, CandidateId, Decision, Event, EventKind, Hash, Source, TaskId, Utility,
};
use tsr_store::Store;

/// The exit code for "no container runtime reachable" (B4).
pub const EXIT_NO_RUNTIME: i32 = 3;

/// The default container image, used when `TSR_EXEC_IMAGE` is unset.
pub const DEFAULT_IMAGE: &str = "docker.io/library/debian:stable-slim";

/// The container image to run in: `TSR_EXEC_IMAGE` if it names a plausible image, else
/// [`DEFAULT_IMAGE`].
///
/// The value is placed in the runtime's argv immediately before the user argv, so it must never be
/// something the runtime would parse as a *flag*: `TSR_EXEC_IMAGE=--privileged` would otherwise
/// disable the container's isolation while the log still recorded `Dispatched { in_sandbox: true }`.
/// A value that is not a plausible image reference (a leading `-`, whitespace, control characters,
/// an empty string) is refused and the default is used instead.
pub fn image() -> String {
    match std::env::var("TSR_EXEC_IMAGE") {
        Ok(v) if is_plausible_image(&v) => v,
        _ => DEFAULT_IMAGE.to_string(),
    }
}

/// Whether `s` is a plausible container image reference: it starts with an alphanumeric and the
/// rest is drawn from the characters an image reference may contain (`[A-Za-z0-9._/:@-]`). This
/// rejects anything the runtime could read as an option (`-...`), as two arguments (whitespace), or
/// as an empty image.
pub fn is_plausible_image(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphanumeric() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '/' | ':' | '@' | '-'))
}

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
    /// its binary exists on `PATH` **and its daemon answers** — a binary that cannot run, or a CLI
    /// with no daemon behind it, is no runtime. The probe asks the daemon (`info`), not the binary
    /// (`--version`): a host with the docker CLI but no daemon must refuse, not record a run that
    /// never happened (repair item 3, PRD §B10).
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

/// Probe a runtime's **daemon** with `info`; a non-zero exit, a spawn failure, or a daemon that
/// does not answer within [`PROBE_TIMEOUT`] means "not reachable". `--version` only proves the CLI
/// is installed, which is exactly the false positive this repair removes: a docker CLI with no
/// daemon answers `--version` and fails `info`.
fn probe(rt: Runtime) -> bool {
    let mut child = match Command::new(rt.binary())
        .arg("info")
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(child) => child,
        Err(_) => return false,
    };
    // A short timeout: a daemon that hangs must not hang `tsr exec`. Poll until the deadline, then
    // kill and treat as unreachable.
    let deadline = std::time::Instant::now() + PROBE_TIMEOUT;
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return status.success(),
            Ok(None) => {
                if std::time::Instant::now() >= deadline {
                    let _ = child.kill();
                    let _ = child.wait();
                    return false;
                }
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            Err(_) => return false,
        }
    }
}

/// How long the daemon probe may take before the runtime is treated as unreachable.
const PROBE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// The sealed-container invocation for a runtime and an argv. Pure: it builds the argument vector
/// and never runs anything, so it can be asserted on without a runtime present.
///
/// The container is a *sibling*: the workspace is bind-mounted at the same path, the root is
/// read-only, the network is off, and the cgroup limits are fixed. The image is [`image`].
///
/// The store's directory is mounted **read-only** over the same path whenever it lies inside the
/// workspace, so a command inside the sandbox cannot rewrite the event log or the session file
/// (repair item 3, PRD §B10). The store directory is `TSR_DB`'s parent if set, else
/// `<workspace>/.tsr`; a store outside the workspace is not mounted at all.
pub fn container_argv(rt: Runtime, workspace: &Path, argv: &[String]) -> Vec<String> {
    container_argv_with_store(rt, workspace, argv, None)
}

/// [`container_argv`] with the store path injected, so the read-only mount can be asserted on
/// without touching the ambient `TSR_DB`. `store` is the store *file*; its parent is mounted.
pub fn container_argv_with_store(
    rt: Runtime,
    workspace: &Path,
    argv: &[String],
    store: Option<&Path>,
) -> Vec<String> {
    let ws_path = crate::store_path::resolve_workspace(workspace);
    let ws = ws_path.display().to_string();
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
    ];
    // The store's directory, read-only, after the workspace mount and before the image (docker
    // options must precede the image). Only when it lies inside the workspace: a store outside it
    // is not reachable from the container anyway, and mounting it would widen the sandbox.
    if let Some(dir) = store_dir_inside_workspace(&ws_path, store) {
        let d = dir.display().to_string();
        v.push("--mount".to_string());
        v.push(format!("type=bind,src={d},dst={d},readonly"));
    }
    v.push(image());
    v.extend(argv.iter().cloned());
    v
}

/// The store's directory when it lies inside `workspace`, else `None`.
///
/// The store file is `TSR_DB` if set, else `<workspace>/.tsr/tsr.db`; the directory mounted is its
/// parent. A store whose directory *is* the workspace is refused by the caller (exit 2) — mounting
/// the workspace read-only over itself would contradict the writable workspace mount.
pub fn store_dir_inside_workspace(workspace: &Path, store: Option<&Path>) -> Option<PathBuf> {
    let ws = crate::store_path::resolve_workspace(workspace);
    let db = match store {
        Some(p) => p.to_path_buf(),
        None => match std::env::var_os("TSR_DB") {
            Some(p) if !p.is_empty() => PathBuf::from(p),
            _ => ws.join(crate::store_path::DEFAULT_DB),
        },
    };
    // Resolve the store path once (relative → workspace, `.`/`..` normalised, longest existing
    // ancestor canonicalised so symlinks resolve) before comparing it with the workspace. Comparing
    // the raw path would drop the read-only mount for every relative or symlinked spelling
    // (review (5)).
    let db = crate::store_path::resolve_store_path(&db, &ws);
    let dir = db.parent()?.to_path_buf();
    if dir == ws {
        return None;
    }
    if dir.starts_with(&ws) {
        Some(dir)
    } else {
        None
    }
}
/// What a [`Runner`] observed: the process exit code and the combined log bytes.
///
/// The log is the raw stdout+stderr the container produced; the caller hashes it into the
/// `Executed` event's `log_digest`. Keeping the bytes (not a pre-computed digest) lets a fake runner
/// script a log and lets the harness compare two runs structurally.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunOutput {
    pub exit_code: i32,
    pub log: Vec<u8>,
}

/// The thing that actually runs the sealed command. Factored out so `exec::run` is drivable without
/// a container: the real runner builds the docker/podman argv and spawns it, a fake runner returns a
/// scripted exit code and log.
///
/// Two methods, because the refusal must be decided *before* anything is recorded (B4: no runtime →
/// refuse, never fall back to the host) while a container that is present but fails to *start* is a
/// recorded `Unresolved`, not a refusal:
/// - [`Runner::available`] is called first; `Err` here means "cannot run at all" and `run_with`
///   records nothing.
/// - [`Runner::run`] is called after `Dispatched` is recorded; `Err` here means the runtime was
///   present but the container could not be started, and the dispatch is recorded `Unresolved`.
pub trait Runner {
    /// Whether this runner can run at all. Called before any event is appended, so a refusal here
    /// leaves the log untouched.
    fn available(&self) -> Result<(), ExecRefusal>;

    /// Run `argv` in `workspace`, returning the exit code and the combined log bytes. `Err` means
    /// the runtime was present but the container could not be started.
    fn run(&self, workspace: &Path, argv: &[String]) -> Result<RunOutput, ExecRefusal>;
}

/// The real runner: a sibling container via Podman or Docker, sealed exactly as [`container_argv`]
/// builds it. Constructed only once a runtime has been detected, so [`Runner::available`] is a
/// no-op and the refusal is decided at construction ([`RealRunner::detect`]).
#[derive(Debug, Clone, Copy)]
pub struct RealRunner {
    runtime: Runtime,
}

impl RealRunner {
    /// Detect a reachable runtime, or refuse. This is the refusal decision: it happens before
    /// `run_with` records anything.
    pub fn detect() -> Result<RealRunner, ExecRefusal> {
        Runtime::detect()
            .map(|runtime| RealRunner { runtime })
            .ok_or(ExecRefusal::NoRuntime)
    }

    /// The runtime this runner will use.
    pub fn runtime(&self) -> Runtime {
        self.runtime
    }
}

impl Runner for RealRunner {
    fn available(&self) -> Result<(), ExecRefusal> {
        // A `RealRunner` only exists once a runtime was detected.
        Ok(())
    }

    fn run(&self, workspace: &Path, argv: &[String]) -> Result<RunOutput, ExecRefusal> {
        // Create the store's directory before the mount, so a read-only bind mount of a
        // not-yet-existing directory cannot fail the container start (repair item 3).
        if let Some(dir) = store_dir_inside_workspace(workspace, None) {
            std::fs::create_dir_all(&dir)
                .map_err(|e| ExecRefusal::StartFailed(format!("cannot create store dir: {e}")))?;
        }
        let args = container_argv(self.runtime, workspace, argv);
        let out = Command::new(&args[0])
            .args(&args[1..])
            .output()
            .map_err(|e| ExecRefusal::StartFailed(e.to_string()))?;
        let mut log = out.stdout;
        log.extend_from_slice(&out.stderr);
        // A runtime that exits 125/126/127 from `run` did not start the container: that is not the
        // command's exit code, so it must never be recorded as `Executed` (repair item 3).
        let code = out.status.code().unwrap_or(-1);
        if matches!(code, 125..=127) {
            return Err(ExecRefusal::StartFailed(format!(
                "container runtime exited {code} without starting the container: {}",
                String::from_utf8_lossy(&log).trim()
            )));
        }
        Ok(RunOutput {
            exit_code: code,
            log,
        })
    }
}

/// A runner that returns a scripted exit code and log instead of running a container. Exported (not
/// `#[cfg(test)]`) so the integration harness in `tests/m0_harness.rs` can inject it into
/// [`run_with`] and drive the whole dispatch lifecycle without a container runtime.
#[derive(Debug, Clone)]
pub struct FakeRunner {
    exit_code: i32,
    log: Vec<u8>,
    /// When set, [`Runner::run`] refuses as if the container could not be started.
    start_error: Option<String>,
}

impl FakeRunner {
    /// A fake that "runs" and returns `exit_code` with `log` as the combined output.
    pub fn new(exit_code: i32, log: impl Into<Vec<u8>>) -> FakeRunner {
        FakeRunner {
            exit_code,
            log: log.into(),
            start_error: None,
        }
    }

    /// A fake whose container cannot be started: the dispatch is recorded `Unresolved`.
    pub fn failing_to_start(reason: impl Into<String>) -> FakeRunner {
        FakeRunner {
            exit_code: -1,
            log: Vec::new(),
            start_error: Some(reason.into()),
        }
    }
}

impl Runner for FakeRunner {
    fn available(&self) -> Result<(), ExecRefusal> {
        Ok(())
    }

    fn run(&self, _workspace: &Path, _argv: &[String]) -> Result<RunOutput, ExecRefusal> {
        match &self.start_error {
            Some(reason) => Err(ExecRefusal::StartFailed(reason.clone())),
            None => Ok(RunOutput {
                exit_code: self.exit_code,
                log: self.log.clone(),
            }),
        }
    }
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
/// then `Executed` (or, if the runtime itself failed to start the container, `Unresolved` and a
/// `StartFailed` refusal so the caller exits 3 — never `Executed`).
///
/// If the task already holds a `Committed`-but-not-`Dispatched` `RunApprovedTest` candidate whose
/// args digest matches this argv (the hook committed it a moment ago), that candidate is dispatched
/// instead of a freshly minted one.
pub fn run(
    store: &Store,
    task: &TaskId,
    workspace: &Path,
    candidate: Option<CandidateId>,
    argv: &[String],
) -> Result<ExecOutcome, ExecRefusal> {
    // A store whose directory *is* the workspace cannot be mounted read-only over itself: refuse
    // (exit 2) before anything is recorded (repair item 3).
    if store_dir_is_workspace(workspace) {
        return Err(ExecRefusal::StoreIsWorkspace);
    }
    // The refusal is decided here, before `run_with` records anything: no runtime → exit 3 and an
    // untouched log.
    let runner = RealRunner::detect()?;
    run_with(store, task, workspace, candidate, argv, &runner)
}

/// Whether the store file's parent is the workspace itself (`TSR_DB=<workspace>/tsr.db`), which
/// cannot be mounted read-only over the writable workspace mount.
pub fn store_dir_is_workspace(workspace: &Path) -> bool {
    let ws = crate::store_path::resolve_workspace(workspace);
    let db = match std::env::var_os("TSR_DB") {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        _ => ws.join(crate::store_path::DEFAULT_DB),
    };
    // Resolve the store path the same way `store_dir_inside_workspace` does (review (5)): otherwise
    // `TSR_DB=tsr.db` would not be refused.
    let db = crate::store_path::resolve_store_path(&db, &ws);
    db.parent().map(|d| d == ws).unwrap_or(false)
}

/// [`run`] with the container invocation injected. The harness drives this with a [`FakeRunner`] to
/// exercise the whole dispatch lifecycle without a container runtime; production calls it with a
/// [`RealRunner`].
///
/// Refusal semantics are unchanged: if `runner.available()` refuses, **nothing is recorded** and the
/// refusal is returned. Otherwise it records `Dispatched { in_sandbox: true }` then `Executed` (or
/// `Unresolved` if the runner could not start the container).
pub fn run_with(
    store: &Store,
    task: &TaskId,
    workspace: &Path,
    candidate: Option<CandidateId>,
    argv: &[String],
    runner: &dyn Runner,
) -> Result<ExecOutcome, ExecRefusal> {
    // Refuse before recording anything.
    runner.available()?;

    // Record the dispatch only once the runner is known to be able to run.
    let cid = match candidate {
        // (b) The agent named the candidate: it must be a candidate of this task that has a forecast
        // and is not yet dispatched. An unknown, unforecast or already-dispatched id is refused with
        // exit 2 and nothing is recorded or run.
        Some(cid) => {
            check_named_candidate(store, task, cid)?;
            cid
        }
        // (d) No `--candidate`: mint a candidate with no forecast and say so on stderr. The log is
        // honest about it, and reconstruct may reject it — that is the intended signal.
        None => {
            eprintln!(
                "tsr: exec: no --candidate given; this run has no forecast (the candidate is minted \
                 here and reconstruct may reject it)"
            );
            let candidate = mint_candidate(argv)?;
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
            cid
        }
    };

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
    let output = runner.run(workspace, argv);
    let duration_ms = started.elapsed().as_millis() as u64;

    match output {
        Ok(out) => {
            let exit_code = out.exit_code;
            let log_digest = Hash::of(&[&out.log]);
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
            // Pass the container's output through to the caller, so an agent running
            // `tsr exec -- cargo test …` sees the result. Best-effort: a closed pipe is not an error.
            let _ = std::io::stdout().write_all(&out.log);
            let _ = std::io::stdout().flush();
            Ok(ExecOutcome {
                exit_code,
                log_digest: Some(log_digest),
                duration_ms,
            })
        }
        Err(e) => {
            let reason = format!("container runtime failed to start: {e}");
            append(
                store,
                task,
                Source::Rules,
                EventKind::Unresolved {
                    candidate: cid,
                    reason: reason.clone(),
                },
            )?;
            // The dispatch is recorded `Unresolved`, never `Executed`, and the caller exits 3: a
            // container that did not start is not a run that happened (repair item 3).
            Err(ExecRefusal::StartFailed(reason))
        }
    }
}

/// Why an execution was refused.
#[derive(Debug, thiserror::Error)]
pub enum ExecRefusal {
    #[error("no container runtime reachable (podman/docker): refusing to run on the host")]
    NoRuntime,
    /// The runtime was present but the container could not be started. This is *not* a refusal to
    /// run at all: the dispatch is recorded `Unresolved`.
    #[error("container runtime failed to start: {0}")]
    StartFailed(String),
    /// `TSR_DB` names a store directly in the workspace, whose directory cannot be mounted
    /// read-only over the writable workspace mount. Refused with exit 2, nothing recorded.
    #[error("TSR_DB must not be the workspace itself: refusing to mount the workspace read-only")]
    StoreIsWorkspace,
    /// `--candidate <id>` named a candidate that is not a candidate of this task, has no forecast,
    /// or is already dispatched. Refused with exit 2, nothing recorded and nothing run.
    #[error("{0}")]
    UnknownCandidate(String),
}

/// The candidate id an argv would produce: the same identity the hook mints for the equivalent
/// `Bash` tool call, because both go through [`crate::hook::mint_candidate`] with the same
/// `(kind, args)` pair. For `Bash` the args string is the command, so the argv is joined by spaces.
pub fn candidate_for(argv: &[String]) -> CandidateId {
    mint_candidate(argv).expect("no conflicts").id
}

/// Mint the candidate for an argv through the shared hook path, so `tsr exec` and the hook agree on
/// identity for the same command. The args string is the argv joined by spaces — the command string
/// the hook digests for a `Bash` call.
fn mint_candidate(argv: &[String]) -> Result<Candidate, ExecRefusal> {
    crate::hook::mint_candidate(
        ActionKind::RunApprovedTest,
        &crate::hook::canonical_args(argv),
    )
    .map_err(|_| ExecRefusal::NoRuntime)
}

/// Check that `cid` is a candidate of `task` that has a forecast and is not yet dispatched, and
/// refuse with exit 2 naming the reason otherwise (operator decision (b)).
///
/// "Has a forecast" means a `Forecast { candidate: cid }` event exists for it. "Not yet dispatched"
/// means no `Dispatched { candidate: cid }` follows the newest `Committed` for it — the same
/// per-occurrence rule [`pending_commitment`] uses, so a re-committed candidate is dispatchable
/// again.
fn check_named_candidate(
    store: &Store,
    task: &TaskId,
    cid: CandidateId,
) -> Result<(), ExecRefusal> {
    let events = store
        .by_task(task)
        .map_err(|e| ExecRefusal::UnknownCandidate(format!("cannot read the log: {e}")))?;
    let hex = cid.0.to_hex();

    let mut proposed = false;
    let mut forecast = false;
    let mut committed = false;
    let mut dispatched = false;
    for (_, e) in &events {
        match &e.kind {
            EventKind::Proposed { candidate } if candidate.id == cid => proposed = true,
            EventKind::Forecast { candidate, .. } if *candidate == cid => forecast = true,
            EventKind::Committed { candidate, .. } if *candidate == cid => {
                committed = true;
                dispatched = false;
            }
            EventKind::Dispatched { candidate, .. } if *candidate == cid => dispatched = true,
            _ => {}
        }
    }

    if !proposed {
        return Err(ExecRefusal::UnknownCandidate(format!(
            "--candidate {hex} is not a candidate of this task"
        )));
    }
    if !forecast {
        return Err(ExecRefusal::UnknownCandidate(format!(
            "--candidate {hex} has no forecast"
        )));
    }
    if !committed {
        return Err(ExecRefusal::UnknownCandidate(format!(
            "--candidate {hex} has not been committed"
        )));
    }
    if dispatched {
        return Err(ExecRefusal::UnknownCandidate(format!(
            "--candidate {hex} is already dispatched"
        )));
    }
    Ok(())
}

/// The candidate of the most recent `Committed` for `(kind, args_digest)` that has no `Dispatched`
/// after it — the commitment this dispatch is meant to fulfil, if any.
///
/// The walk is per *occurrence*, in sequence order, not per id. A candidate id is content-addressed
/// by `(kind, args, parents)`, so the second run of the same command mints the same id as the first:
/// treating "this id has a `Dispatched`" as "dispatched" would make the second run mint a fresh
/// candidate with no forecast (the defect the operator's real record shows at events 19–20). A
/// `Committed` is pending again once a later `Committed` for the same digest follows the
/// `Dispatched`, exactly as a re-`Proposed` candidate is pending again.
///
/// Matching is on the committed candidate's `(kind, args digest)`, not on the id alone: the id also
/// commits to the parents, which differ between the hook's mint and a fresh one.
pub fn pending_commitment(
    store: &Store,
    task: &TaskId,
    kind: ActionKind,
    args_digest: Hash,
) -> Option<CandidateId> {
    let events = store.by_task(task).ok()?;
    let mut pending: Option<CandidateId> = None;
    for (_, e) in &events {
        match &e.kind {
            EventKind::Proposed { candidate } => {
                if candidate.kind == kind && candidate.args == args_digest {
                    // A re-proposal of the same candidate is pending again.
                    pending = Some(candidate.id);
                }
            }
            EventKind::Committed { candidate, .. } => {
                if pending == Some(*candidate) {
                    // Already pending: the commitment does not change which occurrence is pending.
                } else if let Some(c) = events.iter().find_map(|(_, p)| match &p.kind {
                    EventKind::Proposed { candidate: p } if p.id == *candidate => Some(p),
                    _ => None,
                }) {
                    if c.kind == kind && c.args == args_digest {
                        pending = Some(*candidate);
                    }
                }
            }
            EventKind::Dispatched { candidate, .. } => {
                if pending == Some(*candidate) {
                    pending = None;
                }
            }
            _ => {}
        }
    }
    pending
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_image_when_unset() {
        // The default is a constant, so this holds regardless of the ambient environment.
        assert_eq!(DEFAULT_IMAGE, "docker.io/library/debian:stable-slim");
    }
    #[test]
    fn image_reference_validation_rejects_flags_and_junk() {
        // A flag-looking value would be parsed by the runtime as an option, not an image, and would
        // escape the seal while the log still recorded `in_sandbox: true`.
        for bad in [
            "--privileged",
            "--volume=/:/host",
            "--pid=host",
            "img1 --privileged",
            "",
            " ",
            "-",
            "debian:stable-slim\n--privileged",
        ] {
            assert!(
                !is_plausible_image(bad),
                "{bad:?} must not be accepted as an image reference"
            );
        }
        for good in [
            "docker.io/library/debian:stable-slim",
            "debian:stable-slim",
            "debian",
            "registry.example.com:5000/team/img@sha256:abc",
        ] {
            assert!(
                is_plausible_image(good),
                "{good:?} is a plausible image reference"
            );
        }
    }

    #[test]
    fn container_argv_image_slot_is_never_flag_looking() {
        // Whatever `image()` returns, the element it contributes to the argv must not start with
        // `-` (which the runtime would read as an option) and must not contain whitespace.
        let img = image();
        assert!(
            !img.starts_with('-'),
            "image slot must not be a flag: {img:?}"
        );
        assert!(!img.contains(char::is_whitespace), "image slot: {img:?}");
        let v = container_argv(Runtime::Docker, Path::new("/work"), &["echo".to_string()]);
        // The image sits immediately before the argv; the read-only store mount may precede it.
        let img_at = v.iter().position(|a| a == &img).expect("image in argv");
        assert!(!v[img_at].starts_with('-'));
    }

    #[test]
    fn container_argv_uses_the_image_and_passes_argv_verbatim() {
        let argv = vec!["cargo".to_string(), "test".to_string()];
        let v = container_argv(Runtime::Docker, Path::new("/work"), &argv);
        assert_eq!(v[0], "docker");
        assert_eq!(v[1], "run");
        // The image sits immediately before the argv, and the argv is passed verbatim.
        let img_at = v.iter().position(|a| a == &image()).expect("image in argv");
        assert_eq!(v[img_at], image());
        assert_eq!(&v[img_at + 1..], &argv[..]);
    }

    #[test]
    fn candidate_for_matches_the_hook_for_the_same_command() {
        let argv: Vec<String> = ["cargo", "test", "--offline", "-p", "parser"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let cmd = "cargo test --offline -p parser";
        assert_eq!(
            candidate_for(&argv),
            crate::hook::candidate_for_action(ActionKind::RunApprovedTest, cmd)
                .expect("hook candidate"),
            "tsr exec and the hook must mint the same candidate for the same command"
        );
    }

    /// The operator's real `tsr hook --selftest` record from 4d8d7b8 — pinned at
    /// tests/fixtures/m0/real-record-4d8d7b8.events.jsonl because fixtures/m0/session.events.jsonl is
    /// regenerated on a host after this repair and no longer holds the defective sequence — events
    /// 1–18: two `Read`s, then a `tsr forecast` + PreToolUse Bash for `tsr exec -- cargo test
    /// --offline -p parser` (6–8), the dispatch and execution (9–10), the PostToolUse the hook
    /// wrongly proposed and committed (11–12), an Edit (13–15), and the *second* run of the same
    /// command (16–18). The candidate id is content-addressed, so events 6–8 and 16–18 name the
    /// same id `e07bad41…`; the first occurrence was dispatched at 9, the second was not.
    const REAL_LISTING_EVENTS_1_TO_18: &str = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/m0/real-record-4d8d7b8.events.jsonl"
    ));

    /// Build a store holding the operator's real record, truncated to the first `n` events.
    fn store_with_real_listing(n: usize) -> (Store, TaskId) {
        let store = Store::open_in_memory().expect("in-memory store");
        let mut task = None;
        for line in REAL_LISTING_EVENTS_1_TO_18.lines().take(n) {
            let row: serde_json::Value = serde_json::from_str(line).expect("fixture line is JSON");
            let seq = row["seq"].as_i64().expect("seq");
            let ev: Event = serde_json::from_value(row["event"].clone()).expect("event");
            assert_eq!(store.append(&ev).expect("append"), seq, "seq preserved");
            task = Some(ev.task);
        }
        (store, task.expect("at least one event"))
    }

    #[test]
    fn pending_commitment_over_the_real_listing_returns_the_candidate_committed_at_18() {
        let (store, task) = store_with_real_listing(18);
        let digest = crate::hook::args_digest(
            ActionKind::RunApprovedTest,
            "cargo test --offline -p parser",
        );
        let got = pending_commitment(&store, &task, ActionKind::RunApprovedTest, digest);
        let want = CandidateId(
            Hash::from_hex("e07bad41b1e4c9dbfd308a93a53ebd1e41019f8651e715be100c8ea770ff91c3")
                .expect("hex"),
        );
        assert_eq!(
            got,
            Some(want),
            "after events 1-18 the pending commitment is the one committed at event 18"
        );
    }

    #[test]
    fn pending_commitment_is_none_before_any_commitment_and_after_a_dispatch() {
        let digest = crate::hook::args_digest(
            ActionKind::RunApprovedTest,
            "cargo test --offline -p parser",
        );
        // Events 1–5: the two Reads; the run_approved_test candidate is not proposed yet.
        let (store, task) = store_with_real_listing(5);
        assert_eq!(
            pending_commitment(&store, &task, ActionKind::RunApprovedTest, digest),
            None,
            "no commitment exists before event 6"
        );
        // Events 1–6: proposed but not committed — a `Proposed` is pending (01b's rule), so the
        // lookup already names it; the commitment at 8 does not change which occurrence is pending.
        let (store, task) = store_with_real_listing(6);
        assert_eq!(
            pending_commitment(&store, &task, ActionKind::RunApprovedTest, digest),
            Some(CandidateId(
                Hash::from_hex("e07bad41b1e4c9dbfd308a93a53ebd1e41019f8651e715be100c8ea770ff91c3")
                    .expect("hex")
            )),
            "a Proposed is pending before it is committed"
        );
        // Events 1–10: the first occurrence was committed at 8 and dispatched at 9.
        let (store, task) = store_with_real_listing(10);
        assert_eq!(
            pending_commitment(&store, &task, ActionKind::RunApprovedTest, digest),
            None,
            "the only commitment was dispatched at event 9"
        );
        // Events 1–12: the hook's spurious PostToolUse commitment (11–12) is for the same digest,
        // so it is pending again — the second run must dispatch it, not mint a third candidate.
        let (store, task) = store_with_real_listing(12);
        assert_eq!(
            pending_commitment(&store, &task, ActionKind::RunApprovedTest, digest),
            Some(CandidateId(
                Hash::from_hex("e07bad41b1e4c9dbfd308a93a53ebd1e41019f8651e715be100c8ea770ff91c3")
                    .expect("hex")
            )),
            "the commitment at 12 is pending again after the dispatch at 9"
        );
        // A different digest never matches.
        let other = crate::hook::args_digest(ActionKind::RunApprovedTest, "cargo test -p other");
        let (store, task) = store_with_real_listing(18);
        assert_eq!(
            pending_commitment(&store, &task, ActionKind::RunApprovedTest, other),
            None,
            "a different args digest has no commitment"
        );
    }

    /// `run_with` with a `FakeRunner` records the whole dispatch lifecycle without a container:
    /// `Dispatched { in_sandbox: true }` then `Executed` with the scripted exit code and log digest.
    #[test]
    fn run_with_a_fake_runner_records_dispatched_then_executed() {
        let store = Store::open_in_memory().expect("in-memory store");
        let task = TaskId(Hash::of(&[b"fake-runner-task"]));
        let argv: Vec<String> = ["cargo", "test", "--offline", "-p", "parser"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let fake = FakeRunner::new(0, b"ok\n".to_vec());

        let outcome = run_with(&store, &task, Path::new("/work"), None, &argv, &fake)
            .expect("a fake runner never refuses");

        assert_eq!(outcome.exit_code, 0);
        assert_eq!(
            outcome.log_digest,
            Some(Hash::of(&[b"ok\n".as_slice()])),
            "the log digest is the hash of the scripted log bytes"
        );

        let kinds: Vec<&'static str> = store
            .by_task(&task)
            .expect("by_task")
            .into_iter()
            .map(|(_, e)| match e.kind {
                EventKind::Proposed { .. } => "proposed",
                EventKind::Committed { .. } => "committed",
                EventKind::Dispatched { .. } => "dispatched",
                EventKind::Executed { .. } => "executed",
                _ => "other",
            })
            .collect();
        assert_eq!(
            kinds,
            vec!["proposed", "committed", "dispatched", "executed"],
            "a fresh argv mints a candidate, commits it, dispatches it and records the execution"
        );
    }

    /// A fake runner whose container cannot be started records `Unresolved`, not `Executed`, and
    /// refuses with `StartFailed` so the CLI exits 3.
    #[test]
    fn run_with_a_fake_runner_that_cannot_start_records_unresolved() {
        let store = Store::open_in_memory().expect("in-memory store");
        let task = TaskId(Hash::of(&[b"fake-runner-unresolved"]));
        let argv: Vec<String> = ["cargo", "test"].iter().map(|s| s.to_string()).collect();
        let fake = FakeRunner::failing_to_start("boom");

        let err = run_with(&store, &task, Path::new("/work"), None, &argv, &fake)
            .expect_err("a start failure refuses with exit 3");
        assert!(matches!(err, ExecRefusal::StartFailed(_)));
        let has_unresolved = store
            .by_task(&task)
            .expect("by_task")
            .into_iter()
            .any(|(_, e)| matches!(e.kind, EventKind::Unresolved { .. }));
        assert!(
            has_unresolved,
            "a container that cannot start is Unresolved"
        );
        let has_executed = store
            .by_task(&task)
            .expect("by_task")
            .into_iter()
            .any(|(_, e)| matches!(e.kind, EventKind::Executed { .. }));
        assert!(
            !has_executed,
            "a container that cannot start is never Executed"
        );
    }

    /// A runner that refuses before running records **nothing** — the B4 refusal path.
    #[test]
    fn run_with_a_refusing_runner_records_nothing() {
        struct RefusingRunner;
        impl Runner for RefusingRunner {
            fn available(&self) -> Result<(), ExecRefusal> {
                Err(ExecRefusal::NoRuntime)
            }
            fn run(&self, _w: &Path, _a: &[String]) -> Result<RunOutput, ExecRefusal> {
                unreachable!("available() refuses first")
            }
        }

        let store = Store::open_in_memory().expect("in-memory store");
        let task = TaskId(Hash::of(&[b"refusing-runner"]));
        let argv: Vec<String> = ["cargo", "test"].iter().map(|s| s.to_string()).collect();

        let err = run_with(&store, &task, Path::new("/work"), None, &argv, &RefusingRunner)
            .expect_err("a refusing runner refuses");
        assert!(matches!(err, ExecRefusal::NoRuntime));
        assert_eq!(
            store.by_task(&task).expect("by_task").len(),
            0,
            "a refusal records nothing"
        );
    }
}
