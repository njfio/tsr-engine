# Evidence-grounded operational refinements

[Historical live evidence](engine-refinement-evidence.md).

[Current selection corrections and domain boundaries](engine-selection-corrections.md)
supersede the earlier interpretation of subset reward: refinements guide generation
without gaining additional obligation weight.

The direction cycle now asks the installed local model to propose refinements after
an observed contrast, before it generates another action set. The controller binds the
proposal to the current root and the commitment whose observed return it cites.
A refinement contains generated text, an unmet parent index, and admitted evidence IDs.
There is no fixed refinement sentence or cycle-number recipe.

The controller can verify two bounded obligations: inspect missing notes under an
existing inspection want, or include missing notes under an existing account want.
It checks that the parent was unmet in the cited return, the IDs belong to that parent,
those IDs are still missing, the new identity/predicate is distinct, and budgets hold.
The original parent is not replaced, weakened or declared fulfilled. Unsupported
refinements are retained as rejected review results, not satisfied wants.

Accepted refinements retain `Source::Generator`, original lineage and actual contrast
provenance. A content-addressed controller review admits their **typed operational
obligation** to this notebook's selection view. It does not forge `Source::Operator`
or relax global ledger admission. General ledger commands still show their generated
origin; `direction-request` exposes `operational_refinements` and their review scope.
This domain-specific admission follows the user's explicit authorization for autonomous
refinement; it supplies no capability grant, command execution or external authority.

The text's broader meaning remains unverified. For example, the controller can verify
that a generated request to summarize evidence names notes missing from the account;
it cannot establish that its wording captures every semantic implication of “summarize.”
The typed obligation, source assertions, and model explanation are distinct.
The response exposes this uncertainty rather than calling a model assertion true.

The existing local entry point performs refinement and direction calls under one
cumulative frozen call budget:

```sh
cargo build --offline -p tsr-cli
target/debug/tsr engine --directory target/my-refinements direction-init \
  --spec examples/engine-directions/standing-wants.json
# With the installed local-only Ollama service configured as in the direction guide:
python3 scripts/engine-directions.py --directory target/my-refinements \
  --model qwen2.5-coder:32b --calls 10
target/debug/tsr engine --directory target/my-refinements direction-request
```

When a run consumes its allowance, continue the **same directory** with an explicit
named batch instead of editing or resetting the journal:

```sh
python3 scripts/engine-directions.py --directory target/my-refinements \
  --model qwen2.5-coder:32b --calls 10 \
  --batch continuation-1 --batch-calls 10
```

`--calls` remains the original allowance. The new batch appends its additional bound;
repeating the same name/count is idempotent, changing its count is refused, and another
batch cannot be added while an allowance remains. Uncertain calls still block continuation.
A batch is an explicit operator decision, never inferred from a generated desire.
The provider remains local-only, with each call bounded to 240 seconds.

`direction-refine --response FILE` is the adapter command. Stale offers and fabricated
return references fail; generated calls use controller-supplied root/return bindings.
Decoded refinement proposals and independent reviews are saved separately. A declined
refinement does not authorize a retry outside the frozen call budget.

## Expectation transfer

The controller folds actual returns from related prior directions. Relatedness uses:

- The same typed operation and overlapping **declared evidence topics**, not keywords
  in model explanations.
- Distance in observed knowledge/coverage, requested evidence fraction, and how much
  of the requested evidence is new to that notebook state.

A smoothed progress estimate and weighted effect residual are derived afresh from
those returns. A new direction ID can inherit evidence; an unrelated topic cannot.
The prior remains uncertain when evidence is sparse. Effort is normalized to the
admitted evidence set so a larger corpus does not suppress every honest partial step. Exact operation invariants still
bound projections, so inspection cannot erase an account. No learned parameter file or
stored weight is introduced. Frozen assessments cite the commitments supplying evidence.

This is explicit structured-context generalization, not unrestricted semantic learning
or an embedding model. Declared topics may be coarse or incorrect, and observational
successes do not establish causal effectiveness in arbitrary new settings. Generated
refinements can use progress evidence from a related operation without treating past
measurements of a different predicate as measurements of the new predicate.

## What the old `[1,1,1]` meant

It meant all admitted topic-note IDs were marked inspected, all were included in the
materialized account, and the account fit its word budget. The observer was independent
of the model's predictions, but those predicates did not verify source truth,
comprehension, explanatory quality or the whole TSR goal. New refinements use similarly
explicit predicates. `satisfied` is a domain measurement status; semantic status remains
unverified. Contrary later returns reopen affected obligations. Conflicting constraints
can remain unresolved until the bounded run ends.

## What is automatic and what remains operator work

After initialization, the runner requests directions, reviews proposals, commits one,
observes the notebook return, requests refinements, admits mechanically justified
subgoals, and generates the next directions without an operator supplying a next task.
It persists every stage and resumes from the same directory. Generated output never
increases the call allowance or changes executor authority.

The operator still supplies the initial wants, their typed predicates, declared evidence
topics and admitted source notes. Arbitrary prose goals are not automatically converted
into independently verified semantic criteria. The operator starts/stops the installed
local service, chooses a call allowance, and decides whether another bounded batch is
appropriate when a run is unresolved. Source truth and usefulness beyond the typed
predicates still require judgment; this controller does not certify them.

Inspect the current obligations and materialized account with:

```sh
target/debug/tsr engine --directory target/my-refinements direction-request \
  > target/my-refinements/result.json
```

That file exposes partial results as well as terminal ones, `semantic_status`, declined
refinement reviews, selected desires, and the learning counterfactual. An internal
`satisfied` result means the explicit notebook predicates were met; it is not permission
to publish, merge, deploy or treat source claims as true. The separate coding executor
is not wired into this notebook decision loop.
