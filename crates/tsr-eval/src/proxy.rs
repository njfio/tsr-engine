//! Appendix A rule 2: "Every derived number is a labeled proxy. In code, a
//! `Proxy<T>` newtype whose `Display` prints the engineering name (`ev`,
//! `support`, `residual`, `resonance`); in the CLI, the same name. No output
//! ever reads 'Attraction = 0.73'."
//!
//! So every number this crate reports is a [`Proxy`]: it carries the
//! engineering name it is printed under, the plain-language proxy label that
//! says what it stands in for, and the value. `Display` prints the engineering
//! name and the value — never a bare marketing word.

use std::fmt;

/// A derived number that stands in for something we cannot measure directly.
///
/// `name` is the engineering name printed by `Display` (e.g. `completion_rate`).
/// `label` is the proxy label: the sentence that says what the number is a
/// proxy *for* and how it was derived. `value` is the number itself.
#[derive(Debug, Clone, PartialEq)]
pub struct Proxy<T> {
    name: &'static str,
    label: &'static str,
    value: T,
}

impl<T> Proxy<T> {
    pub const fn new(name: &'static str, label: &'static str, value: T) -> Self {
        Self { name, label, value }
    }

    /// The engineering name this proxy is printed under.
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// The plain-language proxy label (Appendix A rule 2).
    pub const fn label(&self) -> &'static str {
        self.label
    }

    pub fn value(&self) -> &T {
        &self.value
    }

    pub fn into_value(self) -> T {
        self.value
    }
}

impl<T: fmt::Display> fmt::Display for Proxy<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The engineering name, never a bare marketing word (Appendix A rule 2).
        write!(f, "{}={}", self.name, self.value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_prints_the_engineering_name_not_a_marketing_word() {
        let p = Proxy::new("completion_rate", "accepted / paired tasks", 0.5_f64);
        assert_eq!(p.to_string(), "completion_rate=0.5");
        assert_eq!(p.name(), "completion_rate");
        assert_eq!(p.label(), "accepted / paired tasks");
    }
}
