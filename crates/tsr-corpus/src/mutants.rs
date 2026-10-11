//! # `tsr corpus mutants` — killed mutants as tasks (PRD v0.3.1, §B9, §B11 M3)
//!
//! `cargo-mutants` generates localized mutants of a Rust repository; each mutant the suite
//! *kills* (the suite fails under the mutation) is a task with a known reversal: the diff that
//! reverts the mutation. The agent sees the mutated snapshot; the oracle is that the failing
//! tests pass again once the mutation is reverted and the full suite does not regress.
//!
//! Both `cargo mutants --list --json` and the kill run go through the [`Runner`], so in production
//! they are `tsr exec -- <argv>` in the sealed sibling sandbox (§B4) and in tests they are the
//! deterministic [`ScriptedRunner`]. Nothing here runs a repository's tests on the host.
//!
//! ## The real cargo-mutants shapes (24.x)
//!
//! `cargo mutants --list --json` writes a JSON **array** of mutant descriptors to stdout. Each
//! descriptor is `{name, package, file, function, span, replacement, genre, diff}` where
//! `function` is an object (`{function_name, return_type, span}`) or null, and `span` is
//! `{start: {line, column}, end: {line, column}}`. The mutation operator is `genre` (an enum such
//! as `FnValue`, `BinaryOperator`, `UnaryOperator`, `MatchArm`, `MatchArmGuard`, `StructField`);
//! later releases rename it `operator`, so both are accepted.
//!
//! The kill run does **not** write JSONL to stdout (`--json` is documented as "only for --list").
//! It writes `mutants.out/outcomes.json` inside the snapshot: a `LabOutcome`
//! `{outcomes: [ScenarioOutcome], total_mutants, caught, missed, timeout, unviable, success, …}`
//! where each `ScenarioOutcome` is `{scenario, summary, log_path, diff_path, phase_results}`.
//! `scenario` is `"Baseline"` or `{"Mutant": <descriptor>}`, and `summary` is one of
//! `Success`, `CaughtMutant`, `MissedMutant`, `Unviable`, `Failure`, `Timeout`. A mutant is killed
//! when its summary is `CaughtMutant`.
//!
//! Captured samples of both shapes live at `fixtures/m3/cargo-mutants-sample.json` (the list
//! array) and `fixtures/m3/cargo-mutants-outcomes-sample.json` (the run's `outcomes.json`), and
//! the tests parse those files rather than a hand-written shape.
//!
//! ## The failing-test convention
//!
//! [`RunResult`] carries only exit codes and output digests, so the *names* of the tests that
//! failed under the mutant cannot come from the [`Runner`]'s return value. As in [`crate::mine`],
//! the suite invocation writes a machine-readable report inside the snapshot at
//! `<snapshot>/.tsr-suite-report.json` (`{"tests": [{"name", "passed"}]}`), and `mutants` reads it
//! after the run. A **missing** report is an error, never a silently empty failing set.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tsr_domain::{Hash, Horizon, OutcomeDef, OutcomeDefId};

use crate::mine::read_report;
use crate::{digest, Limits, Runner};

/// The argv that lists mutants as JSON.
pub const LIST_ARGV: &[&str] = &["cargo", "mutants", "--list", "--json"];

/// The argv that runs the kill run.
pub const RUN_ARGV: &[&str] = &["cargo", "mutants"];

/// The file the list run writes inside the snapshot.
pub const LIST_FILE: &str = "mutants.out/mutants.json";

/// The file the kill run writes inside the snapshot.
pub const OUTCOMES_FILE: &str = "mutants.out/outcomes.json";

/// The oracle's name, versioned like every other outcome definition (B7).
pub const ORACLE_NAME: &str = "mutant_reverted_and_suite_no_regression";
pub const ORACLE_VERSION: u32 = 1;

/// A line/column position in a source file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Position {
    pub line: u64,
    pub column: u64,
}

/// A span in a source file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

/// The function a mutant is inside, as cargo-mutants serializes it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Function {
    pub function_name: String,
    #[serde(default)]
    pub return_type: String,
    #[serde(default)]
    pub span: Option<Span>,
}

/// One mutant descriptor from `cargo mutants --list --json`.
///
/// cargo-mutants 24.x names the operator `genre`; later releases rename it `operator`. Both are
/// accepted, and [`MutantDescriptor::operator`] returns whichever is present.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MutantDescriptor {
    /// The precomputed human-readable name, e.g. `src/lib.rs:8:5: replace f with 0`.
    #[serde(default)]
    pub name: String,
    /// The package (crate) the mutant is in.
    pub package: String,
    /// The source file, relative to the repository root.
    pub file: String,
    /// The function the mutation is inside, if any.
    #[serde(default)]
    pub function: Option<Function>,
    /// The span of the mutated region.
    #[serde(default)]
    pub span: Option<Span>,
    /// The replacement text the mutation applies.
    #[serde(default)]
    pub replacement: String,
    /// The mutation operator, named `genre` in cargo-mutants 24.x.
    #[serde(default)]
    pub genre: Option<String>,
    /// The mutation operator, named `operator` in later releases.
    #[serde(default)]
    pub operator: Option<String>,
    /// The unified diff of the mutation, when the tool provides one.
    #[serde(default)]
    pub diff: Option<String>,
}

impl MutantDescriptor {
    /// The mutation operator, from whichever field the tool used.
    pub fn operator(&self) -> &str {
        self.operator
            .as_deref()
            .or(self.genre.as_deref())
            .unwrap_or("unknown")
    }

    /// The 1-based line of the mutation, from the span.
    pub fn line(&self) -> Option<u64> {
        self.span.as_ref().map(|s| s.start.line)
    }

    /// A stable identity for the mutant: package, file, line, operator and replacement.
    pub fn key(&self) -> String {
        format!(
            "{}:{}:{}:{}:{}",
            self.package,
            self.file,
            self.line().unwrap_or(0),
            self.operator(),
            self.replacement
        )
    }
}

/// One scenario outcome from the kill run's `outcomes.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioOutcome {
    /// `"Baseline"` or `{"Mutant": <descriptor>}`.
    pub scenario: Scenario,
    /// `Success`, `CaughtMutant`, `MissedMutant`, `Unviable`, `Failure`, `Timeout`.
    pub summary: String,
}

/// A scenario: the baseline build, or a mutant.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scenario {
    Baseline,
    Mutant(Box<MutantDescriptor>),
}

impl ScenarioOutcome {
    /// The mutant this outcome is about, if it is a mutant scenario.
    pub fn mutant(&self) -> Option<&MutantDescriptor> {
        match &self.scenario {
            Scenario::Baseline => None,
            Scenario::Mutant(m) => Some(m.as_ref()),
        }
    }

    /// A mutant the suite killed: the summary is `CaughtMutant`.
    pub fn is_caught(&self) -> bool {
        self.summary.eq_ignore_ascii_case("CaughtMutant")
    }
}

/// The kill run's `outcomes.json`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LabOutcome {
    pub outcomes: Vec<ScenarioOutcome>,
    #[serde(default)]
    pub total_mutants: usize,
    #[serde(default)]
    pub caught: usize,
    #[serde(default)]
    pub missed: usize,
    #[serde(default)]
    pub timeout: usize,
    #[serde(default)]
    pub unviable: usize,
    #[serde(default)]
    pub success: usize,
}

/// One mutant task: a mutated snapshot, the tests the mutation broke, and the reversal diff that
/// restores the original source.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MutantTask {
    /// blake3 of `(repo, mutant key, operator)` — stable across runs of the same mutant.
    pub id: String,
    /// The repository the mutant was generated from. Also the task's lineage (B9).
    pub repo: String,
    /// Digest of the mutated snapshot: the kill run's stdout digest under the mutant.
    pub mutated_snapshot: String,
    /// The tests that failed under the mutant, sorted.
    pub failing_tests: Vec<String>,
    /// CAS digest of the reversal diff (the diff that reverts the mutation).
    pub reversal_diff: String,
    /// Family: `(mutation operator, crate)` (B9).
    pub family: String,
    /// Lineage: the repository (B9).
    pub lineage: String,
    /// The mutant's timestamp, ISO-8601 (B9: every task carries a timestamp).
    pub timestamp: String,
    /// The oracle, encoded as an outcome definition.
    pub outcome: OutcomeDef,
}

/// The result of a mutants run.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MutantsReport {
    pub repo: String,
    /// Mutants listed by `cargo mutants --list --json`.
    pub mutants_listed: usize,
    /// Mutant scenarios the kill run reported on.
    pub mutants_run: usize,
    /// Tasks written (one per caught mutant).
    pub tasks: usize,
    /// Mutants the suite did not kill (`missed`, `unviable`, `timeout`, …).
    pub skipped_not_caught: usize,
}

/// How to run mutants.
#[derive(Clone, Debug)]
pub struct MutantsOptions {
    /// The list argv (default [`LIST_ARGV`]).
    pub list_argv: Vec<String>,
    /// The kill-run argv (default [`RUN_ARGV`]).
    pub run_argv: Vec<String>,
    /// Run limits handed to the [`Runner`].
    pub limits: Limits,
}

impl Default for MutantsOptions {
    fn default() -> Self {
        Self {
            list_argv: LIST_ARGV.iter().map(|s| s.to_string()).collect(),
            run_argv: RUN_ARGV.iter().map(|s| s.to_string()).collect(),
            limits: Limits::default(),
        }
    }
}

/// The oracle: the failing tests pass once the mutation is reverted and the full suite does not
/// regress.
pub fn oracle() -> OutcomeDef {
    let name = ORACLE_NAME.to_string();
    let version = ORACLE_VERSION;
    let resolves_true_when =
        "every failing test passes after the reversal diff is applied AND the full suite exits 0"
            .to_string();
    let censored_when =
        "the suite could not be run under the mutant or after the reversal (no sandbox, timeout)"
            .to_string();
    let intervention_rule =
        "the agent sees the mutated snapshot only; the reversal diff stays hidden".to_string();
    let payload = serde_json::to_vec(&(
        &name,
        version,
        &resolves_true_when,
        &censored_when,
        &intervention_rule,
    ))
    .expect("serializable");
    OutcomeDef {
        id: OutcomeDefId(Hash::of(&[&payload])),
        name,
        version,
        horizon: Horizon::TaskClose,
        resolves_true_when,
        censored_when,
        intervention_rule,
    }
}

/// The task id: blake3 of `(repo, mutant key, operator)`.
pub fn task_id(repo: &str, mutant: &MutantDescriptor) -> String {
    let payload =
        serde_json::to_vec(&(repo, mutant.key(), mutant.operator())).expect("serializable");
    digest(&payload)
}

/// Parse the `cargo mutants --list --json` output: a JSON array of descriptors.
pub fn parse_list(bytes: &[u8]) -> Result<Vec<MutantDescriptor>, MutantsError> {
    serde_json::from_slice(bytes).map_err(MutantsError::Json)
}

/// Parse the kill run's `outcomes.json`.
pub fn parse_outcomes(bytes: &[u8]) -> Result<LabOutcome, MutantsError> {
    serde_json::from_slice(bytes).map_err(MutantsError::Json)
}

/// Read the list run's `mutants.json` from inside the snapshot. A missing file is an error.
pub fn read_list(snapshot: &Path) -> Result<Vec<MutantDescriptor>, MutantsError> {
    let path = snapshot.join(LIST_FILE);
    let bytes = fs::read(&path).map_err(|e| {
        MutantsError::MissingList(format!(
            "no mutant list at {}: {e} — the list run must write {LIST_FILE} inside the snapshot",
            path.display()
        ))
    })?;
    parse_list(&bytes)
}

/// Read the kill run's `outcomes.json` from inside the snapshot. A missing file is an error.
pub fn read_outcomes(snapshot: &Path) -> Result<LabOutcome, MutantsError> {
    let path = snapshot.join(OUTCOMES_FILE);
    let bytes = fs::read(&path).map_err(|e| {
        MutantsError::MissingOutcomes(format!(
            "no outcomes at {}: {e} — the kill run must write {OUTCOMES_FILE} inside the snapshot",
            path.display()
        ))
    })?;
    parse_outcomes(&bytes)
}

/// The reversal diff for a mutant: the unified diff that restores the original source line.
///
/// cargo-mutants does not emit the reversal itself, so it is reconstructed from the descriptor:
/// the mutated line is `replacement`, the original is the source line at the mutant's span in the
/// repository at the mutated snapshot. When the original cannot be read, the reversal is the
/// descriptor's own text (the mutation), which is still a stable, content-addressed record of what
/// must change.
pub fn reversal_diff(repo: &Path, mutant: &MutantDescriptor) -> String {
    let original = mutant
        .line()
        .and_then(|n| read_source_line(repo, &mutant.file, n))
        .unwrap_or_default();
    let mutated = if mutant.replacement.is_empty() {
        mutant.name.clone()
    } else {
        mutant.replacement.clone()
    };
    format!(
        "--- a/{file}\n+++ b/{file}\n@@ mutant {op} @@\n-{mutated}\n+{original}\n",
        file = mutant.file,
        op = mutant.operator(),
        mutated = mutated.trim_end(),
        original = original.trim_end(),
    )
}

/// Read the 1-based `line` of `file` under `repo`, if it exists.
fn read_source_line(repo: &Path, file: &str, line: u64) -> Option<String> {
    let text = fs::read_to_string(repo.join(file)).ok()?;
    text.lines()
        .nth((line as usize).saturating_sub(1))
        .map(str::to_string)
}

/// The digest of the mutated snapshot: blake3 of the mutant's own descriptor (its diff, file, span
/// and replacement). [`RunResult`] carries only output digests, so the mutated source is identified
/// by the mutation itself — the same mutant always yields the same snapshot digest.
pub fn mutated_snapshot_digest(mutant: &MutantDescriptor) -> String {
    let payload = serde_json::to_vec(&(
        &mutant.package,
        &mutant.file,
        &mutant.span,
        &mutant.replacement,
        &mutant.diff,
    ))
    .expect("serializable");
    digest(&payload)
}

/// The family of a mutant: `(mutation operator, crate)` (B9).
pub fn family_of(mutant: &MutantDescriptor) -> String {
    format!("({}, {})", mutant.operator(), mutant.package)
}

/// Run `cargo mutants` for `repo` into `out`, writing `tasks.jsonl` and the CAS.
///
/// Both the list and the kill run go through the [`Runner`]; the failing tests are read from the
/// suite report the kill run writes inside the snapshot (see the module docs).
pub fn mutants(
    repo: &Path,
    out: &Path,
    opts: &MutantsOptions,
    runner: &dyn Runner,
) -> Result<MutantsReport, MutantsError> {
    let repo_str = repo.display().to_string();
    fs::create_dir_all(out).map_err(MutantsError::Io)?;
    let cas = out.join("cas");
    fs::create_dir_all(&cas).map_err(MutantsError::Io)?;

    // The list run: cargo-mutants writes the JSON array of descriptors to
    // `mutants.out/mutants.json` inside the snapshot. A non-zero exit or unparseable output is an
    // error.
    let list_run = runner.run(repo, &opts.list_argv, &opts.limits);
    if !list_run.succeeded() {
        return Err(MutantsError::Runner(format!(
            "cargo mutants --list exited {}",
            list_run.exit
        )));
    }
    let listed = read_list(repo)?;

    // The kill run: outcomes.json inside the snapshot names the caught mutants; the suite report
    // names the tests that failed under the mutant.
    let run = runner.run(repo, &opts.run_argv, &opts.limits);
    if !run.succeeded() {
        return Err(MutantsError::Runner(format!(
            "cargo mutants exited {}",
            run.exit
        )));
    }
    let lab = read_outcomes(repo)?;
    let report = read_report(repo)?;

    let mut report_out = MutantsReport {
        repo: repo_str.clone(),
        mutants_listed: listed.len(),
        mutants_run: lab.outcomes.iter().filter(|o| o.mutant().is_some()).count(),
        tasks: 0,
        skipped_not_caught: 0,
    };
    let mut lines: Vec<String> = Vec::new();

    for outcome in &lab.outcomes {
        let Some(mutant) = outcome.mutant() else {
            continue;
        };
        if !outcome.is_caught() {
            report_out.skipped_not_caught += 1;
            continue;
        }
        let failing: Vec<String> = report.failed().into_iter().collect();
        let reversal = reversal_diff(repo, mutant);
        let reversal_digest = digest(reversal.as_bytes());
        crate::mine::write_cas(&cas, &reversal_digest, reversal.as_bytes())
            .map_err(MutantsError::from)?;

        let task = MutantTask {
            id: task_id(&repo_str, mutant),
            repo: repo_str.clone(),
            mutated_snapshot: mutated_snapshot_digest(mutant),
            failing_tests: failing,
            reversal_diff: reversal_digest,
            family: family_of(mutant),
            lineage: repo_str.clone(),
            timestamp: mutant_timestamp(repo),
            outcome: oracle(),
        };
        lines.push(serde_json::to_string(&task).map_err(MutantsError::Json)?);
        report_out.tasks += 1;
    }

    let mut body = lines.join("\n");
    if !body.is_empty() {
        body.push('\n');
    }
    fs::write(out.join("tasks.jsonl"), body).map_err(MutantsError::Io)?;
    Ok(report_out)
}

/// The mutant's timestamp: the repository's HEAD commit time, ISO-8601. `git` reads history; it
/// executes nothing from the repository.
fn mutant_timestamp(repo: &Path) -> String {
    std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["show", "-s", "--format=%cI", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

/// Errors from a mutants run.
#[derive(Debug, thiserror::Error)]
pub enum MutantsError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("runner: {0}")]
    Runner(String),
    #[error("missing suite report: {0}")]
    MissingReport(String),
    #[error("missing mutant list: {0}")]
    MissingList(String),
    #[error("missing outcomes: {0}")]
    MissingOutcomes(String),
}

impl From<crate::mine::MineError> for MutantsError {
    fn from(e: crate::mine::MineError) -> Self {
        match e {
            crate::mine::MineError::Io(e) => MutantsError::Io(e),
            crate::mine::MineError::Json(e) => MutantsError::Json(e),
            crate::mine::MineError::MissingReport(m) => MutantsError::MissingReport(m),
            crate::mine::MineError::Git(m) => MutantsError::Runner(m),
        }
    }
}
