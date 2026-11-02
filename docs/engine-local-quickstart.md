# Run the operator-assisted local engine

This quickstart is verified from a fresh disposable episode on the connected Mac. Run commands
from the repository root. It uses the existing `target/debug/tsr`, Docker, the pinned image in
`examples/engine-cargo/scope.json`, and installed Ollama model `qwen2.5-coder:32b`. It downloads
nothing and makes no paid or external model calls. If any prerequisite is absent, stop and arrange
it separately; this guide does not grant installation or download authority. The example scope's
container user is `501:20`; use an appropriate reviewed scope on another machine.

Build the CLI if needed with the installed Rust 1.90 toolchain:

```sh
/opt/homebrew/bin/cargo build -p tsr-cli --bin tsr --offline
```

## 1. Start the local provider

In a separate terminal, run this foreground service. Stop it with Ctrl-C when finished. If port
11435 is occupied, use your existing trusted local-only service or a different explicit port.

```sh
OLLAMA_HOST=127.0.0.1:11435 OLLAMA_NO_CLOUD=1 OLLAMA_NOPRUNE=1 \
OLLAMA_NUM_PARALLEL=1 OLLAMA_MAX_LOADED_MODELS=1 OLLAMA_MAX_QUEUE=1 \
OLLAMA_KEEP_ALIVE=0 /opt/homebrew/bin/ollama serve
```

## 2. Initialize aims and publish the task contract

Choose a new directory; initialization and output creation refuse overwrites.

```sh
export TSR_DEMO_OUT="$PWD/target/my-local-engine"
mkdir "$TSR_DEMO_OUT"
export TSR_DEMO_RUN="$TSR_DEMO_OUT/run"
target/debug/tsr engine --directory "$TSR_DEMO_RUN" project init \
  --spec examples/engine-cargo/scope.json --source examples/engine-cargo \
  --requirements examples/engine-cargo/requirements.json
target/debug/tsr engine --directory "$TSR_DEMO_RUN" project prepare
target/debug/tsr engine --directory "$TSR_DEMO_RUN" project acceptance inspect
target/debug/tsr engine --directory "$TSR_DEMO_RUN" project acceptance draft feedback \
  --out "$TSR_DEMO_OUT/feedback.json"
python3 - <<'PY'
import json, os
from pathlib import Path
p = Path(os.environ['TSR_DEMO_OUT']) / 'feedback.json'
value = json.loads(p.read_text())
value['guidance'] = json.load(open('examples/engine-cargo/guidance.json'))
p.write_text(json.dumps(value, indent=2) + '\n')
PY
target/debug/tsr engine --directory "$TSR_DEMO_RUN" project acceptance feedback \
  --file "$TSR_DEMO_OUT/feedback.json"
target/debug/tsr engine --directory "$TSR_DEMO_RUN" project request \
  --out "$TSR_DEMO_OUT/request.json"
```

These are operator input files, not edits to internal state. Requirements remain private until
explicit guidance is published. Inspect the request and editable source before generation.

## 3. Generate, commit, restart and execute

```sh
target/debug/tsr engine --directory "$TSR_DEMO_RUN" project generate \
  --model qwen2.5-coder:32b --endpoint http://127.0.0.1:11435 \
  --out "$TSR_DEMO_OUT/proposal.json" --max-output-tokens 1024 --timeout-seconds 240
cat "$TSR_DEMO_OUT/proposal.json"
target/debug/tsr engine --directory "$TSR_DEMO_RUN" project run \
  --response "$TSR_DEMO_OUT/proposal.json" --commit-only
target/debug/tsr engine --directory "$TSR_DEMO_RUN" project status
# This is a new process; the pending commitment and frozen expectations persist.
target/debug/tsr engine --directory "$TSR_DEMO_RUN" project resume
target/debug/tsr engine --directory "$TSR_DEMO_RUN" project status
```

Generation does not execute code. Execution uses the sealed offline container. A timeout, invalid
proposal or no-op stops the attempt; inspect its `.local-receipt.jsonl`. No automatic retry occurs.
For a deliberate retry choose a new output filename. Never rewrite a failed proposal into a
supposed model success. If generation fails, do not continue with a missing proposal file.

## 4. Review semantic correctness, then export

```sh
target/debug/tsr engine --directory "$TSR_DEMO_RUN" project acceptance inspect
target/debug/tsr engine --directory "$TSR_DEMO_RUN" project acceptance draft review \
  --out "$TSR_DEMO_OUT/review.json"
cat "$TSR_DEMO_RUN/coding-work/src/pricing.rs"
cat "$TSR_DEMO_RUN/coding-work/src/discount.rs"
nano "$TSR_DEMO_OUT/review.json"
```

Review is intentionally manual. Compare the actual code and execution result with all three
requirements. Verify representable products, subtraction floored at zero, and overflow without
panic/wrap. Passing existing tests alone does not establish the overflow requirement. Set every
`evidence` entry's `met` flag and a nonempty factual explanation. Choose `accept` only if every
requirement is met and the current tests passed; otherwise choose `reject` or `unresolved`.
Do not copy an acceptance from another episode. The untouched draft cannot accept anything.

```sh
target/debug/tsr engine --directory "$TSR_DEMO_RUN" project acceptance review \
  --file "$TSR_DEMO_OUT/review.json"
target/debug/tsr engine --directory "$TSR_DEMO_RUN" project export --accepted-only
```

If rejected, create a **new** feedback draft, write the specific defects as shareable guidance,
publish it, and repeat generation/execution/review with new filenames. Existing examples and
protected tests remain unchanged. Acceptance/export neither applies the patch to the original
source nor authorizes publication.

## 5. Start a subsequent episode with carryover

```sh
target/debug/tsr engine --directory "$TSR_DEMO_OUT/next" project init \
  --spec examples/engine-cargo/scope.json --source examples/engine-cargo \
  --carry-from "$TSR_DEMO_RUN"
target/debug/tsr engine --directory "$TSR_DEMO_OUT/next" project acceptance inspect
target/debug/tsr engine --directory "$TSR_DEMO_OUT/next" project status
```

The next episode inherits all requirements and compatible reviewed outcomes, not acceptance of
its new source. To work on it, set `TSR_DEMO_RUN` to that directory, prepare it, publish fresh
feedback, then repeat steps 3–4 with new output names. No database, event log or internal snapshot
needs hand editing. Carryover is explicit; the engine does not scan unrelated sessions.

See [the functional audit](engine-functional-audit.md) for what each TSR concept does, verified
results, and limits. This delivers an operator-assisted system; it does not claim unattended review
or statistically proven superiority.
