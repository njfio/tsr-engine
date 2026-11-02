# Bounded edits with large protected assets

This slice separates stored project bytes from proposal context. An admitted
implementation window can live inside a larger Rust file while every byte before
and after it remains immutable, including inline test regions. Protected Cargo
manifests, locks and larger protected Rust files are retained in full but omitted
from the proposal's source text.

## Scope format and bounds

Add an optional owner-authored `edit_windows` map to the existing project spec:

```json
{
  "edit_windows": {
    "src/lib.rs": { "start": 100, "end": 130 }
  }
}
```

Offsets are zero-based UTF-8 byte boundaries in the **initial admitted file**;
`end` is exclusive. Compute them from the actual initial snapshot, not line
numbers or a different Git revision. Only an editable file may have a window.
The source map/import still contains the complete file. The window's original
prefix and suffix become immutable. After edits change the window length, the
controller locates the current body between those same original byte sequences.

Existing specs without windows remain supported. Limits are now:

| Item | Bound |
| --- | ---: |
| Ordinary editable file without a window | 4096 bytes |
| Current editable window | 4096 bytes |
| Full file containing an editable window | 64 KiB |
| Protected Rust source/test file | 64 KiB |
| Protected `Cargo.toml` or `Cargo.lock` | 256 KiB |
| Entire stored project snapshot | 1 MiB |
| Visible source in proposal context | 8192 bytes |
| Owner specification JSON input | 20 MiB |
| Prepared Cargo lock inventory | 512 packages |

Ordinary scopes retain the 12-file limit. Explicit paths, no symlinks, pinned
images, nonroot execution and bounded proposal/response limits remain; the
selected workspace mode below has separate file and deadline limits.
The separate downloading helper still caps downloads at 32 packages; admitting
larger locks is for an already prepared compatible image, not authorization to
download hundreds of packages.

`project request` includes source text only for the current editable window, or
ordinary small files. Protected build assets and protected Rust files larger than
4096 bytes have `source_omitted`, byte count and digest metadata instead of a
`source` field. The `source_digest` and inspection handle always bind the **whole
file and project snapshot**, not just the visible window. Responses continue to
carry ordinary exact edits; generated text cannot change window bounds.

The same boundary checks run during initial admission, patch/set validation,
snapshot integrity checks, restart inspection and export. An edit to an inline
assertion, its `#[cfg(test)]` attribute or surrounding code outside the window is
rejected. External modification of those bytes also invalidates the snapshot.
This protects bytes and scope; it does not add a claim that hostile implementations
cannot manipulate test behavior or forge process output.

## Selected RTS task and observed progress

Selected admitted task: `80d827e3a4c15b244e211567a7bfe6679035d07736e6ea3db7c08c1cdc64f4ad`,
named test `tests::test_supported_languages`, in `rust_tree_sitter`.
Its existing prepared dependency image remains
`sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6`.
No dependency/image rebuild was performed.

The earlier [precheck](engine-rts-window-progress.json) is historical. The
workspace adapter now accepts the actual inherited package metadata and locked
registry dependencies. It imports all 47 tracked package files, the unchanged
workspace manifest, resolver targets for the other members, and the existing
prepared lock: 57 files total. Only the expression in `supported_languages` is
editable. Inline tests and the rest of that file remain immutable.

Owner-authored `named_package` enables this bounded mode. It requires Cargo,
exactly one named library test, and an explicit window for each editable file.
It allows up to 512 files, 512 KiB per protected file, a 16 MiB total snapshot and
600 seconds per command. Editable-file and proposal/response bounds stay the
same. Only editable windows enter model context; the entire protected graph is
bound by the snapshot digest. Workspace manifests and lock bytes remain protected;
Cargo's `--locked --offline` resolver checks inheritance against the prepared image.
Git dependencies, external path dependencies and selected-package build scripts
are refused. This is deliberately a selected-library adapter, not support for
arbitrary Cargo layouts or a complete import of every other workspace member.

The fixed command selects `-p rust_tree_sitter --lib` and the exact owner-named
test. Cargo uses quiet compilation plus explicit `--format=pretty` test output,
so the checker still requires a complete named test inventory and summary.
The first execution exposed that Cargo quiet mode also changes libtest output;
the checker censored that receipt as unavailable. It is preserved under
`target/engine-rts-real-1`, and a fresh run uses the explicit output format.

Build output lives in a fresh controller-created directory for each commitment,
mounted at `/tsr-build`. It is never shared between attempts or exposed through
proposal context. Source is mounted read-only during compilation and execution;
the immutable image supplies already prepared dependencies. There is no network,
privileged execution or host execution of candidate code. A periodic monitor
stops a run exceeding 8 GiB or 100,000 entries and refuses symlinks/special files;
this is a monitored threshold, **not a filesystem hard quota**, and transient
overshoot is possible. Retained output consumes disk until operator cleanup.

Run the local replay with a fresh output directory and the preloaded image:

```sh
cargo build -p tsr-cli --offline
python3 scripts/engine-rts-task-replay.py --out target/engine-rts-real-2 --phase init
python3 scripts/engine-rts-task-replay.py --out target/engine-rts-real-2 --phase fail
python3 scripts/engine-rts-task-replay.py --out target/engine-rts-real-2 --phase correct
python3 scripts/engine-rts-task-replay.py --out target/engine-rts-real-2 --phase verify
```

The correction is an explicit local replay of the original base expression, not
a model-generated repair. The verify phase checks persisted failure feedback,
all protected bytes, the unchanged source repository, restart status, and patch
application to a separate copy of the admitted mutant. It writes
`docs/engine-rts-real-task-result.json` only after those checks pass.
This selected test does not substitute for the full workspace oracle, a baseline
cohort, or M3's complete acceptance criteria.

## Verified real result

[Recorded result](engine-rts-real-task-result.json): the deliberately empty
implementation failed the genuine assertion in 68,467 ms (exit 101). The next
process received persisted `failed_tests` feedback. Restoring the base expression
passed in 75,214 ms (exit 0). These durations include fresh compilation. Each run
executed one test and filtered out 267; neither is a full-suite result.

Verification confirmed all 56 protected files unchanged, the repaired file exactly
equal to the original base, an unchanged source repository, two separate build
directories, and a patch that applies to the original mutant and reproduces the
repair. Restart reports `passed` without dispatch. No paid calls were made.

## Verification

Current workspace adapter checks:

- `cargo test -p tsr-cli -p tsr-corpus --lib --test engine_cli --offline`:
  55 CLI library tests, 2 CLI integration tests and 26 corpus tests passed;
  one existing corpus test ignored.
- `cargo clippy -p tsr-cli -p tsr-corpus --all-targets --offline -- -D warnings`:
  passed.
- Changed Rust files pass rustfmt; `git diff --check` passes.
- The four real replay phases above passed in `target/engine-rts-real-2`.

Earlier window-only checks (historical):

- `cargo test -p tsr-cli --lib --test engine_cli --offline`: 54 library tests and
  two CLI tests passed.
- `cargo clippy -p tsr-cli --all-targets --offline -- -D warnings`: passed.
- Unit regressions exercise protected inline assertions/attributes, prefix/suffix
  integrity, stale handles after window edits, restart, large asset omission,
  malformed UTF-8 offsets, file bounds and the 512-package lock cap.
- A separate sealed **synthetic** Cargo regression exercises a large protected
  lock and a large Rust file with one small editable window. A proposed inline-test
  edit is refused. An implementation repair fails the original inline test;
  a restarted request receives that failure; a corrective edit passes. All
  protected bytes stay unchanged, and the exported patch applies to an original
  disposable copy and reproduces the repaired source.

Reproduce that regression with the existing preloaded image:

```sh
TSR_ENGINE_TEST_IMAGE=sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6 \
TSR_ENGINE_TEST_KEEP=1 python3 -m unittest \
  scripts.test_engine_run.EngineRunTests.test_edit_window_protects_inline_tests_through_restart -v
```

[Regression evidence](engine-window-cli-evidence.json) is explicitly synthetic,
not an RTS task outcome. No paid calls, external source transmission, source-repo
changes, security-setting changes, global Docker restart, push or publication
occurred. The 151 admitted corpus tasks remain intact.

## Subsequent full-suite closeout

The same exported RTS patch now has a [verified full-workspace closeout](engine-rts-full-suite.md):
999 passed, 0 failed, 15 ignored across 96 groups, with all 479 source files
unchanged. The named-test evidence above remains the historical engine replay;
the separate full-suite closeout does not claim a model-generated repair or a
cohort benchmark, and does not automatically broaden the engine's named-test gate.
