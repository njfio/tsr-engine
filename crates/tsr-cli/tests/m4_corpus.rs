//! M4 corpus test (lane 06, item 2): the synthetic corpus regenerates byte-for-byte, and every
//! fixture session satisfies all of `tsr_cli::reconstruct`'s invariants.
//!
//! The generator lives in `tsr_learn::synth`; it is seeded and clock-free, so the same seed yields
//! the same bytes on every platform. This test regenerates into a temp dir and compares against
//! `fixtures/m4/`. Set `TSR_REGEN_FIXTURES=1` to rewrite the fixtures in place instead.
//!
//! The invariants are the PRD's, not the implementer's (see `crates/tsr-cli/src/lib.rs`):
//! - B6: sequence is monotone; every parent precedes its child.
//! - B5: every gated action has a forecast at the same state version, recorded before commitment.
//! - B6: every dispatch is reconciled or explicitly unresolved.
//! - B4: every command ran in the sealed sibling sandbox.

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

    let session = reconstruct::from_events(&path).expect("the corpus reconstructs from events.jsonl");

    assert!(!session.events.is_empty(), "a corpus has events");
    assert!(session.monotone_seq(), "B6: sequence is monotone");
    assert!(
        session.every_parent_precedes_its_child(),
        "B6: causality is finite and prior"
    );
    assert!(
        session.every_gated_action_has_prior_forecast_at_same_state_version(),
        "B5: a gated action is denied until a forecast exists for the current state version"
    );
    assert!(
        session.every_dispatch_reconciled_or_unresolved(),
        "B6: every dispatch is reconciled, or explicitly unresolved — never silent"
    );
    assert!(
        session.every_command_ran_in_sandbox(),
        "B4: no command runs outside the sealed sandbox"
    );
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
    assert_eq!(task_lines.len(), 380, "three families of 120 plus one of 20");

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
    assert!(lineages.len() >= 8, "at least eight lineages: {}", lineages.len());

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