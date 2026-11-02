# Genuine local correction and carryover

The installed `hhao/qwen2.5-coder-tools:latest` model completed a real correction cycle on the
small invoice example, followed by an accepted first proposal in a fresh episode. This verifies
the local operator path; it does not establish whole-system completion, real RTS correctness,
fine-tuning, autonomous semantic review, or benchmark superiority.

Machine-readable evidence: [engine-local-correction-result.json](engine-local-correction-result.json).
Local artifacts: `target/engine-operator-walkthrough`. Branch: `codex/m3-evidence-pipeline`.

## What changed

Removing the unsupported schema string repetition constraints fixed the prior Ollama grammar
crash. The first rerun produced a valid proposal in 20.5 seconds. After rejection, the request
exceeded the conservative context budget because it included redundant opaque controller history.
The local adapter now projects current source, public goal/criteria, acceptance guidance and
beliefs, and the last four outcome/observation batches. Omitted batch counts are explicit; complete
records and source bindings remain in the controller. No replacement code is inserted by this
projection. A context hash supplements the full request hash in each attempt journal.

The five earlier failures remain preserved: 32B model timeout, full-protocol controller refusal,
invalid edit shape/path, exact edit bounds, and the schema grammar crash. The first two protocol
failures lack enough saved diagnostics to reconstruct their exact model output; no raw provider
body was retained. The replacement protocol and controller metadata binding subsequently passed
real generation and execution. No failure is relabeled successful.

## Observed sequence

All replacement source bytes came from the local model. Feedback specified contracts and defects,
never a reference answer or repair implementation. The controller assistant reviewed semantics
separately from that generator; this is not independent human review.

1. Initial model patch multiplied discount arguments and left subtotal broken. Sealed tests failed;
   all three semantic requirements were rejected.
2. Explicit current-file contracts led the model to fix subtraction and multiplication. Existing
   tests passed, but review rejected multiplication before an impossible `u32::MAX` comparison.
   This proposal used ordinary commit-only/resume operations.
3. Model moved the overflowing multiplication into a variable. Tests still passed; review rejected
   the same defect and published a concrete boundary counterexample.
4. Model chose checked multiplication with a maximum-value fallback. Tests passed; source review
   accepted all requirements, then accepted-only export succeeded. Redundant code/imports remain;
   they do not violate the stated functional contract.
5. A new episode initialized from the original broken source with `--carry-from`. It inherited six
   reviewed outcomes, including earlier assistant-authored history. Overflow belief was 2/6 met,
   smoothed probability 0.375; product and discount were 5/6, probability 0.75. Published guidance
   included the prior failure lesson. The model chose saturating multiplication and safe subtraction
   on its first proposal. Tests and source review passed; accepted-only export succeeded again.

This is state-dependent generation, not a scripted candidate menu. It shows the changed context
preceding a changed choice, not causal isolation of learned belief from explicit feedback: source,
feedback, history and protocol projection changed across the sequence. Cross-episode guidance is
explicit operator input; the engine does not autonomously infer and publish that lesson.

## Verification and limits

- Five actual new local inference calls, each capped at 2048 output tokens and 120 seconds.
- Five sealed Cargo executions: one failed, four passed. The overflow judgment comes from source
  semantics, not additional protected tests. All seven protected files match original bytes in
  both episodes; original target RTS checkout remains clean.
- `python3 -m unittest discover -s scripts -p test_engine_local_model.py`: 10 passed.
- `python3 -m unittest discover -s scripts -p 'test_engine*.py'`: 53 discovered, 43 passed,
  10 optional skipped with loopback permission. Initial restricted run had 11 socket permission
  errors; rerun with local socket access passed. No Rust code changed in this continuation.
- `git diff --check`: passed.
- Temporary server on `127.0.0.1:11435` stopped; process exited zero. No model download,
  external inference, paid calls, credential creation, pushes, publication or deployment.
- Paid ledger SHA-256 still matches the preceding checkpoint. The external ten-call/$1 RTS
  request remains pending and undispatched. The RTS metadata contract/context gap remains separate.

The engine now has evidence for an operator-usable local correction and carryover cycle. Small-model
quality, manual semantic review, bounded context, and the existing real-corpus/M3 limitations remain.
