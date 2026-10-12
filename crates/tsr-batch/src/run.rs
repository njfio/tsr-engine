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
pub fn provision_argv(snapshot: &str, dest: &Path) -> Vec<String> {
    vec![
        "tsr".to_string(),
        "corpus".to_string(),
        "materialize".to_string(),
        "--snapshot".to_string(),
        snapshot.to_string(),
        "--out".to_string(),
        dest.display().to_string(),
    ]
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
    let dest = config.out.join("copies").join(&task.id).join(mode.system());
    let provision = runner.run(
        Path::new(&task.snapshot()),
        &provision_argv(task.snapshot(), &dest),
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
            argv: provision_argv(task.snapshot(), &dest).join(" "),
            exit: provision.exit,
        });
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
