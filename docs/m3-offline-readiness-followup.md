# M3 offline readiness follow-up — 2026-10-15

M3 acceptance remains unmet: **44 strict admissions**, not 150; the live paired evidence remains the previously reported small, unsuccessful/censored cohorts. No additional provider calls, disclosures, or charges occurred. Aggregate reconciled spend remains $0.165094,78 settled calls,0 outstanding. Historical results and frozen evaluators are unchanged.

## Offline evaluator repair

`context_01` first failed on the old driver: two 60380-byte inspections exhausted the accumulating 64 KiB observation before a test. The new driver keeps only the latest inspection and test, omits replacement bodies from displayed action history, includes test ID/stdout and stderr digests/diagnostics, labels potentially stale observations, and marks head/tail truncation with original length and digest. Controller evidence still hashes full inspected bytes; policy prerequisites, grants, trusted test IDs and oracle protections are unchanged.

Independent review identified JSON escaping as a second bound. The driver now limits the actual serialized request to 69000 bytes before broker dispatch, including trusted metadata. Unicode/control-character regression passes. Native budget closures now identify generator calls/tokens, tool minutes, wall clock, patch bytes, observation bytes or request bytes. Historical scripted-driver closures retain their prior fallback.

`docs/m3-context-v3-freeze.json` freezes the separately built `target/m3-evaluator-context-v3`. It has **not been used for live evaluation**. Latest-only context can omit relevant earlier files; truncation is explicit. Files over64 KiB remain unsupported. The existing60-second transport timeout and conservative token reservation remain unchanged. This repair does not reclassify old outcomes or establish improved model performance.

## Local qualification continuation

Prepared two unique function candidates after deduplicating the old shortlist against existing cohort plans: `filter_error_map` and `PrebuiltGitignore::is_ignored`. The telemetry candidate is correlated with prior map-helper work. The combined theoretical ceiling is now150, with no margin for rejection; candidates are not admissions and do not establish independent diversity.

`docs/m3-filtering-extension-freeze.json` and `m3-filtering-extension-launch.json` freeze and record a finite boundary handoff. Outer coordinator29246 is intentionally stopped; core controller29252 and its two sealed workers retain their existing timeout supervision. Boundary controller36983 waits up to three hours for clean core completion, verifies frozen inputs and source identity, then screens and qualifies at most two candidates using the historical qualification engine. It resumes the original outer queue only after clean completion, no M3 containers remaining, exact process identity/stopped-state checks, at least 200 GiB free, and a live identity-matched disk watcher with a heartbeat no older than 30 seconds. Failures leave the outer queue paused with an exact reason in `target/m3-filtering-boundary-state/error.json`.

Disk watcher27230 (`target/m3-reserve-watch-v2`) checks every10 seconds and pauses only the original outer coordinator below 200 GiB. The superseded watcher99295 was retired after verifying its replacement. The200 GiB guard allows room for the active frozen cohort (largest 13 candidates, observed RTS allocation ~5.12 GiB per attempt) before the100 GiB reserve. This is a conservative cohort-boundary guard, not an OS disk quota. The extension independently refuses job starts below140 GiB. No third sealed worker is launched.

## Cleanup proposal — no deletion

`docs/m3-terminal-build-cleanup-v2-plan.json` rechecks the prior 30 accepted terminal `copy/target` directories. Every non-copy evidence file was checked byte-for-byte against the preserved admission export. Current allocation is **123971805184 bytes (115.457 GiB)**; logical file bytes total 143297946959. APFS shared extents can make actual reclaimed bytes smaller. Exact paths, inode/device identity and per-target sizes are in the manifest. Newer and active attempts are excluded conservatively. Rebuilding uses the preserved source/preparation in its locked, offline sealed container. Source copies, CAS and reports are retained.

**No cleanup was performed.** Approving that exact manifest at a verified idle boundary, or providing more storage, is needed to extend retention once the disk guard pauses the campaign. Evidence and identity must be rechecked immediately before any deletion.

## Validation

Captured outputs are in `docs/m3-validation/`:

- Fail-first `cargo +1.90.0 test -p tsr-batch context_01`:1 failed, expected; then fixed.
- Workspace suite with the two documented M0 runtime-dependent exclusions:711 passed,0 failed,5 ignored,2 filtered. This run preceded the final additional token-reason test.
- Final `cargo +1.90.0 test -p tsr-batch`:69 passed,0 failed,1 ignored; includes all 3 new regressions, captured stderr/digest/test ID assertions, and existing rules/prerequisite/oracle regressions.
- Workspace clippy with `-D warnings`, build, and fmt check:passed on 1.90.
- Inert boundary/watchdog tests:3 passed, no real signals or workers in tests.
- Existing inert cleanup verifier tests:6 passed, no deletion.
- Independent review: serialized-size and process-handoff findings repaired; final review found no remaining launch blocker. Lefthook was absent in test-created repositories; hooks are not claimed as passed.

Latest durable resource/worker snapshot: `docs/m3-readiness-followup-checkpoint.json`. Actual qualification completion must be read from the controller receipts; this document does not count pending tasks as accepted.
