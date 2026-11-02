# Objective continuation boundary

The released finite-objective path previously could not resume a completed service
whose review hit a provider failure. Its origin required round zero; a fresh adapter
rejected historical receipts as foreign, while the old cycle correctly refused its
pending callback. Billing reconciliation did not reopen that closed grant.

`engine-objective-continuation.py` prepares a separate derivative with a verified
historical prefix and fresh remaining-only resource admission. The shared objective
adapter's `inherited()` projection hook carries historical source, relationship,
review and receipt provenance; the ordinary `Cycle` begins at the missing review
with an empty current journal. No selection algorithm or task-specific strategy is
added. Historical operations never count as new effects or get scheduled again.

The old native resource store is archived byte-for-byte in the derivative only,
after exact proof that the sole unknown is the terminal failed model request.
Unrelated unknown compute/tool/model effects block preparation. The original state,
its pending callback, closed grant and all ledger rows remain untouched.

Current adapters enforce their explicit operation bounds. A new retry hook is off
by default. The continuation alone can admit one confirmed-terminal HTTP 503 retry,
using an exact authenticated provider lookup and preserving the original receipt.
The lookup has a total deadline, not only a socket timeout. Unknown effects stop;
semantic decisions and identity violations cannot enter this retry path.

The proposed packet is `experiments/open-guide-continuation-v1/manifest.json` and
its README describes the complete prospective envelope and commands. It requires
new explicit live authorization; previous unused allowances remain expired.

## Verification

Results and independent review are recorded in the packet verification artifact.
The integration uses real native persistence with explicitly authored transport,
review judgments and cfg(test) worker receipts. It performs no provider requests,
container executions or generated target execution. Such checks are not behavioral
TSR evidence or live success.

Original evidence remains `304e6aab`, `docs/engine-open-guide-live-1.md` and
`target/open-guide-live-1`. Campaign baseline remains 254 calls, 71 groups and
$0.217356 settled, with no new campaign effects in this local work.
