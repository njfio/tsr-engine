# Verified origin export and conserved requirement aliases

A verification-only episode begins with an already generated candidate. Its ordinary
export correctly has an empty edit diff. To export the original repair with checked
lineage, use:

```sh
target/debug/tsr engine --directory target/verification-environment-fixed/episode \
  project export --accepted-only \
  --origin-run target/collection-qwen32-episode/episode \
  --origin-commitment 32779e980953866f3c55d98c01aa7f189f5ac1d61aed2751f91654d5697932db
```

The engine checks the original admitted mutant, controller-registered generated
patch, executed intent, matching observed source return, complete current source
inventory and independent acceptance. It reconstructs the replacement and rejects
wrong commitments, source drift, verification-as-edit origins and an empty original
repair. First-edit initial-mutant origins are supported; later multi-edit histories
fail closed rather than inventing a baseline. No model execution, edit replay,
resource request, label or history mutation occurs.

The returned content-addressed JSON and patch live in the accepted episode's
exports directory. JSON includes original scope/plan/proposals, initial and
candidate snapshot/file hashes, acceptance and ledger references. This is a
machine-checked evidence index against the two local ledgers, not a standalone
cryptographic proof of the entire ledger or provider authorship. `Source::Rules`
is the controller's proposal-registration provenance. Application-level write-once
artifact creation refuses conflicting existing bytes; it is not filesystem-enforced
immutability. Ordinary exports and all historical rejection records remain intact.

Actual accepted-candidate export was checked successfully; result pointer:
`target/verification-environment-fixed/verified-origin-export.json`. No new source
or accepted evidence was manufactured. Original model attribution remains in its
preserved request/response evidence, rather than inferred from this join.

## Core decision correction

Distinct requirement keys with identical text formerly generated the same preference
identity twice. Both then entered the shared decision kernel, whose duplicate-ID
check rejected ranking. New admission rejects duplicate trimmed text before writing
any events. Legacy aliases fold into one obligation and one observation per reviewed
receipt. Conflicting aliases yield failure; missing evidence preserves unknown
unless a failure is already established. This conserves desire influence and avoids
manufacturing confidence through duplicate wording. Original requirements, reviews
and history remain readable and unchanged.

Tests exercise rejected admission with unchanged root, legacy ranking, restart and
unchanged controls, contradictory/missing evidence and alias-order invariance.
Export tests cover empty verification diff versus original repair, repeated export,
wrong origin, drift in either workspace, unchanged event roots and no resource
recharge. CLI library: 142 passed, three ignored; Clippy warnings denied passed.
Independent read-only review cleared both changes. No live inference or prospective
execution occurred in this pass.
