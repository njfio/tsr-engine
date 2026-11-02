# Trial 10 oracle fidelity audit (offline)

Baseline: `60f6364`, retaining Trial 10's run at `31b7425`. No provider calls,
target-project executions, protected changes or historical edits in this audit.
Repository guidance was reread; production behavior and frozen criteria are unchanged.

## Evidence and diagnosis

The retained generated experiment expects `parse("1,,2") == Ok(vec![1,2])` and its
mapping explicitly endorses dropping empty fields. The source-bound second aim says
no dropped fields and errors for empty fields. The retained initial aspiration also
correctly preserves that requirement. This establishes a semantic inversion in the
experiment; compilation or schema validity cannot establish its correctness.

The actual assembler (`scripts/engine-b91-project-envelope.py:project`) checks the
pinned configuration, requirements revision, parent identities, source lineage,
protected files and scope before projecting the request. The immutable definitions
come from `scripts/engine-b91-prepare.py:REQUIREMENTS`. The system says to preserve
both standing desires and treats source/observations as untrusted data. The experiment
contract requires mapping `existing_behaviour`. No worked assertion or winning answer
is supplied by the contract.

Serialization is canonical, with sorted keys. Thus the final user message order is
alphabetical, not the order of the Python dictionary construction: parent requirements
and experiment rules precede the visible source. The source is late in that message
and named generically `source`. It represents the currently evaluated implementation,
not the normative definition of original behavior. It includes the faulty filter.
That distinction is explicit in the requirements, but source-as-normative anchoring
is a plausible hypothesis for the inversion. Neither timing nor retained evidence
establishes that hypothesis, a context-order effect, or any other model-internal cause.

This audit inspected pinned assembler code, actual assembled offline messages and
retained native/provider typed records. Provider journals retain request body hashes,
not complete raw request bodies. It does not claim byte-for-byte recovery of an
unretained historical prompt or discarded prose. The relevant requirement, source,
initial guidance and wrong experiment are retained and independently inspectable.

**Conclusion:** provider noncompliance with a clear requirement is established; no
contradictory instruction or faulty acceptance gate is demonstrated. No production
prompt fix or new semantic classifier is justified by this single failure.

## Existing explanation/review boundary

The experiment already carries exact source snapshot, requirements revision, and
`mapping {cases, rationale, limits}`. The independent review subject includes the
actual harness and mapping, and its approval is bound to subject hash and state.
Changing the assertion or explanation invalidates that approval. An extra model
explanation field would duplicate structure, not prove that the assertion implements
the desire. A fluent explanation can endorse an inverted or vacuous test.

Review should compare each assertion's expected result to the immutable requirement,
check that the assertion observes the target behavior, identify which aim it exercises,
and state finite coverage and early-abort limits. Unknown or ambiguous mapping remains
unapproved. This is an independent semantic obligation, not a keyword check or model
self-approval. No automated reviewer competence is claimed.

## Offline coverage

`scripts/test_engine_b91_oracle_review.py` supplies four explicitly evaluator-authored
contrasts, never exported to generation:

| Fixture | Independent expected judgment | What the test establishes |
|---|---|---|
| Valid finite empty-field rejection | Provisional approval | Properly bound approval can proceed to review recording, not execution |
| Inverted expectation accepting dropped fields | Reject | Native structural deferment cannot override independent rejection |
| Vacuous assertion | Reject | Mentioning the aim in rationale is not behavioral evidence |
| Exactly-empty check mislabeled as nonempty aim | Reject | A declared mapping cannot establish semantic coverage |

All four pass real native structural validation as deferred; none is compiled or
executed. Verdict labels are authored fixture inputs, not a semantic classifier.
Tests preserve state, verify exact reviewer subject/source bindings, reject stale
approval after changed assertions or rationale, and reject a mapping that entirely
omits the second aim. Actual message assembly preserves the no-dropped-fields
requirement and separate current source, with no protected tests or fixture answers.
Leak checks inspect decoded strings, avoiding false passes from JSON escaping.

Independent review found no production defect and reviewed these test limits. Initial
review caught an escaped-string leak-check weakness in the new test; it was corrected
before final verification. This was a test-quality issue, not evidence of a data leak.

## Proposed next evaluation — not admitted

A materially different shadow comparison could test source-role anchoring without
supplying a successful assertion:

- Freeze the same synthetic failed-seed observation, both parents, initial guidance,
  source bytes, authority and requirement text before experiment generation. Do not
  include Trial 10's rejected experiment, reviewer verdict or these fixture answers.
- Four requests maximum to the same configured CheaperInference/deepseek-v4.1-flash,
  two paired comparisons with reversed order; 8192 completion tokens and 240 seconds
  per request, existing 16-KiB input bound. Zero target executions or state adoption.
- Baseline retains current projection; treatment adds only a neutral source-role
  label: current implementation under evaluation, not acceptance criteria. Hold
  other content and sampling settings fixed; record exact payload hashes and order.
- Freeze review criteria before dispatch: consistency of expected values with the
  supplied requirement, non-vacuity, actual second-aim coverage, structural validity,
  explicit uncertainty and finite limits. No expected winning assertion is supplied.
- Blind condition labels/order from independent semantic reviewers where possible;
  preserve every valid, invalid, abstaining or truncated result. Do not select only
  favorable outputs. An uncertain dispatch stops the comparison without retry.
- Report pairwise observations, not significance or reliable causal superiority from
  two pairs. No result changes original desires or authorizes a patch.

Exact shadow payload preparation and independent scope review would precede dispatch.
This proposal is not an existing admission and no Trial 11 is authorized. If the
comparison does not discriminate, retain the provider limitation rather than repeatedly
rewriting prompts or substituting assistant-authored tests.

Final verification: **14 focused tests passed** in 11.791s, covering the new oracle
contrasts plus existing review contracts and contract audit (`target/b91-trial10-oracle-audit-focused-final.log`). `git diff --check` passed. Trial 10 and all captured
prior journal/review hashes remain unchanged. Production code did not change; no new
whole-workspace or full-Python pass is claimed. This is offline coverage, not live
semantic evidence.
