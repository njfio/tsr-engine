//! # `tsr corpus split` — train/holdout splits with clone groups and leakage probes
//! (PRD v0.3.1, §B9, §B11 M3)
//!
//! The PRD's data discipline (§B9): *"Splits are made before any fitting: forward-time by task,
//! lineage-separated by repository, near-clones deduplicated across the boundary (mutant tasks
//! especially). Leakage probes cover future tests, later patches, post-action logs and retrieved
//! episodes."* and *"Holdout access is logged; a holdout tuned against is retired and replaced
//! under a written rule."*
//!
//! This module reads the `tasks.jsonl` and CAS that [`crate::mine`] and [`crate::mutants`] write
//! and produces a **split manifest** (JSON) naming the rule, the seed, the holdout fraction, the
//! train and holdout task ids, the clone groups, and every holdout access.
//!
//! ## The two rules
//!
//! - `--by time`: forward-time by task timestamp. Tasks are sorted by `(timestamp, id)` and the
//!   later `holdout` fraction becomes the holdout. Deterministic given the seed: the seed only
//!   breaks ties among tasks that share a timestamp, so a re-run with the same seed and the same
//!   corpus yields the same split.
//! - `--by repo`: lineage-separated by repository. Whole lineages are assigned to one side, so no
//!   repository's tasks appear in both splits. The seed shuffles the lineage order before the
//!   holdout fraction is taken, so the split is deterministic given the seed but not trivially
//!   "the last repository alphabetically".
//!
//! ## Near-clone deduplication
//!
//! Two tasks are near-clones when their hidden diffs are similar: the diff is normalized (paths
//! stripped of their directory prefix, whitespace collapsed, blank lines dropped), split into
//! word shingles, and compared by Jaccard similarity. `Jaccard >= 0.8` puts them in one **clone
//! group**, and a clone group is **never split across the boundary** — every member lands on the
//! same side. The groups are recorded in the manifest.
//!
//! ## Leakage probes
//!
//! Three probes are implemented as real checks, each of which must be **caught** (the split
//! refuses with a [`SplitError::Leakage`] naming the probe):
//!
//! - **(a) hidden diff in another split's parent snapshot** — a task whose hidden diff is
//!   byte-identical to (or a near-clone of) a diff already present in the *other* split's parent
//!   snapshot. The parent snapshot is the CAS object named by the task's `parent_snapshot` when it
//!   is a CAS digest, else the task's own hidden diff is compared against the other side's parent
//!   snapshot digests.
//! - **(b) a test added after the task's timestamp** — the task's `flipped_tests` (or
//!   `failing_tests`) name a test whose own timestamp is later than the task's timestamp, i.e. the
//!   oracle depends on a test that did not exist when the task was created.
//! - **(c) a post-action log** — the task carries a log entry (a `post_action_log` field, or a
//!   `log` array whose entries are timestamped after the task's timestamp) that records what
//!   happened *after* the action, which would leak the outcome to the agent.
//!
//! A probe that cannot be caught is a defect, not a note: the tests in `tests/split.rs` feed a
//! fixture that trips each one and assert the error.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::digest;

/// The default Jaccard threshold above which two diffs are one clone group.
pub const CLONE_JACCARD: f64 = 0.8;

/// The number of words in a diff shingle.
pub const SHINGLE_SIZE: usize = 3;

/// Which rule produced the split.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SplitRule {
    /// Forward-time by task timestamp.
    Time,
    /// Lineage-separated by repository.
    Repo,
}

impl SplitRule {
    /// Parse the `--by` value.
    pub fn parse(s: &str) -> Result<Self, SplitError> {
        match s {
            "time" => Ok(SplitRule::Time),
            "repo" => Ok(SplitRule::Repo),
            other => Err(SplitError::BadRule(other.to_string())),
        }
    }

    /// The rule's name as it appears in the manifest.
    pub fn as_str(&self) -> &'static str {
        match self {
            SplitRule::Time => "time",
            SplitRule::Repo => "repo",
        }
    }
}

/// One task as the splitter reads it from `tasks.jsonl`.
///
/// The splitter is deliberately tolerant of the two task shapes ([`crate::mine::MinedTask`] and
/// [`crate::mutants::MutantTask`]): it reads the fields both share and the fields each adds, and
/// keeps the raw JSON so nothing is lost.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SplitTask {
    /// The task id (blake3 of the source tuple).
    pub id: String,
    /// The repository the task came from. Also the lineage.
    pub repo: String,
    /// The lineage (repository) — defaults to `repo` when absent.
    #[serde(default)]
    pub lineage: String,
    /// The family (crate, or `(operator, crate)` for mutants).
    #[serde(default)]
    pub family: String,
    /// The task's timestamp, ISO-8601.
    #[serde(default)]
    pub timestamp: String,
    /// The hidden diff's CAS digest (mine) or the reversal diff's CAS digest (mutants).
    #[serde(default)]
    pub hidden_diff: String,
    /// The reversal diff's CAS digest (mutants only).
    #[serde(default)]
    pub reversal_diff: String,
    /// The parent snapshot digest (mine only).
    #[serde(default)]
    pub parent_snapshot: String,
    /// The mutated snapshot digest (mutants only).
    #[serde(default)]
    pub mutated_snapshot: String,
    /// The tests that flipped red→green (mine).
    #[serde(default)]
    pub flipped_tests: Vec<String>,
    /// The tests that failed under the mutant (mutants).
    #[serde(default)]
    pub failing_tests: Vec<String>,
    /// A post-action log, if the task carries one. Any entry timestamped after the task's
    /// timestamp is leakage probe (c).
    #[serde(default)]
    pub post_action_log: Option<serde_json::Value>,
    /// A generic log array, if the task carries one. Timestamped entries after the task's
    /// timestamp are leakage probe (c).
    #[serde(default)]
    pub log: Option<serde_json::Value>,
    /// The raw task JSON, preserved so the manifest can point back at the source.
    #[serde(default)]
    pub raw: serde_json::Value,
}

impl SplitTask {
    /// The CAS digest of this task's hidden diff: `hidden_diff` for a mined task,
    /// `reversal_diff` for a mutant task.
    pub fn diff_digest(&self) -> &str {
        if !self.hidden_diff.is_empty() {
            &self.hidden_diff
        } else {
            &self.reversal_diff
        }
    }

    /// The lineage, falling back to the repository when the field is absent.
    pub fn lineage(&self) -> &str {
        if self.lineage.is_empty() {
            &self.repo
        } else {
            &self.lineage
        }
    }

    /// The tests the oracle depends on: flipped tests for a mined task, failing tests for a
    /// mutant.
    pub fn oracle_tests(&self) -> Vec<&String> {
        let mut out: Vec<&String> = self.flipped_tests.iter().collect();
        out.extend(self.failing_tests.iter());
        out
    }
}

/// A clone group: tasks whose hidden diffs are near-clones (Jaccard >= [`CLONE_JACCARD`]).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CloneGroup {
    /// A stable id for the group: blake3 of the sorted member ids.
    pub id: String,
    /// The member task ids, sorted.
    pub members: Vec<String>,
    /// The side the whole group landed on: `"train"` or `"holdout"`.
    pub side: String,
    /// The pairwise Jaccard similarities that put the members together, for the record.
    pub similarities: Vec<CloneEdge>,
}

/// One edge of a clone group: two tasks and the Jaccard similarity of their diffs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CloneEdge {
    pub a: String,
    pub b: String,
    pub jaccard: f64,
}

/// One logged holdout access (PRD §B9: "Holdout access is logged").
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HoldoutAccess {
    /// What was accessed: the task id, or `"*"` for a whole-split access.
    pub accessed: String,
    /// When the access happened, ISO-8601 UTC.
    pub at: String,
    /// Which command or rule made the access.
    pub by: String,
    /// Why the access happened.
    pub reason: String,
}

/// The split manifest written to `<dir>/split.json`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SplitManifest {
    /// The rule used.
    pub rule: String,
    /// The seed the split was made with.
    pub seed: u64,
    /// The holdout fraction requested.
    pub holdout_fraction: f64,
    /// The Jaccard threshold used for clone grouping.
    pub clone_jaccard: f64,
    /// The train task ids, sorted.
    pub train: Vec<String>,
    /// The holdout task ids, sorted.
    pub holdout: Vec<String>,
    /// The clone groups, sorted by id.
    pub clone_groups: Vec<CloneGroup>,
    /// Every holdout access logged during the split.
    pub holdout_access: Vec<HoldoutAccess>,
    /// The leakage probes that were run and their verdicts.
    pub leakage_probes: Vec<LeakageProbe>,
    /// The lineages on each side, for the record.
    pub train_lineages: Vec<String>,
    pub holdout_lineages: Vec<String>,
}

/// One leakage probe's verdict.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LeakageProbe {
    /// The probe's name: `hidden_diff_in_other_parent_snapshot`, `test_added_after_timestamp`,
    /// `post_action_log`.
    pub probe: String,
    /// `"caught"` when the probe found leakage, `"clean"` when it did not.
    pub verdict: String,
    /// The tasks the probe flagged.
    pub flagged: Vec<String>,
    /// A human-readable note.
    pub note: String,
}

/// How to split.
#[derive(Clone, Debug)]
pub struct SplitOptions {
    /// The rule.
    pub rule: SplitRule,
    /// The holdout fraction, in `(0, 1)`.
    pub holdout: f64,
    /// The seed.
    pub seed: u64,
    /// The Jaccard threshold for clone grouping.
    pub clone_jaccard: f64,
    /// The command string recorded in every holdout access entry.
    pub command: String,
}

impl Default for SplitOptions {
    fn default() -> Self {
        Self {
            rule: SplitRule::Time,
            holdout: 0.2,
            seed: 0,
            clone_jaccard: CLONE_JACCARD,
            command: "tsr corpus split".to_string(),
        }
    }
}

/// Errors from a split run.
#[derive(Debug, thiserror::Error)]
pub enum SplitError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unknown split rule `{0}`: expected `time` or `repo`")]
    BadRule(String),
    #[error("holdout fraction {0} is not in (0, 1)")]
    BadFraction(f64),
    #[error("no tasks in {0}")]
    NoTasks(String),
    #[error("leakage probe `{probe}` caught: {detail}")]
    Leakage { probe: String, detail: String },
}

// ---------------------------------------------------------------------------------------------
// Diff similarity
// ---------------------------------------------------------------------------------------------

/// Normalize a unified diff for shingling: drop the `---`/`+++`/`@@`/`index`/`diff --git`
/// header lines, strip the leading `+`/`-`/space marker, strip directory prefixes from paths,
/// collapse whitespace, and drop blank lines.
///
/// The point is that two diffs that change the same code in the same way compare as similar even
/// when they were produced at different paths or with different context.
pub fn normalize_diff(diff: &str) -> String {
    let mut out = String::new();
    for line in diff.lines() {
        let trimmed = line.trim_end();
        if trimmed.starts_with("diff --git")
            || trimmed.starts_with("index ")
            || trimmed.starts_with("--- ")
            || trimmed.starts_with("+++ ")
            || trimmed.starts_with("@@")
            || trimmed.starts_with("new file mode")
            || trimmed.starts_with("deleted file mode")
            || trimmed.starts_with("similarity index")
            || trimmed.starts_with("rename ")
        {
            continue;
        }
        // Strip the diff marker, then collapse whitespace.
        let body = trimmed
            .strip_prefix('+')
            .or_else(|| trimmed.strip_prefix('-'))
            .or_else(|| trimmed.strip_prefix(' '))
            .unwrap_or(trimmed);
        let collapsed: String = body.split_whitespace().collect::<Vec<_>>().join(" ");
        if collapsed.is_empty() {
            continue;
        }
        out.push_str(&collapsed);
        out.push('\n');
    }
    out
}

/// The word shingles of a normalized diff: every window of [`SHINGLE_SIZE`] words.
pub fn shingles(diff: &str) -> BTreeSet<String> {
    let normalized = normalize_diff(diff);
    let words: Vec<&str> = normalized.split_whitespace().collect();
    if words.len() < SHINGLE_SIZE {
        // A diff too short to shingle is its own single shingle, so two identical short diffs
        // still compare equal.
        let mut set = BTreeSet::new();
        if !words.is_empty() {
            set.insert(words.join(" "));
        }
        return set;
    }
    let mut set = BTreeSet::new();
    for window in words.windows(SHINGLE_SIZE) {
        set.insert(window.join(" "));
    }
    set
}

/// The Jaccard similarity of two shingle sets: `|A ∩ B| / |A ∪ B|`, with two empty sets defined as
/// `1.0` (identical) and one empty set as `0.0`.
pub fn jaccard(a: &BTreeSet<String>, b: &BTreeSet<String>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let inter = a.intersection(b).count() as f64;
    let union = a.union(b).count() as f64;
    inter / union
}

/// The Jaccard similarity of two raw diffs.
pub fn diff_similarity(a: &str, b: &str) -> f64 {
    jaccard(&shingles(a), &shingles(b))
}

// ---------------------------------------------------------------------------------------------
// Reading the corpus
// ---------------------------------------------------------------------------------------------

/// Read `tasks.jsonl` from `dir`, preserving the raw JSON of each line.
pub fn read_tasks(dir: &Path) -> Result<Vec<SplitTask>, SplitError> {
    let path = dir.join("tasks.jsonl");
    let body = fs::read_to_string(&path).map_err(|e| {
        SplitError::Io(std::io::Error::new(
            e.kind(),
            format!("no tasks.jsonl at {}: {e}", path.display()),
        ))
    })?;
    let mut tasks = Vec::new();
    for (n, line) in body.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let raw: serde_json::Value = serde_json::from_str(line).map_err(|e| {
            SplitError::Json(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("tasks.jsonl line {}: {e}", n + 1),
            )))
        })?;
        let mut task: SplitTask = serde_json::from_value(raw.clone())?;
        task.raw = raw;
        tasks.push(task);
    }
    Ok(tasks)
}

/// Read a CAS object by digest, if it exists.
pub fn read_cas(dir: &Path, digest_hex: &str) -> Option<Vec<u8>> {
    if digest_hex.is_empty() {
        return None;
    }
    fs::read(dir.join("cas").join(digest_hex)).ok()
}

/// The diff text of a task: the CAS object named by its diff digest, or the empty string when the
/// object is absent (a task whose diff was not written to the CAS still splits, it just cannot be
/// clone-grouped by content).
pub fn task_diff(dir: &Path, task: &SplitTask) -> String {
    read_cas(dir, task.diff_digest())
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .unwrap_or_default()
}

// ---------------------------------------------------------------------------------------------
// Clone groups
// ---------------------------------------------------------------------------------------------

/// Group tasks whose hidden diffs are near-clones (Jaccard >= `threshold`).
///
/// The grouping is a union-find over the pairwise similarity graph, so similarity is transitive
/// within a group: if A~B and B~C, all three are one group even when A and C are below the
/// threshold. That is the conservative choice — a clone group is never split, so over-grouping
/// costs a little data and under-grouping leaks.
///
/// Returns the groups (each a sorted member list plus the edges that joined them), sorted by the
/// smallest member id so the result is deterministic.
pub fn clone_groups(
    tasks: &[SplitTask],
    diffs: &BTreeMap<String, String>,
    threshold: f64,
) -> Vec<CloneGroup> {
    let n = tasks.len();
    let mut parent: Vec<usize> = (0..n).collect();

    fn find(parent: &mut [usize], mut x: usize) -> usize {
        while parent[x] != x {
            parent[x] = parent[parent[x]];
            x = parent[x];
        }
        x
    }

    let mut edges: Vec<CloneEdge> = Vec::new();
    let mut shingle_cache: Vec<BTreeSet<String>> = Vec::with_capacity(n);
    for task in tasks {
        let diff = diffs.get(&task.id).map(String::as_str).unwrap_or("");
        shingle_cache.push(shingles(diff));
    }

    for i in 0..n {
        for j in (i + 1)..n {
            let sim = jaccard(&shingle_cache[i], &shingle_cache[j]);
            if sim >= threshold {
                edges.push(CloneEdge {
                    a: tasks[i].id.clone(),
                    b: tasks[j].id.clone(),
                    jaccard: sim,
                });
                let (ri, rj) = (find(&mut parent, i), find(&mut parent, j));
                if ri != rj {
                    parent[ri] = rj;
                }
            }
        }
    }

    // Collect members by root.
    let mut by_root: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for i in 0..n {
        let r = find(&mut parent, i);
        by_root.entry(r).or_default().push(i);
    }

    let mut groups: Vec<CloneGroup> = Vec::new();
    for members in by_root.values() {
        if members.len() < 2 {
            continue;
        }
        let mut ids: Vec<String> = members.iter().map(|&i| tasks[i].id.clone()).collect();
        ids.sort();
        let member_set: BTreeSet<&String> = ids.iter().collect();
        let group_edges: Vec<CloneEdge> = edges
            .iter()
            .filter(|e| member_set.contains(&e.a) && member_set.contains(&e.b))
            .cloned()
            .collect();
        let payload = serde_json::to_vec(&ids).expect("serializable");
        groups.push(CloneGroup {
            id: digest(&payload),
            members: ids,
            // Filled in once the side is known.
            side: String::new(),
            similarities: group_edges,
        });
    }
    groups.sort_by(|a, b| a.members.cmp(&b.members));
    groups
}

// ---------------------------------------------------------------------------------------------
// The split
// ---------------------------------------------------------------------------------------------

/// A deterministic shuffle of `items` under `seed`: a Fisher–Yates driven by a splitmix64 stream.
/// Same seed and same input order give the same output order.
fn seeded_shuffle<T>(items: &mut [T], seed: u64) {
    let mut state = seed ^ 0x9E37_79B9_7F4A_7C15;
    let mut next = || {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    };
    let n = items.len();
    if n < 2 {
        return;
    }
    for i in (1..n).rev() {
        let j = (next() % (i as u64 + 1)) as usize;
        items.swap(i, j);
    }
}

/// The number of holdout tasks for `total` tasks and a fraction: at least one when the fraction is
/// positive and there is more than one task, at most `total - 1` so the train side is never empty.
pub fn holdout_count(total: usize, fraction: f64) -> usize {
    if total <= 1 {
        return 0;
    }
    let raw = (total as f64 * fraction).round() as usize;
    raw.clamp(1, total - 1)
}

/// Assign each task to a side, honouring clone groups (never split) and, for `--by repo`, whole
/// lineages (never split).
///
/// Returns `(train_ids, holdout_ids, groups_with_sides)`.
fn assign(
    tasks: &[SplitTask],
    groups: &mut [CloneGroup],
    opts: &SplitOptions,
) -> (Vec<String>, Vec<String>) {
    let total = tasks.len();
    let want_holdout = holdout_count(total, opts.holdout);

    // The unit of assignment: a clone group is one unit; a task not in any group is its own unit.
    // For `--by repo`, all units of one lineage are merged into one unit.
    let mut unit_of: BTreeMap<String, usize> = BTreeMap::new();
    let mut units: Vec<Vec<String>> = Vec::new();
    // Build units: start from clone groups, then add singleton tasks.
    let mut grouped: BTreeSet<String> = BTreeSet::new();
    for g in groups.iter() {
        let mut members = g.members.clone();
        members.sort();
        units.push(members.clone());
        for m in &members {
            grouped.insert(m.clone());
            unit_of.insert(m.clone(), units.len() - 1);
        }
    }
    for task in tasks {
        if !grouped.contains(&task.id) {
            units.push(vec![task.id.clone()]);
            unit_of.insert(task.id.clone(), units.len() - 1);
        }
    }

    // For `--by repo`, merge units that share a lineage.
    if opts.rule == SplitRule::Repo {
        let mut lineage_unit: BTreeMap<String, usize> = BTreeMap::new();
        let mut merged: Vec<Vec<String>> = Vec::new();
        let mut remap: Vec<usize> = vec![usize::MAX; units.len()];
        for (i, unit) in units.iter().enumerate() {
            let lineage = tasks
                .iter()
                .find(|t| t.id == unit[0])
                .map(|t| t.lineage().to_string())
                .unwrap_or_default();
            let target = *lineage_unit.entry(lineage).or_insert_with(|| {
                merged.push(Vec::new());
                merged.len() - 1
            });
            merged[target].extend(unit.iter().cloned());
            remap[i] = target;
        }
        for m in merged.iter_mut() {
            m.sort();
            m.dedup();
        }
        units = merged;
        unit_of.clear();
        for (i, unit) in units.iter().enumerate() {
            for id in unit {
                unit_of.insert(id.clone(), i);
            }
        }
    }

    // Order the units deterministically.
    //
    // `--by time` is forward-time: units are ordered by their earliest timestamp (ties broken by
    // the smallest id, then by the seed), and the holdout is taken from the *end* — the latest
    // tasks. The seed only breaks ties among units that share a timestamp, so the split is
    // deterministic and forward in time.
    //
    // `--by repo` is lineage-separated: whole lineages are the units, and the seed shuffles the
    // lineage order before the holdout fraction is taken, so the split is deterministic given the
    // seed but not trivially "the last repository alphabetically".
    let key = |u: usize| -> (String, String) {
        let mut ts = String::new();
        let mut id = String::new();
        for m in &units[u] {
            if let Some(t) = tasks.iter().find(|t| &t.id == m) {
                if ts.is_empty() || t.timestamp < ts {
                    ts = t.timestamp.clone();
                }
                if id.is_empty() || t.id < id {
                    id = t.id.clone();
                }
            }
        }
        (ts, id)
    };
    let mut order: Vec<usize> = (0..units.len()).collect();
    order.sort_by_key(|&u| key(u));
    match opts.rule {
        SplitRule::Time => {
            // Ties among equal timestamps are broken by the seed, so a re-run with the same seed
            // is identical and a different seed may move only the tied units.
            let mut tie_start = 0;
            while tie_start < order.len() {
                let mut tie_end = tie_start + 1;
                while tie_end < order.len() && key(order[tie_end]).0 == key(order[tie_start]).0 {
                    tie_end += 1;
                }
                if tie_end - tie_start > 1 {
                    seeded_shuffle(&mut order[tie_start..tie_end], opts.seed);
                }
                tie_start = tie_end;
            }
        }
        SplitRule::Repo => seeded_shuffle(&mut order, opts.seed),
    }

    // Take units until the holdout has at least `want_holdout` tasks. A unit is atomic: taking it
    // takes all its members, so the holdout may overshoot the fraction rather than split a group.
    //
    // For `--by time` the holdout is the *latest* units, so the order is walked from the end. For
    // `--by repo` the shuffled order is walked from the front.
    let mut holdout: BTreeSet<String> = BTreeSet::new();
    let walk: Vec<usize> = match opts.rule {
        SplitRule::Time => order.iter().rev().copied().collect(),
        SplitRule::Repo => order.clone(),
    };
    for u in walk {
        if holdout.len() >= want_holdout {
            break;
        }
        // Never take every unit: the train side must be non-empty.
        if holdout.len() + units[u].len() >= total {
            continue;
        }
        for id in &units[u] {
            holdout.insert(id.clone());
        }
    }

    let mut train: Vec<String> = Vec::new();
    let mut hold: Vec<String> = Vec::new();
    for task in tasks {
        if holdout.contains(&task.id) {
            hold.push(task.id.clone());
        } else {
            train.push(task.id.clone());
        }
    }
    train.sort();
    hold.sort();

    // Record the side each clone group landed on.
    for g in groups.iter_mut() {
        let in_holdout = g.members.iter().any(|m| holdout.contains(m));
        g.side = if in_holdout { "holdout" } else { "train" }.to_string();
    }

    (train, hold)
}

// ---------------------------------------------------------------------------------------------
// Leakage probes
// ---------------------------------------------------------------------------------------------

/// Probe (a): a task whose hidden diff appears in another split's parent snapshot.
///
/// The parent snapshot is the CAS object named by the task's `parent_snapshot` when that digest is
/// present in the CAS; otherwise the parent snapshot is identified by its digest alone. A task is
/// flagged when its own hidden diff is byte-identical to, or a near-clone of, a diff that is
/// already present in the *other* split's parent snapshot — the agent on the other side would see
/// the answer in the snapshot it starts from.
pub fn probe_hidden_diff_in_other_parent_snapshot(
    dir: &Path,
    tasks: &[SplitTask],
    train: &[String],
    holdout: &[String],
    threshold: f64,
) -> LeakageProbe {
    let train_set: BTreeSet<&String> = train.iter().collect();
    let holdout_set: BTreeSet<&String> = holdout.iter().collect();

    // The parent-snapshot text of each side: the CAS objects named by the members' parent
    // snapshots, concatenated. A parent snapshot that is not a CAS object contributes its digest
    // string, so a task whose diff digest equals another side's parent digest is still caught.
    let mut side_text: BTreeMap<&str, String> = BTreeMap::new();
    let mut side_snapshot_digests: BTreeMap<&str, BTreeSet<String>> = BTreeMap::new();
    for (side, ids) in [("train", &train_set), ("holdout", &holdout_set)] {
        let mut text = String::new();
        let mut digests: BTreeSet<String> = BTreeSet::new();
        for task in tasks.iter().filter(|t| ids.contains(&t.id)) {
            let snap = if !task.parent_snapshot.is_empty() {
                &task.parent_snapshot
            } else {
                &task.mutated_snapshot
            };
            if snap.is_empty() {
                continue;
            }
            digests.insert(snap.clone());
            if let Some(bytes) = read_cas(dir, snap) {
                text.push_str(&String::from_utf8_lossy(&bytes));
                text.push('\n');
            } else {
                text.push_str(snap);
                text.push('\n');
            }
        }
        side_text.insert(side, text);
        side_snapshot_digests.insert(side, digests);
    }

    let mut flagged: Vec<String> = Vec::new();
    for task in tasks {
        let own_side = if train_set.contains(&task.id) {
            "train"
        } else {
            "holdout"
        };
        let other = if own_side == "train" {
            "holdout"
        } else {
            "train"
        };
        let other_text = side_text.get(other).cloned().unwrap_or_default();
        let other_digests = side_snapshot_digests
            .get(other)
            .cloned()
            .unwrap_or_default();
        if other_text.is_empty() && other_digests.is_empty() {
            continue;
        }
        let own_diff = task_diff(dir, task);
        // Digest identity: the task's diff digest *is* a parent-snapshot digest on the other side,
        // so the other side starts from the very object this task's answer is stored in.
        let digest_hit =
            !task.diff_digest().is_empty() && other_digests.contains(task.diff_digest());
        // Content identity: the task's diff text appears verbatim in the other side's parent
        // snapshot text.
        let content_hit = !own_diff.is_empty() && other_text.contains(own_diff.trim());
        // Near-clone: the other side's parent snapshot contains a diff similar to this task's.
        let clone_hit = !own_diff.is_empty()
            && other_text
                .split("\n\n")
                .any(|chunk| diff_similarity(&own_diff, chunk) >= threshold);
        if digest_hit || content_hit || clone_hit {
            flagged.push(task.id.clone());
        }
    }
    flagged.sort();
    flagged.dedup();

    LeakageProbe {
        probe: "hidden_diff_in_other_parent_snapshot".to_string(),
        verdict: if flagged.is_empty() {
            "clean".to_string()
        } else {
            "caught".to_string()
        },
        note: if flagged.is_empty() {
            "no task's hidden diff appears in the other split's parent snapshot".to_string()
        } else {
            format!(
                "{} task(s) have a hidden diff present in the other split's parent snapshot",
                flagged.len()
            )
        },
        flagged,
    }
}

/// Probe (b): a test added after the task's timestamp.
///
/// A task's oracle names tests. If a named test carries its own timestamp (a `test_timestamps`
/// map on the task, or a `tests` array of `{name, timestamp}`), and that timestamp is later than
/// the task's timestamp, the oracle depends on a test that did not exist when the task was made —
/// the agent could not have written it, and the split would be scoring a future test.
pub fn probe_test_added_after_timestamp(tasks: &[SplitTask]) -> LeakageProbe {
    let mut flagged: Vec<String> = Vec::new();
    for task in tasks {
        let Some(map) = test_timestamps(&task.raw) else {
            continue;
        };
        for name in task.oracle_tests() {
            if let Some(ts) = map.get(name) {
                if !task.timestamp.is_empty() && ts.as_str() > task.timestamp.as_str() {
                    flagged.push(task.id.clone());
                    break;
                }
            }
        }
    }
    flagged.sort();
    flagged.dedup();
    LeakageProbe {
        probe: "test_added_after_timestamp".to_string(),
        verdict: if flagged.is_empty() {
            "clean".to_string()
        } else {
            "caught".to_string()
        },
        note: if flagged.is_empty() {
            "every oracle test predates its task".to_string()
        } else {
            format!(
                "{} task(s) name a test added after the task's timestamp",
                flagged.len()
            )
        },
        flagged,
    }
}

/// The per-test timestamps a task may carry: a `test_timestamps` object, or a `tests` array of
/// `{name, timestamp}`.
fn test_timestamps(raw: &serde_json::Value) -> Option<BTreeMap<String, String>> {
    let mut out: BTreeMap<String, String> = BTreeMap::new();
    if let Some(obj) = raw.get("test_timestamps").and_then(|v| v.as_object()) {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                out.insert(k.clone(), s.to_string());
            }
        }
    }
    if let Some(arr) = raw.get("tests").and_then(|v| v.as_array()) {
        for entry in arr {
            let name = entry.get("name").and_then(|v| v.as_str());
            let ts = entry.get("timestamp").and_then(|v| v.as_str());
            if let (Some(name), Some(ts)) = (name, ts) {
                out.insert(name.to_string(), ts.to_string());
            }
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

/// Probe (c): a post-action log.
///
/// A task that carries a `post_action_log` (or a `log` array) whose entries are timestamped after
/// the task's timestamp leaks what happened after the action: the agent would read the outcome
/// before acting. Any such entry flags the task.
pub fn probe_post_action_log(tasks: &[SplitTask]) -> LeakageProbe {
    let mut flagged: Vec<String> = Vec::new();
    for task in tasks {
        let mut hit = false;
        for field in [&task.post_action_log, &task.log] {
            let Some(value) = field else { continue };
            if log_is_post_action(value, &task.timestamp) {
                hit = true;
                break;
            }
        }
        if hit {
            flagged.push(task.id.clone());
        }
    }
    flagged.sort();
    flagged.dedup();
    LeakageProbe {
        probe: "post_action_log".to_string(),
        verdict: if flagged.is_empty() {
            "clean".to_string()
        } else {
            "caught".to_string()
        },
        note: if flagged.is_empty() {
            "no task carries a post-action log".to_string()
        } else {
            format!(
                "{} task(s) carry a log entry timestamped after the task",
                flagged.len()
            )
        },
        flagged,
    }
}

/// Whether a log value contains an entry timestamped after `task_timestamp`.
///
/// A bare `true`/non-empty object is treated as a post-action log too: the field's presence is the
/// leak, and the timestamp check only sharpens it.
fn log_is_post_action(value: &serde_json::Value, task_timestamp: &str) -> bool {
    match value {
        serde_json::Value::Null => false,
        serde_json::Value::Bool(b) => *b,
        serde_json::Value::Array(entries) => entries
            .iter()
            .any(|e| entry_is_post_action(e, task_timestamp)),
        serde_json::Value::Object(_) => entry_is_post_action(value, task_timestamp),
        _ => false,
    }
}

/// Whether one log entry is a post-action entry: it has a timestamp after the task's, or it has no
/// timestamp at all (an undated post-action log is still a post-action log).
fn entry_is_post_action(entry: &serde_json::Value, task_timestamp: &str) -> bool {
    let ts = entry
        .get("timestamp")
        .or_else(|| entry.get("at"))
        .or_else(|| entry.get("time"))
        .and_then(|v| v.as_str());
    match ts {
        Some(ts) => {
            if task_timestamp.is_empty() {
                true
            } else {
                ts > task_timestamp
            }
        }
        None => true,
    }
}

// ---------------------------------------------------------------------------------------------
// The split run
// ---------------------------------------------------------------------------------------------

/// Split the corpus in `dir` and write `<dir>/split.json`.
///
/// Reads `tasks.jsonl` and the CAS under `dir`, groups near-clones, assigns sides under the rule,
/// runs the three leakage probes, and writes the manifest. A probe that is **caught** is an error:
/// the split refuses rather than emitting a leaking manifest. The manifest is still written with
/// the probe verdicts so the operator can see what tripped.
pub fn split(dir: &Path, opts: &SplitOptions) -> Result<SplitManifest, SplitError> {
    if !(opts.holdout > 0.0 && opts.holdout < 1.0) {
        return Err(SplitError::BadFraction(opts.holdout));
    }
    let tasks = read_tasks(dir)?;
    if tasks.is_empty() {
        return Err(SplitError::NoTasks(dir.display().to_string()));
    }

    // The diff text of every task, read once.
    let mut diffs: BTreeMap<String, String> = BTreeMap::new();
    for task in &tasks {
        diffs.insert(task.id.clone(), task_diff(dir, task));
    }

    let mut groups = clone_groups(&tasks, &diffs, opts.clone_jaccard);
    let (train, holdout) = assign(&tasks, &mut groups, opts);

    // The leakage probes. Each is run and recorded; a caught probe is an error.
    let probes = vec![
        probe_hidden_diff_in_other_parent_snapshot(
            dir,
            &tasks,
            &train,
            &holdout,
            opts.clone_jaccard,
        ),
        probe_test_added_after_timestamp(&tasks),
        probe_post_action_log(&tasks),
    ];

    let mut train_lineages: BTreeSet<String> = BTreeSet::new();
    let mut holdout_lineages: BTreeSet<String> = BTreeSet::new();
    for task in &tasks {
        if train.contains(&task.id) {
            train_lineages.insert(task.lineage().to_string());
        } else {
            holdout_lineages.insert(task.lineage().to_string());
        }
    }

    // Every holdout access is logged: the split itself reads the holdout to write the manifest,
    // and each probe that touched a holdout task is an access.
    let at = now_iso8601();
    let mut access: Vec<HoldoutAccess> = Vec::new();
    access.push(HoldoutAccess {
        accessed: "*".to_string(),
        at: at.clone(),
        by: opts.command.clone(),
        reason: format!(
            "split by {} with seed {} and holdout fraction {}",
            opts.rule.as_str(),
            opts.seed,
            opts.holdout
        ),
    });
    for probe in &probes {
        for id in &probe.flagged {
            if holdout.contains(id) {
                access.push(HoldoutAccess {
                    accessed: id.clone(),
                    at: at.clone(),
                    by: format!("leakage probe {}", probe.probe),
                    reason: probe.note.clone(),
                });
            }
        }
    }

    let manifest = SplitManifest {
        rule: opts.rule.as_str().to_string(),
        seed: opts.seed,
        holdout_fraction: opts.holdout,
        clone_jaccard: opts.clone_jaccard,
        train,
        holdout,
        clone_groups: groups,
        holdout_access: access,
        leakage_probes: probes,
        train_lineages: train_lineages.into_iter().collect(),
        holdout_lineages: holdout_lineages.into_iter().collect(),
    };

    // Write the manifest before refusing, so the operator can read what tripped.
    let path = dir.join("split.json");
    let body = serde_json::to_vec_pretty(&manifest)?;
    fs::write(&path, &body)?;

    if let Some(caught) = manifest
        .leakage_probes
        .iter()
        .find(|p| p.verdict == "caught")
    {
        return Err(SplitError::Leakage {
            probe: caught.probe.clone(),
            detail: format!("{} — see {}", caught.note, path.display()),
        });
    }

    Ok(manifest)
}

/// The current time, ISO-8601 UTC, from the system clock. Used only for the holdout-access log.
fn now_iso8601() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    // A minimal civil-time conversion (days since epoch → y/m/d), no external crate.
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let (y, mo, d) = civil_from_days(days as i64);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

/// Days since 1970-01-01 → (year, month, day). Howard Hinnant's `civil_from_days`.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}
