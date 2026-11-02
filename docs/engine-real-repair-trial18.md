# Real Cargo repair with runtime review: Trial 18

Trial 18 stopped before a repair. It exercised the new separate runtime review on a real sealed Cargo failure, but did not complete an unattended repair. Trial 17 and all older evidence remain unchanged.

## Actual sequence

1. The existing evaluator-authored flawed seed ran in the networkless disposable Cargo project. This is an actual execution, not a simulator or model-generated repair.
2. Luna generated initial relational guidance. A separate Luna invocation, with its own source-bound context, accepted it as provisional guidance. No operator supplied the intermediate approval. Same-model errors may correlate.
3. The next generator response was rejected as `B9.1 experiment: unsupported_experiment_fields`. Its experiment object had seven fields, while the published contract permits only `harness`, `cases`, and `mapping`. The bounded quarantine preserves structural evidence without admitting the response or retaining arbitrary rejected prose.
4. No generated experiment ran, no patch was proposed or applied, no post-patch verification occurred, and no final acceptance was recorded. This attempt is terminal; unused allowances are not recycled into it.

The visible inner contract explicitly forbids additional fields. The response violated it. The enclosing advertised schema is less detailed than that inner contract, but is not contradictory. This result does not justify silently stripping fields or weakening validation. The precise rejected prose is not retained and is not reconstructed.

## Authority and accounting

This used the same frozen B9.1 non-private Rust fixture and existing campaign authority, not a new recipient, data class or execution permission. Source edits remained confined to disposable `src/lib.rs`; original tests were immutable. The run was admitted for at most six generator calls plus six separate reviewer calls and four execution groups, with final independent FileReviewer retained. No new credentials, downloads, security changes, publication or production effects occurred.

Actual use: **two generator calls plus one reviewer call, one real execution group, 14,877 tokens, $0.003149**. Campaign consumption increased from 78 calls / 22 groups / $0.051048 to **81 calls / 23 groups / $0.054197**. There are no unsettled campaign requests. Remaining campaign capacity is 19 calls, 17 groups and $4.945803; this is not a new allowance. Older Garden Trial 1 unknown accounting remains separate.

## Reproducibility

Activation source commit: `3c78fb4`; adapter source: `1d6f0ad`.

- Trial: `target/b91-integrated-trial18`
- Generator admission: `target/b91-integrated-trial18.admission.json`, SHA `484bbd4753603c0e07c64e12f0bcd4c4aa0b53454d3d561b7efdd1598757e6f0`
- Reviewer admission: `target/b91-integrated-trial18.runtime-review.json`, SHA `10bc91d4fe7c1c77f953faa8e9e87d68dc44cd0b487b43cbfd7ac52b331239bd`
- Campaign upgrade: `target/tsr-integrated-campaign-1/runtime-review-upgrade.json`, SHA `25baa3c6727a51049e76e5a1e2078b806c53143ce35f70154eccd9b8271bd223`
- Rejection: `quarantine/4aa140490e82b03c96cd11aeace8e9b14694c4ce99d593b72af520636ce85c5f.json`
- Separate review: `runtime-project-reviews/ba551552ba0f12bf5fbfb33b68fbcfadafa4eb411e7e9ae2a8bdbb1ce45ec6ec.outcome.json`

Read-only inspection (never invokes provider or worker):

```sh
python3 scripts/engine-b91-driver.py inspect --admission target/b91-integrated-trial18.admission.json
```

Offline activation/accounting verification: 22 tests passed, one optional native fixture test skipped. The prior real-native-controller adapter fixture run passed all 11 tests using a fake provider and cfg(test) worker; those fixtures are not live repair evidence. Independent review checked the concrete admissions, source pins, budget headroom and final-review boundary. Missing local lefthook remains disclosed; it was not installed. No full-workspace pass is claimed for this change.

## Meaning and remaining gap

The runtime review boundary now operates on actual project evidence without manual intermediate review files. Only its initial-guidance gate was reached live. This does not establish reliable automatic experiment/patch review, autonomous planning over arbitrary programs, or full semantic fulfillment. B9.1 still has fixed phase order, and final formal acceptance remains independent/operator-assisted.

Earlier Trial 11 and the linked Trial 15 recovery did execute model-generated repairs and pass real protected Cargo checks. This run does not erase that evidence, but it also does not turn those assisted runs into unattended autonomy. The present blocker is reliable generator contract compliance before subsequent review and execution, not lack of a real executor. No new run was launched to hide this failure.

Independent result review: [engine-trial18-result.md](../specs/lanes/reviews/engine-trial18-result.md). `target/b91-integrated-trial18/accounting-summary.json` and `readonly-inspection-audit.json` record exact metering, protected-file equality and no-effect inspection: zero unresolved effects, unchanged trial inventory and campaign ledger.
