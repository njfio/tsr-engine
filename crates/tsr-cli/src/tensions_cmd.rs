//! Explicit operator review. Reading suggestion JSON alone never records trusted links.
use std::path::Path;
use tsr_domain::{Hash, Snapshot, TaskId, Tune};
use tsr_ledger::{
    selection::{select, SelectionContext},
    tensions::{operator_review, record, ReviewInput, TensionTrace},
};
pub fn run(
    store_path: &Path,
    task: &str,
    input_path: &Path,
    operator_review_confirmed: bool,
    family: &str,
    repository: &str,
) -> Result<TensionTrace, String> {
    if !operator_review_confirmed {
        return Err(
            "requires explicit --operator-review; generator suggestions are not reviewed evidence"
                .into(),
        );
    }
    if !store_path.is_file() {
        return Err("existing controller store required".into());
    }
    let store = tsr_store::Store::open(store_path).map_err(|e| e.to_string())?;
    let task = TaskId(Hash::from_hex(task).map_err(|e| e.to_string())?);
    let events: Vec<_> = store
        .by_task(&task)
        .map_err(|e| e.to_string())?
        .into_iter()
        .map(|(_, e)| e)
        .collect();
    if events.is_empty() {
        return Err("unknown task".into());
    }
    if std::fs::metadata(input_path)
        .map_err(|e| e.to_string())?
        .len()
        > 256 * 1024
    {
        return Err("review input too large".into());
    }
    let input: ReviewInput =
        serde_json::from_slice(&std::fs::read(input_path).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let snapshot = Snapshot {
        task,
        version: events.len() as u64,
        root: store.head_root().map_err(|e| e.to_string())?,
        open_criteria: input.open_criteria.clone(),
        missing_evidence: vec![],
    };
    let ledger = tsr_ledger::Ledger::new(&store)
        .collective()
        .map_err(|e| e.to_string())?;
    let selection = select(
        &ledger,
        &snapshot,
        &Tune::default(),
        &SelectionContext {
            family: family.into(),
            repository: repository.into(),
            minimum_depth: 1,
        },
        0,
    )
    .map_err(str::to_string)?;
    let reviewed = operator_review(&ledger, &snapshot, &selection.selected, &events, input)?;
    let artifacts = store_path.with_extension("artifacts");
    if let Ok(meta) = std::fs::symlink_metadata(&artifacts) {
        if !meta.is_dir() {
            return Err("non-directory artifact root".into());
        }
    }
    std::fs::create_dir_all(&artifacts).map_err(|e| e.to_string())?;
    record(
        &store,
        reviewed,
        |digest, bytes| {
            use std::io::Write;
            let path = artifacts.join(digest.to_hex());
            match std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
            {
                Ok(mut f) => {
                    f.write_all(bytes).map_err(|e| e.to_string())?;
                    f.sync_all().map_err(|e| e.to_string())
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    if !std::fs::symlink_metadata(&path)
                        .map_err(|e| e.to_string())?
                        .is_file()
                        || std::fs::read(path).map_err(|e| e.to_string())? != bytes
                    {
                        return Err("invalid evidence object".into());
                    }
                    Ok(())
                }
                Err(e) => Err(e.to_string()),
            }
        },
        "operator-review",
    )
}
