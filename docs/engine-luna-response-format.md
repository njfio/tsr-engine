# Luna repair-stage response format

Read-only official inspection on 2026-10-27 at 00:03:50UTC confirmed `gpt-6-luna` remains visible, advertises structured output, and serves `/v1/chat/completions`. Current metadata lists $0.083813 input/$0.419064 output per million tokens; these differ from earlier snapshots. No inference or authenticated request was made for this inspection.

Sources:
- [Exact Luna public metadata](https://api.cheaperinference.com/public/models/gpt-6-luna).
- [CheaperInference parameters](https://www.cheaperinference.com/docs#parameters) explicitly forwards `response_format` and documents JSON-schema output where supported. It warns that serving-provider interpretation varies and unsupported fields may be ignored or rejected.
- [CheaperInference OpenAPI](https://www.cheaperinference.com/api/openapi.json) permits additional Chat Completion request properties; it does not promise schema enforcement.
- [OpenAI structured output](https://developers.openai.com/api/docs/guides/structured-outputs) documents strict-mode closed objects and required fields, excludes conditional `if`/`then` schemas, and gives the `json_schema` envelope.
- [OpenAI Chat Completion request](https://developers.openai.com/api/reference/resources/chat/subresources/completions/methods/create) specifies the response-format interface and optional strictness.

The existing relational contract has conditional and optional fields; experiment harnesses have model-selected filenames. Sending that schema unchanged as `strict:true` would not meet the documented strict subset. The narrow helper supplies **best-effort `json_schema` with `strict:false` only for experiment and boundary-patch generation**. It describes closed envelope/experiment/case/mapping shapes and the native rationale limit, without prescribing an assertion, source repair or harness filename. Native parsing, source binding, review, admission and validation stay authoritative. No automatic stripping or repair of malformed output occurs.

JSON Schema string length counts characters, while the existing native gate counts UTF-8 bytes. The schema descriptions explicitly retain the 512-byte limit; local checks still reject a multibyte string that fits 512 characters but exceeds 512 bytes. This mechanism is an instruction-format improvement, not proof that the gateway constrains decoding or that future responses will comply. The unchanged deadline/token/accounting gates still apply, including stop on unknown outcomes and no speculative retry.

Local public snapshots: `target/luna-structured-readonly/model.json` SHA256 `fd16b79d468b602c26b4e710fb63c9cf7eb64e4b09728f6950fdf0a0100db103`; `openapi.json` SHA256 `a08e0057ea5f8c5f35e95128c159b327cafa33ecadc6d534b29df7979783f0ba`.

Seven focused offline helper/native-parser checks passed. Actual assembled-request size, helper pinning and transport checks must pass before this helper is activated. No live efficacy result is claimed.
