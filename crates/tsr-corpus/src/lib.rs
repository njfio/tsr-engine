//! # tsr-corpus — the corpus runners (PRD v0.3.1, §B9, §B11 M3)
//!
//! The corpus is built by running a repository's suite at two snapshots and keeping what changed.
//! Every one of those runs is a command, and every command in this engine runs in the sealed
//! sibling sandbox (`tsr exec`, §B4) — never on the host. This crate is the one place that
//! distinction is made, so the rest of the corpus tooling cannot get it wrong:
//!
//! - [`Runner`] is the trait: `run(snapshot, argv, limits) -> RunResult`.
//! - [`ScriptedRunner`] answers from a fixture script. It spawns nothing, so every test in this
//!   crate and in `tsr-batch` is deterministic and needs no container runtime.
//! - [`ExecRunner`] invokes the `tsr` binary on `PATH` as `tsr exec -- <argv>` with the snapshot as
//!   the working directory. It runs *nothing* from the repository on the host: the argv it builds
//!   is `["exec", "--", ...]`, and `tsr exec` refuses (exit 3) when no container runtime exists
//!   (§B4, §B10).
//!
//! Digests are blake3 hex of the captured bytes, so an empty stream still has a digest — the
//! digest of empty input — and two runs that differ in output differ in digest.

#![forbid(unsafe_code)]

pub mod mine;
pub mod mutants;
pub mod split;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------------------------
// Limits
// ---------------------------------------------------------------------------------------------

/// What a single run is allowed to consume: wall-clock milliseconds and captured output bytes.
///
/// `tsr-domain` has no equivalent type — its [`tsr_domain::Budget`] is a *task* budget in
/// generator tokens, tool minutes, wall-clock seconds, generator calls and patch bytes, which is a
/// different quantity at a different granularity. A run limit is per-command, so it is defined
/// here.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Limits {
    /// Wall-clock budget for one run, in milliseconds.
    pub wall_clock_ms: u64,
    /// Maximum bytes captured from each of stdout and stderr.
    pub max_output_bytes: usize,
}

impl Default for Limits {
    fn default() -> Self {
        Self {
            wall_clock_ms: 600_000,
            max_output_bytes: 1 << 20,
        }
    }
}

impl Limits {
    pub fn new(wall_clock_ms: u64, max_output_bytes: usize) -> Self {
        Self {
            wall_clock_ms,
            max_output_bytes,
        }
    }

    /// Truncate a captured stream to `max_output_bytes`, on a char boundary so the bytes stay
    /// valid UTF-8 when they are rendered.
    pub fn truncate<'a>(&self, bytes: &'a [u8]) -> &'a [u8] {
        if bytes.len() <= self.max_output_bytes {
            return bytes;
        }
        let mut end = self.max_output_bytes;
        // `bytes[end]` is the first byte past the cut; if it is a continuation byte, the cut
        // would split a character, so back up to the character's start.
        while end > 0 && (bytes[end] & 0b1100_0000) == 0b1000_0000 {
            end -= 1;
        }
        &bytes[..end]
    }
}

// ---------------------------------------------------------------------------------------------
// RunResult
// ---------------------------------------------------------------------------------------------

/// The outcome of one command run: its exit code, the blake3 hex digests of its captured stdout
/// and stderr, and how long it took.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunResult {
    pub exit: i32,
    pub stdout_digest: String,
    pub stderr_digest: String,
    pub duration_ms: u64,
}

impl RunResult {
    /// Build a result from captured bytes, hashing them. Empty input hashes to the digest of
    /// empty input, which is a real digest and not a sentinel.
    pub fn from_output(exit: i32, stdout: &[u8], stderr: &[u8], duration_ms: u64) -> Self {
        Self {
            exit,
            stdout_digest: digest(stdout),
            stderr_digest: digest(stderr),
            duration_ms,
        }
    }

    /// A result with no captured output — the shape a scripted fixture uses when only the exit
    /// code matters.
    pub fn exit_only(exit: i32) -> Self {
        Self::from_output(exit, b"", b"", 0)
    }

    pub fn succeeded(&self) -> bool {
        self.exit == 0
    }
}

/// blake3 hex of a byte string. The digest of empty input is a fixed, non-empty hex string.
pub fn digest(bytes: &[u8]) -> String {
    blake3::hash(bytes).to_hex().to_string()
}

// ---------------------------------------------------------------------------------------------
// Runner
// ---------------------------------------------------------------------------------------------

/// Run one command against one snapshot, under a limit, and report what happened.
///
/// Implementations must not run repository code on the host: the only sanctioned way to execute
/// anything is `tsr exec` in the sealed sibling sandbox (§B4).
pub trait Runner {
    fn run(&self, snapshot: &Path, argv: &[String], limits: &Limits) -> RunResult;
}

// ---------------------------------------------------------------------------------------------
// ScriptedRunner
// ---------------------------------------------------------------------------------------------

/// A key into a scripted run: the snapshot path and the exact argv.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ScriptKey {
    pub snapshot: PathBuf,
    pub argv: Vec<String>,
}

impl ScriptKey {
    pub fn new(snapshot: impl Into<PathBuf>, argv: &[String]) -> Self {
        Self {
            snapshot: snapshot.into(),
            argv: argv.to_vec(),
        }
    }
}

/// A [`Runner`] that answers from a recorded script. It spawns nothing, so it is deterministic and
/// usable in every test.
///
/// A script is a map from `(snapshot, argv)` to a recorded [`RunResult`], plus a default returned
/// for any key the script does not contain. The default is explicit — a test that forgets to
/// script a run gets the default, not a panic, and can assert on it.
#[derive(Clone, Debug)]
pub struct ScriptedRunner {
    script: BTreeMap<ScriptKey, RunResult>,
    default: RunResult,
    /// Every call made, in order, so a test can assert what was run and against which snapshot.
    calls: std::cell::RefCell<Vec<(ScriptKey, Limits)>>,
}

impl ScriptedRunner {
    /// A runner whose every answer is `default`.
    pub fn new(default: RunResult) -> Self {
        Self {
            script: BTreeMap::new(),
            default,
            calls: std::cell::RefCell::new(Vec::new()),
        }
    }

    /// A runner that answers exit 0 with no output for anything unscripted.
    pub fn ok() -> Self {
        Self::new(RunResult::exit_only(0))
    }

    /// Record the answer for one `(snapshot, argv)` pair. Later recordings for the same key win.
    pub fn script(&mut self, snapshot: impl Into<PathBuf>, argv: &[String], result: RunResult) {
        self.script.insert(ScriptKey::new(snapshot, argv), result);
    }

    /// Record the answer for one key.
    pub fn script_key(&mut self, key: ScriptKey, result: RunResult) {
        self.script.insert(key, result);
    }

    /// The default returned for unscripted keys.
    pub fn default_result(&self) -> &RunResult {
        &self.default
    }

    /// Every call this runner has served, in order.
    pub fn calls(&self) -> Vec<(ScriptKey, Limits)> {
        self.calls.borrow().clone()
    }

    /// How many times this runner was asked to run something.
    pub fn call_count(&self) -> usize {
        self.calls.borrow().len()
    }
}

impl Runner for ScriptedRunner {
    fn run(&self, snapshot: &Path, argv: &[String], limits: &Limits) -> RunResult {
        let key = ScriptKey::new(snapshot, argv);
        self.calls.borrow_mut().push((key.clone(), *limits));
        self.script
            .get(&key)
            .cloned()
            .unwrap_or_else(|| self.default.clone())
    }
}

// ---------------------------------------------------------------------------------------------
// ExecRunner
// ---------------------------------------------------------------------------------------------

/// The argv handed to the `tsr` binary for a command run: `tsr exec -- <argv>`.
///
/// Pure and public so the construction is testable without the binary present.
pub fn exec_argv(argv: &[String]) -> Vec<String> {
    let mut out = Vec::with_capacity(argv.len() + 2);
    out.push("exec".to_string());
    out.push("--".to_string());
    out.extend(argv.iter().cloned());
    out
}

/// A [`Runner`] that runs commands through `tsr exec` in the sealed sibling sandbox (§B4).
///
/// It invokes the `tsr` binary found on `PATH` as `tsr exec -- <argv>` with the snapshot as the
/// working directory, captures stdout and stderr, hashes them, and records the exit code and the
/// duration. It never runs repository code on the host: the repository's own commands are the
/// `argv` handed to `tsr exec`, and `tsr exec` refuses (exit 3) when no container runtime is
/// available rather than falling back to the host.
#[derive(Clone, Debug)]
pub struct ExecRunner {
    /// The binary to invoke. Defaults to `tsr` on `PATH`.
    program: PathBuf,
}

impl Default for ExecRunner {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecRunner {
    /// The runner that invokes `tsr` on `PATH`.
    pub fn new() -> Self {
        Self {
            program: PathBuf::from("tsr"),
        }
    }

    /// The runner that invokes a specific binary — used by the ignored integration test to point
    /// at a built `tsr` without installing it.
    pub fn with_program(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
        }
    }

    pub fn program(&self) -> &Path {
        &self.program
    }
}

impl Runner for ExecRunner {
    fn run(&self, snapshot: &Path, argv: &[String], limits: &Limits) -> RunResult {
        let started = Instant::now();
        let mut cmd = Command::new(&self.program);
        cmd.args(exec_argv(argv));
        cmd.current_dir(snapshot);
        // The sandbox owns containment; this only keeps the child from inheriting a terminal.
        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let output = match cmd.output() {
            Ok(o) => o,
            Err(e) => {
                // No `tsr` on PATH, or it could not be started: that is a failed run, not a panic.
                // The reason goes to stderr so the digest records it.
                let msg = format!(
                    "tsr-corpus: could not run `{}`: {e}\n",
                    self.program.display()
                );
                return RunResult::from_output(
                    127,
                    b"",
                    msg.as_bytes(),
                    elapsed_ms(started, limits),
                );
            }
        };

        let stdout = limits.truncate(&output.stdout);
        let stderr = limits.truncate(&output.stderr);
        let exit = output.status.code().unwrap_or(-1);
        RunResult::from_output(exit, stdout, stderr, elapsed_ms(started, limits))
    }
}

/// Milliseconds since `started`, clamped to the limit so a run that overran reports the limit it
/// was given rather than an unbounded number.
fn elapsed_ms(started: Instant, limits: &Limits) -> u64 {
    let ms = started.elapsed().as_millis() as u64;
    ms.min(limits.wall_clock_ms)
}

/// A convenience: the wall-clock limit as a [`Duration`].
pub fn wall_clock(limits: &Limits) -> Duration {
    Duration::from_millis(limits.wall_clock_ms)
}

// ---------------------------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn scripted_runner_returns_the_scripted_result() {
        let mut r = ScriptedRunner::new(RunResult::exit_only(1));
        let snap = Path::new("/snap/parent");
        let cmd = argv(&["cargo", "test"]);
        let recorded = RunResult::from_output(0, b"ok\n", b"", 42);
        r.script(snap, &cmd, recorded.clone());

        let got = r.run(snap, &cmd, &Limits::default());
        assert_eq!(got, recorded);
        assert_eq!(r.call_count(), 1);
    }

    #[test]
    fn scripted_runner_falls_back_to_the_default() {
        let r = ScriptedRunner::new(RunResult::exit_only(7));
        let got = r.run(
            Path::new("/snap"),
            &argv(&["cargo", "test"]),
            &Limits::default(),
        );
        assert_eq!(got.exit, 7);
        assert_eq!(got, RunResult::exit_only(7));
    }

    #[test]
    fn scripted_runner_distinguishes_snapshot_and_argv() {
        let mut r = ScriptedRunner::ok();
        let cmd = argv(&["cargo", "test"]);
        r.script(Path::new("/snap/parent"), &cmd, RunResult::exit_only(1));
        r.script(Path::new("/snap/commit"), &cmd, RunResult::exit_only(0));

        assert_eq!(
            r.run(Path::new("/snap/parent"), &cmd, &Limits::default())
                .exit,
            1
        );
        assert_eq!(
            r.run(Path::new("/snap/commit"), &cmd, &Limits::default())
                .exit,
            0
        );
        // Same snapshot, different argv: the default.
        assert_eq!(
            r.run(
                Path::new("/snap/parent"),
                &argv(&["cargo", "build"]),
                &Limits::default()
            )
            .exit,
            0
        );
    }

    #[test]
    fn digests_are_stable_and_differ_for_differing_output() {
        let a = RunResult::from_output(0, b"hello\n", b"", 1);
        let b = RunResult::from_output(0, b"hello\n", b"", 1);
        let c = RunResult::from_output(0, b"hello!\n", b"", 1);
        assert_eq!(a.stdout_digest, b.stdout_digest);
        assert_ne!(a.stdout_digest, c.stdout_digest);
        assert_eq!(a.stdout_digest.len(), 64);
    }

    #[test]
    fn empty_output_still_gets_the_digest_of_empty_input() {
        let r = RunResult::exit_only(0);
        assert_eq!(r.stdout_digest, digest(b""));
        assert_eq!(r.stderr_digest, digest(b""));
        assert_eq!(r.stdout_digest, r.stderr_digest);
        assert_eq!(r.stdout_digest.len(), 64);
    }

    #[test]
    fn stdout_and_stderr_digests_are_independent() {
        let r = RunResult::from_output(1, b"out", b"err", 0);
        assert_ne!(r.stdout_digest, r.stderr_digest);
        assert_eq!(r.stdout_digest, digest(b"out"));
        assert_eq!(r.stderr_digest, digest(b"err"));
    }

    #[test]
    fn exec_argv_prefixes_exec_dashdash() {
        assert_eq!(
            exec_argv(&argv(&["cargo", "test"])),
            argv(&["exec", "--", "cargo", "test"])
        );
        assert_eq!(exec_argv(&[]), argv(&["exec", "--"]));
        // A leading dash in the command is not mistaken for a flag of `tsr`.
        assert_eq!(
            exec_argv(&argv(&["--version"])),
            argv(&["exec", "--", "--version"])
        );
    }

    #[test]
    fn limits_truncate_on_a_char_boundary() {
        let l = Limits::new(1000, 4);
        assert_eq!(l.truncate(b"abcdef"), b"abcd");
        assert_eq!(l.truncate(b"abc"), b"abc");
        // "é" is two bytes (0xC3 0xA9); a 2-byte cap would split it, so the cut backs up to 1.
        let l = Limits::new(1000, 2);
        assert_eq!(l.truncate("aé".as_bytes()), b"a");
        // A 3-byte cap lands exactly on the end of "aé" and keeps it whole.
        let l = Limits::new(1000, 3);
        assert_eq!(l.truncate("aé".as_bytes()), "aé".as_bytes());
    }

    #[test]
    fn exec_runner_reports_a_missing_binary_as_a_failed_run() {
        let r = ExecRunner::with_program("/nonexistent/tsr-does-not-exist");
        let got = r.run(
            Path::new("."),
            &argv(&["cargo", "test"]),
            &Limits::default(),
        );
        assert_eq!(got.exit, 127);
        assert_ne!(got.stderr_digest, digest(b""));
        assert_eq!(got.stdout_digest, digest(b""));
    }
}
