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

The first prepared run, r1, was rejected by independent binding review before any provider call, with four blocking findings: check returns were not recorded on the normal path, review criticism was lost after a reading, readings reached the reviewer, and the request still offered development routes. All four were fixed with tests, and r2 was accepted with limitations recorded in its `binding-review.json`. The observation and continuation stages did not run, because the protocol continues only from a retained artifact, so `continue.py` remains unexercised. No result was dropped and no game code was edited. r2 was not repeated unchanged; the rerun below changed the prompt.

# Rerun with a reworded prompt (r4)

One authorized run at commit `f083220`, on the same task and model. The operator chose this after r2 recorded no want. **The prompt change was made after seeing r2's result**, and it is the only runtime change from r2. r3 was prepared and reviewed but never run. Its review found that "Null is valid in any field" was false for a stated want, so r4 corrected that sentence.

The new wording asks "what do you want here?" first. It says to answer in your own words, "not by restating the brief or listing the work ahead", that a want "can be small, tentative or mistaken", and that stating one "commits nothing". It still allows null, and says to use it rather than invent something.

## Result

- **Outcome:** `proposal_retention`. A game was built and executed once, then kept unchanged at the checkpoint. Acceptance was not recorded.
- **Spend:** 16 calls and 16 bills, 50,936 microusd (about $0.051), separate from r2's 36,769 microusd. No unknown calls.
- **Readings:** 5, and all 5 stated a want with an expected return and a falsifier. r2 had 0 wants in 4 readings.
- **Proposals:** 3 implementations rejected by review, the fourth approved and executed, then a retention decision.
- **Returns:** 1, a controller check recorded beside the fourth reading: "Test result: unavailable".

| Reading | Want |
|---|---|
| 1 | "this scaffold to become a small, honest game whose crossing and growth rules are both visible and testable" |
| 2 | "the next version of this project to earn trust in its readouts: routes should account for the hazards they claim to navigate, and each arrow press should mean what the interface says it means" |
| 3 | "a small game whose displayed routes and growth rules are dependable enough that a player can act on them without guessing" |
| 4 | "the next attempt to make only claims its movement, body, and route logic can actually support" |
| 5 | "a source-bound browser run that clarifies whether the implemented crossing and growth rules work under actual controls, and whether the CSP error is merely cosmetic or blocks something important" |

## The game is not shown to work

The smoke check rendered the page, but exited 1 and reported `passed: false`, because the game applies an inline style that the page's Content Security Policy refuses. The harness reported this correctly. No control sequence was run, so neither criterion was observed: crossing a floor on the proven route, or growing from a shard. Whether the style error affects play is also unknown. The model's retention reason says all of this and claims nothing was fulfilled.

## What this does and does not show

- **The wording changed what was written, in this one pair of runs.** A want written in reply to a direct question shows the model answered it. It does not show wanting nobody assigned, and the book's sense of wanting is not measured here. The prompt's words also show up: two wants say "small", and that word appears nowhere in the task information.
- **Wants moved with what came back.** Wants 2 to 4 follow the review criticism, and want 5 follows the executed check. This is what the design is for, but it can also be restated criticism. Nothing separates the two, and the wants may have had no effect on the proposals. That was not measured.
- **Falsifiers were weak and partly rote.** The first four are near copies of "a closer inspection showing the source already contains the game". They test a belief about the scaffold, not the want. This is the form-filling risk the spec names. The fifth is more specific.
- **Expected and actual returns differed, with no verdict.** Readings expected browser control traces, and the one recorded return was a controller check. The first live check return is thin on its own: it names the receipt and "unavailable", not the style error. The model saw that error in the separate browser evidence, and its next reading cites it.
- **Reaching execution is not evidence the reading or the wording helped.** r2 stopped after three rejections and r4 was approved on its fourth attempt, one run each. v4f was approved on its second attempt with no reading at all, so r4 took more rejections than v4f. All three comparisons are uncontrolled, with model variance unmeasured.

## Still unverified

- Whether the game plays, and every observation and continuation stage. `continue.py` has still never run.
- An operator return, in any live run.
- Whether a stated want affects a later proposal.
- Whether wants persist under this wording across repeated runs.
