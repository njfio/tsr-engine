**M0 r5: not done** — `--candidate` binding works for 14 of the 15 listed spellings, but a dispatched id runs again once the hook has committed it, `cat <<< "hi"⏎tsr exec --candidate …` is swallowed, and fmt, clippy and 7 workspace tests are red.

Tip d611b5e (one commit on brief commit 23f5ea9 on f9c64dd). Checker: external crate calling `tsr_cli::reconstruct` + the five invariants m0_real_session.rs makes; "1/1/1/1" = proposed/forecast/commit/dispatch.

## (1) forecast prints the candidate id: DONE
stdout is exactly 64 hex + "\n" (65 bytes), stderr empty (main.rs:433; hook.rs:1081–1106 returns the candidate id). Log: Proposed, Forecast (state_version 3), Committed at index 3. `--event`, `propose_patch` and a repeated forecast print the same id.

## (2) `exec --candidate` per spelling: NOT DONE (14/15)
Real `bash -c` after PreToolUse, fake docker, 1/1/1/1, one docker run, reconstruct ok and five checks true: `| tail -20`, `> out.txt`, `cat "$HOME/x"` (docker got the expanded path), `ls ~`, `ls *.toml` (docker got `ls a.toml b.toml`), `sh -c 'a; b'`, `sh -c "a⏎b"`, `git commit -m "t⏎⏎body"`, `ls [ab].toml`, `echo '*'`, `find . -exec echo {} \;`, `time`, `timeout 60`, `nice -n 5`.
Fails: `cat <<< "hi"⏎tsr exec --candidate ID -- cargo test` (also `<<< hi`): shell.rs:497 reads `<<<` as a here-document and the next line is swallowed as its body. Hook records a generate proposal, exec exits 2 "has not been committed", docker not called. `;`, `&&` or a blank line after the here-string work. shell.rs is untouched, so r4b defect 4 is still open though the commit message lists it closed.

## (3) unknown / unforecast / dispatched ids: NOT DONE
`tsr exec` alone: unknown, another session's id ("is not a candidate of this task"), unforecast (exec-minted and hook-generate ids: "has no forecast"), dispatched ("is already dispatched"), malformed/63/65/empty ("must be a 64-hex"): all exit 2, 0 docker `run` calls.
Through the hook first (what a session does) a dispatched id RUNS AGAIN: hook.rs:731–760 commits any parsed id unchecked, and exec.rs:602–603 clears `dispatched` at every `Committed`. One forecast, same command three times: 3 docker runs, committed=3 dispatched=3 executed=3, B5 false (11011). m0_candidate.rs:474 runs exec twice with no hook between, so it cannot see this. Refused ids leave a stray Committed (unknown id: committed=1, proposed=0).

## (4) hook finds `--candidate`: DONE, with gaps
Found, one Committed, no new proposal (hook-only runs, forecast text ≠ argv): quotes (`"ID"`, `'ID'`, `"--candidate"`), pipes, redirects, `;`/`&&`/newline, comments (also `# it's`), here-docs (`<<EOF`, `<<'EOF'` with apostrophes, `<<-`, two), `$(…)` assignment, `git commit -m "$(cat <<'EOF'…)"`, subshell, braces, if/for/case, `!`, `env`/`VAR=` prefixes, time/timeout -s/nice/nohup, path to tsr, `\⏎`, CRLF. Not taken (correct): `--candidate` inside `echo "…"`, `echo '…'`, a here-doc body, a comment, `sh -c '…'`, or after `--`.
`--candidate=<id>` is NOT supported by the hook (hook.rs:544 compares `== "--candidate"`) though clap accepts it: exec exits 2 "not been committed", one stray proposal. Also missed (exit 2, nothing wrong runs): `bash -c`/`sudo`/`command`/`xargs`, `$ID`, the second of `tsr exec --candidate A … && tsr exec --candidate B …` (A runs, B refused), and any `tsr exec` after a `<(…)` (shell.rs:237–244 + 449: `consume_balanced_parens` re-reads the peeked `(`, depth never returns to 0).

## (5) without `--candidate`: DONE
One stderr line (counted 1, exec.rs:446), candidate minted with no forecast. `reconstruct::from_events` accepts the log; invariant 3 (B5) is false (11011), so a checker that asserts B5 rejects it, as the brief allows. Hook records `run_approved_test` (not generate) for `tsr exec -- X` and exec re-proposes and re-commits the same id: 2 proposals, 2 commits, 1 dispatch. With a forecast for the same argv already logged the line still says "no forecast" and B5 is still false.

## (6) nothing matched: DONE (leftovers)
grep in src for `patterns.jsonl|Pattern|PatternWord|pending_matching|remember_pattern|load_patterns|pair_with`: none; no sidecar file in any store dir. Hook fuzz: 6,116 runs (5,000 random Pre, 1,000 Post, 58 edge strings ×2 stages incl. 1 MB, 200,000 `(`, NUL, `<<<`, truncated `--candidate`) + 3,716 (seed 2) + 54 garbage payloads: 0 non-zero, 0 panics, worst 1.3 s under load.

## (7) gates: NOT DONE
Scope DONE: the commit touches exec.rs, hook.rs, main.rs (Exec flag, its match arm, `run_exec`; Forecast arm untouched), m0_candidate.rs (new), m0_harness.rs, m0_matcher.rs (deleted), all on the Owns list. fixtures/m0, m0_reconstruct.rs, m0_real_session.rs, lib.rs, store_path.rs, shell.rs, m0_ordinary_session.rs, tsr-domain: identical to f9c64dd. m0_harness.rs (23 lines changed): `replay_selftest` takes the `CandidateId` that `record_forecast_for_action` now returns, builds `tsr exec --candidate <hex> -- …` and passes `Some(candidate)` to `run_with`, twice; no assertion removed or loosened, the binding is tighter.
Red: `cargo fmt --all -- --check` 6 diffs (exec.rs:974, hook.rs:1290/1372, m0_candidate.rs:273/350/542). `clippy --workspace --all-targets -D warnings` fails at hook.rs:737 (`task.clone()` on Copy) and m0_candidate.rs:244 (unused `argv`). Build ok. `cargo test --workspace --no-fail-fast`: 290 passed, 8 failed: m0_ordinary_session 5 (defect 3), m0_candidate 2 (defect 4), d2 1 (stale binary, not the lane's). The Docker m0 tests (m0_honest_sandbox 13/13) passed here. git status clean.

## New defects
1. **High:** a dispatched id re-runs through hook+exec; the refusal holds only for exec alone ((3)).
2. **Medium:** the brief's `cat <<< "hi"` before `tsr exec` loses the commit (shell.rs:497); m0_candidate.rs:350 tests `<<<` only after `--`.
3. **Medium, tests:** m0_ordinary_session.rs was not moved to the new protocol: 5 of 8 fail (`a_redirect_pipe_or_expansion_keeps_the_forecast`, `a_forecast_with_different_quoting_matches_the_hook`, `a_wrapped_tsr_exec_is_still_one_candidate` "got 2", `an_expansion_on_tsr_exec_matches_the_argv_exec_receives` "got 3", `an_ordinary_session_reconstructs_and_keeps_every_rule` B5 at :349): they run `tsr exec --` with no `--candidate`. The Definition of done's green workspace is not met.
4. **Medium, tests:** m0_candidate.rs is not hermetic: `bash -c "tsr exec …"` finds `tsr` on PATH, only the fake docker dir is prepended; here 2 of 5 fail ("tsr: command not found"; `pipe` shows dispatch 0), and with a `tsr` on PATH they pass using that binary, not `CARGO_BIN_EXE_tsr`. Forecast text equals the exec argv in every case, so the hook's old digest identity can satisfy the counts without finding `--candidate` (`--candidate $X -- cargo test` gives 1/1/1/1). The 5,000-string fuzz test took 1,990 s (one growing task; ~75 iterations/min at the end) and asserts only exit 0, which `run_hook` returns on every error, so only a panic (101) is caught.
5. **Low-Medium:** fmt and clippy fail (above).
6. **Low:** `--candidate=<id>` is not read by the hook; `<(…)` swallows the rest of the command (pre-existing); a bad id exits 3, not 2, when no container runtime exists (runtime checked first, exec.rs:432).
7. **Low:** dead code and stale docs: shell.rs `WordKind::Wildcard/QuotedWildcard` marking and `unevaluable` (shell.rs:26–33, 98–107, 197, 276) are read by nothing; shell.rs:7–9 still links `crate::hook::Pattern`; hook.rs:120 and :866–868 (a deleted fn's doc glued onto `exec_outcome`).
8. **Info:** exec never compares argv with the forecast, and `Dispatched` records no argv (out of scope per the brief): forecast `cargo test` authorises any argv.

## What I ran
Fake `docker` on PATH recording argv, `tsr` symlinked on PATH, ~140 fresh workspaces; jq payloads to `tsr hook --stage …`; real `bash -c`; `tsr task export`; checker crate; hook-only matrix of 90 command shapes; fuzz 2 seeds + garbage payloads; refusal matrix with and without the hook; fmt and clippy on 1.90.0; workspace tests in the shared target (d2_common_event_id fails there from the old stale binary, as in r4b; passes 2/2 in a fresh target). Scratch in `mktemp -d`, deleted; no /tmp/tsr-* created.
