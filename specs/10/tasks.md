# Tasks #10

1. T1 Write failing tests for C-01 to C-13. Tiers: Conformance, Unit, Property, Snapshot, Integration, Regression.
2. T2 Confirm the plan's four questions and update the spec if needed. Depends on T1.
3. T3 `contract:` commit adding `Reading` and `Returned`. Depends on T2.
4. T4 Ledger record methods and fold. Depends on T3. Mutation tier on the fold and checks.
5. T5 Reading and proposal order in the cycle, and the operator return command. Depends on T4.
6. T6 Python opt-in, reading schema and menu-free prompt. Depends on T5. Fuzz tier on the reading parser, at least 10k iterations.
7. T7 Request view with omitted count, and run counts. Depends on T4. Performance tier: compare request build before and after.
8. T8 PRD, invariant 8, ADR. Depends on T5.
9. T9 Full suite and `cargo mutants --in-diff`. Depends on all.

No paid run is part of this work. A live test needs separate approval.
