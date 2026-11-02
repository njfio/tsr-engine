# Approved M0 repair verification

2026-10-22. User approved exactly `exec.rs`, the refusal test in
`m0_forecast_default.rs`, and scratch-path reuse in `m0_harness.rs`.
Existing assertions, frozen fixtures and hooks remain unchanged.

## Changes

- Refusal test scopes an empty PATH to its final child invocation. No global environment
  mutation or production runtime-discovery change.
- Harness creates each scratch path once, retaining every structural comparison.
- RealRunner retains a created container until inspection, starts it attached, and reads
  daemon-owned State metadata. Only a completed lifecycle with matching client exit and
  successful cleanup produces RunOutput. Missing executable/startup failure, unavailable
  metadata, interrupted attachment or unconfirmed cleanup remains Unresolved. Candidate
  stdout cannot supply this evidence. Genuine command 127 is not special-cased as failure
  to start. The creation command preserves containment arguments and disallows image pulls.
- Unresolved diagnostic wording no longer asserts that every uncertain outcome was a
  startup failure. The existing public error variant and outcome sentinel remain compatible.

## Verification

Fourteen executor unit tests passed, including new lifecycle metadata and injected runtime
sequence tests for Docker/Podman argument construction, startup failure, real command127,
ordinary exits, attachment mismatch, inspection failure and cleanup failure. Podman was
not available for live compatibility verification.

Both exact integration tests were run with normal installed-runtime discovery and supported
permission. The refusal test passed. The harness now reads both generated fixtures and
passes real-versus-real comparison, then fails the unchanged real-versus-fake comparison:
Debian cannot start cargo, so real events are Unresolved while the fake expects Executed0.
This is a newly exposed prerequisite/assertion mismatch, not permission to change assertions
or install a toolchain. A real Rust-capable image alone may still produce different logs
from the fake canned output; matching that digest is not established. The harness also
starts in an otherwise empty scratch directory: `hook.rs::selftest_inner` uses that
directory as workspace and does not create a Cargo project or parser package. Therefore
installing Cargo alone would not satisfy this test. A separate reviewed self-test
fixture/contract repair is needed; this approval permits neither changing the hook nor
weakening the real-versus-fake assertion.

Four no-network/no-pull live probes used the existing immutable Debian image in disposable
workspaces: shell exit0, exit7 and exit127 produced Executed with the matching status;
a nonexistent executable produced Unresolved. Evidence is under
`target/m0-approved-repair/live-exits/results.json` and each case's SQLite store. This
used the repaired lifecycle code before the diagnostic wording-only adjustment.

The probe also exposed an unchanged protected CLI issue:
`main.rs::run_exec` clamps `ExecOutcome.exit_code == -1` to shell exit0. The ledger is
honest, but a shell caller can misinterpret unresolved execution as success. Fixing that
requires a separate narrow exception for the M0 exec arm in `main.rs`: map the negative
unresolved sentinel to a documented nonzero CLI status, preserve real command0 and127,
no-runtime3 and no-session2, with focused CLI regressions. No such edit was made.

## Limits

- A lost/malformed successful create response remains explicitly unknown; no start follows,
  but an unidentified stopped container may remain. Do not claim universal cleanup after
  interruption. No automatic retry is added.
- A killed driver can leave an owned container; lifecycle durability beyond this bounded
  synchronous repair remains a separate concern.
- No provider calls, downloads, credentials, security changes, hook changes or publication.
- Complete offline Rust workspace sweep: **971 passed / 1 failed / 11 ignored**, exit101,
  no timeout, 764.394 seconds. Sole failure:
  `m0_harness::selftest_is_reproducible_structurally_excluding_duration_ms_by_name`,
  now at the real-versus-fake assertion. No tests or assertions were skipped or weakened
  to obtain this result. Prior aggregate remains 966 / 2 / 11 as historical evidence.
- Commands: `rustup run 1.90.0 cargo test --offline --workspace --no-fail-fast`;
  focused `cargo test --offline -p tsr-cli --lib exec::tests` and both named integration
  tests. Logs and bounded-run metadata: `target/m0-approved-repair/`.
- Independent code/evidence review found no remaining blocker within the approved scope.
  Formatting checks for the three edited Rust files and `git diff --check` passed.

Clippy (`rustup run 1.90.0 cargo clippy --offline -p tsr-cli --lib -- -D warnings`)
passed. All owned test sessions terminated; final process check found no matching
Cargo/Rust test processes. `lefthook` remains unavailable; explicit checks above ran.

## Approved extension, 2026-10-23

The user approved `main.rs::run_exec`, the harness's disposable parser setup, a new
`m0_exec_status.rs` integration test, and documentation. This supersedes the two scope
blockers above; earlier failures/receipts remain historical evidence.

`run_exec` now maps the negative unresolved sentinel to shell exit **1**, with an
explicit diagnostic. Genuine command exits 0–255, including127, retain their status;
no-session2 and no-runtime3 remain intact. The ledger still distinguishes a real
command failure from an unresolved execution even when both have shell exit1.

Each real self-test now gets a dependency-free synthetic Cargo package named `parser`
in its own scratch directory. A custom Rust test harness asserts positive, negative,
malformed and empty integer input behavior; only after those checks pass does it emit
exactly `test result: ok`. Project-local Cargo quiet configuration removes routine
progress chatter, not assertion failures. No cargo shim, output rewriting, or digest
derivation from actual output is used. This is a deliberately synthetic lifecycle
contract, not validation of a production parser or the broader TSR semantics.

The child uses the already-installed `tsr-rust:1.90` image by immutable ID:
`sha256:cff6dcceb8b9d203a818b7ff104b89a6aaa353f8576850ca4f58903539417945`.
Production's default image is unchanged. Execution remains no-network/no-pull with the
same read-only root, workspace mount, capabilities and resource limits. Hooks and all
frozen records are unchanged. The existing 19-event assertion and complete structural
comparisons remain unchanged.

Focused real-runtime checks passed:

- Two real self-test repetitions match each other and the FakeRunner reference.
- Deliberately incorrect expected parser result produces assertion output, nonzero
  command status and an Executed failure, with no success marker. An infrastructure
  failure alone cannot satisfy this negative control.
- `m0_exec_status`: shell0, shell7, shell127 match Executed records; a missing executable
  returns1 with an Unresolved record and no Executed event. No-session2 and isolated
  no-runtime3 produce no execution records.

Evidence: `target/m0-approved-extension/focused-results.json` and adjacent per-test
logs. Independent source review found no blocker. Live checks used existing Docker;
no Podman execution claim is made. Tests print an explicit skip if no runtime binary
is available; a detected runtime with a missing pinned image fails rather than pulls.

The single full offline workspace sweep is separately bounded to 1200 seconds, with
start/result metadata and complete log under `target/m0-approved-extension/`.

Final extension sweep: **975 passed, 0 failed, 11 ignored**, exit0, no timeout,
476.916 seconds. Command: `rustup run 1.90.0 cargo test --offline --workspace
--no-fail-fast`. This is one complete run, not combined focused counts. The original
971/1/11 run above remains historical. No tests were newly ignored or assertions
weakened. Passing this suite establishes regression coverage, not semantic TSR
superiority, mature operational calibration or general autonomy.

Final checks: Clippy for the binary and both affected integration targets passed with
`-D warnings`; formatting checks for edited test files and diff whitespace checks
passed. Existing unrelated formatting was not rewritten. Independent code and evidence
reviews found no blocker. All owned test sessions terminated and a final process check
found no matching Rust test processes. `lefthook` remains unavailable; explicit checks
above were used rather than installing software.

Next useful offline step: prepare the exact PRD B9.1 two-desires acceptance scenario
and evidence checklist using existing mechanisms. This clean regression suite does not
supply that prospective semantic evidence or authorize another model/project run.
