#!/usr/bin/env python3
"""Actual native admission and serialized request checks, with dispatch disabled.

Creates adapter journal only in prepared isolated state. No target operations,
model calls, native acceptance or authored semantic memory admission.
"""
import argparse, copy, hashlib, importlib.util, json
from pathlib import Path
from types import SimpleNamespace
P=Path(__file__).resolve().parent;ROOT=P.parents[1]
s=importlib.util.spec_from_file_location('guide_native_preflight',ROOT/'scripts/engine-objective-adapter.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class Captured(Exception):pass

def main():
    ap=argparse.ArgumentParser(description=__doc__);ap.add_argument('--prepared',required=True,type=Path);ap.add_argument('--campaign-manifest',required=True,type=Path);a=ap.parse_args()
    root=a.prepared.resolve();ad=m.e.strict((root/'admission.json').read_bytes());campaign=m.e.strict(a.campaign_manifest.read_bytes())
    c=m.d.Native(root/'state');ledger=SimpleNamespace(manifest=campaign)
    def forbidden(*args,**kwargs):raise AssertionError('external operation forbidden in local preflight')
    adapter=m.Adapter(root/'adapter',ad,c,ledger,transport=forbidden,worker=forbidden)
    assert ad['origin']['version']=='public-objective-origin@v2'
    assert ad['origin']['receipt_checker_profile']=='open-guide@v1'
    view=c.cli('project','cycle','request');request,projection=adapter.project(view)
    assert projection['relational']['history']==[]
    assert projection['objective']['context']['round']==0
    assert projection['objective']['context']['receipts']==[]
    encoded=m.e.canonical(projection)
    for future in json.loads((P/'conditions.json').read_text())['rounds'][1:]:
        assert future['body'].encode() not in encoded
    assert b'deliberately valid for all frozen queries' not in encoded
    stale=copy.deepcopy(view);stale['state_root']='0'*64
    try:adapter.project(stale)
    except ValueError:pass
    else:raise AssertionError('stale root accepted')
    snapshots=[]
    def capture(kind,body,*args,**kwargs):
        wire=m.e.canonical(body);assert len(wire)<=32768
        snapshots.append({'kind':kind,'bytes':len(wire),'sha256':hashlib.sha256(wire).hexdigest()})
        assert body['response_format']=={'type':'json_object'}
        raise Captured()
    adapter.checked=capture
    try:adapter.proposal(view)
    except Captured:pass
    response={'protocol':request['protocol'],'state_root':request['state_root'],'attention_binding':request['next_attention']['binding'],'proposals':[{'kind':'service','choice':{'binding':projection['objective']['binding'],'rationale':'Authored preflight placeholder only, not a generated decision','success_probability':0.5}}]}
    preview=c.json_cli(('project','cycle','preview'),response,'--response')
    adapter.generated=lambda:[response]  # authored candidate only; no journal adoption
    try:adapter.review({**view,'response_digest':preview['response_digest']},response)
    except Captured:pass
    restarted=m.d.Native(root/'state').cli('project','cycle','request')
    assert restarted['binding']==view['binding']
    assert restarted['objective']['context']['receipts']==[]
    result={'checks':['actual_v2_constructor_admission','native_public_projection','future_body_exclusion','stale_root_rejection','actual_proposal_and_review_serialization','native_preview_only','fresh_process_binding_unchanged'],'requests':snapshots,'provider_calls':0,'target_operations':0,'adoptions':0,'launch_authorized':False,'limits':'No generated behavior, target execution, or all-round model context growth tested here.'}
    (root/'LOCAL_PREFLIGHT.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
if __name__=='__main__':main()
