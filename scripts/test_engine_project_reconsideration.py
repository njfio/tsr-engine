"""Explicit local-only advisory fixtures; no live inference or target commands."""
import copy
import unittest
from pathlib import Path
import test_engine_project_cycle as base
import test_engine_project_cycle_adapter as adapter_tests
m=base.m;a=adapter_tests.m

class PublicProjection(unittest.TestCase):
    def context(self):return {'version':'project-relational@v2','semantic_return':{'review':'a'*64,'applicable':True,'unmet':['one'],'unknown':['two'],'meaning':'PRIVATE prose must never be copied'}}
    def test_attributed_negative_unknown_and_no_private_prose(self):
        original=self.context();parents=[{'requirement':'one'},{'requirement':'two'}]
        result=a.semantic_projection(original,parents)
        self.assertEqual(result['unmet'],['one']);self.assertEqual(result['unknown'],['two'])
        self.assertNotIn('PRIVATE',str(result));self.assertEqual(a.semantic_projection(copy.deepcopy(original),parents),result)
        changed=copy.deepcopy(original);changed['semantic_return']['unmet']=[]
        self.assertNotEqual(a.semantic_projection(changed,parents),result)
        changed['semantic_return']['applicable']=False
        self.assertFalse(a.semantic_projection(changed,parents)['applicable'])
    def test_foreign_overlap_shape_identity_rejected(self):
        for key,value in [('unmet',['foreign']),('unknown',['one']),('review','invalid'),('applicable',1)]:
            source=self.context();source['semantic_return'][key]=value
            with self.subTest(key=key),self.assertRaises(ValueError):a.semantic_projection(source,[{'requirement':'one'},{'requirement':'two'}])
        source=self.context();source['semantic_return']['private_count']=3
        with self.assertRaises(ValueError):a.semantic_projection(source,[{'requirement':'one'},{'requirement':'two'}])

class Advisory(base.Cycle):
    def setUp(self):
        super().setUp();self.branch='reconsider';self.evidence={'evidence_key':'e'*64,'lineage':'b'*64,'binding':'c'*64,'snapshot':'a','requirements_revision':'d'*64,'receipt_ids':['f'*64],'semantic_review':None};self.acks=[]
        self.a.update(version='local-project-cycle@v2',advisory_profile='local-fixture-only@v1',max_executions=0)
    def request(self):return {**super().request(),'reconsideration':copy.deepcopy(self.evidence)}
    def proposal(self,state):
        self.calls.append(copy.deepcopy(state));self.counter+=1
        return self.meter('response',{'state_root':state['state_root'],'proposals':[{'kind':'relational','proposal':{'revises':state['reconsideration']['lineage'],'development':{'proposed_development':None}}}]})
    def native(self,kind,value):
        if kind=='advance':
            self.acks.append(('revision',value));self.branch='resume'
            return {'qualitative_admitted':True,'review_required':False,'next':self.request()}
        if kind=='acknowledge':
            self.acks.append((value['outcome'],value));self.branch='propose';return {'executed':False,'next':self.request()}
        if kind=='resume':self.fail('advisory must not execute')
        return super().native(kind,value)
    def test_revision_stops_before_followon_and_replay(self):
        result=self.runner().run();self.assertEqual(result['status'],'advisory_reconsidered');self.assertFalse(result['acceptance_granted'])
        self.assertEqual(len(self.calls),1);self.assertEqual(len(self.reviews),1);self.assertEqual(len(self.acks),1);self.assertEqual(self.actions,[])
        before=(self.root/'cycle.jsonl').read_bytes();self.assertEqual(self.runner().run(),result);self.assertEqual(before,(self.root/'cycle.jsonl').read_bytes())
    def test_no_insight_abstain_and_unknown_are_not_negative(self):
        for decision in ('approve','unknown','reject'):
            with self.subTest(decision=decision):
                self.root=self.root/decision;self.a['journal']=str(self.root);self.branch='reconsider';self.acks=[]
                self.proposal=lambda state:self.meter('response',{'abstain':'No justified new insight'})
                def review(state,response):return self.meter('judgment',{'decision':decision,'response_digest':state['response_digest'],'reviewer':'fixture review','generator':'fixture generation'})
                self.review=review;result=self.runner().run()
                expected={'approve':'abstain','unknown':'unknown','reject':'rejected'}[decision]
                self.assertEqual(result['status'],'advisory_'+expected);self.assertEqual(self.acks[0][0],expected);self.assertEqual(self.actions,[])
    def test_stale_or_withdrawn_lineage_no_apply(self):
        original=self.review
        def changed(state,response):
            value=original(state,response);self.evidence['lineage']='0'*64;return value
        self.review=changed
        with self.assertRaisesRegex(m.d.Blocked,'lineage drift'):self.runner().run()
        self.assertEqual(self.acks,[])
    def test_nonrelational_no_execution(self):
        self.proposal=lambda state:self.meter('response',{'state_root':state['state_root'],'proposals':[{'kind':'edit_set'}]})
        with self.assertRaisesRegex(m.d.Blocked,'revision only'):self.runner().run()
        self.assertEqual(self.acks,[]);self.assertEqual(self.reviews,[])
    def test_legacy_and_no_aspiration_stop_without_callbacks(self):
        self.a.pop('advisory_profile');self.a['version']='local-project-cycle@v1'
        result=self.runner().run();self.assertEqual(result['status'],'review_required');self.assertEqual(self.calls,[])
        self.root=self.root/'no-aspiration';self.a['journal']=str(self.root);self.branch='review_required'
        self.assertEqual(self.runner().run()['status'],'review_required');self.assertEqual(self.calls,[])
    def test_unknown_dispatch_checkpoint_no_repeat(self):
        def pause(event,name):
            if event=='after_effect' and name=='advisory_proposal':raise RuntimeError('interrupted')
        with self.assertRaises(RuntimeError):self.runner(hook=pause).run()
        with self.assertRaisesRegex(m.d.Blocked,'uncertain'):self.runner().run()
        self.assertEqual(len(self.calls),1);self.assertEqual(self.acks,[])
for name in list(base.Cycle.__dict__):
    if name.startswith('test_'):setattr(Advisory,name,None)

class NativeAdvisory(unittest.TestCase):
    def test_actual_native_return_semantic_review_revision_and_restart(self):
        import json
        import test_engine_b91_driver as fixture
        import test_engine_project_cycle_adapter as helper
        f=fixture.NativeSequence(methodName='test_full_native_sequence');f.setUp();self.addCleanup(f.doCleanups)
        f.a=m.d.admission(f.root,fixture=True,model='gpt-6-luna',final_payload_32k=True,schema_corrections=True,review_schema_corrections=True)
        driver=f.driver();controller=driver.initialize();origin=driver.journal().origin;initial_config=controller.cli('project','config')
        campaign=helper.campaign;manifest=campaign.manifest('a'*64,f.root/'local-ledger.jsonl',fixture=True)
        ledger=campaign.Ledger(f.root/'local-ledger.jsonl',manifest,campaign.digest(manifest),create=True)
        def receipt():return {'status':'settled','usd':'0.0001','usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200},'elapsed_seconds':.01}
        counts=[];oldresponses=[];projected=[]
        def transport(body,timeout):
            data=json.loads(body['messages'][1]['content']);counts.append(data)
            if 'criteria' in data:
                binding=data['binding'];value={'review_id':binding['review_id'],'verdict':'accept','findings':{key:{'finding':'met','basis':'static_source','evidence':[binding['evidence_ids'][0]],'explanation':'Authored local fixture reviewer','limitations':'No real semantic competence claim'} for key in data['criteria']}}
            elif not oldresponses:
                value={'alternatives':[{'relational':{'parents':[p['id'] for p in data['parents']],'relationship':'Authored hypothesis linking boundary behavior and preserved functionality','aspiration':'Keep compatible behavior legible through evidence','discriminator':'Finite behavior may challenge this hypothesis','revises':None,'catalysts':[]}}]}
            elif len(oldresponses)==1:
                value={'alternatives':[{'edits':[{'path':'src/lib.rs','replacement':fixture.PATCH}],'rationale':'Authored fixture repair for controlled return'}]}
            else:
                projected.append(data);prior=data['relational_history'][-1];p=prior['proposal'];lineage=data['relational_contract']['allowed_lineage'][0]
                value={'alternatives':[{'relational':{**p,'revises':prior['admission'],'development':{'original_revision':lineage['original_revision'],'prior_revision':lineage['prior_revision'],'retained_meaning':p['aspiration'],'return_reveals':'Finite tests passed; a separate attributed semantic judgment remains negative on one dimension.','unresolved':'That judgment does not become tool truth or permission.','proposed_development':None}},'return_basis':{'binding':data['return_evidence']['binding'],'observed':[x['id'] for x in data['return_evidence']['cases']][:8],'inferred':[],'unknown':['Assertion-level behavior and semantic fulfillment remain unknown']}}]}
            return {'value':value,'receipt':receipt()}
        initial_root=(f.root/'initial-adapter').resolve();aa=a.admission(initial_root,origin,manifest,f.a['expires_at'])
        first=a.Adapter(initial_root,aa,controller,ledger,transport)
        for _ in range(2):
            view=first.native('request',None);response=first.proposal(view)['response'];oldresponses.append(response)
            preview=first.native('preview',{'response':response});review=first.review({**preview['cycle'],'response_digest':preview['response_digest']},response)['judgment']
            fresh=first.native('request',None);first.native('advance',{'binding':fresh['binding'],'response':response,'review':{**review,'binding':fresh['binding'],'state_root':fresh['state_root'],'source_snapshot':fresh['source_snapshot']}})
        result=f.worker(controller,'patch');self.assertEqual(result['target_commands_executed'],0);self.assertEqual(result['execution']['verdict'],'passed')
        before=controller.cli('project','cycle','request');self.assertEqual(before['branch'],'reconsider')
        status=controller.cli('project','acceptance','status')
        requirements=list(origin['parents']);review_value={'snapshot':status['snapshot'],'requirements_revision':status['requirements_revision'],'test_receipt':status['test_receipt'],'decision':'reject','evidence':{p['requirement']:{'met':index==0,'evidence':'PRIVATE independent reviewer prose must remain local'} for index,p in enumerate(requirements)}}
        controller.json_cli(('project','acceptance','review'),review_value)
        current=controller.cli('project','cycle','request');self.assertEqual(current['branch'],'reconsider')
        root=(f.root/'advisory-adapter').resolve();ad=a.admission(root,origin,manifest,f.a['expires_at'],max_decisions=1,max_executions=0,local_advisory=True)
        with self.assertRaisesRegex(ValueError,'no live transport'):a.Adapter(root,ad,controller,ledger)
        self.assertFalse(root.exists())
        with self.assertRaisesRegex(ValueError,'zero execution'):a.admission(root,origin,manifest,f.a['expires_at'],max_executions=1,local_advisory=True)
        advisory=a.Adapter(root,ad,controller,ledger,transport)
        # Explicit fixture-only import of exact earlier authored, natively reviewed
        # response lineage. Not a live migration/admission or generated-model claim.
        for index,response in enumerate(oldresponses):advisory.op('generated_fixture_'+str(index),lambda response=response:{'response':response,'return_basis':{},'fixture_only':True,'source_adapter':a.e.sha(aa)})
        cycle_root=(f.root/'advisory-loop').resolve();ca={'version':'local-project-cycle@v2','advisory_profile':'local-fixture-only@v1','journal':str(cycle_root),'scope':origin['scope'],'max_decisions':1,'max_executions':0,'expires_at':f.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.__file__))}
        stale=copy.deepcopy(current);stale['source_snapshot']='0'*64
        before_calls=ledger.status()['calls']
        with self.assertRaises(ValueError):advisory.project(stale)
        self.assertEqual(ledger.status()['calls'],before_calls)
        output=m.Cycle(cycle_root,ca,advisory.native,advisory.proposal,advisory.review).run()
        self.assertEqual(output['status'],'advisory_reconsidered');self.assertEqual(ledger.status()['calls'],6);self.assertEqual(ledger.status()['execution_groups'],0)
        self.assertNotIn('PRIVATE',json.dumps(projected));self.assertEqual(projected[0]['semantic_return']['unmet'],[requirements[1]['requirement']]);self.assertEqual(projected[0]['semantic_return']['unknown'],[])
        context=controller.request()['coding']['relational'];self.assertEqual(len(context['history']),2);self.assertEqual(context['inputs']['parents'],origin['parents'])
        self.assertEqual(len(context['guidance']),1);self.assertEqual(context['guidance'][0]['admission'],context['history'][-1]['admission'])
        self.assertEqual(context['guidance'][0]['proposal'],context['history'][-1]['proposal'])
        self.assertEqual(controller.cli('project','config'),initial_config)
        self.assertFalse(controller.cli('project','acceptance','status')['accepted'])
        self.assertNotEqual(context['history'][-1]['admission'],projected[0]['relational_history'][-1]['admission'])
        prior=(f.root/'local-ledger.jsonl').read_bytes();self.assertEqual(m.Cycle(cycle_root,ca,advisory.native,advisory.proposal,advisory.review).run(),output)
        self.assertEqual(prior,(f.root/'local-ledger.jsonl').read_bytes())
        # Original unmet aim remains legitimately eligible. The advisory wrapper
        # stops despite that eligibility; it must not close or execute this aim.
        next_choice=controller.cli('project','cycle','request')
        self.assertEqual(next_choice['branch'],'propose');self.assertFalse(next_choice['dispatch_authorized'])
        self.assertFalse(controller.request().get('pending'));self.assertEqual(ledger.status()['execution_groups'],0)
