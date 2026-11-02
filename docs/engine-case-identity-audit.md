# B9.1 case identity audit (offline, 2026-10-25)

Starting evidence: `0487f04`, retained typed Opal/Cedar candidates from
`target/b91-budget-shadow-two-model/`. No further live allowance exists.

The actual assembled experiment contract already said names must exactly match
Rust test names, must not collide with existing case IDs, and must not leave extra
unlisted tests. Both retained candidates violated the exact-name rule by prepending
the integration-test file stem to root-level functions. Opal additionally reused
existing names. There is no demonstrated contradictory instruction; this is provider
noncompliance with a clear general rule. The detailed Cargo naming convention was
implicit, so explaining it is a justified clarification, not proof of the cause or
proof the next model output will improve.

Native deferred validation checks structure and provenance, not arbitrary Rust test
inventory. Read-only code inspection of `crates/tsr-cli/src/engine/adaptive/coding.rs`
shows exact case-name matching against Cargo reports. Missing names produce
`unavailable` and `passed:None`, not passing evidence. The independent reviewer caught
the mismatch before any execution. No execution was performed to test predictions.

## Narrow changes

- `engine-b91-project-envelope.py`: explains that integration-test filenames name
  binary targets, not Rust module prefixes; only actual nested modules add `::`.
  No test assertion, parser outcome, winning proposal or renamed candidate is supplied.
- `engine-b91-driver.py`: experiment and readmission reviewer contexts now include
  current existing case IDs, unchanged requirements and explicit checks for name
  agreement, duplicate/colliding/extra names, requirement-to-assertion coverage, and
  abstention when macros/cfg/unavailable modules prevent static verification. These
  travel with existing state, snapshot, exact subject and subject hash bindings.
  Review cannot rename the proposal, invent execution or substitute mapping prose
  for assertion coverage. Independent approval remains a separate gate.
- `test_engine_b91_oracle_review.py`: authored positive root/nested identities and
  wrong file-prefix/module, missing test, duplicate, extra and macro-unknown cases
  verify the actual reviewer-context path, no mutation, and fail-closed independent
  rejection. Existing stale-subject review tests remain. Positive fixtures are
  intentionally identity-only, not semantic acceptance or generated model evidence.

This is **not an automatic Rust parser**. Structural acceptance is still not semantic
or execution-readiness approval. General Rust inventory may depend on cfg, macro
expansion and the build environment; static uncertainty must remain explicit rather
than guessed by a regex or hidden test renaming. No new mandatory response field was
added that would silently invalidate previous review records.

Independent code review found no blocking issue, confirmed the clarification is
accurate and protected files/validators remain unchanged, and explicitly retained
the dependence on independent identity review. No frozen aims, acceptance checks,
historical trials, provider receipts or model outputs were edited.

Next needed step: use this identity checklist in any future independently approved
experiment review. A live model retry would require a new admission and would test
compliance, not establish broad reliability. The current evidence does not justify
switching providers, relaxing validation or claiming the problem solved by wording.

Verification history: the initial focused run completed 20 tests (13 passed,
7 explicitly skipped), and the envelope-only diagnostic passed 17 tests. The first
full Python run completed 625 tests in 193.242s with 30 subtest errors and 17 skips:
all errors were in the review-boundary fixture whose controller stub omitted the
newly consumed `coding.cases` context. The production native request supplies it.
The nonprotected stub now supplies `fixture_existing`; all original bounds and
assertions remain unchanged. Independent review approved that fixture correction.
Nine repaired review/oracle checks pass in 10.887s. Initial failures are retained in
`target/case-identity-full-python.log`, not relabeled as a clean run.

Final full offline verification: **625 tests in 177.870s: 608 passed, 17 skipped,
zero failures** (`target/case-identity-final-full-python.log`). Existing cleanup
ResourceWarnings remain. No new full Rust sweep is claimed; actual native validator
paths are covered by the Python integration tests. No inference, generated-target
execution, adoption, protected edit or historical evidence mutation occurred.
