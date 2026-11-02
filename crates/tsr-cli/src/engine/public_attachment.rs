//! Root-bound advisory context into the existing native proposal/selection path.
//! No selector, scores, criteria or execution authority are introduced.
use super::*;
use anyhow::Context;
use serde_json::{json, Value};
const SOURCE: &str = "public-meaning-attachment@v1";
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Attach {
    pub target_root: Hash,
    pub source_directory: PathBuf,
    pub source_root: Hash,
    pub context_digest: Hash,
    pub reviewer: String,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Detach {
    pub target_root: Hash,
    pub attachment: EventId,
    pub reviewer: String,
    pub reason: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum Entry {
    Attach { input: Attach },
    Detach { input: Detach },
}
/// Owns a source write reservation until the target request/preview/selection ends.
/// Dropping the SQLite connection rolls the read-only transaction back.
pub(crate) struct Guard {
    _source: Engine,
    pub context: Value,
}
fn text(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 512
}
fn digest(v: &Value) -> Result<Hash> {
    Ok(Hash::of(&[&serde_json::to_vec(v)?]))
}
impl Engine {
    pub fn public_meaning_context(&self) -> Result<Value> {
        let q = self.qualitative_request_v2()?;
        ensure!(
            q["phase"] == "propose"
                && q["pending_reviews"]
                    .as_array()
                    .is_some_and(|x| x.is_empty()),
            "meaning is pending review or stopped"
        );
        let parents = q["parents"].as_array().context("parents")?;
        ensure!(
            parents.iter().all(|p| p["active"] == true),
            "withdrawn meaning cannot attach"
        );
        ensure!(
            parents.iter().any(|p| !p["revision"].is_null())
                || q["guidance"].as_array().is_some_and(|x| !x.is_empty())
                || q["possibility_memory"]
                    .as_array()
                    .is_some_and(|x| !x.is_empty()),
            "reviewed meaning required"
        );
        let mut context = json!({"basis":q["basis"],"parents":q["parents"],"guidance":q["guidance"],"source_parents":q["source_parents"],"constraints":q["constraints"],"observed_returns":[],"role":"advisory meaning only; no acceptance, action permission, empirical outcome or utility weight"});
        if q["possibility_memory"]
            .as_array()
            .is_some_and(|x| !x.is_empty())
        {
            context["possibility_memory"] = q["possibility_memory"].clone();
        }
        if !q["retrieval_observations"].is_null() {
            context["retrieval_observations"] = q["retrieval_observations"].clone();
        }
        for key in [
            "reviewed_information_questions",
            "retrieval_questions",
            "information_expectation",
        ] {
            if !q[key].is_null() {
                context[key] = q[key].clone();
            }
        }
        Ok(context)
    }
    pub fn public_meaning_snapshot(&self) -> Result<Value> {
        let context = self.public_meaning_context()?;
        Ok(
            json!({"source_root":self.store.head_root()?,"context_digest":digest(&context)?,"context":context}),
        )
    }
    fn attachment_latest(&self) -> Result<Option<(EventId, Attach)>> {
        let mut latest = None;
        for (_, e) in self.events()? {
            if let EventKind::Observed {
                observation: Observation::Evidence { source, digest },
            } = &e.kind
            {
                if source == SOURCE {
                    ensure!(e.source == Source::Operator, "attachment provenance");
                    let b = fs::read(self.directory.join("cas").join(digest.to_hex()))?;
                    ensure!(Hash::of(&[&b]) == *digest, "attachment CAS changed");
                    match serde_json::from_slice::<Entry>(&b)? {
                        Entry::Attach { input } => latest = Some((e.id, input)),
                        Entry::Detach { input } => {
                            ensure!(
                                latest.as_ref().map(|x| x.0) == Some(input.attachment),
                                "wrong detach identity"
                            );
                            latest = None;
                        }
                    }
                }
            }
        }
        Ok(latest)
    }
    fn attachment_record(&self, e: Entry) -> Result<EventId> {
        let digest = self.blob(&serde_json::to_vec(&e)?)?;
        self.append(
            EventKind::Observed {
                observation: Observation::Evidence {
                    digest,
                    source: SOURCE.into(),
                },
            },
            Source::Operator,
        )
    }
    fn source_guard(&self, a: &Attach) -> Result<Guard> {
        ensure!(
            a.source_directory.is_absolute() && a.source_directory.join("events.sqlite").is_file(),
            "existing absolute native source required"
        );
        let path = fs::canonicalize(&a.source_directory)?;
        ensure!(
            path == a.source_directory && path != fs::canonicalize(&self.directory)?,
            "source/target isolation"
        );
        let source = Engine::open(&path)?;
        source.store.connection().execute_batch("BEGIN IMMEDIATE")?;
        ensure!(
            source.store.head_root()? == a.source_root,
            "stale source root; detach or explicitly re-review attachment"
        );
        let context = source.public_meaning_context()?;
        ensure!(
            digest(&context)? == a.context_digest,
            "reviewed context mismatch"
        );
        Ok(Guard {
            _source: source,
            context: json!({"source_root":a.source_root,"context_digest":a.context_digest,"public":context}),
        })
    }
    pub(crate) fn public_attachment_guard(&self) -> Result<Option<Guard>> {
        self.attachment_latest()?
            .map(|(_, a)| self.source_guard(&a))
            .transpose()
    }
    pub fn attach_public_meaning(&self, a: Attach) -> Result<EventId> {
        self.write(|| {
            ensure!(
                self.store.head_root()? == a.target_root,
                "stale target root"
            );
            ensure!(
                text(&a.reviewer) && text(&a.reason),
                "bounded attachment review"
            );
            self.adaptive_state()?; // Existing configured native choice context, not a qualitative-only store.
            let _guard = self.source_guard(&a)?;
            self.attachment_record(Entry::Attach { input: a })
        })
    }
    pub fn detach_public_meaning(&self, d: Detach) -> Result<EventId> {
        self.write(|| {
            ensure!(
                self.store.head_root()? == d.target_root && text(&d.reviewer) && text(&d.reason),
                "stale/unattributed detach"
            );
            ensure!(
                self.attachment_latest()?.as_ref().map(|x| x.0) == Some(d.attachment),
                "wrong attachment"
            );
            self.attachment_record(Entry::Detach { input: d })
        })
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::engine::{
        adaptive::{
            provider::{Response, Suggestion, PROTOCOL},
            Action, Spec,
        },
        qualitative as q,
    };
    pub(crate) fn source() -> Result<(tempfile::TempDir, Engine)> {
        let d = tempfile::tempdir()?;
        let e = Engine::open(d.path())?;
        e.qualitative_init(q::Bootstrap {
            parents: vec![
                q::Parent {
                    label: "A".into(),
                    meaning: "A useful beginning".into(),
                    attribution: "authored fixture".into(),
                },
                q::Parent {
                    label: "B".into(),
                    meaning: "Preserved disagreement".into(),
                    attribution: "authored fixture".into(),
                },
            ],
            records: vec![],
            constraints: vec!["No fulfillment claims".into()],
            acceptance_references: vec![],
        })?;
        Ok((d, e))
    }
    pub(crate) fn admit(e: &Engine) -> Result<()> {
        let r = e.qualitative_request()?;
        let p=e.qualitative_submit(serde_json::from_value(json!({"basis":r["basis"],"generator":"fixture-generator","response":{"protocol":PROTOCOL,"state_root":r["state_root"],"attention_binding":r["attention_binding"],"proposals":[{"kind":"relational","binding":r["attention_binding"],"proposal":{"parents":r["parents"].as_array().unwrap().iter().map(|x|x["id"].clone()).collect::<Vec<_>>(),"relationship":"Authored integration relationship","aspiration":"Authored integration meaning, not emergence","discriminator":"Unknown","revises":null,"catalysts":[]}}]}}))?)?;
        e.qualitative_review(q::Review {
            state_root: e.store.head_root()?,
            proposal: p,
            reviewer: "independent-fixture".into(),
            decision: q::Decision::Admit,
            reason: "PRIVATE_REVIEW_NOTE".into(),
        })?;
        Ok(())
    }
    fn target(goal: &str) -> Result<(tempfile::TempDir, Engine)> {
        let d = tempfile::tempdir()?;
        let e = Engine::open(d.path())?;
        e.configure(Spec {
            goal: goal.into(),
            criteria: vec!["readable".into()],
            actions: vec![Action {
                id: "draft".into(),
                description: "Write a draft".into(),
                serves: vec!["readable".into()],
                cost: 0.05,
            }],
            prior: vec![],
            initial_commitment: None,
        })?;
        Ok((d, e))
    }
    pub(crate) fn attachment(source: &Engine, target: &Engine) -> Result<Attach> {
        let snap = source.public_meaning_snapshot()?;
        Ok(Attach {
            target_root: target.store.head_root()?,
            source_directory: fs::canonicalize(&source.directory)?,
            source_root: serde_json::from_value(snap["source_root"].clone())?,
            context_digest: serde_json::from_value(snap["context_digest"].clone())?,
            reviewer: "independent-link-review".into(),
            reason: "Authored fixture connection".into(),
        })
    }
    fn response(e: &Engine, action: &str) -> Result<Response> {
        let r = e.proposal_request()?;
        Ok(Response {
            protocol: PROTOCOL.into(),
            state_root: r.state_root,
            attention_binding: r
                .next_attention
                .as_ref()
                .and_then(|a| serde_json::from_value(a["binding"].clone()).ok()),
            proposals: vec![Suggestion::Existing {
                action: action.into(),
                rationale: "Authored fixture candidate; not model evidence".into(),
            }],
            pursuit_expectations: Default::default(),
            common_event_forecasts: Default::default(),
        })
    }
    #[test]
    fn reviewed_possibility_reaches_generic_request_without_new_desire() -> Result<()> {
        let (_sd, s) = source()?;
        let q = s.qualitative_request_v2()?;
        let parents = q["parents"].clone();
        let sub = q::Submission {
            basis: serde_json::from_value(q["basis"].clone())?,
            generator: "authored-fixture".into(),
            response: Response {
                protocol: PROTOCOL.into(),
                state_root: s.store.head_root()?,
                attention_binding: serde_json::from_value(q["attention_binding"].clone())?,
                proposals: vec![Suggestion::NewCandidate {
                    description: "An untested route with alternatives".into(),
                    serves: vec![q["parents"][0]["id"].as_str().unwrap().into()],
                    rationale: "Authored wiring; usefulness unknown".into(),
                }],
                pursuit_expectations: Default::default(),
                common_event_forecasts: Default::default(),
            },
        };
        let p = s.qualitative_submit_possibility(sub)?;
        s.qualitative_review(q::Review {
            state_root: s.store.head_root()?,
            proposal: p,
            reviewer: "independent-fixture".into(),
            decision: q::Decision::Admit,
            reason: "Retain possibility, not wanting".into(),
        })?;
        assert_eq!(s.qualitative_request_v2()?["parents"], parents);
        let (_td, t) = target("Readable report")?;
        let before = t.proposal_request()?;
        t.attach_public_meaning(attachment(&s, &t)?)?;
        let r = t.proposal_request()?;
        assert_eq!(
            r.public_meaning.as_ref().unwrap()["public"]["possibility_memory"]
                .as_array()
                .unwrap()
                .len(),
            1
        );
        assert_eq!(r.admitted_criteria, before.admitted_criteria);
        assert_eq!(
            serde_json::to_value(&r.actions)?,
            serde_json::to_value(before.actions)?
        );
        assert!(
            t.evaluate_proposal(response(&t, "draft")?, false)?
                .assessments[0]
                .disposition
                == "eligible"
        );
        assert!(t
            .events()?
            .iter()
            .all(|(_, e)| !matches!(e.kind, EventKind::Executed { .. })));
        Ok(())
    }
    #[test]
    fn normal_request_preview_selection_and_restart_share_attachment() -> Result<()> {
        let (_sd, s) = source()?;
        admit(&s)?;
        let (td, t) = target("Readable report")?;
        let before = t.proposal_request()?;
        let a = attachment(&s, &t)?;
        let event = t.attach_public_meaning(a.clone())?;
        let r = t.proposal_request()?;
        assert!(r.public_meaning.is_some());
        assert_eq!(r.admitted_criteria, before.admitted_criteria);
        assert_eq!(
            serde_json::to_value(&r.actions)?,
            serde_json::to_value(&before.actions)?
        );
        let text = serde_json::to_string(&r)?;
        assert!(!text.contains("PRIVATE_REVIEW_NOTE"));
        assert!(!text.contains(s.directory.to_str().unwrap()));
        assert_eq!(
            serde_json::to_value(&r)?,
            serde_json::to_value(t.proposal_request()?)?
        );
        let restarted = Engine::open(td.path())?;
        assert_eq!(
            serde_json::to_value(&r)?,
            serde_json::to_value(restarted.proposal_request()?)?
        );
        assert!(t.attach_public_meaning(a).is_err());
        assert_eq!(t.store.head_root()?, r.state_root);
        let p = response(&t, "draft")?;
        let preview = t.evaluate_proposal(p.clone(), false)?;
        assert!(!preview.committed);
        assert_eq!(preview.assessments[0].disposition, "eligible");
        assert_eq!(t.store.head_root()?, r.state_root);
        let selected = t.evaluate_proposal(p, true)?;
        assert!(selected.committed);
        assert!(selected.state.pending.is_some());
        assert!(t
            .events()?
            .iter()
            .all(|(_, e)| !matches!(e.kind, EventKind::Executed { .. })));
        assert!(t.attachment_latest()?.is_some_and(|x| x.0 == event));
        Ok(())
    }
    #[test]
    fn cross_root_unreviewed_withdrawn_and_changed_source_fail_closed() -> Result<()> {
        let (_sd, s) = source()?;
        let (_td, t) = target("First")?;
        assert!(s.public_meaning_snapshot().is_err());
        admit(&s)?;
        let mut a = attachment(&s, &t)?;
        a.context_digest = Hash::of(&[b"wrong"]);
        assert!(t.attach_public_meaning(a).is_err());
        let a = attachment(&s, &t)?;
        let (_other, other) = target("Different target")?;
        assert!(other.attach_public_meaning(a.clone()).is_err());
        let id = t.attach_public_meaning(a)?;
        let before = t.proposal_request()?;
        let response = response(&t, "draft")?;
        let sq = s.qualitative_request()?;
        s.qualitative_status(q::Status {
            state_root: s.store.head_root()?,
            identity: serde_json::from_value(sq["parents"][0]["id"].clone())?,
            revision: None,
            withdrawn: true,
        })?;
        assert!(t.proposal_request().is_err());
        assert!(t.evaluate_proposal(response, false).is_err());
        assert_eq!(t.store.head_root()?, before.state_root);
        assert!(s.public_meaning_snapshot().is_err());
        t.detach_public_meaning(Detach {
            target_root: t.store.head_root()?,
            attachment: id,
            reviewer: "operator-fixture".into(),
            reason: "Source withdrawn; continue original task only".into(),
        })?;
        let after = t.proposal_request()?;
        assert!(after.public_meaning.is_none());
        assert_eq!(after.admitted_criteria, before.admitted_criteria);
        Ok(())
    }
    #[test]
    fn qualitative_text_does_not_configure_action_or_supply_scores() -> Result<()> {
        let (_sd, s) = source()?;
        admit(&s)?;
        let (_td, t) = target("Task")?;
        let before = t.adaptive_state()?;
        t.attach_public_meaning(attachment(&s, &t)?)?;
        let r = response(&t, "invented-qualitative-action")?;
        let preview = t.evaluate_proposal(r, false)?;
        assert_eq!(preview.assessments[0].disposition, "deferred");
        assert!(!preview.committed);
        assert_eq!(
            serde_json::to_value(before.options)?,
            serde_json::to_value(t.adaptive_state()?.options)?
        );
        Ok(())
    }
    #[test]
    fn source_locked_until_existing_selection_boundary_returns() -> Result<()> {
        let (_sd, s) = source()?;
        admit(&s)?;
        let (_td, t) = target("Task")?;
        t.attach_public_meaning(attachment(&s, &t)?)?;
        s.store
            .connection()
            .busy_timeout(std::time::Duration::ZERO)?;
        let guard = t.public_attachment_guard()?;
        assert!(s
            .store
            .connection()
            .execute_batch("BEGIN IMMEDIATE")
            .is_err());
        drop(guard);
        s.store
            .connection()
            .execute_batch("BEGIN IMMEDIATE; ROLLBACK")?;
        Ok(())
    }
    #[test]
    fn attached_relation_and_return_trace_separates_context_from_selection() -> Result<()> {
        // Authored control-flow fixture, not executed work or model behavior.
        for met in [false, true] {
            let mut chosen = vec![];
            for attached in [false, true] {
                let (_sd, s) = source()?;
                admit(&s)?;
                let original = s.qualitative_request_v2()?;
                let td = tempfile::tempdir()?;
                let t = Engine::open(td.path())?;
                t.configure(Spec {
                    goal: "Authored two-obligation fixture".into(),
                    criteria: vec!["one".into(), "two".into()],
                    actions: ["alpha", "beta"]
                        .into_iter()
                        .map(|id| Action {
                            id: id.into(),
                            description: "Authored alternative; semantic efficacy unproved".into(),
                            serves: vec!["one".into(), "two".into()],
                            cost: 0.05,
                        })
                        .collect(),
                    prior: vec![],
                    initial_commitment: Some("alpha".into()),
                })?;
                if attached {
                    t.attach_public_meaning(attachment(&s, &t)?)?;
                }
                let offer = |e: &Engine| -> Result<Response> {
                    let mut r = response(e, "alpha")?;
                    r.proposals.push(Suggestion::Existing {
                        action: "beta".into(),
                        rationale: "Same authored alternative in every arm".into(),
                    });
                    Ok(r)
                };
                let request = t.proposal_request()?;
                assert_eq!(request.public_meaning.is_some(), attached);
                if attached {
                    assert_eq!(
                        request.public_meaning.as_ref().unwrap()["public"]["guidance"],
                        original["guidance"]
                    );
                }
                let first = t
                    .evaluate_proposal(offer(&t)?, true)?
                    .state
                    .pending
                    .unwrap();
                assert_eq!(first.action, "alpha");
                let frozen = first.forecast.clone();
                let events = t.events()?;
                let fi=events.iter().position(|(_,e)| matches!(&e.kind,EventKind::Forecast{forecast,..} if forecast.id==frozen.id)).unwrap();
                let ci = events
                    .iter()
                    .position(|(_, event)| event.id == first.commitment)
                    .unwrap();
                assert!(fi < ci);
                // No executor is invoked: explicitly injected observer input tests wiring only.
                t.observe(
                    first.commitment,
                    adaptive::ResultInput {
                        met: if met {
                            vec!["one".into(), "two".into()]
                        } else {
                            vec!["one".into()]
                        },
                        evidence: "Authored observer fixture; no real action outcome".into(),
                        proposed_refinement: None,
                    },
                )?;
                let after = t.proposal_request()?;
                assert_eq!(after.admitted_criteria, request.admitted_criteria);
                assert!(!after.prior_outcomes.is_empty());
                assert!(after.frozen_expectations.iter().any(|f| f.id == frozen.id));
                assert_eq!(after.public_meaning, request.public_meaning);
                assert_eq!(s.qualitative_request_v2()?, original);
                let restarted = Engine::open(td.path())?;
                assert_eq!(
                    serde_json::to_value(&after)?,
                    serde_json::to_value(restarted.proposal_request()?)?
                );
                let next = restarted
                    .evaluate_proposal(offer(&restarted)?, true)?
                    .state
                    .pending
                    .unwrap();
                chosen.push(next.action);
                assert!(restarted
                    .events()?
                    .iter()
                    .all(|(_, e)| !matches!(e.kind, EventKind::Executed { .. })));
            }
            // Discrepancy changes pursuit; attached prose alone cannot alter an identical offer.
            assert_eq!(chosen[0], if met { "alpha" } else { "beta" });
            assert_eq!(chosen[0], chosen[1]);
        }
        Ok(())
    }
}
