# Persistent coding loop: bounded execution slice

For novel generated exact edits without a fixed repair catalog, see
[the generated-edit integration](engine-generated-coding.md). The original mode
and evidence below remain available.

The engine now connects admitted action IDs to actual sealed edits and independently
captured test outcomes. This is a two-file Rust demonstration, not real-corpus M3
acceptance or a general autonomous repair agent.

`coding-init` admits one goal and two fixed exact edits to `implementation.rs`.
`request` includes the source, admitted actions, frozen expectations and previous
outcomes. A typed local response proposes existing action IDs; `propose --apply`
selects and commits through the existing expected-value calculation. `coding-plan`
shows the exact source digest, replacement, protected test digest and commands.
`coding-run --commitment HASH --expected-source HASH` rechecks that binding, records
an execution intent, and invokes the existing `SealedRunner` for the edit, compile
and test stages. It reuses `prepare_edit`, `persist_run` and the named-test parser.

Only the disposable `coding-work` directory is mounted. The event store and
`coding-receipts` remain outside it. The pinned, preloaded Docker image runs as a
numeric nonroot user with no network, a read-only root, no added capabilities,
no new privileges, resource limits and a 30-second deadline per command. Proposal
text cannot supply a shell command, edit, test or execution grant. The edit script
is fixed controller code and receives only a reviewed replacement as an argument.
Test source and the allowed implementation states are checked before execution
and after every stage. Manual `observe` is refused for a coding instance.

A complete named-test report produces a Rules-sourced return, frozen-forecast
residual and criterion contrast. Timeouts, unavailable execution, incomplete
output and integrity failures are censored: they clear the pending attempt but
provide no success/failure training. Censored reasons appear in the next request.
Every attempt has a durable intent; an interrupted intent is not automatically
retried. Reconciliation after a controller crash remains manual.

## Run

Build with `/opt/homebrew/bin/cargo build -p tsr-cli --offline`, then:

```sh
python3 scripts/engine-coding-smoke.py \
  --image sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6 \
  --output /tmp/engine-coding-trace.json
```

The image must already exist locally; no image is pulled. The helper creates a
fresh directory under `target`, starts a new CLI process for every operation,
uses local replay proposals, and leaves all evidence available for inspection.
It does not contact a model.

## Observed evidence

[Captured trace](engine-coding-trace.json) includes the actual commands, plans,
run receipts, test stdout, selections, and restarted request. On 2026-10-18:

| Attempt | Actual test return | Folded success estimate | Next selection |
| --- | --- | --- | --- |
| `increment`: `a - b + 1` | exit 101, 0 passed / 2 failed | 1/2 → 1/3 | `correct` |
| `correct`: `a + b` | exit 0, 2 passed / 0 failed | 1/2 → 2/3 | retained in restarted state |

Both proposals offered the same two actions. The observed failure changed their
ranking; the corrective action was not scheduled by cycle number. Protected test
SHA-256 was identical before and after. Six sealed commands ran; zero provider
calls were made.

Validation:

- `cargo test -p tsr-cli --offline --lib --test engine_cli`: 37 library tests and
  1 CLI integration test passed. Includes rejection of manual success and stale
  source, protected-test checks, named-result classification, and censored
  infrastructure returns without prediction training.
- `cargo clippy -p tsr-cli -p tsr-batch --all-targets --offline -- -D warnings`: passed.
- The Docker smoke command above: `failed_tests`, then `passed`; restart selected
  `correct` after the first failure; protected tests unchanged.

Remaining limits: fixed local action catalog and fixture, no model-generated
patch execution, no live provider codec/dispatch in this slice, and no real
corpus claim. Selecting an edit already present is rejected as a no-op. Initial
fixture/configuration creation spans filesystem and database writes; a crash
there requires a fresh directory. Receipt storage uses the existing corpus
persistence implementation; this is not a hostile-host attestation system.
