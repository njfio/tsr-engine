# One-task prospective admission

Follow-up: the bounded episode has now run and closed rejected. See [actual outcomes and next context improvement](engine-one-task-outcome.md). The preparation record below describes the original admission.

Prepared without inference or an operational episode. The runnable preparation is
`scripts/engine-collection-ready.py`; verified output is `target/collection-one-ready-v4`.
This is a development episode, not an unexposed holdout or evidence of calibration.

## Exact scope

- Task: `39385564a640e614fb7a8958eef24266728525a5364238611d2ab52bc345a2d7`.
- Read-only source: `target/m3-history/repo`, commit
  `02cca9df15e72f6506275157b0198526950d6fcb`, RTS repository lineage
  `https://github.com/njfio/rs-agent-code-utility`.
- Admitted mutation snapshot:
  `ea107cdbe701032ce10669c6baaf2031ea5935200310e30db7dc9966f9021413`.
  Mutation and generated lock are verified against the original corpus BLAKE3 CAS
  identities before use. `readiness.json` records those identities and SHA256 of
  all 323 copied files. Git history and reference reversal are never copied.
- Image: `sha256:f4ac0e0b42b4188fa2e645566edcc84c51de05aea0a2947ff06f23e6052100b6`.
- Only editable path: `crates/rts-mcp/src/surface.rs`, the 700-byte parsing
  implementation window recorded in `scope.json`. All bytes outside the window,
  inline tests, other source, build assets and Cargo.lock are protected.
- Standing criteria: exact named-list membership, trimming/empty-entry behavior,
  unknown-name rejection, preserved existing profiles/observability/case behavior,
  named test and workspace suite, independent semantic review. Public contract is
  in `Surface::parse` documentation; no reference patch is model input.
- Allowed actions: controller-issued inspection, bounded exact edit, owned offline
  build/test commands in the sealed nonroot container and independent review/export.
  No arbitrary argv, network, source-repository writes, deployment or publication.

Independent named oracle:
`surface::tests::a_named_list_is_exact_and_an_unknown_name_is_an_error` in `rts-mcp`.
Execution additionally includes the frozen full-workspace oracle:
`cargo test --locked --offline --workspace --no-fail-fast -j 1 --target-dir /tsr-build -- --format=pretty --test-threads=1`.
These demonstrate executed assertions and regression coverage, not correctness for
all possible inputs. Timeout, truncation, missing tests or missing accounting is
unresolved, never a pass. Full suite execution has not been performed in this
readiness pass; the scope includes it for actual candidate execution.

## Bounds and review

Installed local model: `hhao/qwen2.5-coder-tools:latest`; manifest SHA256
`b7fade0651b448e0e57af67d9edcbdcb686cd8a9647c2cb1f123858438af0a33`.
Local manifest/config/layer existence and sizes were checked, not rehashed 4.7GB
weights. Before dispatch compare the provider model digest to this pin; stop on
change, do not download. Instrumented transport uses temperature 0, context 16,384,
output at most 2,048 tokens, timeout 240 seconds, three calls maximum. Total token
reservation 55,296, external charge ceiling 0, episode wall ceiling 3,600,000ms,
tool ceiling 1,800,000ms, patch bytes 32,768. Each command is bounded at 120,000ms.
The adapter has four commands per candidate and separately charges preparation.
Patch accounting charges the reconstructed whole file (~13.4KB), not just its
700-byte window: this budget normally permits **two executions**, even though
three generation slots exist. Exhaustion stops with unmet desire; no reset/refund.
Unknown dispatched requests retain their reservation until honestly reconciled.

The generator receives only admitted context and feedback, never the full source
copy, protected tests, Git history, mutation diff, reversal, qualification results
or reviewer-only evidence. No generator self-review is accepted. A separate review
agent, with operator accountability, receives the frozen contract, current patch,
protected-file integrity report, named and full-suite receipts and budget ledger.
It checks general parsing semantics, unchanged profile/router behavior and absence
of test-specific handling. Review binds snapshot, requirements revision and current
receipt; contradictory or incomplete evidence is unresolved/rejected. Its textual
identity in the template is `independent-review-agent-plus-operator`; replace with
an accountable actual reviewer identity before opening if different.

Record active operator/reviewer effort prospectively in a separate local review
note: reviewer, actual start/end, measured active minutes, method and interruptions.
Unknown remains unknown. Current engine receipt cost does **not** include this
metric; event elapsed time is not operator effort. This is the remaining concrete
cost-evaluation telemetry limitation, not missing authority to prepare this task.

Common event: `task_accepted_in_budget@v1`; family `(UnaryOperator, rts-mcp)`;
clone group `4bad462dc1db25e1b3fd397570886ae97721bf56f7c425aae6135c93b7f38513`.
Record optional generator/common-event forecasts before action, never invent missing
values. One task is not independent family support. Exclude qualification, replay,
synthetic fixtures, prior exposures and overlapping clones from fresh holdout claims.
No mature count is established; the 151 qualified corpus tasks remain intact.

## Runnable preparation and subsequent episode

From the engine checkout (new output directory required):

```sh
python3 scripts/engine-collection-ready.py --out target/collection-one-new
```

The helper requires the existing `target/m3-blake3-checksum` built from
`scripts/m3-blake3-checksum.rs` and the existing local model installation. It copies
source, creates scope/requirements/admission-template and verifies provenance; it
never starts a service, opens a lifecycle, executes source or calls a model.

For the next explicitly resumed local collection pass, initialize a fresh episode
(do not reuse the disclosed preflight run):

```sh
export READY="$PWD/target/collection-one-ready-v4"
export RUN="$READY/episode"
target/debug/tsr engine --directory "$RUN" project init --spec "$READY/scope.json" --source "$READY/source" --requirements "$READY/requirements.json"
target/debug/tsr engine --directory "$RUN" shadow-view > "$READY/episode-state.json"
python3 - <<'PY'
import json, os, pathlib
p = pathlib.Path(os.environ['READY'])
a = json.loads((p/'admission-template.json').read_text())
a['state_root'] = json.loads((p/'episode-state.json').read_text())['state_root']
(p/'admission.json').write_text(json.dumps(a, indent=2)+'\n')
PY
target/debug/tsr engine --directory "$RUN" project-lifecycle-open --admission "$READY/admission.json"
target/debug/tsr engine --directory "$RUN" project prepare
```

Only then start the temporary loopback Ollama service using the existing local
procedure and verify the pinned digest. Future bounded call (not run here):

```sh
python3 scripts/engine-local-model.py --directory "$RUN" --endpoint http://127.0.0.1:11435 --model hhao/qwen2.5-coder-tools:latest --out "$RUN/proposal-1.json" --max-output-tokens 2048 --timeout-seconds 240
target/debug/tsr engine --directory "$RUN" project run --response "$RUN/proposal-1.json"
target/debug/tsr engine --directory "$RUN" project acceptance inspect
```

Review independently with `project acceptance` commands, preserving errors and
rejections. Continue only remaining budgeted requests with new filenames after
journal reconciliation; never supply an authored repair. Export only accepted
results. Close after review/resource reconciliation using `project-lifecycle-close`.
Compute earliest maturity from that actual closure plus seven days with
`engine-collection-plan.py maturity`; use an actual current explicit `--as-of`
for eligibility, never the future maturity date as today's cutoff. There is no
closure or maturity date yet. Stop the temporary provider at terminal result.

## Verified readiness and remaining dependencies

V4 preflight initialization/request succeeded; source request contained exactly
one 700-byte source window. Canonical mutation/lock CAS verification passed. All
workspace crates and their schema/corpus TOML fixtures are frozen. Offline
preparation in the pinned sealed container returned ready with receipt
`14ef1aeac7e27bb030f740b0460d2fecf77a7f279992c1c60f435385283fa79a`.
Repository hooks are excluded. The original optional
`workspace_hook_marker_matches_crate_version` test skips without `.claude/hooks/rts-nudge.sh`;
this check is outside this isolated oracle's coverage. The latency benchmark is
ignored by default. Full-suite runtime readiness remains unverified until actual
candidate execution; dependency success is not suite success.
Preflight is not a prospective operational episode and will not be used as its
outcome, usage or maturity evidence. No model service was started and zero calls
were dispatched. Original repository status was preserved.

The actual task exposed a 64-byte test-name adapter limit. Qualified names now
allow 256 bytes while retaining literal-name validation; original tests unchanged.
CLI library: 131 passed, three ignored. Eight planner/readiness Python checks and
all-target CLI clippy passed. Logs: `target/collection-one-{cli-tests,python-tests,clippy,build}.log`.
Independent review required CAS verification and explicit whole-file patch-budget
limits; both are addressed. No operational Tier2 activation occurred.

No new scope or source-sharing authorization is needed for this local task.
Inference is deliberately pending the next collection instruction because this
pass explicitly prohibits calls. Semantic review must actually happen after a
candidate exists; its absence is not a preflight scope blocker. Mature calibration
requires real eligible episodes, elapsed time and held-out validation; useful
provider generation remains unverified here. Two protected cutoff tests remain
unchanged pending the previously requested narrow invocation decision. These are
specific dependencies, not claims of whole-system completion.
