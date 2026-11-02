# Workshop normal-driver integration

Implements the three gates left by `ec524f9`. The frozen scenario, parent desires,
return catalog, semantic limits and rubric are unchanged (plan hash
`4cb905c9511c5e865bfe8940b8554f8a17e37f0cdadbb97187f251122241c756`).
No live calls or external actions occurred in this implementation turn.

## Implemented behavior

- Registry accepts only the two named, content-pinned synthetic documents. Workshop
  scope requires exact parents/world and exports the frozen semantic boundary in every
  phase. No arbitrary source paths or new provider are accepted. Historical archive
  admissions retain their default path and behavior.
- Workshop preparation and admission loading require the exact four initial events,
  pending first relation and no prior relational history. The initializer performs only
  those evaluator-authored setup events, never inventing an aspiration or next action.
- Empty initial relational output stops as `empty_initial_aspiration`, before controller
  mutation. Its settled receipt remains; unused calls expire, and restart cannot fall
  back to ordinary action generation.
- After one action and its return review, `possibilities_final` generates and validates
  one final possibility batch. It never creates an apply intent, assesses the batch or
  executes another operation. The stored applied marker is a driver-stage receipt,
  not an action; restart terminates `proposal_only_complete` without another dispatch.
- Typed epistemic roles and semantic statuses remain unverified guidance, not authority.

## Runnable entry point

Preparation is offline and refuses an existing target directory. Its parent must exist.
Use an explicit timezone-aware future expiry; there is no automatic renewal:

```sh
mkdir -p target/workshop-demo
python3 scripts/engine-workshop-prepare.py \
  --directory target/workshop-demo/run \
  --expires '<explicit future ISO-8601 timestamp with timezone>'
```

After separate live admission, the normal driver command is:

```sh
python3 scripts/engine-directions.py \
  --directory target/workshop-demo/run \
  --model deepseek-v4.1-flash \
  --synthetic-admission target/workshop-demo/run/admission.json
```

No manual stage commands, candidate edits or supplied aspirations are needed. Inspect
`synthetic-external/terminal.json`, typed outcomes and the last possibility batch.
Restart uses the identical command/admission. Uncertain dispatch or application blocks;
do not reset directories or renew budgets to retry uncertainty.

## Prepared manifest (undispatched)

`target/workshop-ready/run/admission.json` freezes actual source, binary/script hashes,
scenario hash and expiry. Initial root:
`871b0ecf09b010fb547a8edcd13922da2daecaeecb36d49073df83a083816517`.
This artifact alone is not permission to call the provider.

Bounds: **5 calls, 1 simulated operation**, 65536 reserved completion tokens, 1200 seconds
maximum transport. Stages are relation / possibilities / assessment / return relation /
final possibilities, with completion limits 8192 / 8192 / 32768 / 8192 / 8192 and 240
seconds each. Every request retains the 16384-byte ceiling and 16384 reserved input-token
bound. Actual usage and charges are recorded, not replaced by reservations. Fixture
initialization is separately labeled and excluded from generated-operation evidence.

Automatic terminal stops include empty initial aspiration, generation inability,
invalid/truncated output, stale output/state, invalid possibilities/assessment/revision,
controller refusal, provider or usage uncertainty, budget exhaustion, expiry, withdrawal
and final proposal completion. A STOP file pauses before dispatch/application; it does
not revoke or erase consumed reservations. Explicit withdrawal expires unused slots.
Lost dispatch/application acknowledgment remains uncertain and is never replayed blindly.

## Offline evidence

`target/workshop-offline-run/` is an explicitly authored **fake-provider** trace using
the real controller, not live generation, semantic validation or charges. It completed
five phases, one operation, no second apply intent, zero unresolved reservations. Request
sizes: 11889, 12000, 12582, 15157 and 12160 bytes. Real generated text can be longer;
the ceiling is enforced and a larger request must stop, not silently raise limits.

Fresh-process tests interrupt and resume after outcomes and applied stage markers at
all five stages, and after controller receipts for initial/revised relation and action.
Every dispatch boundary also tests unknown completion with no retry. Other cases cover
empty aspiration without state change, exact initialization and admission drift,
source/parent/authority preservation, phase reservations and terminal replay.

This verifies normal-driver mechanics and source limits. It does not validate a winning
aspiration, causal guidance benefit, truthful inference or social fulfillment. Independent
semantic review after a future bounded run must apply the unchanged preregistered rubric.

## Final checks

- New workshop suite: 4 tests passed, with 14 fresh-process resume variants plus
  five unknown-dispatch boundaries and drift/empty-result cases.
- Combined archive/synthetic/workshop cycle suite: 19 passed in 58.521 seconds.
- Relational/evidence suite: 4 passed; frozen workshop design preflight: 1 passed.
  Total distinct affected tests: **24 passed**. `git diff --check` passed.
- Independent final review found no remaining scoped blocker and verified prepared
  manifest hashes and offline trace. No protected tests or frozen scenario bytes changed.
- Manifest SHA256: `01a10f00da0d614b2b39e8251138972ff42030a981b1e6e29d4449e15e6949d7`;
  expiry `2026-10-23T19:11:46.983152+00:00`. Expiry is not a scheduled dispatch.
- No socket/process-list denial was retried. Earlier aggregate sandbox stub-bind errors
  remain separately reported; this is not a new whole-workspace pass claim.
