# Tasks #10

1. T1 Write failing tests for C-01 to C-09. Tiers: Conformance, Unit, Property, Integration, Regression.
2. T2 Confirm the two plan questions and update the spec. Depends on T1.
3. T3 `contract:` commit adding the event kind. Depends on T2.
4. T4 Ledger crossing and fold. Depends on T3. Mutation tier on the fold and checks.
5. T5 Cycle path with no review input. Depends on T4.
6. T6 Python opt-in and adapter. Depends on T5. Fuzz tier: N/A unless T2 finds a new parser.
7. T7 Request bound. Depends on T4. Performance tier: compare request build before and after.
8. T8 PRD, invariant 8, ADR. Depends on T5.
9. T9 Full suite and `cargo mutants --in-diff`. Depends on all.
