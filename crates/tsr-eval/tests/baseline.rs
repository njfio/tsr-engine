//! `tsr eval baseline` over the M3 fixtures (PRD B9 Statistics, Appendix A).
//!
//! Two fixture results directories live under `fixtures/m3/eval/`:
//! - `inconclusive/` — the two systems trade places across lineages, so the
//!   paired difference interval spans zero and the report says INCONCLUSIVE.
//! - `clear/` — the rules system is cheaper, completes more and blocks more
//!   unsafe attempts in every lineage, so every interval excludes zero.
//!
//! The report is byte-reproducible from a fixed seed: two runs with the same
//! seed produce identical bytes.

use std::path::PathBuf;

use tsr_eval::{baseline, PairBy, DEFAULT_SEED};

fn fixture(name: &str) -> PathBuf {
    // CARGO_MANIFEST_DIR is crates/tsr-eval; the fixtures live at the repo root.
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop(); // crates/
    p.pop(); // repo root
    p.push("fixtures/m3/eval");
    p.push(name);
    p
}

fn report(name: &str) -> String {
    let dir = fixture(name);
    let r = baseline(
        &dir,
        PairBy::Task,
        DEFAULT_SEED,
        tsr_eval::DEFAULT_RESAMPLES,
    )
    .unwrap_or_else(|e| panic!("baseline over {}: {e}", dir.display()));
    r.render()
}

#[test]
fn inconclusive_fixture_says_inconclusive() {
    let text = report("inconclusive");
    assert!(
        text.contains("INCONCLUSIVE"),
        "the inconclusive fixture must print INCONCLUSIVE:\n{text}"
    );
    // The primary endpoint (PRD B3) must be the one that is inconclusive.
    let cost = text
        .split("== cost per accepted completion ==")
        .nth(1)
        .expect("cost section present");
    let cost_section = cost.split("== ").next().unwrap();
    assert!(
        cost_section.contains("INCONCLUSIVE"),
        "cost per accepted completion must be INCONCLUSIVE:\n{cost_section}"
    );
}

#[test]
fn clear_fixture_is_not_inconclusive() {
    let text = report("clear");
    assert!(
        !text.contains("INCONCLUSIVE"),
        "the clear fixture must not print INCONCLUSIVE:\n{text}"
    );
    assert_eq!(
        text.matches("clear: the 90% interval excludes zero.")
            .count(),
        4,
        "every metric must be clear:\n{text}"
    );
}

#[test]
fn fixed_seed_reproduces_the_report_byte_for_byte() {
    for name in ["inconclusive", "clear"] {
        let a = report(name);
        let b = report(name);
        assert_eq!(a, b, "report for {name} is not byte-reproducible");
    }
}

#[test]
fn a_different_seed_still_reports_the_same_verdict() {
    // The seed changes the bootstrap draws, not the fixture's verdict.
    let dir = fixture("clear");
    let a = baseline(&dir, PairBy::Task, 1, tsr_eval::DEFAULT_RESAMPLES)
        .unwrap()
        .render();
    let b = baseline(&dir, PairBy::Task, 2, tsr_eval::DEFAULT_RESAMPLES)
        .unwrap()
        .render();
    assert!(!a.contains("INCONCLUSIVE"));
    assert!(!b.contains("INCONCLUSIVE"));
}

#[test]
fn every_metric_carries_a_proxy_label_in_the_report() {
    let text = report("clear");
    for key in [
        "completion_rate",
        "cost_per_accepted",
        "unsafe_attempts_blocked",
        "generator_calls_per_task",
    ] {
        assert!(
            text.contains(&format!("{key}: proxy:")),
            "metric {key} has no proxy label in the report:\n{text}"
        );
    }
}

/// D10: every number the report prints is a labeled proxy — `name=value`, the
/// engineering name — never a bare float (PRD B2 rule 2, Appendix A rule 2).
#[test]
fn d10_every_printed_number_is_a_labeled_proxy() {
    let text = report("clear");
    // The per-system values carry the metric's engineering name.
    for key in [
        "completion_rate",
        "cost_per_accepted",
        "unsafe_attempts_blocked",
        "generator_calls_per_task",
    ] {
        assert!(
            text.contains(&format!("  record: {key}=")),
            "the record value for {key} must print as `{key}=<value>`:\n{text}"
        );
        assert!(
            text.contains(&format!("  rules:  {key}=")),
            "the rules value for {key} must print as `{key}=<value>`:\n{text}"
        );
        assert!(
            text.contains(&format!("  paired diff (rules - record): {key}=")),
            "the paired diff for {key} must print as `{key}=<value>`:\n{text}"
        );
    }
    // No line may print a bare six-decimal float as a metric value.
    for line in text.lines() {
        let t = line.trim_start();
        for prefix in ["record:", "rules:", "paired diff (rules - record):"] {
            if let Some(rest) = t.strip_prefix(prefix) {
                let rest = rest.trim_start();
                assert!(
                    rest.contains('='),
                    "`{prefix}` must print a labeled proxy, not a bare float: {line:?}"
                );
            }
        }
    }
}

#[test]
fn pairing_drops_tasks_present_in_only_one_system() {
    // A results dir with one unpaired task: the report names it and pairs the rest.
    let dir = tempfile::tempdir().unwrap();
    let mut rows = Vec::new();
    for system in ["record", "rules"] {
        rows.push(tsr_batch::results::TaskResult {
            task: "paired".into(),
            system: system.into(),
            outcome: tsr_batch::results::Outcome::Accepted,
            generator_tokens: 1000,
            tool_minutes: 1,
            wall_clock_ms: 1000,
            family: "f".into(),
            lineage: "repo-a".into(),
            state_root_at_start: "aa".into(),
            state_root_at_end: "bb".into(),
            denials: vec![],
            oracle: Some(tsr_batch::results::OracleVerdict {
                flipped_tests_pass: true,
                suite_regressed: false,
                accepted: true,
                exit_code: 0,
            }),
        });
    }
    rows.push(tsr_batch::results::TaskResult {
        task: "only-record".into(),
        system: "record".into(),
        outcome: tsr_batch::results::Outcome::Rejected,
        generator_tokens: 0,
        tool_minutes: 0,
        wall_clock_ms: 0,
        family: "f".into(),
        lineage: "repo-a".into(),
        state_root_at_start: "aa".into(),
        state_root_at_end: "bb".into(),
        denials: vec![],
        oracle: None,
    });
    tsr_batch::results::write_results(&dir.path().join("results.jsonl"), &rows).unwrap();

    let r = baseline(dir.path(), PairBy::Task, DEFAULT_SEED, 200).unwrap();
    assert_eq!(r.paired_tasks, 1);
    assert_eq!(r.unpaired.len(), 1);
    assert_eq!(r.unpaired[0].0, "only-record");
    assert!(r.render().contains("only-record (only in record)"));
}
