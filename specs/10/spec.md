# Spec #10: A single reading each cycle

Status: Accepted by the author on 2026-11-04 (P0: amends invariant 8 and changes the ledger's wire format).

## Problem

v4f's generated wanting came from a menu of forms with a required decline reason, needed an independent model review to open, and never had to meet a return. This reverses *The Sovereign Mirror* in three ways:

- The crossing into wanting is free, and the filter is not a judge one party runs on another (ch. 10). The engine reviews it.
- Tune comes before Focus (ch. 7, 12). The engine starts at the work proposal.
- What goes out comes back from someone real, and the notebook protocol names who and what would show it wrong (ch. 13, 21). The engine records none of this, so the author's "waaaay too easy" never reached the system.

## Decision

The author chose on 2026-11-03: "Cross freely", then "I think it should be a single reading. Simplified and improved."

This replaces the earlier draft of this spec, which kept every wanting route and only removed its reviewer.

## The reading

Under the opt-in, each cycle starts with one reading, written before any work is proposed. All fields are free text, and the four directions sit in one entry because the book says they run together.

| Field | Direction | Meaning |
|---|---|---|
| `want` | Out For | What it wants |
| `sense` | In To | What it senses is current |
| `belief` | Out Through | What it believes now |
| `expect` | In From | What it expects, given the past |
| `expected_return` | | What should come back, and from whom |
| `falsifier` | | What would show this reading was wrong |

Rules:

1. **Nothing is required to be felt.** Any direction can be null, and a blank reading is valid and needs no reason. The prompt offers no forms and asks for no decline. After the first live run recorded four readings with no want, the operator approved asking for the want directly and first (2026-11-04). The prompt now also says a want can be small, tentative or mistaken, that stating one commits nothing, and to answer without restating the brief or listing the work ahead. These lines are meant to lower the threshold for a want and they discourage one kind of want. Nothing enforces them, and their effect is unmeasured. Any direction can still be null, and the prompt asks for no reason.
2. **A want crosses when it is recorded.** There is no reviewer. It stays recorded as generated. It lives in its own reading view, not among ledger preferences, because open preferences feed refinement parents and acceptance aims (see `specs/10/plan.md`).
3. **One current want per task.** A new reading's want replaces the last one in the active view, and every reading stays in the log. A want continues only if it is stated again, so persistence is visible without extra machinery.
4. **A want goes out with an expected return and a falsifier.** This is the only condition on wanting, and it is mechanical. Nothing reads whether either is good.
5. **Returns meet the reading, with no verdict.** The engine puts expected and actual side by side. It doesn't decide whether a reading was right. What the next reading does with a return is the signal.

## Scope

In, all behind the opt-in `reading@v1`:

- The reading turn, before the proposal turn. The reading request carries no action contract.
- The `Reading` and `Returned` ledger events and their fold.
- Returns from the operator, and from controller-recorded checks.
- Removing, for opted-in runs, the menu, `development_declined`, the development half of the deliberation envelope, and generated provisional, success, Contrast, meaning and relationship proposals.
- Amending PRD B2 and B6, and invariant 8.

Out:

- Any change to artifact review, scope grants, budgets or sandbox rules.
- Removing the old routes from the code. Old logs must still replay.
- Any detector of unassigned wanting, or any claim about it (ch. 18).
- Any engine judgement of a reading or return.
- Emotional evidence beyond the free text a reading can hold.
- A paid test as part of this implementation. The operator approved two separately on 2026-11-04 (`experiments/reading-live-v1`): r2, then a rerun with the reworded prompt.

## Acceptance criteria

- AC-1: Given an opted-in cycle, when it starts, then a reading request with no action contract goes first, and no proposal request is sent until a reading is recorded.
- AC-2: Given a reading with a want, when it is recorded, then the want is current in the reading view with no reviewer call and no Council event, its text is recorded as generated, and the view names the reading it came from. No ledger preference is created.
- AC-3: Given a later reading in the same task, when it is recorded, then only its want is current and earlier wants stay in the log. If the text is identical after normalization, the view counts it as restated.
- AC-4: Given a blank reading, when it is recorded, then it is valid, needs no reason, and leaves no want open.
- AC-5: Given a want without an expected return or falsifier, when it is proposed, then it is refused and nothing is appended.
- AC-6: Given an unbounded, malformed or foreign-scope reading, when it is proposed, then it is refused, nothing is appended, and no proposal follows. Regeneration stays within the existing allowance, then the cycle stops.
- AC-7: Given a return from the operator or a controller-recorded check, when it is recorded, then it is appended with its source and may name a reading or none. A generated return is refused.
- AC-8: Given earlier readings and returns, when the next reading request is built, then expected and actual appear side by side with no verdict, within a bound, with the omitted count stated. Everything stays in the log.
- AC-9: Given an opted-in run, when requests are built and responses validated, then no development forms or decline reason appear, and a generated development, provisional or relationship proposal is refused.
- AC-10: Given a current want, when anything asks for execution, then scope grants, artifact review and acceptance aims are unchanged by it, and an edit still needs its own review.
- AC-11: Given a log, admission or frozen experiment from before this change, or a run that doesn't opt in, when it is folded, revalidated or run, then the result is byte-identical, including rejected wantings, which stay inactive.
- AC-12: Given a generated `StatusSet(Open)`, or a forged `Reading` or `Returned` event, when it is folded, then it admits nothing.
- AC-13: Given a restart after a reading or return, when the state is reopened, then the view is the same and nothing is recorded twice.

## Conformance cases

| Case | Input | Expected | AC | Tier |
|---|---|---|---|---|
| C-01 | Opted-in cycle start | First request has no action contract; proposal blocked until reading recorded | 1 | Conformance |
| C-02 | Reading with a want | Current; 0 reviewer calls; 0 Council events; generated provenance; names the reading; preference view unchanged | 2 | Conformance |
| C-03 | Wants A, B, then B again | Only B current; A in log; one restatement counted | 3 | Unit, Property |
| C-04 | All directions null | Recorded; no want open; no reason needed | 4 | Unit |
| C-05 | Want with no falsifier | Refused; log unchanged | 5 | Unit |
| C-06 | Over-long text; foreign scope; bad JSON | Refused; log unchanged; no proposal request | 6 | Unit, Property, Fuzz |
| C-07 | Operator return naming no reading; check return; generated return | First two appended; third refused | 7 | Unit |
| C-08 | Bound + 3 readings, some with returns | Bound shown side by side; "3 omitted"; no verdict field; all in log | 8 | Unit |
| C-09 | Opted-in request; generated provisional proposal | No forms or `development_declined`; proposal refused | 9 | Integration |
| C-10 | Want open, no grant; paired edit rejected in review | Execution refused; aims identical; edit not run | 10 | Integration |
| C-11 | Existing fixtures; rejected expansion; frozen manifests | Same head root and views; rejected stays inactive | 11 | Regression |
| C-12 | Generator `StatusSet(Open)`; `Reading` from a wrong source | Not admitted | 12 | Regression |
| C-13 | Restart after each event | Same view; no duplicate | 13 | Integration |

## Observable signals

Reported per run, as counts only, with no claim about what they mean:

- Readings, blank readings, and reviewer calls for wanting (expected: zero).
- Wants restated across cycles, and wants changed after a return.
- Returns that named a reading and returns that named none.
- No change in what executes.

## Risks to judge when accepting

- **No semantic check remains on wanting.** A want can be banal, or can claim the user wants something. Only its provenance is guaranteed. Returns may correct it later, but nothing forces that.
- **A reading is still assigned.** Asking for it is a prompt, so this removes suggestion, not assignment. It doesn't produce or detect wanting nobody assigned. The direct question adds suggestion back: a want written in reply to it shows the model answered, not that it wanted anything unprompted.
- **Form-filling.** The six fields can become a new menu. A model can fill them in by rote, and a blank reading is the only way out it has. Word-level sameness across cycles would show this in the counts, but nothing prevents it.
- **Self-influence.** Readings enter later prompts, so the model shapes its own next proposals. Authority doesn't read them, but artifact proposals can be influenced.
- **Cost.** One more model call per cycle. It's the price of writing the reading before the proposal contract is in view. The reviewer call for development goes away.
- **Narrower expression.** One current want can't hold several at once, and meaning developments and relationships aren't offered in opted-in runs. The free text is the only place for them.
