# Generated exact edits in the persistent coding loop

For explicit multi-file source/test scopes and edit sets, see
[the project workflow](engine-project.md). The single-module mode below remains supported.

`coding-init --generated` explicitly admits a disposable Rust workspace and one
editable file, `coding-work/implementation.rs`. The goal and protected probe are
operator/controller configuration. There is no preconfigured repair catalog in
this mode: an `exact_edit` proposal supplies a unique old span and replacement,
and Rust validates it using the existing `prepare_edit` boundary.

This slice retains a deliberately small domain: a Rust module providing
`combine(i32, i32) -> i32`, checked for `(2,3) -> 5` and `(7,0) -> 7`. It does not
import an arbitrary project or confer authority to choose commands or tests.
Any bounded Rust edit to this admitted implementation can become a candidate;
adding a novel patch does not require editing Rust controller code.

## Runnable demonstration

From the repository root, with the pinned image already loaded:

```sh
/opt/homebrew/bin/cargo build -p tsr-cli --offline
python3 scripts/engine-coding-smoke.py --generated \
  --image sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6 \
  --output /tmp/engine-generated-coding-trace.json
```

This creates a fresh directory under `target`, runs two novel replay proposals
through the offline completion decoder, and starts a separate CLI process for
every request, ingest, plan, execution and final request. It performs no inference
or network calls. It leaves the workspace, event store and receipts intact.

[Captured real execution](engine-generated-coding-trace.json) demonstrates:

| Generated replacement | Observed behavior | Per-patch estimate |
| --- | --- | --- |
| `(a * 2) - b` | first probe returns `1`, expected `5`; failure | 1/2 → 1/3 |
| `a.checked_add(b).unwrap_or_default()` | returns `5` and `7`; success | 1/2 → 2/3 |

The second patch starts at 1/2, independently of the first patch's result. The
restarted next request includes patch contents, separate estimates, frozen
expectations, outcomes, contrasts, and bounded actual/expected probe outputs.
The protected probe SHA-256 stayed unchanged. The candidate IDs, seven executed
commands and content-addressed receipts are in the trace. The first failing
probe ends its attempt early; the successful attempt runs both probes.

## Short product sequence

```sh
RUN=$(mktemp -d "$PWD/target/generated-edit-XXXXXX")
target/debug/tsr engine --directory "$RUN" coding-init --generated \
  --image sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6
target/debug/tsr engine --directory "$RUN" request > "$RUN/request.json"
python3 scripts/engine-provider-envelope.py --request "$RUN/request.json" > "$RUN/envelope.json"
```

A replay/provider response has this shape, using values from that request:

```json
{
  "protocol": "tsr-proposal@v1",
  "state_root": "COPY_REQUEST_STATE_ROOT",
  "proposals": [{
    "kind": "exact_edit",
    "patch": {
      "path": "implementation.rs",
      "source_digest": "COPY_CODING_SOURCE_DIGEST",
      "inspection_handle": "COPY_CODING_INSPECTION_HANDLE",
      "old_text": "a - b",
      "new_text": "a.checked_add(b).unwrap_or_default()"
    },
    "rationale": "Try addition with a checked result"
  }]
}
```

For a direct replay, save this typed object as `$RUN/proposal.json`. For a saved
chat completion with the typed object in `choices[0].message.content`, decode it:

```sh
python3 scripts/engine-provider-envelope.py --request "$RUN/request.json" \
  --completion "$RUN/completion.json" > "$RUN/proposal.json"
target/debug/tsr engine --directory "$RUN" propose --response "$RUN/proposal.json" --apply
target/debug/tsr engine --directory "$RUN" coding-plan > "$RUN/plan.json"
python3 - "$RUN" <<'PY'
import json, subprocess, sys
from pathlib import Path
run = Path(sys.argv[1])
plan = json.loads((run / 'plan.json').read_text())
subprocess.run(['target/debug/tsr', 'engine', '--directory', str(run), 'coding-run',
                '--commitment', plan['commitment'], '--expected-source', plan['source_before']], check=True)
PY
target/debug/tsr engine --directory "$RUN" request > "$RUN/next-request.json"
```

Run the final request in a new process at any time to resume. Subsequent edits
must use that fresh request's source, handle and state root.

## Authority, evidence and limits

- The inspection handle binds the admitted workspace, configuration, file and
  source bytes. Exact-edit validation rejects stale digests, ambiguous spans,
  symlinks, traversal, protected-test paths and oversized edits. Each span is
  bounded to 2048 bytes; source to 4096 bytes; each response to 8192 bytes. The
  scope admits at most 128 distinct patch identities.
- Patch identity hashes the exact patch and scoped inspection binding. A Rules
  validation record maps it to the existing criterion and fixed execution cost.
  Proposal text cannot add goals, tests, permissions or command arguments.
  Each identity has its own Beta(1,1) count fold; unchosen patches and censored
  outcomes are not negatives. There is no learned cross-patch grouping.
- Execution revalidates source and commitment, freezes an attempt intent, writes
  the prepared replacement as a positional argument to a fixed script, then
  compiles the protected probe. All commands use the existing sealed runner.
  Probe executions mount the workspace **read-only**; the store, receipts and
  credentials are never mounted. The root filesystem is read-only, network is
  disabled, privileges/capabilities are restricted, and each command has a
  30-second deadline.
- The observer checks exact process stdout/exit for each fixed input. Generated
  source cannot make a test-report claim count as a pass. This establishes only
  the two observed behaviors, not general correctness or unseen-input behavior.
  Arbitrary source remains untrusted and runs only inside the sealed environment.
- Timeout, unavailable runtime, incomplete capture, compilation failure and
  integrity failure remain censored. Suspected runtime launch/kill exits are
  treated conservatively. Raw receipt bytes remain outside the workspace;
  bounded probe observations are also recorded in the hashed event history.
  Initialization interruption and interrupted execution still require manual
  reconciliation; the engine never automatically repeats an attempted edit.
- The offline decoder requires one completed assistant response, bounds input,
  rejects duplicate fields, truncation, refusal/tool responses and foreign state.
  Rust performs the final typed-schema and scope validation. Decoding never grants
  execution authority and never contacts a provider.

Validation ([captured Rust output](engine-generated-tests.log)): 40 CLI library tests, 1 CLI integration test and 24 corpus library
tests passed (one preexisting runtime-dependent test ignored). Three Python
codec tests passed. Clippy for `tsr-cli` and `tsr-corpus`, all targets, passed with
warnings denied. The real sealed two-edit run above passed. Focused coverage
includes scope/handle/digest/span rejection, restart, distinct estimates, probe
claim rejection, timeout classification and the read-only mount plan.

The budgeted transport is now connected through `scripts/engine-run.py`. See
[the runnable transport integration](engine-transport.md) for exact local/live
commands, verified accounting/restart behavior and the live payload boundary.
No live inference was performed during that integration. A later authorized
[synthetic live demonstration](engine-live-synthetic.md) succeeded in one call.
