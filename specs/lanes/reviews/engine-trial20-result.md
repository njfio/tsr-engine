# Independent Trial20 result review

Reviewer: `/root/b91_full_review`. Read-only retained evidence review; no provider calls, target executions, review replacement or acceptance/adoption.

Trial20 stopped after the generated patch executed and before the unchanged supplemental experiment was readmitted. Parent reports ten total paid calls, three actual execution groups and $0.009814, with no unknown outcomes. The earlier generator experiment schema correction produced a fresh valid response; this does not make the later reviewer failure a successful review.

## Exact reviewer failure

`runtime-project-reviews/46ac4a0248c21563ef9341d5d47d7b69497d089db0d9a826c8d6d07c47536f36.outcome.json` retains `schema_enum` at schema validation, with one unsupported evidence reference under `epistemic_grounding`. Retained verdict/finding/basis enums are legal; other findings have zero unsupported references. The raw report is not retained (`value:null`), so the offending identifier and explanatory wording are unknown. It is not justified to call this a particular typo, stale identifier, fabricated receipt or semantic rejection.

The actual request contains seven explicit evidence keys and the identical seven allowed evidence-item enum values, including `test_receipt:1738fbe1dce91901006ee0f96d57f66ab90e03a9a3757b53c9999f5dfdc3f4ac`. That receipt is also the exact bound observed receipt. The schema is satisfiable and consistent with the supplied evidence; no demonstrated input-contract contradiction warrants weakening validation. Provider request `d46173b3-5bba-4c19-ae02-ad2f9707783e` completed HTTP200/stop with reported usage and $0.001390; body hash `c99844ad9b442b47465ca759f3077236465d89e54a4aeb13c0fc2f5cc8d3ca68` and candidate hash `68a80cd72746ace7dc838ceb02a7ba1eabf5221da6c051619f6511e8b523229a` preserve failure provenance.

## Patch and observed tests

Current source adds an exactly-empty branch returning `Ok(Vec::new())`; for every nonempty string it executes the original split/map/i64-parse/error/collect expression, without filtering or trimming. This static source-equivalence argument supports preservation beyond the finite tests. Cargo.toml, Cargo.lock and protected tests/contract.rs remain byte-identical to the frozen source.

Actual patch receipt directory `project/coding-receipts/208a280b6922a53d8bea1807518d39358b02dc203632bf0944aa7322f2a7ec53` records edit, compilation and tests with exits 0/0/0; the protected suite passes two of two tests. The earlier generated supplemental experiment checked exact empty success, signed values and rejection of `1,,2`. Its pre-patch complete log shows empty success and failure at the `1,,2` assertion after the signed assertion completed. It has **not** run against the new patch: `checks-after` is absent. No post-verification reflection or final independent native acceptance is established.

## Small justified recovery boundary

A bounded separately admitted correction may request a full fresh review for this known settled schema-invalid report, bound to unchanged current source, criteria, generated harness and exact evidence catalog. It must preserve the original failure and cost, use fresh request identity and shared-budget accounting, and apply every structural and semantic validator anew. No host substitution/removal of the unsupported citation, no inferred approval from the retained `accept` enum, no retry of uncertainty or semantic rejection. A valid fresh review can only permit the already admitted unchanged experiment; its actual return and subsequent review still remain required. The current repair appears semantically sound, but this run is not completed or accepted.
