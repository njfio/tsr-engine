//! `tsr review` — outcome records, superseding labels, the rejection reason into the ledger, and
//! sampled greens with inverse-probability weights (PRD §B5, §B7, §B11 M2 item 4).
//!
//! The tests drive the store API directly, so they assert the exact events appended:
//!
//! 1. a rejection appends an outcome record AND an Operator preference via an `Expansion` whose
//!    `Contrast` is `Rejected(reason)` and whose `catalyzed_by` names the task's open preferences;
//! 2. a second label appends a superseding record — the first is still in the log, unrewritten;
//! 3. sampling with a fixed seed is deterministic and records the inverse-probability weight;
//! 4. every failure is listed regardless of the sample;
//! 5. `--stats` prints weighted and unweighted false-accept rates with the label count.

use tsr_cli::review_cmd::{
    false_accept_rate, label_rows, open_preferences, pending_returns, record_label, sample_greens,
    Label, LabelRow, ReviewArgs,
};
use tsr_domain::{
    Event, EventKind, Hash, LabelStatus, Observation, PreferenceStatus, Source, TaskId,
};
use tsr_ledger::Ledger;
use tsr_store::Store;

/// A task id from a fixed seed, so tests are reproducible.
fn task(seed: u8) -> TaskId {
    TaskId(Hash::of(&[b"m2-review-task", &[seed]]))
}

/// Append an `Executed` return for a candidate, so the task has something to label.
fn append_return(store: &Store, task: &TaskId, seed: u8) -> tsr_domain::EventId {
    let candidate = tsr_domain::CandidateId(Hash::of(&[b"candidate", &[seed]]));
    let ev = Event::new(
        *task,
        Vec::new(),
        Source::Generator,
        EventKind::Executed {
            candidate,
            exit_code: Some(0),
            log_digest: Some(Hash::of(&[b"digest", &[seed]])),
            duration_ms: 10,
        },
        "unix:1",
    );
    store.append(&ev).unwrap();
    ev.id
}

/// Open a preference on the task through an Expansion, so `catalyzed_by` has something to name.
fn open_preference(store: &Store, task: &TaskId, text: &str) -> tsr_domain::PreferenceId {
    let ledger = Ledger::new(store);
    let contrast = tsr_domain::ContrastId(Hash::of(&[b"contrast", text.as_bytes()]));
    ledger
        .expand(
            task,
            Vec::new(),
            contrast,
            text,
            tsr_domain::Scope::Task(*task),
            Vec::new(),
            Vec::new(),
            Source::Operator,
            "unix:1",
        )
        .unwrap()
}

#[test]
fn rejection_appends_record_and_operator_preference() {
    let store = Store::open_in_memory().unwrap();
    let t = task(1);
    let subject = append_return(&store, &t, 1);
    let open = open_preference(&store, &t, "the diff must not touch the schema");

    let record_id = record_label(
        &store,
        &t,
        subject,
        Label::Rejected,
        Some("touched the schema".to_string()),
        None,
        None,
        "unix:2",
    )
    .unwrap();

    // 1. The outcome record is in the log, with Source::Operator and the reason apart from evidence.
    let events = store.by_task(&t).unwrap();
    let record = events
        .iter()
        .find(|(_, e)| e.id == record_id)
        .map(|(_, e)| e)
        .expect("the outcome record is in the log");
    assert_eq!(record.source, Source::Operator);
    match &record.kind {
        EventKind::OutcomeRecorded { record } => {
            assert_eq!(record.subject, subject);
            assert_eq!(record.status, LabelStatus::Invalid);
            assert_eq!(record.reviewer, Source::Operator);
            assert_eq!(record.reason.as_deref(), Some("touched the schema"));
            assert!(record.evidence.is_none(), "the reason is not the evidence");
            assert!(record.supersedes.is_none());
        }
        other => panic!("expected an outcome record, got {other:?}"),
    }

    // 2. The rejection reason entered the ledger as an Operator preference through an Expansion.
    let ledger = Ledger::new(&store);
    let view = ledger.active_for_task(&t).unwrap();
    let rejection = view
        .entries
        .iter()
        .find(|e| e.preference.text.contains("touched the schema"))
        .expect("the rejection reason is a preference");
    assert_eq!(rejection.preference.origin, Source::Operator);
    assert_eq!(rejection.status, Some(PreferenceStatus::Open));
    assert!(
        rejection.catalyzed_by.contains(&open),
        "catalyzed_by names the task's open preferences"
    );
}

#[test]
fn second_label_supersedes_without_rewriting() {
    let store = Store::open_in_memory().unwrap();
    let t = task(2);
    let subject = append_return(&store, &t, 2);

    let first = record_label(
        &store,
        &t,
        subject,
        Label::Accepted,
        None,
        None,
        None,
        "unix:2",
    )
    .unwrap();
    let second = record_label(
        &store,
        &t,
        subject,
        Label::Rejected,
        Some("actually wrong".to_string()),
        None,
        None,
        "unix:3",
    )
    .unwrap();
    assert_ne!(first, second);

    let events = store.by_task(&t).unwrap();
    // The first record is still in the log, unrewritten.
    let first_ev = events
        .iter()
        .find(|(_, e)| e.id == first)
        .map(|(_, e)| e)
        .expect("the first record is still in the log");
    match &first_ev.kind {
        EventKind::OutcomeRecorded { record } => {
            assert_eq!(record.status, LabelStatus::Observed);
            assert!(record.supersedes.is_none());
        }
        other => panic!("expected an outcome record, got {other:?}"),
    }
    // The second names the first as superseded.
    let second_ev = events
        .iter()
        .find(|(_, e)| e.id == second)
        .map(|(_, e)| e)
        .expect("the second record is in the log");
    match &second_ev.kind {
        EventKind::OutcomeRecorded { record } => {
            assert_eq!(record.supersedes, Some(first));
        }
        other => panic!("expected an outcome record, got {other:?}"),
    }

    // The return no longer awaits a label: the standing record is the second.
    let pending = pending_returns(&store, &t).unwrap();
    assert!(pending.is_empty(), "the return is labelled");

    // The standing label is the second, not the first.
    let rows = label_rows(&store, &t).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].label, Label::Rejected);
}

#[test]
fn sampling_is_deterministic_and_records_the_weight() {
    let store = Store::open_in_memory().unwrap();
    let t = task(3);
    let mut returns = Vec::new();
    for i in 0..20u8 {
        returns.push(append_return(&store, &t, i));
    }
    let pending = pending_returns(&store, &t).unwrap();
    assert_eq!(pending.len(), 20);

    let a = sample_greens(&pending, 0.5, 42).unwrap();
    let b = sample_greens(&pending, 0.5, 42).unwrap();
    assert_eq!(a, b, "the same seed draws the same sample");
    assert!(!a.is_empty(), "a 0.5 rate over 20 returns selects some");
    for (_, w) in &a {
        assert!((w - 2.0).abs() < 1e-12, "the weight is 1/rate");
    }

    // A different seed draws a different sample (with overwhelming probability).
    let c = sample_greens(&pending, 0.5, 7).unwrap();
    assert_ne!(a, c, "a different seed draws a different sample");

    // The weight is recorded on the record when the label is written.
    let (subject, weight) = a[0];
    let id = record_label(
        &store,
        &t,
        subject,
        Label::Accepted,
        None,
        None,
        Some(weight),
        "unix:4",
    )
    .unwrap();
    let events = store.by_task(&t).unwrap();
    let ev = &events.iter().find(|(_, e)| e.id == id).unwrap().1;
    match &ev.kind {
        EventKind::OutcomeRecorded { record } => {
            assert_eq!(record.weight, Some(2.0));
        }
        other => panic!("expected an outcome record, got {other:?}"),
    }
}

#[test]
fn every_failure_is_listed_regardless_of_the_sample() {
    let store = Store::open_in_memory().unwrap();
    let t = task(4);
    // A failure: a return with a non-zero exit code.
    let candidate = tsr_domain::CandidateId(Hash::of(&[b"failed-candidate"]));
    let failed = Event::new(
        t,
        Vec::new(),
        Source::Generator,
        EventKind::Executed {
            candidate,
            exit_code: Some(1),
            log_digest: Some(Hash::of(&[b"failed-digest"])),
            duration_ms: 5,
        },
        "unix:1",
    );
    store.append(&failed).unwrap();

    let pending = pending_returns(&store, &t).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].exit_code, Some(1));

    // A sample rate that selects nothing still lists the failure: the listing is unconditional.
    let sampled = sample_greens(&pending, 0.0001, 1).unwrap();
    assert!(sampled.is_empty(), "the sample selected nothing");
    let listed = pending_returns(&store, &t).unwrap();
    assert_eq!(
        listed.len(),
        1,
        "the failure is listed regardless of the sample"
    );
}

#[test]
fn stats_prints_weighted_and_unweighted_false_accept_rates() {
    // Three greens, one of them rejected: unweighted 1/3. The rejected one was sampled at rate
    // 0.5 (weight 2), the two accepted ones reviewed unconditionally (weight 1).
    let rows = vec![
        LabelRow {
            subject: tsr_domain::EventId(Hash::of(&[b"r1"])),
            label: Label::Accepted,
            weight: None,
            family: Some("refactor".to_string()),
        },
        LabelRow {
            subject: tsr_domain::EventId(Hash::of(&[b"r2"])),
            label: Label::Accepted,
            weight: None,
            family: Some("refactor".to_string()),
        },
        LabelRow {
            subject: tsr_domain::EventId(Hash::of(&[b"r3"])),
            label: Label::Rejected,
            weight: Some(2.0),
            family: Some("refactor".to_string()),
        },
    ];
    let rate = false_accept_rate(&rows);
    assert_eq!(rate.labels, 3);
    assert_eq!(rate.greens, 3);
    assert_eq!(rate.false_accepts, 1);
    let unweighted = rate.unweighted().unwrap();
    assert!(
        (unweighted - 1.0 / 3.0).abs() < 1e-12,
        "1 false accept / 3 greens"
    );
    let weighted = rate.weighted().unwrap();
    // weighted false accepts = 2, weighted greens = 1 + 1 + 2 = 4 -> 0.5
    assert!((weighted - 0.5).abs() < 1e-12);
    assert_eq!(rate.labels, 3, "the label count is behind the rate");
}

#[test]
fn review_args_default_lists_pending() {
    let store = Store::open_in_memory().unwrap();
    let t = task(5);
    let subject = append_return(&store, &t, 5);
    let pending = pending_returns(&store, &t).unwrap();
    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0].event, subject);

    // A note is not a label: it does not clear the pending return.
    let note = Event::new(
        t,
        Vec::new(),
        Source::Operator,
        EventKind::Observed {
            observation: Observation::Experience {
                note: "felt slow".to_string(),
            },
        },
        "unix:2",
    );
    store.append(&note).unwrap();
    assert_eq!(pending_returns(&store, &t).unwrap().len(), 1);

    // The args struct carries the store path; the default lists.
    let args = ReviewArgs {
        task: t.0.to_hex(),
        store: ":memory:".to_string(),
        ..Default::default()
    };
    assert!(args.label.is_none());
    assert!(!args.stats);
    let _ = open_preferences(&store, &t).unwrap();
}

#[test]
fn cli_return_flag_corrects_a_standing_label() {
    // `tsr review <task> accepted --return <id>` names an already-labelled return: the correction
    // appends a superseding record, and the standing label becomes the correction.
    let store = Store::open_in_memory().unwrap();
    let t = task(9);
    let subject = append_return(&store, &t, 9);

    let first = record_label(
        &store,
        &t,
        subject,
        Label::Rejected,
        Some("wrong".to_string()),
        None,
        None,
        "unix:9",
    )
    .unwrap();
    let second = record_label(
        &store,
        &t,
        subject,
        Label::Accepted,
        None,
        None,
        None,
        "unix:10",
    )
    .unwrap();
    assert_ne!(first, second);

    let rows = label_rows(&store, &t).unwrap();
    assert_eq!(rows.len(), 1, "only the standing label counts");
    assert_eq!(rows[0].label, Label::Accepted);
    assert_eq!(rows[0].subject, subject);
}

#[test]
fn family_filter_matches_the_tasks_family_scope() {
    // A task's family is the `Scope::Family` of one of its preferences; `--family <f> --stats`
    // keeps only the labels of tasks in that family.
    let store = Store::open_in_memory().unwrap();
    let t = task(11);
    let subject = append_return(&store, &t, 11);
    let ledger = Ledger::new(&store);
    ledger
        .expand(
            &t,
            Vec::new(),
            tsr_domain::ContrastId(Hash::of(&[b"family-contrast"])),
            "prefer small diffs",
            tsr_domain::Scope::Family("refactor".to_string()),
            Vec::new(),
            Vec::new(),
            Source::Operator,
            "unix:11",
        )
        .unwrap();
    record_label(
        &store,
        &t,
        subject,
        Label::Rejected,
        Some("no".into()),
        None,
        None,
        "unix:12",
    )
    .unwrap();

    let rows = label_rows(&store, &t).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].family.as_deref(), Some("refactor"));
}
