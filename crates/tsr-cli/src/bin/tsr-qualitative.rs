//! Native zero-execution qualitative state entry point. No provider or worker.
use anyhow::{bail, Context, Result};
use std::{env, fs, path::Path};
use tsr_cli::engine::Engine;
fn main() -> Result<()> {
    let a: Vec<_> = env::args().collect();
    if a.len() < 3 {
        bail!("usage: tsr-qualitative DIRECTORY information-expectation|forecasted-retrieve|resolve-information|model-contract|check-model-response|content-digest|init|configure-retrieval|retrieve|exploration-policy|question-request|propose-inquiry|user-inquiry|retrieve-for-question|request|request-v2|preview|submit|submit-possibility|review|status|snapshot|attach|detach [JSON_FILE]")
    }
    let e = Engine::open(Path::new(&a[1]))?;
    let read = || -> Result<Vec<u8>> {
        let b = fs::read(a.get(3).context("JSON file required")?)?;
        anyhow::ensure!(b.len() <= 65536, "input bound");
        Ok(b)
    };
    let v = match a[2].as_str() {
        "content-digest" => serde_json::to_value(tsr_domain::Hash::of(&[&read()?]))?,
        "init" => e.qualitative_init(serde_json::from_slice(&read()?)?)?,
        "request" => e.qualitative_request()?,
        "user-inquiry" => {
            serde_json::to_value(e.qualitative_user_inquiry(serde_json::from_slice(&read()?)?)?)?
        }
        "exploration-policy" => serde_json::to_value(
            e.qualitative_set_exploration_policy(serde_json::from_slice(&read()?)?)?,
        )?,
        "model-contract" => {
            let v: serde_json::Value = serde_json::from_slice(&read()?)?;
            e.qualitative_model_contract_for_inquiry(
                v["stage"].as_str().context("stage")?,
                v.get("inquiry_focus")
                    .map(|x| serde_json::from_value(x.clone()))
                    .transpose()?,
            )?
        }
        "check-model-response" => {
            let v: serde_json::Value = serde_json::from_slice(&read()?)?;
            e.qualitative_check_model_response_for_inquiry(
                v["stage"].as_str().context("stage")?,
                &v["value"],
                v.get("inquiry_focus")
                    .map(|x| serde_json::from_value(x.clone()))
                    .transpose()?,
            )?
        }
        "information-expectation" => e.qualitative_information_expectation()?,
        "forecasted-retrieve" => serde_json::to_value(
            e.qualitative_forecasted_retrieve(serde_json::from_slice(&read()?)?)?,
        )?,
        "resolve-information" => {
            let v: serde_json::Value = serde_json::from_slice(&read()?)?;
            e.qualitative_resolve_information(
                serde_json::from_value(v["attempt"].clone())?,
                serde_json::from_value(v["receipt"].clone())?,
            )?
        }
        "question-request" => e.qualitative_question_request()?,
        "propose-inquiry" => {
            serde_json::to_value(e.qualitative_propose_inquiry(serde_json::from_slice(&read()?)?)?)?
        }
        "retrieve-for-question" => serde_json::to_value(
            e.qualitative_retrieve_for_question(serde_json::from_slice(&read()?)?)?,
        )?,
        "request-v2" => e.qualitative_request_v2()?,
        "submit-possibility" => serde_json::to_value(
            e.qualitative_submit_possibility(serde_json::from_slice(&read()?)?)?,
        )?,
        "retrieve" => {
            serde_json::to_value(e.qualitative_retrieve(serde_json::from_slice(&read()?)?)?)?
        }
        "configure-retrieval" => {
            let v: serde_json::Value = serde_json::from_slice(&read()?)?;
            e.qualitative_configure_retrieval(
                serde_json::from_value(v["state_root"].clone())?,
                serde_json::from_value(v["manifest"].clone())?,
            )?
        }
        "snapshot" => e.public_meaning_snapshot()?,
        "attach" => {
            serde_json::to_value(e.attach_public_meaning(serde_json::from_slice(&read()?)?)?)?
        }
        "detach" => {
            serde_json::to_value(e.detach_public_meaning(serde_json::from_slice(&read()?)?)?)?
        }
        "preview" => e.qualitative_preview(serde_json::from_slice(&read()?)?)?,
        "submit" => serde_json::to_value(e.qualitative_submit(serde_json::from_slice(&read()?)?)?)?,
        "review" => e.qualitative_review(serde_json::from_slice(&read()?)?)?,
        "status" => e.qualitative_status(serde_json::from_slice(&read()?)?)?,
        _ => bail!("unknown command"),
    };
    println!("{}", serde_json::to_string_pretty(&v)?);
    Ok(())
}
