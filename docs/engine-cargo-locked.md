# Cargo repairs with locked dependencies

The scoped Cargo adapter now admits exact-version crates.io dependencies backed by
checksums in the protected lockfile. It reuses dependencies already prepared in
the immutable worker image. Generated repairs cannot fetch packages, change
manifests, alter the lockfile, add source configuration or enlarge the file scope.

## Initialize and prepare

The example actually calls `itoa = "=1.0.18"` from editable Rust source. The
approved image already contains that version in `/opt/m3-vendor`, with Cargo's
source replacement configured in the image. No network downloads or image changes
were made for this slice.

```sh
/opt/homebrew/bin/cargo build -p tsr-cli --offline
RUN=$(mktemp -d "$PWD/target/cargo-locked-XXXXXX")
target/debug/tsr engine --directory "$RUN" project-init \
  --spec examples/engine-cargo-locked/scope.json --source examples/engine-cargo-locked
target/debug/tsr engine --directory "$RUN" project-prepare
target/debug/tsr engine --directory "$RUN" project-config
target/debug/tsr engine --directory "$RUN" request > "$RUN/request.json"
```

`project-prepare` runs only the fixed `cargo fetch --locked --offline` command in
the existing sealed runner. Despite Cargo's command name, this mode cannot fetch
from the network: both Cargo and the container prohibit it. It resolves/verifies
availability from the prepared image without compiling or executing project code.
The workspace and image root are read-only. Missing crates or an inconsistent
lockfile return a nonzero CLI exit, preserve protected bytes and leave dependency
readiness false.

Preparation stdout/stderr receipts live outside the candidate mount. A Rules
record binds successful readiness to the immutable admission, including protected
manifest/lock contents and pinned image. Repeated preparation reuses that record,
including after restart. A changed image/scope cannot reuse it. Project snapshots
still check protected bytes before edits, execution, preparation reuse and export.
Dependency-bearing proposals and execution plans require readiness; preparation
must happen before committing a repair. `request.coding.dependencies_ready`
exposes this state.

For crates absent from the image, use the separate
[checksum-verified preparation helper](engine-dependency-download.md) to download
the locked crates and produce a new local pinned image and scope, then initialize
a new run. Dependency acquisition remains separate from repair execution.
The sealed executor never gains network access.

## Repair, restart and export

The existing request/propose/coding-plan/coding-run interface and
`scripts/engine-run.py` work unchanged after preparation. Cargo tests still run
with `--locked --offline --workspace`, a read-only workspace and temporary target
output. The persistent runner retains budgets, commitments, outcomes and named
test feedback. `project-export` produces a patch against the original admitted
source and a result snapshot; it never changes the original project.

Reproduce the dependency-bearing local transport test:

```sh
TSR_ENGINE_TEST_IMAGE=sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6 \
TSR_ENGINE_TEST_KEEP=1 python3 -m unittest \
  scripts.test_engine_run.EngineRunTests.test_locked_cargo_workspace_pause_resume_and_export -v
```

It uses a credential-free loopback provider fixture, not a paid model. A partial
subtotal repair passed `subtotal` and `ledger_identity` but failed `discount` and
`floor`. The runner paused, restarted, received all four named observations, and
applied a corrective discount edit. All four tests then passed. Another restart
returned success without a third proposal. The exported patch applied to a
separate original snapshot and matched every repaired file. Original source,
manifest, lockfile and protected tests were unchanged.

[Recorded evidence](engine-cargo-locked-evidence.json) includes preparation and
restart results. The `.001`/`.002` cost entries are synthetic local billing
fixtures. Actual paid calls and spending for this slice were zero.

## Bounds and verification

Supported direct dependencies and dev-dependencies use exact stable numeric
versions (`"=1.0.18"`), with at most eight entries per section and 512 lock packages in a prepared image.
The separate downloading helper retains its 32-package cap.
All registry lock entries require crates.io provenance and a 64-digit checksum.
Git/path dependencies, alternate registries, version ranges, feature overrides,
workspace dependency inheritance and owner build scripts remain unsupported.
Existing file, source-byte, output and deadline limits still apply. Dependency
source is supplied by the image, not copied into the model context. Transitive
build scripts/proc macros, if present in prepared dependencies, execute only during
the sealed compilation stage under its existing restrictions; they do not execute
during preparation. This example exercises an ordinary library dependency.

Checks actually run:

- `cargo test -p tsr-cli --lib --test engine_cli --offline`: 49 library tests and
  one CLI integration test passed.
- `cargo clippy -p tsr-cli --all-targets --offline -- -D warnings`: passed.
- Sealed dependency-bearing transport repair/restart/export test: passed.
- Sealed `test_missing_locked_dependency_stops_preparation`: passed; unavailable
  version `1.0.999` failed preparation without changing protected files or calling
  a provider.
- Unit regressions cover version/registry/checksum restrictions, failed preparation,
  refusal of repairs before readiness, restart reuse, changed-image binding and
  protected lockfile mutation.

The existing 151 admitted corpus tasks remain available. This synthetic Cargo
repair adds usable engine capability and local execution evidence; it does not
supply broader real engine outcome evidence across that admitted corpus or finish
M3 acceptance. The previous limitation concerning deliberately forged libtest
output also remains; this work does not claim malicious-code-proof attestation.
