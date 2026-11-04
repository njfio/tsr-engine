# Single reading: first live test (r2)

One authorized run of `reading@v1` on CheaperInference `gpt-6-luna`, at commit `4023903`, on the harder roguelike task from `experiments/continuous-game-v4`. One run cannot show a causal TSR advantage, and this one did not reach the parts of the design it was meant to test.

## Result

- **Outcome:** `proposal_abstention`. No game was built and nothing was executed. The source is still the empty scaffold.
- **Spend:** 11 calls, 36,769 microusd (about $0.037), no unknown calls.
- **Readings:** 4. No reading stated a want. All four filled `sense`, `belief` and `expect`, and set `want`, `expected_return` and `falsifier` to null. None was blank.
- **Returns:** 0, because nothing executed.
- **Proposals:** 3 implementations, all rejected by review, then an abstention on the fourth decision.

| Decision | Review | What the review said |
|---|---|---|
| 1 | Rejected | Routes truncated at 28 moves; a player-facing claim about tile collapse did not match the code; floor-generation gap |
| 2 | Rejected | Route text had no length bound, so it could break the 200-character limit |
| 3 | Rejected | Routes bounded at 200 characters, with a fallback message the review did not accept as showing the route |
| 4 | Not reviewed | Abstained rather than submit another unverified implementation with the same route-display and floor-generation gaps |

## What this does and does not show

- **Wanting did not occur.** Removing the menu and allowing a free want produced no want in four tries. This is the main finding. A blank or want-free reading is valid by design, so this is not a defect in the run. But it means the crossing, the falsifier and the changed-after-return count were not exercised live.
- **The prompt may contribute.** It says that fields may be null and that a blank reading "needs no reason", and an empty scaffold gives little to want from. This is a hypothesis, not a finding. Rewording it now would be a change made after seeing the result and would need its own approval and disclosure.
- **Criticism carried forward.** Each later proposal was rejected on a different defect, and the readings mention the earlier rejections. The route-length requirement is a real tension in the brief: three different treatments were rejected, which suggests the review may be hard to satisfy here, though one run cannot separate that from weak proposals.
- **Abstention was honest.** The stated reason is accurate about what was and was not observed, and it claims nothing was fulfilled.
- **Live check returns remain unobserved.** They are covered only by authored sealed tests.

## Comparison with v4f

v4f was on the same task and model. Its first stage had one rejection, then an approval and an execution, while r2 had three rejections and stopped. **This is not evidence that the reading hurts.** The comparison is uncontrolled, with one run on each side and model variance unmeasured. These things also differ:

- The reading turn and the menu-free proposal request.
- The review is unchanged, so its context still names development while the generator's does not.
- The feedback text omits the words naming a development route.
- The native command timeout moved from 120 s to 600 s.
- v4f's rejection limit and `correction_replay` were themselves changed after earlier results.

## Process

The first prepared run, r1, was rejected by independent binding review before any provider call, with four blocking findings: check returns were not recorded on the normal path, review criticism was lost after a reading, readings reached the reviewer, and the request still offered development routes. All four were fixed with tests, and r2 was accepted with limitations recorded in its `binding-review.json`. The observation and continuation stages did not run, because the protocol continues only from a retained artifact, so `continue.py` remains unexercised. No result was dropped, no game code was edited, and the run was not repeated.
