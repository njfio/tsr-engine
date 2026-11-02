//! Authored native fixtures; no provider calls or claimed user preferences.
use tsr_domain::*;
use tsr_ledger::{fold, reviewed, Filter, Ledger};
use tsr_store::Store;
fn task() -> TaskId {
    TaskId(Hash::of(&[b"reviewed-ledger-fixture"]))
}
fn note(s: &Store, source: Source, text: &str) -> EventId {
    let e = Event::new(
        task(),
        vec![],
        source,
        EventKind::Observed {
            observation: Observation::Evidence {
                source: text.into(),
                digest: Hash::of(&[text.as_bytes()]),
            },
        },
        text,
    );
    s.append(&e).unwrap();
    e.id
}
fn setup(long: bool) -> (tempfile::TempDir, Store, PreferenceAdmissionReview, EventId) {
    let d = tempfile::tempdir().unwrap();
    let s = Store::open(&d.path().join("events.jsonl")).unwrap();
    let ret = note(&s, Source::Rules, "actual fixture return");
    let c = Filter::contrast_from_criteria(
        ret,
        vec![(
            CriterionId(Hash::of(&[b"criterion"])),
            UnmetKind::WrongBehaviour,
        )],
        vec![ret],
    );
    let l = Ledger::new(&s);
    l.record_contrast(&task(), vec![ret], c.clone(), "contrast")
        .unwrap();
    let parent = l
        .expand(
            &task(),
            vec![],
            c.id,
            "original retained want",
            Scope::Task(task()),
            vec![],
            vec![],
            Source::Operator,
            "parent",
        )
        .unwrap();
    let p = Preference::from_admitted_text(
        if long {
            "A reviewed generated meaning. ".repeat(30)
        } else {
            "A reviewed generated meaning".into()
        },
        Scope::Task(task()),
        vec![parent],
        Source::Generator,
    )
    .unwrap();
    let expansion = l
        .propose_generated(
            &task(),
            vec![],
            Expansion {
                from_contrast: c.id,
                produced: p.id,
                catalyzed_by: p.parents.clone(),
                text: p.text,
                scope: p.scope,
                parents: p.parents,
                origin: Source::Generator,
            },
            "expansion",
        )
        .unwrap();
    let r = PreferenceAdmissionReview {
        expansion,
        preference: p.id,
        contrast: c.id,
        from_return: ret,
        proposal: note(&s, Source::Generator, "proposal"),
        source_snapshot: Hash::of(&[b"source"]),
        requirements_revision: note(&s, Source::Operator, "requirements"),
        request_binding: Hash::of(&[b"request"]),
        response_digest: Hash::of(&[b"response"]),
        reviewer: "authored independent reviewer".into(),
        generator: "authored generator".into(),
        decision: "approve".into(),
    };
    let evidence = evidence(&s, &r, Source::Council);
    (d, s, r, evidence)
}
fn evidence(s: &Store, r: &PreferenceAdmissionReview, source: Source) -> EventId {
    let e = Event::new(
        task(),
        vec![r.expansion],
        source,
        EventKind::Observed {
            observation: Observation::Evidence {
                source: reviewed::REVIEW_SOURCE.into(),
                digest: Hash::of(&[&serde_json::to_vec(r).unwrap()]),
            },
        },
        "review",
    );
    s.append(&e).unwrap();
    e.id
}
#[test]
fn short_and_extended_are_real_selectable_ledger_nodes_after_review_and_restart() {
    for long in [false, true] {
        let (d, s, r, e) = setup(long);
        let l = Ledger::new(&s);
        assert!(!l.active().unwrap().contains(&r.preference));
        l.admit_reviewed(&task(), s.head_root().unwrap(), r.clone(), e, "admit")
            .unwrap();
        let v = l.collective().unwrap();
        let node = v.get(&r.preference).unwrap();
        assert!(node.is_open());
        assert_eq!(node.preference.origin, Source::Generator);
        assert_eq!(
            node.history,
            vec![(PreferenceStatus::Open, Source::Council)]
        );
        assert_eq!(v.entries.len(), 2);
        assert!(v.entries[0].is_open());
        let snap = Snapshot {
            task: task(),
            version: 0,
            root: v.root,
            open_criteria: vec![],
            missing_evidence: vec![],
        };
        let tune = Tune {
            admitted_sources: vec![Source::Generator],
            quiet_route: 1.0,
            support_floor: 1,
            flow_horizon: 0,
            ..Tune::default()
        };
        let ctx = tsr_ledger::selection::SelectionContext {
            family: "fixture".into(),
            repository: "fixture".into(),
            minimum_depth: 1,
        };
        assert_eq!(
            tsr_ledger::selection::select(&v, &snap, &tune, &ctx, 0)
                .unwrap()
                .selected,
            vec![r.preference]
        );
        drop(s);
        let reopened = Store::open(&d.path().join("events.jsonl")).unwrap();
        assert_eq!(Ledger::new(&reopened).collective().unwrap(), v);
    }
}
#[test]
fn stale_rejection_identity_digest_and_source_substitutions_refused() {
    let (_d, s, r, e) = setup(true);
    let l = Ledger::new(&s);
    let head = s.head_root().unwrap();
    assert!(l
        .admit_reviewed(&task(), Hash([0; 32]), r.clone(), e, "x")
        .is_err());
    for bad in 0..8 {
        let mut x = r.clone();
        match bad {
            0 => x.decision = "reject".into(),
            1 => x.reviewer = x.generator.clone(),
            2 => x.preference = PreferenceId(Hash([0; 32])),
            3 => x.contrast = ContrastId(Hash([0; 32])),
            4 => x.source_snapshot = Hash([0; 32]),
            5 => x.requirements_revision = EventId(Hash([0; 32])),
            6 => x.response_digest = Hash([0; 32]),
            _ => x.from_return = EventId(Hash([0; 32])),
        };
        assert!(l.admit_reviewed(&task(), head, x, e, "x").is_err());
    }
    let forged = evidence(&s, &r, Source::Generator);
    assert!(l
        .admit_reviewed(&task(), s.head_root().unwrap(), r.clone(), forged, "x")
        .is_err());
    assert!(l
        .admit_reviewed(
            &TaskId(Hash([0; 32])),
            s.head_root().unwrap(),
            r.clone(),
            e,
            "x"
        )
        .is_err());
    assert!(!l.active().unwrap().contains(&r.preference));
}
#[test]
fn raw_future_or_forged_admission_does_not_bypass_fold() {
    let (_d, s, r, e) = setup(true);
    let events = s.all().unwrap();
    let root = s.head_root().unwrap();
    for source in [Source::Generator, Source::Operator] {
        let mut raw = events.clone();
        raw.push((
            999,
            Event::new(
                task(),
                vec![r.expansion, e],
                source,
                EventKind::ReviewedPreferenceAdmission {
                    review: Box::new(r.clone()),
                    evidence: e,
                },
                "forged",
            ),
        ));
        assert!(!fold(&raw, root).active().contains(&r.preference));
    }
    let raw = Event::new(
        task(),
        vec![r.expansion, e],
        Source::Council,
        EventKind::ReviewedPreferenceAdmission {
            review: Box::new(r.clone()),
            evidence: e,
        },
        "future",
    );
    let mut reordered = vec![(0, raw)];
    reordered.extend(events);
    assert!(!fold(&reordered, root).active().contains(&r.preference));
}
#[test]
fn duplicate_admission_never_reopens_withdrawn_identity() {
    let (_d, s, r, e) = setup(true);
    let l = Ledger::new(&s);
    l.admit_reviewed(&task(), s.head_root().unwrap(), r.clone(), e, "admit")
        .unwrap();
    l.set_status(
        &task(),
        vec![],
        r.preference,
        PreferenceStatus::Withdrawn,
        Source::Operator,
        "operator cancellation",
    )
    .unwrap();
    assert!(l
        .admit_reviewed(&task(), s.head_root().unwrap(), r.clone(), e, "retry")
        .is_err());
    let raw = Event::new(
        task(),
        vec![r.expansion, e],
        Source::Council,
        EventKind::ReviewedPreferenceAdmission {
            review: Box::new(r.clone()),
            evidence: e,
        },
        "raw replay",
    );
    s.append(&raw).unwrap();
    assert_eq!(
        l.collective().unwrap().get(&r.preference).unwrap().status,
        Some(PreferenceStatus::Withdrawn)
    );
}
#[test]
fn interrupted_transaction_rolls_back_admission_and_restart_can_retry_once() {
    let (d, s, r, e) = setup(true);
    let before = s.head_root().unwrap();
    s.connection().execute_batch("BEGIN IMMEDIATE").unwrap();
    Ledger::new(&s)
        .admit_reviewed(&task(), before, r.clone(), e, "admit")
        .unwrap();
    s.connection().execute_batch("ROLLBACK").unwrap();
    assert_eq!(s.head_root().unwrap(), before);
    drop(s);
    let s = Store::open(&d.path().join("events.jsonl")).unwrap();
    let l = Ledger::new(&s);
    l.admit_reviewed(&task(), before, r.clone(), e, "admit")
        .unwrap();
    assert!(l.active().unwrap().contains(&r.preference));
    assert!(l
        .admit_reviewed(&task(), s.head_root().unwrap(), r, e, "again")
        .is_err());
}
#[test]
fn invalid_prefix_does_not_poison_later_valid_admission() {
    let (_d, s, r, e) = setup(true);
    let raw = Event::new(
        task(),
        vec![e],
        Source::Generator,
        EventKind::ReviewedPreferenceAdmission {
            review: Box::new(r.clone()),
            evidence: e,
        },
        "invalid prefix",
    );
    s.append(&raw).unwrap();
    let l = Ledger::new(&s);
    l.admit_reviewed(&task(), s.head_root().unwrap(), r.clone(), e, "valid")
        .unwrap();
    assert!(l.active().unwrap().contains(&r.preference));
}
#[test]
fn origin_collision_and_cross_task_staging_are_rejected() {
    let (_d, s, r, e) = setup(false);
    let mut events = s.all().unwrap();
    let index = events
        .iter()
        .position(|(_, ev)| ev.id == r.expansion)
        .unwrap();
    let EventKind::Expanded { expansion: mut x } = events[index].1.kind.clone() else {
        unreachable!()
    };
    assert!(Ledger::new(&s)
        .propose_generated(&TaskId(Hash([0; 32])), vec![], x.clone(), "foreign")
        .is_err());
    x.origin = Source::Rules;
    let collision = Event::new(
        task(),
        vec![],
        Source::Rules,
        EventKind::Expanded { expansion: x },
        "collision",
    );
    events.insert(index, (0, collision));
    let fresh = Store::open_in_memory().unwrap();
    for (_, ev) in events {
        fresh.append(&ev).unwrap();
    }
    assert!(Ledger::new(&fresh)
        .admit_reviewed(
            &task(),
            fresh.head_root().unwrap(),
            r.clone(),
            e,
            "valid but conflicting"
        )
        .is_err());
    let raw = Event::new(
        task(),
        vec![e],
        Source::Council,
        EventKind::ReviewedPreferenceAdmission {
            review: Box::new(r.clone()),
            evidence: e,
        },
        "raw",
    );
    fresh.append(&raw).unwrap();
    let view = Ledger::new(&fresh).collective().unwrap();
    let entry = view.get(&r.preference).unwrap();
    assert_eq!(entry.preference.origin, Source::Rules);
    assert!(!entry.is_open());
}
#[test]
fn empty_or_altered_contrast_never_admits() {
    let (_d, s, r, e) = setup(true);
    let events = s.all().unwrap();
    for empty in [true, false] {
        let mut altered = events.clone();
        for (_, ev) in &mut altered {
            if let EventKind::Observed {
                observation:
                    Observation::Return {
                        contrast: Some(c), ..
                    },
            } = &mut ev.kind
            {
                if c.id == r.contrast {
                    if empty {
                        c.unmet.clear()
                    } else {
                        c.evidence.clear()
                    }
                }
            }
        }
        assert!(reviewed::validate(&altered, task(), &r, e).is_err());
    }
}
#[test]
fn intuition_is_not_an_authenticated_return_source() {
    let (_d, s, r, e) = setup(true);
    let mut events = s.all().unwrap();
    for (_, ev) in &mut events {
        if ev.id == r.from_return {
            ev.source = Source::Intuition;
        }
    }
    assert!(reviewed::validate(&events, task(), &r, e).is_err());
    let mut events = s.all().unwrap();
    for (_, ev) in &mut events {
        if matches!(&ev.kind,EventKind::Observed{observation:Observation::Return{contrast:Some(c),..}} if c.id==r.contrast)
        {
            ev.source = Source::Intuition;
        }
    }
    assert!(reviewed::validate(&events, task(), &r, e).is_err());
}

#[test]
fn review_and_contrast_cannot_authenticate_future_subjects() {
    let (_d, s, r, e) = setup(true);
    let mut events = s.all().unwrap();
    let ei = events.iter().position(|(_, v)| v.id == e).unwrap();
    let item = events.remove(ei);
    events.insert(0, item);
    assert!(reviewed::validate(&events, task(), &r, e).is_err());
    let mut events = s.all().unwrap();
    let ei = events
        .iter()
        .position(|(_, v)| v.id == r.expansion)
        .unwrap();
    let item = events.remove(ei);
    events.insert(0, item);
    assert!(reviewed::validate(&events, task(), &r, e).is_err());
}

#[test]
fn contrast_cannot_anticipate_its_return_evidence() {
    let (_d, s, r, e) = setup(true);
    let mut events = s.all().unwrap();
    let i = events
        .iter()
        .position(|(_, v)| v.id == r.from_return)
        .unwrap();
    let ret = events.remove(i);
    let i=events.iter().position(|(_,v)|matches!(&v.kind,EventKind::Observed{observation:Observation::Return{contrast:Some(c),..}} if c.id==r.contrast)).unwrap();
    events.insert(i + 1, ret);
    assert!(reviewed::validate(&events, task(), &r, e).is_err());
}
