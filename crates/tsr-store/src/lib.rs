//! # tsr-store — the index over the append-only log
//!
//! SQLite (WAL) is the *index*; identity is content-addressed (`tsr_domain::Hash`). Append-only is
//! enforced twice: by identity (an altered event is a different event) and by the database
//! (`BEFORE UPDATE` / `BEFORE DELETE` triggers that `RAISE(ABORT)`), so immutability does not depend
//! on anyone's discipline (PRD §B6).
//!
//! Ordering is `seq` and causation; wall-clock time is provenance only.
//!
//! M0 scope: `open`, `append`, `all`, `by_task`, `head_root`. Artifacts (the CAS directory), snapshot
//! materialization and the Culmination views arrive with their milestones.

#![forbid(unsafe_code)]

use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use tsr_domain::{Event, EventId, Hash, TaskId};

pub const SCHEMA_SQL: &str = r#"
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS events (
    seq            INTEGER PRIMARY KEY AUTOINCREMENT,
    id             TEXT    NOT NULL UNIQUE,     -- blake3(payload, parents), hex
    schema_version INTEGER NOT NULL,
    task_id        TEXT    NOT NULL,
    kind           TEXT    NOT NULL,            -- the EventKind tag
    recorded_at    TEXT    NOT NULL,            -- provenance only; ordering is seq + causation
    observed_at    TEXT,
    source         TEXT    NOT NULL,
    parents        TEXT    NOT NULL,            -- JSON array of parent ids
    payload        TEXT    NOT NULL             -- the full Event as JSON
);

CREATE INDEX IF NOT EXISTS events_task_seq ON events(task_id, seq);

-- Append-only, enforced by the database. An update or a delete aborts the statement.
CREATE TRIGGER IF NOT EXISTS events_no_update BEFORE UPDATE ON events
BEGIN
    SELECT RAISE(ABORT, 'events is append-only: updates are refused');
END;

CREATE TRIGGER IF NOT EXISTS events_no_delete BEFORE DELETE ON events
BEGIN
    SELECT RAISE(ABORT, 'events is append-only: deletes are refused');
END;

-- Every parent an event names must already be in the log: causality is finite and prior.
CREATE TRIGGER IF NOT EXISTS events_parents_exist BEFORE INSERT ON events
BEGIN
    SELECT CASE
        WHEN EXISTS (
            SELECT 1 FROM json_each(NEW.parents) AS p
            WHERE NOT EXISTS (SELECT 1 FROM events e WHERE e.id = p.value)
        )
        THEN RAISE(ABORT, 'event names a parent that is not in the log')
    END;
END;
"#;

#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    #[error("sqlite: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("append-only violation: {0}")]
    AppendOnly(String),
}

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        conn.execute_batch(SCHEMA_SQL)?;
        Ok(Self { conn })
    }

    pub fn open_in_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(SCHEMA_SQL)?;
        Ok(Self { conn })
    }

    /// Append an event. Returns its sequence number. Appending an event whose id is already present
    /// is idempotent: the existing seq is returned and nothing is written (the id *is* the content).
    pub fn append(&self, ev: &Event) -> Result<i64, StoreError> {
        if let Some(seq) = self.seq_of(&ev.id)? {
            return Ok(seq);
        }
        let kind_tag = serde_json::to_value(&ev.kind)?
            .get("kind")
            .and_then(|k| k.as_str())
            .unwrap_or("unknown")
            .to_string();
        let parents_json = serde_json::to_string(&ev.parents)?;
        let payload = serde_json::to_string(ev)?;
        let source = serde_json::to_string(&ev.source)?
            .trim_matches('"')
            .to_string();
        self.conn.execute(
            "INSERT INTO events (id, schema_version, task_id, kind, recorded_at, observed_at, source, parents, payload)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                ev.id.to_string(),
                ev.schema_version,
                ev.task.to_string(),
                kind_tag,
                ev.recorded_at,
                ev.observed_at,
                source,
                parents_json,
                payload
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn seq_of(&self, id: &EventId) -> Result<Option<i64>, StoreError> {
        Ok(self
            .conn
            .query_row(
                "SELECT seq FROM events WHERE id = ?1",
                params![id.to_string()],
                |r| r.get(0),
            )
            .optional()?)
    }

    /// Every event, in sequence order.
    pub fn all(&self) -> Result<Vec<(i64, Event)>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT seq, payload FROM events ORDER BY seq")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)))?;
        let mut out = Vec::new();
        for row in rows {
            let (seq, payload) = row?;
            out.push((seq, serde_json::from_str(&payload)?));
        }
        Ok(out)
    }

    /// A task's events, in sequence order.
    pub fn by_task(&self, task: &TaskId) -> Result<Vec<(i64, Event)>, StoreError> {
        let mut stmt = self
            .conn
            .prepare("SELECT seq, payload FROM events WHERE task_id = ?1 ORDER BY seq")?;
        let rows = stmt.query_map(params![task.to_string()], |r| {
            Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?))
        })?;
        let mut out = Vec::new();
        for row in rows {
            let (seq, payload) = row?;
            out.push((seq, serde_json::from_str(&payload)?));
        }
        Ok(out)
    }

    /// One hash for the whole log at its head: blake3 over every event id in sequence order.
    /// This is the `root` a snapshot pins beside its forecasts, and the point an evaluation fork
    /// starts from (PRD §B9, state isolation).
    pub fn head_root(&self) -> Result<Hash, StoreError> {
        let mut stmt = self.conn.prepare("SELECT id FROM events ORDER BY seq")?;
        let ids: Vec<String> = stmt
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<Result<_, _>>()?;
        let parts: Vec<&[u8]> = ids.iter().map(|s| s.as_bytes()).collect();
        Ok(Hash::of(&parts))
    }

    /// Exposed for tests and for the policy's own checks; never for writes outside `append`.
    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tsr_domain::{EventKind, Source};

    fn task() -> TaskId {
        TaskId(Hash::of(&["t".as_bytes()]))
    }

    fn opened(parents: Vec<EventId>) -> Event {
        Event::new(
            task(),
            parents,
            Source::Rules,
            EventKind::TaskOpened { task: task() },
            "2026-10-11T00:00:00Z",
        )
    }

    #[test]
    fn append_is_idempotent_by_identity_and_ordered_by_seq() {
        let s = Store::open_in_memory().unwrap();
        let e1 = opened(vec![]);
        let e2 = opened(vec![e1.id]);
        assert_eq!(s.append(&e1).unwrap(), 1);
        assert_eq!(s.append(&e2).unwrap(), 2);
        assert_eq!(
            s.append(&e1).unwrap(),
            1,
            "same content, same identity, same seq"
        );
        let all = s.all().unwrap();
        assert_eq!(all.len(), 2);
        assert_eq!(all[1].1.parents, vec![e1.id]);
    }

    #[test]
    fn the_database_refuses_update_and_delete() {
        let s = Store::open_in_memory().unwrap();
        let e1 = opened(vec![]);
        s.append(&e1).unwrap();
        let upd = s
            .connection()
            .execute("UPDATE events SET payload = 'x' WHERE seq = 1", []);
        assert!(upd.is_err(), "update must be refused by the trigger");
        let del = s
            .connection()
            .execute("DELETE FROM events WHERE seq = 1", []);
        assert!(del.is_err(), "delete must be refused by the trigger");
        assert_eq!(s.all().unwrap().len(), 1);
    }

    #[test]
    fn a_parent_must_already_be_in_the_log() {
        let s = Store::open_in_memory().unwrap();
        let ghost = EventId(Hash::of(&["never-recorded".as_bytes()]));
        let e = opened(vec![ghost]);
        assert!(s.append(&e).is_err(), "causality is finite and prior");
    }

    #[test]
    fn head_root_changes_with_every_append() {
        let s = Store::open_in_memory().unwrap();
        let r0 = s.head_root().unwrap();
        let e1 = opened(vec![]);
        s.append(&e1).unwrap();
        let r1 = s.head_root().unwrap();
        assert_ne!(r0, r1);
        let e2 = opened(vec![e1.id]);
        s.append(&e2).unwrap();
        assert_ne!(r1, s.head_root().unwrap());
    }
}
