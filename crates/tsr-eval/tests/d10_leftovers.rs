//! D10: the leftover eval defects.
//!
//! Two of the D10 leftovers live in `tsr-eval`:
//!
//! * a task either system reported `Unavailable` is excluded from the
//!   completion metric (it is not a non-completion) and the report says how
//!   many were excluded;
//! * `DuplicateTask` names the duplicate row's task id, not the last key in
//!   the map.
//!
//! Both tests fail on the pre-fix code: the completion metric counted an
//! `Unavailable` task as a non-completion (so the rate was 1/2, not 1/1), and
//! `DuplicateTask.task` was `keys().next_back()` — the lexicographically last
//! task, not the duplicated one.

use std::fs;
use std::path::PathBuf;

/// A unique scratch directory under the target dir.
fn scratch(name: &str) -> PathBuf {
    let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    p.pop();
    p.pop();
    p.push("target");
    p.push("d10-scratch");
    p.push(name);
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).expect("create scratch");
    p
}

fn row(task: &str, system: &str, outcome: &str, accepted: bool) -> String {
    format!(
        r#"{{"task":"{task}","system":"{system}","outcome":"{outcome}","generator_tokens":1000,"tool_minutes":1,"wall_clock_ms":1000,"family":"parser","lineage":"lin","state_root_at_start":"aa","state_root_at_end":"bb","denials":[],"oracle":{{"flipped_tests_pass":{accepted},"suite_regressed":false,"accepted":{accepted},"exit_code":0}}}}"#
    )
}

#[test]
fn d10_unavailable_results_are_excluded_from_completion_with_a_count() {
    let base = scratch("unavailable");
    let record = base.join("record");
    let rules = base.join("rules");
    fs::create_dir_all(&record).unwrap();
    fs::create_dir_all(&rules).unwrap();
    // record: one accepted, one unavailable. rules: one accepted, one unavailable.
    // The unavailable task must be excluded from the completion denominator, so
    // both systems report 1/1 = 1.0, not 1/2 = 0.5.
    fs::write(
        record.join("results.jsonl"),
        format!(
            "{}\n{}\n",
            row("t1", "record", "accepted", true),
            row("t2", "record", "unavailable", false)
        ),
    )
    .unwrap();
    fs::write(
        rules.join("results.jsonl"),
        format!(
            "{}\n{}\n",
            row("t1", "rules", "accepted", true),
            row("t2", "rules", "unavailable", false)
        ),
    )
    .unwrap();

    let report = tsr_eval::baseline::baseline_pair(
        &record,
        &rules,
        tsr_eval::baseline::PairBy::Task,
        1,
        200,
    )
    .expect("baseline_pair");
    let completion = report.metric("completion_rate").expect("completion metric");
    assert_eq!(
        completion.record,
        Some(1.0),
        "the unavailable task must be excluded from the completion denominator"
    );
    assert_eq!(completion.rules, Some(1.0));
    assert_eq!(
        report.unavailable_excluded, 1,
        "the report must count the excluded unavailable task"
    );
    let rendered = report.render();
    assert!(
        rendered.contains("unavailable tasks excluded from completion: 1"),
        "the report must print the excluded count:\n{rendered}"
    );
}

#[test]
fn d10_duplicate_task_names_the_duplicate() {
    let base = scratch("duplicate");
    let record = base.join("record");
    let rules = base.join("rules");
    fs::create_dir_all(&record).unwrap();
    fs::create_dir_all(&rules).unwrap();
    // Two rows for the SAME task id `dup`, plus a lexicographically larger id
    // `zzz` that is NOT duplicated. Pre-fix, DuplicateTask named `zzz` (the last
    // key in the map); it must name `dup`.
    fs::write(
        record.join("results.jsonl"),
        format!(
            "{}\n{}\n{}\n",
            row("dup", "record", "accepted", true),
            row("zzz", "record", "accepted", true),
            row("dup", "record", "accepted", true)
        ),
    )
    .unwrap();
    fs::write(
        rules.join("results.jsonl"),
        format!("{}\n", row("dup", "rules", "accepted", true)),
    )
    .unwrap();

    let err = tsr_eval::baseline::baseline_pair(
        &record,
        &rules,
        tsr_eval::baseline::PairBy::Task,
        1,
        200,
    )
    .expect_err("a duplicate task must be an error");
    let msg = format!("{err}");
    assert!(
        msg.contains("dup"),
        "DuplicateTask must name the duplicated task `dup`, got: {msg}"
    );
    assert!(
        !msg.contains("zzz"),
        "DuplicateTask must not name the unrelated task `zzz`, got: {msg}"
    );
}
