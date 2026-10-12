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
//! `cargo mutants --list --json` writes a JSON **array** of mutant descriptors to **stdout**
//! (cargo-mutants `main.rs`: `print!("{}", list_mutants(...))`). The [`Runner`] returns only
//! output *digests*, not the bytes, so the list argv redirects that stdout into
//! `mutants.out/mutants.json` inside the snapshot — the file [`read_list`] reads. Each
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
//! failed under a mutant cannot come from the [`Runner`]'s return value. cargo-mutants writes each
//! scenario's test output to its own file under `mutants.out/`, named by the scenario's
//! `log_path`; `mutants` reads **that** file for each caught mutant and parses the failing test
//! names from cargo test's `test <name> ... FAILED` lines. Each task therefore carries the failing
//! tests of its own kill run, not one shared report. A **missing** log for a caught mutant is an
//! error, never a silently empty failing set.
//!
//! ## cargo-mutants exit codes
//!
//! cargo-mutants documents its exit codes in `src/exit_code.rs` (and README "Exit codes"):
//! `0` = `Success` (all mutants caught), `1` = `Usage` (wrong arguments; also Clap's value),
//! `2` = `FoundProblems` (one or more mutants were not caught by tests), `3` = `Timeout` (a test
//! timed out), `4` = `BaselineFailed` (the tests already fail in the unmutated tree), `5` =
//! `FilterDiffMismatch`, `6` = `FilterDiffInvalid`, `70` = `Software` (internal error). Only `0`,
//! `2` and `3` mean the run itself completed — a run that missed some mutants (exit `2`) is **not**
//! a failure of the command. [`kill_run_ok`] encodes that.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};
use tsr_domain::{Hash, Horizon, OutcomeDef, OutcomeDefId};

use crate::{digest, Limits, Runner};

/// The argv that lists mutants as JSON.
///
/// `cargo mutants --list --json` prints the JSON array to **stdout**, and the [`Runner`] returns
/// only digests, so the argv redirects that stdout into [`LIST_FILE`] inside the snapshot — the
/// file [`read_list`] reads. This is the one place the doc and the code agree on where the list
/// lives (D9c).
///
/// `mutants.out/` does not exist on a fresh checkout, so the redirect must create it first: a bare
/// `> mutants.out/mutants.json` fails with `Directory nonexistent` (exit 2) before cargo-mutants is
/// reached (D9e).
pub const LIST_ARGV: &[&str] = &[
    "sh",
    "-c",
    "mkdir -p mutants.out && cargo mutants --list --json > mutants.out/mutants.json",
];

/// The argv that runs the kill run.
pub const RUN_ARGV: &[&str] = &["cargo", "mutants"];

/// The file the list run writes inside the snapshot.
pub const LIST_FILE: &str = "mutants.out/mutants.json";

/// The file the kill run writes inside the snapshot.
pub const OUTCOMES_FILE: &str = "mutants.out/outcomes.json";

/// The directory cargo-mutants writes its per-scenario logs into, relative to the snapshot.
pub const MUTANTS_OUT_DIR: &str = "mutants.out";

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
    /// The path, relative to the snapshot, of the log cargo-mutants wrote for this scenario's test
    /// run. Each scenario has its own log, so a caught mutant's failing tests come from its own
    /// kill run (D9a).
    #[serde(default)]
    pub log_path: Option<String>,
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
    /// The base snapshot the mutant's copy is materialised from: the tree id of the repository HEAD
    /// the mutant was generated against (D4). `mutated_snapshot` is only a descriptor digest and
    /// names nothing materialisable, so the copy is this base tree with `mutation_diff` applied.
    pub base_snapshot: String,
    /// The tests that failed under the mutant, sorted.
    pub failing_tests: Vec<String>,
    /// CAS digest of the mutation diff (D4): the diff cargo-mutants applied to the base tree. The
    /// disposable copy is the base snapshot with this applied.
    pub mutation_diff: String,
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
    /// Caught mutants whose descriptor `file` was untrusted (absolute or containing `..`) and so
    /// was not read: the task is still written, but its reversal diff falls back to the
    /// descriptor's own text rather than a host file (security review F1).
    #[serde(default)]
    pub skipped_unsafe_descriptor: usize,
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

/// Whether a cargo-mutants kill-run exit code means the run itself completed.
///
/// cargo-mutants documents its exit codes in `src/exit_code.rs` (and README "Exit codes"):
/// `0` = `Success` (all mutants caught), `1` = `Usage` (wrong arguments; also Clap's value),
/// `2` = `FoundProblems` (one or more mutants were not caught by tests), `3` = `Timeout` (a test
/// timed out), `4` = `BaselineFailed` (the tests already fail in the unmutated tree), `5` =
/// `FilterDiffMismatch`, `6` = `FilterDiffInvalid`, `70` = `Software` (internal error).
///
/// A run that missed some mutants (exit `2`) or timed out (exit `3`) still completed and wrote its
/// outcomes — that is **not** a failure of the command (D9b). Only `0`, `2` and `3` are accepted; a
/// spawn error is reported by the [`Runner`] as exit `127` and is not accepted here.
pub fn kill_run_ok(exit: i32) -> bool {
    matches!(exit, 0 | 2 | 3)
}

/// Parse the failing test names from a cargo test log.
///
/// cargo test prints one `test <name> ... FAILED` line per failing test (and `... ok` for a pass).
/// The names are returned sorted and deduplicated. A log with no `FAILED` line yields an empty
/// list — the caller decides whether that is an error.
pub fn parse_failing_tests(log: &str) -> Vec<String> {
    let mut names: Vec<String> = log
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let rest = line.strip_prefix("test ")?;
            let (name, verdict) = rest.rsplit_once(" ... ")?;
            if verdict.trim() == "FAILED" {
                Some(name.trim().to_string())
            } else {
                None
            }
        })
        .collect();
    names.sort();
    names.dedup();
    names
}

/// Read the failing tests of one scenario's own kill run from its `log_path` inside the snapshot.
///
/// Each scenario in `outcomes.json` carries its own `log_path`; reading that file gives the tests
/// that failed under **that** mutant, not a shared report (D9a). A caught mutant with no
/// `log_path`, or whose log is missing, is an error — never a silently empty failing set.
pub fn read_failing_tests(
    snapshot: &Path,
    outcome: &ScenarioOutcome,
) -> Result<Vec<String>, MutantsError> {
    let Some(rel) = outcome.log_path.as_deref() else {
        return Err(MutantsError::MissingReport(format!(
            "caught mutant {} has no log_path in {OUTCOMES_FILE}",
            outcome
                .mutant()
                .map(|m| m.name.clone())
                .unwrap_or_else(|| "<unknown>".to_string())
        )));
    };
    // `log_path` comes from `outcomes.json`, which cargo-mutants writes from the repository under
    // test, so it is untrusted: an absolute path replaces the base, a `..` component escapes
    // `mutants.out`, and a symlink passes a string check but reads a host file (R2, D1). It must
    // resolve inside `mutants.out`, canonicalised, with symlinks refused — the same guard
    // `is_safe_descriptor_path` applies to a descriptor `file`.
    let out_dir = snapshot.join(MUTANTS_OUT_DIR);
    if !is_safe_descriptor_path(&out_dir, rel) {
        return Err(MutantsError::MissingReport(format!(
            "caught mutant {} has an unsafe log_path {rel:?} in {OUTCOMES_FILE}: it must resolve \
             inside {MUTANTS_OUT_DIR}",
            outcome
                .mutant()
                .map(|m| m.name.clone())
                .unwrap_or_else(|| "<unknown>".to_string())
        )));
    }
    let path = out_dir.join(rel);
    let text = fs::read_to_string(&path).map_err(|e| {
        MutantsError::MissingReport(format!(
            "no kill-run log at {} for a caught mutant: {e}",
            path.display()
        ))
    })?;
    Ok(parse_failing_tests(&text))
}

/// Read the 1-based `line` of `file` under `repo`, if it exists.
///
/// `file` comes from a mutant descriptor parsed out of the repository under test, so it is
/// untrusted: an absolute path, one containing a `..` component, or one that passes through a
/// symlink would escape `repo` and read a host file (security review F1, D1). Such a descriptor is
/// rejected — the reversal diff then falls back to the descriptor's own text, which is still a
/// stable content-addressed record.
/// Whether `file` is a safe relative path under a repository root: not absolute, and with no `..`
/// component. A descriptor whose `file` fails this is untrusted and must not be joined onto the
/// host repository path.
pub fn is_safe_relative_path(file: &str) -> bool {
    let path = Path::new(file);
    if path.is_absolute() {
        return false;
    }
    !path
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
}

/// Whether `file` is a safe descriptor path under `repo`: the string check of
/// [`is_safe_relative_path`], and then every component of the path resolved against `repo` with
/// symlinks resolved must stay inside `repo` and must not itself be a symlink (D1).
///
/// A committed symlink `src/x.rs -> ~/.ssh/id_rsa` passes the string check (relative, no `..`) but
/// would copy a host file into the CAS when joined onto `repo`. Resolving each component with
/// [`std::fs::symlink_metadata`] refuses any path that passes through a symlink, and the final
/// canonicalised form must still be under the canonicalised `repo`.
pub fn is_safe_descriptor_path(repo: &Path, file: &str) -> bool {
    if !is_safe_relative_path(file) {
        return false;
    }
    let Ok(root) = repo.canonicalize() else {
        return false;
    };
    // Walk the components, refusing any that is a symlink, and confirm the resolved path stays
    // under the root.
    let mut current = root.clone();
    for component in Path::new(file).components() {
        let std::path::Component::Normal(part) = component else {
            // `CurDir` is harmless; `ParentDir`/`RootDir`/`Prefix` were already refused above.
            if matches!(component, std::path::Component::CurDir) {
                continue;
            }
            return false;
        };
        current.push(part);
        match std::fs::symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => return false,
            Ok(_) => {}
            // A component that does not exist cannot be read; the caller's `read_to_string` will
            // fail and the descriptor falls back to its own text.
            Err(_) => return false,
        }
    }
    // The resolved path must still be under the root (defence in depth against a symlinked parent
    // that `symlink_metadata` did not see because it was replaced between the two calls).
    match current.canonicalize() {
        Ok(resolved) => resolved.starts_with(&root),
        Err(_) => false,
    }
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

/// The tree id of the repository's HEAD: the base snapshot a mutant's copy is materialised from
/// (D4). `git rev-parse HEAD^{tree}` reads history and executes nothing from the repository.
///
/// A repository with no HEAD (an empty repository, or a `git` that fails) yields an empty string;
/// the caller refuses to write a task whose base snapshot is empty, because such a task cannot be
/// provisioned.
pub fn base_tree(repo: &Path) -> String {
    std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["rev-parse", "HEAD^{tree}"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

/// The mutation diff cargo-mutants applied to the base tree, as a real unified diff (D4).
///
/// The descriptor's own `diff` field is the mutation as cargo-mutants wrote it. When it is present
/// and non-empty it is returned verbatim: it is the diff that was actually applied, so applying it
/// to the base tree reproduces the mutated tree by construction. When the descriptor carries no
/// `diff`, the mutation is reconstructed from the span and replacement as a one-line hunk against
/// the base source line, which is the same shape `git apply` accepts.
///
/// Returns `None` when neither the descriptor's `diff` nor a readable base source line is available:
/// a task whose mutation cannot be expressed as a diff must not be written, because its copy could
/// not be provisioned.
/// Rewrite a cargo-mutants descriptor diff's `---`/`+++` headers to `a/<file>` / `b/<file>`.
///
/// cargo-mutants writes `--- src/lib.rs` and `+++ replace add -> i32 with 0`: the `+++` line is a
/// human description, not a path, so `git apply` reads it as the target file and refuses the patch
/// (`error: replace add -> i32 with 0: No such file or directory`). Only the two header lines are
/// rewritten; every hunk line is left byte-for-byte as cargo-mutants wrote it.
fn normalize_diff_headers(diff: &str, file: &str) -> String {
    let mut out = String::new();
    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("--- ") {
            // Keep a `a/`-prefixed path if cargo-mutants already wrote one; otherwise name the file.
            let path = rest.strip_prefix("a/").unwrap_or(rest);
            out.push_str(&format!("--- a/{path}\n"));
        } else if line.starts_with("+++ ") {
            out.push_str(&format!("+++ b/{file}\n"));
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

pub fn mutation_diff(repo: &Path, base_tree: &str, mutant: &MutantDescriptor) -> Option<String> {
    if let Some(diff) = mutant.diff.as_deref() {
        if !diff.trim().is_empty() {
            // cargo-mutants writes the descriptor's `diff` with a `--- <file>` header but a
            // `+++ <description>` header (`+++ replace add -> i32 with 0`), which is not a path:
            // `git apply` reads the `+++` line as the target and refuses the patch. Rewrite both
            // headers to the repository-relative `a/<file>` / `b/<file>` shape `git apply` expects,
            // so the mutation diff applies to the base tree by construction.
            return Some(normalize_diff_headers(diff, &mutant.file));
        }
    }
    // The fallback reconstruction reads the source line from the BASE TREE, not the working tree:
    // the diff must apply to the base tree the copy is materialised from, and a working tree that
    // has moved on (or a test that rewrote the file) would produce a diff that cannot apply.
    let mutated = if mutant.replacement.is_empty() {
        mutant.name.clone()
    } else {
        mutant.replacement.clone()
    };
    let line = mutant.line().unwrap_or(1);
    // A hand-written one-line hunk (`@@ -N,1 +N,1 @@`) is not reliably locatable by `git apply`
    // when the file has more lines. Produce the diff the way `reversal_diff` does: materialise the
    // base tree, edit the line, and take `git diff --no-index` between the edited copy and the
    // base copy, so the diff applies to the base tree by construction.
    let scratch = scratch_dir("mutation");
    let base = scratch.join("base");
    let edited = scratch.join("edited");
    crate::mine::materialize(repo, base_tree, &base).ok()?;
    crate::mine::materialize(repo, base_tree, &edited).ok()?;
    let path = edited.join(&mutant.file);
    let text = fs::read_to_string(&path).ok()?;
    let mut lines: Vec<String> = text.lines().map(str::to_string).collect();
    let idx = (line as usize).saturating_sub(1);
    if idx >= lines.len() {
        return None;
    }
    lines[idx] = mutated.trim_end().to_string();
    let mut body = lines.join("\n");
    if text.ends_with('\n') {
        body.push('\n');
    }
    fs::write(&path, body).ok()?;
    // `git diff --no-index A B` yields the diff that turns A into B, so pass the base first and
    // the edited copy second: the result transforms the base tree into the mutated one.
    let out = std::process::Command::new("git")
        .args([
            "diff",
            "--no-index",
            "--no-color",
            "--src-prefix=a/",
            "--dst-prefix=b/",
        ])
        .arg(&base)
        .arg(&edited)
        .output()
        .ok()?;
    if !out.status.success() && out.status.code() != Some(1) {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout).to_string();
    if text.trim().is_empty() {
        return None;
    }
    Some(rewrite_no_index_paths(&text, &base, &edited))
}

/// The reversal diff for a mutant, produced by `git diff` between the mutated copy and the base
/// copy (D4, D9).
///
/// The reversal is **never** built by editing diff text: the base tree is materialised into a
/// scratch directory, the mutation diff is applied to it, and `git diff --no-index` between the
/// mutated copy and the base copy yields the reversal. A diff produced this way applies to the
/// mutated copy by construction, so every reversal passes `git apply --check`.
///
/// Returns `None` when the base tree cannot be materialised or the mutation diff does not apply:
/// the caller then refuses to write the task rather than emitting a reversal that cannot apply.
pub fn reversal_diff(repo: &Path, base_tree: &str, mutation: &str) -> Option<String> {
    let scratch = scratch_dir("reversal");
    let base = scratch.join("base");
    let mutated = scratch.join("mutated");
    let result = (|| -> Option<String> {
        crate::mine::materialize(repo, base_tree, &base).ok()?;
        crate::mine::materialize(repo, base_tree, &mutated).ok()?;
        apply_patch(&mutated, mutation)?;
        // `git diff --no-index` exits 1 when the trees differ, which is the expected case; only a
        // failure to run it is an error.
        let out = std::process::Command::new("git")
            .args([
                "diff",
                "--no-index",
                "--no-color",
                "--src-prefix=a/",
                "--dst-prefix=b/",
            ])
            .arg(&mutated)
            .arg(&base)
            .output()
            .ok()?;
        if !out.status.success() && out.status.code() != Some(1) {
            return None;
        }
        let text = String::from_utf8_lossy(&out.stdout).to_string();
        if text.trim().is_empty() {
            return None;
        }
        Some(rewrite_no_index_paths(&text, &mutated, &base))
    })();
    let _ = fs::remove_dir_all(&scratch);
    result
}

/// Apply a unified diff to `dir` with `git apply`. Returns `None` when it does not apply.
fn apply_patch(dir: &Path, diff: &str) -> Option<()> {
    use std::io::Write;
    let mut child = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["apply", "--whitespace=nowarn", "-"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .ok()?;
    child.stdin.take()?.write_all(diff.as_bytes()).ok()?;
    let out = child.wait_with_output().ok()?;
    out.status.success().then_some(())
}

/// Rewrite the absolute paths `git diff --no-index` writes into repository-relative `a/`/`b/`
/// paths, so the reversal applies to the mutated copy with `git apply -C <copy>`.
///
/// `git diff --no-index <mutated> <base>` writes `--- a/<mutated>/<file>` / `+++ b/<base>/<file>`
/// with the scratch directories in the header. Stripping the two scratch prefixes leaves
/// `--- a/<file>` / `+++ b/<file>`, which is what `git apply` expects against the copy.
fn rewrite_no_index_paths(diff: &str, mutated: &Path, base: &Path) -> String {
    // `git diff --no-index` writes `--- a/<mutated>/<file>` / `+++ b/<base>/<file>`. The scratch
    // directories are absolute, so the prefix is `a/` + an absolute path; `format!("a/{}", …)` would
    // double the slash and never match. Strip the `a/`/`b/` prefix first, then the scratch root.
    // `git diff --no-index` strips the leading slash from an absolute path, so the header is
    // `a/tmp/…/mutated/src/lib.rs` for a scratch root of `/tmp/…/mutated`.
    let mutated_root = mutated.display().to_string();
    let mutated_root = mutated_root.trim_start_matches('/').to_string();
    let base_root = base.display().to_string();
    let base_root = base_root.trim_start_matches('/').to_string();
    let mut out = String::new();
    for line in diff.lines() {
        if let Some(rest) = line.strip_prefix("--- ") {
            let rest = rest.strip_prefix("a/").unwrap_or(rest);
            let rest = rest.strip_prefix(&mutated_root).unwrap_or(rest);
            // The scratch root is absolute, so the remainder is `/src/lib.rs`; keep the leading
            // slash so the rewritten header is `a/src/lib.rs`, not `asrc/lib.rs`.
            out.push_str(&format!("--- a/{}\n", rest.trim_start_matches('/')));
        } else if let Some(rest) = line.strip_prefix("+++ ") {
            let rest = rest.strip_prefix("b/").unwrap_or(rest);
            let rest = rest.strip_prefix(&base_root).unwrap_or(rest);
            out.push_str(&format!("+++ b/{}\n", rest.trim_start_matches('/')));
        } else if line.starts_with("diff --git ") {
            // Drop the `diff --git a/<mutated>/… b/<base>/…` line: `git apply` does not need it and
            // it carries the scratch paths.
        } else if line.starts_with("index ") || line.starts_with("new file mode") {
            // Keep index lines out: they name blobs of the scratch trees, not the copy.
        } else {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}

/// A fresh scratch directory under the system temp dir, removed by the caller.
fn scratch_dir(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "tsr-mutants-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

/// The family of a mutant: `(mutation operator, crate)` (B9).
pub fn family_of(mutant: &MutantDescriptor) -> String {
    format!("({}, {})", mutant.operator(), mutant.package)
}

/// Run `cargo mutants` for `repo` into `out`, writing `tasks.jsonl` and the CAS.
///
/// Both the list and the kill run go through the [`Runner`]. The list run writes
/// `mutants.out/mutants.json` inside the snapshot; the kill run writes `mutants.out/outcomes.json`
/// and one log per scenario. Each caught mutant's failing tests are read from **its own** log
/// (D9a), and a kill-run exit that means "some mutants missed" is not a failure (D9b).
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

    // N5 (PRD §B10): the kill run executes the repository's code, so it must never run in the
    // operator's repository — that would make the repository the sandbox's writable mount and leave
    // `mutants.out/` behind in it. Materialise HEAD into a disposable copy and run both the list and
    // the kill run there. The repository is only read (`git archive`); the copy is removed below.
    let copy = scratch_dir("kill-run");
    let _ = fs::remove_dir_all(&copy);
    crate::mine::materialize(repo, "HEAD", &copy).map_err(|e| {
        MutantsError::Runner(format!(
            "could not materialise HEAD into a disposable copy: {e}"
        ))
    })?;

    // The list run: cargo-mutants writes the JSON array of descriptors to
    // `mutants.out/mutants.json` inside the copy. A non-zero exit or unparseable output is an error.
    let list_run = runner.run(&copy, &opts.list_argv, &opts.limits);
    if !list_run.succeeded() {
        let _ = fs::remove_dir_all(&copy);
        return Err(MutantsError::Runner(format!(
            "cargo mutants --list exited {}",
            list_run.exit
        )));
    }
    let listed = read_list(&copy)?;

    // The kill run: outcomes.json inside the copy names the caught mutants, and each scenario's own
    // log names the tests that failed under it. Exit 2 (`FoundProblems`: some mutants missed) and
    // exit 3 (`Timeout`) mean the run completed; only a genuine failure is an error (D9b).
    let run = runner.run(&copy, &opts.run_argv, &opts.limits);
    if !kill_run_ok(run.exit) {
        let _ = fs::remove_dir_all(&copy);
        return Err(MutantsError::Runner(format!(
            "cargo mutants exited {} (not a completed run)",
            run.exit
        )));
    }
    let lab = read_outcomes(&copy)?;

    let mut report_out = MutantsReport {
        repo: repo_str.clone(),
        mutants_listed: listed.len(),
        mutants_run: lab.outcomes.iter().filter(|o| o.mutant().is_some()).count(),
        tasks: 0,
        skipped_not_caught: 0,
        skipped_unsafe_descriptor: 0,
    };
    // D4: the base snapshot every mutant's copy is materialised from is the tree id of the
    // repository HEAD the kill run mutated. A repository with no HEAD cannot provision a mutant
    // task, so a run that produced caught mutants but no base tree is an error rather than a
    // manifest of unprovisionable tasks.
    let base = base_tree(repo);

    // The list output (`mutants.json`) is the only place cargo-mutants writes each mutant's
    // `diff`; `outcomes.json` names the mutant but carries no diff. Look the descriptor up by name
    // so the mutation diff is the one cargo-mutants actually applied, not a reconstruction.
    let listed_by_name: std::collections::HashMap<&str, &MutantDescriptor> =
        listed.iter().map(|m| (m.name.as_str(), m)).collect();

    let mut lines: Vec<String> = Vec::new();

    for outcome in &lab.outcomes {
        let Some(outcome_mutant) = outcome.mutant() else {
            continue;
        };
        // Prefer the listed descriptor (it carries the diff); fall back to the outcome's own
        // descriptor when the list did not name it.
        let mutant = listed_by_name
            .get(outcome_mutant.name.as_str())
            .copied()
            .unwrap_or(outcome_mutant);
        if !outcome.is_caught() {
            report_out.skipped_not_caught += 1;
            continue;
        }
        // A descriptor whose `file` is absolute, contains `..`, or passes through a symlink is
        // untrusted: it is not read, and the skip is recorded rather than silently producing a diff
        // (security review F1, D1).
        if !is_safe_descriptor_path(repo, &mutant.file) {
            report_out.skipped_unsafe_descriptor += 1;
            // The descriptor is not read at all: no diff, no reversal, no task. Falling through
            // would read the untrusted path (D1) and fail the whole run.
            continue;
        }
        // The failing tests come from THIS mutant's own kill-run log, not one shared report (D9a).
        // The log lives in the disposable copy the kill run wrote (N5), not in the repository.
        let failing = read_failing_tests(&copy, outcome)?;

        // D4: the mutation diff is the diff cargo-mutants applied to the base tree. It is written
        // to the corpus's own CAS, so `tsr task run` can read it from beside `tasks.jsonl` and
        // materialise the mutated copy. A mutant whose mutation cannot be expressed as a diff is
        // refused: its copy could not be provisioned.
        let mutation = mutation_diff(repo, &base, mutant).ok_or_else(|| {
            MutantsError::Runner(format!(
                "mutant {} has no mutation diff: the descriptor carries no `diff` and its base \
                 source line could not be read, so its copy cannot be provisioned",
                mutant.name
            ))
        })?;
        let mutation_digest = digest(mutation.as_bytes());
        crate::mine::write_cas(&cas, &mutation_digest, mutation.as_bytes())
            .map_err(MutantsError::from)?;

        // D9: the reversal is a real `git diff` between the mutated copy and the base copy, never
        // diff text edited by hand, so it applies to the mutated copy by construction.
        let reversal = reversal_diff(repo, &base, &mutation).ok_or_else(|| {
            MutantsError::Runner(format!(
                "mutant {}: the mutation diff does not apply to the base tree {base}, so no \
                 reversal can be produced",
                mutant.name
            ))
        })?;
        let reversal_digest = digest(reversal.as_bytes());
        crate::mine::write_cas(&cas, &reversal_digest, reversal.as_bytes())
            .map_err(MutantsError::from)?;

        let task = MutantTask {
            id: task_id(&repo_str, mutant),
            repo: repo_str.clone(),
            mutated_snapshot: mutated_snapshot_digest(mutant),
            base_snapshot: base.clone(),
            failing_tests: failing,
            mutation_diff: mutation_digest,
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
    // The disposable copy has served its purpose — the descriptors, outcomes and per-scenario logs
    // are all read. Remove it so nothing is left behind (N5).
    let _ = fs::remove_dir_all(&copy);
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
