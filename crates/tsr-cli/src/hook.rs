//! `tsr hook` — the recorder (PRD §B4, path A).
//!
//! Claude Code calls this once per hook event with one JSON payload on stdin. M0 **records and
//! never blocks**: every path exits 0 and no `deny`/`block` decision is ever emitted. The gate
//! belongs to `tsr-policy` (M1); containment belongs to `tsr exec`.
//!
//! Tool-name → [`ActionKind`] (B4 line 218):
//! - `Read`, `Grep`, `Glob`, and `rts` calls → `Inspect`;
//! - `Bash` whose command matches the allowlisted test command → `RunApprovedTest`;
//! - `Edit`, `Write` → `ProposePatch`;
//! - the model's own turn (a `SessionStart`/prompt, or any other tool) → `Generate`;
//! - the `Stop` stage → `Stop`.
//!
//! Gated actions (`RunApprovedTest`, `ProposePatch`, `RequestApproval`) are recorded as
//! `Proposed` → `Committed`. The forecast is **not** written here: it enters the log through
//! `tsr forecast` *before* the tool call (B5, forecast-gated execution). A `Bash` test run that is
//! not a `tsr exec -- …` invocation records `Dispatched { in_sandbox: false }` — a host run was
//! observed; only `tsr exec` records a sandboxed dispatch.
//!
//! Nothing is matched (operator decision, M0 r5): a `tsr exec --candidate <64-hex> -- …` call names
//! its candidate, and the hook commits exactly that id and records no new proposal. Without
//! `--candidate` the hook records the `Bash` call as it does for any non-gated tool.

use crate::store_path;

/// The POSIX-shell command parser (see `shell.rs`). Declared here, with an explicit path, so the
/// crate root (`lib.rs`) stays byte-identical to the base commit.
#[path = "shell.rs"]
mod shell;
use serde_json::Value;
use tsr_domain::{
    ActionKind, Candidate, CandidateId, Decision, Event, EventId, EventKind, Forecast, Forecaster,
    Hash, Horizon, OutcomeDefId, Probability, Source, TaskId, Utility,
};
use tsr_store::Store;

/// The allowlisted test command prefix (B5: `cargo test --offline -p parser`). A `Bash` call whose
/// command starts with this is a `RunApprovedTest`; anything else is `Generate` (the model's turn).
pub const ALLOWLISTED_TEST_PREFIX: &str = "cargo test";

/// Split a shell command string into words, respecting single quotes, double quotes and backslash
/// escapes. Quotes are removed; a backslash escapes the next character outside single quotes. This
/// is the same word-splitting the shell applies to the command string, so the hook and `tsr exec`
/// (which is handed the argv after the shell split it) can agree on one encoding.
///
/// This is the *word* level only: it does not split on `;`/`|`/`&`, remove redirections or expand
/// `~`/`$VAR`/globs. Use [`shell::parse`] for the command string the hook receives.
pub fn shell_words(cmd: &str) -> Vec<String> {
    let mut words: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut has_word = false;
    let mut chars = cmd.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;
    while let Some(c) = chars.next() {
        if in_single {
            if c == '\'' {
                in_single = false;
            } else {
                cur.push(c);
            }
            continue;
        }
        if in_double {
            match c {
                '"' => in_double = false,
                '\\' => {
                    // Inside double quotes a backslash escapes only `"`, `\`, `$` and backtick; for
                    // our purposes (re-quoting the same words) escaping the next char is faithful.
                    if let Some(&n) = chars.peek() {
                        if matches!(n, '"' | '\\' | '$' | '`') {
                            cur.push(n);
                            chars.next();
                        } else {
                            cur.push('\\');
                        }
                    } else {
                        cur.push('\\');
                    }
                }
                _ => cur.push(c),
            }
            continue;
        }
        match c {
            '\'' => {
                in_single = true;
                has_word = true;
            }
            '"' => {
                in_double = true;
                has_word = true;
            }
            '\\' => {
                if let Some(n) = chars.next() {
                    cur.push(n);
                    has_word = true;
                }
            }
            c if c.is_whitespace() => {
                if has_word {
                    words.push(std::mem::take(&mut cur));
                    has_word = false;
                }
            }
            _ => {
                cur.push(c);
                has_word = true;
            }
        }
    }
    if has_word {
        words.push(cur);
    }
    words
}

/// Parse a Bash command string with POSIX shell grammar only: split into simple commands on `;`,
/// `&&`, `||`, `|`, `&` and newlines, and remove redirections. No expansion is evaluated — the hook
/// cannot know the shell that will run the command — so a word the shell would expand (glob, `~`,
/// `$VAR`, `${VAR}`, command/process substitution) is kept as written. See [`shell::parse`].
fn parse_command(cmd: &str) -> shell::Parsed {
    shell::parse(cmd)
}

/// The texts of a simple command's words, quotes removed. Used by the prefix/test-run logic, which
/// only needs the literal text of each word.
fn word_texts(words: &[shell::Word]) -> Vec<String> {
    words.iter().map(|w| w.text.clone()).collect()
}

/// Skip the leading `VAR=value` assignments, an `env` prefix (and any assignments after it), and the
/// command-prefix wrappers the shell grammar allows in front of a simple command — `time`, `nohup`,
/// `!`, the keywords `if`/`then`/`do`/`while`, `nice [-n N]` and `timeout [options] DURATION` —
/// returning the remaining words.
///
/// A trailing `env -u`, `nice -n` or `timeout -s` (an option with no argument) is a plain word list,
/// not a panic: every index is clamped to the slice length (defect 3).
fn command_words_after_prefix(words: &[shell::Word]) -> &[shell::Word] {
    let mut i = 0;
    while i < words.len() && is_assignment(&words[i].text) {
        i += 1;
    }
    if words.get(i).map(|w| w.text.as_str()) == Some("env") {
        i += 1;
        // `env` accepts its own options before the command word: `-i`, `-u NAME`, `--unset=NAME`,
        // `-C DIR`, `-S`, `--`, and `NAME=value` assignments. Skip them all.
        while i < words.len() {
            let w = words[i].text.as_str();
            if is_assignment(w) || w == "--" || w == "-i" || w == "-S" {
                i += 1;
            } else if w == "-u" || w == "-C" {
                // Option plus its argument; a trailing option with no argument is a plain word.
                i = (i + 2).min(words.len());
            } else if w.starts_with("--unset=") || w.starts_with("--chdir=") || w.starts_with('-') {
                i += 1;
            } else {
                break;
            }
        }
    }
    let mut rest = &words[i..];
    // The wrappers may nest (`time nice -n 5 timeout 60 cargo test`), so loop until none applies.
    loop {
        match rest {
            [w, ..] if matches!(w.text.as_str(), "time" | "nohup" | "!") => rest = &rest[1..],
            [w, ..] if matches!(w.text.as_str(), "if" | "then" | "do" | "while") => {
                rest = &rest[1..]
            }
            [w, ..] if w.text == "nice" => {
                let mut j = 1;
                if rest.get(j).map(|w| w.text.as_str()) == Some("-n") {
                    // `nice -n N`: a trailing `-n` with no argument is a plain word list (defect 3).
                    j = (j + 2).min(rest.len());
                } else if rest
                    .get(j)
                    .map(|w| w.text.starts_with('-'))
                    .unwrap_or(false)
                {
                    j += 1;
                }
                rest = &rest[j..];
            }
            [w, ..] if w.text == "timeout" => {
                // `timeout [options] DURATION command…`: skip the options, then the duration.
                // `-s SIGNAL` and `-k DURATION` take an argument; other options do not.
                let mut j = 1;
                while j < rest.len() {
                    let t = rest[j].text.as_str();
                    if t == "-s" || t == "-k" {
                        j = (j + 2).min(rest.len());
                    } else if t.starts_with('-') {
                        j += 1;
                    } else {
                        break;
                    }
                }
                if j < rest.len() {
                    j += 1; // the duration
                }
                rest = &rest[j..];
            }
            _ => break,
        }
    }
    rest
}

/// POSIX-quote a word: a word that is non-empty and contains only shell-safe characters is emitted
/// bare; otherwise it is wrapped in single quotes with any embedded `'` replaced by `'\''`.
fn quote_word(word: &str) -> String {
    let safe = !word.is_empty()
        && word
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || "_@%+=:,./-".contains(c));
    if safe {
        word.to_string()
    } else {
        let mut out = String::with_capacity(word.len() + 2);
        out.push('\'');
        for c in word.chars() {
            if c == '\'' {
                out.push_str("'\\''");
            } else {
                out.push(c);
            }
        }
        out.push('\'');
        out
    }
}

/// The one canonical encoding of an argv: each word POSIX-quoted and joined by single spaces. Both
/// the hook (which parses the command string with [`shell_words`]) and `tsr exec` (which is handed
/// the argv) call this, so `sh -c 'a; b'`, `sh -c "a; b"` and `sh -c a\;\ b` all produce one digest.
pub fn canonical_args(words: &[String]) -> String {
    words
        .iter()
        .map(|w| quote_word(w))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Whether a simple command (already parsed, redirections removed, expansions applied) is a test
/// run: after skipping leading `VAR=value` assignments and an `env` prefix (and any assignments after
/// it), the command begins with `cargo test` or `cargo nextest run`, skipping the wrappers the shell
/// grammar allows in front of a test run: `timeout [options] DURATION`, `nice [-n N]`, `time`, and a
/// `+toolchain` argument to cargo (`cargo +1.90.0 test`).
fn words_are_test_run(words: &[shell::Word]) -> bool {
    test_run_words(words).is_some()
}

/// The words of the test run inside a simple command, or `None` when it is not one: after skipping
/// leading `VAR=value` assignments and an `env` prefix (and any assignments after it), the wrappers
/// the shell grammar allows in front of a test run (`timeout [options] DURATION`, `nice [-n N]`,
/// `time`), the command begins with `cargo test` or `cargo nextest run`. The returned slice starts
/// at `cargo` and keeps a `+toolchain` argument (`cargo +1.90.0 test`), so the digest is the argv
/// `tsr exec` receives for the same spelling.
fn test_run_words(words: &[shell::Word]) -> Option<&[shell::Word]> {
    let rest = command_words_after_prefix(words);
    match rest {
        [cargo, test, ..] if cargo.text == "cargo" && test.text == "test" => Some(rest),
        [cargo, nextest, run, ..]
            if cargo.text == "cargo" && nextest.text == "nextest" && run.text == "run" =>
        {
            Some(rest)
        }
        // `cargo +toolchain test`: the `+toolchain` argument is kept in the digest.
        [cargo, toolchain, test, ..]
            if cargo.text == "cargo" && toolchain.text.starts_with('+') && test.text == "test" =>
        {
            Some(rest)
        }
        [cargo, toolchain, nextest, run, ..]
            if cargo.text == "cargo"
                && toolchain.text.starts_with('+')
                && nextest.text == "nextest"
                && run.text == "run" =>
        {
            Some(rest)
        }
        _ => None,
    }
}

/// The words of the test run in a `Bash` command string, or `None`: the first simple command that is
/// a test run, after the shell grammar has split the chain and removed redirections.
fn test_run_command_words(cmd: &str) -> Option<Vec<String>> {
    parse_command(cmd)
        .commands
        .iter()
        .find_map(|c| test_run_words(&c.words).map(word_texts))
}

/// Whether a `Bash` command is a test run: any simple command in the chain, after skipping leading
/// `VAR=value` assignments and an `env` prefix, begins with `cargo test` or `cargo nextest run`.
/// A chain (`cd crate && cargo test`, `a; cargo test`, `x | cargo test`, `cd crate⏎cargo test`) is a
/// test run if any simple command is one.
pub fn is_test_run_command(cmd: &str) -> bool {
    parse_command(cmd)
        .commands
        .iter()
        .any(|c| words_are_test_run(&c.words))
}

/// Whether a word is a leading `VAR=value` assignment (`^[A-Za-z_][A-Za-z0-9_]*=`).
fn is_assignment(word: &str) -> bool {
    let mut chars = word.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    for c in chars {
        if c == '=' {
            return true;
        }
        if !(c.is_ascii_alphanumeric() || c == '_') {
            return false;
        }
    }
    false
}

/// The common decision event (B7): every candidate forecasts `task_accepted_in_budget@v1`.
pub const DECISION_EVENT: &str = "task_accepted_in_budget@v1";

/// A parsed Claude Code hook payload. Every field is optional: unknown fields are ignored and
/// missing fields are tolerated, so a payload from another Claude Code version still records.
#[derive(Debug, Default, Clone)]
pub struct HookPayload {
    pub session_id: Option<String>,
    pub transcript_path: Option<String>,
    pub cwd: Option<String>,
    pub hook_event_name: Option<String>,
    pub tool_name: Option<String>,
    pub tool_input: Option<Value>,
    pub tool_response: Option<Value>,
    pub prompt: Option<String>,
    pub stop_hook_active: Option<bool>,
}

impl HookPayload {
    /// Parse a payload defensively. A non-object or malformed JSON yields an empty payload rather
    /// than an error: the hook records what it can and never blocks.
    pub fn parse(raw: &str) -> Self {
        let value: Value = match serde_json::from_str(raw) {
            Ok(v) => v,
            Err(_) => return Self::default(),
        };
        let s = |k: &str| value.get(k).and_then(|v| v.as_str()).map(str::to_string);
        Self {
            session_id: s("session_id"),
            transcript_path: s("transcript_path"),
            cwd: s("cwd"),
            hook_event_name: s("hook_event_name"),
            tool_name: s("tool_name"),
            tool_input: value.get("tool_input").cloned(),
            tool_response: value.get("tool_response").cloned(),
            prompt: s("prompt"),
            stop_hook_active: value.get("stop_hook_active").and_then(|v| v.as_bool()),
        }
    }

    /// The session id, or a stable fallback so a payload without one still records.
    pub fn session(&self) -> String {
        self.session_id
            .clone()
            .unwrap_or_else(|| "unknown-session".to_string())
    }

    /// The task this payload belongs to.
    pub fn task(&self) -> TaskId {
        store_path::task_for_session(&self.session())
    }

    /// The `Bash` command, if this is a Bash call.
    pub fn bash_command(&self) -> Option<String> {
        if self.tool_name.as_deref() != Some("Bash") {
            return None;
        }
        self.tool_input
            .as_ref()
            .and_then(|v| v.get("command"))
            .and_then(|v| v.as_str())
            .map(str::to_string)
    }
}

/// Map a tool name (and its input) to the typed action of B4.
pub fn action_for(tool_name: Option<&str>, bash_command: Option<&str>) -> ActionKind {
    match tool_name {
        Some("Read") | Some("Grep") | Some("Glob") | Some("rts") => ActionKind::Inspect,
        Some("Edit") | Some("Write") => ActionKind::ProposePatch,
        Some("Bash") => match bash_command {
            Some(cmd) if is_test_run_command(cmd) || is_tsr_exec_command(cmd) => {
                ActionKind::RunApprovedTest
            }
            _ => ActionKind::Generate,
        },
        _ => ActionKind::Generate,
    }
}

/// Whether an action is forecast-gated (B5).
pub fn is_gated(kind: ActionKind) -> bool {
    matches!(
        kind,
        ActionKind::RunApprovedTest | ActionKind::ProposePatch | ActionKind::RequestApproval
    )
}

/// The source of an event, by action: the model's own turn is `Generator`, the rest are `Rules`
/// (the hook is the deterministic recorder, not a generator).
fn source_for(kind: ActionKind) -> Source {
    match kind {
        ActionKind::Generate => Source::Generator,
        _ => Source::Rules,
    }
}

/// The argument string a tool call commits to: for `Bash` the command, for `Edit`/`Write` the file
/// path, otherwise the tool input (or the prompt). This is the single source of the string both the
/// hook and `tsr forecast` digest, so the two paths mint the same candidate for the same call.
pub fn action_args(payload: &HookPayload) -> String {
    if let Some(cmd) = payload.bash_command() {
        // A `tsr exec -- …` call commits to the *inner* command, not the wrapper: the forecast names
        // the command that will run, and `tsr exec` mints its candidate from the argv it is handed.
        // Without this the hook's candidate for `tsr exec -- cargo test …` would differ from the
        // forecast's for `cargo test …`, and the commitment would orphan the forecast.
        if let Some(argv) = tsr_exec_inner_argv(&cmd) {
            return canonical_args(&argv);
        }
        // A test run commits to the canonical argv of the test run itself, so a forecast written
        // `cargo test` matches the hook's `cd crate⏎cargo test`, `timeout 60 cargo test` and
        // `cargo +1.90.0 test` however the shell wrapped or quoted it.
        if let Some(words) = test_run_command_words(&cmd) {
            return canonical_args(&words);
        }
        // Any other Bash call commits to the canonical argv the shell would produce.
        return canonical_args(&shell_words(&cmd));
    }
    if let Some(v) = &payload.tool_input {
        if let Some(fp) = v.get("file_path").and_then(|x| x.as_str()) {
            return fp.to_string();
        }
        return serde_json::to_string(v).unwrap_or_default();
    }
    payload.prompt.clone().unwrap_or_default()
}

/// The one shared digest of a tool call's arguments. Both the hook and `tsr forecast` call this with
/// the same `(kind, args)` pair, so a candidate minted by either path has the same identity. For
/// `Bash` `args` is the command string; for `Edit`/`Write` it is the file path.
pub fn args_digest(kind: ActionKind, args: &str) -> Hash {
    let kind_tag = match kind {
        ActionKind::Inspect => "inspect",
        ActionKind::RunApprovedTest => "run_approved_test",
        ActionKind::Generate => "generate",
        ActionKind::ProposePatch => "propose_patch",
        ActionKind::RequestApproval => "request_approval",
        ActionKind::Stop => "stop",
    };
    Hash::of(&[kind_tag.as_bytes(), args.as_bytes()])
}

/// Mint the candidate for a gated action from its `(kind, args)` pair alone.
///
/// The candidate's identity is `(kind, args digest)` and **nothing else**: `Candidate::new` hashes
/// the parents into the id, so a candidate minted with the events present at forecast time would
/// differ from one minted with the events present at tool-call time, and the hook would never
/// recognise the forecast's proposal. Both `tsr forecast` and the hook go through this function, so
/// the two paths agree on identity and `pending_proposal` matches.
pub fn mint_candidate(kind: ActionKind, args: &str) -> anyhow::Result<Candidate> {
    Ok(Candidate::new(
        kind,
        source_for(kind),
        args_digest(kind, args),
        vec![],
        vec![],
    )?)
}

/// Whether a `Bash` command is a `tsr exec -- …` invocation: after skipping leading `VAR=value`
/// assignments and an `env` prefix (and any assignments after it), the first token's basename is
/// `tsr` (optionally with a path prefix, e.g. `./target/debug/tsr`) and the second token is `exec`.
/// Such a command records its own dispatch inside `tsr exec`, so the hook records nothing further
/// for it. A chain (`cd crate && tsr exec -- …`) is a `tsr exec` invocation if any simple command is
/// one.
pub fn is_tsr_exec_command(cmd: &str) -> bool {
    parse_command(cmd)
        .commands
        .iter()
        .any(|c| words_are_tsr_subcommand(&c.words, "exec"))
}

/// Whether a `Bash` command is a `tsr forecast …` invocation (after the same prefix-skipping, the
/// first token's basename is `tsr` and the second is `forecast`). `tsr forecast` records its own
/// `Proposed` and `Forecast`, so the hook records nothing for the call — before or after it.
/// Recording the call as a `Generate` turn put a PostToolUse `Unresolved` *between* the forecast and
/// the commitment it names, so the forecast's state version never matched (found by the first real
/// Claude Code session, 2026-10-11).
pub fn is_tsr_forecast_command(cmd: &str) -> bool {
    parse_command(cmd)
        .commands
        .iter()
        .any(|c| words_are_tsr_subcommand(&c.words, "forecast"))
}

/// Whether a simple command's words are a `tsr <sub>` invocation: after skipping leading `VAR=value`
/// assignments and an `env` prefix, the first word's basename is `tsr` and the second word is `sub`.
fn words_are_tsr_subcommand(words: &[shell::Word], sub: &str) -> bool {
    let rest = command_words_after_prefix(words);
    match rest {
        [tsr, s, ..] => {
            let base = tsr.text.rsplit('/').next().unwrap_or(&tsr.text);
            base == "tsr" && s.text == sub
        }
        _ => false,
    }
}

/// The candidate id named by a `tsr exec --candidate <64-hex> -- …` invocation, or `None` when `cmd`
/// is not one.
///
/// The hook parses the command string with POSIX shell grammar only (quotes, escapes, separators,
/// here-documents as data, comments); it does not evaluate `~`, `$VAR`, `${VAR}` or globs (it is not
/// the shell that runs the command). Nothing is matched: the agent names the candidate, and the hook
/// commits exactly that id (operator decision (c)).
pub fn tsr_exec_candidate(cmd: &str) -> Option<CandidateId> {
    let parsed = parse_command(cmd);
    for c in &parsed.commands {
        if !words_are_tsr_subcommand(&c.words, "exec") {
            continue;
        }
        let rest = command_words_after_prefix(&c.words);
        // rest[0] is the `tsr` token, rest[1] is `exec`.
        let args = &rest[2..];
        // Find the `--candidate <id>` pair (or the `--candidate=<id>` form) wherever it appears
        // before `--`.
        let mut i = 0;
        while i < args.len() {
            if args[i].text == "--" {
                break;
            }
            if args[i].text == "--candidate" {
                if let Some(id) = args
                    .get(i + 1)
                    .and_then(|w| Hash::from_hex(&w.text).ok())
                    .map(CandidateId)
                {
                    return Some(id);
                }
                return None;
            }
            if let Some(value) = args[i].text.strip_prefix("--candidate=") {
                return Hash::from_hex(value).ok().map(CandidateId);
            }
            i += 1;
        }
        return None;
    }
    None
}

/// Whether `cmd` is a `tsr exec` invocation that names a `--candidate` whose value is **not** a
/// 64-hex candidate id (r5c review defect 5).
///
/// A malformed `--candidate xyz` is a *named* run the hook cannot vouch for: it must be recorded as
/// an ordinary non-gated call, never as a gated test run. [`tsr_exec_candidate`] returns `None` for
/// both "no `--candidate`" and "malformed `--candidate`", so the caller cannot tell them apart from
/// its result alone; this predicate distinguishes the malformed case. A `tsr exec` with no
/// `--candidate` at all is not malformed — it is the ordinary `tsr exec -- …` wrapper, which records
/// its own dispatch.
pub fn tsr_exec_candidate_is_malformed(cmd: &str) -> bool {
    let parsed = parse_command(cmd);
    for c in &parsed.commands {
        if !words_are_tsr_subcommand(&c.words, "exec") {
            continue;
        }
        let rest = command_words_after_prefix(&c.words);
        let args = &rest[2..];
        let mut i = 0;
        while i < args.len() {
            if args[i].text == "--" {
                break;
            }
            if args[i].text == "--candidate" {
                // A `--candidate` with no value, or a value that is not 64 hex, is malformed.
                return args
                    .get(i + 1)
                    .and_then(|w| Hash::from_hex(&w.text).ok())
                    .is_none();
            }
            if let Some(value) = args[i].text.strip_prefix("--candidate=") {
                return Hash::from_hex(value).is_err();
            }
            i += 1;
        }
        return false;
    }
    false
}

/// The argv words after `--` of a `tsr exec -- …` invocation, or `None` when `cmd` is not one.
///
/// Used to give the hook's candidate for the wrapper call the same identity as the forecast's for the
/// inner command, so the commitment does not orphan the forecast.
pub fn tsr_exec_inner_argv(cmd: &str) -> Option<Vec<String>> {
    let parsed = parse_command(cmd);
    for c in &parsed.commands {
        if !words_are_tsr_subcommand(&c.words, "exec") {
            continue;
        }
        let rest = command_words_after_prefix(&c.words);
        let mut inner = &rest[2..];
        // Drop a leading `--candidate <id>` pair (or `--candidate=<id>`) and the `--` separator.
        if inner.first().map(|w| w.text.as_str()) == Some("--candidate") {
            inner = &inner[2.min(inner.len())..];
        } else if inner
            .first()
            .is_some_and(|w| w.text.starts_with("--candidate="))
        {
            inner = &inner[1..];
        }
        if inner.first().map(|w| w.text.as_str()) == Some("--") {
            inner = &inner[1..];
        }
        if inner.is_empty() {
            return None;
        }
        return Some(inner.iter().map(|w| w.text.clone()).collect());
    }
    None
}

thread_local! {
    /// A pinned timestamp for the self-test harness. `recorded_at` is hashed into every event id
    /// (`Event::new`), so a fixture can only be byte-reproducible if the clock is pinned. The
    /// production path leaves this unset and uses the real wall clock.
    static PINNED_CLOCK: std::cell::RefCell<Option<String>> = const { std::cell::RefCell::new(None) };
}

/// Run `f` with the clock pinned to `stamp`, so every event it records gets a deterministic id.
/// Used by the self-test harness; not used on the production path.
pub fn with_pinned_clock<T>(stamp: &str, f: impl FnOnce() -> T) -> T {
    PINNED_CLOCK.with(|c| *c.borrow_mut() = Some(stamp.to_string()));
    let out = f();
    PINNED_CLOCK.with(|c| *c.borrow_mut() = None);
    out
}

/// The current wall-clock time as an RFC 3339 string (provenance only; ordering is seq).
///
/// If the clock is pinned (see [`with_pinned_clock`]) the pinned stamp is returned instead, which
/// makes the self-test fixture reproducible.
pub fn now_rfc3339() -> String {
    if let Some(stamp) = PINNED_CLOCK.with(|c| c.borrow().clone()) {
        return stamp;
    }
    // No chrono dependency: seconds since the epoch, formatted as a fixed-width RFC 3339 UTC stamp.
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let days = secs / 86_400;
    let rem = secs % 86_400;
    let (h, m, s) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    let (y, mo, d) = civil_from_days(days as i64);
    format!("{y:04}-{mo:02}-{d:02}T{h:02}:{m:02}:{s:02}Z")
}

/// Days since 1970-01-01 → (year, month, day). Howard Hinnant's civil_from_days.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// The last event of a task, if any: the parent of the next event.
fn last_event(store: &Store, task: &TaskId) -> anyhow::Result<Option<Event>> {
    Ok(store.by_task(task)?.into_iter().next_back().map(|(_, e)| e))
}

/// The number of events in a task so far (its current state version).
/// The number of events in `task` — the index the next event will occupy in this task's log.
///
/// This must be the task's own count, not the store's `latest_seq`: the store is shared by every
/// session, so a global sequence number diverges from the per-task index as soon as a second task
/// exists, and a forecast recorded against it would miss the commitment's state version.
fn event_count(store: &Store, task: &TaskId) -> anyhow::Result<u64> {
    Ok(store.by_task(task)?.len() as u64)
}

/// Append one event, chaining it to the task's last event.
fn append_chained(
    store: &Store,
    task: &TaskId,
    source: Source,
    kind: EventKind,
) -> anyhow::Result<EventId> {
    let parents = last_event(store, task)?
        .map(|e| vec![e.id])
        .unwrap_or_default();
    let ev = Event::new(*task, parents, source, kind, now_rfc3339());
    store.append(&ev)?;
    Ok(ev.id)
}

/// Record a `SessionStart`: the task is opened (B4: SessionStart opens the task), and the session
/// identity is written to `<cwd>/.tsr/session` so `tsr forecast` / `tsr exec` resolve the same task.
pub fn record_session_start(store: &Store, payload: &HookPayload) -> anyhow::Result<EventId> {
    let task = payload.task();
    if payload.session_id.is_some() {
        store_path::remember_session(&payload.session());
    }
    append_chained(store, &task, Source::Rules, EventKind::TaskOpened { task })
}

/// Record a `Stop`: the task stops with a typed reason (B5). The hook never blocks.
pub fn record_stop(store: &Store, payload: &HookPayload) -> anyhow::Result<EventId> {
    let task = payload.task();
    let reason = if payload.stop_hook_active.unwrap_or(false) {
        "operator"
    } else {
        "done"
    };
    append_chained(
        store,
        &task,
        Source::Rules,
        EventKind::Stop {
            reason: reason.to_string(),
        },
    )
}

/// Record a tool call. Returns the events appended, in order.
///
/// The hook records only what happened. A gated action is recorded as `Proposed` → `Committed`; the
/// forecast is *not* written here — it must already be on record, written by `tsr forecast` before
/// the tool call. If a pending `Proposed` with the same candidate id already exists (the agent
/// forecast first), only `Committed` is appended. A `tsr exec --candidate <id> -- …` call commits
/// exactly that id and records no new proposal (operator decision (c)); a `RunApprovedTest` whose
/// command is any other `tsr exec -- …` invocation records nothing further (exec records its own
/// dispatch); any other `Bash` test run records `Dispatched { in_sandbox: false }` — a host run was
/// observed.
pub fn record_tool(store: &Store, payload: &HookPayload) -> anyhow::Result<Vec<EventId>> {
    // A `tsr forecast …` call is the forecast itself: it records its own events.
    if payload
        .bash_command()
        .as_deref()
        .is_some_and(is_tsr_forecast_command)
    {
        return Ok(Vec::new());
    }
    let task = payload.task();
    let mut kind = action_for(
        payload.tool_name.as_deref(),
        payload.bash_command().as_deref(),
    );
    let mut source = source_for(kind);
    let parents: Vec<EventId> = last_event(store, &task)?
        .map(|e| vec![e.id])
        .unwrap_or_default();

    // Identity is `(kind, args digest)` only — the same candidate `tsr forecast` minted for this
    // tool call, so a prior forecast's `Proposed` is recognised here.
    let mut args = action_args(payload);

    // (c) A `tsr exec --candidate <id> -- …` call names its candidate: the hook commits exactly that
    // id and records no new proposal. Nothing is matched — no pattern, no sidecar, no matcher.
    //
    // The named id is committed only if it is a candidate of *this* task that has a forecast and has
    // not been dispatched since that newest forecast (one forecast authorizes one run). Otherwise the
    // call is recorded as an ordinary non-gated call and one line says why: committing an id the hook
    // cannot vouch for would let an already-dispatched candidate run again.
    //
    // A `--candidate` whose value is not a 64-hex id is a *malformed* named run (defect 5): it is
    // recorded as an ordinary non-gated call with one stderr line, never as a gated test run. Without
    // this branch `tsr_exec_candidate` returns `None` for the malformed value, the block below is
    // skipped, and the call falls through to the `RunApprovedTest` path — committing a test run with
    // no forecast.
    if payload
        .bash_command()
        .as_deref()
        .is_some_and(tsr_exec_candidate_is_malformed)
    {
        eprintln!(
            "tsr: hook: --candidate value is not a 64-hex candidate id; recording the Bash call as \
             an ordinary non-gated call"
        );
        kind = ActionKind::Generate;
        source = source_for(kind);
        args = payload.bash_command().unwrap_or_default();
    } else if let Some(cid) = payload
        .bash_command()
        .as_deref()
        .and_then(tsr_exec_candidate)
    {
        match named_candidate_gate(store, &task, cid)? {
            Ok(()) => {
                let committed = Event::new(
                    task,
                    parents.clone(),
                    source,
                    EventKind::Committed {
                        candidate: cid,
                        decision: Decision::Selected {
                            value: cid,
                            utility: Utility {
                                p_success: 0.5,
                                value: 0.0,
                                cost: 0.0,
                                ev: 0.0,
                            },
                            propensity: 1.0,
                            evidence: parents.clone(),
                        },
                    },
                    now_rfc3339(),
                );
                let id = committed.id;
                store.append(&committed)?;
                return Ok(vec![id]);
            }
            Err(reason) => {
                eprintln!(
                    "tsr: hook: not committing --candidate {}: {reason}; recording the Bash call as \
                     an ordinary non-gated call",
                    cid.0.to_hex()
                );
                // The fallback must be a *non-gated* call: re-deriving the kind from the inner
                // command would mint the very candidate the gate just refused (a `tsr exec --candidate
                // <id> -- cargo test` would otherwise commit the test candidate anyway). Record the
                // model's own turn instead, keyed on the whole Bash command so its candidate differs
                // from the named one.
                kind = ActionKind::Generate;
                source = source_for(kind);
                args = payload.bash_command().unwrap_or_default();
            }
        }
    }

    let candidate = mint_candidate(kind, &args)?;
    let cid = candidate.id;

    let mut batch: Vec<Event> = Vec::new();
    let mut push = |kind: EventKind, source: Source, parents: Vec<EventId>| {
        let ev = Event::new(task, parents, source, kind, now_rfc3339());
        let id = ev.id;
        batch.push(ev);
        id
    };

    // 1. The candidate is proposed, unless it is already pending (the agent forecast first). The
    //    model's own turn is typed `Generate` (B4 line 218); every other action is a `Proposed`
    //    candidate. A `Generate` turn is recorded in the same `Proposed` → `Committed` shape as
    //    every other action: a bare `Generate` event followed by a `Committed` with no prior
    //    `Proposed` is rejected by `reconstruct` (lib.rs:97-101), so an ordinary session (a Bash
    //    `ls`, a `TodoWrite`) would never yield a passing record. `Generate` is not gated (B5), so
    //    no forecast is required and the invariant still holds.
    let already_proposed = pending_proposal(store, &task, cid)?;
    let proposed_id = if already_proposed {
        // The proposal is already on record; the commitment chains to the last event.
        parents
            .last()
            .copied()
            .unwrap_or_else(|| EventId(Hash::of(&[b"genesis"])))
    } else {
        push(
            EventKind::Proposed {
                candidate: candidate.clone(),
            },
            source,
            parents.clone(),
        )
    };

    // 2. The commitment. The decision is a deterministic selection (propensity 1.0). No forecast is
    //    written here: the hook never constructs one.
    let committed_id = push(
        EventKind::Committed {
            candidate: cid,
            decision: Decision::Selected {
                value: cid,
                utility: Utility {
                    p_success: 0.5,
                    value: 0.0,
                    cost: 0.0,
                    ev: 0.0,
                },
                propensity: 1.0,
                evidence: vec![proposed_id],
            },
        },
        source,
        vec![proposed_id],
    );

    // 3. A test run is dispatched. A `tsr exec -- …` command records its own dispatch inside
    //    `tsr exec`; any other `Bash` test run ran on the host, which is what is recorded.
    if kind == ActionKind::RunApprovedTest {
        let is_exec = payload
            .bash_command()
            .as_deref()
            .map(is_tsr_exec_command)
            .unwrap_or(false);
        if !is_exec {
            push(
                EventKind::Dispatched {
                    candidate: cid,
                    in_sandbox: false,
                },
                Source::Rules,
                vec![committed_id],
            );
        }
    }

    let seqs = store.append_all(&batch)?;
    debug_assert_eq!(seqs.len(), batch.len());
    Ok(batch.iter().map(|e| e.id).collect())
}

/// Whether the hook may commit the candidate `cid` a `tsr exec --candidate <id> -- …` call names.
///
/// `Ok(())` only when `cid` is a candidate of `task` (a `Proposed` for it exists), has a forecast (a
/// `Forecast` for it exists), and its newest `Forecast` comes after its newest `Dispatched` (or it
/// has never been dispatched) — **one forecast authorizes one run** (operator correction, r5c
/// re-arm). A candidate id is `(kind, args digest)` only, so the same command forecast again later
/// gets the same id and is dispatchable again; a second run on the same forecast is not. The hook is
/// the one that writes the commitment this PreToolUse authorizes, so it cannot require a prior
/// commitment — `exec::check_named_candidate` adds that condition when it runs the command, and the
/// two agree on which runs are authorized. `Err(reason)` names the first condition that fails, so
/// the caller can print one line and record the call as an ordinary non-gated call instead.
fn named_candidate_gate(
    store: &Store,
    task: &TaskId,
    cid: CandidateId,
) -> anyhow::Result<Result<(), String>> {
    let events = store.by_task(task)?;
    let mut proposed = false;
    let mut last_forecast: Option<usize> = None;
    let mut last_dispatched: Option<usize> = None;
    for (i, (_, e)) in events.iter().enumerate() {
        match &e.kind {
            EventKind::Proposed { candidate } if candidate.id == cid => proposed = true,
            EventKind::Forecast { candidate, .. } if *candidate == cid => last_forecast = Some(i),
            EventKind::Dispatched { candidate, .. } if *candidate == cid => {
                last_dispatched = Some(i)
            }
            _ => {}
        }
    }
    if !proposed {
        return Ok(Err("it is not a candidate of this task".to_string()));
    }
    let Some(forecast) = last_forecast else {
        return Ok(Err("it has no forecast".to_string()));
    };
    if last_dispatched.is_some_and(|d| d > forecast) {
        return Ok(Err("it is already dispatched".to_string()));
    }
    Ok(Ok(()))
}

/// Whether a `Proposed` for `cid` is pending: the newest `Proposed` for it comes after the newest
/// `Committed` for it (or there is no `Committed` yet).
///
/// Ordering, not a set: a candidate may be proposed, committed, and proposed again (a second run of
/// the same command). The second proposal is pending even though an earlier commitment exists, so
/// the hook records `Committed` only and the commitment lands at the state version its forecast
/// named.
fn pending_proposal(store: &Store, task: &TaskId, cid: CandidateId) -> anyhow::Result<bool> {
    let events = store.by_task(task)?;
    let mut last_proposed: Option<usize> = None;
    let mut last_committed: Option<usize> = None;
    for (i, (_, e)) in events.iter().enumerate() {
        match &e.kind {
            EventKind::Proposed { candidate } if candidate.id == cid => last_proposed = Some(i),
            EventKind::Committed { candidate, .. } if *candidate == cid => last_committed = Some(i),
            _ => {}
        }
    }
    Ok(match (last_proposed, last_committed) {
        (Some(p), Some(c)) => p > c,
        (Some(_), None) => true,
        _ => false,
    })
}

/// The exit code and log digest of a `PostToolUse` response, if the payload carries them.
fn exec_outcome(payload: &HookPayload) -> (Option<i32>, Option<Hash>) {
    let resp = match &payload.tool_response {
        Some(r) => r,
        None => return (None, None),
    };
    let exit_code = resp
        .get("exit_code")
        .or_else(|| resp.get("exitCode"))
        .and_then(|v| v.as_i64())
        .map(|c| c as i32);
    let log_digest = resp
        .get("stdout")
        .and_then(|v| v.as_str())
        .map(|s| Hash::of(&[s.as_bytes()]));
    (exit_code, log_digest)
}

/// Record a `PostToolUse`: reconcile the commitment the matching `PreToolUse` made, if any.
///
/// A `PostToolUse` **never** proposes or commits a candidate: the gated action was already recorded
/// by the `PreToolUse` (and, for a `tsr exec -- …` call, dispatched by `tsr exec` itself). This
/// function only reconciles what already happened:
///
/// - a `Bash` call whose command is a `tsr exec -- …` invocation records **nothing** — `tsr exec`
///   recorded the dispatch and the execution;
/// - any other `Bash` call records `Executed` (or `Unresolved`) for the candidate the matching
///   `PreToolUse` committed: the most recent `Committed` with that candidate id that has no
///   `Executed`/`Unresolved` after it;
/// - if there is no such commitment, it records `Unresolved { reason: "post-tool with no prior
///   commitment" }` — never `Proposed` or `Committed`;
/// - a `PostToolUse` for any other tool (`Read`/`Edit`/`Write`/…) records nothing new.
pub fn record_post_tool(store: &Store, payload: &HookPayload) -> anyhow::Result<Vec<EventId>> {
    let task = payload.task();

    // Only a `Bash` call has an execution to reconcile; a `tsr exec -- …` call recorded its own
    // dispatch and execution inside `tsr exec`, and a `tsr forecast …` call is the forecast itself.
    // Everything else records nothing here.
    let cmd = match payload.bash_command() {
        Some(cmd) if !is_tsr_exec_command(&cmd) && !is_tsr_forecast_command(&cmd) => cmd,
        _ => return Ok(Vec::new()),
    };

    let kind = action_for(payload.tool_name.as_deref(), Some(&cmd));
    let cid = mint_candidate(kind, &action_args(payload))?.id;

    let parents = last_event(store, &task)?
        .map(|e| vec![e.id])
        .unwrap_or_default();
    let (exit_code, log_digest) = exec_outcome(payload);
    let event_kind = match committed_without_outcome(store, &task, cid)? {
        Some(cid) => match exit_code {
            Some(code) => EventKind::Executed {
                candidate: cid,
                exit_code: Some(code),
                log_digest,
                duration_ms: 0,
            },
            None => EventKind::Unresolved {
                candidate: cid,
                reason: "no exit code in the hook response".to_string(),
            },
        },
        // No commitment to reconcile: record the truth — an unresolved post-tool observation — and
        // never invent a gated action.
        None => EventKind::Unresolved {
            candidate: cid,
            reason: "post-tool with no prior commitment".to_string(),
        },
    };
    let ev = Event::new(task, parents, Source::Rules, event_kind, now_rfc3339());
    let id = ev.id;
    store.append(&ev)?;
    Ok(vec![id])
}

/// The candidate of the most recent `Committed` for `cid` that has no `Executed`/`Unresolved` after
/// it, or `None` when there is no such commitment.
///
/// Ordering, not a set: a candidate may be committed, executed, and committed again (a second run of
/// the same command). The second commitment is unreconciled even though an earlier `Executed` exists
/// for the same id, so the second `PostToolUse` reconciles the second commitment.
fn committed_without_outcome(
    store: &Store,
    task: &TaskId,
    cid: CandidateId,
) -> anyhow::Result<Option<CandidateId>> {
    let events = store.by_task(task)?;
    let mut pending: Option<CandidateId> = None;
    for (_, e) in &events {
        match &e.kind {
            EventKind::Committed { candidate, .. } if *candidate == cid => pending = Some(cid),
            EventKind::Executed { candidate, .. } | EventKind::Unresolved { candidate, .. }
                if *candidate == cid =>
            {
                pending = None;
            }
            _ => {}
        }
    }
    Ok(pending)
}

/// The `tsr hook` entry point. Reads one payload from `input`, records it, and returns the events
/// appended. Never returns an error for a malformed payload: M0 records and never blocks.
pub fn run(store: &Store, stage: &str, input: &str) -> anyhow::Result<Vec<EventId>> {
    let payload = HookPayload::parse(input);
    // Leave the session identity where `tsr forecast` and `tsr exec` can find it: the hook cannot
    // set the environment of the tool call Claude Code spawns, so `<cwd>/.tsr/session` is the
    // channel that makes the CLI resolve the same task the hook records into. Written on every
    // stage so it tracks the live session. Best-effort, never blocking.
    if payload.session_id.is_some() {
        store_path::remember_session(&payload.session());
    }
    match stage {
        "SessionStart" => Ok(vec![record_session_start(store, &payload)?]),
        "Stop" => Ok(vec![record_stop(store, &payload)?]),
        "PreToolUse" => record_tool(store, &payload),
        "PostToolUse" => record_post_tool(store, &payload),
        // An unknown stage is recorded as the model's own turn rather than dropped.
        _ => record_tool(store, &payload),
    }
}

/// The candidate id a payload would produce, without recording it (used by tests).
pub fn candidate_for(payload: &HookPayload) -> anyhow::Result<CandidateId> {
    let kind = action_for(
        payload.tool_name.as_deref(),
        payload.bash_command().as_deref(),
    );
    candidate_for_action(kind, &action_args(payload))
}

/// The candidate id a `(kind, args)` pair would produce, without recording it. `tsr forecast` and
/// the hook both go through this, so the two paths agree on identity.
pub fn candidate_for_action(kind: ActionKind, args: &str) -> anyhow::Result<CandidateId> {
    Ok(mint_candidate(kind, args)?.id)
}

/// Parse the `--kind` of `tsr forecast`: the three gated actions of B5.
pub fn parse_action_kind(s: &str) -> Option<ActionKind> {
    match s {
        "run_approved_test" => Some(ActionKind::RunApprovedTest),
        "propose_patch" => Some(ActionKind::ProposePatch),
        "request_approval" => Some(ActionKind::RequestApproval),
        _ => None,
    }
}

/// Parse the `--event` of `tsr forecast`: a 64-hex candidate id.
pub fn parse_candidate_id(s: &str) -> Option<CandidateId> {
    Hash::from_hex(s).ok().map(CandidateId)
}

/// The forecaster named by `--source`.
fn forecaster_for(source: &str) -> Forecaster {
    match source {
        "controller" => Forecaster::Controller,
        "operator" => Forecaster::Operator,
        _ => Forecaster::Generator,
    }
}

/// Append a `Forecast` for `cid` at the state version the next event will occupy.
///
/// `state_version` is the number of events in the task *after* the forecast itself is appended — the
/// index the next event (the commitment) will occupy, which is what the M0 invariant checks.
fn append_forecast(
    store: &Store,
    task: &TaskId,
    cid: CandidateId,
    p: f64,
    source: &str,
) -> anyhow::Result<EventId> {
    let state_version = event_count(store, task)? + 1;
    let forecast = Forecast::new(
        OutcomeDefId(Hash::of(&[DECISION_EVENT.as_bytes()])),
        Horizon::ThisDispatch,
        forecaster_for(source),
        Probability::new(p)?,
        state_version,
    );
    append_chained(
        store,
        task,
        Source::Rules,
        EventKind::Forecast {
            forecast,
            candidate: cid,
        },
    )
}

/// The canonical form of a `tsr forecast --args` string: the same canonical argv the hook and
/// `tsr exec` digest. `--args` is a command string, so it is parsed with POSIX shell grammar and
/// re-quoted word by word, exactly as the hook does for a `Bash` command. A forecast written
/// `sh -c "a; b"` therefore names the same candidate as the hook's `sh -c 'a; b'`.
pub fn canonical_forecast_args(args: &str) -> String {
    canonical_args(&shell_words(args))
}

/// Record a forecast for a gated action, before the tool call (B5, B7).
///
/// This is how a forecast enters the log: `tsr forecast` mints the candidate exactly as the hook
/// would for the same tool call (same `ActionKind`, same args digest), appends `Proposed` unless a
/// pending proposal with that id already exists, then appends `Forecast { candidate, state_version }`
/// where `state_version` is the index the next event will occupy — the number of events in the task
/// after the forecast itself is appended. That is the version the commitment will occupy, which is
/// what the M0 invariant checks.
///
/// `args` is canonicalised through [`canonical_forecast_args`] before the digest, so a forecast
/// written with different quoting than the hook's command string still mints the same candidate.
pub fn record_forecast_for_action(
    store: &Store,
    task: &TaskId,
    kind: ActionKind,
    args: &str,
    p: f64,
    source: &str,
) -> anyhow::Result<CandidateId> {
    let args = canonical_forecast_args(args);
    let candidate = mint_candidate(kind, &args)?;
    let cid = candidate.id;

    // Append `Proposed` unless a pending proposal with this id already exists.
    if !pending_proposal(store, task, cid)? {
        append_chained(
            store,
            task,
            source_for(kind),
            EventKind::Proposed { candidate },
        )?;
    }

    append_forecast(store, task, cid, p, source)?;
    // The candidate id is what the agent names on `tsr exec --candidate <id>`: print it, not the
    // forecast event id (operator decision (a)).
    Ok(cid)
}

/// Record a forecast for an already-proposed candidate (`tsr forecast --event <id>`).
///
/// The candidate is not re-minted: the id names a `Proposed` already on record. The forecast is
/// appended at the state version the next event will occupy, exactly as the `--kind/--args` path.
pub fn record_forecast_for_candidate(
    store: &Store,
    task: &TaskId,
    cid: CandidateId,
    p: f64,
    source: &str,
) -> anyhow::Result<CandidateId> {
    append_forecast(store, task, cid, p, source)?;
    Ok(cid)
}

/// The M0 self-test harness: drive realistic Claude Code hook payloads through the hook and write
/// `fixtures/m0/session.events.jsonl`.
///
/// The sequence is the honest one the milestone names: a task opened, two inspections, a forecast
/// before each gated action, a test run through `tsr exec` in the real container, a patch proposal,
/// a second run, a stop. Nothing is fabricated: the forecast is written by the forecast path and the
/// dispatch by `tsr exec`. With no container runtime reachable the harness refuses (exit 3) and
/// writes nothing.
pub fn selftest(
    out: Option<&std::path::Path>,
    update_frozen_fixture: bool,
) -> anyhow::Result<std::path::PathBuf> {
    // `recorded_at` is hashed into every event id, so the fixture is only reproducible if the
    // clock is pinned. The production hook path is untouched and keeps real wall-clock time.
    with_pinned_clock(SELFTEST_RECORDED_AT, || {
        selftest_inner(out, update_frozen_fixture)
    })
}

/// The fixed provenance stamp the self-test fixture is recorded under. Any change to this value
/// changes every event id in `fixtures/m0/session.events.jsonl`.
pub const SELFTEST_RECORDED_AT: &str = "2026-01-01T00:00:00Z";

/// The argv the self-test runs in the container: `cargo test --offline -p parser`.
const SELFTEST_TEST_ARGV: [&str; 5] = ["cargo", "test", "--offline", "-p", "parser"];
/// The full command string the forecast and the hook payload name. Both gated runs use it: a
/// candidate id is `(kind, args digest)` only, so the second run names the same id as the first and
/// is authorized by its own fresh forecast — exactly as `fixtures/m0` runs one candidate twice.
const SELFTEST_TEST_CMD: &str = "cargo test --offline -p parser";

/// Send the hook what Claude Code sends around a `tsr forecast …` Bash call: the PreToolUse before
/// it, or the PostToolUse after it with a Bash tool response as Claude Code shapes it (stdout,
/// stderr, interrupted — no exit code).
fn forecast_call_hooks(
    store: &Store,
    session: &str,
    stage: &str,
    kind: ActionKind,
    args: &str,
    p: f64,
) -> anyhow::Result<()> {
    let kind_flag = match kind {
        ActionKind::RunApprovedTest => "run_approved_test",
        ActionKind::ProposePatch => "propose_patch",
        _ => "request_approval",
    };
    let command =
        format!("tsr forecast --kind {kind_flag} --args \"{args}\" --p {p} --source generator");
    let payload = HookPayload {
        session_id: Some(session.to_string()),
        hook_event_name: Some(stage.to_string()),
        tool_name: Some("Bash".to_string()),
        tool_input: Some(serde_json::json!({ "command": command })),
        tool_response: (stage == "PostToolUse").then(
            || serde_json::json!({ "stdout": "forecast recorded", "stderr": "", "interrupted": false }),
        ),
        ..Default::default()
    };
    let appended = if stage == "PreToolUse" {
        record_tool(store, &payload)?
    } else {
        record_post_tool(store, &payload)?
    };
    anyhow::ensure!(appended.is_empty(), "the hook recorded a tsr forecast call");
    Ok(())
}

fn selftest_inner(
    out: Option<&std::path::Path>,
    update_frozen_fixture: bool,
) -> anyhow::Result<std::path::PathBuf> {
    // Refuse before writing anything when no runtime is reachable: the fixture must never be
    // fabricated, and a half-written fixture is worse than none.
    if crate::exec::Runtime::detect().is_none() {
        anyhow::bail!(
            "no container runtime reachable (podman/docker); refusing to fabricate the fixture"
        );
    }

    // Where the trace lands. `--update-frozen-fixture` is the *only* way to touch the committed
    // `fixtures/m0`; otherwise the caller's `--out` (or a fresh temp dir) is used, so running the
    // selftest from the repository root can never overwrite the frozen fixture (repair item 3).
    let out_dir: std::path::PathBuf = if update_frozen_fixture {
        std::path::PathBuf::from("fixtures/m0")
    } else {
        match out {
            Some(p) => p.to_path_buf(),
            None => {
                let dir = std::env::temp_dir().join(format!(
                    "tsr-m0-selftest-{}-{}",
                    std::process::id(),
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_nanos())
                        .unwrap_or(0)
                ));
                dir
            }
        }
    };

    // The store is built in a scratch directory, not in `out_dir`: a refusal must write nothing
    // into `out_dir`, and the trace is copied there only once every run executed in a container.
    let scratch = std::env::temp_dir().join(format!("tsr-m0-selftest-db-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch)?;
    let db = scratch.join("session.db");
    let store = crate::store_path::open_store_at(&db)?;

    let session = "m0-selftest-session";
    let workspace = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));

    // 1. The task is opened. SessionStart also writes `<cwd>/.tsr/session`, the same channel Claude
    //    Code's tool calls will use; the selftest then resolves the task through that file, so the
    //    fixture proves the real path rather than a shortcut.
    let start = HookPayload {
        session_id: Some(session.to_string()),
        hook_event_name: Some("SessionStart".to_string()),
        cwd: Some(workspace.display().to_string()),
        ..Default::default()
    };
    record_session_start(&store, &start)?;
    let task = crate::store_path::resolve_session_task(None)
        .map_err(|e| anyhow::anyhow!("selftest session file not readable: {e}"))?;
    if task != crate::store_path::task_for_session(session) {
        anyhow::bail!("selftest session file resolved to the wrong task");
    }

    // 2. Two inspections.
    for path in ["crates/tsr-cli/src/lib.rs", "crates/tsr-store/src/lib.rs"] {
        let p = HookPayload {
            session_id: Some(session.to_string()),
            hook_event_name: Some("PreToolUse".to_string()),
            tool_name: Some("Read".to_string()),
            tool_input: Some(serde_json::json!({ "file_path": path })),
            ..Default::default()
        };
        record_tool(&store, &p)?;
    }

    // 3. A forecast before the first gated action, naming the action and its arguments. In a real
    //    session the forecast is itself a `Bash` tool call, so its PreToolUse and PostToolUse reach the
    //    hook around it; the selftest sends both, as Claude Code does.
    forecast_call_hooks(
        &store,
        session,
        "PreToolUse",
        ActionKind::RunApprovedTest,
        SELFTEST_TEST_CMD,
        0.7,
    )?;
    let candidate = record_forecast_for_action(
        &store,
        &task,
        ActionKind::RunApprovedTest,
        SELFTEST_TEST_CMD,
        0.7,
        "generator",
    )?;
    forecast_call_hooks(
        &store,
        session,
        "PostToolUse",
        ActionKind::RunApprovedTest,
        SELFTEST_TEST_CMD,
        0.7,
    )?;

    // 4. The first run: the agent's Bash call is `tsr exec -- cargo test …`. The hook records the
    //    commitment only (the forecast is already on record); `tsr exec` records the dispatch and
    //    the execution in the real container; the matching PostToolUse reconciles nothing further.
    let exec_cmd = format!(
        "tsr exec --candidate {} -- {SELFTEST_TEST_CMD}",
        candidate.0.to_hex()
    );
    let pre = HookPayload {
        session_id: Some(session.to_string()),
        hook_event_name: Some("PreToolUse".to_string()),
        tool_name: Some("Bash".to_string()),
        tool_input: Some(serde_json::json!({ "command": exec_cmd })),
        ..Default::default()
    };
    record_tool(&store, &pre)?;
    let argv: Vec<String> = SELFTEST_TEST_ARGV.iter().map(|s| s.to_string()).collect();
    crate::exec::run(&store, &task, &workspace, Some(candidate), &argv)
        .map_err(|e| anyhow::anyhow!("selftest exec refused: {e}"))?;
    ensure_last_run_executed(&store, &task)?;
    let post = HookPayload {
        session_id: Some(session.to_string()),
        hook_event_name: Some("PostToolUse".to_string()),
        tool_name: Some("Bash".to_string()),
        tool_input: Some(serde_json::json!({ "command": exec_cmd })),
        tool_response: Some(serde_json::json!({ "exit_code": 0, "stdout": "test result: ok" })),
        ..Default::default()
    };
    record_post_tool(&store, &post)?;

    // 5. A forecast before the patch proposal, then the proposal itself (gated).
    forecast_call_hooks(
        &store,
        session,
        "PreToolUse",
        ActionKind::ProposePatch,
        "crates/tsr-cli/src/lib.rs",
        0.6,
    )?;
    record_forecast_for_action(
        &store,
        &task,
        ActionKind::ProposePatch,
        "crates/tsr-cli/src/lib.rs",
        0.6,
        "generator",
    )?;
    forecast_call_hooks(
        &store,
        session,
        "PostToolUse",
        ActionKind::ProposePatch,
        "crates/tsr-cli/src/lib.rs",
        0.6,
    )?;
    let edit = HookPayload {
        session_id: Some(session.to_string()),
        hook_event_name: Some("PreToolUse".to_string()),
        tool_name: Some("Edit".to_string()),
        tool_input: Some(serde_json::json!({ "file_path": "crates/tsr-cli/src/lib.rs" })),
        ..Default::default()
    };
    record_tool(&store, &edit)?;

    // 6. A forecast before the second gated action, then the second run the same way. The second
    //    run is the *same* command as the first, so it names the same candidate id — the fresh
    //    forecast re-arms it (one forecast authorizes one run), exactly as `fixtures/m0` runs one
    //    candidate twice, each time after a fresh forecast.
    forecast_call_hooks(
        &store,
        session,
        "PreToolUse",
        ActionKind::RunApprovedTest,
        SELFTEST_TEST_CMD,
        0.6,
    )?;
    let candidate2 = record_forecast_for_action(
        &store,
        &task,
        ActionKind::RunApprovedTest,
        SELFTEST_TEST_CMD,
        0.6,
        "generator",
    )?;
    forecast_call_hooks(
        &store,
        session,
        "PostToolUse",
        ActionKind::RunApprovedTest,
        SELFTEST_TEST_CMD,
        0.6,
    )?;
    let exec_cmd2 = format!(
        "tsr exec --candidate {} -- {SELFTEST_TEST_CMD}",
        candidate2.0.to_hex()
    );
    let pre2 = HookPayload {
        session_id: Some(session.to_string()),
        hook_event_name: Some("PreToolUse".to_string()),
        tool_name: Some("Bash".to_string()),
        tool_input: Some(serde_json::json!({ "command": exec_cmd2 })),
        ..Default::default()
    };
    record_tool(&store, &pre2)?;
    let argv2: Vec<String> = SELFTEST_TEST_ARGV.iter().map(|s| s.to_string()).collect();
    crate::exec::run(&store, &task, &workspace, Some(candidate2), &argv2)
        .map_err(|e| anyhow::anyhow!("selftest exec refused: {e}"))?;
    ensure_last_run_executed(&store, &task)?;
    let post2 = HookPayload {
        session_id: Some(session.to_string()),
        hook_event_name: Some("PostToolUse".to_string()),
        tool_name: Some("Bash".to_string()),
        tool_input: Some(serde_json::json!({ "command": exec_cmd })),
        tool_response: Some(serde_json::json!({ "exit_code": 0, "stdout": "test result: ok" })),
        ..Default::default()
    };
    record_post_tool(&store, &post2)?;

    // 7. A stop.
    let stop = HookPayload {
        session_id: Some(session.to_string()),
        hook_event_name: Some("Stop".to_string()),
        stop_hook_active: Some(false),
        ..Default::default()
    };
    record_stop(&store, &stop)?;

    // Write the trace as events.jsonl, in sequence order, into the caller's directory. Only now —
    // after every run was confirmed executed in a container — is anything written.
    let jsonl = crate::task::export(&store, &task)?;
    std::fs::create_dir_all(&out_dir)?;
    let out = out_dir.join("session.events.jsonl");
    std::fs::write(&out, jsonl)?;

    // Leave no session file behind: the selftest wrote one to prove the path, but it is not the
    // developer's live session.
    let _ = std::fs::remove_file(crate::store_path::session_file_in(
        &crate::store_path::session_dir(),
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    Ok(out)
}

/// Refuse (exit 3, writing nothing) unless the task's most recent dispatch was actually executed in
/// a container. A `Dispatched` with no `Executed` after it — or an `Unresolved` — means the run did
/// not happen, and the selftest must not fabricate a fixture from it (repair item 3).
fn ensure_last_run_executed(store: &Store, task: &TaskId) -> anyhow::Result<()> {
    let events = store.by_task(task)?;
    let mut last: Option<&EventKind> = None;
    for (_, ev) in &events {
        match &ev.kind {
            EventKind::Dispatched { .. } => last = Some(&ev.kind),
            EventKind::Executed { .. } => last = Some(&ev.kind),
            EventKind::Unresolved { .. } => last = Some(&ev.kind),
            _ => {}
        }
    }
    match last {
        Some(EventKind::Executed { .. }) => Ok(()),
        Some(EventKind::Unresolved { reason, .. }) => anyhow::bail!(
            "selftest run was not executed in a container (Unresolved: {reason}); refusing to write"
        ),
        _ => anyhow::bail!("selftest run was not executed in a container; refusing to write"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh store in a scratch directory under `target/` (never `/tmp`, which this host prunes).
    fn fresh_store(name: &str) -> (Store, std::path::PathBuf) {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/hook-unit")
            .join(name);
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("scratch dir");
        let store = crate::store_path::open_store_at(&dir.join("tsr.db")).expect("open store");
        (store, dir)
    }

    /// The candidate ids of a task's `Committed` events, in sequence order.
    fn committed_candidates(store: &Store, task: &TaskId) -> Vec<String> {
        store
            .by_task(task)
            .expect("by_task")
            .into_iter()
            .filter_map(|(_, ev)| match &ev.kind {
                EventKind::Committed { candidate, .. } => Some(candidate.0.to_hex()),
                _ => None,
            })
            .collect()
    }

    /// The kind tags of a task's events, in sequence order.
    fn tags(store: &Store, task: &TaskId) -> Vec<String> {
        store
            .by_task(task)
            .expect("by_task")
            .into_iter()
            .map(|(_, ev)| {
                serde_json::to_value(&ev.kind).expect("serialize")["kind"]
                    .as_str()
                    .expect("kind tag")
                    .to_string()
            })
            .collect()
    }

    fn bash_payload(session: &str, stage: &str, cmd: &str, exit: Option<i32>) -> HookPayload {
        HookPayload {
            session_id: Some(session.to_string()),
            hook_event_name: Some(stage.to_string()),
            tool_name: Some("Bash".to_string()),
            tool_input: Some(serde_json::json!({ "command": cmd })),
            tool_response: exit.map(|c| serde_json::json!({ "exit_code": c, "stdout": "ok" })),
            ..Default::default()
        }
    }

    /// A `tsr exec -- …` PostToolUse appends nothing: `tsr exec` recorded the dispatch and the run.
    #[test]
    fn post_tool_for_tsr_exec_appends_nothing() {
        let (store, _dir) = fresh_store("post-exec");
        let session = "post-exec";
        let task = store_path::task_for_session(session);
        let cmd = "tsr exec -- cargo test --offline -p parser";

        record_tool(&store, &bash_payload(session, "PreToolUse", cmd, None)).expect("pre");
        let before = tags(&store, &task);
        let appended =
            record_post_tool(&store, &bash_payload(session, "PostToolUse", cmd, Some(0)))
                .expect("post");
        assert!(
            appended.is_empty(),
            "a tsr exec PostToolUse appends nothing"
        );
        assert_eq!(tags(&store, &task), before, "the log is unchanged");
    }

    /// A `tsr forecast …` call is the forecast itself: the hook appends nothing before or after it,
    /// with a bare or a path-prefixed `tsr`, so nothing lands between a forecast and the commitment.
    #[test]
    fn forecast_call_appends_nothing_before_or_after() {
        let (store, _dir) = fresh_store("forecast-call");
        let session = "forecast-call";
        let task = store_path::task_for_session(session);
        for cmd in [
            "tsr forecast --kind propose_patch --args \"src/lib.rs\" --p 0.9 --source generator",
            "/opt/x/bin/tsr forecast --kind run_approved_test --args \"cargo test\" --p 0.8 --source generator",
        ] {
            assert!(is_tsr_forecast_command(cmd));
            let pre = record_tool(&store, &bash_payload(session, "PreToolUse", cmd, None)).expect("pre");
            let post = record_post_tool(&store, &bash_payload(session, "PostToolUse", cmd, None))
                .expect("post");
            assert!(pre.is_empty() && post.is_empty(), "{cmd}: the hook recorded the forecast call");
        }
        assert!(tags(&store, &task).is_empty(), "the log is unchanged");
        assert!(!is_tsr_forecast_command("tsr exec -- tsr forecast"));
        assert!(!is_tsr_forecast_command("echo tsr forecast"));
    }

    /// The real-session shape: Pre(forecast call) → forecast → Post(forecast call) → Pre(Edit). The
    /// commitment lands at the forecast's state version.
    #[test]
    fn forecast_call_hooks_keep_the_state_version() {
        let (store, dir) = fresh_store("forecast-sv");
        let session = "forecast-sv";
        let task = store_path::task_for_session(session);
        let start = HookPayload {
            session_id: Some(session.to_string()),
            hook_event_name: Some("SessionStart".to_string()),
            ..Default::default()
        };
        record_session_start(&store, &start).expect("start");
        let path = "/w/crates/rts-core/src/lib.rs";
        let cmd = format!(
            "tsr forecast --kind propose_patch --args \"{path}\" --p 0.95 --source generator"
        );
        record_tool(&store, &bash_payload(session, "PreToolUse", &cmd, None)).expect("pre");
        record_forecast_for_action(
            &store,
            &task,
            ActionKind::ProposePatch,
            path,
            0.95,
            "generator",
        )
        .expect("forecast");
        record_post_tool(&store, &bash_payload(session, "PostToolUse", &cmd, None)).expect("post");
        let edit = HookPayload {
            session_id: Some(session.to_string()),
            hook_event_name: Some("PreToolUse".to_string()),
            tool_name: Some("Edit".to_string()),
            tool_input: Some(serde_json::json!({ "file_path": path })),
            ..Default::default()
        };
        record_tool(&store, &edit).expect("edit");
        let out = dir.join("events.jsonl");
        std::fs::write(&out, crate::task::export(&store, &task).expect("export")).expect("write");
        let session = crate::reconstruct::from_events(&out).expect("reconstruct");
        assert!(
            session.every_gated_action_has_prior_forecast_at_same_state_version(),
            "the commitment must land at the forecast's state version: {:?}",
            tags(&store, &task)
        );
    }

    /// A path-prefixed `tsr exec` is recognised the same way.
    #[test]
    fn post_tool_for_path_prefixed_tsr_exec_appends_nothing() {
        let (store, _dir) = fresh_store("post-exec-prefix");
        let session = "post-exec-prefix";
        let task = store_path::task_for_session(session);
        let cmd = "./target/debug/tsr exec -- cargo test --offline -p parser";

        record_tool(&store, &bash_payload(session, "PreToolUse", cmd, None)).expect("pre");
        let before = tags(&store, &task);
        let appended =
            record_post_tool(&store, &bash_payload(session, "PostToolUse", cmd, Some(0)))
                .expect("post");
        assert!(appended.is_empty());
        assert_eq!(tags(&store, &task), before);
    }

    /// A host `Bash` PostToolUse reconciles the commitment the PreToolUse made: `Executed`, no new
    /// `Proposed`/`Committed`.
    #[test]
    fn post_tool_for_host_bash_records_executed_without_proposing() {
        let (store, _dir) = fresh_store("post-host");
        let session = "post-host";
        let task = store_path::task_for_session(session);
        let cmd = "cargo test --offline -p parser";

        record_tool(&store, &bash_payload(session, "PreToolUse", cmd, None)).expect("pre");
        assert_eq!(
            tags(&store, &task),
            vec!["proposed", "committed", "dispatched"]
        );

        let appended =
            record_post_tool(&store, &bash_payload(session, "PostToolUse", cmd, Some(0)))
                .expect("post");
        assert_eq!(appended.len(), 1);
        assert_eq!(
            tags(&store, &task),
            vec!["proposed", "committed", "dispatched", "executed"],
            "the PostToolUse reconciles the commitment and invents nothing"
        );
    }

    /// A PostToolUse with no prior commitment records `Unresolved`, never `Proposed`/`Committed`.
    #[test]
    fn post_tool_with_no_prior_commitment_records_unresolved() {
        let (store, _dir) = fresh_store("post-no-commit");
        let session = "post-no-commit";
        let task = store_path::task_for_session(session);
        let cmd = "cargo test --offline -p parser";

        record_post_tool(&store, &bash_payload(session, "PostToolUse", cmd, Some(0)))
            .expect("post");
        assert_eq!(tags(&store, &task), vec!["unresolved"]);
        let reason = store
            .by_task(&task)
            .expect("by_task")
            .into_iter()
            .find_map(|(_, ev)| match ev.kind {
                EventKind::Unresolved { reason, .. } => Some(reason),
                _ => None,
            })
            .expect("an Unresolved event");
        assert_eq!(reason, "post-tool with no prior commitment");
    }

    /// A PostToolUse for a non-Bash tool records nothing new.
    #[test]
    fn post_tool_for_read_edit_write_records_nothing() {
        let (store, _dir) = fresh_store("post-read");
        let session = "post-read";
        let task = store_path::task_for_session(session);

        for tool in ["Read", "Edit", "Write"] {
            let payload = HookPayload {
                session_id: Some(session.to_string()),
                hook_event_name: Some("PostToolUse".to_string()),
                tool_name: Some(tool.to_string()),
                tool_input: Some(serde_json::json!({ "file_path": "crates/tsr-cli/src/lib.rs" })),
                ..Default::default()
            };
            let appended = record_post_tool(&store, &payload).expect("post");
            assert!(appended.is_empty(), "{tool} PostToolUse appends nothing");
        }
        assert!(tags(&store, &task).is_empty(), "the log stays empty");
    }

    /// A second run of the same command reconciles the *second* commitment, not the first.
    #[test]
    fn post_tool_reconciles_the_most_recent_unreconciled_commitment() {
        let (store, _dir) = fresh_store("post-second-run");
        let session = "post-second-run";
        let task = store_path::task_for_session(session);
        let cmd = "cargo test --offline -p parser";

        for _ in 0..2 {
            record_tool(&store, &bash_payload(session, "PreToolUse", cmd, None)).expect("pre");
            record_post_tool(&store, &bash_payload(session, "PostToolUse", cmd, Some(0)))
                .expect("post");
        }
        assert_eq!(
            tags(&store, &task),
            vec![
                "proposed",
                "committed",
                "dispatched",
                "executed",
                "proposed",
                "committed",
                "dispatched",
                "executed"
            ],
            "each run proposes and commits once, and each PostToolUse reconciles its own commitment"
        );
    }

    /// `tsr exec --candidate <id>` is recognised after leading `VAR=value` assignments, an `env`
    /// prefix, and inside a command chain (review item 8a, repair): a prefix must not defeat
    /// `tsr_exec_candidate`.
    #[test]
    fn tsr_exec_candidate_is_recognised_after_prefixes_and_chains() {
        let id = "a".repeat(64);
        for cmd in [
            format!("tsr exec --candidate {id} -- cargo test"),
            format!("RUST_BACKTRACE=1 tsr exec --candidate {id} -- cargo test"),
            format!("env X=1 tsr exec --candidate {id} -- cargo test"),
            format!("cd crate && tsr exec --candidate {id} -- cargo test"),
            format!("./target/debug/tsr exec --candidate {id} -- cargo test"),
            format!("time tsr exec --candidate {id} -- cargo test"),
            format!("nice -n 5 tsr exec --candidate {id} -- cargo test"),
            format!("timeout 60 tsr exec --candidate {id} -- cargo test"),
            format!("nohup tsr exec --candidate {id} -- cargo test"),
        ] {
            assert_eq!(
                tsr_exec_candidate(&cmd).map(|c| c.0.to_hex()),
                Some(id.clone()),
                "{cmd:?} names the candidate"
            );
        }
        for cmd in [
            "tsr forecast --kind run_approved_test",
            "cargo test",
            "ls",
            "tsr exec -- cargo test",
            "tsr exec --candidate nothex -- cargo test",
        ] {
            assert!(
                tsr_exec_candidate(cmd).is_none(),
                "{cmd:?} names no candidate"
            );
        }
    }

    /// Defect 6: the hook reads `--candidate=<id>` as well as `--candidate <id>`, and a `<(…)`
    /// process substitution before `tsr exec` does not hide the candidate.
    #[test]
    fn tsr_exec_candidate_reads_equals_form_and_survives_process_substitution() {
        let id = "b".repeat(64);
        for cmd in [
            format!("tsr exec --candidate={id} -- cargo test"),
            format!("tsr exec --candidate={id} -- diff <(a) <(b)"),
            format!("cat <(echo hi) && tsr exec --candidate {id} -- cargo test"),
            format!("tsr exec --candidate {id} -- diff <(echo a) <(echo b)"),
        ] {
            assert_eq!(
                tsr_exec_candidate(&cmd).map(|c| c.0.to_hex()),
                Some(id.clone()),
                "{cmd:?} names the candidate"
            );
        }
        assert!(
            tsr_exec_candidate("tsr exec --candidate=nothex -- cargo test").is_none(),
            "a bad --candidate=<id> names no candidate"
        );
    }

    /// Defect 2: a here-string `<<< "hi"` before `tsr exec` takes one word and no body, so it does
    /// not swallow the `tsr exec` command that follows it.
    #[test]
    fn here_string_does_not_swallow_the_following_tsr_exec() {
        let id = "c".repeat(64);
        let cmd = format!("cat <<< \"hi\" && tsr exec --candidate {id} -- cargo test");
        assert_eq!(
            tsr_exec_candidate(&cmd).map(|c| c.0.to_hex()),
            Some(id.clone()),
            "the here-string takes one word and the tsr exec is still seen"
        );
        // A here-document still consumes its body (the body is data, not commands).
        let heredoc = format!("cat <<EOF\nhi\nEOF\ntsr exec --candidate {id} -- cargo test");
        assert_eq!(
            tsr_exec_candidate(&heredoc).map(|c| c.0.to_hex()),
            Some(id.clone()),
            "a here-document body is data and the tsr exec after it is seen"
        );
    }

    /// Defect 1 (hook side): the hook commits a named candidate only if it belongs to this task, has
    /// a forecast, and has not been dispatched since that newest forecast. Otherwise it records the
    /// Bash call as an ordinary non-gated call.
    #[test]
    fn hook_commits_a_named_candidate_only_when_it_is_forecast_and_not_dispatched() {
        let (store, _dir) = fresh_store("named-gate");
        let session = "named-gate";
        let task = store_path::task_for_session(session);

        // An id that is not a candidate of this task is not committed.
        let unknown = "d".repeat(64);
        let cmd = format!("tsr exec --candidate {unknown} -- cargo test");
        record_tool(&store, &bash_payload(session, "PreToolUse", &cmd, None)).expect("pre");
        assert!(
            !committed_candidates(&store, &task).contains(&unknown),
            "an unknown id is not committed"
        );

        // A candidate of this task with a forecast is committed.
        let cid = record_forecast_for_action(
            &store,
            &task,
            ActionKind::RunApprovedTest,
            "cargo test",
            0.5,
            "operator",
        )
        .expect("forecast");
        let cmd = format!("tsr exec --candidate {} -- cargo test", cid.0.to_hex());
        record_tool(&store, &bash_payload(session, "PreToolUse", &cmd, None)).expect("pre");
        assert!(
            committed_candidates(&store, &task).contains(&cid.0.to_hex()),
            "a forecast candidate of this task is committed"
        );

        // Once it is dispatched, the hook refuses to commit it again.
        let dispatched = Event::new(
            task,
            vec![],
            Source::Operator,
            EventKind::Dispatched {
                candidate: cid,
                in_sandbox: true,
            },
            now_rfc3339(),
        );
        store.append(&dispatched).expect("dispatched");
        let before = committed_candidates(&store, &task);
        record_tool(&store, &bash_payload(session, "PreToolUse", &cmd, None)).expect("pre");
        let after = committed_candidates(&store, &task);
        assert_eq!(
            after.iter().filter(|c| *c == &cid.0.to_hex()).count(),
            before.iter().filter(|c| *c == &cid.0.to_hex()).count(),
            "an already-dispatched candidate is not committed again"
        );
    }

    /// A test run is recognised after leading `VAR=value` assignments, an `env` prefix, and inside a
    /// command chain (review item 8a).
    #[test]
    fn test_run_classification_skips_prefixes_and_chains() {
        for cmd in [
            "cargo test",
            "cargo test --offline -p parser",
            "RUST_BACKTRACE=1 cargo test",
            "env X=1 cargo test",
            "env RUST_BACKTRACE=1 cargo test",
            "cd crate && cargo test",
            "cargo nextest run",
            "RUST_BACKTRACE=1 cargo nextest run",
            "echo hi; cargo test",
            "true | cargo test",
        ] {
            assert!(is_test_run_command(cmd), "{cmd:?} is a test run");
        }
        for cmd in [
            "ls",
            "cargo build",
            "cargo testx",
            "echo cargo test",
            "cd crate",
            "RUST_BACKTRACE=1 ls",
        ] {
            assert!(!is_test_run_command(cmd), "{cmd:?} is not a test run");
        }
    }

    /// The three spellings of `sh -c 'a; b'` produce one canonical digest (review item 8b).
    #[test]
    fn one_canonical_encoding_for_three_spellings() {
        let a = tsr_exec_inner_argv("tsr exec -- sh -c 'a; b'").expect("single-quoted");
        let b = tsr_exec_inner_argv("tsr exec -- sh -c \"a; b\"").expect("double-quoted");
        let c = tsr_exec_inner_argv("tsr exec -- sh -c a\\;\\ b").expect("escaped");
        assert_eq!(canonical_args(&a), "sh -c 'a; b'");
        assert_eq!(a, b, "single- and double-quoted spellings agree");
        assert_eq!(a, c, "escaped spelling agrees");
    }

    /// A `Generate` turn (a Bash `ls`, a `TodoWrite`) is recorded `Proposed` → `Committed`, so
    /// `reconstruct` accepts it (review item 8c).
    #[test]
    fn a_generate_turn_is_proposed_then_committed() {
        let (store, _dir) = fresh_store("generate-shape");
        let session = "generate-shape";
        let task = store_path::task_for_session(session);
        record_tool(&store, &bash_payload(session, "PreToolUse", "ls", None)).expect("pre");
        record_post_tool(&store, &bash_payload(session, "PostToolUse", "ls", Some(0)))
            .expect("post");
        let t = tags(&store, &task);
        assert_eq!(
            &t[..2],
            &["proposed", "committed"],
            "a Generate turn is Proposed then Committed, never a bare Generate"
        );
        assert!(
            !t.iter().any(|k| k == "generate"),
            "no bare Generate event is recorded: {t:?}"
        );
    }
}
