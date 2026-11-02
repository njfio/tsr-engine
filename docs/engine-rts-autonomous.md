# Bounded autonomous RTS run preparation

The fresh engine is `target/engine-rts-autonomous-ready/engine`, initialized from
the admitted mutant, not the repaired replay. Offline preparation passed. It has
57 scoped files, one editable implementation window, no pending commitment and
no prior repair receipts. Original repository and protected tests are unchanged.
The goal includes the function signature and `LanguageInfo` declaration extracted
from the mutant, within the existing 512-byte goal bound. It contains no reference
correction. The next request is 3,511 bytes.

[Readiness evidence](engine-rts-autonomous-readiness.json) and
[bounded configuration](engine-rts-autonomous-run-config.json) are local records.
The preparation helper never invokes the provider, loads credentials, or executes
the generated live argv. Reproduce in a new output directory:

```sh
python3 scripts/prepare-rts-autonomous.py --out target/engine-rts-autonomous-new
```

The existing `tsr engine --directory DIR project run -- ...` entry point delegates
to the budgeted transport. Two gaps are fixed: Cargo dependency readiness is
checked before provider dispatch, and `coding-run` gets a controller timeout
computed from the sealed plan's command count and deadlines, plus 30 seconds.
For this three-command, 600-second-per-command plan that is 1,830 seconds, rather
than the prior fixed 180 seconds. The executor's deadlines remain unchanged.
Restart recovery, uncertain-charge stops, live-mode approval flag, shared aggregate
ledger, price checks and reservations remain in force.

Prepared bounds: at most 2 proposals, $0.25 run spending/reservations, 100,000
aggregate tokens and 55 seconds per provider call. Provider/model remain
`https://api.cheaperinference.com/v1/` and `deepseek-v4.1-flash`.
The $100 shared aggregate ceiling is unchanged. The checked ledger held $0.617873
and no uncertain reservations. The initial conservative reservation is $0.24;
reservation sizing can stop the loop before two proposals. These are snapshot
checks; the broker rechecks ledger and current prices at dispatch.

## Authorization and remaining live checks

The verified original-room authority includes the October 14 approval of $100
aggregate CheaperInference M3 testing and transmission of M3 task prompts,
relevant repository code and test output (potentially private), excluding
credentials and unrelated personal data. The October 18 follow-up approved the
scoped reconciler workflow and asked us to continue without repeated approval.
The current instruction limits **this preparation slice** to local work; that is
not a revocation of the existing provider/source/spending authority.

The next demonstration uses the same approved provider and data categories. Its
specific source is the `supported_languages` implementation window and small
`LanguageInfo` interface declaration, rather than the earlier `reconciler.rs`
window. It also sends the task goal, named-test identity, controller state/hashes
and bounded test feedback. This is relevant repository code and M3 observation
data within the broader October 14 approval; it does not add a recipient, data
category, aggregate spending limit or benchmark cohort. Protected file contents,
reference corrections, credentials and unrelated sessions are excluded.

No new source-export or spending approval is required for that bounded product
demonstration under the existing authority. The prepared run is capped at $0.25
within the original $100 aggregate, not an additional $100. Execution stops here
because the current task explicitly says to stop before external inference.
The live-mode flag, aggregate ledger, reservation, price, token and uncertainty
gates remain intact. Credentials, provider availability and current model pricing
were deliberately not checked; those remain live preflight checks. A new provider,
unrelated private data, higher aggregate cap or expanded benchmark work would be
a material change requiring its own authorization.

A named-test pass ends the transport loop. A newly generated patch still needs
its own full-suite closeout; the earlier reference repair's pass does not validate
future generated edits or establish model performance.

## Local verification

`python3 -m unittest scripts.test_engine_readiness scripts.test_engine_run -v`:
13 passed, 9 existing Docker-dependent tests skipped (22 discovered). This includes
credential-free loopback transport, token/cost bounds, refusal and uncertain-charge
recovery, plus new readiness/deadline checks. Initial loopback binding was blocked
by the shell sandbox; the same local-only checks passed with narrowly scoped
permission. No external request was made.

The final real-mutant preparation passed its offline dependency, source-window,
request-size, ledger and initial reservation checks. Python compilation and
`git diff --check` passed. No model-generated repair is claimed in this slice.
