# Two-task pilot infrastructure repair

Original attempts and charges remain in `m3-live-pilot-result.json` and `m3-live-pilot-execution/`.

The AgentID record CLI failed before batch dispatch because parallel arms opened the same default SQLite store. The retry supplies a unique absolute controller-owned TSR_DB per arm, outside evaluated copies. No M0 files changed.

The adapter conflated the 70,000-byte protocol input cap with the larger nested JSON wire envelope. Quote, slash, newline and Unicode fixtures reproduce that defect. The wire cap is now 300,000 bytes; observation scope, prompt, schema, model, output limit, conservative remaining-token check and oracle are unchanged. The worst-case priced bound is $0.0560592; each request still reserves $1 under the common $100 ledger.

Original RTS failures cannot be conclusively diagnosed: historical errors were suppressed. The short record failure occurred after billing settlement; the later rules failure followed accumulated observations. Neither is asserted to be the reproduced wire defect. Prospective diagnostics retain fixed stage/code/count/timing/reservation fields only. Provider refusals, malformed proposals and truncation are terminal, without implicit retry.

The explicit prospective retry plan runs AgentID record plus both RTS arms once, preserves originals and retains original AgentID rules for the resulting comparison. Exact tasks, profiles, contexts, helper, launcher, broker and engine are hash-pinned. Both RTS arms use the same repaired configuration. No corpus qualification restarts.

Validation: `python3 -B scripts/test_m3_cheaper_broker.py`: 10 passed (captured in m3-validation/m3-pilot-repair-tests.log). The red boundary test is preserved separately. Tests use harmless local fixtures, mocked transport, and temporary SQLite databases. Rust was unchanged; prior workspace checks remain the applicable engine evidence.

Independent review checked billing, privacy, unchanged evaluation semantics and launcher scope. Its input-freeze blocker was addressed before launch. Near-limit Unicode requests can still hit the conservative token reservation bound; that is a budget refusal, not transport evidence.

Terminal review verified all three exported result hashes, the exact prospective selection, the 36 settlements totaling $0.045569 with zero outstanding reservations, and the saved process/container shutdown check. RTS is one paired acceptance outcome, not two full oracle verdicts; its rules tamper rejection and invalid zero-latency field are explicitly documented. The exporter retains those limitations. No reporting blocker remained.
