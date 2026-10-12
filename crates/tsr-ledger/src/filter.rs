//! The Resistance Contrast Filter, in code (PRD §B2.1, §B6, §B11 M2 item 2).
//!
//! One return produces **two separate records**, kept apart:
//!
//! - **Surprise** — the forecast residual per forecaster (the raw probability minus the observed
//!   outcome). It updates Expectation on the physical side and is recorded as an `Observed`
//!   event carrying [`tsr_domain::Surprise`]. It is **never** an input to expansion.
//! - **Contrast** — desired minus actual, scored against the task's named open criteria: which
//!   were unmet and how ([`UnmetKind`]). It is the **only** input to expansion.
//!
//! The write rule: a Contrast with at least one unmet criterion may produce an `Expansion` with
//! lineage (`parents`, `catalyzed_by`) and referenced evidence; a Surprise never does. The
//! produced preference names the *dimension* the contrast clarified (reliability, regression,
//! cost, rejection) — never the failed strategy's content. The evidence is referenced by event
//! id, never copied into the preference.
//!
//! Both records travel in one `Observed` event as `Observation::Return { surprise, contrast }`:
//! the domain contract keeps them in one observation, and the Filter keeps them apart in the
//! fields — the Surprise list is never read by [`Filter::expand_from_contrast`].

use tsr_domain::{
    Contrast, ContrastId, CriterionId, Event, EventId, EventKind, Forecast, ForecastId, Hash,
    Observation, PreferenceId, Scope, Source, Surprise, TaskId, UnmetKind,
};
use tsr_store::Store;

use crate::error::LedgerError;
use crate::write::Ledger;

/// The Filter over one store. Holds no state: every read is a fold, every write an append.
pub struct Filter<'a> {
    ledger: Ledger<'a>,
}

impl<'a> Filter<'a> {
    pub fn new(store: &'a Store) -> Self {
        Self {
            ledger: Ledger::new(store),
        }
    }

    pub fn ledger(&self) -> &Ledger<'a> {
        &self.ledger
    }

    /// The forecast residual per forecaster: `raw p - observed`, in `[-1, 1]`.
    ///
    /// `observed` is the outcome of the named event (0 or 1). One [`Surprise`] per forecast, in
    /// the order given. This is the physical-side record; it never expands. The sign is forecast
    /// minus observed, the same as the scorer's residual (D11).
    pub fn surprise_from_forecasts(forecasts: &[Forecast], observed: f64) -> Vec<Surprise> {
        forecasts
            .iter()
            .map(|f| Surprise {
                forecast: f.id,
                residual: f.raw.get() - observed,
            })
            .collect()
    }

    /// Score desired minus actual against the task's named open criteria.
    ///
    /// `unmet` pairs each criterion that went unmet with *how* it went unmet; `evidence` are the
    /// event ids the contrast references. The `ContrastId` is content-addressed over the return,
    /// the unmet pairs and the evidence, so the same contrast is the same id.
    pub fn contrast_from_criteria(
        from_return: EventId,
        unmet: Vec<(CriterionId, UnmetKind)>,
        evidence: Vec<EventId>,
    ) -> Contrast {
        let id = contrast_id(&from_return, &unmet, &evidence);
        Contrast {
            id,
            from_return,
            unmet,
            evidence,
        }
    }

    /// Record one return as a single `Observed` event carrying both records apart.
    ///
    /// Returns the appended event's id. The Surprise list and the Contrast are stored side by
    /// side in `Observation::Return`; nothing is added together.
    pub fn record_return(
        &self,
        task: &TaskId,
        parents: Vec<EventId>,
        surprise: Vec<Surprise>,
        contrast: Option<Contrast>,
        recorded_at: impl Into<String>,
    ) -> Result<EventId, LedgerError> {
        let kind = EventKind::Observed {
            observation: Observation::Return { surprise, contrast },
        };
        let ev = Event::new(*task, parents, Source::Rules, kind, recorded_at);
        self.ledger.store().append(&ev)?;
        Ok(ev.id)
    }

    /// The Filter's write rule: a Contrast with at least one unmet criterion expands.
    ///
    /// Returns `Ok(None)` when the contrast has no unmet criterion — nothing crosses. When it
    /// does, appends an `Expansion` whose produced preference names the dimension the contrast
    /// clarified (from the [`UnmetKind`] and the criterion), with lineage (`parents`,
    /// `catalyzed_by`) and the contrast's evidence referenced by id. The failed strategy's
    /// content is never copied into the preference.
    #[allow(clippy::too_many_arguments)]
    pub fn expand_from_contrast(
        &self,
        task: &TaskId,
        parents: Vec<EventId>,
        contrast: &Contrast,
        scope: Scope,
        preference_parents: Vec<PreferenceId>,
        catalyzed_by: Vec<PreferenceId>,
        origin: Source,
        recorded_at: impl Into<String>,
    ) -> Result<Option<PreferenceId>, LedgerError> {
        if contrast.unmet.is_empty() {
            return Ok(None);
        }
        let text = dimension_text(&contrast.unmet);
        let id = self.ledger.expand(
            task,
            parents,
            contrast.id,
            text,
            scope,
            preference_parents,
            catalyzed_by,
            origin,
            recorded_at,
        )?;
        Ok(Some(id))
    }

    /// The whole Filter step for one return: record the two records apart, then expand only if
    /// the contrast has an unmet criterion.
    ///
    /// Returns the `Observed` event id and, when the contrast expanded, the produced
    /// `PreferenceId`. A return with only Surprise returns `(event, None)` and appends no
    /// `Expanded` event.
    #[allow(clippy::too_many_arguments)]
    pub fn filter_return(
        &self,
        task: &TaskId,
        parents: Vec<EventId>,
        surprise: Vec<Surprise>,
        contrast: Option<Contrast>,
        scope: Scope,
        preference_parents: Vec<PreferenceId>,
        catalyzed_by: Vec<PreferenceId>,
        origin: Source,
        recorded_at: impl Into<String>,
    ) -> Result<(EventId, Option<PreferenceId>), LedgerError> {
        let observed = self.record_return(
            task,
            parents.clone(),
            surprise,
            contrast.clone(),
            recorded_at.into(),
        )?;
        let produced = match &contrast {
            Some(c) => self.expand_from_contrast(
                task,
                vec![observed],
                c,
                scope,
                preference_parents,
                catalyzed_by,
                origin,
                format!("{}+expand", observed),
            )?,
            None => None,
        };
        Ok((observed, produced))
    }
}

/// The dimension a contrast clarified, as the produced preference's text.
///
/// The text names the dimension (reliability, regression, cost, rejection) and the criterion it
/// concerns — never the failed strategy's content. One unmet criterion yields one dimension; a
/// contrast with several names each, in the order given.
pub fn dimension_text(unmet: &[(CriterionId, UnmetKind)]) -> String {
    let parts: Vec<String> = unmet
        .iter()
        .map(|(criterion, kind)| format!("{}: {}", dimension_name(kind), criterion))
        .collect();
    format!(
        "the dimension this contrast clarified — {}",
        parts.join("; ")
    )
}

/// The engineering name of an [`UnmetKind`]'s dimension.
pub fn dimension_name(kind: &UnmetKind) -> &'static str {
    match kind {
        UnmetKind::WrongBehaviour => "reliability",
        UnmetKind::Regression => "regression",
        UnmetKind::Cost => "cost",
        UnmetKind::Rejected { .. } => "rejection",
    }
}

/// The content address of a contrast: blake3 over the return, the unmet pairs and the evidence.
///
/// Public so the write path can check a Contrast's `id` against its own content (D12): the same
/// contrast is the same id, and a record whose id is not its content address is refused.
pub fn contrast_id(
    from_return: &EventId,
    unmet: &[(CriterionId, UnmetKind)],
    evidence: &[EventId],
) -> ContrastId {
    let payload = serde_json::to_vec(&(from_return, unmet, evidence)).expect("serializable");
    ContrastId(Hash::of(&[&payload]))
}

/// The forecast ids a Surprise list refers to, for callers that need the residual series.
pub fn surprised_forecasts(surprise: &[Surprise]) -> Vec<ForecastId> {
    surprise.iter().map(|s| s.forecast).collect()
}
