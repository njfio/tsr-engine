//! `tsr task run` — the batch runner over a corpus task manifest (PRD §B9, §B11 M3).
//!
//! This is the CLI face of `tsr-batch`. It reads a `tasks.jsonl`, runs every task under one system
//! (`record` or `rules`) through one driver (`scripted` or `claude-code`), and writes
//! `results.jsonl` plus the per-task stores under `--out`.
//!
//! The `scripted` driver replays a fixture session per task, found next to the manifest:
//! `<manifest-dir>/sessions/<task-id>-<mode>.json`, then `<manifest-dir>/session-<mode>.json`, then
//! the well-formed default `<manifest-dir>/session-inspect-test-propose.json`. The `claude-code`
//! driver is the documented stub: it returns exit 2 with its reason (no Claude Code, no container
//! runtime in this sandbox).

use std::path::Path;

use tsr_batch::driver::{read_session, Mode, ScriptedDriver, ScriptedSession};
use tsr_batch::run::{run_all, BatchConfig};
use tsr_batch::task::read_tasks;
use tsr_batch::ClaudeCodeDriver;
use tsr_corpus::{ExecRunner, Runner, ScriptedRunner};
use tsr_policy::profile::Profile;

/// Run the batch. Returns the process exit code.
pub fn run_cmd(
    mode: &str,
    tasks: &Path,
    driver: &str,
    out: &Path,
    scripted_runner: bool,
) -> std::process::ExitCode {
    match run_inner(mode, tasks, driver, out, scripted_runner) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("tsr: task run: {e}");
            std::process::ExitCode::from(2)
        }
    }
}

fn run_inner(
    mode: &str,
    tasks_path: &Path,
    driver: &str,
    out: &Path,
    scripted_runner: bool,
) -> Result<std::process::ExitCode, String> {
    let mode = match mode {
        "record" => Mode::Record,
        "rules" => Mode::Rules,
        other => return Err(format!("unknown --mode `{other}` (want record|rules)")),
    };

    // D10: the profile is resolved from the repository root, not the working
    // directory, so `tsr task run` behaves the same wherever it is invoked.
    let profile_path = repo_root().join("profiles/default.toml");
    let profile = Profile::load(&profile_path)
        .map_err(|e| format!("loading {}: {e}", profile_path.display()))?;

    let tasks =
        read_tasks(tasks_path).map_err(|e| format!("reading {}: {e}", tasks_path.display()))?;
    if tasks.is_empty() {
        return Err(format!("{} holds no tasks", tasks_path.display()));
    }

    std::fs::create_dir_all(out).map_err(|e| format!("creating {}: {e}", out.display()))?;
    let config = BatchConfig::new(out, profile);

    let manifest_dir = tasks_path.parent().unwrap_or_else(|| Path::new("."));

    // The runner is chosen by the explicit test flag, not by the driver: by default the oracle
    // runs through `ExecRunner` (`tsr exec -- <argv>` in the sealed sibling sandbox, PRD §B4), so
    // the verdict reflects the session's actual work. `ScriptedRunner` is test-only (D3).
    let runner: Box<dyn Runner> = if scripted_runner {
        Box::new(ScriptedRunner::ok())
    } else {
        Box::new(ExecRunner::new())
    };

    let results = match driver {
        "scripted" => {
            // A scripted driver replays a fixture session per task; the oracle still runs through
            // the selected runner (production `ExecRunner` unless the test flag is set).
            let sessions = load_sessions(manifest_dir, &tasks, mode)?;
            run_all(
                &tasks,
                mode,
                |task| {
                    let session = sessions
                        .get(&task.id)
                        .cloned()
                        .unwrap_or_else(|| sessions.values().next().cloned().expect("a session"));
                    Box::new(ScriptedDriver::new(mode, session))
                },
                runner.as_ref(),
                &config,
            )
            .map_err(|e| format!("running the batch: {e}"))?
        }
        "claude-code" => {
            // The stub driver returns `DriverError::Unavailable`; `run_one` records each task as
            // `unavailable` and closes it, and the batch still writes a results file.
            let driver = ClaudeCodeDriver::new("tsr", "claude-sonnet-4-5");
            let mut out_results = Vec::with_capacity(tasks.len());
            for task in &tasks {
                let r = tsr_batch::run::run_one(task, mode, &driver, runner.as_ref(), &config)
                    .map_err(|e| format!("running the batch: {e}"))?;
                out_results.push(r);
            }
            out_results
        }
        other => {
            return Err(format!(
                "unknown --driver `{other}` (want scripted|claude-code)"
            ))
        }
    };

    let results_path = out.join("results.jsonl");
    tsr_batch::write_results(&results_path, &results)
        .map_err(|e| format!("writing {}: {e}", results_path.display()))?;

    let accepted = results
        .iter()
        .filter(|r| r.outcome == tsr_batch::Outcome::Accepted)
        .count();
    let unavailable = results
        .iter()
        .filter(|r| r.outcome == tsr_batch::Outcome::Unavailable)
        .count();
    println!(
        "tsr task run: {} tasks, system {}, driver {} -> {} ({} accepted, {} unavailable)",
        results.len(),
        mode.system(),
        driver,
        results_path.display(),
        accepted,
        unavailable
    );
    Ok(std::process::ExitCode::SUCCESS)
}

/// The repository root: the nearest ancestor of the working directory that
/// holds `profiles/default.toml`, falling back to the crate's own manifest
/// directory (D10). A `tsr task run` invoked from a subdirectory therefore
/// still finds the operator's standing profile.
fn repo_root() -> std::path::PathBuf {
    if let Ok(cwd) = std::env::current_dir() {
        for dir in cwd.ancestors() {
            if dir.join("profiles/default.toml").is_file() {
                return dir.to_path_buf();
            }
        }
    }
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .to_path_buf()
}

/// Load one session per task, in the order the manifest names them.
fn load_sessions(
    manifest_dir: &Path,
    tasks: &[tsr_batch::CorpusTask],
    mode: Mode,
) -> Result<std::collections::BTreeMap<String, ScriptedSession>, String> {
    let mut sessions = std::collections::BTreeMap::new();
    for task in tasks {
        let candidates = [
            manifest_dir
                .join("sessions")
                .join(format!("{}-{}.json", task.id, mode.system())),
            manifest_dir.join(format!("session-{}.json", mode.system())),
            manifest_dir.join("session-inspect-test-propose.json"),
        ];
        let path = candidates.iter().find(|p| p.exists()).ok_or_else(|| {
            format!(
                "no scripted session for task {} under {}",
                task.id,
                manifest_dir.display()
            )
        })?;
        let session = read_session(path).map_err(|e| format!("reading {}: {e}", path.display()))?;
        sessions.insert(task.id.clone(), session);
    }
    Ok(sessions)
}
