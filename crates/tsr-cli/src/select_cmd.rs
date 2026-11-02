//! M3 selection only. Composition and semantic tension inference are not implied.
use std::path::Path;
use tsr_domain::{Hash, Snapshot, TaskId, Tune};
use tsr_ledger::selection::{record_selection, select, SelectionContext, SelectionTrace};

pub fn run(
    store_path: &Path,
    task: &str,
    tune_path: Option<&Path>,
    family: &str,
    repository: &str,
    seed: u64,
    minimum_depth: u32,
) -> Result<SelectionTrace, String> {
    if !store_path.is_file() {
        return Err("selection needs an existing controller store".into());
    }
    let store = tsr_store::Store::open(store_path).map_err(|e| e.to_string())?;
    let task = TaskId(Hash::from_hex(task).map_err(|e| e.to_string())?);
    let events = store.by_task(&task).map_err(|e| e.to_string())?;
    if events.is_empty() {
        return Err("unknown task".into());
    }
    let tune = match tune_path {
        Some(path) => serde_json::from_slice(&std::fs::read(path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?,
        None => Tune::default(),
    };
    let snapshot = Snapshot {
        task,
        version: events.len() as u64,
        root: store.head_root().map_err(|e| e.to_string())?,
        open_criteria: vec![],
        missing_evidence: vec![],
    };
    let ledger = tsr_ledger::Ledger::new(&store)
        .collective()
        .map_err(|e| e.to_string())?;
    let context = SelectionContext {
        family: family.into(),
        repository: repository.into(),
        minimum_depth,
    };
    let trace = select(&ledger, &snapshot, &tune, &context, seed).map_err(str::to_string)?;
    let artifacts = store_path.with_extension("artifacts");
    if let Ok(meta) = std::fs::symlink_metadata(&artifacts) {
        if !meta.is_dir() {
            return Err("artifact directory must not be a symlink or file".into());
        }
    }
    std::fs::create_dir_all(&artifacts).map_err(|e| e.to_string())?;
    // The clock is provenance only; ordering is the event sequence. No shell is used.
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs()
        .to_string();
    record_selection(
        &store,
        &snapshot,
        &trace,
        |digest, bytes| {
            use std::io::Write;
            let path = artifacts.join(digest.to_hex());
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut file) => {
                    file.write_all(bytes).map_err(|e| e.to_string())?;
                    file.sync_all().map_err(|e| e.to_string())
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    if !std::fs::symlink_metadata(&path)
                        .map_err(|e| e.to_string())?
                        .is_file()
                    {
                        return Err("nonregular evidence object".into());
                    }
                    if std::fs::read(path).map_err(|e| e.to_string())? != bytes {
                        return Err("evidence object content mismatch".into());
                    }
                    Ok(())
                }
                Err(error) => Err(error.to_string()),
            }
        },
        &now,
    )?;
    Ok(trace)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tsr_domain::{Event, EventKind, Observation, Source};
    #[test]
    fn selection_records_evidence_outside_candidate_state_and_preserves_identity() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("controller.db");
        let store = tsr_store::Store::open(&path).unwrap();
        let task = TaskId(Hash::of(&[b"task"]));
        let opened = Event::new(
            task,
            vec![],
            Source::Operator,
            EventKind::TaskOpened { task },
            "fixture",
        );
        store.append(&opened).unwrap();
        let trace = run(&path, &task.0.to_hex(), None, "family", "repo", 4, 1).unwrap();
        assert!(trace.abstention.is_some());
        let events = store.by_task(&task).unwrap();
        assert_eq!(events.len(), 2);
        let EventKind::Observed {
            observation: Observation::Evidence { digest, source },
        } = &events[1].1.kind
        else {
            panic!("missing evidence")
        };
        assert_eq!(source, "m3/select@v1");
        let bytes = std::fs::read(path.with_extension("artifacts").join(digest.to_hex())).unwrap();
        assert_eq!(*digest, Hash::of(&[&bytes]));
        assert_eq!(
            serde_json::from_slice::<SelectionTrace>(&bytes).unwrap(),
            trace
        );
    }
}
