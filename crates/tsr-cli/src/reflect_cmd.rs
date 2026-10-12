//! `tsr reflect <task>` — the Mirror view (PRD §B7, §B11 M2 item 7).
//!
//! One closed episode on one screen, in the book's order and nothing else:
//!
//! * **Desire** — the intent and the criteria, each with its open/closed state.
//! * **Imagination** — every candidate considered, with its source (generator | council |
//!   intuition) and which were excluded by conflict.
//! * **Belief** — the committed candidate with each forecaster's number: `raw p` always, `p`
//!   with its calibrator version only when calibrated, otherwise `uncalibrated`; propensity.
//! * **Return** — what came back, its status and evidence digest.
//! * **Residual** — Surprise per forecaster and Contrast per criterion, side by side, never added.
//!
//! Corrections appear as superseding rows. For an Intuition candidate the Mirror renders its
//! support (the preferences served and the tensions addressed); "none yet" is a valid screen
//! until M3/M4.
//!
//! `tsr reflect --family <f>` prints each forecaster's calibration curve (reliability bins and
//! Brier), including the operator's. `--json` emits the same content as a document for tests.
//!
//! Every number is a labeled proxy per Appendix A. No screen renders the four directions as a
//! sequence, and no screen prints a TSR term with a number.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;

use serde::Serialize;
use tsr_domain::{
    Candidate, CandidateId, Decision, Event, EventKind, ForecastId, Hash, Observation,
    PreferenceStatus, Source, TaskId,
};
use tsr_forecast::resolve::{forecaster_name, resolve, TaskFacts};
use tsr_forecast::score::{calibration_label, reliability_bins, Score, ScoredForecast};
use tsr_store::{Store, StoreError};

/// The arguments `tsr reflect` accepts.
#[derive(Debug, Default)]
pub struct ReflectArgs {
    /// The task to render. `None` with `--family` prints the family's calibration curves.
    pub task: Option<String>,
    /// Restrict the calibration curves to one task family.
    pub family: Option<String>,
    /// Emit the same content as a JSON document.
    pub json: bool,
    /// The store holding the log.
    pub store: String,
}

/// Why a reflect command was refused before anything was printed.
#[derive(Debug, thiserror::Error)]
pub enum ReflectError {
    #[error("no task given: `tsr reflect <task>` or `tsr reflect --family <f>`")]
    NoSubject,
    #[error("the store refused the read: {0}")]
    Store(#[from] StoreError),
    #[error("the ledger refused the read: {0}")]
    Ledger(#[from] tsr_ledger::LedgerError),
}

/// One criterion as the Mirror shows it: its id, its text, and whether it is open or closed.
#[derive(Clone, Debug, Serialize)]
pub struct CriterionRow {
    pub id: String,
    pub text: String,
    /// `open` | `closed`.
    pub state: String,
}

/// One candidate as the Mirror shows it.
#[derive(Clone, Debug, Serialize)]
pub struct CandidateRow {
    pub id: String,
    /// `generator` | `council` | `intuition` | `operator` | `rules`.
    pub source: String,
    pub kind: String,
    /// True when the candidate was excluded by conflict (another of its conflict set committed).
    pub excluded_by_conflict: bool,
    /// True when this candidate was the committed one.
    pub committed: bool,
    /// Intuition only: the preferences this step serves.
    pub support: Vec<String>,
    /// Intuition only: the tensions addressed (preference, against, note).
    pub tensions: Vec<String>,
}

/// One forecaster's number on the committed candidate.
#[derive(Clone, Debug, Serialize)]
pub struct BeliefRow {
    pub forecaster: String,
    /// The raw probability, always recorded.
    pub raw_p: f64,
    /// The calibrated probability, only when a validated calibrator exists.
    pub calibrated_p: Option<f64>,
    /// The calibrator version, only when calibrated.
    pub calibrator: Option<String>,
    /// `uncalibrated` when there is no validated calibrator.
    pub calibration: String,
    /// The support (same-family episodes within feature range).
    pub support_episodes: u32,
    /// The propensity: how often this forecaster's number was acted on. A labeled proxy.
    pub propensity: Option<f64>,
}

/// What came back from the dispatch.
#[derive(Clone, Debug, Serialize)]
pub struct ReturnRow {
    pub event: String,
    /// `executed` | `unresolved` | `none`.
    pub status: String,
    pub exit_code: Option<i32>,
    pub evidence_digest: Option<String>,
    /// The operator's label on this return, when one exists.
    pub label: Option<String>,
    /// The label's reason, when the label is rejected/disputed.
    pub label_reason: Option<String>,
    /// True when this row is a correction that supersedes an earlier label.
    pub supersedes: Option<String>,
}

/// One Surprise: the forecast residual per forecaster.
#[derive(Clone, Debug, Serialize)]
pub struct SurpriseRow {
    pub forecaster: String,
    pub forecast: String,
    /// outcome minus raw p, in [-1, 1]. A labeled proxy.
    pub residual: f64,
}

/// One Contrast: desired minus actual, per criterion.
#[derive(Clone, Debug, Serialize)]
pub struct ContrastRow {
    pub criterion: String,
    /// `wrong_behaviour` | `regression` | `cost` | `rejected`.
    pub unmet: String,
    pub reason: Option<String>,
    pub evidence: Vec<String>,
}

/// The whole Mirror document. `--json` emits exactly this.
#[derive(Clone, Debug, Serialize)]
pub struct Mirror {
    pub task: String,
    pub ledger_root: String,
    pub desire: Desire,
    pub imagination: Imagination,
    pub belief: Belief,
    pub returns: Vec<ReturnRow>,
    pub residual: Residual,
}

#[derive(Clone, Debug, Serialize)]
pub struct Desire {
    pub intent: Option<String>,
    pub criteria: Vec<CriterionRow>,
    /// The operator's experience notes on the task, in sequence order (through `notes_for`).
    pub notes: Vec<NoteRow>,
}

/// One operator experience note: the event that carries it and its text.
#[derive(Clone, Debug, Serialize)]
pub struct NoteRow {
    pub event: String,
    pub text: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct Imagination {
    pub candidates: Vec<CandidateRow>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Belief {
    pub committed: Option<String>,
    pub forecasters: Vec<BeliefRow>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Residual {
    pub surprise: Vec<SurpriseRow>,
    pub contrast: Vec<ContrastRow>,
}

/// One forecaster's calibration curve over a family.
#[derive(Clone, Debug, Serialize)]
pub struct CurveRow {
    pub forecaster: String,
    pub n: u32,
    pub brier: Option<f64>,
    pub log_loss: Option<f64>,
    pub bins: Vec<BinRow>,
}

#[derive(Clone, Debug, Serialize)]
pub struct BinRow {
    pub lower: f64,
    pub upper: f64,
    pub count: u32,
    pub mean_p: Option<f64>,
    pub observed: Option<f64>,
}

/// The family calibration document. `--family <f> --json` emits exactly this.
#[derive(Clone, Debug, Serialize)]
pub struct FamilyCurves {
    pub family: String,
    pub curves: Vec<CurveRow>,
}

// ---------------------------------------------------------------------------------------------
// Building the Mirror from the log
// ---------------------------------------------------------------------------------------------

/// The engineering name of a source, as the Mirror prints it.
fn source_name(s: Source) -> &'static str {
    match s {
        Source::Generator => "generator",
        Source::Council => "council",
        Source::Intuition => "intuition",
        Source::Operator => "operator",
        Source::Rules => "rules",
    }
}

/// The engineering name of an action kind.
fn kind_name(k: tsr_domain::ActionKind) -> &'static str {
    use tsr_domain::ActionKind::*;
    match k {
        Inspect => "inspect",
        RunApprovedTest => "run_approved_test",
        Generate => "generate",
        ProposePatch => "propose_patch",
        RequestApproval => "request_approval",
        Stop => "stop",
    }
}

/// The engineering name of a preference status.
fn status_name(s: &PreferenceStatus) -> String {
    match s {
        PreferenceStatus::Open => "open".to_string(),
        PreferenceStatus::Fulfilled => "fulfilled".to_string(),
        PreferenceStatus::Superseded(id) => format!("superseded({})", short(&id.0)),
        PreferenceStatus::Withdrawn => "withdrawn".to_string(),
        PreferenceStatus::Overlapping(id) => format!("overlapping({})", short(&id.0)),
        PreferenceStatus::OutOfScope => "out_of_scope".to_string(),
    }
}

/// The engineering name of an unmet kind.
fn unmet_name(k: &tsr_domain::UnmetKind) -> &'static str {
    use tsr_domain::UnmetKind::*;
    match k {
        WrongBehaviour => "wrong_behaviour",
        Regression => "regression",
        Cost => "cost",
        Rejected { .. } => "rejected",
    }
}

/// The first 12 hex characters of a hash, for a screen that must stay one line wide.
fn short(h: &Hash) -> String {
    let hex = h.to_hex();
    hex.chars().take(12).collect()
}

/// Build the Mirror for one task from its events.
pub fn build_mirror(task: &TaskId, events: &[(i64, Event)], root: Hash) -> Mirror {
    let evs: Vec<&Event> = events.iter().map(|(_, e)| e).collect();

    // --- Desire: the intent and the criteria, with their open/closed state. -------------------
    // The intent is the task's own intent: the task-scoped preference recorded when the task was
    // opened (the desired condition). It is *not* an operator experience note — those are notes,
    // listed separately below, and a note is an observation, never the task's intent.
    let mut intent: Option<String> = None;
    let mut criteria: Vec<CriterionRow> = Vec::new();
    // Criteria are the task's preferences in the task scope: each is a named condition.
    let collective = tsr_ledger::fold(events, root);
    for entry in &collective.entries {
        if matches!(entry.preference.scope, tsr_domain::Scope::Task(t) if t == *task) {
            if intent.is_none() {
                intent = Some(entry.preference.text.clone());
            }
            criteria.push(CriterionRow {
                id: short(&entry.preference.id.0),
                text: entry.preference.text.clone(),
                state: match &entry.status {
                    Some(PreferenceStatus::Open) => "open".to_string(),
                    Some(s) => status_name(s),
                    None => "not_admitted".to_string(),
                },
            });
        }
    }
    // The operator's notes, through the same helper `tsr note` writes through.
    let notes: Vec<NoteRow> = crate::note_cmd::notes_in(events)
        .into_iter()
        .map(|(id, text)| NoteRow {
            event: id.to_string(),
            text,
        })
        .collect();

    // --- Imagination: every candidate considered, its source, and conflict exclusions. --------
    let mut candidates: Vec<Candidate> = Vec::new();
    let mut committed: Option<CandidateId> = None;
    for ev in &evs {
        match &ev.kind {
            EventKind::Proposed { candidate } => candidates.push(candidate.clone()),
            EventKind::Committed { candidate, .. } => committed = Some(*candidate),
            _ => {}
        }
    }
    let committed_set: Vec<CandidateId> = committed.into_iter().collect();
    let mut candidate_rows: Vec<CandidateRow> = Vec::new();
    for c in &candidates {
        let excluded = c
            .conflicts_with
            .iter()
            .any(|other| committed_set.contains(other));
        let support: Vec<String> = c.support.iter().map(|p| short(&p.0)).collect();
        candidate_rows.push(CandidateRow {
            id: short(&c.id.0),
            source: source_name(c.source).to_string(),
            kind: kind_name(c.kind).to_string(),
            excluded_by_conflict: excluded,
            committed: committed_set.contains(&c.id),
            support,
            tensions: Vec::new(),
        });
    }

    // --- Belief: the committed candidate with each forecaster's number. -----------------------
    // Belief is about the *committed* candidate: only its forecasts are listed. The propensity is
    // the committed decision's propensity (the commit record), not a per-forecaster guess.
    let committed_propensity: Option<f64> = evs.iter().find_map(|ev| match &ev.kind {
        EventKind::Committed {
            decision: Decision::Selected { propensity, .. },
            ..
        } => Some(*propensity),
        _ => None,
    });
    let mut belief_rows: Vec<BeliefRow> = Vec::new();
    for ev in &evs {
        if let EventKind::Forecast {
            forecast,
            candidate,
        } = &ev.kind
        {
            if committed != Some(*candidate) {
                continue;
            }
            let calibrated = forecast.calibrated.as_ref();
            belief_rows.push(BeliefRow {
                forecaster: forecaster_name(&forecast.source),
                raw_p: forecast.raw.get(),
                calibrated_p: calibrated.map(|c| c.p.get()),
                calibrator: calibrated.map(|c| c.calibrator.clone()),
                calibration: calibration_label(forecast),
                support_episodes: forecast.support.episodes,
                propensity: committed_propensity,
            });
        }
    }

    // --- Return: what came back, its status and evidence digest; corrections as rows. ---------
    let mut returns: Vec<ReturnRow> = Vec::new();
    for ev in &evs {
        match &ev.kind {
            EventKind::Executed {
                exit_code,
                log_digest,
                ..
            } => returns.push(ReturnRow {
                event: short(&ev.id.0),
                status: "executed".to_string(),
                exit_code: *exit_code,
                evidence_digest: log_digest.map(|h| short(&h)),
                label: None,
                label_reason: None,
                supersedes: None,
            }),
            EventKind::Unresolved { reason, .. } => returns.push(ReturnRow {
                event: short(&ev.id.0),
                status: format!("unresolved: {reason}"),
                exit_code: None,
                evidence_digest: None,
                label: None,
                label_reason: None,
                supersedes: None,
            }),
            EventKind::OutcomeRecorded { record } => returns.push(ReturnRow {
                event: short(&ev.id.0),
                status: "label".to_string(),
                exit_code: None,
                evidence_digest: record.evidence.map(|h| short(&h)),
                label: Some(record.status.name().to_string()),
                label_reason: record.reason.clone(),
                supersedes: record.supersedes.map(|h| short(&h.0)),
            }),
            _ => {}
        }
    }

    // --- Residual: Surprise per forecaster and Contrast per criterion, side by side. ----------
    // A Surprise row names the forecaster that made the forecast it is a residual of.
    let mut forecaster_of: BTreeMap<ForecastId, String> = BTreeMap::new();
    for ev in &evs {
        if let EventKind::Forecast { forecast, .. } = &ev.kind {
            forecaster_of.insert(forecast.id, forecaster_name(&forecast.source));
        }
    }
    let mut surprise: Vec<SurpriseRow> = Vec::new();
    let mut contrast: Vec<ContrastRow> = Vec::new();
    for ev in &evs {
        if let EventKind::Observed {
            observation:
                Observation::Return {
                    surprise: s,
                    contrast: c,
                },
        } = &ev.kind
        {
            for one in s {
                surprise.push(SurpriseRow {
                    forecaster: forecaster_of
                        .get(&one.forecast)
                        .cloned()
                        .unwrap_or_else(|| "(unknown)".to_string()),
                    forecast: short(&one.forecast.0),
                    residual: one.residual,
                });
            }
            if let Some(c) = c {
                for (criterion, kind) in &c.unmet {
                    contrast.push(ContrastRow {
                        criterion: short(&criterion.0),
                        unmet: unmet_name(kind).to_string(),
                        reason: match kind {
                            tsr_domain::UnmetKind::Rejected { reason } => Some(reason.clone()),
                            _ => None,
                        },
                        evidence: c.evidence.iter().map(|e| short(&e.0)).collect(),
                    });
                }
            }
        }
    }

    Mirror {
        task: task.to_string(),
        ledger_root: root.to_hex(),
        desire: Desire {
            intent,
            criteria,
            notes,
        },
        imagination: Imagination {
            candidates: candidate_rows,
        },
        belief: Belief {
            committed: committed_set.first().map(|c| short(&c.0)),
            forecasters: belief_rows,
        },
        returns,
        residual: Residual { surprise, contrast },
    }
}

// ---------------------------------------------------------------------------------------------
// Rendering
// ---------------------------------------------------------------------------------------------

/// Render the Mirror as one screen, in the book's order and nothing else.
pub fn render_mirror(m: &Mirror) -> String {
    let mut out = String::new();
    out.push_str(&format!("Mirror — task {}\n", m.task));
    out.push_str(&format!("ledger root {}\n", m.ledger_root));

    out.push_str("\nDesire\n");
    out.push_str(&format!(
        "  intent: {}\n",
        m.desire.intent.as_deref().unwrap_or("(none recorded)")
    ));
    if m.desire.criteria.is_empty() {
        out.push_str("  criteria: none yet\n");
    } else {
        for c in &m.desire.criteria {
            out.push_str(&format!("  criterion {} [{}] {}\n", c.id, c.state, c.text));
        }
    }
    if m.desire.notes.is_empty() {
        out.push_str("  notes: none\n");
    } else {
        for n in &m.desire.notes {
            out.push_str(&format!("  note {} {}\n", n.event, n.text));
        }
    }

    out.push_str("\nImagination\n");
    if m.imagination.candidates.is_empty() {
        out.push_str("  candidates: none yet\n");
    } else {
        for c in &m.imagination.candidates {
            let mark = if c.committed {
                "committed"
            } else if c.excluded_by_conflict {
                "excluded by conflict"
            } else {
                "considered"
            };
            out.push_str(&format!(
                "  candidate {} ({}) {} — {}\n",
                c.id, c.source, c.kind, mark
            ));
            if c.source == "intuition" {
                let served = if c.support.is_empty() {
                    "none yet".to_string()
                } else {
                    c.support.join(", ")
                };
                let tensions = if c.tensions.is_empty() {
                    "none yet".to_string()
                } else {
                    c.tensions.join("; ")
                };
                out.push_str(&format!("    support (preferences served): {served}\n"));
                out.push_str(&format!("    tensions addressed: {tensions}\n"));
            }
        }
    }

    out.push_str("\nBelief\n");
    out.push_str(&format!(
        "  committed: {}\n",
        m.belief.committed.as_deref().unwrap_or("(none)")
    ));
    if m.belief.forecasters.is_empty() {
        out.push_str("  forecasters: none yet\n");
    } else {
        for b in &m.belief.forecasters {
            let calibrated = match (&b.calibrated_p, &b.calibrator) {
                (Some(p), Some(v)) => format!("p {p:.4} (calibrator {v})"),
                _ => "uncalibrated".to_string(),
            };
            let propensity = match b.propensity {
                Some(p) => format!("{p:.4}"),
                None => "none yet".to_string(),
            };
            out.push_str(&format!(
                "  {}: raw p {:.4}; {}; support {} episodes; propensity {}\n",
                b.forecaster, b.raw_p, calibrated, b.support_episodes, propensity
            ));
        }
    }

    out.push_str("\nReturn\n");
    if m.returns.is_empty() {
        out.push_str("  none yet\n");
    } else {
        for r in &m.returns {
            let digest = r.evidence_digest.as_deref().unwrap_or("(none)");
            out.push_str(&format!(
                "  {} {} exit={} evidence={}\n",
                r.event,
                r.status,
                r.exit_code
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                digest
            ));
            if let Some(label) = &r.label {
                let reason = r.label_reason.as_deref().unwrap_or("");
                let sup = r
                    .supersedes
                    .as_deref()
                    .map(|s| format!(" supersedes {s}"))
                    .unwrap_or_default();
                out.push_str(&format!("    label {label} {reason}{sup}\n"));
            }
        }
    }

    out.push_str("\nResidual\n");
    out.push_str("  Surprise (forecast residual per forecaster)\n");
    if m.residual.surprise.is_empty() {
        out.push_str("    none yet\n");
    } else {
        for s in &m.residual.surprise {
            out.push_str(&format!(
                "    forecast {} by {} residual {:.4}\n",
                s.forecast, s.forecaster, s.residual
            ));
        }
    }
    out.push_str("  Contrast (desired minus actual, per criterion)\n");
    if m.residual.contrast.is_empty() {
        out.push_str("    none yet\n");
    } else {
        for c in &m.residual.contrast {
            let reason = c.reason.as_deref().unwrap_or("");
            out.push_str(&format!(
                "    criterion {} unmet {} {reason} evidence {}\n",
                c.criterion,
                c.unmet,
                c.evidence.join(",")
            ));
        }
    }
    out
}

/// Build the family calibration curves from every scored forecast in the log.
///
/// The log is a flat sequence of events; a forecast resolves against *its own task's* returns, so
/// the fold is per task. A task's family is the first `Scope::Family` preference in log order (the
/// fixtures generator tags each episode that way); only tasks in `family` are scored.
pub fn build_family_curves(family: &str, events: &[(i64, Event)]) -> FamilyCurves {
    // Group the log by task, preserving first-seen order.
    let mut order: Vec<TaskId> = Vec::new();
    let mut by_task: BTreeMap<TaskId, Vec<Event>> = BTreeMap::new();
    for (_, ev) in events {
        if !by_task.contains_key(&ev.task) {
            order.push(ev.task);
        }
        by_task.entry(ev.task).or_default().push(ev.clone());
    }

    let mut scored: Vec<ScoredForecast> = Vec::new();
    for task in order {
        let evs = &by_task[&task];
        // The task's family: the first `Scope::Family` preference in log order.
        let root = Hash::of(&[b"family-curves"]);
        let collective = tsr_ledger::fold(
            &evs.iter()
                .cloned()
                .enumerate()
                .map(|(i, e)| (i as i64, e))
                .collect::<Vec<_>>(),
            root,
        );
        let task_family =
            collective
                .entries
                .iter()
                .find_map(|entry| match &entry.preference.scope {
                    tsr_domain::Scope::Family(name) => Some(name.clone()),
                    _ => None,
                });
        if task_family.as_deref() != Some(family) {
            continue;
        }

        // `TaskFacts::from_events` folds the log's own facts, including the `OutcomeRecorded`
        // labels `tsr review` writes, so each task's forecasts resolve against its own returns.
        let facts = TaskFacts::from_events(evs);
        for ev in evs {
            if let EventKind::Forecast { forecast, .. } = &ev.kind {
                let resolved = resolve(forecast, &facts);
                if let Some(s) =
                    ScoredForecast::from_resolved(&resolved, Some(family.to_string()), None)
                {
                    scored.push(s);
                }
            }
        }
    }
    // Group by forecaster, preserving first-seen order.
    let mut order: Vec<String> = Vec::new();
    let mut by: BTreeMap<String, Vec<ScoredForecast>> = BTreeMap::new();
    for s in &scored {
        let name = forecaster_name(&s.forecaster);
        if !by.contains_key(&name) {
            order.push(name.clone());
        }
        by.entry(name).or_default().push(s.clone());
    }
    let mut curves: Vec<CurveRow> = Vec::new();
    for name in order {
        let group = &by[&name];
        let score = Score::of(group);
        let bins = reliability_bins(group)
            .into_iter()
            .map(|b| BinRow {
                lower: b.lower,
                upper: b.upper,
                count: b.count,
                mean_p: b.mean_p,
                observed: b.observed,
            })
            .collect();
        curves.push(CurveRow {
            forecaster: name,
            n: score.n,
            brier: score.brier,
            log_loss: score.log_loss,
            bins,
        });
    }
    FamilyCurves {
        family: family.to_string(),
        curves,
    }
}

/// Render the family calibration curves as one screen.
pub fn render_family_curves(fc: &FamilyCurves) -> String {
    let mut out = String::new();
    out.push_str(&format!("Calibration curves — family {}\n", fc.family));
    if fc.curves.is_empty() {
        out.push_str("  no scored forecasts yet\n");
        return out;
    }
    for c in &fc.curves {
        let brier = c
            .brier
            .map(|b| format!("{b:.4}"))
            .unwrap_or_else(|| "none".to_string());
        let log_loss = c
            .log_loss
            .map(|l| format!("{l:.4}"))
            .unwrap_or_else(|| "none".to_string());
        out.push_str(&format!(
            "\n  {}: n {} brier {} log_loss {}\n",
            c.forecaster, c.n, brier, log_loss
        ));
        for b in &c.bins {
            let mean_p = b
                .mean_p
                .map(|p| format!("{p:.4}"))
                .unwrap_or_else(|| "-".to_string());
            let observed = b
                .observed
                .map(|o| format!("{o:.4}"))
                .unwrap_or_else(|| "-".to_string());
            out.push_str(&format!(
                "    bin [{:.1},{:.1}) count {} mean_p {} observed {}\n",
                b.lower, b.upper, b.count, mean_p, observed
            ));
        }
    }
    out
}

// ---------------------------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------------------------

/// Run `tsr reflect <task> [--json]` or `tsr reflect --family <f> [--json]`.
///
/// Exit codes:
///
/// | code | meaning |
/// | --- | --- |
/// | 0 | the Mirror or the curves printed |
/// | 2 | the task id, the store or the arguments are malformed |
pub fn run(args: &ReflectArgs) -> ExitCode {
    let store = match Store::open(Path::new(&args.store)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tsr: cannot open store {:?}: {e}", args.store);
            return ExitCode::from(2);
        }
    };

    // `--family <f>`: the calibration curves, over the whole log.
    if let Some(family) = &args.family {
        let events = match store.all() {
            Ok(e) => e,
            Err(e) => {
                eprintln!("tsr: cannot read the log: {e}");
                return ExitCode::from(2);
            }
        };
        let curves = build_family_curves(family, &events);
        if args.json {
            match serde_json::to_string_pretty(&curves) {
                Ok(s) => println!("{s}"),
                Err(e) => {
                    eprintln!("tsr: cannot serialize the curves: {e}");
                    return ExitCode::from(2);
                }
            }
        } else {
            print!("{}", render_family_curves(&curves));
        }
        return ExitCode::from(0);
    }

    let task_hex = match &args.task {
        Some(t) => t,
        None => {
            eprintln!("tsr: {}", ReflectError::NoSubject);
            return ExitCode::from(2);
        }
    };
    let task = match Hash::from_hex(task_hex) {
        Ok(h) => TaskId(h),
        Err(e) => {
            eprintln!("tsr: invalid task id {task_hex:?}: {e}");
            return ExitCode::from(2);
        }
    };
    let events = match store.by_task(&task) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("tsr: cannot read the log: {e}");
            return ExitCode::from(2);
        }
    };
    let root = match store.head_root() {
        Ok(r) => r,
        Err(e) => {
            eprintln!("tsr: cannot read the ledger root: {e}");
            return ExitCode::from(2);
        }
    };
    let mirror = build_mirror(&task, &events, root);
    if args.json {
        match serde_json::to_string_pretty(&mirror) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("tsr: cannot serialize the Mirror: {e}");
                return ExitCode::from(2);
            }
        }
    } else {
        print!("{}", render_mirror(&mirror));
    }
    ExitCode::from(0)
}
