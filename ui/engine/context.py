"""Lossless public context normalization; no inference, authority or dispatch."""
import json

PREFIX='For a question, explain what is supported without requiring a code change. Use the native no-action response when warranted; do not invent observations or user preferences.\nOriginal source-bound public context (historical claims retain their original evidence limits):\n'
RECORDS='Attributed prior assessment/review records (not new player preferences):\n'
INPUTS='Recorded input:\n'
HEADER='Studio public context v2\n'
LEGACY_TAIL='Local user-requested browser project. Choose what, if anything, is justified by the exact user input and recorded evidence. Preserve contributing wants/history; no forced novelty, relationship, or change. No network/assets/packages, private data or publication. The protected start/status interface is a harness contract, not a design choice. '
RULES=('Address the latest user request identified below. Distinguish an explicit creation/change request (including politely phrased "can you") from a question asking for explanation or optional unsolicited refinement. For a change request, use the delegated design freedom to propose a useful permitted approach; an open art direction is not by itself missing authorization or specification. Existing passing checks constrain regressions, not satisfaction of a renewed user goal. For a question, answer what the evidence supports without requiring an edit. Retention or abstention remains valid for a specific justified constraint or when it addresses the actual request; explain unresolved goals honestly. Do not force novelty, relationships, satisfaction or a particular design. Historical instructions remain history; the latest request does not expand permissions. Preserve protected checks and source/evidence boundaries.')

def unpack(brief):
 if brief.startswith(HEADER):
  obj,end=json.JSONDecoder().raw_decode(brief,len(HEADER))
  if set(obj)!={'original_brief','prior_records','latest_request_id','instruction'}:raise ValueError('Public context shape changed')
  events=json.loads(brief[end:].removeprefix('\n'+INPUTS))
  return obj['original_brief'],obj['prior_records'],events
 if not brief.startswith(PREFIX):return brief,[],[]
 while brief.startswith(PREFIX):brief=brief[len(PREFIX):]
 # Legacy wrappers append JSON arrays. Decode whole arrays, never scan inside
 # report/user strings for apparent delimiters. Keep every unique record.
 marks=[RECORDS,INPUTS];positions=[brief.find(m) for m in marks if m in brief]
 original=brief[:min(positions)].rstrip() if positions else brief
 records=[];events=[];cursor=len(original)
 while cursor<len(brief):
  candidates=[(brief.find(m,cursor),m) for m in marks if brief.find(m,cursor)>=0]
  if not candidates:
   if brief[cursor:].strip():raise ValueError('Unrecognized legacy context suffix')
   break
  at,marker=min(candidates);gap=brief[cursor:at].strip()
  if gap and gap!=LEGACY_TAIL.strip():raise ValueError('Unrecognized legacy context separator')
  start=at+len(marker)
  values,end=json.JSONDecoder().raw_decode(brief,start)
  if not isinstance(values,list):raise ValueError('Public history is not an array')
  (records if marker==RECORDS else events).extend(values);cursor=end
 return original,records,events

def unique(values,key):
 out={}
 for value in values:
  identity=key(value)
  if identity in out and out[identity]!=value:raise ValueError('Conflicting public history identity')
  out[identity]=value
 return list(out.values())

def package(prior_brief,records,events):
 original,old_records,old_events=unpack(prior_brief)
 records=unique(old_records+records,lambda x:(x['journal'],x['record']))
 # Current event projection may recompute applicability, without changing content.
 combined={}
 for v in old_events+events:
  old=combined.get(v['id'])
  if old and any(old.get(k)!=v.get(k) for k in ('text','attribution','source_snapshot','report_sha256')):raise ValueError('Public input identity changed')
  combined[v['id']]=v
 events=list(combined.values())
 latest=next((v['id'] for v in reversed(events) if v.get('record_type')!='independent_assessment'),None)
 obj={'original_brief':original,'prior_records':records,'latest_request_id':latest,'instruction':RULES}
 return HEADER+json.dumps(obj,ensure_ascii=False)+'\n'+INPUTS+json.dumps(events,ensure_ascii=False)
