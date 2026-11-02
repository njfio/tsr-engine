# M3 offline progress and oracle-receipt correction

Implemented M3-PROGRESS-01 and M3-RECEIPTS-01 on `codex/m3-evidence-pipeline`. No provider calls or new cohort were run. The historical smoke results and acceptance criteria were not changed.

The new broker protocol is `tsr-broker@v2`. The CLI uses `broker_v2`; `broker.rs`, the smoke adapter, its executable, and all original freeze inputs remain intact. The versioned provider adapter is `scripts/m3-progress-broker.py`; it was tested only through local encoding, without credentials or HTTP. Old-protocol adapters reject the new request instead of silently omitting progress.

## Progress feedback

Every request exposes exact `remaining_action_calls`, including the action requested now. Inspect, test, replace, and stop each consume one model-selected call. Observations do not consume extra calls. Counts include the existing 1024-call hard ceiling and are identical in both modes.

Controller-owned `progress` is separate from untrusted observation text. It contains a content hash of the candidate input tree, `inspection_current`, `test_current`, and explicit submission semantics: `stop` submits the current candidate for independent final-oracle evaluation; a visible test pass is not acceptance; stop consumes a call. Freshness is recomputed from actual input bytes and typed directory entries. A replacement invalidates prior evidence when content changes. Tests that change inputs invalidate both inspection and test freshness. Repeated inspections do not invalidate a still-current test.

The input fingerprint excludes root `.git` and `target` metadata/build outputs; the v2 path validator refuses inspection/replacement in those roots. It rejects symlinks, special files, and non-UTF-8 paths and caps traversal at 10000 entries and 64 MiB of actual bytes. It is a bounded content-tree identity, not a claim about file timestamps or arbitrary build-output effects. The unconditional “repeat to refresh” warning was replaced with instructions to use explicit freshness fields.

## Final-oracle receipts

`oracle_receipts::Recorder` wraps only the protected final oracle's Runner. It persists actual stdout, stderr, exit, duration, command, base snapshot and preparation identity before normalization, using controller-owned CAS outside the candidate copy. A content-addressed manifest binds ordered run references and capture limits to task, mode, expected tests, original snapshot, and the pre-oracle ledger root. An evidence event links its digest into the task's final ledger. Write/digest failures propagate as errors rather than producing verified success.

`OracleVerdict.evidence` is optional for backward compatibility and contains the manifest digest and `verified_success`. `oracle_receipts::verify` rereads the manifest and stream CAS, checks context and exact command/run coverage, requires successful exact named-test markers and complete passing suite summaries, and refuses verification for failed, incomplete, contradictory, corrupt, missing or potentially truncated output. Either stream at or within three bytes of its capture limit is conservatively unverified because UTF-8 truncation can back up three bytes. Raw verdict/acceptance semantics remain separate and unchanged; existing `tsr-eval` statistics continue to mean raw acceptance. Exported verified-success claims must rerun the receipt verifier rather than trust a stored boolean.

These receipts establish integrity and completeness of controller-captured observations. They do not independently prove arbitrary evaluated-code semantics or output authenticity. The base snapshot and ledger root do not themselves hash the final edited candidate tree. No hidden oracle material is added to broker requests.

## Validation and review

The M3-PROGRESS-01 deterministic regression first failed against frozen v1 because the remaining-call field was absent. With v2, the same inert sequence (inspect → failed visible test → patch → successful visible test → repeated inspection → stop) passes in both modes, produces identical progress facts, reaches the protected oracle, and persists verifiable final receipts. All runners in new tests are inert doubles; no corpus code executes on the host.

Independent review found two issues, both fixed: unbounded directory traversal and omitted empty-directory identity. The empty-directory regression was run with the reviewed defect restored temporarily, failed as expected, then passed with the fix restored. The reviewer found no remaining blocker. See [review receipt](m3-progress-v2-review.json).

Commands and captured outputs:

- `cargo +1.90.0 test -p tsr-batch -p tsr-eval --offline`: **120 passed**, one preexisting ignored test; [log](m3-validation/m3-progress-v2-tests.log).
- `cargo +1.90.0 test -p tsr-cli --test m3_broker_tensions --offline`: **2 passed**; [log](m3-validation/m3-progress-v2-cli-tests.log).
- `python3 -B scripts/test_m3_progress_broker.py`: **2 passed**, encode-only; [log](m3-validation/m3-progress-v2-python-tests.log).
- `cargo +1.90.0 clippy -p tsr-batch -p tsr-cli -p tsr-eval --all-targets --offline -- -D warnings`: passed; [log](m3-validation/m3-progress-v2-clippy.log).
- `cargo +1.90.0 fmt --all -- --check` and `git diff --check`: passed.
- Fail-first evidence: [v1 progress](m3-validation/m3-progress-v2-fail-first.log), [directory freshness](m3-validation/m3-progress-v2-directory-fail-first.log).
- All **72 original smoke freeze hashes** and the exact **98-file export set/hashes** remain unchanged.

Boundary coverage includes tests changing source, empty-directory changes, excessive empty directories, missing or corrupt CAS, missing raw capture, persistence failure, timeout/nonzero exit, incomplete suite output, contradictory summary, stdout cap and cap-minus-1/2/3, stderr cap, and exit-0 build diagnostics.

The new executable/build identity is recorded separately in `m3-progress-v2-build.json`. M3 operational acceptance is still unmet: this offline change provides no new real-corpus outcomes or operational reviewed-tension evidence and does not demonstrate that a live model will stop reliably. No paid run, deletion of existing artifacts, push, merge, or publication occurred. Existing unrelated files were preserved.
