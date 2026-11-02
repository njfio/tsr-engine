#!/usr/bin/env python3
"""Synthetic experiment shadow runner. Fresh structural validator, no execution/adoption.

prepare and inspect are offline. run requires a separately approved manifest hash and
explicit paid opt-in. Qwen remains blocked until its total-token control is verified.
"""
import argparse, copy, datetime, decimal, hashlib, importlib.util, json, os
from pathlib import Path
import shutil
import time

spec=importlib.util.spec_from_file_location('shadow_envelope',Path(__file__).with_name('engine-b91-project-envelope.py'))
e=importlib.util.module_from_spec(spec);spec.loader.exec_module(e)
ROOT=e.ROOT
APPROVED_INPUT_HASH='43d10bb1141f4279d12947c5ec0b126ba04bebd02207bfb4665dfb6a6191e7df'
MODELS=('deepseek-v4.1-flash','glm-5.3-flash','qwen3.8-flash')
ADMITTED_MODELS=MODELS+('deepseek-v4.1-flash-1017','gpt-6-luna')
QUOTES=dict(zip(MODELS,(('0.0825','0.3300'),('0.0825','0.2750'),('0.0975','0.3055'))))
QUOTES.update({'deepseek-v4.1-flash-1017':('0.075','0.300'),'gpt-6-luna':('0.065','0.325')})
def model_body(body,model):
    result=dict(body,model=model)
    if model=='gpt-6-luna':result['max_completion_tokens']=result.pop('max_tokens')
    return result
def file_sha(path):return hashlib.sha256(Path(path).read_bytes()).hexdigest()
def save(path,value):
    with open(path,'xb') as f:f.write(e.canonical(value)+b'\n');f.flush();os.fsync(f.fileno())
def now():return datetime.datetime.now(datetime.timezone.utc).isoformat()
def implementation():
    paths=[Path(__file__),ROOT/'target/debug/tsr',*[ROOT/'scripts'/n for n in ('engine-b91-project-envelope.py','engine-b91-prepare.py','engine-b91-evidence.py','engine-local-model.py','engine-synthetic-transport.py','m3-cheaper-broker.py')]]
    return {str(p.resolve()):file_sha(p) for p in paths}
def frozen_input():
    root=ROOT/'target/b91-oracle-shadow';manifest=root/'manifest.json'
    if file_sha(manifest)!=APPROVED_INPUT_HASH:raise ValueError('unapproved historical projection')
    m=json.loads(manifest.read_text())
    for name,h in [('request.json',m['request_sha256']),('baseline.json',m['body_hashes']['baseline'])]:
        if file_sha(root/name)!=h:raise ValueError('historical projection drift')
    return json.loads((root/'request.json').read_text()),json.loads((root/'baseline.json').read_text())

class ValidationOnly:
    def __init__(self,root):
        self.root=Path(root).resolve();self.binding=json.loads((self.root/'validation.json').read_text())
        self.c=e.Controller(self.root/'validator')
    @classmethod
    def create(cls,root,original):
        root=Path(root);root.mkdir(parents=True,exist_ok=False)
        # Copy only known evaluator-authored synthetic source. No original ledger replay.
        source=original['coding']['files'];editable=[x for x in source if x.get('editable')]
        if len(editable)!=1 or editable[0]['path']!='src/lib.rs' or editable[0]['source']!=e.prepare.FAILED:
            raise ValueError('not the admitted synthetic source')
        frozen=root/'synthetic';e.prepare.prepare(frozen)
        validation_source=root/'validation-source'
        shutil.copytree(frozen/'source',validation_source)
        (validation_source/'src/lib.rs').write_text(editable[0]['source'])
        # This is a fresh validation copy, explicitly not the original frozen scenario.
        directory=root/'validator';directory.mkdir();c=e.Controller(directory)
        c.cli('project','init','--spec',str(frozen/'scope.json'),'--source',str(validation_source),'--requirements',str(frozen/'requirements.json'))
        status=c.cli('project','acceptance','status')
        feedback=root/'requirements-guidance.json'
        save(feedback,{'requirements_revision':status['requirements_revision'],'decision_record':None,'guidance':e.prepare.REQUIREMENTS})
        c.cli('project','acceptance','feedback','--file',str(feedback))
        q=c.request()
        save(root/'validation.json',{'version':'validation-only@v1','original_request':e.sha(original),
             'original_snapshot':original['coding']['snapshot'],'source_sha256':hashlib.sha256(editable[0]['source'].encode()).hexdigest(),
             'state_root':q['state_root'],'snapshot':q['coding']['snapshot'],'requirements_revision':q['coding']['relational']['inputs']['requirements_revision'],
             'execution_authority':False,'adoption_authority':False,'created_at':now(),'expires_at':time.time()+3600})
        return cls(root)
    def ready(self,deadline=240):
        if type(deadline) is not int or not 1<=deadline<=240:raise ValueError('invalid deadline')
        b=self.binding
        if b['execution_authority'] is not False or b['adoption_authority'] is not False:raise ValueError('authority drift')
        if b['expires_at']-time.time()<deadline+30:raise ValueError('validation wall runway exhausted')
        q=self.c.request();budget=self.c.cli('project-budget')
        # No native resource/execution admission is created. The shadow run owns its
        # own finite validation deadline; proposals only call the readonly validator.
        if budget.get('active') is not False or budget.get('stopped') is not False or budget.get('exceeded') is not False:
            raise ValueError('validation-only resource context changed')
        for key in ('model_calls','tokens_charged','unknown_usage','unsettled_requests','unresolved_tool_requests','patch_bytes_charged','tool_ms_charged'):
            if type(budget.get(key)) is not int or budget[key]!=0:raise ValueError('validation-only side effect')
        if q['state_root']!=b['state_root'] or q['coding']['snapshot']!=b['snapshot']:raise ValueError('validation source/state drift')
        files=[x for x in q['coding']['files'] if x.get('editable')]
        if len(files)!=1 or hashlib.sha256(files[0]['source'].encode()).hexdigest()!=b['source_sha256']:raise ValueError('validation source drift')
        return q
    def validate(self,value):
        q=self.ready(1)
        e.validate_stage_envelope('experiment',value);e.validate_trial_payload(value,q,'experiment')
        bound=e.local.bind_proposal(q,json.dumps(value));verdict=self.c.validate(bound)
        if verdict['assessments'][0]['disposition']!='deferred':raise ValueError('not deferred')
        self.ready(1)
        return {'experiment':bound['proposals'][0]['experiment'],'provenance':self.binding,'semantic_status':'unreviewed'}
    def control(self):
        # Intentionally vacuous STRUCTURAL control, never supplied to the generator.
        value={'experiment':{'harness':{'tests/tsr_supplemental_control.rs':'#[test] fn shadow_control(){assert!(true);}'},
               'cases':[{'id':'shadow_control','args':[],'expected_stdout':''}],
               'mapping':{'existing_behaviour':{'cases':['shadow_control'],'rationale':'Structural readiness control only.','limits':'Vacuous; no semantic acceptance.'}}}}
        self.validate(value)
        return {'structural_control':'deferred','semantic_acceptance':False,'executed':False}

def prepare(root,models=MODELS):
    root=Path(root).resolve()
    if not root.is_relative_to(ROOT/'target') or root.exists():raise ValueError('fresh disposable target root required')
    original,body=frozen_input();validator=ValidationOnly.create(root,original)
    body.pop('temperature',None)
    for model in models:
        if model not in ADMITTED_MODELS:raise ValueError('model not admitted')
        save(root/(model+'.json'),model_body(body,model))
    control=validator.control()
    m={'version':'shadow-comparison@v2','created_at':now(),'expires_at':time.time()+3300,'order':list(models)+list(reversed(models)),
       'calls':2*len(models),'executions':0,'adoptions':0,'call_seconds':240,'completion_tokens':8192,'quotes':{x:QUOTES[x] for x in models},
       'bodies':{x:file_sha(root/(x+'.json')) for x in models},'validation_sha256':file_sha(root/'validation.json'),
       'implementation':implementation(),'control':control,'live_authorized':False}
    save(root/'manifest.json',m);return e.sha(m)

def run(root,expected,*,allow_paid=False,transport=None,fixture=False):
    root=Path(root).resolve();m=json.loads((root/'manifest.json').read_text())
    if e.sha(m)!=expected or m['implementation']!=implementation():raise ValueError('admission/implementation drift')
    if (root/'events.jsonl').exists():raise ValueError('existing dispatch journal; reconciliation only')
    if not fixture:
        if not allow_paid or m['live_authorized'] is not True:raise ValueError('fresh live admission required')
        if 'qwen3.8-flash' in m['order']:raise ValueError('Qwen total-token control unverified')
    elif transport is None:raise ValueError('fixture requires explicit fake transport')
    if m['executions']!=0 or m['adoptions']!=0 or m['call_seconds']!=240 or m['completion_tokens']!=8192 or m['calls']!=len(m['order']) or not 1<=m['calls']<=6:
        raise ValueError('invalid run limits')
    if any(x not in ADMITTED_MODELS for x in m['order']):raise ValueError('unapproved model')
    if file_sha(root/'validation.json')!=m['validation_sha256']:raise ValueError('validation binding drift')
    for model,h in m['bodies'].items():
        if file_sha(root/(model+'.json'))!=h:raise ValueError('payload drift')
    original,approved_body=frozen_input();approved_body.pop('temperature',None)
    validator=ValidationOnly(root)
    if validator.binding['original_request']!=e.sha(original):raise ValueError('original source binding drift')
    def log(value):
        with open(root/'events.jsonl','ab') as f:f.write(e.canonical(dict(value,fixture_only=fixture))+b'\n');f.flush();os.fsync(f.fileno())
    for index,model in enumerate(m['order']):
        if m['expires_at']-time.time()<270:raise ValueError('run wall runway exhausted')
        validator.ready(240);validator.control() # Actual path before every irreversible dispatch.
        body=json.loads((root/(model+'.json')).read_text())
        if body!=model_body(approved_body,model):raise ValueError('non-allowlisted payload')
        if body['model']!=model or body.get('max_completion_tokens' if model=='gpt-6-luna' else 'max_tokens')!=8192 or body['stream'] is not False or len(json.dumps(body).encode())>16384:raise ValueError('body contract')
        if not fixture:
            import ssl,urllib.request
            tls=ssl.create_default_context()
            if tls.verify_mode!=ssl.CERT_REQUIRED or not tls.check_hostname:raise ValueError('TLS verification required')
            with urllib.request.urlopen('https://api.cheaperinference.com/public/models/'+model,timeout=30) as r:raw=r.read(1048577)
            if len(raw)>1048576:raise ValueError('catalog bound')
            catalog=json.loads(raw)
            for key,quote in zip(('input_per_million','output_per_million'),m['quotes'][model]):
                rate=decimal.Decimal(catalog[key])
                if not rate.is_finite() or not 0<=rate<=decimal.Decimal(quote)*decimal.Decimal('1.05'):raise ValueError('rate drift')
            if catalog['id']!=model:raise ValueError('catalog identity')
            # Recheck after metadata latency, before creating dispatch intent.
            validator.ready(240)
        if m['expires_at']-time.time()<270:raise ValueError('run wall runway exhausted after preflight')
        log({'kind':'intent','index':index,'model':model,'body_sha256':e.sha(body)})
        invoke=transport or e.module('shadow_transport','engine-synthetic-transport.py').invoke
        result=invoke(body,240,expected_model=model);meter=e.measured_receipt(result['receipt']);value=result.get('value')
        outcome={'kind':'outcome','index':index,'metering':meter,'structure':e.candidate_structure(value)}
        stage='envelope'
        try:
            e.validate_stage_envelope('experiment',value);stage='trial_payload';e.validate_trial_payload(value,validator.ready(1),'experiment')
            stage='native_validation';candidate=validator.validate(value)
            save(root/('candidate-'+str(index)+'.json'),candidate);outcome['validation']='deferred_not_semantically_accepted'
        except (ValueError,KeyError,TypeError) as error:
            outcome.update(validation='rejected',failed_stage=stage,error_type=type(error).__name__)
            if isinstance(error,e.StageValidationError):outcome['error_code']=error.code
        log(outcome)
        usage=meter['usage']
        if meter['usd'] is None or meter['usage_status']!='reported' or meter.get('response_status')!='received' or usage['completion_tokens']>8192 or usage['prompt_tokens']>16384:
            log({'kind':'terminal','reason':'unknown_or_overrun','consumed':index+1,'expired':m['calls']-index-1});return
    log({'kind':'terminal','reason':'complete','consumed':m['calls']})

if __name__ == '__main__':
    import runpy, sys
    runpy.run_path(str(Path(__file__).with_name('engine-output-release.py')))['check_cli'](__file__,sys.argv[1:])

if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('mode',choices=['prepare','inspect','run']);p.add_argument('--directory',required=True);p.add_argument('--manifest-sha256');p.add_argument('--allow-paid',action='store_true');a=p.parse_args()
    if a.mode=='prepare':print(prepare(a.directory))
    elif a.mode=='inspect':print(json.dumps(ValidationOnly(a.directory).control()))
    else:run(a.directory,a.manifest_sha256,allow_paid=a.allow_paid)
