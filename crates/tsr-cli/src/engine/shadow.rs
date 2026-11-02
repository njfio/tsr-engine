//! Separate-store shadow evidence: cannot perturb main state roots or seeded decisions.
use super::*;
const CONFIG: &str = "common-event-shadow-config@v1";
const RECEIPT: &str = "common-event-shadow-receipt@v1";
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Update {
    pub state_root: Hash,
    pub config: tsr_learn::shadow::Config,
    pub reason: String,
}
impl Engine {
    pub(crate) fn shadow_store(&self) -> Result<Engine> {
        let path = self.directory.join("shadow");
        if let Ok(metadata) = fs::symlink_metadata(&path) {
            ensure!(
                metadata.is_dir() && !metadata.file_type().is_symlink(),
                "shadow store must be a local directory"
            );
        }
        // Operator-owned foreground directory, not a sandbox against concurrent hostile writes.
        // Reject existing aliases before SQLite or CAS can follow them.
        for name in [
            "events.sqlite",
            "events.sqlite-wal",
            "events.sqlite-shm",
            "cas",
        ] {
            let entry = path.join(name);
            if let Ok(meta) = fs::symlink_metadata(&entry) {
                ensure!(!meta.file_type().is_symlink(), "shadow storage symlink");
                #[cfg(unix)]
                {
                    use std::os::unix::fs::MetadataExt;
                    if meta.is_file() {
                        ensure!(meta.nlink() == 1, "shadow storage hardlink");
                    }
                }
                if name == "cas" {
                    ensure!(meta.is_dir(), "shadow CAS must be a directory");
                    for child in fs::read_dir(&entry)? {
                        let child = child?;
                        let m = fs::symlink_metadata(child.path())?;
                        ensure!(
                            m.is_file() && !m.file_type().is_symlink(),
                            "shadow CAS alias"
                        );
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::MetadataExt;
                            ensure!(m.nlink() == 1, "shadow CAS hardlink");
                        }
                    }
                }
            }
        }
        Engine::open(&path)
    }
    fn shadow_records(&self) -> Result<Vec<(String, serde_json::Value)>> {
        let mut result = vec![];
        for (_, e) in self.events()? {
            if let EventKind::Observed {
                observation: Observation::Evidence { source, digest },
            } = e.kind
            {
                if source != CONFIG && source != RECEIPT {
                    continue;
                }
                ensure!(
                    e.source
                        == if source == CONFIG {
                            Source::Operator
                        } else {
                            Source::Rules
                        },
                    "invalid shadow provenance"
                );
                let bytes = fs::read(self.directory.join("cas").join(digest.to_hex()))?;
                ensure!(Hash::of(&[&bytes]) == digest, "changed shadow evidence");
                result.push((source, serde_json::from_slice(&bytes)?));
            }
        }
        Ok(result)
    }
    pub fn configure_shadow(&self, update: Update) -> Result<serde_json::Value> {
        ensure!(
            update.state_root == self.store.head_root()?,
            "stale shadow configuration"
        );
        ensure!(
            !update.reason.trim().is_empty() && update.reason.len() <= 512,
            "bounded operator reason required"
        );
        ensure!(
            update.config.event == tsr_learn::DECISION_EVENT,
            "shadow event mismatch"
        );
        ensure!(
            !update.config.family.is_empty()
                && update.config.family.len() <= 128
                && !update.config.lineage.is_empty()
                && update.config.lineage.len() <= 512,
            "bounded declared family/lineage required"
        );
        if let Some(cutoff) = &update.config.training_as_of {
            ensure!(
                tsr_learn::maturity::timestamp(cutoff)?
                    <= tsr_learn::maturity::timestamp(&crate::hook::now_rfc3339())?,
                "future training cutoff"
            );
        }
        let side = self.shadow_store()?;
        side.write(|| {
            let digest = side.blob(&serde_json::to_vec(&update)?)?;
            side.append(
                EventKind::Observed {
                    observation: Observation::Evidence {
                        digest,
                        source: CONFIG.into(),
                    },
                },
                Source::Operator,
            )?;
            Ok(())
        })?;
        self.shadow_view()
    }
    pub fn shadow_view(&self) -> Result<serde_json::Value> {
        let records = if self.directory.join("shadow").exists() {
            self.shadow_store()?.shadow_records()?
        } else {
            vec![]
        };
        Ok(
            serde_json::json!({"state_root":self.store.head_root()?,"records":records,"selection_influence":false}),
        )
    }
    pub(super) fn record_shadow(
        &self,
        state: Hash,
        source_version: Hash,
        candidate: &Candidate,
        events: &[(i64, Event)],
        adapter: &str,
    ) {
        if !self.directory.join("shadow").exists() {
            return;
        }
        if let Err(error) =
            self.try_record_shadow(state, source_version, candidate, events, adapter)
        {
            eprintln!("shadow evidence unavailable (factual action unchanged): {error}");
        }
    }
    fn try_record_shadow(
        &self,
        state: Hash,
        source_version: Hash,
        candidate: &Candidate,
        events: &[(i64, Event)],
        adapter: &str,
    ) -> Result<()> {
        let side = self.shadow_store()?;
        let records = side.shadow_records()?;
        let update: Option<Update> = records
            .iter()
            .rev()
            .find(|(s, _)| s == CONFIG)
            .map(|(_, value)| serde_json::from_value(value.clone()))
            .transpose()?;
        // Explicit prospective training admission also requests evidence collection.
        // No model pin is invented to bootstrap the data needed to train a model.
        let training = if adapter == "bounded-project@v1" {
            self.project_resource_admission()?
                .and_then(|(_, a, _, closed)| if closed { None } else { a.training })
        } else {
            None
        };
        if update.is_none() && training.is_none() {
            return Ok(());
        }
        let as_of = super::adaptive::coding::project::lifecycle::now()?;
        // Only an explicitly admitted project episode can provide native lifecycle features.
        let lifecycle_attempt = if adapter == "bounded-project@v1" {
            Some(self.project_lifecycle_prefix(events))
        } else {
            None
        };
        let lifecycle_error = lifecycle_attempt
            .as_ref()
            .and_then(|r| r.as_ref().err())
            .map(|e| e.to_string());
        let lifecycle = lifecycle_attempt.and_then(Result::ok);
        if let Some(p) = &lifecycle {
            ensure!(
                !p.closed,
                "project lifecycle closed; no new episode authority"
            );
        }
        let native_events = lifecycle
            .as_ref()
            .map(|p| p.events.as_slice())
            .unwrap_or(events);
        let receipt_evidence = if adapter == "bounded-project@v1" {
            self.project_feature_evidence(events, &as_of).ok()
        } else {
            None
        };
        let native_adapter = if receipt_evidence.is_some() && lifecycle.is_some() {
            "project-receipt-common-event@v1"
        } else if lifecycle
            .as_ref()
            .is_some_and(|p| p.native_feature_compatible)
        {
            "native-common-event@v1"
        } else {
            adapter
        };
        let (family, lineage) = if let Some(update) = &update {
            (update.config.family.clone(), update.config.lineage.clone())
        } else {
            let training = training.as_ref().expect("admission checked above");
            (training.family.clone(), training.lineage.clone())
        };
        let prediction = if let Some(update) = &update {
            tsr_learn::shadow::evaluate_receipts(
                &update.config,
                tsr_learn::shadow::Snapshot {
                    tool_ms_ceiling: lifecycle.as_ref().map(|p| p.admission.tool_ms_ceiling),
                    state,
                    source_version,
                    candidate,
                    events: native_events,
                    as_of: &as_of,
                    adapter: native_adapter,
                },
                receipt_evidence.as_ref(),
            )
        } else {
            Ok(
                serde_json::json!({"status":"evidence_only","event":tsr_learn::DECISION_EVENT,
            "candidate":candidate.id,"state_root":state,"source_version":source_version,
            "bundle_id":null,"calibrated_probability":null,"prediction_as_of":as_of,
            "selection_influence":false,"label":null,"model_evaluation_requested":false}),
            )
        };
        let mut value = match prediction {
            Ok(value) => value,
            Err(error) => {
                serde_json::json!({"status":"unavailable","reason":error.to_string(),"event":tsr_learn::DECISION_EVENT,
                "candidate":candidate.id,"state_root":state,"source_version":source_version,"bundle_id":update.as_ref().map(|u|u.config.bundle_id),
                "prediction_as_of":as_of,"training_as_of":update.as_ref().and_then(|u|u.config.training_as_of.as_ref()),"adapter":adapter,
                "prefix_digest":Hash::of(&[&serde_json::to_vec(events)?]),"selection_influence":false,"label":null})
            }
        };
        value["adapter"] = adapter.into();
        // Binding fields are required even when no model artifact could be loaded.
        value["prefix_events"] = events.len().into();
        value["prefix_digest"] = serde_json::to_value(Hash::of(&[&serde_json::to_vec(events)?]))?;
        value["declared_family"] = family.into();
        value["declared_lineage"] = lineage.into();
        if let Some(error) = lifecycle_error {
            value["lifecycle_unavailable"] = error.into();
        }
        if adapter == "bounded-project@v1" {
            value["project_observation_features"] = match receipt_evidence {
                Some(evidence) => {
                    value["project_side_prefix_events"] = evidence.side_events.len().into();
                    value["project_side_prefix_digest"] =
                        serde_json::to_value(Hash::of(&[&serde_json::to_vec(
                            &evidence.side_events,
                        )?]))?;
                    match tsr_learn::project_receipts::extract(&evidence) {
                        Ok(features) => serde_json::to_value(features)?,
                        Err(error) => {
                            serde_json::json!({"status":"unavailable","reason":error.to_string()})
                        }
                    }
                }
                None => {
                    serde_json::json!({"status":"unavailable","reason":"receipt evidence unavailable"})
                }
            };
        }
        if let Some(p) = &lifecycle {
            value["project_lifecycle"] = serde_json::to_value(p)?;
            value["accounting_scope"] =
                "prospective instrumented resources only; legacy/uninstrumented usage unknown"
                    .into();
        }
        side.write(|| {
            let digest = side.blob(&serde_json::to_vec(&value)?)?;
            side.append(
                EventKind::Observed {
                    observation: Observation::Evidence {
                        digest,
                        source: RECEIPT.into(),
                    },
                },
                Source::Rules,
            )?;
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn update(root: Hash, path: &Path) -> Update {
        Update {
            state_root: root,
            reason: "test diagnostic only".into(),
            config: tsr_learn::shadow::Config {
                bundle_directory: path.join("missing-bundle"),
                bundle_id: Hash::of(&[b"pin"]),
                dataset_directory: None,
                event: tsr_learn::DECISION_EVENT.into(),
                family: "fixture".into(),
                lineage: "fixture".into(),
                training_as_of: None,
            },
        }
    }
    #[test]
    fn unavailable_receipts_restart_without_mutating_factual_prefix() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let root = e.store.head_root()?;
        let events = e.events()?;
        e.configure_shadow(update(root, dir.path()))?;
        let c = Candidate::new(ActionKind::Inspect, Source::Intuition, root, vec![], vec![])?;
        for adapter in ["notebook@v1", "bounded-project@v1", "adaptive@v1"] {
            e.record_shadow(root, root, &c, &events, adapter);
        }
        assert_eq!(e.store.head_root()?, root);
        assert_eq!(e.events()?, events);
        let view = e.shadow_view()?;
        assert_eq!(view["records"].as_array().unwrap().len(), 4);
        for r in view["records"].as_array().unwrap().iter().skip(1) {
            assert_eq!(r[1]["status"], "unavailable");
            assert_eq!(r[1]["candidate"], serde_json::to_value(c.id)?);
            assert!(r[1]["label"].is_null());
            assert_eq!(r[1]["selection_influence"], false);
        }
        drop(e);
        let e = Engine::open(dir.path())?;
        assert_eq!(e.shadow_view()?, view);
        let mut stale = update(Hash::of(&[b"stale"]), dir.path());
        assert!(e.configure_shadow(stale).is_err());
        stale = update(root, dir.path());
        stale.config.event = "coverage".into();
        assert!(e.configure_shadow(stale).is_err());
        assert_eq!(e.shadow_view()?, view);
        Ok(())
    }
    #[test]
    fn forged_shadow_provenance_is_rejected_and_cannot_mutate_factual_state() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let root = e.store.head_root()?;
        let side = e.shadow_store()?;
        let digest = side.blob(&serde_json::to_vec(&update(root, dir.path()))?)?;
        side.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    source: CONFIG.into(),
                    digest,
                },
            },
            Source::Intuition,
        )?;
        assert!(e.shadow_view().is_err());
        let c = Candidate::new(ActionKind::Inspect, Source::Intuition, root, vec![], vec![])?;
        e.record_shadow(root, root, &c, &[], "notebook@v1");
        assert_eq!(root, e.store.head_root()?);
        assert!(e.events()?.is_empty());
        Ok(())
    }
    #[test]
    #[cfg(unix)]
    fn storage_aliases_cannot_touch_factual_log() -> Result<()> {
        use std::os::unix::fs::symlink;
        for alias in ["db_symlink", "db_hardlink", "cas_symlink", "dangling_root"] {
            let dir = tempfile::tempdir()?;
            let e = Engine::open(dir.path())?;
            let root = e.store.head_root()?;
            fs::create_dir(dir.path().join("shadow"))?;
            match alias {
                "dangling_root" => {
                    fs::remove_dir(dir.path().join("shadow"))?;
                    symlink(dir.path().join("absent"), dir.path().join("shadow"))?;
                }
                "db_symlink" => symlink(
                    dir.path().join("events.sqlite"),
                    dir.path().join("shadow/events.sqlite"),
                )?,
                "db_hardlink" => fs::hard_link(
                    dir.path().join("events.sqlite"),
                    dir.path().join("shadow/events.sqlite"),
                )?,
                _ => symlink(dir.path().join("cas"), dir.path().join("shadow/cas"))?,
            }
            assert!(e.configure_shadow(update(root, dir.path())).is_err());
            assert_eq!(e.store.head_root()?, root);
            assert!(e.events()?.is_empty());
        }
        Ok(())
    }
}
