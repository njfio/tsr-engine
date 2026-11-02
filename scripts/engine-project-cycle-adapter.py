#!/usr/bin/env python3
"""Explicit synthetic-only transport/accounting callbacks for the native open-choice loop.

No default launcher, initialization, acceptance, supplemental execution or automatic retry.
The caller supplies a separately reviewed origin/admission and existing campaign ledger.
"""
import copy
import decimal
import hashlib
import json
from pathlib import Path
import time
import subprocess
import importlib.util

def load(name,file):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
loop=load('open_cycle','engine-project-cycle.py');d=loop.d;e=d.e
review_contract=load('open_review','engine-project-runtime-review.py')
MODEL='gpt-6-luna'
output_mode=load('project_json_output','engine-json-output.py')
bound_output=load('bound_output','engine-bound-output.py')
correction=load('project_correction','engine-bounded-correction.py')
transport_module=load('project_output_transport','engine-synthetic-transport.py')
capacity=load('project_request_capacity','engine-request-capacity.py')

def pins():
    return {**d.implementation('singleton-clamp'),**{n:d.file_sha(Path(__file__).with_name(n)) for n in ('engine-partial-execution.py','engine-execution-not-started.py','engine-generation-quarantine.py','engine-stage-receipts.py','engine-request-capacity.py','engine-bound-output.py','engine-b91-campaign.py','engine-native-cycle.py','engine-project-cycle-adapter.py','engine-project-cycle.py','engine-project-runtime-review.py','engine-b91-project-envelope.py','engine-b91-evidence.py','engine-local-model.py','engine-synthetic-transport.py','engine-repair-suite.py','engine-json-output.py','engine-bounded-correction.py','engine-response-evidence.py','engine-project-output.py','engine-pursuit-packet.py','engine-sequential-packet.py')}}

def admission(root,origin,campaign,expires_at,*,max_decisions=4,max_executions=4,local_advisory=False,release=False,correction_allowance=0,information_admission=None,representation_regeneration=False):
    e.validate_origin(origin)
    value={'version':'synthetic-project-cycle-adapter@v1','directory':str(Path(root).resolve()),'origin':origin,
        'campaign_sha256':e.sha(campaign),'model':MODEL,'max_calls':2*max_decisions,'max_executions':max_executions,
        'expires_at':expires_at,'implementation':pins(),'binary_sha256':d.file_sha(e.ROOT/'target/debug/tsr'),'criteria_sha256':e.sha(review_contract.CRITERIA)}
    if local_advisory:
        if not campaign.get('fixture_only') or max_executions!=0:raise ValueError('local advisory requires fixture ledger and zero execution')
        value.update(version='synthetic-project-cycle-adapter@v2',projection_profile='local-semantic-return@v2')
    if release:
        if local_advisory or type(correction_allowance) is not int or not 0<=correction_allowance<=2 or not 1<=max_decisions<=(7 if information_admission else 4) or not 0<=max_executions<=(3 if information_admission else 2):raise ValueError('release envelope')
        value.update(version='synthetic-project-cycle-adapter@v3',correction_allowance=correction_allowance,max_calls=2*max_decisions+correction_allowance,max_decisions=max_decisions,output_profile='strict-json-bound@v2')
        if information_admission:
            e.digest(information_admission)
            value.update(version='synthetic-project-cycle-adapter@v4',information_admission=information_admission,max_information_reads=1)
    elif correction_allowance:raise ValueError('legacy correction allowance forbidden')
    if type(representation_regeneration) is not bool:raise ValueError('representation admission must be boolean')
    if representation_regeneration:
        if not release or correction_allowance<1:raise ValueError('representation regeneration requires explicit shared correction budget')
        value['representation_regeneration']=True
    return value

def semantic_projection(inputs,parents):
    """Public attributed dimensions only. Never export prose, counts or infer truth."""
    if inputs.get('version')!='project-relational@v2':raise ValueError('semantic return version')
    value=inputs.get('semantic_return')
    if not isinstance(value,dict) or set(value)!={'review','applicable','unmet','unknown','meaning'}:raise ValueError('semantic return fields')
    e.digest(value['review'])
    if type(value['applicable']) is not bool:raise ValueError('semantic applicability')
    allowed={p['requirement'] for p in parents}
    for key in ('unmet','unknown'):
        ids=value[key]
        if not isinstance(ids,list) or any(not isinstance(x,str) for x in ids) or len(ids)!=len(set(ids)) or not set(ids)<=allowed:raise ValueError('unpublished semantic dimension')
    if set(value['unmet'])&set(value['unknown']):raise ValueError('ambiguous semantic dimensions')
    return {'version':'public-semantic-return@v1','review_id':value['review'],'applicable':value['applicable'],'unmet':value['unmet'],'unknown':value['unknown'],
        'attribution':'Independent review judgment, not a tool observation or established truth. Unknown is not negative. Inapplicable judgments are stale, not current evidence. No private review prose or aggregate counts are exported. No execution or acceptance authority.'}

class SemanticChoiceRejected(ValueError):
    """Host validation rejected a candidate; next choice is not a format retry."""

class Adapter:
    release_profiles=('synthetic-project-cycle-adapter@v3','synthetic-project-cycle-adapter@v4')
    row=staticmethod(d.Driver.row);read=d.Driver.read;append=d.Driver.append;op=d.Driver.op
    def __init__(self,root,a,controller,ledger,transport=None,*,worker=None,before_call=None):
        self.root=Path(root).resolve();self.admission=a;self.controller=controller;self.ledger=ledger
        self.release=a.get('version') in self.release_profiles;self.information=a.get('version')=='synthetic-project-cycle-adapter@v4'
        if self.release and ledger.manifest.get('fixture_only') and transport is None:raise ValueError('fixture release requires injected transport')
        self.before_call=before_call
        self.injected_transport=transport
        self.transport=transport or (lambda body,timeout:e.module('open_transport','engine-synthetic-transport.py').invoke(body,timeout,expected_model=MODEL,payload_limit=self.admission.get('payload_bytes',32768)))
        self.local_advisory=a.get('projection_profile')=='local-semantic-return@v2'
        if self.local_advisory and (transport is None or not ledger.manifest.get('fixture_only')):raise ValueError('local advisory has no live transport admission')
        self.worker=worker;self.fixture=bool(ledger.manifest.get('fixture_only')) if self.release else self.local_advisory;self.active=set();self.hook=lambda *_:None
        self.validate_current_admission();self.root.mkdir(parents=True,exist_ok=True);self.file=self.root/'adapter.jsonl'
        if not self.file.exists():
            if any(self.root.iterdir()):raise ValueError('fresh adapter directory required')
            e.private_save(self.file,self.row([],{'kind':'admission','value':a,'fixture_only':self.fixture}))
        self.read();self.trial=e.sha({'trial':str(self.root),'admission':e.sha(a)});self.ledger.require_output_eligible(self.trial)
        self.corrections=correction.Budget(self.root/'corrections.jsonl',a,a.get('correction_allowance',0))
    def validate_current_admission(self):
        a=self.admission
        if set(a)!=set(admission(self.root,a['origin'],self.ledger.manifest,a['expires_at'],max_executions=a['max_executions'],max_decisions=a.get('max_decisions',4),local_advisory=self.local_advisory,release=self.release,correction_allowance=a.get('correction_allowance',0),information_admission=a.get('information_admission'),representation_regeneration=a.get('representation_regeneration',False))):raise ValueError('adapter admission fields')
        if (a['version']!=('synthetic-project-cycle-adapter@v4' if self.information else 'synthetic-project-cycle-adapter@v3' if self.release else 'synthetic-project-cycle-adapter@v2' if self.local_advisory else 'synthetic-project-cycle-adapter@v1') or a['directory']!=str(self.root)
            or a['campaign_sha256']!=e.sha(self.ledger.manifest) or a['model']!=MODEL
            or a['binary_sha256']!=d.file_sha(e.ROOT/'target/debug/tsr') or a['implementation']!=pins() or a['criteria_sha256']!=e.sha(review_contract.CRITERIA)
            or time.time()>=d.stamp(a['expires_at'])):raise ValueError('adapter admission drift/expiry')
        if type(a['max_calls']) is not int or not 0<=a['max_calls']<=(16 if self.information else 10 if self.release else 8) or type(a['max_executions']) is not int or not 0<=a['max_executions']<=4:raise ValueError('adapter limits')
        if self.release and (a['output_profile']!='strict-json-bound@v2' or a['max_calls']!=2*a['max_decisions']+a['correction_allowance'] or not 1<=a['max_decisions']<=(7 if self.information else 4) or not 0<=a['max_executions']<=(3 if self.information else 2)):raise ValueError('release bounds')
        if self.information and (type(a.get('max_information_reads')) is not int or a['max_information_reads']!=1):raise ValueError('information operation bound')
        if self.local_advisory and (a.get('max_executions')!=0 or not self.ledger.manifest.get('fixture_only')):raise ValueError('local advisory zero execution')
        if any(p.is_symlink() for p in (self.root,*self.root.parents)):raise ValueError('adapter symlink')
        if str(self.controller.directory)!=a['origin']['directory']:raise ValueError('native origin directory')
    def enroll(self):
        suite=e.module('open_suite','engine-repair-suite.py')
        identity=e.sha(e.origin_prepare(self.admission['origin']).manifest()) if self.admission['origin'].get('scenario_id') in ('pursuit-v1','sequential-pursuit-v1') else suite.suite_sha256() if self.admission['origin'].get('scenario_id') else None
        return self.op('enroll',lambda:self.ledger.enroll(self.trial,e.sha(self.admission),suite_sha256=identity) or {'enrolled':True})
    def generated(self):
        self.ledger.require_output_eligible(self.trial)
        return [v['response'] for k,v in self.read()[2].items() if k.startswith('generated_')]
    def project(self,view):
        self.validate_current_admission();origin=self.admission['origin'];p=e.origin_prepare(origin);c=self.controller
        request=c.request();coding=request['coding'];relation=coding['relational'];inputs=relation['inputs'];attention=request['next_attention']
        semantic=None
        if (self.local_advisory or self.release) and inputs.get('version')=='project-relational@v2':semantic=semantic_projection(inputs,origin['parents'])
        elif inputs.get('version')!='project-relational@v1' or 'semantic_return' in inputs:raise ValueError('unadmitted semantic return projection')
        if coding.get('information') is not None:
            if not self.information or coding['information']['context']['admission']!=self.admission['information_admission']:raise ValueError('unadmitted information projection')
            expected=p.information_manifest()
            index=[{k:r[k] for k in ('id','digest','attribution','uncertainty')} for r in expected['records']]
            if coding['information']['context']['index']!=index:raise ValueError('information corpus drift/private projection')
        elif self.information:raise ValueError('missing information admission')
        if self.local_advisory and view.get('branch')!='reconsider':raise ValueError('local advisory reconsideration only')
        if c.cli('project','config')!=e.expected_configuration(p):raise ValueError('frozen configuration drift')
        if (view['state_root']!=request['state_root'] or view['source_snapshot']!=coding['snapshot'] or view['scope']!=origin['scope']
            or inputs['requirements_revision']!=origin['requirements_revision'] or inputs['parents']!=origin['parents']
            or attention['scope']!=origin['scope'] or attention['source_snapshot']!=coding['snapshot'] or attention['state_root']!=request['state_root']):raise ValueError('source/criteria/private-context drift')
        files=coding['files'];editable=[x for x in files if x.get('editable')]
        if ({x['path']:x['source_digest'] for x in files if not x.get('editable')}!=origin['protected']
            or len(editable)!=1 or editable[0]['path']!='src/lib.rs' or coding.get('supporting_context')
            or coding['cases']!=e.expected_configuration(p)['cases'] or inputs.get('supplemental')):raise ValueError('unadmitted source or evidence')
        work=c.directory/'coding-work';actual=list(work.rglob('*'))
        if any(x.is_symlink() for x in actual) or {x.relative_to(work).as_posix() for x in actual if x.is_file()}!={'src/lib.rs',*origin['protected']} :raise ValueError('source inventory drift')
        produced=self.generated();allowed={origin['initial_source']}
        for response in produced:
            for proposal in response.get('proposals',[]):
                if proposal['kind']=='edit_set':allowed.update(x['new_text'] for x in proposal['patches'])
        if editable[0]['source'] not in allowed:raise ValueError('source generation lineage')
        relations=[e.relational_identity(x['proposal']) for response in produced for x in response.get('proposals',[]) if x['kind']=='relational']
        history=[]
        for entry in relation['history']:
            if e.relational_identity(entry['proposal']) not in relations:raise ValueError('foreign qualitative guidance')
            history.append({k:entry[k] for k in ('admission','binding','proposal','withdrawn','superseded','current')})
        catalog=e.return_evidence.catalog(request)
        contract=e.payload_contract(request,'final_proposal_only')
        contract['experiment']['rules']=[x for x in contract['experiment']['rules'] if 'experiment stage' not in x]
        actions=[]
        for action in request.get('actions',[]):
            if action.get('id','').startswith('verify-') and action.get('description')=='Verify the admitted current source without editing it':
                e.digest(action['id'][7:]);actions.append({'id':action['id'],'description':action['description']})
        contract.pop('abstain',None)
        context={'scenario':origin.get('scenario_id','original-parser'),'parents':origin['parents'],
            'source':{k:editable[0][k] for k in ('path','source','source_digest','inspection_handle')},
            'case_ids':[x['id'] for x in coding['cases']], 'return_evidence':catalog,
            'relational_history':history,'relational_contract':e.relational_contract(request),
            'contracts':contract,'existing_verification_actions':actions,
            'response_envelopes':{'edits':['edits','rationale','optional common_event_probability'],'relational':['relational','return_basis required for revisions'],'experiment':['experiment'],'existing':['existing:{action:exact verification ID,rationale:text512}'],'abstention':'top-level {abstain:text512}; never an alternative'},
            'return_basis_contract':e.return_evidence.contract(),'constraints':{'editable':['src/lib.rs'],'patch_bytes':2048,'whole_bound_response_bytes':8192,'relational_proposal_json_bytes':4096,'information_proposal_json_bytes':4096,'aggregate_patch_accounting':'Each actualized patch charges the resulting complete editable file bytes to the immutable lifecycle budget; this is separate from per-span bounds. Individual field/alternative maxima cannot all be combined.','protected_tests':'withheld and immutable','authority':'proposal only; qualitative/experiment choices require separate admission'},
            'binding':{'state_root':request['state_root'],'source_snapshot':coding['snapshot'],'attention':attention['binding']}}
        # Same public contract reaches generation and independent review via project().
        # Optional packet interface preserves legacy projections; no scenario routing.
        if hasattr(p,'public_task_context'):
            context['public_task']=p.public_task_context()
        if self.release:
            context['contracts'].pop('experiment',None)
            context['response_envelopes'].pop('experiment',None)
            context['constraints']['supplemental']='No supplemental test creation/admission in this bounded release profile.'
        if self.information:
            context['information']=coding['information']
            context['contracts']['information']={'same_document_schema':'information.choice_schema','note':'The complete authoritative schema is published once at this path; no fields or constraints omitted.'}
            context['response_envelopes']['information']=['information:<choice matching the complete native schema at information.choice_schema>; single alternative only; optional uncertainty detour, not mixed EV ranking']
        if semantic is not None:context['semantic_return']=semantic
        if self.local_advisory or (self.release and view['branch']=='reconsider'):
            context['advisory_reconsideration']={k:view['reconsideration'][k] for k in ('evidence_key','lineage','binding','snapshot','requirements_revision','receipt_ids','semantic_review')}
            context['contracts']={'relational':context['contracts']['relational']}
            context['existing_verification_actions']=[]
            context['response_envelopes']={k:context['response_envelopes'][k] for k in ('relational','abstention')}
            context['constraints']['advisory_only']='Exactly one revision of the named existing lineage or honest abstention; no executable alternatives, new goal, acceptance or follow-on action.'
        return request,context
    def body(self,system,value):
        body={'model':MODEL,'stream':False,'max_completion_tokens':self.admission.get('completion_tokens',8192),'messages':[{'role':'system','content':system},{'role':'user','content':e.canonical(value).decode()}]}
        if self.release:
            body['response_format']=output_mode.response_format(MODEL,output_mode.CAPABILITY)
            # JSON mode requires an explicit format instruction, independent of
            # incidental JSON words in user source or historical evidence.
            if body['response_format']=={'type':'json_object'} and 'json' not in system.lower():
                body['messages'][0]['content']+=' Return a JSON object.'
        if len(capacity.serialized(body))>self.admission.get('payload_bytes',32768):raise d.Blocked('allowlisted payload exceeds admitted model capacity estimate')
        return body
    def known_stop(self,error,status='technical_stop'):
        # Only settled local failure may close outer bookkeeping. Any uncertainty
        # leaves the original pending effect and its reservations untouched.
        budget=self.controller.budget()
        if self.read()[1] or self.ledger.status()['stopped'] or any(budget.get(k,0) for k in ('unsettled_requests','unresolved_tool_requests','unknown_usage','unknown_compute_requests','unknown_external_cost_requests')):return error
        error.known_stage_stop={'version':'known-stage-stop@v1','status':status,'reason':str(error)[:500] or 'Known local boundary refusal'}
        return error
    def call(self,kind,body):
        if self.release and time.time()+240>loop.feedback_policy.deadline(self.admission,d.stamp):raise d.Blocked('insufficient provider deadline headroom')
        _,pending,done=self.read()
        if pending:raise d.Blocked('uncertain adapter effect')
        if self.before_call is not None:self.before_call()
        self.enroll()
        if pending:raise d.Blocked('uncertain adapter effect')
        count=sum(k.startswith(('dispatch_','not_dispatched_')) for k in done)
        if loop.feedback_policy.reached(count,self.admission['max_calls']):raise d.Blocked('adapter call limit')
        ident=e.sha({'admission':e.sha(self.admission),'kind':kind,'index':count,'body':e.sha(body)})
        # Retain the exact allowlisted JSON body, never transport headers or credentials.
        # Source projection has already excluded private/protected native content.
        if set(body)-{'model','stream','max_completion_tokens','messages','response_format'}:raise ValueError('unapproved request evidence fields')
        if any(set(message)!={'role','content'} or message['role'] not in ('system','user') for message in body['messages']):raise ValueError('unapproved request evidence message')
        request_view=self.native('request',None)
        self.op('request_body_'+ident,lambda:{'kind':kind,'body':copy.deepcopy(body),'body_sha256':e.sha(body),'source':{k:request_view[k] for k in ('state_root','source_snapshot','scope')},'evidence_status':'delivered only after matched settled dispatch; citations are model assertions, not causal proof'})
        try:self.ledger.run_call(self.trial,ident,body,240,None,payload_limit=self.admission.get('payload_bytes',32768),preflight_only=True)
        except ValueError as error:raise self.known_stop(error,'capacity_stop')
        e.require_shadow_validation_ready(self.controller)
        self.op('reserve_'+ident,lambda:self.controller.json_cli(('project','resource-reserve-unpriced'),{'request_id':ident,'state_root':self.controller.request()['state_root'],'envelope':e.sha(body),'model':MODEL,'tokens_upper':(capacity.prompt_upper(body) if self.admission.get('context_profile')==capacity.PROFILE else 32768)+body['max_completion_tokens']}))
        wire_bytes=transport_module.serialize_request(body,self.admission.get('payload_bytes',32768))
        self.active_wire_binding={'local_request':ident,'canonical_body_sha256':e.sha(body),'wire_body_sha256':hashlib.sha256(wire_bytes).hexdigest(),'wire_body_bytes':len(wire_bytes),'source':{k:request_view[k] for k in ('state_root','source_snapshot','scope')}}
        result=self.op('dispatch_'+ident,lambda:self.ledger.run_call(self.trial,ident,body,240,self._wire if self.release else self.transport,payload_limit=self.admission.get('payload_bytes',32768)),persist=lambda r:{'resource_stop':bool(r.get('resource_stop')),'receipt':e.measured_receipt(r.get('receipt',{})),'candidate_sha256':e.sha(r.get('value')),'candidate':copy.deepcopy(r.get('value')),'structure':e.candidate_structure(r.get('value'))})
        receipt=result.get('receipt',{});meter=e.measured_receipt(receipt);usage=meter['usage'];known=meter['usage_status']=='reported';cost=None
        if meter.get('usd') is not None:cost=int((decimal.Decimal(meter['usd'])*1000000).to_integral_value(rounding=decimal.ROUND_CEILING))
        elapsed=meter.get('elapsed_seconds')
        self.op('settle_'+ident,lambda:self.controller.json_cli(('project-resource-settle',),{'request_id':ident,'prompt_tokens':usage['prompt_tokens'] if known else None,'output_tokens':usage['completion_tokens'] if known else None,'duration_ns':int(elapsed*1e9) if type(elapsed) in (int,float) and elapsed>=0 else None,'external_cost_microusd':cost,'outcome':'measured' if known else 'unknown'},'--usage'))
        if result.get('resource_stop') or self.controller.budget()['stopped']:raise d.Blocked('resource reconciliation/overrun')
        # Meter before semantic validation. Known malformed reports remain charged.
        loop.Cycle.measured({'response':{},'receipt':{**receipt,'mode':'fixture' if self.fixture else 'accounted'}},'response',self.admission.get('completion_tokens',8192),self.admission.get('input_tokens',32768))
        self.last_result=result;self.last_ident=ident
        return ident,result['value'],{**receipt,'mode':'fixture' if self.fixture else 'accounted'}
    def _wire(self,body,timeout):
        if self.injected_transport is not None:
            return self.injected_transport(body,timeout)
        return transport_module.invoke(body,timeout,expected_model=MODEL,payload_limit=self.admission.get('payload_bytes',32768),failure_evidence=self.evidence,completion_limit=self.admission.get('completion_tokens',8192),on_response_identity=self.retain_response_identity)
    def retain_response_identity(self,identity):
        binding=self.active_wire_binding
        if identity.get('version')!='response-header-identity@v1' or identity.get('request_body_sha256')!=binding['wire_body_sha256'] or identity.get('request_body_bytes')!=binding['wire_body_bytes']:raise ValueError('response identity contract/binding')
        # Nested in the durable dispatch intent. op fsyncs this result before
        # transport starts reading the response body. It never settles usage.
        record={**copy.deepcopy(binding),'admission_sha256':e.sha(self.admission),'identity':copy.deepcopy(identity)}
        name='response_identity_'+binding['local_request']
        prior=self.read()[2].get(name)
        if prior is not None and prior!=record:raise ValueError('response identity substitution')
        return self.op(name,lambda:record)

    def retry_terminal_failure(self,ident,receipt):
        return False  # Opt-in profiles alone may recognize independently settled failures.
    def check_proposal_shape(self,value):
        load("project_shape_check","engine-project-output.py").proposal_shape(value)
    def checked(self,kind,body,validate,review=False):
        helper=load('project_output_contract','engine-project-output.py')
        events=self.corrections.events()
        recorded=[v for k,v in self.read()[2].items() if k.startswith('format_correction_')]
        if len(events)!=len(recorded) or any(v['event']!=event for v,event in zip(recorded,events)):raise d.Blocked('correction journal changed; reconcile without dispatch')
        original=copy.deepcopy(body);metadata={}
        if review:
            original,metadata=bound_output.prepare_review_body(original,aliases=getattr(self,'review_reference_aliases',False))
        source=correction.digest(original);feedback=None
        initial=self.native('request',None)
        source_view={k:initial[k] for k in ('state_root','source_snapshot','scope')}
        if hasattr(self,'retained_format_feedback'):feedback=self.retained_format_feedback(source,initial)
        while True:
            now=self.native('request',None);loop.Cycle.same_source(initial,now)
            if (self.root/'terminal-output.json').exists():raise d.Blocked('output stage is terminal; no reuse')
            wire=self.corrections.body(original,feedback) if feedback else original
            if hasattr(self,'recovery_wire'):wire=self.recovery_wire(original,wire)
            if len(capacity.serialized(wire))>self.admission.get('payload_bytes',32768):raise d.Blocked('correction payload bound')
            replay=self.replay_checked(kind,wire) if hasattr(self,'replay_checked') else None
            number=replay[3] if replay is not None else sum(k.startswith(('dispatch_','not_dispatched_')) for k in self.read()[2])
            # Fixed per-call quarantine directory supports ten calls without changing shared retention limits.
            directory=self.root/('output-'+str(number))
            if directory.exists():
                if not hasattr(self,'recovery_ready') or directory.is_symlink() or not directory.is_dir() or any(directory.iterdir()):raise d.Blocked('existing output quarantine; no overwrite')
                if replay is None:self.recovery_ready()
            else:directory.mkdir(exist_ok=False)
            self.evidence=transport_module.FailureEvidence(directory,'failed-response-1',limit=max(65536,getattr(self,'narrative_limit',65536)))
            try:
                ident,value,receipt=replay[:3] if replay is not None else self.call(kind,wire)
                if not (feedback and feedback.get('category')=='criterion_consistency') and self.retry_terminal_failure(ident,receipt):continue
                try:
                    if value is None:raise helper.FormatFailure(None)
                    raw_value=copy.deepcopy(value)
                    now=self.native('request',None);loop.Cycle.same_source(initial,now)
                    source_keys=('state_root','source_snapshot','scope')
                    source_view={k:initial[k] for k in source_keys}
                    done=self.read()[2]
                    association=bound_output.associate(admission=self.admission,kind=kind,index=number,wire=wire,ident=ident,content=raw_value,dispatch=done['dispatch_'+ident],settled=done.get('settle_'+ident),source=source_view,current_source={k:now[k] for k in source_keys})
                    if review and isinstance(value,dict) and set(value)&set(metadata):
                        if self.admission.get('review_format_regeneration') is True:
                            # Reject the complete old response. A new counted response must omit metadata.
                            schema=json.loads(original['messages'][1]['content'])['response_schema']
                            helper.fail('$','unsupported_field',allowed_fields=list(schema['properties']),required_fields=schema['required'])
                        raise ValueError('model supplied trusted binding metadata; no repair')
                    if review:helper.check(value,json.loads(original['messages'][1]['content'])['response_schema'])
                    else:self.check_proposal_shape(value)
                    if review:
                        value=bound_output.attach_review(value,metadata)
                        if getattr(self,'review_reference_aliases',False):
                            value=bound_output.expand_review(value,wire)
                            association['reference_catalog_sha256']=e.sha(json.loads(wire['messages'][1]['content'])['reference_catalog'])
                    decoded=validate(value)
                    if getattr(self,'narrative_limit',512)!=512 and len(e.canonical(raw_value))>self.narrative_limit:raise ValueError('complete semantic response exceeds aggregate browser memory envelope')
                    self.op('bound_output_'+ident,lambda:{'association':association,'model_content':raw_value,'host_metadata':metadata,'native_content_sha256':e.sha(value),'decoded_sha256':e.sha(decoded),'source':source_view,**({'reference_request':wire} if review and getattr(self,'review_reference_aliases',False) else {})})
                except (ValueError,subprocess.CalledProcessError) as error:
                    retained=self.evidence.retain('native_contract')
                    self.op('rejected_'+ident,lambda:{'kind':kind,'retention':retained,'not_admitted':True,'validation_reason':str(error)[:8192]})
                    if kind=='proposal' and getattr(error,'code',None) in ('no_insight_meaning_drift','stale_edit_span') and hasattr(self,'record_native_rejection') and self.record_native_rejection(error,ident,association,raw_value,source_view):
                        raise SemanticChoiceRejected(str(error)) from error
                    diagnostic=getattr(error,'diagnostic',None) or (error.stderr if isinstance(error,subprocess.CalledProcessError) else None)
                    try:
                        feedback=self.corrections.feedback(kind,source,self.last_result,ident,native_diagnostic=diagnostic,request_source=source_view)
                        event=self.corrections.events()[-1]
                        self.op('format_correction_'+str(event['ordinal']),lambda:{'event':event,'feedback':feedback,'source':source,'request_source':source_view})
                    except ValueError as stop:
                        e.private_save(self.root/'terminal-output.json',{'kind':kind,'failed_request':ident,'reason':str(stop),'validation_reason':str(error)[:8192],'unused_allowances_expired':True})
                        raise self.known_stop(d.Blocked(str(stop))) from error
                    continue
                if review:return ident,value,receipt
                response,basis=decoded
                self.op('generated_'+ident,lambda:{'response':response,'return_basis':basis})
                return {'response':response,'receipt':receipt}
            except subprocess.TimeoutExpired:
                # A local native command timing out is not malformed model output.
                # Preserve exact received bytes before the volatile buffer is cleared;
                # never regenerate or replay a possibly mutating command here.
                self.evidence.retain('local_controller_timeout')
                raise
            finally:self.evidence.clear()
    def proposal(self,view):
        request,context=self.project(view)
        body=self.body('Choose useful alternatives from these synthetic desires, source, guidance and observations. No prescribed sequence. Never mix executable (edits/existing) alternatives with deferred (relational/experiment) alternatives in one batch. A qualitative continuation requires exactly one relational alternative. Return strict JSON {"alternatives":[1..4 distinct payload objects matching one supplied contract each]} or {"abstain":"bounded honest inability/no useful action reason"}. Relational/experiment proposals are deferred for separate admission. No protected test edits or invented outcomes. All rationale/meaning strings <=512 UTF-8 bytes. Proposals grant no authority.',context)
        if self.information:body['messages'][0]['content']+=' An optional information alternative must be the only alternative and match the complete schema at information.choice_schema (referenced by contracts.information) exactly. Its access_probability forecasts byte retrieval only, never task success. Parent relevance must explain a real uncertainty and possible information benefit. No invented user preference, required read or preferred record.'
        if self.local_advisory or (self.release and view['branch']=='reconsider'):
            body['messages'][0]['content']='Source-bound advisory reconsideration only. Return strict JSON {"alternatives":[{"relational":<one existing-lineage revision>,"return_basis":<bound evidence basis>}]} or {"abstain":"honest reason <=512 UTF-8 bytes"}. Preserve parent meanings. Retention without new insight is valid; no forced novelty. Distinguish tool observations, attributed semantic judgments, hypotheses and unknowns. Never offer edits, experiments, new aspirations or action/acceptance authority. Use exact provided lineage and evidence identities.'
            if len(capacity.serialized(body))>self.admission.get('payload_bytes',32768):raise d.Blocked('advisory payload bound')
        if self.release:
            return self.checked('proposal',body,lambda value:self.decode_proposal(value,request,context,view))
        ident,value,receipt=self.call('proposal',body)
        response,basis_records=self.decode_proposal(value,request,context,view)
        self.op('generated_'+ident,lambda:{'response':response,'return_basis':basis_records})
        return {'response':response,'receipt':receipt}
    def decode_proposal(self,value,request,context,view):
        if isinstance(value,dict) and set(value)=={'abstain'}:
            if not isinstance(value['abstain'],str) or not 0<len(value['abstain'].encode())<=512:raise ValueError('bounded abstention required')
            return value,{}
        if not isinstance(value,dict) or set(value)!={'alternatives'} or not isinstance(value['alternatives'],list) or not 1<=len(value['alternatives'])<=4:raise ValueError('alternative response shape')
        # Eligibility is deliberately narrower than general format correction:
        # one whole-file edit, valid path/shape/forecast, exact unchanged source,
        # and no semantic/reconsideration/mixed-alternative failure to override.
        if self.admission.get('representation_regeneration',False) and len(value['alternatives'])==1:
            item=value['alternatives'][0]
            if isinstance(item,dict) and 'edits' in item:
                e.validate_stage_envelope('final_proposal_only',item)
                edits=item.get('edits')
                if len(edits)==1 and isinstance(edits[0],dict) and set(edits[0])=={'path','replacement'} and edits[0]['path']=='src/lib.rs' and isinstance(edits[0]['replacement'],str) and len(edits[0]['replacement'].encode())>2048:
                    if view['branch']!='propose' or self.local_advisory:raise ValueError('executable alternative outside current phase')
                    if not item['rationale'].strip():raise ValueError('blank rationale is not a representation-size defect')
                    current=self.native('request',None)
                    if any(view[k]!=current[k] for k in ('scope','state_root','source_snapshot')):raise ValueError('stale size candidate context')
                    if request['state_root']!=view['state_root'] or request['coding']['snapshot']!=view['source_snapshot']:raise ValueError('stale size candidate source')
                    bound=e.local.bind_proposal(request,json.dumps(item,allow_nan=False))
                    if len(e.canonical(bound))>8192:raise ValueError('aggregate response exceeds native boundary; no representation retry')
                    helper=load('representation_shape','engine-project-output.py')
                    helper.fail('$.alternatives[0].edits[0].replacement','representation_size',max_utf8_bytes=2048,actual_utf8_bytes=len(edits[0]['replacement'].encode()))
        response=None;basis_records={}
        for item in value['alternatives']:
            if self.release and isinstance(item,dict) and 'experiment' in item:raise ValueError('supplemental tests outside this admission')
            index=len(response['proposals']) if response else 0
            if isinstance(item,dict) and set(item)=={'information'}:
                if not self.information or len(value['alternatives'])!=1:raise ValueError('information not admitted or mixed alternatives')
                helper=load('information_shape','engine-project-output.py');helper.check(item['information'],context['information']['choice_schema'],'$.alternatives['+str(index)+'].information')
                part={'protocol':request['protocol'],'state_root':request['state_root'],'attention_binding':request['next_attention']['binding'],'proposals':[{'kind':'information','choice':item['information']}]}
            elif isinstance(item,dict) and set(item)=={'existing'}:
                selected=item['existing']
                if (not isinstance(selected,dict) or set(selected)!={'action','rationale'} or selected['action'] not in {x['id'] for x in context['existing_verification_actions']}
                    or not isinstance(selected['rationale'],str) or not 0<len(selected['rationale'].encode())<=512):raise ValueError('unadmitted existing action')
                part={'protocol':request['protocol'],'state_root':request['state_root'],'attention_binding':request['next_attention']['binding'],'proposals':[{'kind':'existing',**selected}]}
            else:
                e.validate_stage_envelope('final_proposal_only',item)
                e.validate_trial_payload(item,request,'open_choice')
                if 'relational' in item:
                    e.validate_relational_lineage(request,item['relational'],'open_choice')
                    if item['relational'].get('revises') is not None or 'return_basis' in item:basis_records[str(index)]=e.return_evidence.validate(item.get('return_basis'),request)
                part=e.local.bind_proposal(request,json.dumps({k:v for k,v in item.items() if k!='return_basis'},allow_nan=False))
            if response is None:response={k:v for k,v in part.items() if k not in ('proposals','common_event_forecasts')};response['proposals']=[]
            response['proposals'].extend(part['proposals'])
            if 'common_event_forecasts' in part:response.setdefault('common_event_forecasts',{})[str(index)]=part['common_event_forecasts']['0']
        if (self.local_advisory or (self.release and view['branch']=='reconsider')) and (len(response['proposals'])!=1 or response['proposals'][0]['kind']!='relational' or response['proposals'][0]['proposal'].get('revises')!=view['reconsideration']['lineage']):raise ValueError('advisory revision only')
        self.native('preview',{'response':response})
        return response,basis_records
    def review(self,view,response):
        request,context=self.project(view)
        if response not in self.generated():raise ValueError('review candidate not generated here')
        matches=[(k,v) for k,v in self.read()[2].items() if k.startswith('generated_') and v['response']==response]
        generated_key,record=matches[-1]
        generated_request=generated_key[len('generated_'):]
        association=self.verify_generated_output(generated_request,response,record['return_basis'])
        basis=review_contract.indexed_basis(response,record['return_basis'])
        evidence={'context:'+e.sha(context):context,'candidate:'+view['response_digest']:response,'return_basis:'+e.sha(basis):basis}
        if hasattr(self,'proposal_representation_evidence'):
            representation=self.proposal_representation_evidence(request,association,generated_request,response,record['return_basis'])
            evidence['representation:'+e.sha(representation)]=representation
        for assessment in review_contract.attributed_assessments(context,view):
            evidence['assessment:'+e.sha(assessment)]=assessment
        observed=[];information_observed=[]
        for item in context['return_evidence']['cases']:
            key='receipt:'+e.sha(item);evidence[key]=item
            (information_observed if item['channel']=='information_access' else observed).append(key)
        binding={'review_id':e.sha({'admission':e.sha(self.admission),'view':context,'candidate':view['response_digest']}),'evidence_ids':list(evidence),'observed_receipts':observed}
        if self.information:binding['information_receipts']=information_observed
        body=self.body('Independent project review invocation. Review EVERY alternative against ALL criteria and the evidence_contract. Same-model errors may correlate. Return {review_id,verdict:accept|reject|unknown|abstain,findings:{each criterion:{finding:met|not_met|unknown,basis:static_source|observed_test_receipt|unknown,evidence:[exact IDs],explanation:nonblank <=512 UTF-8 bytes,limitations:nonblank <=512 UTF-8 bytes}}}. Accept requires all met; reject requires not_met; unknown requires unknown. Observed basis requires an observed receipt ID. A review permits only already-admitted controller processing, not acceptance or fulfillment.',{'binding':binding,'criteria':review_contract.CRITERIA,'evidence':evidence,'evidence_contract':review_contract.evidence_contract()})
        if self.release:body['messages'][0]['content']=body['messages'][0]['content'].replace('Return {review_id,verdict:', 'Return {verdict:')
        if getattr(self,'narrative_limit',512)!=512:
            body['messages'][0]['content']=body['messages'][0]['content'].replace('<=512 UTF-8 bytes','nonblank within the aggregate response memory envelope')
        if self.information:
            body['messages'][0]['content']+=' For information access use observed_information_receipt with its exact supplied ID. It proves read identity only; fictional text is an attributed assertion, never observed task success. For information proposals judge explicit uncertainty, active parent relevance, plausible source benefit, zero authority expansion and optional exploration. Do not require edits or guaranteed fulfillment from a read. An unsupported or unnecessary inquiry may be rejected or unknown.'
        if self.release:
            helper=load('project_output_contract','engine-project-output.py')
            payload=json.loads(body['messages'][1]['content']);payload['response_schema']=helper.review_schema(binding,review_contract.CRITERIA,compact=self.information,text_limit=getattr(self,'narrative_limit',512))
            body=self.body(body['messages'][0]['content'],payload)
            ident,value,receipt=self.checked('review',body,lambda value:review_contract.validate(value,binding,text_limit=getattr(self,'narrative_limit',512)),review=True)
        else:ident,value,receipt=self.call('review',body)
        verdict=review_contract.validate(value,binding,text_limit=getattr(self,'narrative_limit',512))
        self.op('reviewed_'+ident,lambda:{'binding':binding,'report':value,'generated_request':generated_request,'return_basis_sha256':e.sha(basis)})
        judgment={'decision':'approve' if verdict=='accept' else verdict,'response_digest':view['response_digest'],'reviewer':'separate-runtime-invocation:'+ident,'generator':MODEL}
        if self.release:self.verify_owned_review(judgment,response)
        return {'judgment':judgment,'receipt':receipt}
    def verify_generated_output(self,ident,response,basis):
        record=self.verify_owned_output(ident)
        if record['decoded_sha256']!=e.sha([response,basis]):raise ValueError('review generation/basis association mismatch')
        return record
    def verify_owned_output(self,ident):
        _,pending,done=self.read()
        if pending:raise ValueError('pending response association')
        current=self.native('request',None);source={k:current[k] for k in ('state_root','source_snapshot','scope')}
        try:return bound_output.verify_envelope(done['bound_output_'+ident],admission=self.admission,ident=ident,dispatch=done['dispatch_'+ident],settled=done.get('settle_'+ident),source=source)
        except KeyError:raise ValueError('missing owned response association') from None
    def verify_owned_review(self,review,response=None):
        prefix='separate-runtime-invocation:'
        if not isinstance(review.get('reviewer'),str) or not review['reviewer'].startswith(prefix) or review.get('generator')!=MODEL:raise ValueError('unowned reviewer')
        ident=review['reviewer'][len(prefix):];bound=self.verify_owned_output(ident);done=self.read()[2]
        if bound['association']['kind']!='review':raise ValueError('wrong response role')
        owned=done.get('reviewed_'+ident)
        if owned is None or e.sha(owned['report'])!=bound['native_content_sha256']:raise ValueError('substituted review report')
        report=owned['report']
        decision=report.get('decision',{'accept':'approve'}.get(report.get('verdict'),report.get('verdict')))
        if review['decision']!=decision:raise ValueError('substituted review decision')
        for key in ('response_digest','receipt','evidence'):
            if key in review and key in report and review[key]!=report[key]:raise ValueError('substituted review binding')
        if response is not None:
            if 'candidate' in owned and owned['candidate']!=response:raise ValueError('cross-request review candidate')
            if 'binding' in owned and 'candidate:'+review['response_digest'] not in owned['binding']['evidence_ids']:raise ValueError('cross-request review candidate')
            matches=[(k,v) for k,v in done.items() if k.startswith('generated_') and v['response']==response]
            if 'generated_request' in owned:
                matches=[(k,v) for k,v in matches if k=='generated_'+owned['generated_request']]
            if len(matches)!=1:raise ValueError('unowned or ambiguous generated response')
            key,generated=matches[0]
            basis_hash=e.sha(review_contract.indexed_basis(response,generated['return_basis']))
            if 'generated_request' in owned and (owned.get('return_basis_sha256')!=basis_hash or 'return_basis:'+basis_hash not in owned['binding']['evidence_ids']):raise ValueError('reviewed evidence basis substitution')
            record=self.verify_generated_output(key[len('generated_'):],response,generated['return_basis'])
            if record['association']['kind']!='proposal':raise ValueError('substituted generated response')
        return owned
    def native(self,kind,value):
        self.ledger.require_output_eligible(self.trial)
        self.validate_current_admission()
        c=self.controller
        if kind=='request':return c.cli('project','cycle','request')
        if kind=='response_context' and self.release:return c.cli('project','cycle','response-context')
        if kind=='acknowledge':
            if not (self.local_advisory or self.release):raise ValueError('advisory acknowledgment not admitted')
            return c.json_cli(('project','cycle','acknowledge'),value)
        if kind=='preview':return c.json_cli(('project','cycle','preview'),value['response'],'--response')
        if kind=='advance':
            if self.release:self.verify_owned_review(value['review'],value['response'])
            result=c.json_cli(('project','cycle','advance'),value)
            proposals=value['response']['proposals']
            if len(proposals)==1 and proposals[0]['kind']=='relational' and result.get('review_required'):
                record=result['proposal_record'];current=self.native('request',None)
                self.op('qualitative_'+record,lambda:c.cli('project','relational','admit','--state-root',current['state_root'],'--proposal',record,'--index','0'))
                result={**result,'review_required':False,'qualitative_admitted':True,'next':self.native('request',None)}
            return result
        if self.local_advisory:raise ValueError('local advisory forbids execution')
        if kind=='information_resume':
            if not self.information:raise ValueError('information not admitted')
            if time.time()+30>d.stamp(self.admission['expires_at']):raise d.Blocked('insufficient information deadline headroom')
            self.enroll();view=self.native('request',None)
            if view['binding']!=value['binding'] or view['branch']!='information_resume':raise ValueError('stale information binding')
            if sum(k.startswith('information_execution_') for k in self.read()[2])>=self.admission['max_information_reads']:raise d.Blocked('information operation cap')
            if sum(k.startswith(('execution_','information_execution_')) for k in self.read()[2])>=self.admission['max_executions']:raise d.Blocked('all operation cap')
            ident=e.sha({'admission':e.sha(self.admission),'information_binding':value['binding']})
            def inspect():
                r=subprocess.run([str(e.ROOT/'target/debug/tsr'),'engine','--directory',str(c.directory),'project','cycle','information-resume','--binding',value['binding']],capture_output=True,timeout=30)
                if r.returncode or len(r.stdout)>1048576:raise d.Blocked('information outcome unknown/refused; never retry')
                return e.strict(r.stdout)
            return self.op('information_execution_'+ident,lambda:self.ledger.run_execution(self.trial,ident,e.sha(self.admission),inspect))
        if kind!='resume':raise ValueError('unsupported native callback')
        if self.release and self.ledger.manifest.get('fixture_only') and self.worker is None:raise ValueError('fixture release requires injected worker')
        self.enroll();view=self.native('request',None)
        if view['binding']!=value['binding'] or view['branch']!='resume':raise ValueError('stale execution binding')
        done=self.read()[2]
        if self.information and sum(k.startswith('execution_') for k in done)>=2:raise d.Blocked('sealed execution cap')
        if loop.feedback_policy.reached(sum(k.startswith(('execution_','information_execution_')) for k in done),self.admission['max_executions']):raise d.Blocked('adapter execution limit')
        if self.release and time.time()+self.resume_timeout(view)>loop.feedback_policy.deadline(self.admission,d.stamp):raise d.Blocked('insufficient worker deadline headroom')
        ident=e.sha({'admission':e.sha(self.admission),'binding':value['binding']})
        return self.op('execution_'+ident,lambda:self.ledger.run_execution(self.trial,ident,e.sha(self.admission),lambda:self.worker(view) if self.worker else self.resume_process(view)))

    @staticmethod
    def resume_timeout(view):
        plan=view['plan'];milliseconds=plan['timeout_ms'];commands=plan['commands']
        if type(milliseconds) is not int or milliseconds<=0 or not isinstance(commands,list) or not commands:raise ValueError('native plan time bound')
        seconds=milliseconds*len(commands)/1000+30
        if seconds>240:raise ValueError('native plan exceeds host execution envelope')
        return seconds
    def resume_process(self,view):
        self.validate_current_admission()
        result=subprocess.run([str(e.ROOT/'target/debug/tsr'),'engine','--directory',str(self.controller.directory),'project','cycle','resume','--binding',view['binding']],capture_output=True,timeout=self.resume_timeout(view))
        if result.returncode or len(result.stdout)>1048576:raise d.Blocked('native execution uncertain/refused; reconcile without retry')
        return e.strict(result.stdout)
