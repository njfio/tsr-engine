//! Opt-in access forecasts. Never forecasts of assertion truth or desire fulfillment.
use super::*;
use tsr_forecast::{Resolution, ResolvedForecast, ScoredForecast};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub choice: inquiry::InformationChoice,
    pub probability: f64,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attempt {
    pub choice: inquiry::InformationChoice,
    pub forecast: Forecast,
}
impl Engine {
    fn information_attempts(&self) -> Result<Vec<(EventId, Attempt, Option<EventId>)>> {
        let mut items: Vec<(EventId, Attempt, Option<EventId>)> = vec![];
        for (id, entry) in self.qualitative_entries()? {
            match entry {
                Entry::InformationAttempt { attempt } => items.push((id, attempt, None)),
                Entry::InformationResolved { attempt, receipt } => {
                    let item = items
                        .iter_mut()
                        .find(|x| x.0 == attempt)
                        .context("foreign information attempt")?;
                    ensure!(item.2.is_none(), "duplicate information settlement");
                    item.2 = Some(receipt);
                }
                _ => {}
            }
        }
        Ok(items)
    }
    pub(super) fn check_information_attempt(
        &self,
        id: Option<EventId>,
        request: &retrieval::Request,
        inquiry: Option<EventId>,
    ) -> Result<()> {
        let items = self.information_attempts()?;
        let pending: Vec<_> = items.iter().filter(|x| x.2.is_none()).collect();
        match id {
            None => ensure!(
                pending.is_empty(),
                "unresolved information dispatch; no repeated read"
            ),
            Some(id) => {
                ensure!(
                    pending.len() == 1 && pending[0].0 == id,
                    "stale information dispatch"
                );
                let c = &pending[0].1.choice;
                ensure!(
                    Some(c.inquiry) == inquiry
                        && c.request.corpus == request.corpus
                        && c.request.ids == request.ids,
                    "information dispatch mismatch"
                );
            }
        }
        Ok(())
    }
    /// New opt-in path. Caller probability is uncalibrated and attributed to the operator.
    /// Intent commits before file I/O. Any interruption/error remains unresolved and blocks reads.
    pub fn qualitative_forecasted_retrieve(&self, input: Input) -> Result<EventId> {
        let (id, root) = self.write(|| {
            ensure!(
                input.choice.request.state_root == self.store.head_root()?,
                "stale forecast source"
            );
            self.check_information_attempt(
                None,
                &input.choice.request,
                Some(input.choice.inquiry),
            )?;
            self.current_inquiry(input.choice.inquiry)?;
            ensure!(bounded(&input.choice.relevance), "pre-read relevance required");
            self.validate_exploration_read(input.choice.inquiry, &input.choice.request.ids)?;
            ensure!(matches!(self.qualitative_inquiry_route(input.choice.inquiry)?, inquiry::InquiryRoute::CorpusCandidate { corpus } if corpus == input.choice.request.corpus), "unsupported inquiry source");
            ensure!(self.qualitative_request()?["phase"] == "propose", "pending review or stopped");
            let (manifest, receipts) = self.retrieval_state()?;
            let manifest = manifest.context("no corpus")?;
            ensure!(Hash::of(&[&serde_json::to_vec(&manifest)?]) == input.choice.request.corpus, "wrong corpus");
            ensure!(receipts.len() < 32 && (1..=4).contains(&input.choice.request.ids.len()) && input.choice.request.ids.iter().collect::<BTreeSet<_>>().len() == input.choice.request.ids.len() && input.choice.request.ids.iter().all(|id| manifest.records.iter().any(|r| &r.id == id)), "invalid bounded selection");
            let forecast = Forecast::new(
                OutcomeDefId(Hash::of(&[b"validated_information_receipt@v1"])),
                Horizon::ThisDispatch,
                Forecaster::Operator,
                Probability::new(input.probability)?,
                self.events()?.len() as u64,
            );
            let id = self.qualitative_append(
                Entry::InformationAttempt {
                    attempt: Attempt {
                        choice: input.choice.clone(),
                        forecast,
                    },
                },
                Source::Operator,
            )?;
            Ok((id, self.store.head_root()?))
        })?;
        let mut request = input.choice.request;
        request.state_root = root;
        let receipt = self.qualitative_retrieve_with_attempt(
            request,
            Some(input.choice.inquiry),
            Some(input.choice.relevance),
            Some(id),
        )?;
        self.qualitative_resolve_information(id, receipt)?;
        Ok(receipt)
    }
    /// Reconcile an already durable receipt after interruption; never invokes the reader.
    pub fn qualitative_resolve_information(
        &self,
        attempt: EventId,
        receipt: EventId,
    ) -> Result<Value> {
        self.write(|| {
            let items = self.information_attempts()?;
            let a = items
                .iter()
                .find(|x| x.0 == attempt)
                .context("unknown information attempt")?;
            ensure!(a.2.is_none(), "information attempt already resolved");
            let entries = self.qualitative_entries()?;
            let pos = entries
                .iter()
                .position(|x| x.0 == attempt)
                .context("attempt")?;
            // The receipt must be the immediate next native qualitative event, not a later read.
            let next = entries
                .get(pos + 1)
                .context("no authenticated receipt; unresolved")?;
            ensure!(next.0 == receipt, "receipt is not this dispatch return");
            let Entry::Retrieved { receipt: r } = &next.1 else {
                bail!("no authenticated receipt")
            };
            let c = &a.1.choice;
            ensure!(
                r.inquiry == Some(c.inquiry)
                    && r.request.corpus == c.request.corpus
                    && r.request.ids == c.request.ids
                    && r.pre_read_relevance.as_ref() == Some(&c.relevance),
                "receipt mismatch"
            );
            ensure!(
                r.request_digest == Hash::of(&[&serde_json::to_vec(&r.request)?]),
                "receipt digest mismatch"
            );
            self.qualitative_append(
                Entry::InformationResolved { attempt, receipt },
                Source::Rules,
            )?;
            self.append(
                EventKind::Observed {
                    observation: Observation::Return {
                        surprise: vec![Surprise {
                            forecast: a.1.forecast.id,
                            residual: a.1.forecast.raw.get() - 1.0,
                        }],
                        contrast: None,
                    },
                },
                Source::Rules,
            )?;
            self.qualitative_information_expectation()
        })
    }
    pub fn qualitative_information_expectation(&self) -> Result<Value> {
        let items = self.information_attempts()?;
        let history: Vec<_> = items.iter().map(|(id,a,r)| {
            let resolved = ResolvedForecast { forecast: a.forecast.clone(), resolution: if r.is_some() {Resolution::True} else {Resolution::Unresolved}, evidence: r.iter().copied().collect() };
            let score = ScoredForecast::from_resolved(&resolved, Some("bounded_information_access".into()), None);
            json!({"attempt":id,"forecast":a.forecast,"receipt":r,"resolution":resolved.resolution,"residual":score.as_ref().map(|s|s.residual()),"brier":score.as_ref().map(|s|s.brier())})
        }).collect();
        Ok(
            json!({"event":"validated_information_receipt@v1","history":history,"resolved":items.iter().filter(|x|x.2.is_some()).count(),"unresolved":items.iter().filter(|x|x.2.is_none()).count(),"limit":"Access completion only. Errors/interruptions unresolved, not negative. Success-only observations cannot establish reliability; no fitted probability, acceptance, Contrast, source truth or usefulness inferred."}),
        )
    }
    pub(super) fn add_information_expectation(&self, q: &mut Value) -> Result<()> {
        if !self.information_attempts()?.is_empty() {
            q["information_expectation"] = self.qualitative_information_expectation()?;
        }
        Ok(())
    }
}
