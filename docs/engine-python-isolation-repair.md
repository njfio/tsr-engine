# Python aggregate isolation repair

2026-10-23, following `66faf9c`. Repository protection review found these Python test
files outside the frozen M0 fixtures, protected Rust files and hook scope in AGENTS.md.
No production controller, broker, cost record, baseline, unknown reservation or protected
file was changed. No model calls, target executions, downloads or external actions.

## Reproduction and repair

Before edits, the combined fixed12/base/V3/V5/V6 suites plus the two isolated-budget
checks reproduced **31 tests, two errors and two failures**. Independent source review
confirmed that versioned tests assigned the imported base module's controller globally;
V3 also aliased the base class. The budget checks read a mutable operational ledger but
asserted fixed historical totals. Production accounting correctly reported current spend.

The base suite now uses a class-local controller dependency. Versioned subclasses select
their own controllers, and V3 has its own subclass instead of aliasing the base. Existing
assertions, expected values, fixtures and strict V5/V6 validation remain unchanged.

The isolated-budget tests retain expected strings **0.165094** and **0.208627**. A dedicated
helper creates explicitly synthetic read-only ledgers at those fixed amounts and matching
synthetic baselines in disposable directories. It copies the unchanged broker/helper bytes
and exercises their normal relative-baseline lookup through `python -I`, without credentials.
These synthetic records make no claim to represent historical provider activity. Real ledgers
and baseline files are neither copied into committed fixtures nor modified. No cost is
reclassified, and unknown reservations remain unknown.

## Verification

The complete affected module group passed **35 tests**. New regressions passed **two tests**
in 11.433 seconds: all 24 controller import orders, each running the 29-test suite twice
in one process, plus identical dedicated inputs across fresh temporary directories with
read-only ledger mode and byte-identical broker copies. Independent review found no
blocking issues or weakened assertions. `git diff --check` passed.

Full discovery (`PYTHONPATH=scripts python3 -m unittest discover -s scripts -p
'test*.py'`) passed **573 tests: 557 passed, 0 failures/errors, 16 skipped**, in
113.693 seconds (wrapper 113.958 seconds, exit 0, no timeout). The real cost ledger
and both historical baseline hashes matched before and after. Existing real
execution opt-ins are disabled; default B9.1 native-worker tests remain skipped rather
than counted as newly verified here. Prior native/Rust evidence remains separately dated.

Local evidence: `target/python-isolation-repair/` contains the before reproduction,
order-check log, complete Python log and terminal result with before/after cost-ledger
and baseline hashes. B9.1 live admission remains pending and ungranted.
