# Single criterion reassessment after an internal contradiction

## Scope and authority

On 2026-10-31 at 17:17:10 UTC, the user approved the specific proposal to allow one fresh assessment of identical evidence after an internally contradictory settled criterion response. This change implements that behavior offline. It does not reopen either stopped trial or authorize another paid evaluation. The earlier policy boundary remains documented in `criterion-verdict-consistency-recovery.md` as historical context.

A prospective criterion profile must explicitly contain `consistency_reassessment: "single-fresh@v1"`. Existing profiles remain disabled. The attempt consumes one slot of the admission's existing shared correction allowance; it does not enlarge that allowance or bypass call accounting, reservations, Stop or source validation.

## Mechanism

`engine-bounded-correction.py` recognizes only the typed criterion consistency diagnostic, independently checks the contradictory verdict/findings combination, and requires a settled response with known charge and reported usage. The failed response and charge remain retained. The new request preserves the original authenticated review messages, source, criteria and evidence, adding only exact validator feedback asking for a fresh independent assessment. Nothing chooses which field was intended or normalizes unknown into met.

A durable, admission-bound event records the stage, original request, original wire digest, current native source and feedback before any fresh dispatch. There may be only one consistency event for that criterion stage. The fresh response cannot enter another response-correction or provider-retry loop. Consistent acceptance, rejection, uncertainty and abstention are terminal valid outcomes for this assessment; a second contradiction or malformed response stops. No action execution is repeated.

`engine-criterion-return.py` and the ordinary browser-session entry recover only an interrupted local copy of this exact fsynced ticket. Recovery requires the same capture, native source, settled rejected request and wire (including an authenticated preceding structural correction if present). Checkpoint Stop is checked before recovery. Pending dispatch, execution, other effects, changed evidence/source, unavailable allowance, terminal output and ambiguous tickets remain blocked. Retained successful responses replay without dispatch. Correction-ticket comparisons use their existing Unicode hash convention; no historical digest algorithm changes.

## Verification

Command:

```sh
PYTHONPATH=scripts python3 -m unittest scripts.test_engine_criterion_reassessment scripts.test_engine_criterion_consistency scripts.test_engine_criterion_return.CriterionShape scripts.test_engine_pursuit_reflection scripts.test_engine_bounded_correction scripts.test_engine_browser_session scripts.test_engine_browser_transient -v
```

**52 passed; 5 skipped; 57 discovered.** Log: `target/criterion-reassessment-final-tests.log`.

The five existing browser-session checks require an explicitly supplied frozen ledger input and were not enabled. No historical ledger was used as test input or mutated. New authored regressions exercise the actual checked/stage/correction/bound-output code, ordinary session entry and isolated campaign accounting, with an authored transport and fixed native view. They cover successful correction, valid rejection/uncertainty/abstention, second contradiction, malformed second response, preceding structural correction, Unicode restart/retained validation, each ticket interruption boundary, Stop/resume, changed evidence/source, unknown dispatch and settlement, duplicate replay, exhausted allowance, absent policy, no provider retry and no execution callback. Two calls retain $0.002 fixture charges; an earlier format correction retains three calls/$0.003. These are test charges, not real provider spending.

Python compilation and `git diff --check` passed. All five hashes in the frozen `relational-observation-live-result.json` evidence map still match. No provider calls, browser executions, historical acceptance changes or live admission changes occurred. Native/Rust semantics were not modified; no new full workspace or real-browser pass is claimed. Repository hooks are unavailable in this environment.

## Limits

This is recovery from contradictory assessment output, not evidence of new experiential learning, desire culmination or autonomous live success. The assessor may still contradict itself twice, decline to assess, reject, or report uncertainty. A future live test needs a new prospectively reviewed source/admission binding with the explicit policy and an available shared correction slot. Closed trials and their original results remain closed.

Independent final source review by `contrast_bridge_review` accepted the patch, conditional on the aggregate test result above, which passed. Review found and prompted fixes for session-entry ticket recovery, preceding structural-correction binding and Unicode correction hashes; all three have regression coverage. The reviewer performed no live effects.

## Offline verification closure (follow-up)

The five skips were caused solely by `TSR_BROWSER_SESSION_LEDGER` being unset. `Session` tests deliberately require the exact legacy pre-session baseline (262 calls, 72 groups, $0.225523, ceilings 272/74), so a new empty authored ledger cannot satisfy them. They cover: preservation of historical spend plus the then-authorized $10 headroom; cap/source-prefix/duplicate-grant rejection; unresolved reservation blocking and exclusion of old trials; exact session trial/admission binding; and charges exceeding reservations. These are legacy campaign-fold obligations, not tests of the new durable reassessment ticket itself.

A separate 848-row prefix was copied read-only from the existing campaign, immediately before its first `native-browser-session@v1` grant, into `target/criterion-reassessment-legacy-prefix.jsonl`. Its terminal hash is `061d01ee2247cd8bbebc32a3e9e8e98df6040cd889dd8ea6d6c79a2fc2aaf69b`. Each copied row still equals the original prefix. The tests append authored events only to Python lists and call `Ledger.fold`; they neither write a grant nor reopen a trial. All five previously skipped checks passed. This is historical read-only input with authored prospective events, not a fabricated historical ledger or a new live admission.

With that environment variable supplied, the complete command above now runs **58 tests, all passed, zero skipped**. Log: `target/criterion-reassessment-complete-tests.log`. The additional test prepares/serializes the explicit policy through the real browser admission builder, then uses the ordinary session runner, counted review stage, validator, correction journal, bound-output validation and isolated fixture campaign. It completes opening/storm/reopening with four fixture calls (first contradiction plus fresh acceptance, then valid rejection and uncertainty), one shared reassessment slot and $0.004 fixture charges. No operator stage handoff or execution callback occurs. Replay adds no calls. Serialized assessment requests are checked for exact current-condition context and absence of future condition IDs. Initial fixture mistakes reused a review identity, then changed it without updating its schema; existing binding checks correctly rejected these, and the fixture was corrected. Production validation was unchanged.

The reusable `prepared_three_condition_fixture` in the test builds the same opt-in shape for a future packet. A persisted inspection-only packet is at `target/criterion-reassessment-prospective/packet.json`, SHA-256 `470106f117d78ee12b33ea386dfef294545ac4c29b7159748727d0a103be55fa`. It explicitly identifies its native policy/authority/origin placeholders and is **unadmitted and not live-launch authority**. The old frozen launch remains unchanged and disabled for this policy; it must never be reused as a new launch. A future preparation supplies real fresh native policy/source identities and sets `criterion_return_profile.consistency_reassessment` to `single-fresh@v1`, with a shared correction slot available, before independent binding review/admission. The ordinary builder preserves and validates the field, and the running session needs no operator handoff to invoke it.

Independent review accepted this as offline routing/accounting coverage. The authored Cycle returns retention and its native source is a fixed mock; no browser action, real native condition publication or new live autonomy was tested. The current-condition assertion verifies the fixture's serialized delivery, not every possible production publication. Those limits do not leave the one-attempt wiring/accounting/restart question blocked, but require a freshly authorized and bound real evaluation before any live-success claim. No new production code was needed in this follow-up. The five frozen live evidence hashes remain unchanged. No paid calls or live admissions occurred.
