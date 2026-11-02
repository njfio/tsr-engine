# Action-specific integration readiness

The new `exact-edit@v3` wire contract requires only action-specific fields. Inspect has action/path; test has action/test_id/forecast; edit has action/path/expected_digest/old_text/new_text/forecast; stop has action. No field is optional. Unknown and inactive fields are rejected. Test IDs must be explicitly supplied and allowlisted; the trusted adapter maps validated test_id to unchanged internal Proposal::Test { id }. Request IDs remain host-owned.

Schema, examples and decoder field sets share CONTRACT. Diagnostics retain only fixed paths, types, reasons and one finite action category (inspect/test/edit/stop/missing/invalid/unparsed). Full provider bodies and arbitrary keys/values are not logged. Semantic failures halt; no automatic retry.

Five offline tests passed with 132 matrix cases through schema/decoder and mocked provider transport with real temporary ledger writes. Twenty existing broker tests passed. The full scripted synthetic fixture passed through the real sealed worker, exact edit, visible tests, Stop and protected final oracle. Its exported receipt was independently reread and verified. These checks are not live provider success.

The new freeze is d5de0a72d4e52d4c0af797f282fcf11b212c736d605e21b96ed8e4a7d724a97b (603 inputs). Fresh candidate/state paths are separate from the failed attempt. The original cumulative stage baseline remains $0.229744; launch baseline $0.229988 reflects the prior $0.000244 request. The $5 incremental/40 total request cap is not reset: 39 requests remain at most, with 20 per arm and 1800 seconds per attempt. Existing shared ledger locking, price and settlement checks, worker containment, protected grading and receipt verification remain in force.

Independent review found no blocking contract or budget defect. All prior frozen inputs and the failure remain unchanged. No larger comparison is launched by this controller.
