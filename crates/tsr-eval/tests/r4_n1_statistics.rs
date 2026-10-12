//! R4, N1: statistics honesty — count the resamples actually used, and refuse
//! to read an interval built from a bootstrap that dropped most of its draws.
//!
//! The defect (specs/lanes/reviews/05rr-review.md item (8), new defect N1):
//! `ratio_cluster_bootstrap` (crates/tsr-eval/src/bootstrap.rs) silently drops
//! every resample in which a system has no accepted task, but the report still
//! prints `resamples: 2000` (baseline.rs) and reads the surviving interval as a
//! verdict. Two lineages where `record` accepts only in B and `rules` only in A
//! print `[-5, -5]` and `clear` — a "clear" verdict from a bootstrap that threw
//! away roughly half its draws.
//!
//! The fix: the interval carries the number of resamples used; when more than
//! 5% were dropped the interval is n/a and the verdict INCONCLUSIVE with the
//! reason; the report counts the resamples used.

use tsr_batch::results::{write_results, OracleVerdict, Outcome, TaskResult};
use tsr_eval::{baseline, PairBy, DEFAULT_SEED};

/// One result row. `accepted` drives both the outcome and the oracle verdict so
/// `is_accepted` agrees with it.
fn row(task: &str, system: &str, lineage: &str, accepted: bool, tokens: u64) -> TaskResult {
    TaskResult {
        task: task.into(),
        system: system.into(),
        outcome: if accepted {
            Outcome::Accepted
        } else {
            Outcome::Rejected
        },
        generator_tokens: tokens,
        tool_minutes: 1,
        wall_clock_ms: 1000,
        family: "f".into(),
        lineage: lineage.into(),
        state_root_at_start: "aa".into(),
        state_root_at_end: "bb".into(),
        denials: vec![],
        oracle: Some(OracleVerdict {
            flipped_tests_pass: accepted,
            suite_regressed: false,
            accepted,
            exit_code: 0,
        }),
    }
}

/// Two lineages where `record` accepts only in B and `rules` only in A. A
/// resample that draws only A has no accepted `record` task (record's ratio is
/// undefined); one that draws only B has no accepted `rules` task. With two
/// lineages, half the resamples are dropped — far more than 5%.
fn disjoint_acceptance_dir() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    // Lineage A: rules accepts, record does not.
    // Lineage B: record accepts, rules does not.
    let rows = vec![
        row("a1", "record", "A", false, 0),
        row("a1", "rules", "A", true, 1000),
        row("b1", "record", "B", true, 5000),
        row("b1", "rules", "B", false, 0),
    ];
    write_results(&dir.path().join("results.jsonl"), &rows).unwrap();
    dir
}

/// R4: the report counts the resamples actually used, not the number requested.
#[test]
fn r4_report_counts_resamples_used() {
    let dir = disjoint_acceptance_dir();
    let report = baseline(dir.path(), PairBy::Task, DEFAULT_SEED, 2000).unwrap();
    assert_eq!(report.resamples, 2000, "the request is still 2000");
    assert!(
        report.resamples_used < report.resamples,
        "R4: the report must count the resamples actually used, got {} used of {}",
        report.resamples_used,
        report.resamples
    );
    let text = report.render();
    assert!(
        text.contains("resamples: ") && text.contains("used of 2000 requested"),
        "R4: the report must print the used count:\n{text}"
    );
}

/// N1: when more than 5% of the resamples are dropped, the interval is n/a and
/// the verdict is INCONCLUSIVE with the reason — never a zero-width "clear".
#[test]
fn n1_more_than_five_percent_dropped_is_inconclusive() {
    let dir = disjoint_acceptance_dir();
    let report = baseline(dir.path(), PairBy::Task, DEFAULT_SEED, 2000).unwrap();
    let text = report.render();

    let cost = report.metric("cost_per_accepted").unwrap();
    let d = cost.diff.as_ref().expect("cost diff present");
    assert!(
        d.dropped_fraction() > 0.05,
        "fixture must drop more than 5% of resamples, dropped {} of {}",
        d.dropped(),
        d.resamples
    );
    assert!(
        !d.is_defined(),
        "N1: an interval from a bootstrap that dropped >5% must be undefined"
    );
    assert!(cost.inconclusive(), "N1: >5% dropped must be INCONCLUSIVE");

    assert!(
        text.contains("interval n/a") && text.contains("resamples dropped"),
        "N1: the report must say the interval is n/a and why:\n{text}"
    );
    assert!(
        text.contains("INCONCLUSIVE") && text.contains("more than 5%"),
        "N1: the report must be INCONCLUSIVE with the reason:\n{text}"
    );
    assert!(
        !text.contains("clear: the 90% interval excludes zero."),
        "N1: a bootstrap that dropped >5% must never be reported clear:\n{text}"
    );
}

/// The guard must not swallow the normal case: a bootstrap that drops nothing
/// still produces a defined interval and a real verdict.
#[test]
fn r4_no_drops_still_produces_a_defined_interval() {
    let dir = tempfile::tempdir().unwrap();
    let mut rows = Vec::new();
    for lineage in ["a", "b"] {
        for task in ["t1", "t2"] {
            let id = format!("{lineage}-{task}");
            rows.push(row(&id, "record", lineage, true, 8000));
            rows.push(row(&id, "rules", lineage, true, 1000));
        }
    }
    write_results(&dir.path().join("results.jsonl"), &rows).unwrap();
    let report = baseline(dir.path(), PairBy::Task, DEFAULT_SEED, 2000).unwrap();
    assert_eq!(report.resamples_used, 2000, "nothing dropped here");
    let cost = report.metric("cost_per_accepted").unwrap();
    let d = cost.diff.as_ref().unwrap();
    assert_eq!(d.dropped(), 0);
    assert!(d.is_defined(), "no drops must define the interval");
    assert!(!cost.inconclusive(), "rules is cheaper in both lineages");
}
