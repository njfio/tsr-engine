# Fresh local attention validation — 2026-10-22

The approved bounded validation completed successfully on code `13c0ce6` using only
`examples/engine-invoice/project.json`, a synthetic repository example. No real
invoice, account or payment data was involved. No source was sent externally.

The run is preserved at `target/attention-live/`; the complete accepted local export
is `accepted-export.json`. Original desires, three protected probes, source/spec
hashes, installed model digest, attention and resource admission were frozen before
generation in `manifest.json`, `project.json`, `requirements.json`, `admission.json`
and `request-1.json`. No reference patch or authored replacement was supplied.

| Step | Selected attention | Genuine generated edit | Independently observed return |
| --- | --- | --- | --- |
| 1 | Published `discount` requirement | Discount function only | Floor probe passed; subtotal produced 152 instead of 300; combined invoice produced 122 instead of 270. Overall review rejected; discount semantics met, subtotal unmet. |
| 2 | Published `subtotal` requirement | Pricing function only; discount preserved | All three protected probes passed: 300, 270, 0. Independent static semantic review accepted the frozen requirements within the stated arithmetic boundary. |

Immediately after the first execution, **before new review or guidance**, the
reopened request selected public `test:subtotal` (`request-after-return-1.json`).
After independent review, the semantic selector focused the unmet subtotal
requirement. The review invalidated the old feedback publication binding: the
**identical original requirement text** was rebound to the new review in
`feedback-2.json`. No new task or repair instructions were introduced. This manual
review/publication step remains a real operator-assisted boundary.

Both normal responses carried the exact current attention binding. Two committed
`Rules`-sourced `AttentionConsumed` events match their respective saved requests
and proposals; `attention-consumption.json` includes their actual log references.
The controller supplied only existing spans/opaque metadata; all changed code bytes
came from the installed local model. This trajectory demonstrates consumed focus
and observed adaptation, not a controlled statistical proof that focus caused the
model's edits.

After final independent review, the request reports `no_eligible_attention`.
The episode closed at **2026-10-22T10:32:23.264Z**. The accepted-only export succeeded.
A read-only replay check rejected the old response as stale with the factual root
unchanged and no dispatched execution (`replay-rejected.json`). Original failed
observations and rejection remain in the append-only history. Protected source
SHA-256 still matches the frozen manifest.

## Limits and measured resources

Installed local `qwen2.5-coder:32b`, digest
`4bd6cbf2d094264457a17aab6bd6acd1ed7a72fb8f8be3cfb193f63c78dd56df`,
was verified before dispatch. The existing fixture image digest was verified locally.
Sampling remained temperature 0, context 16384, output bound 2048 tokens and 240
seconds per call. Both calls and both isolated executions consumed the admitted
slots; none remain. No retries, extra model calls or extra candidate executions ran.

- Calls: **2/2**; executions: **2/2**.
- Tokens: **4779 input + 265 output = 5044** (ceiling 69632).
- Model duration including reported loading/runtime: **103.426 seconds**.
- Measured tool duration: **18669 ms** (ceiling 150000 ms).
- Charged patch bytes: **272** (ceiling 8192).
- External inference/spend: **0**; unsettled requests/executions: **0**.
- Temporary loopback Ollama PID60732 was stopped; its session terminated normally.

`summary.json`, `budget-final.json`, both `.local-receipt.jsonl` files and both
execution records preserve the actual accounting. The new admission is closed;
unused slots would expire at closure, and this run has none. Direct sandbox access
to the Docker socket initially failed; the explicitly scoped image check and two
executions subsequently succeeded through the approved escalation path. No daemon,
security, credential or persistent-service configuration was changed.

## Independent review and what this establishes

A separate local review agent inspected both actual candidates, frozen contracts,
protected hashes and observed receipts. Discount subtraction/floor logic is general
for `u32` inputs. Multiplication is correct for products representable as `u32`;
overflow policy beyond that range is unspecified and not established. The three
probes plus static review do not constitute exhaustive input testing.

Active reviewer effort remains **unknown**. This is a **synthetic fixture**, not an
eligible independent operational episode. The native training label remains null.
Close-plus-seven-days would be 2026-10-29T10:32:23.264Z, but merely reaching that date
cannot make synthetic data eligible. No calibration, maturity, benchmark superiority
or general unattended-autonomy claim follows. The historical 151 qualified corpus
tasks and their separate maturity limitations are unchanged.

The scoped integration is live-verified: original desires informed attention, genuine
proposals consumed its binding, isolated returns and independent review changed the
next focus, and completion stopped further generation. Independent semantic review
and rebinding published guidance were manual. No broader execution authority was
created, and no original target repository or protected test was edited.
