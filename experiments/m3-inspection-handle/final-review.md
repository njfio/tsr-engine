# Independent native-harness and auditor review

Reviewed 2026-10-18 without running containers, candidate commands, inference, or cleanup. The separate engine is reported built successfully; this review inspected source and completed saved records.

The current summary contains seven completed arms: record/rules accepted with 24 events each and `oracle:true`; multiple/truncation unavailable with four events each; stale, foreign and wrong-path rejected at the native boundary with process exit 2 and no final result. Saved source hashes show the stale case retains the first successful edit, while foreign/wrong-path retain the original source. The protected-test case was still pending when checked; eight-arm completion is not yet attested by this review.

The harness uses actual ProcessBroker environment clearing, a scripted offline codec, fresh stores/workspaces and sealed execution. It exercises submitted-handle forwarding and native stale/cross-run/path checks. Successful fixtures use inspect/test/edit/test/Stop, in both modes. This is meaningful synthetic integration evidence, not a provider-compatibility or model-reliability measurement. The initial failed setups remain separate from this run.

## Findings and required limits

1. `protected_test` currently checks only process exit 0 and outcome rejected. Before claiming the protected-test case passed for its intended reason, require the saved exact protected-evidence tamper closure/path (and absent acceptance), rather than permitting an unrelated rejection to satisfy the assertion. This can be checked from terminal evidence without rerunning the fixture.
2. `audit.rs` checks canonical event hashes and linear parents, task identities, trusted receipt source, receipt binding, referenced prior inspection digest, submitted proposal/candidate argument hashes, end roots, and available final-oracle receipt verification. This addresses the prior missing issuance mapping. However, it does not independently replay handle invalidation/latest issuance or compare each edit's request source state against the receipt's source state; boundary records omit that state. Describe it as issuance/proposal and oracle evidence verification, not a complete independent freshness replay. Dynamic stale/foreign/wrong-path tests provide separate support.
3. Only record/rules successful paths invoke final oracle receipts. Pre-dispatch codec failures and native edit-precondition failures legitimately have no such receipts; their absence is not success. The protected-test tamper guard may reject before oracle execution and should be labelled accordingly.
4. This harness bypasses provider dispatch and the normal provider billing transport. Socket/credential functions are disabled and the harness checks original ledgers and frozen pins; those checks cannot demonstrate live provider schema adherence or better repair yield. No live adapter/controller is made launch-ready by these fixtures alone.

No fail-open native binding defect was identified in this bounded review. No implementation was changed by the reviewer. Final eight-arm summary and the protected-test closure need terminal confirmation before declaring the prescribed native matrix complete.

## Terminal follow-up: eight arms complete

The completed permanent export now contains all eight expected arms across seven groups. Its 147 listed file hashes independently match. Record and Rules each have one issued receipt, one bound edit, and verified successful final-oracle evidence. Multiple-call and truncation cases are unavailable without edits; stale, foreign and wrong-path cases fail at the native boundary as expected. The protected-test case is now confirmed rejected with exactly `tamper:changed full-suite test resource `tests/answer.rs`` and `oracle:null`.

`finish_audit.py` explicitly asserts that protected-resource closure and path, resolving finding 1 above. It runs typed canonical-event/chain, issuance/proposal and available oracle-receipt audits for every arm, and requires a corrupted issuance export to fail. The terminal summary records that negative check, unchanged 5,362 frozen pins and zero paid dispatches. The scripted harness's terminal checks preserve ledgers and running-container inventory; this reviewer did not rerun workloads or inspect live runtime independently.

The prescribed synthetic matrix is complete. Finding 2's scope limit remains: the auditor does **not** independently replay source freshness/invalidation. The dynamic stale/foreign/wrong-path cases support those properties separately. Nothing here demonstrates improved model reliability, live provider compatibility, or live-launch readiness. No further implementation changes or runtime actions were performed by the reviewer.
