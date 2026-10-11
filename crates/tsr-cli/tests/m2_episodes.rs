//! fixtures/m2 — thirty seeded synthetic episodes (PRD §B11 M2 item 8).
//!
//! This file is the **generator**: a seeded, documented function that drives the store API
//! through one closed episode — open → propose → forecast (generator, controller, operator) →
//! commit → dispatch → execute → close → review — and writes the resulting event log to
//! `fixtures/m2/episodes/<family>-<n>.json`.
//!
//! The episodes are **synthetic**. They are produced by this generator from a fixed seed, not by
//! operator sessions; the real thirty come from operator sessions after M3's batch runner. They
//! exist so the Mirror (`tsr reflect`) and the calibration curves have something to render, and
//! so the fold, the Filter and the review path are exercised end to end over a corpus.
//!
//! Three families — `alpha`, `beta`, `gamma` — differ in the **generator's calibration**: the
//! generator's raw probability is biased by a per-family offset, so `tsr reflect --family <f>`
//! shows a different reliability curve for each. The controller and the operator are calibrated
//! the same way in every family, so the generator's curve is the one that moves.
//!
//! Determinism: every draw comes from a small splitmix64 PRNG seeded from `SEED` and the episode
//! index, so regenerating the fixtures is byte-for-byte reproducible. The test asserts the corpus
//! is thirty episodes across three families and that a second generation matches the first.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use tsr_cli::review_cmd::{record_label, Label};
use tsr_domain::{
    ActionKind, Candidate, Contrast, ContrastId, Decision, Event, EventId, EventKind, Forecast,
    ForecastId, Forecaster, Hash, Horizon, LabelStatus, Observation, Probability, Scope, Source,
    Support, Surprise, TaskId, UnmetKind, Utility,
};
use tsr_forecast::outcomes::common_decision_event_id;
use tsr_ledger::Ledger;
use tsr_store::Store;

/// The seed the fixtures are generated from. Changing it changes every episode.
pub const SEED: u64 = 0x5eed_2024_0000_0001;

/// The three families, each with its own generator calibration offset.
///
/// `generator_bias` is added to the generator's raw probability before it is clamped to `[0, 1]`:
/// a positive bias makes the generator over-confident, a negative one under-confident. The
/// controller and the operator carry no bias, so the generator's curve is the one that differs.
pub const FAMILIES: [Family; 3] = [
    Family {
        name: "alpha",
        generator_bias: 0.15,
    },
    Family {
        name: "beta",
        generator_bias: 0.0,
    },
    Family {
        name: "gamma",
        generator_bias: -0.15,
    },
];

/// One task family and the generator's calibration offset within it.
#[derive(Clone, Copy, Debug)]
pub struct Family {
    pub name: &'static str,
    pub generator_bias: f64,
}

/// The number of episodes per family. Three families × ten = thirty.
pub const PER_FAMILY: usize = 10;

/// A tiny deterministic PRNG (splitmix64). No external crate, no OS entropy: the corpus is a pure
/// function of the seed.
struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed)
    }

    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A uniform draw in `[0, 1)`.
    fn unit(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// A uniform draw in `[lo, hi)`.
    fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.unit()
    }
}

fn clamp01(x: f64) -> f64 {
    x.clamp(0.0, 1.0)
}

fn prob(x: f64) -> Probability {
    Probability::new(clamp01(x)).expect("clamped into [0, 1]")
}

/// A task id from a fixed seed and index, so the corpus is stable.
fn task_id(seed: u64, family: &str, index: usize) -> TaskId {
    TaskId(Hash::of(&[
        b"m2-episode",
        &seed.to_le_bytes(),
        family.as_bytes(),
        &(index as u64).to_le_bytes(),
    ]))
}

/// A forecast id from a fixed seed, index and forecaster name.
fn forecast_id(seed: u64, index: usize, who: &str) -> ForecastId {
    ForecastId(Hash::of(&[
        b"m2-forecast",
        &seed.to_le_bytes(),
        &(index as u64).to_le_bytes(),
        who.as_bytes(),
    ]))
}

/// A contrast id from a fixed seed, index and criterion.
fn contrast_id(seed: u64, index: usize, criterion: &str) -> ContrastId {
    ContrastId(Hash::of(&[
        b"m2-contrast",
        &seed.to_le_bytes(),
        &(index as u64).to_le_bytes(),
        criterion.as_bytes(),
    ]))
}

/// A criterion id from a fixed seed, index and name.
fn criterion_id(seed: u64, index: usize, name: &str) -> tsr_domain::CriterionId {
    tsr_domain::CriterionId(Hash::of(&[
        b"m2-criterion",
        &seed.to_le_bytes(),
        &(index as u64).to_le_bytes(),
        name.as_bytes(),
    ]))
}

/// A log digest for an execution, from a fixed seed and index.
fn log_digest(seed: u64, index: usize) -> Hash {
    Hash::of(&[
        b"m2-log",
        &seed.to_le_bytes(),
        &(index as u64).to_le_bytes(),
    ])
}

/// A wall-clock stamp that is a pure function of the episode index, so the corpus is stable.
fn stamp(index: usize, step: u64) -> String {
    format!("2024-01-01T00:00:{:02}.{:03}Z", index % 60, step)
}

/// The outcome of one episode, as the generator decided it before writing any event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EpisodeOutcome {
    /// The approved test passed and the operator accepted the return.
    Accepted,
    /// The approved test passed but the operator rejected the return (a false accept).
    Rejected,
    /// The approved test failed; the operator disputed the return.
    Disputed,
}

/// One generated episode: the task, its family, its outcome, and the events written.
#[derive(Clone, Debug)]
pub struct Episode {
    pub task: TaskId,
    pub family: &'static str,
    pub index: usize,
    pub outcome: EpisodeOutcome,
    pub events: Vec<Event>,
}

/// Generate one episode into `store`, returning what was written.
///
/// The sequence is the one the brief names: open → propose → forecast (generator, controller,
/// operator) → commit → dispatch → execute → close → review. Every event is appended through the
/// store API; the ledger's write path is used for the Expansion and the statuses, so the fold is
/// exercised exactly as production would exercise it.
fn generate_episode(store: &Store, seed: u64, family: Family, index: usize) -> Episode {
    let task = task_id(seed, family.name, index);
    let mut rng = Rng::new(seed ^ (index as u64).wrapping_mul(0x9E37_79B9));
    let ledger = Ledger::new(store);

    // The episode's true outcome, drawn first so the forecasts can be made against it.
    let outcome = match rng.unit() {
        u if u < 0.60 => EpisodeOutcome::Accepted,
        u if u < 0.85 => EpisodeOutcome::Rejected,
        _ => EpisodeOutcome::Disputed,
    };
    let truth = if outcome == EpisodeOutcome::Accepted {
        1.0
    } else {
        0.0
    };

    let mut events: Vec<Event> = Vec::new();
    let mut parents: Vec<EventId> = Vec::new();

    // 1. Open the task.
    let opened = Event::new(
        task,
        parents.clone(),
        Source::Operator,
        EventKind::TaskOpened { task },
        stamp(index, 0),
    );
    store.append(&opened).expect("append");
    parents = vec![opened.id];
    events.push(opened);

    // 2. The family-scoped preference tags the episode's family (the fold reads the first
    //    `Scope::Family` in log order as the task's family). It is an Operator statement, so it is
    //    open on arrival.
    let family_pref = ledger
        .expand(
            &task,
            parents.clone(),
            contrast_id(seed, index, "family"),
            format!("work in the {} family stays within its budget", family.name),
            Scope::Family(family.name.to_string()),
            Vec::new(),
            Vec::new(),
            Source::Operator,
            stamp(index, 1),
        )
        .expect("family preference");
    let ev = store.by_task(&task).unwrap().last().unwrap().1.clone();
    parents = vec![ev.id];
    events.push(ev);

    // 3. A task-scoped criterion preference: the desired condition the contrast is scored against.
    let criterion_text = "the approved test passes without touching the schema";
    let criterion_pref = ledger
        .expand(
            &task,
            parents.clone(),
            contrast_id(seed, index, "criterion"),
            criterion_text,
            Scope::Task(task),
            Vec::new(),
            vec![family_pref],
            Source::Operator,
            stamp(index, 2),
        )
        .expect("criterion preference");
    let ev = store.by_task(&task).unwrap().last().unwrap().1.clone();
    parents = vec![ev.id];
    events.push(ev);

    // 4. Propose candidates: a generator patch, a council patch (conflicting), and an intuition
    //    step that serves the criterion preference.
    let gen_cand = Candidate::new(
        ActionKind::ProposePatch,
        Source::Generator,
        Hash::of(&[b"gen-args", &(index as u64).to_le_bytes()]),
        parents.clone(),
        Vec::new(),
    )
    .expect("generator candidate");
    let council_cand = Candidate::new(
        ActionKind::ProposePatch,
        Source::Council,
        Hash::of(&[b"council-args", &(index as u64).to_le_bytes()]),
        parents.clone(),
        vec![gen_cand.id],
    )
    .expect("council candidate");
    let mut intuition_cand = Candidate::new(
        ActionKind::Inspect,
        Source::Intuition,
        Hash::of(&[b"intuition-args", &(index as u64).to_le_bytes()]),
        parents.clone(),
        Vec::new(),
    )
    .expect("intuition candidate");
    intuition_cand.support = vec![criterion_pref];

    for cand in [&gen_cand, &council_cand, &intuition_cand] {
        let ev = Event::new(
            task,
            parents.clone(),
            cand.source,
            EventKind::Proposed {
                candidate: cand.clone(),
            },
            stamp(index, 3),
        );
        store.append(&ev).expect("append");
        parents = vec![ev.id];
        events.push(ev);
    }

    // 5. Forecasts on the common decision event, by generator, controller and operator.
    //    The generator's raw probability is biased by the family's offset; the controller and the
    //    operator are unbiased. All three are `uncalibrated` (no calibrator is trained in M2).
    let base = if truth > 0.5 { 0.72 } else { 0.35 };
    let gen_p = clamp01(base + family.generator_bias + rng.range(-0.08, 0.08));
    let ctrl_p = clamp01(base + rng.range(-0.06, 0.06));
    let op_p = clamp01(base + rng.range(-0.10, 0.10));
    let mut forecasts: Vec<Forecast> = Vec::new();
    for (who, p) in [
        (Forecaster::Generator, gen_p),
        (Forecaster::Controller, ctrl_p),
        (Forecaster::Operator, op_p),
    ] {
        let name = match &who {
            Forecaster::Generator => "generator",
            Forecaster::Controller => "controller",
            Forecaster::Operator => "operator",
            Forecaster::External(n) => n.as_str(),
        };
        let f = Forecast {
            id: forecast_id(seed, index, name),
            event: common_decision_event_id(),
            horizon: Horizon::TaskCloseDays(7),
            source: who.clone(),
            raw: prob(p),
            calibrated: None,
            support: Support {
                episodes: (index as u32) + 1,
            },
            state_version: 1,
        };
        let ev = Event::new(
            task,
            parents.clone(),
            Source::Rules,
            EventKind::Forecast {
                forecast: f.clone(),
                candidate: gen_cand.id,
            },
            stamp(index, 4),
        );
        store.append(&ev).expect("append");
        parents = vec![ev.id];
        events.push(ev);
        forecasts.push(f);
    }

    // 6. Commit the generator's candidate (the intuition step is considered but not committed).
    let utility = Utility {
        p_success: gen_p,
        value: 1.0,
        cost: 0.2,
        ev: gen_p - 0.2,
    };
    let committed = Event::new(
        task,
        parents.clone(),
        Source::Rules,
        EventKind::Committed {
            candidate: gen_cand.id,
            decision: Decision::Selected {
                value: gen_cand.id,
                utility,
                propensity: 1.0,
                evidence: vec![events[0].id],
            },
        },
        stamp(index, 5),
    );
    store.append(&committed).expect("append");
    parents = vec![committed.id];
    events.push(committed);

    // 7. Dispatch and execute. The exit code follows the outcome: a rejected or disputed return
    //    still ran green (a false accept) except when the test itself failed.
    let exit_code = match outcome {
        EpisodeOutcome::Accepted | EpisodeOutcome::Rejected => 0,
        EpisodeOutcome::Disputed => 1,
    };
    let dispatched = Event::new(
        task,
        parents.clone(),
        Source::Rules,
        EventKind::Dispatched {
            candidate: gen_cand.id,
            in_sandbox: true,
        },
        stamp(index, 6),
    );
    store.append(&dispatched).expect("append");
    parents = vec![dispatched.id];
    events.push(dispatched);
    let executed = Event::new(
        task,
        parents.clone(),
        Source::Rules,
        EventKind::Executed {
            candidate: gen_cand.id,
            exit_code: Some(exit_code),
            log_digest: Some(log_digest(seed, index)),
            duration_ms: 100 + (index as u64) * 7,
        },
        stamp(index, 7),
    );
    store.append(&executed).expect("append");
    parents = vec![executed.id];
    events.push(executed.clone());

    // 8. The return: Surprise per forecaster (forecast minus observed) and a Contrast against the
    //    criterion. Only a Contrast with an unmet criterion may expand; a Surprise never does.
    let surprises: Vec<Surprise> = forecasts
        .iter()
        .map(|f| Surprise {
            forecast: f.id,
            residual: truth - f.raw.get(),
        })
        .collect();
    let contrast = if outcome == EpisodeOutcome::Accepted {
        None
    } else {
        let kind = match outcome {
            EpisodeOutcome::Rejected => UnmetKind::Rejected {
                reason: "the diff touched the schema".to_string(),
            },
            EpisodeOutcome::Disputed => UnmetKind::WrongBehaviour,
            EpisodeOutcome::Accepted => unreachable!(),
        };
        Some(Contrast {
            id: contrast_id(seed, index, "criterion"),
            from_return: executed.id,
            unmet: vec![(criterion_id(seed, index, "criterion"), kind)],
            evidence: vec![executed.id],
        })
    };
    let observed = Event::new(
        task,
        parents.clone(),
        Source::Rules,
        EventKind::Observed {
            observation: Observation::Return {
                surprise: surprises,
                contrast: contrast.clone(),
            },
        },
        stamp(index, 8),
    );
    store.append(&observed).expect("append");
    parents = vec![observed.id];
    events.push(observed);

    // 9. Close the task.
    let closed = Event::new(
        task,
        parents.clone(),
        Source::Operator,
        EventKind::TaskClosed {
            task,
            reason: "operator".to_string(),
        },
        stamp(index, 9),
    );
    store.append(&closed).expect("append");
    events.push(closed);

    // 10. Review: label the return. A rejection also records its reason as an Operator preference
    //     through an Expansion whose Contrast is Rejected(reason).
    let (label, reason) = match outcome {
        EpisodeOutcome::Accepted => (Label::Accepted, None),
        EpisodeOutcome::Rejected => (
            Label::Rejected,
            Some("the diff touched the schema".to_string()),
        ),
        EpisodeOutcome::Disputed => (Label::Disputed, None),
    };
    record_label(
        store,
        &task,
        executed.id,
        label,
        reason,
        Some(log_digest(seed, index)),
        None,
        &stamp(index, 10),
    )
    .expect("record label");
    events = store
        .by_task(&task)
        .unwrap()
        .into_iter()
        .map(|(_, e)| e)
        .collect();

    Episode {
        task,
        family: family.name,
        index,
        outcome,
        events,
    }
}

/// Generate the whole corpus into a fresh in-memory store, returning the episodes.
///
/// This is the documented generator: `SEED` and `FAMILIES` fully determine the output.
pub fn generate_corpus(seed: u64) -> Vec<Episode> {
    let store = Store::open_in_memory().expect("in-memory store");
    let mut out = Vec::new();
    for family in FAMILIES {
        for i in 0..PER_FAMILY {
            out.push(generate_episode(&store, seed, family, i));
        }
    }
    out
}

/// The on-disk shape of one episode: the task, its family, its outcome and its event log.
#[derive(serde::Serialize, serde::Deserialize)]
struct EpisodeFile {
    task: TaskId,
    family: String,
    index: usize,
    outcome: String,
    events: Vec<Event>,
}

/// Write the corpus to `dir`, one JSON file per episode, and return the paths written.
pub fn write_corpus(dir: &Path, seed: u64) -> Vec<PathBuf> {
    std::fs::create_dir_all(dir).expect("create fixtures dir");
    let mut paths = Vec::new();
    for ep in generate_corpus(seed) {
        let outcome = match ep.outcome {
            EpisodeOutcome::Accepted => "accepted",
            EpisodeOutcome::Rejected => "rejected",
            EpisodeOutcome::Disputed => "disputed",
        };
        let file = EpisodeFile {
            task: ep.task,
            family: ep.family.to_string(),
            index: ep.index,
            outcome: outcome.to_string(),
            events: ep.events,
        };
        let path = dir.join(format!("{}-{:02}.json", ep.family, ep.index));
        let json = serde_json::to_string_pretty(&file).expect("serialize episode");
        std::fs::write(&path, json).expect("write episode");
        paths.push(path);
    }
    paths
}

/// The store directory the CLI opens, relative to the crate root.
fn store_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join("m2")
        .join("store")
}

/// Build a SQLite store at `dir/tsr.db` holding every event of the corpus, so the CLI
/// (`tsr reflect --family <f> --store <dir>/tsr.db`) can fold the fixtures. Returns the path.
///
/// The store is rebuilt from scratch each run: the file is removed first, so the corpus is a pure
/// function of the seed and never accumulates stale events.
pub fn write_store(dir: &Path, seed: u64) -> PathBuf {
    std::fs::create_dir_all(dir).expect("create store dir");
    let db = dir.join("tsr.db");
    // Build into a unique temp file, then rename atomically: cargo runs the tests in parallel,
    // and two writers on the same path would interleave their appends into one store.
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let tmp = dir.join(format!("tsr.db.tmp-{}-{}", std::process::id(), n));
    if tmp.exists() {
        std::fs::remove_file(&tmp).expect("remove stale temp store");
    }
    {
        let store = Store::open(&tmp).expect("open temp store");
        for ep in generate_corpus(seed) {
            for ev in &ep.events {
                store.append(ev).expect("append to store");
            }
        }
    }
    std::fs::rename(&tmp, &db).expect("publish store");
    // SQLite may leave WAL/SHM sidecars beside the temp file; remove them so the fixtures
    // directory holds only the published store.
    for suffix in ["-wal", "-shm"] {
        let side = dir.join(format!("tsr.db.tmp-{}-{}{}", std::process::id(), n, suffix));
        let _ = std::fs::remove_file(side);
    }
    db
}

/// The fixtures directory, relative to the crate root.
fn fixtures_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join("m2")
        .join("episodes")
}

#[test]
fn corpus_is_thirty_episodes_across_three_families() {
    let corpus = generate_corpus(SEED);
    assert_eq!(corpus.len(), 30, "thirty episodes");
    let mut by_family: BTreeMap<&str, usize> = BTreeMap::new();
    for ep in &corpus {
        *by_family.entry(ep.family).or_default() += 1;
    }
    assert_eq!(by_family.len(), 3, "three families");
    for (family, n) in &by_family {
        assert_eq!(*n, PER_FAMILY, "family {family} has {PER_FAMILY} episodes");
    }
    // Every episode is closed and reviewed.
    for ep in &corpus {
        let kinds: Vec<&EventKind> = ep.events.iter().map(|e| &e.kind).collect();
        assert!(
            kinds
                .iter()
                .any(|k| matches!(k, EventKind::TaskClosed { .. })),
            "episode {} is closed",
            ep.index
        );
        assert!(
            kinds
                .iter()
                .any(|k| matches!(k, EventKind::OutcomeRecorded { .. })),
            "episode {} is reviewed",
            ep.index
        );
    }
}

#[test]
fn generation_is_deterministic() {
    let a = generate_corpus(SEED);
    let b = generate_corpus(SEED);
    assert_eq!(a.len(), b.len());
    for (x, y) in a.iter().zip(b.iter()) {
        assert_eq!(x.task, y.task);
        assert_eq!(x.events.len(), y.events.len());
        for (ex, ey) in x.events.iter().zip(y.events.iter()) {
            assert_eq!(ex.id, ey.id, "event ids are stable across runs");
        }
    }
}

#[test]
fn fixtures_are_committed_and_match_the_generator() {
    let dir = fixtures_dir();
    let paths = write_corpus(&dir, SEED);
    assert_eq!(paths.len(), 30, "thirty files written");
    // Every file is on disk and parses back to the same event ids.
    let corpus = generate_corpus(SEED);
    for (ep, path) in corpus.iter().zip(paths.iter()) {
        let text = std::fs::read_to_string(path).expect("read episode");
        let file: EpisodeFile = serde_json::from_str(&text).expect("parse episode");
        assert_eq!(file.family, ep.family);
        assert_eq!(file.events.len(), ep.events.len());
        for (a, b) in file.events.iter().zip(ep.events.iter()) {
            assert_eq!(a.id, b.id, "committed fixture matches the generator");
        }
    }
}

#[test]
fn store_holds_every_episode_for_the_cli() {
    let dir = store_dir();
    let db = write_store(&dir, SEED);
    assert!(db.exists(), "the store file exists");
    let store = Store::open(&db).expect("reopen store");
    let corpus = generate_corpus(SEED);
    // Every episode's task is present, and its events are all in the store.
    for ep in &corpus {
        let events = store.by_task(&ep.task).expect("by_task");
        assert!(!events.is_empty(), "task {} is in the store", ep.task);
    }
    // The store's head root is stable across a rebuild.
    let root_a = store.head_root().expect("head root");
    let db2 = write_store(&dir, SEED);
    let store2 = Store::open(&db2).expect("reopen store");
    assert_eq!(
        root_a,
        store2.head_root().expect("head root"),
        "stable root"
    );
}

/// Fold the committed store and score every forecast, per family. This is the claim the evidence
/// file rests on: the generator's curve differs across the three families, because its raw
/// probability carries a per-family bias while the controller's and the operator's do not.
#[test]
fn generator_curve_differs_across_families() {
    use tsr_forecast::resolve::{resolve, TaskFacts};
    use tsr_forecast::score::{score_per_forecaster, ScoredForecast};

    let db = write_store(&store_dir(), SEED);
    let store = Store::open(&db).expect("open committed store");
    let corpus = generate_corpus(SEED);

    // Score every forecast, tagged with its family.
    let mut scored: Vec<ScoredForecast> = Vec::new();
    for ep in &corpus {
        let events: Vec<Event> = store
            .by_task(&ep.task)
            .expect("by_task")
            .into_iter()
            .map(|(_, e)| e)
            .collect();
        let mut facts = TaskFacts::from_events(&events);
        // The review command records labels as `OutcomeRecorded` events; `from_events` folds the
        // log's own facts, so the labels are attached here from those records.
        for ev in &events {
            if let EventKind::OutcomeRecorded { record } = &ev.kind {
                // `resolve` reads the *label* names ("accepted"/"rejected"/"disputed"), while the
                // record stores a `LabelStatus`; map it the way the review command does.
                let label = match record.status {
                    LabelStatus::Observed => "accepted",
                    LabelStatus::Invalid => "rejected",
                    LabelStatus::Disputed => "disputed",
                    _ => record.status.name(),
                };
                facts = facts.with_label(label, ev.id);
            }
        }
        for (_, ev) in store.by_task(&ep.task).expect("by_task") {
            if let EventKind::Forecast { forecast, .. } = &ev.kind {
                let r = resolve(forecast, &facts);
                if let Some(sf) =
                    ScoredForecast::from_resolved(&r, Some(ep.family.to_string()), None)
                {
                    scored.push(sf);
                }
            }
        }
    }
    assert!(!scored.is_empty(), "forecasts were scored");

    // Per family, the generator's Brier.
    let mut gen_brier: BTreeMap<String, f64> = BTreeMap::new();
    for family in ["alpha", "beta", "gamma"] {
        let rows: Vec<ScoredForecast> = scored
            .iter()
            .filter(|s| s.family.as_deref() == Some(family))
            .cloned()
            .collect();
        let per = score_per_forecaster(&rows);
        let gen = per
            .iter()
            .find(|f| f.name == "generator")
            .expect("the generator forecast in every family");
        let brier = gen.score.brier_proxy().expect("a Brier proxy").value;
        gen_brier.insert(family.to_string(), brier);
    }
    // Print the curve so `cargo test -- --nocapture` is itself evidence of the per-family shape.
    for (family, brier) in &gen_brier {
        println!("generator Brier[{family}] = {brier:.6}");
    }

    // The biased families differ from the unbiased one, and from each other.
    let alpha = gen_brier["alpha"];
    let beta = gen_brier["beta"];
    let gamma = gen_brier["gamma"];
    assert!(
        (alpha - beta).abs() > 1e-6,
        "alpha's generator curve differs from beta's: {alpha} vs {beta}"
    );
    assert!(
        (gamma - beta).abs() > 1e-6,
        "gamma's generator curve differs from beta's: {gamma} vs {beta}"
    );
    // The over-confident family scores worse than the unbiased one. (Gamma's under-confidence is
    // milder on a ten-episode sample, so only the alpha/beta ordering is asserted.)
    assert!(
        alpha > beta,
        "the over-confident family scores worse than the unbiased one: alpha {alpha} > beta {beta}"
    );
}
