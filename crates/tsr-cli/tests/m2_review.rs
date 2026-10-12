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
    Event, EventKind, Hash, LabelStatus, Observation, PreferenceStatus, Source, TaskId, UnmetKind,
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
    // The Filter's write rule admits an expansion only from a Contrast the log holds.
    let contrast = tsr_ledger::Filter::contrast_from_criteria(
        tsr_domain::EventId(Hash::of(&[b"return", text.as_bytes()])),
        vec![(
            tsr_domain::CriterionId(Hash::of(&[b"criterion", text.as_bytes()])),
            tsr_domain::UnmetKind::Rejected {
                reason: "the criterion went unmet".to_string(),
            },
        )],
        Vec::new(),
    );
    ledger
        .record_contrast(task, Vec::new(), contrast.clone(), "unix:1")
        .unwrap();
    ledger
        .expand(
            task,
            Vec::new(),
            contrast.id,
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
            assert_eq!(record.status, LabelStatus::Rejected);
            assert_eq!(record.reviewer, Source::Operator);
            assert_eq!(record.reason.as_deref(), Some("touched the schema"));
            assert!(record.evidence.is_none(), "the reason is not the evidence");
            assert!(record.supersedes.is_none());
        }
        other => panic!("expected an outcome record, got {other:?}"),
    }

    // 2. The rejection reason entered the ledger as a Contrast (Rejected(reason)) recorded in an
    //    Observed event, and the Filter expanded it into an Operator preference that names the
    //    dimension ("rejection"), never the failed strategy's content (PRD §B2.1, lane 04 item 2).
    let after = store.by_task(&t).unwrap();
    let contrast = after
        .iter()
        .find_map(|(_, e)| match &e.kind {
            EventKind::Observed {
                observation: Observation::Return { contrast, .. },
            } => contrast.as_ref().filter(|c| {
                c.unmet.iter().any(|(_, kind)| {
                    matches!(kind, UnmetKind::Rejected { reason } if reason == "touched the schema")
                })
            }),
            _ => None,
        })
        .expect("the rejection wrote a Contrast through the Filter");
    assert!(
        contrast.unmet.iter().any(|(_, kind)| matches!(
            kind,
            UnmetKind::Rejected { reason } if reason == "touched the schema"
        )),
        "the Contrast carries the rejection reason as Rejected(reason)"
    );

    let ledger = Ledger::new(&store);
    let view = ledger.active_for_task(&t).unwrap();
    let rejection = view
        .entries
        .iter()
        .find(|e| e.preference.text.contains("rejection"))
        .expect("the rejection reason is a preference");
    assert!(
        !rejection.preference.text.contains("touched the schema"),
        "the produced preference names the dimension, never the strategy's content"
    );
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
            green: true,
        },
        LabelRow {
            subject: tsr_domain::EventId(Hash::of(&[b"r2"])),
            label: Label::Accepted,
            weight: None,
            family: Some("refactor".to_string()),
            green: true,
        },
        LabelRow {
            subject: tsr_domain::EventId(Hash::of(&[b"r3"])),
            label: Label::Rejected,
            weight: Some(2.0),
            family: Some("refactor".to_string()),
            green: true,
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
        store: Some(":memory:".to_string()),
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
    let contrast = tsr_ledger::Filter::contrast_from_criteria(
        subject,
        vec![(
            tsr_domain::CriterionId(Hash::of(&[b"family-criterion"])),
            tsr_domain::UnmetKind::Rejected {
                reason: "the criterion went unmet".to_string(),
            },
        )],
        Vec::new(),
    );
    ledger
        .record_contrast(&t, Vec::new(), contrast.clone(), "unix:11")
        .unwrap();
    ledger
        .expand(
            &t,
            Vec::new(),
            contrast.id,
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

// ---------------------------------------------------------------------------------------------
// D3 — the review rejection path: a rejection writes a Contrast through the Filter and expands
// from it; `Ledger::expand` refuses a ContrastId with no Contrast event in the log, and refuses a
// Contrast with no unmet criterion.
// ---------------------------------------------------------------------------------------------

/// D3: `Ledger::expand` refuses a ContrastId the log does not hold (exit 2 at the CLI).
#[test]
fn d3_expand_refuses_unknown_contrast_id() {
    let store = Store::open_in_memory().unwrap();
    let t = task(20);
    let ledger = Ledger::new(&store);
    let unknown = tsr_domain::ContrastId(Hash::of(&[b"no-such-contrast"]));
    let err = ledger
        .expand(
            &t,
            Vec::new(),
            unknown,
            "a preference from nowhere",
            tsr_domain::Scope::Task(t),
            Vec::new(),
            Vec::new(),
            Source::Operator,
            "unix:20",
        )
        .expect_err("an unknown contrast id is refused");
    assert!(
        matches!(err, tsr_ledger::LedgerError::UnknownContrast(id) if id == unknown),
        "the refusal names the unknown contrast: {err:?}"
    );
    assert_eq!(err.exit_code(), 2, "the refusal is exit 2");
    // Nothing was appended: the log still holds no expansion.
    let events = store.by_task(&t).unwrap();
    assert!(
        !events
            .iter()
            .any(|(_, e)| matches!(e.kind, EventKind::Expanded { .. })),
        "a refused expansion appends nothing"
    );
}

/// D3: `Ledger::expand` refuses a Contrast that has no unmet criterion.
#[test]
fn d3_expand_refuses_contrast_without_unmet() {
    let store = Store::open_in_memory().unwrap();
    let t = task(21);
    let subject = append_return(&store, &t, 21);
    // A Contrast with an empty `unmet` list: nothing crossed, so nothing may expand.
    let empty = tsr_ledger::Filter::contrast_from_criteria(subject, Vec::new(), vec![subject]);
    let observed = Event::new(
        t,
        Vec::new(),
        Source::Rules,
        EventKind::Observed {
            observation: Observation::Return {
                surprise: Vec::new(),
                contrast: Some(empty.clone()),
            },
        },
        "unix:21",
    );
    store.append(&observed).unwrap();

    let ledger = Ledger::new(&store);
    let err = ledger
        .expand(
            &t,
            vec![observed.id],
            empty.id,
            "a preference from an empty contrast",
            tsr_domain::Scope::Task(t),
            Vec::new(),
            Vec::new(),
            Source::Operator,
            "unix:21",
        )
        .expect_err("a contrast with no unmet criterion is refused");
    assert!(
        matches!(err, tsr_ledger::LedgerError::ContrastWithoutUnmet(id) if id == empty.id),
        "the refusal names the empty contrast: {err:?}"
    );
    assert_eq!(err.exit_code(), 2, "the refusal is exit 2");
}

/// D3: a rejection writes the Contrast `{ unmet: Rejected(reason) }` through the Filter, expands
/// from it, and the Mirror's Contrast section shows the rejection.
#[test]
fn d3_rejection_writes_contrast_and_mirror_shows_it() {
    let store = Store::open_in_memory().unwrap();
    let t = task(22);
    let subject = append_return(&store, &t, 22);
    let reason = "regresses span offsets";

    record_label(
        &store,
        &t,
        subject,
        Label::Rejected,
        Some(reason.to_string()),
        None,
        None,
        "unix:22",
    )
    .unwrap();

    // 1. The log holds a Contrast event whose unmet criterion is Rejected(reason).
    let events = store.by_task(&t).unwrap();
    let contrast = events
        .iter()
        .find_map(|(_, e)| match &e.kind {
            EventKind::Observed {
                observation: Observation::Return { contrast, .. },
            } => contrast.clone(),
            _ => None,
        })
        .expect("the rejection wrote a Contrast event");
    assert_eq!(contrast.from_return, subject);
    assert_eq!(contrast.unmet.len(), 1, "one unmet criterion");
    match &contrast.unmet[0].1 {
        tsr_domain::UnmetKind::Rejected { reason: r } => assert_eq!(r, reason),
        other => panic!("expected Rejected(reason), got {other:?}"),
    }

    // 2. The Expansion names that Contrast, so the Filter is the only path to expansion.
    let expansion = events
        .iter()
        .find_map(|(_, e)| match &e.kind {
            EventKind::Expanded { expansion } => Some(expansion.clone()),
            _ => None,
        })
        .expect("the rejection expanded");
    assert_eq!(
        expansion.from_contrast, contrast.id,
        "the expansion names the written contrast"
    );

    // 3. The Mirror's Contrast section shows the rejection.
    let root = store.head_root().unwrap();
    let mirror = tsr_cli::reflect_cmd::build_mirror(&t, &events, root);
    assert!(
        mirror
            .residual
            .contrast
            .iter()
            .any(|c| c.unmet == "rejected" && c.reason.as_deref() == Some(reason)),
        "the Mirror's Contrast section shows the rejection: {:?}",
        mirror.residual.contrast
    );
}
