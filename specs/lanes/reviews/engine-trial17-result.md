# Trial17 independent result review

Reviewer: `/root/b91_full_review`. Read-only evidence assessment; no provider calls, executions, candidate edits, or retry. Trial17 ended before relational revision review or a model-generated boundary patch. It is not an accepted repair or completed B9.1 cycle.

## Established failure

`target/b91-integrated-trial17/provider.jsonl` records request `a6c2534d-5d62-4046-b203-e80c1c132459` as HTTP 200, received, finish reason `stop`, invalid at `relation_after_experiment` with `return_basis_rejected`. Its measured charge is $0.000849. The driver retains the rejected candidate's bounded structure, not its prose. The canonical JSON-string hash of the schema key `binding` is `913be61629bf526ec23a58e7ea7c74eae0ddcdbb6a5ad15692123aadd8e06b7f`; that field's retained string length is 65 bytes. The required return-evidence identity is a 64-byte digest. This establishes an identity mismatch without reconstructing the rejected content. Other defects or the semantic quality of the unretained revision cannot be determined. There is no evidence of transport uncertainty, model inability, or an instruction requiring a wrong-length identity.

All three provider calls settled: $0.002558, no unknown cost or usage. Two real isolated execution groups occurred. No resume or retry is warranted under this terminal trial.

## Actual experimental evidence

The independently approved generated harness contains genuine assertions and leaves source/protected tests unchanged. Its two named cases test empty input and a finite selection of nonempty inputs. The second case first checks `-7,0,42`, then checks interior empty fields, followed by trailing empty fields, whitespace, malformed input and overflow.

The actual immutable worker stdout is `checks-before/coding-receipts/f62d3d4df72969aab7e1bbd76986c9544e08cf5faa93d6b305f99fbf56dcb52c/cas/d465aab6b5327a14fb2446facf00c91a63e93f36e990c57707ac096753914539` under the trial root. It records:

- `empty_input_handled` and `supplemental_empty_input` passed.
- `existing_behaviour_unchanged` failed on `1,,2`.
- `supplemental_nonempty_preserves_fields` failed at line 11 on `parse("1,,2").is_err()`.
- The Cargo command terminated normally with exit 101; compilation succeeded.

Given the inspected sequential harness and actual failure location, the preceding signed-number assertion completed successfully. The later assertions in that test did not run after the panic; neither their pass nor failure is established. This is a real counterexample to preservation and positive finite evidence for empty input and the signed sample, not universal correctness. The provider's narrower typed return catalog intentionally does not expose assertion-level execution; these reviewer observations must not be attributed to the model's input.

The initial relational aspiration preserved both requirements but chiefly conjoined them; non-additive culmination was not demonstrated. Both standing requirements remain unaccepted overall, with preservation known violated. No boundary patch, successful re-verification, final proposal or acceptance was produced.

## Smallest remaining integration boundary

The project driver already routes source-bound review contexts through `Driver.review` / `FileReviewer`. A project-specific runtime reviewer can implement that existing interface for provisional relation, experiment, patch-scope and receipt-grounding gates, using actual native project evidence and separately metered invocations. Archive-specific review criteria/receipts cannot simply be relabeled. Protected test bodies, private review notes and reference answers must remain excluded; static source reasoning must remain distinct from observed execution. Formal native semantic acceptance must remain the separately authorized operator review. This bridge cannot waive, repair, or retry the preceding malformed return identity, and automated review quality remains to be tested rather than assumed.
