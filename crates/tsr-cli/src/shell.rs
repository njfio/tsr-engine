//! A POSIX-shell command parser, enough of the grammar to find the simple command that is
//! `tsr exec` inside the command string Claude Code hands the hook.
//!
//! The hook sees the *command string*; `tsr exec` sees the argv the shell produced from it. The
//! hook no longer *evaluates* expansions (globs, `~`, `$VAR`) — it cannot, because the shell that
//! runs the command is not the hook's shell. Instead it parses the command with shell grammar only
//! and finds, for the `tsr exec` simple command, the argument words after `--`, quote-removed and
//! with redirections dropped. Nothing is matched: the hook commits the candidate the command names
//! with `--candidate <id>`.
//!
//! The grammar handled here:
//! * quotes (`'…'`, `"…"`) and escapes (`\x`), including backslash-newline continuation;
//! * `#` comments (a `#` at the start of a word, unquoted, runs to end of line);
//! * here-documents: `<<`/`<<-` and their body, which is data, not commands;
//! * here-strings: `<<<` and its single word, which has no body;
//! * separators `;` `&&` `||` `|` `&` and newlines;
//! * subshells `( … )` and groups `{ …; }` as whole tokens only — never splitting a word such as
//!   `{}` or `HEAD@{1}`;
//! * redirections in every form: `>`, `>>`, `<`, `>&`, `<&`, `>&-`, `>&file`, `&>`, `&>>`, `>|`,
//!   `<>`, `<<`, `<<-`, `n>`, `n>&m`;
//! * process substitution `<(…)`/`>(…)` as one unit (a word).
//!
//! Malformed input never panics: a trailing `env -u` or `nice -n` is just a plain word list.

/// One word of a simple command: its text with quotes removed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Word {
    pub text: String,
}

/// One simple command: its words (quotes removed, redirections dropped).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleCommand {
    pub words: Vec<Word>,
}

/// A parsed command string: the simple commands it contains, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parsed {
    pub commands: Vec<SimpleCommand>,
}

/// Parse a command string into simple commands with shell grammar only. No expansion is evaluated:
/// a word the shell would expand is kept as written, so the caller can find the `tsr exec` command
/// and the candidate id it names.
pub fn parse(cmd: &str) -> Parsed {
    let mut commands: Vec<SimpleCommand> = Vec::new();
    let mut words: Vec<Word> = Vec::new();
    let mut cur = String::new();
    let mut has_word = false;
    let mut chars = cmd.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;
    // A `#` starts a comment only at the start of a word (unquoted).
    let mut at_word_start = true;
    // Here-documents whose body has not yet been consumed; the body starts after the next newline.
    let mut heredocs: Vec<Heredoc> = Vec::new();

    macro_rules! flush_word {
        () => {
            if std::mem::replace(&mut has_word, false) {
                words.push(Word {
                    text: std::mem::take(&mut cur),
                });
            }
        };
    }
    macro_rules! flush_command {
        () => {
            flush_word!();
            if !words.is_empty() {
                commands.push(SimpleCommand {
                    words: std::mem::take(&mut words),
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
                    // Inside double quotes a backslash escapes only `"`, `\`, `$` and a backtick;
                    // a backslash-newline is a line continuation (removed).
                    match chars.peek() {
                        Some('"') | Some('\\') | Some('$') | Some('`') => {
                            cur.push(chars.next().unwrap());
                        }
                        Some('\n') => {
                            chars.next();
                        }
                        _ => cur.push('\\'),
                    }
                }
                '$' => {
                    consume_dollar(&mut chars, &mut cur);
                }
                '`' => {
                    consume_backtick(&mut chars, &mut cur);
                }
                _ => cur.push(c),
            }
            continue;
        }
        match c {
            '\'' => {
                in_single = true;
                has_word = true;
                at_word_start = false;
            }
            '"' => {
                in_double = true;
                has_word = true;
                at_word_start = false;
            }
            '\\' => {
                // A backslash escapes the next character; backslash-newline is a continuation.
                match chars.peek() {
                    Some('\n') => {
                        chars.next();
                    }
                    Some(_) => {
                        cur.push(chars.next().unwrap());
                        has_word = true;
                        at_word_start = false;
                    }
                    None => {
                        cur.push('\\');
                        has_word = true;
                        at_word_start = false;
                    }
                }
            }
            '$' => {
                has_word = true;
                at_word_start = false;
                consume_dollar(&mut chars, &mut cur);
            }
            '`' => {
                has_word = true;
                at_word_start = false;
                consume_backtick(&mut chars, &mut cur);
            }
            '~' if at_word_start => {
                // A leading `~` (or `~/…`) is a tilde expansion; the hook keeps it as written.
                has_word = true;
                at_word_start = false;
                cur.push('~');
            }
            '#' if at_word_start => {
                // A comment runs to the end of the line.
                for n in chars.by_ref() {
                    if n == '\n' {
                        break;
                    }
                }
                flush_command!();
            }
            c if c.is_whitespace() => {
                if c == '\n' {
                    flush_command!();
                    // A here-document body begins after the newline that ends its command line.
                    for heredoc in heredocs.drain(..) {
                        consume_heredoc_body(&mut chars, &heredoc);
                    }
                } else {
                    flush_word!();
                    at_word_start = true;
                }
            }
            ';' | '&' | '|' => {
                if c == '&' && chars.peek() == Some(&'>') {
                    // `&>` / `&>>`: a redirection, not a background separator.
                    chars.next();
                    skip_redirection(&mut chars, '>', &mut heredocs);
                    at_word_start = true;
                } else {
                    if chars.peek() == Some(&c) {
                        chars.next();
                    }
                    flush_command!();
                }
            }
            '<' | '>' if chars.peek() == Some(&'(') => {
                // Process substitution `<(…)` / `>(…)`: one word (it yields a path). It is not a
                // redirection, so it must not be dropped as one. Consume the opening `(` here:
                // `consume_balanced_parens` starts *after* it and counts the closing `)`.
                has_word = true;
                at_word_start = false;
                cur.push(c);
                chars.next();
                consume_balanced_parens(&mut chars, &mut cur);
            }
            '<' | '>' => {
                // A redirection: drop the operator, its optional file-descriptor number and its
                // target word. A file-descriptor number immediately before the operator (`2>`) is
                // part of the redirection, not a word.
                if has_word && !cur.is_empty() && cur.chars().all(|c| c.is_ascii_digit()) {
                    cur.clear();
                    has_word = false;
                } else {
                    flush_word!();
                }
                skip_redirection(&mut chars, c, &mut heredocs);
                at_word_start = true;
            }
            '(' | ')' => {
                // A subshell `( … )` wraps simple commands; treat the parenthesis as a command
                // separator so the inner `tsr exec` is found. A parenthesis is always a delimiter in
                // Bash (`foo(bar)` is a syntax error), and `$(…)`/`<(…)` are consumed before this
                // arm, so no lookahead is needed.
                flush_command!();
                at_word_start = true;
            }
            '{' | '}' if at_word_start && is_standalone_delimiter(&mut chars) => {
                // A brace group `{ …; }` wraps simple commands; treat the brace as a command
                // separator so the inner `tsr exec` is found. Only a *standalone* brace counts: a
                // `{`/`}` inside a word (`{}`, `HEAD@{1}`) is part of that word, and Bash requires
                // whitespace after `{` for it to open a group.
                flush_command!();
            }
            '*' | '?' => {
                // An unquoted glob metacharacter: the hook does not expand it (it cannot — the shell
                // that runs the command is not the hook's shell), so it is kept as written.
                cur.push(c);
                has_word = true;
                at_word_start = false;
            }
            _ => {
                cur.push(c);
                has_word = true;
                at_word_start = false;
            }
        }
    }
    flush_command!();
    Parsed { commands }
}

/// Consume a `$…` sequence into `out`, keeping it as written. A command substitution `$(…)` or
/// arithmetic `$((…))` is not evaluated (the hook cannot know the shell that will run the command),
/// so only the `$` is consumed here and the rest is read as ordinary word characters.
fn consume_dollar(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, out: &mut String) {
    match chars.peek() {
        Some('(') => {
            // `$(…)` or `$((…))`: keep as written.
            out.push('$');
        }
        Some('{') => {
            chars.next();
            out.push_str("${");
            for c in chars.by_ref() {
                out.push(c);
                if c == '}' {
                    break;
                }
            }
        }
        Some(c) if c.is_ascii_alphabetic() || *c == '_' => {
            out.push('$');
            while let Some(&c) = chars.peek() {
                if c.is_ascii_alphanumeric() || c == '_' {
                    out.push(c);
                    chars.next();
                } else {
                    break;
                }
            }
        }
        Some(c) if c.is_ascii_digit() => {
            out.push('$');
            chars.next();
        }
        _ => {
            out.push('$');
        }
    }
}

/// Consume a backtick command substitution `` `…` `` into `out`, keeping it as written.
fn consume_backtick(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, out: &mut String) {
    out.push('`');
    while let Some(c) = chars.next() {
        out.push(c);
        if c == '`' {
            break;
        }
        if c == '\\' {
            if let Some(n) = chars.next() {
                out.push(n);
            }
        }
    }
}

/// Whether a `(`/`)`/`{`/`}` just consumed at the start of a word is a *standalone* delimiter (a
/// subshell or brace-group token) rather than the first character of a word such as `{}`, `HEAD@{1}`
/// or `foo(bar)`. It is standalone only when the next character ends the word: whitespace, a
/// separator, a redirection, or end of input. A following `{`/`}`/`(`/`)` does not end the word, so
/// `{}` stays one word.
fn is_standalone_delimiter(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> bool {
    match chars.peek() {
        None => true,
        Some(&c) => c.is_whitespace() || matches!(c, ';' | '&' | '|' | '<' | '>'),
    }
}

/// A pending here-document: its delimiter word (quotes removed) and whether `<<-` strips leading
/// tabs from the body and the delimiter line.
struct Heredoc {
    delim: String,
    strip_tabs: bool,
}

/// Read a here-document delimiter word with quote removal: `EOF`, `'EOF'`, `"EOF"` and `\EOF` all
/// yield `EOF`. Stops at whitespace, a separator or a redirection.
fn read_heredoc_delim(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) -> String {
    let mut out = String::new();
    let mut in_single = false;
    let mut in_double = false;
    while let Some(&c) = chars.peek() {
        if in_single {
            chars.next();
            if c == '\'' {
                in_single = false;
            } else {
                out.push(c);
            }
            continue;
        }
        if in_double {
            chars.next();
            if c == '"' {
                in_double = false;
            } else {
                out.push(c);
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
                if let Some(n) = chars.next() {
                    out.push(n);
                }
            }
            c if c.is_whitespace() || matches!(c, ';' | '&' | '|' | '<' | '>') => break,
            _ => {
                out.push(c);
                chars.next();
            }
        }
    }
    out
}

/// Consume a here-document body: everything up to and including the line whose content equals the
/// delimiter (leading tabs stripped when `<<-`). The body is data, never commands.
fn consume_heredoc_body(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, heredoc: &Heredoc) {
    let mut line = String::new();
    loop {
        match chars.next() {
            None => return,
            Some('\n') => {
                let candidate = if heredoc.strip_tabs {
                    line.trim_start_matches('\t')
                } else {
                    line.as_str()
                };
                if candidate == heredoc.delim {
                    return;
                }
                line.clear();
            }
            Some(c) => line.push(c),
        }
    }
}

/// Consume a balanced `( … )` group into `cur`, starting just after the opening `(`. Nested
/// parentheses and quoted spans are respected. Used for process substitution `<(…)` / `>(…)`.
fn consume_balanced_parens(chars: &mut std::iter::Peekable<std::str::Chars<'_>>, cur: &mut String) {
    cur.push('(');
    let mut depth = 1usize;
    let mut in_single = false;
    let mut in_double = false;
    for c in chars.by_ref() {
        cur.push(c);
        if in_single {
            if c == '\'' {
                in_single = false;
            }
            continue;
        }
        if in_double {
            if c == '"' {
                in_double = false;
            }
            continue;
        }
        match c {
            '\'' => in_single = true,
            '"' => in_double = true,
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
    }
}

/// Skip a redirection after its `<`/`>` operator: an optional file-descriptor number, the operator
/// itself (`>`, `>>`, `<`, `>&`, `<&`, `>|`, `<>`, `&>`), and the target word. `2>&1` has no
/// separate target. A here-document (`<<`/`<<-`) consumes its body, which is data, not commands.
fn skip_redirection(
    chars: &mut std::iter::Peekable<std::str::Chars<'_>>,
    op: char,
    heredocs: &mut Vec<Heredoc>,
) {
    // `&>` / `&>>`: the caller saw `>` but the `&` was consumed as a word char; handle the doubled
    // operator (`>>`, `>&`, `<&`, `>|`, `<>`).
    if chars.peek() == Some(&op) {
        chars.next();
        if op == '<' {
            // `<<<`: a here-string. It takes exactly one word and has **no** body, so it must not be
            // read as a here-document (which would swallow the rest of the line, including a
            // following `tsr exec`). Consume the third `<` and the one word, then return.
            if chars.peek() == Some(&'<') {
                chars.next();
                skip_redirection_target(chars);
                return;
            }
            // `<<` / `<<-`: a here-document. Its body is data until the delimiter line, so record
            // the delimiter and let the main loop consume the body at the next newline.
            let strip_tabs = if chars.peek() == Some(&'-') {
                chars.next();
                true
            } else {
                false
            };
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    chars.next();
                } else {
                    break;
                }
            }
            let delim = read_heredoc_delim(chars);
            heredocs.push(Heredoc { delim, strip_tabs });
            return;
        }
    } else if chars.peek() == Some(&'|') || chars.peek() == Some(&'&') {
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
    skip_redirection_target(chars);
}

/// Skip whitespace and then one target word (quoted or bare) of a redirection or here-string. The
/// word ends at whitespace, a separator or another redirection operator.
fn skip_redirection_target(chars: &mut std::iter::Peekable<std::str::Chars<'_>>) {
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
