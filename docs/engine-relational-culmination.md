# Relational aspiration loop

2026-10-22. Implements the conservative proposal loop from
[the interpretation review](engine-relational-culmination-review.md), grounded in
PRD B2.1/B6/B8 and the user's correction about desires becoming more through their
relationship. This is qualitative possibility guidance, not a numerical theory of
culmination or a claim of whole-engine completion.

## Operational path

The notebook direction path now exposes a `relational_request` after a current
observed return with structured contrast and at least two active original desires.
The ordinary `scripts/engine-directions.py --two-stage` driver uses a budgeted
relational generation stage before the next possibility stage. A frozen assessment
already pending completes first. Every request consumes the existing call allowance;
assessment-only mode cannot apply relational updates.

Fresh applicable relational guidance takes priority over optional mechanical
decomposition for the next possibility; pending frozen possibilities proceed to
assessment. This defers the refinement request without marking it completed or
changing its controller flag. See the [live scheduling diagnosis](engine-relational-a1-local-continuation.md).

The response proposes up to two aspirations, or an empty list. Each has canonical
original-parent identities, relationship text, newly wanted aspiration text, a
question that could discriminate it, optional prior revision and historical catalyst
references. These texts are generated hypotheses, not approved new criteria.
No built-in relationship, answer, scalar combination or action is supplied by this
implementation. Structural checks cannot establish that arbitrary prose is genuinely
novel rather than a paraphrase; that remains an explicit semantic assessment.

The exact state root, latest return commitment, observation revision and evidence
key bind the response. The request includes the actual chosen action and
identity-mapped before/after measurements and contrast. These remain adapter
predicates, never evidence of semantic understanding or a successful discriminator.

Accepted structural submissions append to the existing content-addressed event log.
A stable aspiration identity links immutable revisions. Empty submissions also record
that this evidence was considered, preventing repeated same-evidence requests.
The normal possibility context includes applicable relational guidance, while the
existing action schema, original obligation forecasts, selection, permissions,
budgets and acceptance remain unchanged.

After a subsequent return, old guidance becomes inactive pending reconsideration.
The next relational stage can revise or reaffirm its lineage against that return.
Existing hypotheses can be reconsidered even when mechanical predicates saturate;
without contrast, only eligible existing revisions or an empty review are admitted.
The driver performs a pending review before stopping at mechanical satisfaction,
within its existing call allowance; it does not reopen execution. See the
[runnable return-cycle check](engine-relational-return-cycle.md).
Predicate regression is labeled `parent_regression`; other subsequent
returns require review. Neither is automatically semantic disproof or validation.
Source changes and withdrawn parents remove applicability. Operator withdrawal is
separate from generated proposals. History remains inspectable across restart.
Historical catalysts support further relational proposals; they are not assertions
of current validity, new independent parents, permissions or semantic endorsement.

## Interface

Build and inspect without inference:

```sh
cargo +1.90.0 build --offline -p tsr-cli
./target/debug/tsr engine --directory RUN direction-request
```

The returned `relational_request.response_schema` is the response contract. Submit a
bounded response or operator status using:

```sh
./target/debug/tsr engine --directory RUN direction-relate --response RESPONSE.json
./target/debug/tsr engine --directory RUN direction-relation-status --update STATUS.json
```

Status fields are `state_root`, `identity`, `withdrawn`. Use the current root; stale
responses fail. No command here executes a candidate. The normal local provider
driver consumes the contract automatically within a separately admitted call budget.
Do not run it against the preserved failures under expired allowances.

Bounds: two proposals per consideration, 2–16 parents, up to four existing catalyst
revisions, 512-byte nonempty texts, 128 total revisions. Missing/forward references,
stale revisions, withdrawn parents and duplicate canonical text/parent combinations
are rejected. Alias parents collapse to their original; two aliases of one original
cannot qualify as a relationship. References to prior immutable revisions form an
acyclic provenance graph. Semantically equivalent paraphrases cannot be detected by
these structural checks. At history/context capacity the path stops explicitly;
no history is discarded to obtain another call.

## Verification and limits

Deterministic Rust tests exercise real notebook controller returns and persistence,
revision/restart, original-want and authority invariance, duplicate/alias/forward
reference rejection, withdrawal, observation drift and provenance rejection. A
separately labeled synthetic loss fixture tests regression feedback; it is not a
live semantic counterexample. Python tests use mocked providers and controllers to
verify serialized return evidence, relational-to-possibility propagation, interruption
reconciliation, assessment-only protection and consumed budget across restart.

Independent review found and corrected an omitted-return-content defect before
handoff. Final focused review found no remaining blocker. Checks passed: five
focused Rust tests; full CLI library 202 passed, three existing ignored; 75 Python
driver tests; CLI all-target Clippy with warnings denied; offline binary build;
changed-file rustfmt and diff whitespace checks. Test logs are under
`target/relational-cycle/`; no live inference or project executions occurred.

Implemented: persistent non-single-parent hypotheses, fresh return-driven review,
and normal possibility-context integration. **Independent weighting is not
implemented.** Aspiration endorsement/admission as a separately scored preference,
semantic discriminator evaluation and integration into the project adapter remain
outside this minimal path. The notebook adapter still cannot implement arbitrary
new forms of action. Original wants, prior failed trials and the impossible literal
coverage/brevity combination retain their meaning.

The smallest meaningful next live validation needs a fresh admission: a frozen
local notebook state with two standing wants and a real contrast; one generated
relational proposal; ordinary possibility and assessment calls; one independently
measured already-admitted reversible notebook action; then one relational revision
call. Four calls suffice for these stages if each succeeds (each at most 240 seconds
under the existing context/token limits); any required decomposition stage also
consumes a slot and may stop the run. Keep all failures, unchanged controls and
actual return evidence. Review whether the generated relationship expresses more
than conjunction and influences a proposed direction, not merely whether fields
are present. Original satisfaction and semantic truth must be assessed separately.
This plan is not a new inference authorization and does not revive the stopped
pinned-configuration probes.
