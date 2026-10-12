//! M4 corpus test (lane 06, item 2): the synthetic corpus regenerates byte-for-byte, and every
//! fixture session satisfies all of `tsr_cli::reconstruct`'s invariants.
//!
//! The generator lives in `tsr_learn::synth`; it is seeded and clock-free, so the same seed yields
//! the same bytes on every platform. This test regenerates into a temp dir and compares against
//! `fixtures/m4/`. Set `TSR_REGEN_FIXTURES=1` to rewrite the fixtures in place instead.
//!
//! The invariants are the PRD's, not the implementer's (see `crates/tsr-cli/src/lib.rs`):
//! - B6: sequence is monotone; every parent precedes its child.
//! - B6: every dispatch is reconciled or explicitly unresolved.
//! - B4: every command ran in the sealed sibling sandbox.
//!
//! B5 (every gated action has a forecast at the same state version, recorded before commitment) is
//! asserted **per task**, not on the whole-corpus concatenation: `synth.rs` writes task-local state
//! versions, so the corpus is 380 independent sessions and the concatenation is not one session.
//! See `every_task_satisfies_the_m0_invariants_as_its_own_session`.

use std::path::{Path, PathBuf};

use tsr_cli::reconstruct;

/// The fixtures directory, at the workspace root.
fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/m4")
}

/// A fresh temp directory for this test run, unique per process.
fn temp_dir(tag: &str) -> PathBuf {
    let base = std::env::temp_dir().join(format!("tsr-m4-corpus-{}-{}", tag, std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    std::fs::create_dir_all(&base).expect("a temp dir is creatable");
    base
}

/// Regenerate the corpus into `dir` and return the three file paths.
fn regenerate(dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
    let db = dir.join("corpus.db");
    let out = dir.join("fixtures");
    tsr_learn::synth::generate_fixtures(&db, &out).expect("the corpus generates");
    (
        out.join("events.jsonl"),
        out.join("tasks.jsonl"),
        out.join("labels.jsonl"),
    )
}

/// Read a file as bytes, or panic with a clear message.
fn read(path: &Path) -> Vec<u8> {
    std::fs::read(path).unwrap_or_else(|e| panic!("reading {}: {e}", path.display()))
}

#[test]
fn fixtures_regenerate_byte_identically() {
    let dir = temp_dir("regen");
    let (events, tasks, labels) = regenerate(&dir);
    let fixtures = fixtures_dir();

    let regen = std::env::var("TSR_REGEN_FIXTURES").is_ok_and(|v| v == "1");
    if regen {
        std::fs::create_dir_all(&fixtures).expect("the fixtures dir is creatable");
        for (name, src) in [
            ("events.jsonl", &events),
            ("tasks.jsonl", &tasks),
            ("labels.jsonl", &labels),
        ] {
            std::fs::copy(src, fixtures.join(name))
                .unwrap_or_else(|e| panic!("rewriting {name}: {e}"));
        }
        eprintln!("TSR_REGEN_FIXTURES=1: rewrote {}", fixtures.display());
        return;
    }

    for name in ["events.jsonl", "tasks.jsonl", "labels.jsonl"] {
        let want = fixtures.join(name);
        let got = dir.join("fixtures").join(name);
        assert!(
            want.exists(),
            "fixture {} missing: run with TSR_REGEN_FIXTURES=1 to create it",
            want.display()
        );
        assert_eq!(
            read(&got),
            read(&want),
            "{name} differs from the committed fixture; the generator is not reproducible, or the \
             fixture is stale (run with TSR_REGEN_FIXTURES=1 to rewrite)"
        );
    }
}

#[test]
fn every_fixture_session_satisfies_the_m0_invariants() {
    let path = fixtures_dir().join("events.jsonl");
    assert!(
        path.exists(),
        "fixture missing at {}: run with TSR_REGEN_FIXTURES=1",
        path.display()
    );

    let session =
        reconstruct::from_events(&path).expect("the corpus reconstructs from events.jsonl");

    assert!(!session.events.is_empty(), "a corpus has events");
    assert!(session.monotone_seq(), "B6: sequence is monotone");
    assert!(
        session.every_parent_precedes_its_child(),
        "B6: causality is finite and prior"
    );
    // B5 is not asserted on this concatenation: the corpus is 380 task-local sessions, so a
    // commitment's global index here does not match its forecast's task-local state_version. B5 is
    // checked per task in `every_task_satisfies_the_m0_invariants_as_its_own_session`.
    assert!(
        session.every_dispatch_reconciled_or_unresolved(),
        "B6: every dispatch is reconciled, or explicitly unresolved — never silent"
    );
    assert!(
        session.every_command_ran_in_sandbox(),
        "B4: no command runs outside the sealed sandbox"
    );
}

/// A1/A3/A11: the invariants hold **per task**, not only over the whole corpus as one session.
///
/// Main's hook records a forecast's `state_version` as the number of events in the task's own log
/// (`hook.rs` `event_count`), so a task's log is its own session. Reconstructing all 380 tasks as
/// one session hides the defect: the session-global index happens to line up only when every task
/// is concatenated. Split per task, each task's commitment must still find its forecast at the
/// task-local version.
#[test]
fn every_task_satisfies_the_m0_invariants_as_its_own_session() {
    let path = fixtures_dir().join("events.jsonl");
    assert!(
        path.exists(),
        "fixture missing at {}: run with TSR_REGEN_FIXTURES=1",
        path.display()
    );

    let text = std::fs::read_to_string(&path).expect("events.jsonl is readable");
    // Group events by task, preserving file order within each task.
    let mut by_task: std::collections::BTreeMap<String, Vec<(i64, tsr_domain::Event)>> =
        std::collections::BTreeMap::new();
    for (idx, raw) in text.lines().enumerate() {
        let line = raw.trim();
        if line.is_empty() {
            continue;
        }
        let value: serde_json::Value = serde_json::from_str(line).expect("an event line is JSON");
        let (seq, ev_value) = match value.get("event") {
            Some(ev) => (value.get("seq").and_then(|s| s.as_i64()), ev.clone()),
            None => (None, value),
        };
        let event: tsr_domain::Event =
            serde_json::from_value(ev_value).expect("an event deserializes");
        let seq = seq.unwrap_or((idx + 1) as i64);
        by_task
            .entry(event.task.to_string())
            .or_default()
            .push((seq, event));
    }

    assert!(!by_task.is_empty(), "the corpus has tasks");
    let mut checked = 0usize;
    for (task, events) in &by_task {
        let session = reconstruct::Session {
            events: events.clone(),
        };
        assert!(
            session.monotone_seq(),
            "B6: sequence is monotone within task {task}"
        );
        assert!(
            session.every_parent_precedes_its_child(),
            "B6: causality is prior within task {task}"
        );
        assert!(
            session.every_gated_action_has_prior_forecast_at_same_state_version(),
            "B5: a gated action in task {task} has a forecast at its task-local state version"
        );
        assert!(
            session.every_dispatch_reconciled_or_unresolved(),
            "B6: every dispatch in task {task} is reconciled or unresolved"
        );
        assert!(
            session.every_command_ran_in_sandbox(),
            "B4: no command in task {task} runs outside the sandbox"
        );
        checked += 1;
    }
    assert_eq!(checked, 380, "every task is checked as its own session");
}

#[test]
fn the_corpus_has_the_shape_the_brief_names() {
    let dir = temp_dir("shape");
    let (_, tasks, labels) = regenerate(&dir);

    // tasks.jsonl: 380 rows, four families, at least eight lineages.
    let task_lines: Vec<serde_json::Value> = std::fs::read_to_string(&tasks)
        .expect("tasks.jsonl is readable")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("a task row is JSON"))
        .collect();
    assert_eq!(
        task_lines.len(),
        380,
        "three families of 120 plus one of 20"
    );

    let mut families: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    let mut lineages: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for row in &task_lines {
        *families
            .entry(row["family"].as_str().expect("family").to_string())
            .or_default() += 1;
        lineages.insert(row["lineage"].as_str().expect("lineage").to_string());
    }
    assert_eq!(families.get("overconfident"), Some(&120));
    assert_eq!(families.get("calibrated"), Some(&120));
    assert_eq!(families.get("underconfident"), Some(&120));
    assert_eq!(families.get("rare"), Some(&20));
    assert!(
        lineages.len() >= 8,
        "at least eight lineages: {}",
        lineages.len()
    );

    // labels.jsonl: only reviewed closes, all by the operator.
    let label_lines: Vec<serde_json::Value> = std::fs::read_to_string(&labels)
        .expect("labels.jsonl is readable")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("a label row is JSON"))
        .collect();
    assert!(!label_lines.is_empty(), "some tasks are reviewed");
    for row in &label_lines {
        assert_eq!(row["reviewer"].as_str(), Some("operator"));
    }
}

/// A1/A3/A11: two sessions in one store, driven through the **real** `tsr hook` and `tsr forecast`,
/// each pair their commitment with their own generator forecast.
///
/// This is the end-to-end form of the defect: the hook records a forecast's `state_version` as the
/// number of events in the task's own log, so the second session's commitment must pair with the
/// second session's forecast even though its events sit later in the store. Under the old
/// session-global pairing the second session's forecast (task-local version 1) would not match its
/// commitment's global index, and `generator_raw` would be `None`.
#[test]
fn two_real_sessions_pair_each_commitment_with_its_own_generator_forecast() {
    use std::process::Command;

    let dir = temp_dir("two-sessions");
    let db = dir.join("tsr.db");
    let out = dir.join("dataset");
    let tsr = PathBuf::from(env!("CARGO_BIN_EXE_tsr"));

    // Drive one session through the real binaries: SessionStart, `tsr forecast`, then the gated
    // PreToolUse Bash call that commits to the forecast candidate.
    let drive = |session: &str, cmd: &str| {
        let hook = |stage: &str, payload: serde_json::Value| {
            let mut child = Command::new(&tsr)
                .args(["hook", "--stage", stage])
                .env("TSR_DB", &db)
                .stdin(std::process::Stdio::piped())
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .expect("spawn tsr hook");
            use std::io::Write;
            child
                .stdin
                .as_mut()
                .expect("stdin")
                .write_all(payload.to_string().as_bytes())
                .expect("write payload");
            let out = child.wait_with_output().expect("tsr hook runs");
            assert!(
                out.status.success(),
                "tsr hook {stage} succeeds: {}",
                String::from_utf8_lossy(&out.stderr)
            );
        };

        hook(
            "SessionStart",
            serde_json::json!({
                "session_id": session,
                "hook_event_name": "SessionStart",
                "cwd": "/workspace",
            }),
        );

        let forecast = Command::new(&tsr)
            .args([
                "forecast",
                "--kind",
                "run_approved_test",
                "--args",
                cmd,
                "--p",
                "0.7",
                "--source",
                "generator",
                "--task",
                session,
            ])
            .env("TSR_DB", &db)
            .output()
            .expect("run tsr forecast");
        assert!(
            forecast.status.success(),
            "tsr forecast succeeds: {}",
            String::from_utf8_lossy(&forecast.stderr)
        );

        hook(
            "PreToolUse",
            serde_json::json!({
                "session_id": session,
                "hook_event_name": "PreToolUse",
                "tool_name": "Bash",
                "tool_input": { "command": cmd },
                "cwd": "/workspace",
            }),
        );
    };

    drive("m4-two-sessions-a", "cargo test --offline -p parser");
    drive("m4-two-sessions-b", "cargo test --offline -p lexer");

    // `tsr train dataset --store` reads the whole store and writes rows.jsonl.
    let dataset = Command::new(&tsr)
        .args([
            "train",
            "dataset",
            "--store",
            db.to_str().expect("db path"),
            "--out",
            out.to_str().expect("out path"),
            "--by",
            "time",
            "--calibrate",
            "0.2",
            "--holdout",
            "0.2",
            "--seed",
            "7",
        ])
        .output()
        .expect("run tsr train dataset");
    assert!(
        dataset.status.success(),
        "tsr train dataset succeeds: {}",
        String::from_utf8_lossy(&dataset.stderr)
    );

    let rows: Vec<serde_json::Value> = std::fs::read_to_string(out.join("rows.jsonl"))
        .expect("rows.jsonl is readable")
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("a row is JSON"))
        .collect();

    assert_eq!(rows.len(), 2, "one row per session's commitment");
    for row in &rows {
        assert!(
            row["generator_raw"].is_number(),
            "A1/A3/A11: the commitment in task {} pairs with its generator forecast, got {}",
            row["task"],
            row["generator_raw"]
        );
    }
}
