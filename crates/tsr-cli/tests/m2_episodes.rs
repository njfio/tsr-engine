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
    ActionKind, Candidate, ContrastId, Decision, Event, EventId, EventKind, Forecast, ForecastId,
    Forecaster, Hash, Horizon, LabelStatus, Observation, Probability, Scope, Source, Support,
    Surprise, TaskId, UnmetKind, Utility,
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

/// Record a Contrast the log holds, so a later `expand` may name it (the Filter's write rule
/// refuses an expansion from a contrast the log does not carry). D12: a Contrast's id is its
/// content address, so the contrast is minted through the Filter and its id returned.
fn record_contrast(
    store: &Store,
    task: &TaskId,
    parents: Vec<EventId>,
    criterion: tsr_domain::CriterionId,
    recorded_at: &str,
) -> ContrastId {
    let c = tsr_ledger::Filter::contrast_from_criteria(
        EventId(Hash::of(&[b"m2-return", &criterion.0 .0])),
        vec![(
            criterion,
            tsr_domain::UnmetKind::Rejected {
                reason: "the criterion went unmet".to_string(),
            },
        )],
        vec![],
    );
    let id = c.id;
    Ledger::new(store)
        .record_contrast(task, parents, c, recorded_at)
        .expect("record the contrast");
    id
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
    let family_contrast = record_contrast(
        store,
        &task,
        parents.clone(),
        criterion_id(seed, index, "family"),
        &stamp(index, 1),
    );
    let family_pref = ledger
        .expand(
            &task,
            parents.clone(),
            family_contrast,
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
    let criterion_contrast = record_contrast(
        store,
        &task,
        parents.clone(),
        criterion_id(seed, index, "criterion"),
        &stamp(index, 2),
    );
    let criterion_pref = ledger
        .expand(
            &task,
            parents.clone(),
            criterion_contrast,
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
    //    D11: the residual is `raw p - observed`, the same sign the Filter and the scorer use.
    let surprises: Vec<Surprise> = forecasts
        .iter()
        .map(|f| Surprise {
            forecast: f.id,
            residual: f.raw.get() - truth,
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
        Some(tsr_ledger::Filter::contrast_from_criteria(
            executed.id,
            vec![(criterion_id(seed, index, "criterion"), kind)],
            vec![executed.id],
        ))
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

    // 9. Close the task. R3: the close reason must match the episode's story. Every generated
    //    episode is reviewed (accepted, rejected or disputed), so every one closes `done`; the
    //    `operator` close is reserved for a censored episode (closed with no review), of which the
    //    corpus has none. Closing every episode `operator` censored the whole corpus, so the
    //    accepted episodes resolved false and every reliability bin read `observed 0.0000`.
    let close_reason = match outcome {
        EpisodeOutcome::Accepted | EpisodeOutcome::Rejected | EpisodeOutcome::Disputed => "done",
    };
    let closed = Event::new(
        task,
        parents.clone(),
        Source::Operator,
        EventKind::TaskClosed {
            task,
            reason: close_reason.to_string(),
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

/// The store directory the CLI opens, relative to the crate root.
fn store_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("fixtures")
        .join("m2")
        .join("store")
}

/// Whether the fixtures should be regenerated. Regeneration is explicit: a plain `cargo test`
/// must never rewrite the committed corpus or the store, so a stale fixture fails the comparison
/// instead of being silently refreshed. Set `TSR_REGEN_FIXTURES=1` to regenerate.
fn regen() -> bool {
    std::env::var("TSR_REGEN_FIXTURES").as_deref() == Ok("1")
}

/// The committed store path (`fixtures/m2/store/tsr.db`). It is gitignored — a build artifact the
/// tests compare against, never a source file.
fn committed_store() -> PathBuf {
    store_dir().join("tsr.db")
}

/// A fresh scratch directory under the system temp dir, unique per call. The store tests build
/// here so a plain run never touches the committed store.
fn scratch_dir(tag: &str) -> PathBuf {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!("tsr-m2-{}-{}-{}", tag, std::process::id(), n));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create scratch dir");
    dir
}

/// Build a SQLite store at `dir/tsr.db` holding every event of the corpus, so the CLI
/// (`tsr reflect --family <f> --store <dir>/tsr.db`) can fold the fixtures. Returns the path.
///
/// The store is rebuilt from scratch each run: the file is removed first, so the corpus is a pure
/// function of the seed and never accumulates stale events. Callers pass a scratch directory; the
/// committed store is written only under `TSR_REGEN_FIXTURES=1`.
pub fn build_store(dir: &Path, seed: u64) -> PathBuf {
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

/// Render one episode to the exact bytes `write_corpus` would write for it.
fn render_episode(ep: &Episode) -> String {
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
        events: ep.events.clone(),
    };
    serde_json::to_string_pretty(&file).expect("serialize episode")
}

/// The committed fixtures must be byte-identical to what the generator produces.
///
/// This reads the committed files **before** writing anything, so it can actually fail on a stale
/// corpus: a test that regenerates first and then compares against its own output proves nothing.
#[test]
fn fixtures_are_committed_and_match_the_generator() {
    let dir = fixtures_dir();
    let corpus = generate_corpus(SEED);
    assert_eq!(corpus.len(), 30, "thirty episodes");

    // 0. Regeneration is explicit. A plain `cargo test` never writes the corpus: it reads the
    //    committed bytes first and compares them to the generator's output, so a stale corpus
    //    fails instead of being silently refreshed. `TSR_REGEN_FIXTURES=1` writes it.
    if regen() {
        std::fs::create_dir_all(&dir).expect("create fixtures dir");
        for ep in &corpus {
            let path = dir.join(format!("{}-{:02}.json", ep.family, ep.index));
            std::fs::write(&path, render_episode(ep)).expect("write episode");
        }
    }

    // 1. Read the committed bytes first, and compare them to the generator's output.
    for ep in &corpus {
        let path = dir.join(format!("{}-{:02}.json", ep.family, ep.index));
        let committed = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("committed fixture {} is missing: {e}", path.display()));
        let expected = render_episode(ep);
        assert_eq!(
            committed,
            expected,
            "committed fixture {} differs from the generator (stale corpus?)",
            path.display()
        );
    }

    // 2. Every committed file parses back to the same event ids.
    for ep in &corpus {
        let path = dir.join(format!("{}-{:02}.json", ep.family, ep.index));
        let text = std::fs::read_to_string(&path).expect("read episode");
        let file: EpisodeFile = serde_json::from_str(&text).expect("parse episode");
        assert_eq!(file.family, ep.family);
        assert_eq!(file.events.len(), ep.events.len());
        for (a, b) in file.events.iter().zip(ep.events.iter()) {
            assert_eq!(a.id, b.id, "committed fixture matches the generator");
        }
    }

    // 3. There are exactly thirty committed episode files, and no strays.
    let mut committed: Vec<String> = std::fs::read_dir(&dir)
        .expect("read fixtures dir")
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".json"))
        .collect();
    committed.sort();
    assert_eq!(committed.len(), 30, "thirty committed episode files");
}

/// D11: every Surprise residual in the committed fixtures is forecast minus observed.
///
/// The generator used to write `truth - f.raw.get()` under a comment saying "forecast minus
/// observed", so all 90 Surprise records in `fixtures/m2/episodes/*.json` were observed minus
/// forecast and the Mirror printed the negated residual. This test reads the committed JSON (not
/// the generator) and checks the sign against the library's rule: `residual == raw - observed`,
/// where `observed` is 1 for an accepted episode and 0 otherwise.
#[test]
fn d11_fixture_surprise_residual_is_forecast_minus_observed() {
    let dir = fixtures_dir();
    let mut checked = 0usize;
    for family in ["alpha", "beta", "gamma"] {
        for index in 0..PER_FAMILY {
            let path = dir.join(format!("{family}-{index:02}.json"));
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|e| panic!("committed fixture {} is missing: {e}", path.display()));
            let file: EpisodeFile = serde_json::from_str(&text).expect("parse episode");
            let observed = if file.outcome == "accepted" { 1.0 } else { 0.0 };

            // The raw probability of every forecast in this episode, by id.
            let mut raw: BTreeMap<ForecastId, f64> = BTreeMap::new();
            for ev in &file.events {
                if let EventKind::Forecast { forecast, .. } = &ev.kind {
                    raw.insert(forecast.id, forecast.raw.get());
                }
            }

            for ev in &file.events {
                if let EventKind::Observed {
                    observation: Observation::Return { surprise, .. },
                } = &ev.kind
                {
                    for s in surprise {
                        let p = raw.get(&s.forecast).copied().unwrap_or_else(|| {
                            panic!(
                                "{}: Surprise names forecast {} with no Forecast event",
                                path.display(),
                                s.forecast
                            )
                        });
                        let expected = p - observed;
                        assert!(
                            (s.residual - expected).abs() < 1e-12,
                            "D11: {} forecast {} residual {} is not forecast minus observed \
                             (raw {p} - observed {observed} = {expected})",
                            path.display(),
                            s.forecast,
                            s.residual
                        );
                        checked += 1;
                    }
                }
            }
        }
    }
    assert_eq!(checked, 90, "ninety Surprise records across the corpus");
}

#[test]
fn store_holds_every_episode_for_the_cli() {
    let corpus = generate_corpus(SEED);

    // Build the corpus into a scratch directory. The committed store is never overwritten by a
    // plain run: it is compared against, and only `TSR_REGEN_FIXTURES=1` publishes it.
    let scratch = scratch_dir("store-holds");
    let db = build_store(&scratch, SEED);
    assert!(db.exists(), "the store file exists");
    let store = Store::open(&db).expect("reopen store");
    // Every episode's task is present, and its events are all in the store.
    for ep in &corpus {
        let events = store.by_task(&ep.task).expect("by_task");
        assert!(!events.is_empty(), "task {} is in the store", ep.task);
    }
    // The store's head root is stable across a rebuild.
    let root_a = store.head_root().expect("head root");
    let scratch2 = scratch_dir("store-holds-2");
    let db2 = build_store(&scratch2, SEED);
    let store2 = Store::open(&db2).expect("reopen store");
    assert_eq!(
        root_a,
        store2.head_root().expect("head root"),
        "stable root"
    );

    // The committed store, when present, must fold to the same head root: the scratch build is
    // the corpus, and the committed store is that corpus. Regenerate it only on request.
    let committed = committed_store();
    if regen() {
        std::fs::create_dir_all(store_dir()).expect("create store dir");
        std::fs::copy(&db, &committed).expect("publish committed store");
    }
    if committed.exists() {
        let c = Store::open(&committed).expect("open committed store");
        assert_eq!(
            c.head_root().expect("committed head root"),
            root_a,
            "the committed store folds to the same root as a fresh build"
        );
    }
}

/// D6: a plain `cargo test` must not rewrite the committed fixtures. This test snapshots the
/// committed corpus JSON and the committed store, runs the comparison path, and asserts both are
/// byte-identical afterwards. Regeneration happens only under `TSR_REGEN_FIXTURES=1`.
#[test]
fn d6_fixture_compare_is_read_only() {
    // R6: under `TSR_REGEN_FIXTURES=1` the documented regeneration run must PASS, not exit 101.
    // Regeneration is *supposed* to rewrite the committed fixtures, so the read-only property is
    // only meaningful without it: skip the assertion when regen is requested.
    if regen() {
        return;
    }

    // Snapshot the committed corpus and store before the comparison path runs.
    let corpus = generate_corpus(SEED);
    let mut before_json: Vec<(PathBuf, Vec<u8>)> = Vec::new();
    for ep in &corpus {
        let path = fixtures_dir().join(format!("{}-{:02}.json", ep.family, ep.index));
        before_json.push((
            path.clone(),
            std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display())),
        ));
    }
    let committed = committed_store();
    let before_store = std::fs::read(&committed).ok();

    // Run the comparison path exactly as the other tests do: compare the corpus read-only and
    // build the store into a scratch directory.
    for ep in &corpus {
        let path = fixtures_dir().join(format!("{}-{:02}.json", ep.family, ep.index));
        let committed_bytes = std::fs::read_to_string(&path).expect("read committed fixture");
        assert_eq!(
            committed_bytes,
            render_episode(ep),
            "committed fixture {} differs from the generator",
            path.display()
        );
    }
    let scratch = scratch_dir("d6-readonly");
    let db = build_store(&scratch, SEED);
    assert!(db.exists(), "the scratch store was built");

    // The committed JSON is unchanged.
    for (path, before) in &before_json {
        let after = std::fs::read(path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
        assert_eq!(
            &after,
            before,
            "the comparison path rewrote the committed fixture {}",
            path.display()
        );
    }
    // The committed store is unchanged (or still absent).
    let after_store = std::fs::read(&committed).ok();
    assert_eq!(
        after_store,
        before_store,
        "the comparison path rewrote the committed store {}",
        committed.display()
    );
}

/// Fold the committed store and score every forecast, per family. This is the claim the evidence
/// file rests on: the generator's curve differs across the three families, because its raw
/// probability carries a per-family bias while the controller's and the operator's do not.
#[test]
fn generator_curve_differs_across_families() {
    use tsr_forecast::resolve::{resolve, TaskFacts};
    use tsr_forecast::score::{score_per_forecaster, ScoredForecast};

    // Read the committed store read-only. A plain run never rewrites it; regenerate only on
    // request, and otherwise build a scratch store if the committed one is absent.
    let committed = committed_store();
    if regen() {
        std::fs::create_dir_all(store_dir()).expect("create store dir");
        let built = build_store(&scratch_dir("curve-regen"), SEED);
        std::fs::copy(&built, &committed).expect("publish committed store");
    }
    let db = if committed.exists() {
        committed
    } else {
        build_store(&scratch_dir("curve"), SEED)
    };
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
                    LabelStatus::Rejected => "rejected",
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

/// R3: the fixture families' calibration curves must show a non-zero observed rate.
///
/// Every fixture episode used to close `TaskClosed { reason: "operator" }`, which censors the
/// common decision event, so every reliability bin of `tsr reflect --family` read
/// `observed 0.0000` — the M2 exit evidence was an artifact. With the episodes closing as their
/// story says (`done` for the reviewed ones), each family's curve carries real observations.
#[test]
fn r3_fixture_family_curves_have_nonzero_observed() {
    use tsr_cli::reflect_cmd::build_family_curves;

    let committed = committed_store();
    let db = if committed.exists() {
        committed
    } else {
        build_store(&scratch_dir("r3-curve"), SEED)
    };
    let store = Store::open(&db).expect("open committed store");
    let events = store.all().expect("all events");

    for family in ["alpha", "beta", "gamma"] {
        let curves = build_family_curves(family, &events);
        assert!(
            !curves.curves.is_empty(),
            "family {family} has at least one forecaster curve"
        );
        let mut observed_total = 0.0f64;
        let mut bins_with_observations = 0usize;
        for curve in &curves.curves {
            for bin in &curve.bins {
                if let Some(o) = bin.observed {
                    observed_total += o;
                    if o > 0.0 {
                        bins_with_observations += 1;
                    }
                }
            }
        }
        assert!(
            bins_with_observations > 0,
            "R3: family {family}'s curve has a bin with a non-zero observed rate \
             (observed_total={observed_total})"
        );
    }
}
