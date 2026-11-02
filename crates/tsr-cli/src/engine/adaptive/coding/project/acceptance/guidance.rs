//! Published intent may persist; review, forecast and authority never carry with it.
use super::*;
impl Engine {
    /// Read-only public publications, never private acceptance criteria or action authority.
    pub fn project_published_guidance_history(&self) -> Result<serde_json::Value> {
        let current = self.project_acceptance_status()?;
        let (eligible, provenance) = self.acceptance_guidance(&current)?;
        let records = self.records()?;
        let entries = Ledger::new(&self.store).collective()?.entries;
        let mut publications = Vec::new();
        for (publication, record) in &records {
            let Record::Coding {
                record: CodingRecord::AcceptanceFeedback { feedback },
            } = record
            else {
                continue;
            };
            self.require_project_bridge_source(*publication, Source::Operator)?;
            let aims = records.iter().rev().find_map(|(id, record)| match record {
                Record::Coding {
                    record:
                        CodingRecord::AcceptanceAims {
                            revision,
                            preferences,
                        },
                } if *revision == feedback.requirements_revision => Some((*id, preferences)),
                _ => None,
            });
            if let Some((id, _)) = aims {
                self.require_project_bridge_source(id, Source::Operator)?;
            }
            let scope = records.iter().rev().find_map(|(id, record)| match record {
                Record::Coding {
                    record:
                        CodingRecord::AcceptanceGuidanceScope {
                            publication: p,
                            scope,
                            snapshot,
                        },
                } if p == publication => Some((*id, *scope, *snapshot)),
                _ => None,
            });
            if let Some((id, _, _)) = scope {
                self.require_project_bridge_source(id, Source::Operator)?;
            }
            let latest = provenance["publication"] == serde_json::json!(publication);
            let contributors = feedback
                .guidance
                .iter()
                .map(|(key, meaning)| {
                    let id = aims.and_then(|(_, map)| map.get(key));
                    let status = id
                        .and_then(|id| entries.iter().find(|entry| entry.preference.id == *id))
                        .and_then(|entry| entry.status.clone());
                    serde_json::json!({"id":id,"requirement":key,"meaning":meaning,"status":status,
                    "eligible":latest && eligible.get(key)==Some(meaning)})
                })
                .collect::<Vec<_>>();
            publications.push(serde_json::json!({"publication":publication,"requirements_revision":feedback.requirements_revision,
                "current_revision":Some(feedback.requirements_revision)==current.requirements_revision,
                "scope_admission":scope.map(|(id,_,_)|id),"scope":scope.map(|(_,scope,_)|scope),
                "published_source":scope.map(|(_,_,snapshot)|snapshot),"latest":latest,"contributors":contributors}));
        }
        Ok(
            serde_json::json!({"version":"published-guidance-history@v1","state_root":self.store.head_root()?,
            "current_source":current.snapshot,"eligibility":provenance,"publications":publications,
            "grants_authority":false,"mutates_state":false}),
        )
    }
    /// Changed operator guidance after the latest actual return may resume pursuit.
    /// It is neither a new acceptance judgment nor action authority.
    pub(crate) fn acceptance_pending_feedback(&self) -> Result<bool> {
        let records = self.records()?;
        let Some(receipt_index) = records.iter().rposition(|(_, r)| {
            matches!(
                r,
                Record::Coding {
                    record: CodingRecord::Receipt { .. }
                }
            )
        }) else {
            return Ok(false);
        };
        let latest = records
            .iter()
            .enumerate()
            .rev()
            .find_map(|(i, (_, r))| match r {
                Record::Coding {
                    record: CodingRecord::AcceptanceFeedback { feedback },
                } => Some((i, feedback)),
                _ => None,
            });
        let Some((index, feedback)) = latest else {
            return Ok(false);
        };
        if index <= receipt_index {
            return Ok(false);
        }
        let prior = records[..receipt_index]
            .iter()
            .rev()
            .find_map(|(_, r)| match r {
                Record::Coding {
                    record: CodingRecord::AcceptanceFeedback { feedback },
                } => Some(feedback),
                _ => None,
            });
        if prior.is_some_and(|p| {
            p.requirements_revision == feedback.requirements_revision
                && p.guidance == feedback.guidance
        }) {
            return Ok(false);
        }
        let completion = self.project_acceptance_status()?;
        let (guidance, context) = self.acceptance_guidance(&completion)?;
        Ok(!guidance.is_empty() && context["status"] == "current_publication")
    }
    pub(super) fn acceptance_guidance(
        &self,
        current: &Completion,
    ) -> Result<(BTreeMap<String, String>, serde_json::Value)> {
        let records = self.records()?;
        let Some((publication, feedback)) = records.iter().rev().find_map(|(id, r)| match r {
            Record::Coding {
                record: CodingRecord::AcceptanceFeedback { feedback },
            } => Some((*id, feedback)),
            _ => None,
        }) else {
            return Ok((BTreeMap::new(), serde_json::json!({"status":"unpublished"})));
        };
        let events = self.events()?;
        let operator = |id: EventId| {
            events
                .iter()
                .any(|(_, e)| e.id == id && e.source == Source::Operator)
        };
        ensure!(
            operator(publication),
            "guidance publication requires operator provenance"
        );
        let scope = Hash::of(&[&serde_json::to_vec(&self.project_configuration()?)?]);
        let admitted = records.iter().rev().find_map(|(id, r)| match r {
            Record::Coding {
                record:
                    CodingRecord::AcceptanceGuidanceScope {
                        publication: p,
                        scope: s,
                        snapshot,
                    },
            } if *p == publication => Some((*id, *s, *snapshot)),
            _ => None,
        });
        if let Some((id, _, _)) = admitted {
            ensure!(operator(id), "guidance scope requires operator provenance");
        }
        let rebound = feedback.decision_record != current.decision_record
            || admitted.is_some_and(|(_, _, s)| s != current.snapshot);
        let budget = self.project_budget()?;
        let admission = self.project_resource_admission()?;
        let authority_open = admission.as_ref().is_some_and(|(_, a, _, closed)| {
            !closed
                && budget.active
                && !budget.stopped
                && !budget.exceeded
                && (budget.unlimited_mode
                    || (budget.tool_ms_charged < a.tool_ms_ceiling
                        && budget
                            .limits
                            .as_ref()
                            .is_some_and(|l| budget.elapsed_wall_ms < l.wall_ms)))
        });
        let compatible = Some(feedback.requirements_revision) == current.requirements_revision
            && admitted.is_none_or(|(_, s, _)| s == scope);
        let usable = compatible && admitted.is_some() && (!rebound || authority_open);
        let entries = Ledger::new(&self.store).collective()?.entries;
        let aims = self.acceptance_current_aims()?;
        let guidance = if usable {
            feedback
                .guidance
                .iter()
                .filter(|(key, _)| {
                    aims.get(*key).is_some_and(|id| {
                        entries.iter().any(|e| {
                            e.preference.id == *id
                                && matches!(
                                    e.status,
                                    Some(PreferenceStatus::Open | PreferenceStatus::Fulfilled)
                                )
                        })
                    })
                })
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect()
        } else {
            BTreeMap::new()
        };
        Ok((
            guidance,
            serde_json::json!({"version":"published-guidance-carry@v1","publication":publication,"scope_admission":admitted.map(|(id,_,_)|id),"requirements_revision":feedback.requirements_revision,"scope":scope,"published_source":admitted.map(|(_,_,s)|s),"current_source":current.snapshot,"current_decision":current.decision_record,"status":if !compatible{"intent_or_scope_changed"}else if !usable{"carry_requires_open_scope_bound_admission"}else if rebound{"unchanged_guidance_rebound"}else{"current_publication"},"grants_authority":false,"review_carried":false}),
        ))
    }
}
