use std::process::Command;
#[test]
fn cli_three_cycles_survive_independent_processes() {
    let dir = tempfile::tempdir().unwrap();
    let run = |args: &[&str]| {
        let out = Command::new(env!("CARGO_BIN_EXE_tsr"))
            .args(["engine", "--directory"])
            .arg(dir.path())
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        String::from_utf8(out.stdout).unwrap()
    };
    run(&["init"]);
    let first = run(&["step"]);
    assert!(first.contains("Belief committed Bulk; Return 1/3 delivered"));
    let later = run(&["step", "--cycles", "2"]);
    assert!(later.contains("Cycle 2:"));
    assert!(later.contains("Cycle 3:"));
    assert_eq!(
        later
            .matches("Belief committed Checked; Return 3/3 delivered")
            .count(),
        2
    );
    let saved: serde_json::Value = serde_json::from_str(&run(&["show"])).unwrap();
    assert_eq!(saved["cycles"], 3);
    assert_eq!(saved["preferences"].as_array().unwrap().len(), 2);
}

#[test]
fn project_entry_point_preserves_scope_and_requires_explicit_run_mode() {
    let dir = tempfile::tempdir().unwrap();
    let run = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_tsr"))
            .args(["engine", "--directory"])
            .arg(dir.path())
            .args(["project"])
            .args(args)
            .output()
            .unwrap()
    };
    let spec = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/engine-cargo/scope.json"
    );
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/engine-cargo");
    assert!(run(&["init", "--spec", spec, "--source", source])
        .status
        .success());
    let status: serde_json::Value = serde_json::from_slice(&run(&["status"]).stdout).unwrap();
    assert_eq!(status["status"], "ready_for_proposal");
    assert!(status["protected_files_intact"].as_bool().unwrap());
    assert!(!run(&["run"]).status.success());
    assert!(!run(&["run", "--", "--directory", "/tmp/other"])
        .status
        .success());
    assert!(run(&["resume"]).status.success());
    assert!(run(&["export"]).status.success());
    assert!(run(&["config"]).status.success());
    let requirement_file = dir.path().join("acceptance.json");
    std::fs::write(
        &requirement_file,
        r#"{"behavior":"Preserve all required behavior"}"#,
    )
    .unwrap();
    let registered = run(&[
        "acceptance",
        "requirements",
        "--file",
        requirement_file.to_str().unwrap(),
    ]);
    assert!(registered.status.success());
    let current: serde_json::Value = serde_json::from_slice(&registered.stdout).unwrap();
    assert_eq!(current["status"], "review_required");
    assert_eq!(current["accepted"], false);
    let review = serde_json::json!({"snapshot":current["snapshot"],
        "requirements_revision":current["requirements_revision"],"test_receipt":current["test_receipt"],
        "decision":"accept","evidence":{"behavior":{"met":true,"evidence":"operator review"}}});
    std::fs::write(&requirement_file, serde_json::to_vec(&review).unwrap()).unwrap();
    assert!(!run(&[
        "acceptance",
        "review",
        "--file",
        requirement_file.to_str().unwrap()
    ])
    .status
    .success());
    let status: serde_json::Value =
        serde_json::from_slice(&run(&["acceptance", "status"]).stdout).unwrap();
    assert_eq!(status["desire_status"], "open");
    std::fs::write(&requirement_file, "x".repeat(16385)).unwrap();
    assert!(!run(&[
        "acceptance",
        "requirements",
        "--file",
        requirement_file.to_str().unwrap()
    ])
    .status
    .success());
}

#[test]
fn operator_drafts_and_episode_initialization_need_no_internal_state_edits() {
    let root = tempfile::tempdir().unwrap();
    let first = root.path().join("first");
    let next = root.path().join("next");
    let run = |directory: &std::path::Path, args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_tsr"))
            .args(["engine", "--directory"])
            .arg(directory)
            .arg("project")
            .args(args)
            .output()
            .unwrap()
    };
    let spec = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../examples/engine-cargo/scope.json"
    );
    let source = concat!(env!("CARGO_MANIFEST_DIR"), "/../../examples/engine-cargo");
    let requirements = root.path().join("requirements.json");
    std::fs::write(
        &requirements,
        r#"{"arithmetic":"Private operator requirement"}"#,
    )
    .unwrap();
    assert!(run(
        &first,
        &[
            "init",
            "--spec",
            spec,
            "--source",
            source,
            "--requirements",
            requirements.to_str().unwrap()
        ]
    )
    .status
    .success());
    let inspected: serde_json::Value =
        serde_json::from_slice(&run(&first, &["acceptance", "inspect"]).stdout).unwrap();
    assert_eq!(
        inspected["requirements"]["arithmetic"],
        "Private operator requirement"
    );
    for kind in ["review", "feedback"] {
        let draft = root.path().join(format!("{kind}.json"));
        let args = [
            "acceptance",
            "draft",
            kind,
            "--out",
            draft.to_str().unwrap(),
        ];
        assert!(run(&first, &args).status.success());
        let bytes = std::fs::read(&draft).unwrap();
        assert!(!String::from_utf8_lossy(&bytes).contains("Private operator requirement"));
        assert!(!run(&first, &args).status.success());
        assert_eq!(bytes, std::fs::read(&draft).unwrap());
        assert!(!run(
            &first,
            &["acceptance", kind, "--file", draft.to_str().unwrap()]
        )
        .status
        .success());
    }
    assert!(!run(
        &first,
        &[
            "acceptance",
            "draft",
            "review",
            "--out",
            first.join("coding-work/hidden.json").to_str().unwrap()
        ]
    )
    .status
    .success());
    let request = root.path().join("request.json");
    assert!(
        run(&first, &["request", "--out", request.to_str().unwrap()])
            .status
            .success()
    );
    assert!(!std::fs::read_to_string(&request)
        .unwrap()
        .contains("Private operator requirement"));
    assert!(!run(&first, &["export", "--accepted-only"]).status.success());
    let initialized = run(
        &next,
        &[
            "init",
            "--spec",
            spec,
            "--source",
            source,
            "--carry-from",
            first.to_str().unwrap(),
        ],
    );
    assert!(
        initialized.status.success(),
        "{}",
        String::from_utf8_lossy(&initialized.stderr)
    );
    let next_view: serde_json::Value =
        serde_json::from_slice(&run(&next, &["acceptance", "inspect"]).stdout).unwrap();
    assert_eq!(next_view["requirements"], inspected["requirements"]);
    assert_eq!(next_view["completion"]["accepted"], false);
    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&request).unwrap()).unwrap();
    let file = value["coding"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"] == "src/pricing.rs")
        .unwrap();
    let response = serde_json::json!({"protocol":value["protocol"],"state_root":value["state_root"],"attention_binding":value["next_attention"]["binding"],"proposals":[{"kind":"exact_edit","patch":{"path":file["path"],"source_digest":file["source_digest"],"inspection_handle":file["inspection_handle"],"old_text":"unit_cents + quantity","new_text":"unit_cents * quantity"},"rationale":"Bounded CLI commitment regression"}]});
    let proposal = root.path().join("proposal.json");
    std::fs::write(&proposal, serde_json::to_vec(&response).unwrap()).unwrap();
    let committed = run(
        &first,
        &[
            "run",
            "--response",
            proposal.to_str().unwrap(),
            "--commit-only",
        ],
    );
    assert!(
        committed.status.success(),
        "{}",
        String::from_utf8_lossy(&committed.stderr)
    );
    let status: serde_json::Value = serde_json::from_slice(&committed.stdout).unwrap();
    assert_eq!(status["status"], "committed_not_executed");
    assert_eq!(status["operator_next"], "resume");
}
