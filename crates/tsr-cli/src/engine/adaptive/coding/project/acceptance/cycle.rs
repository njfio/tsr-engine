//! Acceptance participates in the existing preference/forecast/return cycle.
//! Private evidence is never used as a proposal or copied to public guidance.
use super::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Feedback {
    pub requirements_revision: EventId,
    pub decision_record: Option<EventId>,
    /// Operator-authored, explicitly shareable guidance, keyed by requirement.
    pub guidance: BTreeMap<String, String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalReview {
    pub review: EventId,
    pub receipt: EventId,
    pub accepted: bool,
    pub met: BTreeMap<String, bool>,
}
impl Engine {
    pub(crate) fn acceptance_private_aims(&self) -> Result<BTreeSet<PreferenceId>> {
        Ok(self
            .records()?
            .iter()
            .flat_map(|(_, r)| match r {
                Record::Coding {
                    record: CodingRecord::AcceptanceAims { preferences, .. },
                } => preferences.values().copied().collect(),
                _ => Vec::new(),
            })
            .collect())
    }
    pub(crate) fn acceptance_current_aims(&self) -> Result<BTreeMap<String, PreferenceId>> {
        let records = self.records()?;
        let revision = records.iter().rev().find_map(|(id, r)| {
            matches!(
                r,
                Record::Coding {
                    record: CodingRecord::AcceptanceRequirements { .. }
                }
            )
            .then_some(*id)
        });
        Ok(records
            .iter()
            .rev()
            .find_map(|(_, r)| match r {
                Record::Coding {
                    record:
                        CodingRecord::AcceptanceAims {
                            revision: r,
                            preferences,
                        },
                } if Some(*r) == revision => Some(preferences.clone()),
                _ => None,
            })
            .unwrap_or_default())
    }
    /// Selection pressure/support follow folded authority; acceptance criteria do not change.
    pub(crate) fn acceptance_active_aims(&self) -> Result<BTreeMap<String, PreferenceId>> {
        // Fulfilled standing desires remain to account for possible losses.
        let active = Ledger::new(&self.store)
            .collective()?
            .entries
            .into_iter()
            .filter(|entry| {
                matches!(
                    entry.status,
                    Some(PreferenceStatus::Open | PreferenceStatus::Fulfilled)
                )
            })
            .map(|entry| entry.preference.id)
            .collect::<BTreeSet<_>>();
        Ok(self
            .acceptance_current_aims()?
            .into_iter()
            .filter(|(_, id)| active.contains(id))
            .collect())
    }
    pub(crate) fn acceptance_admit_aims(
        &self,
        revision: EventId,
        requirements: &BTreeMap<String, String>,
    ) -> Result<()> {
        // Republishing criteria changes the assessment scope, not the existence
        // or eligibility of earlier wanting. Explicit status changes remain
        // authoritative; historical records are never rewritten or reopened.
        // Attach semantic aims to the original goal lineage, without putting them in test criteria.
        let private = self.acceptance_private_aims()?;
        let parents = Ledger::new(&self.store)
            .collective()?
            .entries
            .iter()
            .filter(|p| !private.contains(&p.preference.id))
            .map(|p| p.preference.id)
            .collect::<Vec<_>>();
        let mut preferences = BTreeMap::new();
        for (id, text) in requirements {
            let contrast = Filter::contrast_from_criteria(
                revision,
                vec![(
                    CriterionId(Hash::of(&[b"acceptance", id.as_bytes()])),
                    UnmetKind::WrongBehaviour,
                )],
                vec![revision],
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
            self.expand(&contrast, text, parents.clone(), Source::Operator)?;
            let preference = Preference::new(
                text,
                Scope::Task(Self::task()),
                parents.clone(),
                Source::Operator,
            )?
            .id;
            let excluded = Ledger::new(&self.store)
                .collective()?
                .get(&preference)
                .is_some_and(|entry| {
                    matches!(
                        entry.status,
                        Some(PreferenceStatus::Withdrawn | PreferenceStatus::OutOfScope)
                    )
                });
            if !excluded {
                self.append(
                    EventKind::StatusSet {
                        preference,
                        status: PreferenceStatus::Open,
                        by: Source::Operator,
                    },
                    Source::Operator,
                )?;
            }
            preferences.insert(id.clone(), preference);
        }
        self.adaptive_record(
            &Record::Coding {
                record: CodingRecord::AcceptanceAims {
                    revision,
                    preferences,
                },
            },
            Source::Operator,
        )?;
        Ok(())
    }
    pub(crate) fn acceptance_reopen_aims(&self) -> Result<()> {
        let aims = self.acceptance_current_aims()?;
        if aims.is_empty() {
            return Ok(());
        }
        ensure!(
            !self.project_acceptance_status()?.accepted,
            "goal already accepted; revise requirements before another repair"
        );
        let entries = Ledger::new(&self.store).collective()?.entries;
        for preference in aims.values() {
            if entries.iter().any(|e| {
                e.preference.id == *preference && e.status == Some(PreferenceStatus::Fulfilled)
            }) {
                self.append(
                    EventKind::StatusSet {
                        preference: *preference,
                        status: PreferenceStatus::Open,
                        by: Source::Operator,
                    },
                    Source::Operator,
                )?;
            }
        }
        Ok(())
    }
    pub fn project_acceptance_feedback(&self, feedback: Feedback) -> Result<Completion> {
        self.write(|| {
            let current = self.project_acceptance_status()?;
            ensure!(
                Some(feedback.requirements_revision) == current.requirements_revision
                    && feedback.decision_record == current.decision_record,
                "stale feedback binding"
            );
            ensure!(
                self.adaptive_state_inner()?.pending.is_none(),
                "cannot publish feedback during execution"
            );
            let requirements = self
                .records()?
                .into_iter()
                .rev()
                .find_map(|(_, r)| match r {
                    Record::Coding {
                        record: CodingRecord::AcceptanceRequirements { requirements },
                    } => Some(requirements),
                    _ => None,
                })
                .ok_or_else(|| anyhow::anyhow!("requirements required"))?;
            ensure!(
                !feedback.guidance.is_empty()
                    && feedback.guidance.len() <= 16
                    && feedback
                        .guidance
                        .iter()
                        .all(|(id, text)| requirements.contains_key(id)
                            && if self.project_configuration().is_ok_and(|s| s.browser) {
                                super::super::browser::narrative(text)
                            } else {
                                short(text)
                            }),
                "bounded guidance must name current requirements"
            );
            // Old stores acquire preferences on their first explicit feedback publication.
            if self.acceptance_current_aims()?.is_empty() {
                self.acceptance_admit_aims(feedback.requirements_revision, &requirements)?;
                if current.accepted {
                    for preference in self.acceptance_current_aims()?.values() {
                        self.append(
                            EventKind::StatusSet {
                                preference: *preference,
                                status: PreferenceStatus::Fulfilled,
                                by: Source::Operator,
                            },
                            Source::Operator,
                        )?;
                    }
                }
            }
            let publication = self.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::AcceptanceFeedback { feedback },
                },
                Source::Operator,
            )?;
            self.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::AcceptanceGuidanceScope {
                        publication,
                        snapshot: current.snapshot,
                        scope: Hash::of(&[&serde_json::to_vec(&self.project_configuration()?)?]),
                    },
                },
                Source::Operator,
            )?;
            self.project_acceptance_status()
        })
    }
    fn acceptance_observations(
        &self,
    ) -> Result<(
        Completion,
        BTreeMap<EventId, HistoricalReview>,
        Option<Review>,
    )> {
        let current = self.project_acceptance_status()?;
        let records = self.records()?;
        // One independent judgment per tested candidate; superseding reviews do not multiply evidence.
        let mut judgments = BTreeMap::new();
        let mut latest = None;
        for (id, r) in &records {
            if let Record::Coding {
                record: CodingRecord::AcceptanceReview { review },
            } = r
            {
                if Some(review.requirements_revision) == current.requirements_revision {
                    if let Some(receipt) = review.test_receipt {
                        judgments.insert(receipt, (*id, review));
                    }
                    latest = Some(review.clone());
                }
            }
        }
        let mut episodes = BTreeMap::new();
        for (_, r) in &records {
            if let Record::Coding {
                record:
                    CodingRecord::AcceptanceHistory {
                        revision,
                        source_episode,
                        observations,
                        ..
                    },
            } = r
            {
                if Some(*revision) == current.requirements_revision {
                    episodes.insert(*source_episode, observations);
                }
            }
        }
        let mut history = BTreeMap::new();
        for points in episodes.values() {
            for point in *points {
                history.insert(point.receipt, point.clone());
            }
        }
        for (receipt, (id, review)) in &judgments {
            if review.decision == Decision::Unresolved {
                history.remove(receipt);
            } else {
                history.insert(
                    *receipt,
                    HistoricalReview {
                        review: *id,
                        receipt: *receipt,
                        accepted: review.decision == Decision::Accept,
                        met: review
                            .evidence
                            .iter()
                            .map(|(id, e)| (id.clone(), e.met))
                            .collect(),
                    },
                );
            }
        }
        Ok((current, history, latest))
    }
    pub(crate) fn acceptance_kernel_inputs(
        &self,
        action: Option<&str>,
    ) -> Result<Vec<(tsr_domain::decision::Obligation, f64, Vec<EventId>)>> {
        use tsr_domain::decision as kernel;
        let (current, history, latest) = self.acceptance_observations()?;
        let bound = matches!(current.status.as_str(), "accepted" | "rejected");
        let records = self.records()?;
        let paths = |action: &str| -> Result<BTreeSet<String>> {
            Ok(self
                .patches_for(action)?
                .unwrap_or_default()
                .into_iter()
                .map(|p| p.path)
                .collect())
        };
        let current_paths = action.map(paths).transpose()?.unwrap_or_default();
        let mut weights = BTreeMap::new();
        for point in history.values() {
            let commitment = records.iter().find_map(|(id, r)| match r {
                Record::Coding {
                    record: CodingRecord::Receipt { commitment, .. },
                } if *id == point.receipt => Some(*commitment),
                _ => None,
            });
            let prior = records.iter().find_map(|(_, r)| match r {
                Record::Pending {
                    commitment: c,
                    action,
                    ..
                } if Some(*c) == commitment => Some(action.as_str()),
                _ => None,
            });
            let weight = match (action, prior) {
                (Some(a), Some(b)) if a == b => 1.0,
                (Some(_), Some(b)) => {
                    let old = paths(b)?;
                    current_paths.intersection(&old).count() as f64
                        / current_paths.union(&old).count().max(1) as f64
                }
                (Some(_), None) => 0.25, // imported receipt lacks action context: weak family evidence only
                (None, _) => 1.0,
            };
            weights.insert(point.review, weight);
        }
        // Legacy aliases may share one preference identity. Count the desire once;
        // a contradictory/unknown alias cannot turn it into established satisfaction.
        let mut grouped: BTreeMap<_, Vec<String>> = BTreeMap::new();
        for (name, id) in self.acceptance_active_aims()? {
            grouped.entry(id).or_default().push(name);
        }
        grouped
            .into_iter()
            .map(|(id, names)| {
                let observations = history
                    .values()
                    .map(|r| kernel::Return {
                        evidence: r.review,
                        weight: weights[&r.review],
                        success: alias_observation(
                            names.iter().map(|name| r.met.get(name).copied()),
                        ),
                        predicted: None,
                        observed: None,
                    })
                    .collect::<Vec<_>>();
                let forecast = kernel::expectation(&observations).map_err(anyhow::Error::msg)?;
                let observed = if bound {
                    latest
                        .as_ref()
                        .and_then(|r| {
                            alias_observation(
                                names.iter().map(|name| r.evidence.get(name).map(|e| e.met)),
                            )
                        })
                        .map(|met| if met { 1.0 } else { 0.0 })
                } else {
                    None
                };
                Ok((
                    kernel::Obligation {
                        id,
                        refines: None,
                        admitted: true,
                        observed,
                    },
                    forecast.probability,
                    forecast.evidence,
                ))
            })
            .collect()
    }
    pub(crate) fn acceptance_cycle(&self) -> Result<serde_json::Value> {
        let (current, history, latest) = self.acceptance_observations()?;
        let records = self.records()?;
        let requirements = records.iter().rev().find_map(|(_, r)| match r {
            Record::Coding {
                record: CodingRecord::AcceptanceRequirements { requirements },
            } => Some(requirements),
            _ => None,
        });
        let fully_reviewed = history
            .values()
            .filter(|r| requirements.is_some_and(|required| r.met.keys().eq(required.keys())))
            .collect::<Vec<_>>();
        let accepted = fully_reviewed.iter().filter(|r| r.accepted).count();
        let p = (1.0 + accepted as f64) / (2.0 + fully_reviewed.len() as f64);
        let (guidance, guidance_provenance) = self.acceptance_guidance(&current)?;
        let uncertainty = records.iter().rev().find_map(|(id, r)| match r {
            Record::Coding {
                record: CodingRecord::AcceptanceUncertainty { review, unknown },
            } if Some(*review) == current.decision_record => Some((*id, unknown.clone())),
            _ => None,
        });
        if let Some((id, unknown)) = &uncertainty {
            ensure!(
                self.events()?
                    .iter()
                    .any(|(_, e)| e.id == *id && e.source == Source::Rules),
                "untrusted review uncertainty provenance"
            );
            let review = records
                .iter()
                .find_map(|(id, r)| match r {
                    Record::Coding {
                        record: CodingRecord::AcceptanceReview { review },
                    } if Some(*id) == current.decision_record => Some(review),
                    _ => None,
                })
                .ok_or_else(|| anyhow::anyhow!("uncertainty without referenced review"))?;
            ensure!(
                (unknown.is_empty() || review.decision == Decision::Unresolved)
                    && unknown
                        .iter()
                        .all(|k| review.evidence.get(k).is_some_and(|e| !e.met)),
                "invalid review uncertainty dimensions"
            );
        }
        let review_applicable = matches!(
            current.status.as_str(),
            "accepted" | "rejected" | "unresolved"
        );
        let unknown = if review_applicable {
            uncertainty.map(|(_, u)| u).unwrap_or_else(|| {
                latest
                    .as_ref()
                    .filter(|r| r.decision == Decision::Unresolved)
                    .map(|r| {
                        r.evidence
                            .iter()
                            .filter(|(_, e)| !e.met)
                            .map(|(k, _)| k.clone())
                            .collect()
                    })
                    .unwrap_or_default()
            })
        } else {
            BTreeSet::new()
        };
        let unmet = latest
            .as_ref()
            .filter(|_| current.status == "rejected" || current.status == "unresolved")
            .map(|r| {
                r.evidence
                    .iter()
                    .filter(|(id, e)| !e.met && !unknown.contains(*id))
                    .map(|(id, _)| id.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let ready = current.status == "rejected"
            && !unmet.is_empty()
            && unmet.iter().all(|id| guidance.contains_key(id));
        let belief=guidance.keys().map(|id| {
            let points=history.values().filter_map(|r|r.met.get(id)).collect::<Vec<_>>();
            let met=points.iter().filter(|v|***v).count();
            (id.clone(),serde_json::json!({"observed":points.len(),"met":met,"prediction":Proxy::new("requirement_met_rate",(1.0+met as f64)/(2.0+points.len() as f64))}))
        }).collect::<BTreeMap<_,_>>();
        let visible_unmet = unmet
            .iter()
            .filter(|id| guidance.contains_key(*id))
            .collect::<Vec<_>>();
        let visible_unknown = unknown
            .iter()
            .filter(|id| guidance.contains_key(*id))
            .collect::<Vec<_>>();
        Ok(
            serde_json::json!({"requirements_revision":current.requirements_revision,"decision_record":current.decision_record,
            "guidance":guidance,"guidance_provenance":guidance_provenance,"belief":belief,"unmet":visible_unmet,"unmet_count":unmet.len(),"unknown":visible_unknown,"unknown_count":if review_applicable{Some(unknown.len())}else{None},"review_evidence_applicable":review_applicable,"repair_ready":ready,
            "next_step":if current.accepted {"complete"} else if ready {"propose_repair"} else {"independent_review_or_feedback"},
            "prediction":Proxy::new("operator_acceptance_rate",p),"reviewed_candidates":history.len(),"fully_reviewed_candidates":fully_reviewed.len(),
            "evidence":history.values().rev().take(16).map(|r|r.review).collect::<Vec<_>>()}),
        )
    }
    pub(crate) fn acceptance_freeze(
        &self,
        candidate: CandidateId,
    ) -> Result<Option<(EventId, Forecast)>> {
        if self.acceptance_current_aims()?.is_empty() {
            return Ok(None);
        }
        let current = self.project_acceptance_status()?;
        let cycle = self.acceptance_cycle()?;
        let forecast = Forecast::new(
            OutcomeDefId(Hash::of(&[b"independent_operator_acceptance@v1"])),
            Horizon::TaskClose,
            Forecaster::Controller,
            Probability::new(cycle["prediction"]["value"].as_f64().unwrap())?,
            self.events()?.len() as u64,
        );
        self.append(
            EventKind::Forecast {
                forecast: forecast.clone(),
                candidate,
            },
            Source::Rules,
        )?;
        Ok(Some((current.requirements_revision.unwrap(), forecast)))
    }
    pub(crate) fn acceptance_return(
        &self,
        record: EventId,
        review: &Review,
        unknown: &BTreeSet<String>,
    ) -> Result<()> {
        let records = self.records()?;
        let commitment = records.iter().find_map(|(id, r)| match r {
            Record::Coding {
                record: CodingRecord::Receipt { commitment, .. },
            } if Some(*id) == review.test_receipt => Some(*commitment),
            _ => None,
        });
        let expectation = records.iter().rev().find_map(|(_, r)| match r {
            Record::Coding {
                record:
                    CodingRecord::AcceptanceExpectation {
                        revision,
                        commitment: c,
                        forecast,
                    },
            } if *revision == review.requirements_revision && Some(*c) == commitment => {
                Some(forecast)
            }
            _ => None,
        });
        let previously_resolved=records.iter().any(|(id,r)|matches!(r,Record::Coding { record:CodingRecord::AcceptanceReview { review:r } } if *id!=record && r.requirements_revision==review.requirements_revision && r.test_receipt==review.test_receipt && r.decision!=Decision::Unresolved));
        let surprise = if review.decision != Decision::Unresolved && !previously_resolved {
            expectation
                .map(|f| {
                    vec![Surprise {
                        forecast: f.id,
                        residual: f.raw.get()
                            - if review.decision == Decision::Accept {
                                1.0
                            } else {
                                0.0
                            },
                    }]
                })
                .unwrap_or_default()
        } else {
            vec![]
        };
        let unmet = review
            .evidence
            .iter()
            .filter(|(id, e)| !e.met && !unknown.contains(*id))
            .map(|(id, _)| {
                (
                    CriterionId(Hash::of(&[b"acceptance", id.as_bytes()])),
                    UnmetKind::WrongBehaviour,
                )
            })
            .collect();
        let contrast = Filter::contrast_from_criteria(record, unmet, vec![record]);
        self.append(
            EventKind::Observed {
                observation: Observation::Return {
                    surprise,
                    contrast: Some(contrast),
                },
            },
            Source::Operator,
        )?;
        let entries = Ledger::new(&self.store).collective()?.entries;
        // Legacy requirement aliases share one desire: fulfillment is conjunction,
        // never whichever key sorts last.
        let mut fulfillment = BTreeMap::<PreferenceId, bool>::new();
        for (key, preference) in self.acceptance_current_aims()? {
            let met = review.evidence.get(&key).is_some_and(|e| e.met) && !unknown.contains(&key);
            fulfillment
                .entry(preference)
                .and_modify(|all| *all &= met)
                .or_insert(met);
        }
        for (preference, met) in &fulfillment {
            if entries.iter().any(|e| {
                e.preference.id == *preference
                    && matches!(
                        e.status,
                        Some(PreferenceStatus::Withdrawn | PreferenceStatus::OutOfScope)
                    )
            }) {
                continue;
            }
            self.append(
                EventKind::StatusSet {
                    preference: *preference,
                    status: if *met {
                        PreferenceStatus::Fulfilled
                    } else {
                        PreferenceStatus::Open
                    },
                    by: Source::Operator,
                },
                Source::Operator,
            )?;
        }
        Ok(())
    }
}

// Conservative conjunction over aliases of the same desire, not extra support.
fn alias_observation(values: impl Iterator<Item = Option<bool>>) -> Option<bool> {
    let mut unknown = false;
    for value in values {
        match value {
            Some(false) => return Some(false),
            None => unknown = true,
            Some(true) => {}
        }
    }
    if unknown {
        None
    } else {
        Some(true)
    }
}
#[cfg(test)]
mod alias_tests {
    use super::*;
    #[test]
    fn duplicate_aliases_do_not_manufacture_satisfaction() {
        for values in [vec![Some(true), Some(false)], vec![Some(false), Some(true)]] {
            assert_eq!(alias_observation(values.into_iter()), Some(false));
        }
        assert_eq!(alias_observation([Some(true), None].into_iter()), None);
        assert_eq!(
            alias_observation([None, Some(false)].into_iter()),
            Some(false)
        );
        assert_eq!(
            alias_observation([Some(true), Some(true)].into_iter()),
            Some(true)
        );
    }
}
