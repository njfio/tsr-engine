# Independent terminal audit

The independent reviewer found no discrepancy in the completed six-attempt experiment.

- DeepSeek: two verified final-oracle successes of three, one stale-digest failure; 16 requests, $0.003152.
- GLM: two verified final-oracle successes of three, one string-valued forecast rejected; 15 requests, $0.005345.
- Total: 31 settled requests, $0.008497 additional, $0.239583 aggregate, no outstanding reservation.

The reviewer verified all 167 exported hashes, six decision/diagnostic/settlement bindings, SQLite integrity, both old and corrected frozen inputs, four checked successful oracle receipts and oracle:true closures. Both failures lack oracle success. The stale-edit candidate is 105 bytes, below the source limit. All attempts stayed within request/time/spending limits.

The primary terminal runtime guard separately passed; the reviewer made no runtime, provider, rerun or mutation calls. The audit supports the reported bounded synthetic outcomes only, not a reliable model ranking or M3 real-corpus acceptance.
