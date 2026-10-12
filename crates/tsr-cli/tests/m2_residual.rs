//! D11: one residual sign. The Filter's Surprise (written to the log and printed by the Mirror)
//! and the scorer's residual must be the same number — forecast minus observed — for the same
//! (forecast, observed) pair. p=0.8 that came true (outcome 1) is -0.200 in both.

use tsr_domain::{Forecast, ForecastId, Forecaster, Hash, Horizon, Probability, Support};
use tsr_forecast::ScoredForecast;
use tsr_ledger::Filter;

/// A forecast with a raw probability, from a fixed seed so the test is reproducible.
fn forecast(seed: u8, p: f64) -> Forecast {
    Forecast {
        id: ForecastId(Hash::of(&[b"m2-residual-forecast".as_slice(), &[seed]])),
        event: tsr_forecast::common_decision_event_id(),
        horizon: Horizon::TaskClose,
        source: Forecaster::Generator,
        raw: Probability::new(p).unwrap(),
        calibrated: None,
        support: Support { episodes: 3 },
        state_version: 0,
    }
}

/// D11: for the same (forecast, observed) pair the Filter's Surprise and the scorer's residual
/// agree, and both are forecast minus observed. p=0.8, outcome 1 -> -0.200 in both.
#[test]
fn d11_filter_and_score_residual_agree() {
    let f = forecast(1, 0.8);
    let observed = 1.0;

    // The Filter's Surprise, as it is written to the log and printed by the Mirror.
    let surprise = Filter::surprise_from_forecasts(std::slice::from_ref(&f), observed);
    assert_eq!(surprise.len(), 1, "one residual per forecast");
    let filter_residual = surprise[0].residual;

    // The scorer's residual for the same pair (score.rs: forecast minus observed).
    let scored = ScoredForecast {
        forecaster: f.source.clone(),
        p: f.raw.get(),
        outcome: observed,
        family: None,
        bundle: None,
    };
    let score_residual = scored.residual();

    // Both are forecast minus observed: 0.8 - 1.0 = -0.200.
    assert!(
        (filter_residual - -0.2).abs() < 1e-12,
        "the Filter's Surprise is forecast minus observed (-0.200), got {filter_residual}"
    );
    assert!(
        (score_residual - -0.2).abs() < 1e-12,
        "the scorer's residual is forecast minus observed (-0.200), got {score_residual}"
    );
    assert!(
        (filter_residual - score_residual).abs() < 1e-12,
        "the Filter's Surprise ({filter_residual}) and the scorer's residual \
         ({score_residual}) must agree"
    );
}

/// D11: the sign holds for a forecast that did not come true too. p=0.2, outcome 0 -> +0.200.
#[test]
fn d11_filter_and_score_residual_agree_on_a_miss() {
    let f = forecast(2, 0.2);
    let observed = 0.0;

    let surprise = Filter::surprise_from_forecasts(std::slice::from_ref(&f), observed);
    let filter_residual = surprise[0].residual;
    let score_residual = ScoredForecast {
        forecaster: f.source.clone(),
        p: f.raw.get(),
        outcome: observed,
        family: None,
        bundle: None,
    }
    .residual();

    assert!(
        (filter_residual - 0.2).abs() < 1e-12,
        "the Filter's Surprise is forecast minus observed (+0.200), got {filter_residual}"
    );
    assert!(
        (filter_residual - score_residual).abs() < 1e-12,
        "the Filter's Surprise ({filter_residual}) and the scorer's residual \
         ({score_residual}) must agree"
    );
}
