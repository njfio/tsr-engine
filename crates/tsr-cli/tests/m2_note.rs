//! `tsr note <task> --text <short>` — the operator's experience entry (PRD §B7, §B11 M2 item 5).
//!
//! Two claims are tested here, and they are the whole point of the command:
//!
//! 1. the note appends exactly one `Observed` event carrying `Observation::Experience` with
//!    `Source::Operator` — its own provenance;
//! 2. the note is **not reward truth**: it creates no `Forecast`, no `Executed`, no outcome
//!    record and no score. A note that moved a number would be a label, and labels belong to
//!    `tsr review`, with their own provenance.
//!
//! Plus the read side the Mirror uses: `notes_for` returns the operator's words in sequence
//! order, and the CLI invocation itself over a real on-disk store.

use tsr_cli::note_cmd::{note_event, notes_for, record_note, validate, NoteError, MAX_NOTE_LEN};
use tsr_domain::{Event, EventKind, Hash, Observation, Source, TaskId};
use tsr_store::Store;

// ---------------------------------------------------------------------------------------------
// Fixtures
// ---------------------------------------------------------------------------------------------

fn task() -> TaskId {
    TaskId(Hash::of(&[b"task-note"]))
}

/// A store with one opened task, so a note has a head to attach to.
fn opened() -> (Store, Event) {
    let s = Store::open_in_memory().unwrap();
    let opened = Event::new(
        task(),
        vec![],
        Source::Rules,
        EventKind::TaskOpened { task: task() },
        "t0",
    );
    s.append(&opened).unwrap();
    (s, opened)
}

/// The kinds appended to the task, in sequence order.
fn kinds(s: &Store) -> Vec<String> {
    s.by_task(&task())
        .unwrap()
        .into_iter()
        .map(|(_, e)| {
            serde_json::to_value(&e.kind)
                .unwrap()
                .get("kind")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// 1. The note is an Observed event: Experience, Source::Operator
// ---------------------------------------------------------------------------------------------

#[test]
fn note_appends_one_observed_event_with_experience_and_operator_source() {
    let (s, opened) = opened();
    let id = record_note(&s, &task(), "the diff was easy to read", "t1").unwrap();

    let events = s.by_task(&task()).unwrap();
    assert_eq!(events.len(), 2, "the note is one event, not a cascade");

    let (_, note) = &events[1];
    assert_eq!(note.id, id);
    assert_eq!(note.source, Source::Operator, "the note's own provenance");
    assert_eq!(
        note.parents,
        vec![opened.id],
        "attached to the state it comments on"
    );
    assert_eq!(
        note.kind,
        EventKind::Observed {
            observation: Observation::Experience {
                note: "the diff was easy to read".to_string(),
            },
        }
    );
    assert_eq!(kinds(&s), vec!["task_opened", "observed"]);
}

#[test]
fn note_event_is_the_same_shape_without_a_store() {
    let e = note_event(task(), vec![], "friction: the test command is slow", "t1");
    assert_eq!(e.source, Source::Operator);
    assert_eq!(
        e.kind,
        EventKind::Observed {
            observation: Observation::Experience {
                note: "friction: the test command is slow".to_string(),
            },
        }
    );
}

#[test]
fn note_text_is_trimmed_and_the_event_id_commits_to_it() {
    let a = note_event(task(), vec![], "  spaced  ", "t1");
    let b = note_event(task(), vec![], "spaced", "t1");
    assert_eq!(a.kind, b.kind, "surrounding whitespace is not content");
    assert_eq!(a.id, b.id);

    let c = note_event(task(), vec![], "different", "t1");
    assert_ne!(a.id, c.id, "the id is the content");
}

// ---------------------------------------------------------------------------------------------
// 2. The note is never reward truth: no forecast, no outcome, no score
// ---------------------------------------------------------------------------------------------

#[test]
fn a_note_creates_no_forecast_and_no_outcome_record() {
    let (s, _) = opened();
    record_note(
        &s,
        &task(),
        "trust: the operator approved without reading",
        "t1",
    )
    .unwrap();

    let events = s.by_task(&task()).unwrap();
    for (_, e) in &events {
        assert!(
            !matches!(
                e.kind,
                EventKind::Forecast { .. }
                    | EventKind::Executed { .. }
                    | EventKind::Unresolved { .. }
                    | EventKind::Committed { .. }
                    | EventKind::Expanded { .. }
                    | EventKind::StatusSet { .. }
            ),
            "a note must not create a record that could move a number: {:?}",
            e.kind
        );
    }
    // The only event the note added is the observation itself.
    assert_eq!(kinds(&s), vec!["task_opened", "observed"]);
}

#[test]
fn a_note_does_not_change_the_log_root_of_anything_but_its_own_task() {
    let (s, _) = opened();
    let other = TaskId(Hash::of(&[b"task-other"]));
    let before = s.head_root().unwrap();
    record_note(&s, &task(), "a note", "t1").unwrap();
    let after = s.head_root().unwrap();
    assert_ne!(before, after, "the note is in the log");
    assert!(
        s.by_task(&other).unwrap().is_empty(),
        "the note lands on its own task and nowhere else"
    );
}

// ---------------------------------------------------------------------------------------------
// 3. The read side the Mirror uses
// ---------------------------------------------------------------------------------------------

#[test]
fn notes_for_returns_the_operators_words_in_sequence_order() {
    let (s, _) = opened();
    record_note(&s, &task(), "first: friction", "t1").unwrap();
    record_note(&s, &task(), "second: trust", "t2").unwrap();

    let notes = notes_for(&s, &task()).unwrap();
    let texts: Vec<&str> = notes.iter().map(|(_, t)| t.as_str()).collect();
    assert_eq!(texts, vec!["first: friction", "second: trust"]);

    // Each note is cited by the event that carries it, so the Mirror can show it beside the
    // system's numbers and point at the record.
    for (id, _) in &notes {
        assert!(s.seq_of(id).unwrap().is_some(), "the note is in the log");
    }
}

#[test]
fn notes_for_ignores_other_sources_and_other_observations() {
    let (s, opened) = opened();
    // A generator's evidence observation is not the operator's experience.
    let evidence = Event::new(
        task(),
        vec![opened.id],
        Source::Generator,
        EventKind::Observed {
            observation: Observation::Evidence {
                digest: Hash::of(&[b"log"]),
                source: "pytest".to_string(),
            },
        },
        "t1",
    );
    s.append(&evidence).unwrap();
    // An operator event that is not an experience note is not a note either.
    let missing = Event::new(
        task(),
        vec![evidence.id],
        Source::Operator,
        EventKind::Observed {
            observation: Observation::MissingEvidence {
                what: "the failing assertion".to_string(),
            },
        },
        "t2",
    );
    s.append(&missing).unwrap();
    record_note(&s, &task(), "the only note", "t3").unwrap();

    let notes = notes_for(&s, &task()).unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].1, "the only note");
}

// ---------------------------------------------------------------------------------------------
// 4. Validation: a note says something, and stays short
// ---------------------------------------------------------------------------------------------

#[test]
fn an_empty_note_is_refused_and_nothing_is_written() {
    let (s, _) = opened();
    assert_eq!(validate("   "), Err(NoteError::Empty));
    assert_eq!(validate(""), Err(NoteError::Empty));
    assert_eq!(kinds(&s), vec!["task_opened"], "nothing was appended");
}

#[test]
fn an_over_long_note_is_refused() {
    let long = "x".repeat(MAX_NOTE_LEN + 1);
    assert_eq!(
        validate(&long),
        Err(NoteError::TooLong {
            len: MAX_NOTE_LEN + 1,
            max: MAX_NOTE_LEN,
        })
    );
    assert!(validate(&"x".repeat(MAX_NOTE_LEN)).is_ok());
}

// ---------------------------------------------------------------------------------------------
// 5. The CLI invocation itself, over a real on-disk store
// ---------------------------------------------------------------------------------------------

#[test]
fn cli_note_records_the_observation_and_prints_it() {
    let dir = std::env::temp_dir().join(format!("tsr-note-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("tsr.db");

    {
        let s = Store::open(&db).unwrap();
        let opened = Event::new(
            task(),
            vec![],
            Source::Rules,
            EventKind::TaskOpened { task: task() },
            "t0",
        );
        s.append(&opened).unwrap();
    }

    let bin = env!("CARGO_BIN_EXE_tsr");
    let out = std::process::Command::new(bin)
        .args([
            "note",
            &task().to_string(),
            "--text",
            "friction: the sandbox took four minutes",
            "--store",
            db.to_str().unwrap(),
        ])
        .output()
        .expect("the tsr binary runs");
    assert_eq!(
        out.status.code(),
        Some(0),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("observation=experience") && stdout.contains("source=operator"),
        "stdout: {stdout}"
    );

    // The note landed in the log as an Observed/Experience event from the operator, and the log
    // holds nothing else: no forecast, no outcome.
    let s = Store::open(&db).unwrap();
    assert_eq!(kinds(&s), vec!["task_opened", "observed"]);
    let notes = notes_for(&s, &task()).unwrap();
    assert_eq!(notes.len(), 1);
    assert_eq!(notes[0].1, "friction: the sandbox took four minutes");

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn cli_note_refuses_an_empty_text_with_exit_2() {
    let dir = std::env::temp_dir().join(format!("tsr-note-empty-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let db = dir.join("tsr.db");
    let bin = env!("CARGO_BIN_EXE_tsr");
    let out = std::process::Command::new(bin)
        .args([
            "note",
            &task().to_string(),
            "--text",
            "   ",
            "--store",
            db.to_str().unwrap(),
        ])
        .output()
        .expect("the tsr binary runs");
    assert_eq!(out.status.code(), Some(2));
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("empty"),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn d12c_note_on_an_unknown_task_exits_2_without_creating_the_store() {
    let dir = std::env::temp_dir().join(format!("tsr-note-unknown-{}", std::process::id()));
    std::fs::remove_dir_all(&dir).ok();
    std::fs::create_dir_all(&dir).unwrap();
    // The store does not exist yet: a note must not create it.
    let db = dir.join("does-not-exist.db");
    assert!(!db.exists());

    let unknown = TaskId(Hash::of(&[b"task-never-opened"]));
    let bin = env!("CARGO_BIN_EXE_tsr");
    let out = std::process::Command::new(bin)
        .args([
            "note",
            &unknown.to_string(),
            "--text",
            "a note on a task the log does not hold",
            "--store",
            db.to_str().unwrap(),
        ])
        .output()
        .expect("the tsr binary runs");
    assert_eq!(
        out.status.code(),
        Some(2),
        "stderr: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        !db.exists(),
        "a note on an unknown task must not create the store at {}",
        db.display()
    );
    std::fs::remove_dir_all(&dir).ok();
}
