//! Pure obligation assessment shared by domain adapters. No dispatch or admission authority.
//! Preference identity and refinement identity are separate from lineage: a child may
//! focus generation, but inherits the original obligation's decision weight.
use crate::{EventId, PreferenceId};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const KERNEL: &str = "conserved-obligations@v1";
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Obligation {
    pub id: PreferenceId,
    pub refines: Option<PreferenceId>,
    pub admitted: bool,
    /// None is unknown, not failed or fulfilled. Evidence remains with the adapter.
    pub observed: Option<f64>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Projection {
    pub obligation: PreferenceId,
    pub anticipated: f64,
    /// Independently justified action bounds, never bounds supplied by the generator.
    pub lower: f64,
    pub upper: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Commitment {
    pub family: String,
    pub supports: Vec<PreferenceId>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Assessment {
    pub kernel: String,
    pub score: f64,
    pub gain: f64,
    pub standing_loss: f64,
    pub continuity_cost: f64,
    pub projected: BTreeMap<PreferenceId, f64>,
    pub unresolved: Vec<PreferenceId>,
    pub observation: BTreeMap<PreferenceId, Option<f64>>,
    pub selected: Vec<PreferenceId>,
    pub probability: f64,
    pub effort: f64,
    pub uncertainty_policy: String,
}
/// Caller supplies only controller-validated authority; this flag never grants it.
pub struct Request<'a> {
    pub obligations: &'a [Obligation],
    pub selected: &'a [PreferenceId],
    pub proposal: &'a Commitment,
    pub previous: Option<&'a Commitment>,
    pub projections: &'a [Projection],
    pub probability: f64,
    pub effort: f64,
    pub authorized: bool,
}
fn unit(v: f64) -> bool {
    v.is_finite() && (0.0..=1.0).contains(&v)
}
fn roots(obligations: &[Obligation]) -> Result<BTreeMap<PreferenceId, PreferenceId>, &'static str> {
    if obligations.is_empty() || obligations.len() > 256 {
        return Err("bounded obligations required");
    }
    let by_id: BTreeMap<_, _> = obligations.iter().map(|o| (o.id, o)).collect();
    if by_id.len() != obligations.len() {
        return Err("duplicate obligation identity");
    }
    let mut roots = BTreeMap::new();
    for o in obligations {
        if o.observed.is_some_and(|v| !unit(v)) {
            return Err("invalid observation");
        }
        let mut at = o.id;
        let mut visited = BTreeSet::new();
        loop {
            if !visited.insert(at) {
                return Err("cyclic refinement");
            }
            let node = by_id.get(&at).ok_or("missing refinement parent")?;
            if !node.admitted {
                break;
            }
            if let Some(parent) = node.refines {
                at = parent;
            } else {
                roots.insert(o.id, at);
                break;
            }
        }
    }
    Ok(roots)
}
fn canonical(
    ids: &[PreferenceId],
    roots: &BTreeMap<PreferenceId, PreferenceId>,
) -> Result<BTreeSet<PreferenceId>, &'static str> {
    ids.iter()
        .map(|id| {
            roots
                .get(id)
                .copied()
                .ok_or("unadmitted or unknown obligation")
        })
        .collect()
}
pub fn assess(r: Request<'_>) -> Result<Assessment, &'static str> {
    if !r.authorized {
        return Err("action authority unresolved or denied");
    }
    if !unit(r.probability) || !r.effort.is_finite() || r.effort < 0.0 {
        return Err("invalid probability or cost");
    }
    let roots = roots(r.obligations)?;
    let selected = canonical(r.selected, &roots)?;
    let supports = canonical(&r.proposal.supports, &roots)?;
    if selected.is_empty() || supports.is_disjoint(&selected) {
        return Err("no admitted selected support");
    }
    let originals: Vec<_> = r
        .obligations
        .iter()
        .filter(|o| roots.get(&o.id) == Some(&o.id))
        .collect();
    let projections: BTreeMap<_, _> = r.projections.iter().map(|p| (p.obligation, p)).collect();
    if projections.len() != r.projections.len() {
        return Err("duplicate projection");
    }
    for p in r.projections {
        if !roots.contains_key(&p.obligation)
            || !unit(p.anticipated)
            || !unit(p.lower)
            || !unit(p.upper)
            || p.lower > p.upper
        {
            return Err("invalid projection");
        }
    }
    let mut before = BTreeMap::new();
    let mut projected = BTreeMap::new();
    let mut unresolved = vec![];
    for o in originals {
        let p = projections
            .get(&o.id)
            .ok_or("missing original projection")?;
        let mut value = p.anticipated.clamp(p.lower, p.upper);
        if let Some(current) = o.observed {
            // Known regressions remain visible even under optimistic forecasts.
            if p.upper < current {
                value = value.min(p.upper);
            }
            before.insert(o.id, current);
        } else {
            unresolved.push(o.id);
            // Planning from unknown is allowed, but never records a successful return.
            before.insert(o.id, 0.0);
        }
        projected.insert(o.id, value);
    }
    let floor = |values: &BTreeMap<PreferenceId, f64>| {
        selected.iter().map(|i| values[i]).fold(1.0, f64::min)
    };
    let mean = selected
        .iter()
        .map(|i| (projected[i] - before[i]).max(0.0))
        .sum::<f64>()
        / selected.len() as f64;
    let loss = before
        .iter()
        .map(|(i, v)| (v - projected[i]).max(0.0))
        .sum::<f64>();
    let gain = (floor(&projected) - floor(&before)).max(0.0) + 0.25 * mean - loss;
    let net = r.probability * gain - r.effort;
    let switching = if let Some(previous) = r.previous {
        let old = canonical(&previous.supports, &roots)?;
        old.iter().any(|i| before[i] < 1.0)
            && (previous.family != r.proposal.family || old != supports)
    } else {
        false
    };
    let continuity = if switching {
        (net.max(0.0) * 0.25).min(0.04)
    } else {
        0.0
    };
    Ok(Assessment {kernel:KERNEL.into(),score:net-continuity,gain,standing_loss:loss,continuity_cost:continuity,projected,unresolved,
        observation:r.obligations.iter().filter(|o|roots.get(&o.id)==Some(&o.id)).map(|o|(o.id,o.observed)).collect(),
        selected:selected.into_iter().collect(),probability:r.probability,effort:r.effort,
        uncertainty_policy:"Unknown uses an optimistic zero planning baseline; no satisfaction or regression claim is established until observed.".into(),
    })
}
/// Receding-horizon assessment; execution/admission remains the adapter's job.
/// `terminal.obligations` are the original current observations, never simulated
/// intermediate outcomes promoted to facts. Bounds must come from the adapter.
pub struct PathRequest<'a> {
    pub terminal: Request<'a>,
    pub intermediate: &'a [Projection],
    pub prerequisite_effort: f64,
    pub delay_fraction: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PathAssessment {
    pub kernel: String,
    pub terminal: Assessment,
    pub score: f64,
    pub intermediate_bounds: BTreeMap<PreferenceId, (f64, f64)>,
    pub standing_risk: f64,
    pub additional_risk_cost: f64,
    pub prerequisite_effort: f64,
    pub delay_cost: f64,
    pub uncertainty_policy: String,
}
pub fn assess_path(r: PathRequest<'_>) -> Result<PathAssessment, &'static str> {
    if !r.prerequisite_effort.is_finite() || r.prerequisite_effort < 0.0 || !unit(r.delay_fraction)
    {
        return Err("invalid prerequisite effort or horizon discount");
    }
    let canonical = roots(r.terminal.obligations)?;
    let mut bounds = BTreeMap::new();
    for p in r.intermediate {
        if !canonical.contains_key(&p.obligation)
            || !unit(p.lower)
            || !unit(p.upper)
            || !unit(p.anticipated)
            || p.lower > p.upper
            || bounds.insert(p.obligation, (p.lower, p.upper)).is_some()
        {
            return Err("invalid or duplicate intermediate projection");
        }
    }
    let terminal_lower: BTreeMap<_, _> = r
        .terminal
        .projections
        .iter()
        .map(|p| (p.obligation, p.lower))
        .collect();
    let terminal = assess(r.terminal)?;
    let mut originals = BTreeMap::new();
    let mut risk = 0.0;
    for (id, current) in &terminal.observation {
        let bound = *bounds
            .get(id)
            .ok_or("missing original intermediate bound")?;
        originals.insert(*id, bound);
        // Unknown is not satisfied evidence. One is only the conservative planning
        // baseline for potential harm; it never changes observation or admission.
        let baseline = current.unwrap_or(1.0);
        risk += (baseline - bound.0.min(terminal_lower[id])).max(0.0);
    }
    // Charge each original's worst prospective loss once, at full weight. The
    // ordinary terminal score already includes probability * terminal loss.
    let additional_risk = (risk - terminal.probability * terminal.standing_loss).max(0.0);
    let delay =
        r.delay_fraction * terminal.probability * (terminal.gain + terminal.standing_loss).max(0.0);
    Ok(PathAssessment {
        kernel: "conserved-prerequisite-path@v1".into(),
        score: terminal.score - r.prerequisite_effort - delay - additional_risk,
        terminal, intermediate_bounds: originals, standing_risk: risk,
        additional_risk_cost: additional_risk, prerequisite_effort: r.prerequisite_effort,
        delay_cost: delay,
        uncertainty_policy: "Intermediate and terminal lower bounds price worst potential loss across each original obligation; unknown current state uses one only as a conservative risk baseline. Future effects remain predictions, never observed satisfaction or authority.".into(),
    })
}

/// A fold over independently observed returns. Missing/disputed outcomes do not
/// become failures; evidence IDs identify the receipts, not provider assertions.
#[derive(Clone, Debug)]
pub struct Return {
    pub evidence: EventId,
    pub weight: f64,
    pub success: Option<bool>,
    pub predicted: Option<f64>,
    pub observed: Option<f64>,
}
#[derive(Clone, Debug)]
pub struct Expectation {
    pub probability: f64,
    pub residual: f64,
    pub evidence: Vec<EventId>,
}
pub fn expectation(returns: &[Return]) -> Result<Expectation, &'static str> {
    let mut yes = 0.0;
    let mut total = 0.0;
    let mut residual = 0.0;
    let mut n = 0.0;
    let mut evidence = BTreeSet::new();
    for r in returns {
        if !r.weight.is_finite()
            || !(0.0..=1.0).contains(&r.weight)
            || r.predicted.is_some_and(|v| !unit(v))
            || r.observed.is_some_and(|v| !unit(v))
        {
            return Err("invalid return evidence");
        }
        if r.weight == 0.0 {
            continue;
        }
        if !evidence.insert(r.evidence) {
            return Err("duplicate return receipt");
        }
        if let Some(success) = r.success {
            total += r.weight;
            if success {
                yes += r.weight;
            }
        }
        if let (Some(p), Some(a)) = (r.predicted, r.observed) {
            residual += (p - a) * r.weight;
            n += r.weight;
        }
    }
    Ok(Expectation {
        probability: (1.0 + yes) / (2.0 + total),
        residual: if n > 0.0 { residual / n } else { 0.0 },
        evidence: evidence.into_iter().collect(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Hash;
    fn id(n: u8) -> PreferenceId {
        PreferenceId(Hash::of(&[&[n]]))
    }
    fn rank(
        obs: &[Obligation],
        selected: &[PreferenceId],
        supports: &[PreferenceId],
        previous: Option<&Commitment>,
        authorized: bool,
    ) -> Result<Assessment, &'static str> {
        let projections = obs
            .iter()
            .map(|o| Projection {
                obligation: o.id,
                anticipated: 0.8,
                lower: 0.0,
                upper: 1.0,
            })
            .collect::<Vec<_>>();
        assess(Request {
            obligations: obs,
            selected,
            proposal: &Commitment {
                family: "act".into(),
                supports: supports.into(),
            },
            previous,
            projections: &projections,
            probability: 0.6,
            effort: 0.01,
            authorized,
        })
    }
    fn path_example(
        intermediate: f64,
        probability: f64,
        unknown: bool,
        refinement: bool,
    ) -> PathAssessment {
        path_with_terminal_bound(intermediate, probability, unknown, refinement, 1.0)
    }
    fn path_with_terminal_bound(
        intermediate: f64,
        probability: f64,
        unknown: bool,
        refinement: bool,
        terminal_lower: f64,
    ) -> PathAssessment {
        let mut obligations = vec![
            Obligation {
                id: id(0),
                refines: None,
                admitted: true,
                observed: Some(0.0),
            },
            Obligation {
                id: id(1),
                refines: None,
                admitted: true,
                observed: if unknown { None } else { Some(1.0) },
            },
        ];
        if refinement {
            obligations.push(Obligation {
                id: id(2),
                refines: Some(id(0)),
                admitted: true,
                observed: Some(0.0),
            });
        }
        let terminal = vec![
            Projection {
                obligation: id(0),
                anticipated: 1.0,
                lower: 1.0,
                upper: 1.0,
            },
            Projection {
                obligation: id(1),
                anticipated: 1.0,
                lower: terminal_lower,
                upper: 1.0,
            },
        ];
        let mut middle = vec![
            Projection {
                obligation: id(0),
                anticipated: 0.5,
                lower: 0.5,
                upper: 0.5,
            },
            Projection {
                obligation: id(1),
                anticipated: 1.0,
                lower: intermediate,
                upper: 1.0,
            },
        ];
        if refinement {
            middle.push(Projection {
                obligation: id(2),
                anticipated: 1.0,
                lower: 0.0,
                upper: 1.0,
            });
        }
        let selected = if refinement {
            vec![id(2), id(0)]
        } else {
            vec![id(0)]
        };
        assess_path(PathRequest {
            terminal: Request {
                obligations: &obligations,
                selected: &selected,
                proposal: &Commitment {
                    family: "act".into(),
                    supports: selected.clone(),
                },
                previous: None,
                projections: &terminal,
                probability,
                effort: 0.01,
                authorized: true,
            },
            intermediate: &middle,
            prerequisite_effort: 0.02,
            delay_fraction: 0.1,
        })
        .unwrap()
    }
    #[test]
    fn optimistic_terminal_prediction_cannot_hide_terminal_risk() {
        let safe = path_example(1.0, 0.8, false, false);
        let risky = path_with_terminal_bound(1.0, 0.8, false, false, 0.0);
        assert_eq!(risky.standing_risk, 1.0);
        assert!((safe.score - risky.score - 1.0).abs() < 1e-12);
        // The same lost original is charged once across both stages.
        let both = path_with_terminal_bound(0.0, 0.8, false, false, 0.0);
        assert_eq!(both.standing_risk, risky.standing_risk);
        assert_eq!(both.score, risky.score);
        assert_eq!(
            path_with_terminal_bound(0.4, 0.8, false, false, 0.2).standing_risk,
            0.8
        );
        assert_eq!(
            path_with_terminal_bound(0.2, 0.8, false, false, 0.4).standing_risk,
            0.8
        );
    }
    #[test]
    fn intermediate_harm_is_not_free_even_when_terminal_promises_recovery() {
        let safe = path_example(1.0, 0.8, false, false);
        let harmful = path_example(0.0, 0.8, false, false);
        assert_eq!(safe.standing_risk, 0.0);
        assert_eq!(harmful.standing_risk, 1.0);
        assert!((safe.score - harmful.score - 1.0).abs() < 1e-12);
        assert!(safe.score > 0.0 && harmful.score < 0.0);
        assert_eq!(path_example(0.0, 0.01, false, false).standing_risk, 1.0);
        assert_eq!(safe.terminal.observation[&id(0)], Some(0.0));
        assert_eq!(safe.intermediate_bounds[&id(0)], (0.5, 0.5));
        assert_eq!(safe.score, path_example(1.0, 0.8, false, true).score);
        assert_eq!(safe.score, path_example(1.0, 0.8, false, false).score);
    }
    #[test]
    fn path_uncertainty_remains_unknown_and_lower_bounds_price_risk() {
        let uncertain = path_example(0.2, 0.8, true, false);
        assert_eq!(uncertain.terminal.observation[&id(1)], None);
        assert!(uncertain.terminal.unresolved.contains(&id(1)));
        assert!((uncertain.standing_risk - 0.8).abs() < 1e-12);
        assert!(uncertain.score < path_example(1.0, 0.8, true, false).score);
    }

    #[test]
    fn identity_conservation_and_authority_are_domain_independent() {
        let mut obs = vec![
            Obligation {
                id: id(0),
                refines: None,
                admitted: true,
                observed: Some(0.2),
            },
            Obligation {
                id: id(1),
                refines: None,
                admitted: true,
                observed: Some(0.4),
            },
        ];
        let baseline = rank(&obs, &[id(0), id(1)], &[id(1), id(0)], None, true).unwrap();
        obs.push(Obligation {
            id: id(2),
            refines: Some(id(0)),
            admitted: true,
            observed: Some(0.0),
        });
        obs.push(Obligation {
            id: id(3),
            refines: Some(id(2)),
            admitted: true,
            observed: Some(0.0),
        });
        let previous = Commitment {
            family: "act".into(),
            supports: vec![id(1), id(3)],
        };
        let alias = rank(
            &obs,
            &[id(3), id(1), id(0), id(2)],
            &[id(0), id(1)],
            Some(&previous),
            true,
        )
        .unwrap();
        assert_eq!(baseline.score, alias.score);
        assert!(rank(&obs, &[id(0)], &[id(0)], None, false).is_err());
        obs[2].admitted = false;
        assert!(rank(&obs, &[id(3)], &[id(3)], None, true).is_err());
        obs[2].admitted = true;
        obs[2].refines = Some(id(3));
        assert!(rank(&obs, &[id(0)], &[id(0)], None, true).is_err());
    }
    #[test]
    fn unknown_is_exposed_and_never_becomes_an_observed_label() {
        let obs = vec![Obligation {
            id: id(0),
            refines: None,
            admitted: true,
            observed: None,
        }];
        assert_eq!(
            rank(&obs, &[id(0)], &[id(0)], None, true)
                .unwrap()
                .unresolved,
            vec![id(0)]
        );
        let missing = Return {
            evidence: EventId(Hash::of(&[b"missing"])),
            weight: 1.0,
            success: None,
            predicted: Some(1.0),
            observed: None,
        };
        let e = expectation(std::slice::from_ref(&missing)).unwrap();
        assert_eq!(e.probability, 0.5);
        assert_eq!(e.residual, 0.0);
        assert!(expectation(&[missing.clone(), missing]).is_err());
    }
    #[test]
    fn contradictory_returns_change_expectation_with_restore_control() {
        let point = |s| Return {
            evidence: EventId(Hash::of(&[&[s as u8]])),
            weight: 1.0,
            success: Some(s),
            predicted: Some(0.9),
            observed: Some(if s { 1.0 } else { 0.0 }),
        };
        let yes = expectation(&[point(true)]).unwrap();
        let both = expectation(&[point(true), point(false)]).unwrap();
        assert!(yes.probability > both.probability);
        assert!(both.residual > yes.residual);
        assert_eq!(
            yes.probability,
            expectation(&[point(true)]).unwrap().probability
        );
    }
}
