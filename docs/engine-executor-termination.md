# Executor termination evidence

The verification-only episode remains closed and rejected for incomplete coverage.
No new candidate execution, inference or limit increase was made in this diagnosis.

## Preserved historical evidence

`target/verification-only-episode/runtime-events.jsonl` contains bounded Docker
lifecycle events read after the run. Worker
`tsr-m3-44019-3-1792656323838977000` started at Unix second 1792656325;
at 1792656381 Docker recorded kill signal 9, die exit 137, then destroy.
The controller receipt reports exit 125 after 57,762 ms with compilation-only
stderr. Startup succeeded; this was not normal compiler completion or the admitted
600-second timeout. Events contain no OOM event for this worker. They do not prove
who initiated the kill or exclude every resource failure.

The old controller has two forced-stop branches yielding 125: process-wait failure
and scratch guard refusal. A transient directory-read failure during compilation
is a plausible guard trigger, not an established cause. Retained scratch is below
size/entry limits, without nonregular entries; post-run inspection cannot establish
what existed during the check. Do not silently relax the guard or retry this closed
episode. Original acceptance and resource receipts are unchanged.

## Reporting correction

Sealed results now include optional versioned controller diagnostics: termination
reason, observed runtime-client exit/signal, worker identity and stdout/stderr
completeness. Scratch refusal distinguishes directory/entry read failure, byte
limit, nonregular entry and combined entry-limit/symlink condition. Runtime-client
status is not represented as container exit status. A runtime exit 125 remains
ambiguous about the workload without independent runtime evidence.

The compatibility exit retains its fail-closed meaning. Output capture is bounded
and preserves already-read bytes even when EOF misses the collection deadline;
incomplete capture never becomes successful oracle evidence. Completeness false
alone does not distinguish truncation, read error and deadline. Legacy results have
unknown diagnostics, not fabricated reconstructed causes. Duration accounting and
reservation reconciliation are unchanged; CLI regression tests exercise them.

Tests cover ordinary/nonzero/reserved exits, signal termination, timeout, startup
failure, bounded output, delayed EOF preserving partial bytes, serialization and
legacy unknowns. Independent review found the partial-prefix and discarded-wait
status gaps; both were fixed and re-reviewed without a remaining blocker.

## Next action

The historical execution cause is unresolved, so the condition for another live
verification (a concrete cause corrected) has not been met. A newly authorized
instrumented diagnostic execution would resolve the remaining ambiguity; it must
be separately admitted against snapshot
`0e538537bb11d61da5de07c3919c9c05521c86bf38d2783a784f297a7bb05632`,
with no edits/inference, explicit budget and no reopening/relabeling of prior runs.
No candidate acceptance, mature-data eligibility or whole-engine completion follows
from this reporting correction.

## Separately admitted diagnostic execution

The parent subsequently authorized one instrumented run. Its admission, unchanged
323-file inventory, monitor, metrics and terminal receipts are preserved under
`target/verification-diagnostic/`. The bounded foreground monitor was independently
reviewed before dispatch: runtime reads restricted to observed worker names,
streamed capture bounded to 32 KiB/2 KiB with completeness/truncation flags,
8-second read deadlines and a 3,000-second monitor deadline. No daemon/security
configuration changed. Candidate memory/CPU/process limits stayed 2 GiB/2/256;
Cargo jobs and test threads were already one. No model or edit allowance.

Named test passed in 111,588 ms. Full suite terminated in 32,635 ms with
`scratch_directory_read_error`; runtime-client signal 9, complete captured streams,
and Docker kill/die 137 establish a controller scratch-guard stop. Sampled memory
peaked at 351.7 MiB, and sampled states report OOMKilled=false. Sampling cannot
establish unsampled peaks; no OOM conclusion is needed to explain this guard stop.
The original OS error subtype is not recorded and must remain unknown.

A deterministic race test demonstrated the guard rejecting a queued child that
vanishes before read_dir. The correction tolerates only NotFound for children;
missing root, permissions, other reads, symlinks and existing numeric limits still
refuse. Metadata failures now skip only NotFound rather than every error. Review
found no blocker. This is a general race correction, not proof that this historical
error was NotFound or that the full suite now passes.

All candidate hashes remain unchanged. The diagnostic episode is closed with
promotion rejected for incomplete coverage. Tool charge including preparation is
151,259 ms; model calls/tokens, patch bytes and external charge are zero; unresolved
requests zero. No second execution was dispatched. Prior closures remain immutable.
Checks: corpus 29 passed/1 ignored; CLI library 139 passed/3 ignored; Clippy with
warnings denied passed. No protected tests changed.

Next exact action: separately admit one unchanged-candidate verification using the
corrected guard, retaining current limits and instrumentation. The single diagnostic
allowance is consumed; do not reuse its closure or imply acceptance before full
suite and independent coverage review succeed.

## Corrected-guard verification

One further run was expressly admitted against the same 323-file snapshot, with
zero edits/inference and unchanged limits. Evidence is in
`target/verification-guard-fixed/`; `binary-sha256.txt` identifies the instrumented
binary. Guard I/O diagnostics now include ErrorKind and raw OS code for directory,
entry and metadata failures without exposing paths. Historical evidence is unchanged.

The named test passed in 112,092 ms. The full workspace command completed normally
with runtime exit 101 in 487,846 ms, complete stdout/stderr and no guard stop.
Its 96 result groups total 996 passed, 3 failed and 15 ignored (not unique corpus
tasks or independent episodes). The failures were:

- rts-mcp public_api: nested rustdoc attempted output beneath read-only source.
- rust_tree_sitter public_api: same read-only output failure.
- tool_surface::an_unknown_tool_name_fails_loudly_at_startup: missing XDG_RUNTIME_DIR
  caused daemon connection to fail before Surface::parse was reached.

The independently reviewed environment correction sets CARGO_TARGET_DIR to the
already mounted /tsr-build, CARGO_BUILD_JOBS=1, and private worker-local 0700 runtime/
state directories for scratch-backed runners without preparation. No source mount,
network, memory, CPU, process or security limit changes. Candidate argv remains
separate positional arguments. No project-specific daemon setting is injected;
the target already resolves a sibling binary. Public-API tests also request an
installed toolchain: this correction is not proof that all offline prerequisites
are satisfied. The new environment has not been live-verified.

The episode is closed rejected; full-suite success was not established. All 323
hashes match. Total charged tool time including preparation is 610,337 ms; model
calls/tokens, patch bytes, external cost and unresolved requests are zero. Active
review effort remains unknown; no maturity or training eligibility is claimed.
The optional-hook exclusion and ignored tests remain disclosed, not counted passed.

Checks on the correction: corpus 30 passed/1 ignored, CLI library 139 passed/3 ignored,
Clippy warnings denied passed, independent review completed. No protected tests or
original target files changed. Next action requires a fresh admission to verify the
worker-environment correction; this run's execution allowance is consumed. Do not
reopen its closure or silently retry it.

## Offline preflight and accepted verification

The three prior failures were `rts-mcp --test public_api`,
`rust_tree_sitter --test public_api`, and
`rts-mcp --test tool_surface::an_unknown_tool_name_fails_loudly_at_startup`.
The separately authorized offline preflight completed in 1,713 ms, exit 0:
nightly-2025-08-02 Cargo/rustdoc were already installed; runtime directory mode was
0700 (501:20); locked offline Cargo metadata targeted /tsr-build. No download or
network change was needed. Evidence: `target/verification-preflight/`.

That satisfied the conditional admission for one new verification using 6fc31b4.
Evidence: `target/verification-environment-fixed/`. Named test passed in 111,752 ms;
full workspace completed in 568,846 ms with exit 0, complete captured streams,
96 groups, **999 passed, 0 failed, 15 ignored**. Both public-API checks and the
unknown-tool startup test now passed. All 323 source hashes remained unchanged.
The optional hook-marker early return and 15 ignored tests are explicit exclusions;
the count is test-result totals, not independent episodes or a superiority measure.

Independent semantic and coverage review confirmed the exact original generated
replacement, list trimming/empty rejection, unknown-name rejection, observability
exceptions, profiles and case behavior. The operator recorded scoped acceptance
against snapshot `0e538537bb11d61da5de07c3919c9c05521c86bf38d2783a784f297a7bb05632`
and receipt `b2c74da0242eff70052d53ea2ec6abaa8959a32e748acce25a055c27d301ecc6`.
`review-result.json`, `independent-review.txt` and `accepted-export.json` retain
these bindings. The built-in verification-only export has an empty patch because
its opening source already contains the repair. `accepted-original-repair.patch`
is a mechanical diff from the original admitted mutant to the same accepted
candidate; its JSON sidecar binds source/proposal/patch hashes. It was not applied
to the original target or published.

Episode closed at 2026-10-22T09:06:40.395Z. Charged tool time, including preparation,
was 684,828 ms; preflight separately cost 1,713 ms. Model calls/tokens, patch charges,
external cost and unresolved requests were zero. Diagnostic-monitor host resources
and active reviewer effort are not fully measured; no total-resource claim is made.
Earliest calendar maturity is 2026-10-29T09:06:40.395Z, but replay origin and unknown
coverage prevent treating this as independent mature training evidence even then.
All prior rejections remain immutable. No extra execution, protected-test changes,
original-target changes, downloads, credentials, publication or persistent services.

The exact repair verification objective is met within the admitted default suite
and independent review. This does not establish general provider reliability,
statistical M3 acceptance, calibrated Tier2 support or unattended semantic review.
