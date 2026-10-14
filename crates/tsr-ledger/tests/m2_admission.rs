//! D2 — admission by source (PRD §B2.1, §B11 M2 item 1; specs/lanes/04r-reflect-repair.md item (1)).
//!
//! A preference whose origin is `Source::Generator` is in the structure but absent from the active
//! view until an **Operator** `StatusSet(Open)` exists for it. A non-operator `Open` on it is
//! recorded (it appears in the history) and ignored for admission. Statuses that are mechanical
//! (`Fulfilled`, `Overlapping`) apply only to preferences already admitted.

use tsr_domain::{
    ContrastId, CriterionId, Event, EventId, EventKind, Hash, Observation, PreferenceStatus, Scope,
    Source, TaskId, UnmetKind,
};
use tsr_ledger::{Ledger, LedgerError};
use tsr_store::Store;

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Store::open(&dir.path().join("events.jsonl")).expect("open store");
    (dir, store)
}

fn task() -> TaskId {
    TaskId(Hash::of(&[b"t-d2"]))
}

fn scope() -> Scope {
    Scope::Task(task())
}

/// A Contrast the log holds: the Filter's write rule refuses an expansion that names anything
/// else, so a test that expands must first record the contrast it expands from.
fn contrast(store: &Store) -> ContrastId {
    // D12: a Contrast's id is its content address, so mint it through the Filter.
    let c = tsr_ledger::Filter::contrast_from_criteria(
        EventId(Hash::of(&[b"return-d2"])),
        vec![(
            CriterionId(Hash::of(&[b"criterion-d2"])),
            UnmetKind::Rejected {
                reason: "the criterion went unmet".to_string(),
            },
        )],
        vec![],
    );
    Ledger::new(store)
        .record_contrast(&task(), vec![], c.clone(), "2026-01-01T00:00:00Z")
        .expect("record the contrast");
    c.id
}

/// D2: a `StatusSet(Open)` by a non-operator source does not admit a Generator-origin preference.
/// The event is recorded (it is in the history) but the preference stays out of the active view.
#[test]
fn d2_generator_open_does_not_admit() {
    let (_dir, store) = store();
    let ledger = Ledger::new(&store);

    let id = ledger
        .expand(
            &task(),
            vec![],
            contrast(&store),
            "the retry budget is the dimension this contrast clarified",
            scope(),
            vec![],
            vec![],
            Source::Generator,
            "2026-01-01T00:00:00Z",
        )
        .expect("expand");

    // A Generator (and a Rules) Open on a Generator-origin preference is recorded, not admitted.
    for by in [Source::Generator, Source::Rules] {
        ledger
            .set_status(
                &task(),
                vec![],
                id,
                PreferenceStatus::Open,
                by,
                "2026-01-01T00:01:00Z",
            )
            .expect("recorded");

        let collective = ledger.collective().expect("fold");
        let entry = collective.get(&id).expect("in the structure");
        assert!(
            entry
                .history
                .iter()
                .any(|(s, b)| *s == PreferenceStatus::Open && *b == by),
            "the non-operator Open is recorded in the history"
        );
        assert!(
            !entry.is_active(),
            "a {by:?} Open must not admit a Generator-origin preference"
        );
        assert!(
            !ledger.active().expect("active").contains(&id),
            "a {by:?} Open must not put it in the active view"
        );
    }
}

/// D2: an Operator `StatusSet(Open)` admits a Generator-origin preference.
#[test]
fn d2_operator_open_admits() {
    let (_dir, store) = store();
    let ledger = Ledger::new(&store);

    let id = ledger
        .expand(
            &task(),
            vec![],
            contrast(&store),
            "the retry budget is the dimension this contrast clarified",
            scope(),
            vec![],
            vec![],
            Source::Generator,
            "2026-01-01T00:00:00Z",
        )
        .expect("expand");

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

/// D2: a mechanical status (`Fulfilled`, `Overlapping`) applies only to a preference already
/// admitted. On a Generator-origin preference that no Operator has opened, it is refused.
#[test]
fn d2_mechanical_status_needs_admission() {
    let (_dir, store) = store();
    let ledger = Ledger::new(&store);

    let id = ledger
        .expand(
            &task(),
            vec![],
            contrast(&store),
            "the criterion: the suite passes on the merged head",
            scope(),
            vec![],
            vec![],
            Source::Generator,
            "2026-01-01T00:00:00Z",
        )
        .expect("expand");

    // A later return arrives; the criterion would close — but the preference was never admitted.
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

    let err = ledger
        .mark_fulfilled(&task(), vec![ret.id], id, "2026-01-01T00:05:01Z")
        .expect_err("a mechanical status on an unadmitted preference is refused");
    assert!(matches!(err, LedgerError::NotActive(_)), "{err:?}");
    assert_eq!(err.exit_code(), 2);

    // After the Operator admits it, the mechanical status applies.
    ledger
        .set_status(
            &task(),
            vec![],
            id,
            PreferenceStatus::Open,
            Source::Operator,
            "2026-01-01T00:06:00Z",
        )
        .expect("operator admits it");
    ledger
        .mark_fulfilled(&task(), vec![ret.id], id, "2026-01-01T00:06:01Z")
        .expect("fulfil after admission");

    let active = ledger.active().expect("active");
    assert_eq!(
        active.get(&id).expect("entry").status,
        Some(PreferenceStatus::Fulfilled)
    );
}
