//! D9 (the Mirror's propensity and intent) — repair of milestone M2, item (4).
//!
//! Two defects, both in `crates/tsr-cli/src/reflect_cmd.rs`:
//!
//! * **propensity** came from the *first* `Committed` in the log (a `find_map` over commits), so
//!   two commits of 0.25 then 0.75 for the same candidate showed `0.2500` for the second. It must
//!   come from the commit record of the candidate the row shows.
//! * **intent** was the first task-scoped preference in the fold, so after a rejection on a task
//!   with no criteria it printed the contrast's dimension (`the dimension this contrast clarified
//!   — rejection: …`). It must be the task's intent, not a rejection's dimension.

use tsr_cli::reflect_cmd::{build_mirror, render_mirror};
use tsr_domain::{
    ActionKind, Candidate, CandidateId, Contrast, ContrastId, CriterionId, Decision, Event,
    EventKind, Forecast, ForecastId, Forecaster, Hash, Horizon, Probability, Source, Support,
    TaskId, UnmetKind, Utility,
};
use tsr_store::Store;

fn task(seed: u8) -> TaskId {
    TaskId(Hash::of(&[b"m2-mirror-task".as_slice(), &[seed]]))
}

fn candidate_id(seed: u8) -> CandidateId {
    CandidateId(Hash::of(&[b"m2-mirror-candidate".as_slice(), &[seed]]))
}

fn forecast(seed: u8, source: Forecaster, raw: f64) -> Forecast {
    Forecast {
        id: ForecastId(Hash::of(&[b"m2-mirror-forecast".as_slice(), &[seed]])),
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
const INTENT: &str = "make the parser accept trailing commas";

/// Open the task and propose one generator candidate.
fn open_and_propose(store: &Store, t: &TaskId, cand: CandidateId) {
    store
        .append(&Event::new(
            *t,
            Vec::new(),
            Source::Operator,
            EventKind::TaskOpened { task: *t },
            "unix:1",
        ))
        .unwrap();

    let mut c = Candidate::new(
        ActionKind::ProposePatch,
        Source::Generator,
        Hash::of(&[b"args-gen"]),
        Vec::new(),
        Vec::new(),
    )
    .unwrap();
    c.id = cand;
    store
        .append(&Event::new(
            *t,
            Vec::new(),
            Source::Generator,
            EventKind::Proposed { candidate: c },
            "unix:1",
        ))
        .unwrap();
}

/// Commit `cand` with a given propensity.
fn commit(store: &Store, t: &TaskId, cand: CandidateId, propensity: f64) {
    store
        .append(&Event::new(
            *t,
            Vec::new(),
            Source::Operator,
            EventKind::Committed {
                candidate: cand,
                decision: Decision::Selected {
                    value: cand,
                    utility: Utility {
                        p_success: 0.7,
                        value: 1.0,
                        cost: 0.1,
                        ev: 0.6,
                    },
                    propensity,
                    evidence: Vec::new(),
                },
            },
            "unix:1",
        ))
        .unwrap();
}

/// D9 (a): the propensity is the commit record of the candidate the row shows.
///
/// Two commits of the same candidate — 0.25 then 0.75 — must show 0.7500 for the row that shows
/// the second commit, not the first commit's 0.2500.
#[test]
fn d9_mirror_propensity_is_the_shown_candidates() {
    let store = Store::open_in_memory().unwrap();
    let t = task(1);
    let cand = candidate_id(1);
    open_and_propose(&store, &t, cand);

    // One forecast on the committed candidate, so Belief has a row to carry the propensity.
    store
        .append(&Event::new(
            t,
            Vec::new(),
            Source::Generator,
            EventKind::Forecast {
                forecast: forecast(0, Forecaster::Generator, 0.7),
                candidate: cand,
            },
            "unix:1",
        ))
        .unwrap();

    // Two commits of the same candidate: 0.25 first, then 0.75.
    commit(&store, &t, cand, 0.25);
    commit(&store, &t, cand, 0.75);

    let events = store.by_task(&t).unwrap();
    let root = store.head_root().unwrap();
    let m = build_mirror(&t, &events, root);

    assert!(!m.belief.forecasters.is_empty(), "belief lists forecasters");
    for row in &m.belief.forecasters {
        assert_eq!(
            row.propensity,
            Some(0.75),
            "the propensity is the commit record of the shown candidate (the second commit)"
        );
    }

    // The rendered screen shows 0.7500, not 0.2500.
    let text = render_mirror(&m);
    assert!(
        text.contains("propensity 0.7500"),
        "the screen shows the second commit's propensity:\n{text}"
    );
    assert!(
        !text.contains("propensity 0.2500"),
        "the screen does not show the first commit's propensity:\n{text}"
    );
}

/// D9 (b): the intent is the task's intent, not a rejection's dimension.
///
/// A task with no criteria: the only task-scoped preference the fold sees after a rejection is the
/// contrast's dimension. The Mirror must still print the task's own intent.
#[test]
fn d9_mirror_intent_is_taskopened() {
    let store = Store::open_in_memory().unwrap();
    let t = task(2);
    let cand = candidate_id(2);
    open_and_propose(&store, &t, cand);

    // A rejection on the task: `record_rejection_preference` writes a task-scoped preference whose
    // text is the contrast's dimension — never the task's intent. It is recorded *first*, so the
    // fold's first task-scoped preference is the dimension: the buggy Mirror printed it as the
    // intent. The task has no criteria, so the dimension is the only other task-scoped preference.
    let subject = tsr_domain::EventId(Hash::of(&[b"rejected-return"]));
    let dimension = tsr_cli::review_cmd::record_rejection_preference(
        &store,
        &t,
        &subject,
        "the diff was edited by hand",
        "unix:1",
    )
    .unwrap();
    let _ = dimension;

    // The task's intent: a task-scoped preference recorded at open (the desired condition).
    let ledger = tsr_ledger::Ledger::new(&store);
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
        .record_contrast(&t, Vec::new(), intent_contrast.clone(), "unix:1")
        .unwrap();
    ledger
        .expand(
            &t,
            Vec::new(),
            intent_contrast.id,
            INTENT,
            tsr_domain::Scope::Task(t),
            Vec::new(),
            Vec::new(),
            Source::Operator,
            "unix:1",
        )
        .unwrap();

    let events = store.by_task(&t).unwrap();
    let root = store.head_root().unwrap();
    let m = build_mirror(&t, &events, root);

    assert_eq!(
        m.desire.intent.as_deref(),
        Some(INTENT),
        "the intent is the task's own intent, not the rejection's dimension"
    );
    assert!(
        !m.desire
            .intent
            .as_deref()
            .unwrap_or_default()
            .contains("the dimension this contrast clarified"),
        "the intent is not the contrast's dimension: {:?}",
        m.desire.intent
    );

    let text = render_mirror(&m);
    assert!(
        text.contains(&format!("intent: {INTENT}")),
        "the screen prints the task's intent:\n{text}"
    );
    assert!(
        !text.contains("intent: the dimension this contrast clarified"),
        "the screen does not print the contrast's dimension as the intent:\n{text}"
    );
}
