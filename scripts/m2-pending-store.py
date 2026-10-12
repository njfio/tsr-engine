#!/usr/bin/env python3
"""Build a scratch store with a pending idempotent dispatch, for the reconcile block.

Usage: python3 scripts/m2-pending-store.py <out.db>

The fixture corpus closes and reviews every episode, so no fixture task has a pending
dispatch. This builds the smallest store that does: TaskOpened -> Proposed(RunApprovedTest)
-> Dispatched, with no Executed after the dispatch. `RunApprovedTest` is the idempotent
action kind (`crates/tsr-cli/src/reconcile_cmd.rs:111`), so `tsr task reconcile` reports
`unresolved retryable` — the documented limitation (D1, N3).
"""
import json
import os
import sqlite3
import sys

TASK = "1a26474df3ed1dc847b171c26c9e882d69e891215480fd8ba1bd0bef2345b58b"
CAND = "dadc31a4d1754d993b4d3f933dbe02931c0428c8d44974d5cc71b950e27725b8"

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


def event(idx, parents, source, kind):
    return {
        "id": format(idx, "064x"),
        "schema_version": 1,
        "task": TASK,
        "parents": parents,
        "recorded_at": "2024-01-01T00:00:00.000Z",
        "observed_at": None,
        "source": source,
        "kind": kind,
    }


def main():
    out = sys.argv[1]
    if os.path.exists(out):
        os.remove(out)
    conn = sqlite3.connect(out)
    conn.executescript(SCHEMA)
    opened = event(1, [], "operator", {"kind": "task_opened", "task": TASK})
    proposed = event(2, [opened["id"]], "rules", {
        "kind": "proposed",
        "candidate": {
            "id": CAND,
            "kind": "run_approved_test",
            "source": "rules",
            "args": format(0xabc, "064x"),
            "parents": [opened["id"]],
            "conflicts_with": [],
            "support": [],
            "forecasts": [],
        },
    })
    dispatched = event(3, [proposed["id"]], "rules", {
        "kind": "dispatched",
        "candidate": CAND,
        "in_sandbox": True,
    })
    for ev in (opened, proposed, dispatched):
        conn.execute(
            "INSERT INTO events (id, schema_version, task_id, kind, recorded_at,"
            " observed_at, source, parents, payload) VALUES (?,?,?,?,?,?,?,?,?)",
            (ev["id"], 1, TASK, ev["kind"]["kind"], ev["recorded_at"], None,
             ev["source"], json.dumps(ev["parents"]), json.dumps(ev)),
        )
    conn.commit()
    conn.close()


if __name__ == "__main__":
    main()