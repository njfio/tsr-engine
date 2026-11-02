"""Advertised JSON-object mode, not proof of decoding enforcement.

Exact official Luna metadata snapshot retrieved2026-10-27 (see audit doc).
CheaperInference docs forward response_format but warn routes may ignore/reject it.
Unsupported metadata fails closed; callers must recheck capability before a live
admission. No endpoint/model substitution, tool calls, fallback or retry.
"""
CAPABILITY={'model':'gpt-6-luna','endpoint':'/v1/chat/completions',
 'features':['structured-output'],'observed_date':'2026-10-27',
 'source':'https://api.cheaperinference.com/public/models/gpt-6-luna',
 'snapshot_sha256':'fd16b79d468b602c26b4e710fb63c9cf7eb64e4b09728f6950fdf0a0100db103'}
def response_format(model,capability):
 if (model!='gpt-6-luna' or capability.get('model')!=model or capability.get('endpoint')!='/v1/chat/completions'
     or 'structured-output' not in capability.get('features',[])):
  raise ValueError('structured_output_not_advertised_for_profile')
 return {'type':'json_object'}
