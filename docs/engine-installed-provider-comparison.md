# Installed-provider comparison and validation evidence

## Scope and interface

The explicit grant combined one unused slot with five new local calls: six further
calls total, each at most 240 seconds, up to three two-stage attempts. The original
journal retains its initial model and budget. `--provider-comparison tools-v1`
adds an explicit comparison record pinned by `--provider-digest`; endpoint and
initial call limit remain frozen. A pending possibility set cannot switch its
originating model/digest before assessment. Uncertain started requests still block
mutation and dispatch.

`--carry-remaining` explicitly adds only the new grant, preserving every consumed
and unused slot. In this run the prior limit was 25, with 24 consumed. Five new
slots increased the limit to 30; no old call was refunded or overwritten. Named
grants and provider configurations are idempotent and frozen across restarts.

Command used (the search is now exhausted; do not rename it to repeat trials):

```sh
python3 scripts/engine-directions.py --directory target/support-live/run \
  --model hhao/qwen2.5-coder-tools:latest --endpoint http://127.0.0.1:11435 \
  --calls 4 --batch installed-provider-v1 --batch-calls 5 --carry-remaining \
  --provider-comparison tools-v1 \
  --provider-digest b7fade0651b448e0e57af67d9edcbdcb686cd8a9647c2cb1f123858438af0a33 \
  --two-stage --possibility-search tools-possibilities-v1 \
  --possibility-temperature 0.7 --possibility-seed 7304 --possibility-samples 3 \
  --stop-file target/support-live/recovery.stop
```

The installed alternative is Qwen 2.5 Coder Tools, 7.6B Q4_K_M, served locally by
Ollama. Possibility seeds were 7304–7306. Assessment would remain temperature zero
with the same immutable-action, authority, support, exact-effect and standing-loss
rules. No assessment was reached. No download, external call or authored candidate
was used. The state and desires were identical to the prior comparison; request
metadata additionally exposed the zero older-offer omission count introduced after
request24. The operator conflict certificate remained excluded from model context.

## Observed result

All three samples returned explicit inability:

| Request | Envelope file | Inability SHA-256 |
| --- | --- | --- |
| 25 | request-25-e44fabcc1c89.json | 3b23d28fcf30fc260a8a8557217f5f2ce293cea6f43ffc50265ad5233c6ff4e7 |
| 26 | request-26-63000ea425f8.json | c937857c97384420a23be1b27531858b520c60e6234e11991bd9df2b4acbb9f3 |
| 27 | request-27-f902535ed7b7.json | 4005ed2a1d96f22fcce575bc2b8e61996c3a1509e438e9875e6ca5ea668a33f7 |

Files are under `target/support-live/run`. Terminal output is
`target/support-live/installed-comparison-live.log`; it reports
`possibility_search_exhausted` and the scoped `generation_limit` evidence.
No candidate was frozen, assessed or executed. Root remains
`3d736f20dfa9683eb7b1d6b0c7d1cf2589fdb4ccc16bf4950e01188f1bf51c4f`,
observation [0.5, 0.8, 0.0], with both original wants and the joint conflict intact.
Three of six slots were consumed, three remain unused. Temporary provider PID
19113 terminated, its session exited zero, and the listener was absent.
Unknown13, inability24 and all prior evidence remain intact.

This establishes that neither tested Qwen model/configuration found useful
possibilities in this preserved context. It does not establish general model
incapacity or impossibility of partial progress. The alternative's statements that
conflicting desires preclude alternatives are unverified generator assertions;
they do not override the independent partial-progress witness. Repeating these
searches unchanged is unjustified. Live generation remains the concrete functional
limit; no actual action return or end-to-end recovery is claimed.

## Separate core evidence correction

Inspection against PRD §B6's separation of possibilities, execution and learning
found that incomplete, tool-using, or malformed inner JSON responses could be
rejected before validation evidence was retained. After the comparison, the driver
was changed to save `provider-response-N.json` **metadata only** before inner-output
validation: canonical response/content hashes, byte counts, completion status,
recognized completion reason, content type and whether tool calls were present.
Response hashes and response byte counts describe canonical JSON serialization,
not original HTTP bytes; content counts/hash describe the decoded content string.
No raw provider body, tool arguments, credentials or source text are logged by this
new artifact. Metadata availability does not establish valid output or action success. Existing admitted/rejected candidate artifacts are unchanged.
Terminal receipts bind the metadata file by hash, including stop-before-apply.

If interrupted after this artifact is saved, explicit reconciliation records that
response evidence is available but deliberately unapplied. It still requires the
same controller root, no application receipt and exclusive driver lock; it does
not retry, refund, assert success or execute content. Outer malformed JSON and
transport failures remain outside this parsed-response metadata path; raw provider
bytes are not recoverable from the metadata. No historical response is invented.
The change does not affect forecast semantics or candidate acceptance.

## Checks

`python3 -m unittest discover -s scripts -p test_engine_directions.py`: 62 passing
(`target/provider-comparison-tests.log`). New checks cover explicit provider pins,
frozen comparison configuration, pending-stage protection, carry arithmetic,
incomplete/tool-using/invalid-inner-JSON evidence, no execution, and interruption
before validation. Python compilation and diff checks pass. Independent review
cleared the comparison interface and reviewed the evidence changes. No protected
tests, target repositories, credentials, publication or persistent access changed.
Corpus remains 151 qualified tasks; mature independent eligible episode count
remains unknown. No new calibrated or semantic-understanding claims are made.
