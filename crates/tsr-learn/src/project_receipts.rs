//! Portable, reconstructible project receipt datasets. No native-label conversion.
//! This schema records measured observations, not semantic acceptance or inspection.
use crate::LearnError;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::Path,
};
use tsr_domain::{Event, EventId, EventKind, Hash, Observation, Source};
pub const SCHEMA: &str = "project-receipt-features@v1";
pub const DATASET: &str = "project-receipt-dataset@v1";
const CONTEXT: &str = "project-context-issued@v1";
fn invalid(s: impl Into<String>) -> LearnError {
    LearnError::Invalid(s.into())
}
pub fn schema_digest() -> Hash {
    Hash::of(&[SCHEMA.as_bytes(),b"case results count observations; passed explicit; legacy unknown; contexts issued not understanding; no review features"])
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Features {
    pub schema: String,
    pub schema_digest: Hash,
    pub prefix_digest: Hash,
    pub as_of: String,
    pub receipts: Vec<EventId>,
    pub case_results: u64,
    pub passed_cases: u64,
    pub failed_cases: u64,
    pub unknown_case_results: u64,
    pub receipts_without_case_results: u64,
    pub contexts_issued: Option<u64>,
    pub distinct_visible_files: Option<u64>,
    pub context_evidence: Vec<EventId>,
    pub semantic_review_used: bool,
}
/// Raw CAS bytes are preserved: a reserialization is not the original content digest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub events: Vec<(i64, Event)>,
    pub side_events: Vec<(i64, Event)>,
    pub blobs: BTreeMap<Hash, String>,
    pub as_of: String,
}
#[derive(Deserialize)]
struct Context {
    key: Hash,
    tip: EventId,
    snapshot: Hash,
    at: String,
    files: BTreeMap<String, Hash>,
}
pub(crate) fn clock(t: &str) -> Result<i128, LearnError> {
    if let Some(ms) = t.strip_prefix("unix-ms:") {
        return ms
            .parse::<u64>()
            .map(|v| i128::from(v) * 1_000_000)
            .map_err(|_| invalid("invalid unix-ms timestamp"));
    }
    crate::maturity::timestamp(t)
}
pub(crate) fn check_events(events: &[(i64, Event)]) -> Result<(), LearnError> {
    let mut ids = BTreeSet::new();
    let mut prior = None;
    for (seq, e) in events {
        if prior.is_some_and(|p| *seq <= p)
            || e.schema_version != Event::SCHEMA_VERSION
            || !ids.insert(e.id)
            || Event::new(
                e.task,
                e.parents.clone(),
                e.source,
                e.kind.clone(),
                &e.recorded_at,
            )
            .id != e.id
        {
            return Err(invalid("event identity/order mismatch"));
        }
        prior = Some(*seq);
    }
    Ok(())
}
pub(crate) fn blob(e: &Evidence, d: Hash) -> Result<serde_json::Value, LearnError> {
    let bytes = e
        .blobs
        .get(&d)
        .ok_or_else(|| invalid("missing receipt CAS evidence"))?;
    if Hash::of(&[bytes.as_bytes()]) != d {
        return Err(invalid("receipt CAS digest mismatch"));
    }
    Ok(serde_json::from_str(bytes)?)
}
/// Pure extractor shared by runtime and artifact reconstruction. No labels are inputs.
pub fn extract(e: &Evidence) -> Result<Features, LearnError> {
    check_events(&e.events)?;
    check_events(&e.side_events)?;
    let cutoff = crate::maturity::timestamp(&e.as_of)?;
    let mut f = Features {
        schema: SCHEMA.into(),
        schema_digest: schema_digest(),
        prefix_digest: Hash::of(&[&serde_json::to_vec(&e.events)?]),
        as_of: e.as_of.clone(),
        receipts: vec![],
        case_results: 0,
        passed_cases: 0,
        failed_cases: 0,
        unknown_case_results: 0,
        receipts_without_case_results: 0,
        contexts_issued: None,
        distinct_visible_files: None,
        context_evidence: vec![],
        semantic_review_used: false,
    };
    let mut tips = BTreeSet::new();
    let mut commitments = BTreeSet::new();
    let mut prior = None;
    for (_, event) in &e.events {
        let time = clock(&event.recorded_at)?;
        if time > cutoff || prior.is_some_and(|p| time < p) {
            return Err(invalid("feature chronology/cutoff mismatch"));
        }
        prior = Some(time);
        tips.insert(event.id);
        if let EventKind::Observed {
            observation: Observation::Evidence { source, digest },
        } = &event.kind
        {
            if source != "configured-cycle@v1" {
                continue;
            }
            let v = blob(e, *digest)?;
            if v["fact"] != "coding" || v["record"]["phase"] != "receipt" {
                continue;
            }
            let r = &v["record"];
            let commitment: EventId = serde_json::from_value(r["commitment"].clone())?;
            if event.source != Source::Rules || !commitments.insert(commitment) {
                return Err(invalid("duplicate/untrusted case receipt"));
            }
            f.receipts.push(event.id);
            let empty = vec![];
            let observations = match r.get("observations") {
                None => &empty,
                Some(v) => v
                    .as_array()
                    .ok_or_else(|| invalid("invalid case observations"))?,
            };
            if observations.is_empty() {
                f.receipts_without_case_results += 1;
            }
            for o in observations {
                f.case_results += 1;
                let passed = if matches!(r["verdict"].as_str(), Some("passed" | "failed_tests")) {
                    match o.get("passed") {
                        None | Some(serde_json::Value::Null) => None,
                        Some(v) => {
                            Some(v.as_bool().ok_or_else(|| invalid("invalid case outcome"))?)
                        }
                    }
                } else {
                    None
                };
                match passed {
                    Some(true) => f.passed_cases += 1,
                    Some(false) => f.failed_cases += 1,
                    None => f.unknown_case_results += 1,
                }
            }
        }
    }
    let mut keys = BTreeSet::new();
    let mut files = BTreeSet::new();
    for (_, event) in &e.side_events {
        if let EventKind::Observed {
            observation: Observation::Evidence { source, digest },
        } = &event.kind
        {
            if source != CONTEXT {
                continue;
            }
            if event.source != Source::Rules {
                return Err(invalid("invalid context source"));
            }
            let c: Context = serde_json::from_value(blob(e, *digest)?)?;
            if !tips.contains(&c.tip) || crate::maturity::timestamp(&c.at)? > cutoff {
                continue;
            }
            if clock(&event.recorded_at)? > cutoff {
                continue;
            }
            if Hash::of(&[&serde_json::to_vec(&(c.tip, c.snapshot, &c.files))?]) != c.key {
                return Err(invalid("context binding mismatch"));
            }
            if keys.insert(c.key) {
                f.context_evidence.push(event.id);
                files.extend(c.files.keys().cloned());
            }
        }
    }
    if !keys.is_empty() {
        f.contexts_issued = Some(keys.len() as u64);
        f.distinct_visible_files = Some(files.len() as u64);
    }
    Ok(f)
}
/// A source-bound snapshot, not an invented native commitment or mature label.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub prediction: EventId,
    pub prediction_event: Event,
    pub prediction_blob: String,
    pub candidate: tsr_domain::CandidateId,
    pub state: Hash,
    pub source_version: Hash,
    pub family: String,
    pub lineage: String,
    pub evidence: Evidence,
    pub features: Features,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub version: String,
    pub schema: String,
    pub schema_digest: Hash,
    pub as_of: String,
    pub source_root: Hash,
    pub rows: Vec<Row>,
    /// Export-time inventory; never a source of as-of features or labels.
    pub export_time_lifecycle_inventory: serde_json::Value,
    pub eligible_rows: usize,
    pub exclusion: String,
    pub digest: Hash,
}
impl Artifact {
    pub fn build(
        as_of: String,
        source_root: Hash,
        rows: Vec<Row>,
        lifecycle: serde_json::Value,
    ) -> Result<Self, LearnError> {
        let mut a = Self {
            version: DATASET.into(),
            schema: SCHEMA.into(),
            schema_digest: schema_digest(),
            as_of,
            source_root,
            rows,
            export_time_lifecycle_inventory: lifecycle,
            eligible_rows: 0,
            exclusion:
                "project common-event eligibility unresolved; no labels or native fit conversion"
                    .into(),
            digest: Hash::of(&[]),
        };
        a.digest = a.content_digest()?;
        a.validate()?;
        Ok(a)
    }
    fn content_digest(&self) -> Result<Hash, LearnError> {
        Ok(Hash::of(&[&serde_json::to_vec(&(
            &self.version,
            &self.schema,
            self.schema_digest,
            &self.as_of,
            self.source_root,
            &self.rows,
            &self.export_time_lifecycle_inventory,
            self.eligible_rows,
            &self.exclusion,
        ))?]))
    }
    pub fn validate(&self) -> Result<(), LearnError> {
        if self.version != DATASET
            || self.schema != SCHEMA
            || self.schema_digest != schema_digest()
            || self.digest != self.content_digest()?
        {
            return Err(invalid("project dataset schema/content binding mismatch"));
        }
        if self.eligible_rows != 0
            || self
                .export_time_lifecycle_inventory
                .get("label")
                .is_some_and(|v| !v.is_null())
        {
            return Err(invalid("project label admission unavailable; cannot promote reviewed status to common-event label"));
        }
        let cutoff = crate::maturity::timestamp(&self.as_of)?;
        let mut ids = BTreeSet::new();
        for row in &self.rows {
            let prediction = &row.prediction_event;
            check_events(&[(1, prediction.clone())])?;
            if prediction.id != row.prediction || clock(&prediction.recorded_at)? > cutoff {
                return Err(invalid("prediction receipt unavailable at artifact cutoff"));
            }
            let value = match &prediction.kind {
                EventKind::Observed {
                    observation: Observation::Evidence { source, digest },
                } if source == "common-event-shadow-receipt@v1"
                    && prediction.source == Source::Rules
                    && Hash::of(&[row.prediction_blob.as_bytes()]) == *digest =>
                {
                    serde_json::from_str::<serde_json::Value>(&row.prediction_blob)?
                }
                _ => return Err(invalid("invalid prediction provenance")),
            };
            if value["project_side_prefix_events"] != row.evidence.side_events.len()
                || value["project_side_prefix_digest"]
                    != serde_json::to_value(Hash::of(&[&serde_json::to_vec(
                        &row.evidence.side_events,
                    )?]))?
            {
                return Err(invalid("prediction side-prefix mismatch"));
            }
            if value["candidate"] != serde_json::to_value(row.candidate)?
                || value["state_root"] != serde_json::to_value(row.state)?
                || value["source_version"] != serde_json::to_value(row.source_version)?
                || value["declared_family"] != row.family
                || value["declared_lineage"] != row.lineage
                || value["prediction_as_of"] != row.evidence.as_of
                || value["prefix_digest"] != serde_json::to_value(row.features.prefix_digest)?
                || value["project_observation_features"] != serde_json::to_value(&row.features)?
            {
                return Err(invalid("prediction/feature binding mismatch"));
            }
            if row.family.trim().is_empty()
                || row.lineage.trim().is_empty()
                || row.family.len() > 128
                || row.lineage.len() > 512
            {
                return Err(invalid("bounded family/lineage required"));
            }
            if !ids.insert((row.candidate, row.state, row.source_version)) {
                return Err(invalid("duplicate project prediction row"));
            }
            if crate::maturity::timestamp(&row.evidence.as_of)? > cutoff
                || extract(&row.evidence)? != row.features
            {
                return Err(invalid("project feature evidence/cutoff mismatch"));
            }
            if row.evidence.events.iter().any(|(_,e)|matches!(e.kind,EventKind::Committed{candidate,..}|EventKind::Executed{candidate,..} if candidate==row.candidate)){return Err(invalid("post-action project row"))}
        }
        Ok(())
    }
    pub fn write(&self, path: &Path) -> Result<(), LearnError> {
        use std::io::Write;
        self.validate()?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?;
        file.write_all(&serde_json::to_vec_pretty(self)?)?;
        file.sync_all()?;
        Ok(())
    }
    pub fn load(path: &Path) -> Result<Self, LearnError> {
        use std::io::Read;
        let mut bytes = vec![];
        std::fs::File::open(path)?
            .take(64 * 1024 * 1024 + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() > 64 * 1024 * 1024 {
            return Err(invalid("project dataset byte limit"));
        }
        let a: Self = serde_json::from_slice(&bytes)?;
        a.validate()?;
        Ok(a)
    }
    pub fn require_fit_eligible(&self) -> Result<(), LearnError> {
        self.validate()?;
        Err(invalid("zero eligible project rows: native labels/maturity and compatible schema-dispatched fitting unavailable; no synthetic or unknown promotion"))
    }
}

pub fn rfc3339(stamp: &str) -> Result<String, LearnError> {
    if let Some(ms) = stamp.strip_prefix("unix-ms:") {
        let ms: u64 = ms
            .parse()
            .map_err(|_| invalid("invalid unix-ms timestamp"))?;
        if ms > 253_402_300_799_999 {
            return Err(invalid("timestamp out of RFC3339 range"));
        }
        let secs = ms / 1000;
        let z = (secs / 86400) as i64 + 719468;
        let era = z / 146097;
        let doe = (z - era * 146097) as u64;
        let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
        let year = yoe as i64 + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = doy - (153 * mp + 2) / 5 + 1;
        let month = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = year + i64::from(month <= 2);
        let rem = secs % 86400;
        return Ok(format!(
            "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}.{:03}Z",
            rem / 3600,
            rem % 3600 / 60,
            rem % 60,
            ms % 1000
        ));
    }
    crate::maturity::timestamp(stamp)?;
    Ok(stamp.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tsr_domain::TaskId;
    fn evidence() -> Evidence {
        let task = TaskId(Hash::of(&[b"fixture"]));
        let e = Event::new(
            task,
            vec![],
            Source::Rules,
            EventKind::TaskOpened { task },
            "2026-01-01T00:00:00Z",
        );
        Evidence {
            events: vec![(1, e)],
            side_events: vec![],
            blobs: BTreeMap::new(),
            as_of: "2026-01-01T00:00:01Z".into(),
        }
    }
    #[test]
    fn unknowns_and_schema_are_not_training_zeros() {
        let e = evidence();
        let f = extract(&e).unwrap();
        assert_eq!(f.contexts_issued, None);
        assert_eq!(f.distinct_visible_files, None);
        let a = Artifact::build(
            e.as_of.clone(),
            Hash::of(&[]),
            vec![],
            serde_json::json!({"label":null}),
        )
        .unwrap();
        assert!(a.require_fit_eligible().is_err());
        let mut spec = crate::features::spec_for_families(&[]);
        spec.version = SCHEMA.into();
        assert!(spec
            .validate_fitted()
            .unwrap_err()
            .contains("schema names/widths mismatch"));
        let mut forged = a;
        forged.eligible_rows = 151;
        forged.digest = forged.content_digest().unwrap();
        assert!(forged.validate().is_err());
    }
    #[test]
    fn receipt_reconstruction_requires_original_cas_and_time() {
        let mut e = evidence();
        let commitment = e.events[0].1.id;
        let bytes=serde_json::json!({"fact":"coding","record":{"phase":"receipt","commitment":commitment,"verdict":"passed","observations":[{"passed":true},{}]}}).to_string();
        let digest = Hash::of(&[bytes.as_bytes()]);
        e.blobs.insert(digest, bytes);
        let event = Event::new(
            e.events[0].1.task,
            vec![commitment],
            Source::Rules,
            EventKind::Observed {
                observation: Observation::Evidence {
                    source: "configured-cycle@v1".into(),
                    digest,
                },
            },
            "2026-01-01T00:00:01Z",
        );
        e.events.push((2, event));
        let f = extract(&e).unwrap();
        assert_eq!(f.passed_cases, 1);
        assert_eq!(f.unknown_case_results, 1);
        let mut missing = e.clone();
        missing.blobs.clear();
        assert!(extract(&missing).is_err());
        let mut future = e.clone();
        future.as_of = "2026-01-01T00:00:00Z".into();
        assert!(extract(&future).is_err());
        let mut mutated = e.clone();
        mutated.blobs.insert(digest, "{}".into());
        assert!(extract(&mutated).is_err());
        // A later review cannot enter the extractor's typed features.
        let review = "{\"fact\":\"coding\",\"record\":{\"phase\":\"acceptance_review\"}}";
        let d = Hash::of(&[review.as_bytes()]);
        e.blobs.insert(d, review.into());
        let review = Event::new(
            e.events[0].1.task,
            vec![e.events[1].1.id],
            Source::Operator,
            EventKind::Observed {
                observation: Observation::Evidence {
                    source: "configured-cycle@v1".into(),
                    digest: d,
                },
            },
            "2026-01-01T00:00:01Z",
        );
        e.events.push((3, review));
        let after = extract(&e).unwrap();
        assert_eq!(after.passed_cases, f.passed_cases);
        assert_eq!(after.unknown_case_results, f.unknown_case_results);
        assert!(!after.semantic_review_used);
    }
}
