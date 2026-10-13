**M0 r2: done** — all five checks hold by behaviour at 79154b2, but two medium residues still lose a forecast in common spellings the brief did not name.

Reconstruct: an external crate calling `tsr_cli::reconstruct` and the six checks `m0_real_session.rs` makes (it passes the real fixture).

## (1) Ordinary session: DONE
The brief's sequence piped into `tsr hook`: 19 events, all six checks true.
- Both gated commits sit at their forecast's `state_version` (10, 15).
- `ls` and TodoWrite: `proposed generate` then `committed`. `tsr exec` reused the hook's candidate.
- A 32-event session (Edit with forecast, repeated `ls`, prompt, WebFetch, a test command run twice) also passes.

## (2) Classification: DONE
Test runs (`proposed run_approved_test` plus host dispatch): `RUST_BACKTRACE=1 cargo test`, `env X=1 cargo test`, `cd crate && cargo test`, `cargo nextest run`.
`generate`: `cargo build`, `echo cargo test`, `cargo test-foo`.

## (3) One digest: DONE
`sh -c 'a; b'`, `sh -c "a; b"` and `sh -c a\;\ b` each give one proposal, one forecast, digest `20f3ba7d2270`, and a log reconstruct accepts. `tsr exec` alone gives the same digest; twelve other quoting forms agree. Code: hook.rs:36, :133; exec.rs:438, :562.

## (4) Low defects: DONE
- store_path.rs:53. With `$O/link → <W>/deep/sub`, `TSR_DB=$O/link/../x/tsr.db` gets the read-only mount of `<W>/deep/x`.
- `TSR_DB` = `<W>/tsr.db`, `tsr.db`, `./tsr.db`, `a/../tsr.db` or a symlink to W: exit 2, no docker run, no `tsr.db` left.
- Fourteen r1b spellings re-run: unchanged.

## (5) Frozen files and gates: DONE
- `fixtures/m0`, `m0_reconstruct.rs`, `m0_real_session.rs`, `lib.rs`, `tsr-domain`: no diff against 8f08b03.
- `git diff --stat 98be827 79154b2`: exec.rs, hook.rs, store_path.rs, `tests/m0_ordinary_session.rs`, specs x2, and **main.rs** (defect 4).
- fmt clean; clippy 1.90 `-D warnings` clean.
- `cargo test --workspace --no-fail-fast`: 288 passed, 0 failed, 0 ignored. No Docker-only failures (CLI present, no daemon; the real-runner half self-skips).

## New defects
1. **Medium: a redirect, pipe or expansion on `tsr exec` loses the forecast.** hook.rs:462 digests every word after `--`; exec gets argv after the shell acted. Forecast `cargo test`, then `tsr exec -- cargo test 2>&1 | tail -20`: 2 proposals, 1 forecast, reconstruct rejects. Same for `> out.txt`, `"$HOME"`, `~`, globs. `&& echo ok` and `; echo done` are fine. Not a regression (the old code failed these too), but it is item 9's symptom in a very common spelling.
2. **Medium: `tsr forecast --args` is not canonicalised** (hook.rs:935). A forecast of `sh -c "a; b"` never matches the hook's `sh -c 'a; b'`: 2 proposals, reconstruct rejects. It works only in canonical form.
3. **Low: some test runs still record `generate`.** `cd crate⏎cargo test` (newline is not a separator, hook.rs:143), `timeout 60 cargo test`, `cargo +1.90.0 test`.
4. **Low, process:** main.rs:483 is off the Owns list. Forced (the check must precede `open_store()`), but the commit message does not flag it.

## What I ran
A fake `docker` recording argv; about 90 fresh workspaces; jq-built payloads piped to `tsr hook --stage …`; `tsr task export`; the checker crate; fmt, clippy, the suite. Tree clean at 79154b2; only gitignored `target/` and `crates/tsr-cli/.tsr/` were touched.
