//! Tests for `tsr corpus split` (PRD v0.3.1, §B9, §B11 M3).
//!
//! Every test builds a corpus directory by hand — a `tasks.jsonl` and a `cas/` — so the splitter
//! is exercised without a repository, a runner or a container. The leakage probes each get a
//! fixture that trips them, and the test asserts the split **refuses**.

use std::fs;
use std::path::{Path, PathBuf};

use tsr_corpus::split::{
    self, clone_groups, diff_similarity, holdout_count, normalize_diff, shingles, SplitError,
    SplitOptions, SplitRule, CLONE_JACCARD,
};

/// A corpus directory under a temp dir, removed when the guard drops.
struct Corpus {
    dir: tempfile::TempDir,
}

impl Corpus {
    fn new() -> Self {
        Corpus {
            dir: tempfile::tempdir().expect("tempdir"),
        }
    }

    fn path(&self) -> &Path {
        self.dir.path()
    }

    /// Write a CAS object and return its digest.
    fn put_cas(&self, bytes: &[u8]) -> String {
        let digest = tsr_corpus::digest(bytes);
        let cas = self.path().join("cas");
        fs::create_dir_all(&cas).expect("cas dir");
        fs::write(cas.join(&digest), bytes).expect("cas write");
        digest
    }

    /// Write `tasks.jsonl` from a list of task JSON values.
    fn write_tasks(&self, tasks: &[serde_json::Value]) {
        let mut body = String::new();
        for task in tasks {
            body.push_str(&serde_json::to_string(task).expect("task json"));
            body.push('\n');
        }
        fs::write(self.path().join("tasks.jsonl"), body).expect("tasks.jsonl");
    }
}

/// A task JSON with the shared shape, plus whatever extra fields the caller adds.
fn task(
    id: &str,
    repo: &str,
    timestamp: &str,
    diff_digest: &str,
    parent_snapshot: &str,
) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "repo": repo,
        "lineage": repo,
        "family": "crate-a",
        "timestamp": timestamp,
        "hidden_diff": diff_digest,
        "parent_snapshot": parent_snapshot,
        "flipped_tests": ["tests::flip"],
    })
}

/// A diff that changes `fn f` to return `n`, with a long shared body so two of these are
/// near-clones (they differ only in the returned constant).
fn diff_returning(n: u32) -> String {
    format!(
        "diff --git a/src/lib.rs b/src/lib.rs\n\
         index 1111111..2222222 100644\n\
         --- a/src/lib.rs\n\
         +++ b/src/lib.rs\n\
         @@ -1,8 +1,8 @@\n\
         -fn f() -> u32 {{ 0 }}\n\
         +fn f() -> u32 {{ {n} }}\n\
         \n\
         fn helper_alpha(x: u32) -> u32 {{ x + 1 }}\n\
         fn helper_beta(x: u32) -> u32 {{ x * 2 }}\n\
         fn helper_gamma(x: u32) -> u32 {{ x - 3 }}\n\
         fn helper_delta(x: u32) -> u32 {{ x / 4 }}\n\
         fn helper_epsilon(x: u32) -> u32 {{ x % 5 }}\n"
    )
}

/// A diff that changes an unrelated function, so it is not a clone of `diff_returning`.
fn diff_unrelated(tag: &str) -> String {
    format!(
        "diff --git a/src/other.rs b/src/other.rs\n\
         --- a/src/other.rs\n\
         +++ b/src/other.rs\n\
         @@ -10,2 +10,2 @@\n\
         -let {tag} = compute_alpha();\n\
         +let {tag} = compute_beta();\n"
    )
}

// ---------------------------------------------------------------------------------------------
// Diff similarity
// ---------------------------------------------------------------------------------------------

#[test]
fn normalize_diff_drops_headers_and_markers() {
    let normalized = normalize_diff(&diff_returning(1));
    assert!(!normalized.contains("diff --git"), "{normalized}");
    assert!(!normalized.contains("@@"), "{normalized}");
    assert!(normalized.contains("fn f() -> u32 { 1 }"), "{normalized}");
}

#[test]
fn identical_diffs_are_similarity_one() {
    let a = diff_returning(1);
    assert_eq!(diff_similarity(&a, &a), 1.0);
}

#[test]
fn unrelated_diffs_are_below_the_clone_threshold() {
    let sim = diff_similarity(&diff_returning(1), &diff_unrelated("x"));
    assert!(
        sim < CLONE_JACCARD,
        "similarity {sim} should be < {CLONE_JACCARD}"
    );
}

#[test]
fn near_identical_diffs_are_above_the_clone_threshold() {
    // The same change with a different context line: still a near-clone.
    let a = diff_returning(1);
    let b = format!("{a} \n context line that differs\n");
    let sim = diff_similarity(&a, &b);
    assert!(
        sim >= CLONE_JACCARD,
        "similarity {sim} should be >= {CLONE_JACCARD}"
    );
}

#[test]
fn shingles_of_a_short_diff_are_not_empty() {
    assert!(!shingles("+x").is_empty());
}

// ---------------------------------------------------------------------------------------------
// Clone groups
// ---------------------------------------------------------------------------------------------

#[test]
fn clone_groups_join_near_clones_and_leave_singletons_alone() {
    let corpus = Corpus::new();
    let d1 = corpus.put_cas(diff_returning(1).as_bytes());
    let d2 = corpus.put_cas(diff_returning(2).as_bytes());
    let d3 = corpus.put_cas(diff_unrelated("y").as_bytes());
    let tasks = vec![
        task("t1", "repo-a", "2024-01-01T00:00:00Z", &d1, ""),
        task("t2", "repo-a", "2024-01-02T00:00:00Z", &d2, ""),
        task("t3", "repo-b", "2024-01-03T00:00:00Z", &d3, ""),
    ];
    corpus.write_tasks(&tasks);
    let read = split::read_tasks(corpus.path()).expect("read");
    let mut diffs = std::collections::BTreeMap::new();
    for t in &read {
        diffs.insert(t.id.clone(), split::task_diff(corpus.path(), t));
    }
    let groups = clone_groups(&read, &diffs, CLONE_JACCARD);
    assert_eq!(groups.len(), 1, "one clone group expected: {groups:?}");
    assert_eq!(groups[0].members, vec!["t1".to_string(), "t2".to_string()]);
}

// ---------------------------------------------------------------------------------------------
// Holdout sizing
// ---------------------------------------------------------------------------------------------

#[test]
fn holdout_count_never_empties_the_train_side() {
    assert_eq!(holdout_count(0, 0.2), 0);
    assert_eq!(holdout_count(1, 0.2), 0);
    assert_eq!(holdout_count(10, 0.2), 2);
    assert_eq!(holdout_count(10, 0.99), 9);
    assert_eq!(holdout_count(2, 0.5), 1);
}

// ---------------------------------------------------------------------------------------------
// The split: clone groups never split, repo separation, forward time, determinism
// ---------------------------------------------------------------------------------------------

/// A corpus of six tasks across two repos, with one near-clone pair.
fn six_task_corpus() -> Corpus {
    let corpus = Corpus::new();
    let d1 = corpus.put_cas(diff_returning(1).as_bytes());
    let d2 = corpus.put_cas(diff_returning(2).as_bytes()); // near-clone of d1
    let d3 = corpus.put_cas(diff_unrelated("a").as_bytes());
    let d4 = corpus.put_cas(diff_unrelated("b").as_bytes());
    let d5 = corpus.put_cas(diff_unrelated("c").as_bytes());
    let d6 = corpus.put_cas(diff_unrelated("d").as_bytes());
    let tasks = vec![
        task("t1", "repo-a", "2024-01-01T00:00:00Z", &d1, ""),
        task("t2", "repo-a", "2024-01-02T00:00:00Z", &d2, ""),
        task("t3", "repo-a", "2024-01-03T00:00:00Z", &d3, ""),
        task("t4", "repo-b", "2024-01-04T00:00:00Z", &d4, ""),
        task("t5", "repo-b", "2024-01-05T00:00:00Z", &d5, ""),
        task("t6", "repo-b", "2024-01-06T00:00:00Z", &d6, ""),
    ];
    corpus.write_tasks(&tasks);
    corpus
}

#[test]
fn clone_group_is_never_split_across_the_boundary() {
    let corpus = six_task_corpus();
    let opts = SplitOptions {
        rule: SplitRule::Time,
        holdout: 0.5,
        seed: 7,
        ..Default::default()
    };
    let manifest = split::split(corpus.path(), &opts).expect("split");
    // t1 and t2 are near-clones: they must be on the same side.
    let t1_train = manifest.train.contains(&"t1".to_string());
    let t2_train = manifest.train.contains(&"t2".to_string());
    assert_eq!(t1_train, t2_train, "clone group split: {manifest:?}");
    let group = manifest
        .clone_groups
        .iter()
        .find(|g| g.members.contains(&"t1".to_string()))
        .expect("clone group for t1");
    assert_eq!(group.members, vec!["t1".to_string(), "t2".to_string()]);
    assert_eq!(group.side, if t1_train { "train" } else { "holdout" });
}

#[test]
fn repo_split_keeps_every_lineage_on_one_side() {
    let corpus = six_task_corpus();
    let opts = SplitOptions {
        rule: SplitRule::Repo,
        holdout: 0.5,
        seed: 3,
        ..Default::default()
    };
    let manifest = split::split(corpus.path(), &opts).expect("split");
    // No lineage may appear on both sides.
    let train: std::collections::BTreeSet<&String> = manifest.train.iter().collect();
    let holdout: std::collections::BTreeSet<&String> = manifest.holdout.iter().collect();
    let both: Vec<&&String> = train.intersection(&holdout).collect();
    assert!(both.is_empty(), "lineage on both sides: {both:?}");
    // repo-a's three tasks are all on one side.
    let a_train = manifest.train.contains(&"t1".to_string());
    assert_eq!(a_train, manifest.train.contains(&"t2".to_string()));
    assert_eq!(a_train, manifest.train.contains(&"t3".to_string()));
    // And the manifest records the lineages per side.
    assert!(manifest.train_lineages.len() + manifest.holdout_lineages.len() == 2);
}

#[test]
fn time_split_is_forward_in_time() {
    let corpus = six_task_corpus();
    let opts = SplitOptions {
        rule: SplitRule::Time,
        holdout: 0.34,
        seed: 0,
        ..Default::default()
    };
    let manifest = split::split(corpus.path(), &opts).expect("split");
    // Every holdout task's timestamp is >= every train task's timestamp, except for clone-group
    // members which are pulled to the same side. t1/t2 are the clone pair; with seed 0 the
    // holdout is the later tasks.
    let ts = |id: &str| -> String {
        match id {
            "t1" => "2024-01-01T00:00:00Z",
            "t2" => "2024-01-02T00:00:00Z",
            "t3" => "2024-01-03T00:00:00Z",
            "t4" => "2024-01-04T00:00:00Z",
            "t5" => "2024-01-05T00:00:00Z",
            "t6" => "2024-01-06T00:00:00Z",
            _ => "",
        }
        .to_string()
    };
    let max_train = manifest
        .train
        .iter()
        .map(|t| ts(t))
        .max()
        .unwrap_or_default();
    let min_holdout = manifest
        .holdout
        .iter()
        .map(|t| ts(t))
        .min()
        .unwrap_or_default();
    assert!(
        min_holdout >= max_train,
        "holdout {min_holdout} precedes train {max_train}: {manifest:?}"
    );
}

#[test]
fn split_is_deterministic_under_a_fixed_seed() {
    let corpus = six_task_corpus();
    let opts = SplitOptions {
        rule: SplitRule::Time,
        holdout: 0.5,
        seed: 42,
        ..Default::default()
    };
    let a = split::split(corpus.path(), &opts).expect("split a");
    let b = split::split(corpus.path(), &opts).expect("split b");
    assert_eq!(a.train, b.train);
    assert_eq!(a.holdout, b.holdout);
    assert_eq!(a.clone_groups, b.clone_groups);
}

#[test]
fn different_seeds_can_move_the_boundary() {
    let corpus = six_task_corpus();
    let mut seen = std::collections::BTreeSet::new();
    for seed in 0..8u64 {
        let opts = SplitOptions {
            rule: SplitRule::Repo,
            holdout: 0.5,
            seed,
            ..Default::default()
        };
        let m = split::split(corpus.path(), &opts).expect("split");
        seen.insert(m.holdout.clone());
    }
    assert!(
        seen.len() > 1,
        "the seed never moved the boundary: {seen:?}"
    );
}

#[test]
fn holdout_access_is_logged_in_the_manifest() {
    let corpus = six_task_corpus();
    let opts = SplitOptions {
        rule: SplitRule::Time,
        holdout: 0.5,
        seed: 1,
        command: "tsr corpus split /tmp/corpus --by time --holdout 0.5 --seed 1".to_string(),
        ..Default::default()
    };
    let manifest = split::split(corpus.path(), &opts).expect("split");
    assert!(
        !manifest.holdout_access.is_empty(),
        "no holdout access logged: {manifest:?}"
    );
    let first = &manifest.holdout_access[0];
    assert_eq!(first.accessed, "*");
    assert!(first.by.contains("tsr corpus split"), "{}", first.by);
    assert!(!first.at.is_empty(), "access has no timestamp");
    // The manifest is on disk too.
    let on_disk: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(corpus.path().join("split.json")).expect("read"))
            .expect("json");
    assert!(on_disk["holdout_access"]
        .as_array()
        .is_some_and(|a| !a.is_empty()));
}

#[test]
fn bad_fraction_is_refused() {
    let corpus = six_task_corpus();
    let opts = SplitOptions {
        holdout: 1.5,
        ..Default::default()
    };
    assert!(matches!(
        split::split(corpus.path(), &opts),
        Err(SplitError::BadFraction(_))
    ));
}

#[test]
fn unknown_rule_is_refused() {
    assert!(matches!(
        SplitRule::parse("nonsense"),
        Err(SplitError::BadRule(_))
    ));
}

// ---------------------------------------------------------------------------------------------
// Leakage probes — each must be CAUGHT
// ---------------------------------------------------------------------------------------------

/// Probe (a): a task whose hidden diff appears in another split's parent snapshot.
///
/// The fixture puts the *same* diff in the parent snapshot of a task on the other side: the agent
/// on that side would read the answer in the snapshot it starts from.
#[test]
fn probe_a_hidden_diff_in_other_parent_snapshot_is_caught() {
    let corpus = Corpus::new();
    let leaked = diff_returning(9);
    let leaked_digest = corpus.put_cas(leaked.as_bytes());
    // The parent snapshot of the holdout task contains the train task's diff verbatim.
    let parent_with_leak = corpus.put_cas(leaked.as_bytes());
    let other = corpus.put_cas(diff_unrelated("z").as_bytes());
    let tasks = vec![
        task("t1", "repo-a", "2024-01-01T00:00:00Z", &leaked_digest, ""),
        task(
            "t2",
            "repo-b",
            "2024-01-02T00:00:00Z",
            &other,
            &parent_with_leak,
        ),
    ];
    corpus.write_tasks(&tasks);
    let opts = SplitOptions {
        rule: SplitRule::Repo,
        holdout: 0.5,
        seed: 0,
        ..Default::default()
    };
    let err = split::split(corpus.path(), &opts).expect_err("leak must be caught");
    match err {
        SplitError::Leakage { probe, .. } => {
            assert_eq!(probe, "hidden_diff_in_other_parent_snapshot")
        }
        other => panic!("wrong error: {other:?}"),
    }
    // The manifest is still written, with the probe verdict recorded.
    let on_disk: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(corpus.path().join("split.json")).expect("read"))
            .expect("json");
    let probes = on_disk["leakage_probes"].as_array().expect("probes");
    assert!(probes
        .iter()
        .any(|p| p["probe"] == "hidden_diff_in_other_parent_snapshot" && p["verdict"] == "caught"));
}

/// Probe (b): a test added after the task's timestamp.
#[test]
fn probe_b_test_added_after_timestamp_is_caught() {
    let corpus = Corpus::new();
    let d1 = corpus.put_cas(diff_returning(1).as_bytes());
    let d2 = corpus.put_cas(diff_unrelated("q").as_bytes());
    let mut t1 = task("t1", "repo-a", "2024-01-01T00:00:00Z", &d1, "");
    // The oracle names a test that was added a month after the task.
    t1["test_timestamps"] = serde_json::json!({
        "tests::flip": "2024-02-01T00:00:00Z"
    });
    let t2 = task("t2", "repo-b", "2024-01-02T00:00:00Z", &d2, "");
    corpus.write_tasks(&[t1, t2]);
    let opts = SplitOptions {
        rule: SplitRule::Repo,
        holdout: 0.5,
        seed: 0,
        ..Default::default()
    };
    let err = split::split(corpus.path(), &opts).expect_err("future test must be caught");
    match err {
        SplitError::Leakage { probe, .. } => assert_eq!(probe, "test_added_after_timestamp"),
        other => panic!("wrong error: {other:?}"),
    }
}

/// Probe (b) negative control: a test that predates the task is clean.
#[test]
fn probe_b_clean_when_the_test_predates_the_task() {
    let corpus = Corpus::new();
    let d1 = corpus.put_cas(diff_returning(1).as_bytes());
    let d2 = corpus.put_cas(diff_unrelated("q").as_bytes());
    let mut t1 = task("t1", "repo-a", "2024-01-01T00:00:00Z", &d1, "");
    t1["test_timestamps"] = serde_json::json!({
        "tests::flip": "2023-12-01T00:00:00Z"
    });
    let t2 = task("t2", "repo-b", "2024-01-02T00:00:00Z", &d2, "");
    corpus.write_tasks(&[t1, t2]);
    let opts = SplitOptions {
        rule: SplitRule::Repo,
        holdout: 0.5,
        seed: 0,
        ..Default::default()
    };
    let manifest = split::split(corpus.path(), &opts).expect("clean split");
    let probe = manifest
        .leakage_probes
        .iter()
        .find(|p| p.probe == "test_added_after_timestamp")
        .expect("probe present");
    assert_eq!(probe.verdict, "clean");
}

/// Probe (c): a post-action log.
#[test]
fn probe_c_post_action_log_is_caught() {
    let corpus = Corpus::new();
    let d1 = corpus.put_cas(diff_returning(1).as_bytes());
    let d2 = corpus.put_cas(diff_unrelated("q").as_bytes());
    let mut t1 = task("t1", "repo-a", "2024-01-01T00:00:00Z", &d1, "");
    // A log entry recorded after the action leaks the outcome.
    t1["post_action_log"] = serde_json::json!([
        {"timestamp": "2024-01-01T00:05:00Z", "event": "oracle_passed"}
    ]);
    let t2 = task("t2", "repo-b", "2024-01-02T00:00:00Z", &d2, "");
    corpus.write_tasks(&[t1, t2]);
    let opts = SplitOptions {
        rule: SplitRule::Repo,
        holdout: 0.5,
        seed: 0,
        ..Default::default()
    };
    let err = split::split(corpus.path(), &opts).expect_err("post-action log must be caught");
    match err {
        SplitError::Leakage { probe, .. } => assert_eq!(probe, "post_action_log"),
        other => panic!("wrong error: {other:?}"),
    }
}

/// Probe (c) negative control: a log entry that predates the task is clean.
#[test]
fn probe_c_clean_when_the_log_predates_the_task() {
    let corpus = Corpus::new();
    let d1 = corpus.put_cas(diff_returning(1).as_bytes());
    let d2 = corpus.put_cas(diff_unrelated("q").as_bytes());
    let mut t1 = task("t1", "repo-a", "2024-01-01T00:00:00Z", &d1, "");
    t1["post_action_log"] = serde_json::json!([
        {"timestamp": "2023-12-31T00:00:00Z", "event": "pre_action_note"}
    ]);
    let t2 = task("t2", "repo-b", "2024-01-02T00:00:00Z", &d2, "");
    corpus.write_tasks(&[t1, t2]);
    let opts = SplitOptions {
        rule: SplitRule::Repo,
        holdout: 0.5,
        seed: 0,
        ..Default::default()
    };
    let manifest = split::split(corpus.path(), &opts).expect("clean split");
    let probe = manifest
        .leakage_probes
        .iter()
        .find(|p| p.probe == "post_action_log")
        .expect("probe present");
    assert_eq!(probe.verdict, "clean");
}

/// The three probes are always recorded in the manifest, even when clean.
#[test]
fn all_three_probes_are_recorded() {
    let corpus = six_task_corpus();
    let opts = SplitOptions {
        rule: SplitRule::Time,
        holdout: 0.5,
        seed: 0,
        ..Default::default()
    };
    let manifest = split::split(corpus.path(), &opts).expect("split");
    let names: Vec<&str> = manifest
        .leakage_probes
        .iter()
        .map(|p| p.probe.as_str())
        .collect();
    assert_eq!(
        names,
        vec![
            "hidden_diff_in_other_parent_snapshot",
            "test_added_after_timestamp",
            "post_action_log"
        ]
    );
    assert!(manifest.leakage_probes.iter().all(|p| p.verdict == "clean"));
}

/// A corpus with no tasks is refused rather than emitting an empty manifest.
#[test]
fn empty_corpus_is_refused() {
    let corpus = Corpus::new();
    corpus.write_tasks(&[]);
    let opts = SplitOptions::default();
    assert!(matches!(
        split::split(corpus.path(), &opts),
        Err(SplitError::NoTasks(_))
    ));
}

/// A missing `tasks.jsonl` is refused with a path in the message.
#[test]
fn missing_tasks_file_is_refused() {
    let corpus = Corpus::new();
    let opts = SplitOptions::default();
    let err = split::split(corpus.path(), &opts).expect_err("missing file");
    assert!(err.to_string().contains("tasks.jsonl"), "{err}");
}

/// The manifest round-trips through JSON.
#[test]
fn manifest_round_trips() {
    let corpus = six_task_corpus();
    let opts = SplitOptions {
        rule: SplitRule::Repo,
        holdout: 0.5,
        seed: 5,
        ..Default::default()
    };
    let manifest = split::split(corpus.path(), &opts).expect("split");
    let text = serde_json::to_string(&manifest).expect("serialize");
    let back: split::SplitManifest = serde_json::from_str(&text).expect("deserialize");
    assert_eq!(manifest, back);
}

/// A `PathBuf` helper so the test file compiles without an unused import.
#[allow(dead_code)]
fn _pathbuf_marker(p: PathBuf) -> PathBuf {
    p
}
