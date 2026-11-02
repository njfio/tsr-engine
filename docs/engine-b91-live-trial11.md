# B9.1 Trial 11 — Luna repairs and reverifies; final reflection blocked locally

Fresh authorized trial, 2026-10-25, compatibility commit `681e110` (from `b0f0f63`).
Exact model `gpt-6-luna` through CheaperInference, using its verified
`max_completion_tokens: 8192` parameter without unsupported temperature settings.
Current public rates were $0.065 input / $0.325 output per million tokens.
The six-call/four-execution admission hash was
`40c3a3e05934b48bc1dff3e01845263f10b244985268631c7a5ef00fbbe83991`.
Independent readiness review checked the fresh one-hour deadline, synthetic scope,
updated active CA bundle, pinned installed image and exact implementation/binary.
No downloads, security changes, protected or original-target edits occurred.

## Actual learning and verification sequence

| Stage | Actual result |
|---|---|
| Evaluator-authored seed | Empty-input check passed; preservation check failed. Both desires remained open. |
| Generated initial relational guidance | Both parent meanings retained; independently reviewed as joint-contract guidance, not proof of richer emergence. |
| Generated second-aim experiment | Correctly tested signed parsing and rejection of malformed, empty-field, trailing-field, whitespace and overflow inputs. Declared case IDs matched actual tests. |
| Experiment execution | Empty-input case passed; nonempty preservation case failed, providing a real counterexample. Case-level outcomes do not identify every executed assertion. |
| Generated return-informed revision | Distinguished exactly empty whole input from empty fields within nonempty input. Cited recorded outcomes; labeled filter behavior a source inference and assertion/value details unknown. |
| Generated boundary patch | Special-cased exactly empty input; restored the original split/parse/collect expression for every nonempty input. Independently reviewed before execution. |
| Original checks on patch | Passed. Receipt `0a47acfbdaf4da8fd2677e4cdc1e4acf31087103d731d7b223c07040402fcaf9`. |
| Unchanged generated experiment on patch | Passed after independent source-bound readmission. Receipt `322471c35c939a13cf623c76b544c60777691fbb595401619c18930582cc8e18`. |
| Post-verification revision | Not run: assembled request exceeded the local payload bound before dispatch. |
| Final proposal and terminal acceptance | Not run. Native acceptance remains false, original desires open. |

The patch and experiment were generated, not supplied by the evaluator. The experiment's
assertions, mappings and cases were unchanged between the failing and passing executions.
Independent source review supports preservation for nonempty input because that branch
is the original expression, beyond the finite examples. This establishes a bounded repair
and reverification, **not a complete B9.1 reflective cycle**. Revision-to-patch continuity
is visible, but no ablation establishes that relational guidance caused the improvement.

The native seed p=0.1 forecast is explicitly evaluator-authored, source/action/return-bound,
not learned calibration. Dimensional Contrast remains available after failure. The
pre-acceptance requirement belief estimates remain 0.5 with zero reviewed observations;
we do not claim a completed belief update or final desire fulfillment. No maturity,
training eligibility, or measured human review minutes are invented.

## Accounting and reproducibility

- **4 calls; 4 execution groups; $0.002317 settled**, no unknown Trial11 usage/charges.
- **11,891 prompt + 3,212 completion = 15,103 tokens**. All calls within 8192 completion.
- **40,806 aggregate tool ms**, 438 patch bytes including evaluator seed and generated patch.
- Two unused calls expired via terminal STOP. No fifth request was dispatched or reserved.
- Historical settled total is **$0.724798**, plus the older Garden Trial1 unknown charge/reservation unchanged.
- Every stage used the normal driver. Independent reviews were operator-assisted at
  five gates (initial guidance, experiment, revised guidance, patch, readmission), with
  fresh-process driver resumes; no manual candidate editing or selection substitution.
- Two fresh read-only inspections match: 107 events, 53 completed effects, no unresolved
  effects, dispatch disabled. Driver/provider journals and review records remained
  byte-identical. All 40 captured prior-trial hashes match.
- No persistent model service was started. All six live driver invocations terminated.

Evidence is under `target/b91-live-trial11/`: native stores, frozen source, provider/driver
journals, intermediate/terminal requests and statuses, STOP, terminal-summary.json,
replay-check.json and offline-payload-repair-check.json. Sibling admission, stage logs,
prior hashes and `target/b91-live-trial11-reviews/` preserve review provenance. Raw provider
responses/reasoning and credentials are not retained. Typed proposals are historical evidence.

Read-only replay (no paid or execution opt-ins):

```sh
python3 scripts/engine-b91-driver.py inspect --admission target/b91-live-trial11.admission.json
cat target/b91-live-trial11/terminal-summary.json
```

## Demonstrated integration defect and offline repair

The fifth-stage wire body was **16,463 bytes**, exceeding the existing 16,384-byte limit
by 79 bytes. This was a known local refusal before reservation/dispatch, not a provider
failure, unclear billing, or semantic rejection. The actual transport encoding confirmed
the size; it was not double-counting. There was one current supplemental return.

The system message repeated field inventories already supplied in the exact per-stage
schemas. Removing those duplicate lists reduces this same body's size to **16,277 bytes**.
The user context, original meanings, all evidence and source bindings remain byte-for-byte
identical. Schemas, validators and size/resource limits are unchanged. The driver now
reports this specific pre-dispatch refusal descriptively rather than a traceback.
This exact repaired payload has only 107 bytes of headroom; a longer subsequent generated
revision can still exceed the limit. This is not a general launch-readiness guarantee.
Larger contexts can still legitimately exceed the limit; nothing silently truncates
history, expands authority or retries. This repair does not revive Trial11's allowance.

Independent review found no semantic/authority instruction lost. Exact-boundary/UTF-8,
no-truncation and oversized-context tests verify zero dispatch/storage/slot consumption.
Full native fixture tests exercise all six stages with Luna and unchanged DeepSeek;
fixtures are mechanics checks, not live semantic results. Verification totals follow below.

The smallest remaining live validation is a separately admitted, source-bound continuation
of the preserved post-verification state for revision and final proposal (no further project
execution), if an explicit continuation admission can be safely represented; otherwise a
fresh full trial. No such live continuation is authorized or run here. Do not rerun the
expired admission or claim this run completed its final reflection.

## Offline verification

- `python3 -m unittest discover -s scripts -p 'test_*.py'`: **630 tests,
  612 passed, 18 skipped, zero failures/errors**, 203.901s; exit 0.
  `target/trial11-full-python.log`. Existing opt-in target/live tests remain disabled;
  native fixture tests are run separately below. Incidental ResourceWarnings are retained.
- `PYTHONPATH=scripts python3 -m unittest test_engine_b91_contract_audit -v`:
  **8 passed**, 8.185s; `target/trial11-payload-checks.log`.
- The exact retained terminal context was reassembled read-only with the repair:
  16,277 bytes, identical user context and non-message settings, no dispatch;
  both journals unchanged (`offline-payload-repair-check.json`).
- Independent implementation review: `readiness_review`; independent terminal
  semantic/accounting/documentation review: `oracle_blind_review`. Neither review
  equates the bounded repair with full B9.1 completion.
- `TSR_B91_TEST_WORKER_BIN="$PWD/target/debug/deps/tsr_cli-62904bb7a7cdafee" PYTHONPATH=scripts python3 -m unittest test_engine_b91_driver test_engine_b91_contract_audit -v`:
  **18 passed**, 581.492s; exit 0 (`target/trial11-final-native-checks.log`). Includes
  complete six-stage native fixture sequences for both models, interruption/restart,
  lineage, review withdrawal, known failure and unknown accounting controls. No real
  target commands or model requests are executed by this fixture worker.
- `git diff --check` passed. No new Rust source change or full Rust workspace sweep
  is claimed. Both offline suite processes reached exit 0; no owned test session remains.
