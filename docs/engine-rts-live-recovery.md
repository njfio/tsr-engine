# Explicit recovery of the approved RTS demonstration

The first settled decode failure is preserved in
[its original receipt](engine-rts-live-result.json). An explicit recovery record
binds that failed step to reservation `c89a26e9-a4c3-4356-b040-9459aa8c170d`;
no event, failure, baseline or spending limit was erased.

The recovery path requires one failed call, exactly one remaining step, unchanged
engine state, no committed/executed proposal, a verified settled decode diagnostic,
verified token usage and a settlement matching the run's cost delta. Other failure
and uncertain-charge stops remain terminal. Restart remembers the recovery record.

The original response exhausted its 2,048 completion tokens. Its exact failure
cannot be reconstructed because its raw body was not retained. The remaining
call used 4,096 completion tokens, with identical source/context and response
schema. The existing reservation calculation already bounds 16,384 output tokens;
the $0.24 reservation plus $0.002149 first-call charge stayed within $0.25. The
first call's 3,408 billed tokens are explicitly included in the 100,000-token cap.
This is a tested explicit recovery, not an automatic retry or budget reset.

The broker now records a bounded finish-reason enum, and the engine saves a
private (0600), no-overwrite response diagnostic outside the candidate workspace.
It contains bounded raw message content/refusal, usage, finish reason and only the
length of any reasoning content. The supplied credential is redacted before
writing; the JSON preview is limited to 16 KiB. No headers or credential file
contents are retained. Private provider content is not committed.

The second call completed normally (`finish_reason=stop`), used 1,705 tokens and
settled at **$0.000316**. Across both calls: **$0.002465**, **5,113 tokens**, no
uncertain reservations; shared aggregate **$0.620338**. Both authorized calls
are now consumed. No further call was issued.

One generated exact-edit proposal passed the Rust authority boundary, was committed
and executed in the sealed environment, and passed the genuine named test.
A restarted request contains its passing feedback. Transport restart returns
`success` without changing the billing ledger. The exported patch applies to the
original admitted mutant and reproduces the generated implementation. The full
workspace closeout uses that patch in a separate 479-file disposable tree, with
copied private build artifacts, unchanged tests and the existing 20-minute stage
bounds. The original repository is unchanged.

The full-suite run exited 0 in **375,166 ms**, after a **29,044 ms** workspace
build: **999 passed, 0 failed, 15 ignored**, no filtered tests, 96 groups. Its test
inventory matches the prior successful suite, and all 479 post-run source hashes
match the exported model-patch snapshot. This passing suite does not remove the
semantic limitation below.

## Semantic limitation

The generated list has only Rust and Python, with version values `1.0` and `3.0`.
The original implementation lists 13 languages and uses grammar-version metadata.
The generated repair is **not equivalent to the original implementation**. Named
test or full-suite success must not be described as complete metadata restoration,
production readiness or benchmark success. This demonstration reveals an oracle
coverage gap; no source correction was substituted for the model output.

## Local validation

Combined local checks: **43 passed, 9 existing optional Docker tests skipped**
(52 discovered). Loopback transport/recovery/readiness tests covered preservation of both
failure records, explicit recovery accounting, larger completion allowance, scope
rejection and no third call. Decoder, pricing/accounting, token limits, redaction,
private-file bounds and malformed-response checks also passed. Existing optional
Docker-dependent Python cases were skipped; the actual live sealed execution and
separate full-suite closeout supply real execution evidence.

See [structured live evidence](engine-rts-live-recovery-result.json) and
[full-suite evidence](engine-rts-model-full-suite-result.json). No raw provider body,
credential, original-repository change, publication or benchmark cohort is included.
