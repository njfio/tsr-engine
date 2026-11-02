# Snapshot-bound verification without replaying an edit

`tsr engine --directory RUN project verify-admit --state-root ROOT --snapshot HASH`
prints a normal existing-action response. Obtain ROOT from `shadow-view` and HASH
from `project status` → `goal_completion.snapshot`; save the response, then use
`project run --response FILE` (or `--commit-only` followed by `project resume`).
Admission requires a fresh state, exact current source, prepared named Cargo scope,
open explicit resource budget, no pending/uncertain work and sufficient tool/wall
reservation. Model proposals cannot create this operator admission.

The action records state/source/configuration provenance and reconstructs only
from Operator evidence. It is typed `RunApprovedTest`, not `ProposePatch` or a new
intuition. Normal selection, frozen controller forecasts, durable intent, sealed
execution, receipts and independent acceptance are reused. It writes no patch:
commands retain the existing stage layout with two fixed `true` placeholders, then
the approved named/full Cargo commands. Source-before equals expected-after;
protected and current source integrity are checked. Drift, repeated attempted
commitment and uncertain execution refuse. Old stale-patch validation is unchanged.

Cost estimation uses compatible zero-edit receipts or explicit command bounds.
Normal tool/wall reservations apply; no patch bytes are charged. Zero model/token/
patch limits now explicitly forbid those resources while wall budget must remain
positive. This is not free computation, calibrated prediction or a new execution
permission derived from model output. The 128 verification-admission bound limits
record growth. Prior completed episodes are never reopened.

## Tests and boundaries

Focused tests cover unchanged verification, source drift, stale root, forged
provenance, restart, repeated/uncertain execution, retained reservation, exhausted
budget, zero model/patch allowance, independent reviewed closure and refusal after
closure. Export JSON deserialization retains `RunApprovedTest`; reopen retains zero
patch charges. This is not an independent dataset-reconstruction proof. Existing
maturity/support/leakage gates remain unchanged. Synthetic fixtures do not establish
operational calibration. Independent code review found no remaining blocker.

## Separate Qwen32B candidate verification

Evidence directory: `target/verification-only-episode/`. It copies exactly the 323
files from the preserved Qwen32B candidate; SHA256 inventory and prior proposal/
closure links are in `provenance.json`. No new candidate or inference was produced.
The earlier rejected closure remains immutable. The prospective origin is `replay`,
so this is not new independent operational training evidence.

The edit-window/read-only suffix offsets shift by -121 bytes to describe the already
shortened candidate. Patch budget is ZERO: verification cannot exploit that metadata
to edit. Model calls/tokens are ZERO. New frozen budget: one verification execution,
600,000 ms per command, 3,000,000 tool-ms including preparation and 3,600,000 wall-ms,
zero external charge. The prior 240-second full oracle reached runtime tests but did
not finish; 600 seconds supplies bounded headroom within the existing named-project
maximum. Each commitment creates a fresh scratch directory; no warm-cache claim.
This changes no previous admission or budget. The pinned image/test commands and
previously disclosed optional-hook exclusion remain unchanged.

Terminal result: named test passed (118,443 ms); full suite returned exit 125
(57,762 ms) during compilation, with no test results. Cause is undetermined:
3,192 captured stderr bytes are below the output cap; retained scratch has 4,507
entries, 2,083,803,427 regular-file bytes and no nonregular entries, below limits.
These post-run checks do not reconstruct transient execution conditions. No retry
was made under the consumed one-execution allowance. Next step is executor
termination-cause instrumentation/diagnosis, then a separately admitted verification
if justified; neither increasing timeout nor changing candidate is yet justified.

All 323 candidate SHA256 hashes remain unchanged. Independent review confirms the
three parsing requirements but rejects promotion for incomplete full-suite evidence.
The new episode is separately closed rejected; the old rejection is untouched.
Charged tool time is 183,963 ms including preparation; model calls/tokens, patch
bytes and external charge are zero. No accepted export was produced.

Final CLI library check: 139 passed, three ignored. Clippy all-targets with warnings
denied passed. Focused tests include state drift after admission and mismatched
configuration provenance. The active configuration is immutable (first admission
wins); appending another configuration is not a scope change.
Active independent reviewer effort remains unknown. Test coverage and semantic
review can establish scoped repair acceptance, but no positive complete-resource
coverage attestation or mature training eligibility will be fabricated.
