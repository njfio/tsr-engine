//! D5: one lineage is never "clear".
//!
//! The cluster bootstrap resamples *lineages*. With a single lineage every
//! resample draws that same lineage, so the bootstrap distribution is a single
//! point and the 90% interval has zero width. A zero-width interval that does
//! not straddle zero is then reported as "clear" — a verdict the data cannot
//! support, because one repository is not a sample of repositories.
//!
//! The report must instead say the interval is undefined ("interval n/a —
//! 1 lineage") and the result is INCONCLUSIVE.

use std::path::PathBuf;

use tsr_batch::results::{write_results, OracleVerdict, Outcome, TaskResult};
use tsr_eval::{baseline, PairBy, DEFAULT_SEED};

fn accepted(task: &str, system: &str, lineage: &str, tokens: u64) -> TaskResult {
    TaskResult {
        task: task.to_string(),
        system: system.to_string(),
        outcome: Outcome::Accepted,
        generator_tokens: tokens,
        tool_minutes: 1,
        wall_clock_ms: 1000,
        family: "f".into(),
        lineage: lineage.to_string(),
        state_root_at_start: "aa".into(),
        state_root_at_end: "bb".into(),
        denials: vec![],
        oracle: Some(OracleVerdict {
            flipped_tests_pass: true,
            suite_regressed: false,
            accepted: true,
            exit_code: 0,
        }),
    }
}

/// A results dir whose paired tasks all belong to one lineage, and where the
/// rules system is unambiguously cheaper — so the point estimate is far from
/// zero and the degenerate interval would otherwise read "clear".
fn one_lineage_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    let mut rows = Vec::new();
    for task in ["t1", "t2", "t3"] {
        rows.push(accepted(task, "record", "solo", 8000));
        rows.push(accepted(task, "rules", "solo", 1000));
    }
    write_results(&dir.path().join("results.jsonl"), &rows).unwrap();
    dir
}

#[test]
fn d5_one_lineage_prints_interval_na_and_inconclusive() {
    let dir = one_lineage_dir();
    let report = baseline(dir.path(), PairBy::Task, DEFAULT_SEED, 2000).unwrap();
    let text = report.render();

    assert_eq!(report.lineages, 1, "fixture must have exactly one lineage");

    // The interval is undefined, not a zero-width [x, x].
    assert!(
        text.contains("interval n/a — 1 lineage"),
        "a single lineage must print `interval n/a — 1 lineage`:\n{text}"
    );
    // And the verdict must be INCONCLUSIVE, never "clear".
    assert!(
        text.contains("INCONCLUSIVE"),
        "a single lineage must be INCONCLUSIVE:\n{text}"
    );
    assert!(
        !text.contains("clear: the 90% interval excludes zero."),
        "a single lineage must never be reported clear:\n{text}"
    );
}

#[test]
fn d5_one_lineage_metric_is_inconclusive() {
    let dir = one_lineage_dir();
    let report = baseline(dir.path(), PairBy::Task, DEFAULT_SEED, 2000).unwrap();
    for m in &report.metrics {
        assert!(
            m.inconclusive(),
            "metric {} with one lineage must be inconclusive, got {:?}",
            m.spec.key,
            m.diff
        );
        assert!(
            !m.diff.as_ref().is_some_and(|d| d.is_defined()),
            "metric {} with one lineage must have an undefined interval",
            m.spec.key
        );
    }
    assert!(report.any_inconclusive());
}

#[test]
fn d5_two_lineages_still_produce_a_defined_interval() {
    // The guard must not swallow the normal case: two lineages give a real
    // interval and a real verdict.
    let dir = tempfile::tempdir().unwrap();
    let mut rows = Vec::new();
    for lineage in ["a", "b"] {
        for task in ["t1", "t2"] {
            let id = format!("{lineage}-{task}");
            rows.push(accepted(&id, "record", lineage, 8000));
            rows.push(accepted(&id, "rules", lineage, 1000));
        }
    }
    write_results(&dir.path().join("results.jsonl"), &rows).unwrap();
    let report = baseline(dir.path(), PairBy::Task, DEFAULT_SEED, 2000).unwrap();
    assert_eq!(report.lineages, 2);
    let cost = report.metric("cost_per_accepted").unwrap();
    assert!(
        cost.diff.as_ref().is_some_and(|d| d.is_defined()),
        "two lineages must define the interval"
    );
    assert!(!cost.inconclusive(), "rules is cheaper in both lineages");
}

#[test]
fn d5_fixture_with_one_lineage_is_inconclusive() {
    // A fixture directory with a single lineage, mirroring the shape of
    // fixtures/m3/eval but with one repository.
    let dir = one_lineage_dir();
    let text = baseline(dir.path(), PairBy::Task, DEFAULT_SEED, 2000)
        .unwrap()
        .render();
    assert!(
        text.contains("interval n/a — 1 lineage"),
        "expected the n/a wording:\n{text}"
    );
}

#[allow(dead_code)]
fn fixture(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop();
    p.push("fixtures/m3/eval");
    p.push(name);
    p
}
