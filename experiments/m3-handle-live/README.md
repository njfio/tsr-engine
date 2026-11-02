# Fixed six-pair transport integration

This is a task-scoped interface validation, not the M3 acceptance cohort. The native engines, source preparation, inputs and transport dependencies are pinned by `docs/m3-handle-live-execution-freeze.json`. `freeze.py` only prepares inputs and hashes files; it does not grant a lease or make provider calls. The controller requires its explicit launch flag and creates a one-shot lease. Never run it without authorization covering this cohort.

The task is `8eb7db184391a66ae727fdfeca4197e7274592687274549d722f3a47a590feca`, a previously used constructed mutation of `njfio/rs-agent-code-utility`, file `crates/rts-daemon/src/reconciler.rs`. There are six matched old digest/new handle pairs in Record mode, in O,N / N,O / O,N / N,O / O,N / N,O order. Limits are twelve attempts, twenty calls and 200,000 generator tokens per attempt, 240 calls and 2.4M tokens overall, $5 additional including reservations within $100 aggregate, and six hours including overhead. No retries, replacement arms or adaptive prompt edits.

Outbound content is limited to that source file (including its embedded tests and subsequent model edits), the two inventoried failing-test names, the approved named-test selector, bounded test stdout/stderr, exit status/duration/output digests, task/request/state identifiers, source digest, freshness/submission status, remaining budgets and the reviewed instructions/schema. New requests also carry the opaque current handle and relative path. Host paths are redacted and credential-shaped text is rejected before credential loading or catalog traffic. Secret detection is a conservative pattern check, not an exhaustive secret scanner; file scope is separately fixed. No mutation/reversal/reference diff, unrelated file, private session, environment value or unrestricted log is sent. The existing credential is used only in the authorization header.

`native_check.py` exercises the actual native child process and exact production entrypoint/core with a local fake provider. Both credential loading and HTTP are replaced before dispatch, socket creation is forbidden, and a copied local ledger receives fake settlements. A sentinel verifies parent environment clearing; macOS may add its own encoding metadata. Inspect/Test/Stop intentionally leaves the known mutant unrepaired, so independent oracle rejection is expected. This validates integration and evidence; it does not measure model repair reliability. `finish_native.py` additionally runs the production result exporter, accounting binding, classification and freshness audit over those outputs.

Tests:

```sh
python3 -m unittest discover -s experiments/m3-handle-live -p 'test_*.py' -v
python3 experiments/m3-handle-live/native_check.py
python3 experiments/m3-handle-live/finish_native.py
```

Native tests need Docker access and at least 240 GiB initially / 200 GiB during execution. They preserve prior attempts and do not stop unrelated containers. They are one-shot fixtures; inspect and choose a new explicit fixture directory before a retry. No paid-call retry is implied.

After explicit cohort authorization, the exact launch is:

```sh
python3 experiments/m3-handle-live/controller.py --approved-paired-handle-validation
```

Do not launch directly through an arm entrypoint. Any terminal state prevents reuse. Catalogue/model/price availability is checked at dispatch; an unavailable model, changed pin, uncertain charge, containment/evidence/disclosure failure, disk or budget bound stops the cohort. Ordinary schema/repair failures retain their assigned slots. Review `docs/m3-handle-live-validation-plan.md` for the full inventory and decision rules. `report.py <terminal.json>` applies the frozen thresholds, keeping all six slots per interface in the denominator. A go authorizes no further work by itself: a TSR comparison is a separate decision.

The freshness fold is independent of candidate files but trusts canonical controller source-state checkpoints; it does not reexecute commands or detect transient changes reverted between snapshots. A classified span/size/budget refusal may leave exactly one undispatched proposal. That exception does not permit missing checkpoints or unexplained proposals. Canonical event identity, receipts and roots are verified before the fold. Each new arm's decision separately binds the complete freshness evidence manifest.
