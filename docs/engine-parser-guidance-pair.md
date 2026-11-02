# Preserved parser guidance comparison — offline preparation

The first approved attempt is recorded in [the failure/reconciliation report](engine-parser-guidance-pair-result.md). A separately authorized [fresh linked attempt completed the pair](engine-parser-guidance-pair-attempt2.md): blinded review found no substantive difference. The frozen preparation below remains unchanged.

This comparison uses genuine generated and independently reviewed Trial 11 guidance with actual isolated test receipts. It does not use the fake-worker reconsideration fixture, reopen the historical trial, adopt its optional final experiment, or execute any proposal. The earlier generic two-call outline was not a frozen manifest; this preparation supplies a concrete pair.

## Selected evidence

Both parent desires are the original evaluator-authored synthetic parser requirements: exactly empty input succeeds with an empty list; every nonempty input preserves original signed-i64 parsing behavior, including errors for malformed or empty fields and overflow. These are not the user's personal preferences.

The initial generated guidance was admitted as `eac40f933d3e4f51a2106e2cb8661293955cacc01bafc017cdb4664e11bf7b51`. Its independently reviewed post-experiment development was admitted as `cd84d35dd7575840522101e816a72933fe5e5a5c119d01c54d940df472910841`. Both use source snapshot `f19b0fc4963f718456e8fe2c1e6d6641e5c129efceec71ef03385305d103fb7d`.

The preserved source is the attempted strategy that filters empty fields. The actual protected receipt `4be453773f628f7a6d17af715e323c5a76baadbc0e7e02b1514b3c8f1eaa8d67` and supplemental receipt `3353f201d24dbdc3976a8185eb39cfc94febfa726e1e3039bef2c192e30910ee` report passing empty-input cases and failing nonempty-preservation cases. Assertion execution details and returned values remain unknown; the filtering behavior is a source inference. Neither arm receives the later successful patch.

The historical independent reviewer approved both responses as provisional grounded guidance, explicitly not proof of richer emergence. The development emphasizes the distinction between an exactly empty whole input and empty fields inside nonempty input. The prior already prohibited dropping fields: this is an **elaboration/emphasis comparison**, not a strong test of non-additive desire culmination. A subsequent record merely updates passing outcomes while retaining core meaning, so it is not used to manufacture a stronger contrast.

## Controlled question and limits

Compare useful next possibilities with prior guidance versus its reviewed development. Hold the synthetic parents, source, actual return evidence, uncertainty, lineage provenance, response instructions, model/settings and zero authority constant. Only the selected guidance projection differs. This is a counterfactual shadow projection of historical evidence, not a claim that the older guidance is presently admitted for execution. Do not duplicate developed narrative in both arms' common context.

No formal unmet/unknown acceptance judgment is invented. Historical qualitative reviewer prose stays local; it substantiates provenance, not an answer supplied to the model. No private repository code, protected test body, personal material, credentials, full native state or later winning patch is exported.

Frozen review criteria must distinguish a materially useful next hypothesis, discriminator or permitted proposed action from rephrasing, conjunction, unsupported novelty, replacement of a parent or invented test results. Identical proposals, justified retention and abstention are valid outcomes. Independent reviewers should receive neutral arm labels without the local arm mapping, and explain concrete differences rather than assign an emergence score.

The prospective envelope is two CheaperInference `gpt-6-luna` requests, each at most 32 KiB, 8192 completion tokens and 240 seconds; zero target executions, adoption or automatic acceptance. Both calls and any uncertain charge count against the existing campaign, never a new budget. Current accounting remains 183/200 calls, 57/60 groups and $0.159002/$5. The existing conservative two-call allowance is $0.10, not a guaranteed provider bill or an already-held exclusive reservation. Current official rates and remaining ledger head must be checked before any separately reviewed launch.

One pair can expose a concrete proposal difference consistent with guidance sensitivity. The developed guidance is longer and reiterates some common evidence; this compares the whole guidance package, not an isolated effect of relational meaning versus emphasis or length. Model nondeterminism, order and the tiny sample prevent claims of reliable causal superiority, generalization, real-world improvement or complete TSR fidelity. It also does not live-test the new v2 semantic-judgment projection, since this historical pair has no such judgment to export.

## Artifact and verification status

The frozen artifact is `target/trial11-guidance-pair-export-v1/`. The initial draft remains separately preserved. Canonical SHA-256 identities:

- Manifest: `020dd9102b3ed5562fe265da3176e475a63fb45c39b6546b206e4abd3962e9f8`
- X, historical prior: `a4b4d23aece71a38322c4e7195fd1b739f72164726a43e9d68948f1b3a4c73c1`, 5,976 request bytes.
- Y, reviewed elaboration: `72cb325d6712158bc4941ae2c2cd34805c7a3ddb9f1bfe4cde82451e20aa7ede`, 7,354 request bytes.

The local manifest freezes X-then-Y order, rubric, source/receipt/review/helper hashes and zero authority. The arm mapping is local; blinded reviewers should not receive that mapping or order. Both bodies use the same exact system instruction and explicit bounded descriptive-choice schema. Outputs would be shadow descriptions, **not native-valid responses, executable patches or admitted actions**. This pair therefore tests generated possibilities, not native ranking or execution. The longer developed guidance and fixed order remain limitations.

```sh
python3 scripts/engine-guidance-pair-export.py check \
  --out target/trial11-guidance-pair-export-v1 \
  --expected-sha256 020dd9102b3ed5562fe265da3176e475a63fb45c39b6546b206e4abd3962e9f8
PYTHONPATH=scripts python3 -m unittest test_engine_guidance_pair_export -v
```

`prepare --out NEW_DIRECTORY` reconstructs the same artifact from preserved history; existing differing output is refused. `check` performs no provider or native command. SQLite history is read through disposable copies including its WAL, so original databases are not opened or checkpointed. Typed generator records, separate operator admissions, exact independent review subjects, Rules-sourced actual receipts/imports and case outcomes are joined before export. The tool pins historical bytes but does not reimplement native BLAKE3 event validation.

Five focused tests passed (6.756 seconds), covering exact single-field difference, restart without writes, altered source/parents/lineage/evidence, private additions, authority expansion, missing reviews and absent/forged/mismatched receipts. Separate invocations of the exact check command passed, including independent review. The review is recorded in `specs/lanes/reviews/engine-parser-guidance-pair.md`; test output is `target/guidance-pair-validation/tests.log`. This is local preparation only: no new native state, generated response, paid call, execution or adoption. No paid call or target execution is authorized by this document. Parent review and fresh campaign/price/accounting preflight remain required before dispatch; no new generated revision is an irreducible prerequisite.
