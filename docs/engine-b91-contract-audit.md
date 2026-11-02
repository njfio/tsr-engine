# B9.1 six-stage offline contract audit

Audit base: `261abe6`, 2026-10-24. No inference, target-project execution or new
trial admission. All positive outputs below are authored mechanics fixtures, not
model discoveries. Previous live trials, charges and expired grants are unchanged.

## Assembled messages and transitions

The audit follows `engine-b91-project-envelope.py::project/fixture_step`,
`engine-local-model.py::bind_proposal`, `engine-b91-driver.py`, and the Rust
`adaptive/provider.rs`, `coding/project/{relational.rs,acceptance/reviewer/supplemental.rs}`
validators. It covers complete system/user messages, not just helper strings.

| Stage | Required envelope | Evidence and transition |
|---|---|---|
| relation_after_seed | relational | Exact source-bound evaluator seed forecast and failed return; both desires retained; initial development absent/null, never fabricated ancestry. |
| experiment | experiment | Reviewed current relational guidance; generated Cargo tests must map the published second aim `existing_behaviour`; separate review/admission precedes execution. |
| relation_after_experiment | relational | Imported source-bound generated experiment return; revision references actual eligible history, preserving parent set. |
| boundary_patch | edits + rationale | Current reviewed guidance and experiment evidence; exactly one complete src/lib.rs replacement, separate native commitment/admission. |
| relation_after_verification | relational | Same experiment re-admitted to changed source and independently observed return; exact revision lineage. |
| final_proposal_only | exactly one of relational, experiment, edits + rationale | Current verified context; validation only, no seventh call or fifth execution, no automatic adoption. |

Every request supplies an exact outer-envelope schema and stage-specific inner
field descriptions. JSON is strict: no fences, duplicate keys, nonfinite numbers,
trailing material, unknown fields or provider-authored controller/authority wrappers.
The parser still checks actual response type, model identity, termination and usage;
the provider is not assumed to enforce the supplied descriptive schema.

Relations use published parent IDs; meaning strings and development text are
nonblank and at most 512 UTF-8 bytes. The trial's persisted identity requires explicit
`revises` and `catalysts` (initial null and empty array). Rust defaults permit their
omission, but the trial now rejects omission before persistence rather than failing
later. Only development's schema-defined absent/null equivalence is normalized for
comparison; records are never rewritten. Revision origin/prior hashes must be actual
eligible references, parent sets unchanged, and no-new-insight must retain meaning.

Experiments declare exact harness/cases/mapping shapes, allowed path namespace,
unique actual Cargo test names, empty args/stdout, nonempty requirement mappings,
coverage of every declared case and finite evidence limits. Package/import metadata
comes from the frozen synthetic Cargo manifest; protected test bodies remain withheld.
Patches expose one admitted path and optional finite [0,1] probability (not null).
Bindings, protocol, source/attention references and execution permissions are host-owned.

Observed return enums remain typed: passed/failed_tests/timeout/unavailable/
integrity_failure; case bool or unknown; supplemental passed_finite_case/
counterexample/unknown and not_met/unknown findings. Finite passing checks cannot
establish universal satisfaction. Stale source, foreign provenance, unknown ancestors,
withdrawal, missing review and uncertain dispatch continue to stop the driver.

## Demonstrated corrections

1. Published the previously missing Cargo package/library identity and no-dependency
   constraint, exact experiment field types, second-aim mapping, text/name limits.
2. Enforced the advertised 4,096-byte aggregate experiment source bound before
   binding/storage, including multibyte strings. The general native adapter intentionally
   has broader limits; this trial does not inherit that broader authority.
3. Aligned one-edit envelope cardinality with the sole admitted file and stated the
   2,048-byte replacement bound.
4. Exposed aggregate native serialized limits: 8,192 bytes for the whole bound response,
   4,096 for a relational proposal and 6,144 for a bound experiment. Individual maxima
   cannot all be combined; native checks remain authoritative.
5. Replaced ASCII-escaped temporary JSON with compact UTF-8 at validation, review-only
   storage and commitment. Previously valid Unicode could expand beyond the response
   byte limit between validation and commitment. No native limit was increased.
6. Made the trial's explicit revises/catalysts persistence requirement visible and
   rejected omission before it could be stored. No broad null normalization was added.

Independent review identified the aggregate serialization limits and the separate
commit writer after the first audit pass; both were addressed. No changed native
validator, acceptance criterion, original desire, test fixture or execution authority.

## Historical evidence and live uncertainty

Trial 1 established a development absent/null comparison mismatch. Trial 2 did not
retain enough structural evidence to identify its precise defect: nothing here
reconstructs that lost output. Trial 3 retained evidence of invalid initial development
ancestry. Trial 4 retained a bare relational payload lacking its outer envelope. These
support specific interface corrections and observed noncompliance, not claims about
unretained semantic quality or proof that prompts caused each failure.

The full assembled request has no remaining identified contradiction after this audit.
That is not a guarantee of provider compliance, generated test usefulness, compilation,
semantic quality, or successful B9.1 behavior. A fresh bounded live trial is materially
justified by these verified corrections, but needs fresh approval: this pass grants no
calls or executions. Keep the same six-call/four-execution envelope and stop-on-invalid
rules; do not silently retry, widen scope, or spend expired allowances.

## Verification

Commands/results are recorded below after completion. Offline native fixture uses the
explicit cfg(test) worker: every worker receipt asserts zero target commands executed.

- `PYTHONPATH=scripts python3 -m unittest test_engine_b91_contract_audit test_engine_b91_project_envelope test_engine_b91_lineage test_engine_b91_wire_contract test_engine_b91_diagnostics`: **37 passed**, 25.886s (`target/b91-audit-focused-final.log`).
- `PYTHONPATH=scripts TSR_B91_TEST_WORKER_BIN="$PWD/target/debug/deps/tsr_cli-62904bb7a7cdafee" python3 -m unittest test_engine_b91_driver.NativeSequence.test_native_lineage_contract_across_generated_fixture_stages`: **1 passed**, 58.762s (`target/b91-audit-native-final.log`). All six assembled messages stay within the existing request bound; real Rust rejects forged lineage, accepts valid authored proposals, and terminal replay makes no additional fixture calls/actions.
- `PYTHONPATH=scripts python3 -m unittest discover -s scripts -p 'test_*.py'`: **597 total, 580 passed, 17 opt-in skips**, zero failures/errors, 131.925s (`target/b91-contract-audit-python.log`). Supported permission used for local test subprocess/socket fixtures. ResourceWarnings about test cleanup occurred; they were not suppressed or represented as test failures.
- `git diff --check`: passed. No Rust implementation changed; full Rust workspace suite was not rerun in this pass.
- Trial 4 driver/provider/STOP/quarantine: all four hashes match saved replay evidence. No prior evidence was edited.
- Independent review of final implementation: no remaining blocking finding; live compliance and semantic quality remain unverified.
