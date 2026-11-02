# Trial18 independent result review

Reviewer: `/root/b91_full_review`. No provider calls, executions, intermediate manual approvals, retry or candidate edits by reviewer.

## Terminal result

Trial18 stopped during generator experiment validation, before experiment review/execution, a boundary patch, or final formal review. Three settled calls (two generator, one separate runtime reviewer), one real seed execution group, known cost **$0.003149**. Parent's campaign total is 81 calls / 23 groups / $0.054197, with no unknown outcome. This is not a repaired project or completed B9.1 cycle.

## Exact retained failure

The experiment provider request `517c9466-d5fa-4275-a25a-24ab40350693` returned HTTP 200, finish reason `stop`, with measured charge $0.001421. `provider.jsonl` records `unsupported_experiment_fields` and quarantine digest `614dccc1c07ff78a56f761cd533c71d961140f8eee4d8e2d674ba5e7f017965c`.

The bounded structural record in `driver.jsonl` shows seven fields inside `experiment`, whereas the published payload contract permits only `harness`, `cases`, and `mapping`. Three forbidden extras can be positively identified without reconstructing text: `relational` is retained by name; canonical JSON-string hashes identify `bound_to` (`d7a1d3b7...`) and `return_basis` (`0ce13e84...`). A fourth extra is retained only as a hash and is not needed to establish rejection. The discarded harness source and explanatory text are not available for semantic judgment. Their correctness must not be inferred from plausible shape or two declared cases.

`payload_contract` explicitly lists the three permitted fields and `additional_fields: false`; the validator rejects additional keys without stripping or rewriting them. The outer envelope schema describes an experiment as an object and is weaker than the complete inner contract, but does not contradict it. The response therefore violated an explicit supplied contract. No evidence supports relaxing the parser or silently discarding extra fields. No repeated live attempt is part of this assessment.

## What automatic review actually demonstrated

The first generated relation combined the exact-empty-input exception with preservation of every nonempty result. It remained provisional guidance and chiefly conjoined the original requirements; this is not evidence of non-additive culmination.

The separately invoked runtime reviewer returned a structurally valid, source-bound acceptance for that initial relation. Its findings accurately distinguished desired behavior from demonstrated behavior, cited the actual receipt's empty-case pass and preservation-case failure, and explicitly left assertion execution/values and universal correctness unknown. It granted no execution or native acceptance authority. The $0.001222 review charge and 2094 completion / 4665 prompt tokens were retained. The driver then admitted the relation and generated the next stage without an intermediate human review file.

This establishes one working automatic intermediate review/admission transition on the real project path. It does not establish reviewer performance on generated test assertions, source patches, return-informed revision or formal correctness. Same-model generator/reviewer errors remain correlated. Both original desires remain traceable and the project remains unaccepted; the known preservation failure was not repaired.

## Remaining boundary

The runtime-review integration did not cause this stop: the invalid experiment failed the existing generator contract before it reached the reviewer. Strengthening declarative inner-schema representation could be investigated offline, but the present evidence does not justify rewriting the output or claiming the underlying experiment was useful. Final formal acceptance remains FileReviewer; no final review artifact was appropriate for this trial.
