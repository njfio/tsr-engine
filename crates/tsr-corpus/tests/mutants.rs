//! `tsr corpus mutants` against the captured cargo-mutants samples (PRD §B9, §B11 M3).
//!
//! The samples under `fixtures/m3/` are the real cargo-mutants 24.x shapes: the list output is a
//! JSON array of descriptors, and the kill run writes `mutants.out/outcomes.json` inside the
//! snapshot. These tests parse those files — not a hand-written shape — and check that a caught
//! mutant yields a task with family `(operator, crate)`, that a missed or unviable mutant yields
//! none, and that a missing or garbled input is an error rather than an empty set.
//!
//! The suite is never run on the host: a [`ReportRunner`] answers the list and kill runs and
//! writes the suite report the run would have written inside the snapshot.

use std::cell::RefCell;
use std::fs;
use std::path::{Path, PathBuf};

use tsr_corpus::mine::{SuiteReport, TestVerdict};
use tsr_corpus::mutants::{
    family_of, mutants, parse_list, parse_outcomes, read_list, read_outcomes, reversal_diff,
    MutantsError, MutantsOptions, LIST_FILE, MUTANTS_OUT_DIR, OUTCOMES_FILE,
};
use tsr_corpus::{Limits, RunResult, Runner};

const LIST_SAMPLE: &str = include_str!("../../../fixtures/m3/cargo-mutants-sample.json");
const OUTCOMES_SAMPLE: &str =
    include_str!("../../../fixtures/m3/cargo-mutants-outcomes-sample.json");

/// A runner that answers the list run with the sample array and the kill run with the sample
/// outcomes, writing the outcomes file and one per-scenario log inside the snapshot.
///
/// The log for each scenario is written at that scenario's own `log_path`, in cargo test's
/// `test <name> ... FAILED` shape, carrying the `report`'s failing tests. This is the real
/// cargo-mutants layout: each scenario has its own log, so a caught mutant's failing tests come
/// from its own kill run (D9a).
struct ReportRunner {
    list: Vec<u8>,
    outcomes: Vec<u8>,
    report: SuiteReport,
    calls: RefCell<Vec<Vec<String>>>,
}

impl ReportRunner {
    fn new(list: &str, outcomes: &str, report: SuiteReport) -> Self {
        Self {
            list: list.as_bytes().to_vec(),
            outcomes: outcomes.as_bytes().to_vec(),
            report,
            calls: RefCell::new(Vec::new()),
        }
    }

    /// The cargo-test-shaped log body for the report's failing tests.
    fn log_body(&self) -> String {
        let mut body = String::new();
        for test in &self.report.tests {
            let verdict = if test.passed { "ok" } else { "FAILED" };
            body.push_str(&format!("test {} ... {}\n", test.name, verdict));
        }
        body
    }
}

impl Runner for ReportRunner {
    fn run(&self, snapshot: &Path, argv: &[String], _limits: &Limits) -> RunResult {
        self.calls.borrow_mut().push(argv.to_vec());
        if argv.iter().any(|a| a.contains("--list")) {
            // The list run writes mutants.out/mutants.json inside the snapshot.
            let path = snapshot.join(LIST_FILE);
            fs::create_dir_all(path.parent().unwrap()).expect("mkdir mutants.out");
            fs::write(&path, &self.list).expect("write list");
            return RunResult::from_output(0, &self.list, b"", 1);
        }
        // The kill run: write the outcomes file and one log per scenario at its own log_path.
        let path = snapshot.join(OUTCOMES_FILE);
        fs::create_dir_all(path.parent().unwrap()).expect("mkdir mutants.out");
        fs::write(&path, &self.outcomes).expect("write outcomes");
        let lab = parse_outcomes(&self.outcomes).expect("parse outcomes");
        let body = self.log_body();
        for outcome in &lab.outcomes {
            if let Some(rel) = outcome.log_path.as_deref() {
                let log = snapshot.join(MUTANTS_OUT_DIR).join(rel);
                fs::create_dir_all(log.parent().unwrap()).expect("mkdir log dir");
                fs::write(&log, &body).expect("write log");
            }
        }
        RunResult::from_output(0, &self.outcomes, b"", 1)
    }
}

fn report(passed: &[&str], failed: &[&str]) -> SuiteReport {
    let mut tests: Vec<TestVerdict> = Vec::new();
    for name in passed {
        tests.push(TestVerdict {
            name: (*name).to_string(),
            passed: true,
        });
    }
    for name in failed {
        tests.push(TestVerdict {
            name: (*name).to_string(),
            passed: false,
        });
    }
    SuiteReport { tests }
}

fn tmpdir(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "tsr-mutants-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir_all(&dir).expect("tmpdir");
    dir
}

/// The list sample parses into ten descriptors with the real field names.
#[test]
fn list_sample_parses() {
    let listed = parse_list(LIST_SAMPLE.as_bytes()).expect("parse list");
    assert_eq!(listed.len(), 10, "ten mutants in the sample");
    let first = &listed[0];
    assert_eq!(first.package, "partial");
    assert_eq!(first.file, "src/lib.rs");
    assert_eq!(first.operator(), "FnValue");
    assert_eq!(first.line(), Some(2));
    assert_eq!(
        first.function.as_ref().map(|f| f.function_name.as_str()),
        Some("add")
    );
    assert_eq!(listed[3].operator(), "BinaryOperator");
    assert_eq!(listed[3].line(), Some(2));
}

/// The outcomes sample parses into a lab outcome with one baseline and ten mutant scenarios.
#[test]
fn outcomes_sample_parses() {
    let lab = parse_outcomes(OUTCOMES_SAMPLE.as_bytes()).expect("parse outcomes");
    assert_eq!(lab.outcomes.len(), 11);
    assert_eq!(lab.total_mutants, 10);
    assert_eq!(lab.caught, 5);
    assert_eq!(lab.missed, 5);
    let mutants: Vec<_> = lab.outcomes.iter().filter_map(|o| o.mutant()).collect();
    assert_eq!(mutants.len(), 10, "baseline is not a mutant");
    assert_eq!(lab.outcomes.iter().filter(|o| o.is_caught()).count(), 5);
}

/// A caught mutant yields a task with family `(operator, crate)`; a missed mutant yields none.
#[test]
fn caught_mutants_become_tasks() {
    let repo = tmpdir("caught");
    let out = repo.join("out");
    let runner = ReportRunner::new(
        LIST_SAMPLE,
        OUTCOMES_SAMPLE,
        report(&["tests::add_works"], &["tests::add_works"]),
    );
    let opts = MutantsOptions::default();
    let report_out = mutants(&repo, &out, &opts, &runner).expect("mutants");

    assert_eq!(report_out.mutants_listed, 10);
    assert_eq!(report_out.mutants_run, 10);
    assert_eq!(report_out.tasks, 5, "five caught mutants");
    assert_eq!(report_out.skipped_not_caught, 5, "five missed mutants");

    let body = fs::read_to_string(out.join("tasks.jsonl")).expect("tasks.jsonl");
    let tasks: Vec<serde_json::Value> = body
        .lines()
        .map(|l| serde_json::from_str(l).expect("task json"))
        .collect();
    assert_eq!(tasks.len(), 5);

    // Family is (operator, crate) for every caught mutant.
    for task in &tasks {
        let family = task["family"].as_str().unwrap();
        assert!(
            family == "(FnValue, partial)" || family == "(BinaryOperator, partial)",
            "unexpected family {family}"
        );
        assert_eq!(
            task["lineage"].as_str().unwrap(),
            repo.display().to_string()
        );
        assert_eq!(
            task["failing_tests"][0].as_str().unwrap(),
            "tests::add_works"
        );
        // The reversal diff is a CAS digest, and the object is present under out/cas.
        let digest = task["reversal_diff"].as_str().unwrap();
        assert!(out.join("cas").join(digest).exists(), "reversal in CAS");
        assert!(!task["mutated_snapshot"].as_str().unwrap().is_empty());
        assert!(!task["id"].as_str().unwrap().is_empty());
    }
    // The caught mutants have distinct ids.
    let ids: std::collections::BTreeSet<&str> =
        tasks.iter().map(|t| t["id"].as_str().unwrap()).collect();
    assert_eq!(ids.len(), 5);

    // Both runs went through the runner, in order: list then kill.
    let calls = runner.calls.borrow();
    assert_eq!(calls.len(), 2);
    assert!(calls[0].iter().any(|a| a.contains("--list")));
    assert!(!calls[1].iter().any(|a| a.contains("--list")));
}

/// A missed-only run yields no tasks and an empty tasks.jsonl.
#[test]
fn missed_mutants_yield_no_tasks() {
    let repo = tmpdir("missed");
    let out = repo.join("out");
    // Keep only the missed scenarios from the sample.
    let lab: serde_json::Value = serde_json::from_str(OUTCOMES_SAMPLE).unwrap();
    let kept: Vec<serde_json::Value> = lab["outcomes"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|o| o["summary"].as_str().unwrap() == "MissedMutant")
        .cloned()
        .collect();
    let mut trimmed = lab.clone();
    trimmed["outcomes"] = serde_json::Value::Array(kept);
    let outcomes = serde_json::to_string(&trimmed).unwrap();

    let runner = ReportRunner::new(LIST_SAMPLE, &outcomes, report(&["t"], &[]));
    let report_out = mutants(&repo, &out, &MutantsOptions::default(), &runner).expect("mutants");
    assert_eq!(report_out.tasks, 0);
    assert_eq!(report_out.skipped_not_caught, 5);
    assert_eq!(fs::read_to_string(out.join("tasks.jsonl")).unwrap(), "");
}

/// A missing outcomes file is an error, not an empty set.
#[test]
fn missing_outcomes_is_an_error() {
    let repo = tmpdir("missing-outcomes");
    let err = read_outcomes(&repo).expect_err("missing outcomes must error");
    assert!(matches!(err, MutantsError::MissingOutcomes(_)), "{err:?}");
}

/// A missing list file is an error, not an empty set.
#[test]
fn missing_list_is_an_error() {
    let repo = tmpdir("missing-list");
    let err = read_list(&repo).expect_err("missing list must error");
    assert!(matches!(err, MutantsError::MissingList(_)), "{err:?}");
}

/// A garbled list output is an error, not an empty set.
#[test]
fn garbled_list_is_an_error() {
    let err = parse_list(b"{ not json").expect_err("garbled list must error");
    assert!(matches!(err, MutantsError::Json(_)), "{err:?}");
}

/// A garbled outcomes file is an error, not an empty set.
#[test]
fn garbled_outcomes_is_an_error() {
    let err = parse_outcomes(b"[]").expect_err("array is not a lab outcome");
    assert!(matches!(err, MutantsError::Json(_)), "{err:?}");
}

/// A non-zero list exit is an error.
#[test]
fn list_failure_is_an_error() {
    struct FailRunner;
    impl Runner for FailRunner {
        fn run(&self, _s: &Path, _a: &[String], _l: &Limits) -> RunResult {
            RunResult::exit_only(1)
        }
    }
    let repo = tmpdir("list-fail");
    let err = mutants(
        &repo,
        &repo.join("out"),
        &MutantsOptions::default(),
        &FailRunner,
    )
    .expect_err("non-zero list exit must error");
    assert!(matches!(err, MutantsError::Runner(_)), "{err:?}");
}

/// The family helper names the operator and the crate.
#[test]
fn family_names_operator_and_crate() {
    let listed = parse_list(LIST_SAMPLE.as_bytes()).unwrap();
    assert_eq!(family_of(&listed[3]), "(BinaryOperator, partial)");
    assert_eq!(family_of(&listed[0]), "(FnValue, partial)");
}

/// Dump the fixture sample's `tasks.jsonl` for the evidence record. Run with `--nocapture`.
#[test]
fn dump_fixture_tasks_jsonl() {
    let repo = tmpdir("dump");
    let out = repo.join("out");
    let runner = ReportRunner::new(
        LIST_SAMPLE,
        OUTCOMES_SAMPLE,
        report(&["tests::add_works"], &["tests::add_works"]),
    );
    mutants(&repo, &out, &MutantsOptions::default(), &runner).expect("mutants");
    let body = fs::read_to_string(out.join("tasks.jsonl")).expect("tasks.jsonl");
    println!("--- tasks.jsonl ---\n{body}--- end ---");
}

// ---------------------------------------------------------------------------------------------
// Regression: a hostile descriptor `file` must not read the host (security review F1)
// ---------------------------------------------------------------------------------------------

/// Build an `outcomes.json` with one caught mutant whose descriptor `file` is `file`.
fn outcomes_with_file(file: &str) -> String {
    serde_json::json!({
        "outcomes": [
            {
                "scenario": {
                    "Mutant": {
                        "name": "x",
                        "package": "p",
                        "file": file,
                        "span": {"start": {"line": 1, "column": 1}, "end": {"line": 1, "column": 2}},
                        "replacement": "0",
                        "genre": "FnValue"
                    }
                },
                "summary": "CaughtMutant",
                "log_path": "log/x.log"
            }
        ],
        "total_mutants": 1,
        "caught": 1
    })
    .to_string()
}

/// A descriptor whose `file` is absolute must not read the host file into the CAS.
///
/// Pre-fix, `read_source_line` joined `/etc/hostname` onto the repo and embedded the host file's
/// first line in the reversal diff written to `out/cas`. The fix rejects the descriptor.
#[test]
fn absolute_descriptor_file_does_not_leak_host_bytes() {
    let host = Path::new("/etc/hostname");
    let Ok(host_text) = fs::read_to_string(host) else {
        // No /etc/hostname on this host: the test cannot observe a leak, but the guard still holds.
        eprintln!("skipping: {host:?} unreadable");
        return;
    };
    let host_line = host_text.lines().next().unwrap_or("").to_string();
    assert!(!host_line.is_empty(), "host file has a first line");

    let repo = tmpdir("abs-file");
    let out = repo.join("out");
    let runner = ReportRunner::new(
        LIST_SAMPLE,
        &outcomes_with_file("/etc/hostname"),
        report(&["t"], &["t"]),
    );
    let report_out = mutants(&repo, &out, &MutantsOptions::default(), &runner).expect("mutants");
    assert_eq!(report_out.tasks, 1, "the caught mutant still yields a task");
    assert_eq!(
        report_out.skipped_unsafe_descriptor, 1,
        "the untrusted descriptor is flagged as skipped"
    );

    // Every CAS object must be free of the host file's content.
    let cas = out.join("cas");
    for entry in fs::read_dir(&cas).expect("cas dir") {
        let entry = entry.expect("cas entry");
        let bytes = fs::read(entry.path()).expect("cas object");
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            !text.contains(&host_line),
            "host file content leaked into CAS object {}: {text}",
            entry.path().display()
        );
    }
}

/// A descriptor whose `file` escapes the repo with `..` must not read a sibling file into the CAS.
#[test]
fn parent_dir_descriptor_file_does_not_leak_sibling_bytes() {
    let repo = tmpdir("dotdot-file");
    // A secret one level above the repo, exactly as the reviewer reproduced.
    let secret = repo.parent().expect("parent").join(format!(
        "SECRET-{}-{}.txt",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::write(&secret, "TOP_SECRET_LINE\n").expect("write secret");
    let secret_name = secret.file_name().unwrap().to_string_lossy().to_string();
    let escape = format!("../{secret_name}");

    let out = repo.join("out");
    let runner = ReportRunner::new(
        LIST_SAMPLE,
        &outcomes_with_file(&escape),
        report(&["t"], &["t"]),
    );
    let report_out = mutants(&repo, &out, &MutantsOptions::default(), &runner).expect("mutants");
    assert_eq!(report_out.tasks, 1);
    assert_eq!(
        report_out.skipped_unsafe_descriptor, 1,
        "the `..` descriptor is flagged as skipped"
    );

    let cas = out.join("cas");
    for entry in fs::read_dir(&cas).expect("cas dir") {
        let entry = entry.expect("cas entry");
        let bytes = fs::read(entry.path()).expect("cas object");
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            !text.contains("TOP_SECRET_LINE"),
            "sibling secret leaked into CAS object {}: {text}",
            entry.path().display()
        );
    }
    let _ = fs::remove_file(&secret);
}

/// D1: a descriptor whose `file` is a committed symlink pointing outside the repo must not read the
/// host file into the CAS.
///
/// Pre-fix, `is_safe_relative_path` accepted `src/x.rs` (relative, no `..`) and `read_source_line`
/// followed the symlink, embedding the host file's first line in the reversal diff written to
/// `out/cas`. The fix resolves every descriptor path against the repository root with symlinks
/// resolved and refuses any path whose resolved form leaves the root or passes through a symlink.
#[test]
fn d1_symlinked_descriptor_file_does_not_leak_host_bytes() {
    let repo = tmpdir("symlink-file");
    // A secret outside the repo, and a committed symlink inside it pointing at the secret.
    let secret = repo.parent().expect("parent").join(format!(
        "D1-SECRET-{}-{}.txt",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::write(&secret, "D1_TOP_SECRET_LINE\n").expect("write secret");
    fs::create_dir_all(repo.join("src")).expect("mkdir src");
    let link = repo.join("src/x.rs");
    std::os::unix::fs::symlink(&secret, &link).expect("symlink");

    let out = repo.join("out");
    let runner = ReportRunner::new(
        LIST_SAMPLE,
        &outcomes_with_file("src/x.rs"),
        report(&["t"], &["t"]),
    );
    let report_out = mutants(&repo, &out, &MutantsOptions::default(), &runner).expect("mutants");
    assert_eq!(report_out.tasks, 1, "the caught mutant still yields a task");
    assert_eq!(
        report_out.skipped_unsafe_descriptor, 1,
        "the symlinked descriptor is flagged as skipped"
    );

    let cas = out.join("cas");
    for entry in fs::read_dir(&cas).expect("cas dir") {
        let entry = entry.expect("cas entry");
        let bytes = fs::read(entry.path()).expect("cas object");
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            !text.contains("D1_TOP_SECRET_LINE"),
            "host file content leaked through the symlink into CAS object {}: {text}",
            entry.path().display()
        );
    }
    let _ = fs::remove_file(&secret);
}

/// A legitimate relative descriptor `file` still reads the source line (the fix must not weaken it).
#[test]
fn relative_descriptor_file_still_reads_the_source_line() {
    let repo = tmpdir("rel-file");
    fs::create_dir_all(repo.join("src")).expect("mkdir src");
    fs::write(repo.join("src/lib.rs"), "fn f() -> u32 { 0 }\n").expect("write source");
    let out = repo.join("out");
    let runner = ReportRunner::new(
        LIST_SAMPLE,
        &outcomes_with_file("src/lib.rs"),
        report(&["t"], &["t"]),
    );
    let report_out = mutants(&repo, &out, &MutantsOptions::default(), &runner).expect("mutants");
    assert_eq!(
        report_out.skipped_unsafe_descriptor, 0,
        "a legitimate relative path is not flagged"
    );

    let cas = out.join("cas");
    let mut found = false;
    for entry in fs::read_dir(&cas).expect("cas dir") {
        let entry = entry.expect("cas entry");
        let text =
            String::from_utf8_lossy(&fs::read(entry.path()).expect("cas object")).into_owned();
        if text.contains("fn f() -> u32 { 0 }") {
            found = true;
        }
    }
    assert!(
        found,
        "the legitimate source line is still in the reversal diff"
    );
}

/// A real cargo-mutants run needs `tsr exec` and a container runtime; the operator runs it.
#[test]
#[ignore = "needs tsr exec and a container runtime; operator runs it"]
fn real_cargo_mutants_run() {
    // The operator points this at a real repository and a real ExecRunner.
    let repo = std::env::var("TSR_MUTANTS_REPO").expect("TSR_MUTANTS_REPO");
    let out = std::env::var("TSR_MUTANTS_OUT").expect("TSR_MUTANTS_OUT");
    let runner = tsr_corpus::ExecRunner::new();
    let report_out = mutants(
        Path::new(&repo),
        Path::new(&out),
        &MutantsOptions::default(),
        &runner,
    )
    .expect("mutants");
    assert!(report_out.tasks > 0, "expected at least one caught mutant");
}

// ---------------------------------------------------------------------------------------------
// D9: per-mutant failing tests, exit codes, mutant list source, real unified diff
// ---------------------------------------------------------------------------------------------

/// A runner that answers the list run with `list`, the kill run with `outcomes`, and writes a
/// per-mutant log for each scenario at its own `log_path` inside the snapshot.
///
/// `logs` maps a scenario's `log_path` to the cargo-test output that scenario's kill run wrote.
/// This is the real cargo-mutants shape: each scenario's test output lives in its own file under
/// `mutants.out/`, named by the scenario's `log_path`.
struct PerMutantLogRunner {
    list: Vec<u8>,
    outcomes: Vec<u8>,
    logs: Vec<(String, String)>,
    kill_exit: i32,
}

impl Runner for PerMutantLogRunner {
    fn run(&self, snapshot: &Path, argv: &[String], _limits: &Limits) -> RunResult {
        if argv.iter().any(|a| a.contains("--list")) {
            let path = snapshot.join(LIST_FILE);
            fs::create_dir_all(path.parent().unwrap()).expect("mkdir mutants.out");
            fs::write(&path, &self.list).expect("write list");
            return RunResult::from_output(0, &self.list, b"", 1);
        }
        let path = snapshot.join(OUTCOMES_FILE);
        fs::create_dir_all(path.parent().unwrap()).expect("mkdir mutants.out");
        fs::write(&path, &self.outcomes).expect("write outcomes");
        for (rel, body) in &self.logs {
            let log = snapshot.join("mutants.out").join(rel);
            fs::create_dir_all(log.parent().unwrap()).expect("mkdir log dir");
            fs::write(&log, body).expect("write log");
        }
        RunResult::from_output(self.kill_exit, &self.outcomes, b"", 1)
    }
}

/// Build an `outcomes.json` with two caught mutants, each with its own `log_path`.
fn two_caught_with_logs() -> String {
    serde_json::json!({
        "outcomes": [
            {
                "scenario": {"Mutant": {
                    "name": "src/lib.rs:2:5: replace add -> i32 with 0",
                    "package": "partial", "file": "src/lib.rs",
                    "span": {"start": {"line": 2, "column": 5}, "end": {"line": 2, "column": 6}},
                    "replacement": "0", "genre": "FnValue"
                }},
                "summary": "CaughtMutant",
                "log_path": "log/mutant_a.log"
            },
            {
                "scenario": {"Mutant": {
                    "name": "src/lib.rs:2:7: replace + with - in add",
                    "package": "partial", "file": "src/lib.rs",
                    "span": {"start": {"line": 2, "column": 7}, "end": {"line": 2, "column": 8}},
                    "replacement": "-", "genre": "BinaryOperator"
                }},
                "summary": "CaughtMutant",
                "log_path": "log/mutant_b.log"
            }
        ],
        "total_mutants": 2, "caught": 2
    })
    .to_string()
}

/// D9(a): each mutant task carries the failing tests of ITS OWN kill run, not one shared report.
///
/// Pre-fix, `mutants` read a single `<snapshot>/.tsr-suite-report.json` for every mutant, so two
/// mutants killed by different tests both carried the same `failing_tests`. The fix reads each
/// scenario's own `log_path` log inside `mutants.out/` and parses the failing test names from it.
#[test]
fn d9_each_mutant_carries_its_own_failing_tests() {
    let repo = tmpdir("d9-per-mutant");
    let out = repo.join("out");
    let runner = PerMutantLogRunner {
        list: LIST_SAMPLE.as_bytes().to_vec(),
        outcomes: two_caught_with_logs().into_bytes(),
        logs: vec![
            (
                "log/mutant_a.log".to_string(),
                "running 2 tests\n\
                 test tests::add_works ... FAILED\n\
                 test tests::sub_works ... ok\n\
                 \n\
                 failures:\n    tests::add_works\n"
                    .to_string(),
            ),
            (
                "log/mutant_b.log".to_string(),
                "running 2 tests\n\
                 test tests::add_works ... ok\n\
                 test tests::sub_works ... FAILED\n\
                 \n\
                 failures:\n    tests::sub_works\n"
                    .to_string(),
            ),
        ],
        kill_exit: 0,
    };
    let report_out = mutants(&repo, &out, &MutantsOptions::default(), &runner).expect("mutants");
    assert_eq!(report_out.tasks, 2, "two caught mutants");

    let body = fs::read_to_string(out.join("tasks.jsonl")).expect("tasks.jsonl");
    let tasks: Vec<serde_json::Value> = body
        .lines()
        .map(|l| serde_json::from_str(l).expect("task json"))
        .collect();
    assert_eq!(tasks.len(), 2);

    let mut failing: Vec<Vec<String>> = tasks
        .iter()
        .map(|t| {
            t["failing_tests"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_str().unwrap().to_string())
                .collect()
        })
        .collect();
    failing.sort();
    assert_eq!(
        failing,
        vec![
            vec!["tests::add_works".to_string()],
            vec!["tests::sub_works".to_string()],
        ],
        "each mutant carries the failing tests of its own kill run, not one shared report"
    );
}

/// D9(b): a cargo-mutants exit code that means "some mutants missed" is NOT a failure.
///
/// cargo-mutants documents its exit codes in `src/exit_code.rs`: 0 = `Success` (all mutants
/// caught), 1 = `Usage`, 2 = `FoundProblems` (one or more mutants were not caught by tests),
/// 3 = `Timeout`, 4 = `BaselineFailed`, 5 = `FilterDiffMismatch`, 6 = `FilterDiffInvalid`,
/// 70 = `Software`. Only 0, 2 and 3 mean the run itself completed. Pre-fix, `mutants` treated any
/// non-zero exit as a failure, so a run that missed some mutants (exit 2) errored instead of
/// yielding the caught mutants' tasks.
#[test]
fn d9_found_problems_exit_is_not_a_failure() {
    let repo = tmpdir("d9-exit-2");
    let out = repo.join("out");
    let runner = PerMutantLogRunner {
        list: LIST_SAMPLE.as_bytes().to_vec(),
        outcomes: two_caught_with_logs().into_bytes(),
        logs: vec![
            (
                "log/mutant_a.log".to_string(),
                "test tests::add_works ... FAILED\n".to_string(),
            ),
            (
                "log/mutant_b.log".to_string(),
                "test tests::sub_works ... FAILED\n".to_string(),
            ),
        ],
        // 2 = FoundProblems: some mutants were missed. The run still completed.
        kill_exit: 2,
    };
    let report_out = mutants(&repo, &out, &MutantsOptions::default(), &runner)
        .expect("exit 2 (FoundProblems) must not be a failure");
    assert_eq!(report_out.tasks, 2, "the caught mutants still yield tasks");
}

/// D9(b): a genuine cargo-mutants failure exit (5 = Failure) IS an error.
#[test]
fn d9_failure_exit_is_an_error() {
    let repo = tmpdir("d9-exit-5");
    let out = repo.join("out");
    let runner = PerMutantLogRunner {
        list: LIST_SAMPLE.as_bytes().to_vec(),
        outcomes: two_caught_with_logs().into_bytes(),
        logs: vec![],
        // 5 = Failure: the run itself failed.
        kill_exit: 5,
    };
    let err = mutants(&repo, &out, &MutantsOptions::default(), &runner)
        .expect_err("exit 5 (Failure) must be an error");
    assert!(matches!(err, MutantsError::Runner(_)), "{err:?}");
}

/// D9(c): the mutant list is read from where the invocation actually writes it.
///
/// `cargo mutants --list --json` writes the JSON array to **stdout** (cargo-mutants `main.rs`
/// `print!("{}", list_mutants(...))`), and the [`Runner`] returns only digests, not stdout bytes.
/// So the list argv must redirect that stdout into `mutants.out/mutants.json` inside the snapshot,
/// which is the file [`read_list`] reads. Pre-fix, `LIST_ARGV` was `cargo mutants --list --json`
/// with no redirect, so the file `read_list` expects was never written: the doc and the code
/// disagreed.
#[test]
fn d9_list_argv_writes_the_file_read_list_reads() {
    let argv = MutantsOptions::default().list_argv;
    let joined = argv.join(" ");
    assert!(
        joined.contains("--list") && joined.contains("--json"),
        "the list run still asks cargo-mutants for JSON: {joined}"
    );
    assert!(
        joined.contains(LIST_FILE),
        "the list argv must write {LIST_FILE} inside the snapshot, the file read_list reads: {joined}"
    );
    assert!(
        joined.contains('>'),
        "the list argv must redirect cargo-mutants' stdout into {LIST_FILE}: {joined}"
    );
}

// ---------------------------------------------------------------------------------------------
// R2, D1 residual: a hostile `log_path` in outcomes.json must not read the host
// ---------------------------------------------------------------------------------------------

/// Build an `outcomes.json` with one caught mutant whose scenario `log_path` is `log_path`.
///
/// The descriptor `file` is a safe relative path, so the only untrusted input is the `log_path`.
fn outcomes_with_log_path(log_path: &str) -> String {
    serde_json::json!({
        "outcomes": [
            {
                "scenario": {
                    "Mutant": {
                        "name": "x",
                        "package": "p",
                        "file": "src/lib.rs",
                        "span": {"start": {"line": 1, "column": 1}, "end": {"line": 1, "column": 2}},
                        "replacement": "0",
                        "genre": "FnValue"
                    }
                },
                "summary": "CaughtMutant",
                "log_path": log_path
            }
        ],
        "total_mutants": 1,
        "caught": 1
    })
    .to_string()
}

/// R2: an absolute `log_path` in `outcomes.json` must not read a host file into `tasks.jsonl`.
///
/// Pre-fix, `read_failing_tests` joined `log_path` onto `<snapshot>/mutants.out` with no check, so
/// `"log_path": "/etc/hostname"` (an absolute path replaces the base) put the host file's lines
/// into `tasks.jsonl` as `failing_tests`. The fix refuses any `log_path` that does not resolve
/// inside `mutants.out`.
#[test]
fn r2_absolute_log_path_does_not_leak_host_bytes() {
    let host = Path::new("/etc/hostname");
    let Ok(host_text) = fs::read_to_string(host) else {
        eprintln!("skipping: {host:?} unreadable");
        return;
    };
    let host_line = host_text.lines().next().unwrap_or("").to_string();
    assert!(!host_line.is_empty(), "host file has a first line");

    let repo = tmpdir("r2-abs-log");
    let out = repo.join("out");
    let runner = PerMutantLogRunner {
        list: LIST_SAMPLE.as_bytes().to_vec(),
        outcomes: outcomes_with_log_path("/etc/hostname").into_bytes(),
        logs: vec![],
        kill_exit: 0,
    };
    let err = mutants(&repo, &out, &MutantsOptions::default(), &runner)
        .expect_err("an absolute log_path must be refused, not read from the host");
    assert!(matches!(err, MutantsError::MissingReport(_)), "{err:?}");

    // Nothing was written: the host file's line is not in tasks.jsonl.
    let body = fs::read_to_string(out.join("tasks.jsonl")).unwrap_or_default();
    assert!(
        !body.contains(&host_line),
        "the host file's first line leaked into tasks.jsonl: {body}"
    );
}

/// R2: a `..` `log_path` in `outcomes.json` must not read a sibling file into `tasks.jsonl`.
///
/// Pre-fix, `"log_path": "../secret.log"` escaped `mutants.out` and read a file beside it. The fix
/// refuses any `log_path` with a `..` component.
#[test]
fn r2_parent_dir_log_path_does_not_leak_sibling_bytes() {
    let repo = tmpdir("r2-parent-log");
    // A file beside mutants.out, inside the snapshot: `../secret.log` would reach it.
    fs::write(repo.join("secret.log"), "test leaked::secret ... FAILED\n").expect("write secret");
    let out = repo.join("out");
    let runner = PerMutantLogRunner {
        list: LIST_SAMPLE.as_bytes().to_vec(),
        outcomes: outcomes_with_log_path("../secret.log").into_bytes(),
        logs: vec![],
        kill_exit: 0,
    };
    let err = mutants(&repo, &out, &MutantsOptions::default(), &runner)
        .expect_err("a `..` log_path must be refused, not read from a sibling");
    assert!(matches!(err, MutantsError::MissingReport(_)), "{err:?}");

    let body = fs::read_to_string(out.join("tasks.jsonl")).unwrap_or_default();
    assert!(
        !body.contains("leaked::secret"),
        "a sibling file's failing test leaked into tasks.jsonl: {body}"
    );
}

/// R2: a `log_path` that passes through a symlink out of `mutants.out` must be refused.
///
/// A committed symlink `mutants.out/log/x.log -> /etc/hostname` passes a string check (relative, no
/// `..`) but would read a host file. The fix resolves each component and refuses a symlink, as
/// `is_safe_descriptor_path` does for a descriptor `file`.
#[test]
fn r2_symlinked_log_path_does_not_leak_host_bytes() {
    let host = Path::new("/etc/hostname");
    let Ok(host_text) = fs::read_to_string(host) else {
        eprintln!("skipping: {host:?} unreadable");
        return;
    };
    let host_line = host_text.lines().next().unwrap_or("").to_string();
    assert!(!host_line.is_empty(), "host file has a first line");

    let repo = tmpdir("r2-symlink-log");
    let log_dir = repo.join(MUTANTS_OUT_DIR).join("log");
    fs::create_dir_all(&log_dir).expect("mkdir log dir");
    std::os::unix::fs::symlink(host, log_dir.join("x.log")).expect("symlink log");

    let out = repo.join("out");
    let runner = PerMutantLogRunner {
        list: LIST_SAMPLE.as_bytes().to_vec(),
        outcomes: outcomes_with_log_path("log/x.log").into_bytes(),
        logs: vec![],
        kill_exit: 0,
    };
    let err = mutants(&repo, &out, &MutantsOptions::default(), &runner)
        .expect_err("a symlinked log_path must be refused, not read from the host");
    assert!(matches!(err, MutantsError::MissingReport(_)), "{err:?}");

    let body = fs::read_to_string(out.join("tasks.jsonl")).unwrap_or_default();
    assert!(
        !body.contains(&host_line),
        "the host file's first line leaked into tasks.jsonl: {body}"
    );
}

/// R2: a safe relative `log_path` inside `mutants.out` still reads its own log.
///
/// The guard must not break the real cargo-mutants layout: `log/x.log` under `mutants.out` is read
/// and its failing test names land in `tasks.jsonl`.
#[test]
fn r2_relative_log_path_still_reads_its_own_log() {
    let repo = tmpdir("r2-rel-log");
    let out = repo.join("out");
    let runner = PerMutantLogRunner {
        list: LIST_SAMPLE.as_bytes().to_vec(),
        outcomes: outcomes_with_log_path("log/x.log").into_bytes(),
        logs: vec![(
            "log/x.log".to_string(),
            "test tests::flipped_add ... FAILED\n".to_string(),
        )],
        kill_exit: 0,
    };
    let report_out = mutants(&repo, &out, &MutantsOptions::default(), &runner).expect("mutants");
    assert_eq!(report_out.tasks, 1, "the caught mutant yields a task");

    let body = fs::read_to_string(out.join("tasks.jsonl")).expect("tasks.jsonl");
    let task: serde_json::Value = serde_json::from_str(body.lines().next().unwrap()).unwrap();
    assert_eq!(
        task["failing_tests"],
        serde_json::json!(["tests::flipped_add"]),
        "a safe relative log_path still yields its own failing tests"
    );
}

/// D9(d): the stored reversal is a real unified diff, not a placeholder header.
///
/// Pre-fix, `reversal_diff` emitted `@@ mutant <op> @@`, which is not a unified-diff hunk header
/// and cannot be applied by `git apply`/`patch`. The fix emits a real `@@ -<line>,1 +<line>,1 @@`
/// hunk header with the mutated line removed and the original line added.
#[test]
fn d9_reversal_is_a_real_unified_diff() {
    let repo = tmpdir("d9-diff");
    fs::create_dir_all(repo.join("src")).expect("mkdir src");
    fs::write(
        repo.join("src/lib.rs"),
        "fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    )
    .expect("write source");
    let out = repo.join("out");
    let runner = PerMutantLogRunner {
        list: LIST_SAMPLE.as_bytes().to_vec(),
        outcomes: two_caught_with_logs().into_bytes(),
        logs: vec![
            (
                "log/mutant_a.log".to_string(),
                "test tests::add_works ... FAILED\n".to_string(),
            ),
            (
                "log/mutant_b.log".to_string(),
                "test tests::sub_works ... FAILED\n".to_string(),
            ),
        ],
        kill_exit: 0,
    };
    mutants(&repo, &out, &MutantsOptions::default(), &runner).expect("mutants");

    let cas = out.join("cas");
    let mut saw_hunk = false;
    for entry in fs::read_dir(&cas).expect("cas dir") {
        let entry = entry.expect("cas entry");
        let text =
            String::from_utf8_lossy(&fs::read(entry.path()).expect("cas object")).into_owned();
        if !text.contains("--- a/") {
            continue;
        }
        assert!(
            text.contains("+++ b/"),
            "a unified diff has a +++ header: {text}"
        );
        // A real hunk header is `@@ -<start>,<len> +<start>,<len> @@`.
        let hunk = text
            .lines()
            .find(|l| l.starts_with("@@"))
            .unwrap_or_else(|| panic!("no hunk header in reversal: {text}"));
        assert!(
            hunk.starts_with("@@ -") && hunk.contains(" +") && hunk.ends_with(" @@"),
            "hunk header is not a real unified-diff header: {hunk}"
        );
        assert!(
            !hunk.contains("mutant"),
            "the placeholder `@@ mutant <op> @@` header is not a unified diff: {hunk}"
        );
        saw_hunk = true;
    }
    assert!(
        saw_hunk,
        "at least one reversal diff was written to the CAS"
    );
}

// ---------------------------------------------------------------------------------------------
// D9: the list argv actually runs, and the reversal actually applies
// ---------------------------------------------------------------------------------------------

/// D9(e): the list argv runs on a fresh checkout.
///
/// Pre-fix, `LIST_ARGV` was `sh -c "cargo mutants --list --json > mutants.out/mutants.json"`.
/// `mutants.out/` does not exist on a fresh checkout, so the shell's redirect fails before
/// cargo-mutants is even reached: `sh: 1: cannot create mutants.out/mutants.json: Directory
/// nonexistent`, exit 2. The fix creates the directory first (`mkdir -p mutants.out && …`), so the
/// redirect succeeds and the file `read_list` reads is written.
///
/// This test **runs the argv** in a temp dir (it does not grep the string): it puts a fake `cargo`
/// on `PATH` that prints the sample array, runs the real `LIST_ARGV`, and asserts the exit is 0 and
/// `mutants.out/mutants.json` holds the sample. Pre-fix the exit is 2 and the file is absent.
#[test]
fn d9_list_argv_runs_on_a_fresh_checkout() {
    let dir = tmpdir("d9-list-argv");
    let bin = dir.join("bin");
    fs::create_dir_all(&bin).expect("mkdir bin");
    let fake_cargo = bin.join("cargo");
    fs::write(&fake_cargo, "#!/bin/sh\ncat \"$TSR_FAKE_LIST\"\n").expect("write fake cargo");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&fake_cargo).expect("meta").permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&fake_cargo, perms).expect("chmod");
    }
    let list_file = dir.join("sample.json");
    fs::write(&list_file, LIST_SAMPLE).expect("write sample");

    let argv = MutantsOptions::default().list_argv;
    let out = std::process::Command::new(&argv[0])
        .args(&argv[1..])
        .current_dir(&dir)
        .env(
            "PATH",
            format!(
                "{}:{}",
                bin.display(),
                std::env::var("PATH").unwrap_or_default()
            ),
        )
        .env("TSR_FAKE_LIST", &list_file)
        .output()
        .expect("run list argv");
    assert!(
        out.status.success(),
        "the list argv must run on a fresh checkout (no mutants.out/ yet); exit {:?}, stderr: {}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr)
    );
    let written = dir.join(LIST_FILE);
    assert!(
        written.is_file(),
        "the list argv must write {LIST_FILE} inside the snapshot; it did not"
    );
    let bytes = fs::read(&written).expect("read written list");
    assert_eq!(
        parse_list(&bytes).expect("parse written list").len(),
        10,
        "the file the list argv wrote is the sample array read_list reads"
    );
}

/// D9(f): the reversal is built from the descriptor's own `diff` and applies to the mutated file.
///
/// Pre-fix, `reversal_diff` reconstructed the reversal from the descriptor's `replacement` and the
/// source line at the span, which for a `BinaryOperator` mutant (`replacement = "-"`) produced
/// `-`/`-`/`+    a + b` — a diff `git apply --check` refuses against the mutated file. The fix
/// reverses the descriptor's own `diff` field (the mutation cargo-mutants actually applied), so the
/// reversal applies cleanly.
///
/// This test materialises the fixture's mutated `src/lib.rs` in a real git repo, writes the
/// reversal `reversal_diff` returns, and runs `git apply --check` on it.
#[test]
fn d9_reversal_applies_to_the_mutated_file() {
    let listed = parse_list(LIST_SAMPLE.as_bytes()).expect("parse list");
    // The `replace + with - in add` mutant: its `replacement` is `-`, which the old reconstruction
    // turned into a diff that cannot apply.
    let mutant = listed
        .iter()
        .find(|m| m.name.contains("replace + with - in add"))
        .expect("the BinaryOperator mutant is in the sample");
    assert_eq!(mutant.replacement, "-");

    let repo = tmpdir("d9-reversal-apply");
    fs::create_dir_all(repo.join("src")).expect("mkdir src");
    // The mutated file, exactly as the descriptor's own diff leaves it.
    fs::write(
        repo.join("src/lib.rs"),
        "pub fn add(a: i32, b: i32) -> i32 {\n    a - /* ~ changed by cargo-mutants ~ */ b\n}\n\npub fn is_positive(n: i32) -> bool {\n    n > 0\n}\n\n#[cfg(test)]\nmod tests {\n",
    )
    .expect("write mutated source");
    git(&repo, &["init", "-q"]);
    git(&repo, &["add", "-A"]);
    git(
        &repo,
        &[
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-qm",
            "mutated",
        ],
    );

    let reversal = reversal_diff(&repo, mutant);
    let patch = repo.join("reversal.patch");
    fs::write(&patch, &reversal).expect("write patch");
    let check = std::process::Command::new("git")
        .arg("-C")
        .arg(&repo)
        .args(["apply", "--check"])
        .arg(&patch)
        .output()
        .expect("git apply --check");
    assert!(
        check.status.success(),
        "the reversal must apply to the mutated file; git apply --check said: {}\npatch:\n{reversal}",
        String::from_utf8_lossy(&check.stderr)
    );
}

/// Run `git` in `repo`, panicking on failure.
fn git(repo: &Path, args: &[&str]) {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(args)
        .output()
        .expect("run git");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}
