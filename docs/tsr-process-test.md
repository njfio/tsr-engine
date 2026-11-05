# The book's own test, run on the engine (spec #12)

*The Sovereign Mirror* makes one prediction you can check: when something is stuck, easing off beats pushing, and the ease is what does it, not just the time away (ch. 21, "Running the Test"). This page pre-registers the same comparison for the engine before any paid run. Write the predictions down first; don't edit them after.

## Arms

Both arms use the process profile (`tsr-process@v1`) with the same model, brief, scaffold, harness, review and authorization. They differ in one admission field:

- **Push (`process_context: full`):** the proposal request carries everything the engine has: every review criticism, assessment and return so far. This is Expectation at full strength (ch. 10).
- **Ease (`process_context: tuned`):** the request carries only the brief, the source, the notebook with its returns, the latest review criticism (as something that happened), the latest return of each kind, and the forms. Earlier history stays in the log.

Launch each with `experiments/tsr-process-v1/prepare.py`, `TSR_CONTEXT=full` or `tuned`, from the same commit, with separate labels. Run them one after the other, never sharing a ledger or native store.

## What to write down before any run

For each arm, the operator records in `target/tsr-process-<label>/predictions.json`, before dispatch:

1. What you expect the run to do: build, try, play, retain, abstain.
2. How many tries you expect to be followed by a rejection.
3. Whether you expect a want to be detected (new after a return), and what it might be.
4. What you expect back from the person who will play the result.
5. **The falsifier:** what result would make you think tuning made no difference.

## What is compared

All of these come from `experiments/tsr-process-v1/report.py`, which only reads the journals:

- **Stuck turns:** proposals until the first edit is kept and executed. Lower means less stuck.
- **Tries followed by rejection,** and whether the next edit fixed what the try showed.
- **Detected wants:** wants new after a return. The operator judges each one against ch. 18: is it traceable to the brief, or did it come from what came back?
- **The person's return:** after each run, the operator plays the final artifact and records what happened through the feedback path. The book's ethics make this the governing reading: if ease on the engine's page lands as weight on the person, theirs is the reading that counts (ch. 13, 21).

## How to read the result

- **Both arms the same on every measure:** the claim that load (Expectation) is what blocks this process is wrong for this engine. Record that plainly.
- **Ease ahead:** consistent with the book. It doesn't yet separate "less context" from "less noise". A follow-up would hold request size fixed and vary only whether earlier verdicts are framed as standing constraints or as things that happened.
- **Push ahead:** the book's direction is backwards here. Record it and take it seriously (ch. 21).

One pair of runs is one observation, not a result. The book asks for a dozen pages. Treat anything less as a note, and never as a causal claim.

## Not covered

- Whether the Current exists. Nothing in a notebook or a journal can show that (ch. 21).
- Whether the model "wants" anything. Detection lists candidates. Judging them is the operator's call.
- Any paid run. Each launch needs the operator's own approval.

## Runs, 2026-11-05

Three launches of `gpt-6-luna`, no spending cap, $0.318 in all. **The pair didn't produce a valid comparison:** the full arm on the same code as the tuned arm stopped on an engine gap at its first review. Predictions were not written down before any of them; the operator said to launch without them.

| Run | Code | Calls | Cost | Stop | What happened |
|---|---|---|---|---|---|
| p1-full | `0c59b31` | 27 | $0.0998 | `technical_stop` at proposal 9 | 4 drafts tried and rejected; the 5th passed review and executed, turning the scaffold into a turn-based game ("Turnfall"). Then 4 plays with the model's own controls: a shard pickup (body length 2) and floor 1 cleared into floor 2. Stopped when a malformed play (action kind `value`) wasn't treated as correctable. Fixed in `820fb85`. |
| p2-full | `4dd782b` | 3 | $0.0064 | `technical_stop` at review 0 | The reviewer cited the draft's browser try as an `observed_test_receipt`. Only controller check results count as receipts, and the refusal isn't correctable, so the run stopped. Not fixed: it is a change to edit review, which #11 and #12 put out of scope. |
| p2-tuned | `4dd782b` | 38 | $0.2118 | `technical_stop`: 12 review rejections | 12 drafts, each tried (10 exited 0, 2 with page or console errors) and each rejected. The model never played, so every try ran the protected checks only: a page load. Reviewers said the claims went beyond what a page load shows. The game is still the empty scaffold. |

- Detected wants (new after a return): 2 in p1-full, both refinements of the starting want after a try (from "become an observable browser game" to making Turnfall's crossing and growth "observably checkable through browser-controlled play"); 0 in p2-tuned.
- No person's return was recorded for any run.
- p1-full and p2-tuned ran on code that differs only in the correction fix, but p1-full never reached that path before its stop, and the two arms also differ in context. Nothing here says which arm is better. Each is one run.
