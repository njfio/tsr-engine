# M3 terminal smoke report

The approved two-task, four-arm run completed on 2026-10-16. All four arms exited cleanly and were rejected at the fixed 20-generator-call limit. Final oracle evaluation was never invoked. M3 acceptance remains unmet.

Execution began **16:27:24.738565 UTC** and finished **16:57:57.897158 UTC**. The model was `deepseek-v4.1-flash` through `api.cheaperinference.com`, running record then rules serially for each task, without automatic retries. The fresh scope-specific approval is retained in [the authorization receipt](m3-smoke-execution-authorization.json).

| Task | Mode | Calls | Process seconds | Settled USD | Outcome |
|---|---|---:|---:|---:|---|
| ProtocolError::to_wire | record | 20 | 510.537 | 0.014266 | rejected: call budget |
| ProtocolError::to_wire | rules | 20 | 480.643 | 0.007099 | rejected: call budget |
| Route::to_wire | record | 20 | 420.561 | 0.010979 | rejected: call budget |
| Route::to_wire | rules | 20 | 420.688 | 0.011189 | rejected: call budget |

Each arm proposed one patch and continued inspections/tests before its budget stop. No patch was accepted. The retained closure is `stop:budget:generator_calls` for every arm; every `oracle` is null. Both task pairs started from the same empty state root in independent stores. There are two observed both-rejected pairs and no discordant pairs. Original process-only and corrected evidence-consistent acceptance counts are identical: **0/2 record, 0/2 rules**. Descriptive Wilson 95% intervals are 0–65.8% per mode; these selected tasks share one repository lineage, so the intervals do not establish population performance, cross-lineage generalization, equivalence, or superiority. Older pilots are not pooled.

All **80 reservations settled**. Incremental provider-reported cost is **$0.043533**; historical settled cost was $0.165094; aggregate settled cost is **$0.208627**, with **$0 outstanding**. These are ledger receipts, not an account invoice. All 80 diagnostics indicate successful settled provider requests. Maximum broker elapsed time was 42.039 seconds; this run did not exercise the extended timeout at its boundary.

## Evidence and independent review

[Terminal audit](m3-smoke-terminal-audit.json) passed: all 98 exported-file SHA256 hashes, all 70 CAS BLAKE3 objects, all 72 frozen-input hashes, four SQLite integrity checks, event-ID-derived roots, exported/live ledger equality, preserved historical ledger prefix, summary accounting, and zero open reservations. Root recomputation checks event IDs in order; it does not independently validate canonical hashing of each event payload.

The evaluator remains frozen at source commit `109d4b0`, executable SHA256 `71f6599c4e4927f98f89f99b5b684f1fcc9e7a585430b3d5d02c3d5cfb001b3a`. Evaluator acceptance criteria and raw outcomes were not changed after observing results. The [terminal reviewer receipt](m3-smoke-terminal-review.json) independently confirms the outcome and accounting interpretation.

[Runtime check](m3-smoke-terminal-runtime.json) at 17:00:20.535781 UTC found no current-run processes or containers. The historical coordinator PID 29246 retained its exact identity and remained stopped. Free disk was 316.926 GiB. No deletion was performed. Final packaging found eight review-generated SQLite sidecars outside the original 98-file manifest; these were preserved under `target/m3-smoke-review-sidecars/`. The final export file set exactly matches the manifest, and all 98 hashes were rechecked unchanged.

## Separately corrected reporting gaps

Changes in `scripts/m3-smoke-evidence.py` now export shared base CAS and the freeze, require process/group/container quiescence before snapshotting, and separately report original process-only acceptance versus evidence-consistent acceptance. The original counting method could count a raw accepted result with contradictory oracle/closure evidence. A harmless regression demonstrates original count 1 versus corrected count 0 while retaining raw accepted count 1. The actual run counts remain 0 under both interpretations.

Changes in `scripts/m3-smoke-audit.py` require exact manifest/export file-set equality, bind summary results and hashes to exported results JSONL, and bind exported/live ledger plus original baseline prefix to summary costs. The checksum helper is `scripts/m3-blake3-checksum.rs`. Reporting changes were tested and independently reviewed separately from the frozen evaluator; [report-tools review](m3-smoke-report-tools-review.json) records that work.

The evaluator still has a latent final-oracle evidence gap: `protected_oracle()` does not persist raw final test execution receipts. If reached, its four-field verdict and derived closure cannot independently reconstruct full-suite exit, test counts, duration, stdout/stderr, or distinguish suite timeout from failure. **None of these four arms reached that oracle**, so this is not missing output from an executed oracle in this run. The retained budget stops support the reported rejections. Generic oracle-output caveats in the machine-readable exports should be read with this run-specific qualification.

## Validation actually completed

Before launch: 14 Python tests and 10 Rust broker tests passed; formatting, `clippy -p tsr-batch --all-targets -- -D warnings`, and the offline build passed. Earlier batch/CLI validation passed 25 Rust batch and 2 CLI tests, with one preexisting ignored test.

For the report changes: three exporter tests and three audit tests passed, with fail-first regression logs retained. The standalone BLAKE3 helper passed its known-empty-input check. Terminal integrity/accounting audit and independent terminal review passed. Test logs are retained under `docs/m3-validation/`; no claim is made that unavailable Lefthook hooks ran.

## M3 completion assessment

The broader corpus contains **151 verified unique admissions**: RTS 123 and AgentID 28, covering two repository lineages and 11 family labels. This is real-repository mutation admission evidence, not proof of representative real operational outcomes. Seven supplementary prepared candidates remain unlaunched.

Implementation and evidence tooling progressed, but M3 acceptance is **not met**. Remaining work includes representative cross-lineage paired outcome evidence, owner-provided sanitized operational selection/reviewed-tension evidence, and final-oracle receipt persistence before claiming independent receipt-level validation of executed final oracles. Synthetic empty-review/abstention fixtures are not real operational data.

All work is on `codex/m3-evidence-pipeline` in `/Users/n/Documents/Codex/2026-10-14/task/tsr-engine-m3`. Terminal evidence is under `docs/m3-smoke-execution/`, with [results](m3-smoke-result.json), [per-arm accounting](m3-smoke-final-accounting.json), and [export manifest](m3-smoke-export-sha256.json). No push, PR publication, merge, deployment, additional cohort, or further paid run was performed. There is no pending or uncertain charge and no denied action blocking this terminal report.

## Trace-only diagnosis and smallest justified next change

The retained event traces show an action-selection/termination loop, not a TSR policy denial. Every proposal was committed; raw results contain no denials. No arm proposed `stop`. A replacement was applied in each arm: patch commits are followed by inspections with a changed source digest, and the driver applies replacements before continuing. The exact sequences are retained in [trace diagnostic](m3-smoke-trace-diagnostic.json).

| Arm | Inspect / test / patch | Patch action | Post-patch evidence |
|---|---|---:|---|
| ProtocolError record | 10 / 9 / 1 | 3 | Eight visible test exits 0; nine inspections of identical changed bytes |
| ProtocolError rules | 15 / 4 / 1 | 4 | Three visible test exits 101; thirteen inspections of identical changed bytes |
| Route record | 17 / 2 / 1 | 5 | No post-patch test; fifteen consecutive inspections of identical changed bytes |
| Route rules | 16 / 3 / 1 | 6 | One visible test exit 101; thirteen consecutive inspections of identical changed bytes |

Every arm's initial visible test exited 101. Exit 101 alone cannot distinguish assertion failure from compile failure; those raw test streams are not reconstructable from these event receipts alone. Exit 0 in the first arm is a visible-test result, not hidden final-oracle acceptance.

The 20-call limit counts one broker/model completion per proposed action, including inspect and test actions. Observations are fed into the next request and are not independently charged against the call count. Thus the 20 proposed actions exactly account for each arm's 20 paid requests. Once the loop exhausts 20 iterations, the harness writes a budget stop and skips final oracle evaluation. Patching is not final submission: an explicit `stop` proposal ends generation normally.

Source inspection identifies a plausible harness contribution, not a proven causal explanation. Each provider request is a fresh system/user pair. The bounded user context retains the latest action, latest inspection, and latest test, not an accumulated action history. The latest test includes its ID, numeric exit, stream digests, and bounded excerpts, so feedback is not intentionally absent. However, the request exposes remaining tokens and deadline but **not remaining calls**, does not explicitly tag whether the test applies to the current edit, and always says inspections/tests may predate edits and to repeat them to refresh—even after current evidence was obtained. A successful replacement is summarized briefly and that action summary disappears on the next action. The broker says “Stop when finished,” but does not explain that `stop` submits the current workspace for independent final grading. Exact request bodies are not retained, so excerpt usefulness or model comprehension cannot be proven retrospectively.

These observations support a model/harness interaction failure to finish under budget. They do not show a safety-policy rejection or establish that TSR rules caused the failures: both modes repeat, and the cleanly passing visible-test arm also fails to stop.

The smallest justified next change is a bounded controller-owned progress summary: remaining action calls, current edit version, inspection/test freshness relative to that version, and explicit `stop` submission semantics. Replace the unconditional refresh warning with accurate freshness status. Validate this offline with a scripted broker sequence (inspect → failing test → replace → passing test → stop), checking that evidence remains current across repeated inspections and that the call counter/stop path are clear. Do not silently auto-accept a visible test pass, increase budgets, expose hidden oracle data, or reinterpret this cohort. This is a recommendation only; no evaluator change or new cohort was launched.
