# Browser project with scoped source access — prospective only

No live run is authorized by this packet. Historical attempts and campaign ledger
remain unchanged. This packet contains a blank three-file scaffold, not a game.

## Task and interface

The actual engine chooses an original maze-chase, falling-block or combined game
from the user's open brief. The editable paths remain index.html, style.css and
app.js. No preferred design, relationship revision or player opinion is supplied.
The generator may propose untested designs, inspect admitted editable source, edit
incrementally, retain its design, or abstain. Native preview and independent
semantic review precede execution. Protected browser tests are withheld.

The browser no longer inherits the Rust 2048-byte span/4096-byte aggregate limits,
or the earlier browser 8192-byte file limit. Rust/default behavior stays unchanged.
No per-edit size gate is added beyond the bounded serialized native response and
resource envelope. Exact path, source hash, unique span, protected files and review
remain mandatory; no truncation or automatic code rewriting occurs.

Large files are represented by an index and explicit UTF-8 byte windows. The
model can request `{inspect:{path,offset}}` for an admitted editable file. Requests
are counted against the same model-call envelope. Inspection does not create
execution authority or access outside the project. Omitted bytes are marked;
reviewers must report unknown when unseen code prevents assessment. Exact edits
are checked against the complete local source and applied in the sealed writable
edit phase by a fixed controller script, not generated shell commands. Only exact
spans cross process arguments, not complete reconstructed files.

## Real finite resource boundaries

This is not unlimited computation. Provider requests remain 32 KiB, completions
8192 tokens and 240 seconds per request. Native browser response parsing allows
128 KiB and advance wrappers 256 KiB; local introspection has an 8 MiB envelope.
The existing 1 MiB complete project-snapshot budget bounds local memory and disk
work; it includes protected files. This is a configurable software resource budget,
not an immutable hardware limit. The previous 64 KiB shared text executor setting
was likewise software: browser validation now explicitly uses the snapshot budget,
while other callers retain their default. File views are transport windows, not
limits on the complete project. Maxima cannot all be combined; full requests are
validated, never silently clipped. Long history may still exhaust request space.

At most four files per edit set and eight admitted editable files are supported by
the existing core; this packet retains its three authorized files. New paths need
prospective admission. The cumulative patch resource budget is 8 MiB for up to eight
executions, not the obsolete 8 KiB budget. Exact writes send spans in UTF-8-safe
argument chunks. Host/process resource failures still stop without uncertain replay.

## Proposed cohesive first-playable session

CheaperInference gpt-6-luna; up to **26 total calls** shared by generation, source
inspection, independent reviews and at most **two format/known-settled size
regenerations**; up to **eight isolated execution groups**; **120 minutes**;
**$1.30 conservative reservation**, below the user's $10/run authorization and
within the currently stricter $5 campaign budget. No provider-failure retry or
automatic second run. Stop on first reviewed functional checkpoint, abstention,
semantic rejection, unknown effects or exhausted resources. User play/change
requests come afterward with genuine attribution, never fabricated feedback.

Baseline is 262 calls /72 groups/$0.225523, no unknowns. The proposal would need
cumulative caps of 288 calls/80 groups, versus current 272/74. **These increases
are not authorized or implemented.** Automatic approval review rejected a proposed
local addition of that grant profile before it executed. No ledger change occurred.
A reviewed cap grant and its explicit implementation remain a live-launch blocker.
The local packet is usable for authored verification; do not present it as admitted
for provider dispatch. No previous closed allowance may be reused.

Same pinned image sha256:58deb9d12b7291f6d2c697b335eb927557d707488df09d1503659d7abdd00883,
UID/GID501:20, no network/pull, read-only root and browser workspace, dropped
capabilities, no-new-privileges and restricted mounts/resources. Only the previously
approved Chromium internal sandbox exception remains. The controller's exact edit
phase retains its preexisting scoped writable mount. No host security change.

Export remains the public brief/wants, admitted fictional editable source windows,
exact candidate edits, bounded observed browser facts and reviewed advisory history.
No private/full state, protected test source, credentials or outside files. The
same content categories apply, although larger projects use more counted requests.

## Local preparation and delivery

Run `python3 experiments/browser-game-v3/prepare.py --output target/NAME
--campaign-manifest PATH --packet-sha256 HASH` as one command. Preparation writes
isolated native state/admission templates only. It does not apply a campaign grant,
reserve money, call a model or run project code. The protected controller test
interface is only a smoke check; it does not establish gameplay quality.

If a future admitted run produces a functional game, its assets are under
`PREPARED/native/coding-work/`. Independently review and deliver only the three
editable assets, excluding protected tests and native state. No public deployment
or host execution is included. Keep persistent state and exact source/receipt
identities for genuine attributed user feedback and later separately admitted work.
Do not call authored harness fixtures engine-produced games or TSR emergence.
