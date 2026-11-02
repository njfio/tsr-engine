//! Local allowlisted retrieval. Receipts establish bytes accessed, never assertion truth.
use super::*;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Record {
    pub id: String,
    pub path: String,
    pub digest: Hash,
    pub attribution: String,
    pub uncertainty: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub root: PathBuf,
    pub records: Vec<Record>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub state_root: Hash,
    pub corpus: Hash,
    pub ids: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receipt {
    pub request: Request,
    pub request_digest: Hash,
    pub parent_context: Hash,
    pub records: Vec<RetrievedRecord>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inquiry: Option<EventId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pre_read_relevance: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RetrievedRecord {
    pub id: String,
    pub digest: Hash,
    pub bytes: Vec<u8>,
    pub attribution: String,
    pub uncertainty: String,
}
fn hash<T: Serialize>(v: &T) -> Result<Hash> {
    Ok(Hash::of(&[&serde_json::to_vec(v)?]))
}
impl Engine {
    pub(super) fn retrieval_state(&self) -> Result<(Option<Manifest>, Vec<(EventId, Receipt)>)> {
        let mut manifest = None;
        let mut receipts = vec![];
        for (id, e) in self.qualitative_entries()? {
            match e {
                Entry::RetrievalConfigured { manifest: m } => {
                    ensure!(manifest.is_none(), "duplicate corpus");
                    manifest = Some(m);
                }
                Entry::Retrieved { receipt } => receipts.push((id, receipt)),
                _ => {}
            }
        }
        Ok((manifest, receipts))
    }
    pub fn qualitative_configure_retrieval(
        &self,
        expected: Hash,
        manifest: Manifest,
    ) -> Result<Value> {
        self.write(|| {
            ensure!(
                self.store.head_root()? == expected,
                "stale corpus configuration"
            );
            let s = self.qualitative_state()?;
            ensure!(
                s.input.records.is_empty()
                    && s.proposals.is_empty()
                    && self.retrieval_state()?.0.is_none(),
                "fresh index-only context required; never retrofit visible history"
            );
            ensure!(
                manifest.root.is_absolute() && fs::canonicalize(&manifest.root)? == manifest.root,
                "canonical corpus root"
            );
            ensure!((1..=32).contains(&manifest.records.len()), "bounded corpus");
            let mut ids = BTreeSet::new();
            let mut paths = BTreeSet::new();
            for r in &manifest.records {
                ensure!(
                    bounded(&r.id)
                        && bounded(&r.path)
                        && bounded(&r.attribution)
                        && bounded(&r.uncertainty)
                        && ids.insert(&r.id)
                        && paths.insert(&r.path),
                    "unique bounded corpus records"
                );
                let bytes = tsr_batch::broker_v2::read_bounded_record(&manifest.root, &r.path)?;
                ensure!(Hash::of(&[&bytes]) == r.digest, "corpus bytes changed");
            }
            self.qualitative_append(Entry::RetrievalConfigured { manifest }, Source::Operator)?;
            self.qualitative_request_v2()
        })
    }
    /// Local admitted-record operation only; no commands, query language or arbitrary paths.
    pub fn qualitative_retrieve(&self, request: Request) -> Result<EventId> {
        self.qualitative_retrieve_bound(request, None, None)
    }
    pub(super) fn qualitative_retrieve_bound(
        &self,
        request: Request,
        inquiry: Option<EventId>,
        relevance: Option<String>,
    ) -> Result<EventId> {
        self.qualitative_retrieve_with_attempt(request, inquiry, relevance, None)
    }
    pub(super) fn qualitative_retrieve_with_attempt(
        &self,
        request: Request,
        inquiry: Option<EventId>,
        relevance: Option<String>,
        attempt: Option<EventId>,
    ) -> Result<EventId> {
        self.write(|| {
            self.check_information_attempt(attempt, &request, inquiry)?;
            if let Some(id) = inquiry {
                self.validate_exploration_read(id, &request.ids)?;
                ensure!(matches!(self.qualitative_inquiry_route(id)?, inquiry::InquiryRoute::CorpusCandidate{corpus} if corpus==request.corpus), "inquiry waiting: source cannot be resolved by corpus reader");
            }
            ensure!(
                inquiry.is_some()
                    || !self
                        .qualitative_entries()?
                        .iter()
                        .any(|(_, e)| matches!(e, Entry::InquiryProposed { .. })),
                "inquiry-aware state requires question-bound retrieval"
            );
            ensure!(
                self.store.head_root()? == request.state_root,
                "stale/replayed retrieval request"
            );
            ensure!(
                self.qualitative_request()?["phase"] == "propose",
                "pending review or stopped"
            );
            let (manifest, receipts) = self.retrieval_state()?;
            let m = manifest.context("no admitted retrieval corpus")?;
            ensure!(hash(&m)? == request.corpus, "wrong corpus");
            ensure!(fs::canonicalize(&m.root)? == m.root, "corpus root changed");
            ensure!(receipts.len() < 32, "retrieval receipt bound");
            ensure!(
                (1..=4).contains(&request.ids.len())
                    && request.ids.iter().collect::<BTreeSet<_>>().len() == request.ids.len(),
                "inspect 1..4 distinct IDs"
            );
            let mut records = vec![];
            for id in &request.ids {
                let r = m
                    .records
                    .iter()
                    .find(|r| &r.id == id)
                    .context("unadmitted record")?;
                let bytes = tsr_batch::broker_v2::read_bounded_record(&m.root, &r.path)?;
                ensure!(Hash::of(&[&bytes]) == r.digest, "altered corpus bytes");
                records.push(RetrievedRecord {
                    id: id.clone(),
                    digest: r.digest,
                    bytes,
                    attribution: r.attribution.clone(),
                    uncertainty: r.uncertainty.clone(),
                });
            }
            let s = self.qualitative_state()?;
            let receipt = Receipt {
                inquiry,
                pre_read_relevance: relevance,
                request_digest: hash(&request)?,
                request,
                parent_context: s.parent_context(&s.active().into_iter().collect::<Vec<_>>()),
                records,
            };
            self.qualitative_append(Entry::Retrieved { receipt }, Source::Rules)
        })
    }
    pub(super) fn valid_qualitative_basis(
        &self,
        basis: &Basis,
        _original_root: Option<Hash>,
    ) -> Result<bool> {
        let s = self.qualitative_state()?;
        match basis {
            Basis::InitialContext { context } => {
                Ok(*context == s.context && self.retrieval_state()?.1.is_empty())
            }
            Basis::RetrievalObservation { context, receipt } => {
                let (_, rs) = self.retrieval_state()?;
                let Some((id, r)) = rs.last() else {
                    return Ok(false);
                };
                // Review binds the pre-quarantine root separately; parent meanings must still match.
                Ok(*context == s.context
                    && receipt == id
                    && r.request_digest == hash(&r.request)?
                    && r.parent_context
                        == s.parent_context(&s.active().into_iter().collect::<Vec<_>>()))
            }
        }
    }
    pub(super) fn add_retrieval_context(&self, q: &mut Value) -> Result<()> {
        let (m, receipts) = self.retrieval_state()?;
        let Some(m) = m else { return Ok(()) };
        q["context_contract"] = json!("qualitative-retrieval-view@v1");
        q["record_index"]=json!(m.records.iter().map(|r|json!({"id":r.id,"digest":r.digest,"attribution":r.attribution,"uncertainty":r.uncertainty})).collect::<Vec<_>>());
        q["retrieval_contract"] = json!({"state_root":q["state_root"],"corpus":hash(&m)?,"ids":"1..4 distinct allowlisted IDs; explicit local admission required","max_receipts":32});
        q["retrieval_observations"] = json!(receipts.iter().map(|(id,r)| {
            let mut v = json!({"receipt":id,"request_digest":r.request_digest,"corpus":r.request.corpus,"selected_ids":r.request.ids,"records":r.records,"meaning":"actual bytes retrieved; assertions unverified, not desire fulfillment"});
            if let Some(inquiry) = r.inquiry { v["inquiry"] = json!(inquiry); v["pre_read_relevance"] = json!(r.pre_read_relevance); }
            v
        }).collect::<Vec<_>>());
        let linked: BTreeSet<_> = receipts.iter().filter_map(|(_, r)| r.inquiry).collect();
        if !linked.is_empty() {
            q["retrieval_questions"] = json!(self
                .qualitative_entries()?
                .into_iter()
                .filter_map(|(id, e)| match e {
                    Entry::InquiryProposed { inquiry, .. } if linked.contains(&id) =>
                        Some(json!({"proposal":id,"inquiry":inquiry})),
                    _ => None,
                })
                .collect::<Vec<_>>());
        }
        if let Some((receipt, _)) = receipts.last() {
            let basis = Basis::RetrievalObservation {
                context: self.qualitative_state()?.context,
                receipt: *receipt,
            };
            if self.valid_qualitative_basis(&basis, None)? {
                q["basis"] = json!(basis);
                q["response_contract"]["basis"] = json!(basis);
            } else {
                q["basis"] = Value::Null;
                q["response_contract"]["basis"] = Value::Null;
                q["phase"] = json!("retrieval_context_stale");
                q["stale_basis_reason"] = json!("Parent context changed; historical receipts remain true access records but are not a current reflection basis. Explicit reinspection or stop; no automatic action.");
            }
        }
        q["instructions"] = json!("Use the exact supplied basis. Initial context and retrieved assertions are distinct from interpretation. A retrieval receipt proves selected bytes were accessed, not truth, fulfillment or world change. Public meaning remains advisory and independently reviewed; keep public_desire basis unknown, relational catalysts empty and development absent because no outcome of pursuing the desires was observed. Retaining meaning or abstaining is legitimate. No execution authority.");
        q["response_contract"]["relational_fields"]["proposal"]["development"] =
            json!("omit; retrieval is not an empirical outcome of pursuing the desires");
        q["feedback_limit"]=json!("Retrieval establishes access and content identity only. It supplies no empirical truth of assertions, fulfillment measure or executable permission. Interpretation still requires independent review.");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn setup() -> Result<(tempfile::TempDir, Engine, Manifest)> {
        let d = tempfile::tempdir()?;
        let root = d.path().join("corpus");
        fs::create_dir(&root)?;
        fs::write(root.join("a.txt"), b"Fictional assertion A; truth unknown")?;
        fs::write(
            root.join("b.txt"),
            b"Fictional conflicting assertion B; truth unknown",
        )?;
        let e = Engine::open(&d.path().join("state"))?;
        e.qualitative_init(Bootstrap {
            parents: vec![
                Parent {
                    label: "a".into(),
                    meaning: "Understand without erasing disagreement".into(),
                    attribution: "authored test".into(),
                },
                Parent {
                    label: "b".into(),
                    meaning: "Retain useful starting points".into(),
                    attribution: "authored test".into(),
                },
            ],
            records: vec![],
            constraints: vec!["Preserve uncertainty".into()],
            acceptance_references: vec![],
        })?;
        let m = Manifest {
            root: fs::canonicalize(root)?,
            records: ["a", "b"]
                .iter()
                .map(|id| Record {
                    id: id.to_string(),
                    path: format!("{id}.txt"),
                    digest: Hash::of(&[
                        &fs::read(d.path().join(format!("corpus/{id}.txt"))).unwrap()
                    ]),
                    attribution: "evaluator-authored fiction".into(),
                    uncertainty: "not verified world truth".into(),
                })
                .collect(),
        };
        Ok((d, e, m))
    }
    fn request(e: &Engine, m: &Manifest) -> Result<Request> {
        Ok(Request {
            state_root: e.store.head_root()?,
            corpus: hash(m)?,
            ids: vec!["a".into()],
        })
    }
    #[test]
    fn real_local_bytes_reflection_review_and_restart() -> Result<()> {
        let (d, e, m) = setup()?;
        e.qualitative_configure_retrieval(e.store.head_root()?, m.clone())?;
        let before = e.qualitative_request_v2()?;
        assert!(!serde_json::to_string(&before)?.contains("Fictional assertion A"));
        let req = request(&e, &m)?;
        let id = e.qualitative_retrieve(req.clone())?;
        assert!(e.qualitative_retrieve(req).is_err());
        let q = e.qualitative_request_v2()?;
        assert_eq!(q["retrieval_observations"][0]["selected_ids"], json!(["a"]));
        assert_eq!(
            q["retrieval_observations"][0]["records"][0]["bytes"],
            json!(fs::read(m.root.join("a.txt"))?)
        );
        assert_eq!(q["observed_returns"], json!([]));
        let mut sub = Submission {
            basis: Basis::RetrievalObservation {
                context: e.qualitative_state()?.context,
                receipt: id,
            },
            generator: "authored-control".into(),
            response: Response {
                protocol: PROTOCOL.into(),
                state_root: e.store.head_root()?,
                attention_binding: serde_json::from_value(q["attention_binding"].clone())?,
                proposals: vec![],
                pursuit_expectations: Default::default(),
                common_event_forecasts: BTreeMap::new(),
            },
        };
        let root = e.store.head_root()?;
        e.qualitative_preview(sub.clone())?;
        assert_eq!(root, e.store.head_root()?); // honest retention
        sub.response.proposals.push(Suggestion::NewCandidate {
            description: "Compare the retrieved assertion with unresolved alternatives".into(),
            serves: vec![q["parents"][0]["id"].as_str().unwrap().into()],
            rationale: "Access is observed; source truth remains unknown".into(),
        });
        let p = e.qualitative_submit_possibility(sub)?;
        e.qualitative_review(Review {
            state_root: e.store.head_root()?,
            proposal: p,
            reviewer: "independent-authored-review".into(),
            decision: Decision::Admit,
            reason: "Authored wiring check, not semantic evidence".into(),
        })?;
        let after = e.qualitative_request_v2()?;
        assert_eq!(after["parents"], before["parents"]);
        assert_eq!(after["authority"], before["authority"]);
        assert_eq!(
            after["possibility_memory"][0]["basis"]["receipt"],
            json!(id)
        );
        assert_eq!(
            e.public_meaning_context()?["retrieval_observations"],
            after["retrieval_observations"]
        );
        drop(e);
        let e = Engine::open(&d.path().join("state"))?;
        assert_eq!(e.qualitative_request_v2()?, after);
        Ok(())
    }
    #[test]
    fn changed_meaning_exposes_stale_basis_instead_of_invalid_next_contract() -> Result<()> {
        use crate::engine::adaptive::coding::project::desire::{
            Basis as PublicBasis, Change, Proposal,
        };
        let (_d, e, m) = setup()?;
        e.qualitative_configure_retrieval(e.store.head_root()?, m.clone())?;
        e.qualitative_retrieve(request(&e, &m)?)?;
        let q = e.qualitative_request_v2()?;
        let id: PreferenceId = serde_json::from_value(q["parents"][0]["id"].clone())?;
        let sub = Submission {
            basis: serde_json::from_value(q["basis"].clone())?,
            generator: "authored".into(),
            response: Response {
                protocol: PROTOCOL.into(),
                state_root: e.store.head_root()?,
                attention_binding: serde_json::from_value(q["attention_binding"].clone())?,
                proposals: vec![Suggestion::PublicDesire {
                    proposal: Proposal {
                        binding: serde_json::from_value(q["attention_binding"].clone())?,
                        identity: id,
                        revises: None,
                        parents: vec![id],
                        meaning: "Authored developed meaning; no observed fulfillment".into(),
                        change: Change::Development,
                        basis: PublicBasis::Unknown,
                        generator: "authored".into(),
                    },
                }],
                pursuit_expectations: Default::default(),
                common_event_forecasts: BTreeMap::new(),
            },
        };
        let p = e.qualitative_submit(sub)?;
        e.qualitative_review(Review {
            state_root: e.store.head_root()?,
            proposal: p,
            reviewer: "independent".into(),
            decision: Decision::Admit,
            reason: "Authored fixture only".into(),
        })?;
        let next = e.qualitative_request_v2()?;
        assert_eq!(next["phase"], "retrieval_context_stale");
        assert!(next["basis"].is_null());
        assert_eq!(next["retrieval_observations"], q["retrieval_observations"]);
        assert!(e.public_meaning_context().is_err());
        Ok(())
    }
    #[test]
    fn wrong_changed_duplicate_and_stale_inputs_fail_closed() -> Result<()> {
        let (_d, e, m) = setup()?;
        e.qualitative_configure_retrieval(e.store.head_root()?, m.clone())?;
        let mut r = request(&e, &m)?;
        r.corpus = Hash::of(&[b"wrong"]);
        assert!(e.qualitative_retrieve(r).is_err());
        let mut r = request(&e, &m)?;
        r.ids = vec!["a".into(), "a".into()];
        assert!(e.qualitative_retrieve(r).is_err());
        let mut r = request(&e, &m)?;
        r.ids = vec!["outside".into()];
        assert!(e.qualitative_retrieve(r).is_err());
        fs::write(m.root.join("a.txt"), b"changed")?;
        assert!(e.qualitative_retrieve(request(&e, &m)?).is_err());
        fs::write(
            m.root.join("a.txt"),
            b"Fictional assertion A; truth unknown",
        )?;
        let receipt = e.qualitative_retrieve(request(&e, &m)?)?;
        let q = e.qualitative_request_v2()?;
        let mut sub = Submission {
            basis: Basis::RetrievalObservation {
                context: e.qualitative_state()?.context,
                receipt,
            },
            generator: "authored".into(),
            response: Response {
                protocol: PROTOCOL.into(),
                state_root: e.store.head_root()?,
                attention_binding: serde_json::from_value(q["attention_binding"].clone())?,
                proposals: vec![],
                pursuit_expectations: Default::default(),
                common_event_forecasts: BTreeMap::new(),
            },
        };
        let mut forged = serde_json::to_value(&sub)?;
        forged["basis"]["truth_verified"] = json!(true);
        assert!(serde_json::from_value::<Submission>(forged).is_err());
        sub.basis = Basis::RetrievalObservation {
            context: e.qualitative_state()?.context,
            receipt: EventId(Hash::of(&[b"fake"])),
        };
        assert!(e.qualitative_preview(sub.clone()).is_err());
        sub.basis = Basis::RetrievalObservation {
            context: e.qualitative_state()?.context,
            receipt,
        };
        e.qualitative_status(Status {
            state_root: e.store.head_root()?,
            identity: serde_json::from_value(q["parents"][0]["id"].clone())?,
            revision: None,
            withdrawn: true,
        })?;
        sub.response.state_root = e.store.head_root()?;
        sub.response.attention_binding =
            serde_json::from_value(e.qualitative_request()?["attention_binding"].clone())?;
        assert!(e.qualitative_preview(sub).is_err());
        Ok(())
    }
    #[test]
    fn traversal_symlink_and_visible_context_not_retrofitted() -> Result<()> {
        let (d, e, m) = setup()?;
        let mut bad = m.clone();
        bad.records[0].path = "../outside".into();
        assert!(e
            .qualitative_configure_retrieval(e.store.head_root()?, bad)
            .is_err());
        #[cfg(unix)]
        {
            fs::write(
                d.path().join("outside"),
                b"Fictional assertion A; truth unknown",
            )?;
            fs::remove_file(m.root.join("a.txt"))?;
            std::os::unix::fs::symlink(d.path().join("outside"), m.root.join("a.txt"))?;
            assert!(e
                .qualitative_configure_retrieval(e.store.head_root()?, m.clone())
                .is_err());
        }
        let e2 = Engine::open(&d.path().join("visible"))?;
        let mut input = e.qualitative_state()?.input;
        input.records.push(VisibleRecord {
            label: "visible".into(),
            content: "Already seen".into(),
            attribution: "fixture".into(),
            uncertainty: "unknown".into(),
        });
        e2.qualitative_init(input)?;
        assert!(e2
            .qualitative_configure_retrieval(e2.store.head_root()?, m)
            .is_err());
        Ok(())
    }
}
