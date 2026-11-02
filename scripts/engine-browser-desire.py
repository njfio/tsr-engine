"""Same-identity public meaning: native validation and separate owned review.

No new preference, criterion, permission, fulfillment or replacement of user input.
Only this session's independently admitted meanings may enter its public projection.
"""
import copy
import hashlib
import json

GENERATOR='gpt-6-luna/browser-public-meaning'

def sha(x):
    return hashlib.sha256(json.dumps(x,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()

def contract():
    return {'envelope':'{public_desire:{identity,revises,parents,meaning,change,basis}}; one advisory candidate alone',
        'identity':'a p alias in public_meaning_references.eligible_contributors; original and generated identities retain immutable origin and obligations',
        'revises':'matching d alias of that parent, or null for first development',
        'parents':'distinct eligible contributor aliases including identity; preserve immutable generated ancestry and the same parent set on revision',
        'meaning':'nonblank UTF-8 text, within the existing browser aggregate 131072-byte response envelope; no extra field cap (not the provider maximum)',
        'change':'clarification|development; paraphrase is not emergence',
        'basis':'{kind:unknown} or {kind:observation,receipt:o alias} or {kind:judgment,review:j alias} when supplied. Judgment is attributed native assessment, never observed truth. Observation must be an actual supplied native receipt, never an attributed assessment.',
        'authority':'Transport owns binding and generator; omit both. Independent review precedes admission. Original acceptance parents and latest operator guidance remain authoritative. This adds a reviewed meaning, not satisfaction, new identity or permission.'}

def history_snapshot(inputs):
    """Validate the native projection envelope, without authenticating its content.

    Legacy/no-development requests omit public_desires. A present malformed
    envelope must never be treated as empty history. Ownership remains separate.
    """
    if 'public_desires' not in inputs:return None
    public=inputs['public_desires']
    if not isinstance(public,dict):raise ValueError('public meaning history: object required')
    for field in ('history','active'):
        if not isinstance(public.get(field),list):raise ValueError('public meaning history: '+field+' array required')
        if any(not isinstance(row,dict) for row in public[field]):raise ValueError('public meaning history: '+field+' entries must be objects')
    return public

def public_history(inputs):
    public=history_snapshot(inputs)
    if public is None:return []
    # A projection is not permission to export arbitrary stored review text.
    return [{k:copy.deepcopy(h[k]) for k in ('revision','previous','next','parent_origins','superseded','withdrawn') if k in h}
            for h in public['history']]

def verify_history(adapter,request):
    inputs=request['coding']['relational']['inputs'];public=history_snapshot(inputs)
    pins=adapter.admission['origin']['public'].get('public_meanings',[])
    if not isinstance(pins,list) or len(pins)!=len(set(pins)) or any(not isinstance(x,str) or len(x)!=64 for x in pins):raise ValueError('public meaning origin pins')
    if public is None:
        if pins:raise ValueError('imported public meaning history missing')
        return
    matched=set();known={}
    for key,value in adapter.read()[2].items():
        if key.startswith('meaning_admission_'):
            for item in value.get('public',{}).get('history',[]):
                known[item.get('revision')]=item
    for item in public['history']:
        if 'revision' not in item or 'next' not in item:raise ValueError('unowned public meaning status')
        expected=known.get(item['revision'])
        # Supersession is a native fold; admitted content must remain byte-identical.
        strip=lambda h:{k:v for k,v in h.items() if k not in ('superseded','withdrawn')}
        imported={sha({**item,'superseded':flag}) for flag in (False,True)} & set(pins)
        matched.update(imported)
        if ((expected is None or strip(item)!=strip(expected)) and not imported) or item.get('withdrawn') is not False:
            raise ValueError('unowned public meaning history')
    if matched!=set(pins):raise ValueError('imported public meaning lineage missing')
    if public['active']!=inputs['parents']:raise ValueError('public meaning active mismatch')
    # Native reconstruction owns active/current semantics; assert exact content came
    # from admitted history or unchanged original acceptance input, never a new parent.
    originals={p['id']:p for p in inputs['acceptance_parents']}
    for h in public['history']:
        origins=[h.get('previous',{}),*h.get('parent_origins',[])]
        for previous in origins:
            if 'original_contributors' not in previous and previous.get('id'):originals.setdefault(previous['id'],previous)
    originals.update({p['id']:p for p in inputs.get('generated_preferences',[]) if p['eligible']})
    latest={h['next']['id']:h['next'] for h in public['history']}
    for p in inputs['parents']:
        if p['id'] not in originals or p.get('requirement')!=originals[p['id']].get('requirement'):raise ValueError('public meaning requirement drift')
        if p.get('revision') and p!=latest.get(p['id']):raise ValueError('public meaning content drift')
        if not p.get('revision') and p['meaning']!=originals[p['id']]['meaning']:raise ValueError('unadmitted public meaning')

def wire(context,request,refs):
    if 'public_desire' not in context['response_contracts']:return
    p={v:k for k,v in refs['parents'].items()};d={v:k for k,v in refs['desire_revisions'].items()}
    for item in context['parents']:
        if item.get('revision'):item['revision']=d[item['revision']]
        if 'parents' in item:item['parents']=[p[x] for x in item['parents']]
    # History is explicit attribution; historical IDs remain immutable provenance,
    # not selectable aliases. Only active identities/revisions are proposal choices.
    context['public_meaning_references']={'eligible_originals':[p[x['id']] for x in request['coding']['relational']['inputs'].get('acceptance_parents',request['coding']['relational']['inputs']['parents']) if x['id'] in p], 'eligible_contributors':list(refs['parents']), 'current_revisions':refs['desire_revisions'],'observations':refs['observation_receipts'],'judgments':refs['meaning_judgments']}
    context['public_meaning_policy']='Current generated meanings do not supersede latest operator guidance in acceptance_parents. If later guidance conflicts, propose an independently reviewed development or retain uncertainty; do not silently discard either history. Meaning admission is not a completed return cycle.'

def expand(value,request,refs):
    if not isinstance(value,dict) or set(value)!={'identity','revises','parents','meaning','change','basis'}:raise ValueError('public meaning model fields; host binding/generator forbidden')
    def resolve(mapping,key):
        if not isinstance(key,str) or key not in mapping:raise ValueError('unknown/stale public meaning reference')
        return mapping[key]
    p=copy.deepcopy(value)
    p['identity']=resolve(refs['parents'],p['identity'])
    if not isinstance(p['parents'],list):raise ValueError('public meaning parents')
    p['parents']=[resolve(refs['parents'],x) for x in p['parents']]
    if p['revises'] is not None:p['revises']=resolve(refs['desire_revisions'],p['revises'])
    if isinstance(p['basis'],dict) and p['basis'].get('kind')=='observation':
        p['basis']['receipt']=resolve(refs['observation_receipts'],p['basis'].get('receipt'))
    if isinstance(p['basis'],dict) and p['basis'].get('kind')=='judgment':
        p['basis']['review']=resolve(refs['meaning_judgments'],p['basis'].get('review'))
    p['binding']=request['coding']['relational']['binding'];p['generator']=GENERATOR
    return p

def bind(request,p):
    if not isinstance(p,dict) or set(p)!={'identity','revises','parents','meaning','change','basis','binding','generator'}:raise ValueError('public meaning fields')
    inputs=request['coding']['relational']['inputs'];parents={x['id']:x for x in inputs['parents']}
    if p['binding']!=request['coding']['relational']['binding'] or p['generator']!=GENERATOR:raise ValueError('public meaning source/attribution')
    old=parents.get(p['identity'])
    if old is None or p['revises']!=old.get('revision'):raise ValueError('stale public meaning identity/revision')
    ids=p['parents']
    if not isinstance(ids,list) or any(not isinstance(x,str) for x in ids) or not 1<=len(ids)<=len(parents) or len(ids)!=len(set(ids)) or not set(ids)<=parents.keys() or p['identity'] not in ids:raise ValueError('public meaning ancestry')
    if p['revises'] is None and 'original_contributors' in old and not set(old.get('preference_parents',old['parents']))<=set(ids):raise ValueError('public meaning must preserve generated ancestry')
    if p['revises'] is not None and set(ids)!=set(old['parents']):raise ValueError('public meaning parent loss')
    if not isinstance(p['meaning'],str) or not p['meaning'].strip() or len(p['meaning'].encode())>131072 or p['meaning'].strip()==old['meaning']:raise ValueError('native public meaning text contract')
    if p['change'] not in ('clarification','development'):raise ValueError('public meaning change')
    basis=p['basis']
    if isinstance(basis,dict) and basis.get('kind')=='judgment':
        semantic=inputs.get('semantic_return',{})
        if set(basis)!={'kind','review'} or semantic.get('applicable') is not True or basis['review']!=semantic.get('review'):raise ValueError('unadmitted public meaning judgment')
    elif basis!={'kind':'unknown'}:
        if not isinstance(basis,dict) or set(basis)!={'kind','receipt'} or basis['kind']!='observation' or basis['receipt'] not in {r['receipt'] for r in inputs.get('returns',[])}:raise ValueError('unadmitted public meaning evidence')
    return {'protocol':request['protocol'],'state_root':request['state_root'],'attention_binding':request['next_attention']['binding'],'proposals':[{'kind':'public_desire','proposal':copy.deepcopy(p)}]}

def admit(adapter,value,result):
    # Exact owned review was verified before the single atomic native advance.
    if not (result.get('meaning_admitted') or result.get('preference_admitted')) or result.get('review_required'):raise ValueError('public meaning missing atomic admission')
    record=result['proposal_record'];key='meaning_admission_'+record
    def capture():
        request=adapter.controller.request();inputs=request['coding']['relational']['inputs']
        if result.get('preference_admitted'):
            history=inputs['generated_preferences'];matches=[h for h in history if h['admission']==result['preference_admission']]
            if len(matches)!=1 or matches[0]['proposal']!=record or matches[0]['review']!=value['review']:raise ValueError('generated preference native receipt mismatch')
            return {'advance_result':copy.deepcopy(result),'advance_input':copy.deepcopy(value),'event':result['preference_admission'],'proposal_record':record,'review':copy.deepcopy(value['review']),'generated_preferences':history,'parents':copy.deepcopy(inputs['parents']),'source_snapshot':request['coding']['snapshot']}
        history=inputs['public_desires']['history']
        matches=[h for h in history if h.get('revision')==result['meaning_admission']]
        if len(matches)!=1 or matches[0]['next']['proposal']!=record or matches[0]['next']['reviewer']!=value['review']['reviewer']:raise ValueError('public meaning native receipt mismatch')
        return {'advance_result':copy.deepcopy(result),'advance_input':copy.deepcopy(value),'event':result['meaning_admission'],'proposal_record':record,'review':copy.deepcopy(value['review']),'public':inputs['public_desires'],'source_snapshot':request['coding']['snapshot']}
    pending=adapter.read()[1]
    if pending:
        if set(pending)!={key}:raise ValueError('unrelated unresolved effect; no meaning recovery')
        # Only this read-only capture is recoverable, after exact native receipt proof.
        proof=adapter.controller.json_cli(('project','cycle','meaning-advance-receipt'),value)
        if proof!=result:raise ValueError('meaning capture recovery receipt drift')
        adapter.append({'kind':'result','name':key,'intent':sha(pending[key]),'value':capture()})
    else:adapter.op(key,capture)
    return result
