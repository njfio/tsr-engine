"""Allowlisted local preparation of native public-desire contexts. No transport or adoption."""
import copy
import importlib.util
import json
from pathlib import Path
s=importlib.util.spec_from_file_location('public_export_envelope',Path(__file__).with_name('engine-b91-project-envelope.py'));e=importlib.util.module_from_spec(s);s.loader.exec_module(e)
def exact(value,keys,name):
    if not isinstance(value,dict) or set(value)!=set(keys):raise ValueError(name)
def text(v):
    if not isinstance(v,str) or not v.strip() or not 0<len(v.encode())<=512:raise ValueError('bounded public text')
    return v
def project(request,origin,reviewed_revisions=None,*,expected_request_sha256):
    e.digest(expected_request_sha256)
    if e.sha(request)!=expected_request_sha256:raise ValueError('stale request projection')
    e.validate_origin(origin)
    coding=request['coding'];r=coding['relational'];i=r['inputs'];attention=request['next_attention']
    if i['version'] not in ('project-relational@v1','project-relational@v3'):raise ValueError('unadmitted evidence profile')
    if (coding['snapshot']!=origin['initial_snapshot'] or i['snapshot']!=origin['initial_snapshot'] or i['requirements_revision']!=origin['requirements_revision'] or attention['scope']!=origin['scope'] or attention['state_root']!=request['state_root']):raise ValueError('stale source/scope')
    # This slice has no target returns, supplemental experiments or semantic judgments.
    if i['returns'] or i['supplemental'] or 'semantic_return' in i:raise ValueError('unadmitted return evidence')
    if i.get('acceptance_parents',i['parents'])!=origin['parents']:raise ValueError('acceptance changed')
    if coding['cases']!=e.expected_configuration(e.origin_prepare(origin))['cases']:raise ValueError('unadmitted cases')
    files=coding['files'];editable=[f for f in files if f.get('editable')]
    if len(editable)!=1 or editable[0]['path']!='src/lib.rs' or editable[0]['source']!=origin['initial_source']:raise ValueError('unadmitted editable source')
    if {f['path']:f['source_digest'] for f in files if not f.get('editable')}!=origin['protected'] or coding.get('supporting_context'):raise ValueError('private/source inventory drift')
    known={p['id']:p for p in origin['parents']};reviewed=reviewed_revisions or {}
    if not isinstance(reviewed,dict):raise ValueError('reviewed content pins required')
    for revision,pin in reviewed.items():e.digest(revision);e.digest(pin)
    parents=[];seen=set()
    for p in i['parents']:
        if p['id'] not in known or p['id'] in seen or p['requirement']!=known[p['id']]['requirement']:raise ValueError('unadmitted parent')
        seen.add(p['id']);revision=p.get('revision')
        if revision is None:
            if p['meaning']!=known[p['id']]['meaning']:raise ValueError('unreviewed meaning')
            parents.append(copy.deepcopy(known[p['id']]));continue
        if revision not in reviewed or e.sha(p)!=reviewed[revision] or p.get('original_meaning')!=known[p['id']]['meaning']:raise ValueError('unreviewed revision/origin')
        if p.get('withdrawn') is not False or p.get('change') not in ('clarification','development'):raise ValueError('inactive/change')
        ancestry=p.get('parents')
        if not isinstance(ancestry,list) or not ancestry or len(set(ancestry))!=len(ancestry) or not set(ancestry)<=known.keys() or p['id'] not in ancestry:raise ValueError('public ancestry')
        # Unknown is the only possible basis in this zero-return preparation.
        if p.get('basis')!={'kind':'unknown'}:raise ValueError('invented observation')
        parents.append({k:copy.deepcopy(p[k]) for k in ('id','requirement','meaning','original_meaning','revision','parents','change','basis')})
        text(p['meaning'])
    if set(p.get('revision') for p in parents if p.get('revision'))!=set(reviewed):raise ValueError('reviewed revision set mismatch')
    history=[]
    if i['version']=='project-relational@v3':
        public=i['public_desires']
        if public['active']!=i['parents']:raise ValueError('active projection mismatch')
        for h in public['history']:
            if 'revision' in h:
                if h['revision'] not in reviewed or e.sha(h['next'])!=reviewed[h['revision']] or h['superseded'] is not False or h['withdrawn'] is not False:raise ValueError('unreviewed historical revision')
                e.digest(h['next']['proposal'])
                history.append({'revision':h['revision'],'superseded':h['superseded'],'withdrawn':h['withdrawn'],'proposal':h['next']['proposal']})
            else:raise ValueError('status changes require separate frozen comparison')
    # Never include review notes, reviewer identities, private criteria/tests, full state,
    # file inventories or arbitrary guidance. Only bound public meanings and IDs.
    if r['history'] or r['guidance']:raise ValueError('unadmitted relational history')
    if request['protocol']!='tsr-proposal@v1':raise ValueError('native protocol')
    for value in (request['state_root'],attention['binding'],r['binding'],editable[0]['source_digest'],editable[0]['inspection_handle']):e.digest(value)
    contracts=e.payload_contract(request,'final_proposal_only')
    contracts.pop('abstain',None)
    contracts['experiment']['rules']=[x for x in contracts['experiment']['rules'] if 'experiment stage' not in x]
    return {'profile':'synthetic-public-desire@v1','parents':parents,'acceptance_parents':copy.deepcopy(origin['parents']),
      'source':{k:editable[0][k] for k in ('path','source','source_digest','inspection_handle')},'public_lineage':history,
      'evidence':{'observed':[],'source_inference':'Source text is available; no tests ran in this instance. Behavior predictions are not observations.','unknown':['test outcomes','semantic fulfillment','usefulness of any new meaning']},
      'binding':{'protocol':request['protocol'],'state_root':request['state_root'],'attention_binding':attention['binding'],'relational_binding':r['binding']},
      'authority':{'executions':0,'acceptance_adoption':False,'public_meaning_adoption':'separate independent source-bound review only'},
      'relational_contract':e.relational_contract(request),'relational_history':[],'existing_case_ids':[case['id'] for case in coding['cases']],'candidate_contracts':contracts}
def body(context,stage):
    if stage not in ('develop','next_proposal'):raise ValueError('stage')
    instruction=('Offer at most one public_desire candidate for an existing parent, or abstain. Preserve original intent and distinguish clarification from development. Unknown is not negative. Do not invent feedback or force novelty.' if stage=='develop' else 'Propose useful alternatives through the ordinary native proposal process, using current public meanings and immutable acceptance. Do not force a difference. No candidate will execute or be adopted in this final step.')
    public={'binding':context['binding']['relational_binding'],'identity':'exact active parent id','revises':'exact revision from that parent, or null if absent','parents':'distinct active parent IDs including identity; retain prior parent set for revisions','meaning':'nonempty text <=512 UTF-8 bytes','change':'clarification or development','basis':{'kind':'unknown'},'generator':'gpt-6-luna/public-desire-generator'}
    contracts={'public_desire':public}
    if stage=='next_proposal':contracts.update(context['candidate_contracts'])
    result={'model':'gpt-6-luna','max_completion_tokens':8192,'stream':False,'messages':[{'role':'system','content':instruction+' Return strict JSON {"alternatives":[payload objects]} or {"abstain":"honest bounded reason"}. Each payload has exactly one supplied wrapper: public_desire, relational, experiment, or edits plus rationale as specified. Never mix executable edits with deferred qualitative/experiment alternatives. All meaning/reason strings <=512 UTF-8 bytes.'},{'role':'user','content':e.canonical({'context':context,'payload_contract':contracts,'count':1 if stage=='develop' else '1..4'}).decode()}]}
    if len(json.dumps(result,ensure_ascii=False).encode())>32768:raise ValueError('payload bound')
    return result

def bind(value,request,stage):
    """Use existing native binder for existing kinds; never invent or fix model references."""
    if stage not in ('develop','next_proposal'):raise ValueError('stage')
    if isinstance(value,dict) and set(value)=={'abstain'}:
        text(value['abstain']);return value
    exact(value,['alternatives'],'response envelope')
    items=value['alternatives']
    if not isinstance(items,list) or not 1<=len(items)<=(1 if stage=='develop' else 4):raise ValueError('proposal count')
    response={'protocol':request['protocol'],'state_root':request['state_root'],'attention_binding':request['next_attention']['binding'],'proposals':[]}
    for item in items:
        if isinstance(item,dict) and set(item)=={'public_desire'}:
            p=item['public_desire'];exact(p,['binding','identity','revises','parents','meaning','change','basis','generator'],'public desire fields')
            if p['binding']!=request['coding']['relational']['binding'] or p['basis']!={'kind':'unknown'}:raise ValueError('stale/invented basis')
            parents={x['id']:x for x in request['coding']['relational']['inputs']['parents']}
            if p['identity'] not in parents or p['revises']!=parents[p['identity']].get('revision'):raise ValueError('stale/unknown parent')
            if not isinstance(p['parents'],list) or len(set(p['parents']))!=len(p['parents']) or not set(p['parents'])<=parents.keys() or p['identity'] not in p['parents']:raise ValueError('parent ancestry')
            if p['change'] not in ('clarification','development'):raise ValueError('change type')
            if p['generator']!='gpt-6-luna/public-desire-generator':raise ValueError('generator attribution')
            text(p['meaning']);text(p['generator'])
            response['proposals'].append({'kind':'public_desire','proposal':copy.deepcopy(p)})
        else:
            if stage=='develop':raise ValueError('public desire only')
            e.validate_stage_envelope('final_proposal_only',item)
            e.validate_trial_payload(item,request,'open_choice')
            if 'relational' in item:e.validate_relational_lineage(request,item['relational'],'open_choice')
            part=e.local.bind_proposal(request,json.dumps(item,allow_nan=False))
            offset=len(response['proposals'])
            response['proposals'].extend(part['proposals'])
            for index,probability in part.get('common_event_forecasts',{}).items():
                response.setdefault('common_event_forecasts',{})[str(offset+int(index))]=probability
    executable=[p['kind'] in ('edit_set','exact_edit','existing') for p in response['proposals']]
    if any(executable) and not all(executable):raise ValueError('mixed executable/deferred batch')
    return response
