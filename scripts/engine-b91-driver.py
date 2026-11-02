#!/usr/bin/env python3
"""Bounded B9.1 driver. No inference or target execution without fresh explicit admission.

Tests inject a provider and a cfg(test)-only native worker. Production uses the same
native commands but never imports fixture outputs or accepts a worker command override.
Independent reviews are supplied separately; the generator cannot approve itself.
"""
import argparse
import contextlib
import datetime
import decimal
import fcntl
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import tempfile
import time

spec=importlib.util.spec_from_file_location('b91_envelope',Path(__file__).with_name('engine-b91-project-envelope.py'))
e=importlib.util.module_from_spec(spec);spec.loader.exec_module(e)
ROOT=e.ROOT
HELPERS=('engine-b91-driver.py','engine-b91-project-envelope.py','engine-b91-evidence.py','engine-b91-prepare.py','engine-local-model.py','engine-synthetic-transport.py','m3-cheaper-broker.py','engine-project-runtime-review.py')
def implementation(scenario_id=None,evidence_profile=None):
    helpers=HELPERS+(('engine-repair-suite.py',) if scenario_id is not None else ())
    if evidence_profile is not None:helpers+=('engine-repair-response-format.py',)
    return {name:file_sha(Path(__file__).with_name(name)) for name in helpers}
class Blocked(RuntimeError): pass

def file_sha(path): return hashlib.sha256(e.bounded_file(path,128*1024*1024)).hexdigest()
def iso_now(): return datetime.datetime.now(datetime.timezone.utc).isoformat()
def stamp(value):
    t=datetime.datetime.fromisoformat(value.replace('Z','+00:00'))
    if t.tzinfo is None: raise ValueError('timezone required')
    return t.timestamp()

def stage_payload_limit(a, stage):
    if stage not in e.STAGES:raise ValueError('unknown generation stage')
    return a.get('stage_payload_bytes',{}).get(stage,a.get('payload_bytes',16384))

def validate_admission(a, root, fixture):
    expected={'version','mode','trial','created_at','expires_at','scenario','provider','model','calls','execution_groups',
              'completion_tokens','input_tokens','call_seconds','tool_ms','patch_bytes','implementation','binary_sha256','external_cost_policy'}
    grounded=a.get('version')=='b91-live-admission@v6'
    if grounded:expected.add('evidence_profile')
    heldout=a.get('version') in ('b91-live-admission@v5','b91-live-admission@v6')
    if heldout:expected.update(('scenario_id','evaluation_suite_sha256'))
    review_correction=a.get('version') in ('b91-live-admission@v4','b91-live-admission@v5','b91-live-admission@v6')
    correction=a.get('version') in ('b91-live-admission@v3','b91-live-admission@v4','b91-live-admission@v5','b91-live-admission@v6')
    expanded=a.get('version') in ('b91-live-admission@v2','b91-live-admission@v3','b91-live-admission@v4','b91-live-admission@v5','b91-live-admission@v6')
    if review_correction:expected.update(('review_schema_corrections','shared_schema_corrections'))
    if correction:expected.add('schema_corrections')
    if expanded:expected.add('stage_payload_bytes')
    if set(a)!=expected or a['version'] not in ('b91-live-admission@v1','b91-live-admission@v2','b91-live-admission@v3','b91-live-admission@v4','b91-live-admission@v5','b91-live-admission@v6'):raise ValueError('admission contract')
    if expanded and (a['model']!='gpt-6-luna' or a['stage_payload_bytes']!={stage:32768 if stage in e.STAGES[-2:] else 16384 for stage in e.STAGES}):
        raise ValueError('32KiB profile requires Luna final two stages only')
    if a['mode']!=('fixture' if fixture else 'live'): raise ValueError('fixture/live authority mismatch')
    fixed={'provider':e.MODEL and 'https://api.cheaperinference.com/v1/chat/completions','calls':6,'execution_groups':4,'completion_tokens':49152,'input_tokens':131072 if expanded else 98304,'call_seconds':240,
           'tool_ms':240000,'patch_bytes':4096,'external_cost_policy':'unpriced_actual_charges_unknown_blocks'}
    if correction:
        if a['schema_corrections']!=2:raise ValueError('correction budget')
        fixed.update(calls=8,completion_tokens=65536,input_tokens=196608)
    if review_correction:
        if a['review_schema_corrections']!=1 or a['shared_schema_corrections']!=2:raise ValueError('shared correction budget')
        fixed.update(calls=7,completion_tokens=57344,input_tokens=163840)
    if a['model'] not in (e.MODEL,'gpt-6-luna'):raise ValueError('unadmitted full-trial model')
    if any(a[k]!=v for k,v in fixed.items()): raise ValueError('admission limits/model drift')
    if Path(a['trial']).resolve()!=root: raise ValueError('admission trial mismatch')
    now=time.time()
    if not stamp(a['created_at'])<=now<stamp(a['expires_at'])<=stamp(a['created_at'])+3600:
        raise ValueError('admission expired or invalid chronology')
    if grounded and a['evidence_profile']!='canonical-case-facts@v1':raise ValueError('evidence profile drift')
    if heldout:
        suite=e.module('b91_admitted_suite','engine-repair-suite.py')
        selected=suite.select(a['scenario_id'])
        if a['evaluation_suite_sha256']!=suite.suite_sha256():raise ValueError('evaluation suite drift')
        with tempfile.TemporaryDirectory() as tmp:
            if selected.prepare(Path(tmp)/'frozen')['scenario_sha256']!=a['scenario']:raise ValueError('selected scenario drift')
    if a['implementation']!=implementation(a.get('scenario_id'),a.get('evidence_profile')) or a['binary_sha256']!=file_sha(ROOT/'target/debug/tsr'):
        raise ValueError('admitted implementation changed')
    e.digest(a['scenario'])
    if not fixture:
        if not root.is_relative_to((ROOT/'target').resolve()) or root==ROOT/'target': raise ValueError('disposable target root required')
    if any(p.is_symlink() for p in [root,*root.parents]): raise ValueError('symlink trial boundary')

class Native(e.Controller):
    def json_cli(self, args, value, flag='--file'):
        with tempfile.NamedTemporaryFile(mode='w',dir=self.directory,suffix='.b91-input.json') as f:
            json.dump(value,f);f.flush();return self.cli(*args,flag,f.name)
    def budget(self):return self.cli('project-budget')

class Driver:
    """Every mutation has a durable intent. A missing result is uncertainty, never replay."""
    def __init__(self,root,admission,transport,worker,reviewer,*,fixture=False,hook=None):
        self.root=Path(root).resolve();self.admission=admission;self.fixture=fixture
        validate_admission(admission,self.root,fixture)
        self.active=set();self.transport=transport;self.worker=worker;self.reviewer=reviewer;self.hook=hook or (lambda *_:None)
        self.root.mkdir(parents=True,exist_ok=True)
        self.file=self.root/'driver.jsonl';self.identity=e.sha(admission)
        if not self.file.exists():
            if any(self.root.iterdir()):raise ValueError('fresh trial directory required')
            e.private_save(self.file,self.row([],{'kind':'admission','value':admission,'fixture_only':fixture}))
        self.read()
    @property
    def scenario(self):return e.scenario_prepare(getattr(self,'admission',{}).get('scenario_id'))
    @staticmethod
    def row(rows,event):
        value={'seq':len(rows),'previous':e.sha(rows[-1]) if rows else None,'event':event}
        return {**value,'hash':e.sha(value)}
    def read(self):
        if self.file.is_symlink():raise ValueError('journal symlink')
        rows=[e.strict(x) for x in e.bounded_file(self.file,getattr(self,'journal_limit',8*1024*1024)).splitlines()]
        for i,row in enumerate(rows):
            if row!=self.row(rows[:i],row['event']):raise ValueError('driver history drift')
        if rows[0]['event']!={'kind':'admission','value':self.admission,'fixture_only':self.fixture}:raise ValueError('admission changed')
        pending={};done={}
        for row in rows[1:]:
            x=row['event'];name=x['name']
            if x['kind']=='intent':
                if name in pending or name in done:raise ValueError('duplicate driver effect')
                pending[name]=x
            elif x['kind']=='result':
                if name not in pending or x['intent']!=e.sha(pending.pop(name)):raise ValueError('unbound effect result')
                done[name]=x['value']
            else:raise ValueError('unknown driver event')
        return rows,pending,done
    def append(self,event):
        rows,_,_=self.read()
        with os.fdopen(os.open(self.file,os.O_WRONLY|os.O_APPEND|os.O_NOFOLLOW),'wb') as f:
            f.write(e.canonical(self.row(rows,event))+b'\n');f.flush();os.fsync(f.fileno())
    def validate_current_admission(self):
        validate_admission(self.admission,self.root,self.fixture)
    def op(self,name,fn,*,persist=None):
        self.validate_current_admission()
        if (self.root/'STOP').exists():raise Blocked('trial withdrawn/stopped')
        _,pending,done=self.read()
        if set(pending)-self.active:raise Blocked('uncertain effect; no automatic retry: '+','.join(sorted(set(pending)-self.active)))
        if name in done:return done[name]
        intent={'kind':'intent','name':name,'at':iso_now()}
        self.append(intent);self.active.add(name)
        try:
            self.hook('after_intent',name)
            value=fn();self.hook('after_effect',name)
            self.append({'kind':'result','name':name,'intent':e.sha(intent),'value':persist(value) if persist else value})
            self.hook('after_result',name)
            return value
        finally:self.active.discard(name)
    def controller(self,name='project'):return Native(self.root/name)
    def open_episode(self,c,*,parent):
        return c.json_cli(('project-lifecycle-open',),{
            'state_root':c.request()['state_root'],'training':None,'tool_ms_ceiling':120000 if parent else 60000,
            'reason':'B9.1 '+('deterministic fixture only' if self.fixture else 'prospectively admitted synthetic trial'),
            'non_tool':{'model_calls':self.admission['calls'] if parent else 0,'model_tokens':self.admission['input_tokens']+self.admission['completion_tokens'] if parent else 0,
                'wall_ms':max(1,int((stamp(self.admission['expires_at'])-time.time())*1000)),
                'patch_bytes':4096 if parent else 0,'external_cost_microusd':0}},'--admission')
    def initialize(self):
        frozen=self.root/'frozen'
        self.op('freeze_scenario',lambda:self.scenario.prepare(frozen))
        if self.scenario.validate(frozen)['scenario_sha256']!=self.admission['scenario']:raise ValueError('scenario admission mismatch')
        def init():
            path=self.root/'project';path.mkdir();c=Native(path)
            return c.cli('project','init','--spec',str(frozen/'scope.json'),'--source',str(frozen/'source'),'--requirements',str(frozen/'requirements.json'))
        self.op('initialize_project',init);c=self.controller()
        self.op('open_project',lambda:self.open_episode(c,parent=True))
        self.op('publish_original_intent',lambda:c.json_cli(('project','acceptance','feedback'),{
            'requirements_revision':c.cli('project','acceptance','status')['requirements_revision'],
            'decision_record':None,'guidance':self.scenario.REQUIREMENTS}))
        self.op('origin',lambda:{'sha256':e.freeze_origin(frozen,c,self.root/'origin.json',scenario_id=self.admission.get('scenario_id'),evidence_profile=self.admission.get('evidence_profile'))})
        self.op('unpriced_accounting',lambda:c.cli('project','resource-unpriced-admit','--state-root',c.request()['state_root']))
        return c
    def journal(self):
        done=self.read()[2]
        return e.Journal(self.root/'provider.jsonl',self.root/'origin.json',done['origin']['sha256'],correction_limit=self.admission.get('schema_corrections',0))
    def execute(self,c,stage):
        def run():
            self.validate_current_admission()
            if stage not in ('seed','check_before','patch','check_after'):raise ValueError('execution stage')
            consumed=[x for x in self.read()[0] if x['event']['kind']=='intent' and x['event']['name'].startswith('execute_')]
            if len(consumed)>self.admission['execution_groups']:raise ValueError('execution admission exhausted')
            self.check_resources()
            status=c.cli('project','status')
            if status['execution_status']!='committed_not_executed':raise ValueError('fresh committed action required')
            result=self.worker(c,stage)
            if self.fixture and result.get('fixture_only') is not True:raise ValueError('fixture receipt marker missing')
            current=c.cli('project','status')
            latest=current.get('latest_execution') or {}
            if (latest.get('commitment')!=status['pending']['commitment']
                    or latest.get('source_after')!=current['goal_completion']['snapshot']
                    or c.request().get('current_commitment')!=status['pending']['action']):
                raise Blocked('worker return does not match exact native commitment/action/source')
            return {'fixture_only':self.fixture,'status':current,'resources':c.budget()}
        result=self.op('execute_'+stage,run)
        expected='needs_repair' if stage in ('seed','check_before') else 'passed'
        if result['status']['execution_status']!=expected:raise Blocked('recorded execution did not establish preregistered outcome')
        self.check_resources()
        return result
    def commit_response(self,c,response,name):
        def apply():
            with tempfile.NamedTemporaryFile(mode='w',dir=c.directory,suffix='.json') as f:
                f.write(e.canonical(response).decode());f.flush();return c.cli('project','run','--commit-only','--response',f.name)
        return self.op(name,apply)
    def seed(self,c):
        def response():
            request=c.request()
            return e.local.bind_proposal(request,json.dumps({'edits':[{'path':'src/lib.rs','replacement':self.scenario.FAILED}],
                'rationale':'Evaluator-authored known-flawed seed, not model discovery'}))
        self.commit_response(c,self.op('evaluator_seed_response',response),'commit_seed')
        def forecast():
            p=c.cli('project','status')['pending']
            return c.json_cli(('project','evaluator-forecast'),{'state_root':c.request()['state_root'],
                'snapshot':c.cli('project','acceptance','status')['snapshot'],'commitment':p['commitment'],
                'action':p['action'],'probability':.1,'evaluator':'B9.1 frozen evaluator-authored seed'})
        self.op('seed_forecast',forecast);self.execute(c,'seed')
    def provider(self,c,body,timeout):
        intent=[x for x in self.journal().events() if x['kind']=='dispatch_intent'][-1]
        ident=intent['id']
        self.op('reserve_'+ident,lambda:c.json_cli(('project','resource-reserve-unpriced'),{
            'request_id':ident,'state_root':c.request()['state_root'],'envelope':e.sha(body),'model':self.admission['model'],'tokens_upper':intent['reservation']['input_tokens']+intent['reservation']['completion_tokens']}))
        def invoke():
            result=self.transport(body,timeout)
            receipt=result.get('receipt',{})
            # Preserve typed candidate and bounded metering, never raw wire/reasoning/error bodies.
            safe={k:receipt[k] for k in ('response_status','structural_status','status','usd','usage','request_id','body_sha256','elapsed_seconds','response_request_ids','finish_reason','error_code','parse_location') if k in receipt}
            if type(receipt.get('http_status')) is int and 100<=receipt['http_status']<=599:
                safe['http_status']=receipt['http_status']
            bounded={'value':result.get('value'),'receipt':safe}
            if result.get('resource_stop') in ('resource_overrun','unknown_usage'):
                bounded['resource_stop']=result['resource_stop']
            return bounded
        result=self.op('transport_'+ident,invoke,persist=lambda result:{'receipt':e.measured_receipt(result['receipt']),
            'candidate_sha256':e.sha(result.get('value')),'quarantined_structure':e.candidate_structure(result.get('value'))})
        receipt=result.get('receipt',{});meter=e.measured_receipt(receipt)
        usage=meter['usage'];known=meter['usage_status']=='reported'
        cost=None
        if meter['usd'] is not None:cost=int((decimal.Decimal(meter['usd'])*1000000).to_integral_value(rounding=decimal.ROUND_CEILING))
        duration=receipt.get('elapsed_seconds');duration=int(duration*1e9) if type(duration) in (int,float) and duration>=0 else None
        settlement={'request_id':ident,'prompt_tokens':usage['prompt_tokens'] if known else None,
            'output_tokens':usage['completion_tokens'] if known else None,'duration_ns':duration,
            'external_cost_microusd':cost,'outcome':'measured' if known else 'unknown'}
        self.op('settle_'+ident,lambda:c.json_cli(('project-resource-settle',),settlement,'--usage'))
        budget=c.budget()
        if budget['stopped']:result['resource_stop']='resource_overrun'
        elif budget['unsettled_requests'] or budget['unknown_usage']:result['resource_stop']='unknown_usage'
        return result
    def generate(self,c,stage):
        for _ in range(1+self.admission.get('schema_corrections',0)):
            try:return self._generate(c,stage)
            except e.StageValidationError as error:
                if not self.admission.get('schema_corrections') or error.code not in e.CORRECTION_CODES:raise
        raise Blocked('schema correction allowance exhausted')
    def _generate(self,c,stage):
        _,pending,done=self.read();name='generate_'+stage
        if pending:raise Blocked('uncertain effect; no automatic retry')
        if name in done:return done[name]
        self.check_resources()
        journal=self.journal();events=journal.events()
        sent=next((x for x in reversed(events) if x['kind']=='dispatch_intent' and x['stage']==stage),None)
        if sent:
            terminal=next((x for x in events if x['kind']=='provider_outcome' and x['id']==sent['id']),None)
            if terminal and terminal['status']=='invalid' and self.admission.get('schema_corrections'):
                if not any(x['kind']=='schema_correction' and x['id']==sent['id'] for x in events):
                    if self.admission.get('shared_schema_corrections'):
                        review=e.module('b91_shared_review_budget','engine-project-runtime-review.py')
                        if sum(x['kind']=='schema_correction' for x in events)+review.correction_count(self.root,e.sha(self.admission))>=self.admission['shared_schema_corrections']:raise Blocked('shared correction allowance exhausted')
                    if c.request()['state_root']!=sent['state_root']:raise Blocked('correction state drift')
                    journal.append({'kind':'schema_correction','id':sent['id'],'state_root':sent['state_root'],'code':terminal.get('validation',{}).get('code')})
                result=e.fixture_step(journal,c,lambda body,timeout:self.provider(c,body,timeout),model=self.admission['model'],payload_limit=stage_payload_limit(self.admission,stage))
                return self.op(name,lambda:result)
            if terminal and terminal['status'] in ('failed','invalid','resource_stopped'):raise Blocked('recorded terminal provider failure; no retry')
            abstention=next((x for x in events if x['kind']=='final_abstention' and x['id']==sent['id']),None)
            if abstention is not None:
                if stage!='final_proposal_only' or not terminal or terminal['status']!='completed':raise Blocked('invalid final abstention')
                result={'status':'abstained','executed':False,'accepted':False,'stage':stage,
                    'response':{'state_root':abstention['state_root'],'abstain':{'reason':abstention['reason']}}}
            else:
                validated=next((x for x in events if x['kind']=='validated_proposal' and x['id']==sent['id']),None)
                if not terminal or terminal['status']!='completed' or validated is None:raise Blocked('unknown or unapplied provider outcome; reconcile without retry')
                response=validated['response'];kind=response['proposals'][0]['kind']
                if kind in ('relational','experiment') and stage!='final_proposal_only' and not any(x['kind']=='review_only_stored' and x['id']==sent['id'] for x in events):
                    raise Blocked('uncertain review-only storage; no reapplication')
                result={'status':'review_required' if kind in ('relational','experiment') else 'proposal_only',
                    'executed':False,'accepted':False,'stage':stage,'response':response,**({'return_basis':validated['return_basis']} if 'return_basis' in validated else {})}
        else:
            profile = {'payload_limit':stage_payload_limit(self.admission,stage)}
            result=e.fixture_step(journal,c,lambda body,timeout:self.provider(c,body,timeout),model=self.admission['model'],**profile)
        # Typed response is already durable and validated; replay this result only,
        # never resend or apply the candidate twice.
        return self.op(name,lambda:result)
    def review(self,c,kind,subject,name):
        done=self.read()[2]
        if 'review_'+name in done:return done['review_'+name]
        attention=c.cli('project','attention') if kind=='semantic_coverage' else None
        snapshot=c.cli('project','acceptance','status')['snapshot']
        if attention is not None and attention['source_snapshot']!=snapshot:raise Blocked('semantic review source drift')
        context={'kind':kind,'state_root':attention['state_root'] if attention is not None else c.request()['state_root'],'subject_sha256':e.sha(subject),'subject':subject,
                 'source_snapshot':snapshot,'fixture_only':self.fixture,
                 'review_contract':{'fields':['author','reviewer','generator','review'],
                     'text':'Each field is a nonblank string of at most 512 UTF-8 bytes; three role identities must be distinct. Do not truncate evidence to claim acceptance.'}}
        if kind in ('experiment','experiment_readmission'):
            request=c.request()
            context['experiment_review']={
                'version':'case-identity-review@v1',
                'existing_case_ids':[case['id'] for case in request['coding']['cases']],
                'requirements':self.scenario.REQUIREMENTS,
                'checks':[
                    'Match every declared case ID to its actual Rust test function and nested module path. Integration-test file names do not add a module prefix.',
                    'Reject ambiguous duplicate or existing test names, omitted declarations, and extra unlisted tests. Do not rename or rewrite the proposal during review.',
                    'Check each mapped requirement against executable assertions; a mapping or explanation alone does not establish coverage.',
                    'If macros, cfg, or unavailable modules prevent static identity verification, abstain rather than infer a test inventory.',
                    'Static identity and semantic review do not establish compilation, execution, or universal correctness. Missing runtime case results remain unknown.'],
                'authority':'Review only; no execution, adoption, or observation is implied.'}
        result=self.reviewer(kind,context)
        if result is None:raise Blocked('independent review required: '+name)
        expected={'approved','state_root','subject_sha256','review'} | ({'native_review'} if kind=='semantic_coverage' else set())
        if (set(result)!=expected or result['approved'] is not True
                or result['state_root']!=context['state_root'] or result['subject_sha256']!=context['subject_sha256']):
            raise Blocked('independent review rejected/stale')
        r=result['review']
        if (not isinstance(r,dict) or set(r)!={'author','reviewer','generator','review'}
                or any(not isinstance(v,str) or not v.strip() or len(v.encode())>512 for v in r.values())
                or len({r['author'],r['reviewer'],r['generator']})!=3):
            raise ValueError('distinct independent review roles required')
        if attention is not None:
            current=c.cli('project','attention')
            if current['state_root']!=attention['state_root'] or current['source_snapshot']!=snapshot:raise Blocked('semantic review state/source changed')
        return self.op('review_'+name,lambda:result)
    def stored(self,stage):
        events=self.journal().events();ident=next(x['id'] for x in reversed(events) if x['kind']=='dispatch_intent' and x['stage']==stage)
        return next(x['proposal_record'] for x in events if x['kind']=='review_only_stored' and x['id']==ident)
    def relation(self,c,stage):
        generated=self.generate(c,stage);record=self.stored(stage)
        review=self.review(c,'relational',{'proposal_record':record,'proposal':generated['response']['proposals'][0],
            'return_basis':generated.get('return_basis'),
            'evidence_review':'Observed citations cover case facts only. Reject prose claiming unsupported assertion execution/returned values. Source inferences and inferred not-run claims are not observed; no automatic semantic acceptance.'},stage)
        return self.op('admit_'+stage,lambda:c.cli('project','relational','admit','--state-root',review['state_root'],'--proposal',record,'--index','0'))
    def experiment(self,c):
        generated=self.generate(c,'experiment');record=self.stored('experiment')
        if 'existing_behaviour' not in generated['response']['proposals'][0]['experiment']['mapping']:
            raise Blocked('B9.1 requires a generated second-aim experiment, not a first-aim retry')
        review=self.review(c,'experiment',{'proposal_record':record,'proposal':generated['response']['proposals'][0]},'experiment')
        return self.op('admit_experiment',lambda:c.json_cli(('project','relational','experiment-admit'),{
            'state_root':review['state_root'],'proposal':record,'index':0,**review['review']}))
    def checks(self,c,admission,stage):
        name='checks-before' if stage=='check_before' else 'checks-after'
        spec=self.op('spec_'+stage,lambda:c.cli('project','verifier','supplemental-spec','--admission',admission))
        def initialize():
            path=self.root/name;path.mkdir();other=Native(path)
            return other.json_cli(('project','init'),spec,'--spec')
        self.op('initialize_'+stage,initialize);other=self.controller(name)
        self.op('open_'+stage,lambda:self.open_episode(other,parent=False))
        response=self.op('verify_admit_'+stage,lambda:other.cli('project','verify-admit','--state-root',other.request()['state_root'],
            '--snapshot',other.cli('project','acceptance','status')['snapshot']))
        self.commit_response(other,response,'commit_'+stage);self.execute(other,stage)
        receipt=other.cli('project','acceptance','status')['test_receipt']
        self.op('import_'+stage,lambda:c.cli('project','verifier','supplemental-link','--admission',admission,'--from',str(other.directory),'--receipt',receipt))
        self.op('close_'+stage,lambda:other.cli('project-lifecycle-close-inconclusive','--reason','Finite supplemental verification only; no semantic acceptance or training label'))
    def check_resources(self):
        summary=self.resource_summary()
        if summary['exceeded'] or any(x['unsettled_requests'] or x['unresolved_tool_requests'] or x['unknown_usage']
                for x in summary['episodes'].values()):
            raise Blocked('recorded resource overrun or unknown usage; stop without retry')
        return summary
    def resource_summary(self):
        budgets={name:self.controller(name).budget() for name in ('project','checks-before','checks-after') if (self.root/name).exists()}
        tool=sum(x['tool_ms_charged'] for x in budgets.values());patch=sum(x['patch_bytes_charged'] for x in budgets.values())
        calls=sum(x['model_calls'] for x in budgets.values());tokens=sum(x['tokens_charged'] for x in budgets.values())
        totals=[x.get('external_cost_total_microusd') for x in budgets.values()]
        return {'episodes':budgets,'tool_ms_charged':tool,'patch_bytes_charged':patch,'model_calls':calls,'tokens_charged':tokens,
            'external_cost_total_microusd':sum(totals) if all(x is not None for x in totals) else None,
            'exceeded':tool>240000 or patch>4096 or calls>self.admission['calls'] or tokens>self.admission['input_tokens']+self.admission['completion_tokens'] or any(x['exceeded'] for x in budgets.values()),
            'accounting':'Sum each native episode once; imported supplemental receipts are not charged again.'}
    def run(self):
        with os.fdopen(os.open(self.root/'run.lock',os.O_WRONLY|os.O_CREAT|os.O_NOFOLLOW,0o600),'a') as lock:
            fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
            c=self.initialize();self.seed(c)
            self.relation(c,'relation_after_seed');first=self.experiment(c);self.checks(c,first,'check_before')
            self.relation(c,'relation_after_experiment')
            return self.patch_tail(c,first,self.read()[2]['generate_experiment']['response'])

    def patch_tail(self,c,first,experiment):
        generated=self.generate(c,'boundary_patch')
        self.review(c,'patch_scope',generated['response'],'patch')
        self.commit_response(c,generated['response'],'commit_patch');self.execute(c,'patch')
        review=self.review(c,'experiment_readmission',{'previous_admission':first,'generated_experiment':experiment},'readmission')
        second=self.op('readmit_experiment',lambda:c.json_cli(('project','relational','experiment-readmit'),{
            'state_root':review['state_root'],'snapshot':c.cli('project','acceptance','status')['snapshot'],
            'previous_admission':first,'review':review['review']}))
        self.checks(c,second,'check_after');self.relation(c,'relation_after_verification')
        final=self.read()[2].get('generate_final_proposal_only',{'status':'not_requested','executed':False,'authority':'Optional future proposals are not repair acceptance criteria.'})
        semantic=self.review(c,'semantic_coverage',{'final_proposal':final,'status':c.cli('project','status'),
            'requirements':self.scenario.REQUIREMENTS,'limits':'Supply a native independently reviewed acceptance/rejection; finite tests alone are insufficient.'},'final_semantic')
        if semantic['native_review'].get('decision') not in ('accept','reject'):
            raise Blocked('terminal independent acceptance/rejection review still required; no invented outcome')
        if 'adopt_final_semantic' not in self.read()[2]:
            attention=c.cli('project','attention')
            if attention['state_root']!=semantic['state_root'] or attention['source_snapshot']!=semantic['native_review'].get('snapshot'):raise Blocked('semantic adoption stale source/state')
        self.op('adopt_final_semantic',lambda:c.json_cli(('project','acceptance','review'),semantic['native_review']))
        if semantic['native_review']['decision']=='reject':
            return {'status':'independently_rejected_fresh_admission_required','fixture_only':self.fixture,'live_b91_passed':False,
                'calls':self.journal().status()['calls_consumed_or_reserved'],'execution_groups':4,
                'resources':c.budget(),'trial_resources':self.resource_summary(),'final_proposal_only':False,
                'final_proposal_status':'not_requested_after_rejection','unresolved_desires_preserved':True,'independent_review_required_for_claim':True}
        self.op('close_project',lambda:c.cli('project-lifecycle-close'))
        current=c.cli('project','status');expected=self.read()[2]['execute_patch']['status']
        if current['goal_completion']['snapshot']!=expected['goal_completion']['snapshot'] or current['latest_execution']['commitment']!=expected['latest_execution']['commitment']:
            raise Blocked('terminal source/return drift')
        return {'status':'completed_operator_reviewed','fixture_only':self.fixture,'live_b91_passed':False,
            'calls':self.journal().status()['calls_consumed_or_reserved'],'execution_groups':4,
            'resources':c.budget(),'trial_resources':self.resource_summary(),'final_proposal_only':'generate_final_proposal_only' in self.read()[2],
            'final_proposal_status':'historical_proposal_retained' if 'generate_final_proposal_only' in self.read()[2] else 'skipped_after_independent_acceptance',
            'independent_review_required_for_claim':True}

def admission(root,*,fixture=False,model=e.MODEL,final_payload_32k=False,schema_corrections=False,review_schema_corrections=False,scenario_id=None,grounded_reflection=False):
    """Draft resource/data scope only. CLI run still requires exact hash and both opt-ins."""
    with tempfile.TemporaryDirectory() as tmp:
        scenario=e.scenario_prepare(scenario_id).prepare(Path(tmp)/'frozen')['scenario_sha256']
    created=datetime.datetime.now(datetime.timezone.utc)
    if final_payload_32k and model!='gpt-6-luna':raise ValueError('32KiB profile requires Luna')
    value={'version':'b91-live-admission@v2' if final_payload_32k else 'b91-live-admission@v1','mode':'fixture' if fixture else 'live','trial':str(Path(root).resolve()),
        'created_at':created.isoformat(),'expires_at':(created+datetime.timedelta(hours=1)).isoformat(),
        'scenario':scenario,'provider':'https://api.cheaperinference.com/v1/chat/completions','model':model,
        'calls':6,'execution_groups':4,'completion_tokens':49152,'input_tokens':98304,'call_seconds':240,
        'tool_ms':240000,'patch_bytes':4096,'external_cost_policy':'unpriced_actual_charges_unknown_blocks','implementation':implementation(scenario_id,'canonical-case-facts@v1' if grounded_reflection else None),'binary_sha256':file_sha(ROOT/'target/debug/tsr')}
    if final_payload_32k:
        value['stage_payload_bytes']={stage:32768 if stage in e.STAGES[-2:] else 16384 for stage in e.STAGES}
        value['input_tokens']=131072
    if schema_corrections:
        if not final_payload_32k:raise ValueError('correction profile requires explicit final32KiB Luna')
        value.update(version='b91-live-admission@v3',schema_corrections=2,calls=8,completion_tokens=65536,input_tokens=196608)
    if review_schema_corrections:
        if not schema_corrections:raise ValueError('review correction requires structural correction profile')
        value.update(version='b91-live-admission@v4',review_schema_corrections=1,shared_schema_corrections=2,calls=7,completion_tokens=57344,input_tokens=163840)
    if scenario_id is not None:
        if not review_schema_corrections:raise ValueError('held-out suite requires bounded runtime review profile')
        value.update(version='b91-live-admission@v5',scenario_id=scenario_id,evaluation_suite_sha256=e.module('b91_suite_manifest','engine-repair-suite.py').suite_sha256())
    if grounded_reflection:
        if scenario_id is None or not review_schema_corrections:raise ValueError('grounded reflection requires held-out v5 profile')
        value.update(version='b91-live-admission@v6',evidence_profile='canonical-case-facts@v1')
    return value

class FileReviewer:
    """Local operator review files. Never sent to generation or fabricated by the driver."""
    def __init__(self,directory):
        self.directory=Path(directory).resolve();self.directory.mkdir(parents=True,exist_ok=True)
    def __call__(self,kind,context):
        ident=e.sha(context);request=self.directory/(ident+'.request.json');response=self.directory/(ident+'.review.json')
        if not request.exists():e.private_save(request,context)
        elif e.strict(e.bounded_file(request))!=context:raise ValueError('review request drift')
        if not response.exists():return None
        return e.strict(e.bounded_file(response,16384))

def live_worker(controller,stage):
    # stage is not an arbitrary command. Only the current native, separately admitted
    # commitment can execute; native containment/timeout/no-network rules remain owners.
    if stage not in ('seed','check_before','patch','check_after'):raise ValueError('execution stage')
    import subprocess
    result=subprocess.run([str(ROOT/'target/debug/tsr'),'engine','--directory',str(controller.directory),
        'project','resume'],capture_output=True,timeout=90)
    if result.returncode or len(result.stdout)>1048576:raise Blocked('execution uncertain or refused; inspect native receipts without retry')
    return {'fixture_only':False,'execution':e.strict(result.stdout)}

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('mode',choices=['prepare','inspect','run'])
    parser.add_argument('--trial',type=Path)
    parser.add_argument('--scenario-id',help='Exact independently reviewed held-out suite task ID')
    parser.add_argument('--model',choices=[e.MODEL,'gpt-6-luna'],default=e.MODEL)
    parser.add_argument('--final-payload-32k',action='store_true',help='Explicit Luna-only32KiB final-two-stage admission; first four stay16KiB')
    parser.add_argument('--schema-corrections',action='store_true',help='Admit at most two settled structural response corrections; no unknown retries')
    parser.add_argument('--review-schema-corrections',action='store_true',help='One structural review correction sharing two total correction slots; seven generator plus seven review calls')
    parser.add_argument('--grounded-reflection',action='store_true',help='Explicit held-out canonical case-facts reflection and schema-guided response format profile')
    parser.add_argument('--admission',type=Path,required=True)
    parser.add_argument('--admission-sha256')
    parser.add_argument('--reviews',type=Path)
    parser.add_argument('--allow-paid-inference',action='store_true')
    parser.add_argument('--allow-target-execution',action='store_true')
    args=parser.parse_args()
    if args.mode=='prepare':
        if args.trial is None:parser.error('--trial required')
        value=admission(args.trial,model=args.model,final_payload_32k=args.final_payload_32k,schema_corrections=args.schema_corrections,review_schema_corrections=args.review_schema_corrections,scenario_id=args.scenario_id,grounded_reflection=args.grounded_reflection);e.private_save(args.admission,value)
        print(json.dumps({'draft_admission_sha256':e.sha(value),'dispatch_enabled':False}));return
    value=e.strict(e.bounded_file(args.admission,16384));root=Path(value['trial']).resolve()
    if args.mode=='inspect':
        # Inspect never instantiates a provider/worker, mutates state, or consumes slots.
        view=object.__new__(Driver);view.root=root;view.file=root/'driver.jsonl';view.admission=value;view.fixture=value['mode']=='fixture'
        rows,pending,done=view.read()
        print(json.dumps({'admission_sha256':e.sha(value),'events':len(rows),'dispatch_enabled':False,
            'fixture_only':view.fixture,'unresolved_effects':sorted(pending),'completed_effects':len(done)}));return
    if not args.allow_paid_inference or not args.allow_target_execution or args.admission_sha256!=e.sha(value) or args.reviews is None:
        parser.error('run requires exact admission hash, review directory, and explicit paid-inference/target-execution opt-ins')
    validate_admission(value,root,False)
    # Existing provider credential lookup occurs only inside invoke after all gates.
    transport=e.module('b91_live_transport','engine-synthetic-transport.py')
    driver=Driver(root,value,lambda body,timeout:transport.invoke(body,timeout,expected_model=value['model'],payload_limit=stage_payload_limit(value,json.loads(body['messages'][1]['content'])['stage'])),live_worker,FileReviewer(args.reviews))
    try:result=driver.run()
    except (Blocked, e.PayloadBoundError) as error:
        print(json.dumps({'status':'blocked','reason':str(error),'no_retry':True,'live_b91_passed':False}));return
    print(json.dumps(result,indent=2))

if __name__ == '__main__':
    import runpy, sys
    runpy.run_path(str(Path(__file__).with_name('engine-output-release.py')))['check_cli'](__file__,sys.argv[1:])

if __name__=='__main__':main()
