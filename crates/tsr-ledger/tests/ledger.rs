//! The ledger's required behaviours (PRD §B11 M2 item 1), each as a test over a real store.

use tsr_domain::{
    ContrastId, Event, EventKind, Hash, Observation, Preference, PreferenceId,
    PreferenceStatus, Scope, Source, TaskId,
};
use tsr_ledger::{fold, normalize_text, Ledger, LedgerError};
use tsr_store::Store;

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Store::open(&dir.path().join("events.jsonl")).expect("open store");
    (dir, store)
}

fn task() -> TaskId {
    TaskId(Hash::of(&[b"t-1"]))
}

fn scope() -> Scope {
    Scope::Task(task())
}

fn contrast() -> ContrastId {
    ContrastId(Hash::of(&[b"c-1"]))
}

/// A generator-origin preference is in the structure but absent from the active view until an
/// Operator `StatusSet(Open)` exists for it; present after.
#[test]
fn generator_preference_waits_for_operator_open() {
    let (_dir, store) = store();
    let ledger = Ledger::new(&store);

    let id = ledger
        .expand(
            &task(),
            vec![],
            contrast(),
            "the retry budget is the dimension this contrast clarified",
            scope(),
            vec![],
            vec![],
            Source::Generator,
            "2026-01-01T00:00:00Z",
        )
        .expect("expand");

    let collective = ledger.collective().expect("fold");
    let entry = collective.get(&id).expect("in the structure");
    assert!(entry.status.is_none(), "generator origin is not open on arrival");
    assert!(!entry.is_active());
    assert!(!ledger.active().expect("active").contains(&id));

    ledger
        .set_status(
            &task(),
            vec![],
            id,
            PreferenceStatus::Open,
            Source::Operator,
            "2026-01-01T00:01:00Z",
        )
        .expect("operator admits it");

    let active = ledger.active().expect("active");
    assert!(active.contains(&id), "present after the operator opens it");
    assert!(active.get(&id).expect("entry").is_open());
}

/// An operator-origin preference — including a rejection reason — is open on arrival.
#[test]
fn operator_preference_is_open_on_arrival() {
    let (_dir, store) = store();
    let ledger = Ledger::new(&store);

    let id = ledger
        .expand(
            &task(),
            vec![],
            contrast(),
            "rejected: the diff changed a public signature without a migration",
            scope(),
            vec![],
            vec![],
            Source::Operator,
            "2026-01-01T00:00:00Z",
        )
        .expect("expand");

    let active = ledger.active().expect("active");
    let entry = active.get(&id).expect("open on arrival");
    assert!(entry.is_open());
    assert_eq!(entry.history.len(), 1);
    assert_eq!(entry.history[0].1, Source::Operator);
}

/// `withdrawn` and `out_of_scope` are operator-only: refused with exit 2 when `by` is not Operator.
#[test]
fn operator_only_statuses_are_refused_for_non_operator() {
    let (_dir, store) = store();
    let ledger = Ledger::new(&store);

    let id = ledger
        .expand(
            &task(),
            vec![],
            contrast(),
            "a preference",
            scope(),
            vec![],
            vec![],
            Source::Operator,
            "2026-01-01T00:00:00Z",
        )
        .expect("expand");

    for status in [PreferenceStatus::Withdrawn, PreferenceStatus::OutOfScope] {
        for by in [Source::Generator, Source::Rules] {
            let err = ledger
                .set_status(
                    &task(),
                    vec![],
                    id,
                    status.clone(),
                    by,
                    "2026-01-01T00:02:00Z",
                )
                .expect_err("refused");
            assert!(matches!(err, LedgerError::OperatorOnly { .. }), "{err:?}");
            assert_eq!(err.exit_code(), 2);
        }
    }

    // The operator may set them.
    ledger
        .set_status(
            &task(),
            vec![],
            id,
            PreferenceStatus::Withdrawn,
            Source::Operator,
            "2026-01-01T00:03:00Z",
        )
        .expect("operator may withdraw");
    assert!(!ledger.active().expect("active").get(&id).expect("entry").is_open());
}

/// A merge finds a duplicate by normalized text within the same scope and yields `Overlapping(id)`.
#[test]
fn duplicate_normalized_text_in_same_scope_overlaps() {
    let (_dir, store) = store();
    let ledger = Ledger::new(&store);

    let first = ledger
        .expand(
            &task(),
            vec![],
            contrast(),
            "The  Retry   Budget",
            scope(),
            vec![],
            vec![],
            Source::Operator,
            "2026-01-01T00:00:00Z",
        )
        .expect("expand");
    let second = ledger
        .expand(
            &task(),
            vec![],
            contrast(),
            "the retry budget",
            scope(),
            vec![],
            vec![],
            Source::Operator,
            "2026-01-01T00:01:00Z",
        )
        .expect("expand");

    assert_eq!(normalize_text("The  Retry   Budget"), "the retry budget");
    assert_eq!(
        ledger.overlapping_with("the retry budget", &scope()).expect("lookup"),
        Some(first)
    );

    let event = ledger
        .mark_overlapping(&task(), vec![], second, "2026-01-01T00:02:00Z")
        .expect("mark")
        .expect("a duplicate exists");
    assert_eq!(event.0, event.0);

    let active = ledger.active().expect("active");
    assert_eq!(
        active.get(&second).expect("entry").status,
        Some(PreferenceStatus::Overlapping(first))
    );

    // A different scope is not a duplicate.
    let other = Scope::Family("f-1".to_string());
    assert_eq!(ledger.overlapping_with("the retry budget", &other).expect("lookup"), None);
}

/// A later return meeting the criterion closes the preference: `Fulfilled`.
#[test]
fn a_later_return_meeting_the_criterion_fulfils() {
    let (_dir, store) = store();
    let ledger = Ledger::new(&store);

    let id = ledger
        .expand(
            &task(),
            vec![],
            contrast(),
            "the criterion: the suite passes on the merged head",
            scope(),
            vec![],
            vec![],
            Source::Operator,
            "2026-01-01T00:00:00Z",
        )
        .expect("expand");
    assert!(ledger.active().expect("active").get(&id).expect("entry").is_open());

    // The later return arrives as an event; the criterion closes and the status is mechanical.
    let ret = Event::new(
        task(),
        vec![],
        Source::Rules,
        EventKind::Observed {
            observation: Observation::Evidence {
                digest: Hash::of(&[b"return:test_pass"]),
                source: "tool:test".to_string(),
            },
        },
        "2026-01-01T00:05:00Z",
    );
    store.append(&ret).expect("append the return");

    ledger
        .mark_fulfilled(&task(), vec![ret.id], id, "2026-01-01T00:05:01Z")
        .expect("fulfil");

    let active = ledger.active().expect("active");
    let entry = active.get(&id).expect("entry");
    assert_eq!(entry.status, Some(PreferenceStatus::Fulfilled));
    assert!(!entry.is_open());
    assert_eq!(entry.history.last().expect("history").1, Source::Rules);
}

/// The fold is deterministic and the root hash equals `store.head_root()`.
#[test]
fn fold_is_deterministic_and_root_is_the_store_head() {
    let (_dir, store) = store();
    let ledger = Ledger::new(&store);

    let a = ledger
        .expand(
            &task(),
            vec![],
            contrast(),
            "first",
            scope(),
            vec![],
            vec![],
            Source::Operator,
            "2026-01-01T00:00:00Z",
        )
        .expect("expand");
    let b = ledger
        .expand(
            &task(),
            vec![],
            contrast(),
            "second",
            scope(),
            vec![a],
            vec![a],
            Source::Generator,
            "2026-01-01T00:01:00Z",
        )
        .expect("expand");
    ledger
        .set_status(
            &task(),
            vec![],
            b,
            PreferenceStatus::Open,
            Source::Operator,
            "2026-01-01T00:02:00Z",
        )
        .expect("open");

    let one = ledger.collective().expect("fold");
    let two = ledger.collective().expect("fold again");
    assert_eq!(one, two, "same events, same view");
    assert_eq!(one.root, store.head_root().expect("head"));
    assert_eq!(one.active(), two.active());

    // The fold is a pure function of the events and the root.
    let events = store.all().expect("all");
    let root = store.head_root().expect("head");
    assert_eq!(fold(&events, root), one);
    assert_eq!(fold(&events, root), one);

    // The produced preference is the content address of its own text, scope and parents.
    let entry = one.get(&b).expect("entry");
    let expected = Preference::new(
        entry.preference.text.clone(),
        entry.preference.scope.clone(),
        entry.preference.parents.clone(),
        entry.preference.origin,
    )
    .expect("mint");
    assert_eq!(entry.preference.id, expected.id);
    assert_eq!(entry.catalyzed_by, vec![a]);
    assert_eq!(entry.preference.parents, vec![a]);
}

/// A status for a preference the log never carried is refused, not silently folded.
#[test]
fn status_for_unknown_preference_is_refused() {
    let (_dir, store) = store();
    let ledger = Ledger::new(&store);
    let err = ledger
        .set_status(
            &task(),
            vec![],
            PreferenceId(Hash::of(&[b"pref-nope"])),
            PreferenceStatus::Open,
            Source::Operator,
            "2026-01-01T00:00:00Z",
        )
        .expect_err("refused");
    assert!(matches!(err, LedgerError::UnknownPreference(_)));
    assert_eq!(err.exit_code(), 2);
}

/// An `Expanded` event round-trips through serde and rebuilds the preference with the same id:
/// the fold reads the node from the log alone, never from a side table.
#[test]
fn expanded_event_round_trips_and_rebuilds_the_preference() {
    let (_dir, store) = store();
    let ledger = Ledger::new(&store);

    let id = ledger
        .expand(
            &task(),
            vec![],
            contrast(),
            "the dimension this contrast clarified",
            scope(),
            vec![],
            vec![],
            Source::Operator,
            "2026-01-01T00:00:00Z",
        )
        .expect("expand");

    let events = store.all().expect("all");
    let (_, ev) = events.first().expect("one event");
    let payload = serde_json::to_string(ev).expect("serialize");
    let back: Event = serde_json::from_str(&payload).expect("deserialize");
    let EventKind::Expanded { expansion } = &back.kind else {
        panic!("an Expanded event");
    };
    let rebuilt = Preference::new(
        expansion.text.clone(),
        expansion.scope.clone(),
        expansion.parents.clone(),
        expansion.origin,
    )
    .expect("mint");
    assert_eq!(rebuilt.id, expansion.produced);
    assert_eq!(rebuilt.id, id);
    assert_eq!(rebuilt.text, "the dimension this contrast clarified");
    assert_eq!(rebuilt.origin, Source::Operator);

    // The fold over the deserialized event alone yields the same node.
    let view = fold(&[(1, back)], store.head_root().expect("head"));
    assert_eq!(view.entries.len(), 1);
    assert_eq!(view.entries[0].preference.id, id);
}

/// A produced id that is not the content address of its own text, scope and parents is refused
/// with exit 2 — the ledger never mints a node it cannot re-derive from the log.
#[test]
fn produced_mismatch_is_refused() {
    let (_dir, store) = store();

    // A hand-built expansion whose `produced` does not address its content.
    let expansion = tsr_domain::Expansion {
        from_contrast: contrast(),
        produced: PreferenceId(Hash::of(&[b"not-the-address"])),
        catalyzed_by: vec![],
        text: "a preference".to_string(),
        scope: scope(),
        parents: vec![],
        origin: Source::Operator,
    };
    let expected = Preference::new(
        expansion.text.clone(),
        expansion.scope.clone(),
        expansion.parents.clone(),
        expansion.origin,
    )
    .expect("mint");
    assert_ne!(expected.id, expansion.produced);

    // The fold refuses to admit a node whose address does not match its content.
    let ev = Event::new(
        task(),
        vec![],
        Source::Operator,
        EventKind::Expanded { expansion },
        "2026-01-01T00:00:00Z",
    );
    let view = fold(&[(1, ev)], store.head_root().expect("head"));
    assert!(view.entries.is_empty(), "a mismatched node is not admitted");
}