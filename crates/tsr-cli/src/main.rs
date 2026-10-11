//! `tsr` — the operator's CLI and the hook binary of the TSR Decision Engine.
//!
//! M0 (PRD §B11) fills `hook`, `exec`, `task` and `forecast`; the later-milestone commands still
//! exit 2 with one sentence naming the PRD section, so scaffolding is never mistaken for a feature.

#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use std::io::Read;
use std::process::ExitCode;
use tsr_cli::{exec, hook, store_path, task};

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
        /// The task to record against (defaults to the session task)
        #[arg(long)]
        task: Option<String>,
    },
    /// The hook entry point: reads a Claude Code hook payload on stdin and records it (M0, B4).
    Hook {
        /// PreToolUse | PostToolUse | SessionStart | Stop
        #[arg(long, default_value = "")]
        stage: String,
        /// Drive realistic payloads through the hook and write fixtures/m0/session.events.jsonl
        #[arg(long)]
        selftest: bool,
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
    Open {
        /// The task name (defaults to the session id)
        #[arg(long)]
        name: Option<String>,
        /// The workspace the task runs in
        #[arg(long)]
        workspace: Option<String>,
        /// The task's intent
        #[arg(long)]
        intent: Option<String>,
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
        Cmd::Task { cmd } => run_task(cmd),
        Cmd::Forecast {
            event,
            p,
            source,
            task,
        } => run_forecast(&event, p, &source, task.as_deref()),
        Cmd::Hook { stage, selftest } => run_hook(&stage, selftest),
        Cmd::Exec { argv } => run_exec(&argv),
        Cmd::Policy { .. } => not_yet("policy check", "B5 (M1, the RATCHET profile)"),
        Cmd::Review { .. } => not_yet("review", "B5/B7 (M2)"),
        Cmd::Reflect { .. } => not_yet("reflect", "B7 the Mirror view (M2)"),
        Cmd::Corpus { .. } => not_yet("corpus", "B9 (M3)"),
        Cmd::Data { .. } => not_yet("data", "B8/B10 (M4)"),
        Cmd::Train => not_yet("train", "B8 Tier 1 (M4)"),
        Cmd::Eval => not_yet("eval", "B9 (M4–M5)"),
        Cmd::Model { .. } => not_yet("model", "B8 bundles (M5)"),
    }
}

fn run_task(cmd: TaskCmd) -> ExitCode {
    let store = match store_path::open_store() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tsr: cannot open store: {e}");
            return ExitCode::from(1);
        }
    };
    match cmd {
        TaskCmd::Open {
            name,
            workspace,
            intent,
        } => {
            let name = name.unwrap_or_else(|| "default".to_string());
            let workspace = workspace.unwrap_or_else(|| {
                std::env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_default()
            });
            let intent = intent.unwrap_or_default();
            match task::open(&store, &name, &workspace, &intent) {
                Ok(id) => {
                    println!("{id}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("tsr: task open failed: {e}");
                    ExitCode::from(1)
                }
            }
        }
        TaskCmd::Status { task: name } => {
            let id = store_path::resolve_task(&store, &name);
            match task::status(&store, &id) {
                Ok((count, last)) => {
                    println!("task {id}: {count} events");
                    if let Some(ev) = last {
                        println!("last: {}", serde_json::to_string(&ev).unwrap_or_default());
                    }
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("tsr: task status failed: {e}");
                    ExitCode::from(1)
                }
            }
        }
        TaskCmd::Export { task: name } => {
            let id = store_path::resolve_task(&store, &name);
            match task::export(&store, &id) {
                Ok(json) => {
                    print!("{json}");
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("tsr: task export failed: {e}");
                    ExitCode::from(1)
                }
            }
        }
    }
}

fn run_forecast(event: &str, p: f64, source: &str, task: Option<&str>) -> ExitCode {
    let store = match store_path::open_store() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tsr: cannot open store: {e}");
            return ExitCode::from(1);
        }
    };
    let task_id = match task {
        Some(name) => store_path::resolve_task(&store, name),
        None => store_path::task_for_session("default"),
    };
    match tsr_cli::hook::record_forecast(&store, &task_id, event, p, source) {
        Ok(id) => {
            println!("{id}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("tsr: forecast failed: {e}");
            ExitCode::from(1)
        }
    }
}

fn run_hook(stage: &str, selftest: bool) -> ExitCode {
    if selftest {
        return match tsr_cli::hook::selftest() {
            Ok(path) => {
                println!("wrote {}", path.display());
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("tsr: hook selftest failed: {e}");
                ExitCode::from(1)
            }
        };
    }
    let mut input = String::new();
    if std::io::stdin().read_to_string(&mut input).is_err() {
        // M0 records and never blocks: an unreadable payload is not a reason to fail the turn.
        return ExitCode::SUCCESS;
    }
    let store = match store_path::open_store() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tsr: cannot open store: {e}");
            return ExitCode::SUCCESS;
        }
    };
    match hook::run(&store, stage, &input) {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => {
            // Never block: a recording failure is logged, not enforced.
            eprintln!("tsr: hook recorded nothing: {e}");
            ExitCode::SUCCESS
        }
    }
}

fn run_exec(argv: &[String]) -> ExitCode {
    let store = match store_path::open_store() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tsr: cannot open store: {e}");
            return ExitCode::from(1);
        }
    };
    let workspace = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
    let task = store_path::task_for_session("default");
    match exec::run(&store, &task, &workspace, argv) {
        Ok(outcome) => ExitCode::from(outcome.exit_code.clamp(0, 255) as u8),
        Err(exec::ExecRefusal::NoRuntime) => {
            eprintln!(
                "tsr: no container runtime reachable (podman/docker); refusing to run on the host"
            );
            ExitCode::from(exec::EXIT_NO_RUNTIME as u8)
        }
    }
}
