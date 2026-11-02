"""Offline fake transport with real campaign ledger/native metering. No target commands."""
import copy
import importlib.util
import json
from pathlib import Path
import unittest
import test_engine_project_cycle as cycle_tests

def load(name,file):
    spec=importlib.util.spec_from_file_location(name,Path(__file__).with_name(file));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
m=load('adapter_test','engine-project-cycle-adapter.py');campaign=load('adapter_campaign','engine-b91-campaign.py')

class CycleBranches(cycle_tests.Cycle):
    def test_resume_timeout_from_native_plan(self):
        self.assertEqual(m.Adapter.resume_timeout({'plan':{'timeout_ms':20000,'commands':[[],[],[]]}}),90)
        with self.assertRaises(ValueError):m.Adapter.resume_timeout({'plan':{'timeout_ms':80000,'commands':[[],[],[]]}})
    def test_honest_abstention_no_review_or_action(self):
        self.proposal=lambda _:self.meter('response',{'abstain':'No useful next action under current evidence'})
        result=self.runner().run();self.assertEqual(result['status'],'proposal_abstention');self.assertEqual(self.reviews,[]);self.assertEqual(self.actions,[])
    def test_qualitative_continuation_not_execution(self):
        original=self.native
        def native(kind,value):
            if kind=='advance' and self.counter==2:
                self.guidance='developed';return {'qualitative_admitted':True,'review_required':False,'next':self.request()}
            return original(kind,value)
        self.native=native;result=self.runner().run()
        self.assertEqual(self.actions,['developed']);self.assertEqual(len(self.calls),2);self.assertEqual(result['status'],'decision_budget')
# Only exercise the new branches here, not repeat inherited whole suite.
for name in list(cycle_tests.Cycle.__dict__):
    if name.startswith('test_'):setattr(CycleBranches,name,None)

class NativeAccounting(unittest.TestCase):
    def setUp(self):
        import test_engine_b91_driver as fixture
        self.base=fixture.NativeSequence(methodName='test_full_native_sequence');self.base.setUp();self.addCleanup(self.base.doCleanups)
        self.driver=self.base.driver();self.c=self.driver.initialize();self.root=self.base.root/'adapter'
        self.origin=self.driver.journal().origin
        self.manifest=campaign.manifest('a'*64,self.base.root/'campaign.jsonl',fixture=True)
        self.ledger=campaign.Ledger(self.base.root/'campaign.jsonl',self.manifest,campaign.digest(self.manifest),create=True)
        self.a=m.admission(self.root,self.origin,self.manifest,self.driver.admission['expires_at'])
        self.calls=[]
    def receipt(self):return {'status':'settled','usd':'0.0001','usage':{'prompt_tokens':100,'completion_tokens':100,'total_tokens':200},'elapsed_seconds':.1}
    def transport(self,body,timeout):
        data=json.loads(body['messages'][1]['content']);self.calls.append(data)
        if 'criteria' in data:
            b=data['binding'];value={'review_id':b['review_id'],'verdict':'accept','findings':{k:{'finding':'met','basis':'static_source','evidence':[b['evidence_ids'][0]],'explanation':'Authored fixture judgment','limitations':'Not semantic competence evidence'} for k in data['criteria']}}
        else:value={'alternatives':[{'edits':[{'path':'src/lib.rs','replacement':self.origin['initial_source']+'\n// fixture alternative\n'}],'rationale':'Authored mechanics alternative'}]}
        return {'value':value,'receipt':self.receipt()}
    def test_native_proposal_review_accounting_and_private_exclusion(self):
        a=m.Adapter(self.root,self.a,self.c,self.ledger,self.transport);view=a.native('request',None)
        p=a.proposal(view);fresh=a.native('request',None);preview=a.native('preview',{'response':p['response']})
        r=a.review({**fresh,'response_digest':preview['response_digest']},p['response'])
        self.assertEqual(r['judgment']['decision'],'approve');self.assertEqual(self.ledger.status()['calls'],2)
        self.assertEqual(self.c.budget()['model_calls'],2);self.assertEqual(self.c.budget()['unsettled_requests'],0)
        exported=json.dumps(self.calls)
        self.assertNotIn(self.c.cli('project','config')['protected']['tests/contract.rs'],exported)
        self.assertNotIn('private_operator',exported)
        before=(self.root/'adapter.jsonl').read_bytes();again=m.Adapter(self.root,self.a,self.c,self.ledger,self.transport)
        self.assertEqual(again.generated(),[p['response']]);self.assertEqual(before,(self.root/'adapter.jsonl').read_bytes())
        forged=copy.deepcopy(view);forged['source_snapshot']='f'*64
        with self.assertRaisesRegex(ValueError,'drift'):again.proposal(forged)
        self.assertEqual(self.ledger.status()['calls'],2)
    def test_full_native_cycle_accounted_execution_restart(self):
        import test_engine_b91_driver as fixture
        original=self.transport;generated=[];executions=[]
        def transport(body,timeout):
            data=json.loads(body['messages'][1]['content'])
            result=original(body,timeout)
            if 'criteria' not in data:
                replacement=m.e.prepare.FAILED if not generated else fixture.PATCH
                result['value']={'alternatives':[{'edits':[{'path':'src/lib.rs','replacement':replacement}],'rationale':'Authored source choice after typed return'}]};generated.append(replacement)
            return result
        def worker(view):
            result=self.base.worker(self.c,'seed' if not executions else 'patch');executions.append(result);return result
        a=m.Adapter(self.root,self.a,self.c,self.ledger,transport,worker=worker)
        def native(kind,value):
            result=a.native(kind,value)
            if kind=='advance' and not executions:
                p=self.c.cli('project','status')['pending']
                self.c.json_cli(('project','evaluator-forecast'),{'state_root':self.c.request()['state_root'],'snapshot':self.c.cli('project','acceptance','status')['snapshot'],'commitment':p['commitment'],'action':p['action'],'probability':.1,'evaluator':'B9.1 frozen evaluator-authored seed'})
                result['next']=a.native('request',None)
            return result
        root=(self.base.root/'open-loop').resolve();view=a.native('request',None)
        admission={'version':'local-project-cycle@v1','journal':str(root),'scope':view['scope'],'max_decisions':2,'max_executions':2,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__))}
        def pause(event,name):
            if event=='after_result' and name=='resume_0':raise RuntimeError('checkpoint')
        with self.assertRaisesRegex(RuntimeError,'checkpoint'):m.loop.Cycle(root,admission,native,a.proposal,a.review,hook=pause).run()
        result=m.loop.Cycle(root,admission,native,a.proposal,a.review).run()
        self.assertEqual(result['status'],'review_required');self.assertEqual(len(executions),2)
        self.assertEqual(self.ledger.status()['calls'],4);self.assertEqual(self.ledger.status()['execution_groups'],2)
        before=self.ledger.path.read_bytes() if hasattr(self.ledger,'path') else (self.base.root/'campaign.jsonl').read_bytes()
        self.assertEqual(m.loop.Cycle(root,admission,native,a.proposal,a.review).run(),result)
        self.assertEqual(before,(self.base.root/'campaign.jsonl').read_bytes())
        self.assertEqual(self.c.budget()['model_calls'],4)

    def test_native_qualitative_admission_then_honest_stop(self):
        original=self.transport;proposed=[]
        def transport(body,timeout):
            data=json.loads(body['messages'][1]['content']);result=original(body,timeout)
            if 'criteria' not in data:
                if proposed:
                    self.assertEqual(len(data['relational_history']),1)
                    result['value']={'abstain':'Fixture: no additional useful permitted choice'}
                else:
                    result['value']={'alternatives':[{'relational':{'parents':[p['id'] for p in data['parents']],'relationship':'Authored fixture relationship','aspiration':'Preserve both meanings while investigating behavior','discriminator':'Actual finite observations remain distinct from inference','revises':None,'catalysts':[]}}]}
                proposed.append(data)
            return result
        a=m.Adapter(self.root,self.a,self.c,self.ledger,transport)
        root=(self.base.root/'qualitative-loop').resolve();initial=a.native('request',None)
        ca={'version':'local-project-cycle@v1','journal':str(root),'scope':initial['scope'],'max_decisions':2,'max_executions':1,'expires_at':self.a['expires_at'],'implementation_sha256':m.d.file_sha(Path(m.loop.__file__))}
        result=m.loop.Cycle(root,ca,a.native,a.proposal,a.review).run()
        self.assertEqual(result['status'],'proposal_abstention');self.assertEqual(self.ledger.status()['calls'],3)
        self.assertEqual(self.ledger.status()['execution_groups'],0);self.assertEqual(self.c.request()['coding']['relational']['inputs']['parents'],self.origin['parents'])
        before=(self.base.root/'campaign.jsonl').read_bytes();self.assertEqual(m.loop.Cycle(root,ca,a.native,a.proposal,a.review).run(),result)
        self.assertEqual(before,(self.base.root/'campaign.jsonl').read_bytes())

    def test_unknown_charge_and_pending_no_retry(self):
        def unknown(*args):return {'value':{'abstain':'fixture'},'receipt':{**self.receipt(),'usd':None}}
        a=m.Adapter(self.root,self.a,self.c,self.ledger,unknown)
        with self.assertRaises(m.d.Blocked):a.proposal(a.native('request',None))
        self.assertTrue(self.ledger.status()['stopped']);self.assertEqual(self.ledger.status()['calls'],1)
        with self.assertRaises(Exception):a.proposal(a.native('request',None))
        self.assertEqual(self.ledger.status()['calls'],1)
    def test_extra_file_blocks_before_transport(self):
        a=m.Adapter(self.root,self.a,self.c,self.ledger,self.transport)
        (self.c.directory/'coding-work/private.txt').write_text('private_operator_secret')
        with self.assertRaises(ValueError):a.proposal(a.native('request',None))
        self.assertEqual(self.calls,[])
