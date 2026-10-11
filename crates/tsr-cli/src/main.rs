//! `tsr` — the operator's CLI and the hook binary of the TSR Decision Engine.
//!
//! M0 skeleton: the command groups exist and say what milestone fills them. Every unimplemented
//! command exits 2 with one sentence naming the PRD section, so a swarm or an operator never
//! mistakes scaffolding for a feature.

#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::process::ExitCode;

mod batch_cmd;
mod corpus_cmd;
mod policy_cmd;

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
    /// Run a batch of corpus tasks through a driver (M3, B9, B11).
    Run {
        /// record | rules
        #[arg(long)]
        mode: String,
        /// The corpus task manifest (tasks.jsonl).
        #[arg(long)]
        tasks: PathBuf,
        /// scripted | claude-code
        #[arg(long)]
        driver: String,
        /// Output directory: results.jsonl and the per-task stores.
        #[arg(long)]
        out: PathBuf,
    },
    Status {
        task: String,
    },
    /// Export a task's trace as JSON (M0 exit evidence).
    Export {
        task: String,
    },
}

#[derive(Subcommand)]
enum PolicyCmd {
    Check { profile: String, event: String },
}

#[derive(Subcommand)]
enum CorpusCmd {
    /// Mine a repository's first-parent history for red→green flip tasks (B9).
    Mine {
        repo: String,
        /// Output directory: `tasks.jsonl` and the CAS are written here.
        #[arg(long)]
        out: PathBuf,
        /// Only walk commits reachable from this revision (default: HEAD).
        #[arg(long)]
        since: Option<String>,
        /// The suite command (default: `cargo test`).
        #[arg(long)]
        suite: Option<String>,
    },
    Mutants {
        repo: String,
        /// Output directory: `tasks.jsonl` and the CAS are written here.
        #[arg(long)]
        out: PathBuf,
    },
    /// Split a corpus into train and holdout with clone groups and leakage probes (B9).
    Split {
        /// The corpus directory: `tasks.jsonl` and the CAS are read here.
        dir: PathBuf,
        /// The split rule: `time` (forward-time by task) or `repo` (lineage-separated).
        #[arg(long)]
        by: String,
        /// The holdout fraction, in (0, 1).
        #[arg(long)]
        holdout: f64,
        /// The seed: the same seed and corpus give the same split.
        #[arg(long)]
        seed: u64,
    },
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
            TaskCmd::Run {
                mode,
                tasks,
                driver,
                out,
            } => batch_cmd::run_cmd(&mode, &tasks, &driver, &out),
            TaskCmd::Status { .. } => not_yet("task status", "B11 M0"),
            TaskCmd::Export { .. } => not_yet("task export", "B11 M0 — the trace export"),
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
        Cmd::Review { .. } => not_yet("review", "B5/B7 (M2)"),
        Cmd::Reflect { .. } => not_yet("reflect", "B7 the Mirror view (M2)"),
        Cmd::Corpus { cmd } => match cmd {
            CorpusCmd::Mine {
                repo,
                out,
                since,
                suite,
            } => corpus_cmd::mine_cmd(repo, out, since, suite),
            CorpusCmd::Mutants { repo, out } => corpus_cmd::mutants_cmd(repo, out),
            CorpusCmd::Split {
                dir,
                by,
                holdout,
                seed,
            } => corpus_cmd::split_cmd(dir, by, holdout, seed),
        },
        Cmd::Data { .. } => not_yet("data", "B8/B10 (M4)"),
        Cmd::Train => not_yet("train", "B8 Tier 1 (M4)"),
        Cmd::Eval => not_yet("eval", "B9 (M4–M5)"),
        Cmd::Model { .. } => not_yet("model", "B8 bundles (M5)"),
    }
}
