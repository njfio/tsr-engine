# Scoped Cargo workspaces

Use the [unified project CLI](engine-project-workflow.md) for the current entry point.

The project engine can import and repair small Cargo workspaces.
[Locked dependency support](engine-cargo-locked.md) extends the original
dependency-free adapter described below. `project-init --source` reads only paths listed in an owner-authored
scope file, copies them into `coding-work`, and saves the complete admission.
The original project is never mounted into an evaluator or edited. The existing
proposal, commitment, sealed execution, receipt and run/resume paths are reused.

## Clean initialization

Build the controller, then initialize the supplied two-package synthetic example:

```sh
/opt/homebrew/bin/cargo build -p tsr-cli --offline
RUN=$(mktemp -d "$PWD/target/cargo-project-XXXXXX")
target/debug/tsr engine --directory "$RUN" project-init \
  --spec examples/engine-cargo/scope.json --source examples/engine-cargo
target/debug/tsr engine --directory "$RUN" project-config
target/debug/tsr engine --directory "$RUN" request > "$RUN/request.json"
```

For another project, author a scope JSON with `cargo: true`, an empty
`probe_entry`, explicit `editable` and `protected` path maps, a goal, pinned image,
nonroot user and command deadline. Map values can be empty placeholders when
using `--source`; initialization replaces them with the exact source bytes.
Without `--source`, map values are the actual contents to copy. Include every
required Cargo manifest, the root lockfile, library source and integration test
source. Manifests, lockfile and integration tests must be protected. Each `cases`
entry has a globally unique test `id`, empty `args` and empty `expected_stdout`.
The example scope is a complete template. No repository crawling or host Cargo
metadata invocation is needed.

Admission currently allows standard library packages, explicit workspace members,
standard `src/lib.rs`, Rust modules and protected integration tests. It refuses
dependencies (including path dependencies), build scripts, proc macros, custom
harness/target configuration, arbitrary manifest fields, symlinks and paths outside
the scope. Limits remain 12 files, 8 editable files, four expected tests and 20 seconds per
command. [Editable-window and protected-asset bounds](engine-protected-windows.md)
now distinguish stored bytes from the 8192-byte visible source context. Test IDs allow alphanumeric/underscore/hyphen segments separated by `::`.

## Propose, run, resume and export

Use `request`, a typed `exact_edit` or `edit_set`, `propose --apply`, `coding-plan`
and `coding-run` as documented in [the project guide](engine-project.md).
Inspection handles bind the whole current snapshot. The unchanged
`scripts/engine-run.py` also accepts Cargo contexts and persists its transport
journal across invocations. No model call is required for the local replay below.

After any completed repair, export a reviewable patch and result snapshot:

```sh
target/debug/tsr engine --directory "$RUN" project-export
```

This writes `exports/<snapshot>.patch` and `exports/<snapshot>.json` beneath the
engine directory. The patch is a whole-file unified diff against the admitted
original bytes. Export checks protected-file integrity. It does not apply the
patch to the owner's project or certify that the latest edits passed; the JSON
contains the saved engine state so the reviewer can inspect outcomes.

The trusted adapter uses the existing M3 `full_test_argv()` with fixed additions:

```text
cargo test --locked --offline --workspace --no-fail-fast -j 1 --target-dir /tmp/tsr-cargo-target --no-run
cargo test --locked --offline --workspace --no-fail-fast -j 1 --target-dir /tmp/tsr-cargo-target -- --test-threads=1
```

Both Cargo stages mount the disposable workspace read-only; target files stay in
ephemeral container `/tmp`. The preceding fixed edit stage writes only validated
path/content pairs. No proposal supplies shell commands. The existing sealed
runner supplies no network, a pinned preloaded image, nonroot user, read-only
container root, resource limits and bounded capture. Each command has its own
fresh container, so the test stage recompiles. Nothing downloads dependencies.

Raw stdout/stderr receipts are stored outside the candidate mount. The controller
requires complete libtest groups, the exact expected test inventory, consistent
failure counts and a matching exit code. Named passed cases enter the existing
return/contrast mechanism. Missing, truncated, inconsistent, timeout and build
failure evidence is censored, not accepted as a successful repair. Build errors
currently stop the automatic loop rather than becoming repair feedback.

## Reproduction and evidence

With the preloaded image in the example scope, the direct replay is:

```sh
python3 scripts/engine-coding-smoke.py \
  --project-spec examples/engine-cargo/scope.json --source examples/engine-cargo \
  --output docs/engine-cargo-replay.json
```

The actual transport pause/resume regression uses a credential-free loopback
provider fixture and the same sealed Cargo executor:

```sh
TSR_ENGINE_TEST_IMAGE=sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6 \
TSR_ENGINE_TEST_KEEP=1 python3 -m unittest \
  scripts.test_engine_run.EngineRunTests.test_cargo_workspace_pause_resume_and_export -v
```

Both replays observed a partial subtotal repair with `subtotal` and
`ledger_identity` passing while `discount` and `floor` failed. After restarting,
the second request contained these four named observations. The corrective
discount edit passed all four tests across both packages. A subsequent restart
returned success with no third completion. The transport test also applied the
exported patch to a disposable copy and compared every admitted file to the
repaired snapshot. Original and protected files were unchanged.

See [direct execution receipts](engine-cargo-replay.json) and
[transport/restart evidence](engine-cargo-transport-evidence.json). There were
**zero paid calls**. The transport evidence's `.001`/`.002` accounting amounts are
synthetic loopback billing fixtures, not spending or claims about model pricing.

Validation performed:

- `cargo test -p tsr-cli --lib --test engine_cli --offline`: 47 library tests and
  one CLI integration test passed.
- `cargo clippy -p tsr-cli -p tsr-batch --all-targets --offline -- -D warnings`:
  passed after fixing one unnecessary unwrap.
- Sealed Cargo direct replay: failed tests → passed.
- Sealed Cargo transport pause/resume/export regression: passed.
- `python3 -m unittest scripts.test_engine_run -v`: 10 passed, four sealed
  tests skipped in this invocation; the Cargo sealed test was run separately above.
- `python3 -m unittest scripts.test_engine_provider_codec -v`: four passed.

This is a working bounded Cargo adapter, not general Cargo repository support.
The test protocol reports process output; it is not cryptographic attestation
against a malicious implementation that deliberately forges libtest output.
Protected test bytes and controller receipts are independently preserved, but
that alone does not solve semantic oracle spoofing. No sandbox escapes or hostile
host payloads were exercised. The examples and proposals are authored synthetic
fixtures. There are already 151 admitted corpus tasks; this example does not
supply broader real engine outcome evidence across that corpus required to
declare M3 acceptance complete.
