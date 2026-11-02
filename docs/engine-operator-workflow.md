# Ordinary operator workflow

The verified CLI path supports initialization, typed local proposals, semantic review, correction,
resume, episode carryover and accepted export without editing the engine database or copying
opaque hashes by hand. Candidate execution still uses the sealed executor.

## Start an episode with its aims

Write a small operator input file such as:

```json
{
  "product": "Compute unit price times quantity when the result fits u32.",
  "discount": "Subtract discount without falling below zero."
}
```

```sh
tsr engine --directory RUN project init --spec SCOPE.json --source SOURCE --requirements requirements.json
tsr engine --directory RUN project prepare
tsr engine --directory RUN project status
tsr engine --directory RUN project acceptance inspect
```

`inspect` is a **private operator view** of requirements, latest review and completion. It is
not a provider request. `status.operator_next` identifies the next operation. Initialization
validates requirement bounds and compatibility before starting the new episode.

## Share guidance and generate a proposal locally

```sh
tsr engine --directory RUN project acceptance draft feedback --out feedback-1.json
# Fill only the guidance strings with information suitable for the generator.
tsr engine --directory RUN project acceptance feedback --file feedback-1.json
```

The draft carries the current revision/review binding automatically. It contains blank guidance,
not private requirement descriptions or review evidence. Blank or stale submissions are refused.
Drafts and requests use exclusive creation, owner-only permissions and paths outside `coding-work`.

**Experimental local model adapter:** actual model-to-accepted-patch correction is now verified on
the small invoice example; see [local correction evidence](engine-local-correction.md). General
coding quality and real RTS correction remain unverified. With an already installed model and an already running local Ollama server:

```sh
tsr engine --directory RUN project generate \
  --model INSTALLED_MODEL --endpoint http://127.0.0.1:11435 \
  --out proposal-1.json --max-output-tokens 2048 --timeout-seconds 120
```

Run the server with cloud features disabled (`OLLAMA_NO_CLOUD=1`) and bound to loopback.
The helper does not start a server, download models, create credentials, use a paid ledger,
or execute the proposal. It refuses nonliteral-loopback endpoints, redirects, obvious remote
models, missing models, pending commitments, completed goals and unpublished rejection feedback.
The operator must use a trusted local server, not a proxy that forwards requests elsewhere.

The model chooses paths, complete visible-unit replacement code and rationale. The controller
attaches the exact current visible old spans and opaque bindings; it never supplies replacement
code. This local adapter supports visible units of at most 2048 bytes; use admitted edit windows
for larger files. Rust previews scope and source validity before a runnable output file is written.
This uses Ollama's [chat API and structured output format](https://docs.ollama.com/api/chat).

Each invocation permits one local inference request, bounded to 2048 generated tokens, a fixed
16,384-token context budget with conservative byte-based admission, a 64-KiB response, and a
caller-selected 1–240-second generation timeout. The model is requested to unload afterward.
A `.local-receipt.jsonl` file records start and completion/failure. Failures never silently retry;
inspect the receipt and choose a new output name for a deliberate new attempt. Controller refusal
diagnostics are bounded. No raw chat response is stored.

A different already-authorized provider can still use `project run -- ...` with its existing
transport configuration. Generation and review never create additional spending permission.

## Inspect, commit and resume

```sh
tsr engine --directory RUN project request --out current-request.json
# Inspect proposal-1.json and the current public request.
tsr engine --directory RUN project run --response proposal-1.json --commit-only
tsr engine --directory RUN project status
tsr engine --directory RUN project resume
```

`--commit-only` freezes the proposal and its expectations without execution. `resume` executes
that unattempted commitment in the sealed executor. Omitting `--commit-only` keeps the existing
commit-and-execute behavior. An uncertain execution is not retried by this command.

## Review semantic completeness, not just test output

```sh
tsr engine --directory RUN project acceptance draft review --out review-1.json
# Set decision and fill every evidence entry after independent inspection.
tsr engine --directory RUN project acceptance review --file review-1.json
```

The draft includes the current snapshot, requirements revision and receipt. Its default decision
is unresolved and evidence is blank, so an untouched draft cannot accidentally accept the goal.
Use `accept`, `reject` or `unresolved`. Inspection remains an operator responsibility; the model
cannot issue this authority through a proposal. Acceptance is not proof that the operator is right.

After rejection, create a new feedback draft, publish sanitized guidance, generate a fresh proposal,
and repeat. All existing edit, execution and budget limits remain in force. To expand the goal,
submit a revised requirements input using `acceptance requirements --file`; old evidence remains
in the append-only record and the new goal needs a fresh review.

## Export and start the next episode

```sh
tsr engine --directory RUN project export --accepted-only
tsr engine --directory NEXT project init --spec NEXT_SCOPE.json --source NEXT_SOURCE --carry-from RUN
```

Accepted-only export refuses unresolved or rejected goals. Ordinary export remains available for
inspection. Neither applies the patch to the original repository or publishes it.

The next initialization inherits the prior requirements and imports compatible reviewed outcomes.
Use `--requirements expanded.json` as well to add aims while preserving existing ones. New aims
start without inherited acceptance evidence. Publish new shareable guidance, prepare, generate,
execute and review the new source; an accepted previous episode cannot accept it automatically.

## Evidence and limits

The operator walkthrough in `target/engine-operator-walkthrough` uses only public CLI operations.
The first episode's edits were authored by this assistant from the current request. After an
additional overflow requirement, the fresh request identified overflow as unmet while product
and discount were met. The assistant proposed `saturating_mul`, preserving the correct discount
function, then exercised commit-only/resume, semantic review and accepted-only export. Those JSON
files are authored input artifacts, not edits to internal engine state. The overflow judgment is
source inspection of the Rust operation, not an added protected test or independent human review.

The earlier loopback provider regression remains explicitly **synthetic**. It validates routing
and persistence, not general generation. Local Ollama attempts are actual model inference from
current state and are recorded separately, including failures; they do not establish benchmark
superiority or long-term learning. The real RTS correction plan remains separately pending.


## Actual local-model attempts at this checkpoint

Five local requests were made, with zero external inference or paid calls:

- `qwen2.5-coder:32b`: 180-second timeout; no proposal or execution.
- Smaller installed Qwen coder, full protocol: controller refusal; no execution.
- Controller-bound metadata protocol: invalid edit shape/path; no execution.
- Path-constrained protocol: Rust refused `exact edit bounds`; no execution.
- Complete visible-unit replacement protocol: the local runner rejected the JSON schema's
  repetition limits and terminated. The helper recorded failure and did not execute anything.

The last failure is concrete: `number of repetitions exceeds sane defaults` in Ollama's grammar
parser. Large string repetition constraints have been removed from the schema while Rust retains
its byte bounds. That change was subsequently revalidated with actual inference; see [the follow-up](engine-local-correction.md).
The temporary loopback server was stopped. Failed receipts remain in the walkthrough directory.
The local-model adapter is experimental; do not present these attempts as successful generation
or as improvements in coding quality. The tested operator path currently uses assistant-authored
responses or an already authorized provider. The external RTS grant is still pending.

The local adapter now supplies current source, published acceptance guidance/beliefs, and at most
four recent outcome/observation batches. It reports omitted batch counts and retains complete
evidence and opaque bindings in the controller. The same conservative context bound still applies.

## Binding a reused scope to source

When reusing a scope whose byte edit window was calculated from a previous episode, pass the
exact corresponding source with `init --source`. A saved scope can contain older embedded source.
Before generation, inspect `project request`: verify that the visible window contains only the
intended function body and that the surrounding protected bytes belong to the intended snapshot.
The local adapter preserves `edit_window` descriptors and refuses unchanged replacements with an
explicit diagnostic. It never strips model-authored function wrappers to manufacture a repair.
