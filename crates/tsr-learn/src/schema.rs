//! One schema dispatch contract for construction, probes, maturity rebuild and application.
use crate::{
    dataset::TaskIndexRow,
    features::{self, FeatureSpec, FeatureVector},
    project_receipts, LearnError,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tsr_domain::{ActionKind, Event, EventKind, Hash, TaskId};
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Input {
    pub artifacts: Vec<project_receipts::Artifact>,
    #[serde(default)]
    pub eligibility: Vec<crate::project_eligibility::Proof>,
}
fn invalid(s: &str) -> LearnError {
    LearnError::Invalid(s.into())
}
pub fn spec(version: &str, families: &[String]) -> Result<FeatureSpec, LearnError> {
    if version == features::FEATURE_SCHEMA_VERSION {
        return Ok(features::spec_for_families(families));
    }
    if version != project_receipts::SCHEMA {
        return Err(invalid("unknown feature schema"));
    }
    let mut s = features::spec_for_families(families);
    s.version = version.into();
    s.names.clear();
    s.numeric_widths.clear();
    s.standardize_mean.clear();
    s.standardize_std.clear();
    for family in families {
        s.names.push(features::family_feature_name(family));
        s.numeric_widths.push(0.0);
    }
    for action in features::ACTION_KINDS {
        s.names.push(features::action_feature_name(action));
        s.numeric_widths.push(0.0);
    }
    for name in [
        "case_results",
        "known_passed_cases",
        "known_failed_cases",
        "unknown_case_results",
        "receipts_without_case_results",
        "contexts_issued",
        "distinct_visible_files",
    ] {
        s.names.push(name.into());
        s.numeric_widths.push(2.0);
    }
    Ok(s)
}
pub fn validate_spec(s: &FeatureSpec) -> Result<(), LearnError> {
    if s.version == features::FEATURE_SCHEMA_VERSION {
        return Ok(());
    }
    let expected = spec(&s.version, &features::families_from_spec(s))?;
    if s.names != expected.names || s.numeric_widths != expected.numeric_widths {
        return Err(invalid("feature schema names/widths mismatch"));
    }
    Ok(())
}
pub type BuiltDataset = (
    crate::dataset::Dataset,
    crate::maturity::Maturity,
    Vec<(i64, Event)>,
    Vec<TaskIndexRow>,
);
impl Input {
    pub fn build(
        &self,
        as_of: &str,
        cfg: crate::dataset::SplitConfig,
        operator: &str,
    ) -> Result<BuiltDataset, LearnError> {
        let mut events = vec![];
        let mut index = vec![];
        for a in &self.artifacts {
            a.validate()?;
            let p = &a.export_time_lifecycle_inventory;
            let task: TaskId = serde_json::from_value(p["task"].clone())?;
            let episode: Vec<(i64, Event)> = serde_json::from_value(p["events"].clone())?;
            let row = a
                .rows
                .first()
                .ok_or_else(|| invalid("episode has no pre-action receipt rows"))?;
            let timestamp = episode
                .first()
                .ok_or_else(|| invalid("empty lifecycle"))?
                .1
                .recorded_at
                .clone();
            index.push(TaskIndexRow {
                task,
                family: row.family.clone(),
                lineage: row.lineage.clone(),
                clone_group: Some(
                    p["admission"]["training"]["clone_group"]
                        .as_str()
                        .unwrap_or(&row.lineage)
                        .into(),
                ),
                timestamp,
                tool_ms_ceiling: p["admission"]["tool_ms_ceiling"].as_u64(),
            });
            for (_, e) in episode {
                events.push((events.len() as i64 + 1, e));
            }
        }
        let labels = self.labels(as_of)?;
        let (ds, proof) = crate::maturity::build_schema(
            &events,
            &index,
            &labels,
            operator,
            cfg,
            self.digest()?,
            as_of,
            Some(self),
        )?;
        Ok((ds, proof, events, index))
    }
    pub fn write_dataset(
        &self,
        as_of: &str,
        cfg: crate::dataset::SplitConfig,
        operator: &str,
        out: &Path,
    ) -> Result<crate::dataset::Dataset, LearnError> {
        let (ds, proof, events, index) = self.build(as_of, cfg, operator)?;
        let available = crate::maturity::available_events(&events, as_of)?;
        let report = {
            let extractor = crate::probes::schema_extractor(&ds.spec, Some(self));
            crate::probes::run_all(&ds, &available, &index, &ds.spec, &extractor, None)
        };
        if !report.is_clean() {
            return Err(invalid(&format!(
                "receipt dataset leakage probes refused: {report:?}"
            )));
        }
        crate::maturity::write_schema(
            &ds,
            &proof,
            &events,
            &index,
            &self.labels(as_of)?,
            out,
            Some(self),
        )?;
        Ok(ds)
    }
    pub fn labels(&self, as_of: &str) -> Result<Vec<crate::decision::LabelRecord>, LearnError> {
        let mut tasks = std::collections::BTreeSet::new();
        let mut originals = std::collections::BTreeSet::new();
        let mut labels = vec![];
        for proof in &self.eligibility {
            if !tasks.insert(proof.task) {
                return Err(invalid("duplicate eligibility proof"));
            }
            let artifact = self
                .artifacts
                .iter()
                .find(|a| {
                    a.export_time_lifecycle_inventory["task"]
                        == serde_json::to_value(proof.task).unwrap()
                })
                .ok_or_else(|| invalid("eligibility has no episode artifact"))?;
            let result = crate::project_eligibility::evaluate(artifact, proof, as_of)?;
            if let Some(original) = result.original_task {
                if !originals.insert(original) {
                    return Err(invalid(
                        "repeated original task cannot manufacture independent episodes",
                    ));
                }
            }
            if let Some(label) = result.label {
                labels.push(label);
            }
        }
        Ok(labels)
    }
    pub fn digest(&self) -> Result<Hash, LearnError> {
        Ok(Hash::of(&[&serde_json::to_vec(self)?]))
    }
    pub fn load(dir: &Path) -> Result<Self, LearnError> {
        use std::io::Read;
        let mut bytes = vec![];
        std::fs::File::open(dir.join("feature-evidence.json"))?
            .take(64 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() > 64 * 1024 * 1024 {
            return Err(invalid("feature evidence byte limit"));
        }
        Ok(serde_json::from_slice(&bytes)?)
    }
    pub fn validate(
        &self,
        events: &[(i64, Event)],
        index: &[TaskIndexRow],
    ) -> Result<(), LearnError> {
        let mut tasks = std::collections::BTreeSet::new();
        for a in &self.artifacts {
            a.validate()?;
            let task: TaskId =
                serde_json::from_value(a.export_time_lifecycle_inventory["task"].clone())?;
            if !tasks.insert(task) {
                return Err(invalid("duplicate episode artifact"));
            }
            let projected: Vec<(i64, Event)> =
                serde_json::from_value(a.export_time_lifecycle_inventory["events"].clone())?;
            let actual: Vec<_> = events
                .iter()
                .filter(|(_, e)| e.task == task)
                .map(|(_, e)| e.clone())
                .collect();
            let expected: Vec<_> = projected.iter().map(|(_, e)| e.clone()).collect();
            // An as-of view may be a strict prefix of its original episode.
            if actual.len() > expected.len() || actual != expected[..actual.len()] {
                return Err(invalid("projected episode source mismatch"));
            }
            let idx = index
                .iter()
                .find(|i| i.task == task)
                .ok_or_else(|| invalid("project episode missing index"))?;
            if a.rows
                .iter()
                .any(|r| r.family != idx.family || r.lineage != idx.lineage)
            {
                return Err(invalid("project family/lineage mismatch"));
            }
            // Until source clone identity is admitted, grouping by lineage is conservative.
            if idx.clone_group.as_deref()
                != Some(
                    a.export_time_lifecycle_inventory["admission"]["training"]["clone_group"]
                        .as_str()
                        .unwrap_or(idx.lineage.as_str()),
                )
            {
                return Err(invalid("project clone grouping must retain lineage"));
            }
        }
        if events.iter().any(|(_, e)| !tasks.contains(&e.task)) {
            return Err(invalid("missing project episode evidence"));
        }
        Ok(())
    }
    pub fn row(
        &self,
        before: &[(i64, Event)],
        index: &TaskIndexRow,
    ) -> Result<&project_receipts::Row, LearnError> {
        let a = self
            .artifacts
            .iter()
            .find(|a| {
                a.export_time_lifecycle_inventory["task"]
                    == serde_json::to_value(index.task).unwrap()
            })
            .ok_or_else(|| invalid("missing receipt episode"))?;
        let projected: Vec<(i64, Event)> =
            serde_json::from_value(a.export_time_lifecycle_inventory["events"].clone())?;
        let position = before.len();
        let (_, next) = projected
            .get(position)
            .ok_or_else(|| invalid("missing receipt commitment"))?;
        if before
            .iter()
            .map(|(_, e)| e)
            .ne(projected[..position].iter().map(|(_, e)| e))
        {
            return Err(invalid("receipt extraction prefix mismatch"));
        }
        let candidate = match next.kind {
            EventKind::Committed { candidate, .. } => candidate,
            _ => return Err(invalid("receipt extraction is not precommitment")),
        };
        let mut rows = a.rows.iter().filter(|r| r.candidate == candidate);
        let row = rows
            .next()
            .ok_or_else(|| invalid("missing pre-action receipt"))?;
        if rows.next().is_some() {
            return Err(invalid("ambiguous pre-action receipt"));
        }
        Ok(row)
    }
}
pub fn receipt_vector(
    s: &FeatureSpec,
    e: &project_receipts::Evidence,
    index: &TaskIndexRow,
    action: ActionKind,
) -> Result<FeatureVector, LearnError> {
    validate_spec(s)?;
    if s.version != project_receipts::SCHEMA {
        return Err(invalid("receipt evidence/native model schema mismatch"));
    }
    let f = project_receipts::extract(e)?;
    let contexts = f
        .contexts_issued
        .ok_or_else(|| invalid("context issuance unknown; numeric feature unavailable"))?;
    let files = f
        .distinct_visible_files
        .ok_or_else(|| invalid("visible file count unknown; numeric feature unavailable"))?;
    let families = features::families_from_spec(s);
    let mut values = vec![];
    for family in &families {
        values.push(if *family == index.family { 1.0 } else { 0.0 });
    }
    for k in features::ACTION_KINDS {
        values.push(if k == action { 1.0 } else { 0.0 });
    }
    for n in [
        f.case_results,
        f.passed_cases,
        f.failed_cases,
        f.unknown_case_results,
        f.receipts_without_case_results,
        contexts,
        files,
    ] {
        if n > 9_007_199_254_740_992 {
            return Err(invalid("feature count exceeds exact numeric range"));
        }
        values.push(n as f64);
    }
    Ok(FeatureVector { values })
}
pub fn extract(
    s: &FeatureSpec,
    before: &[(i64, Event)],
    index: &TaskIndexRow,
    action: ActionKind,
    raw: Option<f64>,
    input: Option<&Input>,
) -> Result<FeatureVector, LearnError> {
    validate_spec(s)?;
    if s.version == features::FEATURE_SCHEMA_VERSION {
        return Ok(features::extract(
            s,
            before,
            index,
            action,
            &features::families_from_spec(s),
            raw,
        ));
    }
    let input = input.ok_or_else(|| invalid("receipt schema requires feature evidence"))?;
    let row = input.row(before, index)?;
    receipt_vector(s, &row.evidence, index, action)
}
