# Refinement and expectation verification — 2026-10-21

The authorized continuation now completes all notebook predicates and survives restart.
This extends `eb09aec` with outcome-grounded generated subgoals, bounded operational
admission, and context-sensitive expectation transfer. The [implementation guide](engine-refinements.md)
describes the admission boundary. [Receipt hashes and counterfactual traces](engine-refinement-live-summary.json)
separate live evidence from synthetic controls.

## Observed live behavior

Both runs used the installed local `qwen2.5-coder:32b`, digest
`4bd6cbf2d094264457a17aab6bd6acd1ed7a72fb8f8be3cfb193f63c78dd56df`.
No reference answer, replacement proposal, or next task was supplied after initialization.

The three-note run (`target/engine-refinement-live/run`) made four local calls:
inspection, generated refinements, a refused action offer, and a corrected composition.
One refinement was refused because its parent/evidence was already covered; the other
was admitted as an explicit missing-note obligation under the original account want.
It entered ledger selection alongside account and brevity and was included in the
controller's predicted-effect vector and ranking. The model's corrected offer produced
an account meeting all four mechanical predicates. Its original wants were unchanged.

The first batch of the five-note run (`target/engine-refinement-transfer/run`) used its
**entire frozen ten-call budget**, including two failed calls that exposed an effort-scaling bug before
any action. After that fix, it performed three actions and three refinement reviews.
Two refinements were admitted; unsupported proposals were retained as declined reviews.
The final two action offers were refused. Its checkpoint observations were
`[0.6, 0.4, 1, 1, 0]`: original knowledge/account coverage and the second derived obligation
remained unmet at that checkpoint. It was recorded as unresolved. The later authorized
continuation below preserves that history and resolves those obligations.

A genuinely new inspection direction targeting the `transfer` note inherited evidence
from the earlier inspection of other notes in the same declared topic. Its forecast
was 0.6 instead of the uninformed 0.5; its score was 0.021 instead of 0.007875. The
counterfactual holds observation, selected desires, offers and belief constant and
removes only learned returns. **The winning index remained the same.** Thus this live
run demonstrates material score/forecast transfer, not a live winner reversal.

The generated refinement's participation in live selection/ranking is verified. That
does not prove the live winner would differ without it. A separate controlled test
establishes that causal effect by holding all other inputs fixed.

## Authorized continuation and completion

The next explicit allowance was ten additional local-only calls, at most 240 seconds
each. The runner appended `batch_authorized: schema-correction, calls: 10` to the same
journal; it did not reset the ledger, rewrite the original limit, or duplicate an
uncertain call. The named batch interface is idempotent and is now an ordinary operator
command. Reusing its name cannot multiply the allowance.

The second batch used all ten calls and completed on the last one. It initially took
an action that made no measured progress. That failed return stayed in the history,
released commitment persistence and informed later forecasts. The model then generated
further inspection/inclusion subgoals, those entered selection, and new actions covered
the missing evidence. There were no hand-authored replacement actions or refinements.

| Cumulative call | Observed result |
|---|---|
| 11 | No progress; original and derived unmet wants preserved |
| 12–13 | Generated inspection subgoal; action increased knowledge coverage to 0.8 |
| 14–15 | Account expanded to coverage 0.6 and fulfilled a derived inclusion obligation |
| 16–18 | Generated subgoal for the last unseen note; corrected offer completed inspection |
| 19–20 | Generated two missing-evidence inclusion obligations; composition covered all notes |

Final observations are nine ones: the three initial predicates plus six generated
operational subgoals. Final root:
`990e60d21729643d6bc2d225e269dfd4071222d5563692d266768ae3fff9011f`.
Restart returned the identical state with **20 total calls and one added batch record**;
it made no additional inference call. All calls have terminal receipts. The temporary
Ollama service was stopped and its exit confirmed.

The tighter eligible-index/vector-length schema is now verified live. One later offer
still overstated semantic support and was refused; the next model response corrected
it. Support links are now also reviewed individually: invalid links are preserved as
`rejected_supports`, while `proposed_supports` records the model's originals. A valid
remaining link can support the unchanged generated action. Actions, purposes and
predicted vectors are not rewritten. A new regression test covers that branch; final
call 20 exercised the new controller version with already-valid links. Offers with no
valid selected support remain refused.

Live counterfactuals continue to show changed forecasts and scores but the same winning
index. This is reported without forcing a different outcome. Controlled interventions
separately establish that learned returns and generated subgoal participation can
change selection. The live loop demonstrated ongoing responsiveness and recovery,
not an unnecessary winner change on every decision.

## Controlled causal and integrity checks

`cargo test --offline -p tsr-cli --lib`: **72 passed** (including 13 deliberation tests).
They cover:

- Desire-only, belief-only and expectation-only interventions, with repeated unchanged
  input and restoration controls.
- A generated, reviewed subgoal changes the selected action under identical observation,
  offers and history; removing its participation restores the prior stable winner.
- Learned returns reverse the ranking of a novel related direction, without relying
  on its identity. Unrelated declared topics receive no transfer; changed observed
  context attenuates transfer. This is structured evidence generalization, not keyword
  satisfaction or unrestricted semantic learning.
- Refinement lineage, cited actual contrast, restart, and contrary returns that reopen
  an obligation rather than deleting it or retaining a false fulfilled status.
- Fabricated provenance, unknown evidence and unsupported refinements cannot create
  wants or authority. Original wants remain unchanged.
- Honest partial inspection remains productive across 2, 5, 16 and 32 notes.

These tests use explicit synthetic interventions. They are not model-discovery evidence.

`python3 -m unittest discover -s scripts -p test_engine_directions.py`: **10 passed**.
Controls cover frozen budgets, uncertain-call refusal, terminal restart, last-call
completion, stale rejection feedback, state-specific provider schema constraints, and explicit idempotent continuation batches.
Workspace/all-targets Clippy with warnings denied, compilation and changed-file
format checks pass. Frozen M0 tests/assets remain unchanged. The previously documented
workspace failures and their runtime-only reproduction are in
[the earlier evidence](engine-directions-evidence.md#runtime-dependent-failures-inspected).

## Recoverable faults and remaining verification

Effort now scales as a fraction of admitted evidence, matching the coverage-based
benefit rather than making larger corpora inherently unattractive. Equivalent subgoals
cannot amplify themselves by merely reordering note IDs. Successful returns clear
stale rejection feedback. Forecast audit comparisons are versioned to avoid comparing
scores produced by different numerical kernels.

The last two live offers still used satisfied support indices or predicted no improvement.
The provider schema now restricts support indices to current eligibility, fixes the
anticipated vector length, and limits refinement parents to eligible typed obligations.
It supplies no action or answer. The schema tightening is now verified live in the
explicit continuation described above. The earlier budget was an operational checkpoint,
not a prohibition on authorized future local work. No budget was broadened silently.

A brief environment disconnect did not cause a duplicate call. There is no outstanding
uncertain inference, service, or approval blocker for this completed scoped run.

## What remains outside the demonstrated endpoint

The core bounded cycle now generates refinements from returns and admits verified
operational obligations into subsequent selection. It also transfers observed outcomes
across related novel actions. It does not establish general source truth, qualitative
understanding, arbitrary autonomous value formation, cross-domain semantic transfer,
or connection of this decision process to the coding executor. The notebook predicates
remain note coverage and a word budget. Unsupported semantic implications remain
unverified even when those predicates read 1.

The bounded notebook decision cycle is complete and its measured obligations are resolved.
This is not whole-system or general semantic autonomy. Live winner reversal was not
observed; the distinction between controlled causal tests and the live trace remains
explicit. Operator setup and judgment requirements are listed in the runnable guide.

No paid/external inference, push, merge, deployment, credential change or target-repository
edit occurred. The paid ledger SHA256 remains
`5a3c0ec91b19db3a32b687dd307e9152c62d122c3ef4a68dc0e56ea1255d9952`.
