# v3 state and submission guidance audit

No harness correctness defect is demonstrated by the retained evidence. No code was changed and no provider request was made.

The exact persisted sequence is inspect → successful exact edit → visible named test exit0 → another decoded edit rejected at the native source-digest precondition. Only the first three actions became proposals. There was no second inspection, Stop, task closure, final oracle, or result row.

## Digest comparison

| Value | Evidence |
| --- | --- |
| Initial inspected file | 104 bytes; digest `13349785e429a8c46716356198dd838beea4b3faffc72fed5e0636bade120ad3` |
| Retained repaired file | 105 bytes; digest `6ed0d735c5a2a1dc1d0020f1df2c1fb8bf4aacb620a95bdf174efb5ddc67f42f` |
| Rejected proposed digest | Not retained; exact value unknown |

The sole persisted inspection digest matches the original red source read from its pinned Git snapshot. The repaired file has a different digest. The native error and small regular file establish that the rejected edit's supplied digest did not match the current source. They do not establish that it equaled the initial digest, the aggregate source-state digest, an example, or any other particular value. No intent is inferred.

Raw outgoing requests and rejected proposal bodies were deliberately not logged. The following account of what the model was sent is therefore reconstructed from verified frozen code, not a captured wire request. The engine executable and relevant construction/adapter sources still match the live freeze.

## State supplied by the harness

`broker_v2.rs:489` records a full-file digest during inspection. Lines511–518 apply an exact edit but do not refresh the retained inspection. Thus **the model is not automatically supplied the new per-file digest after editing**. It must inspect again before another edit.

This retained inspection is explicitly distinguishable from current evidence: `RecentContext::progress` compares its recorded source state with the current source state and sets `inspection_current=false` after a source change. `source_state` is current aggregate identity, not a file digest; the adapter explicitly says not to use it as expected_digest. The context says to use controller progress for freshness and repeat stale or missing evidence.

After a test, the engine includes its exit code and bounded stdout/stderr. It sets `test_current=true` if the test did not change source. This flag is freshness, not success; success is the displayed exit/output. The persisted run records exit0. The test is the known dependency-free fixture, but exact before/after progress flags were not retained separately, so they are not presented as direct trace observations.

## Submission guidance

`Progress.submission` explicitly says Stop submits the current candidate for independent final evaluation, visible test success is not acceptance, and Stop consumes a remaining call. The system instruction also tells the model to choose Stop when finished. That instruction does not compel automatic submission and does not expose hidden-oracle results. A current visible pass permits the model to choose Stop; it does not authorize the controller to submit on its behalf.

The request construction therefore supports a coherent workflow: after a successful edit/test, choose Stop if finished; if another edit is needed, inspect again for its current full-file digest. An old inspection accompanied by a stale flag is not a promise that its digest can be reused. Automatically refreshing the inspected file/digest could be a separately evaluated usability change, but this attempt does not demonstrate that it is a required correctness repair.

Independent review confirmed these findings and the evidence limits. The exact proposed-versus-provided digest comparison cannot be completed retrospectively because the rejected value was not retained.

## Disposition and recommendation

Preserve this as a model edit-precondition failure with incomplete end-to-end execution. Do not silently repair its digest, auto-submit, weaken exact-edit checks, or repeat the paid attempt. The larger comparison remains pending.

For any separately authorized future interface evaluation, first add bounded content-free evidence such as inspection freshness, test freshness/exit category, and booleans comparing a proposed digest with the latest inspection and current file. That would distinguish stale-digest reuse from another wrong digest without retaining provider bodies. Treat any fresh-digest presentation change as a new design experiment, not a proven repair of this failure.

Accounting is unchanged: five cumulative stage requests, $0.001342 stage spend, $0.231086 aggregate, zero outstanding. Supporting hash checks and explicit unknowns are in `m3-integration-v3-state-audit.json`.
