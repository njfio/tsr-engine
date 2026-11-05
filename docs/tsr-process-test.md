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
