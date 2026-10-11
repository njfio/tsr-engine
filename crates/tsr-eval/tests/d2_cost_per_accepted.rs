//! D2: cost per accepted completion is ONE ratio-of-sums metric.
//!
//! The defect (specs/lanes/05r-corpus-repair.md item 4): `baseline.rs` printed
//! the ratio of sums for the point estimate but computed the paired difference,
//! its interval and the verdict from per-task "cost if accepted, else 0", and
//! `cluster_bootstrap` only averaged. So the interval and the verdict described
//! a *different* statistic (the mean of per-task costs) than the number printed
//! (the ratio of sums). The two can even differ in sign.
//!
//! These tests pin the metric down: the paired difference is the difference of
//! the two ratios, the bootstrap resamples lineages and recomputes both ratios
//! per resample, and a system with no accepted task reports the metric as
//! undefined ("n/a"), never 0.

use std::path::PathBuf;

use tsr_batch::results::{write_results, OracleVerdict, Outcome, TaskResult};
use tsr_eval::{baseline, PairBy, DEFAULT_SEED};

/// One result row. `accepted` drives both the outcome and the oracle verdict so
/// `is_accepted` agrees with it.
fn row(
    task: &str,
    system: &str,
    lineage: &str,
    accepted: bool,
    generator_tokens: u64,
    tool_minutes: u32,
) -> TaskResult {
    TaskResult {
        task: task.into(),
        system: system.into(),
        outcome: if accepted {
            Outcome::Accepted
        } else {
            Outcome::Rejected
        },
        generator_tokens,
        tool_minutes,
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

/// Write a results dir from rows and return the tempdir (kept alive by caller).
fn dir_with(rows: &[TaskResult]) -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    write_results(&dir.path().join("results.jsonl"), rows).unwrap();
    dir
}

/// The cost-per-accepted section of the rendered report.
fn cost_section(text: &str) -> String {
    let after = text
        .split("== cost per accepted completion ==")
        .nth(1)
        .expect("cost section present");
    after.split("== ").next().unwrap().to_string()
}

/// D2: a hand-computed example where the ratio-of-sums and the mean-of-ratios
/// differ. The current code prints the ratio-of-sums point estimate but
/// bootstraps the mean-of-ratios, so the printed diff and the interval describe
/// different statistics.
///
/// Two lineages, paired by task id:
///   lineage A: task "a" — record cost 1.0, rules cost 10.0
///   lineage B: tasks "b1","b2","b3" — record all accepted at cost 10.0;
///              rules: "b1" accepted at cost 1.0, "b2"/"b3" rejected.
/// Ratio of sums (the metric): record = (1 + 30)/4 = 7.75;
///   rules = (10 + 1)/2 = 5.5; diff = 5.5 - 7.75 = -2.25.
/// Mean of per-task "cost if accepted else 0" differences (what the current
/// bootstrap averages): record per-task = [1, 10, 10, 10]; rules per-task =
/// [10, 1, 0, 0]; diffs = [9, -9, -10, -10]; mean = -20/4 = -5.0.
/// The two differ, so the interval built from the mean-of-ratios is centred on
/// -5.0, not on the printed -2.25.
#[test]
fn d2_ratio_of_sums_differs_from_mean_of_ratios() {
    let rows = vec![
        // lineage A: one paired task, record cost 1.0, rules cost 10.0
        row("a", "record", "A", true, 1000, 0), // cost 1.0
        row("a", "rules", "A", true, 10000, 0), // cost 10.0
        // lineage B: record has 3 accepted tasks at cost 10.0
        row("b1", "record", "B", true, 10000, 0),
        row("b2", "record", "B", true, 10000, 0),
        row("b3", "record", "B", true, 10000, 0),
        // lineage B: rules has 1 accepted task at cost 1.0 and 2 rejected
        row("b1", "rules", "B", true, 1000, 0),
        row("b2", "rules", "B", false, 0, 0),
        row("b3", "rules", "B", false, 0, 0),
    ];
    let dir = dir_with(&rows);
    let r = baseline(dir.path(), PairBy::Task, DEFAULT_SEED, 2000).unwrap();
    let m = r.metric("cost_per_accepted").expect("cost metric present");

    // The printed values are the ratio of sums, per system.
    let record = m.record.expect("record ratio defined");
    let rules = m.rules.expect("rules ratio defined");
    assert!(
        (record - 7.75).abs() < 1e-9,
        "record ratio-of-sums should be 7.75, got {record}"
    );
    assert!(
        (rules - 5.5).abs() < 1e-9,
        "rules ratio-of-sums should be 5.5, got {rules}"
    );
    // The paired difference is the difference of the two ratios.
    let diff = m.diff.as_ref().expect("diff defined");
    assert!(
        (diff.point - (5.5 - 7.75)).abs() < 1e-9,
        "paired diff should be the difference of the ratios (-2.25), got {}",
        diff.point
    );
    // The interval must contain the point estimate and must not be the
    // degenerate mean-of-ratios value.
    assert!(
        diff.lo <= diff.point && diff.point <= diff.hi,
        "interval [{}, {}] must contain the point estimate {}",
        diff.lo,
        diff.hi,
        diff.point
    );
    assert!(
        (diff.point - (-5.0)).abs() > 1e-9,
        "the point estimate must not be the mean-of-ratios value -5.0"
    );
}

/// D2: the bootstrap resamples lineages and recomputes BOTH ratios per
/// resample. With two lineages that pull in opposite directions the interval
/// must span zero (the two systems are indistinguishable), even though the
/// point estimate is non-zero.
#[test]
fn d2_bootstrap_resamples_lineages_and_recomputes_both_ratios() {
    // Lineage A: rules much cheaper. Lineage B: rules much more expensive.
    // Ratio of sums over both lineages is zero; resampling one lineage at a
    // time gives a wide interval that spans zero.
    let rows = vec![
        row("a", "record", "A", true, 10000, 0), // 10.0
        row("a", "rules", "A", true, 1000, 0),   // 1.0
        row("b", "record", "B", true, 1000, 0),  // 1.0
        row("b", "rules", "B", true, 10000, 0),  // 10.0
    ];
    let dir = dir_with(&rows);
    let r = baseline(dir.path(), PairBy::Task, DEFAULT_SEED, 2000).unwrap();
    let m = r.metric("cost_per_accepted").unwrap();
    // record = (10+1)/2 = 5.5, rules = (1+10)/2 = 5.5, diff = 0.
    assert!(
        (m.record.unwrap() - 5.5).abs() < 1e-9,
        "record {:?}",
        m.record
    );
    assert!((m.rules.unwrap() - 5.5).abs() < 1e-9, "rules {:?}", m.rules);
    let diff = m.diff.as_ref().expect("diff defined");
    assert!((diff.point).abs() < 1e-9, "diff {}", diff.point);
    assert!(
        m.inconclusive(),
        "two opposing lineages must give an interval spanning zero: {:?}",
        m.diff
    );
}

/// D2: a system with no accepted task reports the metric as undefined ("n/a"),
/// never 0. The report must print `n/a` for that system's value.
#[test]
fn d2_no_accepted_task_reports_undefined_not_zero() {
    let rows = vec![
        row("t", "record", "A", true, 1000, 0),
        row("t", "rules", "A", false, 500, 0),
    ];
    let dir = dir_with(&rows);
    let r = baseline(dir.path(), PairBy::Task, DEFAULT_SEED, 200).unwrap();
    let text = r.render();
    let section = cost_section(&text);
    assert!(
        section.contains("rules:  n/a") || section.contains("rules: n/a"),
        "a system with no accepted task must print n/a, not 0:\n{section}"
    );
    assert!(
        !section.contains("rules:  0.000000"),
        "a system with no accepted task must not print 0:\n{section}"
    );
}

/// D2: the metric is defined once. The point estimate printed for each system
/// equals the ratio of sums computed by hand from the same rows.
#[test]
fn d2_point_estimate_is_the_ratio_of_sums() {
    let rows = vec![
        row("t1", "record", "A", true, 2000, 1),  // 3.0
        row("t1", "rules", "A", true, 1000, 0),   // 1.0
        row("t2", "record", "A", false, 9000, 9), // rejected: not counted
        row("t2", "rules", "A", true, 3000, 1),   // 4.0
    ];
    let dir = dir_with(&rows);
    let r = baseline(dir.path(), PairBy::Task, DEFAULT_SEED, 200).unwrap();
    let m = r.metric("cost_per_accepted").unwrap();
    // record: one accepted task, cost 3.0 -> 3.0
    assert!(
        (m.record.unwrap() - 3.0).abs() < 1e-9,
        "record {:?}",
        m.record
    );
    // rules: two accepted tasks, costs 1.0 and 4.0 -> 2.5
    assert!((m.rules.unwrap() - 2.5).abs() < 1e-9, "rules {:?}", m.rules);
    let diff = m.diff.as_ref().expect("diff defined");
    assert!(
        (diff.point - (2.5 - 3.0)).abs() < 1e-9,
        "diff {}",
        diff.point
    );
}

/// Keep the fixture path helper honest: the crate's own fixtures still parse.
#[test]
fn d2_fixtures_still_render() {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop();
    p.push("fixtures/m3/eval/clear");
    let r = baseline(&p, PairBy::Task, DEFAULT_SEED, 200).unwrap();
    assert!(r.metric("cost_per_accepted").is_some());
}
