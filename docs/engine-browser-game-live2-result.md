# Corrected initial browser-game run: no admitted game

The engine proposed a maze-chase design, but validation stopped the run before
review, edits or browser execution. There is no playable artifact. This is a
protocol/admission result, not evidence about gameplay, satisfaction or TSR
relationship development. Frozen first attempt `7d98537a` remains unchanged.

The fresh run used source `363a9b1e`, browser-game-v2 packet SHA-256
`d6e0b4bae6e95f99b271d22e9c706171c4d16520f4c3afe09d6a5d53b0bc800d`,
and isolated root `target/browser-game-live2`. Independent launch review verified
source/binary/runtime pins, public projection, current ledger prefix and exact
scope. Official pricing was refreshed. Launch was 2026-10-29 01:16:39 UTC;
admission deadline was 03:14:50 UTC. The process returned exit 1 after the second
response; no automatic further attempt was made.

## Actual trace

1. The model offered an explicitly untested maze-chase design: keyboard controls,
   collectibles, a moving opponent and win/loss status. It no longer required
   prior player feedback or native preview before proposing. This difference
   does not prove the instruction correction caused it.
2. Its response placed `edits` and `rationale` at the top level rather than inside
   the required `alternatives` array. All three proposed spans were unique and
   within 2048 bytes, but the envelope was invalid. One new counted format
   correction was issued; the original output was not rewritten.
3. The corrected response supplied the envelope. Its `app.js` replacement was
   2150 bytes, exceeding the unchanged 2048-byte per-span limit by 102 bytes.
   The HTML/CSS replacements were 539/506 bytes; all old spans matched uniquely.
   The browser validator's combined error was `exact unique changed span required`.
4. The shared correction classifier did not classify that ValueError as a known
   settled format defect. It recorded a terminal output failure and expired unused
   allowances. The proposed code was neither trimmed nor admitted. There was no
   semantic review, native selection, execution, reflection or acceptance.

Both raw responses and bounded diagnostic metadata remain quarantined locally
under `adapter/output-{0,1}/failed-response-1/`. Provider bodies are not committed.
The first candidate is not a fallback: it remains invalid and was never adopted.

## Accounting and preservation

Two HTTP-200 requests settled at $0.001145 and $0.001172, totaling **$0.002317**.
There were **2 calls, 1 format correction, 0 executions, 0 provider retries**.
Usage: 5124 input tokens (2506 cached), 5476 output tokens, 10600 total. Reasoning
usage is included in output tokens. No unknown charges or actions remain.
Campaign totals: **262 calls / 72 operations / $0.225523**. Caps remain 272/74/$5;
the user's $10-per-run approval did not expand call or execution limits. Remaining
capacity is not a new run grant.

All 42 files in the prior live1 evidence tree retain their hashes. Editable blank
scaffold and protected harness/probe are unchanged. A fresh-process replay refused
the pending cycle proposal intent and dispatched nothing; the ledger hash stayed
`cc6d38f6e946a5817dfd25e900addbbb69d43ca9f6e175dda04245eaf4353ecb`.
This is safe refusal, **not clean terminal cycle replay**. The local pending intent
is distinct from provider uncertainty: both provider responses and bills are known.

## Remaining boundary

A general future correction should make the browser span-size error precise and
assess whether known-settled size regeneration belongs in a prospectively admitted
correction policy. It must preserve exact matching, the 2048/4096 limits, source
binding, counted fresh requests and independent review; it must not reinterpret
this closed run or silently use its remaining allowance. The top-level envelope
is already explicit, but the live model still violated it. No claim that invalid
output is impossible is justified. No further implementation or run was performed
as part of this result recording.

Evidence index: `experiments/browser-game-live2-result/manifest.json`. Detailed
receipts, launch, preservation, terminal and replay evidence remain in the isolated
run directory. No deployment, dependency installation, security change or host
execution of generated code occurred.

Independent read-only result review by `/root/binding_review` confirmed the exact
envelope/size diagnosis, settlement, unchanged assets/history and the distinction
between safe restart refusal and clean terminal replay. The review performed no
model calls, executions or evidence mutations.
