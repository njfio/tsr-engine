# Bounded browser session preparation

The user authorized one session of up to 200 Luna calls within $10, with a proportional maximum of 100 isolated operations and a two-hour deadline. All calls, including source inspection, reviews and up to 20 shared format corrections, count against 200. No provider failure retry is enabled.

Historical accounting remains 262 calls, 72 operations and $0.225523. The new grant is bound to exactly one new trial and admission, plus source suite, ledger prefix and explicit authority. Its cumulative ceilings are 462 calls, 172 operations and $10.225523; historical trials cannot spend the new allowance. Unknowns retain reservations and stop. Each call reserves $0.05; the full prospective envelope is $10, not an intended spend.

Verification: 7 accounting/session tests passed; 7 adapter tests passed with the sealed test skipped in that first invocation; 6 shared correction tests passed. The separately enabled authored sealed browser full-flow passed in 36.858 seconds, including multi-file edits, actual browser observations, feedback and inert restart. These are authored fixtures, not an engine-produced game.

An earlier sealed invocation failed admission drift after the campaign implementation changed during that run. That failure is preserved in target/browser-session-preflight/browser-flow-test.log. Source was frozen before the successful rerun; no deadline or validation was weakened. git diff --check passed.

Commands and captured outputs are in target/browser-session-preflight/{accounting-tests,adapter-tests,correction-tests,browser-flow-frozen}.log. No paid provider call occurred during preparation. Runtime and filesystem controls remain unchanged, including the explicitly approved Chromium-only sandbox exception inside the isolated container. Live playability and TSR development remain unproven.
