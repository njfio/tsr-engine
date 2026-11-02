# Independent review: semantic desire status and choice

Reviewed the working diff from `9babd3c`, 2026-10-25. Read-only review of
implementation and recorded deterministic test output; no model calls, target
execution, generated-code adoption or acceptance action by this reviewer.

## Finding and narrow repair

`acceptance_kernel_inputs` previously treated every current semantic aim as an
admitted ranking obligation, even after operator withdrawal or an out-of-scope
status. Attention already excluded those aims. The new regression reproduced the
inconsistency before the repair (`target/semantic-choice-withdrawal-red.log`).

Filtering the existing folded ledger to `Open | Fulfilled` before grouping semantic
aims is appropriate. Fulfilled standing desires retain their loss protection;
withdrawn, out-of-scope, overlapping and superseded desires no longer contribute
ranking obligations. Grouping still occurs by preference identity, preserving the
conservative alias conjunction. No history, requirement, review or forecast is
rewritten. Operator status authority remains enforced by the ledger fold.

The paired fixture holds source, two alternatives, costs and reviewed return fixed.
Withdrawal and out-of-scope statuses remove the semantic obligation and its ranking
bias; restoration recovers exact prior kernel inputs and option views across
restart. The existing actual-selection assertion remains after restoration. The
focused test passed after repair (`target/semantic-choice-withdrawal-after.log`).
This is deterministic mechanism evidence, not model-generation quality or causal
benefit of relational guidance.

Acceptance still checks the complete immutable requirement map independently of
the ranking fold. Removing pressure from selection cannot itself satisfy a
requirement or authorize an action.

## Directly adjacent consistency item — resolved

At review time `adaptive.rs::next_unlocked` still appended every current semantic
aim to `Candidate.support`, including inactive ones, after the ranking fix excluded
them. This does not reverse the score repair, but the new committed-candidate trace
and downstream shadow input should not claim support for an operator-withdrawn aim.
The final diff factors `acceptance_active_aims` into one folded-status helper used
by both the kernel and the `Candidate.support` extension. The new
`selected_support_excludes_inactive_semantic_aims_without_executing` test checks
withdrawn and out-of-scope cases, records a selected commitment with only the
active semantic aim, and verifies unchanged source, unchanged requirement
identities, no completed execution, no acceptance, and restart persistence. It
passed (`target/semantic-choice-support.log`). This review item is closed.

The added relational fixture also checks the complementary invariant: an authored
and explicitly admitted revision reaches the next ordinary proposal request after
restart, while the fixed option scores and learned expectation inputs remain
unchanged. The comparison excludes only `resource_cost.source_prefix`, asserting
that field exists as a string before removal: `cost.rs` intentionally hashes the
whole event sequence, so newly recorded guidance advances this audit identity.
All numerical costs, source samples, scores, projections and remaining fields are
still compared. This corrects a test expectation, not a production cost policy.
A later unknown semantic return invalidates that guidance without
erasing either revision. This is appropriate evidence of qualitative integration,
not an invented direct weighting rule. That final fixture extension was reviewed
in source; its subsequent execution belongs to the parent's affected-test report.

The fixed-alternative fixture additionally supersedes a rejection with an unresolved
review. Both actions then have unknown observed semantic value, a 0.5 prior, no
review evidence and equal scores; restoring rejection restores differentiated
ranking. This distinguishes unknown from an unmet condition while preserving
immutable original reviews and deduplicating by tested receipt. Source, options,
resource costs and authority are held fixed; no generated-code execution is added.

No remaining blocking finding in the final source diff. Full aggregate verification
is separate from this review and must report any failures honestly.

Final CLI log reviewed: `target/semantic-choice-cli-tests.log` reports **235 passed,
0 failed, 8 ignored**, 55.14 seconds. The parent used supported permission for local
fake-provider socket tests; no real provider calls or target executions were enabled.
The scoped Clippy/build/rustfmt results are reported by the implementer. Python
aggregate was still pending at this review checkpoint; no Python pass is inferred.

Reviewed `docs/engine-semantic-guidance-choice.md` and its linked mechanism-map
update: they correctly distinguish deterministic choice mechanics, qualitative
guidance propagation, and unproven usefulness/reliability/causal benefit. A smaller
future discriminator than another executed cycle is two bounded calls in a
disposable synthetic review copy: a return-grounded revision, then a genuine next
proposal, independently assessed and validated without executing/adopting that
proposal. This requires offline reviewed transport support for semantic-return v2
first; the current external guard must remain closed until that support is explicit.
Such a test would establish proposal relevance only, not an actual improved outcome.

No independent weighting of qualitative relational aspirations is proposed. The
normal proposal context already exposes them; any empirical effect on generated
possibilities remains a separate question.
