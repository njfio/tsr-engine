# B9.1 return evidence boundary

Offline follow-up to Trial 7 (`9daed39`). Historical trials and acceptance checks
are unchanged. No new provider call or target-project execution was performed.

## Finding

Independent inspection of Trial 7's frozen harness and bounded log showed that
the generated supplemental test failed at its first lone-comma assertion.
Subsequent assertions in that function were not reached; the ordinary signed-number
assertions before it passed. The protected original test separately failed the
`1,,2` case. The rejected revision nevertheless attributed an exact returned vector
for `1,,2` to the supplemental run. That vector was inferable from source, but was
not an observed value in that receipt. The independent reviewer correctly rejected
this revision; it never became admitted guidance.

The model-facing native receipts expose case summaries and command status, not
trusted assertion-level telemetry. Candidate-controlled panic output cannot safely
be promoted into such telemetry. Supplemental imported outcomes also omit numeric
exit status; failure does not justify inventing exit 101 there.

## Implemented boundary

`scripts/engine-b91-evidence.py` derives a state/source-bound catalog from existing
native return records. Compact model-facing case references resolve to full receipt,
commitment/bundle, case, outcome and exit provenance. The catalog binding changes
with state, sources and evidence. Cargo case classification uses the native completed
verdict/exit rules. Abbreviated stored diagnostic excerpts do not erase independently
validated case results. Timeouts or inconsistent tuples remain unknown.

Normal B9.1 relational revisions require a separate `return_basis`: observed case
references, current-source-bound predictions, and explicit unknowns. Arbitrary
observed assertion/value fields, invented citations, stale bindings and false source
identities fail validation. Assertions, early abort and not-run status stay unknown
at this typed boundary. In particular the published parser source cannot be cited
as evidence of execution order inside a separate generated test harness.

The outer metadata is persisted in the provider journal, restored on restart, and
included in the independent review subject. It is stripped before the existing
native proposal schema. No execution permission, acceptance or desire satisfaction
is conferred by this metadata. Native schemas and frozen scenarios are unchanged.

This is a gate in the B9.1 driver, not a new global rule for all native callers.
A valid citation does **not** prove that adjacent natural-language prose agrees with
it. Independent semantic review remains mandatory and must reject unsupported
observed-value claims. Useful source inference remains allowed when explicitly
identified; this is not a keyword filter or a claim of automated prose understanding.

## Verification

- Initial focused run: 34 passed. Initial native fixture exposed duplicated catalog
  provenance exceeding the existing 16-KiB request limit. The representation was
  compacted; the limit was not increased.
- Final focused plus six-stage Rust/Python fixture: 6 passed in 60.621s, including
  full real message assembly, native acceptance/rejection, restart/no redispatch,
  and all six requests within the unchanged size bound. Worker outcomes are authored
  fixtures with zero target commands, not live B9.1 evidence.
- Independent code review found no blocking issue; it explicitly retained the
  semantic-review limitation above.
- Full offline Python discovery: **606 total, 589 passed, 17 opt-in skips**,
  zero failures/errors in 136.434s (`target/b91-evidence-full-python.log`).
  Existing resource-cleanup warnings remain visible. No Rust production code changed.

The next meaningful live validation would test whether a generated revision now
keeps source predictions separate from observed case outcomes. No new live trial
is approved or dispatched by this change; Trial 7's failed result remains intact.
