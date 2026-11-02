# Independent fixed12 terminal audit

The run halted; it did not complete all 24 assignments. Twenty-two decisions cover 11 complete pairs. Arm 22 (Rules) has a controller-error process receipt, exit -15, after the live 200 GiB disk guard and a recorded cleanup error naming an owned container. Arm 23 (record) has no launch intent. Both remain visible as missing final outcomes, not oracle failures.

## Verified outcomes and accounting

The pinned receipt checker independently reread the final oracle receipts: record arms 0 and 5 are verified successes; record arm 2 is rejected despite passing its named tests because the full suite regressed. No Rules arm has a verified success. The preregistered operational yields are record 2/12 and Rules 0/12. Missing-outcome bounds are record 2/12–3/12 and Rules 0/12–1/12. Do not perform the preregistered complete-cohort McNemar calculation or claim superiority from this halted, correlated sample.

All 22 per-arm export manifests, decision bindings, SQLite integrity checks, diagnostic/reservation IDs and settlement totals match. The ledger remains identical after the last recorded decision: arm 22 incurred no additional provider charges. There are 106 new settled requests costing $0.304611, cumulative $0.544194, and zero outstanding reservations. The original $0.239583 baseline prefix is preserved. All 5,362 frozen file pins still match freeze 7443c9ee69b4f5d7f8a1276826bd59f8915c083d5b42355710f5f4c6fff75e18.

## Runtime and limitations

Escalated read-only process/container checks found no experiment workers or running M3 containers; the exact container named in the historical cleanup error was absent. The historical error is retained and is not rewritten as successful cleanup. Four unrelated containers were untouched. The independent filesystem measurement was approximately 199.949 GiB free, below the live floor: no claim that the full guard passed is made.

This audit re-read evidence; it did not rerun candidates, inference, or tests. No retries, deletion, harness changes, push, or PR occurred. The constructed-mutation sample uses two lineages, includes disclosed prior exposure, and has incomplete final-pair outcomes. These results alone do not establish complete operational M3 acceptance.

Detailed per-assignment records, receipt verification results, accounting and oracle run metadata are in `m3-fixed12-independent-outcome-audit.json`.
