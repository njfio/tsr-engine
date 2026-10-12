//! D4 / D7 / D8 — review statistics, sampling and the rejected label status.
//!
//! D4: `tsr review --family <f> --stats` must not require a task; it computes over every task of
//! the family. The false-accept rate counts only greens (returns whose own outcome passed) that a
//! reviewer rejected; a failed return is not "green".
//! D8: `--sample` draws only over greens and records the inverse-probability weight on sampled
//! greens only; `--return <id>` is checked against the task (UnknownReturn exits 2).
//! D7: `rejected` is its own `LabelStatus` (do not reuse `Invalid`); `resolve.rs` maps `Invalid`
//! to `invalid`, not to `rejected`.

use tsr_cli::review_cmd::{
    false_accept_rate, label_rows, pending_returns, record_label, sample_greens, Label, LabelRow,
    ReviewArgs, ReviewError,
};
use tsr_domain::{Event, EventKind, Hash, LabelStatus, Source, TaskId};
use tsr_store::Store;

fn task(seed: u8) -> TaskId {
    TaskId(Hash::of(&[b"task", &[seed]]))
}

/// A unique temp db path for this test process.
fn temp_db(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("tsr-review-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join("tsr.db")
}

/// Append an `Executed` return with the given exit code.
fn append_return(store: &Store, task: &TaskId, seed: u8, exit_code: i32) -> tsr_domain::EventId {
    let candidate = tsr_domain::CandidateId(Hash::of(&[b"candidate", &[seed]]));
    let ev = Event::new(
        *task,
        Vec::new(),
        Source::Generator,
        EventKind::Executed {
            candidate,
            exit_code: Some(exit_code),
            log_digest: Some(Hash::of(&[b"digest", &[seed]])),
            duration_ms: 10,
        },
        "unix:1",
    );
    store.append(&ev).unwrap();
    ev.id
}

/// D7: a rejection records `LabelStatus::Rejected`, its own status — not `Invalid`.
#[test]
fn d7_rejection_status_is_rejected_not_invalid() {
    let store = Store::open_in_memory().unwrap();
    let t = task(1);
    let subject = append_return(&store, &t, 1, 0);
    let record_id = record_label(
        &store,
        &t,
        subject,
        Label::Rejected,
        Some("no".to_string()),
        None,
        None,
        "unix:2",
    )
    .unwrap();
    let events = store.by_task(&t).unwrap();
    let record = events
        .iter()
        .find(|(_, e)| e.id == record_id)
        .map(|(_, e)| e)
        .expect("the outcome record is in the log");
    match &record.kind {
        EventKind::OutcomeRecorded { record } => {
            assert_eq!(
                record.status,
                LabelStatus::Rejected,
                "a rejection is its own status, not Invalid"
            );
        }
        other => panic!("expected an outcome record, got {other:?}"),
    }
    // The status round-trips through its name.
    assert_eq!(LabelStatus::Rejected.name(), "rejected");
    assert_eq!(LabelStatus::parse("rejected"), Some(LabelStatus::Rejected));
}

/// D7: `label_rows` maps a `Rejected` record to `Label::Rejected` and marks a green return.
#[test]
fn d7_label_rows_maps_rejected_and_marks_green() {
    let store = Store::open_in_memory().unwrap();
    let t = task(2);
    let green = append_return(&store, &t, 2, 0);
    record_label(
        &store,
        &t,
        green,
        Label::Rejected,
        Some("no".to_string()),
        None,
        None,
        "unix:2",
    )
    .unwrap();
    let rows = label_rows(&store, &t).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].label, Label::Rejected);
    assert!(rows[0].green, "the return's own outcome passed");
}

/// D4: the false-accept rate counts only greens that were rejected; a failed return is not green.
#[test]
fn d4_false_accept_rate_counts_only_greens() {
    // A green that was rejected: a false accept.
    let green_rejected = LabelRow {
        subject: tsr_domain::EventId(Hash::of(&[b"r1"])),
        label: Label::Rejected,
        weight: None,
        family: Some("alpha".to_string()),
        green: true,
    };
    // A green that was accepted: not a false accept.
    let green_accepted = LabelRow {
        subject: tsr_domain::EventId(Hash::of(&[b"r2"])),
        label: Label::Accepted,
        weight: None,
        family: Some("alpha".to_string()),
        green: true,
    };
    // A failed return that was rejected: NOT a green, so it is not a false accept.
    let failed_rejected = LabelRow {
        subject: tsr_domain::EventId(Hash::of(&[b"r3"])),
        label: Label::Rejected,
        weight: None,
        family: Some("alpha".to_string()),
        green: false,
    };
    let rate = false_accept_rate(&[green_rejected, green_accepted, failed_rejected]);
    assert_eq!(rate.greens, 2, "only the two greens count");
    assert_eq!(
        rate.false_accepts, 1,
        "only the rejected green is a false accept"
    );
    assert_eq!(rate.labels, 3, "every label is behind the rate");
    assert_eq!(rate.unweighted(), Some(0.5));
}

/// D8: `--sample` draws only over greens; a failed return is never selected.
#[test]
fn d8_sample_draws_only_over_greens() {
    let store = Store::open_in_memory().unwrap();
    let t = task(3);
    let green = append_return(&store, &t, 3, 0);
    let failed = append_return(&store, &t, 4, 1);
    let pending = pending_returns(&store, &t).unwrap();
    // rate 1.0 selects every green; the failure must not appear.
    let selected = sample_greens(&pending, 1.0, 7).unwrap();
    let ids: Vec<_> = selected.iter().map(|(id, _)| *id).collect();
    assert!(ids.contains(&green), "the green is selected at rate 1.0");
    assert!(!ids.contains(&failed), "a failure is never sampled");
    assert_eq!(selected.len(), 1, "only the green is in the draw");
    assert_eq!(selected[0].1, 1.0, "the weight is 1/rate");
}

/// D8: `--return <id>` naming a return the task does not hold is `UnknownReturn` (exit 2).
#[test]
fn d8_unknown_return_is_refused() {
    let db = temp_db("unknown-return");
    let _ = std::fs::remove_file(&db);
    let store = Store::open(&db).unwrap();
    let t = task(4);
    let _ = append_return(&store, &t, 5, 0);
    drop(store);
    let args = ReviewArgs {
        task: t.0.to_hex(),
        label: Some("accepted".to_string()),
        reason: None,
        return_id: Some(Hash::of(&[b"no-such-return"]).to_hex()),
        sample: None,
        seed: None,
        family: None,
        stats: false,
        store: Some(db.to_string_lossy().to_string()),
    };
    let code = tsr_cli::review_cmd::run(&args);
    assert_eq!(
        format!("{code:?}"),
        format!("{:?}", std::process::ExitCode::from(2)),
        "an unknown return id exits 2"
    );
    // The typed refusal is `UnknownReturn`.
    let err = ReviewError::UnknownReturn(Hash::of(&[b"no-such-return"]).to_hex());
    assert_eq!(err.exit_code(), 2);
}

/// D4: the CLI itself accepts `tsr review --family <f> --stats` with no `<TASK>` positional.
///
/// Before the fix `main.rs` declared `task: String` (required), so clap exited 2 with
/// "required arguments were not provided: <TASK>". The library already handles an empty task for
/// `--stats`; the CLI must make the positional optional and pass an empty string.
#[test]
fn d4_cli_stats_without_task_exits_0() {
    let dir = std::env::temp_dir().join(format!("tsr-review-cli-stats-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("tsr.db");
    let _ = std::fs::remove_file(&db);
    {
        let store = Store::open(&db).unwrap();
        let t = task(20);
        let green = append_return(&store, &t, 20, 0);
        let ledger = tsr_ledger::Ledger::new(&store);
        let contrast = tsr_ledger::Filter::contrast_from_criteria(
            green,
            vec![(
                tsr_domain::CriterionId(Hash::of(&[b"crit", &[20u8]])),
                tsr_domain::UnmetKind::Rejected {
                    reason: "x".to_string(),
                },
            )],
            Vec::new(),
        );
        ledger
            .record_contrast(&t, Vec::new(), contrast.clone(), "unix:1")
            .unwrap();
        ledger
            .expand(
                &t,
                Vec::new(),
                contrast.id,
                "prefer small diffs",
                tsr_domain::Scope::Family("alpha".to_string()),
                Vec::new(),
                Vec::new(),
                Source::Operator,
                "unix:1",
            )
            .unwrap();
        record_label(
            &store,
            &t,
            green,
            Label::Rejected,
            Some("no".to_string()),
            None,
            None,
            "unix:2",
        )
        .unwrap();
    }
    let bin = env!("CARGO_BIN_EXE_tsr");
    let out = std::process::Command::new(bin)
        .args([
            "review",
            "--family",
            "alpha",
            "--stats",
            "--store",
            db.to_str().unwrap(),
        ])
        .output()
        .expect("the tsr binary runs");
    assert_eq!(
        out.status.code(),
        Some(0),
        "`tsr review --family alpha --stats` with no task must exit 0; stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("family=alpha"),
        "the family's stats are printed; stdout: {stdout}"
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// D4: `--stats` without a task computes over every task of the family.
#[test]
fn d4_stats_without_task_over_family() {
    let db = temp_db("stats-family");
    let _ = std::fs::remove_file(&db);
    let store = Store::open(&db).unwrap();
    // Two tasks in family "alpha", one in "beta".
    for (seed, fam) in [(10u8, "alpha"), (11u8, "alpha"), (12u8, "beta")] {
        let t = task(seed);
        let green = append_return(&store, &t, seed, 0);
        // A family-scoped preference names the family.
        let ledger = tsr_ledger::Ledger::new(&store);
        let contrast = tsr_ledger::Filter::contrast_from_criteria(
            green,
            vec![(
                tsr_domain::CriterionId(Hash::of(&[b"crit", &[seed]])),
                tsr_domain::UnmetKind::Rejected {
                    reason: "x".to_string(),
                },
            )],
            Vec::new(),
        );
        ledger
            .record_contrast(&t, Vec::new(), contrast.clone(), "unix:1")
            .unwrap();
        ledger
            .expand(
                &t,
                Vec::new(),
                contrast.id,
                "prefer small diffs",
                tsr_domain::Scope::Family(fam.to_string()),
                Vec::new(),
                Vec::new(),
                Source::Operator,
                "unix:1",
            )
            .unwrap();
        record_label(
            &store,
            &t,
            green,
            Label::Rejected,
            Some("no".to_string()),
            None,
            None,
            "unix:2",
        )
        .unwrap();
    }
    drop(store);
    // `--stats --family alpha` with no task: exit 0 and the family's labels counted.
    let args = ReviewArgs {
        task: String::new(),
        label: None,
        reason: None,
        return_id: None,
        sample: None,
        seed: None,
        family: Some("alpha".to_string()),
        stats: true,
        store: Some(db.to_string_lossy().to_string()),
    };
    let code = tsr_cli::review_cmd::run(&args);
    assert_eq!(
        format!("{code:?}"),
        format!("{:?}", std::process::ExitCode::from(0)),
        "--stats --family without a task exits 0"
    );
}

/// D4: `tsr review --family alpha` without `--stats` must exit 2 with a message about the missing
/// task, not `invalid task id ""`.
///
/// The positional is optional so `--stats` can omit it; without `--stats` the empty string used to
/// reach `Hash::from_hex("")` and print `tsr: invalid task id "": …`, which names the wrong
/// problem. The CLI must say the task is missing.
#[test]
fn d4_cli_family_without_stats_names_the_missing_task() {
    let dir = std::env::temp_dir().join(format!("tsr-review-cli-nostats-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("tsr.db");
    let _ = std::fs::remove_file(&db);
    {
        let store = Store::open(&db).unwrap();
        let t = task(30);
        let _ = append_return(&store, &t, 30, 0);
    }
    let bin = env!("CARGO_BIN_EXE_tsr");
    let out = std::process::Command::new(bin)
        .args([
            "review",
            "--family",
            "alpha",
            "--store",
            db.to_str().unwrap(),
        ])
        .output()
        .expect("the tsr binary runs");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert_eq!(
        out.status.code(),
        Some(2),
        "`tsr review --family alpha` without --stats exits 2; stderr: {stderr}"
    );
    assert!(
        !stderr.contains("invalid task id"),
        "the message must not be `invalid task id \"\"`; stderr: {stderr}"
    );
    assert!(
        stderr.contains("task"),
        "the message names the missing task; stderr: {stderr}"
    );
    std::fs::remove_dir_all(&dir).ok();
}
