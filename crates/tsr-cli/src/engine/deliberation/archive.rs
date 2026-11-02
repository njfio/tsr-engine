//! Evaluator-authored local simulation. Events are not messages to real actors.
use super::*;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRecord {
    pub id: String,
    pub revision: String,
    pub text: String,
    pub affected: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    fn input() -> serde_json::Value {
        serde_json::from_str(include_str!(
            "../../../../../docs/engine-synthetic-archive-inputs.json"
        ))
        .unwrap()
    }
    fn fixture() -> Spec {
        serde_json::from_value(input()["spec"].clone()).unwrap()
    }
    fn register(id: &str, revision: &str) -> Operation {
        Operation::Register {
            challenge: id.into(),
            revision: revision.into(),
        }
    }
    fn route(actor: &str) -> Operation {
        Operation::Route {
            challenge: "access".into(),
            revision: "hours-v1".into(),
            actor: actor.into(),
        }
    }
    fn response(id: &str) -> Operation {
        Operation::RecordResponse {
            challenge: "access".into(),
            revision: "hours-v1".into(),
            response: id.into(),
        }
    }
    #[test]
    fn behavior_and_adversarial_provenance() -> Result<()> {
        let spec = fixture();
        let world = spec.archive.as_ref().unwrap();
        world.validate()?;
        assert_eq!(world.measure(&[], true), 0.0); // Ignored challenge is not addressed.
        let reg = register("access", "hours-v1");
        let h = world.apply(&[], &reg)?;
        assert_eq!(world.measure(&h, false), 0.5);
        assert!(world.apply(&h, &reg).is_err());
        assert!(world.apply(&[], &register("access", "stale")).is_err());
        assert!(world.apply(&h, &route("outsider")).is_err());
        assert!(world.apply(&h, &response("access-review")).is_err());
        assert!(world.apply(&h, &response("invented-resolution")).is_err());
        let h = world.apply(&h, &route("caretaker"))?;
        let acknowledged = world.apply(&h, &response("access-ack"))?;
        assert_eq!(
            world.view(&acknowledged)["challenges"][0]["status"],
            "unresolved"
        );
        assert!(world
            .apply(&acknowledged, &response("access-review"))
            .is_err()); // Wrong route.
        let h = world.apply(&acknowledged, &route("night-reader"))?;
        let addressed = world.apply(&h, &response("access-review"))?;
        assert_eq!(
            world.view(&addressed)["challenges"][0]["status"],
            "addressed_in_simulation"
        );
        assert_eq!(world.measure(&addressed, true), 0.5); // Other challenge still ignored.
        let withdrawal = Operation::Withdraw {
            challenge: "access".into(),
            revision: "hours-v1".into(),
            evidence: "withdrawal-evidence".into(),
        };
        assert!(world.apply(&addressed, &withdrawal).is_err());
        let mut world = world.clone();
        world.withdrawals.push(WithdrawalRecord {
            id: "withdrawal-evidence".into(),
            challenge: "access".into(),
            revision: "hours-v1".into(),
            author: "newcomer".into(),
        });
        world.validate()?;
        let mut forged = world.clone();
        forged.withdrawals[0].author = "caretaker".into();
        assert!(forged.validate().is_err());
        assert!(forged.apply(&addressed, &withdrawal).is_err());
        let mut wrong = world.clone();
        wrong.withdrawals[0].challenge = "meaning".into();
        assert!(wrong.validate().is_err());
        assert!(wrong.apply(&addressed, &withdrawal).is_err());
        let withdrawn = world.apply(&addressed, &withdrawal)?;
        assert!(world.apply(&withdrawn, &withdrawal).is_err());
        assert_eq!(&withdrawn[..addressed.len()], addressed.as_slice());
        assert_eq!(world.measure(&withdrawn, false), 0.0);
        assert!(world.apply(&withdrawn, &route("caretaker")).is_err());
        assert_eq!(
            world.view(&withdrawn)["challenges"][0]["challenge"]["dissent"],
            world.challenges[0].dissent
        );
        assert!(serde_json::from_value::<Operation>(
            serde_json::json!({"kind":"erase","challenge":"access","revision":"hours-v1"})
        )
        .is_err());
        let mut tampered = world.clone();
        tampered.responses[0].revision = "wrong".into();
        assert!(tampered.validate().is_err());
        let mut tampered = world.clone();
        tampered.challenges[0].dissent.clear();
        assert!(tampered.validate().is_err());
        Ok(())
    }
    fn offer(e: &Engine, op: Operation) -> Result<Offer> {
        let (spec, _, book, _) = e.deliberation_state()?;
        let action = Move::Archive { event: op };
        let after = observe(&spec, &transition(&spec, &book, &action)?);
        let before = observe(&spec, &book);
        Ok(Offer {
            state_root: e.store.head_root()?,
            directions: vec![Direction {
                purpose: "DETERMINISTIC fixture operation, not model discovery".into(),
                supports: (0..2).filter(|i| after[*i] > before[*i]).collect(),
                anticipated: after,
                action,
                continuation: None,
                deferred_observation: None,
            }],
        })
    }
    fn import(e: &Engine) -> Result<serde_json::Value> {
        let q = e.deliberation_request()?;
        let r = &q["relational_request"];
        let mut generated = input()["aspiration"].clone();
        generated["parents"] = q["obligation_ids"].clone();
        generated["revises"] = serde_json::Value::Null;
        generated["catalysts"] = serde_json::json!([]);
        e.submit_relational(serde_json::from_value(serde_json::json!({"state_root":q["state_root"],"return_commitment":r["return_commitment"],"observation_revision":r["observation_revision"],"evidence_key":r["evidence_key"],"proposals":[generated]}))?)
    }
    #[test]
    fn shared_loop_preserves_parents_history_and_requires_return_review() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let e = Engine::open(dir.path())?;
        let initial = e.deliberation_initialize(fixture())?;
        let reg = offer(&e, register("access", "hours-v1"))?;
        e.deliberate(reg)?;
        let imported = import(&e)?;
        let proposal = &imported["relational_guidance"][0]["proposal"];
        assert_eq!(proposal["aspiration"], input()["aspiration"]["aspiration"]);
        let old = imported["relational_guidance"][0]["revision"].clone();
        let next = offer(&e, route("night-reader"))?;
        let stale = next.clone();
        e.deliberate(next)?;
        let q = Engine::open(dir.path())?.deliberation_request()?;
        assert_eq!(q["standing_desires"], initial["standing_desires"]);
        assert_eq!(q["authority"], initial["authority"]);
        assert_eq!(q["archive"]["world"], initial["archive"]["world"]);
        assert_eq!(q["archive"]["events"].as_array().unwrap().len(), 2);
        assert_eq!(q["relational_request"]["due"], true);
        assert!(q["relational_guidance"].as_array().unwrap().is_empty());
        assert_eq!(
            q["relational_request"]["existing"][0]["discriminator_tested"],
            "unknown"
        );
        let root = e.store.head_root()?;
        assert!(e.deliberate(stale).is_err());
        assert_eq!(root, e.store.head_root()?);
        let mut revision = proposal.clone();
        revision["revises"] = old;
        let r = &q["relational_request"];
        e.submit_relational(serde_json::from_value(serde_json::json!({"state_root":q["state_root"],"return_commitment":r["return_commitment"],"observation_revision":r["observation_revision"],"evidence_key":r["evidence_key"],"proposals":[revision]}))?)?;
        assert_eq!(
            e.deliberation_request()?["relational_request"]["history"]
                .as_array()
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            Engine::open(dir.path())?.deliberation_request()?,
            e.deliberation_request()?
        );
        Ok(())
    }
    #[test]
    #[ignore = "offline Python/Rust boundary equivalence artifact"]
    fn export_archive_boundary_cases() -> Result<()> {
        let world = fixture().archive.unwrap();
        let reg = register("access", "hours-v1");
        let route = route("night-reader");
        let histories = vec![vec![], vec![reg.clone()], vec![reg.clone(), route.clone()]];
        let candidates = vec![
            reg,
            register("access", "stale"),
            register("meaning", "translation-v1"),
            route,
            super::tests::route("outsider"),
            response("access-review"),
            response("access-ack"),
            response("missing"),
            Operation::RecordResponse {
                challenge: "meaning".into(),
                revision: "translation-v1".into(),
                response: "access-review".into(),
            },
        ];
        let mut rows = vec![];
        for h in &histories {
            for op in &candidates {
                rows.push(serde_json::json!({"world":world,"events":h,"event":op,"valid":world.apply(h,op).is_ok(),"prerequisites":world.prerequisites(h)}));
            }
        }
        std::fs::write(
            std::env::var("TSR_ARCHIVE_CASES")?,
            serde_json::to_vec_pretty(&rows)?,
        )?;
        Ok(())
    }
    #[test]
    #[ignore = "explicit synthetic fixture construction; no model calls"]
    fn build_synthetic_archive_fixture() -> Result<()> {
        let dir = std::path::PathBuf::from(std::env::var("TSR_ARCHIVE_RUN")?);
        ensure!(!dir.exists(), "fresh synthetic destination required");
        let e = Engine::open(&dir)?;
        e.deliberation_initialize(fixture())?;
        // Evaluator-authored scenario setup, explicitly executed by the controller.
        e.deliberate(offer(&e, register("access", "hours-v1"))?)?;
        let q = import(&e)?;
        std::fs::write(
            dir.join("SYNTHETIC-NOT-OPERATIONAL.json"),
            serde_json::to_vec_pretty(
                &serde_json::json!({"synthetic_only":true,"seed_actions":1,"seed_origin":"evaluator fixture setup; not generated", "aspiration_origin":input(),"state_root":q["state_root"],"semantic_fulfillment":"unknown"}),
            )?,
        )?;
        std::fs::write(
            dir.join("initial-request.json"),
            serde_json::to_vec_pretty(&q)?,
        )?;
        Ok(())
    }
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Challenge {
    pub id: String,
    pub decision: String,
    pub revision: String,
    pub author: String,
    pub dissent: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResponseRecord {
    pub id: String,
    pub challenge: String,
    pub revision: String,
    pub actor: String,
    pub text: String,
    pub addresses: bool,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WithdrawalRecord {
    pub id: String,
    pub challenge: String,
    pub revision: String,
    pub author: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct World {
    pub synthetic_only: bool,
    pub decisions: Vec<DecisionRecord>,
    pub challenges: Vec<Challenge>,
    pub responses: Vec<ResponseRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub withdrawals: Vec<WithdrawalRecord>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Operation {
    Register {
        challenge: String,
        revision: String,
    },
    Route {
        challenge: String,
        revision: String,
        actor: String,
    },
    RecordResponse {
        challenge: String,
        revision: String,
        response: String,
    },
    Withdraw {
        challenge: String,
        revision: String,
        evidence: String,
    },
}
impl Operation {
    pub fn challenge(&self) -> &str {
        match self {
            Self::Register { challenge, .. }
            | Self::Route { challenge, .. }
            | Self::RecordResponse { challenge, .. }
            | Self::Withdraw { challenge, .. } => challenge,
        }
    }
    fn revision(&self) -> &str {
        match self {
            Self::Register { revision, .. }
            | Self::Route { revision, .. }
            | Self::RecordResponse { revision, .. }
            | Self::Withdraw { revision, .. } => revision,
        }
    }
}
impl World {
    pub fn validate(&self) -> Result<()> {
        ensure!(self.synthetic_only, "explicit synthetic archive required");
        ensure!(
            (1..=8).contains(&self.decisions.len())
                && (1..=8).contains(&self.challenges.len())
                && self.responses.len() <= 16,
            "bounded archive"
        );
        let unique = |ids: Vec<&str>| {
            ids.iter().all(|s| bounded(s)) && ids.iter().collect::<BTreeSet<_>>().len() == ids.len()
        };
        ensure!(
            unique(self.decisions.iter().map(|d| d.id.as_str()).collect())
                && unique(self.challenges.iter().map(|c| c.id.as_str()).collect())
                && unique(self.responses.iter().map(|r| r.id.as_str()).collect()),
            "unique archive identities"
        );
        for d in &self.decisions {
            ensure!(
                bounded(&d.revision)
                    && bounded(&d.text)
                    && (1..=4).contains(&d.affected.len())
                    && unique(d.affected.iter().map(String::as_str).collect()),
                "bounded decision provenance"
            );
        }
        for c in &self.challenges {
            let d = self
                .decisions
                .iter()
                .find(|d| d.id == c.decision)
                .ok_or_else(|| anyhow::anyhow!("unknown challenged decision"))?;
            ensure!(
                c.revision == d.revision && bounded(&c.author) && bounded(&c.dissent),
                "stale or missing challenge provenance"
            );
        }
        ensure!(
            self.withdrawals.len() <= 8
                && unique(self.withdrawals.iter().map(|w| w.id.as_str()).collect()),
            "bounded unique withdrawal records"
        );
        for w in &self.withdrawals {
            let c = self
                .challenges
                .iter()
                .find(|c| c.id == w.challenge)
                .ok_or_else(|| anyhow::anyhow!("unknown withdrawal challenge"))?;
            ensure!(
                w.author == c.author && w.revision == c.revision,
                "withdrawal requires challenge author and current revision"
            );
        }
        for r in &self.responses {
            let c = self
                .challenges
                .iter()
                .find(|c| c.id == r.challenge)
                .ok_or_else(|| anyhow::anyhow!("unknown response challenge"))?;
            let d = self.decisions.iter().find(|d| d.id == c.decision).unwrap();
            ensure!(
                r.revision == c.revision && d.affected.contains(&r.actor) && bounded(&r.text),
                "invalid response provenance"
            );
        }
        Ok(())
    }
    pub fn apply(&self, history: &[Operation], op: &Operation) -> Result<Vec<Operation>> {
        ensure!(history.len() < 64, "archive event budget exhausted");
        let c = self
            .challenges
            .iter()
            .find(|c| c.id == op.challenge())
            .ok_or_else(|| anyhow::anyhow!("unknown challenge"))?;
        ensure!(op.revision() == c.revision, "stale decision provenance");
        let d = self.decisions.iter().find(|d| d.id == c.decision).unwrap();
        let prior: Vec<_> = history.iter().filter(|e| e.challenge() == c.id).collect();
        ensure!(!history.contains(op), "duplicate archive effect");
        ensure!(
            !prior
                .iter()
                .any(|e| matches!(e, Operation::Withdraw { .. })),
            "challenge withdrawn; history retained"
        );
        let registered = prior
            .iter()
            .any(|e| matches!(e, Operation::Register { .. }));
        match op {
            Operation::Register { .. } => ensure!(!registered, "already registered"),
            Operation::Route { actor, .. } => ensure!(
                registered && d.affected.contains(actor),
                "registration and admitted affected actor required"
            ),
            Operation::RecordResponse { response, .. } => {
                let r = self
                    .responses
                    .iter()
                    .find(|r| r.id == *response)
                    .ok_or_else(|| anyhow::anyhow!("unsupported response/resolution"))?;
                ensure!(
                    registered
                        && r.challenge == c.id
                        && r.revision == c.revision
                        && prior
                            .iter()
                            .any(|e| matches!(e,Operation::Route{actor,..} if *actor==r.actor)),
                    "response needs matching routed challenge"
                );
            }
            Operation::Withdraw { evidence, .. } => {
                let w = self
                    .withdrawals
                    .iter()
                    .find(|w| w.id == *evidence)
                    .ok_or_else(|| anyhow::anyhow!("unsupported withdrawal"))?;
                ensure!(
                    registered
                        && w.challenge == c.id
                        && w.revision == c.revision
                        && w.author == c.author,
                    "withdrawal requires matching admitted author evidence"
                );
            }
        }
        let mut next = history.to_vec();
        next.push(op.clone());
        Ok(next)
    }
    pub fn prerequisites(&self, history: &[Operation]) -> serde_json::Value {
        let rows: Vec<_> = self.challenges.iter().map(|c| {
            let prior: Vec<_> = history.iter().filter(|e|e.challenge()==c.id).collect();
            let routes: BTreeSet<_> = prior.iter().filter_map(|e|match e {Operation::Route {actor,..}=>Some(actor.as_str()), _=>None}).collect();
            let responses: Vec<_> = self.responses.iter().filter(|r|r.challenge==c.id).map(|r|serde_json::json!({"response":r.id,"revision":r.revision,"required_respondent":r.actor,"respondent_already_routed":routes.contains(r.actor.as_str())})).collect();
            serde_json::json!({"challenge":c.id,"decision":c.decision,"required_revision":c.revision,"registered":prior.iter().any(|e|matches!(e,Operation::Register{..})),"withdrawn":prior.iter().any(|e|matches!(e,Operation::Withdraw{..})),"already_routed_actors":routes,"response_prerequisites":responses})
        }).collect();
        serde_json::json!({"contract":"archive-current-history-prerequisites@v1","challenges":rows,"rules":{
            "all":"Exact current challenge/decision revision; not withdrawn; no duplicate event. Each alternative is evaluated independently against current history, not against other offered alternatives.",
            "register":"Challenge must not already be registered; no insider endorsement is required.",
            "route":"Challenge must already be registered; actor must be an admitted affected actor.",
            "record_response":"Challenge must already be registered AND already routed to this response record's exact respondent; admitted response existence alone is insufficient. Citation must match challenge and revision.",
            "withdraw":"Challenge must already be registered; cite admitted withdrawal evidence matching challenge author and revision."},"role":"Prerequisite facts only, not ranking, proposed actions, authority or satisfaction."})
    }
    pub fn view(&self, history: &[Operation]) -> serde_json::Value {
        let challenges:Vec<_>=self.challenges.iter().map(|c| {
            let events:Vec<_>=history.iter().filter(|e|e.challenge()==c.id).collect();
            let registered=events.iter().any(|e|matches!(e,Operation::Register{..}));
            let routed=events.iter().any(|e|matches!(e,Operation::Route{..}));
            let withdrawn=events.iter().any(|e|matches!(e,Operation::Withdraw{..}));
            let addressed=events.iter().any(|e|matches!(e,Operation::RecordResponse{response,..} if self.responses.iter().any(|r|r.id==*response&&r.addresses)));
            serde_json::json!({"challenge":c,"registered":registered,"routed":routed,"status":if withdrawn{"withdrawn"}else if addressed{"addressed_in_simulation"}else{"unresolved"},"addressed":addressed,"withdrawn":withdrawn})
        }).collect();
        serde_json::json!({"synthetic_only":true,"world":self,"events":history,"prerequisites":self.prerequisites(history),"challenges":challenges,"semantic_fulfillment":"unknown","delivery_to_real_people":false})
    }
    pub fn measure(&self, history: &[Operation], traceability: bool) -> f64 {
        let view = self.view(history);
        let rows = view["challenges"].as_array().unwrap();
        rows.iter()
            .map(|r| {
                if r["withdrawn"] == true {
                    return 0.0;
                }
                let registered = if r["registered"] == true { 1.0 } else { 0.0 };
                if traceability {
                    (registered
                        + if r["routed"] == true { 1.0 } else { 0.0 }
                        + if r["addressed"] == true { 1.0 } else { 0.0 })
                        / 3.0
                } else {
                    registered
                }
            })
            .sum::<f64>()
            / rows.len() as f64
    }
}
