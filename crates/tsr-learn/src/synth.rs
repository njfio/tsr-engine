//! The synthetic corpus (lane 06, item 2): a seeded, clock-free generator that drives
//! [`tsr_store::Store::append`] through the honest M0 sequence and writes `fixtures/m4/`.
//!
//! # What this is
//!
//! A corpus of tasks whose *true* acceptance model is known, so the calibration half of M4 can be
//! exercised end to end without waiting for lanes 04 and 05 to merge. Every number here is
//! synthetic; the real exit evidence (controller Brier below generator Brier on M3's mined corpus)
//! waits for those lanes and the operator's runs (`docs/m4-evidence.md`).
//!
//! # The true acceptance model
//!
//! For a task in family `f` with `inspections` inspections before the commitment, the common
//! decision event `task_accepted_in_budget@v1` is a Bernoulli draw:
//!
//! ```text
//!   z_true(t) = base(f) + INSPECT_GAIN * min(inspections, 3)
//!   p_true(t) = sigmoid(z_true(t))
//!   accepted  ~ Bernoulli(p_true(t))
//! ```
//!
//! The generator's *raw* forecast is a distorted view of the same shape:
//!
//! ```text
//!   logit(p_gen) = distortion(f) + INSPECT_GAIN * min(inspections, 3)
//! ```
//!
//! so the generator is overconfident when `distortion(f) > base(f)`, calibrated when equal, and
//! underconfident when lower. The three main families are chosen so that, **with little inspected**
//! (`inspections == 0`), the overconfident family's generator says about `0.9` while the family
//! actually succeeds about `0.4` of the time — §B8's example:
//!
//! | family          | base(f) | distortion(f) | p_true at 0 insp | p_gen at 0 insp |
//! |-----------------|---------|---------------|------------------|-----------------|
//! | `overconfident` | -0.40   | +2.20         | 0.401            | 0.900           |
//! | `calibrated`    |  0.00   |  0.00         | 0.500            | 0.500           |
//! | `underconfident`|  0.85   | -0.85         | 0.701            | 0.300           |
//! | `rare`          | -0.85   | -0.85         | 0.300            | 0.300           |
//!
//! The fourth family, `rare`, exists only to give the per-family minimum (§B8: "≥ 30 per family")
//! a family with a different base rate; it is calibrated by construction.
//!
//! # Shape
//!
//! - three families of 120 tasks (`overconfident`, `calibrated`, `underconfident`) plus a fourth of
//!   20 (`rare`) — 380 tasks;
//! - twelve lineages (`lineage-00` … `lineage-11`), so the cluster bootstrap has clusters;
//! - two to four gated decisions per task;
//! - some tasks censored (never reviewed, or an operator stop) and some still open.
//!
//! # Seed and determinism
//!
//! Everything is drawn from one [`SplitMix64`] seeded with [`SEED`]. No clock is read: every
//! event's `recorded_at` is a fixed, derived timestamp, so the event ids — which hash `recorded_at`
//! in — are reproducible byte for byte. `crates/tsr-cli/tests/m4_corpus.rs` regenerates into a temp
//! dir and asserts byte-identity with `fixtures/m4/`.
//!
//! # The sequence
//!
//! Per task: `TaskOpened` → (`Inspect` proposals) → for each gated decision: `Forecast` (generator
//! always, operator on some rows) → `Committed` → `Dispatched { in_sandbox: true }` → `Executed` →
//! then a close (`TaskClosed`) or a stop (`Stop`) or nothing (still open). Every gated commitment
//! has a forecast at the same state version, every dispatch is reconciled, and every dispatch is in
//! the sandbox — the M0 invariants `tsr_cli::reconstruct` checks.

use std::path::Path;

use tsr_domain::{
    ActionKind, Candidate, CandidateId, Decision, Event, EventId, EventKind, Forecast, Forecaster,
    Hash, Horizon, Probability, Source, TaskId, Utility,
};
use tsr_store::Store;

use crate::decision::LabelRecord;
use crate::rng::SplitMix64;
use crate::LearnError;

/// The one seed the whole corpus is drawn from. Changing it changes every fixture byte.
pub const SEED: u64 = 0x4D34_0000_0000_0001;

/// The shared inspection gain: each inspection (up to three) raises the logit by this much.
pub const INSPECT_GAIN: f64 = 0.9;

/// The operator's name in the labels input. A record whose reviewer is not this is refused (B8).
pub const OPERATOR: &str = "operator";

/// A family of the synthetic corpus: its name, its true base logit and its generator distortion.
#[derive(Clone, Copy, Debug)]
pub struct Family {
    pub name: &'static str,
    /// The true base logit `base(f)`.
    pub base: f64,
    /// The generator's distortion `distortion(f)`.
    pub distortion: f64,
    /// How many tasks the family holds.
    pub tasks: usize,
}

/// The four families, in name order. The first three are the §B8 shape; `rare` gives the per-family
/// minimum a fourth base rate.
pub const FAMILIES: [Family; 4] = [
    Family {
        name: "calibrated",
        base: 0.0,
        distortion: 0.0,
        tasks: 120,
    },
    Family {
        name: "overconfident",
        base: -0.40,
        distortion: 2.20,
        tasks: 120,
    },
    Family {
        name: "rare",
        base: -0.85,
        distortion: -0.85,
        tasks: 20,
    },
    Family {
        name: "underconfident",
        base: 0.85,
        distortion: -0.85,
        tasks: 120,
    },
];

/// The number of lineages. At least eight, so the cluster bootstrap has clusters to resample.
pub const LINEAGES: usize = 12;

/// The sigmoid.
fn sigmoid(z: f64) -> f64 {
    1.0 / (1.0 + (-z).exp())
}

/// The true acceptance probability of a task in `family` with `inspections` inspections so far.
pub fn p_true(family: &Family, inspections: u64) -> f64 {
    sigmoid(family.base + INSPECT_GAIN * inspections.min(3) as f64)
}

/// The generator's raw probability for a task in `family` with `inspections` inspections so far.
pub fn p_generator(family: &Family, inspections: u64) -> f64 {
    sigmoid(family.distortion + INSPECT_GAIN * inspections.min(3) as f64)
}

/// A fixed, derived RFC 3339 timestamp: the epoch advanced by `task_index` days and `step` seconds.
///
/// No clock is read; the value is a pure function of the task's position in the corpus and the
/// event's position in the task, so the fixture is byte-reproducible.
fn stamp(task_index: usize, step: usize) -> String {
    let total_secs = task_index as i64 * 86_400 + step as i64;
    let (y, mo, d, h, mi, s) = civil_from_epoch(1_769_904_000 + total_secs);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

/// Seconds since the epoch → (year, month, day, hour, minute, second), UTC. Howard Hinnant's
/// `civil_from_days`, so the calendar is exact and platform-independent.
fn civil_from_epoch(secs: i64) -> (i64, u32, u32, u32, u32, u32) {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let mo = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (
        if mo <= 2 { y + 1 } else { y },
        mo,
        d,
        h as u32,
        m as u32,
        s as u32,
    )
}

/// The task id for a synthetic task name, minted exactly as `tsr task open --name` mints it:
/// `TaskId(Hash::of(&[b"task:", name]))`.
pub fn task_id(name: &str) -> TaskId {
    TaskId(Hash::of(&[b"task:", name.as_bytes()]))
}

/// The args digest of a synthetic tool call: `Hash::of(&[kind_tag, args])`, so a candidate's
/// identity is `(kind, args digest)` and two calls with the same args share a target.
fn args_digest(kind: ActionKind, args: &str) -> Hash {
    let tag = match kind {
        ActionKind::Inspect => "inspect",
        ActionKind::RunApprovedTest => "run_approved_test",
        ActionKind::Generate => "generate",
        ActionKind::ProposePatch => "propose_patch",
        ActionKind::RequestApproval => "request_approval",
        ActionKind::Stop => "stop",
    };
    Hash::of(&[tag.as_bytes(), args.as_bytes()])
}

/// Mint a candidate for a synthetic tool call, exactly as `tsr-cli`'s `mint_candidate` does.
fn mint(kind: ActionKind, args: &str) -> Candidate {
    let source = if kind == ActionKind::Generate {
        Source::Generator
    } else {
        Source::Rules
    };
    Candidate::new(kind, source, args_digest(kind, args), vec![], vec![])
        .expect("a synthetic candidate is well-formed")
}

/// One task's index row, as written to `tasks.jsonl`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct SynthTaskRow {
    pub task: TaskId,
    pub family: String,
    pub lineage: String,
    pub timestamp: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clone_group: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_ms_ceiling: Option<u64>,
}

/// The result of generating the corpus: the side files (the store is written to disk).
pub struct Corpus {
    /// The task index rows, in generation order.
    pub tasks: Vec<SynthTaskRow>,
    /// The label records, in generation order.
    pub labels: Vec<LabelRecord>,
}

/// How a task ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Terminal {
    /// `TaskClosed` and a label record (reviewed).
    ClosedReviewed,
    /// `TaskClosed` but no label record (never reviewed → censored).
    ClosedUnreviewed,
    /// `Stop { reason: "operator" }` (censored).
    OperatorStop,
    /// No close and no stop (still open → unresolved).
    Open,
}

/// The per-task plan the generator draws before it writes any event.
struct TaskPlan {
    name: String,
    family: Family,
    lineage: String,
    clone_group: Option<String>,
    tool_ms_ceiling: Option<u64>,
    /// The number of inspections before the first gated decision.
    inspections: u64,
    /// The number of gated decisions (2..=4).
    decisions: usize,
    /// Whether the operator also forecasts each decision.
    operator_forecasts: bool,
    /// The terminal state.
    terminal: Terminal,
    /// The true acceptance of each decision, drawn from `p_true`.
    accepted: Vec<bool>,
}

/// Draw one task's plan from the stream.
fn plan_task(rng: &mut SplitMix64, family: Family, index: usize) -> TaskPlan {
    let name = format!("m4-{}-{:03}", family.name, index);
    let lineage = format!("lineage-{:02}", rng.index(LINEAGES));
    // A clone group on roughly a third of tasks, so the split's clone-group rule is exercised.
    let clone_group = if rng.next_f64() < 0.33 {
        Some(format!("clone-{:03}", rng.index(40)))
    } else {
        None
    };
    // A tool-time ceiling on roughly half the tasks, so the budget-fraction feature has both a
    // present and a missing indicator.
    let tool_ms_ceiling = if rng.next_f64() < 0.5 {
        Some(60_000 + rng.below(240_000))
    } else {
        None
    };
    // Little inspected on most tasks, so the overconfident family's distortion is visible.
    let inspections = match rng.below(4) {
        0 => 0,
        1 => 1,
        2 => 2,
        _ => 3,
    };
    let decisions = 2 + rng.index(3); // 2..=4
    let operator_forecasts = rng.next_f64() < 0.5;
    let terminal = match rng.below(10) {
        0 => Terminal::ClosedUnreviewed,
        1 => Terminal::OperatorStop,
        2 => Terminal::Open,
        _ => Terminal::ClosedReviewed,
    };
    // The true acceptance of each decision: the first decision sees `inspections` inspections; each
    // later decision sees one more (the task inspects between decisions).
    let mut accepted = Vec::with_capacity(decisions);
    for d in 0..decisions {
        let insp = inspections + d as u64;
        let p = p_true(&family, insp);
        accepted.push(rng.next_f64() < p);
    }
    TaskPlan {
        name,
        family,
        lineage,
        clone_group,
        tool_ms_ceiling,
        inspections,
        decisions,
        operator_forecasts,
        terminal,
        accepted,
    }
}

/// The generator's raw probability for a decision, as a `Probability`.
fn gen_prob(family: &Family, inspections: u64) -> Probability {
    Probability::new(p_generator(family, inspections)).expect("sigmoid is in [0, 1]")
}

/// The operator's raw probability: the generator's, nudged toward the truth by a fixed amount, so
/// the operator is a distinct forecaster with a distinct (better) calibration.
fn op_prob(family: &Family, inspections: u64) -> Probability {
    let p = 0.5 * p_generator(family, inspections) + 0.5 * p_true(family, inspections);
    Probability::new(p).expect("a convex combination is in [0, 1]")
}

/// The utility the hook writes on a commitment: `p_success` is a placeholder 0.5 (main's hook), and
/// the features never read it.
fn placeholder_utility() -> Utility {
    Utility {
        p_success: 0.5,
        value: 1.0,
        cost: 0.0,
        ev: 0.5,
    }
}

/// Append one event to the store, chaining it to the task's last event, with a fixed timestamp.
///
/// Returns the event's id. `step` is the event's position within the task, so the timestamp is a
/// pure function of `(task_index, step)`.
fn append(
    store: &Store,
    task: &TaskId,
    parents: Vec<EventId>,
    source: Source,
    kind: EventKind,
    task_index: usize,
    step: usize,
) -> Result<EventId, LearnError> {
    let ev = Event::new(*task, parents, source, kind, stamp(task_index, step));
    store.append(&ev)?;
    Ok(ev.id)
}

/// Generate the corpus into `store`, returning the side files.
///
/// The store must be empty (a fresh file). Every event is appended in the honest M0 order, and the
/// forecast's `state_version` is the *global* 0-based index of the commitment in the concatenated
/// log — the version `tsr_cli::reconstruct` checks a gated action against.
pub fn generate(store: &Store) -> Result<Corpus, LearnError> {
    let mut rng = SplitMix64::new(SEED);
    let mut tasks: Vec<SynthTaskRow> = Vec::new();
    let mut labels: Vec<LabelRecord> = Vec::new();
    // The global event count, so a forecast can name the version its commitment will occupy.
    let mut global: u64 = 0;

    let mut task_index = 0usize;
    for family in FAMILIES {
        for i in 0..family.tasks {
            let plan = plan_task(&mut rng, family, i);
            let task = task_id(&plan.name);
            let mut parents: Vec<EventId> = Vec::new();
            let mut step = 0usize;

            // TaskOpened.
            let id = append(
                store,
                &task,
                parents.clone(),
                Source::Operator,
                EventKind::TaskOpened { task },
                task_index,
                step,
            )?;
            parents = vec![id];
            step += 1;
            global += 1;

            // The inspections before the first decision.
            for k in 0..plan.inspections {
                let cand = mint(ActionKind::Inspect, &format!("src/file_{k}.rs"));
                let id = append(
                    store,
                    &task,
                    parents.clone(),
                    Source::Rules,
                    EventKind::Proposed {
                        candidate: cand.clone(),
                    },
                    task_index,
                    step,
                )?;
                parents = vec![id];
                step += 1;
                global += 1;
                // The inspection is dispatched and executed (in the sandbox), so it is reconciled.
                let id = append(
                    store,
                    &task,
                    parents.clone(),
                    Source::Rules,
                    EventKind::Dispatched {
                        candidate: cand.id,
                        in_sandbox: true,
                    },
                    task_index,
                    step,
                )?;
                parents = vec![id];
                step += 1;
                global += 1;
                let id = append(
                    store,
                    &task,
                    parents.clone(),
                    Source::Rules,
                    EventKind::Executed {
                        candidate: cand.id,
                        exit_code: Some(0),
                        log_digest: None,
                        duration_ms: 1_000,
                    },
                    task_index,
                    step,
                )?;
                parents = vec![id];
                step += 1;
                global += 1;
            }

            // The gated decisions.
            for d in 0..plan.decisions {
                let insp = plan.inspections + d as u64;
                // The gated action kind cycles so all three gated kinds appear.
                let kind = match d % 3 {
                    0 => ActionKind::RunApprovedTest,
                    1 => ActionKind::ProposePatch,
                    _ => ActionKind::RequestApproval,
                };
                let cand = mint(kind, &format!("step_{d}"));
                let id = append(
                    store,
                    &task,
                    parents.clone(),
                    Source::Rules,
                    EventKind::Proposed {
                        candidate: cand.clone(),
                    },
                    task_index,
                    step,
                )?;
                parents = vec![id];
                step += 1;
                global += 1;

                // The forecast, at the version the commitment will occupy (global count now).
                let commit_version = global;
                let f = Forecast::new(
                    crate::decision_event_id(),
                    Horizon::TaskClose,
                    Forecaster::Generator,
                    gen_prob(&plan.family, insp),
                    commit_version,
                );
                let id = append(
                    store,
                    &task,
                    parents.clone(),
                    Source::Generator,
                    EventKind::Forecast {
                        forecast: f,
                        candidate: cand.id,
                    },
                    task_index,
                    step,
                )?;
                parents = vec![id];
                step += 1;
                global += 1;

                if plan.operator_forecasts {
                    let f = Forecast::new(
                        crate::decision_event_id(),
                        Horizon::TaskClose,
                        Forecaster::Operator,
                        op_prob(&plan.family, insp),
                        commit_version,
                    );
                    let id = append(
                        store,
                        &task,
                        parents.clone(),
                        Source::Operator,
                        EventKind::Forecast {
                            forecast: f,
                            candidate: cand.id,
                        },
                        task_index,
                        step,
                    )?;
                    parents = vec![id];
                    step += 1;
                    global += 1;
                }

                // The commitment.
                let id = append(
                    store,
                    &task,
                    parents.clone(),
                    Source::Rules,
                    EventKind::Committed {
                        candidate: cand.id,
                        decision: Decision::Selected {
                            value: cand.id,
                            utility: placeholder_utility(),
                            propensity: 1.0,
                            evidence: vec![],
                        },
                    },
                    task_index,
                    step,
                )?;
                parents = vec![id];
                step += 1;
                global += 1;

                // Dispatch (in the sandbox) and execute.
                let id = append(
                    store,
                    &task,
                    parents.clone(),
                    Source::Rules,
                    EventKind::Dispatched {
                        candidate: cand.id,
                        in_sandbox: true,
                    },
                    task_index,
                    step,
                )?;
                parents = vec![id];
                step += 1;
                global += 1;
                let exit = if plan.accepted[d] { 0 } else { 1 };
                let id = append(
                    store,
                    &task,
                    parents.clone(),
                    Source::Rules,
                    EventKind::Executed {
                        candidate: cand.id,
                        exit_code: Some(exit),
                        log_digest: None,
                        duration_ms: 2_000,
                    },
                    task_index,
                    step,
                )?;
                parents = vec![id];
                step += 1;
                global += 1;
            }

            // The terminal state.
            match plan.terminal {
                Terminal::ClosedReviewed | Terminal::ClosedUnreviewed => {
                    let _ = append(
                        store,
                        &task,
                        parents.clone(),
                        Source::Operator,
                        EventKind::TaskClosed {
                            task,
                            reason: "done".to_string(),
                        },
                        task_index,
                        step,
                    )?;
                    global += 1;
                }
                Terminal::OperatorStop => {
                    let _ = append(
                        store,
                        &task,
                        parents.clone(),
                        Source::Operator,
                        EventKind::Stop {
                            reason: "operator".to_string(),
                        },
                        task_index,
                        step,
                    )?;
                    global += 1;
                }
                Terminal::Open => {}
            }

            // The index row.
            tasks.push(SynthTaskRow {
                task,
                family: plan.family.name.to_string(),
                lineage: plan.lineage.clone(),
                timestamp: stamp(task_index, 0),
                clone_group: plan.clone_group.clone(),
                tool_ms_ceiling: plan.tool_ms_ceiling,
            });

            // The label record, only for a reviewed close.
            if plan.terminal == Terminal::ClosedReviewed {
                // The true acceptance of the task: the last decision's outcome decides the diff.
                let accepted = *plan.accepted.last().unwrap_or(&false);
                labels.push(LabelRecord {
                    task,
                    criteria_met: accepted,
                    diff_label: Some(if accepted {
                        crate::decision::DiffLabel::Accepted
                    } else {
                        crate::decision::DiffLabel::Rejected
                    }),
                    reviewer: OPERATOR.to_string(),
                    recorded_at: stamp(task_index, 99),
                });
            }

            task_index += 1;
        }
    }

    Ok(Corpus { tasks, labels })
}

/// Write the corpus to `dir`: `events.jsonl`, `tasks.jsonl`, `labels.jsonl`.
///
/// `events.jsonl` is the `tsr task export` line format (`{"seq": n, "event": …}`), one line per
/// event, in global sequence order. `tasks.jsonl` and `labels.jsonl` are one JSON object per line.
pub fn write_fixtures(store: &Store, corpus: &Corpus, dir: &Path) -> Result<(), LearnError> {
    std::fs::create_dir_all(dir)?;

    // events.jsonl: every event, in sequence order, in the export line format.
    let mut events = String::new();
    for (seq, ev) in store.all()? {
        let line = serde_json::json!({ "seq": seq, "event": ev });
        events.push_str(&serde_json::to_string(&line)?);
        events.push('\n');
    }
    std::fs::write(dir.join("events.jsonl"), events)?;

    // tasks.jsonl.
    let mut tasks = String::new();
    for row in &corpus.tasks {
        tasks.push_str(&serde_json::to_string(row)?);
        tasks.push('\n');
    }
    std::fs::write(dir.join("tasks.jsonl"), tasks)?;

    // labels.jsonl.
    let mut labels = String::new();
    for rec in &corpus.labels {
        labels.push_str(&serde_json::to_string(rec)?);
        labels.push('\n');
    }
    std::fs::write(dir.join("labels.jsonl"), labels)?;

    Ok(())
}

/// Generate the corpus into a fresh store at `db` and write the three fixture files to `dir`.
pub fn generate_fixtures(db: &Path, dir: &Path) -> Result<Corpus, LearnError> {
    let store = Store::open(db)?;
    let corpus = generate(&store)?;
    write_fixtures(&store, &corpus, dir)?;
    Ok(corpus)
}