//! `tsr reflect` — the Mirror view, the family calibration curves, and the JSON document
//! (PRD §B7, §B11 M2 item 7).
//!
//! The tests drive the store API directly, so they assert the exact screen the Mirror renders:
//!
//! 1. `tsr reflect <task> --json` carries the five sections (desire, imagination, belief, returns,
//!    residual) with every forecaster's `raw p` and `uncalibrated`;
//! 2. `tsr reflect --family <f>` prints reliability bins and Brier per forecaster, the operator's
//!    included;
//! 3. the rendered text contains no TSR term followed by a number and no "percent certain";
//! 4. an Intuition candidate renders its support, and "none yet" is a valid screen.

use std::process::Command;

use tsr_cli::reflect_cmd::{
    build_family_curves, build_mirror, render_family_curves, render_mirror, ReflectArgs,
};
use tsr_domain::{
    ActionKind, Candidate, CandidateId, Contrast, ContrastId, CriterionId, Decision, Event,
    EventKind, Forecast, ForecastId, Forecaster, Hash, Horizon, LabelStatus, Observation,
    OutcomeRecord, Probability, Source, Support, Surprise, TaskId, UnmetKind, Utility,
};

/// A task id from a fixed seed, so tests are reproducible.
fn task(seed: u8) -> TaskId {
    TaskId(Hash::of(&[b"m2-reflect-task".as_slice(), &[seed]]))
}

/// A candidate id from a fixed seed.
fn candidate_id(seed: u8) -> CandidateId {
    CandidateId(Hash::of(&[b"m2-reflect-candidate".as_slice(), &[seed]]))
}

/// A forecast about the common decision event, by one forecaster, with a raw probability.
fn forecast(seed: u8, source: Forecaster, raw: f64) -> Forecast {
    Forecast {
        id: ForecastId(Hash::of(&[b"m2-reflect-forecast".as_slice(), &[seed]])),
        event: tsr_forecast::common_decision_event_id(),
        horizon: Horizon::TaskClose,
        source,
        raw: Probability::new(raw).unwrap(),
        calibrated: None,
        support: Support { episodes: 3 },
        state_version: 0,
    }
}

/// Build one closed episode: open, propose (generator + intuition), forecast (generator,
/// controller, operator), commit, dispatch, execute, observe a return, label it, and expand.
fn episode(store: &tsr_store::Store, t: &TaskId) {
    let gen = candidate_id(1);
    let intuition = candidate_id(2);

    // TaskOpened.
    store
        .append(&Event::new(
            *t,
            Vec::new(),
            Source::Operator,
            EventKind::TaskOpened { task: *t },
            "unix:1",
        ))
        .unwrap();

    // A family-scoped preference tags the episode's family (the fold reads the first
    // `Scope::Family` in log order as the task's family).
    let ledger = tsr_ledger::Ledger::new(store);
    ledger
        .expand(
            t,
            Vec::new(),
            ContrastId(Hash::of(&[b"family-contrast"])),
            "work in the parser family stays within its budget",
            tsr_domain::Scope::Family("parser".to_string()),
            Vec::new(),
            Vec::new(),
            Source::Operator,
            "unix:1",
        )
        .unwrap();

    // The operator's intent, as an experience note.
    store
        .append(&Event::new(
            *t,
            Vec::new(),
            Source::Operator,
            EventKind::Observed {
                observation: Observation::Experience {
                    note: "make the parser accept trailing commas".to_string(),
                },
            },
            "unix:1",
        ))
        .unwrap();

    // Two candidates: one generator, one intuition (with support).
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
    int_candidate.support = vec![tsr_domain::PreferenceId(Hash::of(&[b"served"]))];
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

    // Three forecasts on the committed candidate: generator, controller, operator.
    for (i, (src, p)) in [
        (Forecaster::Generator, 0.7),
        (Forecaster::Controller, 0.6),
        (Forecaster::Operator, 0.8),
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
                    candidate: gen,
                },
                "unix:1",
            ))
            .unwrap();
    }

    // Commit the generator candidate.
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
                    propensity: 1.0,
                    evidence: Vec::new(),
                },
            },
            "unix:1",
        ))
        .unwrap();

    // Dispatch and execute.
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

    // The return: a Surprise per forecaster and a Contrast against one criterion.
    let surprise = vec![
        Surprise {
            forecast: ForecastId(Hash::of(&[b"m2-reflect-forecast".as_slice(), &[0]])),
            residual: 0.3,
        },
        Surprise {
            forecast: ForecastId(Hash::of(&[b"m2-reflect-forecast".as_slice(), &[1]])),
            residual: 0.4,
        },
    ];
    let contrast = Contrast {
        id: ContrastId(Hash::of(&[b"contrast"])),
        from_return: tsr_domain::EventId(Hash::of(&[b"return"])),
        unmet: vec![(
            CriterionId(Hash::of(&[b"criterion"])),
            UnmetKind::WrongBehaviour,
        )],
        evidence: vec![tsr_domain::EventId(Hash::of(&[b"evidence"]))],
    };
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

    // A label on the return, then a superseding correction.
    let subject = tsr_domain::EventId(Hash::of(&[b"return"]));
    store
        .append(&Event::new(
            *t,
            Vec::new(),
            Source::Operator,
            EventKind::OutcomeRecorded {
                record: OutcomeRecord {
                    outcome: tsr_forecast::common_decision_event_id(),
                    subject,
                    status: LabelStatus::Observed,
                    reviewer: Source::Operator,
                    reason: None,
                    evidence: Some(Hash::of(&[b"label-evidence"])),
                    supersedes: None,
                    weight: None,
                },
            },
            "unix:1",
        ))
        .unwrap();
    store
        .append(&Event::new(
            *t,
            Vec::new(),
            Source::Operator,
            EventKind::OutcomeRecorded {
                record: OutcomeRecord {
                    outcome: tsr_forecast::common_decision_event_id(),
                    subject,
                    status: LabelStatus::Disputed,
                    reviewer: Source::Operator,
                    reason: Some("the diff was edited by hand".to_string()),
                    evidence: None,
                    supersedes: Some(subject),
                    weight: None,
                },
            },
            "unix:1",
        ))
        .unwrap();

    // Close the task so the common decision event resolves and the forecasts score.
    store
        .append(&Event::new(
            *t,
            Vec::new(),
            Source::Operator,
            EventKind::TaskClosed {
                task: *t,
                reason: "operator".to_string(),
            },
            "unix:1",
        ))
        .unwrap();
}

#[test]
fn json_carries_the_five_sections_with_raw_p_and_uncalibrated() {
    let store = tsr_store::Store::open_in_memory().unwrap();
    let t = task(1);
    episode(&store, &t);
    let events = store.by_task(&t).unwrap();
    let root = store.head_root().unwrap();
    let mirror = build_mirror(&t, &events, root);

    // The five sections are present.
    assert!(mirror.desire.intent.is_some(), "desire carries the intent");
    assert!(
        !mirror.imagination.candidates.is_empty(),
        "imagination lists the candidates"
    );
    assert!(
        !mirror.belief.forecasters.is_empty(),
        "belief lists the forecasters"
    );
    assert!(!mirror.returns.is_empty(), "return lists what came back");
    assert!(
        !mirror.residual.surprise.is_empty(),
        "residual carries the Surprise"
    );
    assert!(
        !mirror.residual.contrast.is_empty(),
        "residual carries the Contrast"
    );

    // Every forecaster's raw p is present and uncalibrated.
    let names: Vec<&str> = mirror
        .belief
        .forecasters
        .iter()
        .map(|b| b.forecaster.as_str())
        .collect();
    assert!(names.contains(&"generator"), "generator forecast present");
    assert!(names.contains(&"controller"), "controller forecast present");
    assert!(names.contains(&"operator"), "operator forecast present");
    for b in &mirror.belief.forecasters {
        assert!(b.raw_p > 0.0, "raw p is recorded");
        assert_eq!(b.calibration, "uncalibrated");
        assert!(b.calibrated_p.is_none());
    }

    // The JSON document round-trips and carries the same sections.
    let json = serde_json::to_string(&mirror).unwrap();
    let v: serde_json::Value = serde_json::from_str(&json).unwrap();
    for key in ["desire", "imagination", "belief", "returns", "residual"] {
        assert!(v.get(key).is_some(), "the JSON carries `{key}`");
    }
    assert!(json.contains("uncalibrated"));
}

#[test]
fn intuition_candidate_renders_its_support() {
    let store = tsr_store::Store::open_in_memory().unwrap();
    let t = task(2);
    episode(&store, &t);
    let events = store.by_task(&t).unwrap();
    let root = store.head_root().unwrap();
    let mirror = build_mirror(&t, &events, root);

    let intuition = mirror
        .imagination
        .candidates
        .iter()
        .find(|c| c.source == "intuition")
        .expect("the intuition candidate is listed");
    assert!(
        !intuition.support.is_empty(),
        "the intuition candidate names the preferences it serves"
    );

    let text = render_mirror(&mirror);
    assert!(text.contains("support (preferences served)"));
    assert!(text.contains("tensions addressed"));
}

#[test]
fn family_curves_print_bins_and_brier_per_forecaster() {
    let store = tsr_store::Store::open_in_memory().unwrap();
    let t = task(3);
    episode(&store, &t);
    let events = store.all().unwrap();
    let curves = build_family_curves("parser", &events);

    let names: Vec<&str> = curves
        .curves
        .iter()
        .map(|c| c.forecaster.as_str())
        .collect();
    assert!(names.contains(&"generator"), "generator curve present");
    assert!(names.contains(&"controller"), "controller curve present");
    assert!(names.contains(&"operator"), "operator curve present");

    let text = render_family_curves(&curves);
    assert!(text.contains("brier"), "the curve prints Brier");
    assert!(text.contains("bin ["), "the curve prints reliability bins");
    assert!(
        text.contains("operator"),
        "the operator's curve is included"
    );
}

#[test]
fn rendered_text_has_no_tsr_term_with_a_number_and_no_percent_certain() {
    let store = tsr_store::Store::open_in_memory().unwrap();
    let t = task(4);
    episode(&store, &t);
    let events = store.by_task(&t).unwrap();
    let root = store.head_root().unwrap();
    let mirror = build_mirror(&t, &events, root);
    let text = render_mirror(&mirror);

    assert!(
        !text.contains("percent certain"),
        "no screen prints `percent certain`"
    );
    // No TSR term is ever followed by a number on the same line: the four directions are not a
    // sequence and no term carries a score.
    for line in text.lines() {
        for term in [
            "Desire",
            "Imagination",
            "Belief",
            "Return",
            "Surprise",
            "Contrast",
        ] {
            if let Some(i) = line.find(term) {
                let after = &line[i + term.len()..];
                let next = after.trim_start().chars().next();
                assert!(
                    !matches!(next, Some(c) if c.is_ascii_digit()),
                    "`{term}` is followed by a number on one line: {line}"
                );
            }
        }
    }
}

#[test]
fn cli_reflect_json_and_family_run() {
    let dir = std::env::temp_dir().join(format!("m2-reflect-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("tsr.db");
    let db_str = db.to_str().unwrap().to_string();

    // Seed the store through the library, then read it back through the CLI.
    {
        let store = tsr_store::Store::open(&db).unwrap();
        let t = task(5);
        episode(&store, &t);
    }

    let bin = env!("CARGO_BIN_EXE_tsr");
    let t = task(5);

    // `tsr reflect <task> --json`
    let out = Command::new(bin)
        .args(["reflect", &t.to_string(), "--json", "--store", &db_str])
        .output()
        .unwrap();
    assert!(out.status.success(), "reflect --json exits 0");
    let stdout = String::from_utf8(out.stdout).unwrap();
    let v: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    for key in ["desire", "imagination", "belief", "returns", "residual"] {
        assert!(v.get(key).is_some(), "the CLI JSON carries `{key}`");
    }
    assert!(stdout.contains("uncalibrated"));

    // `tsr reflect --family <f>`
    let out = Command::new(bin)
        .args(["reflect", "--family", "parser", "--store", &db_str])
        .output()
        .unwrap();
    assert!(out.status.success(), "reflect --family exits 0");
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("brier"), "the CLI prints Brier");
    assert!(stdout.contains("bin ["), "the CLI prints reliability bins");
    assert!(
        stdout.contains("operator"),
        "the operator's curve is included"
    );

    // `tsr reflect` with no subject is refused with exit 2.
    let out = Command::new(bin)
        .args(["reflect", "--store", &db_str])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(2),
        "no subject is refused with exit 2"
    );

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_refuses_a_missing_subject_with_exit_2() {
    // `run` opens its own store, so this only checks the argument guard.
    let code = tsr_cli::reflect_cmd::run(&ReflectArgs {
        task: None,
        family: None,
        json: false,
        store: "does-not-exist.db".to_string(),
    });
    assert_eq!(format!("{code:?}"), "ExitCode(unix_exit_status(2))");
}
