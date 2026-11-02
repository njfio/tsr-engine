//! Fixture-only process/restart lifecycle. No provider, shell action, or project execution.
use serde_json::{json, Value};
use std::{fs, path::Path, process::Command};

fn run(dir: &Path, command: &str, option: Option<(&str, &Value)>, succeeds: bool) -> Value {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_tsr"));
    cmd.args(["engine", "--directory"]).arg(dir).arg(command);
    if let Some((flag, value)) = option {
        let path = dir.join("fixture-input.json");
        fs::write(&path, serde_json::to_vec(value).unwrap()).unwrap();
        cmd.arg(flag).arg(path);
    }
    let out = cmd.output().unwrap();
    assert_eq!(
        out.status.success(),
        succeeds,
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    if succeeds {
        serde_json::from_slice(&out.stdout).unwrap()
    } else {
        Value::String(String::from_utf8_lossy(&out.stderr).into_owned())
    }
}
fn request(dir: &Path) -> Value {
    run(dir, "direction-request", None, true)
}
fn inspect(dir: &Path, state: &Value, second: bool) -> Value {
    let direction = |notes: Value| json!({"purpose":"Fixture inspection of admitted notes", "supports":[0],"anticipated":[1.0,0.0],"action":{"operation":"inspect","notes":notes}});
    run(
        dir,
        "deliberate",
        Some((
            "--response",
            &json!({"state_root":state["state_root"],"directions":[direction(if second {json!(["a","b"])} else {json!(["a"])}),direction(json!(["b"]))]}),
        )),
        true,
    )
}
fn relation(state: &Value, ancestor: Value) -> Value {
    let q = &state["relational_request"];
    json!({"state_root":state["state_root"],"return_commitment":q["return_commitment"],"observation_revision":q["observation_revision"],"evidence_key":q["evidence_key"],"proposals":[{"parents":q["active_originals"],"relationship":"Fixture hypothesis: evidence and explanation can expose gaps in one another","aspiration":"Fixture-only proposed relationship, not established semantic emergence","discriminator":"What does the next observed return leave unknown?","revises":ancestor,"catalysts":[]}]})
}
#[test]
fn relational_lifecycle_survives_distinct_cli_processes_without_inventing_return() {
    let temp = tempfile::tempdir().unwrap();
    let dir = temp.path();
    let initial = run(
        dir,
        "direction-init",
        Some((
            "--spec",
            &json!({"wants":[{"text":"Fixture: understand admitted evidence","aim":{"predicate":"understand","topic":"x"}},{"text":"Fixture: account for admitted evidence","aim":{"predicate":"account","topic":"x"}}],"notes":[{"id":"a","topic":"x","text":"alpha"},{"id":"b","topic":"x","text":"beta"}],"max_cycles":8,"attention_limit":2}),
        )),
        true,
    );
    assert_eq!(initial["relational_request"]["due"], false);
    inspect(dir, &initial, false);
    let observed = request(dir);
    assert_eq!(observed["relational_request"]["due"], true);
    let offer = relation(&observed, Value::Null);
    let admitted = run(dir, "direction-relate", Some(("--response", &offer)), true);
    assert_eq!(request(dir), admitted); // Fresh process, same durable evidence.
    let history = admitted["relational_request"]["history"].clone();
    let ancestor = admitted["relational_guidance"][0]["revision"].clone();
    let identity = admitted["relational_guidance"][0]["identity"].clone();
    assert_eq!(
        admitted["relational_guidance"][0]["semantic_status"],
        "unknown"
    );
    // An unknown claimed return is refused. It cannot create an observation or revision.
    let mut fabricated = relation(&admitted, ancestor.clone());
    fabricated["return_commitment"] =
        json!("0000000000000000000000000000000000000000000000000000000000000000");
    run(
        dir,
        "direction-relate",
        Some(("--response", &fabricated)),
        false,
    );
    assert_eq!(request(dir), admitted);
    inspect(dir, &admitted, true);
    let pending = request(dir);
    assert!(pending["relational_guidance"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(
        pending["relational_request"]["existing"][0]["feedback"],
        "needs_review"
    );
    assert_eq!(
        pending["relational_request"]["existing"][0]["discriminator_tested"],
        "unknown"
    );
    assert_eq!(pending["relational_request"]["history"], history);
    let mut bad = relation(&pending, ancestor.clone());
    bad["return_commitment"] = fabricated["return_commitment"].clone();
    let rejection = run(dir, "direction-relate", Some(("--response", &bad)), false);
    assert!(rejection
        .as_str()
        .unwrap()
        .contains("stale relational evidence"));
    assert_eq!(request(dir), pending);
    let revised = run(
        dir,
        "direction-relate",
        Some(("--response", &relation(&pending, ancestor))),
        true,
    );
    assert_eq!(revised["relational_request"]["history"][0], history[0]);
    assert_eq!(
        revised["relational_request"]["history"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let status = |s: &Value, w: bool| json!({"state_root":s["state_root"],"identity":identity,"withdrawn":w});
    assert_eq!(
        run(
            dir,
            "direction-relation-status",
            Some(("--update", &status(&revised, false))),
            true
        ),
        revised
    );
    let withdrawn = run(
        dir,
        "direction-relation-status",
        Some(("--update", &status(&revised, true))),
        true,
    );
    assert!(withdrawn["relational_guidance"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(
        run(
            dir,
            "direction-relation-status",
            Some(("--update", &status(&withdrawn, true))),
            true
        ),
        withdrawn
    );
    let restored = run(
        dir,
        "direction-relation-status",
        Some(("--update", &status(&withdrawn, false))),
        true,
    );
    assert_eq!(restored["relational_guidance"].as_array().unwrap().len(), 1);
    assert_eq!(
        restored["relational_request"]["history"],
        revised["relational_request"]["history"]
    );
    assert_eq!(
        run(
            dir,
            "direction-relation-status",
            Some(("--update", &status(&restored, false))),
            true
        ),
        restored
    );
    for state in [
        &observed, &admitted, &pending, &revised, &withdrawn, &restored,
    ] {
        for key in ["standing_desires", "obligation_ids", "authority"] {
            assert_eq!(state[key], initial[key], "{key}");
        }
    }
    assert_eq!(restored["cycles"], 2);
    assert_eq!(
        restored["relational_guidance"][0]["semantic_status"],
        "unknown"
    );
}
