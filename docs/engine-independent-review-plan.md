# Minimal independent review integration

This is a prepared implementation plan, not an automated reviewer or new authority.
Current `project acceptance review` is an explicit operator attestation. No automated
reviewer calls were dispatched while preparing this plan.

## Smallest coherent change

Extend `project/acceptance.rs` behind its existing review/status/return contracts;
keep the shared decision kernel unchanged. Add a sibling `reviewer.rs` and typed
append-only records in `coding.rs`, exposed through `engine_project_cmd.rs`:

1. `ReviewAdmissionV1` — appended by the operator, names the current immutable
   project scope and requirements revision, a locally admitted verifier artifact
   digest, permitted evidence sources and per-criterion coverage, budget, expiry,
   and allowed result handling. Separately record generator identity. Merely naming
   a different model or identity in generated JSON is not evidence of independence.
   The admitted verifier must obtain candidate/test evidence through controller
   capture and cannot change candidates, tests, requirements or its own admission.
2. `ReviewRequestV1` — controller freezes admission ID, request ID, factual root,
   exact source snapshot, test receipt and requirements revision. Evidence is an
   allowlisted read-only bundle with content hashes, not the complete operator
   report or generator rationale. Include source/probe contents needed by the
   admitted coverage policy; never imply tests alone establish semantic coverage.
3. `ReviewReportV1` — quarantined observation with request ID, verifier execution
   receipt/artifact digest, per-criterion `met | not_met | unknown`, bounded evidence
   references and coverage limitations. Unrecognized criteria/fields fail closed.
   Record reviewer kind explicitly. Automated duration is not human review effort;
   unknown effort remains unknown. A report never directly emits AcceptanceReview.
4. Controller application — verify trusted request/receipt linkage and execution
   provenance, exact current bindings and unconsumed request; reject generator
   self-approval, stale/replayed/foreign reports, missing evidence, unknown-as-met,
   expired/exhausted admission and changed verifier artifacts. Unknown or conflicting
   evidence yields unresolved review; a fresh report may supersede but never erase
   earlier contrary evidence. No report can grant edit/execution authority.
5. Only an explicitly admitted deterministic verifier with complete declared
   coverage may produce a machine acceptance through the existing acceptance-return
   path. An open-ended semantic model report remains advisory until separate
   operator adoption or a separately justified adjudication policy is admitted.
   Keep `accepted_by_operator` distinct from any future machine-verifier status.

The initial implementation can stop at request/report validation and operator
adoption, without invoking a reviewer or claiming unattended semantic acceptance.
This closes provenance/integration first; it does not disguise manual judgment.
An automated model name cannot resolve the remaining coverage-policy dependency.

## Deterministic acceptance tests

Use existing synthetic invoice fixture without reference solutions in generation:
valid report is quarantined; absent admission, false role, generator identity,
wrong artifact, forged evidence source, partial criterion coverage, unknown result,
conflicting report, changed source/requirements/receipt, stale factual root, replay,
withdrawal, expiry/exhaustion and interrupted receipt all fail closed or remain
unresolved as specified. Restart must preserve consumed request IDs, unknown resource
charges and disagreement. Valid explicit operator adoption must preserve report
provenance and feed the existing desire/expectation return once. Disabled/unconfigured
reviewers must leave current operator behavior unchanged. No tests can establish
real reviewer independence by merely constructing self-asserted JSON fields.

## Inputs needed before real automation

An operator-admitted verifier implementation and coverage policy, independent
source/test evidence, explicit local resource limits and a defined disagreement
policy. For this invoice fixture a deterministic arithmetic verifier could cover a
specified arithmetic domain; it would not establish correctness for arbitrary code
or invent the currently unspecified overflow semantics. This is not permission to
install tools, call external providers, expand source exposure or relax requirements.

## Implemented contract boundary

The admission/request/quarantine/adoption contract is now implemented; see
[verifier contract](engine-verifier-contract.md). Production transport, enforced
provider cancellation/isolation and automatic acceptance remain disabled. The next
step is a reviewed trusted local adapter and a separately bounded shadow comparison,
not treating typed model findings as independent proof.
