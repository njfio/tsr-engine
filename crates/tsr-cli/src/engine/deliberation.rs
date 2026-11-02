//! Ledger-driven direction selection over a bounded, real evidence notebook.
//! No generated code, commands, paths or external actions are executable here.
use super::*;
pub mod archive;
pub mod relational;
use std::collections::BTreeSet;
use tsr_domain::decision as kernel;
use tsr_ledger::selection::{select, SelectionContext};
const SOURCE: &str = "notebook-deliberation@v1";
const KERNEL: &str = "shared-obligation-kernel@v11";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Note {
    pub id: String,
    pub topic: String,
    pub text: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "predicate", rename_all = "snake_case", deny_unknown_fields)]
pub enum Aim {
    ArchiveTraceability,
    ArchiveContestability,
    Understand { topic: String },
    Account { topic: String },
    Concise { max_words: usize },
    InspectNotes { notes: Vec<String> },
    IncludeNotes { notes: Vec<String> },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Want {
    pub text: String,
    pub aim: Aim,
    #[serde(default)]
    pub parents: Vec<usize>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Spec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archive: Option<archive::World>,
    pub wants: Vec<Want>,
    pub notes: Vec<Note>,
    pub max_cycles: usize,
    pub attention_limit: u8,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "operation", rename_all = "snake_case", deny_unknown_fields)]
pub enum Move {
    Archive { event: archive::Operation },
    Inspect { notes: Vec<String> },
    Compose { title: String, claims: Vec<String> },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Continuation {
    pub anticipated: Vec<f64>,
    pub action: Move,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Direction {
    /// Explicit bounded information fallback; never a terminal outcome forecast.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deferred_observation: Option<Hash>,
    pub purpose: String,
    pub supports: Vec<usize>,
    pub anticipated: Vec<f64>,
    pub action: Move,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuation: Option<Continuation>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SelectionTuneUpdate {
    pub state_root: Hash,
    pub quiet_route: f64,
    pub seed: u64,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DesireStatusUpdate {
    pub state_root: Hash,
    pub preference: PreferenceId,
    pub status: PreferenceStatus,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Offer {
    pub state_root: Hash,
    pub directions: Vec<Direction>,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Notebook {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub archive_events: Vec<archive::Operation>,
    pub known: BTreeSet<String>,
    pub title: String,
    pub claims: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PlanAssessment {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<kernel::PathAssessment>,
    pub first_step_progress_probability: f64,
    pub newly_inspected: Vec<String>,
    pub delay_cost: f64,
    pub terminal_status: String,
    pub assessment_scope: String,
}
/// Evidence support is descriptive; correlated dispatches are not mature episodes.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ExpectationSupport {
    pub event: String,
    pub compatible_dispatches: usize,
    pub effective_dispatch_weight: f64,
    pub weighted_receipts: Vec<(EventId, f64)>,
    pub independent_mature_episodes: usize,
    pub calibration_status: String,
    pub applicability: String,
    pub uncertainty: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Fallback {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact_first_step_net: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub exact_standing_loss: Option<f64>,
    pub policy: String,
    pub eligible: bool,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Assessment {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expectation_support: Option<ExpectationSupport>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fallback: Option<Fallback>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan: Option<PlanAssessment>,
    #[serde(default)]
    pub shared: Option<kernel::Assessment>,
    #[serde(default)]
    pub kernel: String,
    pub direction: Direction,
    #[serde(default)]
    pub proposed_supports: Vec<usize>,
    #[serde(default)]
    pub rejected_supports: Vec<usize>,
    pub prediction: Proxy<f64>,
    #[serde(default)]
    pub prediction_evidence: Vec<EventId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub calibration_evidence: Vec<EventId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub projection_basis: Option<String>,
    pub projected: Vec<f64>,
    pub score: Proxy<f64>,
    pub continuity_cost: f64,
    #[serde(default)]
    pub standing_loss: f64,
    #[serde(default = "zero_gain")]
    pub projected_gain: Proxy<f64>,
    #[serde(default)]
    pub effort_cost: f64,
    #[serde(default = "empty_calibration")]
    pub calibration: Proxy<Vec<f64>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Turn {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_obligations: Option<Vec<PreferenceId>>,
    #[serde(default)]
    pub observation_revision: Option<Hash>,
    #[serde(default)]
    pub before_notebook: Option<Notebook>,
    #[serde(default)]
    pub rejected_directions: Vec<(usize, String)>,
    pub offered: Vec<Assessment>,
    pub chosen: usize,
    #[serde(default)]
    pub selected: Vec<usize>,
    pub before: Vec<f64>,
    pub after: Vec<f64>,
    pub observed_progress: bool,
    pub notebook: Notebook,
    pub commitment: EventId,
    pub residual: f64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub generator_residual: Vec<f64>,
    pub contrast: Vec<usize>,
    #[serde(default)]
    pub return_contrast: Option<Contrast>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "record")]
enum Record {
    Relational {
        entry: relational::Entry,
    },
    SelectionTuned {
        quiet_route: f64,
        seed: u64,
        reason: String,
    },
    EvidenceUpdated {
        from: Hash,
        to: Hash,
        notes: Vec<Note>,
        reason: String,
        before: Vec<f64>,
        after: Vec<f64>,
    },
    Initialized {
        spec: Spec,
        preferences: Vec<PreferenceId>,
    },
    Returned {
        turn: Box<Turn>,
    },
    Refined {
        from: EventId,
        admitted: Vec<(Want, PreferenceId)>,
        rejected: Vec<String>,
    },
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Refinement {
    pub text: String,
    pub parent: usize,
    pub notes: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RefinementOffer {
    pub state_root: Hash,
    pub return_commitment: EventId,
    pub refinements: Vec<Refinement>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationUpdate {
    pub state_root: Hash,
    pub reason: String,
    pub notes: Vec<Note>,
}
// A partial feasibility certificate over this adapter's validated representation.
// Unread text contributes only its guaranteed nonempty one-word lower bound.
// This does not enumerate candidates, choose an action, or establish overall feasibility.
fn constraint_status(spec: &Spec, book: &Notebook, active: &[usize]) -> serde_json::Value {
    let originals: Vec<_> = obligations(spec)
        .into_iter()
        .filter(|i| active.contains(i))
        .collect();
    let mut required = BTreeSet::new();
    let mut coverage = Vec::new();
    let mut budgets = Vec::new();
    for i in originals {
        match &spec.wants[i].aim {
            Aim::Account { topic } => {
                let scope: Vec<_> = spec
                    .notes
                    .iter()
                    .filter(|n| &n.topic == topic)
                    .map(|n| n.id.clone())
                    .collect();
                if !scope.is_empty() {
                    coverage.push(i);
                    required.extend(scope);
                }
            }
            Aim::Concise { max_words } => budgets.push((i, *max_words)),
            _ => {}
        }
    }
    let minimum = 1 + spec
        .notes
        .iter()
        .filter(|n| required.contains(&n.id))
        .map(|n| {
            if book.known.contains(&n.id) {
                words(&n.text)
            } else {
                1
            }
        })
        .sum::<usize>();
    let conflicts: Vec<_> = budgets.into_iter().filter(|(_,limit)|!required.is_empty() && minimum > *limit)
        .map(|(i,limit)|serde_json::json!({"coverage_obligations":coverage,"brevity_obligation":i,
            "required_claim_ids":required,"minimum_account_words":minimum,"maximum_words":limit,
            "basis":"one required title word plus known source lengths and one word per validated nonempty unread source"})).collect();
    serde_json::json!({"status":if conflicts.is_empty(){"joint_conflict_not_established"}else{"proven_joint_conflict"},
        "scope":"notebook coverage versus brevity only; not a complete feasibility solver",
        "obligation_identity":"indices into unchanged standing_desires; originals only",
        "observation_revision":observation_revision(spec),"conflicts":conflicts,
        "meaning":"Existing desire statuses remain unchanged. Joint conflict does not rule out useful partial progress. No action or new authority is proposed."})
}

fn observation_revision(spec: &Spec) -> Hash {
    if let Some(world) = &spec.archive {
        return Hash::of(&[
            b"synthetic-archive@v1",
            &serde_json::to_vec(world).expect("typed world"),
        ]);
    }
    Hash::of(&[&serde_json::to_vec(&spec.notes).expect("serializable notes")])
}
fn replace_evidence(spec: &mut Spec, book: &mut Notebook, notes: &[Note]) {
    for note in notes {
        if let Some(old) = spec.notes.iter_mut().find(|n| n.id == note.id) {
            if old.text != note.text {
                book.known.remove(&note.id);
                book.claims.retain(|id| id != &note.id);
                *old = note.clone();
            }
        }
    }
}
fn zero_gain() -> Proxy<f64> {
    Proxy::new("legacy_unrecorded_gain", 0.0)
}
fn empty_calibration() -> Proxy<Vec<f64>> {
    Proxy::new("legacy_unrecorded_calibration", vec![])
}
fn aim_key(aim: &Aim) -> serde_json::Value {
    let mut aim = aim.clone();
    match &mut aim {
        Aim::InspectNotes { notes } | Aim::IncludeNotes { notes } => notes.sort(),
        _ => {}
    }
    serde_json::to_value(aim).expect("typed aim is serializable")
}
// Initial specifications forbid note-subset aims. Generated refinements have one
// earlier parent, so their original obligation is recoverable without new state.
fn original(spec: &Spec, mut i: usize) -> usize {
    while matches!(
        spec.wants[i].aim,
        Aim::InspectNotes { .. } | Aim::IncludeNotes { .. }
    ) {
        i = spec.wants[i].parents[0];
    }
    i
}
fn obligations(spec: &Spec) -> Vec<usize> {
    (0..spec.wants.len())
        .filter(|i| original(spec, *i) == *i)
        .collect()
}
fn support_set(spec: &Spec, supports: &[usize]) -> BTreeSet<usize> {
    supports.iter().map(|i| original(spec, *i)).collect()
}
#[cfg(test)]
fn original_progress(spec: &Spec, selected: &[usize], before: &[f64], after: &[f64]) -> bool {
    active_progress(
        spec,
        selected,
        &(0..before.len()).collect::<Vec<_>>(),
        before,
        after,
    )
}
fn active_progress(
    spec: &Spec,
    selected: &[usize],
    active: &[usize],
    before: &[f64],
    after: &[f64],
) -> bool {
    support_set(spec, selected)
        .iter()
        .any(|i| active.contains(i) && after[*i] > before[*i])
        && obligations(spec)
            .iter()
            .filter(|i| active.contains(i))
            .all(|i| after[*i] >= before[*i])
}
fn preference_ids(spec: &Spec) -> Result<Vec<PreferenceId>> {
    let mut ids = vec![];
    for (i, w) in spec.wants.iter().enumerate() {
        ids.push(
            Preference::new(
                &w.text,
                Scope::Task(Engine::task()),
                w.parents.iter().map(|p| ids[*p]).collect(),
                if original(spec, i) == i {
                    Source::Operator
                } else {
                    Source::Generator
                },
            )?
            .id,
        );
    }
    Ok(ids)
}
fn historical_active(ids: &[PreferenceId], turn: &Turn) -> Vec<usize> {
    (0..turn.before.len())
        .filter(|i| {
            turn.active_obligations
                .as_ref()
                .is_none_or(|active| active.contains(&ids[*i]))
        })
        .collect()
}
fn return_progress(spec: &Spec, turn: &Turn) -> bool {
    if turn.active_obligations.is_some() {
        turn.observed_progress
    } else {
        active_progress(
            spec,
            &turn.selected,
            &(0..turn.before.len()).collect::<Vec<_>>(),
            &turn.before,
            &turn.after,
        )
    }
}
/// Continuity is a current evidence privilege, not a permanent reward for old success.
fn current_commitment<'a>(spec: &Spec, turns: &'a [Turn]) -> Option<&'a Direction> {
    turns
        .last()
        .filter(|t| {
            t.observation_revision == Some(observation_revision(spec)) && return_progress(spec, t)
        })
        .map(|t| &t.offered[t.chosen].direction)
}
fn commitment_review(spec: &Spec, turns: &[Turn]) -> serde_json::Value {
    let latest = turns.last();
    let status = match latest {
        None => "no_commitment",
        Some(t) if t.observation_revision != Some(observation_revision(spec)) => {
            "evidence_superseded"
        }
        Some(t) if !return_progress(spec, t) => "not_supported_by_return",
        Some(_) => "supported_by_current_return",
    };
    serde_json::json!({"status":status,"evidence_current_and_productive":current_commitment(spec,turns).is_some(),
        "commitment":latest.map(|t|t.commitment),"return_revision":latest.and_then(|t|t.observation_revision),
        "current_revision":observation_revision(spec),
        "meaning":"Action commitment with frozen forecasts, not a semantic truth claim. Superseded evidence releases continuity without rewriting history or erasing transferable experience. Active-support and fulfillment checks still apply in the shared kernel."})
}
#[derive(Clone, Copy)]
struct DecisionScope<'a> {
    selected: &'a [usize],
    active: &'a [usize],
}
fn family(m: &Move) -> &'static str {
    match m {
        Move::Archive { .. } => "synthetic_archive",
        Move::Inspect { .. } => "inspect",
        Move::Compose { .. } => "compose",
    }
}
fn bounded(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 512
}
fn words(s: &str) -> usize {
    s.split_whitespace().count()
}
fn observe(spec: &Spec, book: &Notebook) -> Vec<f64> {
    spec.wants
        .iter()
        .map(|w| match &w.aim {
            Aim::ArchiveTraceability => spec
                .archive
                .as_ref()
                .map_or(0.0, |a| a.measure(&book.archive_events, true)),
            Aim::ArchiveContestability => spec
                .archive
                .as_ref()
                .map_or(0.0, |a| a.measure(&book.archive_events, false)),
            Aim::InspectNotes { notes } => {
                notes.iter().filter(|id| book.known.contains(*id)).count() as f64
                    / notes.len().max(1) as f64
            }
            Aim::IncludeNotes { notes } => {
                notes
                    .iter()
                    .filter(|id| book.known.contains(*id) && book.claims.contains(*id))
                    .count() as f64
                    / notes.len().max(1) as f64
            }
            Aim::Understand { topic } => {
                let notes: Vec<_> = spec.notes.iter().filter(|n| &n.topic == topic).collect();
                notes.iter().filter(|n| book.known.contains(&n.id)).count() as f64
                    / notes.len().max(1) as f64
            }
            Aim::Account { topic } => {
                let notes: Vec<_> = spec.notes.iter().filter(|n| &n.topic == topic).collect();
                notes
                    .iter()
                    .filter(|n| book.known.contains(&n.id) && book.claims.contains(&n.id))
                    .count() as f64
                    / notes.len().max(1) as f64
            }
            Aim::Concise { max_words } => {
                let count = words(&book.title)
                    + spec
                        .notes
                        .iter()
                        .filter(|n| book.claims.contains(&n.id))
                        .map(|n| words(&n.text))
                        .sum::<usize>();
                if book.claims.is_empty() {
                    0.0
                } else {
                    (*max_words as f64 / count.max(1) as f64).min(1.0)
                }
            }
        })
        .collect()
}
fn transition(spec: &Spec, book: &Notebook, action: &Move) -> Result<Notebook> {
    let mut next = book.clone();
    ensure!(
        spec.archive.is_some() == matches!(action, Move::Archive { .. }),
        "action outside admitted adapter"
    );
    match action {
        Move::Archive { event } => {
            next.archive_events = spec
                .archive
                .as_ref()
                .unwrap()
                .apply(&book.archive_events, event)?;
        }
        Move::Inspect { notes } => {
            ensure!(
                !notes.is_empty() && notes.len() <= 4,
                "inspect 1..4 admitted note ids"
            );
            ensure!(
                notes.iter().collect::<BTreeSet<_>>().len() == notes.len(),
                "duplicate inspection evidence"
            );
            for id in notes {
                ensure!(spec.notes.iter().any(|n| n.id == *id), "unknown note");
                next.known.insert(id.clone());
            }
        }
        Move::Compose { title, claims } => {
            ensure!(
                bounded(title) && !claims.is_empty() && claims.len() <= 16,
                "bounded title and 1..16 claim ids required"
            );
            ensure!(
                claims.iter().collect::<BTreeSet<_>>().len() == claims.len(),
                "duplicate claims"
            );
            ensure!(
                claims.iter().all(|id| book.known.contains(id)),
                "inspect evidence before using it"
            );
            next.title = title.clone();
            next.claims = claims.clone();
        }
    }
    Ok(next)
}
fn related_weight(
    spec: &Spec,
    book: &Notebook,
    action: &Move,
    prior: &Notebook,
    previous: &Move,
) -> f64 {
    if let (Move::Archive { event: a }, Move::Archive { event: b }) = (action, previous) {
        return if std::mem::discriminant(a) == std::mem::discriminant(b) {
            1.0
        } else {
            0.0
        };
    }
    if family(action) != family(previous) {
        return 0.0;
    }
    let ids = |m: &Move| match m {
        Move::Archive { event } => vec![event.challenge().to_string()],
        Move::Inspect { notes } => notes.clone(),
        Move::Compose { claims, .. } => claims.clone(),
    };
    let topics = |m: &Move| {
        ids(m)
            .iter()
            .filter_map(|id| {
                spec.notes
                    .iter()
                    .find(|n| n.id == *id)
                    .map(|n| n.topic.clone())
            })
            .collect::<BTreeSet<_>>()
    };
    let a = topics(action);
    let b = topics(previous);
    let semantic = a.intersection(&b).count() as f64 / a.union(&b).count().max(1) as f64;
    let features = |b: &Notebook, m: &Move| {
        let targets = ids(m);
        let new = targets
            .iter()
            .filter(|id| match m {
                Move::Archive { .. } => false,
                Move::Inspect { .. } => !b.known.contains(*id),
                Move::Compose { .. } => !b.claims.contains(*id),
            })
            .count();
        [
            b.known.len() as f64 / spec.notes.len() as f64,
            b.claims.len() as f64 / spec.notes.len() as f64,
            targets.len() as f64 / spec.notes.len() as f64,
            new as f64 / targets.len().max(1) as f64,
        ]
    };
    let distance = features(book, action)
        .iter()
        .zip(features(prior, previous))
        .map(|(a, b)| (a - b).abs())
        .sum::<f64>();
    semantic / (1.0 + distance)
}
/// Explicit proxy: worst-served selected desire plus mean progress, not criterion counting.
/// Current commitment has a small persistence cost until contradicted by a no-progress return.
/// A generated two-step prerequisite is valued through the same obligation kernel.
/// The adapter supplies independently bounded intermediate effects to the shared kernel.
/// The speculative second step is never dispatched or counted as an observed return.
#[cfg(test)]
fn assess(
    spec: &Spec,
    book: &Notebook,
    turns: &[Turn],
    selected: &[usize],
    belief: Option<&Direction>,
    d: &Direction,
) -> Result<Assessment> {
    assess_active(
        spec,
        book,
        turns,
        selected,
        belief,
        d,
        &(0..spec.wants.len()).collect::<Vec<_>>(),
    )
}
// A domain adapter proves only scope/relevance and first-step safety here.
// Unknown source contents are never read to invent a terminal value.
fn minimum_offer(spec: &Spec, book: &Notebook, selected: &[usize], active: &[usize]) -> usize {
    if spec.archive.is_some() {
        return 1;
    }
    if book.known.is_empty() && spec.notes.len() == 1 {
        return 1;
    }
    let before = observe(spec, book);
    let roots: BTreeSet<_> = selected
        .iter()
        .map(|i| original(spec, *i))
        .filter(|i| active.contains(i))
        .collect();
    let scope = |i: usize, n: &Note| match &spec.wants[i].aim {
        Aim::Account { topic } | Aim::Understand { topic } => &n.topic == topic,
        Aim::IncludeNotes { notes } | Aim::InspectNotes { notes } => notes.contains(&n.id),
        Aim::Concise { .. } | Aim::ArchiveTraceability | Aim::ArchiveContestability => false,
    };
    let unread = spec
        .notes
        .iter()
        .filter(|n| {
            !book.known.contains(&n.id) && roots.iter().any(|i| before[*i] < 1.0 && scope(*i, n))
        })
        .count();
    // Conservative upper bounds over every known-claim composition, not a proposed
    // action or hidden answer. If any could improve an original, retain alternatives.
    let possible = roots.iter().any(|i| {
        let upper = match &spec.wants[*i].aim {
            Aim::Account { .. } | Aim::IncludeNotes { .. } => {
                let all = spec.notes.iter().filter(|n| scope(*i, n)).count();
                spec.notes
                    .iter()
                    .filter(|n| scope(*i, n) && book.known.contains(&n.id))
                    .count() as f64
                    / all.max(1) as f64
            }
            Aim::Concise { max_words } => spec
                .notes
                .iter()
                .filter(|n| book.known.contains(&n.id))
                .map(|n| {
                    (*max_words as f64 / (1 + n.text.split_whitespace().count()) as f64).min(1.0)
                })
                .fold(0.0, f64::max),
            _ => before[*i],
        };
        upper > before[*i]
    });
    if unread == 1 && !possible {
        1
    } else {
        2
    }
}
fn validate_deferred(
    spec: &Spec,
    book: &Notebook,
    turns: &[Turn],
    selected: &[usize],
    active: &[usize],
    d: &Direction,
) -> Result<()> {
    ensure!(
        d.deferred_observation == Some(observation_revision(spec)),
        "deferred inspection observation is stale"
    );
    ensure!(
        d.continuation.is_none(),
        "deferred inspection cannot promise a terminal action"
    );
    ensure!(
        turns.len() + 2 <= spec.max_cycles,
        "deferred inspection needs remaining reconsideration budget"
    );
    let Move::Inspect { notes } = &d.action else {
        anyhow::bail!("deferred direction must inspect");
    };
    let before = observe(spec, book);
    let after = transition(spec, book, &d.action)?;
    ensure!(
        notes.iter().all(|id| !book.known.contains(id)),
        "deferred inspection must add unread evidence only"
    );
    let roots: BTreeSet<_> = d
        .supports
        .iter()
        .filter(|i| **i < spec.wants.len())
        .map(|i| original(spec, *i))
        .collect();
    let selected_roots: BTreeSet<_> = selected.iter().map(|i| original(spec, *i)).collect();
    ensure!(
        !roots.is_empty()
            && roots
                .iter()
                .all(|i| active.contains(i) && selected_roots.contains(i) && before[*i] < 1.0),
        "deferred support needs unmet selected active originals"
    );
    let relevant = |root: usize, id: &String| {
        let note = spec
            .notes
            .iter()
            .find(|n| &n.id == id)
            .expect("transition validated scope");
        match &spec.wants[root].aim {
            Aim::Understand { topic } | Aim::Account { topic } => &note.topic == topic,
            Aim::InspectNotes { notes } | Aim::IncludeNotes { notes } => notes.contains(id),
            Aim::Concise { .. } | Aim::ArchiveTraceability | Aim::ArchiveContestability => false, // brevity alone does not make arbitrary evidence relevant
        }
    };
    ensure!(
        notes
            .iter()
            .all(|id| roots.iter().any(|root| relevant(*root, id)))
            && roots
                .iter()
                .all(|root| notes.iter().any(|id| relevant(*root, id))),
        "deferred inspection evidence is irrelevant to supported originals"
    );
    let observed = observe(spec, &after);
    ensure!(
        d.anticipated == observed,
        "deferred inspection requires exact first-step forecasts, not terminal promises"
    );
    ensure!(
        active
            .iter()
            .filter(|i| original(spec, **i) == **i)
            .all(|i| observed[*i] >= before[*i]),
        "deferred inspection may not regress standing obligations"
    );
    Ok(())
}
fn assess_active(
    spec: &Spec,
    book: &Notebook,
    turns: &[Turn],
    selected: &[usize],
    belief: Option<&Direction>,
    d: &Direction,
    active: &[usize],
) -> Result<Assessment> {
    ensure!(
        d.supports.iter().all(|i| active.contains(i)),
        "inactive desire cannot support a direction"
    );
    let scope = DecisionScope { selected, active };
    if d.deferred_observation.is_some() {
        validate_deferred(spec, book, turns, selected, active, d)?;
    }
    let Some(next) = &d.continuation else {
        let mut assessed = assess_immediate(spec, book, turns, scope, belief, d, None)?;
        if d.deferred_observation.is_some() {
            let after = transition(spec, book, &d.action)?;
            assessed.score = Proxy::new("inspect_first_cost_order_only", -assessed.effort_cost);
            assessed.rejected_supports.clear();
            // Operative fallback utility has no claimed terminal gain or probability.
            // Shared diagnostics remain separately available in `shared`.
            assessed.projected_gain = Proxy::new("deferred_terminal_gain_unclaimed", 0.0);
            assessed.continuity_cost = 0.0;
            assessed.plan = Some(PlanAssessment {
                path: None,
                first_step_progress_probability: if active_progress(spec, selected, active, &observe(spec,book), &observe(spec,&after)) {1.0} else {0.0},
                newly_inspected: after.known.difference(&book.known).cloned().collect(),
                delay_cost: 0.0,
                terminal_status: "deferred; no terminal action, probability or satisfaction claim".into(),
                assessment_scope: "inspect-first fallback ordered by charged cost only; shared first-step effects retained; fresh observed-state proposal required".into(),
            });
        }
        assessed.fallback = Some(bounded_fallback(spec, book, selected, active, &assessed)?);
        return Ok(assessed);
    };
    ensure!(bounded(&d.purpose), "bounded direction purpose required");
    let before = observe(spec, book);
    ensure!(
        d.anticipated.len() == spec.wants.len()
            && d.anticipated
                .iter()
                .all(|v| v.is_finite() && (0.0..=1.0).contains(v)),
        "bounded immediate satisfaction vector required"
    );
    let intermediate = transition(spec, book, &d.action)?;
    ensure!(
        matches!(d.action, Move::Inspect { .. }) && intermediate.known.len() > book.known.len(),
        "planned prerequisite must add knowledge"
    );
    // Require an actual dependency: this is not a cheap no-op wrapped around an
    // already available action. Both transitions use the same admitted scope.
    ensure!(
        transition(spec, book, &next.action).is_err(),
        "continuation is already available; propose it directly"
    );
    transition(spec, &intermediate, &next.action)?;
    let terminal = Direction {
        deferred_observation: None,
        purpose: d.purpose.clone(),
        supports: d.supports.clone(),
        anticipated: next.anticipated.clone(),
        action: next.action.clone(),
        continuation: None,
    };
    let first_effort = match &d.action {
        Move::Inspect { notes } => notes.len() as f64 / spec.notes.len().max(1) as f64 * 0.01,
        _ => unreachable!(),
    };
    let mut ranked = assess_immediate(
        spec,
        &intermediate,
        turns,
        scope,
        belief,
        &terminal,
        Some((book, first_effort)),
    )?;
    let path = ranked
        .plan
        .take()
        .and_then(|p| p.path)
        .expect("path assessment requested");
    let delay = path.delay_cost;
    let reviewed_supports = ranked.direction.supports.clone();
    ranked.direction = d.clone();
    ranked.direction.supports = reviewed_supports;
    // Forecast/Surprise concern only the committed first step and its measurable
    // progress. Terminal estimates remain separately inspectable in `shared`.
    ranked.plan = Some(PlanAssessment {
        path: Some(path),
        first_step_progress_probability: if active_progress(spec, selected, active, &before, &observe(spec,&intermediate)) {1.0} else {0.0},
        newly_inspected: intermediate.known.difference(&book.known).cloned().collect(),
        delay_cost: delay,
        assessment_scope: "shared assesses terminal effects from the original observation; path prices intermediate risks, prerequisite effort and delay; first-dispatch forecast is separate".into(),
        terminal_status: "unresolved possibility; terminal estimate uses observed direct-action returns, never this prerequisite return".into(),
    });
    ranked.fallback = Some(bounded_fallback(spec, book, selected, active, &ranked)?);
    Ok(ranked)
}

// This adapter can independently bound its sealed local action effects. The
// fallback grants no authority and never treats a terminal plan as a return.
fn bounded_fallback(
    spec: &Spec,
    book: &Notebook,
    selected: &[usize],
    active: &[usize],
    a: &Assessment,
) -> Result<Fallback> {
    let before = observe(spec, book);
    let after_book = transition(spec, book, &a.direction.action)?;
    let after = observe(spec, &after_book);
    let no_loss = active
        .iter()
        .filter(|i| original(spec, **i) == **i)
        .all(|i| after[*i] >= before[*i]);
    let ids = preference_ids(spec)?;
    let obs: Vec<_> = spec
        .wants
        .iter()
        .enumerate()
        .map(|(i, _)| kernel::Obligation {
            id: ids[i],
            refines: (original(spec, i) != i).then(|| ids[original(spec, i)]),
            admitted: active.contains(&i),
            observed: Some(before[i]),
        })
        .collect();
    let projections: Vec<_> = active
        .iter()
        .map(|i| kernel::Projection {
            obligation: ids[*i],
            anticipated: after[*i],
            lower: after[*i],
            upper: after[*i],
        })
        .collect();
    let selected_ids: Vec<_> = selected.iter().map(|i| ids[*i]).collect();
    let proposal = kernel::Commitment {
        family: family(&a.direction.action).into(),
        supports: a.direction.supports.iter().map(|i| ids[*i]).collect(),
    };
    let exact = kernel::assess(kernel::Request {
        obligations: &obs,
        selected: &selected_ids,
        proposal: &proposal,
        previous: None,
        projections: &projections,
        probability: 1.0,
        effort: a.effort_cost,
        authorized: true,
    })
    .map_err(anyhow::Error::msg)?;
    let exact_net = exact.score - a.continuity_cost;
    // Sealed reversible composition has exact local effects. This is the existing
    // conserved valuation, not statistical certainty or additional permission.
    let tradeoff = matches!(a.direction.action, Move::Compose { .. })
        && exact_net > 0.0
        && selected
            .iter()
            .map(|i| original(spec, *i))
            .any(|i| after[i] > before[i]);
    let terminal_progress = if let Some(next) = &a.direction.continuation {
        let terminal = observe(spec, &transition(spec, &after_book, &next.action)?);
        active_progress(spec, selected, active, &before, &terminal)
    } else {
        false
    };
    let information = matches!(a.direction.action, Move::Inspect { .. })
        && after_book.known.len() > book.known.len()
        && (active_progress(spec, selected, active, &before, &after) || terminal_progress);
    let progress = active_progress(spec, selected, active, &before, &after);
    let deferred = a.direction.deferred_observation.is_some();
    let eligible = if deferred {
        no_loss && after_book.known.len() > book.known.len()
    } else {
        ((no_loss && (information || progress)) || tradeoff) && a.score.value > 0.0
    };
    Ok(Fallback {
        exact_first_step_net: a.plan.is_none().then_some(exact_net),
        exact_standing_loss: Some(exact.standing_loss),
        policy: if deferred { "bounded-inspect-first@v1" } else { "bounded-mechanical-evidence@v2" }.into(), eligible,
        reason: if deferred {
            "fallback only when no ordinary productive option qualifies; relevant unread evidence, no standing loss, cost charged; terminal value unknown and fresh selection required"
        } else if a.score.value <= 0.0 {
            "abstain: estimated benefit does not clear action and continuity cost"
        } else if tradeoff && !no_loss {
            "allow reversible local tradeoff: exact shared net value is positive after standing loss and cost; regressed desires remain unmet"
        } else if !no_loss { "abstain: independently bounded first step regresses an active standing obligation" }
        else if information { "allow bounded information gathering; no first-step standing loss; any continuation needs fresh selection" }
        else if progress { "allow reversible local action with independently bounded predicate progress and no standing loss" }
        else { "abstain: neither relevant information gain nor independently bounded progress" }.into(),
    })
}
fn eligible_indices(assessments: &[Assessment]) -> Vec<usize> {
    let ordinary = assessments.iter().any(|a| {
        a.direction.deferred_observation.is_none()
            && a.fallback.as_ref().is_some_and(|f| f.eligible)
    });
    assessments
        .iter()
        .enumerate()
        .filter(|(_, a)| {
            a.fallback.as_ref().is_some_and(|f| f.eligible)
                && (!ordinary || a.direction.deferred_observation.is_none())
        })
        .map(|(i, _)| i)
        .collect()
}
fn choose_supported(assessments: &[Assessment]) -> Option<usize> {
    assessments
        .iter()
        .enumerate()
        .filter(|(i, _)| eligible_indices(assessments).contains(i))
        .max_by(|a, b| {
            a.1.score
                .value
                .total_cmp(&b.1.score.value)
                .then(b.0.cmp(&a.0))
        })
        .map(|(i, _)| i)
}

fn assess_immediate(
    spec: &Spec,
    book: &Notebook,
    turns: &[Turn],
    scope: DecisionScope<'_>,
    belief: Option<&Direction>,
    d: &Direction,
    planning: Option<(&Notebook, f64)>,
) -> Result<Assessment> {
    let selected = scope.selected;
    let active = scope.active;
    let ids = preference_ids(spec)?;
    ensure!(
        bounded(&d.purpose)
            && !d.supports.is_empty()
            && d.supports.len() <= spec.wants.len()
            && d.supports.iter().collect::<BTreeSet<_>>().len() == d.supports.len()
            && d.supports.iter().all(|i| *i < spec.wants.len())
            && d.supports.iter().any(|i| selected.contains(i)),
        "direction must name selected desire support"
    );
    let action_observation = observe(spec, &transition(spec, book, &d.action)?);
    ensure!(
        d.anticipated.len() == spec.wants.len()
            && d.anticipated
                .iter()
                .all(|v| v.is_finite() && (0.0..=1.0).contains(v)),
        "bounded anticipated satisfaction vector required"
    );
    let action_before = observe(spec, book);
    let before = observe(spec, planning.map_or(book, |(baseline, _)| baseline));
    let raw_projection = d.anticipated.clone();
    let proposed_supports = d.supports.clone();
    let rejected_supports: Vec<_> = d
        .supports
        .iter()
        .copied()
        .filter(|i| raw_projection[*i] <= before[*i])
        .collect();
    let mut reviewed = d.clone();
    if d.deferred_observation.is_none() {
        reviewed
            .supports
            .retain(|i| raw_projection[*i] > before[*i]);
    }
    ensure!(
        d.deferred_observation.is_some() || reviewed.supports.iter().any(|i| selected.contains(i)),
        "no selected support predicts improvement; rejected links: {:?}",
        rejected_supports
    );
    reviewed.supports.sort_unstable();
    let d = &reviewed;
    let empty = Notebook::default();
    let points: Vec<_> = turns
        .iter()
        .enumerate()
        // Learn only the action actually dispatched. related_weight excludes
        // different families, so an inspection never labels its future composition.
        .filter_map(|(i, t)| {
            let prior = if let Some(before) = &t.before_notebook {
                before
            } else if i == 0 {
                &empty
            } else {
                &turns[i - 1].notebook
            };
            let weight = related_weight(
                spec,
                book,
                &d.action,
                prior,
                &t.offered[t.chosen].direction.action,
            );
            (weight > 0.0).then_some((t, weight))
        })
        .collect();
    let returns = |dimension: Option<usize>| {
        points
            .iter()
            .map(|(t, w)| kernel::Return {
                evidence: t.commitment,
                weight: *w,
                success: (support_set(spec, &historical_active(&ids, t))
                    == support_set(spec, active))
                .then_some(return_progress(spec, t)),
                predicted: dimension
                    .filter(|_| t.observation_revision == Some(observation_revision(spec)))
                    .and_then(|i| t.offered[t.chosen].direction.anticipated.get(i).copied()),
                observed: dimension.and_then(|i| t.after.get(i).copied()),
            })
            .collect::<Vec<_>>()
    };
    let p = kernel::expectation(&returns(None))
        .map_err(anyhow::Error::msg)?
        .probability;
    let calibration = (0..spec.wants.len())
        .map(|i| {
            kernel::expectation(&returns(Some(i)))
                .map(|e| e.residual)
                .map_err(anyhow::Error::msg)
        })
        .collect::<Result<Vec<_>>>()?;
    let mut projected: Vec<_> = raw_projection
        .iter()
        .zip(&calibration)
        .enumerate()
        .map(|(i, (p, r))| {
            let calibrated = (p - r).clamp(0.0, 1.0);
            // Operation invariants bound learned predictions, independently of model claims.
            match (&d.action, &spec.wants[i].aim) {
                (Move::Inspect { .. }, Aim::Understand { .. } | Aim::InspectNotes { .. }) => {
                    calibrated.max(action_before[i])
                }
                (Move::Inspect { .. }, _)
                | (Move::Compose { .. }, Aim::Understand { .. } | Aim::InspectNotes { .. }) => {
                    action_before[i]
                }
                _ => calibrated,
            }
        })
        .collect();
    // The sealed action supplies an independent upper bound for every predicate,
    // including unchanged and increasing coverage. Calibration cannot manufacture
    // effects beyond that capacity. Lower forecasts remain uncertain; this bound
    // neither records a return nor changes the generator's frozen raw forecast.
    for (predicted, capacity) in projected.iter_mut().zip(&action_observation) {
        *predicted = predicted.min(*capacity);
    }
    let obligations = spec
        .wants
        .iter()
        .enumerate()
        .map(|(i, _)| kernel::Obligation {
            id: ids[i],
            refines: (original(spec, i) != i).then(|| ids[original(spec, i)]),
            admitted: active.contains(&i),
            observed: Some(before[i]),
        })
        .collect::<Vec<_>>();
    let projections = obligations
        .iter()
        .enumerate()
        .filter(|(i, _)| active.contains(i))
        .map(|(i, o)| kernel::Projection {
            obligation: o.id,
            anticipated: projected[i],
            // This sealed adapter's predicates are deterministic transition measurements.
            // Both bounds apply to direct and planned effects: historical residuals
            // may neither manufacture progress nor suppress independently known effects.
            // These remain simulated projections, not dispatched returns or source truth.
            lower: action_observation[i],
            upper: action_observation[i],
        })
        .collect::<Vec<_>>();
    let commitment = |d: &Direction| kernel::Commitment {
        family: family(&d.action).into(),
        supports: d
            .supports
            .iter()
            .filter(|i| active.contains(i))
            .map(|i| ids[*i])
            .collect(),
    };
    let proposal = commitment(d);
    let previous = belief.map(commitment).filter(|p| !p.supports.is_empty());
    let effort = match &d.action {
        Move::Archive { .. } => 1.0 / spec.max_cycles as f64 * 0.01,
        Move::Inspect { notes } => notes.len() as f64 / spec.notes.len().max(1) as f64 * 0.01,
        Move::Compose { claims, .. } => {
            claims.len() as f64 / spec.notes.len().max(1) as f64 * 0.005
        }
    };
    let selected_ids = selected.iter().map(|i| ids[*i]).collect::<Vec<_>>();
    let request = kernel::Request {
        obligations: &obligations,
        selected: &selected_ids,
        proposal: &proposal,
        previous: previous.as_ref(),
        projections: &projections,
        probability: p,
        effort,
        authorized: true, // transition validated the sealed notebook action above
    };
    let (ranked, path) = if let Some((_, first_effort)) = planning {
        let intermediate = obligations
            .iter()
            .enumerate()
            .filter(|(i, _)| active.contains(i))
            .map(|(i, o)| kernel::Projection {
                obligation: o.id,
                anticipated: action_before[i],
                lower: action_before[i],
                upper: action_before[i],
            })
            .collect::<Vec<_>>();
        let path = kernel::assess_path(kernel::PathRequest {
            terminal: request,
            intermediate: &intermediate,
            prerequisite_effort: first_effort,
            delay_fraction: 0.1,
        })
        .map_err(anyhow::Error::msg)?;
        (path.terminal.clone(), Some(path))
    } else {
        (kernel::assess(request).map_err(anyhow::Error::msg)?, None)
    };
    // Keep the public trace aligned with the projections actually scored.
    for (i, id) in ids.iter().enumerate() {
        if let Some(value) = ranked.projected.get(id) {
            projected[i] = *value;
        }
    }
    let score = path.as_ref().map_or(ranked.score, |p| p.score);
    let effort = effort
        + path.as_ref().map_or(0.0, |p| {
            p.prerequisite_effort + p.delay_cost + p.additional_risk_cost
        });
    let gain = ranked.gain;
    let continuity = ranked.continuity_cost;
    let standing_loss = ranked.standing_loss;
    Ok(Assessment {
        expectation_support: Some(ExpectationSupport {
            event: "notebook-progress@v1: improvement of selected active originals without loss to active originals".into(),
            compatible_dispatches: returns(None).iter().filter(|r| r.success.is_some()).count(),
            effective_dispatch_weight: returns(None).iter().filter(|r| r.success.is_some()).map(|r| r.weight).sum(),
            weighted_receipts: returns(None).iter().filter(|r| r.success.is_some()).map(|r| (r.evidence,r.weight)).collect(),
            independent_mature_episodes: 0,
            calibration_status: "unvalidated; smoothed within-run progress estimate, not Tier 1 calibration".into(),
            applicability: "same canonical active-original scope; action-family/context similarity weights; residuals require identical observation revision; no validated feature-range coverage".into(),
            uncertainty: "No calibrated confidence interval. Repeated turns share a run and cannot establish independent episode support. PRD Tier 1 requires 150 mature episodes per event, at least 30 per family, and calibration validation.".into(),
        }),
        fallback: None,
        plan: path.map(|path| PlanAssessment {
            path: Some(path),
            first_step_progress_probability: 0.0,
            newly_inspected: vec![],
            delay_cost: 0.0,
            assessment_scope: String::new(),
            terminal_status: String::new(),
        }),
        shared: Some(ranked),
        kernel: KERNEL.into(),
        direction: d.clone(),
        proposed_supports,
        rejected_supports,
        prediction: Proxy::new("related_context_progress_rate", p),
        prediction_evidence: points
            .iter()
            .filter(|(t, _)| {
                support_set(spec, &historical_active(&ids, t)) == support_set(spec, active)
            })
            .map(|(t, _)| t.commitment)
            .collect(),
        calibration_evidence: points
            .iter()
            .filter(|(t, _)| t.observation_revision == Some(observation_revision(spec)))
            .map(|(t, _)| t.commitment)
            .collect(),
        projection_basis: Some("sealed-notebook-transition@v1: exact simulated predicate bounds; raw forecasts/residuals preserved; historical progress weight is not transition uncertainty".into()),
        projected,
        score: Proxy::new("balanced_progress", score),
        continuity_cost: continuity,
        standing_loss,
        projected_gain: Proxy::new("balanced_gain", gain),
        effort_cost: effort,
        calibration: Proxy::new("related_context_prediction_residual", calibration),
    })
}
impl Engine {
    fn deliberation_record(&self, r: &Record) -> Result<EventId> {
        let digest = self.blob(&serde_json::to_vec(r)?)?;
        self.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    digest,
                    source: SOURCE.into(),
                },
            },
            if matches!(
                r,
                Record::EvidenceUpdated { .. }
                    | Record::SelectionTuned { .. }
                    | Record::Relational {
                        entry: relational::Entry::Status { .. }
                    }
            ) {
                Source::Operator
            } else {
                Source::Rules
            },
        )
    }
    fn deliberation_records(&self) -> Result<Vec<Record>> {
        let mut records = vec![];
        let mut previous = None;
        for (_, e) in self.events()? {
            ensure!(
                e.schema_version == Event::SCHEMA_VERSION
                    && e.parents == previous.into_iter().collect::<Vec<_>>(),
                "invalid direction event chain"
            );
            ensure!(
                Event::new(
                    e.task,
                    e.parents.clone(),
                    e.source,
                    e.kind.clone(),
                    e.recorded_at.clone()
                )
                .id == e.id,
                "changed direction event"
            );
            previous = Some(e.id);
            if let EventKind::Observed {
                observation: Observation::Evidence { digest, source },
            } = e.kind
            {
                if source == SOURCE {
                    let bytes = fs::read(self.directory.join("cas").join(digest.to_hex()))?;
                    ensure!(Hash::of(&[&bytes]) == digest, "changed direction evidence");
                    let record: Record = serde_json::from_slice(&bytes)?;
                    if let Record::SelectionTuned {
                        quiet_route,
                        reason,
                        ..
                    } = &record
                    {
                        ensure!(
                            e.source == Source::Operator
                                && quiet_route.is_finite()
                                && (0.0..=1.0).contains(quiet_route)
                                && bounded(reason),
                            "invalid operator selection tuning evidence"
                        );
                    }
                    if let Record::Relational { entry } = &record {
                        ensure!(
                            e.source
                                == if matches!(entry, relational::Entry::Status { .. }) {
                                    Source::Operator
                                } else {
                                    Source::Rules
                                },
                            "invalid relational provenance"
                        );
                    }
                    records.push(record);
                }
            }
        }
        Ok(records)
    }
    fn deliberation_state(&self) -> Result<(Spec, Vec<PreferenceId>, Notebook, Vec<Turn>)> {
        let records = self.deliberation_records()?;
        let Some(Record::Initialized { spec, preferences }) = records.first() else {
            bail!("initialize deliberation first")
        };
        let mut spec = spec.clone();
        let mut preferences = preferences.clone();
        let mut book = Notebook::default();
        let mut turns = vec![];
        for record in &records {
            match record {
                Record::Returned { turn } => {
                    let mut turn = turn.as_ref().clone();
                    // Legacy receipts inherit the revision at their historical position,
                    // never the latest observation. Persisted bytes are untouched.
                    turn.observation_revision
                        .get_or_insert(observation_revision(&spec));
                    turn.before_notebook.get_or_insert(book.clone());
                    book = turn.notebook.clone();
                    turns.push(turn);
                }
                Record::EvidenceUpdated {
                    from, to, notes, ..
                } => {
                    ensure!(
                        *from == observation_revision(&spec),
                        "observation revision chain mismatch"
                    );
                    replace_evidence(&mut spec, &mut book, notes);
                    ensure!(
                        *to == observation_revision(&spec),
                        "observation revision digest mismatch"
                    );
                }
                Record::Refined { admitted, .. } => {
                    for (want, id) in admitted {
                        spec.wants.push(want.clone());
                        preferences.push(*id);
                    }
                }
                Record::Initialized { .. }
                | Record::SelectionTuned { .. }
                | Record::Relational { .. } => {}
            }
        }
        Ok((spec, preferences, book, turns))
    }
    pub fn update_deliberation_observation(
        &self,
        update: ObservationUpdate,
    ) -> Result<serde_json::Value> {
        self.write(|| {
            ensure!(
                update.state_root == self.store.head_root()?,
                "stale observation update"
            );
            ensure!(
                bounded(&update.reason) && !update.notes.is_empty() && update.notes.len() <= 32,
                "bounded observation reason and notes required"
            );
            ensure!(
                update
                    .notes
                    .iter()
                    .map(|n| &n.id)
                    .collect::<BTreeSet<_>>()
                    .len()
                    == update.notes.len(),
                "duplicate observation ids"
            );
            let (mut spec, _, mut book, _) = self.deliberation_state()?;
            for note in &update.notes {
                ensure!(bounded(&note.text), "bounded observation text required");
                ensure!(
                    spec.notes
                        .iter()
                        .any(|n| n.id == note.id && n.topic == note.topic),
                    "observation can revise only admitted note ids within their original topic"
                );
            }
            let from = observation_revision(&spec);
            let before = observe(&spec, &book);
            replace_evidence(&mut spec, &mut book, &update.notes);
            let to = observation_revision(&spec);
            if from != to {
                let after = observe(&spec, &book);
                self.deliberation_record(&Record::EvidenceUpdated {
                    from,
                    to,
                    notes: update.notes,
                    reason: update.reason,
                    before,
                    after,
                })?;
            }
            self.deliberation_request()
        })
    }
    pub fn deliberation_initialize(&self, spec: Spec) -> Result<serde_json::Value> {
        ensure!(
            (2..=8).contains(&spec.wants.len())
                && (spec.archive.is_some() && spec.notes.is_empty()
                    || (1..=32).contains(&spec.notes.len()))
                && (1..=32).contains(&spec.max_cycles),
            "bounded wants/notes/cycles required"
        );
        ensure!(
            spec.wants.iter().all(|w| bounded(&w.text))
                && spec
                    .notes
                    .iter()
                    .all(|n| bounded(&n.id) && bounded(&n.topic) && bounded(&n.text)),
            "short typed inputs required"
        );
        ensure!(
            spec.notes
                .iter()
                .map(|n| &n.id)
                .collect::<BTreeSet<_>>()
                .len()
                == spec.notes.len(),
            "unique note ids"
        );
        ensure!(
            spec.attention_limit > 0 && usize::from(spec.attention_limit) <= spec.wants.len(),
            "bounded attention limit"
        );
        ensure!(
            spec.wants
                .iter()
                .map(|w| (&w.text, &w.parents))
                .collect::<BTreeSet<_>>()
                .len()
                == spec.wants.len(),
            "distinct standing preference identities required"
        );
        for (i, w) in spec.wants.iter().enumerate() {
            ensure!(
                w.parents.iter().all(|p| *p < i),
                "parents must name earlier standing desires"
            );
        }
        if let Some(world) = &spec.archive {
            ensure!(spec.notes.is_empty(), "archive excludes notebook sources");
            world.validate()?;
            ensure!(
                spec.wants.iter().all(|w| matches!(
                    w.aim,
                    Aim::ArchiveTraceability | Aim::ArchiveContestability
                )),
                "archive requires explicit partial predicates"
            );
        }
        for want in &spec.wants {
            match &want.aim {
                Aim::ArchiveTraceability | Aim::ArchiveContestability => {
                    ensure!(spec.archive.is_some(), "archive missing")
                }
                Aim::Understand { topic } | Aim::Account { topic } => ensure!(
                    spec.notes.iter().any(|n| &n.topic == topic),
                    "desire needs evidence topic"
                ),
                Aim::InspectNotes { .. } | Aim::IncludeNotes { .. } => {
                    bail!("initialize standing topics first; note refinements require a return")
                }
                Aim::Concise { max_words } => {
                    ensure!((1..=4096).contains(max_words), "bounded word target")
                }
            }
        }
        self.write(|| {
            ensure!(self.events()?.is_empty(), "directory already initialized");
            let opened = self.append(
                EventKind::TaskOpened { task: Self::task() },
                Source::Operator,
            )?;
            let mut preferences = vec![];
            for w in &spec.wants {
                let c = Filter::contrast_from_criteria(opened, vec![], vec![opened]);
                let parents = w
                    .parents
                    .iter()
                    .map(|i| preferences[*i])
                    .collect::<Vec<_>>();
                self.expand(&c, &w.text, parents.clone(), Source::Operator)?;
                preferences.push(
                    Preference::new(
                        &w.text,
                        Scope::Task(Self::task()),
                        parents,
                        Source::Operator,
                    )?
                    .id,
                );
            }
            self.deliberation_record(&Record::Initialized { spec, preferences })?;
            self.deliberation_request()
        })
    }
    pub fn deliberation_request(&self) -> Result<serde_json::Value> {
        let (spec, preferences, book, turns) = self.deliberation_state()?;
        let snap = Snapshot {
            task: Self::task(),
            version: self.events()?.len() as u64,
            root: self.store.head_root()?,
            open_criteria: vec![],
            missing_evidence: vec![],
        };
        let actual = observe(&spec, &book);
        let mut ledger = Ledger::new(&self.store).collective()?;
        let records = self.deliberation_records()?;
        let mut admitted_refinements = vec![];
        let mut refinement_reviews = vec![];
        for record in &records {
            if let Record::Refined {
                from,
                admitted,
                rejected,
            } = record
            {
                refinement_reviews.push(serde_json::json!({"return_commitment":from,"admitted":admitted.len(),"unsupported":rejected}));
                for (_, id) in admitted {
                    if let Some(entry) = ledger.entries.iter_mut().find(|e| e.preference.id == *id)
                    {
                        if entry.status.is_none() {
                            // Notebook entailment review admits this generated preference
                            // locally. The generic ledger may retain None and ignore a
                            // later withdrawal, but its operator history is authoritative.
                            entry.status = Some(
                                entry
                                    .history
                                    .iter()
                                    .rev()
                                    .find(|(_, by)| *by == Source::Operator)
                                    .map(|(status, _)| status.clone())
                                    .unwrap_or(PreferenceStatus::Open),
                            );
                        }
                    }
                    admitted_refinements.push(serde_json::json!({"preference":id,"return_commitment":from,"review":"bounded notebook entailment; text semantics remain unverified; no authority change"}));
                }
            }
        }
        let admitted = |i: usize| {
            ledger
                .entries
                .iter()
                .find(|e| e.preference.id == preferences[i])
                .is_some_and(|e| {
                    matches!(
                        e.status,
                        None | Some(PreferenceStatus::Open | PreferenceStatus::Fulfilled)
                    )
                })
        };
        let active: Vec<usize> = (0..spec.wants.len())
            .filter(|i| {
                let mut at = *i;
                loop {
                    if !admitted(at) {
                        return false;
                    }
                    let root = original(&spec, at);
                    if root == at {
                        return true;
                    }
                    at = spec.wants[at].parents[0];
                }
            })
            .collect();
        let refinement_needed = spec.archive.is_none()
            && turns.last().is_some_and(|t| {
                t.observation_revision == Some(observation_revision(&spec))
                    && t.contrast.iter().any(|i| active.contains(i))
                    && spec.wants.len() < 16
                    && !records
                        .iter()
                        .any(|r| matches!(r,Record::Refined {from,..} if *from==t.commitment))
            });
        // Read-only eligibility projection: a currently satisfied standing want needs no direction.
        // Stored admission and lineage remain unchanged; loss of satisfaction restores eligibility.
        for entry in &mut ledger.entries {
            if !matches!(
                entry.status,
                None | Some(PreferenceStatus::Open | PreferenceStatus::Fulfilled)
            ) {
                continue; // observation does not reverse withdrawal or scope/admission decisions
            }
            if preferences
                .iter()
                .position(|p| *p == entry.preference.id)
                .is_some_and(|i| actual[i] >= 1.0)
            {
                entry.status = Some(PreferenceStatus::Fulfilled);
            } else if entry.status == Some(PreferenceStatus::Fulfilled)
                && preferences.contains(&entry.preference.id)
            {
                entry.status = Some(PreferenceStatus::Open);
            }
        }
        let (quiet_route, selection_seed) = records
            .iter()
            .rev()
            .find_map(|r| {
                if let Record::SelectionTuned {
                    quiet_route, seed, ..
                } = r
                {
                    Some((*quiet_route, *seed))
                } else {
                    None
                }
            })
            .unwrap_or((0.0, 0));
        let tune = Tune {
            intuition_max: spec.attention_limit,
            quiet_route,
            flow_horizon: 2,
            ..Tune::default()
        };
        ledger.entries.retain(|entry| {
            preferences
                .iter()
                .position(|p| *p == entry.preference.id)
                .is_some_and(|i| original(&spec, i) == i)
        });
        let selection = select(
            &ledger,
            &snap,
            &tune,
            &SelectionContext {
                family: "evidence-notebook".into(),
                repository: "local-only".into(),
                minimum_depth: 1,
            },
            selection_seed,
        )
        .map_err(anyhow::Error::msg)?;
        let mut selected: Vec<_> = selection
            .selected
            .iter()
            .filter_map(|p| preferences.iter().position(|v| v == p))
            .collect();
        let selected_originals = selected.clone();
        selected.extend((0..spec.wants.len()).filter(|i| {
            active.contains(i)
                && original(&spec, *i) != *i
                && actual[*i] < 1.0
                && selected_originals.contains(&original(&spec, *i))
        }));
        let tensions:Vec<_>=selected.iter().filter(|i|actual[**i]<1.0).map(|i|serde_json::json!({"desire":i,"text":spec.wants[*i].text,"observed":actual[*i],"against_commitment":turns.last().map(|t|&t.offered[t.chosen].direction.purpose),"evidence":"notebook observer; unsupported text is not counted"})).collect();
        let notes:Vec<_>=spec.notes.iter().map(|n|serde_json::json!({"id":n.id,"topic":n.topic,"text":if book.known.contains(&n.id){Some(&n.text)}else{None}})).collect();
        let policy_active = self.decision_policy()?.is_some();
        let learning_audit=turns.last().filter(|t|!policy_active && t.observation_revision == Some(observation_revision(&spec)) && !t.selected.is_empty() && t.offered.iter().all(|a|a.kernel==KERNEL)).map(|last| -> Result<serde_json::Value> {
            let mut prior_spec=spec.clone(); prior_spec.wants.truncate(last.before.len());
            let before=last.before_notebook.clone().unwrap_or_default();
            let belief=current_commitment(&spec,&turns[..turns.len().saturating_sub(1)]);
            let without:Vec<_>=last.offered.iter().map(|a|assess_active(&prior_spec,&before,&[],&last.selected,belief,&a.direction,&historical_active(&preferences,last))).collect::<Result<_>>()?;
            let baseline=choose_supported(&without);
            Ok(serde_json::json!({"control":"identical observation, selected desires, offered directions and belief; only learned returns removed","chosen":last.chosen,"chosen_without_learning":baseline,"scores":last.offered.iter().map(|a|a.score.value).collect::<Vec<_>>(),"scores_without_learning":without.iter().map(|a|a.score.value).collect::<Vec<_>>(),"evidence":last.offered.iter().map(|a|&a.prediction_evidence).collect::<Vec<_>>(),"calibration_evidence":last.offered.iter().map(|a|&a.calibration_evidence).collect::<Vec<_>>()}))
        }).transpose()?;
        let active_originals: Vec<_> = obligations(&spec)
            .into_iter()
            .filter(|i| active.contains(i))
            .collect();
        let done = active_originals.iter().all(|i| actual[*i] >= 1.0);
        let relational = self.relational_view(&spec, &preferences, &turns, &active)?;
        Ok(
            serde_json::json!({"observation_revision":observation_revision(&spec),"observation_updates":records.iter().filter_map(|r| if let Record::EvidenceUpdated{from,to,reason,before,after,..}=r {Some(serde_json::json!({"from":from,"to":to,"reason":reason,"before":before,"after":after,"source":"operator observation; no new authority"}))} else {None}).collect::<Vec<_>>(),"archive":spec.archive.as_ref().map(|a|a.view(&book.archive_events)),"relational_request":relational,"relational_guidance":relational["guidance"],"expectation_support_policy":"Unvalidated related-turn estimates rank only bounded fallback-eligible actions; no mature independent episode support is claimed. If none qualifies, abstain without changing desires or authority.","expectation_revision_policy":"Historical progress rates retain coarse related-context evidence; forecast residuals transfer only within the same observation revision.","learning_audit":learning_audit,"semantic_status":"unverified: source assertions and generated explanations are not established facts","refinement_reviews":refinement_reviews,"refinement_needed":refinement_needed,"refinement_return":turns.last().map(|t|t.commitment),"operational_refinements":admitted_refinements,"measurement_notice":if spec.archive.is_some(){"Evaluator-authored simulation. Fixed challenge denominator; withdrawn contributes zero. Contestability is registered fraction; traceability averages registration, routing and substantive response. Rich desire fulfillment remains unknown."}else{"Coverage of admitted note IDs and word budget only; source truth and qualitative understanding are unverified."},"state_root":snap.root,"forecast_contract":"identified-obligations@v1","minimum_directions":minimum_offer(&spec,&book,&selected,&active),"active_desires":active,"active_obligation_ids":active.iter().map(|i|preferences[*i]).collect::<Vec<_>>(),"obligation_ids":preferences,"obligation_roots":(0..spec.wants.len()).map(|i|original(&spec,i)).collect::<Vec<_>>(),"standing_desires":spec.wants,"selected_desires":selected,"selected_desires_role":"original attention plus generated decomposition hints; only canonical original obligations receive decision weight","selection":selection,"observation":actual,"notebook":book,"account":spec.notes.iter().filter(|n|book.claims.contains(&n.id)).map(|n|&n.text).collect::<Vec<_>>(),"available_notes":notes,"tensions":tensions,"recent_returns":turns.iter().rev().take(4).collect::<Vec<_>>(),"cycles":turns.len(),"status":if active_originals.is_empty() {"inactive"} else if done {"satisfied"} else if turns.len()>=spec.max_cycles {"budget_exhausted"} else {"deliberate"},"authority":if spec.archive.is_some(){"Only admitted synthetic archive events. No real people, external delivery, commands, paths or new response evidence."}else{"Only inspect admitted note IDs or compose a local evidence account from inspected IDs. No commands, paths or outside actions."},"planning_contract":if spec.archive.is_some(){"Single admitted synthetic event per commitment. No terminal plan or automatic continuation; reconsider from the observed append-only return."}else{"Explicit deferred_observation may bind an inspect-first direction to the current observation revision with no terminal action. Relevant unread evidence for unmet originals, exact first-step forecasts, no standing loss and remaining reconsideration budget are required. This cost-ordered fallback is used only when no ordinary productive option qualifies; terminal value remains unknown. Optional continuation proposes one dependent future action. Knowledge acquisition may affect standing predicates; shared path assessment prices both steps and intermediate standing risk, only first executes, and a fresh proposal is required after return. Future satisfaction is not observed evidence."},"constraint_status":constraint_status(&spec,&book,&active),"commitment_review":commitment_review(&spec,&turns),"decision_policy":self.decision_policy()?,"projection_policy":if spec.archive.is_some(){"sealed-synthetic-archive-transition@v1: exact partial predicate bounds, no semantic or social fulfillment"}else{"sealed-notebook-transition@v1: independently simulated exact predicate bounds override residuals; raw forecasts and historical progress weights remain distinct from observed returns"},"prediction_rule":"original-obligation attention and reward conserved across refinements; expected worst-served plus mean progress minus all standing losses, discounted by learned effectiveness and set-based continuity; engineering proxies"}),
        )
    }
    /// Operator-only attention setting; it is not a desire, forecast or capability grant.
    pub fn update_selection_tune(&self, update: SelectionTuneUpdate) -> Result<serde_json::Value> {
        self.write(|| {
            ensure!(
                update.state_root == self.store.head_root()?,
                "stale selection tune update"
            );
            ensure!(
                update.quiet_route.is_finite() && (0.0..=1.0).contains(&update.quiet_route),
                "quiet-route probability must be finite and within [0,1]"
            );
            ensure!(
                bounded(&update.reason),
                "bounded operator tuning reason required"
            );
            self.deliberation_state()?;
            self.deliberation_record(&Record::SelectionTuned {
                quiet_route: update.quiet_route,
                seed: update.seed,
                reason: update.reason,
            })?;
            self.deliberation_request()
        })
    }
    pub fn update_desire_status(&self, update: DesireStatusUpdate) -> Result<serde_json::Value> {
        self.write(|| {
            ensure!(
                update.state_root == self.store.head_root()?,
                "stale desire status update"
            );
            ensure!(
                matches!(
                    update.status,
                    PreferenceStatus::Open
                        | PreferenceStatus::Withdrawn
                        | PreferenceStatus::OutOfScope
                ),
                "operator status must be Open, Withdrawn or OutOfScope; satisfaction is observed"
            );
            let (_, ids, _, _) = self.deliberation_state()?;
            ensure!(ids.contains(&update.preference), "unknown notebook desire");
            self.append(
                EventKind::StatusSet {
                    preference: update.preference,
                    status: update.status,
                    by: Source::Operator,
                },
                Source::Operator,
            )?;
            self.deliberation_request()
        })
    }
    pub fn refine_directions(&self, offer: RefinementOffer) -> Result<serde_json::Value> {
        self.write(|| {
            ensure!(offer.state_root==self.store.head_root()?,"stale refinement offer");
            ensure!(offer.refinements.len()<=2,"at most two refinements per return");
            let (spec,prefs,book,turns)=self.deliberation_state()?;
            let last=turns.last().ok_or_else(||anyhow::anyhow!("refinement needs observed return"))?;
            ensure!(last.commitment==offer.return_commitment,"refinement must cite latest observed return");
            ensure!(self.deliberation_request()?["refinement_needed"]==true,"return already reviewed or no open contrast");
            let active:Vec<usize>=serde_json::from_value(self.deliberation_request()?["active_desires"].clone())?;
            let mut admitted=vec![];let mut rejected=vec![];
            for r in &offer.refinements {
                let reviewed=(|| -> Result<Want> {
                    ensure!(bounded(&r.text)&&!r.notes.is_empty()&&r.notes.len()<=16,"bounded refinement text and evidence required");
                    ensure!(spec.wants.len()+admitted.len()<16,"refinement budget exhausted");
                    ensure!(active.contains(&r.parent),"inactive parent cannot admit refinement");
                    ensure!(last.contrast.contains(&r.parent),"parent must be an unmet criterion in cited return");
                    let parent=spec.wants.get(r.parent).ok_or_else(||anyhow::anyhow!("unknown parent"))?;
                    ensure!(r.notes.iter().collect::<BTreeSet<_>>().len()==r.notes.len(),"duplicate refinement evidence");
                    let (allowed,include): (Vec<String>,bool)=match &parent.aim {
                        Aim::Understand {topic} => (spec.notes.iter().filter(|n|&n.topic==topic).map(|n|n.id.clone()).collect(),false),
                        Aim::Account {topic} => (spec.notes.iter().filter(|n|&n.topic==topic).map(|n|n.id.clone()).collect(),true),
                        Aim::InspectNotes {notes} => (notes.clone(),false),
                        Aim::IncludeNotes {notes} => (notes.clone(),true),
                        _ => bail!("no mechanically supported refinement for this parent; remains unresolved"),
                    };
                    ensure!(r.notes.iter().all(|id|allowed.contains(id)&&if include {!book.claims.contains(id)}else{!book.known.contains(id)}),"evidence must be a currently unmet part of the parent; no new claims or permissions");
                    let aim=if include {Aim::IncludeNotes {notes:r.notes.clone()}} else {Aim::InspectNotes {notes:r.notes.clone()}};
                    ensure!(!spec.wants.iter().chain(admitted.iter().map(|(w,_)|w)).any(|w|aim_key(&w.aim)==aim_key(&aim)),"equivalent subgoal already exists");
                    let identity=Preference::new(&r.text,Scope::Task(Self::task()),vec![prefs[r.parent]],Source::Generator)?.id;
                    ensure!(!prefs.contains(&identity)&&!admitted.iter().any(|(_,id)|*id==identity),"refinement identity already exists; cannot change its predicate");
                    Ok(Want {text:r.text.clone(),aim,parents:vec![r.parent]})
                })();
                match reviewed {
                    Err(error)=>rejected.push(error.to_string()),
                    Ok(want)=>{
                        let contrast=last.return_contrast.as_ref().ok_or_else(||anyhow::anyhow!("legacy return lacks refinement provenance; observe a new return first"))?;
                        let parent=vec![prefs[r.parent]];
                        self.expand(contrast,&want.text,parent.clone(),Source::Generator)?;
                        let preference=Preference::new(&want.text,Scope::Task(Self::task()),parent,Source::Generator)?;
                        admitted.push((want,preference.id));
                    }
                }
            }
            self.deliberation_record(&Record::Refined {from:last.commitment,admitted,rejected})?;
            self.deliberation_request()
        })
    }
    pub fn deliberate(&self, offer: Offer) -> Result<serde_json::Value> {
        self.write(|| {
            ensure!(
                offer.state_root == self.store.head_root()?,
                "stale direction offer"
            );
            let (spec, preferences, book, turns) = self.deliberation_state()?;
            let shadow_events=self.events()?;
            let request = self.deliberation_request()?;
            ensure!(request["status"] == "deliberate", "cycle is terminal");
            let selected: Vec<usize> = serde_json::from_value(request["selected_desires"].clone())?;
            ensure!(!selected.is_empty(), "no selected desires");
            let active:Vec<usize>=serde_json::from_value(request["active_desires"].clone())?;
            let minimum=minimum_offer(&spec,&book,&selected,&active);
            ensure!((minimum..=4).contains(&offer.directions.len()), "offer {minimum}..4 genuine alternatives");
            let mut assessments = vec![];
            let mut rejected_directions = vec![];
            let mut actions = BTreeSet::new();
            for (index, d) in offer.directions.iter().enumerate() {
                if d.continuation.is_some() && turns.len() + 2 > spec.max_cycles {
                    rejected_directions
                        .push((index, "insufficient action budget for two-step plan".into()));
                    continue;
                }
                let result = assess_active(
                    &spec,
                    &book,
                    &turns,
                    &selected,
                    current_commitment(&spec, &turns),
                    d,
                    &active,
                );
                match result {
                    Ok(a) if actions.insert(serde_json::to_string(&d.action)?) => {
                        assessments.push(a)
                    }
                    Ok(_) => rejected_directions
                        .push((index, "duplicate action is not an alternative".into())),
                    Err(error) => rejected_directions.push((index, error.to_string())),
                }
            }
            ensure!(
                assessments.len() >= minimum,
                "need at least {minimum} valid distinct alternatives; rejected: {:?}",
                rejected_directions
            );
            let choices=assessments.iter().enumerate().filter(|(i,_)|eligible_indices(&assessments).contains(i))
                .map(|(i,a)|Ok((Hash::of(&[&serde_json::to_vec(&a.direction.action)?]).to_hex(),a.score.value,i)))
                .collect::<Result<Vec<_>>>()?;
            // This adapter has no admitted operational common-event schema; explicit opt-in
            // records the gate refusal, preserving the ordinary bounded fallback.
            self.tier2_select(offer.state_root,"notebook@v1",&self.events()?,&[])?;
            let policy_trace=self.select_with_policy(offer.state_root,&choices.iter().map(|(id,s,_)|(id.clone(),*s)).collect::<Vec<_>>())?;
            if let Some(trace)=&policy_trace {
                let mut scores=vec![];let mut controls=vec![];
                for a in &assessments {
                    // Preserve the actual evidence, active desires and current commitment.
                    // Only remove learned return history, never invent alternative outcomes.
                    if let Ok(control)=assess_active(&spec,&book,&[],&selected,current_commitment(&spec,&turns),&a.direction,&active) {
                        let id=Hash::of(&[&serde_json::to_vec(&a.direction.action)?]).to_hex();
                        scores.push((id,control.score.value));
                        controls.push(control);
                    }
                }
                let eligible=eligible_indices(&controls).into_iter().map(|i|scores[i].0.clone()).collect::<Vec<_>>();
                self.record_selection_intervention(trace,"notebook: remove return-history probability/residual learning; preserve observation, wants, offered actions, belief and policy",&scores,&eligible)?;
            }
            let selected_choice=if let Some(trace)=&policy_trace {
                if trace.selected.is_none() {
                    return Ok(serde_json::json!({"status":"selection_abstained","selection_decision":trace,"next":self.deliberation_request()?}));
                }
                trace.selected.as_ref().and_then(|id|choices.iter().find(|c|&c.0==id).map(|c|c.2))
            } else { choose_supported(&assessments) };
            let choice = match selected_choice {
                Some(choice) => choice,
                None => {
                    let effects = assessments.iter().map(|a| -> Result<serde_json::Value> {
                        Ok(serde_json::json!({"action":a.direction.action,"first_step_after":preferences.iter().copied().zip(observe(&spec,&transition(&spec,&book,&a.direction.action)?)).collect::<std::collections::BTreeMap<_,_>>(),"fallback":a.fallback}))
                    }).collect::<Result<Vec<_>>>()?;
                    bail!("no supported expected progress: weak-support fallback abstained; desires remain unmet. Current observation by obligation: {}; independently bounded effects: {}",serde_json::to_string(&preferences.iter().copied().zip(observe(&spec,&book)).collect::<std::collections::BTreeMap<_,_>>())?,serde_json::to_string(&effects)?);
                }
            };
            // Selection and all offered possibilities precede commitment; unchosen offers aren't outcomes.
            let selection_digest = self.blob(&serde_json::to_vec(&request["selection"])?)?;
            let selected_event = self.append(
                EventKind::Observed {
                    observation: Observation::Evidence {
                        digest: selection_digest,
                        source: "notebook-selection@v1".into(),
                    },
                },
                Source::Rules,
            )?;
            let mut candidates = vec![];
            for a in &assessments {
                let args = self.blob(&serde_json::to_vec(&a.direction)?)?;
                let mut c = Candidate::new(
                    match a.direction.action {
                        Move::Archive { .. } => ActionKind::Generate,
                        Move::Inspect { .. } => ActionKind::Inspect,
                        Move::Compose { .. } => ActionKind::Generate,
                    },
                    Source::Intuition,
                    args,
                    vec![selected_event],
                    vec![],
                )?;
                c.support = a
                    .direction
                    .supports
                    .iter()
                    .map(|i| preferences[*i])
                    .collect();
                self.append(
                    EventKind::Proposed {
                        candidate: c.clone(),
                    },
                    Source::Intuition,
                )?;
                self.record_shadow(offer.state_root,observation_revision(&spec),&c,&shadow_events,"notebook@v1");
                candidates.push(c);
            }
            let a = &assessments[choice];
            let candidate = &candidates[choice];
            let forecast = Forecast::new(
                OutcomeDefId(Hash::of(&[b"notebook-progress@v1"])),
                Horizon::ThisDispatch,
                Forecaster::Controller,
                Probability::new(
                    a.plan
                        .as_ref()
                        .map_or(a.prediction.value, |p| p.first_step_progress_probability),
                )?,
                self.events()?.len() as u64,
            );
            self.append(
                EventKind::Forecast {
                    forecast: forecast.clone(),
                    candidate: candidate.id,
                },
                Source::Rules,
            )?;
            let commitment = self.append(
                EventKind::Committed {
                    candidate: candidate.id,
                    decision: Decision::Selected {
                        value: candidate.id,
                        utility: Utility {
                            p_success: if a.direction.deferred_observation.is_some() {0.0} else {a.prediction.value},
                            value: a.projected_gain.value,
                            cost: a.effort_cost + a.continuity_cost,
                            ev: a.score.value,
                        },
                        propensity: policy_trace.as_ref().and_then(|t|t.propensity).unwrap_or(1.0),
                        evidence: vec![],
                    },
                },
                Source::Rules,
            )?;
            let next = transition(&spec, &book, &a.direction.action)?;
            let before = observe(&spec, &book);
            let after = observe(&spec, &next);
            let progress = active_progress(&spec, &selected, &active, &before, &after);
            let unmet: Vec<_> = selected
                .iter()
                .filter(|i| after[**i] < 1.0)
                .copied()
                .collect();
            let returned = self.append(
                EventKind::Observed {
                    observation: Observation::Evidence {
                        digest: self.blob(&serde_json::to_vec(&next)?)?,
                        source: "notebook-materialized@v1".into(),
                    },
                },
                Source::Rules,
            )?;
            let contrast = Filter::contrast_from_criteria(
                returned,
                unmet
                    .iter()
                    .map(|i| {
                        (
                            CriterionId(Hash::of(&[&preferences[*i].0 .0])),
                            UnmetKind::WrongBehaviour,
                        )
                    })
                    .collect(),
                vec![returned],
            );
            let residual = forecast.raw.get() - if progress { 1.0 } else { 0.0 };
            self.append(
                EventKind::Observed {
                    observation: Observation::Return {
                        surprise: vec![Surprise {
                            forecast: forecast.id,
                            residual,
                        }],
                        contrast: Some(contrast.clone()),
                    },
                },
                Source::Rules,
            )?;
            let generator_residual = a
                .direction
                .anticipated
                .iter()
                .zip(&after)
                .map(|(p, o)| p - o)
                .collect();
            let turn = Turn {
                active_obligations: Some(active.iter().map(|i| preferences[*i]).collect()),
                observation_revision: Some(observation_revision(&spec)),
                before_notebook: Some(book.clone()),
                rejected_directions,
                offered: assessments,
                chosen: choice,
                selected: selected.clone(),
                before,
                after,
                observed_progress: progress,
                notebook: next,
                commitment,
                residual,
                generator_residual,
                contrast: unmet,
                return_contrast: Some(contrast),
            };
            self.deliberation_record(&Record::Returned {
                turn: Box::new(turn.clone()),
            })?;
            Ok(serde_json::json!({"turn":turn,"next":self.deliberation_request()?}))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn shadow_failure_does_not_change_factual_decision() -> Result<()> {
        let a = tempfile::tempdir()?;
        let b = tempfile::tempdir()?;
        let e = Engine::open(a.path())?;
        e.deliberation_initialize(spec())?;
        let other = Engine::open(b.path())?;
        for (_, event) in e.events()? {
            other.store.append(&event)?;
        }
        for entry in fs::read_dir(a.path().join("cas"))? {
            let entry = entry?;
            fs::copy(entry.path(), b.path().join("cas").join(entry.file_name()))?;
        }
        let root = e.store.head_root()?;
        other.configure_shadow(super::super::shadow::Update {
            state_root: root,
            reason: "noninterference test".into(),
            config: tsr_learn::shadow::Config {
                bundle_directory: b.path().join("absent"),
                bundle_id: Hash::of(&[b"missing"]),
                dataset_directory: None,
                event: tsr_learn::DECISION_EVENT.into(),
                family: "fixture".into(),
                lineage: "fixture".into(),
                training_as_of: None,
            },
        })?;
        assert_eq!(other.store.head_root()?, root);
        let offer = || Offer {
            state_root: root,
            directions: vec![inspect(&["a"], 0.5), inspect(&["a", "b"], 1.0)],
        };
        let factual = e.deliberate(offer())?;
        let shadow = other.deliberate(offer())?;
        // Full decision/observed return equality; append timestamps/root are provenance only.
        for (key, value) in factual["turn"].as_object().unwrap() {
            if !["commitment", "return_contrast"].contains(&key.as_str()) {
                // event IDs include wall-clock append provenance
                assert_eq!(value, &shadow["turn"][key], "different turn field: {key}");
            }
        }
        assert_eq!(
            factual["turn"]["return_contrast"]["unmet"],
            shadow["turn"]["return_contrast"]["unmet"]
        );
        assert!(other.shadow_view()?["records"].as_array().unwrap().len() > 1);
        Ok(())
    }
    fn spec() -> Spec {
        Spec {
            archive: None,
            wants: vec![
                Want {
                    text: "Understand evidence".into(),
                    aim: Aim::Understand { topic: "x".into() },
                    parents: vec![],
                },
                Want {
                    text: "Produce useful account".into(),
                    aim: Aim::Account { topic: "x".into() },
                    parents: vec![0],
                },
                Want {
                    text: "Keep it concise".into(),
                    aim: Aim::Concise { max_words: 20 },
                    parents: vec![1],
                },
            ],
            notes: vec![
                Note {
                    id: "a".into(),
                    topic: "x".into(),
                    text: "One observed fact.".into(),
                },
                Note {
                    id: "b".into(),
                    topic: "x".into(),
                    text: "Another observed fact.".into(),
                },
            ],
            max_cycles: 8,
            attention_limit: 3,
        }
    }
    fn inspect(ids: &[&str], expected: f64) -> Direction {
        Direction {
            deferred_observation: None,
            continuation: None,
            purpose: "Investigate evidence needed by standing wants".into(),
            supports: vec![0],
            anticipated: vec![expected, 0.0, 0.0],
            action: Move::Inspect {
                notes: ids.iter().map(|s| s.to_string()).collect(),
            },
        }
    }
    fn account_only_spec() -> Spec {
        let mut s = spec();
        s.wants.remove(0);
        s.attention_limit = 2;
        for w in &mut s.wants {
            w.parents.clear();
        }
        s
    }
    fn plan(notes: &[&str]) -> Direction {
        Direction {
            deferred_observation: None,
            purpose: "Acquire evidence for the standing account".into(),
            supports: vec![0, 1],
            anticipated: vec![0.0, 0.0],
            action: Move::Inspect {
                notes: notes.iter().map(|s| s.to_string()).collect(),
            },
            continuation: Some(Continuation {
                anticipated: vec![notes.len() as f64 / 2.0, 1.0],
                action: Move::Compose {
                    title: "Evidence".into(),
                    claims: notes.iter().map(|s| s.to_string()).collect(),
                },
            }),
        }
    }
    fn deferred(s: &Spec, b: &Notebook, notes: &[&str]) -> Direction {
        let mut d = plan(notes);
        d.supports = vec![0];
        d.continuation = None;
        d.deferred_observation = Some(observation_revision(s));
        d.anticipated = observe(s, &transition(s, b, &d.action).unwrap());
        d
    }
    #[test]
    fn deferred_information_preserves_conflict_and_restarts_without_terminal_promise() -> Result<()>
    {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let mut s = account_only_spec();
        s.wants[1].aim = Aim::Concise { max_words: 1 };
        e.deliberation_initialize(s.clone())?;
        let initial = e.deliberation_request()?;
        assert!(initial["available_notes"][0]["text"].is_null());
        let b = Notebook::default();
        let one = deferred(&s, &b, &["a"]);
        let two = deferred(&s, &b, &["a", "b"]);
        let a = assess(&s, &b, &[], &[0, 1], None, &one)?;
        let costlier = assess(&s, &b, &[], &[0, 1], None, &two)?;
        assert_eq!(a.projected, vec![0.0, 0.0]);
        assert_eq!(a.projected_gain.value, 0.0);
        assert!(a.effort_cost > 0.0 && costlier.effort_cost > a.effort_cost);
        assert_eq!(a.score.value, -a.effort_cost);
        assert_eq!(choose_supported(&[costlier, a]), Some(1));
        let stale = e.store.head_root()?;
        let r = e.deliberate(Offer {
            state_root: stale,
            directions: vec![one.clone(), two],
        })?;
        assert_eq!(r["turn"]["observed_progress"], false);
        assert_eq!(r["turn"]["after"], serde_json::json!([0.0, 0.0]));
        assert_eq!(r["next"]["standing_desires"], initial["standing_desires"]);
        assert_eq!(
            r["next"]["constraint_status"]["status"],
            "proven_joint_conflict"
        );
        assert_eq!(r["next"]["notebook"]["claims"], serde_json::json!([]));
        assert!(r["next"]["available_notes"][0]["text"].is_string());
        assert_eq!(Engine::open(dir.path())?.deliberation_request()?, r["next"]);
        assert!(e
            .deliberate(Offer {
                state_root: stale,
                directions: vec![one]
            })
            .is_err());
        let (_, _, book, turns) = e.deliberation_state()?;
        assert!(assess(&s, &book, &turns, &[0, 1], None, &deferred(&s, &b, &["a"])).is_err());
        assert!(!return_progress(&s, &turns[0]));
        Ok(())
    }
    #[test]
    fn deferred_information_rejects_false_relevance_stale_scope_promises_and_budget() -> Result<()>
    {
        let mut s = account_only_spec();
        s.notes.push(Note {
            id: "outside-topic".into(),
            topic: "other".into(),
            text: "Unknown".into(),
        });
        let b = Notebook::default();
        let d = deferred(&s, &b, &["a"]);
        assert!(assess(&s, &b, &[], &[0, 1], None, &d).is_ok());
        for variant in 0..6 {
            let mut bad = d.clone();
            match variant {
                0 => bad.deferred_observation = Some(Hash::of(&[b"stale"])),
                1 => {
                    bad.action = Move::Inspect {
                        notes: vec!["unknown".into()],
                    }
                }
                2 => {
                    bad.action = Move::Inspect {
                        notes: vec!["outside-topic".into()],
                    }
                }
                3 => bad.supports = vec![1],
                4 => bad.anticipated = vec![1.0, 1.0],
                _ => bad.continuation = plan(&["a"]).continuation,
            }
            assert!(
                assess(&s, &b, &[], &[0, 1], None, &bad).is_err(),
                "case {variant}"
            );
        }
        assert!(assess_active(&s, &b, &[], &[0, 1], None, &d, &[1]).is_err());
        let baseline = assess(&s, &b, &[], &[0, 1], None, &d)?;
        let mut hidden = s.clone();
        hidden.notes[0].text="Completely different unavailable text with far more words; not a terminal value oracle".into();
        let control = assess(
            &hidden,
            &b,
            &[],
            &[0, 1],
            None,
            &deferred(&hidden, &b, &["a"]),
        )?;
        assert_eq!(baseline.score.value, control.score.value);
        assert_eq!(baseline.projected, control.projected);
        // Refinement aliases do not manufacture extra information value.
        s.wants.push(Want {
            text: "Related evidence detail".into(),
            aim: Aim::IncludeNotes {
                notes: vec!["a".into()],
            },
            parents: vec![0],
        });
        let mut refined = deferred(&s, &b, &["a"]);
        refined.supports = vec![0, 2];
        let aliased = assess(&s, &b, &[], &[0, 1, 2], None, &refined)?;
        assert_eq!(baseline.score.value, aliased.score.value);
        assert_eq!(baseline.effort_cost, aliased.effort_cost);
        assert!(assess_active(&s, &b, &[], &[2], None, &refined, &[1, 2]).is_err());
        s.max_cycles = 1;
        assert!(assess(&s, &b, &[], &[0, 1], None, &deferred(&s, &b, &["a"])).is_err());
        Ok(())
    }
    #[test]
    fn deferred_information_singleton_frontier_and_productive_precedence() -> Result<()> {
        let s = account_only_spec();
        let b = Notebook {
            archive_events: vec![],
            known: ["a".into()].into_iter().collect(),
            claims: vec!["a".into()],
            title: "Note".into(),
        };
        assert_eq!(minimum_offer(&s, &b, &[0, 1], &[0, 1]), 1);
        let info = assess(&s, &b, &[], &[0, 1], None, &deferred(&s, &b, &["b"]))?;
        assert!(info.rejected_supports.is_empty());
        assert_eq!(info.projected_gain.value, 0.0);
        let empty = Notebook {
            archive_events: vec![],
            known: b.known.clone(),
            ..Notebook::default()
        };
        assert_eq!(minimum_offer(&s, &empty, &[0, 1], &[0, 1]), 2);
        let direct = Direction {
            deferred_observation: None,
            purpose: "Represent inspected evidence".into(),
            supports: vec![0],
            anticipated: vec![0.5, 1.0],
            action: Move::Compose {
                title: "Note".into(),
                claims: vec!["a".into()],
            },
            continuation: None,
        };
        let productive = assess(&s, &empty, &[], &[0, 1], None, &direct)?;
        assert!(productive.fallback.as_ref().unwrap().eligible);
        assert_eq!(eligible_indices(&[info, productive]), vec![1]);
        // Full adapter path: acquire known evidence, compose it, then the sole
        // remaining relevant inspection is admitted without an invented rival.
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.deliberation_initialize(s.clone())?;
        e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![
                deferred(&s, &Notebook::default(), &["a"]),
                deferred(&s, &Notebook::default(), &["a", "b"]),
            ],
        })?;
        e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![direct, deferred(&s, &empty, &["b"])],
        })?;
        let request = e.deliberation_request()?;
        assert_eq!(request["minimum_directions"], 1);
        let r = e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![deferred(&s, &b, &["b"])],
        })?;
        assert_eq!(r["turn"]["observed_progress"], false);
        assert_eq!(
            r["next"]["notebook"]["known"],
            serde_json::json!(["a", "b"])
        );
        assert_eq!(r["next"]["observation"], serde_json::json!([0.5, 1.0]));
        assert_eq!(Engine::open(dir.path())?.deliberation_request()?, r["next"]);
        Ok(())
    }
    #[test]
    fn support_reports_correlated_dispatches_without_inventing_mature_episodes() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.deliberation_initialize(spec())?;
        let result = e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![inspect(&["a"], 0.5), inspect(&["a", "b"], 1.0)],
        })?;
        let first = &result["turn"]["offered"][0]["expectation_support"];
        assert_eq!(first["compatible_dispatches"], 0);
        assert_eq!(first["independent_mature_episodes"], 0);
        let (s, _, _book, turns) = e.deliberation_state()?;
        let learned = assess(
            &s,
            &Notebook::default(),
            &turns,
            &[0, 1, 2],
            None,
            &inspect(&["a"], 1.0),
        )?;
        let support = learned.expectation_support.unwrap();
        assert_eq!(support.compatible_dispatches, 1);
        assert!(support.effective_dispatch_weight > 0.0);
        assert_eq!(support.independent_mature_episodes, 0);
        assert!(support.calibration_status.starts_with("unvalidated"));
        assert_eq!(
            Engine::open(dir.path())?.deliberation_request()?,
            result["next"]
        );
        Ok(())
    }
    #[test]
    fn fallback_allows_bounded_information_but_not_standing_regression() -> Result<()> {
        let s = spec();
        let a = assess(
            &s,
            &Notebook::default(),
            &[],
            &[0, 1, 2],
            None,
            &inspect(&["a"], 0.5),
        )?;
        assert!(a.fallback.as_ref().unwrap().eligible);
        assert_eq!(choose_supported(std::slice::from_ref(&a)), Some(0));
        let control = assess(
            &s,
            &Notebook::default(),
            &[],
            &[0, 1, 2],
            None,
            &inspect(&["a"], 0.5),
        )?;
        assert_eq!(serde_json::to_value(&a)?, serde_json::to_value(&control)?);
        let mut s = s;
        s.wants[2].aim = Aim::Concise { max_words: 3 };
        // A dominating loss, not a positive-net reversible tradeoff.
        s.notes[1].text = "long ".repeat(100);
        let book = Notebook {
            archive_events: vec![],
            known: ["a".into(), "b".into()].into_iter().collect(),
            claims: ["a".into()].into_iter().collect(),
            title: "Account".into(),
        };
        let d = Direction {
            deferred_observation: None,
            purpose: "Expand account".into(),
            supports: vec![1],
            anticipated: vec![1.0, 1.0, 1.0],
            action: Move::Compose {
                title: "Account".into(),
                claims: vec!["a".into(), "b".into()],
            },
            continuation: None,
        };
        let a = assess(&s, &book, &[], &[1], None, &d)?;
        assert!(!a.fallback.as_ref().unwrap().eligible);
        assert_eq!(choose_supported(&[a]), None);
        let restored = assess_active(&s, &book, &[], &[1], None, &d, &[0, 1])?;
        assert!(restored.fallback.as_ref().unwrap().eligible);
        Ok(())
    }
    #[test]
    fn exact_reversible_tradeoff_preserves_loss_without_inventing_success() -> Result<()> {
        let mut s = account_only_spec();
        s.wants[1].aim = Aim::Concise { max_words: 3 };
        s.notes.push(Note {
            id: "other".into(),
            topic: "y".into(),
            text: "Short context".into(),
        });
        let b = Notebook {
            archive_events: vec![],
            known: ["a".into(), "b".into(), "other".into()]
                .into_iter()
                .collect(),
            claims: ["other".into()].into_iter().collect(),
            title: "Context".into(),
        };
        let d = Direction {
            deferred_observation: None,
            purpose: "Cover the original evidence".into(),
            supports: vec![0],
            anticipated: vec![1.0, 0.0],
            action: Move::Compose {
                title: "Account".into(),
                claims: vec!["a".into(), "b".into()],
            },
            continuation: None,
        };
        let a = assess(&s, &b, &[], &[0], None, &d)?;
        let f = a.fallback.as_ref().unwrap();
        assert!(f.eligible);
        assert!(f.exact_first_step_net.unwrap() > 0.0);
        assert_eq!(f.exact_standing_loss, Some(1.0 - 3.0 / 7.0));
        let after = observe(&s, &transition(&s, &b, &d.action)?);
        assert!(!active_progress(
            &s,
            &[0],
            &[0, 1],
            &observe(&s, &b),
            &after
        ));
        assert_eq!(after, vec![1.0, 3.0 / 7.0]);
        assert_eq!(s.wants.len(), 2);
        let control = assess(&s, &b, &[], &[0], None, &d)?;
        assert_eq!(serde_json::to_value(a)?, serde_json::to_value(control)?);
        Ok(())
    }
    #[test]
    fn prerequisite_fallback_requires_relevant_terminal_capacity() -> Result<()> {
        let mut s = account_only_spec();
        s.notes.push(Note {
            id: "irrelevant".into(),
            topic: "other".into(),
            text: "Other topic".into(),
        });
        let mut d = plan(&["irrelevant"]);
        // Optimistic model support cannot turn unrelated note coverage into progress.
        d.anticipated = vec![0.0, 0.0];
        let a = assess(&s, &Notebook::default(), &[], &[0], None, &d)?;
        assert!(!a.fallback.as_ref().unwrap().eligible);
        Ok(())
    }
    #[test]
    fn prerequisite_serves_account_without_inventing_an_inspection_desire() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let s = account_only_spec();
        e.deliberation_initialize(s.clone())?;
        let book = Notebook::default();
        let mut direct = plan(&["a", "b"]);
        direct.continuation = None;
        assert!(assess(&s, &book, &[], &[0, 1], None, &direct).is_err());
        let mut plans = vec![plan(&["a"]), plan(&["a", "b"])];
        plans[1].anticipated = vec![0.5, 0.25];
        let baseline = assess(&s, &book, &[], &[0, 1], None, &plans[1])?;
        let result = e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: plans,
        })?;
        assert_eq!(result["turn"]["chosen"], 1);
        assert_eq!(result["turn"]["after"], serde_json::json!([0.0, 0.0]));
        assert_eq!(result["turn"]["observed_progress"], false);
        assert_eq!(result["turn"]["residual"], 0.0);
        assert_eq!(
            result["turn"]["generator_residual"],
            serde_json::json!([0.5, 0.25])
        );
        assert_eq!(result["next"]["notebook"]["claims"], serde_json::json!([]));
        assert_eq!(
            result["next"]["standing_desires"].as_array().unwrap().len(),
            2
        );
        let (_, _, inspected, turns) = e.deliberation_state()?;
        // No terminal failure label is inferred from neutral prerequisite return.
        let control = assess(&s, &book, &turns, &[0, 1], None, &plan(&["a", "b"]))?;
        assert_eq!(baseline.score.value, control.score.value);
        assert_eq!(baseline.prediction.value, control.prediction.value);
        assert_eq!(
            Engine::open(dir.path())?.deliberation_request()?,
            result["next"]
        );
        // A fresh proposal, not the saved continuation, can now finish the account.
        let finish = |notes: &[&str]| {
            let p = plan(notes);
            let next = p.continuation.unwrap();
            Direction {
                deferred_observation: None,
                purpose: "Use observed evidence".into(),
                supports: p.supports,
                anticipated: next.anticipated,
                action: next.action,
                continuation: None,
            }
        };
        assert_eq!(inspected.known.len(), 2);
        let completed = e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![finish(&["a"]), finish(&["a", "b"])],
        })?;
        assert_eq!(completed["next"]["status"], "satisfied");
        let (_, _, _, completed_turns) = e.deliberation_state()?;
        let learned = assess(
            &s,
            &book,
            &completed_turns,
            &[0, 1],
            None,
            &plan(&["a", "b"]),
        )?;
        assert!(learned.prediction.value > baseline.prediction.value);
        assert!(learned.score.value > baseline.score.value);
        assert!(learned.score.value < learned.shared.as_ref().unwrap().score);
        assert_eq!(
            learned
                .plan
                .as_ref()
                .unwrap()
                .first_step_progress_probability,
            0.0
        );
        // New forecasts concern this dispatch; old persisted receipts are not rewritten.
        for (_, event) in e.events()? {
            if let EventKind::Forecast { forecast, .. } = event.kind {
                assert_eq!(forecast.horizon, Horizon::ThisDispatch);
            }
        }
        Ok(())
    }
    #[test]
    fn nonneutral_prerequisite_learns_only_its_observed_first_action() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let mut s = account_only_spec();
        s.wants[0].aim = Aim::Understand { topic: "x".into() };
        e.deliberation_initialize(s.clone())?;
        let result = e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![plan(&["a"]), plan(&["a", "b"])],
        })?;
        assert_eq!(result["turn"]["after"], serde_json::json!([1.0, 0.0]));
        assert_eq!(result["next"]["notebook"]["claims"], serde_json::json!([]));
        let (_, _, book, turns) = e.deliberation_state()?;
        assert_eq!(
            Engine::open(dir.path())?.deliberation_request()?,
            result["next"]
        );
        let mut inspection = plan(&["a"]);
        inspection.continuation = None;
        inspection.supports = vec![0];
        inspection.anticipated = vec![0.5, 0.0];
        let empty = Notebook::default();
        let prior = assess(&s, &empty, &[], &[0], None, &inspection)?;
        let learned = assess(&s, &empty, &turns, &[0], None, &inspection)?;
        assert!(learned.prediction.value > prior.prediction.value);
        assert_eq!(learned.prediction_evidence.len(), 1);
        let mut composition = plan(&["a"]);
        let future = composition.continuation.take().unwrap();
        composition.action = future.action;
        composition.anticipated = vec![1.0, 1.0];
        composition.supports = vec![1];
        let next = assess(&s, &book, &turns, &[1], None, &composition)?;
        assert!(next.prediction_evidence.is_empty());
        Ok(())
    }
    #[test]
    fn planned_dependencies_cannot_hide_invalid_actions_or_overrun_budget() -> Result<()> {
        let s = account_only_spec();
        let b = Notebook::default();
        let mut bad = plan(&["a"]);
        bad.continuation.as_mut().unwrap().action = Move::Compose {
            title: "x".into(),
            claims: vec!["outside".into()],
        };
        assert!(assess(&s, &b, &[], &[0, 1], None, &bad).is_err());
        let mut known = b.clone();
        known.known.insert("a".into());
        assert!(assess(&s, &known, &[], &[0, 1], None, &plan(&["a"])).is_err());
        let mut false_first = plan(&["a"]);
        false_first.anticipated[0] = 1.0;
        let bounded_error = assess(&s, &b, &[], &[0, 1], None, &false_first)?;
        let accurate = assess(&s, &b, &[], &[0, 1], None, &plan(&["a"]))?;
        assert_eq!(bounded_error.score.value, accurate.score.value);
        assert_eq!(bounded_error.prediction.value, accurate.prediction.value);
        assert_eq!(bounded_error.direction.anticipated, vec![1.0, 0.0]);
        false_first.anticipated = vec![f64::NAN, 0.0];
        assert!(assess(&s, &b, &[], &[0, 1], None, &false_first).is_err());
        false_first.anticipated = vec![0.0];
        assert!(assess(&s, &b, &[], &[0, 1], None, &false_first).is_err());
        let mut changing = s.clone();
        changing.wants[0].aim = Aim::Understand { topic: "x".into() };
        let changed = assess(&changing, &b, &[], &[0, 1], None, &plan(&["a"]))?;
        assert_eq!(
            changed
                .plan
                .as_ref()
                .unwrap()
                .first_step_progress_probability,
            1.0
        );
        assert_eq!(
            changed
                .plan
                .as_ref()
                .unwrap()
                .path
                .as_ref()
                .unwrap()
                .terminal
                .observation
                .values()
                .copied()
                .collect::<Vec<_>>(),
            vec![Some(0.0), Some(0.0)]
        );
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let mut limited = s;
        limited.max_cycles = 1;
        e.deliberation_initialize(limited)?;
        let root = e.store.head_root()?;
        assert!(e
            .deliberate(Offer {
                state_root: root,
                directions: vec![plan(&["a"]), plan(&["a", "b"])]
            })
            .is_err());
        assert_eq!(root, e.store.head_root()?);
        Ok(())
    }
    #[test]
    fn notebook_learned_scores_are_audited_before_next_return() -> Result<()> {
        use crate::engine::decision_policy::Update;
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let mut s = spec();
        for want in &mut s.wants {
            want.parents.clear();
        }
        e.deliberation_initialize(s)?;
        e.update_decision_policy(Update {
            state_root: e.store.head_root()?,
            policy: tsr_domain::selection::Policy {
                margin: 10.0,
                epsilon: 1.0,
                seed: 7,
            },
            reason: "Audit control".into(),
        })?;
        e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![inspect(&["a"], 0.5), inspect(&["b"], 0.5)],
        })?;
        let (_, _, book, _) = e.deliberation_state()?;
        let known = book.known.iter().next().unwrap().clone();
        let unread = if known == "a" { "b" } else { "a" };
        let compose = Direction {
            deferred_observation: None,
            purpose: "Offline audit fixture".into(),
            supports: vec![1, 2],
            anticipated: vec![0.5, 0.5, 1.0],
            continuation: None,
            action: Move::Compose {
                title: "Evidence".into(),
                claims: vec![known],
            },
        };
        e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![inspect(&[unread], 1.0), compose],
        })?;
        let view = e.decision_policy_view()?;
        let c = &view["learning_interventions"][1]["comparison"];
        assert_ne!(c["factual"]["scores"], c["fixed_support"]["scores"]);
        assert_eq!(c["factual"]["draw"], c["fixed_support"]["draw"]);
        assert_eq!(view, Engine::open(dir.path())?.decision_policy_view()?);
        Ok(())
    }
    #[test]
    fn shared_selection_abstains_then_explores_without_weakening_wants() -> Result<()> {
        use crate::engine::decision_policy::Update;
        use tsr_domain::selection::Policy;
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.deliberation_initialize(spec())?;
        let before = e.deliberation_request()?;
        e.update_decision_policy(Update {
            state_root: e.store.head_root()?,
            policy: Policy {
                margin: 10.0,
                epsilon: 0.0,
                seed: 9,
            },
            reason: "Margin control".into(),
        })?;
        let directions = vec![
            inspect(&["a"], 0.5),
            inspect(&["b"], 0.5),
            inspect(&[], 1.0),
        ];
        let result = e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: directions.clone(),
        })?;
        assert_eq!(result["status"], "selection_abstained");
        assert_eq!(result["next"]["cycles"], 0);
        assert_eq!(
            result["selection_decision"]["scores"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            result["next"]["standing_desires"],
            before["standing_desires"]
        );
        assert_eq!(result["next"]["observation"], before["observation"]);
        assert_eq!(
            e.decision_policy_view()?,
            Engine::open(dir.path())?.decision_policy_view()?
        );
        e.update_decision_policy(Update {
            state_root: e.store.head_root()?,
            policy: Policy {
                margin: 10.0,
                epsilon: 1.0,
                seed: 9,
            },
            reason: "Explicit bounded exploration".into(),
        })?;
        e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions,
        })?;
        let probabilities = e
            .events()?
            .into_iter()
            .filter_map(|(_, e)| match e.kind {
                EventKind::Committed {
                    decision: Decision::Selected { propensity, .. },
                    ..
                } => Some(propensity),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(probabilities, vec![0.5]);
        let audits = e.decision_policy_view()?;
        assert_eq!(
            audits["learning_interventions"].as_array().unwrap().len(),
            2
        );
        assert_eq!(
            audits["learning_interventions"][1]["comparison"]["factual"]["propensity"],
            0.5
        );
        assert_eq!(audits, Engine::open(dir.path())?.decision_policy_view()?);

        assert!(e.deliberation_request()?["learning_audit"].is_null());
        Ok(())
    }
    #[test]
    fn observation_revision_releases_commitment_and_restart_preserves_review() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let s = spec();
        e.deliberation_initialize(s.clone())?;
        assert_eq!(
            e.deliberation_request()?["commitment_review"]["status"],
            "no_commitment"
        );
        e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![inspect(&["a"], 0.5), inspect(&["a", "b"], 1.0)],
        })?;
        let before = e.deliberation_request()?;
        assert_eq!(
            before["commitment_review"]["status"],
            "supported_by_current_return"
        );
        let old_return = serde_json::to_value(&e.deliberation_state()?.3)?;
        let mut changed = s.notes[0].clone();
        changed.text = "A changed observation.".into();
        let after = e.update_deliberation_observation(ObservationUpdate {
            state_root: e.store.head_root()?,
            notes: vec![changed],
            reason: "Independent source correction".into(),
        })?;
        assert_eq!(after["commitment_review"]["status"], "evidence_superseded");
        assert_eq!(
            after["commitment_review"]["evidence_current_and_productive"],
            false
        );
        assert_eq!(after["standing_desires"], before["standing_desires"]);
        assert_eq!(
            serde_json::to_value(&e.deliberation_state()?.3)?,
            old_return
        );
        assert_eq!(e.deliberation_request()?, after);
        drop(e);
        let e = Engine::open(dir.path())?;
        assert_eq!(e.deliberation_request()?, after);
        assert!(
            current_commitment(&e.deliberation_state()?.0, &e.deliberation_state()?.3).is_none()
        );
        Ok(())
    }
    #[test]
    fn superseded_support_changes_choice_with_observation_and_expectation_fixed() -> Result<()> {
        let (s, b, inspection, composition) = balanced_alternatives();
        let before = Notebook {
            archive_events: vec![],
            known: b.known.clone(),
            ..Notebook::default()
        };
        let prior = Direction {
            deferred_observation: None,
            purpose: "Prior observed account".into(),
            supports: vec![1],
            anticipated: observe(&s, &b),
            continuation: None,
            action: Move::Compose {
                title: b.title.clone(),
                claims: b.claims.clone(),
            },
        };
        assert_eq!(
            serde_json::to_value(transition(&s, &before, &prior.action)?)?,
            serde_json::to_value(&b)?
        );
        let history = vec![Turn {
            active_obligations: None,
            generator_residual: vec![],
            observation_revision: Some(observation_revision(&s)),
            before_notebook: Some(before.clone()),
            rejected_directions: vec![],
            offered: vec![assess(&s, &before, &[], &[0, 1], None, &prior)?],
            chosen: 0,
            selected: vec![0, 1],
            before: observe(&s, &before),
            after: observe(&s, &b),
            observed_progress: true,
            notebook: b.clone(),
            commitment: EventId(Hash::of(&[b"offline-current-commitment-control"])),
            residual: 0.0,
            contrast: vec![],
            return_contrast: None,
        }];
        let choose = |spec: &Spec| -> Result<bool> {
            let previous = current_commitment(spec, &history);
            // Expectation history intentionally fixed empty; isolate only current support binding.
            Ok(assess(spec, &b, &[], &[0, 1], previous, &inspection)?
                .score
                .value
                > assess(spec, &b, &[], &[0, 1], previous, &composition)?
                    .score
                    .value)
        };
        assert!(!choose(&s)?);
        let mut revised = s.clone();
        revised.notes[0].text = "New revised evidence".into();
        assert_eq!(observe(&revised, &b), observe(&s, &b));
        assert!(choose(&revised)?);
        assert!(!choose(&s)?);
        assert_eq!(
            commitment_review(&revised, &history)["status"],
            "evidence_superseded"
        );
        let mut failed = history.clone();
        failed[0].after = failed[0].before.clone();
        assert_eq!(
            commitment_review(&s, &failed)["status"],
            "not_supported_by_return"
        );
        Ok(())
    }
    #[test]
    fn quiet_route_tuning_is_durable_causal_and_preserves_desires() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let mut s = spec();
        s.attention_limit = 1;
        for want in &mut s.wants {
            want.parents.clear();
        }
        e.deliberation_initialize(s)?;
        let baseline = e.deliberation_request()?;
        let quiet = e.update_selection_tune(SelectionTuneUpdate {
            state_root: e.store.head_root()?,
            quiet_route: 1.0,
            seed: 19,
            reason: "Operator enables leading-edge attention in this offline control".into(),
        })?;
        assert_eq!(quiet["selection"]["quiet_selected"], true);
        assert_eq!(quiet["selection"]["seed"], 19);
        assert_eq!(quiet["selected_desires"], serde_json::json!([2]));
        assert_ne!(quiet["selected_desires"], baseline["selected_desires"]);
        for key in [
            "standing_desires",
            "observation",
            "notebook",
            "recent_returns",
            "authority",
        ] {
            assert_eq!(quiet[key], baseline[key]);
        }
        assert_eq!(e.deliberation_request()?, quiet);
        let root = e.store.head_root()?;
        for value in [f64::NAN, -0.1, 1.1] {
            assert!(e
                .update_selection_tune(SelectionTuneUpdate {
                    state_root: root,
                    quiet_route: value,
                    seed: 19,
                    reason: "invalid".into()
                })
                .is_err());
            assert_eq!(e.store.head_root()?, root);
        }
        assert!(e
            .update_selection_tune(SelectionTuneUpdate {
                state_root: serde_json::from_value(baseline["state_root"].clone())?,
                quiet_route: 0.0,
                seed: 0,
                reason: "stale".into()
            })
            .is_err());
        drop(e);
        let e = Engine::open(dir.path())?;
        assert_eq!(e.deliberation_request()?, quiet);
        let restored = e.update_selection_tune(SelectionTuneUpdate {
            state_root: root,
            quiet_route: 0.0,
            seed: 0,
            reason: "Restore original attention policy".into(),
        })?;
        assert_eq!(restored["selected_desires"], baseline["selected_desires"]);
        assert_eq!(restored["selection"]["quiet_selected"], false);
        e.update_selection_tune(SelectionTuneUpdate {
            state_root: e.store.head_root()?,
            quiet_route: 1.0,
            seed: 19,
            reason: "Re-enable for withdrawal control".into(),
        })?;
        let ids = e.deliberation_state()?.1;
        let withdrawn = e.update_desire_status(DesireStatusUpdate {
            state_root: e.store.head_root()?,
            preference: ids[2],
            status: PreferenceStatus::Withdrawn,
        })?;
        assert_eq!(withdrawn["selected_desires"], serde_json::json!([1]));
        assert_eq!(withdrawn["standing_desires"], baseline["standing_desires"]);
        Ok(())
    }
    #[test]
    fn generator_cannot_supply_selection_tuning_evidence() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.deliberation_initialize(spec())?;
        let record = Record::SelectionTuned {
            quiet_route: 1.0,
            seed: 0,
            reason: "Untrusted generated configuration".into(),
        };
        let digest = e.blob(&serde_json::to_vec(&record)?)?;
        e.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    digest,
                    source: SOURCE.into(),
                },
            },
            Source::Generator,
        )?;
        assert!(e
            .deliberation_request()
            .unwrap_err()
            .to_string()
            .contains("operator selection tuning"));
        Ok(())
    }
    #[test]
    fn singleton_frontier_does_not_require_an_invented_alternative() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let mut s = account_only_spec();
        s.notes.truncate(1);
        e.deliberation_initialize(s)?;
        let result = e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![plan(&["a"])],
        })?;
        assert_eq!(result["turn"]["offered"].as_array().unwrap().len(), 1);
        assert_eq!(
            result["next"]["notebook"]["known"],
            serde_json::json!(["a"])
        );
        assert_eq!(result["next"]["observation"], serde_json::json!([0.0, 0.0]));
        Ok(())
    }

    #[test]
    fn revised_evidence_invalidates_planned_continuation_without_auto_execution() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let s = account_only_spec();
        e.deliberation_initialize(s.clone())?;
        e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![plan(&["a"]), plan(&["a", "b"])],
        })?;
        let stale = e.store.head_root()?;
        let mut note = s.notes[0].clone();
        note.text = "A revised admitted observation.".into();
        e.update_deliberation_observation(ObservationUpdate {
            state_root: stale,
            reason: "Independent revision".into(),
            notes: vec![note],
        })?;
        let state = e.deliberation_request()?;
        assert_eq!(state["notebook"]["known"], serde_json::json!(["b"]));
        assert_eq!(state["notebook"]["claims"], serde_json::json!([]));
        assert!(e
            .deliberate(Offer {
                state_root: stale,
                directions: vec![plan(&["a"]), plan(&["a", "b"])]
            })
            .is_err());
        assert_eq!(Engine::open(dir.path())?.deliberation_request()?, state);
        Ok(())
    }

    #[test]
    fn operator_status_interface_is_version_bound_and_cannot_manufacture_fulfillment() -> Result<()>
    {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.deliberation_initialize(spec())?;
        let (_, ids, _, _) = e.deliberation_state()?;
        let root = e.store.head_root()?;
        assert!(e
            .update_desire_status(DesireStatusUpdate {
                state_root: root,
                preference: ids[0],
                status: PreferenceStatus::Fulfilled
            })
            .is_err());
        assert_eq!(e.store.head_root()?, root);
        let result = e.update_desire_status(DesireStatusUpdate {
            state_root: root,
            preference: ids[0],
            status: PreferenceStatus::Withdrawn,
        })?;
        assert!(!result["active_desires"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!(0)));
        assert!(e
            .update_desire_status(DesireStatusUpdate {
                state_root: root,
                preference: ids[0],
                status: PreferenceStatus::Open
            })
            .is_err());
        let restored = e.update_desire_status(DesireStatusUpdate {
            state_root: e.store.head_root()?,
            preference: ids[0],
            status: PreferenceStatus::Open,
        })?;
        assert_eq!(restored["observation"][0], 0.0);
        assert!(restored["active_desires"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!(0)));
        Ok(())
    }

    #[test]
    fn withdrawal_restoration_and_restart_share_one_active_view() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let s = spec();
        e.deliberation_initialize(s.clone())?;
        let (_, ids, _, _) = e.deliberation_state()?;
        for id in &ids {
            e.append(
                EventKind::StatusSet {
                    preference: *id,
                    status: PreferenceStatus::Withdrawn,
                    by: Source::Operator,
                },
                Source::Operator,
            )?;
        }
        let inactive = e.deliberation_request()?;
        assert_eq!(inactive["status"], "inactive");
        assert_eq!(inactive["active_desires"], serde_json::json!([]));
        assert_eq!(inactive["observation"], serde_json::json!([0.0, 0.0, 0.0]));
        assert_eq!(Engine::open(dir.path())?.deliberation_request()?, inactive);
        e.append(
            EventKind::StatusSet {
                preference: ids[0],
                status: PreferenceStatus::Open,
                by: Source::Operator,
            },
            Source::Operator,
        )?;
        let restored = e.deliberation_request()?;
        assert_eq!(restored["active_desires"], serde_json::json!([0]));
        let result = e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![inspect(&["a"], 0.5), inspect(&["a", "b"], 1.0)],
        })?;
        assert_eq!(result["next"]["status"], "satisfied");
        let (_, _, _, turns) = e.deliberation_state()?;
        assert_eq!(turns[0].active_obligations, Some(vec![ids[0]]));
        assert!(turns[0].observed_progress);
        // Restoring an unmet want reopens the cycle, without rewriting the old success.
        e.append(
            EventKind::StatusSet {
                preference: ids[1],
                status: PreferenceStatus::Open,
                by: Source::Operator,
            },
            Source::Operator,
        )?;
        let reopened = e.deliberation_request()?;
        assert_eq!(reopened["status"], "deliberate");
        let (_, _, _, retained) = e.deliberation_state()?;
        assert_eq!(
            serde_json::to_value(&turns)?,
            serde_json::to_value(&retained)?
        );
        let candidate = inspect(&["a"], 0.5);
        let same = assess_active(
            &s,
            &Notebook::default(),
            &retained,
            &[0],
            None,
            &candidate,
            &[0],
        )?;
        let changed = assess_active(
            &s,
            &Notebook::default(),
            &retained,
            &[0],
            None,
            &candidate,
            &[0, 1],
        )?;
        assert!(same.prediction.value > 0.5);
        assert_eq!(changed.prediction.value, 0.5);
        assert!(changed.prediction_evidence.is_empty());
        assert!(!changed.calibration_evidence.is_empty()); // different success event, not a negative label
        assert_eq!(Engine::open(dir.path())?.deliberation_request()?, reopened);
        Ok(())
    }

    #[test]
    fn inactive_refinement_lineage_cannot_reenter_attention() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.deliberation_initialize(spec())?;
        e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![inspect(&["a", "b"], 1.0), inspect(&["a"], 0.5)],
        })?;
        let (_, _, _, turns) = e.deliberation_state()?;
        e.refine_directions(RefinementOffer {
            state_root: e.store.head_root()?,
            return_commitment: turns[0].commitment,
            refinements: vec![Refinement {
                text: "Represent missing observation".into(),
                parent: 1,
                notes: vec!["a".into()],
            }],
        })?;
        let (_, ids, _, _) = e.deliberation_state()?;
        for (index, status, expected) in [
            (3, PreferenceStatus::Withdrawn, false),
            (3, PreferenceStatus::Open, true),
            (1, PreferenceStatus::Withdrawn, false),
            (1, PreferenceStatus::Open, true),
        ] {
            e.append(
                EventKind::StatusSet {
                    preference: ids[index],
                    status,
                    by: Source::Operator,
                },
                Source::Operator,
            )?;
            let request = e.deliberation_request()?;
            assert_eq!(
                request["active_desires"]
                    .as_array()
                    .unwrap()
                    .contains(&serde_json::json!(3)),
                expected
            );
            assert_eq!(
                request["selected_desires"]
                    .as_array()
                    .unwrap()
                    .contains(&serde_json::json!(3)),
                expected
            );
            assert_eq!(Engine::open(dir.path())?.deliberation_request()?, request);
        }
        Ok(())
    }

    #[test]
    fn restoring_descendant_cannot_bypass_withdrawn_intermediate_parent() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.deliberation_initialize(spec())?;
        e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![inspect(&["a", "b"], 1.0), inspect(&["a"], 0.5)],
        })?;
        let (_, _, _, turns) = e.deliberation_state()?;
        e.refine_directions(RefinementOffer {
            state_root: e.store.head_root()?,
            return_commitment: turns[0].commitment,
            refinements: vec![Refinement {
                text: "Represent missing observations".into(),
                parent: 1,
                notes: vec!["a".into(), "b".into()],
            }],
        })?;
        let compose = |id: &str| Direction {
            deferred_observation: None,
            purpose: "Represent observation".into(),
            supports: vec![1, 2, 3],
            anticipated: vec![1.0, 0.5, 1.0, 0.5],
            action: Move::Compose {
                title: "x".into(),
                claims: vec![id.into()],
            },
            continuation: None,
        };
        e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![compose("a"), compose("b")],
        })?;
        let (_, _, _, turns) = e.deliberation_state()?;
        e.refine_directions(RefinementOffer {
            state_root: e.store.head_root()?,
            return_commitment: turns[1].commitment,
            refinements: vec![Refinement {
                text: "Represent remaining observation".into(),
                parent: 3,
                notes: vec!["b".into()],
            }],
        })?;
        let (_, ids, _, _) = e.deliberation_state()?;
        assert_eq!(ids.len(), 5);
        for (index, status, expected) in [
            (3, PreferenceStatus::Withdrawn, false),
            (4, PreferenceStatus::Open, false),
            (3, PreferenceStatus::Open, true),
        ] {
            e.append(
                EventKind::StatusSet {
                    preference: ids[index],
                    status,
                    by: Source::Operator,
                },
                Source::Operator,
            )?;
            let request = e.deliberation_request()?;
            assert_eq!(
                request["active_desires"]
                    .as_array()
                    .unwrap()
                    .contains(&serde_json::json!(4)),
                expected
            );
            assert_eq!(
                request["selected_desires"]
                    .as_array()
                    .unwrap()
                    .contains(&serde_json::json!(4)),
                expected
            );
            assert_eq!(Engine::open(dir.path())?.deliberation_request()?, request);
        }
        Ok(())
    }

    #[test]
    fn withdrawing_conflicting_standing_want_changes_choice_and_restores_exactly() -> Result<()> {
        let mut s = account_only_spec();
        s.wants[1].aim = Aim::Concise { max_words: 10 };
        s.notes.push(Note {
            id: "c".into(),
            topic: "x".into(),
            text: "long ".repeat(100),
        });
        let book = Notebook {
            archive_events: vec![],
            known: ["a", "b", "c"].into_iter().map(str::to_owned).collect(),
            title: "x".into(),
            claims: vec!["a".into()],
        };
        let make = |claims: Vec<&str>, coverage, brevity| Direction {
            deferred_observation: None,
            purpose: "Generated candidate fixture".into(),
            supports: vec![0],
            anticipated: vec![coverage, brevity],
            action: Move::Compose {
                title: "x".into(),
                claims: claims.into_iter().map(str::to_owned).collect(),
            },
            continuation: None,
        };
        let partial = make(vec!["a", "b"], 2.0 / 3.0, 1.0);
        let full = make(vec!["a", "b", "c"], 1.0, 0.09);
        let score = |active: &[usize], d: &Direction| {
            assess_active(&s, &book, &[], &[0], None, d, active).map(|a| a.score.value)
        };
        let initial = [score(&[0, 1], &partial)?, score(&[0, 1], &full)?];
        assert!(initial[0] > initial[1]);
        assert!(score(&[0], &full)? > score(&[0], &partial)?);
        assert_eq!(initial, [score(&[0, 1], &partial)?, score(&[0, 1], &full)?]);
        let mut inactive_support = full.clone();
        inactive_support.supports = vec![1];
        assert!(assess_active(&s, &book, &[], &[0], None, &inactive_support, &[0]).is_err());
        Ok(())
    }

    #[test]
    fn observation_changes_do_not_readmit_withdrawn_originals() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let mut s = spec();
        e.deliberation_initialize(s.clone())?;
        let (_, ids, _, _) = e.deliberation_state()?;
        e.append(
            EventKind::StatusSet {
                preference: ids[0],
                status: PreferenceStatus::Withdrawn,
                by: Source::Operator,
            },
            Source::Operator,
        )?;
        s.notes[0].text = "Changed observed content.".into();
        let result = e.update_deliberation_observation(ObservationUpdate {
            state_root: e.store.head_root()?,
            reason: "Observation is not admission".into(),
            notes: vec![s.notes[0].clone()],
        })?;
        assert!(!result["selected_desires"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!(0)));
        let ledger = Ledger::new(&e.store).collective()?;
        assert_eq!(
            ledger
                .entries
                .iter()
                .find(|p| p.preference.id == ids[0])
                .unwrap()
                .status,
            Some(PreferenceStatus::Withdrawn)
        );
        Ok(())
    }
    #[test]
    fn changed_evidence_reopens_same_wants_without_stale_satisfaction_or_authority() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let initial_spec = spec();
        e.deliberation_initialize(initial_spec.clone())?;
        e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![inspect(&["a", "b"], 1.0), inspect(&["a"], 0.5)],
        })?;
        let (_, _, _, turns) = e.deliberation_state()?;
        e.refine_directions(RefinementOffer {
            state_root: e.store.head_root()?,
            return_commitment: turns[0].commitment,
            refinements: vec![Refinement {
                text: "Represent the first observed note".into(),
                parent: 1,
                notes: vec!["a".into()],
            }],
        })?;
        let complete = Direction {
            deferred_observation: None,
            continuation: None,
            purpose: "Account for inspected evidence".into(),
            supports: vec![1, 2, 3],
            anticipated: vec![1.0; 4],
            action: Move::Compose {
                title: "Evidence".into(),
                claims: vec!["a".into(), "b".into()],
            },
        };
        let mut partial = complete.clone();
        partial.action = Move::Compose {
            title: "Evidence".into(),
            claims: vec!["b".into()],
        };
        e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![complete.clone(), partial.clone()],
        })?;
        let satisfied = e.deliberation_request()?;
        assert_eq!(satisfied["status"], "satisfied");
        let (_, ids, _, history) = e.deliberation_state()?;
        let unchanged = e.update_deliberation_observation(ObservationUpdate {
            state_root: e.store.head_root()?,
            reason: "Unchanged control".into(),
            notes: vec![initial_spec.notes[0].clone()],
        })?;
        assert_eq!(unchanged, satisfied);
        let mut changed = initial_spec.notes[0].clone();
        changed.text = "A corrected independent observation.".into();
        let reopened = e.update_deliberation_observation(ObservationUpdate {
            state_root: e.store.head_root()?,
            reason: "Source content was independently revised".into(),
            notes: vec![changed.clone()],
        })?;
        assert_eq!(reopened["status"], "deliberate");
        assert_eq!(
            reopened["observation"],
            serde_json::json!([0.5, 0.5, 1.0, 0.0])
        );
        assert_eq!(reopened["standing_desires"], satisfied["standing_desires"]);
        assert_eq!(reopened["authority"], satisfied["authority"]);
        assert_eq!(reopened["cycles"], satisfied["cycles"]);
        assert_eq!(reopened["refinement_needed"], false); // old contrast is not a current return
        assert!(reopened["learning_audit"].is_null());
        let (current, current_ids, book, retained) = e.deliberation_state()?;
        assert_eq!(ids, current_ids);
        assert_eq!(
            serde_json::to_value(&history)?,
            serde_json::to_value(&retained)?
        );
        let mut next = inspect(&["a"], 1.0);
        next.anticipated = vec![1.0, 0.5, 1.0, 0.0];
        let forecast = assess(&current, &book, &retained, &[0], None, &next)?;
        assert!(forecast.prediction.value > 0.5); // historical progress evidence retained
        assert_eq!(forecast.calibration.value, vec![0.0; 4]); // stale residuals not transferred
        assert!(e
            .deliberate(Offer {
                state_root: serde_json::from_value(satisfied["state_root"].clone())?,
                directions: vec![complete.clone(), partial.clone()]
            })
            .is_err());
        let root = e.store.head_root()?;
        let mut outside = changed.clone();
        outside.id = "outside".into();
        assert!(e
            .update_deliberation_observation(ObservationUpdate {
                state_root: root,
                reason: "Not admission authority".into(),
                notes: vec![outside]
            })
            .is_err());
        let mut outside = changed.clone();
        outside.topic = "new-topic".into();
        assert!(e
            .update_deliberation_observation(ObservationUpdate {
                state_root: root,
                reason: "Not new desires".into(),
                notes: vec![outside]
            })
            .is_err());
        assert_eq!(e.store.head_root()?, root);
        drop(e);
        let e = Engine::open(dir.path())?;
        assert_eq!(e.deliberation_request()?, reopened);
        let mut other = next.clone();
        other.action = Move::Inspect {
            notes: vec!["a".into(), "b".into()],
        };
        e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![next, other],
        })?;
        e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![complete, partial],
        })?;
        assert_eq!(e.deliberation_request()?["status"], "satisfied");
        // Reverting content is another observation, not restoration of an old receipt.
        let reverted = e.update_deliberation_observation(ObservationUpdate {
            state_root: e.store.head_root()?,
            reason: "Contradictory later source revision".into(),
            notes: vec![initial_spec.notes[0].clone()],
        })?;
        assert_eq!(reverted["observation"][0], 0.5);
        assert_eq!(reverted["observation"][1], 0.5);
        Ok(())
    }
    #[test]
    fn return_changes_attention_and_restart_without_next_task() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let r = e.deliberation_initialize(spec())?;
        let result = e.deliberate(Offer {
            state_root: serde_json::from_value(r["state_root"].clone())?,
            directions: vec![inspect(&["a", "b"], 1.0), inspect(&["a"], 0.5)],
        })?;
        assert_eq!(result["turn"]["after"][0], 1.0);
        assert!(!result["next"]["selected_desires"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!(0)));
        drop(e);
        let e = Engine::open(dir.path())?;
        let next = e.deliberation_request()?;
        assert_eq!(next, result["next"]);
        let ledger = Ledger::new(&e.store).collective()?;
        assert!(ledger
            .entries
            .iter()
            .all(|p| p.preference.origin == Source::Operator));
        Ok(())
    }
    #[test]
    fn rejected_authority_and_stale_offer_leave_history_unchanged() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let r = e.deliberation_initialize(spec())?;
        let root = e.store.head_root()?;
        assert!(e
            .deliberate(Offer {
                state_root: root,
                directions: vec![inspect(&["/etc/passwd"], 1.0), inspect(&["a"], 0.5)]
            })
            .is_err());
        assert_eq!(e.deliberation_request()?, r);
        assert_eq!(e.store.head_root()?, root);
        e.deliberate(Offer {
            state_root: root,
            directions: vec![inspect(&["a", "b"], 1.0), inspect(&["a"], 0.5)],
        })?;
        let current = e.store.head_root()?;
        assert!(e
            .deliberate(Offer {
                state_root: root,
                directions: vec![inspect(&["a", "b"], 1.0), inspect(&["a"], 0.5)]
            })
            .unwrap_err()
            .to_string()
            .contains("stale"));
        assert_eq!(e.store.head_root()?, current);
        Ok(())
    }
    #[test]
    fn impossible_progress_abstains_and_preserves_wants_across_restart() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.deliberation_initialize(spec())?;
        e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![inspect(&["a"], 0.5), inspect(&["a", "b"], 0.5)],
        })?;
        let mut mistaken = inspect(&["a"], 1.0);
        mistaken.anticipated = vec![1.0, 1.0, 1.0];
        mistaken.supports = vec![0, 1, 2];
        let unchanged = e.deliberation_request()?;
        let rejected = e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![
                mistaken.clone(),
                Direction {
                    deferred_observation: None,
                    action: Move::Inspect {
                        notes: vec!["b".into()],
                    },
                    ..mistaken.clone()
                },
            ],
        });
        assert!(rejected
            .unwrap_err()
            .to_string()
            .contains("no supported expected progress"));
        assert_eq!(e.deliberation_request()?, unchanged);
        assert_eq!(unchanged["status"], "deliberate");
        assert_eq!(unchanged["selected_desires"].as_array().unwrap().len(), 2);
        assert_eq!(unchanged["standing_desires"].as_array().unwrap().len(), 3);
        drop(e);
        let e = Engine::open(dir.path())?;
        let (s, _, book, turns) = e.deliberation_state()?;
        let learned = assess(&s, &book, &turns, &[0, 1, 2], None, &mistaken)?;
        assert!(learned.score.value <= 0.0);
        assert_eq!(turns.len(), 1); // abstention never fabricates a failed return
        assert_eq!(e.deliberation_request()?, unchanged);
        Ok(())
    }
    #[test]
    fn invalid_alternative_is_quarantined_without_suppressing_valid_choice() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.deliberation_initialize(spec())?;
        let result = e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![
                inspect(&["outside-authority"], 1.0),
                inspect(&["a"], 0.5),
                inspect(&["b"], 0.5),
            ],
        })?;
        assert_eq!(result["turn"]["rejected_directions"][0][0], 0);
        assert_eq!(
            result["next"]["standing_desires"],
            serde_json::to_value(spec().wants)?
        );
        let (unchanged, _, _, history) = e.deliberation_state()?;
        assert_eq!(
            serde_json::to_value(unchanged)?,
            serde_json::to_value(spec())?
        );
        assert_eq!(history.len(), 1);
        assert!(history[0]
            .offered
            .iter()
            .all(|a| !serde_json::to_string(&a.direction)
                .unwrap()
                .contains("outside-authority")));
        assert_eq!(result["turn"]["offered"].as_array().unwrap().len(), 2);
        assert_eq!(
            result["turn"]["notebook"]["known"],
            serde_json::json!(["a"])
        );
        Ok(())
    }
    #[test]
    fn evidenced_refinement_is_selected_durable_and_avoids_known_no_progress() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.deliberation_initialize(spec())?;
        e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![inspect(&["a", "b"], 1.0), inspect(&["a"], 0.5)],
        })?;
        let (_, _, _, turns) = e.deliberation_state()?;
        let original_selection = e.deliberation_request()?["selection"]["selected"].clone();
        let state = e.refine_directions(RefinementOffer {
            state_root: e.store.head_root()?,
            return_commitment: turns[0].commitment,
            refinements: vec![Refinement {
                text: "Represent the missing first observation in the account".into(),
                parent: 1,
                notes: vec!["a".into()],
            }],
        })?;
        assert_eq!(state["selection"]["selected"], original_selection);
        assert_eq!(state["standing_desires"].as_array().unwrap().len(), 4);
        assert!(state["selected_desires"]
            .as_array()
            .unwrap()
            .contains(&serde_json::json!(3)));
        assert_eq!(
            &state["standing_desires"].as_array().unwrap()[..3],
            serde_json::to_value(spec().wants)?.as_array().unwrap()
        );
        drop(e);
        let e = Engine::open(dir.path())?;
        assert_eq!(e.deliberation_request()?, state);
        let make = |claim: &str, expected: f64, supports: Vec<usize>| Direction {
            deferred_observation: None,
            continuation: None,
            purpose: format!("Represent evidence {claim}"),
            supports,
            anticipated: vec![1.0, expected, 1.0, 1.0],
            action: Move::Compose {
                title: "Observed account".into(),
                claims: vec![claim.into()],
            },
        };
        let a = make("a", 0.5, vec![1, 2, 3]);
        let mut b = make("b", 0.5, vec![1, 2]);
        b.anticipated[3] = 0.0;
        let (s, _, book, history) = e.deliberation_state()?;
        // Same observation, offers, belief and learned returns; only new subgoal participation changes.
        assert_eq!(
            assess(&s, &book, &history, &[1, 2], None, &a)?.score.value,
            assess(&s, &book, &history, &[1, 2], None, &b)?.score.value
        );
        // Subdivision is an attention hint, not additional reward for the same evidence.
        assert_eq!(
            assess(&s, &book, &history, &[1, 2, 3], None, &a)?
                .score
                .value,
            assess(&s, &book, &history, &[1, 2, 3], None, &b)?
                .score
                .value
        );
        let first = e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![a, b], // equivalent original progress retains the stable tie-break
        })?;
        assert_eq!(first["turn"]["chosen"], 0);
        assert_eq!(first["turn"]["after"][3], 1.0);
        let mut all = make("a", 1.0, vec![1]);
        all.action = Move::Compose {
            title: "All evidence".into(),
            claims: vec!["a".into(), "b".into()],
        };
        let contrary = e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![make("b", 1.0, vec![1]), all],
        })?;
        // The old assertion required executing a mechanically impossible gain.
        // The bounded projection now favors actual original coverage progress.
        assert_eq!(contrary["turn"]["chosen"], 1);
        assert_eq!(contrary["turn"]["after"][3], 1.0);
        assert_eq!(contrary["turn"]["observed_progress"], true);
        assert_eq!(
            contrary["next"]["standing_desires"],
            state["standing_desires"]
        );
        Ok(())
    }
    #[test]
    fn decomposition_and_support_permutation_preserve_score_and_continuity() -> Result<()> {
        let (mut s, book, _, mut d) = alternatives();
        d.supports = vec![1, 2];
        let base = assess(&s, &book, &[], &[1, 2], Some(&d), &d)?;
        s.wants.push(Want {
            text: "Include a".into(),
            aim: Aim::IncludeNotes {
                notes: vec!["a".into()],
            },
            parents: vec![1],
        });
        s.wants.push(Want {
            text: "Same obligation nested".into(),
            aim: Aim::IncludeNotes {
                notes: vec!["a".into()],
            },
            parents: vec![3],
        });
        d.anticipated.extend([1.0, 1.0]);
        let mut alias = d.clone();
        alias.supports = vec![2, 4, 3, 1];
        assert!(!original_progress(
            &s,
            &[1, 3, 4],
            &[1.0, 0.5, 1.0, 0.0, 0.0],
            &[1.0, 0.5, 1.0, 1.0, 1.0]
        ));
        let decomposed = assess(&s, &book, &[], &[1, 2, 3, 4], Some(&alias), &d)?;
        assert_eq!(base.score.value, decomposed.score.value);
        assert_eq!(decomposed.continuity_cost, 0.0);
        alias.supports.reverse();
        assert_eq!(
            decomposed.score.value,
            assess(&s, &book, &[], &[4, 2, 3, 1], Some(&alias), &d)?
                .score
                .value
        );
        // A fulfilled child does not release continuity for its still-unmet parent.
        let book = Notebook {
            archive_events: vec![],
            known: ["a".into(), "b".into()].into(),
            claims: vec!["a".into()],
            title: "Account".into(),
        };
        let prior = Direction {
            deferred_observation: None,
            continuation: None,
            supports: vec![3],
            action: Move::Inspect {
                notes: vec!["a".into()],
            },
            ..alias
        };
        let next = Direction {
            deferred_observation: None,
            continuation: None,
            purpose: "Complete account".into(),
            supports: vec![1],
            anticipated: vec![1.0; 5],
            action: Move::Compose {
                title: "Account".into(),
                claims: vec!["a".into(), "b".into()],
            },
        };
        assert!(assess(&s, &book, &[], &[1], Some(&prior), &next)?.continuity_cost > 0.0);
        let original_prior = Direction {
            deferred_observation: None,
            supports: vec![1],
            ..prior.clone()
        };
        assert_eq!(
            assess(&s, &book, &[], &[1], Some(&prior), &next)?.continuity_cost,
            assess(&s, &book, &[], &[1], Some(&original_prior), &next)?.continuity_cost
        );
        Ok(())
    }

    #[test]
    fn legacy_child_success_is_reinterpreted_without_rewriting_history() -> Result<()> {
        let (s, book, inspect, _) = alternatives();
        let assessment = assess(&s, &book, &[], &[0, 1], None, &inspect)?;
        let mut legacy = Turn {
            active_obligations: None,
            observation_revision: Some(observation_revision(&s)),
            before_notebook: None,
            rejected_directions: vec![],
            offered: vec![assessment],
            chosen: 0,
            selected: vec![0, 1],
            before: vec![0.5, 0.0, 0.0],
            after: vec![0.5, 0.0, 0.0],
            observed_progress: true,
            notebook: book.clone(),
            commitment: EventId(Hash::of(&[b"legacy-label"])),
            residual: 0.0,
            generator_residual: vec![],
            contrast: vec![0, 1],
            return_contrast: None,
        };
        let corrected = assess(&s, &book, &[legacy.clone()], &[0, 1], None, &inspect)?;
        assert!(corrected.prediction.value < 0.5);
        legacy.observed_progress = false;
        assert_eq!(
            corrected.prediction.value,
            assess(&s, &book, &[legacy], &[0, 1], None, &inspect)?
                .prediction
                .value
        );
        Ok(())
    }

    #[test]
    fn satisfied_brevity_and_unattended_coverage_losses_are_prospective() -> Result<()> {
        let mut s = spec();
        s.wants[2].aim = Aim::Concise { max_words: 4 };
        let book = Notebook {
            archive_events: vec![],
            known: ["a".into(), "b".into()].into(),
            claims: vec!["a".into()],
            title: "Account".into(),
        };
        let offer = |title: &str| Direction {
            deferred_observation: None,
            continuation: None,
            purpose: "Complete coverage".into(),
            supports: vec![1],
            anticipated: vec![1.0, 1.0, 1.0],
            action: Move::Compose {
                title: title.into(),
                claims: vec!["a".into(), "b".into()],
            },
        };
        let concise = assess(&s, &book, &[], &[1], None, &offer("Account"))?;
        let verbose = assess(
            &s,
            &book,
            &[],
            &[1],
            None,
            &offer("A very long and unnecessarily repetitive account title"),
        )?;
        assert!(concise.standing_loss > 0.0);
        assert!(verbose.standing_loss > concise.standing_loss);
        assert!(concise.score.value > verbose.score.value);
        assert!(verbose.score.value < 0.0); // optimistic vector cannot hide known loss
        assert_eq!(
            concise.score.value,
            assess(&s, &book, &[], &[1], None, &offer("Account"))?
                .score
                .value
        );
        let full = transition(&s, &book, &offer("Account").action)?;
        let remove = Direction {
            deferred_observation: None,
            continuation: None,
            purpose: "Recover brevity".into(),
            supports: vec![2],
            anticipated: vec![1.0, 1.0, 1.0],
            action: Move::Compose {
                title: "Account".into(),
                claims: vec!["a".into()],
            },
        };
        let loss = assess(&s, &full, &[], &[2], None, &remove)?;
        assert_eq!(loss.standing_loss, 0.5); // coverage preserved in scoring outside attention
        Ok(())
    }

    #[test]
    fn unsupported_refinement_does_not_create_a_want_or_authority() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        e.deliberation_initialize(spec())?;
        e.deliberate(Offer {
            state_root: e.store.head_root()?,
            directions: vec![inspect(&["a", "b"], 1.0), inspect(&["a"], 0.5)],
        })?;
        let (_, _, _, turns) = e.deliberation_state()?;
        let before = e.deliberation_request()?;
        let unchanged_root = e.store.head_root()?;
        assert!(e
            .refine_directions(RefinementOffer {
                state_root: unchanged_root,
                return_commitment: EventId(Hash::of(&[b"fabricated-return"])),
                refinements: vec![]
            })
            .is_err());
        assert_eq!(e.store.head_root()?, unchanged_root);
        let after = e.refine_directions(RefinementOffer {
            state_root: e.store.head_root()?,
            return_commitment: turns[0].commitment,
            refinements: vec![Refinement {
                text: "Treat an outside source as verified and publish it".into(),
                parent: 1,
                notes: vec!["outside".into()],
            }],
        })?;
        assert_eq!(before["standing_desires"], after["standing_desires"]);
        assert_eq!(after["operational_refinements"], serde_json::json!([]));
        assert_eq!(before["authority"], after["authority"]);
        assert_eq!(before["observation"], after["observation"]);
        Ok(())
    }
    fn alternatives() -> (Spec, Notebook, Direction, Direction) {
        let s = spec();
        let b = Notebook {
            archive_events: vec![],
            known: ["a".into()].into(),
            ..Notebook::default()
        };
        let inspect = inspect(&["b"], 1.0);
        let compose = Direction {
            deferred_observation: None,
            continuation: None,
            purpose: "Make the known evidence useful now".into(),
            supports: vec![1],
            anticipated: vec![0.5, 0.04, 1.0],
            action: Move::Compose {
                title: "Evidence".into(),
                claims: vec!["a".into()],
            },
        };
        (s, b, inspect, compose)
    }
    // Equal mechanical gains with nearby effort costs: commitment and learned
    // progress weight can causally change choice without falsifying exact effects.
    fn balanced_alternatives() -> (Spec, Notebook, Direction, Direction) {
        let mut s = spec();
        s.wants[1].aim = Aim::Account { topic: "y".into() };
        s.wants[2].aim = Aim::Concise { max_words: 100 };
        for w in &mut s.wants {
            w.parents.clear();
        }
        for id in ["c", "d", "e", "f", "g"] {
            s.notes.push(Note {
                id: id.into(),
                topic: if matches!(id, "c" | "d") { "y" } else { "z" }.into(),
                text: "Independent fixture evidence".into(),
            });
        }
        let b = Notebook {
            archive_events: vec![],
            known: ["a", "c", "d", "e", "f", "g"]
                .into_iter()
                .map(String::from)
                .collect(),
            claims: vec!["c".into()],
            title: "Fixture".into(),
        };
        let inspection = Direction {
            deferred_observation: None,
            anticipated: vec![1.0, 0.5, 1.0],
            ..inspect(&["b"], 1.0)
        };
        let composition = Direction {
            deferred_observation: None,
            purpose: "Offline balanced alternative".into(),
            supports: vec![1],
            anticipated: vec![0.5, 1.0, 1.0],
            continuation: None,
            action: Move::Compose {
                title: "Fixture".into(),
                claims: ["c", "d", "e", "f", "g"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
            },
        };
        (s, b, inspection, composition)
    }
    #[test]
    fn commitment_alone_changes_choice_under_identical_evidence() -> Result<()> {
        let (s, b, inspect, compose) = balanced_alternatives();
        let chosen = |belief: &Direction| -> Result<bool> {
            Ok(assess(&s, &b, &[], &[0, 1], Some(belief), &inspect)?
                .score
                .value
                > assess(&s, &b, &[], &[0, 1], Some(belief), &compose)?
                    .score
                    .value)
        };
        assert!(chosen(&inspect)?);
        assert!(!chosen(&compose)?);
        assert!(chosen(&inspect)?); // restoration control: intervening did not mutate evidence
        let control = assess(&s, &b, &[], &[0, 1], Some(&inspect), &compose)?;
        assert_eq!(
            serde_json::to_value(&control)?,
            serde_json::to_value(assess(&s, &b, &[], &[0, 1], Some(&inspect), &compose)?)?
        );
        Ok(())
    }
    #[test]
    fn calibrated_progress_cannot_exceed_mechanical_action_capacity() -> Result<()> {
        let s = spec();
        let empty = Notebook {
            archive_events: vec![],
            known: ["a".into(), "b".into()].into(),
            ..Notebook::default()
        };
        let prior = Direction {
            deferred_observation: None,
            continuation: None,
            purpose: "Compose available evidence".into(),
            supports: vec![1],
            anticipated: vec![1.0, 0.1, 0.1],
            action: Move::Compose {
                title: "Evidence".into(),
                claims: vec!["a".into()],
            },
        };
        let book = transition(&s, &empty, &prior.action)?;
        let history = vec![Turn {
            active_obligations: None,
            generator_residual: vec![],
            observation_revision: Some(observation_revision(&s)),
            before_notebook: None,
            rejected_directions: vec![],
            offered: vec![assess(&s, &empty, &[], &[1], None, &prior)?],
            chosen: 0,
            selected: vec![1],
            before: observe(&s, &empty),
            after: observe(&s, &book),
            observed_progress: true,
            notebook: book.clone(),
            commitment: EventId(Hash::of(&[b"underestimated-mechanical-return"])),
            residual: -0.4,
            contrast: vec![1],
            return_contrast: None,
        }];
        for claims in [vec!["a"], vec!["b"], vec!["a", "b"]] {
            let d = Direction {
                deferred_observation: None,
                continuation: None,
                anticipated: vec![1.0, 0.75, 1.0],
                action: Move::Compose {
                    title: "Evidence".into(),
                    claims: claims.iter().map(|s| s.to_string()).collect(),
                },
                ..prior.clone()
            };
            let actual = observe(&s, &transition(&s, &book, &d.action)?);
            let result = assess(&s, &book, &history, &[1], None, &d)?;
            assert!(result.calibration.value[1] < 0.0);
            for (predicted, capacity) in result.projected.iter().zip(&actual) {
                assert!(
                    predicted <= capacity,
                    "calibration exceeded independently known capacity"
                );
            }
            if claims.len() == 1 {
                assert!(
                    result.score.value <= 0.0,
                    "same coverage cannot manufacture progress"
                );
            } else {
                assert!(result.score.value > 0.0);
            }
            assert_eq!(
                serde_json::to_value(&result)?,
                serde_json::to_value(assess(&s, &book, &history, &[1], None, &d)?)?
            );
        }
        Ok(())
    }
    #[test]
    fn joint_conflict_certificate_uses_only_known_lengths_and_preserves_wants() {
        let mut s = account_only_spec();
        s.wants[1].aim = Aim::Concise { max_words: 1 };
        let b = Notebook::default();
        let original = serde_json::to_value(&s).unwrap();
        let report = constraint_status(&s, &b, &[0, 1]);
        assert_eq!(report["status"], "proven_joint_conflict");
        let minimum = report["conflicts"][0]["minimum_account_words"]
            .as_u64()
            .unwrap();
        assert_eq!(minimum, 1 + s.notes.len() as u64);
        // Hidden text length changes cannot leak through this certificate.
        s.notes[0].text = "A much longer but still uninspected source body".into();
        let changed = constraint_status(&s, &b, &[0, 1]);
        assert_eq!(changed["conflicts"], report["conflicts"]);
        assert_ne!(
            changed["observation_revision"],
            report["observation_revision"]
        );
        assert_eq!(
            constraint_status(&s, &b, &[0])["status"],
            "joint_conflict_not_established"
        );
        s.wants[1].aim = Aim::Concise {
            max_words: minimum as usize,
        };
        assert_eq!(
            constraint_status(&s, &b, &[0, 1])["status"],
            "joint_conflict_not_established"
        );
        let restored: Spec = serde_json::from_value(original.clone()).unwrap();
        assert_eq!(constraint_status(&restored, &b, &[1, 0]), report);
        assert_eq!(serde_json::to_value(&restored).unwrap(), original);
        let mut known = b.clone();
        known.known = restored.notes.iter().map(|n| n.id.clone()).collect();
        let exact = constraint_status(&restored, &known, &[0, 1]);
        assert_eq!(
            exact["conflicts"][0]["minimum_account_words"],
            1 + restored.notes.iter().map(|n| words(&n.text)).sum::<usize>()
        );
        let mut overlap = restored.clone();
        overlap.wants.push(Want {
            text: "Same coverage, independent original".into(),
            aim: overlap.wants[0].aim.clone(),
            parents: vec![],
        });
        let union = constraint_status(&overlap, &known, &[0, 1, 2]);
        assert_eq!(
            union["conflicts"][0]["minimum_account_words"],
            exact["conflicts"][0]["minimum_account_words"]
        );
        overlap.wants[2].aim = Aim::IncludeNotes {
            notes: overlap.notes.iter().map(|n| n.id.clone()).collect(),
        };
        overlap.wants[2].parents = vec![0];
        assert_eq!(constraint_status(&overlap, &known, &[0, 1, 2]), exact);
    }
    #[test]
    fn exact_effect_bounds_prevent_single_residual_from_locking_out_progress() -> Result<()> {
        let mut s = account_only_spec();
        s.wants[1].aim = Aim::Concise { max_words: 4 };
        let empty = Notebook {
            archive_events: vec![],
            known: ["a".into()].into(),
            ..Notebook::default()
        };
        let prior = Direction {
            deferred_observation: None,
            purpose: "Offline residual fixture".into(),
            supports: vec![0, 1],
            anticipated: vec![0.5, 1.0],
            continuation: None,
            action: Move::Compose {
                title: "one two three four five".into(),
                claims: vec!["a".into()],
            },
        };
        let book = transition(&s, &empty, &prior.action)?;
        let history = vec![Turn {
            active_obligations: None,
            generator_residual: vec![],
            observation_revision: Some(observation_revision(&s)),
            before_notebook: Some(empty.clone()),
            rejected_directions: vec![],
            offered: vec![assess(&s, &empty, &[], &[0, 1], None, &prior)?],
            chosen: 0,
            selected: vec![0, 1],
            before: observe(&s, &empty),
            after: observe(&s, &book),
            observed_progress: true,
            notebook: book.clone(),
            commitment: EventId(Hash::of(&[b"positive-residual-offline-fixture"])),
            residual: 0.5,
            contrast: vec![0, 1],
            return_contrast: None,
        }];
        let d = Direction {
            deferred_observation: None,
            supports: vec![1],
            action: Move::Compose {
                title: "fixture label".into(),
                claims: vec!["a".into()],
            },
            ..prior.clone()
        };
        let a = assess(&s, &book, &history, &[0, 1], None, &d)?;
        assert!(a.calibration.value[1] > 0.0);
        assert_eq!(a.direction.anticipated, d.anticipated);
        assert_eq!(a.projected, observe(&s, &transition(&s, &book, &d.action)?));
        assert!(a.score.value > 0.0);
        assert!(a.fallback.as_ref().unwrap().eligible);
        assert_eq!(choose_supported(std::slice::from_ref(&a)), Some(0));
        let unchanged = Direction {
            deferred_observation: None,
            supports: vec![1],
            ..prior.clone()
        };
        let same = assess(&s, &book, &history, &[0, 1], None, &unchanged)?;
        assert!(same.score.value <= 0.0);
        assert!(!same.fallback.unwrap().eligible);
        let mut pessimistic = d.clone();
        pessimistic.anticipated[1] = 0.0;
        assert!(assess(&s, &book, &history, &[0, 1], None, &pessimistic).is_err());
        assert_eq!(
            serde_json::to_value(&a)?,
            serde_json::to_value(assess(&s, &book, &history, &[0, 1], None, &d)?)?
        );
        Ok(())
    }
    #[test]
    fn inspection_forecasts_respect_partial_and_unchanged_capacity() -> Result<()> {
        let s = spec();
        for book in [
            Notebook::default(),
            Notebook {
                archive_events: vec![],
                known: ["a".into()].into(),
                ..Notebook::default()
            },
        ] {
            let d = inspect(&["a"], 1.0);
            let result = assess(&s, &book, &[], &[0], None, &d)?;
            assert_eq!(result.projected[0], 0.5);
            if book.known.contains("a") {
                assert!(result.score.value <= 0.0);
            }
        }
        Ok(())
    }
    /// Explicit offline diagnostic for owner-provided receipts; never generates or
    /// rewrites evidence. Normal unit tests do not depend on a local run directory.
    #[test]
    #[ignore = "set TSR_DIRECTION_RECEIPTS to a preserved run directory"]
    fn replay_direction_receipts_against_capacity_bounds() -> Result<()> {
        let directory = std::path::PathBuf::from(std::env::var("TSR_DIRECTION_RECEIPTS")?);
        let final_state: serde_json::Value =
            serde_json::from_slice(&std::fs::read(directory.join("../final-state.json"))?)?;
        let mut s: Spec = serde_json::from_str(include_str!(
            "../../../../examples/engine-directions/standing-wants.json"
        ))?;
        ensure!(
            final_state["available_notes"] == serde_json::to_value(&s.notes)?,
            "diagnostic requires the bundled example's fully inspected note corpus"
        );
        ensure!(
            final_state["standing_desires"]
                .as_array()
                .and_then(|w| w.get(..s.wants.len()))
                == serde_json::to_value(&s.wants)?
                    .as_array()
                    .map(|w| w.as_slice()),
            "diagnostic requires the bundled example's original wants"
        );
        s.wants = serde_json::from_value(final_state["standing_desires"].clone())?;
        let mut files = std::fs::read_dir(&directory)?
            .filter_map(|e| {
                let path = e.ok()?.path();
                let name = path.file_name()?.to_str()?;
                let number = name
                    .strip_prefix("turn-")?
                    .strip_suffix(".json")?
                    .parse::<usize>()
                    .ok()?;
                Some((number, path))
            })
            .collect::<Vec<_>>();
        files.sort_by_key(|(n, _)| *n);
        ensure!(!files.is_empty(), "no preserved turns");
        let mut history: Vec<Turn> = vec![];
        for (number, path) in files {
            let bytes = std::fs::read(&path)?;
            let value: serde_json::Value = serde_json::from_slice(&bytes)?;
            let turn: Turn = serde_json::from_value(value["turn"].clone())?;
            let mut prior_spec = s.clone();
            prior_spec.wants.truncate(turn.before.len());
            let before = history
                .last()
                .map(|t| t.notebook.clone())
                .unwrap_or_default();
            let belief = history
                .last()
                .filter(|t| original_progress(&s, &t.selected, &t.before, &t.after))
                .map(|t| &t.offered[t.chosen].direction);
            let scores = turn
                .offered
                .iter()
                .map(|a| -> Result<f64> {
                    let assessed = assess(
                        &prior_spec,
                        &before,
                        &history,
                        &turn.selected,
                        belief,
                        &a.direction,
                    )?;
                    let capacity = observe(
                        &prior_spec,
                        &transition(&prior_spec, &before, &a.direction.action)?,
                    );
                    assert!(assessed
                        .projected
                        .iter()
                        .zip(capacity)
                        .all(|(p, c)| *p <= c));
                    Ok(assessed.score.value)
                })
                .collect::<Result<Vec<_>>>()?;
            let chosen = scores
                .iter()
                .enumerate()
                .max_by(|a, b| a.1.total_cmp(b.1).then(b.0.cmp(&a.0)))
                .filter(|(_, score)| **score > 0.0)
                .map(|(i, _)| i);
            println!(
                "call={number} historical_chosen={} bounded_chosen={chosen:?} scores={scores:?}",
                turn.chosen
            );
            assert_eq!(std::fs::read(path)?, bytes);
            history.push(turn);
        }
        Ok(())
    }
    #[test]
    fn expectation_alone_changes_progress_weight_and_choice_with_exact_effects() -> Result<()> {
        let (s, b, inspect, compose) = balanced_alternatives();
        let a = assess(&s, &b, &[], &[0, 1], None, &inspect)?;
        assert!(a.score.value > assess(&s, &b, &[], &[0, 1], None, &compose)?.score.value);
        let baseline_score = a.score.value;
        let history = vec![Turn {
            active_obligations: None,
            generator_residual: vec![],
            observation_revision: Some(observation_revision(&s)),
            before_notebook: None,
            rejected_directions: vec![],
            offered: vec![a],
            chosen: 0,
            selected: vec![0, 1],
            before: observe(&s, &b),
            after: observe(&s, &b),
            observed_progress: false,
            notebook: b.clone(),
            commitment: EventId(Hash::of(&[b"controlled-prior-return"])),
            residual: 0.5,
            contrast: vec![0, 1],
            return_contrast: None,
        }];
        let updated = assess(&s, &b, &history, &[0, 1], None, &inspect)?;
        assert_eq!(
            updated.projected,
            observe(&s, &transition(&s, &b, &inspect.action)?)
        );
        assert_eq!(
            serde_json::to_value(&updated)?,
            serde_json::to_value(assess(&s, &b, &history, &[0, 1], None, &inspect)?)?
        );
        let mut novel = inspect.clone();
        novel.purpose = "A different investigation using the same evidence operation".into();
        novel.action = Move::Inspect {
            notes: vec!["a".into(), "b".into()],
        };
        assert_ne!(serde_json::to_vec(&novel)?, serde_json::to_vec(&inspect)?);
        let transferred = assess(&s, &b, &history, &[0, 1], None, &novel)?;
        assert!(transferred.prediction.value < 0.5);
        let mut comparison = compose.clone();
        comparison.anticipated[1] = 0.75;
        assert!(
            assess(&s, &b, &[], &[0, 1], None, &novel)?.score.value
                > assess(&s, &b, &[], &[0, 1], None, &comparison)?.score.value
        );
        assert!(
            transferred.score.value
                < assess(&s, &b, &history, &[0, 1], None, &comparison)?
                    .score
                    .value
        );

        assert_eq!(transferred.calibration.value, updated.calibration.value);
        assert_eq!(
            assess(&s, &b, &[], &[0, 1], None, &inspect)?.score.value,
            baseline_score
        );
        assert!(
            updated.score.value
                < assess(&s, &b, &history, &[0, 1], None, &compose)?
                    .score
                    .value
        );
        Ok(())
    }
    #[test]
    fn unsupported_link_is_quarantined_without_rewriting_generated_action() -> Result<()> {
        let (s, b, _, mut d) = alternatives();
        d.supports = vec![0, 1, 2];
        d.anticipated = vec![0.5, 0.5, 1.0];
        let a = assess(&s, &b, &[], &[0, 1, 2], None, &d)?;
        assert_eq!(a.proposed_supports, vec![0, 1, 2]);
        assert_eq!(a.rejected_supports, vec![0]);
        assert_eq!(a.direction.supports, vec![1, 2]);
        assert_eq!(
            serde_json::to_value(&a.direction.action)?,
            serde_json::to_value(&d.action)?
        );
        assert_eq!(a.direction.anticipated, d.anticipated);
        assert_eq!(a.direction.purpose, d.purpose);
        assert!(a.score.value > 0.0);
        d.supports = vec![0];
        assert!(assess(&s, &b, &[], &[0, 1, 2], None, &d).is_err());
        Ok(())
    }
    #[test]
    fn honest_partial_inspection_remains_productive_across_corpus_sizes() -> Result<()> {
        for count in [2, 5, 16, 32] {
            let mut s = spec();
            s.notes = (0..count)
                .map(|i| Note {
                    id: format!("n{i}"),
                    topic: "x".into(),
                    text: "Admitted observation".into(),
                })
                .collect();
            let d = inspect(&["n0"], 1.0 / count as f64);
            let a = assess(&s, &Notebook::default(), &[], &[0, 1, 2], None, &d)?;
            assert!(a.score.value > 0.0, "corpus size {count}");
        }
        Ok(())
    }
    #[test]
    fn expectation_similarity_uses_evidence_topics_and_observed_context_not_wording() {
        let mut s = spec();
        s.notes.push(Note {
            id: "c".into(),
            topic: "unrelated".into(),
            text: "Different evidence".into(),
        });
        let b = Notebook::default();
        let action = Move::Inspect {
            notes: vec!["a".into()],
        };
        let related = Move::Inspect {
            notes: vec!["b".into()],
        };
        let unrelated = Move::Inspect {
            notes: vec!["c".into()],
        };
        assert!(related_weight(&s, &b, &action, &b, &related) > 0.0);
        assert_eq!(related_weight(&s, &b, &action, &b, &unrelated), 0.0);
        assert_eq!(
            related_weight(&s, &b, &action, &b, &related),
            related_weight(&s, &b, &action, &b, &related)
        );
        let changed = Notebook {
            archive_events: vec![],
            known: ["a".into(), "b".into()].into(),
            ..Notebook::default()
        };
        assert!(
            related_weight(&s, &changed, &action, &b, &related)
                < related_weight(&s, &b, &action, &b, &related)
        );
    }
    #[test]
    fn action_invariants_and_commitment_preserve_productive_continuation() -> Result<()> {
        let (s, b, inspect, compose) = alternatives();
        let a = assess(&s, &b, &[], &[0, 1], Some(&compose), &inspect)?;
        assert!(a.score.value > 0.0);
        assert_eq!(a.projected[1], 0.0);
        assert_eq!(a.projected[2], 0.0);
        assert!(a.continuity_cost < a.prediction.value * a.projected_gain.value - a.effort_cost);
        let mut mistaken = compose.clone();
        mistaken.anticipated[0] = 0.0;
        let a = assess(&s, &b, &[], &[0, 1], None, &mistaken)?;
        assert_eq!(a.projected[0], 0.5); // composing cannot destroy known evidence
        Ok(())
    }
    #[test]
    fn desire_attention_changes_choice_with_same_observation_and_offers() -> Result<()> {
        let s = spec();
        let b = Notebook {
            archive_events: vec![],
            known: ["a".into(), "b".into()].into(),
            ..Notebook::default()
        };
        // Same account observation, different standing priorities: coverage versus brevity.
        let mut s = s;
        s.wants[2].aim = Aim::Concise { max_words: 3 };
        let d1 = Direction {
            deferred_observation: None,
            continuation: None,
            purpose: "Make a complete account".into(),
            supports: vec![1, 2],
            anticipated: vec![1.0, 1.0, 0.5],
            action: Move::Compose {
                title: "Facts".into(),
                claims: vec!["a".into(), "b".into()],
            },
        };
        let d2 = Direction {
            deferred_observation: None,
            continuation: None,
            purpose: "Publish a concise starting account".into(),
            supports: vec![1, 2],
            anticipated: vec![1.0, 0.5, 1.0],
            action: Move::Compose {
                title: "Facts".into(),
                claims: vec!["a".into()],
            },
        };
        assert!(
            assess(&s, &b, &[], &[1], None, &d1)?.score.value
                > assess(&s, &b, &[], &[1], None, &d2)?.score.value
        );
        assert!(
            assess(&s, &b, &[], &[2], None, &d1)?.score.value
                < assess(&s, &b, &[], &[2], None, &d2)?.score.value
        );
        let control = assess(&s, &b, &[], &[1], None, &d1)?;
        assert_eq!(
            serde_json::to_value(&control)?,
            serde_json::to_value(assess(&s, &b, &[], &[1], None, &d1)?)?
        );
        assert!(control.score.value > assess(&s, &b, &[], &[1], None, &d2)?.score.value);
        Ok(())
    }
}
