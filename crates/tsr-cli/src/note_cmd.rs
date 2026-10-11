//! `tsr note <task> --text <short>` — the operator's experience entry (PRD §B7, §B11 M2 item 5).
//!
//! At approval and at close the CLI offers two optional entries, one keypress each: a probability
//! for the pending event and a short note on friction or trust. The probability is the existing
//! `tsr forecast --source operator`; this command is the note.
//!
//! The note is recorded as `Observation::Experience` with `Source::Operator`, and it is exactly
//! that: an observation. It is evidence about usefulness and consequences that tests miss. It is
//! **never reward truth**, never proof of an external fact, and never presented as anything the
//! system feels (§B7). So this command appends one `Observed` event and nothing else — no
//! `Forecast`, no `Executed`, no outcome record, no score. A note that could move a number would
//! be a label, and labels are `tsr review`'s job, with their own provenance.
//!
//! The Mirror (`tsr reflect`, task #5) reads the note back through [`notes_for`], which returns
//! the operator's experience entries in sequence order so they can be rendered beside the
//! system's own numbers.

use std::path::Path;
use std::process::ExitCode;

use tsr_domain::{Event, EventId, EventKind, Hash, Observation, Source, TaskId};
use tsr_store::{Store, StoreError};

/// The longest note this command accepts. A note is a short entry, not a report: the PRD calls it
/// "a short note on friction or trust". Refusing a longer one keeps the Mirror's screen readable
/// and keeps the note from becoming a place to smuggle in a label.
pub const MAX_NOTE_LEN: usize = 500;

/// Why a note was refused before anything was written.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum NoteError {
    #[error("the note is empty; a note says something about friction or trust")]
    Empty,
    #[error("the note is {len} characters; the limit is {max}")]
    TooLong { len: usize, max: usize },
}

/// Validate a note's text. Pure: it writes nothing and touches no store.
pub fn validate(text: &str) -> Result<(), NoteError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(NoteError::Empty);
    }
    if trimmed.chars().count() > MAX_NOTE_LEN {
        return Err(NoteError::TooLong {
            len: trimmed.chars().count(),
            max: MAX_NOTE_LEN,
        });
    }
    Ok(())
}

/// The event a note appends: `Observed { observation: Experience { note } }`, `Source::Operator`.
///
/// `parents` is the task's current head — the events the note was written against — so the note
/// is causally attached to the state it comments on. `recorded_at` is wall-clock provenance only;
/// ordering is the store's sequence (§B6).
pub fn note_event(
    task: TaskId,
    parents: Vec<EventId>,
    text: &str,
    recorded_at: impl Into<String>,
) -> Event {
    Event::new(
        task,
        parents,
        Source::Operator,
        EventKind::Observed {
            observation: Observation::Experience {
                note: text.trim().to_string(),
            },
        },
        recorded_at,
    )
}

/// Append one operator note to `task`'s log. Returns the appended event's id.
///
/// The only write is the `Observed` event. Nothing here creates a forecast, an outcome record or
/// a score: the note is an observation, and the log is the only place it goes.
pub fn record_note(
    store: &Store,
    task: &TaskId,
    text: &str,
    recorded_at: &str,
) -> Result<EventId, StoreError> {
    let parents = head_parents(store, task)?;
    let event = note_event(*task, parents, text, recorded_at);
    store.append(&event)?;
    Ok(event.id)
}

/// The task's current head event ids: the last event of the task, if any.
///
/// A note on a task with no events yet has no parent — it is the first thing in that task's log.
fn head_parents(store: &Store, task: &TaskId) -> Result<Vec<EventId>, StoreError> {
    Ok(store
        .by_task(task)?
        .last()
        .map(|(_, e)| vec![e.id])
        .unwrap_or_default())
}

/// Every operator experience note on a task, in sequence order, as `(event id, text)`.
///
/// This is the read side the Mirror uses: the operator's own words, with the event that carries
/// them, so the screen can show the note beside the system's numbers and cite it by id.
pub fn notes_for(store: &Store, task: &TaskId) -> Result<Vec<(EventId, String)>, StoreError> {
    let mut out = Vec::new();
    for (_, e) in store.by_task(task)? {
        if e.source != Source::Operator {
            continue;
        }
        if let EventKind::Observed {
            observation: Observation::Experience { note },
        } = &e.kind
        {
            out.push((e.id, note.clone()));
        }
    }
    Ok(out)
}

/// Run `tsr note <task> --text <short>`.
///
/// Exit codes:
///
/// | code | meaning |
/// | --- | --- |
/// | 0 | the note was appended |
/// | 2 | the task id, the store or the note text is malformed |
pub fn run(task_hex: &str, text: &str, store_path: &str) -> ExitCode {
    let task = match Hash::from_hex(task_hex) {
        Ok(h) => TaskId(h),
        Err(e) => {
            eprintln!("tsr: invalid task id {task_hex:?}: {e}");
            return ExitCode::from(2);
        }
    };
    if let Err(e) = validate(text) {
        eprintln!("tsr: {e}");
        return ExitCode::from(2);
    }
    let store = match Store::open(Path::new(store_path)) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("tsr: cannot open store {store_path:?}: {e}");
            return ExitCode::from(2);
        }
    };
    let recorded_at = now_iso8601();
    match record_note(&store, &task, text, &recorded_at) {
        Ok(id) => {
            println!("noted {id} (observation=experience source=operator)");
            ExitCode::from(0)
        }
        Err(e) => {
            eprintln!("tsr: cannot record the note: {e}");
            ExitCode::from(2)
        }
    }
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
