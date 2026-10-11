//! Isotonic calibration by pool-adjacent-violators (PRD §B8, §B12).
//!
//! The calibrator is fit over the head's outputs on the calibrate slice: the head's raw `p` is the
//! input, the resolved label is the target, and PAV produces a monotone step function. Breakpoints
//! live only in the bundle; nothing here writes a fitted parameter to the log or the store.

use serde::{Deserialize, Serialize};

/// A fitted isotonic calibrator: a monotone step function over the head's raw outputs.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Isotonic {
    /// `(x, y)` breakpoints, sorted by `x`, with `y` non-decreasing.
    pub breakpoints: Vec<(f64, f64)>,
}

/// Fit isotonic calibration by pool-adjacent-violators.
///
/// `xs` are the head's raw outputs on the calibrate slice, `ys` the resolved labels (0/1). The
/// result is a monotone step function: for any `x`, [`apply_isotonic`] returns the pooled mean of
/// the block whose `x`-range contains it.
pub fn fit_isotonic(xs: &[f64], ys: &[f64]) -> Isotonic {
    assert_eq!(xs.len(), ys.len(), "xs and ys must be the same length");
    if xs.is_empty() {
        return Isotonic {
            breakpoints: Vec::new(),
        };
    }

    // Sort by x (stable, so ties keep their input order — deterministic).
    let mut idx: Vec<usize> = (0..xs.len()).collect();
    idx.sort_by(|&a, &b| {
        xs[a]
            .partial_cmp(&xs[b])
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // PAV: each block carries (weight, sum_y, max_x). Merge while the previous block's mean exceeds
    // the current block's mean.
    struct Block {
        weight: f64,
        sum: f64,
        max_x: f64,
    }
    let mut blocks: Vec<Block> = Vec::new();
    for &i in &idx {
        blocks.push(Block {
            weight: 1.0,
            sum: ys[i],
            max_x: xs[i],
        });
        while blocks.len() >= 2 {
            let n = blocks.len();
            let prev_mean = blocks[n - 2].sum / blocks[n - 2].weight;
            let cur_mean = blocks[n - 1].sum / blocks[n - 1].weight;
            if prev_mean > cur_mean {
                let cur = blocks.pop().unwrap();
                let prev = blocks.last_mut().unwrap();
                prev.weight += cur.weight;
                prev.sum += cur.sum;
                prev.max_x = prev.max_x.max(cur.max_x);
            } else {
                break;
            }
        }
    }

    // The breakpoints: each block's pooled mean, keyed by the block's largest x.
    let breakpoints: Vec<(f64, f64)> = blocks.iter().map(|b| (b.max_x, b.sum / b.weight)).collect();
    Isotonic { breakpoints }
}

/// Apply the calibrator: the pooled mean of the block whose `x`-range contains `x`.
///
/// Below the first breakpoint the first block's mean is returned; above the last, the last block's.
pub fn apply_isotonic(iso: &Isotonic, x: f64) -> f64 {
    if iso.breakpoints.is_empty() {
        return x;
    }
    for (bx, by) in &iso.breakpoints {
        if x <= *bx {
            return *by;
        }
    }
    iso.breakpoints.last().unwrap().1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isotonic_is_monotone() {
        // A deliberately non-monotone input.
        let xs = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        let ys = vec![1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0];
        let iso = fit_isotonic(&xs, &ys);
        let mut prev = f64::NEG_INFINITY;
        for (_, y) in &iso.breakpoints {
            assert!(*y >= prev - 1e-12, "breakpoints are non-decreasing");
            prev = *y;
        }
        // Applying it is monotone in x.
        let mut prev_p = f64::NEG_INFINITY;
        for i in 0..=100 {
            let x = i as f64 / 100.0;
            let p = apply_isotonic(&iso, x);
            assert!(p >= prev_p - 1e-12, "apply is monotone");
            prev_p = p;
        }
    }

    #[test]
    fn fit_twice_gives_the_same_calibrator() {
        let xs = vec![0.2, 0.5, 0.5, 0.8, 0.9];
        let ys = vec![0.0, 1.0, 0.0, 1.0, 1.0];
        assert_eq!(fit_isotonic(&xs, &ys), fit_isotonic(&xs, &ys));
    }

    #[test]
    fn a_perfectly_calibrated_input_is_reproduced() {
        // Two blocks: x<=0.5 -> 0.0, x>0.5 -> 1.0.
        let xs = vec![0.1, 0.2, 0.3, 0.7, 0.8, 0.9];
        let ys = vec![0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let iso = fit_isotonic(&xs, &ys);
        assert_eq!(apply_isotonic(&iso, 0.2), 0.0);
        assert_eq!(apply_isotonic(&iso, 0.8), 1.0);
    }

    #[test]
    fn empty_input_is_identity() {
        let iso = fit_isotonic(&[], &[]);
        assert!(iso.breakpoints.is_empty());
        assert_eq!(apply_isotonic(&iso, 0.42), 0.42);
    }
}
