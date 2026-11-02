//! Opt-in static web verification on the existing sealed project runner.
//! Model JavaScript is browser content, never a Node harness module.
use super::*;
use anyhow::Context;
use serde_json::Value;
/// Local decoded proposal memory envelope. Narrative fields have no separate
/// small text cap; immutable path/identity and permission checks remain distinct.
pub const RESPONSE_BYTES: usize = 131072;
pub fn narrative(text: &str) -> bool {
    !text.trim().is_empty() && text.len() <= RESPONSE_BYTES
}
pub const HARNESS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../scripts/engine-browser-harness.cjs"
));
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Probe {
    version: String,
    assets: Vec<String>,
    entry: String,
    actions: Vec<BrowserAction>,
    checks: Vec<Check>,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct BrowserAction {
    kind: String,
    value: Value,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Check {
    selector: String,
    text: String,
}
pub(super) fn validate(spec: &ProjectSpec) -> Result<()> {
    ensure!(
        !spec.cargo
            && spec.named_package.is_none()
            && !spec.full_workspace_oracle
            && spec.edit_windows.is_empty()
            && spec.supporting_spans.is_empty(),
        "browser profile cannot mix execution modes"
    );
    ensure!(
        spec.probe_entry == "browser-harness.cjs"
            && spec
                .protected
                .get(&spec.probe_entry)
                .is_some_and(|s| s == HARNESS),
        "exact protected browser harness required"
    );
    ensure!(spec.timeout_ms <= 180000, "browser deadline bound");
    let expected: BTreeSet<_> = spec.editable.keys().cloned().collect();
    for case in &spec.cases {
        ensure!(
            case.args.len() == 1 && case.expected_stdout == "browser-observation@v1",
            "browser case contract"
        );
        let p = &case.args[0];
        ensure!(
            p.ends_with(".json") && valid_path(p),
            "protected probe path"
        );
        let source = spec
            .protected
            .get(p)
            .context("protected browser probe required")?;
        ensure!(source.len() <= 8192, "browser probe size");
        let probe: Probe = serde_json::from_str(source)?;
        ensure!(
            probe.version == "browser-probe@v1"
                && probe.assets.iter().cloned().collect::<BTreeSet<_>>() == expected
                && probe.assets.len() == expected.len()
                && expected.contains(&probe.entry)
                && probe.entry.ends_with(".html"),
            "browser asset inventory"
        );
        ensure!(
            probe.actions.len() <= 12 && (1..=8).contains(&probe.checks.len()),
            "browser action/check bound"
        );
        for a in probe.actions {
            ensure!(
                match a.kind.as_str() {
                    "wait" => a.value.as_u64().is_some_and(|n| n <= 1000),
                    "click" | "key" => a
                        .value
                        .as_str()
                        .is_some_and(|s| !s.is_empty() && s.len() <= 128),
                    _ => false,
                },
                "unsupported browser action"
            );
        }
        for q in probe.checks {
            ensure!(
                !q.selector.is_empty() && q.selector.len() <= 128 && q.text.len() <= 128,
                "browser check bound"
            );
        }
    }
    ensure!(
        spec.protected.len()
            == 1 + spec
                .cases
                .iter()
                .map(|c| &c.args[0])
                .collect::<BTreeSet<_>>()
                .len(),
        "unadmitted browser protected assets"
    );
    Ok(())
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Report {
    version: String,
    passed: bool,
    files: BTreeMap<String, String>,
    observations: Vec<Value>,
    errors: Vec<String>,
    error_count: u64,
}
pub(crate) fn classify(run: &RunResult, expected: &BTreeSet<String>) -> Verdict {
    if run.exit == 124 {
        return Verdict::Timeout;
    }
    if !run.execution.as_ref().is_some_and(|d| {
        d.termination == "runtime_exit"
            && d.runtime_exit == Some(run.exit)
            && d.stdout_complete
            && d.stderr_complete
    }) || !matches!(run.exit, 0 | 1)
        || !run.stderr.is_empty()
        || run.stdout.len() > 4001
    {
        return Verdict::Unavailable;
    }
    let Ok(v) = serde_json::from_slice::<Report>(&run.stdout) else {
        return Verdict::Unavailable;
    };
    if v.version != "browser-observation@v1"
        || expected.is_empty()
        || v.files.keys().cloned().collect::<BTreeSet<_>>() != *expected
        || v.files
            .values()
            .any(|h| h.len() != 64 || !h.bytes().all(|c| c.is_ascii_hexdigit()))
        || v.observations.len() > 22
        || v.errors.len() > 8
        || v.errors.iter().any(|e| e.len() > 64)
        || v.errors.len() != v.error_count.min(8) as usize
    {
        return Verdict::Unavailable;
    }
    let mut checks = 0;
    let mut met = true;
    let mut renders = 0;
    for o in &v.observations {
        let Some(fields) = o.as_object() else {
            return Verdict::Unavailable;
        };
        let valid = match o["kind"].as_str() {
            Some("text") => {
                checks += 1;
                met &= o["met"] == true;
                fields.len() == 6
                    && o["selector"]
                        .as_str()
                        .is_some_and(|s| !s.is_empty() && s.len() <= 128)
                    && o["text"].as_str().is_some_and(|s| s.chars().count() <= 256)
                    && o["met"].is_boolean()
                    && o["abbreviated"].is_boolean()
                    && o["original_length"].as_u64().is_some()
            }
            Some("render") => {
                renders += 1;
                fields.len() == 2
                    && o["sha256"]
                        .as_str()
                        .is_some_and(|s| s.len() == 64 && s.bytes().all(|c| c.is_ascii_hexdigit()))
            }
            Some("click" | "key") => {
                fields.len() == 2
                    && o["value"]
                        .as_str()
                        .is_some_and(|s| !s.is_empty() && s.len() <= 128)
            }
            Some("wait") => fields.len() == 2 && o["value"].as_u64().is_some_and(|n| n <= 1000),
            _ => false,
        };
        if !valid {
            return Verdict::Unavailable;
        }
    }
    if !(1..=8).contains(&checks)
        || renders != 2
        || v.passed != (met && v.error_count == 0)
        || (run.exit == 0) != v.passed
    {
        return Verdict::Unavailable;
    }
    if v.passed {
        Verdict::Passed
    } else {
        Verdict::FailedTests
    }
}
impl Engine {
    pub(crate) fn browser_verdict(&self, run: &RunResult, spec: &ProjectSpec) -> Result<Verdict> {
        // Sealed receipt and before/after native snapshot bind this report. Its SHA-256
        // file hashes are browser evidence, not native BLAKE3 content identities.
        self.project_snapshot(spec)?;
        let expected = spec.editable.keys().cloned().collect();
        Ok(classify(run, &expected))
    }
}
#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use serde_json::json;
    pub(crate) fn spec() -> ProjectSpec {
        serde_json::from_value(json!({"goal":"Authored web wiring only","image":"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","user":"501:20","browser":true,"editable":{"index.html":"<button id='start'>Start</button>","app.js":"// authored fixture\n"},"protected":{"browser-harness.cjs":HARNESS,"probe.json":json!({"version":"browser-probe@v1","assets":["index.html","app.js"],"entry":"index.html","actions":[{"kind":"click","value":"#start"}],"checks":[{"selector":"#start","text":"Start"}]}).to_string()},"probe_entry":"browser-harness.cjs","cases":[{"id":"browser_smoke","args":["probe.json"],"expected_stdout":"browser-observation@v1"}],"timeout_ms":30000})).unwrap()
    }
    #[test]
    fn profile_rejects_harness_scope_and_mode_tampering() -> Result<()> {
        let s = spec();
        s.validate()?;
        assert!(!s.browser_container_only);
        assert!(serde_json::to_value(&s)?
            .get("browser_container_only")
            .is_none());
        let mut opt_in = s.clone();
        opt_in.browser_container_only = true;
        opt_in.validate()?;
        assert_ne!(serde_json::to_value(&s)?, serde_json::to_value(&opt_in)?);
        opt_in.browser = false;
        assert!(opt_in.validate().is_err());
        let mut bad = s.clone();
        bad.protected
            .get_mut("browser-harness.cjs")
            .unwrap()
            .push(' ');
        assert!(bad.validate().is_err());
        let mut bad = s.clone();
        bad.cargo = true;
        assert!(bad.validate().is_err());
        let mut bad = s.clone();
        bad.editable.insert("package.json".into(), "{}".into());
        assert!(bad.validate().is_err());
        let mut bad = s.clone();
        bad.cases[0].args[0] = "../probe.json".into();
        assert!(bad.validate().is_err());
        let mut bad = s.clone();
        bad.protected
            .insert("private.txt".into(), "not exportable".into());
        assert!(bad.validate().is_err());
        Ok(())
    }
    #[test]
    fn classify_is_not_exit_zero_or_console_truth() {
        let mut r = RunResult::exit_only(0);
        r.stdout = b"PASS".to_vec();
        assert_eq!(classify(&r, &BTreeSet::new()), Verdict::Unavailable);
        r.stdout=serde_json::to_vec(&json!({"version":"browser-observation@v1","passed":true,"files":{},"observations":[{"kind":"text","met":false}],"errors":[]})).unwrap();
        assert_eq!(classify(&r, &BTreeSet::new()), Verdict::Unavailable);
        r.exit = 124;
        assert_eq!(classify(&r, &BTreeSet::new()), Verdict::Timeout);
    }
    #[test]
    fn strict_completion_inventory_and_unknown_controls() {
        let expected = BTreeSet::from(["index.html".into()]);
        let value = json!({"version":"browser-observation@v1","passed":true,"files":{"index.html":"a".repeat(64)},"observations":[{"kind":"render","sha256":"b".repeat(64)},{"kind":"text","selector":"#start","text":"Play","met":true,"abbreviated":false,"original_length":4},{"kind":"render","sha256":"c".repeat(64)}],"errors":[],"error_count":0});
        let mut r = RunResult::from_output(0, &serde_json::to_vec(&value).unwrap(), b"", 1);
        assert_eq!(classify(&r, &expected), Verdict::Unavailable);
        r.execution = Some(tsr_corpus::ExecutionDiagnostics {
            version: 1,
            termination: "runtime_exit".into(),
            runtime_exit: Some(0),
            runtime_signal: None,
            stdout_complete: true,
            stderr_complete: true,
            worker: "authored fixture".into(),
        });
        assert_eq!(classify(&r, &expected), Verdict::Passed);
        assert_eq!(
            classify(&r, &BTreeSet::from(["other.js".into()])),
            Verdict::Unavailable
        );
        r.execution.as_mut().unwrap().stdout_complete = false;
        assert_eq!(classify(&r, &expected), Verdict::Unavailable);
        r.execution.as_mut().unwrap().stdout_complete = true;
        r.execution.as_mut().unwrap().termination = "wall_timeout".into();
        assert_eq!(classify(&r, &expected), Verdict::Unavailable);
        r.execution.as_mut().unwrap().termination = "runtime_exit".into();
        r.stdout.extend(b"garbage");
        assert_eq!(classify(&r, &expected), Verdict::Unavailable);
    }
}
