//! `tsr task run` — the batch runner over a corpus task manifest (PRD §B9, §B11 M3).
//!
//! This is the CLI face of `tsr-batch`. It reads a `tasks.jsonl`, runs every task under one system
//! (`record` or `rules`) through one driver (`scripted` or `claude-code`), and writes
//! `results-<mode>.jsonl` plus the per-task stores under `--out`. One file per mode means the
//! operator's sequence — record then rules into the same `--out` — keeps both systems' rows, and
//! `tsr eval baseline <out>` pairs them (D8).
//!
//! The `scripted` driver replays a fixture session per task, found next to the manifest:
//! `<manifest-dir>/sessions/<task-id>-<mode>.json`, then `<manifest-dir>/session-<mode>.json`, then
//! the well-formed default `<manifest-dir>/session-inspect-test-propose.json`. The `claude-code`
//! driver is the documented stub: it returns exit 2 with its reason (no Claude Code, no container
//! runtime in this sandbox).

use std::path::{Path, PathBuf};

use tsr_batch::driver::{read_session, Mode, ScriptedDriver, ScriptedRepo, ScriptedSession};
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
    profile: Option<&Path>,
) -> std::process::ExitCode {
    match run_inner(mode, tasks, driver, out, scripted_runner, profile) {
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
    profile_arg: Option<&Path>,
) -> Result<std::process::ExitCode, String> {
    let mode = match mode {
        "record" => Mode::Record,
        "rules" => Mode::Rules,
        other => return Err(format!("unknown --mode `{other}` (want record|rules)")),
    };

    // D10: the profile comes from `--profile`, else from the store's repository root (the
    // directory holding the store's `.tsr/`, as the M0 commands resolve the store), else the
    // command exits 2 naming where it looked. There is no compile-time source path.
    let profile_path = resolve_profile(profile_arg)?;
    let profile = Profile::load(&profile_path)
        .map_err(|e| format!("loading {}: {e}", profile_path.display()))?;

    let tasks =
        read_tasks(tasks_path).map_err(|e| format!("reading {}: {e}", tasks_path.display()))?;
    if tasks.is_empty() {
        return Err(format!("{} holds no tasks", tasks_path.display()));
    }

    std::fs::create_dir_all(out).map_err(|e| format!("creating {}: {e}", out.display()))?;
    let manifest_dir = tasks_path.parent().unwrap_or_else(|| Path::new("."));
    // D4: the corpus directory is where `tasks.jsonl` lives. A mutant's mutation diff is a CAS key
    // in that directory's own `cas/`, so provisioning reads it from beside the manifest rather than
    // from the batch's `--out`.
    let mut config = BatchConfig::new(out, profile);
    config.corpus_dir = manifest_dir.to_path_buf();

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
                    // D6: in the shipped rules mode the Tier 0 controller is attached, so the
                    // read-before-test rule and the tool-minute budget are enforced on the session
                    // the CLI replays — not only in tests. In record mode the controller is
                    // ignored (system 1 gates nothing), so it is not attached.
                    let driver = if mode == Mode::Rules {
                        let repo = scripted_repo_for(task, &config.profile);
                        ScriptedDriver::new(mode, session)
                            .with_tier0(repo, config.profile.budgets.tool_minutes)
                    } else {
                        ScriptedDriver::new(mode, session)
                    };
                    Box::new(driver)
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

    // D8: one file per mode, so record then rules into the same --out do not
    // overwrite each other. `tsr eval baseline <out>` reads both.
    let results_path = out.join(format!("results-{}.jsonl", mode.system()));
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

/// Resolve the protection profile (D10).
///
/// `--profile` names it directly. Otherwise the profile is looked for at
/// `<repository-root>/profiles/default.toml`, where `<repository-root>` is the nearest ancestor of
/// the working directory that holds `profiles/default.toml` — the same walk the M0 commands use to
/// find the store's session file. If no ancestor holds one the command exits 2 and says where it
/// looked. There is no compile-time source path: a binary built on one machine must not silently
/// load that machine's profile on another.
fn resolve_profile(profile_arg: Option<&Path>) -> Result<PathBuf, String> {
    if let Some(p) = profile_arg {
        return Ok(p.to_path_buf());
    }
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    for dir in cwd.ancestors() {
        let candidate = dir.join("profiles/default.toml");
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(format!(
        "no profile: pass --profile <path>, or place one at {} (looked upward from {})",
        cwd.join("profiles/default.toml").display(),
        cwd.display()
    ))
}

/// The scripted repository the Tier 0 controller checks, derived from the **task**, not the
/// session (N4).
///
/// The controller's read-before-test rule compares the paths a session inspects against the
/// failing assertion and its call site. Deriving those from the session's own `inspect` steps made
/// the rule vacuous: any session that inspected any two files before its first test passed, because
/// the session chose what it had to have read. The ground truth is the task: the files to have been
/// read are the files that define the task's failing tests, and the failing test id is the task's
/// own failing test.
///
/// A Rust test path `a::b::c` is defined in the file of module `a::b`; for a crate whose module
/// `a` is a top-level module that is `src/a.rs` (unit tests) or `tests/a.rs` (integration tests).
/// The first such file is the failing assertion's file and the second its call site. A task whose
/// failing tests name no module (no `::`) yields no file, and the rule is then not enforced — the
/// task's oracle test files (item 2) are what close that gap.
fn scripted_repo_for(task: &tsr_batch::CorpusTask, profile: &Profile) -> ScriptedRepo {
    let files = oracle_test_files(task);
    let assertion = files.first().cloned().unwrap_or_default();
    let call_site = files.get(1).cloned().unwrap_or_else(|| assertion.clone());
    let failing_test_id = failing_test_command(task, profile);
    ScriptedRepo::new(assertion, call_site, failing_test_id)
}

/// The profile test command id that runs the task's failing test (N4).
///
/// The session names an allowlisted command id, never the task's test path, so the controller's
/// `failing_test_id` is the profile command whose argv covers the failing test's crate — the
/// command the session would run to exercise it. A task whose failing test names no crate, or a
/// profile with no command that names it, falls back to the profile's first command.
fn failing_test_command(task: &tsr_batch::CorpusTask, profile: &Profile) -> String {
    let crate_name = task
        .failing_tests()
        .into_iter()
        .find_map(|test| test.split_once("::").map(|(module, _)| module.to_string()));
    if let Some(crate_name) = crate_name {
        let head = crate_name.split("::").next().unwrap_or(&crate_name);
        if let Some(cmd) = profile
            .tests
            .iter()
            .find(|t| t.argv.iter().any(|a| a == head))
        {
            return cmd.id.clone();
        }
    }
    profile
        .tests
        .first()
        .map(|t| t.id.clone())
        .unwrap_or_default()
}

/// The repository paths of the files that define a task's failing tests, derived from the test
/// paths themselves (N4).
///
/// A test path `a::b::c` names a test `c` in module `a::b`; the file that defines it is the file
/// of module `a` — `src/a.rs` for a unit-test module, `tests/a.rs` for an integration test. The
/// paths are returned sorted and deduplicated so the assertion and call site are stable.
fn oracle_test_files(task: &tsr_batch::CorpusTask) -> Vec<String> {
    let mut files: Vec<String> = Vec::new();
    for test in task.failing_tests() {
        let Some((module, _)) = test.split_once("::") else {
            continue;
        };
        let head = module.split("::").next().unwrap_or(module);
        if head.is_empty() {
            continue;
        }
        files.push(format!("src/{head}.rs"));
        files.push(format!("tests/{head}.rs"));
    }
    files.sort();
    files.dedup();
    files
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
