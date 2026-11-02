# Real RTS repair with installed local inference

This continuation repairs the real `supported_languages` task using installed local models,
without paid calls, a reference repair, or changes to the original RTS repository. The external
ten-call/$1 request remains pending and undispatched. Full verification results are captured in
[the result record](engine-rts-local-correction-result.json) and
[the separate full-suite report](engine-rts-local-full-suite-result.json).

## Contract and provenance

The legitimate contract comes from `crates/rts-core/src/languages/mod.rs` at the pinned RTS
checkout: `Language::all()` supplies the complete ordered collection; `name()`, `version()` and
`file_extensions()` supply each entry's authoritative metadata. The public `LanguageInfo`
interface supplies the field types. No `supported_languages` reference implementation was given
to the generator. [The extracted metadata contract](engine-rts-local-metadata-contract.json)
records all 13 languages, values, source path and SHA-256 for review.

The local generator received public API signatures, the completeness/order requirement, the
visible failed candidate body, persistent outcomes and published rejection feedback. It did not
receive an assistant-authored replacement expression or a hidden test. The controller attached
only the original source span and opaque bindings. Every replacement code byte came from a model.

## Failures retained and corrections made

1. The first initialization mistakenly omitted `--source`: saved byte offsets matched the failed
   model source, while embedded source in that scope was an older mutant. Its attempt is preserved
   but excluded from valid repair evidence. A new episode explicitly used the rejected model's
   workspace and verified the body-only window. No original target or protected file was altered.
2. In the correctly bound episode, the smaller model expanded to 12 guessed entries. The named
   test passed, but independent source review rejected omitted Python, wrong display names,
   invented versions and incomplete extensions. The goal remained open.
3. The smaller model then used the correct public accessors but supplied a whole nested function.
   Compilation failed with E0308 because the existing outer function returned unit. The local
   adapter was found to omit `edit_window` in its compact context; that descriptor is now preserved.
4. Two subsequent small-model proposals were unchanged. Both were refused without execution.
   The adapter now diagnoses unchanged source before Rust preview instead of a generic span error.
5. The already-installed `qwen2.5-coder:32b` model generated the body-only correction in about
   49.5 seconds. It derives all entries and fields from the public API. No model download or
   assistant rewriting was used. The prior small model was `hhao/qwen2.5-coder-tools:latest`.

There were six local inference calls, including the setup-invalid attempt and two no-ops. Each
small-model call was bounded to 120 seconds/2048 output tokens; the larger call to 240 seconds/1024
output tokens. Receipts and proposal hashes remain under `target/engine-rts-local-correction`.
No failed attempt, event or receipt was overwritten.

## Independent acceptance

The controller assistant, separate from the local generator, inspects the final function against
the immutable public API contract. The final expression iterates `Language::all()` without
filtering or reordering, produces one `LanguageInfo` per element, and obtains each field from its
corresponding authoritative accessor. Those accessors and their 13-element collection are
protected and verified unchanged. This is source-semantic attestation, not independent human
review or a claim that the weak named test proves completeness.

The named test runs through the ordinary engine. A separate full-suite runner uses an immutable
image, read-only source, no network, a nonroot user, dropped capabilities, bounded resources and
an isolated clone of prior build scratch. It checks the full 479-file source inventory and exact
previous successful test inventory. The final review is bound to the current snapshot, requirement
revision and named receipt; the full-suite evidence is cited in the operator attestation. Accepted
export follows that review and does not modify the source repository.

The accepted patch is relative to the rejected two-entry candidate that initialized this episode,
not an automatically applied change to the original repository. The original checkout remains clean.

## Scope of the result

This verifies a real operator-assisted correction using genuine local generation and persistent
rejection feedback. It does not establish independent model discovery of the task contract,
autonomous semantic judgment, fine-tuning or benchmark superiority. The controller supplied
legitimate API facts and increasingly specific defect explanations. M3 remains incomplete under
its corpus and statistical acceptance requirements. See [practical completion criteria](engine-practical-completion.md).

## Verified closeout

- Final protected named test: passed.
- Separate full workspace build: exit 0, 33.716 seconds.
- Full workspace test: exit 0, 380.962 seconds; **999 passed, 15 ignored, zero failed**,
  with the exact prior successful test inventory across 96 summary groups.
- All 479 full-suite source files unchanged; 56 protected candidate files and the immutable
  prefix/suffix around the admitted body verified. Accepted source equals full-suite source.
- Current status: `goal_accepted`; both semantic desires fulfilled; accepted-only export produced.
- Adapter tests: 11 passed. Full Python engine regressions: 54 discovered, 44 passed, 10 optional
  skipped. `git diff --check` passed. No Rust code changed in this continuation.
- Temporary inference server stopped. Paid ledger and pending external request hashes unchanged.
  No external calls, paid calls, push, deployment or original-target modification.
