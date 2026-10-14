//! M4 probes CLI test (lane 06, item 4): the `tsr train` CLI runs the leakage probes and refuses
//! a leaking or incomplete dataset with exit 2.
//!
//! Defect ids: A12 (`dataset` runs all three probes before writing and exits 2 instead of writing a
//! leaking split), A6 (a forged `split.json` is caught), B3 (`fit` exits 2 naming `events.jsonl` or
//! `tasks.jsonl` when missing from the dataset directory).
//!
//! The corpus is hand-built here, not taken from `fixtures/m4/` (item 2 changes those numbers).

use std::path::{Path, PathBuf};
use std::process::Command;

use tsr_domain::{
    ActionKind, Candidate, Decision, Event, EventKind, Forecast, Forecaster, Hash, Horizon,
    Probability, Source, TaskId, Utility,
};

/// The `tsr` binary cargo built for this test.
fn tsr_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_tsr"))
}

/// A fresh temp directory for this test run, unique per process and tag.
fn temp_dir(tag: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("tsr-m4-probes-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("a temp dir is creatable");
    base
}

/// Run `tsr <args>` and return `(exit_code, stdout, stderr)`.
fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(tsr_bin())
        .args(args)
        .output()
        .expect("the tsr binary runs");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn task(tag: &[u8]) -> TaskId {
    TaskId(Hash::of(&[tag]))
}

fn ev(task: TaskId, kind: EventKind, tag: &[u8], at: &str) -> Event {
    let mut e = Event::new(task, Vec::new(), Source::Rules, kind, at);
    e.recorded_at = format!("{at}#{}", hex(tag));
    e
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// One task's session: opened, proposed, a generator forecast at the commitment's task-local index,
/// committed, closed. `p` is the generator's probability.
fn session(t: TaskId, tag: &[u8], p: f64, at: &str) -> Vec<Event> {
    let c = Candidate::new(
        ActionKind::ProposePatch,
        Source::Generator,
        Hash::of(&[tag]),
        Vec::new(),
        Vec::new(),
    )
    .unwrap();
    let cid = c.id;
    vec![
        ev(t, EventKind::TaskOpened { task: t }, tag, at),
        ev(t, EventKind::Proposed { candidate: c }, tag, at),
        ev(
            t,
            EventKind::Forecast {
                forecast: Forecast::new(
                    tsr_learn::decision_event_id(),
                    Horizon::ThisDispatch,
                    Forecaster::Generator,
                    Probability::new(p).unwrap(),
                    3,
                ),
                candidate: cid,
            },
            tag,
            at,
        ),
        ev(
            t,
            EventKind::Committed {
                candidate: cid,
                decision: Decision::Selected {
                    value: cid,
                    utility: Utility {
                        p_success: 0.5,
                        value: 0.0,
                        cost: 0.0,
                        ev: 0.0,
                    },
                    propensity: 1.0,
                    evidence: Vec::new(),
                },
            },
            tag,
            at,
        ),
        ev(
            t,
            EventKind::TaskClosed {
                task: t,
                reason: "done".to_string(),
            },
            tag,
            at,
        ),
    ]
}

/// Write a corpus of `n` tasks: family `a` (even index) and family `b` (odd index), each task in
/// its own lineage and clone group, timestamps increasing. Returns `(events, tasks, labels)`.
///
/// With `leak`, every task is in one family and the generator probability varies per task: tasks
/// with `p > 0.5` are labelled true and tasks with `p < 0.5` false. The `generator_logit` feature
/// then separates the fit labels by a threshold, but it takes **several** values on each side, so
/// only the threshold rule (A4) catches it — the old singleton rule does not. Without `leak`, the
/// label alternates and no feature separates.
fn write_corpus(dir: &Path, n: usize, leak: bool) -> (PathBuf, PathBuf, PathBuf) {
    let mut events = String::new();
    let mut tasks = String::new();
    let mut labels = String::new();
    let mut seq = 1i64;
    for i in 0..n {
        let tag = format!("t{i}");
        let t = task(tag.as_bytes());
        // A single family, so no one-hot separates; the separator is the continuous logit.
        let family = "a";
        let (p, label) = if leak {
            // p in {0.6, 0.7, 0.8, 0.9} for true, {0.1, 0.2, 0.3, 0.4} for false: multi-valued.
            let k = (i / 2) % 4;
            if i % 2 == 0 {
                (0.6 + 0.1 * k as f64, true)
            } else {
                (0.1 + 0.1 * k as f64, false)
            }
        } else {
            (0.7, (i / 2) % 2 == 0)
        };
        let ts = format!("2026-02-{:02}T00:00:00Z", i + 1);
        for e in session(t, tag.as_bytes(), p, &ts) {
            let line = serde_json::json!({ "seq": seq, "event": e });
            events.push_str(&serde_json::to_string(&line).unwrap());
            events.push('\n');
            seq += 1;
        }
        let row = serde_json::json!({
            "task": t.to_string(),
            "family": family,
            "lineage": format!("lineage-{i:02}"),
            "timestamp": ts,
            "clone_group": format!("clone-{i:03}"),
        });
        tasks.push_str(&serde_json::to_string(&row).unwrap());
        tasks.push('\n');
        let lab = serde_json::json!({
            "task": t.to_string(),
            "criteria_met": label,
            "diff_label": if label { "accepted" } else { "rejected" },
            "reviewer": "operator",
            "recorded_at": ts,
        });
        labels.push_str(&serde_json::to_string(&lab).unwrap());
        labels.push('\n');
    }
    let events_path = dir.join("events.jsonl");
    let tasks_path = dir.join("tasks.jsonl");
    let labels_path = dir.join("labels.jsonl");
    std::fs::write(&events_path, events).unwrap();
    std::fs::write(&tasks_path, tasks).unwrap();
    std::fs::write(&labels_path, labels).unwrap();
    (events_path, tasks_path, labels_path)
}

/// Build a dataset from the hand-built corpus with a fit/calibrate/held_out split.
fn build_dataset(dir: &Path, n: usize) -> PathBuf {
    let (events, tasks, labels) = write_corpus(dir, n, false);
    let ds = dir.join("dataset");
    let (code, stdout, stderr) = run(&[
        "train",
        "dataset",
        "--events",
        events.to_str().unwrap(),
        "--tasks",
        tasks.to_str().unwrap(),
        "--labels",
        labels.to_str().unwrap(),
        "--by",
        "time",
        "--calibrate",
        "0.25",
        "--holdout",
        "0.25",
        "--seed",
        "7",
        "--out",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "dataset should build:\n{stdout}\n{stderr}");
    ds
}

/// A12: `tsr train dataset` runs the probes before writing. A corpus whose family one-hot separates
/// the fit labels perfectly is refused with exit 2, and nothing is written.
#[test]
fn a12_dataset_refuses_a_leaking_split_before_writing() {
    let dir = temp_dir("a12");
    let (events, tasks, labels) = write_corpus(&dir, 8, true);
    let ds = dir.join("dataset");
    let (code, stdout, stderr) = run(&[
        "train",
        "dataset",
        "--events",
        events.to_str().unwrap(),
        "--tasks",
        tasks.to_str().unwrap(),
        "--labels",
        labels.to_str().unwrap(),
        "--by",
        "time",
        // F9/B3 (b): a share that leaves a side empty is now refused before the probes run, so use
        // a split with all three sides non-empty; the leak is in the corpus, not the split.
        "--calibrate",
        "0.25",
        "--holdout",
        "0.25",
        "--seed",
        "7",
        "--out",
        ds.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "a leaking split must be refused with exit 2:\n{stdout}\n{stderr}"
    );
    assert!(
        stderr.contains("label_reach") || stderr.contains("separates"),
        "the refusal names the probe and the feature:\n{stderr}"
    );
    assert!(
        !ds.join("rows.jsonl").exists(),
        "nothing is written when a probe catches a leak"
    );
}

/// A6: a forged `split.json` that moves held_out tasks to fit is caught by `fit` with exit 2.
#[test]
fn a6_fit_refuses_a_forged_split_json() {
    let dir = temp_dir("a6");
    let ds = build_dataset(&dir, 8);
    // Forge split.json: move every held_out task to fit.
    let split_path = ds.join("split.json");
    let text = std::fs::read_to_string(&split_path).unwrap();
    let mut v: serde_json::Value = serde_json::from_str(&text).unwrap();
    if let Some(map) = v.get_mut("assignments").and_then(|a| a.as_object_mut()) {
        for (_, side) in map.iter_mut() {
            if side.as_str() == Some("held_out") {
                *side = serde_json::Value::String("fit".to_string());
            }
        }
    }
    std::fs::write(&split_path, serde_json::to_string_pretty(&v).unwrap()).unwrap();

    let out = dir.join("bundle");
    let (code, stdout, stderr) = run(&[
        "train",
        "fit",
        ds.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "a forged split.json must be refused with exit 2:\n{stdout}\n{stderr}"
    );
    assert!(
        stderr.contains("split") || stderr.contains("probe"),
        "the refusal names the split/probe:\n{stderr}"
    );
}

/// B3: `fit` exits 2 naming `events.jsonl` when it is missing from the dataset directory.
#[test]
fn b3_fit_refuses_a_dataset_missing_events_jsonl() {
    let dir = temp_dir("b3-events");
    let ds = build_dataset(&dir, 8);
    std::fs::remove_file(ds.join("events.jsonl")).unwrap();
    let out = dir.join("bundle");
    let (code, stdout, stderr) = run(&[
        "train",
        "fit",
        ds.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "a missing events.jsonl must be refused with exit 2:\n{stdout}\n{stderr}"
    );
    assert!(
        stderr.contains("events.jsonl"),
        "the refusal names events.jsonl:\n{stderr}"
    );
}

/// B3: `fit` exits 2 naming `tasks.jsonl` when it is missing from the dataset directory.
#[test]
fn b3_fit_refuses_a_dataset_missing_tasks_jsonl() {
    let dir = temp_dir("b3-tasks");
    let ds = build_dataset(&dir, 8);
    std::fs::remove_file(ds.join("tasks.jsonl")).unwrap();
    let out = dir.join("bundle");
    let (code, stdout, stderr) = run(&[
        "train",
        "fit",
        ds.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(
        code, 2,
        "a missing tasks.jsonl must be refused with exit 2:\n{stdout}\n{stderr}"
    );
    assert!(
        stderr.contains("tasks.jsonl"),
        "the refusal names tasks.jsonl:\n{stderr}"
    );
}
