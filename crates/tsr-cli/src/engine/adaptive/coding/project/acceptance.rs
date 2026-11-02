//! Independent operator acceptance, separate from test observations and proposer authority.
use super::*;
mod cycle;
mod guidance;
pub use cycle::{Feedback, HistoricalReview};
mod history;
mod operator;
pub mod reviewer;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub met: bool,
    pub evidence: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Accept,
    Reject,
    Unresolved,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    pub snapshot: Hash,
    pub requirements_revision: EventId,
    pub test_receipt: Option<EventId>,
    pub decision: Decision,
    pub evidence: BTreeMap<String, Evidence>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Completion {
    pub status: String,
    pub accepted: bool,
    pub desire_status: String,
    pub semantic_completeness: String,
    pub snapshot: Hash,
    pub requirements_revision: Option<EventId>,
    pub test_receipt: Option<EventId>,
    pub test_verdict: Option<Verdict>,
    pub decision_record: Option<EventId>,
}
impl Engine {
    /// Read-only fold. Legacy projects are unresolved until independent acceptance exists.
    pub fn project_acceptance_status(&self) -> Result<Completion> {
        let spec = self.project_configuration()?;
        let snapshot = digest(&self.project_snapshot(&spec)?)?;
        let records = self.records()?;
        let requirements = records.iter().rev().find_map(|(id, r)| match r {
            Record::Coding {
                record: CodingRecord::AcceptanceRequirements { requirements },
            } => Some((*id, requirements)),
            _ => None,
        });
        let receipt = records.iter().rev().find_map(|(id, r)| match r {
            Record::Coding {
                record:
                    CodingRecord::Receipt {
                        verdict,
                        source_after,
                        ..
                    },
            } => Some((*id, verdict.clone(), *source_after)),
            _ => None,
        });
        let pending = records
            .iter()
            .rev()
            .find_map(|(_, r)| match r {
                Record::Pending { .. } => Some(true),
                Record::Returned { .. } | Record::Censored { .. } => Some(false),
                _ => None,
            })
            .unwrap_or(false)
            || self.project_information_pending()?.is_some();
        let review = records.iter().rev().find_map(|(id, r)| match r {
            Record::Coding {
                record: CodingRecord::AcceptanceReview { review },
            } => Some((*id, review)),
            _ => None,
        });
        let mut completion = Completion {
            status: if requirements.is_none() {
                "requirements_missing"
            } else {
                "review_required"
            }
            .into(),
            accepted: false,
            desire_status: "open".into(),
            semantic_completeness: "unresolved".into(),
            snapshot,
            requirements_revision: requirements.map(|(id, _)| id),
            test_receipt: receipt.as_ref().map(|r| r.0),
            test_verdict: receipt.as_ref().map(|r| r.1.clone()),
            decision_record: review.map(|(id, _)| id),
        };
        if let Some((_, review)) = review {
            if Some(review.requirements_revision) != completion.requirements_revision
                || review.snapshot != snapshot
                || review.test_receipt != completion.test_receipt
                || pending
            {
                completion.status = "stale_review".into();
            } else {
                match review.decision {
                    Decision::Reject => {
                        completion.status = "rejected".into();
                        completion.semantic_completeness = "not_met".into();
                    }
                    Decision::Unresolved => completion.status = "unresolved".into(),
                    Decision::Accept => {
                        if receipt
                            .as_ref()
                            .is_some_and(|r| r.1 == Verdict::Passed && r.2 == Some(snapshot))
                        {
                            completion.status = "accepted".into();
                            completion.accepted = true;
                            completion.desire_status = "fulfilled".into();
                            completion.semantic_completeness = "accepted_by_operator".into();
                        } else {
                            completion.status = "test_evidence_required".into();
                        }
                    }
                }
            }
        }
        if completion.accepted
            && self
                .supplemental_for(snapshot, completion.requirements_revision)?
                .iter()
                .any(|o| o.findings.values().any(|v| *v == reviewer::Finding::NotMet))
        {
            completion.accepted = false;
            completion.status = "supplemental_conflict_review_required".into();
            completion.desire_status = "open".into();
            completion.semantic_completeness = "conflicting_independent_evidence".into();
        }
        Ok(completion)
    }
    /// Requirement text and evidence remain controller-only; never inserted into proposal context.
    pub fn project_acceptance_requirements(
        &self,
        requirements: BTreeMap<String, String>,
    ) -> Result<Completion> {
        validate_requirements(&requirements)?;
        self.write(|| {
            self.project_configuration()?;
            ensure!(
                self.objective_config()?.is_none(),
                "objective criteria frozen; create a separately admitted objective instead"
            );
            ensure!(
                self.adaptive_state_inner()?.pending.is_none()
                    && self.project_information_pending()?.is_none(),
                "finish pending execution before changing acceptance requirements"
            );
            let revision = self.adaptive_record(
                &Record::Coding {
                    record: CodingRecord::AcceptanceRequirements {
                        requirements: requirements.clone(),
                    },
                },
                Source::Operator,
            )?;
            self.acceptance_admit_aims(revision, &requirements)?;
            self.project_acceptance_status()
        })
    }
    /// Explicit operator attestation. Test results or generated rationale never create this record.
    pub fn project_acceptance_review(&self, review: Review) -> Result<Completion> {
        self.write(|| self.acceptance_review_inner(review))
    }
    fn acceptance_review_inner(&self, review: Review) -> Result<Completion> {
        let unknown = if review.decision == Decision::Unresolved {
            review
                .evidence
                .iter()
                .filter(|(_, e)| !e.met)
                .map(|(k, _)| k.clone())
                .collect()
        } else {
            BTreeSet::new()
        };
        self.acceptance_review_with_uncertainty(review, unknown)
    }
    fn acceptance_review_with_uncertainty(
        &self,
        review: Review,
        unknown: BTreeSet<String>,
    ) -> Result<Completion> {
        ensure!(
            unknown.is_subset(&review.evidence.keys().cloned().collect())
                && (unknown.is_empty() || review.decision == Decision::Unresolved)
                && unknown.iter().all(|key| !review.evidence[key].met),
            "invalid acceptance uncertainty"
        );
        ensure!(
            self.adaptive_state_inner()?.pending.is_none()
                && self.project_information_pending()?.is_none(),
            "cannot review pending execution"
        );
        let current = self.project_acceptance_status()?;
        ensure!(
            review.snapshot == current.snapshot
                && Some(review.requirements_revision) == current.requirements_revision
                && review.test_receipt == current.test_receipt,
            "stale acceptance evidence binding"
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
            .ok_or_else(|| anyhow::anyhow!("operator requirements required"))?;
        ensure!(
            review.evidence.keys().eq(requirements.keys())
                && review.evidence.values().all(|e| short(&e.evidence)),
            "supply bounded independent evidence for every requirement"
        );
        let all_met = review.evidence.values().all(|e| e.met);
        if review.decision == Decision::Accept {
            ensure!(
                !self
                    .supplemental_for(current.snapshot, current.requirements_revision)?
                    .iter()
                    .any(|o| o.findings.values().any(|v| *v == reviewer::Finding::NotMet)),
                "resolve current supplemental counterexample before acceptance"
            );
            ensure!(
                all_met && current.test_verdict == Some(Verdict::Passed),
                "acceptance requires all requirements met and passing tests"
            );
            let spec = self.project_configuration()?;
            let snapshot = digest(&self.project_snapshot(&spec)?)?;
            ensure!(
                self.records()?.iter().rev().find_map(|(_, r)| match r {
                    Record::Coding {
                        record: CodingRecord::Receipt { source_after, .. },
                    } => Some(*source_after == Some(snapshot)),
                    _ => None,
                }) == Some(true),
                "tests must bind current source"
            );
        } else if review.decision == Decision::Reject {
            ensure!(!all_met, "rejection requires an unmet requirement");
        }
        let record = self.adaptive_record(
            &Record::Coding {
                record: CodingRecord::AcceptanceReview {
                    review: review.clone(),
                },
            },
            Source::Operator,
        )?;
        self.adaptive_record(
            &Record::Coding {
                record: CodingRecord::AcceptanceUncertainty {
                    review: record,
                    unknown: unknown.clone(),
                },
            },
            Source::Rules,
        )?;
        self.acceptance_return(record, &review, &unknown)?;
        self.project_acceptance_status()
    }
}

pub fn validate_requirements(requirements: &BTreeMap<String, String>) -> Result<()> {
    ensure!(
        !requirements.is_empty()
            && requirements.len() <= 16
            && requirements
                .iter()
                .all(|(id, text)| short(id) && short(text)),
        "1..16 bounded operator requirements required"
    );
    let unique: std::collections::BTreeSet<_> = requirements.values().map(|s| s.trim()).collect();
    ensure!(
        unique.len() == requirements.len(),
        "duplicate requirement text aliases one desire; provide one key per distinct requirement"
    );
    Ok(())
}
