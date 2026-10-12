//! The Filter's required behaviours (PRD §B2.1, §B6, §B11 M2 item 2), each as a test over a real
//! store: from one return, two separate records; only Contrast expands.

use tsr_domain::{
    CriterionId, Event, EventId, EventKind, Forecast, ForecastId, Forecaster, Hash, Horizon,
    Observation, OutcomeDefId, Probability, Scope, Source, Support, TaskId, UnmetKind,
};
use tsr_ledger::{dimension_name, Filter, Ledger};
use tsr_store::Store;

fn store() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().expect("tempdir");
    let store = Store::open(&dir.path().join("events.jsonl")).expect("open store");
    (dir, store)
}

fn task() -> TaskId {
    TaskId(Hash::of(&[b"t-filter"]))
}

fn scope() -> Scope {
    Scope::Task(task())
}

fn criterion(name: &str) -> CriterionId {
    CriterionId(Hash::of(&[name.as_bytes()]))
}

fn forecast(id: &str, p: f64) -> Forecast {
    Forecast {
        id: ForecastId(Hash::of(&[id.as_bytes()])),
        event: OutcomeDefId(Hash::of(&[b"task_accepted_in_budget@v1"])),
        horizon: Horizon::TaskClose,
        source: Forecaster::Generator,
        raw: Probability::new(p).expect("probability"),
        calibrated: None,
        support: Support { episodes: 3 },
        state_version: 1,
    }
}

/// A return event to hang the contrast's `from_return` on.
fn a_return(store: &Store) -> EventId {
    let ev = Event::new(
        task(),
        vec![],
        Source::Rules,
        EventKind::Observed {
            observation: Observation::MissingEvidence {
                what: "the return".to_string(),
            },
        },
        "2026-01-01T00:00:00Z",
    );
    store.append(&ev).expect("append return");
    ev.id
}

/// Record a Contrast through the Filter, so `Ledger::expand` finds it in the log (D3: expansion
/// refuses a contrast id with no Contrast event). Returns the `Observed` event's id.
fn record_contrast(store: &Store, contrast: &tsr_domain::Contrast, at: &str) -> EventId {
    Filter::new(store)
        .record_return(&task(), vec![], vec![], Some(contrast.clone()), at)
        .expect("record the contrast")
}

fn expanded_count(store: &Store) -> usize {
    store
        .all()
        .expect("all")
        .iter()
        .filter(|(_, e)| matches!(e.kind, EventKind::Expanded { .. }))
        .count()
}

/// (a) A return with only Surprise appends no `Expanded` event.
#[test]
fn surprise_alone_never_expands() {
    let (_dir, store) = store();
    let filter = Filter::new(&store);

    let surprise =
        Filter::surprise_from_forecasts(&[forecast("f-1", 0.8), forecast("f-2", 0.3)], 0.0);
    assert_eq!(surprise.len(), 2, "one residual per forecaster");
    // D11: forecast minus observed, so a forecast of 0.8 against outcome 0 is +0.8.
    assert!((surprise[0].residual - 0.8).abs() < 1e-12);
    assert!((surprise[1].residual - 0.3).abs() < 1e-12);

    let (observed, produced) = filter
        .filter_return(
            &task(),
            vec![],
            surprise.clone(),
            None,
            scope(),
            vec![],
            vec![],
            Source::Generator,
            "2026-01-01T00:01:00Z",
        )
        .expect("filter the return");

    assert!(produced.is_none(), "a Surprise never produces an Expansion");
    assert_eq!(expanded_count(&store), 0, "no Expanded event was appended");

    // The Surprise is recorded, as an Observed event carrying the domain's Surprise.
    let events = store.all().expect("all");
    let ev = events
        .iter()
        .find(|(_, e)| e.id == observed)
        .map(|(_, e)| e)
        .expect("the observed event");
    match &ev.kind {
        EventKind::Observed {
            observation:
                Observation::Return {
                    surprise: s,
                    contrast,
                },
        } => {
            assert_eq!(s, &surprise);
            assert!(contrast.is_none());
        }
        other => panic!("expected an Observed return, got {other:?}"),
    }
}

/// (b) A Contrast with an unmet criterion appends `Expanded` with lineage and referenced evidence.
#[test]
fn contrast_with_an_unmet_criterion_expands_with_lineage() {
    let (_dir, store) = store();
    let filter = Filter::new(&store);
    let ledger = Ledger::new(&store);

    let ret = a_return(&store);
    let evidence = vec![ret];
    let contrast = Filter::contrast_from_criteria(
        ret,
        vec![(criterion("c-reliability"), UnmetKind::WrongBehaviour)],
        evidence.clone(),
    );
    // D3: the contrast must exist in the log before `expand` will accept its id.
    record_contrast(&store, &contrast, "2026-01-01T00:00:20Z");

    // A preference that made the contrast matter (catalyzed_by) and a lineage parent.
    let parent = ledger
        .expand(
            &task(),
            vec![],
            contrast.id,
            "the task must not regress span offsets",
            scope(),
            vec![],
            vec![],
            Source::Operator,
            "2026-01-01T00:00:30Z",
        )
        .expect("seed a preference");

    let produced = filter
        .expand_from_contrast(
            &task(),
            vec![ret],
            &contrast,
            scope(),
            vec![parent],
            vec![parent],
            Source::Generator,
            "2026-01-01T00:02:00Z",
        )
        .expect("expand")
        .expect("a contrast with an unmet criterion expands");

    assert_eq!(
        expanded_count(&store),
        2,
        "the seed plus the contrast's expansion"
    );

    let view = ledger.collective().expect("fold");
    let entry = view.get(&produced).expect("the produced preference");
    assert_eq!(
        entry.from_contrast,
        Some(contrast.id),
        "lineage: from_contrast"
    );
    assert_eq!(entry.catalyzed_by, vec![parent], "lineage: catalyzed_by");
    assert_eq!(
        entry.preference.parents,
        vec![parent],
        "lineage: expanded-from parents"
    );
    assert_eq!(entry.preference.origin, Source::Generator);
}

/// (c) The produced preference names the dimension and does not contain the failed strategy's text.
#[test]
fn produced_preference_names_the_dimension_not_the_strategy() {
    let (_dir, store) = store();
    let filter = Filter::new(&store);

    let ret = a_return(&store);
    let failed_strategy = "rewrite the parser to use a regex over the whole file";
    let contrast = Filter::contrast_from_criteria(
        ret,
        vec![
            (criterion("c-behaviour"), UnmetKind::WrongBehaviour),
            (criterion("c-regression"), UnmetKind::Regression),
            (criterion("c-cost"), UnmetKind::Cost),
            (
                criterion("c-rejected"),
                UnmetKind::Rejected {
                    reason: "regresses span offsets".to_string(),
                },
            ),
        ],
        vec![ret],
    );
    // D3: the contrast must exist in the log before `expand` will accept its id.
    record_contrast(&store, &contrast, "2026-01-01T00:02:30Z");

    let produced = filter
        .expand_from_contrast(
            &task(),
            vec![ret],
            &contrast,
            scope(),
            vec![],
            vec![],
            Source::Generator,
            "2026-01-01T00:03:00Z",
        )
        .expect("expand")
        .expect("expands");

    let view = Ledger::new(&store).collective().expect("fold");
    let text = &view.get(&produced).expect("produced").preference.text;

    for dimension in ["reliability", "regression", "cost", "rejection"] {
        assert!(
            text.contains(dimension),
            "the preference names the {dimension} dimension: {text}"
        );
    }
    assert!(
        !text.contains(failed_strategy),
        "the failed strategy's content is never copied into the preference: {text}"
    );
    assert!(
        !text.contains("regex"),
        "no fragment of the failed strategy leaks: {text}"
    );
    // The dimension names come from the UnmetKind, not from the strategy.
    assert_eq!(dimension_name(&UnmetKind::WrongBehaviour), "reliability");
    assert_eq!(dimension_name(&UnmetKind::Cost), "cost");
}

/// (d) Evidence ids appear in the Contrast, not copied into the preference.
#[test]
fn evidence_is_referenced_never_copied() {
    let (_dir, store) = store();
    let filter = Filter::new(&store);

    let ret = a_return(&store);
    let log_digest = Hash::of(&[b"the tool log"]);
    let evidence = vec![ret];
    let contrast = Filter::contrast_from_criteria(
        ret,
        vec![(criterion("c-cost"), UnmetKind::Cost)],
        evidence.clone(),
    );
    // D3: the contrast must exist in the log before `expand` will accept its id.
    record_contrast(&store, &contrast, "2026-01-01T00:03:30Z");

    let produced = filter
        .expand_from_contrast(
            &task(),
            vec![ret],
            &contrast,
            scope(),
            vec![],
            vec![],
            Source::Generator,
            "2026-01-01T00:04:00Z",
        )
        .expect("expand")
        .expect("expands");

    // The evidence ids are on the contrast record.
    assert_eq!(contrast.evidence, evidence);
    assert_eq!(contrast.from_return, ret);

    // The preference text carries no event id and no digest.
    let view = Ledger::new(&store).collective().expect("fold");
    let text = &view.get(&produced).expect("produced").preference.text;
    assert!(
        !text.contains(&ret.to_string()),
        "the evidence event id is referenced, not copied: {text}"
    );
    assert!(
        !text.contains(&log_digest.to_string()),
        "no digest is copied into the preference: {text}"
    );
}

/// A contrast with no unmet criterion does not expand either.
#[test]
fn contrast_without_unmet_criteria_does_not_expand() {
    let (_dir, store) = store();
    let filter = Filter::new(&store);

    let ret = a_return(&store);
    let contrast = Filter::contrast_from_criteria(ret, vec![], vec![ret]);

    let produced = filter
        .expand_from_contrast(
            &task(),
            vec![ret],
            &contrast,
            scope(),
            vec![],
            vec![],
            Source::Generator,
            "2026-01-01T00:05:00Z",
        )
        .expect("no error");

    assert!(produced.is_none(), "nothing unmet, nothing crosses");
    assert_eq!(expanded_count(&store), 0);
}

/// The two records are kept apart: a return can carry both, and only the contrast expands.
#[test]
fn both_records_are_recorded_apart_and_only_contrast_expands() {
    let (_dir, store) = store();
    let filter = Filter::new(&store);

    let ret = a_return(&store);
    let surprise = Filter::surprise_from_forecasts(&[forecast("f-1", 0.9)], 1.0);
    let contrast = Filter::contrast_from_criteria(
        ret,
        vec![(criterion("c-behaviour"), UnmetKind::WrongBehaviour)],
        vec![ret],
    );

    let (observed, produced) = filter
        .filter_return(
            &task(),
            vec![],
            surprise.clone(),
            Some(contrast.clone()),
            scope(),
            vec![],
            vec![],
            Source::Generator,
            "2026-01-01T00:06:00Z",
        )
        .expect("filter");

    assert!(produced.is_some(), "the contrast expands");
    assert_eq!(expanded_count(&store), 1);

    let events = store.all().expect("all");
    let ev = events
        .iter()
        .find(|(_, e)| e.id == observed)
        .map(|(_, e)| e)
        .expect("observed");
    match &ev.kind {
        EventKind::Observed {
            observation:
                Observation::Return {
                    surprise: s,
                    contrast: c,
                },
        } => {
            assert_eq!(s, &surprise, "the Surprise is recorded beside the Contrast");
            assert_eq!(c.as_ref(), Some(&contrast));
        }
        other => panic!("expected an Observed return, got {other:?}"),
    }
}
