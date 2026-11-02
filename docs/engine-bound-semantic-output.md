# Transport-owned identity, model-owned judgment

Prospective local repair of the closed objective-service live2 result (`e4350de4`).
No historical response, admission, result, acceptance or ledger is rewritten.

## Failure and correction policy

The round reviewer returned a 65-character binding where the request required a
64-character identity (an extra `f`). The receipt identity was correct. Native
`const` validation rejected the mismatch before admission. This was not an
eligible format correction: identity failure has no allowlisted shape diagnostic.
One earlier shape correction had been consumed; remaining allowance did not
authorize identity repair. That fail-closed classification remains unchanged.
The historical restart safely refused redispatch; it was not a clean terminal
replay. The incomplete objective and lack of adaptive pursuit remain unchanged.

## New boundary

`bound-semantic-output@v1` / `strict-json-bound@v2` separates immutable local
invocation metadata from semantic model content. The objective adapter now uses
`synthetic-objective-adapter@v2`; shared project release admissions require the new
output profile and source pins. Old admissions cannot run against the new code.

The synchronous transport associates the received content with the exact local
request. Before native admission the adapter checks the admission, request body,
stage/index, current root/source/scope, raw parsed-content digest, settled dispatch
and metering receipt. Journal ownership is checked again at action advancement
and round import, and before returning any review, including final assessment.
This is local executor/TLS provenance, not a provider-signed statement that the
model understood the context. Rust checks source, bindings and admission; it does
not authenticate HTTP. The trusted computing base includes the adapter, transport
and local journal. This does not defend against a compromised host rewriting that
trusted state or a provider internally misassociating a response. Same-model review
errors remain possible.

Only explicit immutable fields are host-owned:

- Review top-level `review_id`, `response_digest`, `binding`, and `receipt`, where
  required by that request's native contract.
- Objective service `choice.binding`, relationship wrapper `binding`, and the
  single admitted edit's `source_digest` / `inspection_handle`.
- Existing native proposal protocol/root/attention construction remains local.

The model must omit these fields in the new wire contract. Supplying them, even
correctly, is rejected; no existing wrong identity is stripped or overwritten.
The response is reconstructed only after known-settled association, then checked
against the unchanged native contract before its envelope is recorded or admitted. The model's raw content and the host metadata are separately
retained in the local journal. No provider response body is committed.

Selected evidence IDs, `return_basis.binding`, parent identities, revision lineage,
selected action/path, old/new source text, judgments, reasons and uncertainty stay
model-authored and strictly checked. JSON-object mode remains configured; it does
not enforce semantic truth, full schema or duplicate-key semantics. Rejection,
unknown and abstention remain legitimate; identity errors do not become retries.
The existing shared bounded correction budget is unchanged.

## Supported path inventory

The upgraded objective path covers proposal generation, proposal review,
relationship reconsideration/review, every service-round review and final objective
assessment. Shared project release review uses the same semantic-only envelope and
owned-import check. Its existing generated edits/relations already use the local
native binder; explicitly selected information/evidence identities remain strict.

Legacy non-release/library fixtures and frozen historical launchers retain their
old contracts and existing launch gates. The separate qualitative inquiry workflow
has not been migrated by this patch; its strict identity checks are unchanged. No
claim is made that every past harness uses this new envelope. No launcher gains
export, execution, acceptance or spending authority from a version change.

## Verification and limits

Independent source review found and resolved Unicode canonicalization divergence,
an over-broad inference of host ownership from arbitrary constants, final handoff
ownership coverage, and an ambiguous instruction about nested evidence bindings.
The implementation now hashes exactly like the project adapter, uses an explicit
identity allowlist and names the exact omitted paths.

Focused controls cover the captured extra-character shape (without committing the raw
provider response body), Unicode, stale/cross-root/cross-request substitution, mixed metadata,
unknown/unsettled effects, replay, missing generator ownership, substituted
candidate/decision, model-injected metadata, strict citations, and rejection/refusal.
The full objective test uses the real native CLI and explicitly authored transport
and cfg(test) worker receipts. It executes no generated program and proves wiring,
not model correctness or TSR development. Test results are recorded below.

A future live validation would require a fresh reviewed packet/admission and
explicit authorization; no unused slot from the closed run survives. The campaign
remains 234/252 calls, 65/68 operations, $0.195539/$5. A full five-operation batch
cannot fit the three remaining operation slots without a separately approved limit
change. This patch requests no such change and performs no live effects.

Completed verification:

- 13 dedicated identity/association/import controls passed.
- 11 native release tests passed in 211.809 seconds, including its authored full
  packet flow, corrections, exhaustion, unknown outcomes and restart.
- 6 correction tests passed; 15 cycle tests passed (one separate native test skipped
  without its explicit worker environment).
- 6 information controls passed (full information flow skipped); 6 release-gate
  controls passed.
- All 15 live2 evidence hashes and campaign ledger hash
  `7b7d532c0060edb35d8726ee4d2d1f942764e2135437ed4e2dfa2089d484ce6a`
  remain unchanged.

Two exploratory objective checks stopped on admission/source-pin drift while
independent-review corrections were landing (359.485 and 16.326 seconds). These
were local authored fixtures, not live attempts. They are not counted as full-flow
passes. The conclusive check runs with implementation frozen after review.

Commands (repository root; existing offline native binaries):

```sh
python3 -m unittest discover -s scripts -p test_engine_bound_output.py -v
python3 -m unittest discover -s scripts -p test_engine_bounded_correction.py -v
python3 -m unittest discover -s scripts -p test_engine_project_cycle.py -v
python3 -m unittest discover -s scripts -p test_engine_information_release.py -v
python3 -m unittest discover -s scripts -p test_engine_output_release.py -v
TSR_B91_TEST_WORKER_BIN="$PWD/target/debug/deps/tsr_cli-95f29d427efcd336" python3 -m unittest discover -s scripts -p test_engine_native_release.py -v
TSR_OBJECTIVE_TEST_WORKER_BIN="$PWD/target/debug/deps/tsr_cli-95f29d427efcd336" python3 -m unittest discover -s scripts -p test_engine_objective_adapter.py -v
```

Independent source review: `/root/binding_review` reviewed the actual final diff,
request serialization, import ownership and conceptual trust boundary. No unresolved
source finding remained before the final full-flow run. No Rust source or protected
acceptance tests changed; no new native build or dependency download was required.

Conclusive frozen objective flow: **passed in 907.839 seconds**, all 20 authored
requests (two bounded format corrections), patch smoke plus three authored service
worker outcomes, separate reviews, reconsideration, final assessment and inert
reconstructed-cycle replay. Maximum serialized request: **27,399 bytes**. No target
commands executed. Across the listed suites: **58 passed, 2 explicitly skipped**.
The skipped controls are separate native/information flows without their optional
worker environment, not failures concealed as successes.

Captured logs and machine-readable pins/results:
[`experiments/bound-semantic-output-local-v1`](../experiments/bound-semantic-output-local-v1).
No live validation is authorized or claimed. The historical run's malformed review
and safe redispatch refusal remain unchanged; this prospective fix does not reopen it.
