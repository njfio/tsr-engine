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
  fresh installation. Existing local project histories remain untouched.
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

Integration verification is in progress. Initial focused Python execution passed
51 tests with 38 explicit native opt-ins skipped. Those opt-ins and historical
archive-dependent tests are not silently counted as passing. The initial full Rust
run exposed local loopback-permission failures; affected checks will be rerun with
the required local test permissions. Initial UI tests exposed private fixture
dependencies; those failures are retained locally and are being corrected with
authored fixtures. Final results and CI links will be recorded before merge.

Historical raw-run Python audits (for example M3 failure-contract and fixed12
archive regressions) require their preserved local archives. They are not part of
the portable authored CI subset. No historical assertion or frozen fixture is
weakened to obtain a public-checkout pass.
