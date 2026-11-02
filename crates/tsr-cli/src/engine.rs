//! Persisted local full-cycle operationalization of PRD B2/B2.1/B6/B8.
//! No commands or provider calls: the queue domain is a pure local transition.
use anyhow::{bail, ensure, Result};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tsr_domain::*;
use tsr_ledger::{Filter, Ledger};
use tsr_store::Store;

const FACT_SOURCE: &str = "local-queue-cycle@v1";
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Route {
    Bulk,
    Checked,
}
impl Route {
    fn cost(self) -> f64 {
        match self {
            Self::Bulk => 0.05,
            Self::Checked => 0.15,
        }
    }
    fn deliver(self, requested: u32) -> u32 {
        match self {
            Self::Bulk => requested.min(1),
            Self::Checked => requested,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "fact", rename_all = "snake_case")]
enum Fact {
    Initialized {
        batch_size: u32,
    },
    Probe {
        route: Route,
        requested: u32,
        delivered: u32,
    },
    Outcome {
        route: Route,
        requested: u32,
        delivered: u32,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Estimate {
    pub route: Route,
    pub successes: u32,
    pub failures: u32,
    pub prediction: Proxy<f64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct View {
    pub root: Hash,
    pub cycles: usize,
    pub batch_size: u32,
    pub last_commitment: Option<CandidateId>,
    pub last_route: Option<Route>,
    pub estimates: Vec<Estimate>,
    pub unmet_pressure: Proxy<f64>,
    pub preferences: Vec<tsr_ledger::LedgerEntry>,
}
/// A future adapter may suggest these bounded routes; it cannot admit evidence or write state.
pub trait Proposer {
    fn propose(&self, view: &View) -> Vec<Route>;
}
pub struct LocalProposer;
impl Proposer for LocalProposer {
    fn propose(&self, view: &View) -> Vec<Route> {
        // Contrast requests a better-supported alternative to the current commitment.
        let mut routes = vec![Route::Bulk, Route::Checked];
        routes.sort_by(|a, b| {
            let score = |r: &Route| {
                view.estimates
                    .iter()
                    .find(|e| e.route == *r)
                    .unwrap()
                    .prediction
                    .value
                    - r.cost()
            };
            score(b).total_cmp(&score(a))
        });
        if view.unmet_pressure.value > 0.0 {
            if let Some(last) = view.last_route {
                let contradicted = view
                    .estimates
                    .iter()
                    .any(|e| e.route == last && e.failures > 0);
                if contradicted {
                    routes.sort_by_key(|r| *r == last);
                }
            }
        }
        routes
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    pub route: Route,
    pub admitted: bool,
    pub reason: String,
    pub evidence: EventId,
    pub prediction: Proxy<f64>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct Trace {
    pub cycle: usize,
    pub before: View,
    pub possibilities: Vec<Choice>,
    pub tensions: Vec<String>,
    pub chosen: Route,
    pub commitment: EventId,
    pub requested: u32,
    pub delivered: u32,
    pub residual: Proxy<f64>,
    pub unmet: bool,
    pub after: View,
}

pub struct Engine {
    store: Store,
    directory: PathBuf,
    // Decoding only: callers still reread and hash CAS bytes before every hit.
    record_cache: std::cell::RefCell<std::collections::BTreeMap<Hash, adaptive::Record>>,
}
impl Engine {
    pub fn open(directory: &Path) -> Result<Self> {
        fs::create_dir_all(directory.join("cas"))?;
        Ok(Self {
            store: Store::open(&directory.join("events.sqlite"))?,
            directory: directory.into(),
            record_cache: Default::default(),
        })
    }
    fn task() -> TaskId {
        TaskId(Hash::of(&[FACT_SOURCE.as_bytes()]))
    }
    fn events(&self) -> Result<Vec<(i64, Event)>> {
        Ok(self.store.by_task(&Self::task())?)
    }
    fn append(&self, kind: EventKind, source: Source) -> Result<EventId> {
        let history = self.events()?;
        let parents = history.last().map(|(_, e)| vec![e.id]).unwrap_or_default();
        let timestamp = format!(
            "unix-ms:{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_millis()
        );
        let event = Event::new(Self::task(), parents, source, kind, timestamp);
        self.store.append(&event)?;
        Ok(event.id)
    }
    fn blob(&self, bytes: &[u8]) -> Result<Hash> {
        let digest = Hash::of(&[bytes]);
        let path = self.directory.join("cas").join(digest.to_hex());
        match fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&path)
        {
            Ok(mut f) => {
                f.write_all(bytes)?;
                f.sync_all()?;
                fs::File::open(path.parent().unwrap())?.sync_all()?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                ensure!(fs::read(path)? == bytes, "CAS mismatch")
            }
            Err(e) => return Err(e.into()),
        }
        Ok(digest)
    }
    fn fact(&self, fact: &Fact) -> Result<EventId> {
        let digest = self.blob(&serde_json::to_vec(fact)?)?;
        self.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    digest,
                    source: FACT_SOURCE.into(),
                },
            },
            Source::Rules,
        )
    }
    fn read_fact(&self, event: &Event) -> Result<Option<Fact>> {
        if let EventKind::Observed {
            observation: Observation::Evidence { digest, source },
        } = &event.kind
        {
            if source == FACT_SOURCE {
                let bytes = fs::read(self.directory.join("cas").join(digest.to_hex()))?;
                ensure!(Hash::of(&[&bytes]) == *digest, "CAS evidence changed");
                return Ok(Some(serde_json::from_slice(&bytes)?));
            }
        }
        Ok(None)
    }
    fn write<T>(&self, op: impl FnOnce() -> Result<T>) -> Result<T> {
        self.store.connection().execute_batch("BEGIN IMMEDIATE")?;
        match op() {
            Ok(result) => {
                self.store.connection().execute_batch("COMMIT")?;
                Ok(result)
            }
            Err(error) => {
                self.store.connection().execute_batch("ROLLBACK")?;
                Err(error)
            }
        }
    }
    pub fn initialize(&self, batch_size: u32) -> Result<View> {
        ensure!((2..=100).contains(&batch_size), "batch size must be 2..100");
        self.write(|| {
            ensure!(
                self.events()?.is_empty(),
                "already initialized; use step or show to resume"
            );
            self.append(
                EventKind::TaskOpened { task: Self::task() },
                Source::Operator,
            )?;
            let baseline = self.fact(&Fact::Initialized { batch_size })?;
            let contrast = Filter::contrast_from_criteria(
                baseline,
                vec![(Self::criterion(), UnmetKind::WrongBehaviour)],
                vec![baseline],
            );
            self.append(
                EventKind::Observed {
                    observation: Observation::Return {
                        surprise: vec![],
                        contrast: Some(contrast.clone()),
                    },
                },
                Source::Operator,
            )?;
            self.expand(
                &contrast,
                "Deliver each requested batch completely",
                vec![],
                Source::Operator,
            )?;
            self.view()
        })
    }
    fn criterion() -> CriterionId {
        CriterionId(Hash::of(&[b"complete_batch@v1"]))
    }
    fn expand(
        &self,
        contrast: &Contrast,
        text: &str,
        parents: Vec<PreferenceId>,
        origin: Source,
    ) -> Result<()> {
        let preference = Preference::new(text, Scope::Task(Self::task()), parents.clone(), origin)?;
        // Deduplicate the same refinement instead of recruiting an unbounded chain per failure.
        if Ledger::new(&self.store)
            .collective()?
            .entries
            .iter()
            .any(|e| e.preference.id == preference.id)
        {
            return Ok(());
        }
        self.append(
            EventKind::Expanded {
                expansion: Expansion {
                    from_contrast: contrast.id,
                    produced: preference.id,
                    catalyzed_by: parents.clone(),
                    text: preference.text,
                    scope: preference.scope,
                    parents,
                    origin,
                },
            },
            origin,
        )?;
        Ok(())
    }
    pub fn view(&self) -> Result<View> {
        let own_transaction = self.store.connection().is_autocommit();
        if own_transaction {
            self.store.connection().execute_batch("BEGIN")?;
        }
        let result = self.view_snapshot();
        if own_transaction {
            self.store.connection().execute_batch("ROLLBACK")?;
        }
        result
    }
    fn view_snapshot(&self) -> Result<View> {
        let events = self.events()?;
        let mut batch_size = None;
        let mut cycles = 0;
        let mut failures = 0;
        let mut last_commitment = None;
        let mut last_route = None;
        let mut estimates = vec![
            Estimate {
                route: Route::Bulk,
                successes: 0,
                failures: 0,
                prediction: Proxy::new("empirical_success", 0.5),
            },
            Estimate {
                route: Route::Checked,
                successes: 0,
                failures: 0,
                prediction: Proxy::new("empirical_success", 0.5),
            },
        ];
        let mut previous = None;
        for (_, e) in &events {
            ensure!(
                e.schema_version == Event::SCHEMA_VERSION && e.task == Self::task(),
                "event schema/task mismatch"
            );
            ensure!(
                e.parents == previous.into_iter().collect::<Vec<_>>(),
                "broken cycle event chain"
            );
            previous = Some(e.id);
            let canonical = Event::new(
                e.task,
                e.parents.clone(),
                e.source,
                e.kind.clone(),
                e.recorded_at.clone(),
            );
            ensure!(canonical.id == e.id, "event content identity mismatch");
            if let EventKind::Committed { candidate, .. } = e.kind {
                last_commitment = Some(candidate);
            }
            let fact = self.read_fact(e)?;
            if fact.is_some() {
                ensure!(
                    e.source == Source::Rules,
                    "domain evidence must come from local observer"
                );
            }
            match fact {
                Some(Fact::Initialized { batch_size: n }) => {
                    ensure!(
                        batch_size.is_none() && (2..=100).contains(&n),
                        "invalid initialization"
                    );
                    batch_size = Some(n);
                }
                Some(Fact::Outcome {
                    route,
                    requested,
                    delivered,
                }) => {
                    ensure!(
                        Some(requested) == batch_size && delivered <= requested,
                        "invalid outcome"
                    );
                    cycles += 1;
                    last_route = Some(route);
                    let estimate = estimates.iter_mut().find(|e| e.route == route).unwrap();
                    if delivered == requested {
                        estimate.successes += 1;
                    } else {
                        estimate.failures += 1;
                        failures += 1;
                    }
                }
                _ => {}
            }
        }
        for estimate in &mut estimates {
            estimate.prediction.value = (1.0 + f64::from(estimate.successes))
                / (2.0 + f64::from(estimate.successes + estimate.failures));
        }
        Ok(View {
            root: self.store.head_root()?,
            cycles,
            batch_size: batch_size
                .ok_or_else(|| anyhow::anyhow!("not initialized; run engine init"))?,
            last_commitment,
            last_route,
            estimates,
            unmet_pressure: Proxy::new(
                "unmet_pressure",
                f64::from(failures) / (1.0 + f64::from(failures)),
            ),
            preferences: Ledger::new(&self.store).collective()?.entries,
        })
    }
    pub fn admit(&self, id: PreferenceId) -> Result<View> {
        self.write(|| {
            ensure!(
                self.view()?
                    .preferences
                    .iter()
                    .any(|p| p.preference.id == id),
                "unknown preference"
            );
            self.append(
                EventKind::StatusSet {
                    preference: id,
                    status: PreferenceStatus::Open,
                    by: Source::Operator,
                },
                Source::Operator,
            )?;
            self.view()
        })
    }
    pub fn step(&self, proposer: &dyn Proposer) -> Result<Trace> {
        self.write(|| {
            let before = self.view()?;
            let routes = proposer.propose(&before);
            ensure!(
                !routes.is_empty() && routes.len() <= 2,
                "proposer must offer one or two bounded routes"
            );
            ensure!(
                routes.len() == 1 || routes[0] != routes[1],
                "duplicate route"
            );
            let support: Vec<_> = before
                .preferences
                .iter()
                .filter(|p| p.is_open())
                .map(|p| p.preference.id)
                .collect();
            ensure!(!support.is_empty(), "no admitted open criterion");
            let mut possibilities = vec![];
            let mut selected = None;
            for route in routes {
                let estimate = before.estimates.iter().find(|e| e.route == route).unwrap();
                // A one-item probe establishes a working route, not proof of whole-batch success.
                let delivered = route.deliver(1);
                let evidence = self.fact(&Fact::Probe {
                    route,
                    requested: 1,
                    delivered,
                })?;
                let admitted = delivered == 1 && estimate.failures == 0;
                let reason = if admitted {
                    "one-item probe passed; no contradictory batch evidence"
                } else {
                    "prior batch evidence contradicts this route; choose another bounded step"
                };
                possibilities.push(Choice {
                    route,
                    admitted,
                    reason: reason.into(),
                    evidence,
                    prediction: estimate.prediction.clone(),
                });
                let args = self.blob(&serde_json::to_vec(&route)?)?;
                let mut candidate = Candidate::new(
                    ActionKind::Generate,
                    Source::Intuition,
                    args,
                    vec![evidence],
                    vec![],
                )?;
                candidate.support = support.clone();
                self.append(
                    EventKind::Proposed {
                        candidate: candidate.clone(),
                    },
                    Source::Intuition,
                )?;
                if admitted && selected.is_none() {
                    selected = Some((route, candidate, evidence, estimate.prediction.value));
                }
            }
            let Some((route, candidate, evidence, p)) = selected else {
                bail!("no evidence-supported route; no cycle committed")
            };
            let forecast = Forecast::new(
                OutcomeDefId(Hash::of(&[b"complete_batch@v1"])),
                Horizon::TaskClose,
                Forecaster::Controller,
                Probability::new(p)?,
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
                            p_success: p,
                            value: 1.0,
                            cost: route.cost(),
                            ev: p - route.cost(),
                        },
                        propensity: 1.0,
                        evidence: vec![evidence],
                    },
                },
                Source::Rules,
            )?;
            let delivered = route.deliver(before.batch_size);
            let returned = self.fact(&Fact::Outcome {
                route,
                requested: before.batch_size,
                delivered,
            })?;
            let unmet = delivered != before.batch_size;
            let contrast = Filter::contrast_from_criteria(
                returned,
                if unmet {
                    vec![(Self::criterion(), UnmetKind::WrongBehaviour)]
                } else {
                    vec![]
                },
                vec![returned],
            );
            let residual = p - if unmet { 0.0 } else { 1.0 };
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
            if unmet {
                self.expand(
                    &contrast,
                    "Deliver batches without dropping queued items",
                    vec![support[0]],
                    Source::Generator,
                )?;
            }
            let tensions = vec![format!("Open batch-completion criterion versus previous commitment {:?}; observed unmet-pressure {}; predictions {:?}", before.last_route, before.unmet_pressure.value, before.estimates.iter().map(|e| (e.route,e.prediction.value)).collect::<Vec<_>>())];
            Ok(Trace {
                cycle: before.cycles + 1,
                requested: before.batch_size,
                before,
                tensions,
                possibilities,
                chosen: route,
                commitment,
                delivered,
                residual: Proxy::new("residual", residual),
                unmet,
                after: self.view()?,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    struct OnlyBulk;
    impl Proposer for OnlyBulk {
        fn propose(&self, _: &View) -> Vec<Route> {
            vec![Route::Bulk]
        }
    }
    #[test]
    fn three_cycles_change_commitment_from_return_and_resume() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let engine = Engine::open(dir.path())?;
        engine.initialize(3)?;
        let first = engine.step(&LocalProposer)?;
        assert_eq!(first.chosen, Route::Bulk);
        assert_eq!(first.delivered, 1);
        assert!(first.unmet);
        assert_eq!(first.before.unmet_pressure.value, 0.0);
        assert_eq!(first.after.unmet_pressure.value, 0.5);
        assert_eq!(first.after.estimates[0].prediction.value, 1.0 / 3.0);
        assert_eq!(first.after.preferences.len(), 2);
        assert!(!first.after.preferences[1].is_active());
        let pending = first.after.preferences[1].preference.id;
        engine.admit(pending)?;
        assert!(engine.view()?.preferences[1].is_open());
        let second = engine.step(&LocalProposer)?;
        assert_eq!(second.chosen, Route::Checked);
        assert!(!second.unmet);
        assert!(second
            .possibilities
            .iter()
            .any(|p| p.route == Route::Bulk && !p.admitted));
        let saved = serde_json::to_value(engine.view()?)?;
        drop(engine);
        let resumed = Engine::open(dir.path())?;
        assert_eq!(serde_json::to_value(resumed.view()?)?, saved);
        let third = resumed.step(&LocalProposer)?;
        assert_eq!(third.cycle, 3);
        assert_eq!(third.chosen, Route::Checked);
        assert_eq!(third.delivered, 3);
        assert!(
            third.before.estimates[1].prediction.value
                > second.before.estimates[1].prediction.value
        );
        assert_eq!(third.after.preferences.len(), 2);
        assert!(resumed.initialize(3).is_err());
        Ok(())
    }
    #[test]
    fn evidence_rejects_repeated_failed_route_without_partial_cycle() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let engine = Engine::open(dir.path())?;
        engine.initialize(4)?;
        engine.step(&OnlyBulk)?;
        let root = engine.view()?.root;
        assert!(engine.step(&OnlyBulk).is_err());
        assert_eq!(engine.view()?.root, root);
        assert_eq!(engine.view()?.cycles, 1);
        Ok(())
    }
    #[test]
    fn frozen_forecast_precedes_commitment_and_surprise_does_not_expand_success() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let engine = Engine::open(dir.path())?;
        engine.initialize(3)?;
        engine.step(&LocalProposer)?;
        let before = engine.view()?.preferences.len();
        let success = engine.step(&LocalProposer)?;
        assert!(success.residual.value != 0.0);
        assert!(!success.unmet);
        assert_eq!(engine.view()?.preferences.len(), before);
        let events = engine.events()?;
        for (index, (_, event)) in events.iter().enumerate() {
            if let EventKind::Committed { candidate, .. } = event.kind {
                match &events[index - 1].1.kind {
                    EventKind::Forecast {
                        candidate: c,
                        forecast,
                    } => {
                        assert_eq!(*c, candidate);
                        assert_eq!(forecast.state_version, index as u64 - 1);
                    }
                    _ => panic!("commitment without frozen forecast"),
                }
            }
        }
        for _ in 0..20 {
            engine.step(&LocalProposer)?;
        }
        assert!(engine.view()?.unmet_pressure.value < 1.0);
        assert_eq!(engine.view()?.preferences.len(), before);
        Ok(())
    }
    #[test]
    fn changed_evidence_is_refused_on_restart() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let engine = Engine::open(dir.path())?;
        engine.initialize(3)?;
        let path = fs::read_dir(dir.path().join("cas"))?
            .next()
            .unwrap()?
            .path();
        fs::write(path, b"{}")?;
        assert!(engine.view().is_err());
        Ok(())
    }
}

#[cfg(test)]
mod integrity_tests {
    use super::*;
    #[test]
    fn disconnected_canonical_event_is_not_a_valid_cycle_history() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let engine = Engine::open(dir.path())?;
        engine.initialize(3)?;
        let event = Event::new(
            Engine::task(),
            vec![],
            Source::Rules,
            EventKind::Stop {
                reason: "interposed".into(),
            },
            "test",
        );
        engine.store.append(&event)?;
        assert!(engine
            .view()
            .unwrap_err()
            .to_string()
            .contains("broken cycle event chain"));
        Ok(())
    }
}

pub mod adaptive;

pub mod decision_policy;
pub mod deliberation;
pub mod shadow;

pub mod tier2;

pub mod qualitative;

pub mod public_attachment;
