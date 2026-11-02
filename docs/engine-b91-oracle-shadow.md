# B9.1 source-role shadow comparison

Run on `2590dfc`, 2026-10-25, after explicit approval for four synthetic-only
CheaperInference/deepseek-v4.1-flash requests. This is not Trial 11. No generated
code was compiled, executed, adopted or patched. Trials 1–10 remain unchanged.

## Frozen comparison and boundaries

The preregistered design in `engine-b91-oracle-fidelity-audit.md` was frozen before
dispatch. Baseline → labelled, then labelled → baseline; temperature 0, same model,
8192 completion tokens and 240 seconds per request. The final reviewed manifest is
`target/b91-oracle-shadow/manifest.json`, SHA-256
`43d10bb1141f4279d12947c5ec0b126ba04bebd02207bfb4665dfb6a6191e7df`.
Independent pre-dispatch review required pinning the native validator binary and
stopping on measured token overruns; both were fixed before any call. Earlier draft
manifest/runner remain alongside the final versions.

The baseline uses the normal allowlisted experiment projection from Trial 10's
current read-only state. Its semantic inputs retain the failed seed, both parents,
source bytes and admitted initial aspiration. No rejected experiment, review note,
protected test or evaluator-authored successful assertion is sent. This is a fresh
projection with a current state hash, **not** recovery of the historical wire request
or restoration of an earlier ledger root. No supplemental return is present.

The sole payload difference is an added `source.role` string:

> Current implementation under evaluation, not acceptance criteria.

Requirement text, source, aspiration, observations, instructions, schemas, bounds and
sampling remain identical. Baseline is 9844 bytes and treatment 9923 bytes under the
existing JSON encoding size check. Exact synthetic payloads and hashes are retained
locally. The two occurrences of each condition have identical request-body hashes.

The rubric was fixed before calls: expected values consistent with requirements,
nonvacuous behavioral assertions, actual second-aim coverage, structural validity
separately, and honest finite limits/uncertainty. No preferred output was supplied.

## Complete results

| Order / label | Condition | Structural result | Independent semantic judgment | Settled USD |
|---|---|---|---|---:|
| 1 / Sable | Baseline | Native deferred | Requirement-aligned finite test; both aims | 0.000480 |
| 2 / Birch | Labelled | Malformed JSON content | Unknown; no typed candidate retained | 0.000484 |
| 3 / Amber | Labelled | Native deferred | Requirement-aligned finite test; both aims | 0.002383 |
| 4 / Cobalt | Baseline | Native deferred | Requirement-aligned finite test; both aims | 0.000473 |

All three retained candidates assert that exactly empty input succeeds and that
`parse("1,,2")` returns an error. They also exercise ordinary signed parsing,
whitespace rejection and positive overflow. Sable adds a trailing empty field;
Amber adds leading/trailing empty fields, leading whitespace and malformed letters.
Cobalt has narrower finite coverage. These are genuine generated assertions, not
substituted evaluator answers. Their expected rejection of the current faulty filter
is a **source-derived prediction**, not an observed execution result.

None is inverted, vacuous, or limited to the wrong aim. None proves the universal
nonempty behavior requirement. The blind reviewer noted omissions including signed
integer boundaries, negative overflow, explicit plus signs, leading zeros,
separator-only strings and exact error-string preservation. Assertions grouped in a
single test can abort early if later executed; no assertion execution is claimed here.

Birch's provider termination was `stop`, with settled usage and a content JSON parse
error. It was not a timeout, truncated completion, or unknown dispatch. Raw content
and reasoning were not retained under repository policy; the exact syntax mistake
and semantic quality cannot be reconstructed. Hash, structural diagnostic, provider
request identity and metering remain. Invalidity did not trigger a replacement call.

The semantic reviewer inspected only the three typed candidate artifacts and fixed
synthetic requirements/source, without condition assignments or order. It reported
no unblinding. The pre-dispatch reviewer necessarily saw assignments and was separate.
Both reviews involved no provider calls or target execution. Native deferral is only
structural eligibility for review, not semantic acceptance or execution permission.

## Accounting, integrity and interpretation

Exactly **4 requests, 0 executions, 0 adoptions**, **$0.003820 settled**, no unknown
charges or unsettled dispatches. Usage: 10017 prompt + 5988 completion = 16005 tokens;
4174 reported reasoning tokens are included in completion. All calls remained within
per-request bounds. No allowance remains. No persistent runtime was started.

Despite identical labelled request bodies, the provider reported 2504 versus 2529
prompt tokens and 0 versus 4174 reasoning tokens. The cause is unknown; no hidden
client-side setting or input change is evidenced. Temperature zero does not establish
deterministic provider behavior. Receipts are retained rather than normalized away.

Pair 1 has a correct baseline and an unassessable malformed treatment; pair 2 has
correct outputs from both conditions, with broader finite treatment coverage. This
provides **no reliable evidence that the source-role label improves oracle fidelity**.
Both baseline samples avoided Trial 10's inversion. Two pairs cannot identify reliable
causation, broad model competence or general autonomy. No production prompt change
is justified by this comparison, and malformed output remains a reliability limit.

Do not continue label sampling merely to obtain a favorable result. A separately
approved full B9.1 run would still need independently reviewed requirement-aligned
experiments, actual return-informed patching and unchanged experiment verification;
this comparison grants none of that execution authority. Trial 11 remains unapproved.

## Verification and local evidence

- `target/b91-oracle-shadow/events.jsonl`: durable intents, all outcomes, terminal cost.
- `*.candidate.json`: schema/native-validated typed shadow artifacts only; no adoption.
- `baseline.json`, `labelled.json`, `request.json`, `manifest.json`, reviewed `run.py`.
- `python3 target/b91-oracle-shadow/replay.py`: standalone read-only accounting and
  integrity replay, no provider import or dispatch. Two fresh processes produced
  identical results (`replay1.json`, `replay2.json`).
- All pinned prior Trial 10 journal/review/state hashes unchanged; earlier historical
  evidence remains untouched. Historical Garden Trial 1 unknown charges are separate
  and are not cleared by this comparison's zero unknown count.
- 14 focused oracle/review/contract tests passed in 12.825s:
  `PYTHONPATH=scripts python3 -m unittest test_engine_b91_oracle_review test_engine_b91_review_contract test_engine_b91_contract_audit`.
  Log: `target/b91-oracle-shadow/offline-checks.log`.

Production code, protected files and frozen acceptance criteria are unchanged. No new
full-workspace test pass is claimed. No credentials, raw provider response bodies or
reasoning text are included in this report.
