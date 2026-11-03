#!/usr/bin/env python3
"""Browser projection on the shared native cycle/output/accounting boundary.
No automatic setup or live launch. An explicit reviewed admission is required.
"""
import copy
import hashlib
import json
import subprocess
from pathlib import Path
import time
import importlib.util
s=importlib.util.spec_from_file_location('browser_projection',Path(__file__).with_name('engine-browser-project.py'));web=importlib.util.module_from_spec(s);s.loader.exec_module(web)
base=web.base;e=base.e;d=base.d;loop=base.loop;load=base.load
IMAGE='sha256:58deb9d12b7291f6d2c697b335eb927557d707488df09d1503659d7abdd00883'
recovery=load('browser_transient','engine-browser-transient.py')
assessment=load('browser_assessment','engine-browser-assessment.py')
clock=load('paired_clock','engine-pair-clock.py')
feedback_policy=load('browser_feedback_policy','engine-feedback-policy.py')
VERSION='browser-project-cycle-adapter@v7'
def pins():
    return {**base.pins(),**{n:d.file_sha(Path(__file__).with_name(n)) for n in ('engine-browser-deliberation.py','engine-criterion-return.py','engine-browser-expansion.py','engine-browser-desire.py','engine-pursuit-reflection.py','engine-feedback-policy.py','engine-proposal-references.py','engine-browser-harness-v1.cjs','engine-browser-session.py','engine-browser-assessment.py','engine-pair-clock.py','engine-browser-ordinary.py','engine-browser-transient.py','engine-browser-predispatch.py','engine-browser-response-recovery.py','engine-objective-continuation.py','engine-browser-adapter.py','engine-browser-project.py','engine-browser-harness.cjs')}}
def supported_harness(text):
    if not isinstance(text,str):return False
    # Current harness, the previous exact harness (12 actions) kept for admitted projects, or an older pinned one.
    return any(text==Path(__file__).with_name(n).read_text() for n in ('engine-browser-harness.cjs','engine-browser-harness-v1.cjs')) or hashlib.sha256(text.encode()).hexdigest()=='b4e0204756fcb497a4993bf5e8d4039ccdbfb872d92d13f9e493f4b53e3129dc'

def validate_feedback_events(events,parents):
    allowed={p['requirement'] for p in parents}
    if not isinstance(events,list):raise ValueError('feedback events must be an explicit list')
    ids=set()
    for event in events:
        if not isinstance(event,dict) or set(event)!={'id','attribution','guidance'} or event['attribution'] not in ('verified_user_input','synthetic_harness_verification'):raise ValueError('feedback attribution contract')
        if not isinstance(event['id'],str) or not event['id'] or event['id'] in ids:raise ValueError('feedback identity')
        ids.add(event['id'])
        guidance=event['guidance']
        if not isinstance(guidance,dict) or not guidance or not set(guidance)<=allowed or any(not isinstance(v,str) or not v.strip() or len(v.encode())>web.NATIVE_RESPONSE_LIMIT for v in guidance.values()):raise ValueError('feedback public meaning scope')
def admission(root,origin,configuration,campaign,expires_at,*,max_decisions=4,max_executions=2,correction_allowance=0,feedback_events=None,feedback_cycle=None,transient_recovery=None,process_profile=None,active_window_seconds=None,independent_assessment=None,unlimited_policy_sha256=None,criterion_return_profile=None):
    load('criterion_profile','engine-criterion-return.py').validate(criterion_return_profile)
    unlimited=feedback_policy.unlimited({'unlimited_policy_sha256':unlimited_policy_sha256})
    if unlimited:
        if max_decisions is not None or max_executions is not None or expires_at is not None or active_window_seconds is not None:raise ValueError('unlimited totals must be explicitly absent')
    elif type(max_decisions) is not int or not 1<=max_decisions<=90 or type(max_executions) is not int or not 0<=max_executions<=100:raise ValueError('browser release bounds')
    if type(correction_allowance) is not int or not 0<=correction_allowance<=20:raise ValueError('bounded technical correction allowance')
    if feedback_cycle is not None:
        feedback_policy.validate(feedback_cycle,max_decisions)
    if process_profile not in (None,'ordinary-memory@v1'):raise ValueError('unadmitted process profile')
    assessment.validate(independent_assessment,e.ROOT)
    clock.validate(active_window_seconds)
    recovery.validate(transient_recovery)
    if transient_recovery is not None and feedback_cycle is None:raise ValueError('provider recovery requires persistent feedback-session authority')
    feedback_events=feedback_events or []
    validate_feedback_events(feedback_events,origin['public']['parents'])
    return {**({'criterion_return_profile':criterion_return_profile} if criterion_return_profile is not None else {}),**({'unlimited_policy_sha256':unlimited_policy_sha256} if unlimited else {}),**({'independent_assessment':independent_assessment} if independent_assessment is not None else {}),**({'active_window_seconds':active_window_seconds} if active_window_seconds is not None else {}),**({'process_profile':process_profile} if process_profile else {}),**({'transient_recovery':transient_recovery} if transient_recovery is not None else {}),**({'feedback_cycle':feedback_cycle} if feedback_cycle is not None else {}),'feedback_events':feedback_events,'version':VERSION,'directory':str(Path(root).resolve()),'origin':origin,'configuration_sha256':e.sha(configuration),'campaign_sha256':e.sha(campaign),'model':base.MODEL,'max_decisions':max_decisions,'max_calls':None if unlimited else 2*max_decisions+correction_allowance,'max_executions':max_executions,'correction_allowance':correction_allowance,'output_profile':'strict-json-bound@v2','completion_tokens':32768,'context_profile':base.capacity.PROFILE,'input_tokens':base.capacity.INPUT_TOKENS,'payload_bytes':base.capacity.PAYLOAD_BYTES,'incomplete_regeneration':True,'review_format_regeneration':True,'max_review_rejections':3,'representation_regeneration':True,'representation_profile':'browser-exact-edits@v2','expires_at':expires_at,'implementation':pins(),'binary_sha256':d.file_sha(e.ROOT/'target/debug/tsr'),'criteria_sha256':e.sha(base.review_contract.CRITERIA)}
def continuation_key(value):
    index=value.get('index')
    if type(index) is not int or index<0:raise ValueError('continuation index')
    checkpoint=value.get('checkpoint_id')
    if checkpoint is None:return str(index)
    if not isinstance(checkpoint,str) or len(checkpoint)!=64 or any(c not in '0123456789abcdef' for c in checkpoint):raise ValueError('continuation checkpoint identity')
    return checkpoint+'_'+str(index)

def semantic_candidate(response,basis):
    def semantic(value):
        if isinstance(value,dict):return {k:semantic(v) for k,v in value.items() if k not in ('binding','source_digest','inspection_handle')}
        if isinstance(value,list):return [semantic(v) for v in value]
        return value
    return e.sha(semantic({'proposals':response.get('proposals',[]),'choice':{k:response[k] for k in ('retain','abstain') if k in response},'return_basis':basis,**({'pursuit_expectations':response['pursuit_expectations']} if response.get('pursuit_expectations') else {})}))

def return_witness(execution,basis):
    completion=execution.get('state',{}).get('goal_completion',{})
    receipt=completion.get('test_receipt');snapshot=completion.get('snapshot')
    linked=[item for item in basis.values() if item.get('snapshot')==snapshot and any(r.get('receipt')==receipt for r in item.get('resolved_observations',[]))] if receipt and snapshot else []
    return {'reviewed_return':bool(linked),'execution_receipt':receipt,'source_snapshot':snapshot,'return_basis_sha256':e.sha(basis),'linked_basis_count':len(linked)}

class Adapter(recovery.Recovery,base.Adapter):
    recovery_load=staticmethod(load)
    recovery_sha=staticmethod(e.sha)
    recovery_stamp=staticmethod(d.stamp)
    journal_limit=base.capacity.JOURNAL_BYTES
    review_reference_aliases=True
    narrative_limit=web.NATIVE_RESPONSE_LIMIT
    release_profiles=(VERSION,)
    def validate_current_admission(self):
        a=self.admission;origin=a['origin'];c=self.controller
        if set(origin)-{'directory','public','initial_sources','prior_review_feedback','prior_validation_feedback'} or not {'directory','public','initial_sources'}<=set(origin):raise ValueError('browser origin fields')
        if str(c.directory)!=origin['directory']:raise ValueError('native origin directory')
        config=c.cli('project','config')
        expected=admission(self.root,origin,config,self.ledger.manifest,a['expires_at'],max_decisions=a['max_decisions'],max_executions=a['max_executions'],correction_allowance=a['correction_allowance'],feedback_events=a['feedback_events'],feedback_cycle=a.get('feedback_cycle'),transient_recovery=a.get('transient_recovery'),process_profile=a.get('process_profile'),active_window_seconds=a.get('active_window_seconds'),independent_assessment=a.get('independent_assessment'),unlimited_policy_sha256=a.get('unlimited_policy_sha256'),criterion_return_profile=a.get('criterion_return_profile'))
        if a!=expected or time.time()>=feedback_policy.deadline(a,d.stamp):raise ValueError('browser admission drift/expiry')
        if feedback_policy.unlimited(a) and self.ledger.status().get('unlimited_policy_sha256')!=a['unlimited_policy_sha256']:raise ValueError('unlimited policy not recorded in campaign')
        if config.get('browser') is not True or config.get('browser_container_only') is not True or config['image']!=IMAGE or config['user']!='501:20' or config.get('cargo') or config.get('named_package') or config.get('full_workspace_oracle'):raise ValueError('pinned container-only browser profile required')
        # Retained projects keep their immutable protected verifier. This audited
        # predecessor differs only in bounded error-message detail, not isolation/checks.
        harness=config['protected'].get('browser-harness.cjs')
        if not supported_harness(harness):raise ValueError('protected harness drift')
        if sorted(config['editable'])!=sorted(origin['initial_sources']) or sorted(config['editable'])!=sorted(origin['public']['editable']):raise ValueError('initial source inventory drift')
        if any(not isinstance(text,str) or not 0<len(text.encode())<=1048576 for text in origin['initial_sources'].values()) or sum(len(text.encode()) for text in origin['initial_sources'].values())>1048576:raise ValueError('reviewed baseline source bounds')
        if len(config['cases'])!=1 or config['timeout_ms']>30000:raise ValueError('bounded one-probe browser profile')
        if any(p.is_symlink() for p in (self.root,*self.root.parents)):raise ValueError('adapter symlink')
    def _wire(self,body,timeout):
        result=clock.wire(self.admission,self.read()[2],d.stamp,body,timeout,super()._wire)
        if result.get('receipt',{}).get('usage_status')=='local_not_dispatched':
            marker={'version':'host-no-network-dispatch@v1','admission_sha256':e.sha(self.admission),'body_sha256':e.sha(body),'clock':self.read()[2]['session_clock'],'reason':'active_or_outer_deadline_expired_before_network','provider_response':False,'charge_microusd':0}
            self.op('local_no_dispatch_'+e.sha(marker),lambda:marker)
        return result
    def clock_deadline(self):
        return clock.deadline(self.admission,self.read()[2],d.stamp)
    def start_clock(self):
        return clock.start(self.admission,self.op,time.time(),d.stamp)
    def call(self,kind,body):
        if self.admission.get('active_window_seconds'):
            policy=self.admission.get('transient_recovery') or {}
            clock.require(self.admission,self.read()[2],d.stamp,240+policy.get('backoff_seconds',0))
        ticket=load('browser_predispatch','engine-browser-predispatch.py').ticket(self)
        if ticket is not None and (kind!=ticket.get('kind','proposal') or body!=ticket['body']):raise ValueError('recovered callback body changed')
        return super().call(kind,body)
    def enroll(self):
        # Scope authority is a new exact suite grant, never the old graph grant.
        suite=e.sha({'public':self.admission['origin']['public'],'configuration':self.admission['configuration_sha256']})
        if self.ledger.manifest.get('fixture_only') and not feedback_policy.unlimited(self.admission):suite=None
        return self.op('enroll',lambda:self.ledger.enroll(self.trial,e.sha(self.admission),suite_sha256=suite) or {'enrolled':True})
    def scope_grant(self,authority_sha256,reviewer,session_cap_microusd,*,extend_authority=False):
        """Prepare only; use the exact same canonical identities as enrollment."""
        self.validate_current_admission()
        with self.ledger.locked() as (_,rows):
            status=self.ledger.fold(rows)[0];head=rows[-1]['hash']
        if status['stopped']:raise ValueError('unresolved campaign effects')
        a=self.admission
        if feedback_policy.unlimited(a):
            if status.get('unlimited_policy_sha256')!=a['unlimited_policy_sha256'] or session_cap_microusd is not None:raise ValueError('unlimited campaign policy mismatch')
            return {'version':'native-browser-harness@v3','session_trial_sha256':self.trial,'session_admission_sha256':e.sha(a),'manifest_sha256':self.ledger.hash,'ledger_prefix_head':head,'previous_max_calls':status['effective_max_calls'],'max_calls':None,'previous_max_execution_groups':status['effective_max_execution_groups'],'max_execution_groups':None,'cap_microusd':None,'reservation_microusd':50000,'authority_sha256':authority_sha256,'reviewer':reviewer,'suite_manifest_sha256':e.sha({'public':a['origin']['public'],'configuration':a['configuration_sha256']}),'source_class':'newly-authored-nonprivate-browser-project','baseline':{'calls':status['calls'],'execution_groups':status['execution_groups'],'settled_microusd':status['known_cost_microusd']},'session_max_calls':None,'session_max_execution_groups':None,'session_cap_microusd':None,'unlimited_policy_sha256':a['unlimited_policy_sha256'],'expires_at':None}
        return {'version':'native-browser-harness@v2' if extend_authority else 'native-browser-harness@v1','session_trial_sha256':self.trial,'session_admission_sha256':e.sha(a),
            'manifest_sha256':self.ledger.hash,'ledger_prefix_head':head,'previous_max_calls':status['effective_max_calls'],'max_calls':max(status['effective_max_calls'],status['calls']+a['max_calls']) if extend_authority else status['effective_max_calls'],
            'previous_max_execution_groups':status['effective_max_execution_groups'],'max_execution_groups':max(status['effective_max_execution_groups'],status['execution_groups']+a['max_executions']) if extend_authority else status['effective_max_execution_groups'],
            'cap_microusd':max(status['effective_cap_microusd'],status['known_cost_microusd']+session_cap_microusd) if extend_authority else status['effective_cap_microusd'],'reservation_microusd':50000,'authority_sha256':authority_sha256,'reviewer':reviewer,
            'suite_manifest_sha256':e.sha({'public':a['origin']['public'],'configuration':a['configuration_sha256']}),'source_class':'newly-authored-nonprivate-browser-project',
            'baseline':{'calls':status['calls'],'execution_groups':status['execution_groups'],'settled_microusd':status['known_cost_microusd']},
            'session_max_calls':a['max_calls'],'session_max_execution_groups':a['max_executions'],'session_cap_microusd':session_cap_microusd}
    def project(self,view):
        self.validate_current_admission();request=self.controller.request();coding=request['coding'];origin=self.admission['origin'];public=copy.deepcopy(origin['public'])
        applied=[v for k,v in self.read()[2].items() if k.startswith('feedback_')]
        if applied:public['parents']=applied[-1].get('acceptance_parents',applied[-1]['parents'])
        if any(view[k]!=v for k,v in (('state_root',request['state_root']),('source_snapshot',coding['snapshot']),('scope',public['scope']))):raise ValueError('stale browser view')
        work=self.controller.directory/'coding-work';paths=list(work.rglob('*'))
        if any(p.is_symlink() for p in paths) or {p.relative_to(work).as_posix() for p in paths if p.is_file()}!=set(origin['initial_sources'])|set(public['protected']):raise ValueError('unadmitted workspace content')
        # A fresh independently admitted public baseline may be a prior checkpoint.
        # Its source/receipt/history digests are explicit scope, never automatically imported.
        # Later versions must be a sequence of this journal's exact generated edits.
        # Actual authenticated source transitions, never a lattice of unchosen edits.
        actual={f['path']:f['source'] for f in coding['files'] if f['editable']}
        identities=[(snapshot,{p:e.sha(v) for p,v in origin['initial_sources'].items()}) for snapshot in origin['public']['source_snapshots']]
        identities += [(v['snapshot'],v['files']) for k,v in self.read()[2].items() if k.startswith('executed_source_')]
        if (coding['snapshot'],{p:e.sha(v) for p,v in actual.items()}) not in identities:
            # Recover only local bookkeeping after an authenticated completed execution.
            completed=[v for k,v in self.read()[2].items() if k.startswith('execution_') and v.get('state',{}).get('goal_completion',{}).get('snapshot')==coding['snapshot']]
            if not completed:raise ValueError('source lacks authenticated execution lineage')
            self.record_execution_source(completed[-1])
        done=self.read()[2];owned=set();snapshots=set(public['source_snapshots'])
        for key,result in done.items():
            if key.startswith('execution_'):
                execution=result.get('execution',result);owned.update(execution.get('receipts',[]))
                completion=execution.get('state',{}).get('goal_completion',{})
                if completion.get('snapshot'):snapshots.add(completion['snapshot'])
                if completion.get('test_receipt'):owned.add(completion['test_receipt'])
        if coding['snapshot'] not in snapshots:raise ValueError('source lacks owned completed execution')
        returns=coding['relational']['inputs'].get('returns',[])
        for item in returns:
            if item.get('receipt') not in owned and e.sha(item) not in public['public_returns']:raise ValueError('unowned browser receipt')
        produced=[e.relational_identity(p['proposal']) for r in self.generated() for p in r.get('proposals',[]) if p['kind']=='relational']
        history=coding['relational'].get('history',[])
        if any(e.relational_identity(h['proposal']) not in produced and not web.owns_relationship_history(h,public['public_relationships']) for h in history):raise ValueError('unowned relationship')
        public['source_snapshots']=[coding['snapshot']];public['public_returns']=[e.sha(r) for r in returns];public['public_relationships']=[e.sha(h) for h in history]
        views=[v for k,v in self.read()[2].items() if k.startswith('source_view_') and v['state_root']==request['state_root']]
        load('browser_desire','engine-browser-desire.py').verify_history(self,request)
        load('browser_expansion','engine-browser-expansion.py').verify(self,request)
        public['public_preferences']=[e.sha(h) for h in coding['relational']['inputs'].get('generated_preferences',[])]
        public['public_meanings']=[e.sha(h) for h in coding['relational']['inputs'].get('public_desires',{}).get('history',[])]
        context=web.project(request,public,views[-1]['inspect'] if views else None,getattr(self,'context_budget',self.admission['payload_bytes']-65536))
        if applied:context['feedback_provenance']=[{'id':v['event']['id'],'attribution':v['event']['attribution'],'publication_state_root':v['state_root'],'parents_sha256':e.sha(v['parents']),'scope':'admitted public guidance only, not observed player behavior or new authority'} for v in applied]
        context['constraints']['completion_capacity']='32768 tokens including reasoning; complete incremental proposals are permitted. This is a session allocation, not the advertised model maximum.'
        if view['branch']=='reconsider':
            context['advisory_reconsideration']=view['reconsideration']
            for key in ('edits_or_abstention','existing','edit_layout'):context['response_contracts'].pop(key,None)
            context['response_envelope']={'instruction':'Return {alternatives:[one advisory candidate]}, {deliberation:{artifact:{retain:reason},development:one advisory candidate,artifact_dependency:independent|requires_development}}, {retain:reason,development_declined:reason} (development_declined names the development forms in choice_summary you considered and why none is worth proposing now), or {abstain:reason}. No edit, verification execution or inspection in this dedicated reconsideration. Prior lineage is context, not a mandatory target. Independent review is required. Retention does not imply fulfillment or acceptance.','max_alternatives':1}

        load('browser_pursuit_reflection','engine-pursuit-reflection.py').project(self,request,context)
        load('browser_criterion_return','engine-criterion-return.py').project(self,request,context)
        feedback=[]
        for prior in (origin.get('prior_review_feedback'),origin.get('prior_validation_feedback')):
            if not prior:continue
            inherited_validation=prior is origin.get('prior_validation_feedback')
            if inherited_validation and (prior.get('code')!='no_insight_meaning_drift' or prior.get('status')!='unadmitted_response_independently_assessed' or not isinstance(prior.get('candidate_sha256'),str) or len(prior['candidate_sha256'])!=64 or any(c not in '0123456789abcdef' for c in prior['candidate_sha256'])):raise ValueError('prior validation identity/constraint')
            if set(prior)!=({'source_snapshot','scope','parents_sha256','failed_response_sha256','assessment','attribution','status'}|({'candidate_sha256','code'} if inherited_validation else set())) or prior['status'] not in ('unadmitted_response_independently_assessed','admitted_review_unknown_missing_evidence','independent_gameplay_assessment') or not isinstance(prior['assessment'],str) or not prior['assessment'].strip():raise ValueError('prior criticism contract')
            for key in ('source_snapshot','scope','parents_sha256','failed_response_sha256'):
                if not isinstance(prior[key],str) or len(prior[key])!=64 or any(c not in '0123456789abcdef' for c in prior[key]):raise ValueError('prior criticism identity')
            if prior['scope']!=view['scope'] or prior['parents_sha256']!=e.sha(origin['public']['parents']) or prior['source_snapshot'] not in origin['public']['source_snapshots']:raise ValueError('prior criticism root/parents/source mismatch')
            if not isinstance(prior['attribution'],str) or not 0<len(prior['attribution'].encode())<=256:raise ValueError('prior criticism attribution')
            # Artifact verification is performed in reviewed preparation and pinned by admission.
            if prior['source_snapshot']==view['source_snapshot']:feedback.append(prior)
        for key,item in self.read()[2].items():
            if key.startswith('review_rejection_'):
                if item['source']!={k:view[k] for k in ('state_root','source_snapshot','scope')}:
                    carried=[v for k,v in self.read()[2].items() if k.startswith('session_continue_') and v.get('after')=={k:view[k] for k in ('state_root','source_snapshot','scope')} and v.get('evidence_anchor')==self.evidence_anchor() and key in v.get('criticism_keys',[])]
                    if not carried:continue
                feedback.append(item)
        if feedback:context['review_criticism']={'status':'attributed fallible judgments, not observed gameplay or authority','items':feedback,'instruction':'Address relevant criticism in a fresh proposal, retain a justified alternative, or abstain. Unknown findings describe insufficient evidence, not a substantive rejection. Never claim a rejected or unknown proposal executed. No obligation to obtain approval by repeated rewording.'}
        native_feedback=[v for k,v in self.read()[2].items() if k.startswith('native_rejection_') and v['source']=={k:view[k] for k in ('state_root','source_snapshot','scope')}]
        if native_feedback:context['native_validation_feedback']={'items':native_feedback,'instruction':'These candidates were not admitted or executed. Make a fresh decision from the current supplied source and exact typed rejection: inspect source, propose a valid current-source action, retain prior meaning, justify development, or abstain. No obligation to obtain acceptance. This is not a formatting correction or observed gameplay.'}
        context['own_decisions']=[{'response':v['response'],'source':self.read()[2].get('bound_output_'+k[len('generated_'):],{}).get('source')} for k,v in self.read()[2].items() if k.startswith('generated_')]
        context['independent_assessments']=[{**v,'applicability':'current_source' if v['snapshot']==view['source_snapshot'] else 'historical_source_only'} for k,v in self.read()[2].items() if k.startswith('independent_assessment_')]
        context['evidence_contract']=base.review_contract.evidence_contract()
        return request,context
    def record_execution_source(self,result):
        request=self.controller.request();coding=request['coding']
        completion=result.get('state',{}).get('goal_completion',{})
        if completion.get('snapshot')!=coding['snapshot']:raise ValueError('execution/source snapshot mismatch')
        record={'snapshot':coding['snapshot'],'receipts':result['receipts'],'files':{f['path']:e.sha(f['source']) for f in coding['files'] if f['editable']}}
        return self.op('executed_source_'+coding['snapshot'],lambda:record)
    def apply_feedback(self,index):
        self.validate_current_admission()
        events=self.admission['feedback_events']
        if type(index) is not int or not 0<=index<len(events):raise ValueError('feedback outside admission')
        done=self.read()[2]
        if 'feedback_'+str(index) in done:return done['feedback_'+str(index)]
        if sum(k.startswith('feedback_') for k in done)!=index:raise ValueError('feedback order')
        if self.ledger.status()['stopped']:raise ValueError('unresolved campaign effect')
        c=self.controller;budget=c.budget()
        if any(budget[k] for k in ('unsettled_requests','unresolved_tool_requests','unknown_usage','unknown_compute_requests','unknown_external_cost_requests')):raise ValueError('unresolved native effect')
        if self.native('request',None)['branch'] not in ('review_required','propose'):raise ValueError('feedback requires stable checkpoint')
        request=c.request();completion=c.cli('project','acceptance','status')
        event=events[index]
        draft=web.feedback_draft(web.checkpoint(request,completion,budget),request,completion,event['id'],event['guidance'],attribution=event['attribution'])
        before={'source':request['coding']['snapshot'],'requirements':completion['requirements_revision'],'config':e.sha(c.cli('project','config')),'parents':request['coding']['relational']['inputs']['parents']}
        def publish():
            result=c.json_cli(('project','acceptance','feedback'),draft['native_feedback'])
            after=c.request()
            if after['coding']['snapshot']!=before['source'] or e.sha(c.cli('project','config'))!=before['config'] or after['coding']['relational']['inputs']['requirements_revision']!=before['requirements']:raise ValueError('feedback changed source/authority/acceptance')
            return {'event':event,'attribution':draft['attribution'],'before':before,'result':result,'parents':after['coding']['relational']['inputs']['parents'],'acceptance_parents':after['coding']['relational']['inputs'].get('acceptance_parents',after['coding']['relational']['inputs']['parents']),'state_root':after['state_root']}
        return self.op('feedback_'+str(index),publish)
    def evidence_anchor(self):
        request=self.controller.request();coding=request['coding']
        return e.sha({'snapshot':coding['snapshot'],'inputs':coding['relational']['inputs'],'history':coding['relational']['history'],'configuration':self.controller.cli('project','config')})
    def continue_session(self,value):
        self.validate_current_admission()
        policy=self.admission.get('feedback_cycle')
        if not policy:raise ValueError('session continuation not admitted')
        done=self.read()[2];key=continuation_key(value);name='session_continue_'+key
        if name in done:
            prior=done[name]
            if prior['instruction']['binding']!=value['binding'] or prior['purpose']!=value['reason']:raise ValueError('continuation replay request mismatch')
            self.session_transition_result(prior)
            return prior
        if feedback_policy.reached(len([k for k in done if k.startswith('session_continue_')]),policy['max_continuations']):raise ValueError('session continuation allowance exhausted')
        before=super().native('request',None)
        if before['branch'] not in ('review_required','propose') or before['binding']!=value['binding']:raise ValueError('stale session continuation')
        budget=self.controller.budget()
        if self.ledger.status()['stopped'] or any(budget[k] for k in ('unsettled_requests','unresolved_tool_requests','unknown_usage','unknown_compute_requests','unknown_external_cost_requests')):raise ValueError('unresolved effects; no continuation')
        anchor=self.evidence_anchor()
        # Only criticism recorded in this adapter, whose evidence anchor still matches.
        keys=[k for k,v in done.items() if k.startswith('review_rejection_') and v.get('evidence_anchor')==anchor]
        instruction={'binding':before['binding'],'message_id':('checkpoint-continuation:'+e.sha({'authorization':policy['authorization_id'],'checkpoint':value['checkpoint_id'],'index':value['index']})) if 'checkpoint_id' in value else policy['authorization_id']+':'+key,'reason':'Previously authorized finite feedback cycle: allow an ordinary reviewed choice. No new wanting, acceptance, action permission or prescribed response.'}
        prepared=self.op('continuation_input_'+key,lambda:{'native_required':before['branch']=='review_required','purpose':value['reason'],'instruction':instruction,'before':{k:before[k] for k in ('state_root','source_snapshot','scope')},'evidence_anchor':anchor,'criticism_keys':keys})
        if prepared['instruction']!=instruction or prepared['evidence_anchor']!=anchor:raise ValueError('continuation preparation drift')
        def issue():
            result=self.controller.json_cli(('project','cycle','continue'),instruction) if prepared['native_required'] else {'continuation':None,'next':before}
            return self.continuation_result(prepared,result)
        return self.op(name,issue)
    def continuation_result(self,prepared,result):
        after=super().native('request',None)
        if self.evidence_anchor()!=prepared['evidence_anchor'] or result['next']!=after:raise ValueError('continuation changed evidence or authority')
        return {**prepared,'result':result,'after':{k:after[k] for k in ('state_root','source_snapshot','scope')}}
    def session_transition_result(self,prepared):
        if prepared['native_required']:
            return self.controller.json_cli(('project','cycle','continuation-receipt'),prepared['instruction'])
        current=super().native('request',None)
        if current['branch']!='propose' or current['binding']!=prepared['instruction']['binding']:raise ValueError('stale acknowledged ordinary-choice transition')
        return {'continuation':None,'next':current}
    def recover_session_transition(self):
        # Launcher owns the single-process lock. A committed native continuation
        # has a read-only exact receipt; absence never authorizes redispatch.
        self.validate_current_admission();rows,pending,done=self.read()
        if pending and load('criterion_ticket_recovery','engine-criterion-return.py').recover_reassessment_record(self):rows,pending,done=self.read()
        if pending and any(k.startswith('meaning_admission_') for k in pending):
            if (self.root/'STOP').exists() or len(pending)!=1 or self.ledger.status()['stopped']:raise ValueError('Stop or unrelated pending effects prevent meaning recovery')
            candidates=[v for k,v in done.items() if k.startswith('meaning_transition_input_')]
            if not candidates:raise ValueError('missing durable meaning transition input')
            result=self.native('meaning_advance_receipt',candidates[-1])
            if 'meaning_admission_'+result['proposal_record'] not in pending:raise ValueError('meaning recovery request mismatch')
            return 1
        if pending and load('browser_criterion_recovery','engine-criterion-return.py').recover(self):rows,pending,done=self.read()
        if pending and assessment.recover(self):rows,pending,done=self.read()
        if not pending:
            executions=[v for k,v in done.items() if k.startswith('execution_')]
            if executions and self.admission.get('independent_assessment'):
                self.record_execution_source(executions[-1]);assessment.collect(self,self.controller.request())
            return 0
        if not self.admission.get('feedback_cycle') or len(pending)!=1 or self.ledger.status()['stopped']:raise ValueError('unresolved adapter effect')
        name,intent=next(iter(pending.items()))
        if not name.startswith('session_continue_'):raise ValueError('unresolved non-continuation effect; no retry')
        prepared=done.get('continuation_input_'+name[len('session_continue_'):])
        if prepared is None:raise ValueError('missing durable continuation instruction')
        result=self.session_transition_result(prepared)
        value=self.continuation_result(prepared,result)
        self.append({'kind':'result','name':name,'intent':e.sha(intent),'value':value})
        return 1
    def complete_pending_return(self):
        """Finish authentic retained returns before changing guidance or making a choice."""
        self.validate_current_admission()
        self.ledger.require_output_eligible(self.trial)
        if (self.root/'STOP').exists() or self.read()[1] or self.ledger.status()['stopped']:
            raise ValueError('Stop or unresolved effects prevent return continuation')
        load('browser_pursuit_reflection','engine-pursuit-reflection.py').reflect(self)
        load('browser_criterion_return','engine-criterion-return.py').reflect(self)

    def native(self,kind,value):
        self.ledger.require_output_eligible(self.trial)
        if kind=='deliberation_component':return load('deliberation_derive','engine-browser-deliberation.py').derive(self,value)
        if kind=='execution_stage_receipt':
            self.validate_current_admission();done=self.read()[2];executions=[v for k,v in done.items() if k.startswith('execution_')]
            if self.read()[1] or self.ledger.status()['stopped'] or type(value.get('index')) is not int or value['index']!=len(executions)-1:raise ValueError('unresolved or wrong execution recovery')
            expected='execution_'+e.sha({'admission':e.sha(self.admission),'binding':value.get('binding')})
            if expected not in done or done[expected]!=executions[-1]:raise ValueError('execution recovery original binding mismatch')
            result=done[expected];request=self.controller.request()
            if result['state']['goal_completion']['snapshot']!=request['coding']['snapshot']:raise ValueError('execution recovery source drift')
            self.record_execution_source(result)
            if self.admission.get('independent_assessment'):assessment.collect(self,request)
            load('browser_pursuit_reflection','engine-pursuit-reflection.py').reflect(self)
            load('browser_criterion_return','engine-criterion-return.py').reflect(self)
            return result
        if kind=='provider_recovery_ready':return self.recovery_ready()
        if kind=='feedback_witness':
            self.validate_current_admission();done=self.read()[2]
            judgment=value['judgment'];ident=judgment['reviewer'].removeprefix('separate-runtime-invocation:');review=done.get('reviewed_'+ident)
            if judgment['decision']!='approve' or not review or 'candidate:'+judgment['response_digest'] not in review['binding']['evidence_ids']:raise ValueError('feedback completion review mismatch')
            generated_id=review.get('generated_request');key='generated_'+str(generated_id);generated=done.get(key)
            if generated is None or generated['response']!=value['response']:raise ValueError('unowned feedback completion candidate')
            review_bound=self.base_bound_component_origin(ident)
            if review_bound['association']['kind']!='review' or review_bound['native_content_sha256']!=e.sha(review['report']) or review['report'].get('verdict')!='accept':raise ValueError('feedback reviewer provenance')
            if review_bound['source']!={k:value['context'][k] for k in ('state_root','source_snapshot','scope')}:raise ValueError('feedback review source drift')
            bound=(load('browser_deliberation','engine-browser-deliberation.py').verify(self,generated_id,generated['response'],generated['return_basis'],historical_context=value['context']) if generated_id.startswith('component_') else self.base_bound_component_origin(generated_id))
            capture=done.get('qualitative_'+value['advance']['proposal_record'])
            if not capture or capture.get('advance_result')!=value['advance'] or capture.get('advance_input',{}).get('response')!=value['response'] or any(capture['advance_input']['review'].get(k)!=v for k,v in judgment.items()):raise ValueError('feedback admission association')
            if value['execution'] not in [v for k,v in done.items() if k.startswith('execution_')]:raise ValueError('unowned returned execution')
            basis_hash=e.sha(base.review_contract.indexed_basis(value['response'],generated['return_basis']))
            if bound['decoded_sha256']!=e.sha([generated['response'],generated['return_basis']]) or review.get('return_basis_sha256')!=basis_hash or 'return_basis:'+basis_hash not in review['binding']['evidence_ids']:raise ValueError('feedback completion content/basis drift')
            witness=return_witness(value['execution'],generated['return_basis'])
            return {**witness,'proposal_record':value['advance']['proposal_record'],'review_record':value['advance']['review_record'],'captured_context_sha256':e.sha(value['context']),'generated_request':key[len('generated_'):],'independent_review_request':ident}
        if kind=='continue_session':return self.continue_session(value)
        if kind=='session_transition_receipt':
            self.validate_current_admission()
            if self.read()[1]:raise ValueError('unresolved transition')
            record=self.read()[2].get('session_continue_'+continuation_key(value))
            if record is None:raise ValueError('missing completed transition')
            receipt=self.session_transition_result(record)
            if self.continuation_result(record,receipt)!=record:raise ValueError('stale transition receipt')
            return {'reason':record['purpose'],'transition':record}
        if kind=='preview' and self.admission.get('feedback_cycle'):
            before=super().native('request',None)
            try:return super().native(kind,value)
            except subprocess.TimeoutExpired:
                # subprocess.run has killed/waited for the child. Native preview is
                # rollback-only. No other operation is eligible for this local retry.
                if hasattr(self,'evidence'):self.evidence.retain('local_controller_timeout')
                after=super().native('request',None);loop.Cycle.same_source(before,after)
                if before['binding']!=after['binding'] or self.read()[1] or self.ledger.status()['stopped']:raise
                key='local_preview_retry_'+e.sha({'binding':before['binding'],'response':value})
                if key in self.read()[2] or time.time()+120>feedback_policy.deadline(self.admission,d.stamp):raise
                self.op(key,lambda:{'binding':before['binding'],'response_sha256':e.sha(value),'provider_replayed':False,'reason':'known terminated rollback-only preview; unchanged native binding'})
                return super().native(kind,value)

        if kind=='meaning_advance_receipt':
            self.validate_current_admission()
            if (self.root/'STOP').exists() or self.ledger.status()['stopped'] or any(not k.startswith(('meaning_admission_','qualitative_')) for k in self.read()[1]):raise ValueError('unresolved effects or Stop prevent meaning recovery')
            result=self.controller.json_cli(('project','cycle','meaning-advance-receipt'),value)
            if result.get('qualitative_admitted'):
                return load('browser_deliberation_capture','engine-browser-deliberation.py').capture_relationship(self,value,result)
            return load('browser_desire','engine-browser-desire.py').admit(self,value,result)
        if kind=='advance' and any(p.get('kind') in ('relational','public_desire','contrast_preference','success_preference','provisional_preference') for p in value['response']['proposals']):
            if len(value['response']['proposals'])!=1:raise ValueError('public meaning must be independently reviewed alone')
            self.validate_current_admission()
            self.verify_owned_review(value['review'],value['response'])
            self.op('meaning_transition_input_'+e.sha(value),lambda:copy.deepcopy(value))
            result=self.controller.json_cli(('project','cycle','preference-advance' if value['response']['proposals'][0]['kind'] in ('contrast_preference','success_preference','provisional_preference') else 'meaning-advance'),value)
            if result.get('qualitative_admitted'):
                return load('browser_deliberation_capture','engine-browser-deliberation.py').capture_relationship(self,value,result)
            return load('browser_desire','engine-browser-desire.py').admit(self,value,result)
        if kind=='resume':
            if self.admission.get('active_window_seconds'):clock.require(self.admission,self.read()[2],d.stamp,self.resume_timeout(super().native('request',None)))
            result=super().native(kind,value)
            self.record_execution_source(result)
            assessment.collect(self,self.controller.request())
            load('browser_pursuit_reflection','engine-pursuit-reflection.py').reflect(self)
            load('browser_criterion_return','engine-criterion-return.py').reflect(self)
            return result
        if kind!='reject_proposal':return super().native(kind,value)
        self.validate_current_admission()
        if value['judgment']['decision'] not in ('reject','unknown'):raise ValueError('only settled semantic rejection or uncertainty can be recorded')
        view=super().native('request',None);loop.Cycle.same_source(value['source'],view)
        owned=self.verify_owned_review(value['judgment'],value['response'])
        current=[v for k,v in self.read()[2].items() if k.startswith('review_rejection_')]
        self.require_rejection_room()
        generated=[v for k,v in self.read()[2].items() if k.startswith('generated_') and v['response']==value['response']]
        if 'generated_request' in owned:
            generated=[v for k,v in self.read()[2].items() if k=='generated_'+owned['generated_request'] and v['response']==value['response']]
        if len(generated)!=1:raise ValueError('unowned rejection candidate')
        candidate=semantic_candidate(value['response'],generated[0]['return_basis'])
        if any(v['candidate_sha256']==candidate and v.get('evidence_anchor')==self.evidence_anchor() for v in current):raise self.known_stop(ValueError('repeated identical rejected candidate; no reviewer resampling'))
        record={'evidence_anchor':self.evidence_anchor(),'source':{k:view[k] for k in ('state_root','source_snapshot','scope')},'decision':value['judgment']['decision'],'candidate_sha256':candidate,'response_digest':value['judgment']['response_digest'],'attribution':value['judgment']['reviewer'],'report':owned['report'],'grants_authority':False,'executed':False}
        return self.op('review_rejection_'+e.sha(value['judgment']['reviewer']),lambda:record)
    def wire_context(self,request,context):
        return load('browser_proposal_references','engine-proposal-references.py').project(context,request)
    def proposal_body(self,view,context):
        # One shared text for every arm and branch: the paired comparison requires identical
        # system messages. Each limit is stated once, in the ground rules, not attached to
        # each mention of development, so no option reads as the cautious default.
        references=load('browser_proposal_references','engine-proposal-references.py')
        self.offered_development=references.development_forms(context.get('choice_summary') if isinstance(context,dict) else None)
        system=('Consider the current wants, relationships, possibilities and supplied experience for this public browser project, and decide what to propose. '
            'choice_summary lists the proposal forms available now and their prerequisites; it is not approval or a ranking. '
            'Three kinds of choice are open and none is preferred: keep the artifact, change the artifact, or develop meaning (a relationship, a meaning development, or provisional wanting). '
            'Artifact and meaning are decided separately: a deliberation envelope pairs one artifact choice (an edit or retention) with one development choice (an advisory proposal or null), with explicit dependency and separate reviews; keeping the artifact alone does not preclude development. '
            'Provisional wanting needs only a coherent connection to current contributors; it does not need a defect, corrective evidence or observed success, and its fulfillment stays unknown. '
            'Ground rules for every choice: use the exact user words, current source and attributed observations; state uncertainty and distinguish delivered evidence from the evidence you rely on, citing observation/source identities when useful; '
            'unknown is not unmet; generating a possible wanting is distinct from asserting an empirical fact or attributing a preference to the user; citations and reasons are not proof of causal learning or player satisfaction; no novelty, activity, relationship or preferred design is required, and none is ruled out. '
            'Positive feedback alone does not request a change, but an explicit creation or improvement request is an active goal, including when phrased as a question. Open design choices are delegated work, not inherently missing information. Passing checks are not satisfaction of a renewed goal. Initial designs can be untested and need no prior player feedback. '
            'Consider alternatives and return 1..4 distinct candidates allowed by the supplied response contracts; do not mix executable and advisory candidates in an alternatives batch; use the deliberation envelope for one of each. You may instead inspect an admitted source window. '
            'Exact edits use old_text/new_text. A complete incremental edit is allowed; never return partial JSON or unfinished code. Preserve protected checks, runtime, permissions and acceptance. '
            'The controller previews proposals, obtains independent review, and executes only with authorization; never claim these later steps happened already. '
            'To keep both artifact and meaning unchanged, return {retain:reason, development_declined:reason} whenever choice_summary lists any development form: development_declined names the development forms you considered and why none is worth proposing now. When choice_summary lists no development form, return {retain:reason}. '
            'Retention ends this checkpoint, not future separately admitted input. Use {abstain:reason} for inability or declining further work; this stops the scheduled session. Neither creates user acceptance. Return only the supplied JSON contract.')
        if view['branch']=='reconsider':system+=' This is advisory reconsideration: a new relationship, revision, same-identity development or provisional/supported wanting may be proposed. Prior lineage is context, not a required target. Artifact intent may only retain here; no execution. Abstention remains valid.'
        return self.body(system,context)
    def check_retention_reason(self,value):
        if not isinstance(value['retain'],str) or not value['retain'].strip() or len(value['retain'].encode())>self.narrative_limit:raise ValueError('retention reason required within response capacity')
    def check_proposal_shape(self,value):
        helper=load('browser_proposal_shape','engine-project-output.py')
        offered=getattr(self,'offered_development',[])
        if isinstance(value,dict) and set(value) in ({'retain'},{'retain','development_declined'}):
            self.check_retention_reason(value)
            if 'development_declined' in value:
                x=value['development_declined']
                if not isinstance(x,str):helper.fail('$.development_declined','string_required',expected_type='string')
                references=load('browser_proposal_references','engine-proposal-references.py')
                room=self.narrative_limit-len((value['retain']+references.DECLINED_PREFIX).encode())
                if not x.strip() or len(x.encode())>room:helper.fail('$.development_declined','text_bound',nonblank=True,max_utf8_bytes=max(0,room))
            elif offered:
                # Retention is a decision about development too, at the same cost as proposing it.
                helper.fail('$','required_field',required_fields=['development_declined','retain'])
            return
        if isinstance(value,dict) and set(value)=={'inspect'}:return
        if isinstance(value,dict) and set(value)=={'deliberation'}:
            parts=load('deliberation_shape','engine-browser-deliberation.py').shape(value)
            if offered and parts['development'] is None and isinstance(parts['artifact'],dict) and set(parts['artifact'])=={'retain'}:
                # Retaining both through the envelope must not skip the development decision.
                helper.fail('$.deliberation.development','object_required',expected_type='object',valid_locations=['$.retain','$.development_declined'])
            for name in ('artifact','development'):
                item=parts[name]
                if item is None:continue
                if set(item)=={'retain'}:self.check_retention_reason(item)
                else:self.check_proposal_shape({'alternatives':[item]})
            return
        if isinstance(value,dict) and 'abstain' not in value:
            # A top-level field that belongs in an alternative: say where, never echo unknown names.
            misplaced=sorted(set(value)&{'edits','rationale','expectation','existing'})
            if misplaced:helper.fail('$','unsupported_field',allowed_fields=['alternatives'],required_fields=['alternatives'],valid_locations=['$.alternatives[0].'+k for k in misplaced])
        helper.proposal_shape(value,text_limit=self.narrative_limit)
        if isinstance(value,dict) and set(value)=={'alternatives'} and isinstance(value['alternatives'],list) and len(value['alternatives'])==1:
            item=value['alternatives'][0]
            if isinstance(item,dict) and set(item)=={'abstain'} and isinstance(item['abstain'],str) and item['abstain'].strip():
                helper.fail('$.alternatives[0].abstain','unsupported_field',valid_locations=['$.abstain'])
    component_sha=staticmethod(e.sha)
    def base_bound_component_origin(self,ident):
        done=self.read()[2]
        record=done['bound_output_'+ident]
        return base.bound_output.verify_envelope(record,admission=self.admission,ident=ident,dispatch=done['dispatch_'+ident],settled=done.get('settle_'+ident),source=record['source'])
    def verify_generated_output(self,ident,response,basis):
        if ident.startswith('component_'):return load('deliberation_verify','engine-browser-deliberation.py').verify(self,ident,response,basis)
        return super().verify_generated_output(ident,response,basis)
    def proposal_representation_evidence(self,request,record,ident,response,basis):
        if ident.startswith('component_'):return {'kind':'explicit_component_derivation','proof':record['derivation'],'original_association':record['original_association']['association'],'not_a_new_model_response':True}

        wire=self.read()[2]['request_body_'+ident]['body']
        helper=load('browser_review_representation','engine-proposal-references.py')
        references=json.loads(wire['messages'][1]['content'])['proposal_references']
        captured=self.read()[2].get('proposal_native_'+references['request_sha256'])
        # Old records retain strict current-request reconstruction. New records
        # retain the exact local generation request, never exporting private state.
        return helper.review_provenance(request,record,wire,response,basis,captured=captured)
    def rejection_count(self):
        done=self.read()[2]
        return sum(k.startswith(('native_rejection_','review_rejection_')) for k in done)+int(bool(self.admission['origin'].get('prior_validation_feedback')))
    def require_rejection_room(self):
        if self.rejection_count()>=self.admission['max_review_rejections']:raise self.known_stop(ValueError('shared semantic rejection allowance exhausted'))
    def resume_criterion_reassessment(self,name,body):
        return load('criterion_reassessment','engine-criterion-return.py').resume_reassessment(self,name,body)
    def retained_format_feedback(self,source,current):
        done=self.read()[2];dispatches=[k[len('dispatch_'):] for k in done if k.startswith('dispatch_')]
        if not dispatches:return None
        records=[v for k,v in done.items() if k.startswith('format_correction_') and v.get('event',{}).get('failed_request')==dispatches[-1]]
        if not records:
            durable=[v for v in self.corrections.events() if v.get('failed_request')==dispatches[-1] and 'feedback' in v and 'request_source' in v]
            if len(durable)==1:
                event=durable[0];records=[{'event':event,'feedback':event['feedback'],'source':event['source'],'request_source':event['request_source']}]
        if not records:
            latest=dispatches[-1];saved=done.get('request_body_'+latest,{}).get('body')
            if saved and not any(k in done for k in ('generated_'+latest,'reviewed_'+latest,'rejected_'+latest,'provider_zero_'+latest)):
                original=copy.deepcopy(saved);messages=original.get('messages',[])
                if len(messages)>2 and messages[-1].get('role')=='user':
                    try:correction=json.loads(messages[-1]['content'])
                    except (ValueError,KeyError,TypeError):correction={}
                    original['messages']=messages[:-1]
                    if set(correction)=={'protocol_correction'} and base.correction.digest(original)==source:
                        feedback=correction['protocol_correction']
                        if self.corrections.body(original,feedback)==saved:
                            records=[{'event':v,'feedback':v['feedback'],'source':v['source'],'request_source':v['request_source']} for v in self.corrections.events() if v.get('source')==source and v.get('feedback')==feedback and 'request_source' in v]
        if not records:return None
        if len(records)!=1:raise ValueError('ambiguous correction recovery')
        record=records[0];ident=record['event']['failed_request']
        if record.get('source')!=source or record.get('request_source')!={k:current[k] for k in ('state_root','source_snapshot','scope')} or record['event'] not in self.corrections.events() or 'settle_'+ident not in done or 'rejected_'+ident not in done or self.ledger.status()['stopped']:raise ValueError('correction recovery source/accounting mismatch')
        if not isinstance(record.get('feedback'),dict):raise ValueError('missing durable correction feedback')
        self.op('format_correction_'+str(record['event']['ordinal']),lambda:record)
        return record['feedback']
    def predispatch_tokens(self,body):
        return base.capacity.prompt_upper(body)+body['max_completion_tokens']
    def recover_settled_response(self):
        self.validate_current_admission()
        return load("browser_response_recovery","engine-browser-response-recovery.py").recover(self)
    def replay_checked(self,kind,body):
        result=load("browser_response_replay","engine-browser-response-recovery.py").replay(self,kind,body)
        if result is not None:
            loop.Cycle.measured({'response':{},'receipt':result[2]},'response',self.admission.get('completion_tokens',8192),self.admission.get('input_tokens',32768))
        return result
    def can_resume_checked(self,kind,body):
        wire,_=base.bound_output.prepare_review_body(body,aliases=self.review_reference_aliases)
        feedback=self.retained_format_feedback(base.correction.digest(wire),self.native('request',None))
        if feedback:wire=self.corrections.body(wire,feedback)
        ticket=load("browser_predispatch_stage","engine-browser-predispatch.py").ticket(self)
        return (ticket is not None and ticket.get("kind","proposal")==kind and ticket["body"]==wire) or self.replay_checked(kind,wire) is not None
    def recover_predispatch(self):
        self.validate_current_admission()
        return load('browser_predispatch','engine-browser-predispatch.py').recover(self)
    def recovery_ready(self):
        self.validate_current_admission()
        _,pending,done=self.read()
        if pending or self.ledger.status()['stopped']:raise ValueError('unresolved effect; no callback recovery')
        ticket=load('browser_predispatch','engine-browser-predispatch.py').ticket(self)
        if ticket is not None:return {'ready':True,'kind':'proven_not_dispatched','request':ticket['request']}
        response=load('browser_response_ready','engine-browser-response-recovery.py').latest(self)
        if response is not None:
            self.replay_checked(response['kind'],response['record']['body'])
            return {'ready':True,'kind':'settled_content','request':response['request']}
        dispatches=[k[len('dispatch_'):] for k in done if k.startswith('dispatch_')]
        if dispatches:
            ident=dispatches[-1];record=done.get('native_rejection_'+ident)
            if record is not None:
                current=self.native('request',None);source={k:current[k] for k in ('state_root','source_snapshot','scope')}
                association=record['association'];dispatch=done['dispatch_'+ident]
                if association.get('kind')!='proposal' or record['code'] not in ('no_insight_meaning_drift','stale_edit_span') or record['candidate_sha256']!=e.sha(record['candidate']):raise ValueError('consistency recovery provenance mismatch')
                base.bound_output.verify_envelope({'association':association,'model_content':record['candidate'],'source':record['source']},admission=self.admission,ident=ident,dispatch=dispatch,settled=done.get('settle_'+ident),source=source)
                if self.rejection_count()>self.admission['max_review_rejections']:raise ValueError('rejection allowance drift')
                return {'ready':True,'kind':'settled_native_consistency_feedback','request':ident}
        records=[v for k,v in done.items() if k.startswith('format_correction_') and dispatches and v.get('event',{}).get('failed_request')==dispatches[-1]]
        durable=[v for v in self.corrections.events() if dispatches and v.get('failed_request')==dispatches[-1] and 'feedback' in v and 'request_source' in v]
        if records or durable:
            self.retained_format_feedback(records[-1].get('source') if records else durable[-1]['source'],self.native('request',None))
            return {'ready':True,'kind':'settled_format_feedback','request':dispatches[-1]}
        return super().recovery_ready()
    def record_native_rejection(self,error,ident,association,raw_value,source):
        # Only typed consistency or zero-match content on the authenticated unchanged
        # request becomes fresh-choice feedback. Actual source drift remains terminal.
        if not self.admission.get('feedback_cycle') or not isinstance(error,e.StageValidationError) or error.code not in ('no_insight_meaning_drift','stale_edit_span'):return False
        current=self.native('request',None)
        if source!={k:current[k] for k in ('state_root','source_snapshot','scope')}:raise ValueError('stale native rejection')
        done=self.read()[2];candidate=e.sha(raw_value)
        prior=[v for k,v in done.items() if k.startswith('native_rejection_')]
        if any(v['candidate_sha256']==candidate and v['source']==source for v in prior):raise self.known_stop(ValueError('identical invalid candidate under unchanged source; no resampling'))
        inherited=self.admission['origin'].get('prior_validation_feedback')
        if inherited and inherited['candidate_sha256']==candidate and inherited['source_snapshot']==source['source_snapshot'] and inherited['scope']==source['scope']:raise self.known_stop(ValueError('identical inherited invalid candidate; no resampling'))
        self.require_rejection_room()
        record={'request_id':ident,'association':association,'source':source,'candidate_sha256':candidate,'candidate':raw_value,'code':error.code,'attribution':'native deterministic semantic consistency validator','details':getattr(error,'details',[]),'constraint':('The old_text matched zero locations in the current allowed file. The supplied current source and these source identities are authoritative. Make a fresh choice: inspect, propose an exact current-source edit, retain, or abstain. No patch was applied; no obligation to repeat the same change.' if error.code=='stale_edit_span' else 'When proposed_development is absent or null, relationship, aspiration and discriminator must remain verbatim from the referenced prior proposal. Changed meaning requires explicit justified development; independent review still applies.'),'admitted':False,'executed':False,'grants_authority':False}
        self.op('native_rejection_'+ident,lambda:record)
        return True
    def proposal(self,view):
        while True:
            request,context=self.project(view)
            references=load('browser_proposal_references','engine-proposal-references.py')
            wire_context=self.wire_context(request,context)
            self.op('proposal_native_'+e.sha(request),lambda:copy.deepcopy(request))
            try:
                result=self.checked('proposal',self.proposal_body(view,wire_context),lambda v:self.decode_proposal(references.expand(v,wire_context['proposal_references'],request),request,context,view))
            except base.SemanticChoiceRejected:
                continue
            if 'inspect' not in result['response']:return result
            count=len([k for k in self.read()[2] if k.startswith('source_view_')])
            self.op('source_view_'+str(count),lambda:{'state_root':request['state_root'],'inspect':result['response']['inspect']})
    def review(self,view,response):
        generated=[v for k,v in self.read()[2].items() if k.startswith('generated_') and v['response']==response]
        if not generated:raise ValueError('unowned review candidate')
        identity=semantic_candidate(response,generated[-1]['return_basis']);anchor=self.evidence_anchor()
        if any(k.startswith('review_rejection_') and v.get('candidate_sha256')==identity and v.get('evidence_anchor')==anchor for k,v in self.read()[2].items()):
            raise self.known_stop(ValueError('identical rejected candidate under unchanged evidence; no second reviewer request'))
        self.context_budget=max(2048,self.admission['payload_bytes']-65536-len(base.capacity.serialized(response)))
        try:return super().review(view,response)
        finally:self.context_budget=self.admission['payload_bytes']-65536
    def decode_proposal(self,value,request,context,view):
        if isinstance(value,dict) and set(value)=={'deliberation'}:
            helper=load('deliberation_decode','engine-browser-deliberation.py');parts=helper.shape(value);responses={};bases={}
            for name in ('development','artifact'):
                item=parts[name]
                if item is None:responses[name]=None;bases[name]={};continue
                responses[name],bases[name]=self.decode_proposal(item if set(item)=={'retain'} else {'alternatives':[item]},request,context,view)
                if set(responses[name])!={'retain'}:
                    allowed=helper.DEVELOPMENT if name=='development' else helper.ARTIFACT
                    if len(responses[name]['proposals'])!=1 or responses[name]['proposals'][0]['kind'] not in allowed:raise ValueError('invalid '+name+' component')
                elif name!='artifact':raise ValueError('development must be advisory or null')
            return {'deliberation':{**responses,'artifact_dependency':parts['artifact_dependency'],'guard':helper.guard(request)}},bases
        if isinstance(value,dict) and set(value)=={'retain'}:
            # Already expanded: a stated development decision is folded into this reason.
            self.check_retention_reason(value)
            if view['branch'] not in ('propose','reconsider'):raise ValueError('retention only during deliberation')
            return value,{}
        if isinstance(value,dict) and set(value)=={'inspect'}:
            if view['branch']!='propose':raise ValueError('inspection only during ordinary proposal')
            item=value['inspect'];files={f['path']:f['source'] for f in request['coding']['files'] if f['editable']}
            if not isinstance(item,dict) or set(item)!={'path','offset'} or item['path'] not in files or type(item['offset']) is not int:raise ValueError('unadmitted inspection')
            raw=files[item['path']].encode();offset=item['offset']
            if not 0<=offset<len(raw):raise ValueError('inspection offset outside file')
            raw[:offset].decode('utf-8')
            if context['source_access']['current_view']==item:raise ValueError('repeated identical source view; stop without loop')
            return value,{}

        if isinstance(value,dict) and set(value)=={'abstain'}:
            return web.bind_edit(request,value,lambda _:None),{}
        if not isinstance(value,dict) or set(value)!={'alternatives'} or not isinstance(value['alternatives'],list) or not 1<=len(value['alternatives'])<=4:raise ValueError('alternative envelope')
        response=None;basis={}
        for index,item in enumerate(value['alternatives']):
            try:part=web.bind_edit(request,item,lambda _:None)
            except ValueError as error:
                if getattr(error,'code',None)=='stale_edit_span' and (len(value['alternatives'])!=1 or set(item)-{'expectation'}!={'edits','rationale'}):raise ValueError('mixed candidate stale span remains terminal') from error
                if getattr(error,'diagnostic',None) and 'representation_size' in error.diagnostic:
                    if len(value['alternatives'])!=1 or set(item)-{'expectation'}!={'edits','rationale'}:raise ValueError('mixed/alternative size failure is not eligible for regeneration') from error
                    detail=json.loads(error.diagnostic.split('TSR_SHAPE_V1:',1)[1])
                    if detail['path']=='$':raise ValueError('serialized native response exceeds boundary; no representation retry') from error
                    helper=load('browser_size_diagnostic','engine-project-output.py')
                    helper.fail('$.alternatives[0]'+detail['path'][1:],detail['code'],max_utf8_bytes=detail['max_utf8_bytes'],actual_utf8_bytes=detail['actual_utf8_bytes'])
                raise
            if 'abstain' in part:raise ValueError('abstention cannot mix alternatives')
            if 'return_basis' in item:basis[str(index)]=e.return_evidence.validate(item['return_basis'],request,narrative_bytes=web.narrative_capacity(request))
            if response is None:response={k:v for k,v in part.items() if k not in ('proposals','pursuit_expectations')};response['proposals']=[]
            offset=len(response['proposals'])
            for key,expectation in part.get('pursuit_expectations',{}).items():
                response.setdefault('pursuit_expectations',{})[str(offset+int(key))]=expectation
            response['proposals'].extend(part['proposals'])
        if view['branch']=='reconsider' and (len(response['proposals'])!=1 or response['proposals'][0]['kind'] not in ('relational','public_desire','provisional_preference','contrast_preference','success_preference')):raise ValueError('reconsideration advisory form required')
        if any(p['kind'] in ('public_desire','contrast_preference','success_preference','provisional_preference') for p in response['proposals']) and len(response['proposals'])!=1:raise ValueError('public meaning requires one advisory candidate')
        self.native('preview',{'response':response})
        return response,basis
