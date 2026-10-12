//! The batch run: one engine task per corpus task, per system (PRD §B9, §B11 M3).
//!
//! For each corpus task and each system the runner:
//!
//! 1. opens a fresh store (state isolation, B9) and records the head root at start;
//! 2. opens an engine task from the corpus task — the intent, the criteria from the oracle, the
//!    limits and the capabilities;
//! 3. provisions the disposable copy at the parent snapshot **through the [`Runner`]**, so nothing
//!    in this crate touches the host repository;
//! 4. drives the session through a [`Driver`];
//! 5. runs the oracle **through the [`Runner`]**, in the session's copy;
//! 6. closes the task and records the per-task result.
//!
//! The runner never runs a repository's tests on the host: every command goes through the Runner,
//! which in production is `ExecRunner` invoking `tsr exec -- <argv>` in the sealed sibling sandbox.

use std::path::{Path, PathBuf};

use tsr_corpus::{Limits, RunResult, Runner};
use tsr_domain::{Hash, TaskId};
use tsr_policy::gate::Grant;
use tsr_policy::profile::Profile;
use tsr_store::Store;

use crate::driver::{Driver, DriverError, Mode};
use crate::results::{OracleVerdict, Outcome, TaskResult};
use crate::session::SessionCtx;
use crate::task::CorpusTask;

/// Why a batch run failed.
#[derive(Debug, thiserror::Error)]
pub enum RunError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("store: {0}")]
    Store(#[from] tsr_store::StoreError),
    #[error("session: {0}")]
    Session(#[from] crate::session::SessionError),
    #[error("driver: {0}")]
    Driver(#[from] DriverError),
    #[error("profile: {0}")]
    Profile(#[from] tsr_policy::profile::ProfileError),
    /// The disposable copy was never materialised: the provisioning run exited non-zero (D4).
    #[error("provisioning failed: `{argv}` exited {exit}")]
    ProvisioningFailed { argv: String, exit: i32 },
    /// The task's snapshot is not a hex object id (R1): a snapshot from `tasks.jsonl` is untrusted
    /// and must never become a path the batch runs from.
    #[error("task {task}: unsafe snapshot {snapshot:?}: a snapshot must be a hex object id")]
    UnsafeSnapshot { task: String, snapshot: String },
    /// The task id is not a safe path component (R2, D1): `run_one` joins it into
    /// `out/state/<id>` and `out/copies/<id>`, so an id that could name a path outside `--out` is
    /// refused here as well as where `tasks.jsonl` is parsed.
    #[error("unsafe task id {id:?}: must match [A-Za-z0-9._-]{{1,128}} and not be `.` or `..`")]
    InvalidTaskId { id: String },
    /// A mutant's mutation diff could not be read or applied (D4): the copy is not the mutated
    /// tree, so it must not be driven.
    #[error("mutation diff {digest}: {reason}")]
    MutationDiff { digest: String, reason: String },
}

/// The configuration of a batch run.
pub struct BatchConfig {
    /// Where the results, the stores and the provisioned copies go.
    pub out: PathBuf,
    /// The RATCHET profile every gated action is checked against.
    pub profile: Profile,
    /// The limits every Runner call runs under.
    pub limits: Limits,
    /// The clock stamp every event carries (a batch run is deterministic).
    pub clock: String,
}

impl BatchConfig {
    /// A config with the default limits and a fixed clock.
    pub fn new(out: impl Into<PathBuf>, profile: Profile) -> Self {
        Self {
            out: out.into(),
            profile,
            limits: Limits::new(60_000, 1 << 20),
            clock: "2026-01-01T00:00:00Z".to_string(),
        }
    }
}

/// The capabilities a task is granted, derived from the corpus task and the profile.
///
/// The grant is the intersection the gate takes: the kinds the task may take, the read scope, the
/// allowlisted test ids and the budgets. It is built from the profile, so a task can never be
/// granted more than the operator's standing configuration allows.
pub fn grant_for(task: &CorpusTask, profile: &Profile) -> Grant {
    Grant {
        task: task_id(task),
        allowed_kinds: vec![
            tsr_domain::ActionKind::Inspect,
            tsr_domain::ActionKind::RunApprovedTest,
            tsr_domain::ActionKind::Generate,
            tsr_domain::ActionKind::ProposePatch,
            tsr_domain::ActionKind::RequestApproval,
            tsr_domain::ActionKind::Stop,
        ],
        read_scope_paths: profile.read_scope.paths.clone(),
        read_scope_rts_tools: profile.read_scope.rts_tools.clone(),
        allowed_test_ids: profile.tests.iter().map(|t| t.id.clone()).collect(),
        budgets: profile.budgets.clone(),
    }
}

/// The engine task id for a corpus task: blake3 of the corpus task id.
pub fn task_id(task: &CorpusTask) -> TaskId {
    TaskId(Hash::of(&[task.id.as_bytes()]))
}

/// The argv that provisions the disposable copy at a snapshot, run through the Runner.
///
/// The batch runner does not clone or check out anything itself: it asks the sealed sibling sandbox
/// to materialize the snapshot. In production this is `tsr exec -- tsr corpus materialize …`.
///
/// `repo` is the repository the snapshot lives in. It is passed explicitly (D4): without `--repo`
/// `materialize` defaults to `.`, which is the batch's own working directory and not the
/// repository the task was mined from.
pub fn provision_argv(repo: &Path, snapshot: &str, dest: &Path) -> Vec<String> {
    vec![
        "tsr".to_string(),
        "corpus".to_string(),
        "materialize".to_string(),
        "--repo".to_string(),
        repo.display().to_string(),
        "--snapshot".to_string(),
        snapshot.to_string(),
        "--out".to_string(),
        dest.display().to_string(),
    ]
}

/// Whether `snapshot` is a hex object id — the only shape a snapshot value may take (R1).
///
/// A snapshot from `tasks.jsonl` is untrusted: it becomes the working directory of the
/// provisioning run, so a value like `/etc` would make `/etc` the sandbox's writable mount
/// (PRD §B10: the disposable copy is the only writable mount). A git object id is 4–64 lowercase
/// hex characters; anything else — an absolute path, a `..` component, a ref name — is refused.
pub fn is_hex_object_id(snapshot: &str) -> bool {
    (4..=64).contains(&snapshot.len()) && snapshot.bytes().all(|b| b.is_ascii_hexdigit())
}

/// Apply a unified diff to the copy at `dest` with `git apply`.
///
/// The diff is the mutant's own mutation diff (D4). `git apply` is run with `-C dest` so it applies
/// against the copy, and it executes nothing from the repository. A diff that does not apply is an
/// error: a copy that is not the mutated tree must not be driven.
fn apply_diff(dest: &Path, diff: &[u8]) -> Result<(), RunError> {
    use std::io::Write;
    let mut child = std::process::Command::new("git")
        .arg("-C")
        .arg(dest)
        .args(["apply", "--whitespace=nowarn", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| RunError::MutationDiff {
            digest: String::new(),
            reason: format!("could not run git apply: {e}"),
        })?;
    child
        .stdin
        .take()
        .expect("piped stdin")
        .write_all(diff)
        .map_err(|e| RunError::MutationDiff {
            digest: String::new(),
            reason: format!("writing the diff: {e}"),
        })?;
    let out = child
        .wait_with_output()
        .map_err(|e| RunError::MutationDiff {
            digest: String::new(),
            reason: format!("waiting for git apply: {e}"),
        })?;
    if !out.status.success() {
        return Err(RunError::MutationDiff {
            digest: String::new(),
            reason: format!(
                "git apply failed: {}",
                String::from_utf8_lossy(&out.stderr).trim()
            ),
        });
    }
    Ok(())
}

/// The batch-owned root the disposable copies are provisioned under.
///
/// Provisioning never runs from a path taken from `tasks.jsonl` (R1): the working directory is a
/// directory this batch owns, under `--out`, and the snapshot is only ever a value passed as an
/// argument to `materialize`.
pub fn copies_root(out: &Path) -> PathBuf {
    out.join("copies")
}

/// The argv that runs the flipped tests, run through the Runner.
///
/// `dir` is the working directory the oracle runs in — the session's copy, not the parent
/// snapshot (D3). The Runner sets it as the child's cwd, and the argv names it too so the sealed
/// sandbox can materialise the same tree.
pub fn flipped_tests_argv(dir: &Path, tests: &[String]) -> Vec<String> {
    vec![
        "tsr".to_string(),
        "corpus".to_string(),
        "oracle".to_string(),
        "--snapshot".to_string(),
        dir.display().to_string(),
        "--tests".to_string(),
        tests.join(","),
    ]
}

/// The argv that runs the full suite, run through the Runner.
///
/// `dir` is the working directory the oracle runs in — the session's copy, not the parent
/// snapshot (D3).
pub fn full_suite_argv(dir: &Path) -> Vec<String> {
    vec![
        "tsr".to_string(),
        "corpus".to_string(),
        "oracle".to_string(),
        "--snapshot".to_string(),
        dir.display().to_string(),
        "--full-suite".to_string(),
    ]
}

/// Run one corpus task under one system, returning its result.
///
/// `driver` drives the session; `runner` provisions the copy and runs the oracle. The store is
/// fresh per (task, system), so the state root at start is the empty store's root.
pub fn run_one(
    task: &CorpusTask,
    mode: Mode,
    driver: &dyn Driver,
    runner: &dyn Runner,
    config: &BatchConfig,
) -> Result<TaskResult, RunError> {
    // R2, D1: `run_one` is public, so a caller can hand it a `CorpusTask` that never went through
    // `read_tasks`. The id is joined into `out/state/<id>` and `out/copies/<id>` below, so it is
    // re-checked here — an absolute id would replace the base and a `..` id would escape `--out`.
    if !crate::task::is_safe_task_id(&task.id) {
        return Err(RunError::InvalidTaskId {
            id: task.id.clone(),
        });
    }
    let state_dir = config.out.join("state").join(&task.id);
    std::fs::create_dir_all(&state_dir)?;
    let store_path = state_dir.join(format!("{}.sqlite", mode.system()));
    // D10: re-running into an existing `--out` starts from an empty store. A
    // leftover store from a previous run would make `state_root_at_start` the
    // old head, so the run would not be the fresh, isolated session B9 requires.
    if store_path.exists() {
        std::fs::remove_file(&store_path)?;
    }
    let store = Store::open(&store_path)?;
    let state_root_at_start = store.head_root()?.to_hex();

    let grant = grant_for(task, &config.profile);
    let mut ctx = SessionCtx::new(
        store,
        task_id(task),
        config.profile.clone(),
        grant,
        config.clock.clone(),
    );

    // (2) Open the engine task from the corpus task.
    ctx.open_task()?;

    // (3) Provision the disposable copy at the parent snapshot, through the Runner.
    //
    // R1: the snapshot value from `tasks.jsonl` is untrusted. It is accepted only as a hex object
    // id, and the provisioning run's working directory is a batch-owned root under `--out`, never
    // the snapshot value itself. A value like `/etc` would otherwise make `/etc` the sandbox's
    // writable mount (PRD §B10: the disposable copy is the only writable mount).
    let snapshot = task.snapshot();
    if !is_hex_object_id(snapshot) {
        return Err(RunError::UnsafeSnapshot {
            task: task.id.clone(),
            snapshot: snapshot.to_string(),
        });
    }
    let copies_root = copies_root(&config.out);
    std::fs::create_dir_all(&copies_root)?;
    let dest = copies_root.join(&task.id).join(mode.system());
    let provision = runner.run(
        &copies_root,
        &provision_argv(Path::new(&task.repo), snapshot, &dest),
        &config.limits,
    );
    ctx.observe(tsr_domain::Observation::Evidence {
        digest: Hash::of(&[provision.stdout_digest.as_bytes()]),
        source: format!("provision:{}", provision.exit),
    })?;
    // D4: a non-zero provisioning exit means the copy was never materialised. Proceeding would run
    // the session and the oracle against a directory that does not hold the snapshot, so refuse.
    if provision.exit != 0 {
        return Err(RunError::ProvisioningFailed {
            argv: provision_argv(Path::new(&task.repo), snapshot, &dest).join(" "),
            exit: provision.exit,
        });
    }

    // (3b) A mutant's copy is its base snapshot materialised with the mutant's own diff applied
    // (D4). The base tree is unmutated, so the mutation diff is applied to `dest` before the
    // session sees it. The diff is read from the CAS by digest; a missing or unsafe digest is an
    // error, never a silently unmutated copy.
    //
    // The diff is applied only when provisioning actually materialised a copy at `dest`. A
    // stand-in runner (`ScriptedRunner`) answers exit 0 without creating a tree, so there is no
    // copy to mutate and nothing to apply the diff to; a real runner that exits 0 always creates
    // `dest`, and for it a missing or unsafe digest is still a hard error. Reading the CAS is
    // therefore gated on the copy existing, so a scripted run never needs a CAS it does not use.
    if let Some(diff_digest) = task.mutation_diff.as_deref() {
        if dest.exists() {
            let cas = tsr_corpus::mine::cas_dir(&config.out);
            let diff = tsr_corpus::mine::read_cas(&cas, diff_digest).map_err(|e| {
                RunError::MutationDiff {
                    digest: diff_digest.to_string(),
                    reason: e.to_string(),
                }
            })?;
            apply_diff(&dest, &diff)?;
        }
    }

    // (4) Drive the session.
    let report = match driver.drive(&mut ctx, task) {
        Ok(r) => r,
        Err(DriverError::Unavailable(reason)) => {
            // The driver cannot run here: record the task as unavailable and close it.
            ctx.close(format!("driver unavailable: {reason}"))?;
            let state_root_at_end = ctx.head_root()?.to_hex();
            return Ok(TaskResult {
                task: task.id.clone(),
                system: mode.system().to_string(),
                outcome: Outcome::Unavailable,
                generator_tokens: 0,
                tool_minutes: 0,
                wall_clock_ms: 0,
                family: task.family.clone(),
                lineage: task.lineage.clone(),
                state_root_at_start,
                state_root_at_end,
                denials: Vec::new(),
                oracle: None,
            });
        }
        Err(e) => return Err(e.into()),
    };

    // (5) Run the oracle through the Runner, in the session's copy (`dest`), not the parent
    // snapshot: the oracle judges the work the session actually did (D3).
    let flipped = runner.run(
        &dest,
        &flipped_tests_argv(&dest, &task.failing_tests()),
        &config.limits,
    );
    let suite = runner.run(&dest, &full_suite_argv(&dest), &config.limits);
    let oracle = oracle_verdict(&flipped, &suite);

    // (6) Close the task and record the result.
    ctx.close(format!("oracle:{}", oracle.accepted))?;
    let state_root_at_end = ctx.head_root()?.to_hex();

    let outcome = if oracle.accepted {
        Outcome::Accepted
    } else {
        Outcome::Rejected
    };

    Ok(TaskResult {
        task: task.id.clone(),
        system: mode.system().to_string(),
        outcome,
        generator_tokens: report.generator_tokens,
        tool_minutes: report.tool_minutes,
        wall_clock_ms: report.wall_clock_ms,
        family: task.family.clone(),
        lineage: task.lineage.clone(),
        state_root_at_start,
        state_root_at_end,
        denials: report.denials,
        oracle: Some(oracle),
    })
}

/// The oracle: the flipped tests pass and the full suite does not regress.
pub fn oracle_verdict(flipped: &RunResult, suite: &RunResult) -> OracleVerdict {
    let flipped_tests_pass = flipped.succeeded();
    let suite_regressed = !suite.succeeded();
    OracleVerdict {
        flipped_tests_pass,
        suite_regressed,
        accepted: flipped_tests_pass && !suite_regressed,
        exit_code: flipped.exit,
    }
}

/// Run every task under one system, returning the results in task order.
pub fn run_all(
    tasks: &[CorpusTask],
    mode: Mode,
    driver_for: impl Fn(&CorpusTask) -> Box<dyn Driver>,
    runner: &dyn Runner,
    config: &BatchConfig,
) -> Result<Vec<TaskResult>, RunError> {
    let mut out = Vec::with_capacity(tasks.len());
    for task in tasks {
        let driver = driver_for(task);
        out.push(run_one(task, mode, driver.as_ref(), runner, config)?);
    }
    Ok(out)
}
