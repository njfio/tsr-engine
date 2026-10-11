//! `tsr` — the operator's CLI and the hook binary of the TSR Decision Engine.
//!
//! M0 skeleton: the command groups exist and say what milestone fills them. Every unimplemented
//! command exits 2 with one sentence naming the PRD section, so a swarm or an operator never
//! mistakes scaffolding for a feature.

#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use std::process::ExitCode;

mod policy_cmd;
use tsr_cli::note_cmd;
use tsr_cli::reconcile_cmd;

#[derive(Parser)]
#[command(
    name = "tsr",
    version,
    about = "TSR Decision Engine — PRD v0.3.1 (docs/PRD.md)"
)]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Task intake, status, trace export (M0: B3, B5, B11).
    Task {
        #[command(subcommand)]
        cmd: TaskCmd,
    },
    /// Record a forecast for the current state version before a gated action (B5, B7).
    Forecast {
        /// Named event, e.g. task_accepted_in_budget@v1
        #[arg(long)]
        event: String,
        /// Raw probability in [0, 1]; always recorded, acted on only when calibrated
        #[arg(long)]
        p: f64,
        /// Forecaster: generator | controller | operator
        #[arg(long, default_value = "generator")]
        source: String,
    },
    /// The hook entry point: reads a Claude Code hook payload on stdin and records it (M0, B4).
    Hook {
        /// PreToolUse | PostToolUse | SessionStart | Stop
        #[arg(long)]
        stage: String,
    },
    /// Run a command in the sealed sibling sandbox and reconcile the result (M0, B4).
    Exec {
        #[arg(trailing_var_arg = true)]
        argv: Vec<String>,
    },
    /// Evaluate a RATCHET protection profile against an event (M1, B5).
    Policy {
        #[command(subcommand)]
        cmd: PolicyCmd,
    },
    /// Record the operator's experience note on a task (M2, B7).
    Note {
        task: String,
        /// A short note on friction or trust; never reward truth.
        #[arg(long)]
        text: String,
        /// The store holding the task's log.
        #[arg(long, default_value = "tsr.db")]
        store: String,
    },
    /// Label outcomes after close (M2, B5, B7).
    Review { task: String },
    /// The Mirror view: one closed episode, in the book's order (M2, B7).
    Reflect { task: String },
    /// Corpus tooling: mine, mutants (M3, B9).
    Corpus {
        #[command(subcommand)]
        cmd: CorpusCmd,
    },
    /// Dataset validation and `forget` (M4, B8, B10).
    Data {
        #[command(subcommand)]
        cmd: DataCmd,
    },
    /// Train calibration heads (M4, B8).
    Train,
    /// Evaluate: calibration, paired comparison, state isolation (M4–M5, B9).
    Eval,
    /// Bundle registry: promote, rollback (M5, B8).
    Model {
        #[command(subcommand)]
        cmd: ModelCmd,
    },
}

#[derive(Subcommand)]
enum TaskCmd {
    /// Open a task: workspace, intent, criteria, limits, capabilities (FR 001).
    Open,
    Status {
        task: String,
    },
    /// Export a task's trace as JSON (M0 exit evidence).
    Export {
        task: String,
    },
    /// Reconcile dispatches with no recorded outcome (M2, B4/B6).
    Reconcile {
        task: String,
        /// The store holding the task's log.
        #[arg(long, default_value = "tsr.db")]
        store: String,
    },
}

#[derive(Subcommand)]
enum PolicyCmd {
    Check { profile: String, event: String },
}

#[derive(Subcommand)]
enum CorpusCmd {
    Mine { repo: String },
    Mutants { repo: String },
}

#[derive(Subcommand)]
enum DataCmd {
    Validate,
    Forget { episode: String },
}

#[derive(Subcommand)]
enum ModelCmd {
    List,
    Promote { bundle: String },
    Rollback,
}

fn not_yet(what: &str, section: &str) -> ExitCode {
    eprintln!("tsr: `{what}` is not implemented yet — see docs/PRD.md {section}. This is the M0 skeleton.");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match cli.cmd {
        Cmd::Task { cmd } => match cmd {
            TaskCmd::Open => not_yet("task open", "B3/B5 (M0)"),
            TaskCmd::Status { .. } => not_yet("task status", "B11 M0"),
            TaskCmd::Export { .. } => not_yet("task export", "B11 M0 — the trace export"),
            TaskCmd::Reconcile { task, store } => reconcile_cmd::run(&task, &store),
        },
        Cmd::Forecast { .. } => not_yet(
            "forecast",
            "B5 forecast-gated execution (M1) and B7 (M0 records it)",
        ),
        Cmd::Hook { .. } => not_yet("hook", "B4 path A — tsr-hook records every tool call (M0)"),
        Cmd::Exec { .. } => not_yet("exec", "B4 — the sealed sibling sandbox (M0)"),
        Cmd::Policy { cmd } => match cmd {
            PolicyCmd::Check { profile, event } => policy_cmd::run(&profile, &event),
        },
        Cmd::Note { task, text, store } => note_cmd::run(&task, &text, &store),
        Cmd::Review { .. } => not_yet("review", "B5/B7 (M2)"),
        Cmd::Reflect { .. } => not_yet("reflect", "B7 the Mirror view (M2)"),
        Cmd::Corpus { .. } => not_yet("corpus", "B9 (M3)"),
        Cmd::Data { .. } => not_yet("data", "B8/B10 (M4)"),
        Cmd::Train => not_yet("train", "B8 Tier 1 (M4)"),
        Cmd::Eval => not_yet("eval", "B9 (M4–M5)"),
        Cmd::Model { .. } => not_yet("model", "B8 bundles (M5)"),
    }
}
