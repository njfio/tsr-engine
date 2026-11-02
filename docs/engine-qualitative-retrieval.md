# Local retrieval feedback into qualitative state

This prospective interface uses the existing native store/review fold and extracts the existing broker's bounded file reader. It creates no selector, utility, proxy fulfillment aim or scenario rule. No historical all-visible state is changed.

## Local path

1. Initialize a fresh qualitative store with parents and **no visible record bodies**.
2. `configure-retrieval` admits an immutable local corpus manifest bound to the current root: canonical directory, unique record IDs/relative paths, native byte digests, attribution and uncertainty. This operator step checks every file but does not expose bodies in the initial request.
3. `request-v2` supplies a versioned index, corpus digest and typed retrieval request contract. A requested list of 1–4 record IDs uses the existing bounded reader; arbitrary commands, queries and caller paths are absent. The local manifest is read authority, never provider-export authority.
4. `retrieve` performs actual reads and appends a Rules-origin receipt binding the exact request/root, corpus digest, IDs, bytes/digests, attribution and current parent context. Replayed roots fail; no externally supplied receipt can be imported. Each file is limited to 64 KiB; each state to 32 receipts.
5. `request-v2` exposes retrieved bytes separately from initial context and model interpretation, with `retrieval_observation` basis. Ordinary reflection uses existing preview, quarantine and independent review; admitted possibilities/relationships then persist through the existing path. The public attachment carries retrieval provenance alongside reviewed memory.

A receipt proves access/content identity only. It does not verify fictional assertions, satisfy desires or constitute empirical consequences of pursuing them. Unsupported semantic claims still need independent review; rejecting forged typed fields is not a semantic truth detector.

```sh
target/debug/tsr-qualitative STATE configure-retrieval CONFIG.json
target/debug/tsr-qualitative STATE request-v2
target/debug/tsr-qualitative STATE retrieve REQUEST.json
target/debug/tsr-qualitative STATE request-v2
# Existing preview / submit or submit-possibility / review commands follow.
```

CONFIG contains `state_root` and `manifest` (`root`, `records`). Each record has `id`, `path`, native `digest`, `attribution`, `uncertainty`. REQUEST contains exact `state_root`, `corpus`, `ids`. These are local APIs, not an unattended paid runner. No live/model-selected retrieval has been demonstrated.

## Boundaries and review repairs

The shared Unix reader uses descriptor-relative opens with NOFOLLOW for every root/path component, requires a regular final file, and bounds bytes. Its independently admitted root must be canonical. The broker canonicalizes its copy before calling; retrieval uses its pinned canonical root. Non-Unix inspection fails closed rather than falling back to the old path-check/open race. The already-installed rustix 1.1.5 filesystem primitive is reused; no installation/download or OS setting changed.

A changed parent meaning/status invalidates the receipt as a current reflection basis. The prospective view explicitly says `retrieval_context_stale`, with null basis, while retaining historical access records. It does not automatically reread. Explicit reinspection or stop is required; this conservative limitation is not a claim that historical bytes became false.

Initial-context records cannot be retrofitted into an index-only experiment. Old export helpers and completed trial stopping rules remain untouched. Full native receipt history can exceed provider limits and is **not** an approved export projection.

## Prospective fictional packet and future scope

`target/qualitative-retrieval-preparation-1/index-plan.json` and `corpus/` contain a local-only prospective packet derived from the exact frozen source `9e6205e264e716c6dc3b2864a8125c482e5955d1216cd4d08db86a45aeebed9a`. All six bodies are uniformly withheld; index topics preserve source labels. SHA-256 file pins are preparation evidence, not substitutes for native content hashes. No campaign store, retrieval or generated adoption was created. Native manifest/admission and bounded provider projection remain prerequisites.

A future concrete approval would cover at most four Luna calls (choose IDs; reflect from actual retrieved bytes; independent review if development/persistence warranted; next proposal preview), one counted local corpus inspection of at most four records, zero code execution/real-world action, and reviewed advisory updates only in a fresh isolated store. Stop on abstention, invalid/unknown outcome or absent justified development; no mandatory four-call trajectory. All effects count under existing totals, not a reopened grant. This scope is **not currently launched or externally admitted**.

Current campaign remains 192/200 calls, 57/60 groups, $0.161431/$5. Temporary-file unit tests exercise a real read and authored interpretation/review; they are wiring evidence, not model behavior, campaign target executions or emergence.

## Verification and independent review

Offline installed Rust 1.90: initial qualitative group 14 passed; after review fixes the retrieval subgroup 4 passed (including parent-development→explicit stale basis), broker-v2 group 14 passed, attachment group 5 passed; both binaries built. Legacy Python export group 4 passed against rebuilt binaries. No full workspace suite claimed. Tests use temporary files and authored interpretations, not campaign target execution or live model evidence.

Independent reviewer `/root/retrieval_review` identified the inherited check/open race and unusable post-development basis; both were repaired and re-reviewed with no remaining blocking finding. Non-Unix fail-closed behavior and conservative reinspection requirement are explicit compatibility limits. Future export still needs a bounded projection: full native byte-array history is unsuitable for direct provider transmission.

Prepared index-plan file SHA-256: `13fab12fe5d392232ef6f8c1a80e36759afe04da7b73e2c313e9d558e837d3ac`.
