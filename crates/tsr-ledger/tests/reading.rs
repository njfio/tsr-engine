//! Spec #10: a single reading each cycle. Its want crosses with no reviewer and stays out of the
//! preference view; returns come back beside it with no verdict.

use std::sync::atomic::{AtomicU64, Ordering};

use tsr_domain::{
    Event, EventId, EventKind, ExpectedReturn, Hash, PreferenceStatus, Reading, ReadingReturn,
    Scope, Source, TaskId, READING_TEXT_MAX,
};
use tsr_ledger::{bounded_readings, reading_view, Ledger, LedgerError};
use tsr_store::Store;

const AT: &str = "2026-11-04T00:00:00Z";

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Store::open(&dir.path().join("events.jsonl")).expect("open store");
    (dir, store)
}

fn task() -> TaskId {
    TaskId(Hash::of(&[b"t-reading"]))
}

fn wanting(want: &str) -> Reading {
    Reading {
        want: Some(want.to_string()),
        sense: Some("the jump feels late".to_string()),
        belief: None,
        expect: None,
        expected_return: Some(ExpectedReturn {
            from: "the operator".to_string(),
            what: "says it is harder".to_string(),
        }),
        falsifier: Some("the operator says it is still too easy".to_string()),
        gap: None,
    }
}

fn blank() -> Reading {
    Reading {
        want: None,
        sense: None,
        belief: None,
        expect: None,
        expected_return: None,
        falsifier: None,
        gap: None,
    }
}

/// Every call is a new event: the store is idempotent by identity, which includes the time.
fn record(store: &Store, reading: Reading) -> EventId {
    static CLOCK: AtomicU64 = AtomicU64::new(0);
    let at = format!(
        "2026-11-04T00:00:{:02}Z",
        CLOCK.fetch_add(1, Ordering::Relaxed) % 60
    );
    let at = format!("{at}-{}", CLOCK.load(Ordering::Relaxed));
    Ledger::new(store)
        .record_reading(&task(), vec![], reading, at)
        .expect("record reading")
}

fn view(store: &Store) -> tsr_ledger::ReadingView {
    reading_view(&store.all().expect("events"), &task())
}

fn operator_return(reading: Option<EventId>, text: &str) -> ReadingReturn {
    ReadingReturn {
        reading,
        text: text.to_string(),
        evidence: None,
    }
}

#[test]
fn spec_c02_want_crosses_with_no_reviewer_and_no_preference() {
    let (_d, s) = store();
    let before = Ledger::new(&s).collective().unwrap().entries.len();
    let id = record(&s, wanting("a harder second level"));
    let v = view(&s);
    let current = v.current_want.expect("current want");
    assert_eq!(current.text, "a harder second level");
    assert_eq!(current.reading, id);
    assert_eq!(current.origin, Source::Generator);
    let events = s.all().unwrap();
    assert!(events.iter().all(|(_, e)| e.source != Source::Council));
    assert_eq!(events.last().unwrap().1.source, Source::Rules);
    assert_eq!(Ledger::new(&s).collective().unwrap().entries.len(), before);
}

#[test]
fn spec_c03_only_the_latest_want_is_current_and_restatement_is_counted() {
    let (_d, s) = store();
    record(&s, wanting("want A"));
    record(&s, wanting("want B"));
    record(&s, wanting("  WANT   b "));
    let v = view(&s);
    assert_eq!(v.current_want.as_ref().unwrap().text, "  WANT   b ");
    assert_eq!(v.readings.len(), 3);
    assert_eq!(v.readings[0].reading.want.as_deref(), Some("want A"));
    assert_eq!(v.counts.restated, 1);
}

#[test]
fn spec_c04_blank_reading_is_valid_and_leaves_no_want_current() {
    let (_d, s) = store();
    record(&s, wanting("want A"));
    record(&s, blank());
    let v = view(&s);
    assert!(v.current_want.is_none());
    assert_eq!(v.counts.readings, 2);
    assert_eq!(v.counts.blank, 1);
}

#[test]
fn spec_c04_a_reading_with_any_one_direction_is_not_blank() {
    let some = || Some("something".to_string());
    assert!(blank().is_blank());
    assert!(!wanting("a harder second level").is_blank());
    for reading in [
        Reading {
            sense: some(),
            ..blank()
        },
        Reading {
            belief: some(),
            ..blank()
        },
        Reading {
            expect: some(),
            ..blank()
        },
    ] {
        assert!(!reading.is_blank(), "{reading:?}");
        reading.validate().unwrap();
    }
}

#[test]
fn spec_c05_want_needs_an_expected_return_and_a_falsifier() {
    let (_d, s) = store();
    let mut no_falsifier = wanting("want A");
    no_falsifier.falsifier = None;
    let mut no_return = wanting("want A");
    no_return.expected_return = None;
    for bad in [no_falsifier, no_return] {
        assert!(matches!(
            Ledger::new(&s).record_reading(&task(), vec![], bad, AT),
            Err(LedgerError::Contract(_))
        ));
    }
    assert!(s.all().unwrap().is_empty());
}

#[test]
fn spec_c06_unbounded_or_blank_text_is_refused_and_nothing_is_appended() {
    let (_d, s) = store();
    let long = "x".repeat(READING_TEXT_MAX + 1);
    let mut over = wanting("want A");
    over.sense = Some(long.clone());
    let mut empty = wanting("want A");
    empty.belief = Some("  ".to_string());
    let mut expected_only = blank();
    expected_only.expected_return = wanting("x").expected_return;
    let mut falsifier_only = blank();
    falsifier_only.falsifier = Some("something".to_string());
    let mut long_from = wanting("want A");
    long_from.expected_return.as_mut().unwrap().from = long;
    for bad in [over, empty, expected_only, falsifier_only, long_from] {
        assert!(Ledger::new(&s)
            .record_reading(&task(), vec![], bad, AT)
            .is_err());
    }
    assert!(s.all().unwrap().is_empty());
    let at_limit = Reading {
        sense: Some("x".repeat(READING_TEXT_MAX)),
        ..wanting("want A")
    };
    assert!(Ledger::new(&s)
        .record_reading(&task(), vec![], at_limit, AT)
        .is_ok());
}

#[test]
fn reading_deserialization_refuses_unknown_fields() {
    let forged = r#"{"want":null,"sense":null,"belief":null,"expect":null,"expected_return":null,"falsifier":null,"scope":"other"}"#;
    assert!(serde_json::from_str::<Reading>(forged).is_err());
}

#[test]
fn spec_c07_returns_come_from_operator_or_rules_only() {
    let (_d, s) = store();
    let reading = record(&s, wanting("want A"));
    let l = Ledger::new(&s);
    l.record_return(
        &task(),
        Source::Operator,
        operator_return(None, "waaaay too easy"),
        AT,
    )
    .unwrap();
    l.record_return(
        &task(),
        Source::Rules,
        ReadingReturn {
            reading: Some(reading),
            text: "tests failed".to_string(),
            evidence: Some(Hash::of(&[b"receipt"])),
        },
        AT,
    )
    .unwrap();
    let count = s.all().unwrap().len();
    for source in [Source::Generator, Source::Council, Source::Intuition] {
        assert!(l
            .record_return(&task(), source, operator_return(None, "generated"), AT)
            .is_err());
    }
    assert_eq!(s.all().unwrap().len(), count);
    let v = view(&s);
    assert_eq!(v.unattached_returns.len(), 1);
    assert_eq!(v.unattached_returns[0].from, Source::Operator);
    assert_eq!(v.readings[0].returns.len(), 1);
    assert_eq!(v.counts.returns_named, 1);
    assert_eq!(v.counts.returns_unnamed, 1);
}

#[test]
fn return_refuses_blank_unbounded_or_foreign_reading() {
    let (_d, s) = store();
    let l = Ledger::new(&s);
    let other = TaskId(Hash::of(&[b"t-other"]));
    let foreign = l
        .record_reading(&other, vec![], wanting("foreign"), AT)
        .unwrap();
    let count = s.all().unwrap().len();
    for bad in [
        operator_return(None, "   "),
        operator_return(None, &"x".repeat(READING_TEXT_MAX + 1)),
        operator_return(Some(foreign), "wrong task"),
        operator_return(Some(EventId(Hash::of(&[b"missing"]))), "no such reading"),
    ] {
        assert!(l.record_return(&task(), Source::Operator, bad, AT).is_err());
    }
    assert_eq!(s.all().unwrap().len(), count);
}

#[test]
fn counts_a_want_changed_after_a_return_only_when_a_return_came_between() {
    let (_d, s) = store();
    let l = Ledger::new(&s);
    record(&s, wanting("want A"));
    record(&s, wanting("want B"));
    assert_eq!(view(&s).counts.changed_after_return, 0);
    l.record_return(
        &task(),
        Source::Operator,
        operator_return(None, "first"),
        AT,
    )
    .unwrap();
    record(&s, wanting("want B"));
    assert_eq!(view(&s).counts.changed_after_return, 0);
    l.record_return(
        &task(),
        Source::Operator,
        operator_return(None, "second"),
        AT,
    )
    .unwrap();
    record(&s, wanting("want C"));
    assert_eq!(view(&s).counts.changed_after_return, 1);
}

#[test]
fn spec_c08_bounded_view_keeps_latest_states_omitted_and_gives_no_verdict() {
    let (_d, s) = store();
    let l = Ledger::new(&s);
    for i in 0..5 {
        let id = record(&s, wanting(&format!("want {i}")));
        l.record_return(
            &task(),
            Source::Operator,
            operator_return(Some(id), &format!("return {i}")),
            AT,
        )
        .unwrap();
    }
    let bounded = bounded_readings(&view(&s), 2);
    assert_eq!(bounded.omitted, 3);
    assert_eq!(bounded.readings.len(), 2);
    assert_eq!(bounded.readings[0].reading.want.as_deref(), Some("want 3"));
    assert_eq!(bounded.readings[1].returns[0].text, "return 4");
    assert_eq!(bounded.current_want.as_ref().unwrap().text, "want 4");
    assert_eq!(view(&s).readings.len(), 5);
    let text = serde_json::to_string(&bounded).unwrap();
    for word in ["verdict", "falsified", "correct", "score"] {
        assert!(!text.contains(word), "{word}");
    }
}

#[test]
fn bounded_view_omits_old_unattached_returns_with_a_count() {
    let (_d, s) = store();
    let l = Ledger::new(&s);
    for i in 0..4 {
        l.record_return(
            &task(),
            Source::Operator,
            operator_return(None, &format!("return {i}")),
            AT,
        )
        .unwrap();
    }
    let bounded = bounded_readings(&view(&s), 3);
    assert_eq!(bounded.omitted_returns, 1);
    assert_eq!(bounded.unattached_returns[0].text, "return 1");
}

#[test]
fn bounded_view_bounds_returns_on_a_reading_and_counts_them() {
    let (_d, s) = store();
    let l = Ledger::new(&s);
    let id = record(&s, wanting("want A"));
    for i in 0..3 {
        l.record_return(
            &task(),
            Source::Rules,
            operator_return(Some(id), &format!("return {i}")),
            AT,
        )
        .unwrap();
    }
    let bounded = bounded_readings(&view(&s), 2);
    assert_eq!(bounded.omitted_returns, 1);
    assert_eq!(bounded.readings[0].returns[0].text, "return 1");
}

#[test]
fn spec_c10_want_never_becomes_an_open_preference_or_aim() {
    let (_d, s) = store();
    record(&s, wanting("want A"));
    let c = Ledger::new(&s).collective().unwrap();
    assert!(c.entries.is_empty());
    assert!(c.open().is_empty());
}

#[test]
fn spec_c11_logs_without_readings_have_an_empty_view() {
    let (_d, s) = store();
    let v = view(&s);
    assert!(v.current_want.is_none());
    assert!(v.readings.is_empty());
    assert_eq!(v.counts.readings, 0);
}

#[test]
fn spec_c12_forged_events_admit_nothing() {
    let (_d, s) = store();
    let genuine = record(&s, wanting("genuine"));
    // A reading from any source but the controller's rules.
    let forged = Event::new(
        task(),
        vec![],
        Source::Generator,
        EventKind::Reading {
            reading: wanting("forged"),
        },
        AT,
    );
    s.append(&forged).unwrap();
    // A malformed reading written straight to the log.
    let malformed = Event::new(
        task(),
        vec![],
        Source::Rules,
        EventKind::Reading {
            reading: Reading {
                falsifier: None,
                gap: None,
                ..wanting("malformed")
            },
        },
        AT,
    );
    s.append(&malformed).unwrap();
    // A return from the generator, naming the genuine reading.
    let generated = Event::new(
        task(),
        vec![],
        Source::Generator,
        EventKind::Returned {
            returned: operator_return(Some(genuine), "forged return"),
        },
        AT,
    );
    s.append(&generated).unwrap();
    // A generated status over nothing the view holds.
    let status = Event::new(
        task(),
        vec![],
        Source::Generator,
        EventKind::StatusSet {
            preference: tsr_domain::Preference::from_admitted_text(
                "genuine",
                Scope::Task(task()),
                vec![],
                Source::Generator,
            )
            .unwrap()
            .id,
            status: PreferenceStatus::Open,
            by: Source::Generator,
        },
        AT,
    );
    s.append(&status).unwrap();
    let v = view(&s);
    assert_eq!(v.current_want.unwrap().text, "genuine");
    assert_eq!(v.readings.len(), 1);
    assert!(v.readings[0].returns.is_empty());
    assert!(v.unattached_returns.is_empty());
    assert!(Ledger::new(&s).collective().unwrap().entries.is_empty());
}

#[test]
fn return_chains_to_the_tasks_latest_event() {
    let (_d, s) = store();
    let id = record(&s, wanting("want A"));
    let returned = Ledger::new(&s)
        .record_return(&task(), Source::Operator, operator_return(None, "no"), AT)
        .unwrap();
    let events = s.all().unwrap();
    let event = events.iter().find(|(_, e)| e.id == returned).unwrap();
    assert_eq!(event.1.parents, vec![id]);
}

#[test]
fn view_ignores_other_tasks() {
    let (_d, s) = store();
    let other = TaskId(Hash::of(&[b"t-other"]));
    Ledger::new(&s)
        .record_reading(&other, vec![], wanting("foreign"), AT)
        .unwrap();
    assert!(view(&s).current_want.is_none());
    assert_eq!(view(&s).counts.readings, 0);
}

#[test]
fn spec_c13_reopened_store_gives_the_same_view() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("events.jsonl");
    let before = {
        let s = Store::open(&path).unwrap();
        let l = Ledger::new(&s);
        let id = l
            .record_reading(&task(), vec![], wanting("want A"), AT)
            .unwrap();
        l.record_return(
            &task(),
            Source::Operator,
            operator_return(Some(id), "no"),
            AT,
        )
        .unwrap();
        // The same event again is one event: the store is idempotent by identity.
        l.record_reading(&task(), vec![], wanting("want A"), AT)
            .unwrap();
        view(&s)
    };
    let s = Store::open(&path).unwrap();
    assert_eq!(view(&s), before);
    assert_eq!(before.counts.readings, 1);
}

#[test]
fn randomized_text_never_panics_and_validation_decides_every_append() {
    // Tier: property, without a new dependency. A small deterministic generator; 20k cases.
    let (_d, s) = store();
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    let mut next = || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let text = |n: u64| -> Option<String> {
        match n % 4 {
            0 => None,
            1 => Some(" \t\n".to_string()),
            2 => Some("é".repeat((n % 3000) as usize)),
            _ => Some("wanting".to_string()),
        }
    };
    let mut accepted = 0_u64;
    for _ in 0..20_000 {
        let r = Reading {
            want: text(next()),
            sense: text(next()),
            belief: text(next()),
            expect: text(next()),
            expected_return: text(next()).map(|what| ExpectedReturn {
                from: "someone".to_string(),
                what,
            }),
            falsifier: text(next()),
            gap: None,
        };
        let valid = r.validate().is_ok();
        let fields = [&r.want, &r.sense, &r.belief, &r.expect, &r.falsifier];
        let within = fields.iter().all(|f| {
            f.as_ref()
                .is_none_or(|t| !t.trim().is_empty() && t.len() <= READING_TEXT_MAX)
        });
        if r.want.is_some() && (r.falsifier.is_none() || r.expected_return.is_none()) {
            assert!(!valid);
        }
        if !within {
            assert!(!valid);
        }
        if valid {
            accepted += 1;
        }
    }
    assert!(accepted > 0);
    assert!(s.all().unwrap().is_empty());
}

/// Spec #12: `gap` is optional, bounded like the other fields, and absent from the serialized
/// form when unset, so a reading without it keeps the identity it had before the field existed.
#[test]
fn spec12_gap_is_optional_bounded_and_identity_preserving() {
    let without = wanting("a harder second level");
    let text = serde_json::to_string(&without).unwrap();
    assert!(!text.contains("gap"), "{text}");
    let with = Reading {
        gap: Some("the want is far from what the game does now".to_string()),
        ..wanting("a harder second level")
    };
    with.validate().unwrap();
    assert!(serde_json::to_string(&with).unwrap().contains("\"gap\""));
    let blank_gap = Reading {
        gap: Some("   ".to_string()),
        ..wanting("a harder second level")
    };
    assert!(blank_gap.validate().is_err());
    let parsed: Reading = serde_json::from_str(&text).unwrap();
    assert_eq!(parsed, without);
}
