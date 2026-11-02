# November 2026 engine integration

This public integration combines the corrected engine at
`5b3706ea` (`codex/continuous-development-audit`) with the original line's
additional completion report at `b15f36b1` (`codex/m3-evidence-pipeline`). The
integration starts at public `main` commit `8b74019f`. Both original local
branches, worktrees, commits and raw evidence remain unchanged.

## Why this is a sanitized snapshot

The unpublished history contains captured execution archives, SQLite databases,
content-addressed runtime payloads and compiled binaries. Merely deleting those
from a later commit would still publish them through ancestry. Neither original
branch is therefore a parent of this public integration commit. Code and selected
reports are transferred into a fresh branch; the source manifest records their
identities. This does not claim that the private historical objects were pushed.

The export retained 1,872 code, fixture and report files, including the original
line's seven additional summary files. It excluded 24,075 paths: raw archives,
logs/binaries and delivered game artifacts. Authored browser test examples are
fixtures, not the delivered games. Raw `target/` journals and unrelated untracked
files were never included. Required synthetic archive/workshop inputs and frozen
M0–M4 fixtures remain available. Historical reports may refer to local evidence
that is intentionally not in this public checkout.

A non-printing pattern scan covered 12,753 unique unpublished blobs, approximately
244 MB. It found no matching provider/GitHub/cloud credentials; a private-key
marker was an authored redaction negative test. This is bounded scanning, not
proof that arbitrary runtime archives are public-safe. Independent review also
checked the selected tree and critical authority/review boundaries.

## Current engine and limits

The native engine supports independent component review, optional provisional
wanting alongside artifact edits or retention, source-bound return interpretation,
scoped success/Contrast routes, preserved contributor ancestry, durable recovery,
Stop and explicit resource policies. Development is optional; execution authority
never follows automatically from a new meaning or desire.

The latest live test completed with 15 calls, 20 execution groups and $0.073252.
It produced and repaired an artifact, consumed bounded observations and retained
artifact/meaning. It did not produce a formal relationship or new Preference.
Mapping and stage preparation were assisted. See
[the exact result and limitations](continuous-game-live-v2-result.md).

## Integration-only changes

- Studio no longer seeds genuine historical user messages or private projects on
  fresh installation. Existing local project histories remain untouched. Empty
  startup and stale selections preserve drafts and show the new-project flow.
- New-project initialization uses the same browser scope, protected harness,
  controls and probe from public source instead of reading a historical game's
  private scope file. This initializes state; it grants no paid execution rights.
- UI verification creates labeled authored native fixtures instead of depending
  on excluded Pond/Jungle runtime and game files.
- Current README guidance separates historical status from current capabilities.
- Rustfmt normalizes 32 existing Rust files; frozen M0 sources/fixtures remain
  byte-identical. Original and integration hashes are both recorded.
- CI runs Rust workspace, focused contract/recovery, selected native transport and
  UI tests without provider calls. Its container has no nested Docker runtime;
  runtime-dependent tests report their existing unavailable/opt-in behavior.

A fresh checkout still needs operator-provisioned campaign/runtime/admission
state before paid Studio Send. Unlimited mode is an explicit admitted policy, not
an automatic grant to new users. No credentials or campaign approvals are shipped
as executable authority for a new installation.

## Verification status

Local integration verification completed:

- Rust workspace: initial run reported 1,138 passes, eight environment-dependent
  failures and 13 ignored tests. The five loopback failures and three container
  failures all passed in affected-target reruns with required permissions:
  10 reviewer tests, one exec-status test and 14 harness tests. Counts overlap the
  initial run; they are not added as distinct tests. Failures remain recorded.
- Focused Python contract/recovery: 51 passed, 38 explicit native opt-ins skipped.
- Selected native/browser opt-ins: four passed, including recursive provisional
  identity, interrupted component admission, edit-plus-development, and an
  independent artifact after development-review abstention. Transport is authored;
  two cases execute the existing isolated browser runtime. No provider calls.
- Portable UI: 88 passed, four preserved-history opt-ins skipped. Two targeted
  history/concurrency checks also passed (overlapping suite coverage).
- Fresh-install HTTP tests pass. Actual networkless Chromium fixture checks
  cover empty startup, no nonexistent historical-project request, polling/reload
  draft preservation, stale-selection recovery preserving both drafts, and no
  JavaScript/load errors. These are authored fixtures, not live model evidence.
- Rustfmt and diff whitespace checks pass. Frozen M0 sources/fixtures remain
  byte-identical. No fresh Clippy or exhaustive security-proof claim is made.

The initial UI failure revealed a hidden dependency on a real game; authored
fixtures now provide the intended native state and explicit queued-input
concurrency condition. A fresh-install directory omission and stale-selection
frontend issue were caught and corrected before merge. No test assertion was
weakened to accept a failing engine result.

Independent integration and focused security reviews passed. The PR is
[GitHub #7](https://github.com/njfio/tsr-engine/pull/7); its latest-commit checks
must pass before merge. Sanitized local verification hashes are recorded in
`integration-verification.json`. Raw logs remain local, not published.
Historical raw-run Python audits (for example M3 failure-contract and fixed12
archive regressions) require their preserved local archives. They are not part of
the portable authored CI subset. No historical assertion or frozen fixture is
weakened to obtain a public-checkout pass.

Initial GitHub CI run 41467202871 failed in Rust tests. Python-backed fixture
transports did not produce their reports, and two D4/R1 tests collided on a
shared default SQLite store. The underlying Python loader stderr was not retained,
so a specific loader error is not proven. CI now uses the existing Debian Rust
image's Python instead of the external setup-python toolcache, with an isolated
startup/import and decoder preflight that removes LD_LIBRARY_PATH. That image
passes the same decoder locally. D4/R1 tests now select their own scratch stores;
assertions and production locking remain unchanged; all four targeted D4/R1 tests
pass locally (2.84 seconds), and rustfmt/diff checks pass. The failed CI is preserved,
and a new exact-head run must pass before merge.

Replacement CI 41470974098 passed the complete Rust workspace, then the authored
Python admission fixture rejected target/debug/tsr at the existing 128 MiB hashing
bound. CI now uses limited debug information (CARGO_PROFILE_DEV_DEBUG=1) throughout
the job and explicitly checks binary size after build. Optimization level, debug
assertions, overflow checks and production admission bounds are unchanged. This
build-only adjustment received independent review; final CI remains the merge gate.

CI 41473223515 passed Rust, binary size validation, contract/recovery and native
development/restart checks. Its UI suite had one failure: the stale-guidance
negative test stopped at missing Docker preflight rather than reaching the
asserted guidance rejection. The fixture now supplies an explicitly authored
runtime preflight and asserts no worker launch, while keeping real native source
mutation and the exact guidance-rejection assertion. Production runtime checks
are unchanged. This failure remains retained; a fresh CI run verifies the patch.
The corrected exact UI case passed locally (1/1, 2.929 seconds), with independent
fixture review and no production edits.
