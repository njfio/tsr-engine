//! `tsr review <task>` — labels on returns after close (PRD §B5, §B7, §B11 M2 item 4).
//!
//! A return is what came back from a dispatch. It awaits a label until an operator gives it one:
//! `accepted`, `rejected --reason <text>` or `disputed`. The label is an **outcome record** — an
//! event with `Source::Operator`, the reviewer, `observed_at` and `recorded_at`, and a status of
//! `pending | observed | censored | missing | disputed | invalid`. A correction appends a
//! **superseding** record that names the one it replaces; the log is append-only, so the first
//! record is still there, unrewritten.
//!
//! The rejection reason is stored apart from tool results: it lives in the record's `reason`
//! field, never in the evidence digest, and it enters the ledger directly as an Operator
//! preference through an `Expansion` whose `Contrast` is `Rejected(reason)` and whose
//! `catalyzed_by` names the task's open preferences (§B2.1, §B7).
//!
//! Sampling: `--sample <rate> --seed <n>` selects greens (accepted returns) for review with the
//! inverse-probability weight `1/rate` recorded on the record. Every failure is listed
//! unconditionally — sampling never hides a failure. `--family <f> --stats` prints the
//! false-accept rate (green but rejected), weighted and unweighted, with the number of labels
//! behind it.
//!
//! Nothing here writes a score or a weight into the log beyond the record's own `weight` field,
//! which is provenance for the estimator, not a belief.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::ExitCode;

use tsr_domain::{
    CriterionId, Event, EventId, EventKind, Hash, LabelStatus, OutcomeRecord, PreferenceId,
    PreferenceStatus, Scope, Source, TaskId, UnmetKind,
};
use tsr_ledger::{Filter, Ledger};
use tsr_store::{Store, StoreError};

/// The outcome definition a review label resolves. The PRD names `diff_accepted` as the event a
/// review settles (§B7); a label on a return is exactly that.
pub const REVIEW_OUTCOME: &str = "diff_accepted";
/// The version of [`REVIEW_OUTCOME`].
pub const REVIEW_OUTCOME_VERSION: u32 = 1;

/// Why a review command was refused before anything was written.
///
/// `PartialEq` is implemented by hand: the wrapped store, ledger and contract errors do not
/// implement it, and two wrapped errors are never equal — only the argument refusals are.
#[derive(Debug, thiserror::Error)]
pub enum ReviewError {
    #[error("unknown label {0:?}; expected accepted, rejected or disputed")]
    UnknownLabel(String),
    #[error("`rejected` needs --reason <text>")]
    MissingReason,
    #[error("the sample rate must be in (0, 1]; got {0}")]
    BadRate(f64),
    #[error("no return {0} on this task")]
    UnknownReturn(String),
    #[error("the rejection contrast did not expand")]
    ContrastDidNotExpand,
    #[error("the store refused the write: {0}")]
    Store(#[from] StoreError),
    #[error("the ledger refused the write: {0}")]
    Ledger(#[from] tsr_ledger::LedgerError),
    #[error("the contract refused the value: {0}")]
    Contract(#[from] tsr_domain::ContractError),
}

impl PartialEq for ReviewError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (ReviewError::UnknownLabel(a), ReviewError::UnknownLabel(b)) => a == b,
            (ReviewError::MissingReason, ReviewError::MissingReason) => true,
            (ReviewError::BadRate(a), ReviewError::BadRate(b)) => a == b,
            (ReviewError::UnknownReturn(a), ReviewError::UnknownReturn(b)) => a == b,
            // The wrapped errors carry no `PartialEq`; a wrapped refusal is never equal to another.
            _ => false,
        }
    }
}

impl ReviewError {
    /// The process exit code for this refusal: every argument refusal is exit 2.
    pub fn exit_code(&self) -> u8 {
        2
    }
}

/// The label an operator gives a return.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Label {
    Accepted,
    Rejected,
    Disputed,
}

impl Label {
    /// Parse the CLI's spelling.
    pub fn parse(s: &str) -> Result<Self, ReviewError> {
        match s {
            "accepted" => Ok(Label::Accepted),
            "rejected" => Ok(Label::Rejected),
            "disputed" => Ok(Label::Disputed),
            other => Err(ReviewError::UnknownLabel(other.to_string())),
        }
    }

    /// The status the label records.
    pub fn status(self) -> LabelStatus {
        match self {
            Label::Accepted => LabelStatus::Observed,
            Label::Rejected => LabelStatus::Rejected,
            Label::Disputed => LabelStatus::Disputed,
        }
    }

    /// The engineering name.
    pub fn name(self) -> &'static str {
        match self {
            Label::Accepted => "accepted",
            Label::Rejected => "rejected",
            Label::Disputed => "disputed",
        }
    }
}

/// A return awaiting a label: the `Executed` event and the candidate it settled.
#[derive(Clone, Debug, PartialEq)]
pub struct PendingReturn {
    pub event: EventId,
    pub candidate: tsr_domain::CandidateId,
    pub exit_code: Option<i32>,
    pub log_digest: Option<Hash>,
}

/// Every return on `task` that awaits a label, in sequence order.
///
/// A return is an `Executed` event. It awaits a label until an `OutcomeRecorded` event names it as
/// its subject. A record that has itself been superseded does not count: the return awaits the
/// label that stands.
pub fn pending_returns(store: &Store, task: &TaskId) -> Result<Vec<PendingReturn>, StoreError> {
    let events = store.by_task(task)?;
    let mut labelled: BTreeMap<EventId, EventId> = BTreeMap::new();
    let mut superseded: Vec<EventId> = Vec::new();
    for (_, e) in &events {
        if let EventKind::OutcomeRecorded { record } = &e.kind {
            labelled.insert(record.subject, e.id);
            if let Some(prev) = record.supersedes {
                superseded.push(prev);
            }
        }
    }
    let mut out = Vec::new();
    for (_, e) in &events {
        if let EventKind::Executed {
            candidate,
            exit_code,
            log_digest,
            ..
        } = &e.kind
        {
            let standing = labelled
                .get(&e.id)
                .copied()
                .filter(|id| !superseded.contains(id));
            if standing.is_none() {
                out.push(PendingReturn {
                    event: e.id,
                    candidate: *candidate,
                    exit_code: *exit_code,
                    log_digest: *log_digest,
                });
            }
        }
    }
    Ok(out)
}

/// The outcome record a label appends. Pure: it writes nothing.
///
/// `supersedes` names the record this one replaces, when it is a correction. `weight` is the
/// inverse-probability weight when the return was sampled, `None` when it was reviewed
/// unconditionally.
#[allow(clippy::too_many_arguments)]
pub fn outcome_record(
    subject: EventId,
    label: Label,
    reason: Option<String>,
    evidence: Option<Hash>,
    supersedes: Option<EventId>,
    weight: Option<f64>,
) -> Result<OutcomeRecord, ReviewError> {
    if label == Label::Rejected && reason.as_deref().map(str::trim).unwrap_or("").is_empty() {
        return Err(ReviewError::MissingReason);
    }
    // Route through the one minter so the id is `blake3("diff_accepted@v1")` — the same bytes
    // `tsr hook` hashes for its named events (D2).
    let outcome = tsr_forecast::outcome_def_id(REVIEW_OUTCOME, REVIEW_OUTCOME_VERSION);
    Ok(OutcomeRecord {
        outcome,
        subject,
        status: label.status(),
        reviewer: Source::Operator,
        reason: reason.map(|r| r.trim().to_string()),
        evidence,
        supersedes,
        weight,
    })
}

/// The event a label appends: `OutcomeRecorded { record }`, `Source::Operator`.
///
/// The record carries its own provenance: `recorded_at` is when the label was written and
/// `observed_at` is when the reviewer observed the return — the same instant here, since the
/// operator labels what they are looking at. Both are wall-clock provenance only; ordering is the
/// store's sequence and causation (§B6).
pub fn outcome_event(
    task: TaskId,
    parents: Vec<EventId>,
    record: OutcomeRecord,
    recorded_at: impl Into<String>,
) -> Event {
    let recorded_at = recorded_at.into();
    let mut event = Event::new(
        task,
        parents,
        Source::Operator,
        EventKind::OutcomeRecorded { record },
        recorded_at.clone(),
    );
    event.observed_at = Some(recorded_at);
    event
}

/// The task's current head event ids: the last event of the task, if any.
fn head_parents(store: &Store, task: &TaskId) -> Result<Vec<EventId>, StoreError> {
    Ok(store
        .by_task(task)?
        .last()
        .map(|(_, e)| vec![e.id])
        .unwrap_or_default())
}

/// The record that currently stands for `subject`, if any: the latest record naming it that has
/// not itself been superseded.
pub fn standing_record(
    store: &Store,
    task: &TaskId,
    subject: &EventId,
) -> Result<Option<EventId>, StoreError> {
    let events = store.by_task(task)?;
    let mut superseded: Vec<EventId> = Vec::new();
    let mut latest: Option<EventId> = None;
    for (_, e) in &events {
        if let EventKind::OutcomeRecorded { record } = &e.kind {
            if let Some(prev) = record.supersedes {
                superseded.push(prev);
            }
            if &record.subject == subject {
                latest = Some(e.id);
            }
        }
    }
    Ok(latest.filter(|id| !superseded.contains(id)))
}

/// The task's open preferences, in log order — the `catalyzed_by` of a rejection expansion.
pub fn open_preferences(store: &Store, task: &TaskId) -> Result<Vec<PreferenceId>, ReviewError> {
    let ledger = Ledger::new(store);
    let view = ledger.active_for_task(task)?;
    Ok(view
        .entries
        .iter()
        .filter(|e| matches!(e.status, Some(PreferenceStatus::Open)))
        .map(|e| e.preference.id)
        .collect())
}

/// Record a rejection's reason as an Operator preference through an Expansion.
///
/// The produced preference's text names the dimension the contrast clarified — the rejection — and
/// never the failed strategy's content. `catalyzed_by` names the task's open preferences. Returns
/// the produced preference's id.
pub fn record_rejection_preference(
    store: &Store,
    task: &TaskId,
    subject: &EventId,
    reason: &str,
    recorded_at: &str,
) -> Result<PreferenceId, ReviewError> {
    let filter = Filter::new(store);
    let catalyzed_by = open_preferences(store, task)?;
    // The rejection is a Contrast against the return: the criterion it failed is the return
    // itself, and how it failed is `Rejected(reason)`. The Filter writes the Contrast as an
    // `Observed` event and expands from it — the only path from a return to a preference.
    let criterion = CriterionId(subject.0);
    let contrast = Filter::contrast_from_criteria(
        *subject,
        vec![(
            criterion,
            UnmetKind::Rejected {
                reason: reason.trim().to_string(),
            },
        )],
        vec![*subject],
    );
    let parents = head_parents(store, task)?;
    let (_observed, produced) = filter.filter_return(
        task,
        parents,
        Vec::new(),
        Some(contrast),
        Scope::Task(*task),
        Vec::new(),
        catalyzed_by,
        Source::Operator,
        recorded_at,
    )?;
    produced.ok_or(ReviewError::ContrastDidNotExpand)
}

/// Record a label on a return. Returns the appended outcome event's id.
///
/// A second label on the same return appends a **superseding** record: `supersedes` names the
/// record that stood, and the first record stays in the log, unrewritten. A rejection also records
/// its reason as an Operator preference through an Expansion.
#[allow(clippy::too_many_arguments)]
pub fn record_label(
    store: &Store,
    task: &TaskId,
    subject: EventId,
    label: Label,
    reason: Option<String>,
    evidence: Option<Hash>,
    weight: Option<f64>,
    recorded_at: &str,
) -> Result<EventId, ReviewError> {
    let supersedes = standing_record(store, task, &subject)?;
    let record = outcome_record(subject, label, reason.clone(), evidence, supersedes, weight)?;
    let parents = head_parents(store, task)?;
    let event = outcome_event(*task, parents, record, recorded_at);
    store.append(&event)?;
    if label == Label::Rejected {
        let reason = reason.unwrap_or_default();
        record_rejection_preference(store, task, &subject, &reason, recorded_at)?;
    }
    Ok(event.id)
}

/// A deterministic sample of greens for review (PRD §B7).
///
/// `rate` is the sampling rate; `seed` fixes the draw. The selection is a hash of (seed, return
/// event id) mapped into [0, 1) and compared to `rate`, so it is reproducible without a random
/// number generator and independent of the order returns are listed in. The inverse-probability
/// weight `1/rate` is recorded on each selected record.
pub fn sample_greens(
    returns: &[PendingReturn],
    rate: f64,
    seed: u64,
) -> Result<Vec<(EventId, f64)>, ReviewError> {
    if !(rate > 0.0 && rate <= 1.0) {
        return Err(ReviewError::BadRate(rate));
    }
    let weight = 1.0 / rate;
    let mut out = Vec::new();
    for r in returns {
        // Sampling draws only over greens: a return whose own outcome passed. A failure is listed
        // unconditionally and is never sampled (PRD §B7).
        if r.exit_code.map(|c| c != 0).unwrap_or(true) {
            continue;
        }
        let mut buf = seed.to_le_bytes().to_vec();
        buf.extend_from_slice(&r.event.0 .0);
        let h = Hash::of(&[&buf]);
        // Map the first 8 bytes into [0, 1).
        let mut b = [0u8; 8];
        b.copy_from_slice(&h.0[..8]);
        let u = u64::from_le_bytes(b) as f64 / (u64::MAX as f64 + 1.0);
        if u < rate {
            out.push((r.event, weight));
        }
    }
    Ok(out)
}

/// One label behind the false-accept rate.
#[derive(Clone, Debug, PartialEq)]
pub struct LabelRow {
    pub subject: EventId,
    pub label: Label,
    pub weight: Option<f64>,
    /// The family the return belongs to, when the caller knows it.
    pub family: Option<String>,
    /// Whether the return's own outcome passed (exit 0). Only a green can be a false accept.
    pub green: bool,
}

/// The false-accept rate: greens that were rejected, weighted and unweighted, with the number of
/// labels behind it (PRD §B7).
#[derive(Clone, Debug, PartialEq)]
pub struct FalseAcceptRate {
    /// Greens (accepted) that were later rejected, unweighted count.
    pub false_accepts: usize,
    /// All greens, unweighted count.
    pub greens: usize,
    /// Sum of inverse-probability weights over false accepts.
    pub weighted_false_accepts: f64,
    /// Sum of inverse-probability weights over all greens.
    pub weighted_greens: f64,
    /// The number of labels behind the rate.
    pub labels: usize,
}

impl FalseAcceptRate {
    /// Unweighted false-accept rate, or `None` when there are no greens.
    pub fn unweighted(&self) -> Option<f64> {
        if self.greens == 0 {
            None
        } else {
            Some(self.false_accepts as f64 / self.greens as f64)
        }
    }

    /// Weighted false-accept rate, or `None` when there are no weighted greens.
    pub fn weighted(&self) -> Option<f64> {
        if self.weighted_greens == 0.0 {
            None
        } else {
            Some(self.weighted_false_accepts / self.weighted_greens)
        }
    }
}

/// Compute the false-accept rate over a set of label rows.
///
/// A "green" is a return the system produced as a success (tests passed) and the operator then
/// reviewed; a false accept is a green the operator rejected. Both `accepted` and `rejected`
/// labels are greens — a rejection is only meaningful because the return was green — so the
/// denominator is every labelled green and the numerator is the rejected ones. The unweighted
/// rate counts each label once; the weighted rate uses the inverse-probability weight recorded on
/// the record (1.0 when the return was reviewed unconditionally).
pub fn false_accept_rate(rows: &[LabelRow]) -> FalseAcceptRate {
    let mut r = FalseAcceptRate {
        false_accepts: 0,
        greens: 0,
        weighted_false_accepts: 0.0,
        weighted_greens: 0.0,
        labels: rows.len(),
    };
    for row in rows {
        // Only a green (the return's own outcome passed) can be a false accept. A failed return
        // that was rejected is not a green and never enters the denominator.
        if !row.green {
            continue;
        }
        match row.label {
            Label::Accepted => {
                r.greens += 1;
                r.weighted_greens += row.weight.unwrap_or(1.0);
            }
            Label::Rejected => {
                // A rejected green is a false accept, and it is still a green, so it belongs in
                // the denominator too.
                r.greens += 1;
                r.false_accepts += 1;
                let w = row.weight.unwrap_or(1.0);
                r.weighted_greens += w;
                r.weighted_false_accepts += w;
            }
            // `disputed` is neither a green nor a false accept: the operator did not decide.
            Label::Disputed => {}
        }
    }
    r
}

/// The arguments `tsr review` accepts.
#[derive(Clone, Debug, Default)]
pub struct ReviewArgs {
    pub task: String,
    /// `accepted`, `rejected` or `disputed`; `None` lists the returns awaiting a label.
    pub label: Option<String>,
    pub reason: Option<String>,
    /// `--return <event-id>`: name the return to label (a correction supersedes its standing label).
    pub return_id: Option<String>,
    /// `--sample <rate>`: select greens for review.
    pub sample: Option<f64>,
    /// `--seed <n>`: fix the sampling draw.
    pub seed: Option<u64>,
    /// `--family <f>`: restrict `--stats` to one family.
    pub family: Option<String>,
    /// `--stats`: print the false-accept rate.
    pub stats: bool,
    pub store: String,
}

/// Run `tsr review <task> [label] [--reason <text>] [--sample <rate> --seed <n>] [--family <f> --stats]`.
///
/// Exit codes:
///
/// | code | meaning |
/// | --- | --- |
/// | 0 | the label was appended, or the listing printed |
/// | 2 | the task id, the store, the label or the arguments are malformed |
pub fn run(args: &ReviewArgs) -> ExitCode {
    let store = match Store::open(Path::new(&args.store)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tsr: cannot open store {:?}: {e}", args.store);
            return ExitCode::from(2);
        }
    };
    // `--stats` does not require a task: it computes over every task of the family (D4).
    if args.stats {
        if args.task.trim().is_empty() {
            return print_stats_all(&store, args.family.as_deref());
        }
        let task = match Hash::from_hex(&args.task) {
            Ok(h) => TaskId(h),
            Err(e) => {
                eprintln!("tsr: invalid task id {:?}: {e}", args.task);
                return ExitCode::from(2);
            }
        };
        return print_stats(&store, &task, args.family.as_deref());
    }
    let task = match Hash::from_hex(&args.task) {
        Ok(h) => TaskId(h),
        Err(e) => {
            eprintln!("tsr: invalid task id {:?}: {e}", args.task);
            return ExitCode::from(2);
        }
    };
    let recorded_at = now_iso8601();

    let label = match &args.label {
        None => {
            // No label: list what awaits one. With `--sample`, list the greens the draw selected
            // (with their inverse-probability weight) and every failure unconditionally.
            return match (args.sample, args.seed) {
                (Some(rate), Some(seed)) => list_sampled(&store, &task, rate, seed),
                (Some(rate), None) => {
                    eprintln!("tsr: --sample {rate} needs --seed <n> to fix the draw");
                    ExitCode::from(2)
                }
                _ => list_pending(&store, &task),
            };
        }
        Some(s) => match Label::parse(s) {
            Ok(l) => l,
            Err(e) => {
                eprintln!("tsr: {e}");
                return ExitCode::from(2);
            }
        },
    };

    let pending = match pending_returns(&store, &task) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("tsr: cannot read the log: {e}");
            return ExitCode::from(2);
        }
    };
    // The subject is the return the operator names with `--return`, or the first pending one.
    // Naming an already-labelled return is a correction: `record_label` appends a superseding
    // record rather than rewriting the standing one.
    let subject = match &args.return_id {
        Some(hex) => match Hash::from_hex(hex) {
            Ok(h) => {
                let id = EventId(h);
                // The named return must be a return on this task (D8): an unknown id is refused.
                // A correction names an already-labelled return, so check every `Executed` event,
                // not only the pending ones.
                let is_return = store
                    .by_task(&task)
                    .map(|evs| {
                        evs.iter().any(|(_, e)| {
                            e.id == id && matches!(e.kind, EventKind::Executed { .. })
                        })
                    })
                    .unwrap_or(false);
                if !is_return {
                    eprintln!("tsr: {hex:?} is not a return on this task");
                    return ExitCode::from(2);
                }
                id
            }
            Err(e) => {
                eprintln!("tsr: invalid return id {hex:?}: {e}");
                return ExitCode::from(2);
            }
        },
        None => match pending.first() {
            Some(p) => p.event,
            None => {
                eprintln!("tsr: no return on this task awaits a label");
                return ExitCode::from(2);
            }
        },
    };

    let weight = match (args.sample, args.seed) {
        (Some(rate), Some(seed)) => match sample_greens(&pending, rate, seed) {
            Ok(sel) => sel.iter().find(|(id, _)| *id == subject).map(|(_, w)| *w),
            Err(e) => {
                eprintln!("tsr: {e}");
                return ExitCode::from(2);
            }
        },
        _ => None,
    };

    match record_label(
        &store,
        &task,
        subject,
        label,
        args.reason.clone(),
        None,
        weight,
        &recorded_at,
    ) {
        Ok(id) => {
            let w = weight
                .map(|w| format!(" weight={w:.4}"))
                .unwrap_or_default();
            println!(
                "reviewed {subject} label={} status={}{w} record={id}",
                label.name(),
                label.status().name()
            );
            ExitCode::from(0)
        }
        Err(e) => {
            eprintln!("tsr: cannot record the label: {e}");
            ExitCode::from(2)
        }
    }
}

/// List the returns awaiting a label, and every failure unconditionally.
fn list_pending(store: &Store, task: &TaskId) -> ExitCode {
    let pending = match pending_returns(store, task) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("tsr: cannot read the log: {e}");
            return ExitCode::from(2);
        }
    };
    if pending.is_empty() {
        println!("no returns await a label");
        return ExitCode::from(0);
    }
    println!("{} return(s) await a label:", pending.len());
    for p in &pending {
        let digest = p
            .log_digest
            .map(|d| d.to_hex())
            .unwrap_or_else(|| "none".to_string());
        let exit = p
            .exit_code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "none".to_string());
        println!(
            "  return={} candidate={} exit={exit} digest={digest}",
            p.event, p.candidate
        );
    }
    ExitCode::from(0)
}

/// List the greens the sample selected, with their inverse-probability weight, and every failure
/// unconditionally (PRD §B7: sampling never hides a failure).
fn list_sampled(store: &Store, task: &TaskId, rate: f64, seed: u64) -> ExitCode {
    let pending = match pending_returns(store, task) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("tsr: cannot read the log: {e}");
            return ExitCode::from(2);
        }
    };
    let selected = match sample_greens(&pending, rate, seed) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tsr: {e}");
            return ExitCode::from(2);
        }
    };
    let selected_ids: Vec<EventId> = selected.iter().map(|(id, _)| *id).collect();
    let weight_of = |id: &EventId| selected.iter().find(|(s, _)| s == id).map(|(_, w)| *w);

    let failures: Vec<&PendingReturn> = pending
        .iter()
        .filter(|p| p.exit_code.map(|c| c != 0).unwrap_or(false))
        .collect();
    let greens: Vec<&PendingReturn> = pending
        .iter()
        .filter(|p| !failures.iter().any(|f| f.event == p.event))
        .collect();

    println!(
        "sample rate={rate} seed={seed}: {} of {} green(s) selected; {} failure(s) listed unconditionally",
        selected_ids.len(),
        greens.len(),
        failures.len()
    );
    for p in &greens {
        match weight_of(&p.event) {
            Some(w) => println!("  selected return={} weight={w:.4}", p.event),
            None => println!("  not selected return={}", p.event),
        }
    }
    for p in &failures {
        let exit = p
            .exit_code
            .map(|c| c.to_string())
            .unwrap_or_else(|| "none".to_string());
        println!("  failure return={} exit={exit}", p.event);
    }
    ExitCode::from(0)
}

/// Print the false-accept rate over the task's labels.
fn print_stats(store: &Store, task: &TaskId, family: Option<&str>) -> ExitCode {
    let rows = match label_rows(store, task) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("tsr: cannot read the log: {e}");
            return ExitCode::from(2);
        }
    };
    let rows: Vec<LabelRow> = match family {
        Some(f) => rows
            .into_iter()
            .filter(|r| r.family.as_deref() == Some(f))
            .collect(),
        None => rows,
    };
    let rate = false_accept_rate(&rows);
    let unweighted = rate
        .unweighted()
        .map(|v| format!("{v:.4}"))
        .unwrap_or_else(|| "n/a".to_string());
    let weighted = rate
        .weighted()
        .map(|v| format!("{v:.4}"))
        .unwrap_or_else(|| "n/a".to_string());
    let fam = family.unwrap_or("all");
    println!("family={fam} labels={}", rate.labels);
    println!("false_accept_rate unweighted={unweighted} weighted={weighted}");
    println!(
        "greens={} false_accepts={} weighted_greens={:.4} weighted_false_accepts={:.4}",
        rate.greens, rate.false_accepts, rate.weighted_greens, rate.weighted_false_accepts
    );
    ExitCode::from(0)
}

/// Print the false-accept rate over every task of the family (D4: `--stats` needs no task).
fn print_stats_all(store: &Store, family: Option<&str>) -> ExitCode {
    let events = match store.all() {
        Ok(e) => e,
        Err(e) => {
            eprintln!("tsr: cannot read the log: {e}");
            return ExitCode::from(2);
        }
    };
    // Distinct tasks, in first-seen order.
    let mut tasks: Vec<TaskId> = Vec::new();
    for (_, e) in &events {
        if !tasks.contains(&e.task) {
            tasks.push(e.task);
        }
    }
    let mut rows: Vec<LabelRow> = Vec::new();
    for task in &tasks {
        match label_rows(store, task) {
            Ok(mut r) => rows.append(&mut r),
            Err(e) => {
                eprintln!("tsr: cannot read the log: {e}");
                return ExitCode::from(2);
            }
        }
    }
    let rows: Vec<LabelRow> = match family {
        Some(f) => rows
            .into_iter()
            .filter(|r| r.family.as_deref() == Some(f))
            .collect(),
        None => rows,
    };
    let rate = false_accept_rate(&rows);
    let unweighted = rate
        .unweighted()
        .map(|v| format!("{v:.4}"))
        .unwrap_or_else(|| "n/a".to_string());
    let weighted = rate
        .weighted()
        .map(|v| format!("{v:.4}"))
        .unwrap_or_else(|| "n/a".to_string());
    let fam = family.unwrap_or("all");
    println!("family={fam} labels={}", rate.labels);
    println!("false_accept_rate unweighted={unweighted} weighted={weighted}");
    println!(
        "greens={} false_accepts={} weighted_greens={:.4} weighted_false_accepts={:.4}",
        rate.greens, rate.false_accepts, rate.weighted_greens, rate.weighted_false_accepts
    );
    ExitCode::from(0)
}

/// The task's family, when the log names one.
///
/// The domain carries no per-task family field: a family is a `Scope::Family` a preference was
/// stated at. The task's family is therefore the first `Scope::Family` among its preferences, in
/// log order. `None` when the task has no family-scoped preference.
pub fn task_family(store: &Store, task: &TaskId) -> Result<Option<String>, ReviewError> {
    let ledger = Ledger::new(store);
    let collective = ledger.collective_for_task(task)?;
    for entry in &collective.entries {
        if let Scope::Family(f) = &entry.preference.scope {
            return Ok(Some(f.clone()));
        }
    }
    Ok(None)
}

/// Every standing label on the task, as rows for the estimator.
///
/// A record that has been superseded by a later one is not the standing label: only the latest
/// record per subject counts, and only when nothing supersedes it.
pub fn label_rows(store: &Store, task: &TaskId) -> Result<Vec<LabelRow>, ReviewError> {
    let family = task_family(store, task)?;
    let events = store.by_task(task)?;
    // The return's own outcome: an `Executed` event's exit code. A green is exit 0.
    let mut green_of: BTreeMap<EventId, bool> = BTreeMap::new();
    for (_, e) in &events {
        if let EventKind::Executed { exit_code, .. } = &e.kind {
            green_of.insert(e.id, exit_code.map(|c| c == 0).unwrap_or(false));
        }
    }
    let mut superseded: Vec<EventId> = Vec::new();
    let mut latest: BTreeMap<EventId, (EventId, Label, Option<f64>)> = BTreeMap::new();
    for (_, e) in &events {
        if let EventKind::OutcomeRecorded { record } = &e.kind {
            if let Some(prev) = record.supersedes {
                superseded.push(prev);
            }
            let label = match record.status {
                LabelStatus::Observed => Some(Label::Accepted),
                LabelStatus::Rejected => Some(Label::Rejected),
                LabelStatus::Disputed => Some(Label::Disputed),
                _ => None,
            };
            if let Some(l) = label {
                latest.insert(record.subject, (e.id, l, record.weight));
            }
        }
    }
    let mut out = Vec::new();
    for (subject, (record_id, label, weight)) in latest {
        if !superseded.contains(&record_id) {
            out.push(LabelRow {
                subject,
                label,
                weight,
                family: family.clone(),
                green: green_of.get(&subject).copied().unwrap_or(false),
            });
        }
    }
    Ok(out)
}

/// Wall-clock provenance only; ordering is the store's sequence and causation (§B6).
fn now_iso8601() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("unix:{secs}")
}
