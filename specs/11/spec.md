# Spec #11: A want for a missing ability gets acted on

Status: Reviewed (P1, multi-module: flagged for human review in the PR)
Milestone: Wanting crossing. Follows #10.

## Problem

In run r4 the model's last want was a run of the game under real controls. It then kept the game and said it had "no admitted action" to do that. Nothing in the engine could act on this. Retention ended the run, and playtests are prepared by an operator, stage by stage, with a new review for each. So the run couldn't correct itself against the game actually running.

The purpose of the loop is that the process reaches this want itself and the want decides what comes next.

## Rules

1. **Opt-in, on top of the reading.** `abilities_profile: want-abilities@v1` needs `reading@v1` and a `spend_cap_microusd` above zero. A run without it behaves exactly as before.
2. **A need.** A proposal may return `{need: {what, because}}`, naming an ability it lacks in its own words. Nothing reads its meaning. It is recorded beside the latest reading and grants nothing by itself.
3. **Asking reveals what is in scope.** `play` is not offered until a need has been stated. The first need reveals it, whatever the need's words say. No judge decides whether the words match.
4. **Play.** `{play: {actions, checks}}` runs the current source in the protected browser harness, in the same no-network, read-only container as the existing checks, with the model's own clicks, keys, waits and checks. It changes no source and needs no review, because it changes nothing. The result comes back beside the latest reading as written, in order: exit code, errors, readouts, then the action list, cut at the return's byte bound with a note. The engine adds no verdict. When a control fails, the harness itself exits 2 and reports only that error, not what came before it.
5. **A need beyond scope goes to the operator.** A need stated when every ability is already revealed ends the run with `need_outside_scope` and its text. Nothing is built or granted automatically.
6. **Retention.** Retaining with new evidence since the last retention (an executed change, a play, a need, a review rejection) takes another reading. Retaining with nothing new ends the run with `proposal_retention`.
7. **Spending cap.** Before every model dispatch, including source inspections, format corrections and recovery, the engine compares the campaign's committed spend with the cap. At or above it, the run ends with `spend_cap`. A call already under way can overshoot by that one call, which is the most a run can overshoot.
8. **Action stays gated.** Edits still need independent review, execution still needs its binding, and acceptance is never recorded. A play can't reach any of them.
9. **No retries of uncertain effects.** A play is an accounted execution. A crash during one blocks the run, and a completed play is never run again on restart.

## Acceptance criteria

- AC-1: Given no abilities profile, when a run proceeds, then its prompts, forms and stops are unchanged, and a `need` or `play` is refused.
- AC-2: Given the profile without a reading profile or a positive cap, when it is admitted, then admission is refused.
- AC-3: Given a run with the profile and no need yet, when a proposal is requested, then `need` is offered, `play` is not, and a `play` is refused.
- AC-4: Given a need, when it is recorded, then it appears beside the latest reading, no reviewer is called, the next turn is a new reading, and `play` is offered from then on.
- AC-5: Given a revealed `play`, when it is proposed, then the harness runs on the current source, no source changes, no reviewer is called, and the result appears beside the latest reading with no verdict.
- AC-6: Given a play that fails, including a harness refusal or a non-zero exit, when it returns, then the failure is the return and the run continues.
- AC-7: Given every ability revealed, when another need is stated, then the run ends with `need_outside_scope` and the need's text, with desires open and no acceptance.
- AC-8: Given a retention with new evidence since the last, when it is proposed, then another reading follows; given none, then the run ends with `proposal_retention`.
- AC-9: Given committed spend at the cap, when any model call would be dispatched, including an inspection or a format correction, then no call is made and the run ends with `spend_cap`.
- AC-10: Given an edit after a play, when it is proposed, then it still goes to independent review, and a rejection leaves the source unchanged.
- AC-11: Given a restart after a need or play, when the run resumes, then nothing is recorded or run twice; given a crash during a play, then the run blocks.
- AC-12: Given malformed `need` or `play` values, when they are checked, then they are refused with no effect, and random values raise nothing but `ValueError`.

## Scope

In: the need and play forms, the outer stops above, a harness runner, and a launcher with one launch and one binding review.

Out: changes to the protected harness, edit review or acceptance; Rust changes; judging what a need means; other abilities; any paid run (separate approval); removing the older stage scripts, which frozen experiments still use.

## Conformance cases

| Case | Input | Expected | AC | Tier |
|---|---|---|---|---|
| C-01 | Reading-only run; `need` proposed | Refused; prompt and forms byte-equal to before | 1 | Regression |
| C-02 | Profile with no reading; cap 0, negative, missing or bool | Admission refused | 2 | Unit |
| C-03 | First proposal request; `play` proposed | Offers `need` only; `play` refused | 3 | Conformance |
| C-04 | `need` then a retain | Return beside the reading; 0 review calls; reading N+1; `play` offered | 4 | Conformance |
| C-05 | Need, play with click and check | Runner got the current source and the model's controls; source same; 0 review calls; return carries errors and readouts | 5 | Conformance |
| C-06 | Runner reports exit 1, exit 2, timeout | Each comes back as a return; next reading follows | 6 | Functional |
| C-07 | Need, need | Ends `need_outside_scope` with the text | 7 | Conformance |
| C-08 | Retain after a need; retain again | Reading, then ends `proposal_retention` | 8 | Conformance |
| C-09 | Committed spend at the cap; real ledger with repeated inspections and bad shapes | No further call; ends `spend_cap` | 9 | Conformance, Integration |
| C-10 | Need, play, edit | Reviewer called for the edit | 10 | Integration |
| C-11 | Restart after a finished run; process killed right after a play; runner raises mid-play | Runner called once in each; last case blocks | 11 | Integration |
| C-12 | Bad shapes; 20k random values | `ValueError` only | 12 | Unit, Fuzz |
| C-13 | Real harness in the local image on a fixture game | Report parsed; clicks and readouts present | 5 | Integration |

## Observable signals

Counts per run, with no claim about what they mean: needs stated, turn of the first need, plays, plays followed by an edit, edits followed by a play, retentions, and the stop reason.

## Risks to judge when accepting

- **`need` is itself a prompt.** Offering the form suggests it, so a need shows the model used an offered form, not that it wanted anything unprompted. The prompt doesn't name `play` until a need is stated, which is the only protection.
- **Asking always unlocks.** A rote need on the first turn reveals `play` at once, so a run can skip the want entirely. Turn of the first need, and whether a want preceded it, are reported.
- **Less review.** Independent review happens once at launch, not per stage. Mistakes mid-run meet only the in-run reviewer, and that reviewer never sees a play.
- **Page text enters prompts.** Readouts are generated by code the model wrote and come back as text. They grant nothing, but they can influence later proposals.
- **Self-assessment.** The model writes its own checks, so a run can pass checks that test nothing. The harness reports what happened; nothing judges whether the checks were good.
- **Loops.** A model could play forever without changing anything. Only the cap stops this. Each play is followed by paid calls, so the cap does bind.
