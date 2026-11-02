# Existing evidence and calibrated support

Run the read-only report from the repository root:

```sh
python3 scripts/engine-evidence-inventory.py
```

The saved output is `docs/engine-evidence-inventory.json`. It hashes explicit input
files, cross-checks summaries against individual admission receipts, validates the
strict admission protocol, deduplicates task/function identities and reconciles
Fixed12 aggregate counters against independently audited assignments. It does not
rerun candidate code, authenticate a reviewer merely from a string, or turn oracle
success into an operator review. Six fixture tests check disagreement rejection,
duplicate/retry handling, missing protocol, path bounds and read-only behavior.

## What exists

- **151 qualified tasks**, from 165 admission reports: 123 RTS and 28 AgentID,
  across 11 mutation/package families. All 151 task identities and lineage/function
  groups are distinct in these manifests. Qualification uses baseline green,
  named red, full red, exact reversal and full green evidence.
- Fixed12 records **24 assigned, 23 launched, 22 classified arms and 11 complete
  pairs**. The independent audit records three completed final oracles: two accepted
  and one rejected. Interrupted/unlaunched work is not converted into failure labels.
- The Fixed12 sampling population was 149 complete-run qualifications, not every
  one of the 151 qualifications. Prior exposure and correlated arms prevent treating
  this as a fresh independent validation cohort. Frozen/export manifests and the
  independent outcome audit preserve provenance.
- The separately documented M4 cohort of 151 examples is synthetic. Its equal
  count does not connect it to these 151 qualified mutation tasks.

## What is not established

The eligible count for `task_accepted_in_budget@v1` is **unknown**, not zero and not
151. The selected artifacts do not establish an explicit mapping joining episode
events, closure/budget evidence, independent operator labels, review times and
task/family/lineage/clone metadata. PRD B7 also specifies task-close plus seven
days for the common event; the Fixed12 terminal report is dated October 18, only
four days before this October 22 inventory. No discovered labels JSONL file establishes that
mapping. Test qualification and final-oracle completion are valuable evidence, but
neither alone establishes the common accepted-in-budget event or its maturity.

`tsr train dataset` now requires an explicit `--as-of` cutoff and writes a
`task-close-plus-seven-days@v1` maturity record with the original inputs. Fitting
reconstructs eligibility and rows before counting support. Legacy datasets remain
readable but cannot be fitted without rebuilding from original evidence. See
[the policy and verification](engine-maturity.md). This implements the time gate;
it does not turn qualification tasks or final-oracle results into independent
reviewed episodes, authenticate operator assertions, or establish calibration.

To use qualifying historical episodes, supply the missing explicit mappings and
review provenance without fabricating labels, preserve disputed/censored outcomes,
and exclude previously exposed evaluation examples from a fresh holdout. Then use
the existing dataset command with time/repository splits and clone groups. Fitting
requires at least 150 mature reviewed distinct tasks on fit plus calibrate and at least 30
per family, followed by validation; merely reaching the counts is insufficient.

No campaign, paid inference, external transmission or corpus mutation was performed
for this reconciliation. Bounded decision dynamics and local fallback can continue
without claiming that these statistical calibration gates are met.
