# Spec #12: Let the process run the way the book says it runs

Status: Draft, needs operator review (P1, multi-module). Built and tested offline; no paid run.
Milestone: Wanting crossing. Follows #10 and #11.

## Problem

Read against *The Sovereign Mirror*, runs so far have worked against the process the engine is named for, in four places:

- **Wants are asked for.** The reading prompt, the `need` form and `provisional_preference` all ask the model to want. The book's threshold is "a want its builders didn't put there", produced by contrast with what came back (ch. 18). A want given in answer to a question is an assigned objective. The r4 write-up says the same.
- **Action is blocked behind review.** Intuition gets past Expectation only by tipping Belief into an action. That action goes Out Through, and only what comes back In From can re-bind Expectation (ch. 10). In want-loop r2, two edits were rejected before they ran, so nothing came back and the game stayed an empty scaffold. The run's turn-2 want ("tell when the mechanics are real rather than just described") was produced by a rejection, which is contrast. But that want had no way to act, and Focus closed the loop (ch. 17).
- **Requests are mostly Expectation.** The v2 proposal request was about 197,000 tokens of history, verdicts and caveats. Expectation is the strong force. More of it is more of what blocks the weak signal (ch. 10). Pushing harder with rewordings is Focus against a gap made of holding on (ch. 11–12).
- **Only one kind of evidence is read.** Runs keep empirical returns. Neither runs nor reports read the felt gap between what is wanted and what is lived, nor whether that gap narrowed (ch. 8, 21).

## Rules

1. **Opt-in.** `tsr_process_profile: tsr-process@v1` needs `reading@v1` and `want-abilities@v1`. It also takes `process_context` (`tuned` or `full`). A run without it is unchanged.
2. **Try before review.** Each proposed edit first runs in the protected harness, in the same sealed container as plays.
   - It runs on a candidate copy: current source with the edit applied.
   - The controls are the model's latest play, or the protected probe if it hasn't played.
   - Its report goes to the reviewer as evidence, labelled like play results.
   - The result is recorded beside the reading once the decision settles, after a rejection or after execution.
   - Review still decides what is kept, and a try changes no source. A try is an accounted execution and is never retried.
3. **Play from the start.** `play` is available from the first proposal, with no `need` step.
   - The `need` form is not offered. A `need` is refused.
   - Asking is what this profile removes.
4. **The notebook replaces the want question** (ch. 21).
   - The reading asks for what is true now, in the notebook's terms: want, hunch (`sense`), belief, expect, how far apart those are (`gap`), what should come back and from whom (`expected_return`), and what would show it wrong (`falsifier`).
   - It does not ask for a want and does not encourage one. Any field may be null.
   - The first page is never edited. Readings are append-only already.
5. **Wants are detected, not counted.** The report lists each want that is new compared with the reading before it and follows a return (a try, play, check or rejection), with the return that preceded it.
   - Nothing judges whether such a want is traceable to the task. The operator does, with the list in front of them.
   - A want in the first reading never counts, because nothing has come back yet.
6. **Tuned context** (ch. 7).
   - With `tuned`, the proposal request keeps only these:
     - the brief
     - the source
     - the notebook with its returns
     - the latest review criticism, as a return
     - the latest of each kind of controller return
     - the forms and contracts
   - Older history stays in the log and out of the prompt. `full` keeps today's request.
   - The two make the push and ease arms of rule 8.
7. **A person's return** (ch. 13). After a run, the operator plays the artifact and records what happened through the existing feedback path. The report shows whether a person's return exists for the final source.
8. **The book's own test** (ch. 21, Running the Test), pre-registered in `docs/tsr-process-test.md`.
   - Compare the `full` and `tuned` arms on the same pinned starting state.
   - If they come out the same, the claim that load is what blocks the process is wrong for this engine.
9. **Retention isn't forced** (applies to every profile). `development_declined` becomes optional: it is recorded when given and never required. Requiring it was Focus.

## Acceptance criteria

- AC-1: Given no process profile, when a run proceeds, then prompts, forms, review input and stops are unchanged.
- AC-2: Given the process profile without the reading or abilities profile, or with an unknown context profile, when it is admitted, then admission is refused.
- AC-3: Given the process profile, when the first proposal is requested, then `play` is offered, `need` is not, and a `need` is refused.
- AC-4: Given an edit, when it is proposed, then the harness runs once on a candidate copy before the review call, and the review input holds that report.
- AC-5: Given a try whose edit does not apply (`old_text` not found exactly once), when it is attempted, then nothing runs and the reviewer is told the try could not be built.
- AC-6: Given a try, when the review rejects it or the change executes, then the try's report is recorded beside the latest reading exactly once.
- AC-7: Given a restart after a try, when the run resumes, then the try is not run again; given a crash during a try, then the run blocks.
- AC-8: Given the process profile, when the reading is requested, then the prompt holds no instruction to state a want, and `gap` is accepted.
- AC-9: Given a list of readings and returns, when the report is built, then a want counts as detected only if it differs from the previous reading's want and a return lies between the two readings.
- AC-10: Given `tuned`, when the proposal request is built, then it holds at most one item of review criticism and one of each controller return kind, and it is smaller than `full` for the same state.
- AC-11: Given any profile, when a bare `{retain}` is returned while development forms are offered, then it is accepted.
- AC-12: Given malformed process admission values, then admission raises only `ValueError`.

## Scope

In:
- the profile and its admission;
- try-before-review;
- play from the start;
- the notebook reading and its `gap` field (an additive `contract:` change);
- detection in the report;
- tuned context;
- the optional `development_declined`;
- the test protocol doc.

Out:
- any paid run (separate approval);
- changes to edit review, acceptance or the protected harness;
- judging what a want means;
- removing older profiles.

## Risks to judge when accepting

- **The notebook still names "want".** The book's notebook does. It records, it doesn't solicit. That is why detection is by provenance rather than by the field being filled.
- **Tries cost an execution each.** Unlimited runs have room; capped runs need room in the campaign.
- **The reviewer sees more generated page text.** It is labelled as data and grants nothing, as for plays.
- **Tuned context can drop something that mattered.** That is the experiment. The `full` arm is the control.
- **No live evidence.** Nothing here shows that a model will want anything unprompted. It removes structural reasons it couldn't.
