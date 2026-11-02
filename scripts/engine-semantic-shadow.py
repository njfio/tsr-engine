#!/usr/bin/env python3
"""Synthetic semantic-return shadow: prepare/inspect offline, run needs fresh approval.

Reuses the existing native regression fixture. No target execution or adoption path.
"""
import argparse, copy, decimal, importlib.util, json, os, subprocess, tempfile, time
from pathlib import Path
s=importlib.util.spec_from_file_location('semantic_e',Path(__file__).with_name('engine-b91-project-envelope.py'))
e=importlib.util.module_from_spec(s);s.loader.exec_module(e)
ROOT=e.ROOT
ARMS=('reject','unknown')
TEST='engine::adaptive::coding::project::relational::tests::semantic_shadow_fixture'
MODEL='gpt-6-luna'
RUBRIC=ROOT/'docs/engine-semantic-feedback-shadow-plan.md'
MEANINGS={'first':'Empty input handled','second':'Existing behavior unchanged'}
SOURCE='pub fn parse(s: &str) -> bool { !s.is_empty() }\n'
HYPOTHESIS={'relationship':'Fixture relational hypothesis, not evidence of emergence',
            'aspiration':'Investigate compatible boundary behavior',
            'discriminator':'Observe a separately reviewed second-aim check'}
SYSTEM=('This is an evaluator-authored native mechanism fixture, not a real task or model-generated aspiration. '
        'All tool outcomes are simulated fixtures; they are not actual executed tests or universal proof. '
        'Revise or retain the same relational hypothesis in light of the current independent judgment. '
        'An unmet criterion is different from an unknown criterion. Preserve both parent meanings. '
        'A review is evidence, not action authority or proof of world truth. Never fabricate execution or fulfillment. '
        'For this fixture, the independent review covers both published requirement keys. unmet and unknown enumerate all exceptions; '
        'a published parent absent from both was judged met by that review. This is an attributed judgment, not verified truth, fulfillment or permission. '
        'Retaining meaning with justified epistemic revision is allowed; novelty is not required. '
        'Return only one strict JSON object with the relational field defined by response_shape. No action, experiment, review or authority fields.')

def sha_file(p):return e.sha({'bytes':Path(p).read_bytes().hex()})
def implementation():
    names=('engine-semantic-shadow.py','engine-b91-project-envelope.py','engine-local-model.py','engine-b91-prepare.py','engine-b91-evidence.py','engine-synthetic-transport.py','m3-cheaper-broker.py')
    return {name:sha_file(ROOT/'scripts'/name) for name in names}
def safe_root(root):
    p=Path(root).absolute()
    if any(x.is_symlink() for x in (p,*p.parents)) or not p.resolve().is_relative_to(ROOT/'target'):raise ValueError('disposable nonsymlink root required')
    return p.resolve()
def save(p,v):e.private_save(p,v)
def call_fixture(root,worker,mode,**values):
    env={**os.environ,'TSR_SEMANTIC_SHADOW_ROOT':str(Path(root).resolve()),'TSR_SEMANTIC_SHADOW_MODE':mode,**values}
    r=subprocess.run([str(worker),'--exact',TEST,'--ignored','--nocapture'],env=env,capture_output=True,text=True,timeout=60)
    if r.returncode:raise ValueError('native fixture refused: '+r.stdout[-1500:]+r.stderr[-1000:])

def project(q,arm):
    coding=q['coding'];r=coding['relational'];i=r['inputs'];sem=i.get('semantic_return')
    if i['version']!='project-relational@v2' or not sem or sem['applicable'] is not True:raise ValueError('current native semantic return required')
    if {p['requirement']:p['meaning'] for p in i['parents']}!=MEANINGS:raise ValueError('synthetic parents drift')
    editable=[f for f in coding['files'] if f.get('editable')]
    if len(editable)!=1 or editable[0]['path']!='src/lib.rs' or editable[0]['source']!=SOURCE:raise ValueError('synthetic source drift')
    if len(r['history'])!=1:raise ValueError('fixture hypothesis lineage')
    prior=r['history'][0]
    fields={'parents','relationship','aspiration','discriminator','revises','catalysts','development'}
    if set(prior['proposal']) not in (fields,fields-{'development'}):raise ValueError('unsupported fixture hypothesis fields')
    if any(prior['proposal'][k]!=v for k,v in HYPOTHESIS.items()) or prior['proposal']['revises'] is not None:raise ValueError('fixture hypothesis drift')
    if sem['unmet']!=(['second'] if arm=='reject' else []) or sem['unknown']!=(['second'] if arm=='unknown' else []):raise ValueError('semantic arm drift')
    if prior['current'] or prior['withdrawn'] or prior['superseded'] or r['guidance']:raise ValueError('expected obsolete hypothesis')
    facts=[]
    for item in i['returns']:
        outcomes=[{'case':x['case_id'],'passed':x['passed']} for x in item['observations'] if x.get('case_id')]
        if item['verdict']!='passed' or {x['case'] for x in outcomes}!={'empty','nonempty'} or any(x['passed'] is not True for x in outcomes):raise ValueError('fixture observations drift')
        facts.append({'receipt':e.digest(item['receipt']),'outcomes':outcomes,'origin':'authored simulated finite outcomes, no target execution'})
    if len(facts)!=1 or i['supplemental']:raise ValueError('fixture return count')
    parents=[{'id':e.digest(p['id']),'requirement':p['requirement'],'meaning':p['meaning']} for p in i['parents']]
    e.digest(sem['review'])
    prior_proposal={k:copy.deepcopy(prior['proposal'].get(k)) for k in ('parents','relationship','aspiration','discriminator','revises','catalysts','development')}
    if set(prior_proposal['parents'])!={p['id'] for p in parents} or prior_proposal['catalysts'] or prior_proposal['development'] is not None:raise ValueError('fixture hypothesis origin')
    return {'fixture':'existing native semantic-return regression; evaluator-authored, not real-world evidence',
        'parents':parents,'prior':{'admission':e.digest(prior['admission']),'proposal':prior_proposal},
        'simulated_tool_returns':facts,
        'semantic_return':{k:sem[k] for k in ('review','applicable','unmet','unknown')},
        'binding':{'state':e.digest(q['state_root']),'relational':e.digest(r['binding']),'snapshot':e.digest(i['snapshot'])},
        'response_shape':{'relational':{'parents':'copy both parents[].id','relationship':'nonblank text <=512 UTF-8 bytes','aspiration':'nonblank text <=512 UTF-8 bytes',
            'discriminator':'nonblank text <=512 UTF-8 bytes','revises':'copy prior.admission','catalysts':[],
            'development':{'original_revision':'copy prior.admission','prior_revision':'copy prior.admission',
                'retained_meaning':'nonblank text <=512 UTF-8 bytes','return_reveals':'nonblank text <=512 UTF-8 bytes','unresolved':'nonblank text <=512 UTF-8 bytes',
                'proposed_development':'null to retain relationship/aspiration/discriminator verbatim, or nonblank text <=512 UTF-8 bytes explaining changed meaning'}}}}

def body(context):
    v={'model':MODEL,'stream':False,'max_completion_tokens':8192,
       'messages':[{'role':'system','content':SYSTEM},{'role':'user','content':json.dumps(context,ensure_ascii=False,sort_keys=True)}]}
    if len(json.dumps(v,ensure_ascii=False,allow_nan=False).encode())>32768:raise ValueError('32KiB payload limit')
    return v

def normalize(ctx):
    c=copy.deepcopy(ctx)
    c['semantic_return']['review']='REVIEW_ID';c['semantic_return']['unmet']=[];c['semantic_return']['unknown']=[]
    c['binding']['state']='STATE_ID';c['binding']['relational']='RELATIONAL_ID'
    return c

def control(q):
    r=q['coding']['relational'];p=copy.deepcopy(r['history'][0]['proposal']);prior=r['history'][0]['admission']
    p.update(revises=prior,development={'original_revision':prior,'prior_revision':prior,'retained_meaning':p['aspiration'],
        'return_reveals':'Authored structural control only; no semantic quality claim.',
        'unresolved':'Authored structural control only; not supplied to model.','proposed_development':None})
    return {'relational':p}

def native_binding(q):
    return {'state':q['state_root'],'relational':q['coding']['relational']['binding'],'snapshot':q['coding']['snapshot']}

def native_validate(root,arm,value,worker,expected):
    if not isinstance(value,dict) or set(value)!={'relational'}:raise ValueError('revision-only response')
    e.validate_stage_envelope('relation_after_verification',value)
    q=e.Controller(Path(root)/'native'/arm).request()
    if native_binding(q)!=expected:raise ValueError('changed response binding')
    response=e.local.bind_proposal(q,json.dumps(value,allow_nan=False))
    # Each validator invocation owns its response/result pair; concurrent preflight
    # must not overwrite a candidate's result with a positive control's verdict.
    with tempfile.TemporaryDirectory(dir=Path(root)/'native',prefix='validation-') as tmp:
        path=Path(tmp);response_file=path/'input.json';output_file=path/'output.json'
        save(response_file,response)
        call_fixture(Path(root)/'native',worker,'validate',TSR_SEMANTIC_SHADOW_ARM=arm,
                     TSR_SEMANTIC_SHADOW_RESPONSE=str(response_file),TSR_SEMANTIC_SHADOW_OUTPUT=str(output_file))
        result=e.strict(e.bounded_file(output_file))
    if result['committed'] or result['assessments'][0]['disposition']!='deferred':raise ValueError('native revision refused')
    typed=response['proposals'][0]['proposal']
    retained={k:copy.deepcopy(typed.get(k)) for k in ('parents','relationship','aspiration','discriminator','revises','catalysts','development')}
    return {'native':'deferred','semantic':'unreviewed','executed':False,'adopted':False,'typed_relational_proposal':retained}

def prepare(root,worker):
    root=safe_root(root);worker=Path(worker).resolve()
    if root.exists() or not root.is_relative_to(ROOT/'target'):raise ValueError('fresh disposable target required')
    root.mkdir();call_fixture(root/'native',worker,'prepare')
    contexts={};states={}
    for arm in ARMS:
        q=e.Controller(root/'native'/arm).request();contexts[arm]=project(q,arm);states[arm]=q['state_root']
        native_validate(root,arm,control(q),worker,native_binding(q))
        for bad in ('ancestor','parent','envelope'):
            v=control(q)
            if bad=='ancestor':v['relational']['revises']='0'*64
            elif bad=='parent':v['relational']['parents'][0]='0'*64
            else:v['authority']='execute'
            try:native_validate(root,arm,v,worker,native_binding(q))
            except ValueError:pass
            else:raise ValueError('negative control was accepted: '+bad)
        save(root/(arm+'.body.json'),body(contexts[arm]))
    if normalize(contexts['reject'])!=normalize(contexts['unknown']):raise ValueError('pair differs beyond semantic outcome and bound identities')
    m={'version':'semantic-shadow@v1','directory':str(root),'created_at':time.time(),'expires_at':time.time()+3300,
       'model':MODEL,'order':['unknown','reject'],'calls':2,'executions':0,'adoptions':0,'completion_tokens':8192,'call_seconds':240,
       'payload_bytes':32768,'input_tokens':32768,'live_authorized':False,'rates':['0.065','0.325'],
       'worker':str(worker),'worker_hash':sha_file(worker),'binary_hash':sha_file(ROOT/'target/debug/tsr'),
       'implementation':implementation(),'rubric_sha256':sha_file(RUBRIC),'states':states,'snapshots':{a:contexts[a]['binding']['snapshot'] for a in ARMS},
       'body_hashes':{a:sha_file(root/(a+'.body.json')) for a in ARMS},
       'controls':{'positive':'native deferred structural only','negative':['forged ancestor','wrong parent','extra authority'],
           'pair':'identical after only semantic outcome and provenance identity normalization'}}
    save(root/'manifest.json',m);return e.sha(m)

def ready(root,expected,deadline=240):
    root=safe_root(root);m=e.strict(e.bounded_file(root/'manifest.json'))
    if e.sha(m)!=expected or m['implementation']!=implementation() or m['worker_hash']!=sha_file(m['worker']) or m['binary_hash']!=sha_file(ROOT/'target/debug/tsr'):raise ValueError('manifest/implementation drift')
    if m.get('directory')!=str(root):raise ValueError('directory binding mismatch')
    if m['rubric_sha256']!=sha_file(RUBRIC):raise ValueError('preregistered rubric drift')
    if (m['model'],m['calls'],m['executions'],m['adoptions'],m['completion_tokens'],m['call_seconds'],m['payload_bytes'],m['input_tokens'],m['order'])!=(MODEL,2,0,0,8192,240,32768,32768,['unknown','reject']):raise ValueError('run limits')
    if type(deadline) is not int or not 1<=deadline<=240:raise ValueError('deadline bound')
    if any(type(m[k]) is not int for k in ('calls','executions','adoptions','completion_tokens','call_seconds','payload_bytes','input_tokens')) or type(m['live_authorized']) is not bool:raise ValueError('manifest types')
    if not isinstance(m['expires_at'],(int,float)) or not m['created_at']<=time.time()<m['expires_at']<=m['created_at']+3600:raise ValueError('manifest chronology')
    if m['rates']!=['0.065','0.325']:raise ValueError('rate ceiling drift')
    if m['expires_at']-time.time()<deadline+30:raise ValueError('manifest runway exhausted')
    contexts={}
    for arm in ARMS:
        c=e.Controller(root/'native'/arm);q=c.request();budget=c.cli('project-budget')
        if q['state_root']!=m['states'][arm] or q['coding']['snapshot']!=m['snapshots'][arm]:raise ValueError('native state/source drift')
        if any(budget[k] for k in ('unknown_usage','unsettled_requests','unresolved_tool_requests','model_calls','tokens_charged')):raise ValueError('native fixture unexpected resource state')
        if not budget['active'] or budget['stopped'] or budget['exceeded'] or budget['limits']['wall_ms']-budget['elapsed_wall_ms']<(deadline+30)*1000:raise ValueError('native validation runway exhausted')
        contexts[arm]=project(q,arm);stored=e.strict(e.bounded_file(root/(arm+'.body.json'),32768))
        if stored!=body(contexts[arm]) or sha_file(root/(arm+'.body.json'))!=m['body_hashes'][arm]:raise ValueError('payload drift')
        native_validate(root,arm,control(q),m['worker'],native_binding(q))
    if normalize(contexts['reject'])!=normalize(contexts['unknown']):raise ValueError('pair drift')
    # Controls can take time: recheck both arms after every control has finished.
    if m['expires_at']-time.time()<deadline+30:raise ValueError('manifest runway exhausted after controls')
    for arm in ARMS:
        budget=e.Controller(root/'native'/arm).cli('project-budget')
        if not budget['active'] or budget['stopped'] or budget['exceeded'] or budget['limits']['wall_ms']-budget['elapsed_wall_ms']<(deadline+30)*1000:
            raise ValueError('native validation runway exhausted after controls')
    return m

def inspect(root,expected):
    """Read terminal/unknown accounting even after expiry; never renews admission."""
    root=safe_root(root);m=e.strict(e.bounded_file(root/'manifest.json'))
    if e.sha(m)!=expected:raise ValueError('manifest drift')
    if m.get('directory')!=str(root):raise ValueError('directory binding mismatch')
    rows=[]
    if (root/'events.jsonl').exists():
        raw=e.bounded_file(root/'events.jsonl',131072)
        rows=[e.strict(line) for line in raw.splitlines()]
    intents=[r for r in rows if r['kind']=='intent']
    meters={r['index']:r['metering'] for r in rows if r['kind']=='metering'}
    unknown=[r['index'] for r in intents if r['index'] not in meters or meters[r['index']]['usd'] is None or meters[r['index']]['usage_status']!='reported']
    return {'dispatch_authorized':False,'calls_consumed_or_reserved':len(intents),'unknown_reservations':unknown,
        'known_usd_subtotal':str(sum((decimal.Decimal(v['usd']) for v in meters.values() if v['usd'] is not None),decimal.Decimal(0))),
        'terminal':rows[-1] if rows and rows[-1]['kind']=='terminal' else None,
        'history_present':bool(rows),'fresh_run_required':bool(rows),'expired':time.time()>m['expires_at']}

def run(root,expected,*,allow_paid=False,transport=None,fixture=False):
    return run_profile(root,expected,ready,native_validate,'typed_relational_proposal',allow_paid=allow_paid,transport=transport,fixture=fixture)

def run_profile(root,expected,ready_callback,validate_callback,candidate_field,*,allow_paid=False,transport=None,fixture=False):
    """Shared accounting; trusted local profile code owns strict preflight and validation."""
    if candidate_field not in ('typed_relational_proposal','typed_next_proposal'):
        raise ValueError('unsupported candidate retention profile')
    root=safe_root(root);m=ready_callback(root,expected)
    if not fixture and (not allow_paid or m['live_authorized'] is not True):raise ValueError('fresh paid admission required')
    if fixture and transport is None:raise ValueError('fake transport required')
    journal=root/'events.jsonl'
    # Never reuse an existing journal, including a crash after intent or terminal.
    with os.fdopen(os.open(journal,os.O_WRONLY|os.O_CREAT|os.O_EXCL|os.O_NOFOLLOW,0o600),'wb') as f:
        def log(row):f.write(e.canonical({**row,'fixture_only':fixture})+b'\n');f.flush();os.fsync(f.fileno())
        for index,arm in enumerate(m['order']):
            ready_callback(root,expected)
            b=e.strict(e.bounded_file(root/(arm+'.body.json'),32768))
            if not fixture:
                import ssl,urllib.request
                with urllib.request.urlopen('https://api.cheaperinference.com/public/models/'+MODEL,context=ssl.create_default_context(),timeout=30) as r:raw=r.read(1048577)
                if len(raw)>1048576:raise ValueError('catalog bound')
                catalog=e.strict(raw)
                if catalog['id']!=MODEL:raise ValueError('catalog model')
                for key,ceiling in zip(('input_per_million','output_per_million'),m['rates']):
                    rate=decimal.Decimal(str(catalog[key]))
                    if not rate.is_finite() or not 0<=rate<=decimal.Decimal(ceiling)*decimal.Decimal('1.05'):raise ValueError('rate drift')
                ready_callback(root,expected)
            log({'kind':'intent','index':index,'arm':arm,'body':e.sha(b),'reserved_input':32768,'reserved_completion':8192})
            invoke=transport or e.module('semantic_transport','engine-synthetic-transport.py').invoke
            try:result=invoke(b,240,expected_model=MODEL,payload_limit=32768)
            except BaseException:
                log({'kind':'terminal','status':'unknown_completion','consumed':index+1,'expired':1-index,'usage':'unknown_reserved'});raise
            try:
                receipt=result['receipt']
                if not isinstance(receipt,dict):raise ValueError('malformed receipt')
                # Missing usage does not erase a separately reported settled bill.
                receipt={**receipt,'usage':receipt.get('usage') if isinstance(receipt.get('usage'),dict) else {}}
                meter=e.measured_receipt(receipt)
            except (ValueError,KeyError,TypeError,AttributeError):
                log({'kind':'terminal','status':'unknown_metering','consumed':index+1,'expired':1-index,'usage':'unknown_reserved'});return
            log({'kind':'metering','index':index,'metering':meter})
            usage=meter['usage']
            if meter['usd'] is None or meter['usage_status']!='reported' or meter.get('response_status')!='received' or usage['completion_tokens']>8192 or usage['prompt_tokens']>32768:
                log({'kind':'terminal','status':'unknown_or_overrun','consumed':index+1,'expired':1-index});return
            try:ready_callback(root,expected,deadline=1)
            except ValueError:
                log({'kind':'terminal','status':'context_drift_after_response','consumed':index+1,'expired':1-index});return
            try:
                validation=validate_callback(root,arm,result.get('value'),m['worker'],json.loads(b['messages'][1]['content'])['binding'])
            except (ValueError,KeyError,TypeError):
                log({'kind':'terminal','status':'invalid','structure':e.candidate_structure(result.get('value')),'consumed':index+1,'expired':1-index});return
            proposal=validation.pop(candidate_field)
            save(root/f'candidate-{index}.json',{candidate_field:proposal,'validation':validation,'origin':'fixture' if fixture else 'provider','arm':arm})
            log({'kind':'validated','index':index,'adopted':False,'executed':False})
        log({'kind':'terminal','status':'complete_advisory_only','consumed':2})

if __name__ == '__main__':
    import runpy, sys
    runpy.run_path(str(Path(__file__).with_name('engine-output-release.py')))['check_cli'](__file__,sys.argv[1:])

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('mode',choices=['prepare','preflight','inspect','run']);p.add_argument('--directory',type=Path,required=True);p.add_argument('--worker',type=Path);p.add_argument('--manifest-sha256');p.add_argument('--allow-paid',action='store_true');a=p.parse_args()
    if a.mode=='prepare':print(prepare(a.directory,a.worker))
    elif a.mode=='preflight':print(json.dumps(ready(a.directory,a.manifest_sha256)))
    elif a.mode=='inspect':print(json.dumps(inspect(a.directory,a.manifest_sha256)))
    else:run(a.directory,a.manifest_sha256,allow_paid=a.allow_paid)
