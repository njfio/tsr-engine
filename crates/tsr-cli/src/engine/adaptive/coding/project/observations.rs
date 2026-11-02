//! Pre-action project observations, separate from native action-count features.
use super::*;
pub use tsr_learn::project_receipts::{schema_digest, Features, SCHEMA};
const CONTEXT: &str = "project-context-issued@v1";
#[derive(Serialize, Deserialize)]
struct Context {
    key: Hash,
    tip: EventId,
    snapshot: Hash,
    at: String,
    files: BTreeMap<String, Hash>,
}
impl Engine {
    pub(super) fn record_project_context(
        &self,
        files: &[serde_json::Value],
        snapshot: Hash,
    ) -> Result<()> {
        if !self.directory.join("shadow").exists() {
            return Ok(());
        }
        // No implicit admission for legacy runs; context logging only inside an admitted episode.
        let lifecycle = self.project_lifecycle_view()?;
        if lifecycle.closed {
            return Ok(());
        }
        let tip = self
            .events()?
            .last()
            .ok_or_else(|| anyhow::anyhow!("empty project"))?
            .1
            .id;
        let files: BTreeMap<String, Hash> = files
            .iter()
            .filter(|f| f.get("source").is_some())
            .map(|f| {
                Ok((
                    f["path"]
                        .as_str()
                        .ok_or_else(|| anyhow::anyhow!("context path"))?
                        .into(),
                    serde_json::from_value(f["source_digest"].clone())?,
                ))
            })
            .collect::<Result<_>>()?;
        let key = Hash::of(&[&serde_json::to_vec(&(tip, snapshot, &files))?]);
        let side = self.shadow_store()?;
        side.write(|| {
            for (_, e) in side.events()? {
                if let EventKind::Observed {
                    observation: Observation::Evidence { source, digest },
                } = e.kind
                {
                    if source == CONTEXT {
                        let bytes = fs::read(side.directory.join("cas").join(digest.to_hex()))?;
                        ensure!(
                            Hash::of(&[&bytes]) == digest && e.source == Source::Rules,
                            "invalid context provenance"
                        );
                        let c: Context = serde_json::from_slice(&bytes)?;
                        if c.key == key {
                            return Ok(());
                        }
                    }
                }
            }
            let c = Context {
                key,
                tip,
                snapshot,
                at: lifecycle::now()?,
                files,
            };
            let digest = side.blob(&serde_json::to_vec(&c)?)?;
            side.append(
                EventKind::Observed {
                    observation: Observation::Evidence {
                        source: CONTEXT.into(),
                        digest,
                    },
                },
                Source::Rules,
            )?;
            Ok(())
        })
    }
    pub(crate) fn project_receipt_features(
        &self,
        events: &[(i64, Event)],
        as_of: &str,
        schema: &str,
    ) -> Result<Features> {
        ensure!(schema == SCHEMA, "incompatible project feature schema");
        Ok(tsr_learn::project_receipts::extract(
            &self.project_feature_evidence(events, as_of)?,
        )?)
    }
    pub(crate) fn project_feature_evidence(
        &self,
        events: &[(i64, Event)],
        as_of: &str,
    ) -> Result<tsr_learn::project_receipts::Evidence> {
        let side_events = if self.directory.join("shadow").exists() {
            self.shadow_store()?.events()?
        } else {
            vec![]
        };
        let mut blobs = BTreeMap::new();
        for (records, root) in [
            (events, self.directory.clone()),
            (side_events.as_slice(), self.directory.join("shadow")),
        ] {
            for (_, e) in records {
                if let EventKind::Observed {
                    observation: Observation::Evidence { source, digest },
                } = &e.kind
                {
                    if source == "configured-cycle@v1"
                        || source == CONTEXT
                        || source == "common-event-shadow-receipt@v1"
                        || source == "project-common-event-lifecycle@v1"
                        || source == "project-resource-accounting@v1"
                        || source == tsr_learn::project_eligibility::SOURCE
                    {
                        let bytes = fs::read(root.join("cas").join(digest.to_hex()))?;
                        ensure!(Hash::of(&[&bytes]) == *digest, "feature CAS changed");
                        blobs.insert(*digest, String::from_utf8(bytes)?);
                    }
                }
            }
        }
        Ok(tsr_learn::project_receipts::Evidence {
            events: events.to_vec(),
            side_events,
            blobs,
            as_of: as_of.into(),
        })
    }
    pub fn project_features(&self) -> Result<Features> {
        self.project_receipt_features(&self.events()?, &lifecycle::now()?, SCHEMA)
    }
}
