# B9.1 optional-field comparison repair

2026-10-23, following stopped Trial1 (`a510345`). No provider requests, real target
executions or new live admission in this repair. Trial1 receipts, STOP marker and unused
expired slots are preserved; no historical evidence is reserialized or rewritten.

## Exact correction

Rust `engine/deliberation/relational.rs::Proposal.development` explicitly uses
`default, skip_serializing_if = Option::is_none`. Python journal proposals may contain
`development: null`, while native history omits it. Both mean absent Development under
that exact schema. The envelope now compares canonical copies with only that top-level
null field removed. Non-null Development objects remain byte-canonically compared;
nested nulls, types, list order, unknown fields and every other proposal value are not
normalized. Top-level field inventory and development object/null type fail closed.

The correction is applied at **both** synthetic guidance provenance comparison and
independent-admission stage comparison. Missing prior proposal, withdrawn/superseded
history, noncurrent guidance, stale source, changed parent IDs and foreign guidance
remain rejected. No permissions, criteria, source allowlists or Rust contracts changed.

## Verification and review

The focused envelope suite initially passed 17 tests. The first full native driver
run then exposed the same raw-equality defect in the second stage-admission comparison;
its failed log is retained. After fixing that second site, the full deterministic driver
passed in **53.927 seconds**. It uses real Rust state, admission and serialization, with
explicitly authored fake provider responses and cfg(test)-only worker returns. No Cargo
project or container command executes in this fixture path.

The fixture now emits explicit `development: null` on initial proposals. After all six
stages/four returns it asserts that the journal retains null, native history omits it,
and they compare equal. It also runs the completed driver again, verifying unchanged
history and no additional fixture calls/actions. Revision objects remain non-null and
are exercised later in the same sequence. This replaces the earlier homogeneous-fixture
blind spot; it does not establish model capability or live B9.1 success.

Negative regressions cover wrong development types, non-null changes, nested omission,
unknown fields, changed meaning/parent/revision IDs, stale source, withdrawal,
supersession and noncurrent guidance. Independent review of both comparison sites found
no blocking issue or loosened authority/provenance gate. `git diff --check` passed.

Full Python discovery passed **575 tests: 559 passed, 0 failures/errors, 16 skipped**,
in 123.015 seconds (wrapper exit 0, no timeout). The 16 existing opt-in skips remain;
the full native sequence above was run separately. Trial1 driver/provider journals,
STOP and terminal summary hashes matched before/after. Both verification processes
finished. Local logs:
`target/b91-null-fix/{envelope.log,native.log,native-corrected.log,python.log,result.json}`.

## Next bounded live plan — not authorized by this repair

The existing driver binds the admission hash and implementation hashes into the journal.
It has no supported migration for reopening a stopped expired trial under new code.
Therefore the runnable next plan is a **fresh Trial2 directory/admission**, retaining
Trial1 as independent evidence, using the same frozen synthetic scenario and criteria:
maximum six CheaperInference/deepseek-v4.1-flash calls (8,192 completion tokens and 240
seconds each; total 49,152 completion / 98,304 input), four networkless disposable
execution groups, 240,000 ms aggregate tool reservation and one-hour expiry. Groups
remain seed failure, generated second-aim experiment, one boundary patch and unchanged
experiment re-verification. Only disposable src/lib.rs patch authority; immutable tests
and separate native reviews unchanged. Final continuation remains proposal-only.

A five-call/three-group continuation reusing Trial1 is not a currently supported path;
removing STOP or rewriting its admission would violate history bindings. Do not imply
that this repair grants either option. Fresh approval, reviewed new hashes and ordinary
supported network/runtime permission are required. Stop on invalidity/uncertainty,
retain costs and never replay unknown requests. No live command was launched here.
