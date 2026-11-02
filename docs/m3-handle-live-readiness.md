# Six-pair interface validation readiness

The scope is the task-scoped transport wrapper and execution freeze. This does not complete M3 or provide model-repair evidence. M3's PRD acceptance still requires at least 150 tasks with family/lineage, baseline numbers with intervals, and logged selections/tensions.

## Frozen task and outbound content

- Repository lineage: `https://github.com/njfio/rs-agent-code-utility`.
- Task: `8eb7db184391a66ae727fdfeca4197e7274592687274549d722f3a47a590feca`.
- Source: `crates/rts-daemon/src/reconciler.rs`, initially 11,893 bytes, SHA256 `5e49fb5cd5e47230f8e4c28aee0cebdf5e1f4156ffc3adf3d8d8eccb9e043c07`.
- This is a reused constructed mutation task, not a holdout or an independently mined historical fix.

The provider may receive the allowlisted source including embedded tests and subsequent candidate edits; the names `reconciler::tests::throttle_passes_through_under_cap` and `reconciler::tests::throttle_sleeps_when_bucket_drained`; the `named-failing-test` selector; visible test output capped at 2 KiB per stdout/stderr stream plus exit status, duration and output digests; native request/task/state identifiers, state version/source digest, inspection/test freshness, submission status and remaining budgets; the current opaque inspection handle and relative path in the new arm, or inspected full-file digest in the old arm; and the fixed instructions, schema and model settings. Native inspection is capped at 64 KiB, with a 56 KiB displayed source excerpt.

Host paths are redacted. Credential-shaped content is rejected before credential/catalog access. Neither environment values nor hidden mutation/reversal/reference patches, oracle-only files, private sessions, other source files or unrestricted logs are in the payload. The source allowlist and metadata shape are enforced separately from the conservative credential-pattern check; that check is not an exhaustive secret detector.

Endpoint: `https://api.cheaperinference.com/v1/chat/completions`. Model: `deepseek-v4.1-flash`. Catalog availability and price must pass the existing preflight at execution time. Temperature, reasoning and routing remain provider defaults, so this cannot identify a pure token-field effect or guarantee a fixed upstream model revision.

## Allocation, decisions and stopping

Six matched pairs, twelve fresh Record-mode attempts: O,N / N,O / O,N / N,O / O,N / N,O. Each attempt has 20 calls, 200,000 billed generator tokens, 16,384 maximum output tokens/call, 15 tool-minutes, 1,800 seconds and 131,072 patch bytes. Global limits: 240 calls, 2.4M tokens, 6 hours including overhead, and $5 additional including reservations within $100 aggregate. Settled baseline is $0.544194; maximum cumulative settled/reserved amount is $5.544194. No retries or replacements. Initial/live free-space floors are 240/200 GiB.

A go to a separately scoped TSR comparison requires all 12 terminal classifications, at least 5/6 new-arm oracle completions and 3/6 verified acceptances, no new malformed/stale endings or unexplained evidence failures, at least 3 new attempts with independently verified second-or-later edits, and new completion/acceptance counts no lower than old. Missing slots remain in the six-per-interface denominators. Insufficient multi-edit evidence is inconclusive; ties show viability only. `experiments/m3-handle-live/report.py` encodes those thresholds.

Stop on changed pins, disclosure, containment/evidence failure, uncertain billing, unavailable model, disk or budget bounds. A terminal state forbids restart. Ordinary classified schema, span and repair failures retain their slot and may continue to the next fixed attempt. The controller reserves before paid dispatch, checks settlements, and binds exported evidence plus the independent freshness manifest before finalizing each decision.

## Validation and limitations

Offline integration is complete: 14 wrapper/lifecycle/report tests, 15 existing replay/codec regressions, and 9 existing controller regressions passed. Both actual native arms completed Inspect → Test → Stop with three fake settlements each and a checked negative oracle result. The new arm passed canonical auditing and independent freshness replay (two source checkpoints, no edits). The production exporter, accounting binder and terminal classifier passed for both arms. See [native summary](m3-handle-live-offline-evidence/summary.json), [controller checks](m3-handle-live-offline-evidence/controller-summary.json), [wire inventory](m3-handle-live-offline-evidence/wire-inventory.json), and [execution freeze](m3-handle-live-execution-freeze.json). The offline native test uses the real engine process boundary, exact production entrypoints and this frozen real-source task. Credential access/HTTP are replaced by a local fake provider, network socket creation is disabled, and fake charges are written only to copied ledgers. Inspect/Test/Stop intentionally leaves the mutant unchanged. A checked negative oracle result demonstrates rejection-path integration, not successful repair.

The first offline attempts stopped before fake dispatch because macOS adds `__CF_USER_TEXT_ENCODING` after environment clearing. The corrected wrapper permits that OS metadata key, HOME/PATH and Python's locale key; no environment value enters the payload. Failed fixture directories are preserved. The parent sentinel remains absent at the child boundary.

Freshness replay verifies canonical controller evidence independently of candidate files, but trusts before/after source-state checkpoints. It does not reexecute tests or detect transient source mutations restored between snapshots. The earlier synthetic edit/handle safety suite remains separate from this real-source transport fixture.

No paid calls, push, PR, merge, deployment, Docker restart or unrelated-container changes are part of this offline completion. The six held build directories and unrelated work remain untouched.

The live six-pair cohort has not run. It remains a separately scoped execution decision under the detailed validation plan; no model reliability go/no-go result exists yet. The finalized wrapper and inputs remove the prior offline integration blocker. No push, PR or deployment is needed to execute the frozen local plan once authorized.
