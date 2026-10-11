//! `tsr-batch` — the batch runner for systems 1 (record) and 2 (rules) over the M3 corpus.
//!
//! This crate is the corpus-and-batch half of milestone M3 (PRD §B9, §B11 M3). It runs one engine
//! task per corpus task, under one of two systems:
//!
//! - **System 1 (record)** records the session and gates nothing. It is the baseline.
//! - **System 2 (rules)** runs the Tier 0 controller and puts every gated action through
//!   `tsr-policy`'s gate under a RATCHET profile. A denial is recorded with its reason.
//!
//! The pieces:
//!
//! - [`task`] — the corpus task as the runner reads it, from `tasks.jsonl`;
//! - [`session`] — the recorded session: events through `tsr-store`, gated by `tsr-policy`;
//! - [`driver`] — the [`driver::Driver`] trait, [`driver::ScriptedDriver`] and the specified
//!   [`driver::ClaudeCodeDriver`] stub;
//! - [`run`] — the batch orchestration: open, provision, drive, oracle, close, record;
//! - [`results`] — the per-task result record `tsr eval baseline` reads;
//! - [`invariants`] — the check that every recorded session satisfies
//!   `tsr_cli::reconstruct::Session`'s invariants.
//!
//! Nothing in this crate runs a repository's tests on the host: every command goes through a
//! [`tsr_corpus::Runner`], which in production is `ExecRunner` invoking `tsr exec -- <argv>` in the
//! sealed sibling sandbox (PRD §B4).

pub mod driver;
pub mod invariants;
pub mod results;
pub mod run;
pub mod session;
pub mod task;

pub use driver::{
    ClaudeCodeDriver, Driver, DriverError, DriverReport, Mode, ScriptedDriver, ScriptedSession,
    ScriptedStep, StepArgs, DRIVER_UNAVAILABLE_EXIT,
};
pub use invariants::{check_session_invariants, InvariantViolation};
pub use results::{read_results, write_results, OracleVerdict, Outcome, TaskResult};
pub use run::{grant_for, oracle_verdict, run_all, run_one, task_id, BatchConfig, RunError};
pub use session::{SessionCtx, SessionError};
pub use task::{read_tasks, write_tasks, CorpusTask, TaskError, TaskKind};
