use super::super::super::budget;
use super::local::*;
use super::*;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::time::{Duration, Instant};
fn fixture(
    endpoint: String,
) -> Result<(tempfile::TempDir, Engine, EventId, Request, local::Config)> {
    let (d, e, mut a) = super::tests::setup()?;
    e.project_lifecycle_open(super::super::super::lifecycle::Admission {
        training: None,
        state_root: e.store.head_root()?,
        tool_ms_ceiling: 1,
        reason: "local verifier transport fixture".into(),
        non_tool: Some(budget::Limits {
            model_calls: 2,
            model_tokens: 20000,
            wall_ms: 60000,
            patch_bytes: 0,
            external_cost_microusd: 0,
        }),
    })?;
    a.version = "verifier-admission@v2".into();
    a.prerequisites = a
        .coverage
        .keys()
        .map(|k| (k.clone(), typed::Prerequisite::StaticSemantic))
        .collect();
    a.tokens_per_request = 10000;
    a.timeout_ms = 2000;
    let c = local::Config {
        version: "local-verifier@v1".into(),
        endpoint,
        model: "stub-local".into(),
        digest: a.artifact,
        generator_model_digest: Hash::of(&[b"different-generator"]),
        context_tokens: 8192,
        output_tokens: 512,
    };
    let admission = e.project_verifier_admit(a)?;
    e.project_verifier_local_configure(admission, c.clone())?;
    let (id, r) = e.project_verifier_request(admission)?;
    Ok((d, e, id, r, c))
}
fn response(r: &Request) -> serde_json::Value {
    let findings=r.requirements.keys().map(|k|(k.clone(),serde_json::json!({"finding":"unknown","basis":{"kind":"static_semantic"},"evidence":if k=="subtotal"{vec!["src/pricing.rs","tests/probe.rs","test_receipt"]}else{vec!["src/discount.rs","tests/probe.rs","test_receipt"]},"explanation":"Independent stub has no semantic authority","limitations":"Test fixture, not model evidence"}))).collect::<BTreeMap<_,_>>();
    serde_json::json!({"model":"stub-local","done":true,"done_reason":"stop","prompt_eval_count":100,"eval_count":50,"total_duration":1000000,"message":{"role":"assistant","content":serde_json::json!({"findings":findings}).to_string()}})
}
// Synthetic loopback stub only. Never contacts an installed provider or executes project code.
fn server(
    listener: TcpListener,
    c: local::Config,
    reply: Vec<u8>,
    count: usize,
    delay: bool,
) -> std::thread::JoinHandle<Vec<serde_json::Value>> {
    server_with_chat_sync(listener, c, reply, count, delay, None)
}
fn server_with_chat_sync(
    listener: TcpListener,
    c: local::Config,
    reply: Vec<u8>,
    count: usize,
    delay: bool,
    chat_sync: Option<(std::sync::mpsc::Sender<()>, std::sync::mpsc::Receiver<()>)>,
) -> std::thread::JoinHandle<Vec<serde_json::Value>> {
    std::thread::spawn(move || {
        listener.set_nonblocking(true).unwrap();
        let deadline = Instant::now() + Duration::from_secs(6);
        let mut bodies = vec![];
        for _ in 0..count {
            let mut s = loop {
                match listener.accept() {
                    Ok((s, _)) => break s,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        if Instant::now() > deadline {
                            return bodies;
                        }
                        std::thread::sleep(Duration::from_millis(5));
                    }
                    Err(e) => panic!("{e}"),
                }
            };
            s.set_nonblocking(false).unwrap();
            s.set_read_timeout(Some(Duration::from_secs(2))).unwrap();
            let mut bytes = vec![];
            let mut buf = [0; 4096];
            let (end, length) = loop {
                let n = s.read(&mut buf).unwrap();
                assert!(n > 0);
                bytes.extend_from_slice(&buf[..n]);
                if let Some(end) = bytes.windows(4).position(|w| w == b"\r\n\r\n") {
                    let h = String::from_utf8_lossy(&bytes[..end]);
                    let len = h
                        .lines()
                        .find_map(|l| l.strip_prefix("Content-Length: "))
                        .unwrap()
                        .parse::<usize>()
                        .unwrap();
                    break (end + 4, len);
                }
            };
            while bytes.len() < end + length {
                let n = s.read(&mut buf).unwrap();
                assert!(n > 0);
                bytes.extend_from_slice(&buf[..n]);
            }
            let chat = bytes.starts_with(b"POST");
            let content = if chat {
                bodies.push(serde_json::from_slice(&bytes[end..]).unwrap());
                if let Some((ready, release)) = &chat_sync {
                    ready.send(()).unwrap();
                    release.recv_timeout(Duration::from_secs(3)).unwrap();
                } else if delay {
                    std::thread::sleep(Duration::from_millis(2200));
                }
                reply.clone()
            } else {
                serde_json::json!({"models":[{"name":c.model,"digest":c.digest.to_string(),"size":1,"details":{"format":"gguf"}}]}).to_string().into_bytes()
            };
            let header = format!(
                "HTTP/1.0 200 OK\r\nContent-Length: {}\r\n\r\n",
                content.len()
            );
            let _ = s.write_all(header.as_bytes());
            let _ = s.write_all(&content);
        }
        bodies
    })
}
#[test]
fn local_transport_isolated_metered_quarantined_and_not_replayable() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let (_d, e, id, r, c) = fixture(format!("http://{}", listener.local_addr()?))?;
    let worker = server(listener, c, response(&r).to_string().into_bytes(), 3, false);
    let finished = e.project_verifier_local_run(id)?;
    let requests = worker.join().unwrap();
    assert_eq!(requests.len(), 1);
    let req = &requests[0];
    assert_eq!(req["messages"].as_array().unwrap().len(), 2);
    let context: serde_json::Value =
        serde_json::from_str(req["messages"][1]["content"].as_str().unwrap())?;
    assert_eq!(
        context
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>(),
        BTreeSet::from([
            "coverage".into(),
            "evidence".into(),
            "requirements".into(),
            "prerequisites".into(),
            "observed_tests".into()
        ])
    );
    assert!(
        e.verifier_records()?.iter().any(|(k, x)| *k == finished
            && matches!(
                x,
                Record::Finished {
                    report: Some(_),
                    ..
                }
            )),
        "quarantined stub result: {:?}",
        e.verifier_records()?
    );
    assert!(!e.project_acceptance_status()?.accepted);
    let b = e.project_budget()?;
    assert_eq!(b.model_calls, 1);
    assert_eq!(b.tokens_charged, 150);
    assert_eq!(b.measured_local_duration_ns, 1000000);
    assert!(e.project_verifier_local_run(id).is_err());
    assert_eq!(
        e.project_verifier_local_reconcile(id, "already recorded".into())?,
        finished
    );
    Ok(())
}
#[test]
fn local_transport_rejects_bad_usage_malformed_truncated_and_injected_reports() -> Result<()> {
    for mode in 0..8 {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let (_d, e, id, r, c) = fixture(format!("http://{}", listener.local_addr()?))?;
        let mut value = response(&r);
        match mode {
            0 => {
                value.as_object_mut().unwrap().remove("eval_count");
            }
            1 => value["eval_count"] = serde_json::json!(-1),
            2 => value["done_reason"] = serde_json::json!("length"),
            3 => value["message"]["content"] = serde_json::json!("{"),
            4 => {
                let mut report: serde_json::Value =
                    serde_json::from_str(value["message"]["content"].as_str().unwrap())?;
                report["findings"]["subtotal"]["evidence"] = serde_json::json!(["invented-proof"]);
                value["message"]["content"] = serde_json::json!(report.to_string());
            }
            5 => value["message"]["tool_calls"] = serde_json::json!([{"name":"approve"}]),
            _ => value["total_duration"] = serde_json::Value::Null,
        }
        let worker = server(
            listener,
            c,
            if mode == 7 {
                b"{truncated".to_vec()
            } else {
                value.to_string().into_bytes()
            },
            if mode == 3 || mode == 4 { 3 } else { 2 },
            false,
        );
        let finished = e.project_verifier_local_run(id)?;
        worker.join().unwrap();
        assert!(
            e.verifier_records()?
                .iter()
                .any(|(k, x)| *k == finished && matches!(x, Record::Finished { report: None, .. })),
            "mode {mode}"
        );
        assert!(e.project_verifier_adopt(finished).is_err());
        assert_eq!(e.project_budget()?.model_calls, 1);
        assert_eq!(e.project_budget()?.unsettled_requests, 0);
        if mode < 2 {
            assert_eq!(e.project_budget()?.tokens_charged, 8704);
        }
    }
    Ok(())
}
#[test]
fn local_timeout_and_precancel_preserve_reservations() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let (_d, e, id, r, c) = fixture(format!("http://{}", listener.local_addr()?))?;
    let worker = server(listener, c, response(&r).to_string().into_bytes(), 2, true);
    let done = e.project_verifier_local_run(id)?;
    worker.join().unwrap();
    assert!(e.project_verifier_adopt(done).is_err());
    assert_eq!(e.project_budget()?.tokens_charged, 8704);
    assert_eq!(e.project_budget()?.unknown_usage, 1);
    let (_d, e, id, _, _) = fixture("http://127.0.0.1:1".into())?;
    e.project_verifier_local_cancel(id)?;
    e.project_verifier_local_run(id)?;
    assert_eq!(e.project_budget()?.model_calls, 0);
    Ok(())
}
#[test]
fn local_reconcile_before_after_send_and_recorded_response_restart() -> Result<()> {
    for stage in 0..3 {
        let (d, e, id, r, c) = fixture("http://127.0.0.1:1".into())?;
        e.verifier_append(
            Record::Started {
                request: id,
                identity: "isolated-static-verifier".into(),
                artifact: c.digest,
            },
            Source::Rules,
        )?;
        let resource = format!("verifier-local-{id}");
        let envelope = Hash::of(&[b"stub-envelope"]);
        e.project_resource_reserve(budget::Reserve {
            request_id: resource.clone(),
            state_root: e.store.head_root()?,
            envelope: envelope.to_string(),
            model: c.digest.to_string(),
            local: true,
            tokens_upper: 8704,
            external_cost_upper: 0,
        })?;
        if stage > 0 {
            e.verifier_append(
                Record::LocalProgress {
                    request: id,
                    progress: Progress::DispatchIntent {
                        resource: resource.clone(),
                        envelope,
                    },
                },
                Source::Rules,
            )?;
        }
        if stage > 1 {
            e.verifier_append(
                Record::LocalProgress {
                    request: id,
                    progress: Progress::Received {
                        response: Hash::of(&[b"response"]),
                        usage: budget::Settle {
                            request_id: resource,
                            prompt_tokens: Some(100),
                            output_tokens: Some(50),
                            duration_ns: Some(1000000),
                            external_cost_microusd: Some(0),
                            outcome: "measured".into(),
                        },
                    },
                },
                Source::Rules,
            )?;
        }
        drop(e);
        let e = Engine::open(d.path())?;
        let done =
            e.project_verifier_local_reconcile(id, "interrupted process confirmed absent".into())?;
        assert_eq!(
            e.project_budget()?.tokens_charged,
            if stage == 2 { 150 } else { 8704 }
        );
        assert_eq!(e.project_budget()?.unsettled_requests, 0);
        assert!(e.project_verifier_local_run(id).is_err());
        assert!(e.project_verifier_adopt(done).is_err());
        assert!(e.verifier_fresh(&r).is_ok());
    }
    Ok(())
}
#[test]
fn local_rejects_stale_source_endpoint_identity_and_changed_configuration() -> Result<()> {
    let (d, e, id, r, mut c) = fixture("http://127.0.0.1:1".into())?;
    c.endpoint = "https://external.invalid:443".into();
    assert!(e.project_verifier_local_configure(r.admission, c).is_err());
    std::fs::write(d.path().join("coding-work/src/pricing.rs"), "changed")?;
    assert!(e.project_verifier_local_run(id).is_err());
    assert_eq!(e.project_budget()?.model_calls, 0);
    Ok(())
}
#[test]
fn local_cancel_after_response_is_atomically_quarantined() -> Result<()> {
    let (_d, e, id, r, c) = fixture("http://127.0.0.1:1".into())?;
    struct Cancels<'a> {
        e: &'a Engine,
        id: EventId,
        c: local::Config,
        body: String,
    }
    impl Verifier for Cancels<'_> {
        fn identity(&self) -> (&str, Hash) {
            ("isolated-static-verifier", self.c.digest)
        }
        fn evaluate(&self, _: &Request, _: u64, _: u64) -> Result<Output> {
            self.e.project_verifier_local_cancel(self.id)?;
            Ok(Output {
                json: self.body.clone(),
                tokens: Some(150),
            })
        }
    }
    let body = response(&r)["message"]["content"]
        .as_str()
        .unwrap()
        .to_string();
    let done = e.project_verifier_run(id, &Cancels { e: &e, id, c, body })?;
    assert!(e
        .verifier_records()?
        .iter()
        .any(|(k, r)| *k == done && matches!(r, Record::Finished { report: None, .. })));
    assert!(e.project_verifier_adopt(done).is_err());
    Ok(())
}
#[test]
fn local_live_cancel_and_lock_exclusion() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    let (d, e, id, r, c) = fixture(format!("http://{}", listener.local_addr()?))?;
    let (ready_tx, ready_rx) = std::sync::mpsc::channel();
    let (release_tx, release_rx) = std::sync::mpsc::channel();
    let worker = server_with_chat_sync(
        listener,
        c,
        response(&r).to_string().into_bytes(),
        2,
        true,
        Some((ready_tx, release_rx)),
    );
    let path = d.path().to_path_buf();
    let cancel = std::thread::spawn(move || -> Result<()> {
        let e = Engine::open(&path)?;
        // DispatchIntent precedes HTTP transmission; cancel only after the complete
        // request is observed, while the stub holds the response. Pre-send cancellation
        // remains separately tested and must not weaken the shared EOF assertions.
        ready_rx.recv_timeout(Duration::from_secs(3))?;
        assert!(e.verifier_records()?.iter().any(|(_,r)|matches!(r,Record::LocalProgress{request,progress:Progress::DispatchIntent{..}} if *request==id)));
        assert!(e
            .project_verifier_local_reconcile(id, "must not interrupt active lock".into())
            .is_err());
        e.project_verifier_local_cancel(id)?;
        Ok(())
    });
    let done = e.project_verifier_local_run(id)?;
    cancel.join().unwrap()?;
    release_tx.send(())?;
    assert_eq!(worker.join().unwrap().len(), 1);
    assert!(e.project_verifier_adopt(done).is_err());
    assert_eq!(e.project_budget()?.tokens_charged, 8704);
    assert!(e.verifier_records()?.iter().any(|(_, r)| matches!(
        r,
        Record::LocalProgress {
            progress: Progress::Terminal {
                cancellation_requested: true,
                provider_completion_known: false,
                ..
            },
            ..
        }
    )));
    Ok(())
}
#[test]
fn local_exhausted_budget_prevents_dispatch_without_refund() -> Result<()> {
    let (_d, e, id, _, c) = fixture("http://127.0.0.1:1".into())?;
    e.project_resource_reserve(budget::Reserve {
        request_id: "prior-local".into(),
        state_root: e.store.head_root()?,
        envelope: Hash::of(&[b"prior"]).to_string(),
        model: c.digest.to_string(),
        local: true,
        tokens_upper: 20000,
        external_cost_upper: 0,
    })?;
    e.project_resource_settle(budget::Settle {
        request_id: "prior-local".into(),
        prompt_tokens: Some(19000),
        output_tokens: Some(1000),
        duration_ns: Some(1),
        external_cost_microusd: Some(0),
        outcome: "measured".into(),
    })?;
    let done = e.project_verifier_local_run(id)?;
    assert!(e.project_verifier_adopt(done).is_err());
    assert_eq!(e.project_budget()?.model_calls, 1);
    assert_eq!(e.project_budget()?.tokens_charged, 20000);
    assert!(!e.verifier_records()?.iter().any(|(_, r)| matches!(
        r,
        Record::LocalProgress {
            progress: Progress::DispatchIntent { .. },
            ..
        }
    )));
    Ok(())
}
