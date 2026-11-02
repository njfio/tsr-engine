# Real repair Trial 19: protocol correction succeeded, patch not applied

At source `65c432d`, the engine ran the frozen real Cargo fixture with separate runtime reviews and an explicit two-correction allowance. The earlier Trial 18 remains unchanged.

The authored flawed seed passed the empty-input check and failed preservation of existing behavior. Initial generated guidance passed a separate Luna review. The generated experiment first violated its envelope with unsupported fields. That failure and its $0.001181 charge were retained. The engine sent one new counted request with the exact validation error; the fresh full response passed strict native validation and separate review, and its tests actually ran in the sealed worker.

The experiment was nonvacuous: signed-value and malformed-value assertions preceded the observed `1,,2` failure; assertions after that failure were not executed. The empty-input case passed. The subsequent relational revision used the actual narrower evidence, preserving both original desires. Independent audit found its interpretation appropriately bounded.

The boundary-patch response was rejected as `native_validation_rejected` before any application. Retained structural evidence proves its rationale was 592 UTF-8 bytes against the published 512-byte limit. The generic native verdict may have other causes; the unretained 247-byte replacement cannot be evaluated or reconstructed. No patch execution, original-test rerun, unchanged-experiment reverification, final reflection or acceptance occurred. This run is incomplete, not successful.

## Accounting and integrity

- Five generator requests plus three independent runtime-review requests: eight total.
- Two real execution groups; 41,880 tokens; settled charge **$0.007628**.
- Campaign total **89 calls / 25 groups / $0.061825**, no campaign unknowns. Remaining **11 calls / 15 groups / $4.938175**; no reset. Older Garden Trial 1 unknown reservation remains separate.
- All original protected files remained byte-identical. Generated experiment source matched actual execution. Read-only inspection changed neither history nor ledger and found no unresolved effects.

Evidence: `target/b91-integrated-trial19/{driver.jsonl,provider.jsonl,quarantine,runtime-project-reviews,accounting-summary.json,readonly-inspection-audit.json}`. [Independent review](../specs/lanes/reviews/engine-trial19-result.md).

Protocol correction is now demonstrated with a real provider failure and actual execution afterward. Semantic repair completion is still unproven for this attempt. Native rejection remains outside its existing correction allowance; any further work must use a distinct source-bound admission and preserve this terminal history, not silently reopen it. A separately reviewed prefix continuation could avoid repeating the seed and experiment, request a wholly new patch and keep final formal acceptance independent. It cannot assume that shortening the old rationale would make the unretained patch valid.
