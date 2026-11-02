//! Local operator views and drafts; never included in provider context.
use super::*;
impl Engine {
    pub fn project_acceptance_requirements_view(&self) -> Result<BTreeMap<String, String>> {
        self.records()?
            .into_iter()
            .rev()
            .find_map(|(_, r)| match r {
                Record::Coding {
                    record: CodingRecord::AcceptanceRequirements { requirements },
                } => Some(requirements),
                _ => None,
            })
            .ok_or_else(|| anyhow::anyhow!("register operator requirements first"))
    }
    pub fn project_acceptance_inspect(&self) -> Result<serde_json::Value> {
        let latest_review = self
            .records()?
            .into_iter()
            .rev()
            .find_map(|(_, r)| match r {
                Record::Coding {
                    record: CodingRecord::AcceptanceReview { review },
                } => Some(review),
                _ => None,
            });
        Ok(
            serde_json::json!({"visibility":"private_operator_only","requirements":self.project_acceptance_requirements_view()?,
            "completion":self.project_acceptance_status()?,"latest_review":latest_review,"cycle":self.acceptance_cycle()?}),
        )
    }
    pub fn project_acceptance_draft(&self, feedback: bool) -> Result<serde_json::Value> {
        let current = self.project_acceptance_status()?;
        let requirements = self.project_acceptance_requirements_view()?;
        let revision = current
            .requirements_revision
            .ok_or_else(|| anyhow::anyhow!("requirements required"))?;
        if feedback {
            Ok(serde_json::to_value(Feedback {
                requirements_revision: revision,
                decision_record: current.decision_record,
                guidance: requirements
                    .keys()
                    .map(|id| (id.clone(), String::new()))
                    .collect(),
            })?)
        } else {
            Ok(serde_json::to_value(Review {
                snapshot: current.snapshot,
                requirements_revision: revision,
                test_receipt: current.test_receipt,
                decision: Decision::Unresolved,
                evidence: requirements
                    .keys()
                    .map(|id| {
                        (
                            id.clone(),
                            Evidence {
                                met: false,
                                evidence: String::new(),
                            },
                        )
                    })
                    .collect(),
            })?)
        }
    }
}
