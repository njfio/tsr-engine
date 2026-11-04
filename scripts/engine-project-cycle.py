#!/usr/bin/env python3
"""Local-only open-choice orchestration with injected, separately accounted callbacks.

No CLI transport, credentials, external export, B9.1 sequence or execution grant.
Callers must bind callbacks to an admitted native scope and account every effect.
"""
import json
import copy
import decimal
import fcntl
import importlib.util
import os
from pathlib import Path
import time

spec=importlib.util.spec_from_file_location('cycle_durable','%s/engine-b91-driver.py'%Path(__file__).parent)
d=importlib.util.module_from_spec(spec);spec.loader.exec_module(d)
e=d.e
capacity=e.module('cycle_capacity','engine-request-capacity.py')
feedback_policy=e.module('cycle_feedback_policy','engine-feedback-policy.py')
stage_receipts=e.module('cycle_stage_receipts','engine-stage-receipts.py')

def pending_resume_binding(done,index):
    # Relational admissions are advances too; execution indices cannot index them.
    advances=[v for k,v in done.items() if k.startswith('advance_')]
    completed=[k for k in done if k.startswith('resume_')]
    if len(completed)!=index or not advances or advances[-1]['next'].get('branch')!='resume':raise d.Blocked('no reviewed execution advance for pending resume')
    return advances[-1]['next']['binding']

class _KnownStageStop(RuntimeError):
    def __init__(self,record):
        self.record=record
        super().__init__(record['reason'])

class Cycle:
    # Reuse the existing content-addressed intent/result protocol, not its admission.
    row=staticmethod(d.Driver.row)
    read=d.Driver.read
    append=d.Driver.append
    def __init__(self,root,admission,native,proposal,review,*,hook=None,reading=None,abilities=None):
        self.reading=reading
        self.abilities=abilities
        self.root=Path(root).absolute();self.admission=admission;self.fixture=admission.get('advisory_profile')=='local-fixture-only@v1';self.continuous=admission.get('version') in ('local-project-cycle@v3','local-project-cycle@v4','local-project-cycle@v5');self.objective=admission.get('version')=='local-project-cycle@v5';self.information=admission.get('version')=='local-project-cycle@v4'
        # Local introspection is not the provider export. V4 has seven bounded decisions.
        self.local_context_limit=8388608 if admission.get('local_source_profile')=='browser-scoped-source@v1' else 131072 if self.information or self.objective else 32768
        self.journal_limit=capacity.JOURNAL_BYTES if admission.get('local_source_profile')=='browser-scoped-source@v1' else 8*1024*1024
        self.native=native;self.proposal=proposal;self.review=review;self.active=set();self.hook=hook or (lambda *_:None)
        self.validate_current_admission();self.root.mkdir(parents=True,exist_ok=True);self.file=self.root/'cycle.jsonl'
        if not self.file.exists():
            if any(self.root.iterdir()):raise ValueError('fresh cycle journal directory required')
            e.private_save(self.file,self.row([],{'kind':'admission','value':admission,'fixture_only':self.fixture}))
        self.read()
    def validate_current_admission(self):
        a=self.admission
        unlimited=feedback_policy.unlimited(a)
        fields={'version','journal','scope','max_decisions','max_executions','expires_at','implementation_sha256'}
        if unlimited:
            fields.add('unlimited_policy_sha256')
            if a.get('local_source_profile')!='browser-scoped-source@v1' or a.get('version')!='local-project-cycle@v3' or any(a.get(k) is not None for k in ('max_decisions','max_executions','expires_at')):raise ValueError('unlimited profile only for policy-bound browser cycle')
        for field,profile in (('stage_receipts',stage_receipts.PROFILE),('review_feedback','attributed-review-feedback@v1')):
            if field in a:
                fields.add(field)
                if a.get('local_source_profile')!='browser-scoped-source@v1' or a.get('version')!='local-project-cycle@v3' or a[field]!=profile:raise ValueError('unsupported prospective cycle profile: '+field)
        if 'checkpoint_id' in a:
            fields.add('checkpoint_id')
            if a.get('local_source_profile')!='browser-scoped-source@v1' or not isinstance(a['checkpoint_id'],str) or len(a['checkpoint_id'])!=64 or any(c not in '0123456789abcdef' for c in a['checkpoint_id']):raise ValueError('checkpoint identity')
        if 'feedback_cycle' in a:
            fields.add('feedback_cycle')
            policy=a['feedback_cycle']
            if a.get('local_source_profile')!='browser-scoped-source@v1':raise ValueError('invalid feedback cycle source profile')
            feedback_policy.validate(policy,a['max_decisions'])
        if 'input_tokens' in a:
            fields.add('input_tokens')
            if a.get('local_source_profile')!='browser-scoped-source@v1' or a['input_tokens']!=capacity.INPUT_TOKENS:raise ValueError('unadmitted context allocation')
        if 'max_review_rejections' in a:
            fields.add('max_review_rejections')
            if a.get('local_source_profile')!='browser-scoped-source@v1' or type(a['max_review_rejections']) is not int or not 0<=a['max_review_rejections']<=(12 if a.get('unlimited_policy_sha256') else 3):raise ValueError('unadmitted rejection recovery')
        if 'completion_tokens' in a:
            fields.add('completion_tokens')
            if a.get('local_source_profile')!='browser-scoped-source@v1' or type(a['completion_tokens']) is not int or a['completion_tokens']!=32768:raise ValueError('unadmitted completion allocation')
        if 'local_source_profile' in a:
            fields.add('local_source_profile')
            if a['local_source_profile']!='browser-scoped-source@v1' or a['version']!='local-project-cycle@v3':raise ValueError('unsupported local source profile')
        if a.get('version')=='local-project-cycle@v2':
            fields.add('advisory_profile')
            if a.get('advisory_profile')!='local-fixture-only@v1' or a.get('max_executions')!=0:raise ValueError('local advisory-only profile required')
        elif a.get('version') in ('local-project-cycle@v3','local-project-cycle@v4','local-project-cycle@v5'):
            if self.objective:
                fields.add('max_service_rounds')
                if type(a.get('max_service_rounds')) is not int or not 1<=a['max_service_rounds']<=3:raise ValueError('finite service round bound')
            if self.information:
                fields.add('max_information_reads')
                if (type(a.get('max_information_reads')) is not int or a['max_information_reads']!=1):raise ValueError('information read bound')
            if not unlimited and (a['max_decisions']>(99 if a.get('local_source_profile')=='browser-scoped-source@v1' else 9 if self.objective else 7 if self.information else 4) or a['max_executions']>(100 if a.get('local_source_profile')=='browser-scoped-source@v1' else 5 if self.objective else 3 if self.information else 2)):raise ValueError('bounded native release profile')
        elif a.get('version')!='local-project-cycle@v1':raise ValueError('local cycle version')
        if set(a)!=fields:raise ValueError('local cycle admission shape')
        if a['journal']!=str(self.root) or any(p.is_symlink() for p in [self.root,*self.root.parents]):raise ValueError('cycle journal boundary')
        for key,maximum in (('max_decisions',99 if a.get('local_source_profile')=='browser-scoped-source@v1' else 32),('max_executions',100 if a.get('local_source_profile')=='browser-scoped-source@v1' else 16)):
            if not unlimited and (type(a[key]) is not int or not 0<=a[key]<=maximum):raise ValueError('cycle resource bound')
        if a['implementation_sha256']!=d.file_sha(Path(__file__)) or time.time()>=feedback_policy.deadline(a,d.stamp):raise ValueError('cycle expired/implementation changed')
    def op(self,name,fn,*,persist=None):
        if 'stage_receipts' not in self.admission:
            return d.Driver.op(self,name,fn,persist=persist)
        self.validate_current_admission()
        if (self.root/'STOP').exists():raise d.Blocked('trial withdrawn/stopped')
        _,pending,done=self.read()
        if set(pending)-self.active:raise d.Blocked('uncertain effect; no automatic retry: '+','.join(sorted(set(pending)-self.active)))
        if name in done:return done[name]
        intent={'kind':'intent','name':name,'at':d.iso_now()}
        self.append(intent);self.active.add(name)
        try:
            self.hook('after_intent',name)
            stopped=None
            try:value=fn()
            except Exception as error:
                classification=getattr(error,'known_stage_stop',None)
                if not self.valid_stage_stop(classification):raise
                stopped={**classification,'stage':name}
                value={'known_stage_stop':stopped}
            stored=value if stopped else (persist(value) if persist else value)
            rows,_,_=self.read()
            stage_receipts.Receipts(self.root,self.admission).save(rows,intent,stored)
            self.hook('after_receipt',name)
            self.hook('after_effect',name)
            self.append({'kind':'result','name':name,'intent':e.sha(intent),'value':stored})
            self.hook('after_result',name)
            if stopped:raise _KnownStageStop(stopped)
            return value
        finally:self.active.discard(name)
    @staticmethod
    def valid_stage_stop(value):
        # Internal callback protocol only. Model dictionaries are never exceptions.
        return (isinstance(value,dict) and set(value)=={'version','status','reason'}
                and value['version']=='known-stage-stop@v1'
                and value['status'] in ('capacity_stop','technical_stop','spend_cap')
                and isinstance(value['reason'],str) and bool(value['reason'].strip())
                and len(value['reason'].encode())<=2048)
    def close_stage_stop(self,record):
        return self.terminal(record['status'],reason=record['reason'],stage=record['stage'],
                             known_settled=True,callback_retried=False,desires_remain_open=True)
    def model_stage(self,name,kind,context,response=None):
        if name in self.read()[2]:
            return self.op(name,lambda:self.invoke_model_stage({'kind':kind,'context':context,'response':response}))
        if self.admission.get('feedback_cycle'):
            captured=self.op('callback_input_'+name,lambda:{'kind':kind,'context':copy.deepcopy(context),'response':copy.deepcopy(response)})
            if captured!={'kind':kind,'context':context,'response':response}:raise d.Blocked('callback input drift')
        return self.op(name,lambda:self.invoke_model_stage({'kind':kind,'context':context,'response':response}))
    def invoke_model_stage(self,spec):
        self.same_source(spec['context'],self.current())
        if spec['kind']=='proposal':return self.measured_profile(self.proposal(copy.deepcopy(spec['context'])),'response')
        if spec['kind']=='review':return self.measured_profile(self.review(copy.deepcopy(spec['context']),copy.deepcopy(spec['response'])),'judgment')
        raise d.Blocked('unsupported callback recovery')
    def _recover_pending(self):
        self.validate_current_admission()
        if (self.root/'STOP').exists():raise d.Blocked('cycle stopped; no stage recovery')
        rows,pending,done=self.read()
        if not pending:return 0
        if 'stage_receipts' not in self.admission or len(pending)!=1:raise d.Blocked('uncertain effect; reconcile without retry')
        intent=next(iter(pending.values()))
        receipt=stage_receipts.Receipts(self.root,self.admission).load(rows,intent)
        if receipt is None and intent.get('local_record_profile')=='compound-value@v1':
            import re
            if not re.fullmatch(r'(compound_[0-9]+_(start|complete|(development|artifact)_(outcome|derive_input))|(request|proposal|meaning_input)_[0-9]+[ad]|terminal)',intent['name']):raise d.Blocked('invalid local component record')
            receipt={'value':intent['local_value']}
        if receipt is None and intent['name'].startswith('compound_') and intent['name'].endswith('_derive'):
            spec=done.get(intent['name']+'_input')
            if spec is None:raise d.Blocked('missing exact component derivation input')
            receipt={'value':self.native('deliberation_component',spec)}
        if receipt is None and intent['name'].startswith('advance_'):
            value=done.get('meaning_input_'+intent['name'][len('advance_'):])
            if value is not None:
                recovered=self.native('meaning_advance_receipt',value)
                stage_receipts.Receipts(self.root,self.admission).save(rows,intent,recovered);receipt={'value':recovered}
        if receipt is None and intent['name'].startswith('reconsider_') and intent['name'].endswith('_advance'):
            value=done.get('meaning_input_'+intent['name'])
            if value is not None:
                recovered=self.native('meaning_advance_receipt',value)
                stage_receipts.Receipts(self.root,self.admission).save(rows,intent,recovered);receipt={'value':recovered}
        if receipt is None and self.admission.get('feedback_cycle') and intent['name'].startswith('resume_'):
            index=int(intent['name'][len('resume_'):])
            value=self.native('execution_stage_receipt',{'index':index,'binding':pending_resume_binding(done,index)})
            stage_receipts.Receipts(self.root,self.admission).save(rows,intent,value);receipt={'value':value}
        if receipt is None and self.admission.get('feedback_cycle') and intent['name'].startswith('session_continue_'):
            index=int(intent['name'][len('session_continue_'):])
            value=self.native('session_transition_receipt',{'index':index,**({'checkpoint_id':self.admission['checkpoint_id']} if 'checkpoint_id' in self.admission else {})})
            stage_receipts.Receipts(self.root,self.admission).save(rows,intent,value)
            receipt={'value':value}
        if receipt is None and self.admission.get('feedback_cycle'):
            spec=self.read()[2].get('callback_input_'+intent['name'])
            if spec is not None:
                self.native('provider_recovery_ready',None)
                value=self.invoke_model_stage(spec)
                stage_receipts.Receipts(self.root,self.admission).save(rows,intent,value)
                receipt={'value':value}
        if receipt is None:raise d.Blocked('uncertain effect; successful stage receipt missing; no retry')
        self.append({'kind':'result','name':intent['name'],'intent':e.sha(intent),'value':receipt['value']})
        return 1
    def recover_pending(self):
        """Close an exact completed outer stage only; never invoke its callback.

        Launcher must first reject pending nested adapter or external effects.
        """
        with os.fdopen(os.open(self.root/'run.lock',os.O_CREAT|os.O_WRONLY|os.O_NOFOLLOW,0o600),'a') as lock:
            fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
            return self._recover_pending()
    def current(self,consuming=False):
        value=self.native('response_context' if consuming and self.continuous else 'request',None)
        if value['scope']!=self.admission['scope'] or value.get('dispatch_authorized') is not False:raise ValueError('native scope/authority drift')
        return value
    @staticmethod
    def same_source(before,after):
        if any(before[k]!=after[k] for k in ('scope','state_root','source_snapshot')):raise d.Blocked('source/state changed across callback')
    @staticmethod
    def measured(result,key,completion_limit=8192,input_limit=32768):
        if set(result)!={key,'receipt'} or not isinstance(result[key],dict):raise ValueError('typed callback result required')
        if result['receipt'].get('mode') not in ('fixture','accounted'):raise ValueError('callback receipt provenance required')
        meter=e.measured_receipt(result['receipt']);u=meter.get('usage') or {};elapsed=meter.get('elapsed_seconds')
        if (meter.get('usage_status')!='reported' or meter.get('usd') is None
            or type(elapsed) not in (int,float) or not 0<=elapsed<=240
            or any(type(u.get(k)) is not int or u[k]<0 for k in ('prompt_tokens','completion_tokens','total_tokens'))
            or u['prompt_tokens']>input_limit or u['completion_tokens']>completion_limit
            or u['prompt_tokens']+u['completion_tokens']!=u['total_tokens']):raise d.Blocked('callback metering unknown/exceeded')
        try:cost=decimal.Decimal(str(meter['usd']))
        except decimal.InvalidOperation:raise d.Blocked('callback cost invalid') from None
        if not cost.is_finite() or cost<0:raise d.Blocked('callback cost invalid')
        return {key:result[key],'receipt':meter,'callback_mode':result['receipt']['mode']}
    def measured_profile(self,result,key):
        return self.measured(result,key,self.admission.get('completion_tokens',8192),self.admission.get('input_tokens',32768))
    def terminal(self,status,**fields):
        if status in ('proposal_retention','proposal_abstention','idle','advisory_reconsidered','feedback_cycle_complete','post_action_advisory_admitted'):
            request=self.refresh_request()
            if request is not None:
                current=self.current()
                if current['branch'] not in ('resume','information_resume','service_resume','blocked','reconcile'):
                    fields={**fields,'prior_outcome':status,'refresh':request,'refresh_sha256':e.sha(request),'state_root':current['state_root'],'source_snapshot':current['source_snapshot'],'scope':current['scope'],'desires_remain_open':True,'acceptance_recorded':False}
                    status='context_refresh_required'
        value={'status':status,'built_in_transport':False,'callback_effects_require_external_admission':True,**fields}
        if any(k.startswith('compound_') and k.endswith('_complete') for k in self.read()[2]):
            return self.record_component('terminal',value)
        return self.op('terminal',lambda:value)
    @classmethod
    def same_advisory(cls,before,after):
        cls.same_source(before,after)
        if after.get('branch')!='reconsider' or before.get('reconsideration')!=after.get('reconsideration'):raise d.Blocked('advisory evidence/lineage drift')
    @staticmethod
    def decision_count(done):
        return sum(k.startswith(('advance_','rejected_proposal_','ability_')) or (k.startswith('reconsider_') and k.endswith('_acknowledge')) for k in done)
    def reconsider(self,current):
        index=str(self.decision_count(self.read()[2]));prefix=('reconsider_'+index+'_') if self.continuous else 'advisory_'
        if feedback_policy.reached(int(index),self.admission['max_decisions']):return self.terminal('decision_budget')
        captured=self.op(prefix+'request',lambda:current)
        self.same_advisory(captured,self.current())
        if len(e.canonical(captured))>self.local_context_limit:raise d.Blocked('advisory context bound')
        proposal=self.model_stage(prefix+'proposal','proposal',captured)
        if not self.continuous and proposal['callback_mode']!='fixture':raise d.Blocked('local advisory profile forbids external callbacks')
        response=proposal['response'];fresh=self.current();self.same_advisory(captured,fresh)
        if set(response)=={'deliberation'}:
            return self.deliberation(index,response,captured)
        abstain=set(response) in ({'abstain'},{'retain'})
        if abstain:
            if not isinstance(next(iter(response.values())),str) or not 0<len(next(iter(response.values())).encode())<=(131072 if self.admission.get('local_source_profile')=='browser-scoped-source@v1' else 512):raise ValueError('bounded advisory abstention')
            digest=e.sha(response)
        else:
            proposals=response.get('proposals',[])
            browser=self.admission.get('local_source_profile')=='browser-scoped-source@v1'
            if len(proposals)!=1 or (proposals[0].get('kind') not in ('relational','public_desire','provisional_preference','contrast_preference','success_preference') if browser else proposals[0].get('kind')!='relational' or proposals[0].get('proposal',{}).get('revises')!=captured['reconsideration']['lineage']):raise d.Blocked('unsupported advisory choice')
            preview=self.native('preview',{'response':response});self.same_advisory(captured,preview['cycle']);digest=preview['response_digest']
        if len(e.canonical({'context':fresh,'response':response}))>self.local_context_limit:raise d.Blocked('advisory review context bound')
        reviewed=self.model_stage(prefix+'review','review',{**fresh,'response_digest':digest},response)
        if not self.continuous and reviewed['callback_mode']!='fixture':raise d.Blocked('local advisory profile forbids external review')
        judgment=reviewed['judgment']
        if set(judgment)!={'decision','reviewer','generator','response_digest'} or judgment['response_digest']!=digest or judgment['decision'] not in ('approve','reject','unknown','abstain'):raise ValueError('advisory review binding')
        latest=self.current(consuming=True);self.same_advisory(captured,latest)
        if abstain or judgment['decision']!='approve':
            outcome={'reject':'rejected','unknown':'unknown','abstain':'abstain'}.get(judgment['decision'],'abstain')
            if set(response)=={'retain'} and judgment['decision']=='approve':outcome='no_new_insight'
            if not abstain and judgment['decision'] in ('reject','unknown') and self.admission.get('feedback_cycle'):
                self.op(prefix+'criticism',lambda:self.native('reject_proposal',{'source':latest,'response':response,'judgment':judgment}))
            self.op(prefix+'acknowledge',lambda:self.native('acknowledge',{'binding':latest['binding'],'evidence_key':latest['reconsideration']['evidence_key'],'outcome':outcome,'rationale':'Attributed reviewed advisory outcome; no action or acceptance authority.','reviewer':judgment['reviewer'],'generator':judgment['generator']}))
            if not abstain and outcome in ('rejected','unknown') and self.admission.get('feedback_cycle'):
                return self.session_continue('review_feedback')
            return self.terminal('advisory_'+outcome,desires_remain_open=True)
        preview=self.native('preview',{'response':response});self.same_advisory(captured,preview['cycle'])
        if preview['response_digest']!=digest:raise d.Blocked('advisory reviewed response drift')
        latest=preview['cycle'];attestation={**judgment,'state_root':latest['state_root'],'source_snapshot':latest['source_snapshot'],'binding':latest['binding']}
        value={'binding':latest['binding'],'response':response,'review':attestation}
        if self.admission.get('local_source_profile')=='browser-scoped-source@v1':self.op('meaning_input_'+prefix+'advance',lambda:value)
        result=self.op(prefix+'advance',lambda:self.native('advance',value))
        if self.continuous:
            if not any(result.get(k) for k in ('qualitative_admitted','meaning_admitted','preference_admitted')) or result.get('review_required'):return self.terminal('review_required')
            self.op('advance_'+index,lambda:result)
            return None
        return self.terminal('advisory_reconsidered',execution_authorized=False,acceptance_granted=False)
    def record_component(self,name,value):
        """Durable local value only; never invokes a callback or an external effect."""
        self.validate_current_admission()
        if (self.root/'STOP').exists():raise d.Blocked('cycle stopped')
        _,pending,done=self.read()
        if name in done:
            if done[name]!=value:raise d.Blocked('component record drift')
            return done[name]
        if pending:raise d.Blocked('pending effect before component record')
        intent={'kind':'intent','name':name,'at':d.iso_now(),'local_record_profile':'compound-value@v1','local_value':copy.deepcopy(value)}
        self.append(intent);self.hook('after_intent',name)
        self.append({'kind':'result','name':name,'intent':e.sha(intent),'value':value});self.hook('after_result',name)
        return value

    def deliberation(self,index,whole,captured):
        """Two durable component paths. Never send a mixed batch to native code."""
        parts=whole['deliberation'];prefix='compound_'+index
        self.record_component(prefix+'_start',{'response':whole,'context':captured})
        outcome=None
        for name,suffix in (('development','d'),('artifact','a')):
            if parts[name] is None:continue
            key=index+suffix;done=self.read()[2]
            disposition=done.get(prefix+'_'+name+'_outcome')
            if disposition is not None:
                if name=='development':outcome=disposition
                elif disposition['judgment']['decision']=='abstain':
                    terminal={'status':'review_abstain','development':outcome,'artifact_executed':False,'desires_remain_open':True}
                    self.record_component(prefix+'_complete',{'artifact':'review_abstain','development':outcome,'terminal':terminal})
                    return self.terminal(**terminal)
                continue
            if name=='artifact':
                if parts['artifact_dependency']=='requires_development' and (not outcome or outcome['judgment']['decision']!='approve'):
                    return self.terminal('deliberation_dependency_unmet',development=outcome,artifact_executed=False,desires_remain_open=True)
                current=self.current()
                if current['branch']=='review_required':
                    stop=self.session_continue(prefix+'_artifact')
                    if stop is not None:return stop
                elif current['branch'] not in ('propose','reconsider'):raise d.Blocked('component cannot cross pending effect or closed scope')
            derive_input=self.record_component(prefix+'_'+name+'_derive_input',{'response':whole,'component':name,'development_outcome':outcome})
            derived=self.op(prefix+'_'+name+'_derive',lambda:self.native('deliberation_component',derive_input))
            response=derived['response']
            if set(response)=={'retain'}:
                current=self.current();retention_review=self.read()[2].get('review_'+key,{}).get('judgment')
                # A dedicated return question must be closed by a reviewed
                # interpretation, not by silently treating artifact retention as
                # an unchanged-meaning decision. An admitted development already
                # records that interpretation atomically in native state.
                if current['branch']=='reconsider':
                    digest=e.sha(response)
                    reviewed=self.model_stage('review_'+key,'review',{**current,'response_digest':digest},response)
                    judgment=reviewed['judgment'];retention_review=judgment;latest=self.current(consuming=True)
                    self.same_advisory(current,latest)
                    if judgment['response_digest']!=digest or judgment['decision'] not in ('approve','reject','unknown','abstain'):raise d.Blocked('retention review mismatch')
                    result={'approve':'no_new_insight','reject':'rejected','unknown':'unknown','abstain':'abstain'}[judgment['decision']]
                    self.op(prefix+'_acknowledge',lambda:self.native('acknowledge',{'binding':latest['binding'],'evidence_key':latest['reconsideration']['evidence_key'],'outcome':result,'rationale':'Reviewed artifact retention with separate development disposition; no fulfillment or action authority.','reviewer':judgment['reviewer'],'generator':judgment['generator']}))
                    current=self.current()
                terminal={'status':'proposal_retention','reason':response['retain'],'state_root':current['state_root'],'source_snapshot':current['source_snapshot'],'development':outcome,'desires_remain_open':True,'acceptance_recorded':False}
                if retention_review is not None:
                    terminal['retention_review']=retention_review
                    if retention_review['decision']!='approve':terminal['status']='advisory_'+{'reject':'rejected','unknown':'unknown','abstain':'abstain'}[retention_review['decision']]
                self.record_component(prefix+'_complete',{'artifact':'retained','development':outcome,'terminal':terminal})
                return self.terminal(**terminal)
            done=self.read()[2]
            if 'advance_'+key in done:
                reviewed=done['review_'+key];result=done['advance_'+key]
            else:
                current=self.current();context=self.record_component('request_'+key,current);self.same_source(context,current)
                self.record_component('proposal_'+key,{'response':response,'derived_from':prefix,'component':name})
                preview=self.native('preview',{'response':response});self.same_source(context,preview['cycle'])
                reviewed=self.model_stage('review_'+key,'review',{**context,'response_digest':preview['response_digest']},response)
                judgment=reviewed['judgment']
                if judgment['response_digest']!=preview['response_digest'] or judgment['decision'] not in ('approve','reject','unknown','abstain'):raise d.Blocked('component review mismatch')
                latest=self.current();self.same_source(context,latest)
                result=None
                if judgment['decision']=='approve':
                    final=self.native('preview',{'response':response});self.same_source(context,final['cycle'])
                    if final['response_digest']!=preview['response_digest']:raise d.Blocked('component preview changed')
                    latest=final['cycle'];value={'binding':latest['binding'],'response':response,'review':{**judgment,'state_root':latest['state_root'],'source_snapshot':latest['source_snapshot'],'binding':latest['binding']}}
                    if name=='development':self.record_component('meaning_input_'+key,value)
                    result=self.op('advance_'+key,lambda:self.native('advance',value))
                    if name=='development' and not any(result.get(k) for k in ('meaning_admitted','preference_admitted','qualitative_admitted')):raise d.Blocked('development still awaiting native admission')
                elif judgment['decision'] in ('reject','unknown'):
                    self.op('rejected_proposal_'+key,lambda:self.native('reject_proposal',{'source':latest,'response':response,'judgment':judgment}))
            disposition={'response':response,'judgment':reviewed['judgment'],'advance':result}
            self.record_component(prefix+'_'+name+'_outcome',disposition)
            if name=='development':outcome=disposition
            elif disposition['judgment']['decision']=='abstain':
                terminal={'status':'review_abstain','development':outcome,'artifact_executed':False,'desires_remain_open':True}
                self.record_component(prefix+'_complete',{'artifact':'review_abstain','development':outcome,'terminal':terminal})
                return self.terminal(**terminal)
        self.record_component(prefix+'_complete',{'development':outcome,'artifact':'separately_reviewed'})
        return None

    def session_continue(self,reason):
        done=self.read()[2];policy=self.admission['feedback_cycle']
        index=sum(k.startswith('session_continue_') for k in done)
        if feedback_policy.reached(index,policy['max_continuations']):return self.terminal('continuation_budget',desires_remain_open=True)
        current=self.current()
        if current['branch'] not in ('review_required','propose'):return self.terminal('continuation_ineligible',native_branch=current['branch'])
        self.op('session_continue_'+str(index),lambda:{'reason':reason,'transition':self.native('continue_session',{'binding':current['binding'],'index':index,'reason':reason,**({'checkpoint_id':self.admission['checkpoint_id']} if 'checkpoint_id' in self.admission else {})})})
        return None
    def run(self):
        with os.fdopen(os.open(self.root/'run.lock',os.O_CREAT|os.O_WRONLY|os.O_NOFOLLOW,0o600),'a') as lock:
            fcntl.flock(lock,fcntl.LOCK_EX|fcntl.LOCK_NB)
            try:return self._run_locked()
            except _KnownStageStop as stop:return self.close_stage_stop(stop.record)
    def refresh_request(self):
        """Pause only after a complete returned action, never mid-decision/effect.
        This file can request a pause, not grant authority or alter frozen input.
        A successor must obtain a new admission and authenticate this terminal.
        """
        path=self.root/'CONTEXT_REFRESH'
        if not path.exists():return None
        with os.fdopen(os.open(path,os.O_RDONLY|os.O_NOFOLLOW),'rb') as stream:
            if os.fstat(stream.fileno()).st_size>16384:raise ValueError('context refresh control too large')
            request=json.load(stream)
        if (set(request)!={'version','cycle_admission_sha256','request_id','input_ids','inputs_sha256'}
            or request['version']!='context-refresh@v1' or request['cycle_admission_sha256']!=e.sha(self.admission)
            or not isinstance(request['request_id'],str) or not isinstance(request['input_ids'],list)
            or not request['input_ids'] or any(not isinstance(x,str) for x in request['input_ids'])
            or len(set(request['input_ids']))!=len(request['input_ids'])
            or not isinstance(request['inputs_sha256'],str) or len(request['inputs_sha256'])!=64):
            raise ValueError('context refresh binding invalid')
        return request
    def context_refresh(self,done):
        request=self.refresh_request()
        if request is None:return None
        # resume includes independently collected browser evidence. An unfinished
        # newer proposal/review/advance must finish before this boundary.
        names=list(done)
        resumes=[i for i,k in enumerate(names) if k.startswith('resume_')]
        if not resumes or any(k.startswith(('request_','proposal_','review_','advance_','reconsider_')) for k in names[resumes[-1]+1:]):return None
        current=self.current()
        if current['branch'] in ('resume','information_resume','service_resume','blocked','reconcile'):return None
        return self.terminal('context_refresh_required',refresh=request,refresh_sha256=e.sha(request),
            state_root=current['state_root'],source_snapshot=current['source_snapshot'],scope=current['scope'],
            desires_remain_open=True,acceptance_recorded=False)
    def _run_locked(self):
        self._recover_pending()
        while True:
            self.validate_current_admission()
            if (self.root/'STOP').exists():raise d.Blocked('cycle withdrawn')
            _,pending,done=self.read()
            if pending:raise d.Blocked('uncertain effect; reconcile without retry')
            if 'terminal' in done:return done['terminal']
            refreshed=self.context_refresh(done)
            if refreshed is not None:return refreshed
            if 'stage_receipts' in self.admission:
                for name,value in done.items():
                    if isinstance(value,dict) and set(value)=={'known_stage_stop'}:
                        record=value['known_stage_stop']
                        if not isinstance(record,dict) or record.get('stage')!=name or not self.valid_stage_stop({k:v for k,v in record.items() if k!='stage'}):raise ValueError('invalid stored stage stop')
                        return self.close_stage_stop(record)
            completed=[v['terminal'] for k,v in done.items() if k.startswith('compound_') and k.endswith('_complete') and 'terminal' in v]
            if completed:return self.terminal(**completed[-1])
            unfinished=[(k,v) for k,v in done.items() if k.startswith('compound_') and k.endswith('_start') and k[:-6]+'_complete' not in done]
            if unfinished:
                key,value=unfinished[0];result=self.deliberation(key[len('compound_'):-len('_start')],value['response'],value['context'])
                if result is not None:return result
                continue
            if self.continuous:
                for key,result in list(done.items()):
                    if key.startswith('reconsider_') and key.endswith('_advance') and any(result.get(k) for k in ('qualitative_admitted','meaning_admitted','preference_admitted')):
                        marker='advance_'+key.split('_')[1]
                        if marker not in done:self.op(marker,lambda result=result:result)
                done=self.read()[2]
            advances=[done[k] for k in done if k.startswith('advance_')]
            if self.admission.get('feedback_cycle') and advances and advances[-1].get('qualitative_admitted') and any(k.startswith('resume_') for k in done) and 'pursuit_after_'+[k for k in done if k.startswith('advance_')][-1] not in done:
                key=[k for k in done if k.startswith('advance_')][-1];index=key[len('advance_'):]
                prefix='reconsider_'+index+'_' if 'reconsider_'+index+'_proposal' in done else ''
                proposal=done[prefix+'proposal' if prefix else 'proposal_'+index]
                review=done[prefix+'review' if prefix else 'review_'+index]
                context=done[prefix+'request' if prefix else 'request_'+index]
                execution=[v for k,v in done.items() if k.startswith('resume_')][-1]
                witness_key=('feedback_completion_'+index) if feedback_policy.ongoing(self.admission.get('feedback_cycle')) else 'feedback_completion'
                witness=self.op(witness_key,lambda:self.native('feedback_witness',{'execution':execution,'response':proposal['response'],'judgment':review['judgment'],'context':context,'advance':advances[-1]}))
                if feedback_policy.ongoing(self.admission.get('feedback_cycle')) and witness['reviewed_return']:
                    purpose='after_reviewed_return_'+index
                    if not any(k.startswith('session_continue_') and v.get('reason')==purpose for k,v in self.read()[2].items()):
                        result=self.session_continue(purpose)
                        if result is not None:return result
                    self.op('pursuit_after_'+key,lambda:{'witness':witness_key,'execution_receipt':witness['execution_receipt'],'source_snapshot':witness['source_snapshot'],'reason':purpose})
                    continue
                return self.terminal('feedback_cycle_complete' if witness['reviewed_return'] else 'post_action_advisory_admitted',acceptance_recorded=False,desires_remain_open=True,observed_execution=True,**witness)
            if advances and (advances[-1].get('meaning_admitted') or advances[-1].get('preference_admitted')):
                key='meaning_after_'+[k for k in done if k.startswith('advance_')][-1]
                if key not in done:
                    if not self.admission.get('feedback_cycle'):return self.terminal('preference_admitted' if advances[-1].get('preference_admitted') else 'meaning_admitted',acceptance_recorded=False)
                    result=self.session_continue(key)
                    if result is not None:return result
                    self.op(key,lambda:{'meaning_admission':advances[-1].get('meaning_admission') or advances[-1].get('preference_admission'),'not_relational_reflection':True})
                    continue
            if advances and advances[-1].get('review_required') and not self.objective:return self.terminal('review_required')
            if advances and not advances[-1].get('qualitative_admitted') and not (advances[-1].get('meaning_admitted') or advances[-1].get('preference_admitted')) and advances[-1].get('next',{}).get('branch') not in ('resume','information_resume','service_resume'):return self.terminal('native_no_progress',native_branch=advances[-1].get('next',{}).get('branch'))
            current=self.current();branch=current['branch']
            if branch=='review_required' and self.admission.get('feedback_cycle'):
                has_reflection=any(k.startswith('advance_') and v.get('qualitative_admitted') for k,v in done.items())
                purpose=('after_reflection_'+str(len(advances)) if feedback_policy.ongoing(self.admission.get('feedback_cycle')) else 'after_reflection') if has_reflection else 'initial_checkpoint'
                used=any(k.startswith('session_continue_') and v.get('reason')==purpose for k,v in done.items())
                if any(k.startswith('resume_') for k in done) and not has_reflection:
                    # A project with no admitted relationship still gets a free
                    # next choice informed by the actual return; no synthesis is forced.
                    purpose='after_action_'+str(sum(k.startswith('resume_') for k in done))
                    used=any(k.startswith('session_continue_') and v.get('reason')==purpose for k,v in done.items())
                if not used:
                    result=self.session_continue(purpose)
                    if result is not None:return result
                    continue
            if branch in ('stop','idle','blocked','reconcile','review_required'):
                return self.terminal(branch,native_reason=current.get('reason'))
            if branch=='reconsider' and self.reading is not None:return self.terminal('reading_reconsideration_not_offered',desires_remain_open=True,acceptance_recorded=False)
            if branch=='reconsider':
                if not self.continuous and self.admission.get('advisory_profile')!='local-fixture-only@v1':return self.terminal('review_required',native_reason='advisory_reconsideration_not_admitted')
                result=self.reconsider(current)
                if result is not None:return result
                continue
            if branch=='objective_review_required':
                if not self.objective:return self.terminal('objective_not_admitted')
                context=current['objective'];stage=context['context']['next']
                if stage=='unresolved':return self.terminal('objective_unknown')
                if stage not in ('round_review','objective_review'):raise ValueError('objective review stage')
                index=str(context['context']['round']);prefix='objective_'+stage+'_'+index
                captured=self.op(prefix+'_context',lambda:current)
                self.same_source(captured,current)
                reviewed=self.op(prefix+'_review',lambda:self.measured_profile(self.review(copy.deepcopy(captured),{'objective_stage':stage}),'judgment'))
                latest=self.current();self.same_source(captured,latest)
                if latest['objective']!=context:raise d.Blocked('objective review evidence drift')
                judgment=reviewed['judgment']
                keys={'binding','decision','reviewer','generator','evidence'}|({'receipt'} if stage=='round_review' else set())
                if set(judgment)!=keys or judgment['binding']!=context['binding']:raise ValueError('exact objective review contract')
                if judgment['decision'] not in (('met','not_met','unknown') if stage=='round_review' else ('accept','reject','abstain')):raise ValueError('objective review decision')
                # Native review validates exact receipt identity and independent attribution.
                # Final acceptance is intentionally a separate derivative-copy boundary.
                if stage=='objective_review':return self.terminal('objective_final_assessment',judgment=judgment,acceptance_recorded=False)
                self.op(prefix+'_admit',lambda:self.native('objective_review',{'review':{**judgment,'state_root':latest['state_root']}}))
                continue
            if branch=='service_resume':
                if not self.objective:return self.terminal('service_not_admitted')
                services=[k for k in done if k.startswith('service_resume_')]
                operations=[k for k in done if k.startswith(('resume_','information_resume_','service_resume_'))]
                if len(services)>=self.admission['max_service_rounds'] or len(operations)>=self.admission['max_executions']:return self.terminal('execution_budget')
                candidates=[v for k,v in done.items() if k.startswith('advance_') and v.get('service_prepared')]
                if len(candidates)<=len(services):return self.terminal('unreviewed_service')
                self.same_source(candidates[-1]['next'],current)
                self.op('service_resume_'+str(len(services)),lambda:self.native('service_resume',{'binding':current['binding']}))
                continue
            if branch=='information_resume':
                if not self.information:return self.terminal('information_not_admitted')
                reads=[k for k in done if k.startswith('information_resume_')]
                operations=[k for k in done if k.startswith(('resume_','information_resume_'))]
                if len(reads)>=self.admission['max_information_reads'] or len(operations)>=self.admission['max_executions']:return self.terminal('execution_budget')
                candidates=[v for k,v in done.items() if k.startswith('advance_') and v.get('information_prepared')]
                if len(candidates)<=len(reads):return self.terminal('unreviewed_information')
                self.same_source(candidates[-1]['next'],current)
                self.op('information_resume_'+str(len(reads)),lambda:self.native('information_resume',{'binding':current['binding']}))
                continue
            if branch=='resume':
                executions=[k for k in done if k.startswith('resume_')]
                if self.information and len(executions)>=2:return self.terminal('sealed_execution_budget')
                if feedback_policy.reached(len(executions)+sum(k.startswith(('information_resume_','service_resume_')) for k in done),self.admission['max_executions']):return self.terminal('execution_budget')
                advances=[done[k] for k in done if k.startswith('advance_') and not done[k].get('qualitative_admitted') and not done[k].get('meaning_admitted') and not done[k].get('preference_admitted') and not done[k].get('information_prepared') and not done[k].get('service_prepared')]
                if len(advances)<=len(executions):return self.terminal('unreviewed_pending_execution')
                expected=advances[-1]['next'];self.same_source(expected,current)
                if expected.get('plan')!=current.get('plan'):raise d.Blocked('reviewed execution plan drift')
                # Exact native binding owns commitment/source/authority. Callback must
                # reserve campaign execution before invoking sealed native resume.
                self.op('resume_'+str(len(executions)),lambda:self.native('resume',{'binding':current['binding']}))
                continue
            if branch!='propose':raise ValueError('unsupported native branch')
            completed=self.decision_count(done);index=str(completed)
            if feedback_policy.reached(completed,self.admission['max_decisions']):return self.terminal('decision_budget')
            if self.abilities is not None and self.abilities.spend_reached():return self.terminal('spend_cap',desires_remain_open=True,acceptance_recorded=False)
            # The reading comes first and changes the state root, so the request is captured after it.
            if self.reading is not None and 'reading_'+index not in done:
                self.op('reading_'+index,lambda:self.reading(current))
                continue
            captured=self.op('request_'+index,lambda:current)
            self.same_source(captured,current)
            if len(e.canonical(captured))>self.local_context_limit:raise d.Blocked('proposal context bound')
            proposed=self.model_stage('proposal_'+index,'proposal',captured)
            response=proposed['response']
            if self.abilities is not None and set(response)=={'need'}:
                outcome=self.op('ability_need_'+index,lambda:self.abilities.ability_need(response['need']))
                if outcome['unmet']:return self.terminal('need_outside_scope',need=outcome['need'],desires_remain_open=True,acceptance_recorded=False)
                continue
            if self.abilities is not None and set(response)=={'play'}:
                self.op('ability_play_'+index,lambda:self.abilities.ability_play(response['play']))
                continue
            if set(response)=={'deliberation'}:
                result=self.deliberation(index,response,captured)
                if result is not None:return result
                continue
            if set(response)=={'retain'}:
                reason=response['retain']
                if self.admission.get('local_source_profile')!='browser-scoped-source@v1' or not isinstance(reason,str) or not 0<len(reason.encode())<=131072:raise ValueError('bounded browser retention required')
                fresh=self.current();self.same_source(captured,fresh)
                if self.abilities is not None and self.abilities.ability_new_evidence(list(self.read()[2])):
                    self.op('ability_retain_'+index,lambda:{'reason':reason})
                    continue
                return self.terminal('proposal_retention',reason=reason,state_root=fresh['state_root'],source_snapshot=fresh['source_snapshot'],desires_remain_open=True,acceptance_recorded=False)
            if set(response)=={'abstain'}:
                reason=response['abstain']
                if not isinstance(reason,str) or not 0<len(reason.encode())<=(131072 if self.admission.get('local_source_profile')=='browser-scoped-source@v1' else 512):raise ValueError('bounded abstention required')
                return self.terminal('proposal_abstention',reason=reason,desires_remain_open=True)
            fresh=self.current();self.same_source(captured,fresh)
            if response.get('state_root')!=fresh['state_root']:raise d.Blocked('stale proposal response')
            if fresh['branch']!='propose':raise d.Blocked('native branch changed')
            preview=self.native('preview',{'response':response});self.same_source(fresh,preview['cycle'])
            if len(e.canonical({'context':fresh,'response':response}))>self.local_context_limit:raise d.Blocked('review context bound')
            if self.abilities is not None and self.abilities.spend_reached():return self.terminal('spend_cap',desires_remain_open=True,acceptance_recorded=False)
            reviewed=self.model_stage('review_'+index,'review',{**fresh,'response_digest':preview['response_digest']},response)
            judgment=reviewed['judgment']
            if set(judgment)!={'decision','reviewer','generator','response_digest'} or judgment['decision'] not in ('approve','reject','unknown','abstain'):raise ValueError('review judgment shape')
            feedback_eligible=judgment['decision']=='reject' or (judgment['decision']=='unknown' and self.admission.get('review_feedback')=='attributed-review-feedback@v1')
            if feedback_eligible and self.admission.get('max_review_rejections',0):
                if sum(k.startswith('rejected_proposal_') for k in done)>=self.admission['max_review_rejections']:return self.terminal('review_rejection_budget')
                latest=self.current();self.same_source(captured,latest)
                if judgment['response_digest']!=preview['response_digest']:raise d.Blocked('rejected response digest drift')
                self.op('rejected_proposal_'+index,lambda:self.native('reject_proposal',{'source':latest,'response':response,'judgment':judgment}))
                continue
            if judgment['decision']!='approve':return self.terminal('review_'+judgment['decision'])
            final_preview=self.native('preview',{'response':response});latest=final_preview['cycle'];self.same_source(captured,latest)
            if judgment['response_digest']!=final_preview['response_digest'] or final_preview['response_digest']!=preview['response_digest']:raise d.Blocked('reviewed response digest drift')
            if latest['branch']!='propose':raise d.Blocked('native branch changed')
            # Host attestation binds the settled resource head. It does not change
            # generated response/meaning or turn review into execution permission.
            attestation={**judgment,'state_root':latest['state_root'],'source_snapshot':latest['source_snapshot'],'binding':latest['binding']}
            advance_input={'binding':latest['binding'],'response':response,'review':attestation}
            if len(response['proposals'])==1 and response['proposals'][0].get('kind') in ('public_desire','contrast_preference','success_preference','provisional_preference'):
                self.op('meaning_input_'+index,lambda:advance_input)
            self.op('advance_'+index,lambda:self.native('advance',advance_input))
