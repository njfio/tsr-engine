//! Scoring: Brier, log loss, reliability bins, and the residual series.
//!
//! Every number here is a labeled proxy (Appendix A, rule 2): the residual prints as
//! `residual=…`, never as a TSR term. No calibrator is trained in this crate (M4); a
//! forecast's `calibrated` stays `None` and prints as `uncalibrated`.

use serde::{Deserialize, Serialize};
use tsr_domain::{Forecast, Forecaster, Proxy};

use crate::resolve::{forecaster_name, Resolution, ResolvedForecast};

/// The number of equal-width reliability bins (B7: ten).
pub const RELIABILITY_BINS: usize = 10;

/// One scored forecast: the probability, the outcome, and the forecaster that made it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ScoredForecast {
    pub forecaster: Forecaster,
    /// The raw probability, always recorded.
    pub p: f64,
    /// The outcome: 1.0 or 0.0. Censored and unresolved forecasts are never scored.
    pub outcome: f64,
    /// The task family, when the caller knows it.
    pub family: Option<String>,
    /// The bundle the forecast was made under, when the caller knows it.
    pub bundle: Option<String>,
}

impl ScoredForecast {
    /// Build a scored forecast from a resolved one, or `None` when it is censored/unresolved.
    pub fn from_resolved(
        r: &ResolvedForecast,
        family: Option<String>,
        bundle: Option<String>,
    ) -> Option<Self> {
        let outcome = r.resolution.outcome()?;
        Some(Self {
            forecaster: r.forecast.source.clone(),
            p: r.forecast.raw.get(),
            outcome,
            family,
            bundle,
        })
    }

    /// The residual: forecast minus observed, in `[-1, 1]` (PRD B7: "the residual — forecast
    /// minus return"). This is the Surprise's magnitude, and the Expansion Dilation proxy's
    /// ordinate. A forecast of 0.8 that came true has residual -0.2.
    pub fn residual(&self) -> f64 {
        self.p - self.outcome
    }

    /// The Brier contribution of this forecast: `(p - outcome)^2`.
    pub fn brier(&self) -> f64 {
        (self.p - self.outcome).powi(2)
    }

    /// The log loss contribution, clipped so a confident wrong forecast is finite.
    pub fn log_loss(&self) -> f64 {
        let p = self.p.clamp(LOG_LOSS_EPSILON, 1.0 - LOG_LOSS_EPSILON);
        -(self.outcome * p.ln() + (1.0 - self.outcome) * (1.0 - p).ln())
    }
}

/// The clip for log loss: a probability of exactly 0 or 1 would make the loss infinite, so
/// the score is computed at the nearest representable probability. The clip is named so the
/// Mirror can print it.
pub const LOG_LOSS_EPSILON: f64 = 1e-15;

/// A reliability bin: equal width `1/10`, with its count and the mean probability and
/// observed frequency inside it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ReliabilityBin {
    /// The bin's lower edge, `i / 10`.
    pub lower: f64,
    /// The bin's upper edge, `(i + 1) / 10`.
    pub upper: f64,
    /// How many scored forecasts fell in the bin.
    pub count: u32,
    /// The mean raw probability of the forecasts in the bin; `None` when empty.
    pub mean_p: Option<f64>,
    /// The observed frequency (mean outcome) in the bin; `None` when empty.
    pub observed: Option<f64>,
}

/// A score over a set of forecasts: Brier, log loss, and the reliability bins.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Score {
    /// How many forecasts were scored (censored and unresolved are excluded).
    pub n: u32,
    /// Mean Brier score over the set; `None` when `n == 0`.
    pub brier: Option<f64>,
    /// Mean log loss over the set; `None` when `n == 0`.
    pub log_loss: Option<f64>,
    /// Ten equal-width bins, always present, empty ones with `count: 0`.
    pub bins: Vec<ReliabilityBin>,
}

impl Score {
    /// Score a set of forecasts. Censored and unresolved forecasts must already be excluded
    /// by the caller (see [`ScoredForecast::from_resolved`]).
    pub fn of(forecasts: &[ScoredForecast]) -> Self {
        let n = forecasts.len() as u32;
        let brier = if n == 0 {
            None
        } else {
            Some(forecasts.iter().map(|f| f.brier()).sum::<f64>() / n as f64)
        };
        let log_loss = if n == 0 {
            None
        } else {
            Some(forecasts.iter().map(|f| f.log_loss()).sum::<f64>() / n as f64)
        };
        Self {
            n,
            brier,
            log_loss,
            bins: reliability_bins(forecasts),
        }
    }

    /// The Brier score as a labeled proxy, or `None` when there is nothing to score.
    pub fn brier_proxy(&self) -> Option<Proxy<f64>> {
        self.brier.map(|b| Proxy::new("brier", b))
    }

    /// The log loss as a labeled proxy, or `None` when there is nothing to score.
    pub fn log_loss_proxy(&self) -> Option<Proxy<f64>> {
        self.log_loss.map(|l| Proxy::new("log_loss", l))
    }
}

/// Ten equal-width bins over `[0, 1]`, with counts. A forecast at exactly 1.0 falls in the
/// last bin; a forecast at exactly 0.0 falls in the first.
pub fn reliability_bins(forecasts: &[ScoredForecast]) -> Vec<ReliabilityBin> {
    let mut bins: Vec<ReliabilityBin> = (0..RELIABILITY_BINS)
        .map(|i| ReliabilityBin {
            lower: i as f64 / RELIABILITY_BINS as f64,
            upper: (i + 1) as f64 / RELIABILITY_BINS as f64,
            count: 0,
            mean_p: None,
            observed: None,
        })
        .collect();
    let mut sums: Vec<(f64, f64)> = vec![(0.0, 0.0); RELIABILITY_BINS];
    for f in forecasts {
        let idx = bin_index(f.p);
        bins[idx].count += 1;
        sums[idx].0 += f.p;
        sums[idx].1 += f.outcome;
    }
    for (i, b) in bins.iter_mut().enumerate() {
        if b.count > 0 {
            b.mean_p = Some(sums[i].0 / b.count as f64);
            b.observed = Some(sums[i].1 / b.count as f64);
        }
    }
    bins
}

/// The bin index for a probability: `floor(p * 10)`, clamped to `[0, 9]`.
pub fn bin_index(p: f64) -> usize {
    let idx = (p * RELIABILITY_BINS as f64).floor() as i64;
    idx.clamp(0, RELIABILITY_BINS as i64 - 1) as usize
}

/// A score per forecaster: the forecaster's name and its [`Score`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ForecasterScore {
    pub forecaster: Forecaster,
    pub name: String,
    pub score: Score,
}

/// Score a set of forecasts per forecaster. The order is the order forecasters first appear.
pub fn score_per_forecaster(forecasts: &[ScoredForecast]) -> Vec<ForecasterScore> {
    let mut groups: Vec<(Forecaster, Vec<ScoredForecast>)> = Vec::new();
    for f in forecasts {
        match groups.iter_mut().find(|(k, _)| k == &f.forecaster) {
            Some((_, v)) => v.push(f.clone()),
            None => groups.push((f.forecaster.clone(), vec![f.clone()])),
        }
    }
    groups
        .into_iter()
        .map(|(forecaster, v)| ForecasterScore {
            name: forecaster_name(&forecaster),
            score: Score::of(&v),
            forecaster,
        })
        .collect()
}

/// A score per task family: the family name and its [`Score`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FamilyScore {
    pub family: String,
    pub score: Score,
}

/// Score a set of forecasts per task family. Forecasts with no family are grouped under
/// `"(unlabelled)"`.
pub fn score_per_family(forecasts: &[ScoredForecast]) -> Vec<FamilyScore> {
    let mut groups: Vec<(String, Vec<ScoredForecast>)> = Vec::new();
    for f in forecasts {
        let family = f.family.clone().unwrap_or_else(|| "(unlabelled)".into());
        match groups.iter_mut().find(|(k, _)| k == &family) {
            Some((_, v)) => v.push(f.clone()),
            None => groups.push((family, vec![f.clone()])),
        }
    }
    groups
        .into_iter()
        .map(|(family, v)| FamilyScore {
            score: Score::of(&v),
            family,
        })
        .collect()
}

/// A score per bundle: the bundle name and its [`Score`].
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct BundleScore {
    pub bundle: String,
    pub score: Score,
}

/// Score a set of forecasts per bundle. Forecasts with no bundle are grouped under
/// `"(unbundled)"`.
pub fn score_per_bundle(forecasts: &[ScoredForecast]) -> Vec<BundleScore> {
    let mut groups: Vec<(String, Vec<ScoredForecast>)> = Vec::new();
    for f in forecasts {
        let bundle = f.bundle.clone().unwrap_or_else(|| "(unbundled)".into());
        match groups.iter_mut().find(|(k, _)| k == &bundle) {
            Some((_, v)) => v.push(f.clone()),
            None => groups.push((bundle, vec![f.clone()])),
        }
    }
    groups
        .into_iter()
        .map(|(bundle, v)| BundleScore {
            score: Score::of(&v),
            bundle,
        })
        .collect()
}

/// One point of the residual series: the residual at a point in time, with the forecaster
/// that made the forecast.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResidualPoint {
    /// The time the forecast was recorded, as the log's `recorded_at`.
    pub at: String,
    pub forecaster: Forecaster,
    /// Outcome minus raw probability, in `[-1, 1]`.
    pub residual: f64,
}

/// The residual series over time for one family, labeled as the Expansion Dilation proxy.
///
/// The label is the engineering name `expansion_dilation`, never a TSR term: the curve is a
/// proxy for whether anything is learning, and the Mirror prints it as such.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ResidualSeries {
    pub family: String,
    /// The proxy's name, always `expansion_dilation`.
    pub proxy: String,
    /// The points, in the order the caller supplied them (log order).
    pub points: Vec<ResidualPoint>,
}

impl ResidualSeries {
    /// Build the series for one family from scored forecasts with their recorded times.
    pub fn of(family: &str, points: Vec<ResidualPoint>) -> Self {
        Self {
            family: family.to_string(),
            proxy: "expansion_dilation".into(),
            points,
        }
    }

    /// The mean residual, as a labeled proxy; `None` when the series is empty.
    pub fn mean(&self) -> Option<Proxy<f64>> {
        if self.points.is_empty() {
            return None;
        }
        let sum: f64 = self.points.iter().map(|p| p.residual).sum();
        Some(Proxy::new("residual", sum / self.points.len() as f64))
    }

    /// The residual series per family, from scored forecasts with their recorded times.
    pub fn per_family(points: Vec<(String, ResidualPoint)>) -> Vec<ResidualSeries> {
        let mut out: Vec<ResidualSeries> = Vec::new();
        for (family, point) in points {
            if let Some(s) = out.iter_mut().find(|s| s.family == family) {
                s.points.push(point);
            } else {
                out.push(ResidualSeries::of(&family, vec![point]));
            }
        }
        out
    }
}

/// The residual of a resolved forecast, as a labeled proxy, or `None` when it is not scored.
/// The residual is forecast minus observed (PRD B7), so a confident forecast that came true has
/// a negative residual.
pub fn residual_proxy(r: &ResolvedForecast) -> Option<Proxy<f64>> {
    let outcome = r.resolution.outcome()?;
    Some(Proxy::new("residual", r.forecast.raw.get() - outcome))
}

/// Whether a forecast is calibrated. In M2 nothing is: `calibrated` stays `None` and prints
/// as `uncalibrated` (Appendix A, rule 3).
pub fn calibration_label(f: &Forecast) -> String {
    match &f.calibrated {
        Some(c) => format!("p={} ({})", c.p.get(), c.calibrator),
        None => "uncalibrated".into(),
    }
}

/// The resolution's name, for the Mirror.
pub fn resolution_name(r: Resolution) -> &'static str {
    r.name()
}
