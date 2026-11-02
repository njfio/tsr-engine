# Prospective size-failure recovery and mechanical-limit audit

The closed result `8d4aa68d` remains a failed first proposal, not a corrected run.
Its retained JSON contains a 2,600-byte replacement, a 274-byte rationale and a
3,024-byte response. No patch was adopted or executed. No minimum required source
size can be inferred from that one proposed implementation.

## Origin and scope of the bound

The 2,048-byte limit began with generic scoped exact edits in commit `03f4ee13`
(`docs/engine-generated-coding.md`, Authority, evidence and limits), then applied
to multi-file edit sets in `baef6c4a`. The complete B9.1 wire contract published the
same limit in `0dd264ed`. It predates this shortlist task. Native `Response`
validation and `validate_patch` both enforce old/new span limits; this adapter uses
whole-file replacement, so its source replacement inherits the span limit.

The bound is a finite implementation/scope constraint, not a protected semantic
acceptance criterion or a measurement derived from this task's complexity. It
still belongs to the admitted interface and must not be ignored. Raising only the
Python limit would fail native validation; a larger current file would also affect
the next patch's old span. Native aggregate patch accounting charges resulting file
bytes for each actualized patch, so 4,096 supports two 2,048-byte replacements.

Recommended narrow repair: keep every bound and allow a new admission to opt into
a full fresh generation for a known, settled oversized **single permitted edit**.
This consumes the same two shared corrections as JSON/shape correction. The
retained response is not shortened, repaired or accepted. Exact structural/byte
feedback supplies no winning algorithm, tests or relationship. The new response
must independently pass all native checks and runtime review.

An alternative 4 KiB whole-file interface would require a coherent native span
contract, producer contract and aggregate-budget change, plus payload-growth review.
It is not implemented or smuggled into this repair. The captured response alone is
not evidence that this larger authority is necessary. If repeated bounded fresh
generation cannot produce a faithful candidate, report inability; do not enlarge
limits or resample indefinitely.

## Mechanical audit

| Boundary | Current constraint | Consequence / treatment |
| --- | --- | --- |
| Replacement / old span | 2,048 UTF-8 bytes each; exactly one editable `src/lib.rs` | Whole-file adapter inherits native bound. New opt-in recovery never changes it. |
| Aggregate patch resource | 4,096 resulting-file bytes across admitted actualizations | Two maximum replacements fit; exhaustion is terminal, not output correction. |
| Native response | 8,192 serialized bytes, including host-added fields and old source | Per-field maxima cannot all be combined. Producer must see this overall constraint. |
| Alternatives | 1–4; no executable/deferred mixture; a read or qualitative revision stands alone | Extra/forbidden alternatives must not be masked by an earlier size error. |
| Relational / information proposal | 4,096 serialized bytes per typed proposal | Lineage and field names consume space; individual text maxima cannot all combine. |
| Human-readable text | Nonblank and at most 512 UTF-8 bytes in bounded meaning/rationale fields | Existing shape correction remains separate from semantic acceptance. |
| Provider request | 32,768 serialized bytes, including appended correction feedback | Preflight refuses oversized context; no evidence truncation or automatic retry. |
| Native local context | Explicit 131,072-byte profile; older defaults unchanged | Local introspection allowance is not export authority. |
| Output / token accounting | 8,192 completion tokens; prompt and total reservations remain checked | Overruns or unknown usage stop, never trigger a guessed replay. |
| Information access | One group, 2,048 bytes per record, 4,096 total; fixed allowlist/digests | Access proves bytes, not truth. Path, source, corpus or permission defects are terminal. |
| Operations / time | One read and two sealed groups; 240 seconds aggregate tool reservation | No extra operation for recovery; deadline/resource exhaustion stops. |
| Model envelope | Seven possible reviewed decisions plus two shared corrections = 16 calls | Reviewer and correction calls count. Semantic rejection and abstention do not retry. |
| Wall time | 90 minutes; each request at most 240 seconds | Full remaining envelope checked before launch; no silent extension. |
| Independent acceptance | Exact source/requirements/receipt binding, separate review copy | Packing compliance and passing syntax do not establish task fulfillment. |

The previous actual authored full-flow maximum was 28,894 bytes. That is not a
proof that every permitted unique history fits 32 KiB. Schema deduplication retains
all evidence and relationships; long unique histories can still terminate safely.
This limitation remains explicit rather than concealed by a general success claim.

## Prospective authority

No call or operation is authorized by this document. A next complete attempt would
use the same frozen public shortlist packet and rubric, CheaperInference
`gpt-6-luna`, at most 16 total generator/reviewer requests including two shared
format-or-opted-in-size regenerations, one bounded read and two isolated program
groups, 90 minutes, and at most $0.80 reservation within the unchanged $5 total.
From 223 calls / 61 groups / $0.181596, the complete envelope requires total caps
**239 calls / 64 groups**: one additional campaign call, no operation-cap increase.
The prior unused allowance is expired; fresh approval/admission is required.
Final supported acceptance remains a separate evidence-bound review-copy decision.

This prospective test still may produce immediate success, abstention or unchanged
meaning. It does not force a failure or relationship revision. Code length and
protocol compliance remain practical constraints on the experiment, not evidence
for or against TSR's conceptual claim. Ordinary memory plus the same model remains
a possible explanation for any adaptation; no causal advantage is claimed.

## Implemented and verified

`representation_regeneration=true` is an explicit new admission field, absent by
default. Only a single, otherwise well-shaped edit of the admitted source path in
normal proposal phase qualifies. Nonblank bounded rationale, finite forecast,
current source identities and the overall native response bound are checked first.
The trusted diagnostic gives `$.alternatives[0].edits[0].replacement`, actual UTF-8
bytes and the unchanged maximum. A known settled charge is mandatory. The new
request carries the original task/context plus precise feedback; every request is
counted and the previous response remains quarantined. The fresh response then
passes the same native validation and independent review as any other proposal.

The response's path, identifiers, source or code are never edited to make it pass.
Mixed alternatives cannot hide a later forbidden option behind an earlier size
failure. A stale post-response context is retained as a terminal rejection, not
used for regeneration. Semantic judgments, unknown effects, expired authority and
resource exhaustion remain outside correction eligibility. Format and size errors
share two slots, with restart/exhaustion preventing reuse.

The data-only preparer now supports prospective authority profile
`native-information-authority@v2`, with the explicit regeneration flag and exact
239/64 cap proposal. It predicts the existing deterministic append-only upgrade
row to bind the prospective grant to the correct post-upgrade ledger head. It does
not append that row, apply a grant or create paid authority itself.

**32 unique local checks passed:** eight new regeneration/native-preview/review/
restart checks, six information-preparation compatibility checks and eighteen
shared correction/release/public-contract regressions. The actual retained failure
was checked locally by hash/shape/size; its raw contents were not committed or
rewritten. Authored valid follow-up responses demonstrate wiring only. Tests used
no provider requests, corpus inspection or target execution. Native Rust source
and binary are unchanged, so the prior full-flow tests were not redundantly rerun.
Two initial fixture errors were corrected (catching the actual stale exception and
normalizing a temporary path alias); they did not justify weakening production
validation. Three initially skipped legacy checks were rerun with the explicit
cfg(test) binary and passed in 18.863 seconds.

Independent review by `/root/reader_integration_review` found no unresolved material
issue, including source/authority precedence, shared-budget exhaustion, prospective
grant identity and the conceptual limit that packing recovery is not TSR evidence.

Reproduce the targeted local checks with the existing cfg(test) binary:

```sh
TSR_B91_TEST_WORKER_BIN="$PWD/target/debug/deps/tsr_cli-95f29d427efcd336" \
  PYTHONPATH=scripts python3 -m unittest \
  test_engine_representation_release test_engine_information_release.Bounds \
  test_engine_bounded_correction test_engine_output_release \
  test_engine_native_release.Release test_engine_native_release.Contracts
```

The captured-response check explicitly skips if historical local quarantine is
absent; it does not reconstruct lost content. On this machine it ran and passed.
Select the corresponding test binary if a later native build identity changes.

Frozen prospective release: `experiments/information-pursuit-release-v2/manifest.json`.
File SHA-256: `b163cabbc2289da74c1e57a27e49576a515ddd1b0f00496adee3711683e54c52`.
Canonical JSON SHA-256 for future authority:
`fdf0a2eab009df3b3ed4b7ec1ce57797601951766a663b376519aaf47b4c2319`.
The public task, condition records, oracle and rubric remain the original frozen
`sequential-pursuit-v1` packet. Future preparation uses the existing
`engine-information-prepare.py` command with this release and a genuine new
`native-information-authority@v2` record. No such approval record or new live state
was created during this work. The failed live result and campaign ledger hashes
were reverified unchanged.
