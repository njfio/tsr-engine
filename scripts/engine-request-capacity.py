"""Explicit Luna browser capacity; byte upper bound is conservative, not a tokenizer.
Official public catalog: context_length1050000, max_output_tokens128000 (2026-10-29).
Reserve one input token per serialized UTF-8 byte plus framing allowance. Measured
usage remains authoritative; exceeding the estimate fails closed, never refunds.
"""
import json
CONTEXT_TOKENS=1050000
COMPLETION_TOKENS=32768
FRAMING_TOKENS=4096
INPUT_TOKENS=CONTEXT_TOKENS-COMPLETION_TOKENS
PAYLOAD_BYTES=INPUT_TOKENS-FRAMING_TOKENS
PROFILE='luna-context-budget@v1'
# Local append-only evidence allocation for a maximum 200-request session.
# This is an explicit disk/memory budget, not a provider or code-size limit.
JOURNAL_BYTES=512*1024*1024
def serialized(body):return json.dumps(body,allow_nan=False,ensure_ascii=False).encode()
def prompt_upper(body):
 n=len(serialized(body))+FRAMING_TOKENS
 if n>INPUT_TOKENS:raise ValueError('request plus completion exceeds admitted model context estimate')
 return n
