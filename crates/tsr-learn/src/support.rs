//! Support and the floor (PRD §B8, §B12).
//!
//! Support is the number of fit- and calibrate-slice episodes of the same family whose every
//! feature lies within a declared window of the query's — §B8's "same-family episodes within
//! feature range". Binary and one-hot features (width `0.0` in the spec) must match exactly; each
//! numeric feature has a width in the spec. Support is compared with the floor and reported; it
//! never weights, scales or orders anything.
//!
//! The bundle carries the support index (each training row's family and feature vector, no labels),
//! so support needs only the bundle.

use serde::{Deserialize, Serialize};

use crate::features::{FeatureSpec, FeatureVector};
use tsr_domain::Support;

/// One entry of the support index: a training row's family and feature vector, no label.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SupportEntry {
    pub family: String,
    pub features: FeatureVector,
}

/// The default support floor (§B12, open): `Tune::default().support_floor`.
pub fn default_support_floor() -> u32 {
    tsr_domain::Tune::default().support_floor
}

/// Count the fit- and calibrate-slice episodes of `query_family` whose every feature lies within
/// the spec's declared window of `query`.
///
/// A feature with width `0.0` (binary or one-hot) must match exactly; a numeric feature must lie
/// within `±width`. A length mismatch on any feature is not within range.
pub fn support(
    spec: &FeatureSpec,
    index: &[SupportEntry],
    query_family: &str,
    query: &FeatureVector,
) -> Support {
    let mut episodes: u32 = 0;
    for entry in index {
        if entry.family != query_family {
            continue;
        }
        if within_range(spec, &entry.features, query) {
            episodes += 1;
        }
    }
    Support { episodes }
}

/// Whether every feature of `candidate` lies within the spec's window of `query`.
fn within_range(spec: &FeatureSpec, candidate: &FeatureVector, query: &FeatureVector) -> bool {
    if candidate.values.len() != query.values.len() {
        return false;
    }
    for (i, (c, q)) in candidate.values.iter().zip(query.values.iter()).enumerate() {
        let width = spec.numeric_widths.get(i).copied().unwrap_or(0.0);
        if width <= 0.0 {
            // Binary or one-hot: must match exactly.
            if (c - q).abs() > 1e-12 {
                return false;
            }
        } else if (c - q).abs() > width {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::spec_for_families;

    fn entry(family: &str, values: Vec<f64>) -> SupportEntry {
        SupportEntry {
            family: family.to_string(),
            features: FeatureVector { values },
        }
    }

    #[test]
    fn support_counts_same_family_within_range() {
        let spec = spec_for_families(&["calibrated".to_string()]);
        let dim = spec.names.len();
        let mut q = vec![0.0; dim];
        q[0] = 1.0; // family one-hot
                    // inspections_so_far is at index (families + 6 actions) = 7 for one family.
        let insp = spec
            .names
            .iter()
            .position(|n| n == "inspections_so_far")
            .unwrap();
        q[insp] = 3.0;

        let mut near = vec![0.0; dim];
        near[0] = 1.0;
        near[insp] = 4.0; // within width 2.0
        let mut far = vec![0.0; dim];
        far[0] = 1.0;
        far[insp] = 9.0; // outside width 2.0
        let mut other_family = vec![0.0; dim];
        other_family[0] = 0.0; // not this family
        other_family[insp] = 3.0;

        let index = vec![
            entry("calibrated", near),
            entry("calibrated", far),
            entry("calibrated", other_family),
        ];
        let s = support(&spec, &index, "calibrated", &FeatureVector { values: q });
        assert_eq!(s.episodes, 1, "only the near, same-family entry counts");
    }

    #[test]
    fn binary_features_must_match_exactly() {
        let spec = spec_for_families(&["calibrated".to_string()]);
        let dim = spec.names.len();
        let mut q = vec![0.0; dim];
        q[0] = 1.0;
        let mut c = vec![0.0; dim];
        c[0] = 0.0; // family one-hot differs -> not within range
        let index = vec![entry("calibrated", c)];
        let s = support(&spec, &index, "calibrated", &FeatureVector { values: q });
        assert_eq!(s.episodes, 0);
    }

    #[test]
    fn default_floor_is_thirty() {
        assert_eq!(default_support_floor(), 30);
    }
}
