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
    #[error("snapshot preparation: {0}")]
    Preparation(String),
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
    /// The task's repository is not a safe relative path (R1, F2): it is passed to `materialize` as
    /// `--repo`, so an absolute path or a `..` component would name a repository outside the store.
    #[error(
        "task {task}: unsafe repo {repo:?}: a repository must be a relative path with no `..`"
    )]
    UnsafeRepo { task: String, repo: String },
    /// The task id is not a safe path component (R2, D1): `run_one` joins it into
    /// `out/state/<id>` and `out/copies/<id>`, so an id that could name a path outside `--out` is
    /// refused here as well as where `tasks.jsonl` is parsed.
    #[error("unsafe task id {id:?}: must match [A-Za-z0-9._-]{{1,128}} and not be `.` or `..`")]
    InvalidTaskId { id: String },
    /// A mutant's mutation diff could not be read or applied (D4): the copy is not the mutated
    /// tree, so it must not be driven.
    #[error("mutation diff {digest}: {reason}")]
    MutationDiff { digest: String, reason: String },
    /// An oracle-test file path is not safe (D3, item 2): it is joined into the snapshot and the
    /// copy, so an absolute path, a `..` component or a symlink would name a file outside them.
    #[error(
        "unsafe oracle_test_files path {path:?}: must be relative with no `..` and no symlink"
    )]
    UnsafeOracleFile { path: String },
    /// An oracle-test file could not be restored into the copy (D3, item 2): the oracle cannot judge
    /// the copy with the snapshot's tests, so the task is refused rather than judged.
    #[error("oracle_test_files {path:?}: {reason}")]
    OracleFile { path: String, reason: String },
}

/// The configuration of a batch run.
pub struct BatchConfig {
    /// Where the results, the stores and the provisioned copies go.
    pub out: PathBuf,
    /// The corpus directory the manifest was read from: the directory holding `tasks.jsonl` and
    /// the corpus's own `cas/` (D4). A mutant's mutation diff is a CAS key in *this* directory's
    /// CAS, not the batch's `--out` CAS, so provisioning reads it from beside the manifest.
    pub corpus_dir: PathBuf,
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
        let out = out.into();
        Self {
            corpus_dir: out.clone(),
            out,
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

/// Whether `repo` is a safe repository path (R1, F2).
///
/// `repo` comes from `tasks.jsonl` and is passed to `materialize` as `--repo`, which runs
/// `git -C <repo> archive <snapshot>`. The traversal that matters is `..`: a `..` component lets a
/// task name a repository outside the store, so it is refused here — the same shape
/// `is_safe_descriptor_path` requires of a descriptor path. An absolute path is *not* refused: a
/// batch may legitimately be pointed at a repository by absolute path (the batch tests do exactly
/// that), and `git -C <abs>` reads only the repository it names. The empty string is refused (it
/// would name the batch's own working directory).
pub fn is_safe_repo(repo: &str) -> bool {
    if repo.is_empty() {
        return false;
    }
    Path::new(repo)
        .components()
        .all(|c| !matches!(c, std::path::Component::ParentDir))
}

/// Apply a unified diff to the copy at `dest` with `git apply`.
///
/// The diff is the mutant's own mutation diff (D4). `git apply` is run with `-C dest` so it applies
/// against the copy, and it executes nothing from the repository. A diff that does not apply is an
/// error: a copy that is not the mutated tree must not be driven.
pub(crate) fn apply_diff(dest: &Path, diff: &[u8]) -> Result<(), RunError> {
    use std::io::Write;
    let canonical = dest.canonicalize()?;
    let ceiling = canonical.parent().unwrap_or(&canonical);
    let mut child = std::process::Command::new("git")
        .arg("-C")
        .arg(dest)
        // A copy under another checkout must not discover that outer repository:
        // git apply otherwise treats the patch as outside its current prefix and skips it.
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .env("GIT_CEILING_DIRECTORIES", ceiling)
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

/// Make `path` absolute against the process working directory (N2).
///
/// `--out` and a mined `task.repo` may both be relative. The provisioning run is given the copies
/// root as its working directory, so a relative `dest` would resolve against that root and land the
/// copy in `brel/copies/brel/copies/…`; a relative `--repo` would resolve against the copies root
/// too, where the repository is not. Resolving both against the process working directory — the
/// directory the batch was launched from — makes every path absolute before use. An already-absolute
/// path is returned unchanged.
pub fn absolute(path: &Path) -> Result<PathBuf, RunError> {
    if path.is_absolute() {
        return Ok(path.to_path_buf());
    }
    let cwd = std::env::current_dir()?;
    Ok(cwd.join(path))
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
pub fn full_suite_argv(_dir: &Path) -> Vec<String> {
    vec!["cargo".to_string(), "test".to_string()]
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
    // R1 (F2): `repo` is untrusted too. It is passed to `materialize` as `--repo`, which runs
    // `git -C <repo> archive <snapshot>`. Absolute operator-selected paths are supported;
    // parent traversal and empty paths are refused. See is_safe_repo.
    if !is_safe_repo(&task.repo) {
        return Err(RunError::UnsafeRepo {
            task: task.id.clone(),
            repo: task.repo.clone(),
        });
    }
    // N2: every path is made absolute before use. `--out` may be relative, and the provisioning run
    // is given the copies root as its working directory; a relative `dest` would then resolve
    // *against* that root (`brel/copies/brel/copies/…`) and the copy would land in the wrong place.
    // The repository is resolved against the process working directory — the directory the batch was
    // launched from, which is where a relatively-mined `task.repo` is meaningful — so a repo mined by
    // relative path is still materialisable.
    let out_abs = absolute(&config.out)?;
    let copies_root = copies_root(&out_abs);
    std::fs::create_dir_all(&copies_root)?;
    let dest = copies_root.join(&task.id).join(mode.system());
    let repo_abs = absolute(Path::new(&task.repo))?;
    // D3 (item 2): the corpus's own CAS (beside `tasks.jsonl`), where the oracle runs' stdout is
    // read back from by the no-match guard below.
    let _cas_dir = tsr_corpus::mine::cas_dir(&config.corpus_dir);
    // R1: a re-run into the same `--out` must get a *fresh* copy. A leftover `dest` from a previous
    // run — or a file a previous session edited — is not the snapshot, so it is cleared before
    // provisioning (the store is cleared the same way above). `materialize` replaces a pre-existing
    // `dest` too; clearing here means the copy is fresh even when the runner is a stand-in that
    // materialises nothing.
    if dest.exists() {
        std::fs::remove_dir_all(&dest)?;
    }
    if task.preparation_id.as_deref() != runner.preparation_id() {
        return Err(RunError::Preparation(
            "task and worker preparation identities differ".into(),
        ));
    }
    let provision = runner.provision(&copies_root, &repo_abs, snapshot, &dest, &config.limits);
    ctx.observe(tsr_domain::Observation::Evidence {
        digest: Hash::of(&[provision.stdout_digest.as_bytes()]),
        source: format!("provision:{}", provision.exit),
    })?;
    // D4: a non-zero provisioning exit means the copy was never materialised. Proceeding would run
    // the session and the oracle against a directory that does not hold the snapshot, so refuse.
    if provision.exit != 0 {
        ctx.close(format!(
            "unavailable: provisioning exited {}",
            provision.exit
        ))?;
        return Ok(TaskResult {
            task: task.id.clone(),
            system: mode.system().into(),
            outcome: Outcome::Unavailable,
            generator_tokens: 0,
            tool_minutes: 0,
            wall_clock_ms: provision.duration_ms,
            family: task.family.clone(),
            lineage: task.lineage.clone(),
            state_root_at_start,
            state_root_at_end: ctx.head_root()?.to_hex(),
            denials: Vec::new(),
            oracle: None,
        });
    }

    runner
        .prepare(&repo_abs, snapshot, &dest, &out_abs)
        .map_err(RunError::Preparation)?;
    if let Some(id) = runner.preparation_id() {
        ctx.observe(tsr_domain::Observation::Evidence {
            digest: Hash::of(&[id.as_bytes()]),
            source: format!("preparation:{id}"),
        })?;
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
            // D4: the mutation diff lives in the corpus directory's own CAS, beside `tasks.jsonl`,
            // not in the batch's `--out` CAS. `tsr corpus mutants --out <corpus>` writes it there,
            // and `tsr task run --tasks <corpus>/tasks.jsonl` reads it from there.
            let cas = tsr_corpus::mine::cas_dir(&config.corpus_dir);
            let diff = tsr_corpus::mine::read_cas(&cas, diff_digest).map_err(|e| {
                RunError::MutationDiff {
                    digest: diff_digest.to_string(),
                    reason: e.to_string(),
                }
            })?;
            apply_diff(&dest, &diff)?;
        }
    }

    // Capture trusted evidence before evaluated code runs; never recover it from the copy.
    let mut oracle_snapshot = load_oracle_snapshot(&repo_abs, snapshot, task)?;
    if let Some(lock) = runner.prepared_lock(snapshot) {
        if oracle_snapshot
            .insert("Cargo.lock".into(), lock.to_vec())
            .is_some()
        {
            return Err(RunError::Preparation(
                "prepared lock must not replace snapshot lock".into(),
            ));
        }
    }

    // (4) Drive the session.
    let report = match driver.drive(&mut ctx, task, &dest) {
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

    // (4b) D6: a Tier 0 stop ends *this* task, not the batch. The driver has already
    // recorded `Stop(budget)` (or `Stop(blocked)` for a read-before-test refusal, F2); close the
    // task and record it as rejected without running the oracle — the attempt is over, so there
    // is nothing for the oracle to judge. The batch continues with the next task.
    if report.unavailable_reason.is_some() || report.stopped_on_budget || report.stopped_on_tier0 {
        let close_reason = if let Some(reason) = &report.unavailable_reason {
            format!("unavailable:{reason}")
        } else if report.stopped_on_budget {
            report
                .budget_reason
                .as_ref()
                .map(|reason| format!("stop:budget:{reason}"))
                .unwrap_or_else(|| "stop:budget".to_string())
        } else {
            format!(
                "stop:blocked:{}",
                report.tier0_reason.as_deref().unwrap_or("tier 0 refusal")
            )
        };
        ctx.close(close_reason)?;
        let state_root_at_end = ctx.head_root()?.to_hex();
        return Ok(TaskResult {
            task: task.id.clone(),
            system: mode.system().to_string(),
            outcome: if report.unavailable_reason.is_some() {
                Outcome::Unavailable
            } else {
                Outcome::Rejected
            },
            generator_tokens: report.generator_tokens,
            tool_minutes: report.tool_minutes,
            wall_clock_ms: report.wall_clock_ms,
            family: task.family.clone(),
            lineage: task.lineage.clone(),
            state_root_at_start,
            state_root_at_end,
            denials: report.denials,
            oracle: None,
        });
    }

    // (4c) D3/P1-P5: no restore. The oracle judges the copy *as it is*, with tests the session
    // cannot change. Before running anything, the copy is compared with the task's snapshot: a
    // session that added, removed or changed a build-configuration file, or planted a symlink the
    // snapshot did not have, is rejected here, naming the path. Then each oracle test is compared
    // with the snapshot's version of the same `#[test]` function, token for token.
    //
    // Missing evidence is an error, including for scripted runs. A fixture must provide
    // a real bounded snapshot rather than silently disabling integrity checks.
    if !dest.is_dir() {
        return Err(RunError::OracleFile {
            path: snapshot.to_string(),
            reason: "oracle copy or trusted snapshot unavailable".to_string(),
        });
    }
    {
        if let Some(reason) = tamper_reason(&oracle_snapshot, &dest, task)? {
            ctx.close(format!("tamper:{reason}"))?;
            let state_root_at_end = ctx.head_root()?.to_hex();
            return Ok(TaskResult {
                task: task.id.clone(),
                system: mode.system().to_string(),
                outcome: Outcome::Rejected,
                generator_tokens: report.generator_tokens,
                tool_minutes: report.tool_minutes,
                wall_clock_ms: report.wall_clock_ms,
                family: task.family.clone(),
                lineage: task.lineage.clone(),
                state_root_at_start,
                state_root_at_end,
                denials: report.denials,
                oracle: None,
            });
        }
    }

    // (5) Run the oracle through the Runner, in the session's copy (`dest`), not the parent
    // snapshot: the oracle judges the work the session actually did (D3).
    //
    // D3/P3: each oracle test is run on its own with `cargo test -- <name> --exact`, and the run's
    // *real* stdout is read (not a CAS nothing writes). A test passes only when the exit code is 0
    // AND the output carries `test <name> ... ok` for that exact name; a filter that matched nothing
    // is a failure. The full suite is run when the task asks for it, exit 0 required.
    let recorder = crate::oracle_receipts::Recorder {
        runner,
        out: &out_abs,
        snapshot,
        runs: Default::default(),
        error: Default::default(),
    };
    let (flipped, suite) =
        protected_oracle(&recorder, &dest, task, &oracle_snapshot, &config.limits)?;
    let mut oracle = oracle_verdict(&flipped, &suite);
    let evidence = recorder
        .finish(task, mode, ctx.head_root()?.to_hex(), oracle.accepted)
        .map_err(RunError::Preparation)?;
    ctx.observe(tsr_domain::Observation::Evidence {
        digest: Hash::from_hex(&evidence.manifest)
            .map_err(|e| RunError::Preparation(e.to_string()))?,
        source: "final-oracle-receipts@v1".into(),
    })?;
    oracle.evidence = Some(evidence);

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

/// Restore the task's oracle-test files into the copy from the snapshot (D3, item 2).
///
/// The session works in a disposable copy of the task and may edit anything in it, including the
/// test that defines the task. The oracle must judge the copy with the *snapshot's* tests, so before
/// it runs, every file the task names as an oracle-test file is written into the copy from the
/// snapshot, overwriting whatever the session left there. A session that deletes the test, edits its
/// assertion or renames it therefore cannot pass: the file is back, byte for byte, when the oracle
/// runs.
///
/// The paths are untrusted (`tasks.jsonl` is data): a path that is absolute, contains a `..`
/// component, or names a symlink is refused rather than read or written, the same shape
/// `is_safe_repo` requires of a repository. A path the snapshot does not hold is an error: the task
/// The build-configuration files a session may not add, remove or change (D3, P2).
///
/// A session that can write `Cargo.toml`, `Cargo.lock`, `build.rs`, a toolchain file or anything
/// under `.cargo/` can change how the oracle's tests are built and run — a `.cargo/config.toml` with
/// `runner = "true"` makes every test "pass" without running it. These paths are compared byte for
/// byte against the snapshot at any depth, and any difference rejects the session.
fn is_build_config(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    let under_cargo = rel.split('/').any(|c| c == ".cargo");
    under_cargo
        || matches!(
            name,
            "Cargo.toml" | "Cargo.lock" | "build.rs" | "rust-toolchain" | "rust-toolchain.toml"
        )
}

/// Every file in `root`, as repository-relative paths, following no symlink (D3).
///
/// A symlink is reported as its own path and never descended into: the tamper check compares the
/// copy's symlinks with the snapshot's, so a link the snapshot did not have is caught, and a link
/// the snapshot did have is not followed out of the tree.
pub(crate) fn walk_files(root: &Path) -> Result<Vec<String>, RunError> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
            Err(e) => return Err(e.into()),
        };
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            let rel = path
                .strip_prefix(root)
                .map_err(|_| RunError::Io(std::io::Error::other("walk escaped its root")))?
                .to_string_lossy()
                .replace('\\', "/");
            let meta = std::fs::symlink_metadata(&path)?;
            if meta.file_type().is_symlink() {
                out.push(rel);
            } else if meta.is_dir() {
                stack.push(path);
            } else {
                out.push(rel);
            }
        }
    }
    out.sort();
    Ok(out)
}

/// The snapshot's file at `rel`, read straight from the repository (D3).
///
/// `git show <snapshot>:<rel>` is only a read: no second tree is materialised and the copy is
/// untouched. `Ok(None)` means the snapshot does not hold the path.
pub(crate) fn snapshot_file(
    repo: &Path,
    snapshot: &str,
    rel: &str,
) -> Result<Option<Vec<u8>>, RunError> {
    let spec = format!("{snapshot}:{rel}");
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["show", &spec])
        .output()
        .map_err(|e| RunError::OracleFile {
            path: rel.to_string(),
            reason: format!("running git show: {e}"),
        })?;
    if !out.status.success() {
        return Ok(None);
    }
    Ok(Some(out.stdout))
}

/// Whether `snapshot` names a revision the repository actually holds (D3).
///
/// A synthetic fixture whose `repo` is not a real git repository, or whose snapshot is a placeholder
/// id, has no snapshot to compare against; the tamper check is skipped there rather than failing on
/// a file the snapshot "lacks". A real task always resolves.
fn snapshot_resolves(repo: &Path, snapshot: &str) -> bool {
    std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["cat-file", "-e", snapshot])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Whether the copy is a tampered copy of the snapshot (D3, P1-P5).
///
/// Returns `Some(reason)` naming the offending path when the session added, removed or changed a
/// build-configuration file at any depth, or planted a symlink the snapshot did not have. Returns
/// `None` when the copy is honest. A path the task lists as an oracle-test file that the snapshot
/// does not hold is an error, not a silent skip (P5).
pub(crate) type OracleSnapshot = std::collections::BTreeMap<String, Vec<u8>>;

pub(crate) fn load_oracle_snapshot(
    repo: &Path,
    snapshot: &str,
    task: &CorpusTask,
) -> Result<OracleSnapshot, RunError> {
    if !snapshot_resolves(repo, snapshot) {
        return Err(RunError::OracleFile {
            path: snapshot.into(),
            reason: "trusted snapshot unavailable".into(),
        });
    }
    let mut evidence = OracleSnapshot::new();
    for rel in snapshot_files(repo, snapshot)? {
        if is_build_config(&rel)
            || rel.ends_with(".rs")
            || dedicated_test_path(&rel)
            || task.oracle_test_files.contains(&rel)
        {
            if snapshot_is_symlink(repo, snapshot, &rel)? {
                return Err(RunError::OracleFile {
                    path: rel,
                    reason: "oracle evidence must be a regular file".into(),
                });
            }
            let bytes =
                snapshot_file(repo, snapshot, &rel)?.ok_or_else(|| RunError::OracleFile {
                    path: rel.clone(),
                    reason: "missing snapshot file".into(),
                })?;
            evidence.insert(rel, bytes);
        }
    }
    for rel in &task.oracle_test_files {
        if !is_safe_oracle_path(rel) || !evidence.contains_key(rel) {
            return Err(RunError::OracleFile {
                path: rel.clone(),
                reason: "unsafe or missing oracle file".into(),
            });
        }
    }
    if task.failing_tests().is_empty() || task.oracle_test_files.is_empty() {
        return Err(RunError::OracleFile {
            path: snapshot.into(),
            reason: "empty oracle test set".into(),
        });
    }
    for test in task.failing_tests() {
        let count = task
            .oracle_test_files
            .iter()
            .filter(|rel| {
                evidence
                    .get(*rel)
                    .and_then(|bytes| std::str::from_utf8(bytes).ok())
                    .and_then(|source| test_item_at(&evidence, rel, source, &test))
                    .is_some()
            })
            .count();
        if count != 1 {
            return Err(RunError::OracleFile {
                path: test,
                reason: "missing or ambiguous trusted test identity".into(),
            });
        }
    }
    Ok(evidence)
}

pub(crate) fn tamper_reason(
    snapshot: &OracleSnapshot,
    dest: &Path,
    task: &CorpusTask,
) -> Result<Option<String>, RunError> {
    if !std::fs::symlink_metadata(dest)?.is_dir() {
        return Ok(Some(
            "oracle copy root must be a directory, not a symlink".into(),
        ));
    }
    let files = walk_files(dest)?;
    // Inspect all links before reading any candidate bytes.
    for rel in &files {
        if !std::fs::symlink_metadata(dest.join(rel))?.is_file() {
            return Ok(Some(format!("unsupported symlink `{rel}`")));
        }
    }
    for rel in files.iter().filter(|p| is_build_config(p)) {
        let copy = std::fs::read(dest.join(rel))?;
        if snapshot.get(rel) != Some(&copy) {
            return Ok(Some(format!(
                "added or changed build configuration `{rel}`"
            )));
        }
    }
    for rel in snapshot.keys().filter(|p| is_build_config(p)) {
        if !dest.join(rel).is_file() {
            return Ok(Some(format!("removed build configuration `{rel}`")));
        }
    }
    for rel in &task.oracle_test_files {
        if let Some((roots, _)) = external_roots(rel) {
            for root in roots {
                if let Some(want) = snapshot.get(&root) {
                    if std::fs::read(dest.join(&root)).ok().as_ref() != Some(want) {
                        return Ok(Some(format!("changed oracle module root `{root}`")));
                    }
                }
            }
        }
    }
    for (rel, original) in snapshot {
        if dedicated_test_path(rel) {
            if std::fs::read(dest.join(rel)).ok().as_ref() != Some(original) {
                return Ok(Some(format!("changed full-suite test resource `{rel}`")));
            }
        } else if rel.ends_with(".rs") && !is_build_config(rel) {
            let current = std::fs::read(dest.join(rel)).ok();
            let want = all_test_evidence(original);
            let got = current.as_deref().and_then(all_test_evidence);
            if want.is_none() {
                if current.as_ref() != Some(original) {
                    return Ok(Some(format!("changed unparseable snapshot source `{rel}`")));
                }
            } else if got != want {
                return Ok(Some(format!("changed full-suite test evidence `{rel}`")));
            }
        }
    }
    for rel in files
        .iter()
        .filter(|rel| !snapshot.contains_key(*rel) && !rel.starts_with("target/"))
    {
        if dedicated_test_path(rel)
            || (rel.ends_with(".rs")
                && all_test_evidence(&std::fs::read(dest.join(rel))?)
                    .is_some_and(|s| !s.is_empty()))
        {
            return Ok(Some(format!("added full-suite test evidence `{rel}`")));
        }
    }
    // Dedicated test files are evidence, including their helpers/imports/macros.
    for rel in &task.oracle_test_files {
        if (rel.starts_with("tests/") || rel.contains("/tests/"))
            && snapshot.get(rel).map(Vec::as_slice) != std::fs::read(dest.join(rel)).ok().as_deref()
        {
            return Ok(Some(format!("changed dedicated test file `{rel}`")));
        }
    }
    // A named test must resolve to exactly one trusted file, not every listed file.
    for test in task.failing_tests() {
        let matches: Vec<_> = task
            .oracle_test_files
            .iter()
            .filter_map(|rel| {
                let source = std::str::from_utf8(snapshot.get(rel)?).ok()?;
                test_item_at(snapshot, rel, source, &test).map(|tokens| (rel, tokens))
            })
            .collect();
        if matches.len() != 1 {
            return Err(RunError::OracleFile {
                path: test,
                reason: "missing or ambiguous trusted test identity".into(),
            });
        }
        let (rel, want) = &matches[0];
        for absent in &want.absent_paths {
            match std::fs::symlink_metadata(dest.join(absent)) {
                Ok(_) => {
                    return Ok(Some(format!(
                        "added alternate oracle module path `{absent}`"
                    )))
                }
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => return Err(e.into()),
            }
        }
        for ancestor in &want.ancestors {
            if snapshot.get(ancestor).map(Vec::as_slice)
                != std::fs::read(dest.join(ancestor)).ok().as_deref()
            {
                return Ok(Some(format!("changed oracle module ancestor `{ancestor}`")));
            }
        }
        let got = std::fs::read_to_string(dest.join(rel))
            .ok()
            .and_then(|source| test_item_at(snapshot, rel, &source, &test));
        if got.as_ref() != Some(want) {
            return Ok(Some(format!(
                "removed or changed oracle test `{test}` in `{rel}`"
            )));
        }
    }
    Ok(None)
}

/// Every path the snapshot holds, as repository-relative paths (D3).
pub(crate) fn snapshot_files(repo: &Path, snapshot: &str) -> Result<Vec<String>, RunError> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["ls-tree", "-r", "--name-only", snapshot])
        .output()
        .map_err(|e| RunError::OracleFile {
            path: snapshot.to_string(),
            reason: format!("running git ls-tree: {e}"),
        })?;
    if !out.status.success() {
        return Err(RunError::OracleFile {
            path: snapshot.into(),
            reason: "cannot enumerate trusted snapshot".into(),
        });
    }
    Ok(String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect())
}

/// Whether the snapshot's entry at `rel` is a symlink (mode 120000).
pub(crate) fn snapshot_is_symlink(
    repo: &Path,
    snapshot: &str,
    rel: &str,
) -> Result<bool, RunError> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["ls-tree", snapshot, "--", rel])
        .output()
        .map_err(|e| RunError::OracleFile {
            path: rel.to_string(),
            reason: format!("running git ls-tree: {e}"),
        })?;
    Ok(String::from_utf8_lossy(&out.stdout).starts_with("120000"))
}

fn dedicated_test_path(rel: &str) -> bool {
    rel.starts_with("tests/") || rel.contains("/tests/")
}
/// Freeze test regions and the declarations/attributes that determine their reachability.
/// Production function bodies outside these regions are intentionally excluded.
fn all_test_evidence(bytes: &[u8]) -> Option<String> {
    use quote::ToTokens;
    fn collect(items: &[syn::Item]) -> Vec<String> {
        let mut out = Vec::new();
        for item in items {
            match item {
                syn::Item::Fn(f)
                    if f.attrs.iter().any(|a| {
                        a.path().segments.last().is_some_and(|s| {
                            matches!(s.ident.to_string().as_str(), "test" | "bench")
                        })
                    }) =>
                {
                    out.push(f.to_token_stream().to_string())
                }
                syn::Item::Mod(m) => {
                    let test_only = m
                        .attrs
                        .iter()
                        .any(|a| a.meta.to_token_stream().to_string() == "cfg (test)");
                    if test_only || m.content.is_none() {
                        out.push(m.to_token_stream().to_string());
                    } else if let Some((_, items)) = &m.content {
                        let nested = collect(items);
                        if !nested.is_empty() {
                            out.push(format!(
                                "{} {} {:?}",
                                m.attrs
                                    .iter()
                                    .map(|a| a.to_token_stream().to_string())
                                    .collect::<Vec<_>>()
                                    .join(" "),
                                m.ident,
                                nested
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
        out
    }
    let file = syn::parse_file(std::str::from_utf8(bytes).ok()?).ok()?;
    let attrs = file
        .attrs
        .iter()
        .map(|a| a.to_token_stream().to_string())
        .collect::<Vec<_>>()
        .join(" ");
    let parts = collect(&file.items);
    if attrs.is_empty() && parts.is_empty() {
        Some(String::new())
    } else {
        Some(format!("{attrs} {parts:?}"))
    }
}
fn protected_oracle(
    runner: &dyn Runner,
    dest: &Path,
    task: &CorpusTask,
    snapshot: &OracleSnapshot,
    limits: &Limits,
) -> Result<(RunResult, RunResult), RunError> {
    let guard = || -> Result<Option<RunResult>, RunError> {
        Ok(tamper_reason(snapshot, dest, task)?
            .map(|reason| RunResult::from_output(1, b"", reason.as_bytes(), 0)))
    };
    if let Some(failure) = guard()? {
        return Ok((failure.clone(), failure));
    }
    let mut flipped = RunResult::exit_only(1);
    for test in task.failing_tests() {
        flipped = run_oracle_tests(runner, dest, &[test], limits);
        if let Some(failure) = guard()? {
            return Ok((failure.clone(), failure));
        }
        if !flipped.succeeded() {
            break;
        }
    }
    let suite = runner.run(dest, &full_suite_argv(dest), limits);
    if let Some(failure) = guard()? {
        return Ok((failure.clone(), failure));
    }
    Ok((flipped, suite))
}

// Preserve the existing direct-module root guard in addition to resolved-chain evidence.
fn external_roots(rel: &str) -> Option<([String; 2], String)> {
    let (base, file) = rel.rsplit_once("src/")?;
    if file.contains('/') || matches!(file, "lib.rs" | "main.rs" | "mod.rs") {
        return None;
    }
    let module = file.strip_suffix(".rs")?;
    Some((
        [format!("{base}src/lib.rs"), format!("{base}src/main.rs")],
        module.into(),
    ))
}

/// A test identity and the trusted files declaring its external module chain.
/// Ancestors are frozen bytewise; production edits there are deliberately unsupported.
#[derive(PartialEq, Eq)]
struct TestEvidence {
    tokens: String,
    ancestors: Vec<String>,
    absent_paths: Vec<String>,
}

/// Follow only plain external declarations through standard Rust source layouts.
/// No cfg/path attributes, macro-generated ancestors, inline/external mixtures or guesses.
fn test_item_at(
    snapshot: &OracleSnapshot,
    rel: &str,
    source: &str,
    name: &str,
) -> Option<TestEvidence> {
    if !is_safe_oracle_path(rel) {
        return None;
    }
    let source_path = rel.rsplit_once("src/");
    if dedicated_test_path(rel)
        || source_path.is_some_and(|(_, file)| matches!(file, "lib.rs" | "main.rs"))
    {
        return Some(TestEvidence {
            tokens: test_item(source, name)?,
            ancestors: Vec::new(),
            absent_paths: Vec::new(),
        });
    }
    let (base, file) = source_path?;
    let stem = file.strip_suffix(".rs")?;
    let stem = stem.strip_suffix("/mod").unwrap_or(stem);
    let modules: Vec<_> = stem.split('/').collect();
    if modules
        .iter()
        .any(|m| matches!(*m, "lib" | "main" | "mod") || syn::parse_str::<syn::Ident>(m).is_err())
    {
        return None;
    }
    let prefix = format!("{}::", modules.join("::"));
    let local = name.strip_prefix(&prefix)?;
    fn declarations(bytes: &[u8], module: &str) -> Option<usize> {
        let file = syn::parse_file(std::str::from_utf8(bytes).ok()?).ok()?;
        // This built-in lint cannot select or redirect a module. The entire
        // ancestor remains bytewise frozen by tamper_reason, including the lint.
        // Do not generalize this to cfg_attr, arbitrary lints, or custom attributes.
        let supported_file_attr = |a: &syn::Attribute| {
            a.path().is_ident("doc")
                || (matches!(a.style, syn::AttrStyle::Inner(_))
                    && a.path().is_ident("deny")
                    && a.parse_args::<syn::Path>()
                        .is_ok_and(|path| path.is_ident("unsafe_code")))
        };
        if file.attrs.iter().any(|a| !supported_file_attr(a))
            || file.items.iter().any(|i| matches!(i, syn::Item::Macro(_)))
        {
            return None;
        }
        let mut count = 0;
        for item in file.items {
            if let syn::Item::Mod(m) = item {
                if m.ident == module {
                    if m.content.is_some() || m.attrs.iter().any(|a| !a.path().is_ident("doc")) {
                        return None;
                    }
                    count += 1;
                }
            }
        }
        Some(count)
    }
    let mut roots = Vec::new();
    let mut ancestors = Vec::new();
    let mut absent_paths = Vec::new();
    for root in [format!("{base}src/lib.rs"), format!("{base}src/main.rs")] {
        if let Some(bytes) = snapshot.get(&root) {
            ancestors.push(root.clone());
            match declarations(bytes, modules[0])? {
                0 => {}
                1 => roots.push(root),
                _ => return None,
            }
        } else {
            absent_paths.push(root);
        }
    }
    if roots.len() != 1 {
        return None;
    }
    let mut parent = roots.pop()?;
    let mut directory = format!("{base}src");
    for (index, module) in modules.iter().enumerate() {
        if declarations(snapshot.get(&parent)?, module)? != 1 {
            return None;
        }
        if !ancestors.contains(&parent) {
            ancestors.push(parent);
        }
        let flat = format!("{directory}/{module}.rs");
        let nested = format!("{directory}/{module}/mod.rs");
        // The final source is supplied separately by the caller. Intermediate files
        // must be present in the trusted snapshot, never read from the candidate.
        let mut choices = Vec::new();
        for path in [flat, nested] {
            if snapshot.contains_key(&path) || path == rel {
                choices.push(path);
            } else {
                absent_paths.push(path);
            }
        }
        if choices.len() != 1 {
            return None;
        }
        parent = choices.into_iter().next()?;
        if index + 1 == modules.len() && parent != rel {
            return None;
        }
        directory = format!("{directory}/{module}");
    }
    Some(TestEvidence {
        tokens: test_item(source, local)?,
        ancestors,
        absent_paths,
    })
}

/// Resolve a test by its complete inline-module path. Ambiguous declarations fail closed.
/// External modules/macros not represented in this source are unsupported.
fn test_item(source: &str, name: &str) -> Option<String> {
    use quote::ToTokens;
    fn collect(items: &[syn::Item], prefix: &str, name: &str, found: &mut Vec<String>) {
        for item in items {
            match item {
                syn::Item::Fn(f)
                    if f.attrs.iter().any(|a| {
                        a.path().is_ident("test")
                            || (a.path().segments.len() == 2
                                && a.path().segments[0].ident == "tokio"
                                && a.path().segments[1].ident == "test")
                    }) =>
                {
                    if format!("{prefix}{}", f.sig.ident) == name {
                        found.push(f.to_token_stream().to_string());
                    }
                }
                syn::Item::Mod(m) => {
                    if let Some((_, items)) = &m.content {
                        let before = found.len();
                        collect(items, &format!("{prefix}{}::", m.ident), name, found);
                        let attrs = m
                            .attrs
                            .iter()
                            .map(|a| a.to_token_stream().to_string())
                            .collect::<Vec<_>>()
                            .join(" ");
                        let test_only = m
                            .attrs
                            .iter()
                            .any(|a| a.meta.to_token_stream().to_string() == "cfg (test)");
                        for value in &mut found[before..] {
                            *value = if test_only {
                                m.to_token_stream().to_string()
                            } else {
                                format!("{attrs} {value}")
                            };
                        }
                    }
                }
                _ => {}
            }
        }
    }
    let file = syn::parse_file(source).ok()?;
    let mut found = Vec::new();
    collect(&file.items, "", name, &mut found);
    if found.len() == 1 {
        found.pop()
    } else {
        None
    }
}

/// Whether `path` is a safe oracle-test file path: relative, no `..` component, no root.
///
/// The path is joined into both the snapshot and the copy, so an absolute path or a `..` component
/// would name a file outside them (D1).
pub fn is_safe_oracle_path(path: &str) -> bool {
    if path.is_empty() || path.starts_with('/') || path.contains('\\') {
        return false;
    }
    !path
        .split('/')
        .any(|c| c == ".." || c == "." || c.is_empty())
}

/// Run each oracle test in the copy with `cargo test -- <name> --exact` (D3, P3).
///
/// The run's *real* stdout is read — not a CAS nothing writes. The returned result succeeds only
/// when every named test's run exited 0 AND its output carries `test <name> ... ok` for that exact
/// name; a filter that matched nothing is a failure. The result is the first failing run, or the
/// last passing one when all pass.
fn run_oracle_tests(
    runner: &dyn Runner,
    dest: &Path,
    tests: &[String],
    limits: &Limits,
) -> RunResult {
    let mut last = RunResult::exit_only(1);
    for name in tests {
        let argv = exact_test_argv(dest, name);
        let result = runner.run(dest, &argv, limits);
        let text = String::from_utf8_lossy(&result.stdout);
        let ran = text
            .lines()
            .any(|l| l.trim() == format!("test {name} ... ok"));
        if !result.succeeded() || !ran {
            return RunResult::from_output(1, &result.stdout, b"", result.duration_ms);
        }
        last = result;
    }
    last
}

/// The argv that runs one oracle test exactly: `cargo test -- <name> --exact` (D3, P3).
///
/// The run goes through the Runner, which invokes `tsr exec -- <argv>` in the sealed sandbox with
/// the copy as its working directory, so the argv is the suite command itself — not the `tsr corpus
/// oracle` wrapper, whose stdout is its own verdict line and would hide cargo's `test <name> ...
/// ok`. Reading cargo's real stdout is what makes a filter that matched nothing a failure.
pub fn exact_test_argv(_dir: &Path, name: &str) -> Vec<String> {
    vec![
        "cargo".to_string(),
        "test".to_string(),
        "--".to_string(),
        name.to_string(),
        "--exact".to_string(),
    ]
}

/// The oracle: the flipped tests pass and the full suite does not regress.
pub fn oracle_verdict(flipped: &RunResult, suite: &RunResult) -> OracleVerdict {
    let flipped_tests_pass = flipped.succeeded();
    let suite_regressed = !suite.succeeded();
    OracleVerdict {
        evidence: None,
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

#[cfg(test)]
mod oracle_integrity_regression {
    use super::*;
    struct Output(&'static str);
    impl Runner for Output {
        fn run(&self, _: &Path, _: &[String], _: &Limits) -> RunResult {
            RunResult::from_output(0, self.0.as_bytes(), b"", 0)
        }
    }
    #[test]
    fn exact_evidence_required_even_without_summary() {
        for output in ["", "oracle: pass (exit 0)", "test other ... ok"] {
            assert!(!run_oracle_tests(
                &Output(output),
                Path::new("."),
                &["tests::check".into()],
                &Limits::new(1000, 1024)
            )
            .succeeded());
        }
    }
    #[test]
    fn inline_test_module_helpers_and_macros_are_frozen() {
        let base="fn implementation() {} #[cfg(test)] mod tests { fn helper()->i32 { 2 } #[test] fn adds() { assert_eq!(helper(),2); } }";
        let changed = base.replace("{ 2 }", "{ 3 }");
        assert_ne!(
            test_item(base, "tests::adds"),
            test_item(&changed, "tests::adds")
        );
        let macros = base.replace(
            "fn helper()",
            "macro_rules! assert_eq { ($($t:tt)*) => {}; } fn helper()",
        );
        assert_ne!(
            test_item(base, "tests::adds"),
            test_item(&macros, "tests::adds")
        );
        let implementation = base.replace(
            "fn implementation() {}",
            "fn implementation() { let _x=2; }",
        );
        assert_eq!(
            test_item(base, "tests::adds"),
            test_item(&implementation, "tests::adds")
        );
    }
    #[test]
    fn empty_test_set_is_not_success() {
        assert!(
            !run_oracle_tests(&Output(""), Path::new("."), &[], &Limits::new(1000, 1024))
                .succeeded()
        );
    }
    #[test]
    fn test_identity_includes_module_path() {
        let source = "mod left { #[test] fn check() { assert!(false); } } mod right { #[test] fn check() { assert!(true); } }";
        assert_ne!(
            test_item(source, "left::check"),
            test_item(source, "right::check")
        );
        assert!(test_item(source, "missing::check").is_none());
    }
    #[test]
    fn duplicate_test_identity_is_ambiguous() {
        assert!(test_item("#[test] fn check() {} #[test] fn check() {}", "check").is_none());
    }
    #[test]
    fn bounded_integrity_matrix_for_both_task_kinds_and_layouts() {
        let template = include_str!("../../../fixtures/m3/batch/tasks.jsonl")
            .lines()
            .next()
            .unwrap();
        for mutant in [false, true] {
            for inline in [false, true] {
                let mut task: CorpusTask = serde_json::from_str(template).unwrap();
                let (path, name, source) = if inline {
                    ("src/lib.rs", "tests::check", "fn add() -> i32 { 0 } mod tests { #[test] fn check() { assert_eq!(super::add(), 2); } }")
                } else {
                    (
                        "tests/check.rs",
                        "check",
                        "#[test] fn check() { assert_eq!(demo::add(), 2); }",
                    )
                };
                task.flipped_tests = vec![name.into()];
                task.failing_tests.clear();
                task.mutated_snapshot = None;
                if mutant {
                    task.mutated_snapshot = Some("mutant".into());
                    task.failing_tests = std::mem::take(&mut task.flipped_tests);
                }
                task.oracle_test_files = vec![path.into()];
                let snapshot = OracleSnapshot::from([(path.into(), source.as_bytes().to_vec())]);
                let dir = tempfile::tempdir().unwrap();
                let file = dir.path().join(path);
                std::fs::create_dir_all(file.parent().unwrap()).unwrap();
                // The guard checks integrity, not program correctness. Actual verdicts require
                // separate executor evidence, tested above; none is fabricated by this matrix.
                for candidate in [source.to_string(), source.replace("{ 0 }", "{ 2 }")] {
                    std::fs::write(&file, candidate).unwrap();
                    assert!(tamper_reason(&snapshot, dir.path(), &task)
                        .unwrap()
                        .is_none());
                }
                for candidate in [
                    String::new(),
                    source.replace("fn check", "fn renamed"),
                    source.replace(", 2)", ", 0)"),
                ] {
                    std::fs::write(&file, candidate).unwrap();
                    assert!(tamper_reason(&snapshot, dir.path(), &task)
                        .unwrap()
                        .is_some());
                }
                std::fs::write(&file, source).unwrap();
                for config in ["build.rs", "Cargo.toml", ".cargo/config.toml"] {
                    let config_path = dir.path().join(config);
                    std::fs::create_dir_all(config_path.parent().unwrap()).unwrap();
                    std::fs::write(&config_path, "bounded inert fixture").unwrap();
                    assert!(tamper_reason(&snapshot, dir.path(), &task)
                        .unwrap()
                        .is_some());
                    std::fs::remove_file(config_path).unwrap();
                }
                #[cfg(unix)]
                {
                    let outside = tempfile::tempdir().unwrap();
                    std::fs::write(outside.path().join("sentinel"), "unchanged").unwrap();
                    std::os::unix::fs::symlink(outside.path(), dir.path().join("link")).unwrap();
                    assert!(tamper_reason(&snapshot, dir.path(), &task)
                        .unwrap()
                        .is_some());
                    assert_eq!(
                        std::fs::read_to_string(outside.path().join("sentinel")).unwrap(),
                        "unchanged"
                    );
                }
            }
        }
    }
    #[test]
    fn missing_snapshot_is_an_error() {
        let task: CorpusTask = serde_json::from_str(
            include_str!("../../../fixtures/m3/batch/tasks.jsonl")
                .lines()
                .next()
                .unwrap(),
        )
        .unwrap();
        let dir = tempfile::tempdir().unwrap();
        assert!(load_oracle_snapshot(
            dir.path(),
            "0000000000000000000000000000000000000000",
            &task
        )
        .is_err());
    }
    #[test]
    fn changed_module_attributes_are_tampering() {
        assert_ne!(
            test_item(
                "#[cfg(test)] mod tests { #[test] fn check() {} }",
                "tests::check"
            ),
            test_item(
                "#[cfg(any())] mod tests { #[test] fn check() {} }",
                "tests::check"
            )
        );
    }
    #[test]
    fn exact_evidence_succeeds() {
        assert!(run_oracle_tests(
            &Output("test tests::check ... ok\n"),
            Path::new("."),
            &["tests::check".into()],
            &Limits::new(1000, 1024)
        )
        .succeeded());
    }
}

#[cfg(test)]
mod external_module_regression {
    use super::*;
    #[test]
    fn tokio_test_identity_preserves_attributes_and_rejects_unknown_macros() {
        let source = "#[cfg(test)] mod tests { #[tokio::test(flavor=\"current_thread\")] async fn holds() {} }";
        let item = test_item(source, "tests::holds").unwrap();
        assert!(item.contains("current_thread"));
        assert_ne!(
            Some(item),
            test_item(
                &source.replace("current_thread", "multi_thread"),
                "tests::holds"
            )
        );
        assert!(test_item(&source.replace("tokio", "unknown"), "tests::holds").is_none());
    }
    #[test]
    fn binary_root_is_supported_but_duplicate_or_redirected_roots_are_not() {
        let source = "#[cfg(test)] mod tests { #[test] fn holds() {} }";
        let path = "crates/example/src/feature.rs";
        let mut snapshot = OracleSnapshot::new();
        snapshot.insert(
            "crates/example/src/main.rs".into(),
            b"mod feature; fn main() {}".to_vec(),
        );
        assert!(test_item_at(&snapshot, path, source, "feature::tests::holds").is_some());
        snapshot.insert("crates/example/src/lib.rs".into(), b"mod feature;".to_vec());
        assert!(test_item_at(&snapshot, path, source, "feature::tests::holds").is_none());
        snapshot.insert(
            "crates/example/src/lib.rs".into(),
            b"#[path=\"other.rs\"] mod feature;".to_vec(),
        );
        assert!(test_item_at(&snapshot, path, source, "feature::tests::holds").is_none());
    }
    #[test]
    fn external_test_prefix_requires_its_trusted_plain_declaration() {
        let source = "#[cfg(test)] mod tests { #[test] fn holds() {} }";
        let mut snapshot = OracleSnapshot::new();
        let path = "crates/example/src/feature.rs";
        assert!(test_item_at(&snapshot, path, source, "feature::tests::holds").is_none());
        snapshot.insert(
            "crates/example/src/lib.rs".into(),
            b"pub mod feature;".to_vec(),
        );
        assert!(test_item_at(&snapshot, path, source, "feature::tests::holds").is_some());
        assert!(test_item_at(&snapshot, path, source, "different::tests::holds").is_none());
        snapshot.insert(
            "crates/example/src/lib.rs".into(),
            b"#[path=\"other.rs\"] mod feature;".to_vec(),
        );
        assert!(test_item_at(&snapshot, path, source, "feature::tests::holds").is_none());
    }
}

#[cfg(test)]
mod full_suite_evidence_regression {
    use super::*;
    use std::{cell::Cell, fs, process::Command};
    struct Inert {
        calls: Cell<usize>,
        alter_at: usize,
    }
    impl Runner for Inert {
        fn run(&self, copy: &Path, _: &[String], _: &Limits) -> RunResult {
            let call = self.calls.get() + 1;
            self.calls.set(call);
            if call == self.alter_at {
                fs::write(copy.join("tests/other.rs"), "#[test] fn other() {}\n").unwrap();
            }
            RunResult::from_output(0, b"test named ... ok\n", b"", 0)
        }
    }
    #[test]
    fn unlisted_tests_are_frozen_before_and_after_execution() {
        let dir = tempfile::tempdir().unwrap();
        let repo = dir.path().join("repo");
        fs::create_dir(&repo).unwrap();
        let files=[
            ("tests/named.rs","#[test] fn named() {assert!(true);}"),
            ("tests/other.rs","#[test] fn other() {assert!(false);}"),
            ("tests/input.txt","unlisted test resource"),
            ("src/lib.rs","mod other; fn production()->i32 {1} #[cfg(test)] mod tests {fn helper()->i32 {2} #[test] fn inline(){assert_eq!(helper(),2);}}"),
            ("src/other.rs","#[test] fn separate(){assert!(false);}"),
        ];
        for (rel, source) in files {
            let p = repo.join(rel);
            fs::create_dir_all(p.parent().unwrap()).unwrap();
            fs::write(p, source).unwrap();
        }
        let git = |args: &[&str]| {
            let r = Command::new("git")
                .arg("-C")
                .arg(&repo)
                .args([
                    "-c",
                    "core.hooksPath=/dev/null",
                    "-c",
                    "user.name=fixture",
                    "-c",
                    "user.email=fixture@example.invalid",
                ])
                .args(args)
                .output()
                .unwrap();
            assert!(r.status.success());
            r.stdout
        };
        git(&["init", "-q"]);
        git(&["add", "."]);
        git(&["commit", "-qm", "fixture"]);
        let tree = String::from_utf8(git(&["rev-parse", "HEAD^{tree}"]))
            .unwrap()
            .trim()
            .to_string();
        let mut task: CorpusTask = serde_json::from_str(
            include_str!("../../../fixtures/m3/batch/tasks.jsonl")
                .lines()
                .next()
                .unwrap(),
        )
        .unwrap();
        task.oracle_test_files = vec!["tests/named.rs".into()];
        task.flipped_tests = vec!["named".into()];
        task.failing_tests.clear();
        let trusted = load_oracle_snapshot(&repo, &tree, &task).unwrap();
        assert!(
            trusted.contains_key("tests/other.rs")
                && trusted.contains_key("src/other.rs")
                && trusted.contains_key("tests/input.txt")
        );
        let copy = dir.path().join("copy");
        tsr_corpus::mine::materialize(&repo, &tree, &copy).unwrap();
        let restore = || {
            for (rel, source) in files {
                fs::write(copy.join(rel), source).unwrap();
            }
        };
        for rel in [
            "tests/other.rs",
            "tests/input.txt",
            "src/other.rs",
            "src/lib.rs",
        ] {
            for remove in [false, true] {
                restore();
                if remove {
                    fs::remove_file(copy.join(rel)).unwrap();
                } else {
                    fs::write(copy.join(rel), "").unwrap();
                }
                let runner = Inert {
                    calls: Cell::new(0),
                    alter_at: usize::MAX,
                };
                let (named, full) =
                    protected_oracle(&runner, &copy, &task, &trusted, &Limits::default()).unwrap();
                assert!(!oracle_verdict(&named, &full).accepted);
                assert_eq!(runner.calls.get(), 0);
            }
        }
        restore();
        fs::write(copy.join("src/lib.rs"), files[3].1.replace("{1}", "{9}")).unwrap();
        assert!(tamper_reason(&trusted, &copy, &task).unwrap().is_none());
        for alter_at in [1, 2] {
            restore();
            let runner = Inert {
                calls: Cell::new(0),
                alter_at,
            };
            let (named, full) =
                protected_oracle(&runner, &copy, &task, &trusted, &Limits::default()).unwrap();
            assert!(!oracle_verdict(&named, &full).accepted);
            assert_eq!(runner.calls.get(), alter_at);
        }
    }
}

#[cfg(test)]
mod nested_module_identity_regression {
    use super::*;
    use std::cell::Cell;
    const TEST: &str = "outer::inner::tests::holds";
    const SOURCE: &str = "fn production()->u8 {1} #[cfg(test)] mod tests { #[test] fn holds(){ assert_eq!(super::production(),1); } }";
    fn snapshot(outer: &str, inner: &str) -> OracleSnapshot {
        [
            ("src/lib.rs", "pub mod outer;"),
            (outer, "pub mod inner; fn unrelated()->u8 {1}"),
            (inner, SOURCE),
        ]
        .into_iter()
        .map(|(p, s)| (p.into(), s.as_bytes().to_vec()))
        .collect()
    }
    #[test]
    fn m3_module_1_resolves_both_standard_nested_layouts() {
        for outer in ["src/outer.rs", "src/outer/mod.rs"] {
            for inner in ["src/outer/inner.rs", "src/outer/inner/mod.rs"] {
                let s = snapshot(outer, inner);
                assert!(
                    test_item_at(&s, inner, SOURCE, TEST).is_some(),
                    "{outer} {inner}"
                );
                assert!(test_item_at(&s, inner, SOURCE, "other::inner::tests::holds").is_none());
                assert!(test_item_at(&s, inner, SOURCE, "outer::wrong::tests::holds").is_none());
            }
        }
    }
    #[test]
    fn m3_module_1_cannot_skip_external_chain_with_duplicated_inline_prefix() {
        let path = "src/outer/inner.rs";
        let spoof = "mod outer { mod inner { #[test] fn holds() {} } }";
        assert!(test_item_at(&OracleSnapshot::new(), path, spoof, "outer::inner::holds").is_none());
        let s = snapshot("src/outer/mod.rs", path);
        assert!(test_item_at(&s, path, spoof, "outer::inner::holds").is_none());
        assert!(test_item_at(&s, path, spoof, "outer::inner::outer::inner::holds").is_some());
    }
    #[test]
    fn m3_module_2_resolves_deny_unsafe_code_but_freezes_the_ancestor() {
        let path = "src/outer/inner.rs";
        let mut s = snapshot("src/outer/mod.rs", path);
        s.insert(
            "src/lib.rs".into(),
            b"#![deny(unsafe_code)] pub mod outer;".to_vec(),
        );
        assert!(test_item_at(&s, path, SOURCE, TEST).is_some());
        let dir = tempfile::tempdir().unwrap();
        for (p, b) in &s {
            let p = dir.path().join(p);
            std::fs::create_dir_all(p.parent().unwrap()).unwrap();
            std::fs::write(p, b).unwrap();
        }
        let mut task: CorpusTask = serde_json::from_str(
            include_str!("../../../fixtures/m3/batch/tasks.jsonl")
                .lines()
                .next()
                .unwrap(),
        )
        .unwrap();
        task.oracle_test_files = vec![path.into()];
        task.flipped_tests = vec![TEST.into()];
        task.failing_tests.clear();
        assert!(tamper_reason(&s, dir.path(), &task).unwrap().is_none());
        for changed in [
            "pub mod outer;",
            "#![allow(unsafe_code)] pub mod outer;",
            "#![deny(unsafe_code)] mod outer {}",
        ] {
            std::fs::write(dir.path().join("src/lib.rs"), changed).unwrap();
            assert!(tamper_reason(&s, dir.path(), &task).unwrap().is_some());
        }
    }

    #[test]
    fn m3_module_2_keeps_unknown_and_conditional_attributes_closed() {
        let path = "src/outer/inner.rs";
        for attribute in [
            "#![cfg(test)]",
            "#![cfg_attr(test, deny(unsafe_code))]",
            "#![deny(warnings)]",
            "#![deny(unsafe_code, warnings)]",
            "#![deny(unsafe_code = true)]",
            "#![custom]",
            "#![allow(unsafe_code)]",
        ] {
            let mut s = snapshot("src/outer/mod.rs", path);
            s.insert(
                "src/lib.rs".into(),
                format!("{attribute} pub mod outer;").into_bytes(),
            );
            assert!(
                test_item_at(&s, path, SOURCE, TEST).is_none(),
                "{attribute}"
            );
        }
    }

    #[test]
    fn m3_module_1_refuses_ambiguous_or_unsupported_chains() {
        let path = "src/outer/inner.rs";
        for (file, text) in [
            ("src/outer.rs", "pub mod inner;"),
            ("src/outer/inner/mod.rs", SOURCE),
            ("src/main.rs", "pub mod outer;"),
            ("src/lib.rs", "pub mod outer; mod outer;"),
            ("src/lib.rs", "#[cfg(unix)] pub mod outer;"),
            ("src/lib.rs", "include!(\"generated.rs\"); pub mod outer;"),
            ("src/outer/mod.rs", "#[path=\"elsewhere.rs\"] mod inner;"),
            (
                "src/outer/mod.rs",
                "#[cfg_attr(unix, path=\"other.rs\")] mod inner;",
            ),
            ("src/outer/mod.rs", "mod inner {}"),
            ("src/outer/mod.rs", "mod inner; mod inner;"),
            ("src/outer/mod.rs", "#![cfg(unix)] mod inner;"),
            ("src/outer/mod.rs", "include!(\"generated.rs\"); mod inner;"),
            ("src/outer/mod.rs", "mod unrelated;"),
        ] {
            let mut s = snapshot("src/outer/mod.rs", path);
            s.insert(file.into(), text.as_bytes().to_vec());
            assert!(
                test_item_at(&s, path, SOURCE, TEST).is_none(),
                "{file}: {text}"
            );
        }
        let mut s = snapshot("src/outer/mod.rs", path);
        s.remove("src/outer/mod.rs");
        assert!(test_item_at(&s, path, SOURCE, TEST).is_none());
    }
    struct Inert {
        calls: Cell<usize>,
        alter_at: usize,
    }
    impl Runner for Inert {
        fn run(&self, copy: &Path, _: &[String], _: &Limits) -> RunResult {
            let n = self.calls.get() + 1;
            self.calls.set(n);
            if n == self.alter_at {
                std::fs::write(
                    copy.join("src/outer/mod.rs"),
                    "mod inner; fn unrelated()->u8 {9}",
                )
                .unwrap();
            }
            RunResult::from_output(0, format!("test {TEST} ... ok\n").as_bytes(), b"", 0)
        }
    }
    #[test]
    fn m3_module_1_freezes_ancestors_before_and_after_each_run() {
        let dir = tempfile::tempdir().unwrap();
        let dest = dir.path();
        let path = "src/outer/inner.rs";
        let s = snapshot("src/outer/mod.rs", path);
        let restore = || {
            for (p, b) in &s {
                let p = dest.join(p);
                std::fs::create_dir_all(p.parent().unwrap()).unwrap();
                std::fs::write(p, b).unwrap();
            }
        };
        let mut task: CorpusTask = serde_json::from_str(
            include_str!("../../../fixtures/m3/batch/tasks.jsonl")
                .lines()
                .next()
                .unwrap(),
        )
        .unwrap();
        task.oracle_test_files = vec![path.into()];
        task.flipped_tests = vec![TEST.into()];
        task.failing_tests.clear();
        restore();
        std::fs::write(dest.join(path), SOURCE.replace("{1}", "{2}")).unwrap();
        let runner = Inert {
            calls: Cell::new(0),
            alter_at: usize::MAX,
        };
        let (named, full) = protected_oracle(&runner, dest, &task, &s, &Limits::default()).unwrap();
        assert!(oracle_verdict(&named, &full).accepted);
        assert_eq!(runner.calls.get(), 2);
        for changed in [
            "mod inner; fn unrelated()->u8 {9}",
            "#[path=\"other.rs\"] mod inner;",
            "",
        ] {
            restore();
            std::fs::write(dest.join("src/outer/mod.rs"), changed).unwrap();
            let runner = Inert {
                calls: Cell::new(0),
                alter_at: usize::MAX,
            };
            let (named, full) =
                protected_oracle(&runner, dest, &task, &s, &Limits::default()).unwrap();
            assert!(!oracle_verdict(&named, &full).accepted);
            assert_eq!(runner.calls.get(), 0);
        }
        restore();
        std::fs::remove_file(dest.join("src/outer/mod.rs")).unwrap();
        assert!(tamper_reason(&s, dest, &task).unwrap().is_some());
        for alternate in ["src/outer.rs", "src/outer/inner/mod.rs", "src/main.rs"] {
            restore();
            let added = dest.join(alternate);
            std::fs::create_dir_all(added.parent().unwrap()).unwrap();
            std::fs::write(&added, "fn unrelated() {}").unwrap();
            let runner = Inert {
                calls: Cell::new(0),
                alter_at: usize::MAX,
            };
            let (named, full) =
                protected_oracle(&runner, dest, &task, &s, &Limits::default()).unwrap();
            assert!(!oracle_verdict(&named, &full).accepted);
            assert_eq!(runner.calls.get(), 0);
            std::fs::remove_file(added).unwrap();
        }
        for at in [1, 2] {
            restore();
            let runner = Inert {
                calls: Cell::new(0),
                alter_at: at,
            };
            let (named, full) =
                protected_oracle(&runner, dest, &task, &s, &Limits::default()).unwrap();
            assert!(!oracle_verdict(&named, &full).accepted);
            assert_eq!(runner.calls.get(), at);
        }
    }
}
