# Independent held-out suite and campaign extension review

Reviewer: independent agent `/root/suite_review`, 2026-10-26. This review inspected source and contracts; it did not run a target program or make provider calls.

## Scenario/oracle assessment

The new tasks in `scripts/engine-repair-suite.py` are evaluator-authored non-private programs, not user repositories or model-generated tasks. They preserve the B9.1 experiment-before-repair sequence. Both goals are simultaneously satisfiable; an authored seed violates preservation, rather than demonstrating that the desires conflict logically.

- **Singleton clamp:** the baseline rejects equal or reversed bounds and otherwise uses overflow-free `max`/`min`. The requested extension permits equal bounds. The authored seed removes validation entirely: singleton tests pass, but reversed bounds return a value instead of the specified error. The protected oracle includes equal endpoints, reversed bounds, interior/exterior values, and signed integer extremes. These are substantive finite checks, with explicit expected values rather than a digest derived from the implementation.
- **Unicode prefix:** the baseline rejects zero or counts beyond Unicode scalar length, then takes scalar values. The requested extension permits zero. The authored seed uses UTF-8 byte length for bounds: zero tests pass, but multibyte strings with excessive scalar counts wrongly succeed. The oracle includes ASCII, multibyte characters, combining scalars, empty strings and oversized counts. The requirement explicitly concerns scalar values, not grapheme clusters; expected results are consistent with that meaning.

The frozen oracles therefore distinguish the intended seed failures from valid behavior by inspection. This is **not an executed seed/oracle result**. Actual native receipts must establish execution results. Finite tests alone do not prove all-input correctness; final independent source review remains necessary. No reference repair appears in the scenario artifacts or model export. Protected test bodies and oracle commentary are excluded from provider context by the existing export boundary, subject to the integration tests below.

The two tasks have different APIs and error mechanisms, but both retain the same boundary-extension/preservation pattern. Report this as limited transfer across two fresh domains, not broad repair generalization. The original parser must be reported separately as a regression. The preregistered order and success/failure/assistance/correction/resource metrics are appropriate: failed and unknown attempts remain in the denominator, no post-result task replacement, and no semantic winner is prescribed. Runtime reviews use the same model in separate contexts, so correlated error remains possible.

## Campaign accounting assessment

The new exact `b91-evaluation-extension@v1` grant requires the prior effective 140-call/40-group limits, grants only 200 calls/60 groups, retains the $5 ceiling and $0.05 per-call conservative reservation, and binds the approved suite hash and source class. The original manifest and earlier call grant remain unchanged. Ledger folding enforces limits at each historical event; a later grant cannot cover a previously excessive execution. Unknown outcomes remain reserved and block further effects. Suite enrollment must match an already recorded grant; changed or omitted suite binding on repeated enrollment fails.

The planned three-run envelope is 42 calls/12 groups. Against the verified starting counts 133/40, it fits the approved 200/60 totals and leaves 25 calls/eight groups, not a renewed campaign. `preflight_run` explicitly reports a nonexclusive readiness check, not a durable whole-run reservation; safe operation relies on one sequential campaign driver and per-effect locked reservations. Do not describe this as exclusive reservation.

Independent command run: `PYTHONPATH=scripts python3 -m unittest test_engine_b91_campaign_extension -q` — **15 tests passed in 6.470 s**. These are offline ledger tests, with no provider or target execution. They cover temporal cap enforcement, prior grant/scope requirements, duplicate/rehashed grants, suite enrollment and unknown reservation preservation. I found no accounting blocker in this diff.

## Adapter review status

Initial adapter inspection confirms explicit per-origin/per-driver scenario selection rather than a global mutable fixture override. New v5 admission binds scenario ID, scenario digest, suite digest and implementation hashes. Legacy admission shapes remain separate. New configuration/origin checks bind exact editable and protected files; original desires are selected consistently for proposal generation and runtime review. Two minor findings were corrected: new tasks now use a synthetic Rust-program system label while the legacy parser instruction remains unchanged; malformed v2 origins reject with a deliberate validation error before scenario resolution. Independent command `PYTHONPATH=scripts python3 -m unittest test_engine_repair_suite_adapter -q` passed all five current tests in 3.370 s, covering native initialization, source/requirement identity, restart without effects, admission/origin tampering and legacy isolation. No provider or worker callback ran. Subsequent complete native fixture coverage is recorded below; initialization alone was not treated as proving that path.

## Preregistration and readiness boundaries

Reviewed `docs/engine-heldout-repair-plan.md`: order, denominators, frozen criteria, shared protocol corrections, independent final assessment and limited-generalization caveats agree with the machine-readable suite. The first counted execution in each run provides actual seed validation; no unmetered target prevalidation is authorized by this review. The two new task programs and fixed tests were judged statically, and are not described as dynamically validated. Requested documentation clarification: call 42/12 the planned/preflighted envelope, not an exclusive reservation. The reported current official quote ($0.075958 input/$0.379791 output per million tokens) remains subject to the existing fresh quote and conservative reservation checks; this review did not retrieve prices or establish a provider-enforced spending cap.

The added `SuiteNativeSequence` regression was also inspected: both tasks use actual native controller serialization with an explicitly marked `cfg(test)` worker and authored provider outcomes. It checks all five consumed generation stages, exact fresh requirements in runtime-review evidence, withheld protected test/oracle text, and restart with no calls. The authored fake patch deliberately restores the baseline rather than supplying a hidden correct solution; its passing fake receipts are protocol evidence only. The Rust change is confined to the `#[cfg(test)]` worker and adds the new primary case name to authored fixture outcomes. Production sealed execution and acceptance tests are unchanged. A direct-test-entry ordering issue was reported so direct script execution cannot omit the new test class.


## Final disposition and exact artifact verification

The adapter author supplied the terminal result of session99878: **six tests passed in125.268s**, including both complete native sequences, all five consumed generation stages, three runtime relational-review projections per task, and restart. I inspected that test implementation; its outcomes are explicitly authored fixtures, never target execution or live competence evidence. The direct-entry guard is now after the final test class. The author also reports38 affected checks passing. No duplicate full sequence was started by this reviewer.

Independent read-only artifact verification passed for:

- `target/heldout-evaluation/campaign-upgrade.json`: canonical hash `1fb4695516085839e0c1a8a71e4a3f5b73f0a120e8bc26da1b6ff9c302e6d6f3`.
- `target/heldout-evaluation/evaluation-grant.json`: canonical hash `91987e6588b31051e7426ace0f68cf26bb8621005d1c1498aa5cbbbe49884aad`.
- Frozen suite digest `3c8dd0978d049f68d3bcbf8e89494ab6f4859c724ebf5207003c39d8a21e94ee`.

The implementation upgrade matches the actual campaign script and original manifest. Its prefix is the actual current ledger head. The grant binds the exact next upgrade-row hash, approved200/60/$5 limits, frozen suite digest, and hash of the trusted user approval messageID. An in-memory fold preserves133calls/40groups/$0.106171 with no unknowns and yields67calls/20groups remaining. The ledger bytes were verified unchanged after this read-only check; no grant was attached and no effects dispatched by this reviewer.

**Disposition:** no outstanding material code, oracle, accounting or preregistration finding for this bounded evaluation. The reviewed artifacts may be attached through the existing normal approval/admission path. Each live run still needs its own exact fresh source-bound admission and fresh price/resource preflight; this review grants no new authority. Record actual stage outcomes and all denominators, and retain the limited-transfer and final-independent-assessment qualifications.

First live run admission also independently verified without effects: `target/heldout-clamp-1.admission.json` hash `e60470d10b114ba81e0d6b37601e00b1ad249c7eae7eb7310e4e7db4cc464a23`; runtime-review admission hash `a4eac1380184761a33841b87d666e800c3977103cda6e985629cc4e2733b67df`. Exact v5 `singleton-clamp`/suite/source/implementation validation passed; runtime admission matches the driver and original campaign manifest. Boundaries are seven generator plus seven reviewer calls, four groups, two shared correction slots,8192 completion tokens/240seconds per request,240000ms tool allowance and one-hour expiry2026-10-27T00:38:27.288338UTC. Provider/model and16/32KiB stage profiles remain fixed. Final native acceptance remains a separate FileReviewer decision; no extra action is granted by review. This is readiness evidence, not a live outcome.

Subsequent prospective admissions were independently verified read-only before their dispatch:

- Unicode: driver hash `ec22c57bf156cf7c3ac56f109041f4fc3723fb580c29e03a5e04213a2be49cbb`; runtime hash `a56a13ec26c95809ab565fe631dc4e0ea88b40db9fc8e3949337a859894b43d2`; exact registered `unicode-prefix`/v5 suite binding; expiry2026-10-27T00:42:28.950535UTC.
- Original parser regression: driver hash `6af9b0bdfc07dde44a9e44de0968485363e28eaae0b277d36426f7306da42631`; runtime hash `c062e3d672f7e0a8c776f68ae6081c282d69fe785a28fbb0358aa1e7e0235e5e`; legacy v4 with original scenario digest independently regenerated and compared; expiry2026-10-27T00:49:28.612232UTC. This is regression, not a third held-out task.

Both preserve the same14-call/four-group,8192-token/240-second,240000ms-tool profile and two shared corrections. Runtime admissions exactly match their driver/campaign bindings and current implementation. No provider or worker effects were invoked for these checks; launch-time price and remaining-cap checks are still required.
