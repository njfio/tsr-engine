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
    ActionKind, Candidate, Decision, Event, EventId, EventKind, Forecast, Forecaster, Hash,
    Horizon, Probability, Source, TaskId, Utility,
};
use tsr_store::Store;

use crate::decision::{DiffLabel, LabelRecord};
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

/// The number of consecutive task indices that share a clone group. A clone group is a contiguous
/// block, so it never straddles a time boundary (the time split keeps it whole).
pub const CLONE_BLOCK: usize = 6;

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
    /// The task-level acceptance: the common decision event `task_accepted_in_budget@v1`, a single
    /// Bernoulli draw at the task's own inspection count (`p_true(family, inspections)`), per the
    /// module header. This is the label every row of the task carries; it is NOT the last
    /// decision's draw, which would sample the highest inspection count and inflate the rate.
    task_accepted: bool,
}

/// Draw one task's plan from the stream.
///
/// `task_index` is the task's global position, which fixes its timestamp (`stamp(task_index, 0)`).
/// Clone groups are drawn from **contiguous blocks** of task indices, so a clone group never
/// straddles a time boundary: the time split keeps a group whole and every held_out task is still
/// later than every fit/calibrate task.
fn plan_task(rng: &mut SplitMix64, family: Family, index: usize, task_index: usize) -> TaskPlan {
    let name = format!("m4-{}-{:03}", family.name, index);
    let lineage = format!("lineage-{:02}", rng.index(LINEAGES));
    // A clone group on roughly a third of tasks, as a contiguous block of `CLONE_BLOCK` task
    // indices, so the split's clone-group rule is exercised without interleaving groups in time.
    let block = task_index / CLONE_BLOCK;
    let clone_group = if block % 3 == 0 {
        Some(format!("clone-{:03}", block))
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
    // The task-level common decision event: one draw at the task's own inspection count.
    let task_accepted = rng.next_f64() < p_true(&family, inspections);
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
        task_accepted,
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

/// Build one event, chaining it to the task's last event, with a fixed timestamp, and push it onto
/// `out`. Returns the event's id. `step` is the event's position within the task, so the timestamp
/// is a pure function of `(task_index, step)`.
///
/// Events are built in memory and appended in one transaction at the end ([`Store::append_all`]):
/// the id is a pure function of the content and parents, so nothing needs the store to be written
/// first, and one transaction keeps the generator fast.
fn push(
    out: &mut Vec<Event>,
    global: &mut u64,
    task: &TaskId,
    parents: Vec<EventId>,
    source: Source,
    kind: EventKind,
    at: (usize, usize),
) -> EventId {
    let (task_index, step) = at;
    let ev = Event::new(*task, parents, source, kind, stamp(task_index, step));
    let id = ev.id;
    out.push(ev);
    *global += 1;
    id
}

/// Generate the whole corpus into `store`, returning the side files.
///
/// The store must be empty: the generator appends the corpus in one transaction and the event ids
/// are content-derived, so a second run over a non-empty store would collide.
pub fn generate(store: &Store) -> Result<Corpus, LearnError> {
    let mut rng = SplitMix64::new(SEED);
    let mut tasks: Vec<SynthTaskRow> = Vec::new();
    let mut labels: Vec<LabelRecord> = Vec::new();
    // Every event, built in memory and appended in one transaction at the end.
    let mut events: Vec<Event> = Vec::new();

    // The global event count, so a forecast can name the version its commitment will occupy.
    let mut global: u64 = 0;

    // The families are interleaved in time, not laid out in contiguous blocks: `stamp` makes a
    // task's timestamp a pure function of its global position, so a block layout would put only
    // the last family on the late side of a forward-time split. Round-robin the (family, i) pairs
    // so every side of a `--by time` split sees all four families.
    // A9/A10/B11: spread each family evenly across the whole range, so a small family is not
    // confined to the first timestamps. Task `i` of a family of `n` gets position
    // `i * total / n`; the sort is stable, so families keep their declared order on ties. The
    // fourth family has 20 tasks, so under the acceptance split (`--by time --calibrate 0.2
    // --holdout 0.2 --seed 7`) every side holds tasks of all four families.
    let total: usize = FAMILIES.iter().map(|f| f.tasks).sum();
    let mut plan_order: Vec<(Family, usize)> = Vec::new();
    for family in FAMILIES {
        for i in 0..family.tasks {
            plan_order.push((family, i));
        }
    }
    plan_order.sort_by_key(|(family, i)| i * total / family.tasks);

    for (task_index, (family, i)) in plan_order.into_iter().enumerate() {
        {
            let plan = plan_task(&mut rng, family, i, task_index);
            let task = task_id(&plan.name);
            let mut parents: Vec<EventId> = Vec::new();
            let mut step = 0usize;

            // TaskOpened.
            let id = push(
                &mut events,
                &mut global,
                &task,
                parents.clone(),
                Source::Operator,
                EventKind::TaskOpened { task },
                (task_index, step),
            );
            parents = vec![id];
            step += 1;

            // The inspections before the first decision.
            for k in 0..plan.inspections {
                let cand = mint(ActionKind::Inspect, &format!("src/file_{k}.rs"));
                let id = push(
                    &mut events,
                    &mut global,
                    &task,
                    parents.clone(),
                    Source::Rules,
                    EventKind::Proposed {
                        candidate: cand.clone(),
                    },
                    (task_index, step),
                );
                parents = vec![id];
                step += 1;
                // The inspection is dispatched and executed (in the sandbox), so it is reconciled.
                let id = push(
                    &mut events,
                    &mut global,
                    &task,
                    parents.clone(),
                    Source::Rules,
                    EventKind::Dispatched {
                        candidate: cand.id,
                        in_sandbox: true,
                    },
                    (task_index, step),
                );
                parents = vec![id];
                step += 1;
                let id = push(
                    &mut events,
                    &mut global,
                    &task,
                    parents.clone(),
                    Source::Rules,
                    EventKind::Executed {
                        candidate: cand.id,
                        exit_code: Some(0),
                        log_digest: None,
                        duration_ms: 1_000,
                    },
                    (task_index, step),
                );
                parents = vec![id];
                step += 1;
            }

            // The gated decisions.
            for d in 0..plan.decisions {
                // A9/A10/B11: the generator forecasts at the task's own inspection count, the
                // same state the label `task_accepted` is drawn from (line 322), so each family's
                // documented distortion holds at every decision position.
                let insp = plan.inspections;
                // The gated action kind cycles so all three gated kinds appear.
                let kind = match d % 3 {
                    0 => ActionKind::RunApprovedTest,
                    1 => ActionKind::ProposePatch,
                    _ => ActionKind::RequestApproval,
                };
                let cand = mint(kind, &format!("step_{d}"));
                let id = push(
                    &mut events,
                    &mut global,
                    &task,
                    parents.clone(),
                    Source::Rules,
                    EventKind::Proposed {
                        candidate: cand.clone(),
                    },
                    (task_index, step),
                );
                parents = vec![id];
                step += 1;

                // The forecast, at the version the commitment will occupy. The commitment is
                // pushed after the generator forecast (and the operator forecast, when present),
                // so its 0-based index in the **task's own log** is `step` plus those events.
                // Main's hook records a forecast's `state_version` as the number of events in the
                // task's own log (`hook.rs` `event_count`), and `decision_rows` pairs on that
                // task-local index (A1/A3/A11), so the version is task-local, not session-global.
                let commit_version = step as u64 + 1 + u64::from(plan.operator_forecasts);
                let f = Forecast::new(
                    crate::decision_event_id(),
                    Horizon::TaskClose,
                    Forecaster::Generator,
                    gen_prob(&plan.family, insp),
                    commit_version,
                );
                let id = push(
                    &mut events,
                    &mut global,
                    &task,
                    parents.clone(),
                    Source::Generator,
                    EventKind::Forecast {
                        forecast: f,
                        candidate: cand.id,
                    },
                    (task_index, step),
                );
                parents = vec![id];
                step += 1;

                if plan.operator_forecasts {
                    let f = Forecast::new(
                        crate::decision_event_id(),
                        Horizon::TaskClose,
                        Forecaster::Operator,
                        op_prob(&plan.family, insp),
                        commit_version,
                    );
                    let id = push(
                        &mut events,
                        &mut global,
                        &task,
                        parents.clone(),
                        Source::Operator,
                        EventKind::Forecast {
                            forecast: f,
                            candidate: cand.id,
                        },
                        (task_index, step),
                    );
                    parents = vec![id];
                    step += 1;
                }

                // The commitment.
                let id = push(
                    &mut events,
                    &mut global,
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
                    (task_index, step),
                );
                parents = vec![id];
                step += 1;

                // Dispatch (in the sandbox) and execute.
                let id = push(
                    &mut events,
                    &mut global,
                    &task,
                    parents.clone(),
                    Source::Rules,
                    EventKind::Dispatched {
                        candidate: cand.id,
                        in_sandbox: true,
                    },
                    (task_index, step),
                );
                parents = vec![id];
                step += 1;
                let exit = if plan.accepted[d] { 0 } else { 1 };
                let id = push(
                    &mut events,
                    &mut global,
                    &task,
                    parents.clone(),
                    Source::Rules,
                    EventKind::Executed {
                        candidate: cand.id,
                        exit_code: Some(exit),
                        log_digest: None,
                        duration_ms: 2_000,
                    },
                    (task_index, step),
                );
                parents = vec![id];
                step += 1;
            }

            // The terminal state.
            match plan.terminal {
                Terminal::ClosedReviewed | Terminal::ClosedUnreviewed => {
                    let _ = push(
                        &mut events,
                        &mut global,
                        &task,
                        parents.clone(),
                        Source::Operator,
                        EventKind::TaskClosed {
                            task,
                            reason: "done".to_string(),
                        },
                        (task_index, step),
                    );
                }
                Terminal::OperatorStop => {
                    let _ = push(
                        &mut events,
                        &mut global,
                        &task,
                        parents.clone(),
                        Source::Operator,
                        EventKind::Stop {
                            reason: "operator".to_string(),
                        },
                        (task_index, step),
                    );
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

            // The label record, only for a reviewed close (B8: labels only after review).
            if plan.terminal == Terminal::ClosedReviewed {
                // The task closed with criteria met and the diff accepted, unless the last decision
                // failed — then the diff is rejected. A disputed diff resolves false, as lane 04
                // resolves `diff_accepted`.
                let (criteria_met, diff_label) = if plan.task_accepted {
                    (true, DiffLabel::Accepted)
                } else {
                    (false, DiffLabel::Rejected)
                };
                labels.push(LabelRecord {
                    task,
                    criteria_met,
                    diff_label: Some(diff_label),
                    reviewer: OPERATOR.to_string(),
                    recorded_at: stamp(task_index, step + 1),
                });
            }
        }
    }

    // One transaction for the whole corpus: the ids are content-derived, so nothing needed the
    // store while the events were built.
    store.append_all(&events)?;

    Ok(Corpus { tasks, labels })
}

/// Write the three fixture files under `dir`, creating it if needed.
///
/// `events.jsonl` is the `tsr task export` line format (`{"seq": n, "event": {…}}`), one line per
/// event in global sequence order; `tasks.jsonl` and `labels.jsonl` are one JSON object per line.
pub fn write_fixtures(dir: &Path, store: &Store, corpus: &Corpus) -> Result<(), LearnError> {
    std::fs::create_dir_all(dir)?;
    let events = store.all()?;
    let mut out = String::new();
    for (seq, ev) in &events {
        let line = serde_json::json!({ "seq": seq, "event": ev });
        out.push_str(&serde_json::to_string(&line)?);
        out.push('\n');
    }
    std::fs::write(dir.join("events.jsonl"), out)?;

    let mut out = String::new();
    for row in &corpus.tasks {
        out.push_str(&serde_json::to_string(row)?);
        out.push('\n');
    }
    std::fs::write(dir.join("tasks.jsonl"), out)?;

    let mut out = String::new();
    for rec in &corpus.labels {
        out.push_str(&serde_json::to_string(rec)?);
        out.push('\n');
    }
    std::fs::write(dir.join("labels.jsonl"), out)?;
    Ok(())
}

/// Generate the corpus into a fresh store at `db` and write the fixtures under `dir`.
pub fn generate_fixtures(db: &Path, dir: &Path) -> Result<Corpus, LearnError> {
    let store = Store::open(db)?;
    let corpus = generate(&store)?;
    write_fixtures(dir, &store, &corpus)?;
    Ok(corpus)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The fixtures directory, at the workspace root.
    fn fixtures_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/m4")
    }

    /// Regenerate `fixtures/m4/` in place. Run with `TSR_REGEN_FIXTURES=1`:
    /// `TSR_REGEN_FIXTURES=1 cargo test -p tsr-learn synth::tests::regen -- --nocapture`.
    #[test]
    fn regen() {
        if std::env::var("TSR_REGEN_FIXTURES").ok().as_deref() != Some("1") {
            return;
        }
        let dir = std::env::temp_dir().join(format!("tsr-m4-regen-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a temp dir is creatable");
        let db = dir.join("corpus.db");
        let out = dir.join("fixtures");
        generate_fixtures(&db, &out).expect("the corpus generates");
        let fixtures = fixtures_dir();
        std::fs::create_dir_all(&fixtures).expect("the fixtures dir is creatable");
        for name in ["events.jsonl", "tasks.jsonl", "labels.jsonl"] {
            std::fs::copy(out.join(name), fixtures.join(name))
                .unwrap_or_else(|e| panic!("rewriting {name}: {e}"));
        }
        eprintln!("TSR_REGEN_FIXTURES=1: rewrote {}", fixtures.display());
    }

    /// The generator is deterministic: two runs over fresh stores give identical bytes.
    #[test]
    fn the_generator_is_deterministic() {
        let a = std::env::temp_dir().join(format!("tsr-m4-det-a-{}", std::process::id()));
        let b = std::env::temp_dir().join(format!("tsr-m4-det-b-{}", std::process::id()));
        for d in [&a, &b] {
            let _ = std::fs::remove_dir_all(d);
            std::fs::create_dir_all(d).expect("a temp dir is creatable");
        }
        generate_fixtures(&a.join("corpus.db"), &a.join("fixtures")).expect("generates");
        generate_fixtures(&b.join("corpus.db"), &b.join("fixtures")).expect("generates");
        for name in ["events.jsonl", "tasks.jsonl", "labels.jsonl"] {
            let x = std::fs::read(a.join("fixtures").join(name)).expect("readable");
            let y = std::fs::read(b.join("fixtures").join(name)).expect("readable");
            assert_eq!(x, y, "{name} is not reproducible");
        }
    }

    /// The shape the brief names: 380 tasks, four families, at least eight lineages.
    #[test]
    fn the_shape_is_the_briefs() {
        let dir = std::env::temp_dir().join(format!("tsr-m4-shape-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a temp dir is creatable");
        let corpus =
            generate_fixtures(&dir.join("corpus.db"), &dir.join("fixtures")).expect("generates");
        assert_eq!(corpus.tasks.len(), 380);
        let mut families: std::collections::BTreeMap<&str, usize> =
            std::collections::BTreeMap::new();
        let mut lineages: std::collections::BTreeSet<&str> = std::collections::BTreeSet::new();
        for row in &corpus.tasks {
            *families.entry(row.family.as_str()).or_default() += 1;
            lineages.insert(row.lineage.as_str());
        }
        assert_eq!(families.get("overconfident"), Some(&120));
        assert_eq!(families.get("calibrated"), Some(&120));
        assert_eq!(families.get("underconfident"), Some(&120));
        assert_eq!(families.get("rare"), Some(&20));
        assert!(lineages.len() >= 8, "lineages: {}", lineages.len());
        assert!(!corpus.labels.is_empty(), "some tasks are reviewed");
        for rec in &corpus.labels {
            assert_eq!(rec.reviewer, OPERATOR);
        }
    }

    /// A9/A10/B11: the generator's p and the label's true probability come from the same state for
    /// every decision, so each family's documented distortion holds at every decision position.
    ///
    /// The header model makes both `p_true` and `p_generator` functions of the task's inspection
    /// count alone, and no inspection is logged between decisions, so within one task the
    /// generator's raw p must be the same at every decision position and must equal
    /// `p_generator(family, inspections)`. Before the fix the generator forecast at
    /// `inspections + d` while the label was drawn at `inspections`, so the "calibrated" family was
    /// overconfident after the first decision and the controller's measured gain was inflated.
    #[test]
    fn a9_a10_b11_generator_and_label_share_one_state_per_decision() {
        let dir = std::env::temp_dir().join(format!("tsr-m4-samestate-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("a temp dir is creatable");
        let db = dir.join("corpus.db");
        let corpus = generate_fixtures(&db, &dir.join("fixtures")).expect("generates");

        let store = Store::open(&db).expect("the store opens");
        let events = store.all().expect("the log reads");
        let rows = crate::decision::decision_rows(&events);
        let labels = crate::decision::resolve_labels(&events, &corpus.labels, OPERATOR)
            .expect("the labels resolve");

        // The family and inspection count per task, from the corpus rows.
        let mut family_of: std::collections::BTreeMap<String, String> =
            std::collections::BTreeMap::new();
        for row in &corpus.tasks {
            family_of.insert(row.task.to_string(), row.family.clone());
        }
        // Count the inspections before the first decision, per task, from the log.
        let mut inspections_of: std::collections::BTreeMap<String, u64> =
            std::collections::BTreeMap::new();
        for (_, ev) in &events {
            if let EventKind::Proposed { candidate } = &ev.kind {
                if candidate.kind == ActionKind::Inspect {
                    *inspections_of.entry(ev.task.to_string()).or_default() += 1;
                }
            }
        }
        let label_of: std::collections::BTreeMap<String, bool> = labels
            .iter()
            .filter_map(|l| match l.label {
                crate::decision::Label::True => Some((l.task.to_string(), true)),
                crate::decision::Label::False => Some((l.task.to_string(), false)),
                _ => None,
            })
            .collect();

        // (1) Within a task, the generator's raw p is constant across decision positions.
        let mut by_task: std::collections::BTreeMap<String, Vec<f64>> =
            std::collections::BTreeMap::new();
        for dr in &rows {
            if let Some(p) = dr.generator_raw() {
                by_task.entry(dr.task.to_string()).or_default().push(p);
            }
        }
        assert!(
            !by_task.is_empty(),
            "some decisions have a generator forecast"
        );
        for (task, ps) in &by_task {
            let first = ps[0];
            for p in ps {
                assert!(
                    (p - first).abs() < 1e-12,
                    "A9/A10/B11: task {task} forecasts at different states across decisions: {ps:?}"
                );
            }
        }

        // (2) Per (family, inspection count) — the decision position — the generator's p matches
        // `p_generator(family, inspections)` and the label rate matches `p_true(family,
        // inspections)`, within a stated tolerance. Grouping by the inspection count is what makes
        // the check meaningful: a family's tasks have different inspection counts, so a mean over
        // the whole family would not equal any single model value.
        let mut gen_sum: std::collections::BTreeMap<(String, u64), (f64, usize)> =
            std::collections::BTreeMap::new();
        let mut label_sum: std::collections::BTreeMap<(String, u64), (usize, usize)> =
            std::collections::BTreeMap::new();
        for dr in &rows {
            let task = dr.task.to_string();
            let Some(family) = family_of.get(&task).cloned() else {
                continue;
            };
            let insp = inspections_of.get(&task).copied().unwrap_or(0);
            let model = FAMILIES.iter().find(|f| f.name == family).unwrap();
            if let Some(p) = dr.generator_raw() {
                let e = gen_sum.entry((family.clone(), insp)).or_default();
                e.0 += p;
                e.1 += 1;
                assert!(
                    (p - p_generator(model, insp)).abs() < 1e-12,
                    "A9/A10/B11: {family} generator p {p} != p_generator({insp}) {}",
                    p_generator(model, insp)
                );
            }
        }
        // The label is per task, so count each task once — a task with four decisions must not
        // weigh four times in the rate.
        for (task, label) in &label_of {
            let Some(family) = family_of.get(task).cloned() else {
                continue;
            };
            let insp = inspections_of.get(task).copied().unwrap_or(0);
            let e = label_sum.entry((family, insp)).or_default();
            e.0 += usize::from(*label);
            e.1 += 1;
        }
        assert!(
            !gen_sum.is_empty(),
            "some decisions have a generator forecast"
        );
        for ((family, insp), (sum, n)) in &gen_sum {
            let model = FAMILIES.iter().find(|f| f.name == *family).unwrap();
            let mean = sum / *n as f64;
            assert!(
                (mean - p_generator(model, *insp)).abs() < 0.05,
                "A9/A10/B11: {family}@insp={insp} mean generator p {mean} vs model {}",
                p_generator(model, *insp)
            );
        }
        for ((family, insp), (hits, n)) in &label_sum {
            // The label is a Bernoulli draw, so the observed rate only approaches `p_true` as the
            // sample grows. The tolerance is a three-sigma binomial bound on the rate, stated here
            // and in the module header: `3 * sqrt(p(1-p)/n)`, floored at 0.05 so a tiny sample is
            // not held to a bound tighter than the draw can meet.
            let model = FAMILIES.iter().find(|f| f.name == *family).unwrap();
            let p = p_true(model, *insp);
            let rate = *hits as f64 / *n as f64;
            let tol = (3.0 * (p * (1.0 - p) / *n as f64).sqrt()).max(0.05);
            assert!(
                (rate - p).abs() <= tol,
                "A9/A10/B11: {family}@insp={insp} label rate {rate} (n={n}) vs model {p} (tol {tol})"
            );
        }
    }

    #[test]
    fn the_overconfident_family_is_the_b8_example() {
        let over = FAMILIES.iter().find(|f| f.name == "overconfident").unwrap();
        assert!(
            (p_generator(over, 0) - 0.9).abs() < 0.02,
            "{}",
            p_generator(over, 0)
        );
        assert!((p_true(over, 0) - 0.4).abs() < 0.02, "{}", p_true(over, 0));
    }
}
