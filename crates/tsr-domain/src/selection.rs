//! Versioned bounded selection over already-admissible candidates; no authority grant.
use crate::Hash;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
pub const VERSION: &str = "bounded-near-tie@v1";
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub margin: f64,
    pub epsilon: f64,
    pub seed: u64,
}
impl Policy {
    pub fn validate(&self) -> Result<(), &'static str> {
        if !self.margin.is_finite()
            || self.margin < 0.0
            || !self.epsilon.is_finite()
            || !(0.0..=1.0).contains(&self.epsilon)
        {
            return Err("invalid selection policy");
        }
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Trace {
    pub version: String,
    pub policy: Policy,
    pub state: Hash,
    pub scores: Vec<(String, f64)>,
    pub probabilities: Vec<(String, f64)>,
    pub selected: Option<String>,
    pub propensity: Option<f64>,
    pub reason: String,
    pub draw: Option<u64>,
}
/// Epsilon mixture of stable best and uniform near-tie set. Integer bins make
/// logged probabilities exact for the 53-bit draw, including rounding effects.
pub fn select(
    policy: &Policy,
    state: Hash,
    candidates: &[(String, f64)],
) -> Result<Trace, &'static str> {
    select_draw(policy, state, candidates, None)
}
fn select_draw(
    policy: &Policy,
    state: Hash,
    candidates: &[(String, f64)],
    saved_draw: Option<u64>,
) -> Result<Trace, &'static str> {
    if saved_draw.is_some_and(|d| d >= (1 << 53)) {
        return Err("invalid saved draw");
    }
    policy.validate()?;
    if candidates.len() > 256
        || candidates
            .iter()
            .any(|(id, s)| id.is_empty() || !s.is_finite())
    {
        return Err("invalid bounded candidates");
    }
    let mut ids = BTreeSet::new();
    if candidates.iter().any(|(id, _)| !ids.insert(id)) {
        return Err("duplicate candidate identity");
    }
    let mut scores = candidates.to_vec();
    scores.sort_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    let mut trace = Trace {
        version: VERSION.into(),
        policy: policy.clone(),
        state,
        scores: scores.clone(),
        probabilities: vec![],
        selected: None,
        propensity: None,
        reason: "no_admissible_action".into(),
        draw: None,
    };
    let Some(best) = scores.first() else {
        return Ok(trace);
    };
    let tied = scores
        .iter()
        .take_while(|(_, score)| best.1 - *score <= policy.margin)
        .count();
    if tied > 1 && policy.epsilon == 0.0 {
        trace.reason = "near_tie_abstention".into();
        return Ok(trace);
    }
    if tied == 1 {
        trace.selected = Some(best.0.clone());
        trace.propensity = Some(1.0);
        trace.probabilities = scores
            .iter()
            .map(|(id, _)| (id.clone(), f64::from(id == &best.0)))
            .collect();
        trace.reason = "separated_best".into();
        return Ok(trace);
    }
    const SPACE: u64 = 1 << 53;
    let encoded = serde_json::to_vec(&scores).map_err(|_| "candidate encoding")?;
    let hash = Hash::of(&[
        VERSION.as_bytes(),
        &state.0,
        &policy.seed.to_le_bytes(),
        &encoded,
    ]);
    let draw =
        saved_draw.unwrap_or_else(|| u64::from_le_bytes(hash.0[..8].try_into().unwrap()) >> 11);
    trace.draw = Some(draw);
    trace.reason = "opt_in_near_tie_mixture".into();
    let mut lower = 0;
    for (i, (id, _)) in scores.iter().enumerate() {
        let upper = if i + 1 >= tied {
            SPACE
        } else {
            (((1.0 - policy.epsilon) + policy.epsilon * (i + 1) as f64 / tied as f64)
                * SPACE as f64)
                .round() as u64
        };
        let probability = (upper - lower) as f64 / SPACE as f64;
        trace.probabilities.push((id.clone(), probability));
        if (lower..upper).contains(&draw) {
            trace.selected = Some(id.clone());
            trace.propensity = Some(probability);
        }
        lower = upper;
    }
    Ok(trace)
}

/// Pure model-state intervention. Fixed factual eligibility is the primary control;
/// changed eligibility is a separately labelled hypothetical, never dispatch authority.
pub fn counterfactual(
    factual: &Trace,
    scores: &[(String, f64)],
    eligible: &[String],
) -> Result<serde_json::Value, &'static str> {
    if factual.version != VERSION {
        return Ok(
            serde_json::json!({"status":"unsupported_historical_policy","version":factual.version}),
        );
    }
    if select(&factual.policy, factual.state, &factual.scores)? != *factual {
        return Err("factual trace does not replay");
    }
    // Validate all scores, even scores removed by hypothetical eligibility.
    select_draw(&factual.policy, factual.state, scores, factual.draw)?;
    let all: std::collections::BTreeMap<_, _> = scores.iter().cloned().collect();
    let factual_ids: BTreeSet<_> = factual.scores.iter().map(|(id, _)| id.clone()).collect();
    let eligible_ids: BTreeSet<_> = eligible.iter().cloned().collect();
    if eligible_ids.len() != eligible.len() || eligible_ids.iter().any(|id| !all.contains_key(id)) {
        return Err("invalid counterfactual eligibility");
    }
    let missing: Vec<_> = factual_ids
        .iter()
        .filter(|id| !all.contains_key(*id))
        .cloned()
        .collect();
    let replay = |set: &BTreeSet<String>| -> Result<Trace, &'static str> {
        let candidates = set
            .iter()
            .map(|id| (id.clone(), all[id]))
            .collect::<Vec<_>>();
        let mut t = select_draw(&factual.policy, factual.state, &candidates, factual.draw)?;
        if factual.draw.is_none() && t.draw.is_some() {
            // v1 deterministic/abstaining receipts did not save an unused random draw.
            t.draw = None;
            t.selected = None;
            t.propensity = None;
            t.reason = "missing_saved_draw_for_sampling".into();
        }
        Ok(t)
    };
    let fixed = if missing.is_empty() {
        Some(replay(&factual_ids)?)
    } else {
        None
    };
    let changed = replay(&eligible_ids)?;
    let factual_best = factual.scores.first().map(|s| s.0.clone());
    let counterfactual_best = fixed
        .as_ref()
        .and_then(|t| t.scores.first().map(|s| s.0.clone()));
    let choice_changed = fixed
        .as_ref()
        .filter(|t| t.reason != "missing_saved_draw_for_sampling")
        .map(|t| t.selected != factual.selected);
    Ok(
        serde_json::json!({"status":"controlled_model_state_intervention","factual":factual,"fixed_support":fixed,
        "changed_support_scenario":changed,"support_changed":factual_ids!=eligible_ids,
        "removed":factual_ids.difference(&eligible_ids).collect::<Vec<_>>(),"added":eligible_ids.difference(&factual_ids).collect::<Vec<_>>(),
        "missing_factual_scores":missing,"factual_best":factual_best,"counterfactual_best":counterfactual_best,
        "ranking_changed":if fixed.is_some(){Some(factual_best!=counterfactual_best)}else{None},"same_draw_choice_changed":choice_changed,
        "factual_sampling_departure":factual.draw.is_some() && factual.selected!=factual_best,
        "meaning":"Same recorded policy/state/draw; only declared model scores and separately labelled eligibility are intervened on. Fixed-support replay is not authority to dispatch an action no longer eligible. Not empirical causal identification or an observed alternative outcome."}),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn boundaries_replay_and_exact_probabilities() {
        let mut p = Policy {
            margin: 0.1,
            epsilon: 0.0,
            seed: 7,
        };
        let state = Hash::of(&[b"same"]);
        assert_eq!(
            select(&p, state, &[]).unwrap().reason,
            "no_admissible_action"
        );
        let a = vec![
            ("b".into(), 1.0),
            ("a".into(), 1.0),
            ("outside".into(), 0.1),
        ];
        assert_eq!(select(&p, state, &a).unwrap().reason, "near_tie_abstention");
        assert_eq!(
            select(&p, state, &[("a".into(), 1.0), ("b".into(), 0.95)])
                .unwrap()
                .selected,
            None
        );
        assert_eq!(
            select(&p, state, &[("a".into(), 1.0), ("b".into(), 0.8)])
                .unwrap()
                .propensity,
            Some(1.0)
        );
        p.epsilon = 0.5;
        let t = select(&p, state, &a).unwrap();
        assert_eq!(
            t.probabilities,
            vec![
                ("a".into(), 0.75),
                ("b".into(), 0.25),
                ("outside".into(), 0.0)
            ]
        );
        assert_eq!(
            t.propensity,
            Some(
                t.probabilities
                    .iter()
                    .find(|(id, _)| Some(id) == t.selected.as_ref())
                    .unwrap()
                    .1
            )
        );
        let mut reversed = a.clone();
        reversed.reverse();
        assert_eq!(t, select(&p, state, &reversed).unwrap());
        assert_eq!(t, select(&p, state, &a).unwrap());
        p.epsilon = 1.0;
        assert_eq!(select(&p, state, &a).unwrap().probabilities[0].1, 0.5);
        p.epsilon = f64::NAN;
        assert!(select(&p, state, &a).is_err());
    }
    #[test]
    fn quantized_three_way_and_invalid_candidates() {
        let state = Hash::of(&[b"three"]);
        let mut p = Policy {
            margin: 0.0,
            epsilon: 1.0,
            seed: 3,
        };
        let candidates = vec![("a".into(), 1.0), ("b".into(), 1.0), ("c".into(), 1.0)];
        let trace = select(&p, state, &candidates).unwrap();
        assert_eq!(trace.probabilities.iter().map(|(_, p)| p).sum::<f64>(), 1.0);
        assert!(trace
            .probabilities
            .iter()
            .all(|(_, p)| (*p - 1.0 / 3.0).abs() < 1e-15));
        p.epsilon = f64::MIN_POSITIVE;
        let trace = select(&p, state, &candidates).unwrap();
        assert_eq!(
            trace.probabilities,
            vec![("a".into(), 1.0), ("b".into(), 0.0), ("c".into(), 0.0)]
        );
        assert_eq!(trace.selected, Some("a".into()));
        assert!(select(&p, state, &[("a".into(), 1.0), ("a".into(), 2.0)]).is_err());
        for score in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert!(select(&p, state, &[("a".into(), score)]).is_err());
        }
    }
    #[test]
    fn interventions_hold_draw_and_expose_support_changes() {
        let state = Hash::of(&[b"intervention"]);
        let p = Policy {
            margin: 0.2,
            epsilon: 0.5,
            seed: 17,
        };
        let scores = vec![("a".into(), 1.0), ("b".into(), 0.9)];
        let factual = select(&p, state, &scores).unwrap();
        let ids = vec!["a".into(), "b".into()];
        let same = counterfactual(&factual, &scores, &ids).unwrap();
        assert_eq!(
            same["fixed_support"],
            serde_json::to_value(&factual).unwrap()
        );
        assert_eq!(same["same_draw_choice_changed"], false);
        let reversed = vec![("b".into(), 1.0), ("a".into(), 0.9)];
        let changed = counterfactual(&factual, &reversed, &ids).unwrap();
        assert_eq!(
            changed["fixed_support"]["draw"],
            serde_json::json!(factual.draw)
        );
        assert_eq!(changed["ranking_changed"], true);
        assert_eq!(
            changed["factual"]["propensity"],
            serde_json::json!(factual.propensity)
        );
        let reduced = counterfactual(&factual, &scores, &["b".into()]).unwrap();
        assert_eq!(reduced["support_changed"], true);
        assert_eq!(reduced["fixed_support"], same["fixed_support"]);
        assert_eq!(reduced["changed_support_scenario"]["selected"], "b");
        let none = counterfactual(&factual, &scores, &[]).unwrap();
        assert_eq!(
            none["changed_support_scenario"]["reason"],
            "no_admissible_action"
        );
        let missing = counterfactual(&factual, &[("b".into(), 0.9)], &["b".into()]).unwrap();
        assert!(missing["fixed_support"].is_null());
        assert_eq!(missing["missing_factual_scores"], serde_json::json!(["a"]));
        let mut historical = factual.clone();
        historical.version = "unknown-policy@v0".into();
        assert_eq!(
            counterfactual(&historical, &scores, &ids).unwrap()["status"],
            "unsupported_historical_policy"
        );
        let mut forged = factual.clone();
        forged.propensity = Some(0.123);
        assert!(counterfactual(&forged, &scores, &ids).is_err());
    }
    #[test]
    fn absent_historical_draw_never_becomes_invented_sampling_evidence() {
        let state = Hash::of(&[b"no-draw"]);
        let scores = vec![("a".into(), 1.0), ("b".into(), 0.1)];
        let p = Policy {
            margin: 0.0,
            epsilon: 1.0,
            seed: 3,
        };
        let factual = select(&p, state, &scores).unwrap();
        assert_eq!(factual.draw, None);
        let tied = vec![("a".into(), 1.0), ("b".into(), 1.0)];
        let ids = vec!["a".into(), "b".into()];
        let c = counterfactual(&factual, &tied, &ids).unwrap();
        assert_eq!(
            c["fixed_support"]["reason"],
            "missing_saved_draw_for_sampling"
        );
        assert!(c["fixed_support"]["selected"].is_null());
        assert!(c["same_draw_choice_changed"].is_null());
        let p = Policy { epsilon: 0.0, ..p };
        let factual = select(&p, state, &tied).unwrap();
        assert_eq!(factual.reason, "near_tie_abstention");
        let c = counterfactual(&factual, &scores, &ids).unwrap();
        assert_eq!(c["fixed_support"]["selected"], "a");
        assert_eq!(c["same_draw_choice_changed"], true);
        let empty = select(&p, state, &[]).unwrap();
        assert_eq!(
            counterfactual(&empty, &[], &[]).unwrap()["fixed_support"]["reason"],
            "no_admissible_action"
        );
        let one = vec![("a".into(), 1.0)];
        let factual = select(&p, state, &one).unwrap();
        assert_eq!(
            counterfactual(&factual, &one, &["a".into()]).unwrap()["fixed_support"]["propensity"],
            1.0
        );
    }
}
