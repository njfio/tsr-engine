# Scoped multi-file Rust projects

For importing a normal, bounded Cargo workspace and exporting patches, see
[the Cargo adapter guide](engine-cargo.md).

The persistent engine now accepts an owner-authored project specification with
explicit editable Rust files, protected Rust files, a protected probe entrypoint,
named input/output test cases and a goal. It copies only those admitted bytes into
a disposable workspace. `project-config` shows that immutable admission;
`request` shows current scoped source plus bounded experience. The existing
budgeted runner provides run/resume and stops at success, budget or uncertainty.

A generated proposal can contain one exact edit or an `edit_set` spanning up to
four different admitted files. It does not require adding controller actions in
Rust. The model may repair the whole project in one step. It cannot change the
file scope, protected tests, test cases, commands, runtime image or permissions.

## Initialize, inspect, run and resume

The newly authored example is [examples/engine-invoice/project.json](../examples/engine-invoice/project.json).
It admits `src/pricing.rs` and `src/discount.rs`; `tests/probe.rs` is protected.
The initial project has subtotal and discount bugs, including a nonnegative-total
requirement. There are no external dependencies or private repository inputs.

From the repository root:

```sh
/opt/homebrew/bin/cargo build -p tsr-cli --offline
RUN=$(mktemp -d "$PWD/target/invoice-project-XXXXXX")
target/debug/tsr engine --directory "$RUN" project-init \
  --spec examples/engine-invoice/project.json
target/debug/tsr engine --directory "$RUN" project-config
target/debug/tsr engine --directory "$RUN" request > "$RUN/request.json"
```

The specification is an explicit content manifest: `editable` and `protected` map
relative file paths to initial UTF-8 contents. It also names `goal`, pinned `image`,
numeric nonroot `user`, `probe_entry`, `cases` and `timeout_ms`. It does not scan a
repository or collect unrelated files. For another scoped project, construct the
manifest from exactly the files the owner intends to admit.

With authorization for the displayed payload, the same existing transport runs
and resumes the initialized project:

```sh
python3 scripts/engine-run.py --directory "$RUN" --approved-live \
  --ledger "$PWD/target/m3-provider-cost.jsonl" \
  --max-steps 6 --max-usd 2 --max-tokens 400000 \
  --call-timeout-seconds 55 --steps-this-invocation 1
```

The optional one-step checkpoint lets the operator inspect the next `request`
before the next dispatch. Resume by repeating the same command and overall bounds;
the journal retains calls, cost baseline, commitments and outcomes. Omit the pause
to iterate automatically. A completed run returns success without another call.
The saved scope and budget cannot be expanded by a generated goal refinement.

For offline proposals, use the same `request` → typed response → `propose --apply`
→ `coding-plan` → `coding-run` sequence documented in
[the generated-edit guide](engine-generated-coding.md). Each patch copies the
chosen file's `source_digest` and `inspection_handle` from `coding.files`:

```json
{
  "protocol": "tsr-proposal@v1",
  "state_root": "CURRENT_REQUEST_ROOT",
  "proposals": [{
    "kind": "edit_set",
    "patches": [{
      "path": "src/pricing.rs",
      "source_digest": "CURRENT_FILE_DIGEST",
      "inspection_handle": "CURRENT_FILE_HANDLE",
      "old_text": "unit_cents + quantity",
      "new_text": "unit_cents * quantity"
    }],
    "rationale": "Repair the admitted subtotal implementation"
  }]
}
```

Add another distinct-file patch to that set for a combined repair. The whole set
is validated before commitment; inspection handles bind the complete admitted
project snapshot, so changing another file invalidates an old handle. A set's
identity is independent of edit ordering. Each patch/set retains its own forecast
and outcome counts; one patch's success is not generalized to other patches.

## Execution and feedback

The controller prepares every exact edit with the existing digest/unique-span
validator. A fixed script writes the prepared path/content pairs as quoted
positional arguments, then the fixed compiler adapter invokes:

```text
rustc --edition=2021 <protected probe_entry> -o fixture-probe
./fixture-probe <owner-configured case arguments>
```

Proposals cannot supply shell or compiler arguments. All commands use the existing
sealed nonroot runner. Probe execution mounts the workspace read-only; the event
store, budget ledger and receipts remain outside that mount. The controller
checks every protected file and all admitted source bytes after each stage and
rejects unadmitted files or symlinks. Partial write/integrity failures stop for
reconciliation; a multi-file write is not claimed to be a filesystem transaction.

All configured cases run after ordinary test failures to collect useful feedback.
Each passing case records its own met criterion; contrast names only the remaining
unmet criteria. Actual stdout/exit is checked against the owner's expected output,
not against a candidate's claim that tests passed. Timeout, unavailable execution,
compilation failure, incomplete capture and integrity failures are censored and do
not train success/failure counts. These failures stop the transport loop rather
than triggering hidden retries.

Generated goal refinements remain suggestions. They cannot add editable paths or
weaken protected tests. The admitted configuration is persisted independently of
preferences; a changed project scope requires explicit owner configuration of a
new run. Tests cover a goal-refinement suggestion failing to expand authority.

## Local replay, then actual live result

The local sealed replay command is:

```sh
python3 scripts/engine-coding-smoke.py \
  --project-spec examples/engine-invoice/project.json \
  --output /tmp/engine-project-replay.json
```

[Captured replay](engine-project-replay.json) repaired pricing first. Subtotal
passed while discount and floor remained unmet. A new process reconstructed those
named observations; the second scoped edit repaired discount, and all cases
passed. This is a replay test of feedback persistence, not a model-learning claim.

For the authorized live demonstration, a fresh project was initialized under
`target/engine-project-live-avqw1v3r`. The [payload preflight](engine-project-live-preflight.json)
verified that the 7123-byte request contained only the new synthetic project,
protected probe, synthetic goal/cases and fresh state—no private source, personal
data, environment values, credentials or local sensitive paths. The existing key
was used normally and never displayed or copied.

The exact live run was:

```sh
python3 scripts/engine-run.py \
  --directory target/engine-project-live-avqw1v3r --approved-live \
  --ledger "$PWD/target/m3-provider-cost.jsonl" \
  --max-steps 6 --max-usd 2 --max-tokens 400000 \
  --call-timeout-seconds 55 --steps-this-invocation 1 \
  > target/engine-project-live-avqw1v3r/live-invocation-1.json
```

[Actual live evidence](engine-project-live-result.json): DeepSeek returned one
edit set changing both files, using `saturating_mul` for subtotal and
`saturating_sub` for discounts. All five sealed commands exited 0. The three
observed outputs were `300`, `270` and `0`; protected source remained unchanged.
The frozen estimate was 0.5 and the completed patch-set estimate became 2/3.
It succeeded first try, so **no live feedback-driven subsequent proposal was
needed or demonstrated**. No failure was forced and no extra call was made.

| Live accounting | Value |
| --- | --- |
| Completion calls / steps | 1 / 1 |
| Billed tokens | 2684 |
| Reservation | $0.25 |
| Settled additional cost | **$0.000419** |
| Cumulative cost before / after | $0.617454 / **$0.617873** |
| Unresolved reservations | 0 |

A new process reconstructed the source, observations, forecast and outcome.
Repeating the bounded runner on that completed state returned success with the
ledger byte-for-byte unchanged. The actual workspace, receipts and resume result
remain available in the directory above. No containment failure was observed.
No benchmark cohort, unrelated cleanup, credentials, Docker restart, push or
publication was created.

## Verification and current bounds

[Captured Rust tests](engine-project-tests.log): 44 library tests and one CLI
integration test passed. New checks cover partial named-case returns and restart,
a complete multi-file bundle, stale cross-file handles, duplicate paths, protected
file/goal authority, and rejecting oversized bundles before committing. Clippy
for `tsr-cli`, all targets, passed with warnings denied. The existing 16 broker
and four codec checks passed. Local replay and the live run used the real sealed
adapter; restart was verified without another inference call.

The current adapter is for self-contained Rust module projects compiled with a
protected probe, not arbitrary Cargo workspaces or dependency/build-script
execution. Bounds: at most eight editable and twelve total admitted files,
4096 bytes per file and 8192 total source bytes, four cases, four distinct-file
edits per set, 2048 bytes per span, and an 8192-byte proposal response. A request
is bounded to 32768 bytes; older patch text can be omitted while current source
and recent observations are retained. Command deadlines are owner-configured up
to 20 seconds. Initialization or interrupted execution can require manual
reconciliation. The observed cases establish only their tested behaviors; this
is not M3 real-corpus acceptance or proof of general project correctness.
