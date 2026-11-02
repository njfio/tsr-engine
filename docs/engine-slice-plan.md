# Persisted engine slice

Reuse `tsr-domain` candidates, forecasts, commitments, Surprise/Contrast, `tsr-store` append-only SQLite, and `tsr-ledger` preference/status folds (PRD B2, B2.1, B6, B8). Add `tsr engine init|step|show` with an explicit store directory. Typed local-domain facts live in content-addressed files referenced by existing Evidence events; no shared contract or M0 behavior changes.

The deterministic queue domain has a cheap bulk route and a checked route. A bounded probe supplies evidence before commitment; observed delivery updates per-route expectations. Unmet desired criteria produce separate contrast, a bounded derived pressure signal, and pending preference refinements. Generated refinements require explicit operator admission. A proposer trait receives the reconstructed view, never storage authority. Atomic event batches plus a per-directory writer lock prevent half cycles and concurrent updates; restart folds the same history.

Deliver three readable cycles, durable resume, and focused tests for changed choices, failed evidence admission, pressure bounds, forecast/return separation, and persistence. This is a local operational slice, not calibrated production decision-making or M3 completion. No provider requests or benchmark expansion.
