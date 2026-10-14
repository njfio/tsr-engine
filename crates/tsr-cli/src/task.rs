//! `tsr task open | status | export <id>` — task intake, status and trace export (PRD §B6).
//!
//! - `open` records a `TaskOpened` for a task named by `--name` (or the session id), with the
//!   workspace, intent, criteria and limits carried in the event's provenance.
//! - `status` prints the task's event count and last event.
//! - `export <id>` writes every event of the task as JSON, in sequence order, one per line.

use crate::store_path;
use serde_json::json;
use tsr_domain::{Event, EventKind, Source, TaskId};
use tsr_store::Store;

/// Open a task: record `TaskOpened` and return the task id.
pub fn open(store: &Store, name: &str, workspace: &str, intent: &str) -> anyhow::Result<TaskId> {
    let task = store_path::task_for_name(name);
    let parents = last_parent(store, &task);
    let ev = Event::new(
        task,
        parents,
        Source::Operator,
        EventKind::TaskOpened { task },
        crate::hook::now_rfc3339(),
    );
    store.append(&ev)?;
    // The intake detail (workspace, intent) is provenance; M0 keeps it in a sibling Observed-free
    // form by recording it in the task's first event's recorded_at is not enough, so we also emit a
    // Generate event carrying the intent digest as the model's opening turn.
    let intent_digest = tsr_domain::Hash::of(&[workspace.as_bytes(), intent.as_bytes()]);
    let gen = Event::new(
        task,
        vec![ev.id],
        Source::Operator,
        EventKind::Generate {
            candidate: tsr_domain::CandidateId(intent_digest),
        },
        crate::hook::now_rfc3339(),
    );
    store.append(&gen)?;
    Ok(task)
}

/// The status of a task: its event count and last event.
pub fn status(store: &Store, task: &TaskId) -> anyhow::Result<(usize, Option<Event>)> {
    let events = store.by_task(task)?;
    // The task's *own* event count, not the store-wide highest sequence number: `latest_seq` is the
    // largest seq in the whole store, so it over-counts as soon as a second task exists.
    let count = events.len();
    let last = events.into_iter().next_back().map(|(_, e)| e);
    Ok((count, last))
}

/// Export every event of a task as JSON, in sequence order, one per line.
pub fn export(store: &Store, task: &TaskId) -> anyhow::Result<String> {
    let events = store.by_task(task)?;
    let mut out = String::new();
    for (seq, ev) in events {
        let line = json!({ "seq": seq, "event": ev });
        out.push_str(&serde_json::to_string(&line)?);
        out.push('\n');
    }
    Ok(out)
}

fn last_parent(store: &Store, task: &TaskId) -> Vec<tsr_domain::EventId> {
    store
        .by_task(task)
        .ok()
        .and_then(|v| v.into_iter().next_back().map(|(_, e)| vec![e.id]))
        .unwrap_or_default()
}
