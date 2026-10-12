//! Tier 1 head: logistic regression over the structured features (PRD §B8, §B12).
//!
//! Hand-rolled in pure Rust — no ML framework, no network, no RNG crate. The head is fit on the fit
//! slice with features standardized by fit-slice statistics (stored in the spec's bundle), a
//! declared L2 penalty, and full-batch gradient updates in a fixed order for a fixed number of
//! iterations. Only the common event gets a head, as the M4 row says.
//!
//! Coefficients live only in the bundle; nothing here writes a fitted parameter to the log or the
//! store.

use serde::{Deserialize, Serialize};

use crate::dataset::Row;
use crate::features::{FeatureSpec, FeatureVector};

/// The declared L2 penalty and iteration count of the Tier 1 head (fixed, so fit is deterministic).
pub const DEFAULT_L2: f64 = 1e-3;
pub const DEFAULT_ITERS: u32 = 500;
/// The learning rate of the full-batch updates.
pub const LEARNING_RATE: f64 = 0.5;

/// A fitted logistic head: weights, bias, and the fit settings.
///
/// B15: the standardization statistics are **not** here — they live in the feature spec
/// (`features.json`), which is the declared schema, and the learning rate is declared there too.
/// The head carries only what is fitted.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Head {
    pub weights: Vec<f64>,
    pub bias: f64,
    pub l2: f64,
    pub iters: u32,
}

fn sigmoid(z: f64) -> f64 {
    if z >= 0.0 {
        1.0 / (1.0 + (-z).exp())
    } else {
        let e = z.exp();
        e / (1.0 + e)
    }
}

/// Fit-slice standardization statistics: per-feature mean and standard deviation.
///
/// B15: these are computed from the fit slice and stored in the feature spec, not in the head.
pub fn standardize_stats(rows: &[Row], dim: usize) -> (Vec<f64>, Vec<f64>) {
    let n = rows.len().max(1) as f64;
    let mut mean = vec![0.0; dim];
    for r in rows {
        for (i, v) in r.features.values.iter().enumerate().take(dim) {
            mean[i] += v;
        }
    }
    for m in mean.iter_mut() {
        *m /= n;
    }
    let mut var = vec![0.0; dim];
    for r in rows {
        for (i, v) in r.features.values.iter().enumerate().take(dim) {
            let d = v - mean[i];
            var[i] += d * d;
        }
    }
    let mut std = vec![1.0; dim];
    for i in 0..dim {
        let s = (var[i] / n).sqrt();
        std[i] = if s > 1e-12 { s } else { 1.0 };
    }
    (mean, std)
}

/// Standardize a feature vector with the spec's declared statistics (B15).
fn standardize(spec: &FeatureSpec, x: &FeatureVector) -> Vec<f64> {
    x.values
        .iter()
        .enumerate()
        .map(|(i, v)| {
            let m = spec.standardize_mean.get(i).copied().unwrap_or(0.0);
            let s = spec.standardize_std.get(i).copied().unwrap_or(1.0);
            (v - m) / s
        })
        .collect()
}

/// Fit the Tier 1 head on the fit slice.
///
/// Rows with no resolved label are ignored. The update order is the row order (fixed), the number
/// of iterations is fixed, and the L2 penalty is declared — so the same rows give the same head.
pub fn fit_head(rows: &[Row], spec: &FeatureSpec, l2: f64, iters: u32) -> Head {
    let dim = spec.names.len();
    let labelled: Vec<&Row> = rows.iter().filter(|r| r.label.is_some()).collect();

    let mut head = Head {
        weights: vec![0.0; dim],
        bias: 0.0,
        l2,
        iters,
    };
    if labelled.is_empty() {
        return head;
    }

    // Pre-standardize the fit rows once, with the spec's declared statistics (B15).
    let xs: Vec<Vec<f64>> = labelled
        .iter()
        .map(|r| standardize(spec, &r.features))
        .collect();
    let ys: Vec<f64> = labelled
        .iter()
        .map(|r| if r.label == Some(true) { 1.0 } else { 0.0 })
        .collect();
    let n = xs.len() as f64;

    for _ in 0..iters {
        let mut grad_w = vec![0.0; dim];
        let mut grad_b = 0.0;
        for (x, &y) in xs.iter().zip(ys.iter()) {
            let z: f64 = head.bias
                + x.iter()
                    .zip(head.weights.iter())
                    .map(|(a, b)| a * b)
                    .sum::<f64>();
            let err = sigmoid(z) - y;
            for (g, xi) in grad_w.iter_mut().zip(x.iter()) {
                *g += err * xi;
            }
            grad_b += err;
        }
        let lr = spec.learning_rate;
        for (i, g) in grad_w.iter().enumerate() {
            // L2 penalty on the weights (not the bias).
            let g = g / n + l2 * head.weights[i];
            head.weights[i] -= lr * g;
        }
        head.bias -= lr * (grad_b / n);
    }
    head
}

/// The head's output for a feature vector: the controller's `raw p`, always.
pub fn predict(head: &Head, spec: &FeatureSpec, x: &FeatureVector) -> f64 {
    let xs = standardize(spec, x);
    let z: f64 = head.bias
        + xs.iter()
            .zip(head.weights.iter())
            .map(|(a, b)| a * b)
            .sum::<f64>();
    sigmoid(z)
}

/// B1: the head's output, refusing a feature vector whose length differs from the head's weights.
///
/// `predict` zips the vector against the weights, so a shorter vector silently drops the tail and a
/// longer one silently ignores it — a misaligned vector then prints a plausible number. Callers
/// that score a dataset against a bundle use this so a length mismatch is an error, not a score.
pub fn predict_checked(
    head: &Head,
    spec: &FeatureSpec,
    x: &FeatureVector,
) -> Result<f64, crate::LearnError> {
    if x.values.len() != head.weights.len() {
        return Err(crate::LearnError::Invalid(format!(
            "feature length {} does not match the head's {} weights",
            x.values.len(),
            head.weights.len()
        )));
    }
    Ok(predict(head, spec, x))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dataset::{Row, Side};
    use crate::features::spec_for_families;
    use tsr_domain::{ActionKind, EventId, Hash, TaskId};

    fn row(family: &str, values: Vec<f64>, label: bool) -> Row {
        Row {
            id: Hash::of(&[b"row"]),
            task: TaskId(Hash::of(&[b"t"])),
            commitment: EventId(Hash::of(&[b"c"])),
            index: 0,
            action: ActionKind::ProposePatch,
            family: family.to_string(),
            lineage: "l".to_string(),
            side: Side::Fit,
            features: FeatureVector { values },
            label: Some(label),
            evidence: Vec::new(),
            generator_raw: None,
            operator_raw: None,
            controller_logged: None,
        }
    }

    #[test]
    fn fit_twice_gives_the_same_head() {
        let spec = spec_for_families(&["calibrated".to_string()]);
        let dim = spec.names.len();
        let rows: Vec<Row> = (0..20)
            .map(|i| {
                let mut v = vec![0.0; dim];
                v[0] = 1.0;
                v[dim - 1] = i as f64 / 20.0;
                row("calibrated", v, i % 2 == 0)
            })
            .collect();
        let a = fit_head(&rows, &spec, DEFAULT_L2, DEFAULT_ITERS);
        let b = fit_head(&rows, &spec, DEFAULT_L2, DEFAULT_ITERS);
        assert_eq!(a, b, "fit is deterministic");
    }

    #[test]
    fn head_learns_a_separable_signal() {
        let spec = spec_for_families(&["calibrated".to_string()]);
        let dim = spec.names.len();
        // Feature 0 (family one-hot) is 1 for positives, 0 for negatives.
        let rows: Vec<Row> = (0..40)
            .map(|i| {
                let mut v = vec![0.0; dim];
                let pos = i % 2 == 0;
                v[0] = if pos { 1.0 } else { 0.0 };
                row("calibrated", v, pos)
            })
            .collect();
        let head = fit_head(&rows, &spec, DEFAULT_L2, DEFAULT_ITERS);
        let mut pos = vec![0.0; dim];
        pos[0] = 1.0;
        let mut neg = vec![0.0; dim];
        neg[0] = 0.0;
        let p_pos = predict(&head, &spec, &FeatureVector { values: pos });
        let p_neg = predict(&head, &spec, &FeatureVector { values: neg });
        assert!(
            p_pos > p_neg,
            "the head separates the signal: {p_pos} > {p_neg}"
        );
        assert!(p_pos > 0.5 && p_neg < 0.5);
    }

    #[test]
    fn unlabelled_rows_are_ignored() {
        let spec = spec_for_families(&["calibrated".to_string()]);
        let dim = spec.names.len();
        let mut rows: Vec<Row> = (0..10)
            .map(|i| {
                let mut v = vec![0.0; dim];
                v[0] = 1.0;
                row("calibrated", v, i % 2 == 0)
            })
            .collect();
        let head_a = fit_head(&rows, &spec, DEFAULT_L2, DEFAULT_ITERS);
        // Add an unlabelled row: the head is unchanged.
        let mut v = vec![0.0; dim];
        v[0] = 1.0;
        let mut r = row("calibrated", v, true);
        r.label = None;
        rows.push(r);
        let head_b = fit_head(&rows, &spec, DEFAULT_L2, DEFAULT_ITERS);
        assert_eq!(head_a, head_b);
    }
}
