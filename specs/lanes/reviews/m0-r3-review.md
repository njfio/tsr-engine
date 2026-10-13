**M0 r3: not done** — every spelling the brief names works end to end, but the new parser regresses quoted globs and `{}`, panics on a trailing `env -u`/`nice -n`, and breaks the brief's own rule for `$(…)` and `<(…)`.

Tip 16d3dc5 (two lane commits, brief says one). Checker: external crate calling `tsr_cli::reconstruct` + the six checks `m0_real_session.rs` makes.

## (1) Brief spellings: DONE
Real `bash -c` runs `tsr exec` after the `PreToolUse` hook; forecast = the argv exec receives. Each gave proposed=1, forecast=1, dispatched=1, all checks true:
`cargo test 2>&1 | tail -20`, `> out.txt`, `2>/dev/null`, `cat "$HOME/x"`, `${HOME}/x`, `ls ~`, `ls *.toml` (-> `a.toml b.toml`), forecast `sh -c "a; b"` vs hook `sh -c 'a; b'` (and the reverse, and `sh -c a\;\ b`). hook.rs:985 canonicalises `--args`.

## (2) Classification: DONE
`cd crate⏎cargo test`, `timeout 60 cargo test`, `cargo +1.90.0 test` -> `proposed run_approved_test` (also `nice -n 5`, `time`, `cargo +nightly nextest run`, `timeout 1m nice -n 5`). `echo cargo test`, `cargo build`, `cargo test-foo`, `time cargo build`, `timeout 60 echo cargo test` -> `generate`. Forecast `cargo test` matches `timeout 60 cargo test`; `cargo +1.90.0 test` digests with its `+1.90.0` (forecast it that way).

## (3) Unevaluable words: NOT DONE
- `"$(date)"`, `` `date` ``, `"$((1+2))"`, two substitutions in one command: digested as written, exactly one stderr line (PreToolUse only; PostToolUse silent). Substitutions outside `tsr exec` print nothing.
- Unquoted `$(date)`, `$((1+2))`, `pre$(date)post`: the line prints, but shell.rs:209 returns after pushing `$` and the `(` at shell.rs:182 splits the command. Digest is `echo '$'`, not as written.
- `<(…)` / `>(…)` (shell.rs:169): eaten as a redirection. `diff <(echo a) <(echo b)` digests `diff a` and prints nothing.

## (4) Own spellings: ~70 by hand, 86-command corpus, ~5,400 random strings
Fine (one proposal, one forecast, accepted): here-doc, `&&`/`||` chains, `( )`, `{ …; }`, `env -i PATH=$PATH`, nested quotes, unmatched glob, `>|`, `&>`, `2>&1 >f`, `>f 2>&1`, `>&2`, `< f`, `| tee`, tabs, `"cargo"`, `&`, trailing `;`. Defects below. Random strings without `env -u`/`env -C`/`nice -n`: 0 panics, worst 0.4 s.

## (5) Scope and gates: DONE
- Lane diff: hook.rs, new shell.rs, tests/m0_ordinary_session.rs. The two specs files come from brief commit 0b27c57. exec.rs, main.rs, lib.rs untouched. shell.rs is off the literal Owns list (no forecast.rs exists); wired by `#[path = "shell.rs"] mod shell;` at hook.rs:23-25, a private child of `hook`, so lib.rs stays byte-identical.
- Byte-identical to 79154b2: fixtures/m0, m0_reconstruct.rs, m0_real_session.rs, lib.rs, store_path.rs, tsr-domain, tsr-policy, README.md.
- fmt clean; clippy 1.90 `-D warnings` clean; `cargo test --workspace --no-fail-fast`: 293 passed, 0 failed, 0 ignored. The two Docker-dependent m0_harness tests (`selftest_is_reproducible_structurally_excluding_duration_ms_by_name`, `selftest_without_a_runtime_exits_3_and_writes_nothing`) pass here (CLI, no daemon). Tree clean; only gitignored `target/` and `crates/tsr-cli/.tsr/`.

## New defects
1. **Medium, regression:** quoted or escaped globs are expanded (quotes are lost in parse, hook.rs:532 calls shell.rs:328). With `Cargo.toml` in cwd, `tsr exec -- find . -name "*.toml"` gives 2 proposals; 79154b2 accepted it.
2. **Medium, regression:** `{` and `}` split a command mid-word (shell.rs:182). `find . -exec echo {} \;`, `git show HEAD@{1}`, `xargs -I{} echo {}` all lose the forecast; 79154b2 accepted all three. Brace expansion (`a{,.bak}`) is silently wrong.
3. **Medium, new panic:** `env -u`, `env -C` or `nice -n` as the last words of any simple command exit 101 in both stages (hook.rs:156 `i += 2`, hook.rs:236). Malformed input, not ordinary, but 79154b2 returns 0 and the hook runs on every Bash call.
4. **Medium:** the (3) failures above.
5. **Medium (also in 79154b2):** backslash-newline (shell.rs:116) pushes a `\n` word: `tsr exec -- cargo test \⏎ --release` gives 2 proposals.
6. **Medium (also in 79154b2):** no `#` comments and here-doc bodies are parsed as commands. `tsr exec -- cargo test # run tests` gives 2 proposals; an apostrophe in a comment line or here-doc body before `tsr exec` swallows the rest.
7. **Medium:** an unquoted empty expansion becomes an empty word (shell.rs:122 sets has_word first): `echo $UNSET x` digests `echo '' x`, argv is `echo x`. Silent too: `${X:-d}`, `$?`, `$$`, `$'…'`, `export X=1 && … $X`.
8. **Low:** glob divergence: `*` matches dotfiles (`ls *` picks up `.tsr`), a wildcard in a directory part (`crates/*/Cargo.toml`) is not expanded, and `cd sub && … *.toml` globs in the hook's cwd.
9. **Low:** `time`/`nice`/`timeout`/`nohup`/`!`/`if …; then`/`do` before `tsr exec` lose the forecast (command_words_after_prefix, hook.rs:134, skips only `env`). `timeout -s KILL 60 cargo test` and `timeout -k 5 60 cargo test` classify as `generate` (hook.rs:218). `>&-` and `>&file` leave a stray word.
10. **Low:** exponential glob matcher (shell.rs:370): ten `*a` against a 40-char file name hung the hook past 60 s (six: 1.3 s). Realistic regexes take 0.02 s.
11. **Low:** `--args` is argv text: a forecast typed like the command (`ls *.toml`, `cat "$HOME/x"`, `… 2>&1 | tail`) never matches. Two `tsr exec` in one command: only the first is digested (as before).

## What I ran
Fake `docker` on PATH recording argv, `tsr` on PATH, ~70 fresh workspaces; jq-built payloads to `tsr hook --stage …`; real `bash -c` for the command; `tsr task export`; checker crate. Same cases on a 79154b2 build (git archive, separate target dir) for regressions. 86-command corpus and 5,400 random strings through both stages. fmt, clippy, full suite. Scratch in `/home/claude/r3/m2/rev-*`.
