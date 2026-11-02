# Budgeted transport integrated with the coding engine

The same runner now supports [scoped multi-file Rust projects](engine-project.md),
including a captured successful synthetic live run.

A subsequent authorized synthetic live demonstration succeeded in one call; see
[the actual result and cost](engine-live-synthetic.md). Local-only statements below
describe the original transport validation.

The model-backed path is now one bounded command, `scripts/engine-run.py`, operating
on an initialized generated-edit engine directory. It assembles the current
request, invokes the existing broker with the engine envelope/decoder, ingests the
typed response through Rust, executes the admitted edit/probes, records the return,
and requests another proposal until success or an explicit stop. It is not an
evaluation cohort or a benchmark runner.

## Run the validated local path

From this worktree, with the existing image loaded:

```sh
/opt/homebrew/bin/cargo build -p tsr-cli --offline
TSR_ENGINE_TEST_KEEP=1 \
TSR_ENGINE_TEST_IMAGE=sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6 \
python3 scripts/test_engine_run.py \
  EngineRunTests.test_real_sealed_two_proposal_loop_resumes_with_same_accounting -v
```

The test starts a fake provider on literal `127.0.0.1`, initializes a disposable
engine, and invokes the production runner with these arguments (PORT and RUN are
allocated by the test):

```sh
python3 scripts/engine-run.py --directory "$RUN" \
  --local-provider "http://127.0.0.1:$PORT/v1/" \
  --fixture-ledger "$RUN/local-cost.jsonl" \
  --max-steps 2 --max-usd 5 --max-tokens 200000 \
  --steps-this-invocation 1
# Resume with the same configuration, omitting the optional pause:
python3 scripts/engine-run.py --directory "$RUN" \
  --local-provider "http://127.0.0.1:$PORT/v1/" \
  --fixture-ledger "$RUN/local-cost.jsonl" \
  --max-steps 2 --max-usd 5 --max-tokens 200000
```

Local mode injects only the credential-free loopback endpoint into the existing
HTTP client. The request envelope, price validation, reservation/settlement,
decoder, Rust ingestion, sealed execution, journal and stopping logic are the same
as live mode. Local ledgers are marked fixtures and cannot be selected for live
mode. Redirects and proxies are disabled. A local endpoint cannot receive an
Authorization header; the credential loader is not invoked in this mode.

## Observed result

[Captured local evidence](engine-transport-local-evidence.json) records two
completions, a failed patch followed by a successful novel patch, and separate
estimates of 1/3 and 2/3. The second HTTP request included the first probe's actual
`1` versus expected `5`. The first invocation paused after one completed step;
the second resumed and succeeded. Another restart made no new request. A simulated
interruption after the engine saved its outcome recovered that outcome without
repeating execution or inference. The protected probe remained unchanged.

The fixture ledger contains two reservations and two settlements totaling
**0.002 simulated USD**, with zero unresolved reservations. This is test data,
not a real charge. No external request or credential was transmitted.

The successful engine, its transport journal and raw sealed receipts were retained
at `target/engine-transport-7txs2iod`. The fake server has stopped. Engine state can
still be inspected with:

```sh
target/debug/tsr engine --directory target/engine-transport-7txs2iod request
```

The real aggregate ledger, `target/m3-provider-cost.jsonl`, remained identical to
the earlier stop snapshot. Its SHA-256 is
`0f47216c6983974c27e2162898aefc3a5bd6de9c678710d3030287432179833a`.

## Budgets and restart

The existing broker checks the selected model's catalog price and replaces a
reservation only with settled USD billing. Legacy broker calls reserve $2. Engine
calls reserve ten times the conservative request cost (serialized byte count plus
8192 framing tokens, and the existing 16384-output-token bound), rounded upward
to cents. This retains the safety margin while allowing a small request under a
$1 run ceiling. The engine supplies an additional run ceiling; the broker enforces the
smaller of that ceiling and the existing $100 aggregate cap while holding the
ledger lock. A busy ledger stops this runner without waiting indefinitely. Other concurrent ledger spending reduces the remaining run allowance
conservatively. Token allowance and a completion deadline (default 55 seconds,
maximum 55) are also checked. The envelope caps output at 2048 tokens; the reused
broker reserves tokens conservatively using its larger existing output bound.

The append-only, fsynced `engine-transport.jsonl`, outside the candidate mount,
records configuration, call intent, decoded typed proposal, commitment, execution
intent and outcome. It never records credentials or raw provider response bodies.
A nonblocking lock prevents simultaneous runners for one engine. Restart must
retain the same provider, ledger and overall bounds. `--steps-this-invocation`
is only a pause; it does not reset the overall step or cost limits.

Stops include success, step/token/cost limit, refusal, invalid proposal, censored
execution, and uncertain billing. A missing response after a started call is not
retried, even if billing is known. A missing execution outcome is not rerun. A
persisted engine receipt can complete an interrupted transport-journal update.
An ambiguous commit gap stops for manual reconciliation. Unsettled reservations
block further inference, including on restart. Configuration/parse failures exit
2; controlled terminal outcomes return JSON with an explicit `status`.

## Live configuration readiness and minimal authorization

At the original integration checkpoint, **no live call had run**. The later
[synthetic demonstration](engine-live-synthetic.md) records the first live call.
Availability, catalog pricing, credential validity and model response compatibility
were checked during that later demonstration; future runs still recheck pricing.

For a live demonstration, the precise runtime requirements are:

- Destination: `https://api.cheaperinference.com/v1/models` for the catalog and
  `https://api.cheaperinference.com/v1/chat/completions` for inference; model
  `deepseek-v4.1-flash`. No configurable remote destination is accepted.
- Use the existing `CHEAPINFERENCE_API_KEY` entry in
  `~/.config/ds_swarm/env`. This work did not read or create credentials. The key
  belongs only in the Authorization header, never the prompt or candidate mount.
- Use the existing shared `target/m3-provider-cost.jsonl`; live mode refuses
  another ledger or a fixture ledger. Outstanding reservations require manual
  reconciliation before any new completion.
- Authorize the current engine request payload: the admitted goal/criteria and
  preferences; `implementation.rs` source; fixed protected probe and input/output
  cases; source digest/inspection handle; and bounded relevant patch, forecast,
  contrast and probe-outcome history. No unrelated repository files, private
  sessions, environment contents, credentials or cost ledger are sent.
- Authorize at most two completion attempts and a $5 reservation-aware run
  ceiling within the existing $100 aggregate limit, followed by the already
  admitted sealed edits/probes. The aggregate approval does not cause a dispatch;
  the live command must be deliberately invoked.

The exact outbound body can be reviewed without credentials or network:

```sh
target/debug/tsr engine --directory "$RUN" request > "$RUN/request.json"
python3 scripts/engine-provider-envelope.py --request "$RUN/request.json" > "$RUN/envelope.json"
```

When live execution of that payload is authorized, this is the command sequence
from the repository root (documented here, **not executed**):

```sh
RUN=$(mktemp -d "$PWD/target/engine-live-XXXXXX")
target/debug/tsr engine --directory "$RUN" coding-init --generated \
  --image sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6
python3 scripts/engine-run.py --directory "$RUN" --approved-live \
  --ledger "$PWD/target/m3-provider-cost.jsonl" \
  --max-steps 2 --max-usd 5 --max-tokens 200000 --call-timeout-seconds 55
```

The remaining live step is execution with that authorization and existing runtime
configuration, not another transport implementation. The coding domain still has
one editable Rust module and two protected probes; this is not M3 corpus acceptance
or general arbitrary-project automation.

## Verification

[Captured checks](engine-transport-tests.log): 10 transport tests passed, followed
by focused censored-outcome recovery and busy-ledger tests (12 total); 15 existing broker
checks and 4 codec checks passed. They cover real sealed edit/probe execution,
restart without duplicate calls, lost responses, refusal, invalid scope, timeout,
unsettled billing, aggregate/step/token/run-cost stops, and censored recovery.
Python compilation and `git diff --check` passed. No Rust implementation changed
in this transport slice. The initial unprivileged local-server attempt was blocked
by the host sandbox; the authorized loopback/Docker run succeeded with the needed
execution permission. No approval action remains pending.
