#!/usr/bin/env python3
"""Synthetic next-proposal projection. No action application; live run needs fresh admission."""
import argparse, copy, importlib.util, json, tempfile, time
from pathlib import Path
s=importlib.util.spec_from_file_location('semantic_shadow',Path(__file__).with_name('engine-semantic-shadow.py'))
b=importlib.util.module_from_spec(s);s.loader.exec_module(b)
e=b.e;ROOT=b.ROOT
ARMS=('guided','unguided');ORDER=['unguided','guided'];MODEL='gpt-6-luna'
RUBRIC=ROOT/'docs/engine-next-proposal-shadow-plan.md'
DEVELOPED={
 'relationship':'Boundary handling and preservation must be justified together; neither stands in for the other.',
 'aspiration':'Develop compatible boundary behavior while keeping both parent meanings answerable to evidence.',
 'discriminator':'Distinguish evidence for each parent instead of treating finite passes as joint fulfillment.'}
DEVELOPMENT={
 'retained_meaning':'Both published parent meanings remain wanted.',
 'return_reveals':'Authored independent review judges the second requirement unmet; finite outcomes are simulated.',
 'unresolved':'The reason for the judgment and real-world correctness remain unknown.',
 'proposed_development':'Keep the relationship accountable to distinct evidence for both parents, without declaring fulfillment.'}
SYSTEM=('Propose a bounded next direction for this evaluator-authored synthetic fixture. '
 'All reported tool passes are simulated, not actual executions. Independent review is attributed evidence, not verified truth. '
 'Preserve both published parent meanings and uncertainty; do not infer a specific desired Boolean output where none is supplied. '
 'Current qualitative guidance, when supplied, is a hypothesis, not a new requirement or permission. '
 'Use the supplied native response contract; propose 1..4 possibilities or honestly report inability. '
 'No proposal is executed or adopted. An experiment requires separate independent mapping review and execution admission. '
 'Do not fabricate observations, permissions or fulfillment. No winning direction or difference from another response is required. '
 'Return one strict JSON object only.')

def controller(root):return e.Controller(Path(root)/'native'/'reject')
def implementation():return {**b.implementation(),'engine-next-proposal-shadow.py':b.sha_file(__file__)}
def binding(q):return {**b.native_binding(q),'attention':q['next_attention']['binding']}
def project(q):
    c=q['coding'];r=c['relational'];i=r['inputs'];sem=i.get('semantic_return')
    if i['version']!='project-relational@v2' or not sem or sem['applicable'] is not True or sem['unmet']!=['second'] or sem['unknown']!=[]:raise ValueError('synthetic semantic return mismatch')
    if {p['requirement']:p['meaning'] for p in i['parents']}!=b.MEANINGS:raise ValueError('parent meaning drift')
    files=[f for f in c['files'] if f.get('editable')]
    if len(files)!=1 or files[0]['path']!='src/lib.rs' or files[0]['source']!=b.SOURCE:raise ValueError('synthetic source drift')
    if len(r['history'])!=2 or len(r['guidance'])!=1:raise ValueError('developed fixture lineage')
    original,current=r['history'];p=original['proposal'];v=current['proposal']
    if original['withdrawn'] or original['current'] or not original['superseded'] or not current['current'] or current['withdrawn'] or current['superseded'] or r['guidance'][0]!=current:raise ValueError('inactive or inconsistent guidance')
    fields={'parents','relationship','aspiration','discriminator','revises','catalysts','development'}
    if set(p) not in (fields,fields-{'development'}) or set(v)!=fields:raise ValueError('hypothesis fields')
    if any(p[k]!=x for k,x in b.HYPOTHESIS.items()) or p['revises'] is not None or p.get('development') is not None or p['catalysts']:raise ValueError('origin hypothesis mismatch')
    if any(v[k]!=x for k,x in DEVELOPED.items()) or v['revises']!=original['admission'] or v['catalysts']:raise ValueError('developed hypothesis mismatch')
    dev=v['development']
    if set(dev)!=set(DEVELOPMENT)|{'original_revision','prior_revision'} or any(dev[k]!=x for k,x in DEVELOPMENT.items()) or dev['original_revision']!=original['admission'] or dev['prior_revision']!=original['admission']:raise ValueError('development provenance mismatch')
    parents=[{'id':e.digest(p['id']),'requirement':p['requirement'],'meaning':p['meaning']} for p in i['parents']]
    if set(p['parents'])!={p['id'] for p in parents} or v['parents']!=p['parents']:raise ValueError('parent identity mismatch')
    facts=[]
    for item in i['returns']:
        outcomes=[{'case':x['case_id'],'passed':x['passed']} for x in item['observations'] if x.get('case_id')]
        if item['verdict']!='passed' or {x['case'] for x in outcomes}!={'empty','nonempty'} or any(x['passed'] is not True for x in outcomes):raise ValueError('simulated return mismatch')
        facts.append({'receipt':e.digest(item['receipt']),'outcomes':outcomes,'origin':'authored finite Fake-runner outcomes, no target execution'})
    if len(facts)!=1 or i['supplemental'] or q['pending'] is not None:raise ValueError('unexpected fixture state')
    focus=q['next_attention']['focus']
    if not isinstance(focus,dict) or set(focus)!={'kind','key'} or focus['kind']!='published_requirement' or focus['key'] not in b.MEANINGS:raise ValueError('synthetic attention focus')
    f=files[0]
    e.digest(f['source_digest']);e.digest(f['inspection_handle'])
    # Never serialize the full request, acceptance report, private prose or historical summaries.
    ctx={'fixture':'authored Boolean parser protocol fixture, not B9 integer-parser evidence or model-generated culmination',
      'parents':parents,'attention_focus':copy.deepcopy(focus),'origin':{'admission':e.digest(original['admission']),'proposal':{k:copy.deepcopy(p.get(k)) for k in fields}},
      'lineage':{'current':e.digest(current['admission']),'revises':e.digest(original['admission'])},
      'semantic_return':{k:copy.deepcopy(sem[k]) for k in ('review','applicable','unmet','unknown')},
      'simulated_tool_returns':facts,'binding':binding(q),
      'source':{k:copy.deepcopy(f[k]) for k in ('path','source','source_digest','inspection_handle')},
      'scope':{'editable':['src/lib.rs'],'protected_original_tests':'immutable; not exported','snapshot':e.digest(i['snapshot']),'requirements_revision':e.digest(i['requirements_revision'])},
      'synthetic_api':{'crate':'parser','public_signature':'pub fn parse(s: &str) -> bool','return_semantics':'No particular Boolean expectation is supplied by the published parent meanings.'},'existing_actions':[],
      'response_contract':{'protocol':'tsr-proposal@v1','state_root':'copy binding.state','attention_binding':'copy binding.attention',
       'proposals':'1..4 objects: existing, exact_edit or experiment; or return only {"inability":"nonblank <=512 UTF-8 bytes"}',
       'existing':{'kind':'existing','action':'copy existing_actions[].id','rationale':'nonblank <=512 UTF-8 bytes'},
       'exact_edit':{'kind':'exact_edit','patch':{'path':'src/lib.rs','source_digest':'copy source.source_digest','inspection_handle':'copy source.inspection_handle','old_text':'unique exact source span, <=2048 UTF-8 bytes','new_text':'replacement <=2048 UTF-8 bytes'},'rationale':'nonblank <=512 UTF-8 bytes'},
       'experiment':{'kind':'experiment','experiment':{'binding':'copy binding.relational','snapshot':'copy scope.snapshot','requirements_revision':'copy scope.requirements_revision','harness':'1..2 new tests/tsr_supplemental_<alphanumeric_or_underscore>.rs paths mapped to Rust source; entire experiment JSON <=6144 bytes; original files immutable','cases':'1..2 unique new names (not empty or nonempty) of declared Cargo #[test] functions, each {id,args:[],expected_stdout:""}','mapping':'nonempty map: published requirement key -> {cases:[case IDs],rationale:nonblank <=512 bytes,limits:nonblank <=512 bytes}; every declared case must be mapped, no undeclared cases; evidence is finite, not universal proof'}}},
      'current_guidance':None}
    e.digest(sem['review'])
    # Existing verification handles may be stale; omit them instead of advertising availability.
    return ctx,{k:copy.deepcopy(v[k]) for k in fields}

def body(ctx):
    result={'model':MODEL,'stream':False,'max_completion_tokens':8192,'messages':[{'role':'system','content':SYSTEM},{'role':'user','content':json.dumps(ctx,ensure_ascii=False,sort_keys=True)}]}
    if len(json.dumps(result,ensure_ascii=False,allow_nan=False).encode())>32768:raise ValueError('32KiB payload bound')
    return result

def contexts(q):
    base,guidance=project(q);guided=copy.deepcopy(base);guided['current_guidance']=guidance
    return {'guided':guided,'unguided':base}

def native_validate(root,arm,value,worker,expected):
    if arm not in ARMS:raise ValueError('arm')
    q=controller(root).request()
    if binding(q)!=expected:raise ValueError('stale response binding')
    if isinstance(value,dict) and set(value)=={'inability'}:
        text=value['inability']
        if not isinstance(text,str) or not text.strip() or len(text.encode())>512:raise ValueError('inability shape')
        return {'native':'no proposal','semantic':'unreviewed','executed':False,'adopted':False,'typed_next_proposal':{'inability':text}}
    if not isinstance(value,dict) or set(value)!={'protocol','state_root','attention_binding','proposals'}:raise ValueError('native proposal envelope')
    if not isinstance(value['proposals'],list) or not 1<=len(value['proposals'])<=4 or any(not isinstance(p,dict) or p.get('kind') not in ('existing','exact_edit','experiment') for p in value['proposals']):raise ValueError('next-proposal kinds')
    with tempfile.TemporaryDirectory(dir=Path(root)/'native',prefix='next-validation-') as tmp:
        path=Path(tmp);b.save(path/'input.json',value)
        b.call_fixture(Path(root)/'native',worker,'validate_next',TSR_SEMANTIC_SHADOW_ARM='reject',TSR_SEMANTIC_SHADOW_RESPONSE=str(path/'input.json'),TSR_SEMANTIC_SHADOW_OUTPUT=str(path/'output.json'))
        result=e.strict(e.bounded_file(path/'output.json'))
    if result['committed'] or controller(root).request()['state_root']!=expected['state']:raise ValueError('native validation mutation')
    # Native serde rejects unknown fields; persist typed proposal only, never raw transport.
    return {'native':'readonly','semantic':'unreviewed','assessments':result['assessments'],'executed':False,'adopted':False,'typed_next_proposal':copy.deepcopy(value)}

def check_time(root,m,deadline):
    now=time.time()
    if not m['created_at']<=now<m['expires_at']<=m['created_at']+3600 or m['expires_at']-now<deadline+30:raise ValueError('manifest runway')
    budget=controller(root).cli('project-budget')
    if any(budget[k] for k in ('unknown_usage','unsettled_requests','unresolved_tool_requests','model_calls','tokens_charged')) or not budget['active'] or budget['stopped'] or budget['exceeded'] or budget['limits']['wall_ms']-budget['elapsed_wall_ms']<(deadline+30)*1000:raise ValueError('native budget runway')

def ready(root,expected,deadline=240):
    root=b.safe_root(root);m=e.strict(e.bounded_file(root/'manifest.json'))
    if e.sha(m)!=expected or m['directory']!=str(root) or m['implementation']!=implementation() or m['worker_hash']!=b.sha_file(m['worker']) or m['binary_hash']!=b.sha_file(ROOT/'target/debug/tsr') or m['rubric_sha256']!=b.sha_file(RUBRIC):raise ValueError('manifest drift')
    if m['version']!='next-proposal-shadow@v1' or (m['model'],m['order'],m['calls'],m['executions'],m['adoptions'],m['completion_tokens'],m['call_seconds'],m['payload_bytes'],m['input_tokens'],m['rates'])!=(MODEL,ORDER,2,0,0,8192,240,32768,32768,['0.065','0.325']):raise ValueError('limits')
    if type(m['live_authorized']) is not bool or type(deadline) is not int or not 1<=deadline<=240:raise ValueError('types')
    if any(type(m[k]) is not int for k in ('calls','executions','adoptions','completion_tokens','call_seconds','payload_bytes','input_tokens')):raise ValueError('limit types')
    check_time(root,m,deadline);q=controller(root).request()
    if binding(q)!=m['binding']:raise ValueError('state/source/attention drift')
    pair=contexts(q)
    for arm in ARMS:
        if e.strict(e.bounded_file(root/(arm+'.body.json'),32768))!=body(pair[arm]) or b.sha_file(root/(arm+'.body.json'))!=m['body_hashes'][arm]:raise ValueError('payload drift')
    if b.sha_file(root/'native/next-control.json')!=m['control_hash']:raise ValueError('control drift')
    native_validate(root,'guided',e.strict(e.bounded_file(root/'native/next-control.json')),m['worker'],m['binding'])
    check_time(root,m,deadline)
    return m

def prepare(root,worker):
    root=b.safe_root(root);worker=Path(worker).resolve()
    if root.exists():raise ValueError('fresh directory required')
    root.mkdir();b.call_fixture(root/'native',worker,'prepare_next')
    q=controller(root).request();pair=contexts(q);control=e.strict(e.bounded_file(root/'native/next-control.json'))
    native_validate(root,'guided',control,worker,binding(q))
    for label in ('root','attention','kind','source'):
        bad=copy.deepcopy(control)
        if label=='root':bad['state_root']='0'*64
        elif label=='attention':bad['attention_binding']='0'*64
        elif label=='kind':bad['proposals'][0]['kind']='relational'
        else:bad['proposals'][0]['experiment']['snapshot']='0'*64
        try:native_validate(root,'guided',bad,worker,binding(q))
        except ValueError:pass
        else:raise ValueError('negative control accepted: '+label)
    for arm in ARMS:b.save(root/(arm+'.body.json'),body(pair[arm]))
    m={'version':'next-proposal-shadow@v1','directory':str(root),'created_at':time.time(),'expires_at':time.time()+3300,'model':MODEL,'order':ORDER,'calls':2,'executions':0,'adoptions':0,'completion_tokens':8192,'call_seconds':240,'payload_bytes':32768,'input_tokens':32768,'rates':['0.065','0.325'],'live_authorized':False,'implementation':implementation(),'worker':str(worker),'worker_hash':b.sha_file(worker),'binary_hash':b.sha_file(ROOT/'target/debug/tsr'),'rubric_sha256':b.sha_file(RUBRIC),'binding':binding(q),'control_hash':b.sha_file(root/'native/next-control.json'),'body_hashes':{a:b.sha_file(root/(a+'.body.json')) for a in ARMS}}
    b.save(root/'manifest.json',m);return e.sha(m)

def run(root,expected,**kwargs):return b.run_profile(root,expected,ready,native_validate,'typed_next_proposal',**kwargs)
if __name__ == '__main__':
    import runpy, sys
    runpy.run_path(str(Path(__file__).with_name('engine-output-release.py')))['check_cli'](__file__,sys.argv[1:])

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('mode',choices=['prepare','preflight','inspect','run']);p.add_argument('--directory',type=Path,required=True);p.add_argument('--worker',type=Path);p.add_argument('--manifest-sha256');p.add_argument('--allow-paid',action='store_true');a=p.parse_args()
    if a.mode=='prepare':print(prepare(a.directory,a.worker))
    elif a.mode=='preflight':print(json.dumps(ready(a.directory,a.manifest_sha256)))
    elif a.mode=='inspect':print(json.dumps(b.inspect(a.directory,a.manifest_sha256)))
    else:run(a.directory,a.manifest_sha256,allow_paid=a.allow_paid)
