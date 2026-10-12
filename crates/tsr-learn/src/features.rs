//! features@v1 — the versioned feature schema (PRD §B8, lane 06 item 3).
//!
//! A feature reads **only** the task's events strictly before the commitment and the task index.
//! It never reads a label, never `recorded_at` or `observed_at` (B6: wall-clock is provenance,
//! not a feature), and never `Committed.decision.utility` — main's hook writes a placeholder
//! `p_success` of 0.5 there, and reading it would fabricate a feature.
//!
//! The schema is versioned with the bundle ([`FEATURE_SCHEMA_VERSION`]). The §B8 features main's
//! log cannot supply are listed in [`FeatureSpec::absent`] with the reason, for a later schema
//! version; none is imputed.

use serde::{Deserialize, Serialize};
use tsr_domain::{ActionKind, Event, EventKind};

use crate::dataset::TaskIndexRow;

/// The feature-schema version carried in the bundle (§B8: "the feature schema is versioned with
/// the bundle").
pub const FEATURE_SCHEMA_VERSION: &str = "features@v1";

/// A §B8 feature the log cannot supply, with the reason it is absent. Listed, never imputed.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AbsentFeature {
    /// The §B8 name.
    pub name: String,
    /// Why main's log cannot supply it.
    pub reason: String,
}

/// The features@v1 spec: the ordered feature names, the numeric window widths used by support
/// (§B8 "within feature range"), and the absent features.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeatureSpec {
    pub version: String,
    /// Feature names, in the order of [`FeatureVector::values`].
    pub names: Vec<String>,
    /// Per-feature window width for support's "within feature range". Binary and one-hot features
    /// carry width `0.0` (they must match exactly); numeric features carry a positive width.
    pub numeric_widths: Vec<f64>,
    /// §B8 features the log cannot supply, with the reason.
    pub absent: Vec<AbsentFeature>,
}

/// A feature vector: the values, in the order of [`FeatureSpec::names`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeatureVector {
    pub values: Vec<f64>,
}

impl FeatureVector {
    pub fn get(&self, spec: &FeatureSpec, name: &str) -> Option<f64> {
        spec.names
            .iter()
            .position(|n| n == name)
            .and_then(|i| self.values.get(i).copied())
    }
}

/// The six action kinds, in a fixed order, for the action-kind one-hot.
pub const ACTION_KINDS: [ActionKind; 6] = [
    ActionKind::Inspect,
    ActionKind::RunApprovedTest,
    ActionKind::Generate,
    ActionKind::ProposePatch,
    ActionKind::RequestApproval,
    ActionKind::Stop,
];

/// The action-kind feature name for a kind.
pub fn action_feature_name(kind: ActionKind) -> String {
    let tag = match kind {
        ActionKind::Inspect => "inspect",
        ActionKind::RunApprovedTest => "run_approved_test",
        ActionKind::Generate => "generate",
        ActionKind::ProposePatch => "propose_patch",
        ActionKind::RequestApproval => "request_approval",
        ActionKind::Stop => "stop",
    };
    format!("action:{tag}")
}

/// The family one-hot feature name.
pub fn family_feature_name(family: &str) -> String {
    format!("family:{family}")
}

/// Recover the families a spec was built for, from its one-hot names, in name order.
pub fn families_from_spec(spec: &FeatureSpec) -> Vec<String> {
    spec.names
        .iter()
        .filter_map(|n| n.strip_prefix("family:"))
        .filter(|f| !f.ends_with("_x_generator_logit"))
        .map(|f| f.to_string())
        .collect()
}

/// The family × generator-logit interaction feature name.
pub fn family_interaction_name(family: &str) -> String {
    format!("family:{family}_x_generator_logit")
}

/// The clip applied to the generator's raw probability before the logit, so a raw `0.0` or `1.0`
/// does not produce an infinite feature. Named in the spec and in the report.
pub const GENERATOR_LOGIT_CLIP: f64 = 1e-3;

/// The clipped logit of a probability in `[0, 1]`.
pub fn clipped_logit(p: f64) -> f64 {
    let p = p.clamp(GENERATOR_LOGIT_CLIP, 1.0 - GENERATOR_LOGIT_CLIP);
    (p / (1.0 - p)).ln()
}

/// Build the features@v1 spec for a fit slice's families.
///
/// `families` is the set of families in the fit slice, in name order; the one-hot and the
/// family × generator-logit interactions are over exactly those.
pub fn spec_for_families(families: &[String]) -> FeatureSpec {
    let mut names: Vec<String> = Vec::new();
    let mut widths: Vec<f64> = Vec::new();

    // Family one-hot (binary: width 0.0).
    for f in families {
        names.push(family_feature_name(f));
        widths.push(0.0);
    }
    // Action-kind one-hot (binary: width 0.0).
    for k in ACTION_KINDS {
        names.push(action_feature_name(k));
        widths.push(0.0);
    }
    // Numeric features, each with a declared window width for support.
    let numeric: [(&str, f64); 8] = [
        ("inspections_so_far", 2.0),
        ("distinct_inspect_targets", 2.0),
        ("patches_so_far", 1.0),
        ("distinct_patch_targets", 1.0),
        ("generator_logit", 1.0),
        ("test_runs_so_far", 2.0),
        ("failed_test_runs_so_far", 2.0),
        ("budget_fraction_used", 0.25),
    ];
    for (n, w) in numeric {
        names.push(n.to_string());
        widths.push(w);
    }
    // Family × generator-logit interactions (numeric: width 1.0).
    for f in families {
        names.push(family_interaction_name(f));
        widths.push(1.0);
    }
    // Budget-ceiling-missing indicator (binary: width 0.0).
    names.push("budget_ceiling_missing".to_string());
    widths.push(0.0);
    // Generator-missing indicator (binary: width 0.0). A1/A3/A11: a missing generator forecast is
    // not imputed as p = 0.5 — the logit is 0 **only** with this indicator set, so the model can
    // tell "the generator said 0.5" from "the generator said nothing".
    names.push("generator_missing".to_string());
    widths.push(0.0);

    FeatureSpec {
        version: FEATURE_SCHEMA_VERSION.to_string(),
        names,
        numeric_widths: widths,
        absent: absent_features(),
    }
}

/// The counts a feature reads from the events strictly before the commitment.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PreActionCounts {
    pub inspections: u64,
    pub distinct_inspect_targets: u64,
    pub patches: u64,
    pub distinct_patch_targets: u64,
    pub test_runs: u64,
    pub failed_test_runs: u64,
    pub tool_ms_used: u64,
}

/// Count the pre-action facts from the events strictly before the commitment.
///
/// Only `Proposed` (the candidate's kind and args digest), `Executed` (exit code and duration) and
/// `Dispatched` are read. Nothing reads `recorded_at`, `observed_at`, a label, or
/// `Committed.decision.utility`.
pub fn pre_action_counts(events_before: &[(i64, Event)]) -> PreActionCounts {
    use std::collections::HashSet;

    let mut c = PreActionCounts::default();
    let mut inspect_targets: HashSet<tsr_domain::Hash> = HashSet::new();
    let mut patch_targets: HashSet<tsr_domain::Hash> = HashSet::new();
    // The kind and args digest of each proposed candidate, so an Executed can be attributed.
    let mut proposed: std::collections::HashMap<
        tsr_domain::CandidateId,
        (ActionKind, tsr_domain::Hash),
    > = std::collections::HashMap::new();

    for (_, e) in events_before {
        match &e.kind {
            EventKind::Proposed { candidate } => {
                proposed.insert(candidate.id, (candidate.kind, candidate.args));
                if candidate.kind == ActionKind::Inspect {
                    c.inspections += 1;
                    inspect_targets.insert(candidate.args);
                }
            }
            // A8: "patches so far" and "distinct patch targets" count patches that were *committed*
            // before this commitment, not merely proposed. A Proposed patch that was never committed
            // is not a patch the task has made.
            EventKind::Committed { candidate, .. } => {
                if let Some((kind, args)) = proposed.get(candidate) {
                    if *kind == ActionKind::ProposePatch {
                        c.patches += 1;
                        patch_targets.insert(*args);
                    }
                }
            }
            EventKind::Executed {
                candidate,
                exit_code,
                duration_ms,
                ..
            } => {
                c.tool_ms_used = c.tool_ms_used.saturating_add(*duration_ms);
                if let Some((kind, _)) = proposed.get(candidate) {
                    if *kind == ActionKind::RunApprovedTest {
                        c.test_runs += 1;
                        if exit_code.is_some_and(|code| code != 0) {
                            c.failed_test_runs += 1;
                        }
                    }
                }
            }
            _ => {}
        }
    }
    c.distinct_inspect_targets = inspect_targets.len() as u64;
    c.distinct_patch_targets = patch_targets.len() as u64;
    c
}

/// Extract the features@v1 vector for a commitment.
///
/// `events_before` holds the task's events **strictly before** the commitment, in log order.
/// `index_row` is the task's index row. `action` is the committed action's kind. `families` is the
/// fit slice's families, in name order. `generator_raw` is the generator's raw probability on the
/// common event for this commitment, if the generator forecast it; `None` yields a generator-logit
/// of `0.0`, zero interactions, and `generator_missing = 1.0` (the logit is 0 only with the
/// indicator set — a missing forecast is never imputed as p = 0.5).
pub fn extract(
    spec: &FeatureSpec,
    events_before: &[(i64, Event)],
    index_row: &TaskIndexRow,
    action: ActionKind,
    families: &[String],
    generator_raw: Option<f64>,
) -> FeatureVector {
    let c = pre_action_counts(events_before);
    let logit = generator_raw.map(clipped_logit).unwrap_or(0.0);
    let action_name = action_feature_name(action);

    let mut values = Vec::with_capacity(spec.names.len());
    for name in &spec.names {
        let v = if let Some(f) = name.strip_prefix("family:") {
            if let Some(fam) = f.strip_suffix("_x_generator_logit") {
                if fam == index_row.family {
                    logit
                } else {
                    0.0
                }
            } else if f == index_row.family {
                1.0
            } else {
                0.0
            }
        } else if name.starts_with("action:") {
            if *name == action_name {
                1.0
            } else {
                0.0
            }
        } else {
            match name.as_str() {
                "inspections_so_far" => c.inspections as f64,
                "distinct_inspect_targets" => c.distinct_inspect_targets as f64,
                "patches_so_far" => c.patches as f64,
                "distinct_patch_targets" => c.distinct_patch_targets as f64,
                "generator_logit" => logit,
                "test_runs_so_far" => c.test_runs as f64,
                "failed_test_runs_so_far" => c.failed_test_runs as f64,
                "budget_fraction_used" => match index_row.tool_ms_ceiling {
                    Some(ceiling) if ceiling > 0 => c.tool_ms_used as f64 / ceiling as f64,
                    _ => 0.0,
                },
                "budget_ceiling_missing" => {
                    if index_row.tool_ms_ceiling.is_none() {
                        1.0
                    } else {
                        0.0
                    }
                }
                // A1/A3/A11: the logit is 0 only when the generator forecast is missing, and this
                // indicator says so; a present forecast of 0.5 yields logit 0 with the indicator 0.
                "generator_missing" => {
                    if generator_raw.is_none() {
                        1.0
                    } else {
                        0.0
                    }
                }
                _ => 0.0,
            }
        };
        values.push(v);
    }
    let _ = families;
    FeatureVector { values }
}

/// The §B8 features main's log cannot supply, with the reason. None is imputed.
pub fn absent_features() -> Vec<AbsentFeature> {
    vec![
        AbsentFeature {
            name: "patch_bytes".to_string(),
            reason: "the log records a candidate's args digest, not the patch body; the CAS body \
                     is not read here (B6: the digest is the record)"
                .to_string(),
        },
        AbsentFeature {
            name: "failing_test_count".to_string(),
            reason: "the log records an Executed exit code, not the count of failing tests; \
                     failed_test_runs_so_far is the log's stand-in and is named for what it measures"
                .to_string(),
        },
        AbsentFeature {
            name: "failing_assertion_file_read".to_string(),
            reason: "the log holds no assertion-to-file mapping; distinct_inspect_targets stands \
                     in for files read and is named for what it measures"
                .to_string(),
        },
        AbsentFeature {
            name: "council_agreement".to_string(),
            reason: "no council ran in main's log; a council's agreement is not recorded"
                .to_string(),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use tsr_domain::{Candidate, Hash, Source, TaskId};

    fn index_row(family: &str, ceiling: Option<u64>) -> TaskIndexRow {
        TaskIndexRow {
            task: TaskId(Hash::of(&[b"t"])),
            family: family.to_string(),
            lineage: "repo-a".to_string(),
            timestamp: "2024-01-01T00:00:00Z".to_string(),
            clone_group: None,
            tool_ms_ceiling: ceiling,
        }
    }

    fn proposed(kind: ActionKind, tag: &[u8]) -> EventKind {
        EventKind::Proposed {
            candidate: Candidate::new(
                kind,
                Source::Generator,
                Hash::of(&[tag]),
                Vec::new(),
                Vec::new(),
            )
            .unwrap(),
        }
    }

    fn committed(candidate: tsr_domain::CandidateId) -> EventKind {
        EventKind::Committed {
            candidate,
            decision: tsr_domain::Decision::Selected {
                value: candidate,
                utility: tsr_domain::Utility {
                    p_success: 0.5,
                    value: 0.0,
                    cost: 0.0,
                    ev: 0.0,
                },
                propensity: 1.0,
                evidence: Vec::new(),
            },
        }
    }

    fn ev(kind: EventKind, tag: &[u8]) -> (i64, Event) {
        let t = TaskId(Hash::of(&[b"t"]));
        let mut e = Event::new(t, Vec::new(), Source::Rules, kind, "2024-01-01T00:00:00Z");
        e.recorded_at = format!("2024-01-01T00:00:00Z#{}", hex(tag));
        (1, e)
    }

    fn hex(b: &[u8]) -> String {
        b.iter().map(|x| format!("{x:02x}")).collect()
    }

    #[test]
    fn spec_has_one_hot_and_numeric_features() {
        let spec = spec_for_families(&["a".to_string(), "b".to_string()]);
        assert_eq!(spec.version, "features@v1");
        assert!(spec.names.contains(&"family:a".to_string()));
        assert!(spec
            .names
            .contains(&"family:b_x_generator_logit".to_string()));
        assert!(spec.names.contains(&"inspections_so_far".to_string()));
        assert!(spec.names.contains(&"budget_ceiling_missing".to_string()));
        assert!(spec.names.contains(&"generator_missing".to_string()));
        assert_eq!(spec.names.len(), spec.numeric_widths.len());
        assert_eq!(spec.absent.len(), 4);
    }

    #[test]
    fn counts_read_only_pre_action_events() {
        // A8: a patch counts only once it is *committed*, so the Proposed patch below is committed
        // by the Committed event that follows it.
        let patch = Candidate::new(
            ActionKind::ProposePatch,
            Source::Generator,
            Hash::of(&[b"p1"]),
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
        let patch_id = patch.id;
        let events = vec![
            ev(proposed(ActionKind::Inspect, b"i1"), b"1"),
            ev(proposed(ActionKind::Inspect, b"i1"), b"2"), // same target
            ev(proposed(ActionKind::Inspect, b"i2"), b"3"),
            ev(EventKind::Proposed { candidate: patch }, b"4"),
            ev(committed(patch_id), b"5"),
        ];
        let c = pre_action_counts(&events);
        assert_eq!(c.inspections, 3);
        assert_eq!(c.distinct_inspect_targets, 2);
        assert_eq!(c.patches, 1);
        assert_eq!(c.distinct_patch_targets, 1);
    }

    #[test]
    fn a8_patches_count_only_committed_patches_not_proposed_ones() {
        // A8: "patches so far" and "distinct patch targets" count patches *committed* before the
        // commitment, not Proposed events. Two patches are proposed; only one is committed.
        let committed_patch = Candidate::new(
            ActionKind::ProposePatch,
            Source::Generator,
            Hash::of(&[b"p-committed"]),
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
        let committed_id = committed_patch.id;
        let uncommitted_patch = Candidate::new(
            ActionKind::ProposePatch,
            Source::Generator,
            Hash::of(&[b"p-uncommitted"]),
            Vec::new(),
            Vec::new(),
        )
        .unwrap();

        let events = vec![
            ev(
                EventKind::Proposed {
                    candidate: committed_patch.clone(),
                },
                b"1",
            ),
            ev(
                EventKind::Proposed {
                    candidate: uncommitted_patch.clone(),
                },
                b"2",
            ),
            ev(committed(committed_id), b"3"),
        ];
        let c = pre_action_counts(&events);
        assert_eq!(
            c.patches, 1,
            "A8: only the committed patch counts, not the merely proposed one"
        );
        assert_eq!(
            c.distinct_patch_targets, 1,
            "A8: distinct patch targets count committed patches only"
        );

        // With no Committed event at all, no patch has been made.
        let only_proposed = vec![
            ev(
                EventKind::Proposed {
                    candidate: committed_patch,
                },
                b"1",
            ),
            ev(
                EventKind::Proposed {
                    candidate: uncommitted_patch,
                },
                b"2",
            ),
        ];
        let c2 = pre_action_counts(&only_proposed);
        assert_eq!(c2.patches, 0, "A8: a Proposed patch is not a patch made");
        assert_eq!(c2.distinct_patch_targets, 0);
    }

    #[test]
    fn failed_test_runs_come_from_executed_exit_codes() {
        let cand = Candidate::new(
            ActionKind::RunApprovedTest,
            Source::Generator,
            Hash::of(&[b"t1"]),
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
        let cid = cand.id;
        let events = vec![
            ev(EventKind::Proposed { candidate: cand }, b"1"),
            ev(
                EventKind::Executed {
                    candidate: cid,
                    exit_code: Some(1),
                    log_digest: None,
                    duration_ms: 100,
                },
                b"2",
            ),
            ev(
                EventKind::Executed {
                    candidate: cid,
                    exit_code: Some(0),
                    log_digest: None,
                    duration_ms: 50,
                },
                b"3",
            ),
        ];
        let c = pre_action_counts(&events);
        assert_eq!(c.test_runs, 2);
        assert_eq!(c.failed_test_runs, 1);
        assert_eq!(c.tool_ms_used, 150);
    }

    #[test]
    fn extract_sets_family_one_hot_and_action_one_hot() {
        let spec = spec_for_families(&["a".to_string(), "b".to_string()]);
        let row = index_row("b", Some(1000));
        let events = vec![ev(proposed(ActionKind::Inspect, b"i1"), b"1")];
        let fv = extract(
            &spec,
            &events,
            &row,
            ActionKind::ProposePatch,
            &[],
            Some(0.9),
        );
        assert_eq!(fv.get(&spec, "family:a"), Some(0.0));
        assert_eq!(fv.get(&spec, "family:b"), Some(1.0));
        assert_eq!(fv.get(&spec, "action:propose_patch"), Some(1.0));
        assert_eq!(fv.get(&spec, "action:inspect"), Some(0.0));
        assert_eq!(fv.get(&spec, "inspections_so_far"), Some(1.0));
        // The interaction is the logit for the row's family, zero elsewhere.
        let logit = clipped_logit(0.9);
        assert!((fv.get(&spec, "family:b_x_generator_logit").unwrap() - logit).abs() < 1e-12);
        assert_eq!(fv.get(&spec, "family:a_x_generator_logit"), Some(0.0));
    }

    #[test]
    fn budget_fraction_and_missing_indicator() {
        let spec = spec_for_families(&["a".to_string()]);
        let row = index_row("a", Some(200));
        let cand = Candidate::new(
            ActionKind::RunApprovedTest,
            Source::Generator,
            Hash::of(&[b"t1"]),
            Vec::new(),
            Vec::new(),
        )
        .unwrap();
        let cid = cand.id;
        let events = vec![
            ev(EventKind::Proposed { candidate: cand }, b"1"),
            ev(
                EventKind::Executed {
                    candidate: cid,
                    exit_code: Some(0),
                    log_digest: None,
                    duration_ms: 50,
                },
                b"2",
            ),
        ];
        let fv = extract(&spec, &events, &row, ActionKind::RunApprovedTest, &[], None);
        assert_eq!(fv.get(&spec, "budget_fraction_used"), Some(0.25));
        assert_eq!(fv.get(&spec, "budget_ceiling_missing"), Some(0.0));

        let row2 = index_row("a", None);
        let fv2 = extract(
            &spec,
            &events,
            &row2,
            ActionKind::RunApprovedTest,
            &[],
            None,
        );
        assert_eq!(fv2.get(&spec, "budget_fraction_used"), Some(0.0));
        assert_eq!(fv2.get(&spec, "budget_ceiling_missing"), Some(1.0));
    }

    /// A1/A3/A11: a missing generator forecast is not imputed as p = 0.5. The logit is 0 **only**
    /// with `generator_missing = 1`; a present forecast of 0.5 yields logit 0 with the indicator 0.
    #[test]
    fn a_missing_generator_forecast_sets_the_indicator_not_a_fake_logit() {
        let spec = spec_for_families(&["a".to_string()]);
        let row = index_row("a", Some(200));
        let events = vec![ev(proposed(ActionKind::Inspect, b"i1"), b"1")];

        let missing = extract(&spec, &events, &row, ActionKind::Inspect, &[], None);
        assert_eq!(missing.get(&spec, "generator_missing"), Some(1.0));
        assert_eq!(missing.get(&spec, "generator_logit"), Some(0.0));

        let present = extract(&spec, &events, &row, ActionKind::Inspect, &[], Some(0.5));
        assert_eq!(present.get(&spec, "generator_missing"), Some(0.0));
        assert_eq!(present.get(&spec, "generator_logit"), Some(0.0));
        // The two vectors differ: the indicator distinguishes "said 0.5" from "said nothing".
        assert_ne!(missing.values, present.values);
    }

    #[test]
    fn clipped_logit_is_finite_at_the_ends() {
        assert!(clipped_logit(0.0).is_finite());
        assert!(clipped_logit(1.0).is_finite());
        assert!(clipped_logit(0.5).abs() < 1e-12);
    }
}
