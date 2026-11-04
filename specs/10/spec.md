# Spec #10: Generated wanting crosses without semantic review

Status: Draft. Needs author acceptance (P0: amends invariant 8 and adds a ledger event).

## Problem

Generated wanting becomes active only after an independent model reviewer approves it. `tsr_ledger::active::fold` keeps a generated preference out of the active view until an Operator status or a validated Council admission exists. This reverses the book's asymmetry, where the crossing into wanting is free and the crossing into action meets resistance (ch. 10). The book also says the filter is not a judge one party runs on another.

The review doesn't protect authority. Invariant 4 already limits authority to grants, allowlists, budgets and approvals, and a generated wanting isn't an acceptance aim.

## Decision

The author chose "Cross freely" on 2026-11-03, in answer to: "Should wanting stay gated by review, or cross freely with only action gated?"

## Scope

In:

- A new opt-in path that opens a generated preference with no reviewer, for the provisional, Contrast and success routes.
- The same for meaning development and relationships, if the plan's check confirms they share the cycle's admission path.
- Removing the model review call for a development choice on that path.
- A bounded view of crossed wanting in later requests.
- Amending PRD B2 and B6, and invariant 8.

Out:

- Any change to artifact review, scope grants, budgets or sandbox rules.
- Reinterpreting existing logs.
- The prompt's menu of forms and `development_declined`.
- Any claim about spontaneous or unassigned wanting.
- A paid test.

## Mechanical checks that remain

These are deterministic and read no meaning:

- Text is bounded and well formed.
- Contributors exist and are eligible for this task.
- Scope is this task.
- Origin is recorded as `Generator`, set by the controller.
- The binding to the current source is exact.
- An identical identity doesn't create a second entry.

## Acceptance criteria

- AC-1: Given an admission that opts in, when a well-formed generated wanting is proposed, then it is open in the active view with no reviewer call and no Council event.
- AC-2: Given that wanting, when the log is read, then its origin is `Generator` and it carries a distinct provenance from reviewed and Operator wanting.
- AC-3: Given a crossed wanting, when anything asks for execution, then scope grants, artifact review and acceptance aims are unchanged by it.
- AC-4: Given an edit paired with a wanting, when the wanting crosses, then the edit still needs its own review before execution.
- AC-5: Given a log or admission from before this change, when it is folded or revalidated, then the result is byte-identical, including generated preferences a reviewer rejected, which stay inactive.
- AC-6: Given a malformed, foreign-scope or wrong-contributor wanting, when it is proposed, then it is refused deterministically and nothing is appended.
- AC-7: Given a generated `StatusSet(Open)` or a forged crossing event, when it is folded, then it admits nothing.
- AC-8: Given more crossed wanting than the request bound allows, when a request is built, then all of it stays in the log and the request states how many were omitted.
- AC-9: Given a restart after a crossing, when the state is reopened, then the same wanting is open and is not crossed twice.

## Conformance cases

| Case | Input | Expected | AC | Tier |
|---|---|---|---|---|
| C-01 | Opted-in provisional wanting | Open; 0 reviewer calls; 0 Council events | 1 | Conformance |
| C-02 | Same, read from the log | Origin `Generator`; crossing provenance | 2 | Unit |
| C-03 | Crossed wanting, then a request to execute with no grant | Refused as before; aims identical | 3 | Integration |
| C-04 | Edit plus wanting, edit review rejected | Wanting open; edit not executed | 4 | Integration |
| C-05 | Existing fixtures, and a rejected expansion | Same head root and views; rejected stays inactive | 5 | Regression |
| C-06 | Over-long text; foreign scope; unknown contributor | Refused; log unchanged | 6 | Unit, Property |
| C-07 | Generator `StatusSet(Open)`; crossing event from a wrong source | Not admitted | 7 | Regression |
| C-08 | Bound + 3 wantings | Bound shown, "3 omitted", all in log | 8 | Unit |
| C-09 | Restart after crossing | Same view; no duplicate | 9 | Integration |

## Observable signals

- Development proposals that cross, with reviewer calls for development at zero.
- No change in what executes.

## Risks to judge when accepting

- **No semantic check remains.** A wanting can be banal, or can claim the user wants something. Provenance stays accurate, but the words are not checked.
- **More is not truer.** Removing review may produce more wanting without producing any that nobody assigned. It doesn't address the menu.
- **Self-influence.** Crossed text enters later prompts, so the generator can shape its own next proposals. Authority doesn't read it, but artifact proposals can.
