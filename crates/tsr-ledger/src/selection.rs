//! Deterministic M3 preference selection. Scores are ephemeral folds, never stored.
//! Kernel `lineage-half-retain@v1`: half the mass stays at each node, half is
//! shared over its undirected parent/catalyst edges. Scope supplies the seed.
use crate::CollectiveReality;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use tsr_domain::{Hash, PreferenceId, Scope, Snapshot, Tune};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SelectionContext {
    pub family: String,
    pub repository: String,
    /// Operator-declared minimum number of admitted open preference nodes.
    pub minimum_depth: u32,
}

/// Audit facts only: no persisted diffusion weights or inferred semantic labels.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SelectionTrace {
    pub state_root: Hash,
    pub state_version: u64,
    pub kernel: String,
    pub context: SelectionContext,
    pub settings: Tune,
    pub seed: u64,
    pub quiet_selected: bool,
    pub selected: Vec<PreferenceId>,
    pub abstention: Option<String>,
}

pub fn select(
    ledger: &CollectiveReality,
    snapshot: &Snapshot,
    tune: &Tune,
    context: &SelectionContext,
    seed: u64,
) -> Result<SelectionTrace, &'static str> {
    if ledger.root != snapshot.root {
        return Err("ledger does not match the pinned snapshot");
    }
    if !tune.quiet_route.is_finite() || !(0.0..=1.0).contains(&tune.quiet_route) {
        return Err("quiet-route probability must be finite and within [0,1]");
    }
    if tune.flow_horizon > 10_000 || ledger.entries.len() > 10_000 {
        return Err("selection exceeds the bounded v1 graph/horizon limits");
    }
    let mut trace = SelectionTrace {
        state_root: snapshot.root,
        state_version: snapshot.version,
        kernel: "lineage-half-retain@v1".into(),
        context: context.clone(),
        settings: tune.clone(),
        seed,
        quiet_selected: false,
        selected: Vec::new(),
        abstention: None,
    };
    let nodes: Vec<_> = ledger
        .entries
        .iter()
        .filter(|entry| entry.is_open() && tune.admitted_sources.contains(&entry.preference.origin))
        .collect();
    if nodes.len() < context.minimum_depth as usize || nodes.is_empty() {
        trace.abstention = Some("insufficient admitted open preference support".into());
        return Ok(trace);
    }
    let index: BTreeMap<_, _> = nodes
        .iter()
        .enumerate()
        .map(|(i, n)| (n.preference.id, i))
        .collect();
    let mut edges = vec![BTreeSet::new(); nodes.len()];
    for (i, node) in nodes.iter().enumerate() {
        for parent in node.preference.parents.iter().chain(&node.catalyzed_by) {
            if let Some(&j) = index.get(parent) {
                if i != j {
                    edges[i].insert(j);
                    edges[j].insert(i);
                }
            }
        }
    }
    let digest = Hash::of(&[b"m3-quiet-route@v1", &snapshot.root.0, &seed.to_le_bytes()]);
    let draw = u64::from_le_bytes(digest.0[..8].try_into().expect("eight bytes")) >> 11;
    trace.quiet_selected = (draw as f64 / (1u64 << 53) as f64) < tune.quiet_route;
    let mut mass = vec![0.0; nodes.len()];
    if trace.quiet_selected {
        // Leading-edge route: most recently admitted open node in log order.
        *mass.last_mut().expect("nonempty") = 1.0;
    } else {
        for (i, node) in nodes.iter().enumerate() {
            let relevant = match &node.preference.scope {
                Scope::Task(task) => *task == snapshot.task,
                Scope::Family(family) => family == &context.family,
                Scope::Repository(repo) => repo == &context.repository,
                Scope::Operator => true,
            };
            if relevant {
                mass[i] = 1.0;
            }
        }
        let total: f64 = mass.iter().sum();
        if total == 0.0 {
            trace.abstention = Some("no scope-matched seed".into());
            return Ok(trace);
        }
        for value in &mut mass {
            *value /= total;
        }
    }
    for _ in 0..tune.flow_horizon {
        let mut next = vec![0.0; nodes.len()];
        for (i, neighbors) in edges.iter().enumerate() {
            if neighbors.is_empty() {
                next[i] += mass[i];
                continue;
            }
            next[i] += mass[i] * 0.5;
            for &j in neighbors {
                next[j] += mass[i] * 0.5 / neighbors.len() as f64;
            }
        }
        mass = next;
    }
    let mut ranked: Vec<_> = nodes
        .iter()
        .enumerate()
        .filter(|(i, _)| mass[*i] > 0.0)
        .collect();
    ranked.sort_by(|(i, a), (j, b)| {
        mass[*j]
            .total_cmp(&mass[*i])
            .then(a.preference.id.cmp(&b.preference.id))
    });
    trace.selected = ranked
        .iter()
        .take(tune.intuition_max as usize)
        .map(|(_, n)| n.preference.id)
        .collect();
    Ok(trace)
}

/// Append a reference to the complete trace, supplied to a trusted artifact writer.
/// The writer must persist bytes outside the candidate workspace before returning.
/// No inference output is stored as a preference or as authority.
pub fn record_selection(
    store: &tsr_store::Store,
    snapshot: &Snapshot,
    trace: &SelectionTrace,
    persist: impl FnOnce(Hash, &[u8]) -> Result<(), String>,
    recorded_at: &str,
) -> Result<tsr_domain::EventId, String> {
    use tsr_domain::{Event, EventKind, Observation, Source};
    if store.head_root().map_err(|e| e.to_string())? != snapshot.root
        || trace.state_root != snapshot.root
        || trace.state_version != snapshot.version
    {
        return Err("stale selection trace".into());
    }
    let events = store.by_task(&snapshot.task).map_err(|e| e.to_string())?;
    if events.is_empty() || events.len() as u64 != snapshot.version {
        return Err("unknown task or stale task version".into());
    }
    let bytes = serde_json::to_vec(trace).map_err(|e| e.to_string())?;
    let digest = Hash::of(&[&bytes]);
    persist(digest, &bytes)?;
    let parents = events.last().map(|(_, e)| vec![e.id]).unwrap_or_default();
    let event = Event::new(
        snapshot.task,
        parents,
        Source::Rules,
        EventKind::Observed {
            observation: Observation::Evidence {
                digest,
                source: "m3/select@v1".into(),
            },
        },
        recorded_at,
    );
    store.append(&event).map_err(|e| e.to_string())?;
    Ok(event.id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tsr_domain::{Preference, PreferenceStatus, Source, TaskId};
    fn setup() -> (CollectiveReality, Snapshot, Tune, SelectionContext) {
        let root = Hash::of(&[b"fixture"]);
        let task = TaskId(Hash::of(&[b"task"]));
        let first = Preference::new("fast", Scope::Task(task), vec![], Source::Operator).unwrap();
        let second = Preference::new(
            "small",
            Scope::Family("other".into()),
            vec![first.id],
            Source::Operator,
        )
        .unwrap();
        let entries = [first, second]
            .into_iter()
            .map(|preference| crate::LedgerEntry {
                preference,
                expanded_by: None,
                from_contrast: None,
                from_success: None,
                from_provisional: None,
                catalyzed_by: vec![],
                status: Some(PreferenceStatus::Open),
                history: vec![],
            })
            .collect();
        (
            CollectiveReality { entries, root },
            Snapshot {
                task,
                version: 0,
                root,
                open_criteria: vec![],
                missing_evidence: vec![],
            },
            Tune {
                support_floor: 1,
                flow_horizon: 0,
                intuition_max: 2,
                ..Tune::default()
            },
            SelectionContext {
                family: "family".into(),
                repository: "repo".into(),
                minimum_depth: 1,
            },
        )
    }
    #[test]
    fn horizon_controls_diffusion_and_closed_nodes_never_return() {
        let (mut ledger, snap, mut tune, ctx) = setup();
        assert_eq!(
            select(&ledger, &snap, &tune, &ctx, 1)
                .unwrap()
                .selected
                .len(),
            1
        );
        tune.flow_horizon = 1;
        assert_eq!(
            select(&ledger, &snap, &tune, &ctx, 1)
                .unwrap()
                .selected
                .len(),
            2
        );
        ledger.entries[1].status = Some(PreferenceStatus::Withdrawn);
        assert_eq!(
            select(&ledger, &snap, &tune, &ctx, 1).unwrap().selected,
            vec![ledger.entries[0].preference.id]
        );
    }
    #[test]
    fn quiet_frontier_is_reproducible_and_support_floor_abstains() {
        let (ledger, snap, mut tune, mut ctx) = setup();
        tune.quiet_route = 1.0;
        let trace = select(&ledger, &snap, &tune, &ctx, 3).unwrap();
        assert!(trace.quiet_selected);
        assert_eq!(trace.selected, vec![ledger.entries[1].preference.id]);
        assert_eq!(trace, select(&ledger, &snap, &tune, &ctx, 3).unwrap());
        ctx.minimum_depth = 3;
        assert!(select(&ledger, &snap, &tune, &ctx, 3)
            .unwrap()
            .selected
            .is_empty());
    }
    #[test]
    fn provenance_and_settings_are_checked() {
        let (mut ledger, snap, mut tune, ctx) = setup();
        tune.quiet_route = f64::NAN;
        assert!(select(&ledger, &snap, &tune, &ctx, 0).is_err());
        tune.quiet_route = 0.0;
        ledger.root = Hash::of(&[b"different"]);
        assert!(select(&ledger, &snap, &tune, &ctx, 0).is_err());
    }
}
