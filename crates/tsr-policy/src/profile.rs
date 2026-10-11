//! The RATCHET protection-profile format (PRD §B5, §B4).
//!
//! A profile is the configuration `tsr-policy` loads per task. It names the read scope (path
//! prefixes and `rts` tools), the exact allowlisted test command ids, the per-task budgets and the
//! boundary actions that require an operator approval. It carries no scores, no reflections, no
//! repository text and no provider output: it is authority, not evidence.
//!
//! The format is TOML. [`Profile::from_toml_str`] parses a string; [`Profile::load`] reads a file.
//! Both reject an empty or duplicate test-command id, because an allowlist that cannot name a
//! command unambiguously cannot authorize one.

use std::collections::BTreeSet;
use std::path::Path;

use serde::{Deserialize, Serialize};

/// A RATCHET protection profile: the whole configuration `tsr-policy` gates against.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    /// Format version. Bumped when the shape changes incompatibly.
    pub version: u32,
    /// Human-readable profile name (e.g. `default`).
    pub name: String,
    /// What the task may read.
    pub read_scope: ReadScope,
    /// The exact allowlisted test commands, by id.
    pub tests: Vec<TestCommand>,
    /// Per-task budgets, reserved before dispatch.
    pub budgets: Budgets,
    /// Which boundary actions require an operator approval.
    pub approvals: ApprovalPolicy,
}

/// The read allowlist: path prefixes and `rts` tool names the task may inspect.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadScope {
    /// Allowed path prefixes (glob-ish; a candidate path must start with one of these).
    pub paths: Vec<String>,
    /// Allowed `rts` tool names.
    pub rts_tools: Vec<String>,
}

/// One allowlisted test command. The `id` is the exact token a candidate must name; `argv` is the
/// command that id expands to. The model never supplies `argv` — it supplies the id.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestCommand {
    /// The exact allowlisted command id (e.g. `cargo-test-parser`).
    pub id: String,
    /// The argv this id expands to (e.g. `["cargo", "test", "--offline", "-p", "parser"]`).
    pub argv: Vec<String>,
}

/// Per-task budgets (PRD §B5). Reserved before dispatch; on exhaustion the task stops with
/// `Stop(budget)`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Budgets {
    /// Generator tokens.
    pub generator_tokens: u64,
    /// Tool minutes.
    pub tool_minutes: u32,
    /// Wall-clock seconds.
    pub wall_clock_secs: u64,
    /// Generator calls.
    pub generator_calls: u32,
    /// Patch bytes.
    pub patch_bytes: u64,
}

/// Which boundary actions require an operator approval. `true` means an approval is required.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalPolicy {
    /// Promoting a diff out of the sandbox to the host checkout.
    pub promote_diff: bool,
    /// Running a command that is not on the allowlist.
    pub non_allowlisted_command: bool,
    /// Any network use.
    pub network: bool,
}

/// Why a profile could not be loaded or parsed.
#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    /// The file could not be read.
    #[error("could not read profile file {path}: {source}")]
    Io {
        /// The path that failed.
        path: String,
        /// The underlying I/O error.
        source: std::io::Error,
    },
    /// The TOML did not parse into a profile.
    #[error("could not parse profile TOML: {0}")]
    Toml(#[from] toml::de::Error),
    /// A test command id was empty.
    #[error("test command id at index {index} is empty")]
    EmptyTestId {
        /// The index of the offending entry.
        index: usize,
    },
    /// A test command id appeared more than once.
    #[error("duplicate test command id: {id}")]
    DuplicateTestId {
        /// The repeated id.
        id: String,
    },
    /// A read-scope path prefix was empty.
    #[error("read-scope path prefix at index {index} is empty")]
    EmptyPathPrefix {
        /// The index of the offending entry.
        index: usize,
    },
}

impl Profile {
    /// Parse a profile from a TOML string, rejecting empty or duplicate test-command ids.
    pub fn from_toml_str(s: &str) -> Result<Profile, ProfileError> {
        let profile: Profile = toml::from_str(s)?;
        profile.validate()?;
        Ok(profile)
    }

    /// Load a profile from a TOML file, rejecting empty or duplicate test-command ids.
    pub fn load(path: &Path) -> Result<Profile, ProfileError> {
        let text = std::fs::read_to_string(path).map_err(|source| ProfileError::Io {
            path: path.display().to_string(),
            source,
        })?;
        Profile::from_toml_str(&text)
    }

    /// Serialize this profile back to TOML.
    pub fn to_toml_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(self)
    }

    /// The allowlisted test command with this id, if any.
    pub fn test_command(&self, id: &str) -> Option<&TestCommand> {
        self.tests.iter().find(|t| t.id == id)
    }

    /// Reject empty or duplicate test-command ids, and empty read-scope path prefixes.
    fn validate(&self) -> Result<(), ProfileError> {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        for (index, test) in self.tests.iter().enumerate() {
            if test.id.trim().is_empty() {
                return Err(ProfileError::EmptyTestId { index });
            }
            if !seen.insert(test.id.as_str()) {
                return Err(ProfileError::DuplicateTestId {
                    id: test.id.clone(),
                });
            }
        }
        // An empty path prefix is a prefix of every path: it would widen the read scope to the whole
        // repository. Reject it at load, so a malformed profile cannot be constructed from TOML.
        for (index, prefix) in self.read_scope.paths.iter().enumerate() {
            if prefix.trim().is_empty() {
                return Err(ProfileError::EmptyPathPrefix { index });
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_TOML: &str = include_str!("../../../profiles/default.toml");

    #[test]
    fn default_profile_parses_and_round_trips() {
        let profile = Profile::from_toml_str(DEFAULT_TOML).expect("default.toml must parse");

        // Every field is exercised by the example.
        assert_eq!(profile.version, 1);
        assert_eq!(profile.name, "default");
        assert!(!profile.read_scope.paths.is_empty());
        assert!(!profile.read_scope.rts_tools.is_empty());
        assert!(profile.tests.len() >= 2);
        assert!(profile.budgets.generator_tokens > 0);
        assert!(profile.budgets.tool_minutes > 0);
        assert!(profile.budgets.wall_clock_secs > 0);
        assert!(profile.budgets.generator_calls > 0);
        assert!(profile.budgets.patch_bytes > 0);
        assert!(profile.approvals.promote_diff);
        assert!(profile.approvals.non_allowlisted_command);
        assert!(profile.approvals.network);

        // Round-trip: serialize then re-parse yields an equal profile.
        let text = profile.to_toml_string().expect("serialize");
        let again = Profile::from_toml_str(&text).expect("re-parse");
        assert_eq!(profile, again);
    }

    #[test]
    fn duplicate_test_id_is_rejected() {
        let toml = r#"
version = 1
name = "dup"
[read_scope]
paths = ["src/"]
rts_tools = ["search"]
[[tests]]
id = "cargo-test"
argv = ["cargo", "test"]
[[tests]]
id = "cargo-test"
argv = ["cargo", "test", "--offline"]
[budgets]
generator_tokens = 1
tool_minutes = 1
wall_clock_secs = 1
generator_calls = 1
patch_bytes = 1
[approvals]
promote_diff = true
non_allowlisted_command = true
network = true
"#;
        match Profile::from_toml_str(toml) {
            Err(ProfileError::DuplicateTestId { id }) => assert_eq!(id, "cargo-test"),
            other => panic!("expected DuplicateTestId, got {other:?}"),
        }
    }

    #[test]
    fn empty_test_id_is_rejected() {
        let toml = r#"
version = 1
name = "empty"
[read_scope]
paths = ["src/"]
rts_tools = ["search"]
[[tests]]
id = "  "
argv = ["cargo", "test"]
[budgets]
generator_tokens = 1
tool_minutes = 1
wall_clock_secs = 1
generator_calls = 1
patch_bytes = 1
[approvals]
promote_diff = true
non_allowlisted_command = true
network = true
"#;
        match Profile::from_toml_str(toml) {
            Err(ProfileError::EmptyTestId { index }) => assert_eq!(index, 0),
            other => panic!("expected EmptyTestId, got {other:?}"),
        }
    }

    #[test]
    fn test_command_lookup_by_id() {
        let profile = Profile::from_toml_str(DEFAULT_TOML).expect("default.toml must parse");
        let first = &profile.tests[0];
        let found = profile.test_command(&first.id).expect("id must resolve");
        assert_eq!(found, first);
        assert!(profile.test_command("not-allowlisted").is_none());
    }
}
