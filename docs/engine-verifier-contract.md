# Quarantined independent verifier contract

The implementation is in `project/acceptance/reviewer.rs`, integrated with the
existing coding event log, project source capture, acceptance/return path and CLI.
It enables an explicit operator admission, controller-frozen request, trusted-host
adapter boundary, quarantined report and separate operator adoption. **Automatic acceptance remains disabled.** The subsequently implemented
[bounded local transport](engine-local-verifier.md) requires explicit configuration. No live model calls or
project executions were used to develop this boundary.

## Normal local interfaces

```
tsr engine --directory RUN project verifier admit --file admission.json
tsr engine --directory RUN project verifier request --admission EVENT_ID
tsr engine --directory RUN project verifier inspect
tsr engine --directory RUN project verifier revoke --admission EVENT_ID --reason REASON
tsr engine --directory RUN project verifier adopt --report EVENT_ID
```

`admit` requires `verifier-admission@v1`, the project configuration hash (the scope
shown by `project attention`), current requirements revision, distinct admitted
verifier/generator invocation identities, pinned verifier artifact hash, per-key
coverage paths, maximum requests (1–8), tokens/request (1–32768), timeout (1–240000
ms) and RFC3339 expiry no more than one day away. Coverage must name every requirement
and collectively include all admitted source/protected files and `test_receipt`.
The reserved evidence key cannot be a source path. This is an explicit additional
review admission, not inferred authority from successful generation or test results.

`request` consumes a durable request slot immediately. It freezes the exact captured
source digest, requirements, current test receipt, factual prefix, and evidence
bundle digest. It returns only source files, requirements and a controller-derived
receipt summary; no generator rationale or prior review conclusions enter the
verifier context. Tested source and current source are distinct; a stale test result
has no current verdict. Source capture is checked against its snapshot and freshness
is rechecked before persistence. The bundle is capped at 128 KiB.

The controller factual prefix excludes verifier bookkeeping only. Any substantive
engine event, requirement/source/test change invalidates the request. Revocation,
withdrawal, project closure and expired/stopped active project resources also block
use. Reading the operator inspection does not dispatch anything.

## Trusted adapter, not model self-attestation

`Engine::project_verifier_run` accepts a compiled trusted-host `Verifier` implementation,
not generated executable code or a JSON receipt. The controller matches its identity
and artifact to admission, records `Started` **before** invocation, supplies a fresh
separate evidence context, and binds the returned findings to that request. The
response cannot declare identity, role, source, budgets or authority. No CLI command
imports receipts, calls arbitrary executables or runs this interface today.

The future production adapter must independently enforce local endpoint/digest,
timeout cancellation, output/token bounds, resource metering and separation from
proposal invocation/context. The current core checks returned duration/token limits
and retains unknown/failed usage, but cannot interrupt an arbitrary Rust trait
implementation. Tests use an explicitly synthetic in-process adapter only. Distinct
identity strings—or using a different model—do not prove independent errors. The
same model in a separate context remains statistically correlated.

A started request cannot run again. Interrupted/failed/unknown work retains its full
reservation; request count never refunds. `Finished` preserves the response digest,
controller-bound findings or invalid-result diagnostic, measured adapter token usage
or explicit unknown, and measured host duration. It does not record human effort.
The core admission bounds requests and their reserved token/time ceilings; a future
production transport must connect these to the existing project resource journal
before any live dispatch. No operational metering/calibration claim follows from
synthetic adapter results.

## Findings and adoption

A report is a strict bounded JSON object with exactly `findings`, keyed by every
requirement. Each value has `finding` (`met`, `not_met`, `unknown`), the complete
admitted `evidence` reference set, bounded `explanation`, and nonempty `limitations`.
Malformed, extra-field, unsupported-path, missing-criterion or over-budget output
remains invalid and cannot be adopted. Findings do not alter acceptance or desires
until explicit operator adoption.

`adopt` checks freshness, provenance, unconsumed report and all competing valid
reports for identical source/criteria/test/evidence. A new bookkeeping root does
not erase disagreement. Unknown/conflicting dimensions remain unresolved; they do
not produce WrongBehaviour contrast. Known NotMet dimensions retain genuine contrast,
and known Met dimensions stay distinguishable. Legacy aliases sharing a preference
fulfill only when **every** alias is known met, independent of key order. Withdrawn
and out-of-scope desires never revive.

Even an all-Met report cannot bypass passing protected-test evidence bound to current
source. Adoption reuses existing acceptance validation and return mechanics in one
transaction and links the immutable report. Failed adoption does not consume it.
Successful adoption is recorded once; replay fails. Accepted status remains
`accepted_by_operator`, not machine or human review certification. Direct explicit
operator review remains available; no existing mode silently becomes automatic.

## Verification and remaining boundary

Tests use temporary synthetic projects/fake runners only. They cover quarantine,
positive integration after observed fake tests, separate adoption, no-test acceptance
refusal, stale source/requirements, withdrawn goals, wrong identity/artifact,
Generator-sourced forged records, malformed/uncovered reports, unknown/over-limit
resources, reservation exhaustion/interruption, revocation/restart, unknown contrast,
per-key fulfillment, alias order and disagreement across adoption.

No real independence, evaluator correctness or mature data is established by these
tests. Protected files and original target repositories remain unchanged. Evidence
and independent code-review summary are in `target/verifier-contract/`.

The [local transport and exact two-request shadow plan](engine-local-verifier.md)
now implement the transport prerequisite. The original evaluation scope remains: admit one pinned installed local verifier,
two separate requests/calls (each at most 240 seconds and 2048 output tokens) over
fresh frozen copies of the existing synthetic rejected and accepted invoice states.
Supply source/criteria/protected observations but no generator rationale or prior
review verdict. Preserve both results, including abstention/disagreement, and compare
against separate operator semantic review. No candidate execution, automatic adoption
or acceptance authority is needed. This plan is not a live-call admission. The first
production adapter must demonstrate hard timeout/restart/resource behavior before
that evaluation; no provider download, external transmission or credentials are implied.

Final validation: **170 CLI library tests passed, 3 existing ignores**; CLI all-target
clippy with `-D warnings`, offline build, changed-file rustfmt and `git diff --check`
passed. The normal CLI smoke admitted/froze/inspected/revoked a synthetic request,
rejected reuse and adoption of a non-report, and retained unaccepted status with zero
model calls/project executions (`target/verifier-contract/cli-smoke/result.json`).
Independent review findings were corrected and re-reviewed; its summary and check
logs are alongside that artifact. No protected test or original target file changed.
