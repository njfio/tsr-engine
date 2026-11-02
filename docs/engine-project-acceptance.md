# Independent project acceptance

A passing test receipt is an observation about the admitted tests. It does not establish
that the user's goal is complete. Project status now reports `execution_status: passed`
with `status: tests_passed_unaccepted` until an independent operator review accepts the
current source. `goal_completion` appears in project status, persisted-state views and
export metadata. Legacy projects default to unresolved acceptance.

## Operator workflow

Keep acceptance inputs outside `coding-work`. Define a JSON object with 1–16 requirement
IDs and descriptions (each nonempty and at most 512 bytes). For example:

```json
{"coverage":"Preserve all required API behavior","metadata":"Return accurate metadata"}
```

Register and inspect it:

```sh
tsr engine --directory RUN project acceptance requirements --file requirements.json
tsr engine --directory RUN project acceptance status
```

Independently inspect the result against those requirements. Create a review JSON using
**the exact** `snapshot`, `requirements_revision` and `test_receipt` returned by status:

```json
{
  "snapshot": "<current snapshot hash>",
  "requirements_revision": "<current requirements event ID>",
  "test_receipt": "<current test receipt event ID>",
  "decision": "reject",
  "evidence": {
    "coverage": {"met": false, "evidence": "Independent inspection found missing API entries."},
    "metadata": {"met": true, "evidence": "Inspected values match the admitted metadata contract."}
  }
}
```

```sh
tsr engine --directory RUN project acceptance review --file review.json
tsr engine --directory RUN project status
tsr engine --directory RUN project export
```

`decision` is `accept`, `reject` or `unresolved`. Supply evidence for every requirement;
evidence text is nonempty and at most 512 bytes. Both input files are limited to 16 KiB.
Acceptance requires every requirement marked met and a passing test receipt bound to the
current source. Rejection requires at least one unmet requirement. No review is admitted
while execution is pending. Source changes, new requirements or a new test receipt make
an older review stale. To reconsider, inspect current evidence and submit a new review.

An accepted review yields `goal_accepted`, `accepted: true`, `desire_status: fulfilled`
and `semantic_completeness: accepted_by_operator`. Without it the desire remains open.
Export remains available for inspection even when unaccepted; it includes `goal_accepted`
and saves metadata under the snapshot plus state-root hash, preserving older exports.
Acceptance does not authorize publication, deployment or a push.

Requirements and reviews are append-only operator records, distinct from model proposals
and test forecasts. Their text is excluded from proposal requests. The operator must keep
private reference material out of public goals, source excerpts and test feedback as well.
The CLI is a trusted operator boundary, not an OS authentication system; do not give
candidate code access to it or the controller store. `met` and evidence text are operator
attestations, not an automatic proof of semantic equivalence or authenticity of arbitrary
external evidence. An operator can make an incorrect judgment; the controller enforces
explicit authority, complete coverage of declared requirements and fresh evidence bindings.

## Local validation and live-case result

The Rust regression deliberately supplies passing test observations for an incomplete
invoice patch, rejects it independently, then completes the second behavior and accepts
it with a fresh review. It checks restart/export, stale snapshots, changed requirements,
pending execution, missing evidence and exclusion of private text from proposer requests.
This is a synthetic controller regression with a fake runner, not real-corpus validation.
CLI tests exercise bounded input, registration and refusal to accept without passing tests.
Python tests exercise accepted, unaccepted and source-changed terminal routing.

Commands run on Rust 1.90:

```sh
cargo test -p tsr-cli --lib --test engine_cli --offline
cargo clippy -p tsr-cli --all-targets --offline -- -D warnings
python3 -m unittest discover -s scripts -p 'test_engine_*.py'
```

Results: 56 library tests and 2 CLI integration tests passed. Clippy passed. Python:
41 discovered, 32 passed, 9 optional Docker tests skipped. The initial sandbox run could
not bind loopback fixture ports; the authorized rerun succeeded. No external provider
was used by these checks.

The existing live RTS repair is now independently **rejected**: two language entries
replace thirteen and version/extension metadata is incomplete. Local status, resume,
export and transport restart all preserve test pass while reporting the goal unaccepted
and desire open. The transport replay patched provider dispatch and network functions to
raise if invoked: neither was called. All 56 protected files, old exports, historical
evidence and the spending ledger remained unchanged; the original repository is clean.
See [structured local evidence](engine-rts-acceptance-result.json). The previous full-suite
result (999 passed, 15 ignored) remains valid as a test observation and was not rerun.

The two-call live demonstration remains terminal. This change makes its limitation
explicit; it does not repair that model patch or establish M3's 150-task corpus,
lineage-controlled baseline intervals, or full milestone acceptance.

## Continue after review

[Feedback and episode carryover](engine-continuous-cycle.md) connects these reviews to the next
proposal. Explicitly publish sanitized guidance with `acceptance feedback --file`; private
requirement and review text remain excluded. `acceptance carry --from` imports reviewed
observations into a later episode that preserves the source requirements. Neither command dispatches inference.
