//! The `Driver` trait and its two implementations (PRD §B9, §B11 M3).
//!
//! A driver takes an open [`SessionCtx`] and drives one engine task to a close. It is the only
//! place a *policy* lives: the session records what the driver does, and the gate decides whether a
//! gated action may proceed. The two systems differ only in their driver:
//!
//! - **System 1 (record)** — [`ScriptedDriver`] with `mode = record`: it replays the fixture's
//!   candidate actions and records them. It gates nothing: no candidate is put to the gate, so no
//!   action is ever denied. This is the record-only hook, and it is the baseline the rules system is
//!   measured against.
//! - **System 2 (rules)** — [`ScriptedDriver`] with `mode = rules`: it runs the Tier 0 controller
//!   (inspect until the failing assertion and its call site have been read, then test, then propose,
//!   then stop on budget), and every gated action passes through `tsr-policy`'s gate under the
//!   RATCHET profile. A denial is recorded with its reason and the action does not happen.
//!
//! [`ClaudeCodeDriver`] is specified but not implemented: the swarm sandbox has no Claude Code and
//! no container runtime, so it returns exit 2 with that reason. Its test is `#[ignore]`d.

use std::path::Path;

use serde::{Deserialize, Serialize};
use tsr_domain::{
    ActionKind, Candidate, CandidateId, EventId, Hash, Observation, OutcomeDefId, Source,
};
use tsr_policy::gate::{ActionArgs, BoundaryAction};

use crate::session::{SessionCtx, SessionError};
use crate::task::CorpusTask;

/// The typed arguments of a scripted step, in a serializable form.
///
/// `tsr_policy::gate::ActionArgs` carries no serde derives (it is policy's type, and this crate
/// does not modify policy), so the fixture format uses this mirror and converts to `ActionArgs`
/// when the gate is consulted. The two are kept in step by [`StepArgs::to_action_args`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "arg", rename_all = "snake_case")]
pub enum StepArgs {
    /// An `Inspect` of a path.
    InspectPath { path: String },
    /// An `Inspect` via an `rts` tool.
    InspectRtsTool { tool: String },
    /// A `RunApprovedTest` naming an allowlisted command id.
    TestId { id: String },
    /// A `ProposePatch` of this many bytes.
    PatchBytes { bytes: u64 },
    /// A `RequestApproval` for a boundary action.
    Boundary { action: String },
    /// A `Generate`.
    Generate,
    /// A `Stop`.
    Stop,
}

impl StepArgs {
    /// Convert to the gate's typed arguments.
    pub fn to_action_args(&self) -> ActionArgs {
        match self {
            StepArgs::InspectPath { path } => ActionArgs::InspectPath(path.clone()),
            StepArgs::InspectRtsTool { tool } => ActionArgs::InspectRtsTool(tool.clone()),
            StepArgs::TestId { id } => ActionArgs::TestId(id.clone()),
            StepArgs::PatchBytes { bytes } => ActionArgs::PatchBytes(*bytes),
            StepArgs::Boundary { action } => ActionArgs::Boundary(match action.as_str() {
                "promote_diff" => BoundaryAction::PromoteDiff,
                "non_allowlisted_command" => BoundaryAction::NonAllowlistedCommand,
                _ => BoundaryAction::Network,
            }),
            StepArgs::Generate => ActionArgs::Generate,
            StepArgs::Stop => ActionArgs::Stop,
        }
    }
}

/// Why a driver failed.
#[derive(Debug, thiserror::Error)]
pub enum DriverError {
    #[error("session: {0}")]
    Session(#[from] SessionError),
    #[error("the fixture session is malformed: {0}")]
    Fixture(String),
    #[error("the driver is not available in this sandbox: {0}")]
    Unavailable(String),
    /// The Tier 0 controller refused a step (D6): read-before-test, or the budget is exhausted.
    #[error("tier 0: {0}")]
    Tier0(#[from] Tier0Error),
}

/// What a driver did, for the per-task result record.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DriverReport {
    /// Generator tokens the driver spent.
    pub generator_tokens: u64,
    /// Tool minutes the driver spent.
    pub tool_minutes: u32,
    /// Wall-clock milliseconds the driver spent.
    pub wall_clock_ms: u64,
    /// Gated actions the gate denied, with the gate's reason.
    pub denials: Vec<Denial>,
    /// Whether the driver reached a `Stop`.
    pub stopped: bool,
    /// Whether the attempt ended on a gate denial (D6): a denial ends the attempt, it does not
    /// continue to the next step.
    #[serde(default)]
    pub ended_on_denial: bool,
    /// Whether the attempt ended on the Tier 0 budget (D6): the controller recorded `Stop(budget)`
    /// and the attempt ended there. The caller closes the task and does not run the oracle.
    #[serde(default)]
    pub stopped_on_budget: bool,
    /// Whether the attempt ended on a Tier 0 refusal other than the budget (D6, F2): a
    /// read-before-test refusal ends *this* task — it is closed as rejected and the batch
    /// continues with the next task — rather than aborting the whole batch.
    #[serde(default)]
    pub stopped_on_tier0: bool,
    /// The Tier 0 refusal's reason, when [`DriverReport::stopped_on_tier0`] is set.
    #[serde(default)]
    pub tier0_reason: Option<String>,
}

/// One recorded denial.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Denial {
    /// The candidate the gate refused.
    pub candidate: String,
    /// The gate's own reason line.
    pub reason: String,
}

/// Drives one engine task.
pub trait Driver {
    /// The driver's name, as it appears in the results.
    fn name(&self) -> &'static str;

    /// Drive the session to a close. The session is already open (`TaskOpened` recorded).
    ///
    /// `copy` is the absolute path of the task's disposable copy (D3): every session step that
    /// touches files or runs a command runs with `copy` as its working directory, and only there.
    /// A driver that edits the repository instead of the copy leaves the fix where the oracle
    /// cannot see it, so the copy is an argument, not ambient state.
    fn drive(
        &self,
        ctx: &mut SessionCtx,
        task: &CorpusTask,
        copy: &Path,
    ) -> Result<DriverReport, DriverError>;
}

// ---------------------------------------------------------------------------------------------
// The scripted driver
// ---------------------------------------------------------------------------------------------

/// One step of a fixture session: a candidate action, its forecast, and what follows.
///
/// The forecast is recorded *before* the action is committed, at the state version current when the
/// step runs — which is what the gate and the reconstruct invariant require. `forecast` is `None`
/// for a step that deliberately omits one, so a test can prove the gate denies it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScriptedStep {
    /// The typed action.
    pub kind: ActionKind,
    /// The typed arguments the gate reads.
    pub args: StepArgs,
    /// The forecast probability, or `None` to omit the forecast (the gate must then deny a gated
    /// kind).
    pub forecast: Option<f64>,
    /// The named event the forecast is about.
    pub forecast_event: Option<String>,
    /// Whether to dispatch and execute the action after committing it.
    pub dispatch: bool,
    /// The exit code the execution reports, when `dispatch` is true.
    pub exit_code: Option<i32>,
    /// The duration the execution reports, in milliseconds.
    pub duration_ms: u64,
    /// Generator tokens this step spends.
    pub generator_tokens: u64,
    /// Tool minutes this step spends.
    pub tool_minutes: u32,
    /// Wall-clock milliseconds this step spends.
    pub wall_clock_ms: u64,
    /// A file edit this step applies inside the task's disposable copy (D3), or `None` for a step
    /// that touches no file. The path is relative to the copy's root; the driver writes it there
    /// and nowhere else, so a session's fix lands where the oracle judges it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub edit: Option<ScriptedEdit>,
}

/// A file edit a scripted step applies inside the task's disposable copy (D3).
///
/// The path is relative to the copy's root and must not escape it: an absolute path or a `..`
/// component is refused, so a fixture cannot write outside the copy the oracle judges.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScriptedEdit {
    /// The path to write, relative to the copy's root.
    pub path: String,
    /// The file's new contents.
    pub contents: String,
    /// When set, the edit creates a symlink at `path` pointing at this target instead of writing
    /// `contents`. A session can plant a symlink the snapshot did not have; the tamper check (D3)
    /// rejects it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub symlink: Option<String>,
}

impl ScriptedEdit {
    /// An edit that writes `contents` to `path` (relative to the copy's root).
    pub fn new(path: impl Into<String>, contents: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            contents: contents.into(),
            symlink: None,
        }
    }

    /// Apply the edit inside `copy`, refusing a path that would escape it.
    ///
    /// The copy is the only writable mount (PRD §B10), so a path that is absolute, carries a `..`
    /// component, or is a Windows-style escape (`C:\...`, `..\...`) is an error rather than a write
    /// outside the sandbox. A path that passes the string check is then resolved component by
    /// component with [`std::fs::symlink_metadata`]: `git archive` preserves committed symlinks, so
    /// a mined repository can plant `escape -> <dir outside the repo>` in the snapshot the copy is
    /// materialised from, and `copy.join(rel)` would follow it out of the copy. Any symlink
    /// component, and any resolved path outside the canonicalised copy root, is refused before
    /// `create_dir_all`/`fs::write`.
    pub fn apply(&self, copy: &Path) -> Result<(), DriverError> {
        let rel = Path::new(&self.path);
        if rel.is_absolute()
            || rel
                .components()
                .any(|c| matches!(c, std::path::Component::ParentDir))
            // A Windows drive prefix (`C:\...`) or a `..\` component is a literal escape on this
            // host but is documented as refused, so the guard does not depend on the platform.
            || self.path.contains('\\')
        {
            return Err(DriverError::Fixture(format!(
                "edit path `{}` escapes the copy",
                self.path
            )));
        }
        let target = copy.join(rel);
        // Resolve each component against the copy, refusing any symlink and confirming the result
        // stays under the canonicalised copy root (the pattern of `is_safe_descriptor_path`).
        let root = copy
            .canonicalize()
            .map_err(|e| DriverError::Fixture(format!("resolving copy {}: {e}", copy.display())))?;
        let mut current = root.clone();
        for component in rel.components() {
            let std::path::Component::Normal(part) = component else {
                if matches!(component, std::path::Component::CurDir) {
                    continue;
                }
                return Err(DriverError::Fixture(format!(
                    "edit path `{}` escapes the copy",
                    self.path
                )));
            };
            current.push(part);
            match std::fs::symlink_metadata(&current) {
                // A symlink component (including a symlinked final file) would write through it.
                Ok(meta) if meta.file_type().is_symlink() => {
                    return Err(DriverError::Fixture(format!(
                        "edit path `{}` escapes the copy through a symlink",
                        self.path
                    )));
                }
                // A component that does not exist yet is fine: `create_dir_all` will make it.
                Ok(_) | Err(_) => {}
            }
        }
        // Defence in depth: the resolved path must still be under the copy root.
        if let Ok(resolved) = current.canonicalize() {
            if !resolved.starts_with(&root) {
                return Err(DriverError::Fixture(format!(
                    "edit path `{}` escapes the copy",
                    self.path
                )));
            }
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| DriverError::Fixture(format!("creating {}: {e}", parent.display())))?;
        }
        if let Some(link) = &self.symlink {
            // A session can plant a symlink the snapshot did not have; the tamper check (D3) rejects
            // it. The link target is written verbatim, so a link to a path outside the copy is
            // exactly the escape the tamper check must catch.
            let _ = std::fs::remove_file(&target);
            return std::os::unix::fs::symlink(link, &target).map_err(|e| {
                DriverError::Fixture(format!("symlinking {}: {e}", target.display()))
            });
        }
        std::fs::write(&target, &self.contents)
            .map_err(|e| DriverError::Fixture(format!("writing {}: {e}", target.display())))
    }
}

impl ScriptedStep {
    /// A step that inspects a path, with a forecast.
    pub fn inspect(path: &str, forecast: f64) -> Self {
        Self {
            kind: ActionKind::Inspect,
            args: StepArgs::InspectPath {
                path: path.to_string(),
            },
            forecast: Some(forecast),
            forecast_event: Some("test_pass".to_string()),
            dispatch: false,
            exit_code: None,
            duration_ms: 0,
            generator_tokens: 0,
            tool_minutes: 0,
            wall_clock_ms: 0,
            edit: None,
        }
    }

    /// A step that runs an allowlisted test, with a forecast, dispatched and executed.
    pub fn run_test(test_id: &str, forecast: f64, exit_code: i32) -> Self {
        Self {
            kind: ActionKind::RunApprovedTest,
            args: StepArgs::TestId {
                id: test_id.to_string(),
            },
            forecast: Some(forecast),
            forecast_event: Some("test_pass".to_string()),
            dispatch: true,
            exit_code: Some(exit_code),
            duration_ms: 1200,
            generator_tokens: 0,
            tool_minutes: 1,
            wall_clock_ms: 1200,
            edit: None,
        }
    }

    /// A step that proposes a patch of `bytes` bytes, with a forecast.
    pub fn propose_patch(bytes: u64, forecast: f64) -> Self {
        Self {
            kind: ActionKind::ProposePatch,
            args: StepArgs::PatchBytes { bytes },
            forecast: Some(forecast),
            forecast_event: Some("diff_accepted".to_string()),
            dispatch: false,
            exit_code: None,
            duration_ms: 0,
            generator_tokens: 0,
            tool_minutes: 0,
            wall_clock_ms: 0,
            edit: None,
        }
    }

    /// A step that stops.
    pub fn stop() -> Self {
        Self {
            kind: ActionKind::Stop,
            args: StepArgs::Stop,
            forecast: None,
            forecast_event: None,
            dispatch: false,
            exit_code: None,
            duration_ms: 0,
            generator_tokens: 0,
            tool_minutes: 0,
            wall_clock_ms: 0,
            edit: None,
        }
    }
}

/// A fixture session: the ordered steps a scripted driver replays.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScriptedSession {
    /// A name for the session, for the results.
    pub name: String,
    /// The steps, in order.
    pub steps: Vec<ScriptedStep>,
}

/// Which system a scripted driver emulates.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mode {
    /// System 1: record, gate nothing.
    Record,
    /// System 2: the Tier 0 controller, every gated action through the gate.
    Rules,
}

impl Mode {
    /// The system's name, as it appears in the results.
    pub fn system(self) -> &'static str {
        match self {
            Mode::Record => "record",
            Mode::Rules => "rules",
        }
    }
}

/// Replays a fixture session. Used by every test.
/// A Tier 0 policy violation (D6): the controller refused a step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Tier0Error {
    /// The failing test was about to run before the failing assertion and its call site were read.
    ReadBeforeTest {
        /// The test id the step would have run.
        test_id: String,
        /// The paths that are still unread.
        unread: Vec<String>,
    },
    /// The tool-minute budget is exhausted: the attempt stops with `Stop(budget)`.
    BudgetExhausted {
        /// Tool minutes spent so far.
        spent: u32,
        /// The budget.
        budget: u32,
    },
}

impl std::fmt::Display for Tier0Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Tier0Error::ReadBeforeTest { test_id, unread } => write!(
                f,
                "read-before-test: {test_id} ran before reading {}",
                unread.join(", ")
            ),
            Tier0Error::BudgetExhausted { spent, budget } => {
                write!(f, "budget exhausted: {spent}/{budget} tool-minutes")
            }
        }
    }
}

impl std::error::Error for Tier0Error {}

/// A scripted repository for the Tier 0 controller: the failing assertion, its call site, and the
/// allowlisted test id that runs the failing test. The controller reads no files: it is told which
/// paths the session inspected, and decides from that.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScriptedRepo {
    /// The path of the failing assertion.
    pub assertion: String,
    /// The path of the failing assertion's call site.
    pub call_site: String,
    /// The allowlisted test id that runs the failing test.
    pub failing_test_id: String,
}

impl ScriptedRepo {
    /// A scripted repository.
    pub fn new(
        assertion: impl Into<String>,
        call_site: impl Into<String>,
        failing_test_id: impl Into<String>,
    ) -> Self {
        Self {
            assertion: assertion.into(),
            call_site: call_site.into(),
            failing_test_id: failing_test_id.into(),
        }
    }
}

/// The Tier 0 controller (PRD §B9): inspect until the failing assertion and its call site are
/// read, then run the test, then propose; stop on budget; a denial ends the attempt.
///
/// The controller is a plain value: the driver owns it and calls [`Tier0Controller::check`] for
/// each step before the step is proposed. It enforces the two rules that are decidable from the
/// step stream alone — read-before-test and the budget stop. The third rule, stop on denial, is
/// enforced by the driver, which ends the attempt when the gate denies a step.
#[derive(Clone, Debug)]
pub struct Tier0Controller {
    repo: ScriptedRepo,
    budget: u32,
    spent: u32,
    read_assertion: bool,
    read_call_site: bool,
    ran_test: bool,
    stopped_on_budget: bool,
}

impl Tier0Controller {
    /// A controller for a scripted repository and a tool-minute budget.
    pub fn new(repo: ScriptedRepo, budget: u32) -> Self {
        Self {
            repo,
            budget,
            spent: 0,
            read_assertion: false,
            read_call_site: false,
            ran_test: false,
            stopped_on_budget: false,
        }
    }

    /// Whether the failing assertion has been read.
    pub fn read_assertion(&self) -> bool {
        self.read_assertion
    }

    /// Whether the failing assertion's call site has been read.
    pub fn read_call_site(&self) -> bool {
        self.read_call_site
    }

    /// Whether the failing test has run.
    pub fn ran_test(&self) -> bool {
        self.ran_test
    }

    /// Whether the controller stopped on budget.
    pub fn stopped_on_budget(&self) -> bool {
        self.stopped_on_budget
    }

    /// Tool minutes spent so far.
    pub fn spent(&self) -> u32 {
        self.spent
    }

    /// The budget.
    pub fn budget(&self) -> u32 {
        self.budget
    }

    /// Check a step against the Tier 0 policy. Returns `Ok(())` when the step may proceed, or a
    /// [`Tier0Error`] when the policy refuses it.
    pub fn check(&mut self, step: &ScriptedStep) -> Result<(), Tier0Error> {
        // Rule 2: stop on budget. The step that would *cross* the budget is refused, not the step
        // after it (D6, F1): checking `spent >= budget` before adding the step's minutes let the
        // crossing step be committed, so a session whose overspending step was last was accepted.
        // The check is `spent + step.tool_minutes > budget`, so a step that exactly fits is
        // allowed and the first step that would exceed the budget stops the attempt.
        if self.spent.saturating_add(step.tool_minutes) > self.budget {
            self.stopped_on_budget = true;
            return Err(Tier0Error::BudgetExhausted {
                spent: self.spent,
                budget: self.budget,
            });
        }

        match &step.args {
            StepArgs::InspectPath { path } => {
                if *path == self.repo.assertion {
                    self.read_assertion = true;
                }
                if *path == self.repo.call_site {
                    self.read_call_site = true;
                }
            }
            StepArgs::TestId { id } => {
                // Rule 1: read-before-test. The failing test may not run until the failing
                // assertion and its call site have both been read.
                if *id == self.repo.failing_test_id && !self.ran_test {
                    let mut unread = Vec::new();
                    if !self.read_assertion {
                        unread.push(self.repo.assertion.clone());
                    }
                    if !self.read_call_site {
                        unread.push(self.repo.call_site.clone());
                    }
                    if !unread.is_empty() {
                        return Err(Tier0Error::ReadBeforeTest {
                            test_id: id.clone(),
                            unread,
                        });
                    }
                    self.ran_test = true;
                }
            }
            _ => {}
        }

        self.spent = self.spent.saturating_add(step.tool_minutes);
        Ok(())
    }
}

pub struct ScriptedDriver {
    mode: Mode,
    session: ScriptedSession,
    tier0: Option<Tier0Controller>,
}

impl ScriptedDriver {
    /// A scripted driver for a mode and a session.
    pub fn new(mode: Mode, session: ScriptedSession) -> Self {
        Self {
            mode,
            session,
            tier0: None,
        }
    }

    /// Attach the Tier 0 controller (D6) for the rules system. In `Mode::Rules` the driver runs
    /// the controller for each step; in `Mode::Record` it is ignored (system 1 gates nothing).
    pub fn with_tier0(mut self, repo: ScriptedRepo, budget: u32) -> Self {
        self.tier0 = Some(Tier0Controller::new(repo, budget));
        self
    }

    /// The Tier 0 controller, if one is attached.
    pub fn tier0(&self) -> Option<&Tier0Controller> {
        self.tier0.as_ref()
    }

    /// The session this driver replays.
    pub fn session(&self) -> &ScriptedSession {
        &self.session
    }

    /// The mode this driver runs in.
    pub fn mode(&self) -> Mode {
        self.mode
    }

    /// Mint a candidate for a step, with the current head as its parent.
    fn candidate_for(
        &self,
        ctx: &SessionCtx,
        step: &ScriptedStep,
    ) -> Result<Candidate, DriverError> {
        let args = Hash::of(&[serde_json::to_vec(&step.args)
            .map_err(|e| DriverError::Fixture(e.to_string()))?
            .as_slice()]);
        let parents: Vec<EventId> = ctx.events().last().map(|e| vec![e.id]).unwrap_or_default();
        Candidate::new(step.kind, Source::Rules, args, parents, Vec::new())
            .map_err(|e| DriverError::Fixture(e.to_string()))
    }

    /// The outcome-def id a step's forecast names.
    fn forecast_event(&self, step: &ScriptedStep) -> OutcomeDefId {
        let name = step.forecast_event.as_deref().unwrap_or("test_pass");
        OutcomeDefId(Hash::of(&[name.as_bytes()]))
    }
}

impl Driver for ScriptedDriver {
    fn name(&self) -> &'static str {
        match self.mode {
            Mode::Record => "scripted-record",
            Mode::Rules => "scripted-rules",
        }
    }

    fn drive(
        &self,
        ctx: &mut SessionCtx,
        _task: &CorpusTask,
        copy: &Path,
    ) -> Result<DriverReport, DriverError> {
        let mut report = DriverReport::default();
        // The Tier 0 controller (D6) is consulted for each step in rules mode, before the step is
        // proposed. A refusal ends the attempt: the driver returns the error and does not continue.
        let mut tier0 = self.tier0.clone();
        for step in &self.session.steps {
            if self.mode == Mode::Rules {
                if let Some(controller) = tier0.as_mut() {
                    match controller.check(step) {
                        Ok(()) => {}
                        // D6: a budget stop ends *this* attempt with `Stop(budget)`. It is not an
                        // error that aborts the batch: the driver records the Stop, marks the
                        // report, and returns, so `run_one` closes the task and the batch moves on
                        // to the next task.
                        Err(Tier0Error::BudgetExhausted { .. }) => {
                            ctx.stop("budget")?;
                            report.stopped = true;
                            report.stopped_on_budget = true;
                            return Ok(report);
                        }
                        // D6 (F2): a read-before-test refusal is the same class — it ends *this*
                        // task, not the batch. The controller records `Stop(blocked)` with the
                        // refusal's reason, marks the report, and returns; `run_one` closes the
                        // task as rejected and the batch continues with the next task. Before this
                        // fix the refusal returned `Err`, which aborted the whole batch.
                        Err(e @ Tier0Error::ReadBeforeTest { .. }) => {
                            ctx.stop("blocked")?;
                            report.stopped = true;
                            report.stopped_on_tier0 = true;
                            report.tier0_reason = Some(e.to_string());
                            return Ok(report);
                        }
                    }
                }
            }
            let candidate = self.candidate_for(ctx, step)?;
            ctx.propose(candidate.clone())?;

            // Record the forecast before the action, at the current state version. In record mode
            // the forecast is still recorded (the hook records what it sees); the difference is
            // that record mode never consults the gate.
            if let Some(p) = step.forecast {
                ctx.forecast(candidate.id, self.forecast_event(step), p)?;
            }

            match self.mode {
                Mode::Record => {
                    // System 1 gates nothing: commit without consulting the gate.
                    ctx.commit_ungated(&candidate)?;
                }
                Mode::Rules => {
                    // System 2: every action passes through the gate.
                    match ctx.commit(&candidate, step.args.to_action_args()) {
                        Ok(_) => {}
                        Err(SessionError::Denied(reason)) => {
                            report.denials.push(Denial {
                                candidate: candidate.id.to_string(),
                                reason,
                            });
                            // A denied action does not happen: no dispatch, no execution.
                            report.generator_tokens += step.generator_tokens;
                            report.tool_minutes += step.tool_minutes;
                            report.wall_clock_ms += step.wall_clock_ms;
                            if step.kind == ActionKind::Stop {
                                report.stopped = true;
                            }
                            // D6: a denial ends the attempt; it does not continue to the next step.
                            report.ended_on_denial = true;
                            break;
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
            }

            // D3: a step that edits a file applies it inside the task's disposable copy, and only
            // there. The edit happens after the gate has allowed the step (a denied step does not
            // happen), so a rules-mode denial leaves the copy untouched.
            if let Some(edit) = &step.edit {
                edit.apply(copy)?;
            }

            if step.dispatch {
                ctx.dispatch(candidate.id)?;
                let log_digest = Some(Hash::of(&[candidate.id.to_string().as_bytes()]));
                ctx.execute(candidate.id, step.exit_code, log_digest, step.duration_ms)?;
            }

            report.generator_tokens += step.generator_tokens;
            report.tool_minutes += step.tool_minutes;
            report.wall_clock_ms += step.wall_clock_ms;
            if step.kind == ActionKind::Stop {
                report.stopped = true;
            }
        }
        Ok(report)
    }
}

// ---------------------------------------------------------------------------------------------
// The Claude Code driver (specified, not implemented)
// ---------------------------------------------------------------------------------------------

/// Drives a session through Claude Code's hook protocol.
///
/// # How the hook payloads arrive
///
/// Claude Code invokes a configured hook command with a JSON payload on stdin and reads a JSON
/// decision on stdout. The batch runner installs the hook as `tsr hook` (the `Hook` arm of
/// `tsr-cli`'s `main.rs`), which maps the payload's `tool_name` to an [`ActionKind`] with
/// `tsr_policy::action::action_kind_for_tool`, evaluates the gate, and prints
/// `{"decision":"allow"|"deny"|"ask","reason":"…"}`. The payloads that matter are:
///
/// - `PreToolUse` — carries `tool_name` and `tool_input`; the hook answers allow/deny/ask. A
///   `deny` stops the tool call; the model reads the reason and must record what it names.
/// - `PostToolUse` — carries the tool result; the hook records an `Executed` (or `Unresolved`)
///   event, reconciling the dispatch.
/// - `Stop` — the model's turn ended; the hook records `TaskClosed`.
///
/// # The egress allowlist
///
/// The sandbox has no network. The hook denies any tool call that would reach the network
/// (`WebFetch`, `WebSearch`, a `Bash` argv naming `curl`/`wget`/`git push`) with
/// `BoundaryAction::Network`, which the profile requires an approval for. The only egress is the
/// model provider itself, which the harness owns, not the task.
///
/// # The CLAUDE.md instruction to forecast first
///
/// The task's `CLAUDE.md` tells the model: *before any `Bash` (test) or `Edit`/`Write` (patch)
/// call, record a forecast for the current state version; the hook will deny the call otherwise,
/// naming what to record.* This is the forecast-gated execution rule (PRD §B5) stated where the
/// model can read it.
///
/// # Why this is a stub
///
/// The swarm sandbox has no Claude Code binary and no container runtime, so this driver cannot run
/// here. It returns exit 2 with that reason. Its test is `#[ignore]`d with the same reason; the
/// operator runs it on a host that has both.
pub struct ClaudeCodeDriver {
    /// The `tsr` binary the hook is installed as.
    pub tsr_bin: String,
    /// The model to drive.
    pub model: String,
}

impl ClaudeCodeDriver {
    /// The reason this driver is unavailable in the swarm sandbox.
    pub const UNAVAILABLE: &'static str =
        "no Claude Code binary and no container runtime in the swarm sandbox; the operator runs \
         this driver on a host that has both";

    /// A driver for a `tsr` binary and a model.
    pub fn new(tsr_bin: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            tsr_bin: tsr_bin.into(),
            model: model.into(),
        }
    }
}

impl Driver for ClaudeCodeDriver {
    fn name(&self) -> &'static str {
        "claude-code"
    }

    fn drive(
        &self,
        _ctx: &mut SessionCtx,
        _task: &CorpusTask,
        _copy: &Path,
    ) -> Result<DriverReport, DriverError> {
        Err(DriverError::Unavailable(Self::UNAVAILABLE.to_string()))
    }
}

/// The exit code a driver that cannot run returns (PRD §B11 M3: "exit 2 with the reason").
pub const DRIVER_UNAVAILABLE_EXIT: i32 = 2;

/// Read a fixture session from a JSON file.
pub fn read_session(path: &Path) -> Result<ScriptedSession, DriverError> {
    let text = std::fs::read_to_string(path).map_err(|e| DriverError::Fixture(e.to_string()))?;
    serde_json::from_str(&text).map_err(|e| DriverError::Fixture(e.to_string()))
}

/// Write a fixture session as JSON.
pub fn write_session(path: &Path, session: &ScriptedSession) -> Result<(), DriverError> {
    let text =
        serde_json::to_string_pretty(session).map_err(|e| DriverError::Fixture(e.to_string()))?;
    std::fs::write(path, text).map_err(|e| DriverError::Fixture(e.to_string()))
}

/// An observation a driver records, for the fixture sessions.
pub fn evidence_observation(digest: Hash, source: &str) -> Observation {
    Observation::Evidence {
        digest,
        source: source.to_string(),
    }
}

/// A candidate id from a hex string, for tests that name one.
pub fn candidate_id_from_hex(hex: &str) -> Result<CandidateId, DriverError> {
    Hash::from_hex(hex)
        .map(CandidateId)
        .map_err(|e| DriverError::Fixture(e.to_string()))
}
