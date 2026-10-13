//! A POSIX-shell command parser, enough of the grammar to find the simple command that is
//! `tsr exec` inside the command string Claude Code hands the hook.
//!
//! The hook sees the *command string*; `tsr exec` sees the argv the shell produced from it. The two
//! must agree on one candidate, so the hook has to do what the shell did: split the string into
//! simple commands on `;`, `&&`, `||`, `|`, `&` and newlines, remove redirections (`2>&1`, `> f`,
//! `>> f`, `< f`, `2>/dev/null`), and expand `~`, `$VAR`, `${VAR}` and globs the way the shell
//! would, from the hook's environment and cwd.
//!
//! A word the parser cannot evaluate (command or process substitution, arithmetic) is kept as
//! written and reported through [`Parsed::unevaluable`], so the caller can warn that the forecast
//! may not match.

use std::path::{Path, PathBuf};

/// One simple command: its words (quotes removed, expansions applied) and whether any word could
/// not be evaluated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleCommand {
    pub words: Vec<String>,
    /// A word the parser could not evaluate (command/process substitution, arithmetic). It is kept
    /// in `words` as written; the caller warns that the forecast may not match.
    pub unevaluable: bool,
}

/// A parsed command string: the simple commands it contains, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parsed {
    pub commands: Vec<SimpleCommand>,
}

/// Parse a command string into simple commands, applying the shell's word splitting, redirection
/// removal and expansion. `cwd` is the directory globs expand against; `env` resolves `$VAR`.
pub fn parse(cmd: &str, _cwd: &Path, env: &dyn Fn(&str) -> Option<String>) -> Parsed {
    let mut commands: Vec<SimpleCommand> = Vec::new();
    let mut words: Vec<String> = Vec::new();
    let mut cur = String::new();
    let mut has_word = false;
    let mut unevaluable = false;
    let mut chars = cmd.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;

    // Flush the word being built, if any.
    macro_rules! flush_word {
        () => {
            if has_word {
                words.push(std::mem::take(&mut cur));
                has_word = false;
            }
        };
    }
    // Flush the simple command being built, if any.
    macro_rules! flush_command {
        () => {
            if has_word {
                words.push(std::mem::take(&mut cur));
                has_word = false;
            }
            if !words.is_empty() {
                commands.push(SimpleCommand {
                    words: std::mem::take(&mut words),
                    unevaluable: std::mem::replace(&mut unevaluable, false),
                });
            }
        };
    }

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
                '$' => {
                    if !expand_dollar(&mut chars, &mut cur, env) {
                        unevaluable = true;
                    }
                }
                '`' => {
                    // Command substitution: keep it as written and warn.
                    cur.push('`');
                    unevaluable = true;
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
            '$' => {
                has_word = true;
                if !expand_dollar(&mut chars, &mut cur, env) {
                    unevaluable = true;
                }
            }
            '`' => {
                has_word = true;
                cur.push('`');
                unevaluable = true;
            }
            '~' if !has_word => {
                // A leading `~` (or `~/…`) expands to `$HOME`.
                has_word = true;
                match chars.peek() {
                    Some('/') => {
                        if let Some(home) = env("HOME") {
                            cur.push_str(&home);
                        } else {
                            cur.push('~');
                        }
                    }
                    None => {
                        if let Some(home) = env("HOME") {
                            cur.push_str(&home);
                        } else {
                            cur.push('~');
                        }
                    }
                    _ => cur.push('~'),
                }
            }
            c if c.is_whitespace() => {
                // A newline separates commands like `;`; other whitespace separates words.
                if c == '\n' {
                    flush_command!();
                } else {
                    flush_word!();
                }
            }
            ';' | '&' | '|' => {
                // `&&` and `||` are one operator; `&` and `|` are one each.
                if chars.peek() == Some(&c) {
                    chars.next();
                }
                flush_command!();
            }
            '<' | '>' => {
                // A redirection: drop the operator, its optional file-descriptor number and its
                // target word. `2>&1` is a redirection with no separate target. A file-descriptor
                // number immediately before the operator (`2>`, `2>&1`) is part of the redirection,
                // not a word; a digit separated by whitespace (`echo 2 > f`) is an argument.
                if has_word && !cur.is_empty() && cur.chars().all(|c| c.is_ascii_digit()) {
                    cur.clear();
                    has_word = false;
                } else {
                    flush_word!();
                }
                skip_redirection(&mut chars, c);
            }
            '(' | ')' | '{' | '}' => {
                // A subshell `( … )` or a brace group `{ …; }` wraps simple commands; descend into
                // it by treating the delimiter as a command separator, so the inner `tsr exec` is
                // found. A `{`/`}` that is part of a word (e.g. `${VAR}`) is handled by the `$`
                // expansion above and never reaches here.
                flush_command!();
            }
            _ => {
                cur.push(c);
                has_word = true;
            }
        }
    }
    flush_command!();
    let _ = has_word;
    Parsed { commands }
}

/// Expand a `$…` sequence into `out`. Returns `false` when the word cannot be evaluated (command
/// substitution `$(…)`, arithmetic `$((…))`, or a backtick), in which case the text is kept as
/// written.
fn expand_dollar(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    out: &mut String,
    env: &dyn Fn(&str) -> Option<String>,
) -> bool {
    match chars.peek() {
        Some('(') => {
            // `$(…)` or `$((…))`: keep as written, unevaluable.
            out.push('$');
            false
        }
        Some('{') => {
            chars.next();
            let mut name = String::new();
            for c in chars.by_ref() {
                if c == '}' {
                    break;
                }
                name.push(c);
            }
            match env(&name) {
                Some(v) => out.push_str(&v),
                None => {
                    // An unset variable expands to nothing, as the shell does.
                }
            }
            true
        }
        Some(c) if c.is_ascii_alphabetic() || *c == '_' => {
            let mut name = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    name.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
            if let Some(v) = env(&name) {
                out.push_str(&v);
            }
            true
        }
        Some(c) if c.is_ascii_digit() => {
            // `$1` and friends: positional parameters are not in the hook's environment.
            chars.next();
            true
        }
        _ => {
            out.push('$');
            true
        }
    }
}

/// Skip a redirection after its `<`/`>` operator: an optional file-descriptor number, the operator
/// itself (`>`, `>>`, `<`, `>&`, `<&`), and the target word. `2>&1` has no separate target.
fn skip_redirection(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, op: char) {
    // A file-descriptor number may precede the operator (`2>`), but the caller has already consumed
    // the operator, so a leading digit here is the *target* of a `>&`/`<&` form. Consume the
    // doubled operator (`>>`, `>&`, `<&`).
    if chars.peek() == Some(&op) {
        chars.next();
    }
    // `>&1` / `<&1`: the target is a file descriptor, not a word.
    if chars.peek() == Some(&'&') {
        chars.next();
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() {
                chars.next();
            } else {
                break;
            }
        }
        return;
    }
    // Skip whitespace, then the target word (quoted or bare).
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else {
            break;
        }
    }
    let mut in_single = false;
    let mut in_double = false;
    while let Some(&c) = chars.peek() {
        if in_single {
            chars.next();
            if c == '\'' {
                in_single = false;
            }
            continue;
        }
        if in_double {
            chars.next();
            if c == '"' {
                in_double = false;
            }
            continue;
        }
        match c {
            '\'' => {
                in_single = true;
                chars.next();
            }
            '"' => {
                in_double = true;
                chars.next();
            }
            '\\' => {
                chars.next();
                chars.next();
            }
            c if c.is_whitespace() || matches!(c, ';' | '&' | '|' | '<' | '>') => break,
            _ => {
                chars.next();
            }
        }
    }
}

/// Expand a glob pattern against `cwd`, as the shell would: a word containing `*`, `?` or `[` is
/// replaced by the sorted list of matching paths (relative to `cwd`), or kept as written when
/// nothing matches. A pattern with a `/` is matched against the path from `cwd`.
pub fn expand_glob(word: &str, cwd: &Path) -> Vec<String> {
    if !word.contains('*') && !word.contains('?') && !word.contains('[') {
        return vec![word.to_string()];
    }
    let mut matches: Vec<String> = Vec::new();
    let (dir, pattern) = match word.rfind('/') {
        Some(i) => (&word[..i], &word[i + 1..]),
        None => ("", word),
    };
    let base: PathBuf = if dir.is_empty() {
        cwd.to_path_buf()
    } else if dir.starts_with('/') {
        PathBuf::from(dir)
    } else {
        cwd.join(dir)
    };
    if let Ok(entries) = std::fs::read_dir(&base) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if glob_match(pattern, &name) {
                matches.push(if dir.is_empty() {
                    name
                } else {
                    format!("{dir}/{name}")
                });
            }
        }
    }
    if matches.is_empty() {
        return vec![word.to_string()];
    }
    matches.sort();
    matches
}

/// Whether `name` matches a glob `pattern` (`*`, `?`, `[…]`), with no `/` crossing.
fn glob_match(pattern: &str, name: &str) -> bool {
    let p: Vec<char> = pattern.chars().collect();
    let n: Vec<char> = name.chars().collect();
    glob_match_at(&p, &n)
}

fn glob_match_at(p: &[char], n: &[char]) -> bool {
    if p.is_empty() {
        return n.is_empty();
    }
    match p[0] {
        '*' => {
            // `*` matches any run of characters (not `/`).
            for i in 0..=n.len() {
                if glob_match_at(&p[1..], &n[i..]) {
                    return true;
                }
            }
            false
        }
        '?' => !n.is_empty() && glob_match_at(&p[1..], &n[1..]),
        '[' => {
            if n.is_empty() {
                return false;
            }
            let mut i = 1;
            let negate = p.get(1) == Some(&'!') || p.get(1) == Some(&'^');
            if negate {
                i = 2;
            }
            let mut matched = false;
            let mut closed = false;
            while i < p.len() {
                if p[i] == ']' && i > if negate { 2 } else { 1 } {
                    closed = true;
                    break;
                }
                if i + 2 < p.len() && p[i + 1] == '-' && p[i + 2] != ']' {
                    if n[0] >= p[i] && n[0] <= p[i + 2] {
                        matched = true;
                    }
                    i += 3;
                } else {
                    if n[0] == p[i] {
                        matched = true;
                    }
                    i += 1;
                }
            }
            if !closed {
                return false;
            }
            if matched != negate {
                glob_match_at(&p[i + 1..], &n[1..])
            } else {
                false
            }
        }
        c => !n.is_empty() && n[0] == c && glob_match_at(&p[1..], &n[1..]),
    }
}
