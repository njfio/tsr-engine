# Final offline TSR integration pass

2026-10-31. Authored fixtures only; no provider calls or live autonomy claim.

This document records the `d98bd718` checkpoint. The subsequent prospective
ordinary-return connection and its separate verification are documented in
`ordinary-criterion-return.md`; the missing connection described below is the
checkpoint's finding, not a claim that later source changes were already tested.

## Capability and remaining boundary

The native ledger integration at `a866c24f` is real: canonical Generator
Preference expansion, exact Contrast/return lineage, typed independent Council
admission, generic active-ledger eligibility, and subsequent project composition.
It preserves original contributors, obligations and permissions. Generic selection
still obeys the existing Tune. Eligibility is not fulfillment or a new user command.
See `docs/contrast-preference-ledger-integration.md` for the source contract.

Two halves are verified, but an uninterrupted ordinary loop connecting them is
not yet demonstrated:

1. Frozen qualitative anticipation -> authentic action return -> independently
   reviewed interpretation -> later request/choice (`project/pursuit.rs`,
   `scripts/engine-pursuit-reflection.py`).
2. Already authenticated structured Contrast plus reviewed aspiration -> native
   Preference -> Council eligibility -> later composition/choice and a fresh
   adapter admission (`project/expansion.rs`, `tsr-ledger/src/reviewed.rs`,
   `scripts/test_engine_preference_session.py`).

The expansion fixture explicitly seeds criterion-level rejection and aspiration.
`project_current_contrast` requires a current acceptance review and structured
unmet named criteria. The ordinary pursuit reflection carries prose `contrast`;
`engine-browser-assessment.py` carries source-bound control observations, explicitly
not native acceptance. Neither automatically supplies that required structured
Contrast. Turning uncertainty or prose into unmet criteria would be unsound.
The remaining engineering connection is an independently reviewed, exact-source,
named-criterion return-to-Contrast channel through ordinary orchestration, with
unknown kept distinct from unmet and no promotion to user acceptance. This is not
closed by the current integration tests and must not be advertised as a complete
autonomous new-desire cycle.

The source basis is checked-in PRD B2.1/B6/B8 and recorded author clarifications,
not direct book review. Successful combination without Contrast remains a separate
optional author clarification; it is not required to implement the specified
Contrast path. New identity existence alone is not causal behavioral learning.

## Prospective provenance correction

Independent full-chain review identified Operator tagging on model-reviewed
qualitative reflection events. New writes use a versioned Council record;
legacy Operator records stay readable with their old projection. Exact generator,
reviewer, source and receipt checks remain. Regressions reject wrong-source variants,
verify rollback, preserve legacy row shape and restart, and assert that prose
uncertainty does not become structured Contrast. This does not migrate or relabel
old history.

## Verification

Final command results and source pins are recorded in
`target/tsr-final-offline-integration/`. The final-source native library suite passed 336 tests (8 existing ignored),
and its sealed-browser multi-file/provenance/restart case passed again using the
installed pinned image (12.69 seconds). Scoped Rust formatting, Python syntax and
`git diff --check` passed. The original-build full workspace run completed: 1,110 passed, 1 failed,
13 ignored (exit 101). The sole failure is the unchanged learning-report test
`b1_report_refuses_a_foreign_dataset`: it hard-codes `/tmp/none` as a logging
directory, while that path is an existing zero-byte regular file. Independent
read-only review confirmed `report.rs:1066` → `report_logged:324` →
`probes.rs:644 create_dir_all`. The file is unchanged at `6d1ce769`; no
integration dependency explains this filesystem collision. The unrelated path
was not removed or modified. This remains a failed compatibility check, not a
claimed clean workspace. The independent review also checked fresh-admission
continuity: it imports the same authenticated native store under exact public
export pins; it is not cross-project memory transfer or new-user-feedback proof.

## Preserved authority and next live binding

The denied `experiments/relational-experience-v1/manifest.json` remains frozen at
SHA-256 `4d74840dde08956641575b8e87ed424650fed73f312ac8dc5fb062513efb950b`.
The campaign ledger remains at SHA-256
`5ee77092fd21eae203f94146349ccd3594feeaba2d3a7fb1b29df8d24e1444b0`:
395 calls, 129 groups, $0.644668 known plus the historical $0.05 uncertain
reservation estimate, not a verified upper bound. No paid dispatch or pending
packet rebind occurred. Old project-only nodes, games and failed trials remain.

Any live confirmation must cover the resulting evaluation scope; approval of the
frozen denied action cannot silently transfer to changed code or protocol. After
that confirmation, any revised test needs a separately
identified prospective packet/admission binding the final native binary and source
hashes, changed adapter dependency hashes if applicable, exact public projection,
current ledger prefix, retained initial state, installed runtime and actual price
metadata. The pending frozen action must not be silently changed. A live test of
the complete new-Preference cycle additionally needs the missing structured
assessment connection above; current capabilities can only be tested with their
stated limits. No further game polishing or invented goals are justified by this
integration pass.

The initial exact-name native filter matched zero tests and is not counted as
verification; the complete final-source library suite replaced it. A sealed Python
invocation used the wrong class name after two real browser cases passed; the
corrected third case passed separately. A redundant inherited Python test loader
was interrupted and replaced with explicit owned methods; its partial output is
retained, not counted. No product validation was weakened for these test invocations.

Workspace lint is not clean. Final all-targets Clippy reproduces the identical
baseline diagnostics: four library errors in `engine/qualitative/{inquiry,retrieval}.rs`
and `engine/qualitative.rs`, plus a test-only complex return type in
`project/cycle.rs:1100` (unchanged since `72c76342a`). Whole-workspace formatting
also remains nonclean: 18 existing affected paths versus 19 before this patch,
with no new path. Touched-file formatting passes. No unrelated lint refactor or
warning suppression was added.

Read-only binding audit: the current adapter adds `engine-browser-expansion.py`
and `engine-browser-desire.py`; seven existing implementation hashes differ:
`engine-project-runtime-review.py`, `engine-project-cycle.py`,
`engine-proposal-references.py`, `engine-browser-session.py`,
`engine-browser-ordinary.py`, `engine-browser-adapter.py`, and
`engine-browser-project.py`. Exact old/current SHA-256 values are retained in
`prospective-binding-diff.json`. The frozen packet explicitly says “no new identity
assertion”; testing new Preference formation is therefore a protocol/scope change,
not just a binary rebind. No such scope change or admission was made here.

Final-binary adapter reruns also passed: fresh native Preference session continuity
(1 test, 109.809 seconds) and all three isolated browser/qualitative/desire/staged
cases (3 tests, 431.496 seconds). These repeat coverage rather than increasing the
distinct authored total of 50 passes and one optional-worker skip. Independent
source and test review accepted this scoped provenance correction. The subsequent
ordinary-return criterion bridge remains separate ongoing work.
