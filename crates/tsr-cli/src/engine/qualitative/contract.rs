//! Native-owned bounded producer profile, emitted and checked by the same definition.
//! Not provider JSON-Schema enforcement; native semantic validation remains mandatory.
use super::*;
use inquiry::{AnswerSource, Origin};
pub const VERSION: &str = "qualitative-model-contract@v1";
const GENERATOR: &str = "gpt-6-luna/qualitative-generator";
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Shape {
    Literal {
        value: Value,
    },
    Text {
        max_utf8_bytes: usize,
        nonblank: bool,
    },
    Array {
        items: Box<Shape>,
        min_items: usize,
        max_items: usize,
        unique: bool,
        contains: Vec<Value>,
    },
    Object {
        fields: BTreeMap<String, Shape>,
        optional: Vec<String>,
        additional_fields: bool,
    },
    Union {
        variants: Vec<Shape>,
    },
}
fn lit(v: impl Serialize) -> Shape {
    Shape::Literal { value: json!(v) }
}
fn text() -> Shape {
    Shape::Text {
        max_utf8_bytes: MAX_TEXT_BYTES,
        nonblank: true,
    }
}
fn obj(fields: Vec<(&str, Shape)>, optional: &[&str]) -> Shape {
    Shape::Object {
        fields: fields.into_iter().map(|(k, v)| (k.into(), v)).collect(),
        optional: optional.iter().map(|v| v.to_string()).collect(),
        additional_fields: false,
    }
}
fn union(variants: Vec<Shape>) -> Shape {
    Shape::Union { variants }
}
fn nullable(s: Shape) -> Shape {
    union(vec![lit(Value::Null), s])
}
fn array(items: Shape, min: usize, max: usize, unique: bool) -> Shape {
    Shape::Array {
        items: Box::new(items),
        min_items: min,
        max_items: max,
        unique,
        contains: vec![],
    }
}
fn choices(v: &Value) -> Shape {
    union(v.as_array().unwrap_or(&vec![]).iter().map(lit).collect())
}
impl Shape {
    fn locations(&self, name: &str, path: &str, out: &mut Vec<String>) {
        match self {
            Self::Object { fields, .. } => {
                for (key, shape) in fields {
                    let next = format!("{path}.{key}");
                    if key == name {
                        out.push(next.clone());
                    }
                    shape.locations(name, &next, out);
                }
            }
            Self::Array { items, .. } => items.locations(name, &format!("{path}[]"), out),
            Self::Union { variants } => {
                for shape in variants {
                    shape.locations(name, path, out);
                }
            }
            _ => (),
        }
    }
    fn diagnostic(
        &self,
        root: &Shape,
        path: &str,
        code: &str,
        field: Option<(&str, usize)>,
    ) -> anyhow::Error {
        let mut detail = json!({"version":"native-shape-diagnostic@v1", "code":code, "path":path.replacen("response", "$", 1)});
        if let Self::Object {
            fields, optional, ..
        } = self
        {
            detail["allowed_fields"] = json!(fields.keys().collect::<Vec<_>>());
            detail["required_fields"] = json!(fields
                .keys()
                .filter(|k| !optional.contains(k))
                .collect::<Vec<_>>());
        }
        match self {
            Self::Text {
                max_utf8_bytes,
                nonblank,
            } => {
                detail["expected_type"] = json!("string");
                detail["max_utf8_bytes"] = json!(max_utf8_bytes);
                detail["nonblank"] = json!(nonblank);
            }
            Self::Array {
                min_items,
                max_items,
                ..
            } => {
                detail["expected_type"] = json!("array");
                detail["min_items"] = json!(min_items);
                detail["max_items"] = json!(max_items);
            }
            Self::Object { .. } => {
                detail["expected_type"] = json!("object");
            }
            _ => (),
        }
        if let Some((name, ordinal)) = field {
            let mut locations = vec![];
            root.locations(name, "$", &mut locations);
            locations.sort();
            locations.dedup();
            // Only native-contract names (and the request-envelope stage label)
            // may leave the quarantine. Never echo arbitrary model field names.
            if !locations.is_empty() || name == "stage" {
                detail["path"] = json!(format!("{}.{name}", path.replacen("response", "$", 1)));
                detail["valid_locations"] = json!(locations);
            } else {
                detail["unknown_key_ordinal"] = json!(ordinal);
                detail["field_name_withheld"] = json!(true);
            }
        }
        anyhow::anyhow!("TSR_SHAPE_V1:{}", detail)
    }
    fn check(&self, v: &Value, path: &str) -> Result<()> {
        self.check_inner(v, path, self)
    }
    fn check_inner(&self, v: &Value, path: &str, root: &Shape) -> Result<()> {
        match self {
            Self::Literal { value } => ensure!(v == value, "{path}: exact value/identity required"),
            Self::Text {
                max_utf8_bytes,
                nonblank,
            } => {
                let s = v
                    .as_str()
                    .ok_or_else(|| self.diagnostic(root, path, "string_required", None))?;
                if s.len() > *max_utf8_bytes || (*nonblank && s.trim().is_empty()) {
                    return Err(self.diagnostic(root, path, "text_bound", None));
                }
            }
            Self::Array {
                items,
                min_items,
                max_items,
                unique,
                contains,
            } => {
                let a = v
                    .as_array()
                    .ok_or_else(|| self.diagnostic(root, path, "array_required", None))?;
                if !(*min_items..=*max_items).contains(&a.len()) {
                    return Err(self.diagnostic(root, path, "array_length", None));
                }
                for (i, x) in a.iter().enumerate() {
                    items.check_inner(x, &format!("{path}[{i}]"), root)?;
                    if *unique {
                        ensure!(!a[..i].contains(x), "{path}: duplicate item");
                    }
                }
                ensure!(
                    contains.iter().all(|x| a.contains(x)),
                    "{path}: required lineage missing"
                );
            }
            Self::Object {
                fields, optional, ..
            } => {
                let o = v
                    .as_object()
                    .ok_or_else(|| self.diagnostic(root, path, "object_required", None))?;
                if let Some((ordinal, name)) =
                    o.keys().enumerate().find(|(_, k)| !fields.contains_key(*k))
                {
                    return Err(self.diagnostic(
                        root,
                        path,
                        "unsupported_field",
                        Some((name, ordinal)),
                    ));
                }
                for (k, s) in fields {
                    match o.get(k) {
                        Some(x) => s.check_inner(x, &format!("{path}.{k}"), root)?,
                        None => {
                            if !optional.contains(k) {
                                return Err(self.diagnostic(
                                    root,
                                    &format!("{path}.{k}"),
                                    "required_field",
                                    None,
                                ));
                            }
                        }
                    }
                }
            }
            Self::Union { variants } => {
                let mut errors = vec![];
                for variant in variants {
                    match variant.check_inner(v, path, root) {
                        Ok(()) => return Ok(()),
                        Err(error) => errors.push(error.to_string()),
                    }
                }
                // Diagnostic paths/types only: never echo response content.
                let detail: String = errors
                    .into_iter()
                    .take(4)
                    .collect::<Vec<_>>()
                    .join("; ")
                    .chars()
                    .take(1600)
                    .collect();
                bail!("{path}: no permitted variant: {detail}");
            }
        }
        Ok(())
    }
}
impl Engine {
    fn qualitative_stage_shape(&self, stage: &str, focus: Option<EventId>) -> Result<Shape> {
        let q = self.qualitative_question_request()?;
        let ids = json!(q["parents"]
            .as_array()
            .context("parents")?
            .iter()
            .filter(|p| p["active"] == true)
            .map(|p| p["id"].clone())
            .collect::<Vec<_>>());
        let parent_list = || array(choices(&ids), 1, 16, true);
        let corpus = q["retrieval_contract"]["corpus"].clone();
        let records = json!(q["record_index"]
            .as_array()
            .context("corpus required")?
            .iter()
            .map(|r| r["id"].clone())
            .collect::<Vec<_>>());
        let mut sources = vec![obj(
            vec![
                ("kind", lit("corpus_assertions")),
                ("corpus", lit(&corpus)),
                ("rationale", text()),
            ],
            &[],
        )];
        for source in [
            AnswerSource::UserPreference,
            AnswerSource::UserAuthority,
            AnswerSource::ExternalEvidence,
            AnswerSource::Mixed,
            AnswerSource::Unknown,
        ] {
            sources.push(lit(source));
        }
        let base_inquiry =
            |origin: Origin, source: Shape, exploration: Shape, optional: &[&str]| {
                obj(
                    vec![
                        ("state_root", lit(&q["state_root"])),
                        ("attention", lit(&q["inquiry_binding"])),
                        ("parents", parent_list()),
                        ("question", text()),
                        ("uncertainty", text()),
                        ("attribution", lit("model:gpt-6-luna:inquiry")),
                        ("origin", lit(origin)),
                        ("answer_source", source),
                        ("exploration", exploration),
                    ],
                    optional,
                )
            };
        match stage {
            "inquiry" => {
                let mut variants = vec![base_inquiry(
                    Origin::ModelProposed,
                    nullable(union(sources.clone())),
                    lit(Value::Null),
                    &["answer_source", "exploration"],
                )];
                if q["exploration_policy"]["enabled"] == true {
                    let exploration = obj(
                        vec![
                            ("policy", lit(&q["exploration_policy"]["policy"])),
                            ("considered_records", array(choices(&records), 1, 32, true)),
                            ("possible_information_benefit", text()),
                            ("why_no_user_priority_needed", text()),
                        ],
                        &[],
                    );
                    variants.push(base_inquiry(
                        Origin::EngineExploration,
                        sources[0].clone(),
                        exploration,
                        &[],
                    ));
                }
                Ok(obj(vec![("inquiry", nullable(union(variants)))], &[]))
            }
            "inquiry_review" | "review" => {
                let pending = q["pending_reviews"].as_array().context("pending")?;
                ensure!(pending.len() == 1, "exactly one pending review");
                let reviewer = if stage == "inquiry_review" {
                    "model:gpt-6-luna:inquiry-review"
                } else {
                    "gpt-6-luna/independent-context-review"
                };
                Ok(obj(
                    vec![
                        ("state_root", lit(&q["state_root"])),
                        ("proposal", lit(&pending[0]["proposal"])),
                        ("reviewer", lit(reviewer)),
                        (
                            "decision",
                            union(
                                [Decision::Admit, Decision::Reject, Decision::Unknown]
                                    .iter()
                                    .map(lit)
                                    .collect(),
                            ),
                        ),
                        ("reason", text()),
                    ],
                    &[],
                ))
            }
            "select" => {
                let inquiries = q["reviewed_information_questions"]
                    .as_array()
                    .context("reviewed inquiry")?;
                let i = if let Some(id) = focus {
                    inquiries
                        .iter()
                        .find(|i| i["proposal"] == json!(id))
                        .context("focused inquiry is not current and reviewed")?
                } else {
                    ensure!(
                        inquiries.len() == 1,
                        "explicit inquiry focus required for multiple reviewed inquiries"
                    );
                    &inquiries[0]
                };
                let options = i["inquiry"]["exploration"]["considered_records"]
                    .as_array()
                    .map(|_| i["inquiry"]["exploration"]["considered_records"].clone())
                    .unwrap_or(records);
                Ok(obj(
                    vec![
                        ("inquiry", lit(&i["proposal"])),
                        (
                            "request",
                            obj(
                                vec![
                                    ("state_root", lit(&q["state_root"])),
                                    ("corpus", lit(corpus)),
                                    ("ids", array(choices(&options), 0, 1, true)),
                                ],
                                &[],
                            ),
                        ),
                        ("relevance", text()),
                    ],
                    &[],
                ))
            }
            "reflect" | "next" => {
                let mut suggestions = vec![obj(
                    vec![
                        ("kind", lit("new_candidate")),
                        ("description", text()),
                        ("serves", array(choices(&ids), 1, 16, false)),
                        ("rationale", text()),
                    ],
                    &[],
                )];
                for p in q["parents"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter(|p| p["active"] == true)
                {
                    let mut parents = parent_list();
                    if let Shape::Array { contains, .. } = &mut parents {
                        *contains = vec![p["id"].clone()];
                        if let Some(prior) = p["parents"].as_array() {
                            contains.extend(prior.clone());
                        }
                    }
                    let proposal = obj(
                        vec![
                            ("binding", lit(&q["attention_binding"])),
                            ("identity", lit(&p["id"])),
                            ("revises", lit(&p["revision"])),
                            ("parents", parents),
                            ("meaning", text()),
                            (
                                "change",
                                union(vec![
                                    lit(adaptive::coding::project::desire::Change::Clarification),
                                    lit(adaptive::coding::project::desire::Change::Development),
                                ]),
                            ),
                            ("basis", lit(DesireBasis::Unknown)),
                            ("generator", lit(GENERATOR)),
                        ],
                        if p["revision"].is_null() {
                            &["revises"]
                        } else {
                            &[]
                        },
                    );
                    suggestions.push(obj(
                        vec![("kind", lit("public_desire")), ("proposal", proposal)],
                        &[],
                    ));
                }
                if ids.as_array().unwrap().len() >= 2 {
                    let revisions = std::iter::once(lit(Value::Null))
                        .chain(
                            q["guidance"]
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|g| lit(&g["revision"])),
                        )
                        .collect();
                    let relation = obj(
                        vec![
                            ("parents", array(choices(&ids), 2, 16, true)),
                            ("relationship", text()),
                            ("aspiration", text()),
                            ("discriminator", text()),
                            ("revises", union(revisions)),
                            ("catalysts", lit(json!([]))),
                            ("development", lit(Value::Null)),
                        ],
                        &["revises", "catalysts", "development"],
                    );
                    suggestions.push(obj(
                        vec![
                            ("kind", lit("relational")),
                            ("binding", lit(&q["attention_binding"])),
                            ("proposal", relation),
                        ],
                        &[],
                    ));
                }
                Ok(obj(
                    vec![
                        ("basis", lit(&q["basis"])),
                        ("generator", lit(GENERATOR)),
                        (
                            "response",
                            obj(
                                vec![
                                    ("protocol", lit(PROTOCOL)),
                                    ("state_root", lit(&q["state_root"])),
                                    ("attention_binding", lit(&q["attention_binding"])),
                                    ("proposals", array(union(suggestions), 0, 1, false)),
                                    ("common_event_forecasts", lit(json!({}))),
                                ],
                                &["common_event_forecasts"],
                            ),
                        ),
                    ],
                    &[],
                ))
            }
            _ => bail!("unknown model stage"),
        }
    }
    pub fn qualitative_model_contract(&self, stage: &str) -> Result<Value> {
        self.qualitative_model_contract_for_inquiry(stage, None)
    }
    pub fn qualitative_model_contract_for_inquiry(
        &self,
        stage: &str,
        focus: Option<EventId>,
    ) -> Result<Value> {
        ensure!(
            focus.is_none() || stage == "select",
            "inquiry focus only for selection"
        );
        let schema = self.qualitative_stage_shape(stage, focus)?;
        Ok(
            json!({"version":VERSION,"stage":stage,"state_root":self.qualitative_question_request()?["state_root"],"schema":schema,"interpretation":"Return an instance, not this schema. Arrays are JSON arrays even for one item. Object fields not listed are forbidden. optional permits omission, not null unless a null literal is allowed. Text bounds count UTF-8 bytes and require nonblank text. Literal values and IDs must be copied exactly. Union means any one complete listed variant.","native_constraints":["This is a bounded producer profile, not every general native variant. UserSupplied origin is operator-only and excluded from model responses.","Native semantic validation remains mandatory: active original parent lineage, exact current state/basis, distinct independent reviewer, no duplicate or unchanged claimed development, revision ancestry preserved.","Initial context and retrieval assertions are not empirical fulfillment. Relational development must be absent/null and catalysts absent/empty. Retaining aspiration while changing relationship is allowed.","Public desire meaning must differ, preserve identity and existing parent ancestry; its basis stays unknown. generator must match the enclosing submission.","Origin/source labels and benefit are fallible proposals, never authority. Review must judge plausibility and suitability. Unknown or absent answer source waits. Policy grants no action or spending authority.","Selection [] is abstention handled before the reader. At most one read and one reflection candidate in this prospective profile."]}),
        )
    }
    pub fn qualitative_check_model_response(&self, stage: &str, value: &Value) -> Result<Value> {
        self.qualitative_check_model_response_for_inquiry(stage, value, None)
    }
    pub fn qualitative_check_model_response_for_inquiry(
        &self,
        stage: &str,
        value: &Value,
        focus: Option<EventId>,
    ) -> Result<Value> {
        ensure!(
            focus.is_none() || stage == "select",
            "inquiry focus only for selection"
        );
        self.qualitative_stage_shape(stage, focus)?
            .check(value, "response")?;
        match stage {
            "inquiry" => {
                if !value["inquiry"].is_null() {
                    let _: inquiry::Inquiry = serde_json::from_value(value["inquiry"].clone())?;
                }
            }
            "select" => {
                let _: inquiry::InformationChoice = serde_json::from_value(value.clone())?;
            }
            "review" | "inquiry_review" => {
                let _: Review = serde_json::from_value(value.clone())?;
            }
            "reflect" | "next" => {
                let sub: Submission = serde_json::from_value(value.clone())?;
                self.qualitative_validate(&sub, None)?;
            }
            _ => bail!("unknown model stage"),
        }
        Ok(
            json!({"valid":true,"validation":"producer profile and native deserialization; native admission remains separate","contract":VERSION,"adopted":false,"executed":false}),
        )
    }
}
