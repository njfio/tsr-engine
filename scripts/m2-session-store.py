#!/usr/bin/env python3
"""Load an events.jsonl session log into a store the CLI can fold.

Usage: python3 scripts/m2-session-store.py <events.jsonl> <out.db>

Each line is either a bare `Event` or `{"seq": n, "event": Event}` (the shape
`tsr task export` and the hook write). The store schema matches `crates/tsr-store`:
the same columns and the same append-only triggers, so the CLI reads it identically.
"""
import json
import os
import sqlite3
import sys

SCHEMA = """
CREATE TABLE events (
    seq            INTEGER PRIMARY KEY AUTOINCREMENT,
    id             TEXT    NOT NULL UNIQUE,
    schema_version INTEGER NOT NULL,
    task_id        TEXT    NOT NULL,
    kind           TEXT    NOT NULL,
    recorded_at    TEXT    NOT NULL,
    observed_at    TEXT,
    source         TEXT    NOT NULL,
    parents        TEXT    NOT NULL,
    payload        TEXT    NOT NULL
);
CREATE TRIGGER events_no_update BEFORE UPDATE ON events
BEGIN SELECT RAISE(ABORT, 'events is append-only: updates are refused'); END;
CREATE TRIGGER events_no_delete BEFORE DELETE ON events
BEGIN SELECT RAISE(ABORT, 'events is append-only: deletes are refused'); END;
CREATE TRIGGER events_parents_exist BEFORE INSERT ON events
BEGIN SELECT CASE WHEN EXISTS (
    SELECT 1 FROM json_each(NEW.parents) AS p
    WHERE NOT EXISTS (SELECT 1 FROM events e WHERE e.id = p.value))
THEN RAISE(ABORT, 'event names a parent that is not in the log') END; END;
"""


def main():
    src, out = sys.argv[1], sys.argv[2]
    if os.path.exists(out):
        os.remove(out)
    conn = sqlite3.connect(out)
    conn.executescript(SCHEMA)
    with open(src) as fh:
        for line in fh:
            line = line.strip()
            if not line:
                continue
            value = json.loads(line)
            ev = value["event"] if "event" in value else value
            conn.execute(
                "INSERT INTO events (id, schema_version, task_id, kind, recorded_at,"
                " observed_at, source, parents, payload) VALUES (?,?,?,?,?,?,?,?,?)",
                (ev["id"], ev["schema_version"], ev["task"], ev["kind"]["kind"],
                 ev["recorded_at"], ev.get("observed_at"), ev["source"],
                 json.dumps(ev["parents"]), json.dumps(ev)),
            )
    conn.commit()
    conn.close()


if __name__ == "__main__":
    main()