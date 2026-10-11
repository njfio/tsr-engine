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

use tsr_corpus::mine::{SuiteReport, TestVerdict, REPORT_FILE};
use tsr_corpus::mutants::{
    family_of, mutants, parse_list, parse_outcomes, read_list, read_outcomes, MutantsError,
    MutantsOptions, LIST_FILE, OUTCOMES_FILE,
};
use tsr_corpus::{Limits, RunResult, Runner};

const LIST_SAMPLE: &str = include_str!("../../../fixtures/m3/cargo-mutants-sample.json");
const OUTCOMES_SAMPLE: &str =
    include_str!("../../../fixtures/m3/cargo-mutants-outcomes-sample.json");

/// A runner that answers the list run with the sample array and the kill run with the sample
/// outcomes, writing both the outcomes file and the suite report inside the snapshot.
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
}

impl Runner for ReportRunner {
    fn run(&self, snapshot: &Path, argv: &[String], _limits: &Limits) -> RunResult {
        self.calls.borrow_mut().push(argv.to_vec());
        if argv.iter().any(|a| a == "--list") {
            // The list run writes mutants.out/mutants.json inside the snapshot.
            let path = snapshot.join(LIST_FILE);
            fs::create_dir_all(path.parent().unwrap()).expect("mkdir mutants.out");
            fs::write(&path, &self.list).expect("write list");
            return RunResult::from_output(0, &self.list, b"", 1);
        }
        // The kill run: write the outcomes file and the suite report inside the snapshot.
        let path = snapshot.join(OUTCOMES_FILE);
        fs::create_dir_all(path.parent().unwrap()).expect("mkdir mutants.out");
        fs::write(&path, &self.outcomes).expect("write outcomes");
        let body = serde_json::to_vec(&self.report).expect("serializable");
        fs::write(snapshot.join(REPORT_FILE), &body).expect("write report");
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
    assert!(calls[0].iter().any(|a| a == "--list"));
    assert!(!calls[1].iter().any(|a| a == "--list"));
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
