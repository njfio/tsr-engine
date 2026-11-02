//! Read-only end-to-end audit; never executes candidate commands.
#[path = "inspection_receipt.rs"]
mod inspection_receipt;
use std::{collections::BTreeMap, path::Path};
use tsr_domain::{Event, EventKind, Hash, Observation, Source};
fn root(events: &[Event]) -> Hash {
    let ids: Vec<String> = events.iter().map(|e| e.id.to_string()).collect();
    Hash::of(&ids.iter().map(|s| s.as_bytes()).collect::<Vec<_>>())
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let dir = Path::new(&args[1]);
    let mode = &args[2];
    let events: Vec<Event> =
        serde_json::from_slice(&std::fs::read(dir.join("events.json")).unwrap()).unwrap();
    let tasks = tsr_batch::read_tasks(&dir.join("input/tasks.jsonl")).unwrap();
    assert_eq!(tasks.len(), 1);
    let task = &tasks[0];
    let task_id = tsr_batch::task_id(task);
    let boundary: Vec<serde_json::Value> = std::fs::read_to_string(dir.join("boundary.jsonl"))
        .unwrap_or_default()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let mut receipts = BTreeMap::new();
    let mut run = None;
    let mut edits = 0;
    for (i, event) in events.iter().enumerate() {
        assert_eq!(event.task, task_id);
        assert!(event.observed_at.is_none());
        let expected = Event::new(
            event.task,
            event.parents.clone(),
            event.source,
            event.kind.clone(),
            event.recorded_at.clone(),
        );
        assert_eq!(event.id, expected.id, "event content identity");
        assert_eq!(event.schema_version, Event::SCHEMA_VERSION);
        assert_eq!(
            event.parents,
            if i == 0 {
                vec![]
            } else {
                vec![events[i - 1].id]
            },
            "append-only parent chain"
        );
        if let EventKind::Observed {
            observation: Observation::Evidence { source, digest },
        } = &event.kind
        {
            if source.starts_with("broker/inspection-handle-receipt@v1:") {
                assert_eq!(event.source, Source::Rules);
                let r = inspection_receipt::Receipt::verify(source, *digest).unwrap();
                assert_eq!(r.task, task.id);
                if let Some(id) = run {
                    assert_eq!(id, r.run)
                } else {
                    run = Some(r.run)
                };
                let referenced = events[..i]
                    .iter()
                    .find(|e| e.id.to_string() == r.inspection_event)
                    .expect("prior inspection event");
                assert_eq!(referenced.source, Source::Rules);
                match &referenced.kind {
                    EventKind::Observed {
                        observation: Observation::Evidence { source, digest },
                    } => {
                        assert_eq!(source, "broker/inspect@v1");
                        assert_eq!(*digest, r.file_digest);
                    }
                    _ => panic!("not inspection"),
                }
                let request = boundary
                    .iter()
                    .find(|b| b["call"].as_u64() == Some(u64::from(r.call)))
                    .unwrap();
                assert_eq!(request["proposal"]["action"], "inspect");
                assert_eq!(request["proposal"]["path"], r.path);
                assert!(receipts.insert(r.handle.to_string(), r).is_none());
            }
        }
        if let EventKind::Proposed { candidate } = &event.kind {
            let matched = boundary
                .iter()
                .find(|b| {
                    let proposal: tsr_batch::broker_v2::Proposal =
                        serde_json::from_value(b["proposal"].clone()).unwrap();
                    Hash::of(&[&serde_json::to_vec(&proposal).unwrap()]) == candidate.args
                })
                .expect("candidate commits to actual submitted proposal");
            if matched["proposal"]["action"] == "edit" {
                let handle = matched["proposal"]["inspection_handle"].as_str().unwrap();
                let r = receipts.get(handle).expect("previous issuance");
                assert_eq!(matched["proposal"]["path"], r.path);
                assert_eq!(matched["inspection"]["handle"], handle);
                edits += 1;
            }
        }
    }
    let result_path = dir.join(format!("out/results-{mode}.jsonl"));
    let mut oracle = false;
    if result_path.exists() {
        let result: serde_json::Value =
            serde_json::from_slice(&std::fs::read(result_path).unwrap()).unwrap();
        assert_eq!(result["state_root_at_end"], root(&events).to_string());
        if !result["oracle"].is_null() {
            let evidence = &result["oracle"]["evidence"];
            let manifest = evidence["manifest"].as_str().unwrap();
            let index=events.iter().position(|e|matches!(&e.kind,EventKind::Observed{observation:Observation::Evidence{source,..}} if source=="final-oracle-receipts@v1")).unwrap();
            if let EventKind::Observed {
                observation: Observation::Evidence { digest, .. },
            } = &events[index].kind
            {
                assert_eq!(digest.to_string(), manifest)
            }
            let mode = if mode == "rules" {
                tsr_batch::Mode::Rules
            } else {
                tsr_batch::Mode::Record
            };
            let accepted = tsr_batch::oracle_receipts::verify(
                &dir.join("out"),
                manifest,
                task,
                mode,
                &root(&events[..index]).to_string(),
            )
            .unwrap();
            assert_eq!(accepted, evidence["verified_success"].as_bool().unwrap());
            assert_eq!(accepted, result["outcome"] == "accepted");
            oracle = true;
        }
    }
    println!(
        "{}",
        serde_json::json!({"events":events.len(),"issuance_receipts":receipts.len(),"bound_edits":edits,"oracle_receipts_verified":oracle})
    );
}
