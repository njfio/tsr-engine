//! D9 (the Mirror) and D12 (`tsr note` on an unknown task) — repair of milestone M2.
//!
//! D9: the Mirror must name the forecaster on every Surprise row, take the propensity from the
//! commit record, list only the committed candidate's forecasts under Belief, take Desire's intent
//! from the task's own intent (the task-scoped preference recorded at open) rather than from an
//! operator note, and list operator notes as notes (through `notes_for`).
//!
//! D12: `tsr note` on a task the log does not hold must exit 2.

use std::process::Command;

use tsr_cli::reflect_cmd::{build_mirror, render_mirror};
use tsr_domain::{
    ActionKind, Candidate, CandidateId, CriterionId, Decision, Event,
    EventKind, Forecast, ForecastId, Forecaster, Hash, Horizon, Observation, Probability, Source,
    Support, Surprise, TaskId, UnmetKind, Utility,
};
use tsr_store::Store;

fn task(seed: u8) -> TaskId {
    TaskId(Hash::of(&[b"d9-task".as_slice(), &[seed]]))
}

fn candidate_id(seed: u8) -> CandidateId {
    CandidateId(Hash::of(&[b"d9-candidate".as_slice(), &[seed]]))
}

fn forecast(seed: u8, source: Forecaster, raw: f64) -> Forecast {
    Forecast {
        id: ForecastId(Hash::of(&[b"d9-forecast".as_slice(), &[seed]])),
        event: tsr_forecast::common_decision_event_id(),
        horizon: Horizon::TaskClose,
        source,
        raw: Probability::new(raw).unwrap(),
        calibrated: None,
        support: Support { episodes: 3 },
        state_version: 0,
    }
}

/// The task's intent, recorded as a task-scoped preference (the desired condition at open).
const INTENT: &str = "the parser accepts trailing commas";
/// The operator's experience note — a note, never the intent.
const NOTE: &str = "friction: the sandbox took four minutes";

/// Build one episode: open, record the task's intent as a task-scoped preference, note, propose a
/// generator candidate (committed) and an intuition candidate (not committed), forecast on both,
/// commit the generator with a distinctive propensity, dispatch, execute, and observe a return.
fn episode(store: &Store, t: &TaskId) {
    let gen = candidate_id(1);
    let intuition = candidate_id(2);

    store
        .append(&Event::new(
            *t,
            Vec::new(),
            Source::Operator,
            EventKind::TaskOpened { task: *t },
            "unix:1",
        ))
        .unwrap();

    // The task's intent: a task-scoped preference, recorded through the Filter.
    let ledger = tsr_ledger::Ledger::new(store);
    let intent_contrast = tsr_ledger::Filter::contrast_from_criteria(
        tsr_domain::EventId(Hash::of(&[b"intent-return"])),
        vec![(
            CriterionId(Hash::of(&[b"intent-criterion"])),
            UnmetKind::Rejected {
                reason: "the criterion went unmet".to_string(),
            },
        )],
        Vec::new(),
    );
    ledger
        .record_contrast(t, Vec::new(), intent_contrast.clone(), "unix:1")
        .unwrap();
    ledger
        .expand(
            t,
            Vec::new(),
            intent_contrast.id,
            INTENT,
            tsr_domain::Scope::Task(*t),
            Vec::new(),
            Vec::new(),
            Source::Operator,
            "unix:1",
        )
        .unwrap();

    // The operator's experience note: a note, not the intent.
    store
        .append(&Event::new(
            *t,
            Vec::new(),
            Source::Operator,
            EventKind::Observed {
                observation: Observation::Experience {
                    note: NOTE.to_string(),
                },
            },
            "unix:1",
        ))
        .unwrap();

    // Two candidates: generator (committed) and intuition (not committed).
    let mut gen_candidate = Candidate::new(
        ActionKind::ProposePatch,
        Source::Generator,
        Hash::of(&[b"args-gen"]),
        Vec::new(),
        Vec::new(),
    )
    .unwrap();
    gen_candidate.id = gen;
    store
        .append(&Event::new(
            *t,
            Vec::new(),
            Source::Generator,
            EventKind::Proposed {
                candidate: gen_candidate,
            },
            "unix:1",
        ))
        .unwrap();

    let mut int_candidate = Candidate::new(
        ActionKind::ProposePatch,
        Source::Intuition,
        Hash::of(&[b"args-intuition"]),
        Vec::new(),
        Vec::new(),
    )
    .unwrap();
    int_candidate.id = intuition;
    store
        .append(&Event::new(
            *t,
            Vec::new(),
            Source::Intuition,
            EventKind::Proposed {
                candidate: int_candidate,
            },
            "unix:1",
        ))
        .unwrap();

    // Forecasts: two on the committed generator candidate, one on the uncommitted intuition one.
    for (i, (src, p, cand)) in [
        (Forecaster::Generator, 0.7, gen),
        (Forecaster::Controller, 0.6, gen),
        (Forecaster::Operator, 0.8, intuition),
    ]
    .into_iter()
    .enumerate()
    {
        store
            .append(&Event::new(
                *t,
                Vec::new(),
                Source::Generator,
                EventKind::Forecast {
                    forecast: forecast(i as u8, src, p),
                    candidate: cand,
                },
                "unix:1",
            ))
            .unwrap();
    }

    // Commit the generator candidate with a distinctive propensity.
    store
        .append(&Event::new(
            *t,
            Vec::new(),
            Source::Operator,
            EventKind::Committed {
                candidate: gen,
                decision: Decision::Selected {
                    value: gen,
                    utility: Utility {
                        p_success: 0.7,
                        value: 1.0,
                        cost: 0.1,
                        ev: 0.6,
                    },
                    propensity: 0.42,
                    evidence: Vec::new(),
                },
            },
            "unix:1",
        ))
        .unwrap();

    store
        .append(&Event::new(
            *t,
            Vec::new(),
            Source::Generator,
            EventKind::Dispatched {
                candidate: gen,
                in_sandbox: true,
            },
            "unix:1",
        ))
        .unwrap();
    store
        .append(&Event::new(
            *t,
            Vec::new(),
            Source::Generator,
            EventKind::Executed {
                candidate: gen,
                exit_code: Some(0),
                log_digest: Some(Hash::of(&[b"log"])),
                duration_ms: 12,
            },
            "unix:1",
        ))
        .unwrap();

    // The return: a Surprise per forecast (generator and controller), and a Contrast.
    let surprise = vec![
        Surprise {
            forecast: ForecastId(Hash::of(&[b"d9-forecast".as_slice(), &[0]])),
            residual: 0.3,
        },
        Surprise {
            forecast: ForecastId(Hash::of(&[b"d9-forecast".as_slice(), &[1]])),
            residual: 0.4,
        },
    ];
    let contrast = tsr_ledger::Filter::contrast_from_criteria(
        tsr_domain::EventId(Hash::of(&[b"return"])),
        vec![(
            CriterionId(Hash::of(&[b"criterion"])),
            UnmetKind::WrongBehaviour,
        )],
        vec![tsr_domain::EventId(Hash::of(&[b"evidence"]))],
    );
    store
        .append(&Event::new(
            *t,
            Vec::new(),
            Source::Generator,
            EventKind::Observed {
                observation: Observation::Return {
                    surprise,
                    contrast: Some(contrast),
                },
            },
            "unix:1",
        ))
        .unwrap();
}

fn mirror_for(seed: u8) -> (Store, TaskId, tsr_cli::reflect_cmd::Mirror) {
    let store = Store::open_in_memory().unwrap();
    let t = task(seed);
    episode(&store, &t);
    let events = store.by_task(&t).unwrap();
    let root = store.head_root().unwrap();
    let m = build_mirror(&t, &events, root);
    (store, t, m)
}

// ---------------------------------------------------------------------------------------------
// D9 (a): Surprise rows name their forecaster
// ---------------------------------------------------------------------------------------------

#[test]
fn d9_surprise_rows_name_their_forecaster() {
    let (_s, _t, m) = mirror_for(1);
    assert_eq!(m.residual.surprise.len(), 2, "two surprise rows");
    // Forecast 0 was made by the generator, forecast 1 by the controller.
    assert_eq!(
        m.residual.surprise[0].forecaster, "generator",
        "the first surprise row names the generator"
    );
    assert_eq!(
        m.residual.surprise[1].forecaster, "controller",
        "the second surprise row names the controller"
    );
    for row in &m.residual.surprise {
        assert!(
            !row.forecaster.is_empty(),
            "every surprise row names its forecaster"
        );
    }
    // The rendered screen shows the name too. Scope the assertion to the Residual section: the
    // names also appear under Imagination, so a whole-screen `contains` cannot fail on this defect.
    let text = render_mirror(&m);
    let residual = text
        .split("Surprise (forecast residual per forecaster)")
        .nth(1)
        .expect("the Residual section is rendered");
    let residual = residual
        .split("Contrast (desired minus actual, per criterion)")
        .next()
        .expect("the Contrast header follows the Surprise rows");
    // Each Surprise row must name its forecaster, in the row itself.
    assert!(
        residual.contains("by generator"),
        "the rendered Surprise row names the generator:\n{residual}"
    );
    assert!(
        residual.contains("by controller"),
        "the rendered Surprise row names the controller:\n{residual}"
    );
    // And the exact row format carries the forecaster between the forecast id and the residual.
    let first = m.residual.surprise[0].forecast.clone();
    assert!(
        residual.contains(&format!("forecast {first} by generator residual")),
        "the Surprise row is `forecast <id> by <forecaster> residual <r>`:\n{residual}"
    );
}

// ---------------------------------------------------------------------------------------------
// D9 (b): propensity comes from the commit record
// ---------------------------------------------------------------------------------------------

#[test]
fn d9_propensity_comes_from_the_commit_record() {
    let (_s, _t, m) = mirror_for(2);
    assert!(!m.belief.forecasters.is_empty(), "belief lists forecasters");
    for row in &m.belief.forecasters {
        assert_eq!(
            row.propensity,
            Some(0.42),
            "propensity is the committed decision's propensity, not none"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// D9 (c): Belief lists the committed candidate's forecasts only
// ---------------------------------------------------------------------------------------------

#[test]
fn d9_belief_lists_only_the_committed_candidates_forecasts() {
    let (_s, _t, m) = mirror_for(3);
    // Two forecasts were made on the committed generator candidate; one on the uncommitted
    // intuition candidate. Only the two committed ones belong under Belief.
    assert_eq!(
        m.belief.forecasters.len(),
        2,
        "only the committed candidate's forecasts are listed"
    );
    let names: Vec<&str> = m
        .belief
        .forecasters
        .iter()
        .map(|r| r.forecaster.as_str())
        .collect();
    assert!(names.contains(&"generator"), "names: {names:?}");
    assert!(names.contains(&"controller"), "names: {names:?}");
    assert!(
        !names.contains(&"operator"),
        "the operator's forecast was on the uncommitted candidate, so it is not listed: {names:?}"
    );
}

// ---------------------------------------------------------------------------------------------
// D9 (d): Desire's intent is the task's intent, not the operator note
// ---------------------------------------------------------------------------------------------

#[test]
fn d9_desire_intent_is_the_tasks_intent_not_the_note() {
    let (_s, _t, m) = mirror_for(4);
    assert_eq!(
        m.desire.intent.as_deref(),
        Some(INTENT),
        "the intent is the task-scoped preference recorded at open"
    );
    assert_ne!(
        m.desire.intent.as_deref(),
        Some(NOTE),
        "the operator's note is not the intent"
    );
}

// ---------------------------------------------------------------------------------------------
// D9 (e): operator notes are listed as notes, through notes_for
// ---------------------------------------------------------------------------------------------

#[test]
fn d9_operator_notes_are_listed_as_notes() {
    let (s, t, m) = mirror_for(5);
    // The Mirror's notes match notes_for exactly.
    let expected = tsr_cli::note_cmd::notes_for(&s, &t).unwrap();
    assert_eq!(expected.len(), 1, "one operator note in the log");
    assert_eq!(m.desire.notes.len(), 1, "the Mirror lists the note");
    assert_eq!(m.desire.notes[0].text, NOTE);
    assert_eq!(m.desire.notes[0].event, expected[0].0.to_string());
    // The rendered screen shows the note under Desire.
    let text = render_mirror(&m);
    assert!(
        text.contains(NOTE),
        "the rendered Desire lists the note:\n{text}"
    );
}

// ---------------------------------------------------------------------------------------------
// D12: `tsr note` on a task the log does not hold exits 2
// ---------------------------------------------------------------------------------------------

#[test]
fn d12_note_on_unknown_task_exits_2() {
    let dir = std::env::temp_dir().join(format!("tsr-d12-note-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("tsr.db");
    // A store that holds a *different* task, so the store opens but the task is unknown.
    {
        let s = Store::open(&db).unwrap();
        let other = task(99);
        s.append(&Event::new(
            other,
            vec![],
            Source::Rules,
            EventKind::TaskOpened { task: other },
            "t0",
        ))
        .unwrap();
    }

    let unknown = task(7);
    let bin = env!("CARGO_BIN_EXE_tsr");
    let out = Command::new(bin)
        .args([
            "note",
            &unknown.to_string(),
            "--text",
            "a note on a task the log does not hold",
            "--store",
            db.to_str().unwrap(),
        ])
        .output()
        .expect("the tsr binary runs");
    assert_eq!(
        out.status.code(),
        Some(2),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    // Nothing was written for the unknown task.
    let s = Store::open(&db).unwrap();
    assert!(
        s.by_task(&unknown).unwrap().is_empty(),
        "no event was appended for the unknown task"
    );

    std::fs::remove_dir_all(&dir).ok();
}
