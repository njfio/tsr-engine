use super::typed::*;
use super::*;
fn setup() -> Result<(tempfile::TempDir, Engine, Admission)> {
    let (d, e, mut a) = super::tests::setup()?;
    a.version = "verifier-admission@v2".into();
    a.prerequisites = BTreeMap::from([
        ("subtotal".into(), Prerequisite::StaticSemantic),
        (
            "discount".into(),
            Prerequisite::ObservedTests {
                test_identity: e.project_verifier_test_identity()?,
            },
        ),
    ]);
    Ok((d, e, a))
}
fn report(a: &Admission) -> Report {
    Report {
        findings: a
            .coverage
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    Assessment {
                        finding: Finding::Unknown,
                        evidence: v.clone(),
                        explanation: "Synthetic typed fixture; no real execution".into(),
                        limitations: "No semantic competence established".into(),
                        basis: Some(Basis::Unknown),
                    },
                )
            })
            .collect(),
    }
}
fn receipt(e: &Engine, passed: bool, source: Hash, origin: Source) -> Result<EventId> {
    // Synthetic trusted-controller fixture only, no executable code or operational evidence.
    e.adaptive_record(
        &AdaptiveRecord::Coding {
            record: CodingRecord::Receipt {
                commitment: EventId(Hash::of(&[b"fixture-commitment"])),
                verdict: if passed {
                    Verdict::Passed
                } else {
                    Verdict::FailedTests
                },
                runs: vec![],
                source_after: Some(source),
                observations: vec![],
            },
        },
        origin,
    )
}
#[test]
fn typed_absent_receipt_cannot_be_success_or_static_downgrade() -> Result<()> {
    let (_d, e, a) = setup()?;
    let id = e.project_verifier_admit(a.clone())?;
    let (_, r) = e.project_verifier_request(id)?;
    let mut good = report(&a);
    let x = good.findings.get_mut("subtotal").unwrap();
    x.finding = Finding::NotMet;
    x.basis = Some(Basis::StaticSemantic);
    e.validate_verifier_report(&r, &a, &good)?;
    for basis in [
        Basis::Unknown,
        Basis::StaticSemantic,
        Basis::ObservedTests {
            observation: ObservedTests {
                receipt: EventId(Hash::of(&[b"invented"])),
                snapshot: r.snapshot,
                test_identity: e.project_verifier_test_identity()?,
                outcome: Outcome::Passed,
            },
        },
    ] {
        let mut bad = good.clone();
        let x = bad.findings.get_mut("discount").unwrap();
        x.finding = Finding::Met;
        x.basis = Some(basis);
        x.explanation = "All probes passed".into();
        assert!(e.validate_verifier_report(&r, &a, &bad).is_err());
    }
    Ok(())
}
#[test]
fn typed_exact_receipts_outcomes_and_mixed_semantics() -> Result<()> {
    for passed in [true, false] {
        let (_d, e, mut a) = setup()?;
        a.prerequisites.insert(
            "discount".into(),
            Prerequisite::Mixed {
                test_identity: e.project_verifier_test_identity()?,
            },
        );
        receipt(
            &e,
            passed,
            e.project_acceptance_status()?.snapshot,
            Source::Rules,
        )?;
        let id = e.project_verifier_admit(a.clone())?;
        let (_, r) = e.project_verifier_request(id)?;
        let observed = r.observed_tests.clone().unwrap();
        let mut good = report(&a);
        let x = good.findings.get_mut("discount").unwrap();
        x.finding = if passed {
            Finding::Met
        } else {
            Finding::NotMet
        };
        x.basis = Some(Basis::Mixed {
            observation: observed.clone(),
            static_finding: Finding::Met,
        });
        e.validate_verifier_report(&r, &a, &good)?;
        for mode in 0..5 {
            let mut bad = good.clone();
            let x = bad.findings.get_mut("discount").unwrap();
            let mut observation = observed.clone();
            match mode {
                0 => observation.receipt = EventId(Hash::of(&[b"wrong"])),
                1 => observation.snapshot = Hash::of(&[b"stale"]),
                2 => observation.test_identity = Hash::of(&[b"different-tests"]),
                3 => observation.outcome = Outcome::Unknown,
                _ => {
                    x.finding = Finding::Met;
                    if !passed {
                        observation.outcome = Outcome::Passed;
                    }
                }
            };
            x.basis = Some(Basis::Mixed {
                observation,
                static_finding: if mode == 4 {
                    Finding::Unknown
                } else {
                    Finding::Met
                },
            });
            assert!(e.validate_verifier_report(&r, &a, &bad).is_err());
        }
        // Passing probes do not override a static semantic counterexample.
        if passed {
            let x = good.findings.get_mut("discount").unwrap();
            x.finding = Finding::NotMet;
            x.basis = Some(Basis::Mixed {
                observation: observed,
                static_finding: Finding::NotMet,
            });
            e.validate_verifier_report(&r, &a, &good)?;
        }
    }
    Ok(())
}
#[test]
fn typed_stale_and_foreign_receipts_never_become_current() -> Result<()> {
    let (_d, e, a) = setup()?;
    receipt(&e, true, Hash::of(&[b"old-source"]), Source::Rules)?;
    let id = e.project_verifier_admit(a)?;
    let (_, r) = e.project_verifier_request(id)?;
    assert!(r.observed_tests.is_none());
    let (_d, e, a) = setup()?;
    receipt(
        &e,
        true,
        e.project_acceptance_status()?.snapshot,
        Source::Generator,
    )?;
    let id = e.project_verifier_admit(a)?;
    assert!(e.project_verifier_request(id).is_err());
    Ok(())
}
struct Fake {
    a: Admission,
    report: Report,
}
impl Verifier for Fake {
    fn identity(&self) -> (&str, Hash) {
        (&self.a.verifier_identity, self.a.artifact)
    }
    fn evaluate(&self, _: &Request, _: u64, _: u64) -> Result<Output> {
        Ok(Output {
            json: serde_json::to_string(&self.report)?,
            tokens: Some(30),
        })
    }
}
#[test]
fn typed_false_success_is_retained_rejected_and_not_replayed() -> Result<()> {
    let (_d, e, a) = setup()?;
    let id = e.project_verifier_admit(a.clone())?;
    let (request, _) = e.project_verifier_request(id)?;
    let mut bad = report(&a);
    bad.findings.get_mut("discount").unwrap().finding = Finding::Met;
    let fake = Fake { a, report: bad };
    let done = e.project_verifier_run(request, &fake)?;
    assert!(e
        .verifier_records()?
        .iter()
        .any(|(_, r)| matches!(r,Record::RejectedReport{request:id,..} if *id==request)));
    assert!(e.project_verifier_adopt(done).is_err());
    assert!(e.project_verifier_run(request, &fake).is_err());
    assert!(!e.project_acceptance_status()?.accepted);
    Ok(())
}
fn open(e: &Engine) -> Result<()> {
    e.project_lifecycle_open(super::super::super::lifecycle::Admission {
        training: None,
        state_root: e.store.head_root()?,
        tool_ms_ceiling: 1,
        reason: "advisory only synthetic fixture".into(),
        non_tool: Some(super::super::super::budget::Limits {
            model_calls: 1,
            model_tokens: 1000,
            wall_ms: 60000,
            patch_bytes: 0,
            external_cost_microusd: 0,
        }),
    })?;
    Ok(())
}
#[test]
fn advisory_closure_preserves_wants_budget_unknown_and_restart() -> Result<()> {
    let (d, e, a) = setup()?;
    open(&e)?;
    let id = e.project_verifier_admit(a.clone())?;
    let (request, _) = e.project_verifier_request(id)?;
    let before = serde_json::to_value(Ledger::new(&e.store).collective()?.entries)?;
    let done = e.project_verifier_run(
        request,
        &Fake {
            report: report(&a),
            a,
        },
    )?;
    let p = e.project_lifecycle_close_advisory("shadow finished with unknown semantics".into())?;
    assert!(p.closed);
    assert!(p.reviewed_accepted.is_none() && p.label.is_none() && p.review.is_none());
    assert_eq!(
        before,
        serde_json::to_value(Ledger::new(&e.store).collective()?.entries)?
    );
    assert!(!e.project_acceptance_status()?.accepted);
    assert!(e.project_verifier_adopt(done).is_err());
    assert!(e.project_verifier_request(id).is_err());
    assert!(e.project_budget()?.stopped);
    let e = Engine::open(d.path())?;
    assert_eq!(
        serde_json::to_value(&p)?,
        serde_json::to_value(e.project_lifecycle_close_advisory("idempotent".into())?)?
    );
    let eligibility = e.project_eligibility_view()?;
    assert_eq!(
        eligibility["eligibility"]["status"],
        "advisory_only_no_execution_label"
    );
    Ok(())
}
#[test]
fn advisory_closure_blocks_unfinished_before_reservation_and_unsettled_usage() -> Result<()> {
    let (_d, e, a) = setup()?;
    open(&e)?;
    let id = e.project_verifier_admit(a.clone())?;
    let (request, _) = e.project_verifier_request(id)?;
    e.verifier_append(
        Record::Started {
            request,
            identity: a.verifier_identity,
            artifact: a.artifact,
        },
        Source::Rules,
    )?;
    assert!(e
        .project_lifecycle_close_advisory("unfinished".into())
        .is_err());
    e.project_verifier_local_reconcile(request, "synthetic abandoned before send".into())?;
    use super::super::super::budget;
    e.project_resource_reserve(budget::Reserve {
        request_id: "unknown-call".into(),
        state_root: e.store.head_root()?,
        envelope: "fixture".into(),
        model: "fixture".into(),
        local: true,
        tokens_upper: 900,
        external_cost_upper: 0,
    })?;
    assert!(e
        .project_lifecycle_close_advisory("unsettled".into())
        .is_err());
    e.project_resource_settle(budget::Settle {
        request_id: "unknown-call".into(),
        prompt_tokens: None,
        output_tokens: None,
        duration_ns: None,
        external_cost_microusd: Some(0),
        outcome: "unknown".into(),
    })?;
    e.project_lifecycle_close_advisory("settled unknown retained".into())?;
    assert_eq!(e.project_budget()?.tokens_charged, 900);
    assert_eq!(e.project_budget()?.unknown_usage, 1);
    Ok(())
}

#[test]
fn typed_suite_identity_changes_with_actual_execution_selection() -> Result<()> {
    let (_d, e, _) = setup()?;
    let spec = e.project_configuration()?;
    let base = suite_identity(&spec)?;
    assert_eq!(base, suite_identity(&spec.clone())?);
    for mode in 0..5 {
        let mut changed = spec.clone();
        match mode {
            0 => changed.named_package = Some("another-package".into()),
            1 => changed.full_workspace_oracle = !changed.full_workspace_oracle,
            2 => changed.image = "sha256:different-runtime".into(),
            3 => changed.cases[0].expected_stdout = "different".into(),
            _ => changed
                .protected
                .insert("new_test.rs".into(), "additional test".into())
                .map_or((), |_| ()),
        };
        assert_ne!(base, suite_identity(&changed)?);
    }
    Ok(())
}
