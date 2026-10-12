//! `tsr` — the operator's CLI and the hook binary of the TSR Decision Engine.
//!
//! M0 (PRD §B11) fills `hook`, `exec`, `task` and `forecast`; the later-milestone commands still
//! exit 2 with one sentence naming the PRD section, so scaffolding is never mistaken for a feature.

#![forbid(unsafe_code)]

use clap::{Parser, Subcommand};
use std::io::Read;
use std::path::PathBuf;
use std::process::ExitCode;
use tsr_cli::{exec, hook, store_path, task};

mod batch_cmd;
mod corpus_cmd;
mod eval_cmd;
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
        /// The gated action being forecast: run_approved_test | propose_patch | request_approval
        #[arg(long)]
        kind: Option<String>,
        /// The action's arguments: the Bash command, or the Edit/Write file path
        #[arg(long)]
        args: Option<String>,
        /// An already-proposed candidate id (alternative to --kind/--args)
        #[arg(long)]
        event: Option<String>,
        /// Raw probability in [0, 1]; always recorded, acted on only when calibrated
        #[arg(long)]
        p: f64,
        /// Forecaster: generator | controller | operator
        #[arg(long, default_value = "generator")]
        source: String,
        /// The task to record against (defaults to the session task)
        #[arg(long)]
        task: Option<String>,
        /// The session to record against (defaults to the session the hook last saw)
        #[arg(long)]
        session: Option<String>,
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
    Eval {
        #[command(subcommand)]
        cmd: EvalCmd,
    },
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
        /// Test-only: run the oracle through a scripted runner (a canned success) instead of the
        /// production `ExecRunner`. Never use this for evidence: a scripted oracle cannot tell a
        /// session that fixed the flipped test from one that did not (D3).
        #[arg(long)]
        scripted_runner: bool,
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
    /// Materialise a snapshot into a disposable copy (D4).
    ///
    /// The batch runner provisions the copy it drives a session in by asking the sealed sandbox to
    /// run this command: `tsr corpus materialize --snapshot <sha> --out <dest>`. It is a real
    /// subcommand so the argv the runner builds names a command that exists.
    Materialize {
        /// The snapshot to materialise: a git revision (sha, tag or ref).
        #[arg(long)]
        snapshot: String,
        /// The directory to materialise into.
        #[arg(long)]
        out: PathBuf,
        /// The repository to materialise from (default: the working directory).
        #[arg(long)]
        repo: Option<PathBuf>,
    },
    /// Run the oracle in a materialised copy and report pass/fail (D4).
    ///
    /// The batch runner runs the oracle by asking the sealed sandbox to run this command:
    /// `tsr corpus oracle --snapshot <dir> --tests <csv>` (the flipped tests) or
    /// `tsr corpus oracle --snapshot <dir> --full-suite` (the regression check). It is a real
    /// subcommand so the argv the runner builds names a command that exists.
    Oracle {
        /// The directory the oracle runs in: the session's copy, not the parent snapshot (D3).
        #[arg(long)]
        snapshot: PathBuf,
        /// The comma-separated test ids to run (the flipped tests).
        #[arg(long, conflicts_with = "full_suite")]
        tests: Option<String>,
        /// Run the full suite instead of a named set of tests.
        #[arg(long)]
        full_suite: bool,
        /// The suite command (default: `cargo test`).
        #[arg(long)]
        suite: Option<String>,
    },
}

#[derive(Subcommand)]
enum DataCmd {
    Validate,
    Forget { episode: String },
}

#[derive(Subcommand)]
enum EvalCmd {
    /// The first numbers with intervals: paired by task, cluster bootstrap by
    /// lineage, 90% intervals, INCONCLUSIVE when an interval spans zero (B9).
    Baseline {
        /// The results directory: `results.jsonl` is read here. Use this when
        /// one file holds both systems; use `--record`/`--rules` when each run
        /// wrote its own directory.
        #[arg(conflicts_with_all = ["record", "rules"])]
        results_dir: Option<PathBuf>,
        /// The record system's results directory (`results.jsonl` is read here).
        #[arg(long, requires = "rules")]
        record: Option<PathBuf>,
        /// The rules system's results directory (`results.jsonl` is read here).
        #[arg(long, requires = "record")]
        rules: Option<PathBuf>,
        /// The pairing rule (only `task` is defined).
        #[arg(long, default_value = "task")]
        pair_by: String,
        /// The bootstrap seed: the same seed gives the same report.
        #[arg(long, default_value_t = tsr_eval::DEFAULT_SEED)]
        seed: u64,
    },
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
            kind,
            args,
            event,
            p,
            source,
            task,
            session,
        } => run_forecast(
            kind.as_deref(),
            args.as_deref(),
            event.as_deref(),
            p,
            &source,
            task.as_deref(),
            session.as_deref(),
        ),
        Cmd::Hook { stage, selftest } => run_hook(&stage, selftest),
        Cmd::Exec { argv } => run_exec(&argv),
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
            CorpusCmd::Materialize {
                snapshot,
                out,
                repo,
            } => corpus_cmd::materialize_cmd(repo.as_deref(), &snapshot, &out),
            CorpusCmd::Oracle {
                snapshot,
                tests,
                full_suite,
                suite,
            } => corpus_cmd::oracle_cmd(&snapshot, tests.as_deref(), full_suite, suite.as_deref()),
        },
        Cmd::Data { .. } => not_yet("data", "B8/B10 (M4)"),
        Cmd::Train => not_yet("train", "B8 Tier 1 (M4)"),
        Cmd::Eval { cmd } => match cmd {
            EvalCmd::Baseline {
                results_dir,
                record,
                rules,
                pair_by,
                seed,
            } => eval_cmd::baseline_cmd(
                results_dir.as_deref(),
                record.as_deref(),
                rules.as_deref(),
                &pair_by,
                seed,
            ),
        },
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
        TaskCmd::Run {
            mode,
            tasks,
            driver,
            out,
            scripted_runner,
        } => batch_cmd::run_cmd(&mode, &tasks, &driver, &out, scripted_runner),
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

fn run_forecast(
    kind: Option<&str>,
    args: Option<&str>,
    event: Option<&str>,
    p: f64,
    source: &str,
    task: Option<&str>,
    session: Option<&str>,
) -> ExitCode {
    let store = match store_path::open_store() {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tsr: cannot open store: {e}");
            return ExitCode::from(1);
        }
    };
    // One session-resolution rule, shared with the hook and `tsr exec`: an explicit `--task` wins,
    // then `--session`/`TSR_SESSION`/the `.tsr/session` file the hook wrote. No "latest task" and no
    // `hash("default")` fallback — with no session we refuse rather than guess.
    let task_id = match task {
        Some(name) => store_path::resolve_task(&store, name),
        None => match store_path::resolve_session_task(session) {
            Ok(id) => id,
            Err(e) => {
                eprintln!("tsr: {e}");
                return ExitCode::from(2);
            }
        },
    };
    if !matches!(source, "generator" | "controller" | "operator") {
        eprintln!("tsr: unknown --source `{source}` (generator|controller|operator)");
        return ExitCode::from(1);
    }
    let result = match (kind, args, event) {
        (Some(kind), Some(args), None) => {
            let kind = match hook::parse_action_kind(kind) {
                Some(k) => k,
                None => {
                    eprintln!(
                        "tsr: unknown --kind `{kind}` (run_approved_test|propose_patch|request_approval)"
                    );
                    return ExitCode::from(1);
                }
            };
            hook::record_forecast_for_action(&store, &task_id, kind, args, p, source)
        }
        (None, None, Some(event)) => {
            let cid = match hook::parse_candidate_id(event) {
                Some(c) => c,
                None => {
                    eprintln!("tsr: --event must be a 64-hex candidate id");
                    return ExitCode::from(1);
                }
            };
            hook::record_forecast_for_candidate(&store, &task_id, cid, p, source)
        }
        _ => {
            eprintln!("tsr: forecast needs either --kind <kind> --args <string> or --event <id>");
            return ExitCode::from(1);
        }
    };
    match result {
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
                eprintln!("tsr: hook selftest: {e}");
                ExitCode::from(3)
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
    // The same session-resolution rule the hook used to commit the candidate: `TSR_SESSION` (set by
    // the caller) or the `.tsr/session` file the hook wrote, so the committed-candidate reuse fires.
    let task = match store_path::resolve_session_task(None) {
        Ok(id) => id,
        Err(e) => {
            eprintln!("tsr: {e}");
            return ExitCode::from(2);
        }
    };
    match exec::run(&store, &task, &workspace, argv) {
        Ok(outcome) => ExitCode::from(outcome.exit_code.clamp(0, 255) as u8),
        Err(exec::ExecRefusal::NoRuntime) => {
            eprintln!(
                "tsr: no container runtime reachable (podman/docker); refusing to run on the host"
            );
            ExitCode::from(exec::EXIT_NO_RUNTIME as u8)
        }
        // `run` records a container that fails to start as `Unresolved` and returns `Ok`, so this
        // arm is unreachable from the CLI; it exists so the match stays exhaustive.
        Err(exec::ExecRefusal::StartFailed(reason)) => {
            eprintln!("tsr: container runtime failed to start: {reason}");
            ExitCode::from(exec::EXIT_NO_RUNTIME as u8)
        }
    }
}
