//! `tsr hook` — the recorder (PRD §B4, path A).
//!
//! Claude Code calls this once per hook event with one JSON payload on stdin. M0 **records and
//! never blocks**: every path exits 0 and no `deny`/`block` decision is ever emitted. The gate
//! belongs to `tsr-policy` (M1); containment belongs to `tsr exec`.
//!
//! Tool-name → [`ActionKind`] (B4 line 218):
//! - `Read`, `Grep`, `Glob`, and `rts` calls → `Inspect`;
//! - `Bash` whose command matches the allowlisted test command → `RunApprovedTest`;
//! - `Edit`, `Write` → `ProposePatch`;
//! - the model's own turn (a `SessionStart`/prompt, or any other tool) → `Generate`;
//! - the `Stop` stage → `Stop`.
//!
//! Gated actions (`RunApprovedTest`, `ProposePatch`, `RequestApproval`) are recorded as
//! `Proposed` → `Forecast` → `Committed` in one batch, so the forecast is on record at the state
//! version the commitment occupies (B5, forecast-gated execution). A sandboxed test run then
//! records `Dispatched { in_sandbox: true }` and `Executed`/`Unresolved`.

use crate::store_path;
use serde_json::Value;
use tsr_domain::{
    ActionKind, Candidate, CandidateId, Decision, Event, EventId, EventKind, Forecast, Forecaster,
    Hash, Horizon, OutcomeDefId, Probability, Source, TaskId, Utility,
};
use tsr_store::Store;

/// The allowlisted test command prefix (B5: `cargo test --offline -p parser`). A `Bash` call whose
/// command starts with this is a `RunApprovedTest`; anything else is `Generate` (the model's turn).
pub const ALLOWLISTED_TEST_PREFIX: &str = "cargo test";

/// The common decision event (B7): every candidate forecasts `task_accepted_in_budget@v1`.
pub const DECISION_EVENT: &str = "task_accepted_in_budget@v1";

/// A parsed Claude Code hook payload. Every field is optional: unknown fields are ignored and
/// missing fields are tolerated, so a payload from another Claude Code version still records.
#[derive(Debug, Default, Clone)]
pub struct HookPayload {
    pub session_id: Option<String>,
    pub transcript_path: Option<String>,
    pub cwd: Option<String>,
    pub hook_event_name: Option<String>,
    pub tool_name: Option<String>,
    pub tool_input: Option<Value>,
    pub tool_response: Option<Value>,
    pub prompt: Option<String>,
    pub stop_hook_active: Option<bool>,
}

impl HookPayload {
    /// Parse a payload defensively. A non-object or malformed JSON yields an empty payload rather
    /// than an error: the hook records what it can and never blocks.
    pub fn parse(raw: &str) -> Self {
        let value: Value = match serde_json::from_str(raw) {
            Ok(v) => v,
            Err(_) => return Self::default(),
        };
        let s = |k: &str| value.get(k).and_then(|v| v.as_str()).map(str::to_string);
        Self {
            session_id: s("session_id"),
            transcript_path: s("transcript_path"),
            cwd: s("cwd"),
            hook_event_name: s("hook_event_name"),
            tool_name: s("tool_name"),
            tool_input: value.get("tool_input").cloned(),
            tool_response: value.get("tool_response").cloned(),
            prompt: s("prompt"),
            stop_hook_active: value.get("stop_hook_active").and_then(|v| v.as_bool()),
        }
    }

    /// The session id, or a stable fallback so a payload without one still records.
    pub fn session(&self) -> String {
        self.session_id
            .clone()
            .unwrap_or_else(|| "unknown-session".to_string())
    }

    /// The task this payload belongs to.
    pub fn task(&self) -> TaskId {
        store_path::task_for_session(&self.session())
    }

    /// The `Bash` command, if this is a Bash call.
    pub fn bash_command(&self) -> Option<String> {
        if self.tool_name.as_deref() != Some("Bash") {
            return None;
        }
        self.tool_input
            .as_ref()
            .and_then(|v| v.get("command"))
            .and_then(|v| v.as_str())
            .map(str::to_string)
    }
}

/// Map a tool name (and its input) to the typed action of B4.
pub fn action_for(tool_name: Option<&str>, bash_command: Option<&str>) -> ActionKind {
    match tool_name {
        Some("Read") | Some("Grep") | Some("Glob") | Some("rts") => ActionKind::Inspect,
        Some("Edit") | Some("Write") => ActionKind::ProposePatch,
        Some("Bash") => match bash_command {
            Some(cmd) if cmd.trim_start().starts_with(ALLOWLISTED_TEST_PREFIX) => {
                ActionKind::RunApprovedTest
            }
            _ => ActionKind::Generate,
        },
        _ => ActionKind::Generate,
    }
}

/// Whether an action is forecast-gated (B5).
pub fn is_gated(kind: ActionKind) -> bool {
    matches!(
        kind,
        ActionKind::RunApprovedTest | ActionKind::ProposePatch | ActionKind::RequestApproval
    )
}

/// The source of an event, by action: the model's own turn is `Generator`, the rest are `Rules`
/// (the hook is the deterministic recorder, not a generator).
fn source_for(kind: ActionKind) -> Source {
    match kind {
        ActionKind::Generate => Source::Generator,
        _ => Source::Rules,
    }
}

/// A digest of a tool's arguments, so a candidate's identity commits to what it was called with.
fn args_digest(payload: &HookPayload) -> Hash {
    let mut parts: Vec<Vec<u8>> = Vec::new();
    if let Some(t) = &payload.tool_name {
        parts.push(t.as_bytes().to_vec());
    }
    if let Some(i) = &payload.tool_input {
        parts.push(serde_json::to_vec(i).unwrap_or_default());
    }
    if let Some(p) = &payload.prompt {
        parts.push(p.as_bytes().to_vec());
    }
    let refs: Vec<&[u8]> = parts.iter().map(|p| p.as_slice()).collect();
    Hash::of(&refs)
}

thread_local! {
    /// A pinned timestamp for the self-test harness. `recorded_at` is hashed into every event id
    /// (`Event::new`), so a fixture can only be byte-reproducible if the clock is pinned. The
    /// production path leaves this unset and uses the real wall clock.
    static PINNED_CLOCK: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// Run `f` with the clock pinned to `stamp`, so every event it records gets a deterministic id.
/// Used by the self-test harness; not used on the production path.
pub fn with_pinned_clock<T>(stamp: &str, f: impl FnOnce() -> T) -> T {
    PINNED_CLOCK.with(|c| *c.borrow_mut() = Some(stamp.to_string()));
    let out = f();
    PINNED_CLOCK.with(|c| *c.borrow_mut() = None);
    out
}

/// The current wall-clock time as an RFC 3339 string (provenance only; ordering is seq).
///
/// If the clock is pinned (see [`with_pinned_clock`]) the pinned stamp is returned instead, which
/// makes the self-test fixture reproducible.
pub fn now_rfc3339() -> String {
    if let Some(stamp) = PINNED_CLOCK.with(|c| c.borrow().clone()) {
        return stamp;
    }
    // No chrono dependency: seconds since the epoch, formatted as a fixed-width RFC 3339 UTC stamp.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let (y, mo, d) = civil_from_days(days as i64);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

/// Days since 1970-01-01 → (year, month, day). Howard Hinnant's civil_from_days.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// The last event of a task, if any: the parent of the next event.
fn last_event(store: &Store, task: &TaskId) -> anyhow::Result<Option<Event>> {
    Ok(store.by_task(task)?.into_iter().next_back().map(|(_, e)| e))
}

/// The number of events in a task so far (its current state version).
fn event_count(store: &Store, task: &TaskId) -> anyhow::Result<u64> {
    Ok(store.latest_seq(task)?.unwrap_or(0) as u64)
}

/// Append one event, chaining it to the task's last event.
fn append_chained(
    store: &Store,
    task: &TaskId,
    source: Source,
    kind: EventKind,
) -> anyhow::Result<EventId> {
    let parents = last_event(store, task)?
        .map(|e| vec![e.id])
        .unwrap_or_default();
    let ev = Event::new(*task, parents, source, kind, now_rfc3339());
    store.append(&ev)?;
    Ok(ev.id)
}

/// Record a `SessionStart`: the task is opened (B4: SessionStart opens the task).
pub fn record_session_start(store: &Store, payload: &HookPayload) -> anyhow::Result<EventId> {
    let task = payload.task();
    append_chained(store, &task, Source::Rules, EventKind::TaskOpened { task })
}

/// Record a `Stop`: the task stops with a typed reason (B5). The hook never blocks.
pub fn record_stop(store: &Store, payload: &HookPayload) -> anyhow::Result<EventId> {
    let task = payload.task();
    let reason = if payload.stop_hook_active.unwrap_or(false) {
        "operator"
    } else {
        "done"
    };
    append_chained(
        store,
        &task,
        Source::Rules,
        EventKind::Stop {
            reason: reason.to_string(),
        },
    )
}

/// Record a tool call. Returns the events appended, in order.
///
/// A gated action is recorded as `Proposed` → `Forecast` → `Committed` in one batch: the forecast
/// is pinned to the state version the commitment occupies, which is what the M0 invariant checks.
/// A `RunApprovedTest` additionally records `Dispatched { in_sandbox: true }` and then `Executed`
/// (from the `PostToolUse` response) or `Unresolved`.
pub fn record_tool(store: &Store, payload: &HookPayload) -> anyhow::Result<Vec<EventId>> {
    let task = payload.task();
    let kind = action_for(
        payload.tool_name.as_deref(),
        payload.bash_command().as_deref(),
    );
    let source = source_for(kind);
    let args = args_digest(payload);
    let parents: Vec<EventId> = last_event(store, &task)?
        .map(|e| vec![e.id])
        .unwrap_or_default();

    let candidate = Candidate::new(kind, source, args, parents.clone(), vec![])?;
    let cid = candidate.id;

    let mut batch: Vec<Event> = Vec::new();
    let mut push = |kind: EventKind, source: Source, parents: Vec<EventId>| {
        let ev = Event::new(task, parents, source, kind, now_rfc3339());
        let id = ev.id;
        batch.push(ev);
        id
    };

    // 1. The candidate is proposed. The model's own turn is typed `Generate` (B4 line 218); every
    //    other action is a `Proposed` candidate.
    let proposed_id = if kind == ActionKind::Generate {
        push(
            EventKind::Generate { candidate: cid },
            source,
            parents.clone(),
        )
    } else {
        push(
            EventKind::Proposed {
                candidate: candidate.clone(),
            },
            source,
            parents.clone(),
        )
    };

    // 2. A gated action gets a forecast at the state version its commitment will occupy.
    //    The commitment is the next event after the forecast, so its 0-based index is
    //    (events before the batch) + (index of the forecast in the batch) + 1.
    let mut forecast_id = None;
    if is_gated(kind) {
        let before = event_count(store, &task)?;
        // The forecast is the second event of the batch (index 1), so the commitment that follows
        // it is at index 2 within the batch.
        let commit_version = before + 2;
        let forecast = Forecast::new(
            OutcomeDefId(Hash::of(&[DECISION_EVENT.as_bytes()])),
            Horizon::ThisDispatch,
            Forecaster::Generator,
            Probability::new(0.5)?,
            commit_version,
        );
        forecast_id = Some(push(
            EventKind::Forecast {
                forecast,
                candidate: cid,
            },
            source,
            vec![proposed_id],
        ));
    }

    // 3. The commitment. The decision is a deterministic selection (propensity 1.0).
    let commit_parents = forecast_id.map(|f| vec![f]).unwrap_or(vec![proposed_id]);
    let committed_id = push(
        EventKind::Committed {
            candidate: cid,
            decision: Decision::Selected {
                value: cid,
                utility: Utility {
                    p_success: 0.5,
                    value: 0.0,
                    cost: 0.0,
                    ev: 0.0,
                },
                propensity: 1.0,
                evidence: vec![proposed_id],
            },
        },
        source,
        commit_parents,
    );

    // 4. A test run is dispatched into the sealed sandbox. The reconciliation (`Executed` or
    //    `Unresolved`) is recorded by the matching `PostToolUse`, which carries the exit code.
    if kind == ActionKind::RunApprovedTest {
        push(
            EventKind::Dispatched {
                candidate: cid,
                in_sandbox: true,
            },
            Source::Rules,
            vec![committed_id],
        );
    }

    let seqs = store.append_all(&batch)?;
    debug_assert_eq!(seqs.len(), batch.len());
    Ok(batch.iter().map(|e| e.id).collect())
}

/// The exit code and log digest of a `PostToolUse` response, if the payload carries them.
fn exec_outcome(payload: &HookPayload) -> (Option<i32>, Option<Hash>) {
    let resp = match &payload.tool_response {
        Some(r) => r,
        None => return (None, None),
    };
    let exit_code = resp
        .get("exit_code")
        .or_else(|| resp.get("exitCode"))
        .and_then(|v| v.as_i64())
        .map(|c| c as i32);
    let log_digest = resp
        .get("stdout")
        .and_then(|v| v.as_str())
        .map(|s| Hash::of(&[s.as_bytes()]));
    (exit_code, log_digest)
}

/// Record a `PostToolUse`: reconcile the pending dispatch of the task, if any.
///
/// A `PostToolUse` for a `Bash` test run carries the exit code of the run the `PreToolUse` already
/// dispatched. It records `Executed` (or `Unresolved`) for that same candidate — it does not mint a
/// second candidate, so one run is one dispatch and one reconciliation.
pub fn record_post_tool(store: &Store, payload: &HookPayload) -> anyhow::Result<Vec<EventId>> {
    let task = payload.task();
    let pending = pending_dispatch(store, &task)?;
    let cid = match pending {
        Some(cid) => cid,
        // No pending dispatch: fall back to recording the call as a fresh tool event.
        None => return record_tool(store, payload),
    };
    let parents = last_event(store, &task)?
        .map(|e| vec![e.id])
        .unwrap_or_default();
    let (exit_code, log_digest) = exec_outcome(payload);
    let kind = match exit_code {
        Some(code) => EventKind::Executed {
            candidate: cid,
            exit_code: Some(code),
            log_digest,
            duration_ms: 0,
        },
        None => EventKind::Unresolved {
            candidate: cid,
            reason: "no exit code in the hook response".to_string(),
        },
    };
    let ev = Event::new(task, parents, Source::Rules, kind, now_rfc3339());
    let id = ev.id;
    store.append(&ev)?;
    Ok(vec![id])
}

/// The candidate of the newest `Dispatched` event that has no `Executed`/`Unresolved` yet.
fn pending_dispatch(store: &Store, task: &TaskId) -> anyhow::Result<Option<CandidateId>> {
    use std::collections::HashSet;
    let events = store.by_task(task)?;
    let mut settled: HashSet<CandidateId> = HashSet::new();
    let mut pending: Option<CandidateId> = None;
    for (_, e) in &events {
        match &e.kind {
            EventKind::Dispatched { candidate, .. } => pending = Some(*candidate),
            EventKind::Executed { candidate, .. } | EventKind::Unresolved { candidate, .. } => {
                settled.insert(*candidate);
            }
            _ => {}
        }
    }
    Ok(pending.filter(|c| !settled.contains(c)))
}

/// The `tsr hook` entry point. Reads one payload from `input`, records it, and returns the events
/// appended. Never returns an error for a malformed payload: M0 records and never blocks.
pub fn run(store: &Store, stage: &str, input: &str) -> anyhow::Result<Vec<EventId>> {
    let payload = HookPayload::parse(input);
    match stage {
        "SessionStart" => Ok(vec![record_session_start(store, &payload)?]),
        "Stop" => Ok(vec![record_stop(store, &payload)?]),
        "PreToolUse" => record_tool(store, &payload),
        "PostToolUse" => record_post_tool(store, &payload),
        // An unknown stage is recorded as the model's own turn rather than dropped.
        _ => record_tool(store, &payload),
    }
}

/// The candidate id a payload would produce, without recording it (used by tests).
pub fn candidate_for(payload: &HookPayload) -> anyhow::Result<CandidateId> {
    let kind = action_for(
        payload.tool_name.as_deref(),
        payload.bash_command().as_deref(),
    );
    let source = source_for(kind);
    Ok(Candidate::new(kind, source, args_digest(payload), vec![], vec![])?.id)
}

/// Record a `Forecast` for the current state version (B5, B7).
///
/// The state version is the number of events in the task *including* the forecast being recorded —
/// i.e. the version the next commitment will occupy. That is what makes a forecast recorded before
/// a gated action line up with the commitment's own state version, which is the M0 invariant.
pub fn record_forecast(
    store: &Store,
    task: &TaskId,
    event: &str,
    p: f64,
    source: &str,
) -> anyhow::Result<EventId> {
    let forecaster = match source {
        "controller" => Forecaster::Controller,
        "operator" => Forecaster::Operator,
        _ => Forecaster::Generator,
    };
    let before = event_count(store, task)?;
    let state_version = before + 1;
    let forecast = Forecast::new(
        OutcomeDefId(Hash::of(&[event.as_bytes()])),
        Horizon::ThisDispatch,
        forecaster,
        Probability::new(p)?,
        state_version,
    );
    let parents = last_event(store, task)?
        .map(|e| vec![e.id])
        .unwrap_or_default();
    let ev = Event::new(
        *task,
        parents,
        Source::Rules,
        EventKind::Forecast {
            forecast,
            candidate: CandidateId(Hash::of(&[event.as_bytes()])),
        },
        now_rfc3339(),
    );
    let id = ev.id;
    store.append(&ev)?;
    Ok(id)
}

/// The M0 self-test harness: drive realistic Claude Code hook payloads through the hook and write
/// `fixtures/m0/session.events.jsonl`.
///
/// The sequence is the one the milestone names: a task opened, two inspections, a forecast, a
/// sandboxed test run, a patch proposal, a forecast, a second run, a stop. It runs against a fresh
/// store at `fixtures/m0/session.db` so it is reproducible and never touches the operator's store.
pub fn selftest() -> anyhow::Result<std::path::PathBuf> {
    // `recorded_at` is hashed into every event id, so the fixture is only reproducible if the
    // clock is pinned. The production hook path is untouched and keeps real wall-clock time.
    with_pinned_clock(SELFTEST_RECORDED_AT, selftest_inner)
}

/// The fixed provenance stamp the self-test fixture is recorded under. Any change to this value
/// changes every event id in `fixtures/m0/session.events.jsonl`.
pub const SELFTEST_RECORDED_AT: &str = "2026-01-01T00:00:00Z";

fn selftest_inner() -> anyhow::Result<std::path::PathBuf> {
    let fixture_dir = std::path::Path::new("fixtures/m0");
    std::fs::create_dir_all(fixture_dir)?;
    let db = fixture_dir.join("session.db");
    let _ = std::fs::remove_file(&db);
    let store = crate::store_path::open_store_at(&db)?;

    let session = "m0-selftest-session";
    let task = crate::store_path::task_for_session(session);

    // 1. The task is opened.
    let start = HookPayload {
        session_id: Some(session.to_string()),
        hook_event_name: Some("SessionStart".to_string()),
        cwd: Some(
            std::env::current_dir()
                .map(|p| p.display().to_string())
                .unwrap_or_default(),
        ),
        ..Default::default()
    };
    record_session_start(&store, &start)?;

    // 2. Two inspections.
    for path in ["crates/tsr-cli/src/lib.rs", "crates/tsr-store/src/lib.rs"] {
        let p = HookPayload {
            session_id: Some(session.to_string()),
            hook_event_name: Some("PreToolUse".to_string()),
            tool_name: Some("Read".to_string()),
            tool_input: Some(serde_json::json!({ "file_path": path })),
            ..Default::default()
        };
        record_tool(&store, &p)?;
    }

    // 3. A forecast before the first gated action.
    record_forecast(&store, &task, DECISION_EVENT, 0.7, "generator")?;

    // 4. A sandboxed test run (PreToolUse proposes+commits+dispatches; PostToolUse reconciles).
    let test_cmd = "cargo test --offline -p parser";
    let pre = HookPayload {
        session_id: Some(session.to_string()),
        hook_event_name: Some("PreToolUse".to_string()),
        tool_name: Some("Bash".to_string()),
        tool_input: Some(serde_json::json!({ "command": test_cmd })),
        ..Default::default()
    };
    record_tool(&store, &pre)?;
    let post = HookPayload {
        session_id: Some(session.to_string()),
        hook_event_name: Some("PostToolUse".to_string()),
        tool_name: Some("Bash".to_string()),
        tool_input: Some(serde_json::json!({ "command": test_cmd })),
        tool_response: Some(serde_json::json!({ "exit_code": 0, "stdout": "test result: ok" })),
        ..Default::default()
    };
    record_post_tool(&store, &post)?;

    // 5. A patch proposal (gated).
    let edit = HookPayload {
        session_id: Some(session.to_string()),
        hook_event_name: Some("PreToolUse".to_string()),
        tool_name: Some("Edit".to_string()),
        tool_input: Some(serde_json::json!({ "file_path": "crates/tsr-cli/src/lib.rs" })),
        ..Default::default()
    };
    record_tool(&store, &edit)?;

    // 6. A forecast before the second gated action.
    record_forecast(&store, &task, DECISION_EVENT, 0.6, "generator")?;

    // 7. A second run.
    let test_cmd2 = "cargo test --offline -p parser";
    let pre2 = HookPayload {
        session_id: Some(session.to_string()),
        hook_event_name: Some("PreToolUse".to_string()),
        tool_name: Some("Bash".to_string()),
        tool_input: Some(serde_json::json!({ "command": test_cmd2 })),
        ..Default::default()
    };
    record_tool(&store, &pre2)?;
    let post2 = HookPayload {
        session_id: Some(session.to_string()),
        hook_event_name: Some("PostToolUse".to_string()),
        tool_name: Some("Bash".to_string()),
        tool_input: Some(serde_json::json!({ "command": test_cmd2 })),
        tool_response: Some(serde_json::json!({ "exit_code": 0, "stdout": "test result: ok" })),
        ..Default::default()
    };
    record_post_tool(&store, &post2)?;

    // 8. A stop.
    let stop = HookPayload {
        session_id: Some(session.to_string()),
        hook_event_name: Some("Stop".to_string()),
        stop_hook_active: Some(false),
        ..Default::default()
    };
    record_stop(&store, &stop)?;

    // Write the trace as events.jsonl, in sequence order.
    let jsonl = crate::task::export(&store, &task)?;
    let out = fixture_dir.join("session.events.jsonl");
    std::fs::write(&out, jsonl)?;
    Ok(out)
}
