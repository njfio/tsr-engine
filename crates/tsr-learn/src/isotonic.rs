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
///
/// Tied `x` values are pooled **before** PAV (A13): rows that share an input are one block, so the
/// fitted calibrator does not depend on the order the rows arrive in. The merge loop then runs over
/// the pooled blocks until the sequence is non-decreasing (R1/F1).
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

    // PAV, two passes (R1/F1). Each block carries (weight, sum_y, max_x).
    struct Block {
        weight: f64,
        sum: f64,
        max_x: f64,
    }

    // Pass 1: pool every maximal run of EQUAL x into one block, before any merging. Pooling ties
    // first is what makes the result independent of the order the rows arrive in (A13): a tie can
    // never be split across two blocks, so no merge decision can depend on arrival order.
    let mut blocks: Vec<Block> = Vec::new();
    for &i in &idx {
        match blocks.last_mut() {
            Some(last) if last.max_x == xs[i] => {
                last.weight += 1.0;
                last.sum += ys[i];
            }
            _ => blocks.push(Block {
                weight: 1.0,
                sum: ys[i],
                max_x: xs[i],
            }),
        }
    }

    // Pass 2: merge any adjacent block whose mean exceeds the next block's mean, repeating until
    // the whole sequence is non-decreasing. Merging can expose a new violation to the left, so the
    // loop steps back one block after each merge.
    let mut i = 0;
    while i + 1 < blocks.len() {
        let left_mean = blocks[i].sum / blocks[i].weight;
        let right_mean = blocks[i + 1].sum / blocks[i + 1].weight;
        if left_mean > right_mean {
            let right = blocks.remove(i + 1);
            blocks[i].weight += right.weight;
            blocks[i].sum += right.sum;
            blocks[i].max_x = blocks[i].max_x.max(right.max_x);
            // The merged block may now violate against its new left neighbour.
            if i > 0 {
                i -= 1;
            }
        } else {
            i += 1;
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

    /// A tiny deterministic LCG so the seeded tests need no RNG crate.
    struct Lcg(u64);
    impl Lcg {
        fn next_u64(&mut self) -> u64 {
            self.0 = self
                .0
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.0
        }
        fn next_f64(&mut self) -> f64 {
            (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
        }
    }

    fn assert_non_decreasing(iso: &Isotonic, ctx: &str) {
        let mut prev = f64::NEG_INFINITY;
        for (x, y) in &iso.breakpoints {
            assert!(
                *y >= prev - 1e-12,
                "{ctx}: breakpoints must be non-decreasing, got {prev} then {y} at x={x}"
            );
            prev = *y;
        }
    }

    #[test]
    fn isotonic_is_monotone() {
        // A deliberately non-monotone input.
        let xs = vec![0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8];
        let ys = vec![1.0, 0.0, 1.0, 0.0, 1.0, 0.0, 1.0, 0.0];
        let iso = fit_isotonic(&xs, &ys);
        assert_non_decreasing(&iso, "isotonic_is_monotone");
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

    #[test]
    fn a13_pav_pools_tied_inputs_so_row_order_does_not_matter() {
        // A13: tied x values must be pooled before PAV, so the fitted calibrator does not depend on
        // the order the rows arrive in. Three rows share x = 0.5 with labels 1, 0, 0.
        let xs = vec![0.5, 0.5, 0.5];
        let ys = vec![1.0, 0.0, 0.0];
        let forward = fit_isotonic(&xs, &ys);

        // The same rows, shuffled: the labels arrive in a different order.
        let xs_shuffled = vec![0.5, 0.5, 0.5];
        let ys_shuffled = vec![0.0, 1.0, 0.0];
        let shuffled = fit_isotonic(&xs_shuffled, &ys_shuffled);

        assert_eq!(
            forward, shuffled,
            "A13: PAV must pool tied inputs, so the calibrator is independent of row order"
        );
        // The pooled mean of the tied block is 1/3, and it is the only breakpoint.
        assert_eq!(forward.breakpoints.len(), 1, "one pooled block for one x");
        assert!(
            (forward.breakpoints[0].1 - 1.0 / 3.0).abs() < 1e-12,
            "the tied block's pooled mean is 1/3, got {}",
            forward.breakpoints[0].1
        );
    }

    /// R1/F1: the hand-worked example from review b. Pooling ties first gives two blocks:
    /// (0.1, mean 1.0) and (0.2, mean 0.5). The first block's mean exceeds the second's, so PAV
    /// merges them into one block of weight 4, sum 2, mean 0.5 — the sequence stays non-decreasing.
    /// (Review b's stated expectation `[(0.1,1.0),(0.2,0.5)]` is itself non-monotone and cannot be
    /// the PAV result; the merge it demands in the same sentence yields the single block below.)
    #[test]
    fn r1_f1_hand_worked_example_is_non_decreasing() {
        let iso = fit_isotonic(&[0.1, 0.2, 0.2, 0.2], &[1.0, 1.0, 0.0, 0.0]);
        assert_eq!(
            iso.breakpoints,
            vec![(0.2, 0.5)],
            "R1/F1: fit_isotonic([0.1,0.2,0.2,0.2],[1,1,0,0]) must merge to [(0.2,0.5)]"
        );
        assert_non_decreasing(&iso, "r1_f1_hand_worked_example");
    }

    /// R1: the review-a repro. xs=[.1,.1,.7,.7,.7] ys=[1,0,1,0,0] must be non-decreasing.
    #[test]
    fn r1_review_a_repro_is_non_decreasing() {
        let iso = fit_isotonic(&[0.1, 0.1, 0.7, 0.7, 0.7], &[1.0, 0.0, 1.0, 0.0, 0.0]);
        assert_non_decreasing(&iso, "r1_review_a_repro");
        // The first block pools to 0.5; the second pools to 1/3 and must merge into it.
        assert_eq!(iso.breakpoints, vec![(0.7, 0.4)]);
    }

    /// R1/F1: seeded random inputs — the output is non-decreasing and invariant to row order.
    #[test]
    fn r1_f1_seeded_random_is_monotone_and_order_invariant() {
        for seed in 0..64u64 {
            let mut rng = Lcg(seed.wrapping_mul(0x9E3779B97F4A7C15).wrapping_add(1));
            let n = 1 + (rng.next_u64() % 200) as usize;
            let mut xs = Vec::with_capacity(n);
            let mut ys = Vec::with_capacity(n);
            for _ in 0..n {
                // A small grid of x values so ties are common.
                let x = (rng.next_u64() % 10) as f64 / 10.0;
                let y = if rng.next_f64() < 0.5 { 0.0 } else { 1.0 };
                xs.push(x);
                ys.push(y);
            }
            let iso = fit_isotonic(&xs, &ys);
            assert_non_decreasing(&iso, &format!("seed {seed}"));

            // Shuffle the same rows (Fisher-Yates with the same LCG) and refit.
            let mut order: Vec<usize> = (0..n).collect();
            for i in (1..n).rev() {
                let j = (rng.next_u64() % (i as u64 + 1)) as usize;
                order.swap(i, j);
            }
            let xs_shuffled: Vec<f64> = order.iter().map(|&i| xs[i]).collect();
            let ys_shuffled: Vec<f64> = order.iter().map(|&i| ys[i]).collect();
            let iso_shuffled = fit_isotonic(&xs_shuffled, &ys_shuffled);
            assert_eq!(
                iso, iso_shuffled,
                "R1/F1: seed {seed}: the calibrator must not depend on row order"
            );
        }
    }
}
