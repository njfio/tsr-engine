"""No live provider/target execution. Authored protocol fixtures, not a B9.1 pass."""
import copy
import importlib.util
import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

spec=importlib.util.spec_from_file_location('b91_envelope',Path(__file__).with_name('engine-b91-project-envelope.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

class ProjectEnvelope(unittest.TestCase):
    def setUp(self):
        self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup);self.root=Path(self.tmp.name)
        frozen=self.root/'frozen';m.prepare.prepare(frozen)
        engine=self.root/'engine';engine.mkdir()
        self.controller=m.Controller(engine)
        self.controller.cli('project','init','--spec',str(frozen/'scope.json'),'--source',str(frozen/'source'),'--requirements',str(frozen/'requirements.json'))
        status=self.controller.cli('project','acceptance','status');feedback=self.root/'feedback.json'
        feedback.write_text(json.dumps({'requirements_revision':status['requirements_revision'],'decision_record':None,'guidance':m.prepare.REQUIREMENTS}))
        self.controller.cli('project','acceptance','feedback','--file',str(feedback))
        self.origin=self.root/'origin.json';self.binding=m.freeze_origin(frozen,self.controller,self.origin)
        self.path=self.root/'journal.jsonl';self.journal=m.Journal(self.path,self.origin,self.binding)
    def reload(self): return m.Journal(self.path,self.origin,self.binding)
    def relation(self,request):
        return {'relational':{'parents':[p['id'] for p in request['coding']['relational']['inputs']['parents']],
            'relationship':'Evaluator-authored protocol fixture relationship','aspiration':'Protocol fixture meaning',
            'discriminator':'Finite observations only','revises':None,'catalysts':[]}}
    def fake(self):
        outer=self
        class Fake:
            directory=outer.controller.directory
            def __init__(self):
                self.current=outer.controller.request();self.stores=0
                self.current['current_commitment']='fixture-seed-action'
                forecast={'raw':.1,'calibrated':None,'source':{'external':'evaluator-authored:fixture'}}
                self.forecast_status={'version':'project-evaluator-forecast@v1','records':[
                    {'record':'3'*64,'admission':{'probability':.1,'commitment':'b'*64,'snapshot':outer.journal.origin['initial_snapshot'],
                     'action':'fixture-seed-action','evaluator':'fixture'},'forecast':forecast},
                    {'record':'4'*64,'admission_record':'3'*64,'receipt':'a'*64,'commitment':'b'*64,'observed_success':False,
                     'forecast':forecast,'residual':.1}]}
                self.current['coding']['files'][2]['source']=m.prepare.FAILED
                self.current['coding']['relational']['inputs']['returns']=[{'receipt':'a'*64,'commitment':'b'*64,
                    'verdict':'failed_tests','observations':[{'case_id':'empty_input_handled','passed':True,'stdout':'PRIVATE ASSERTION'},
                    {'case_id':'existing_behaviour_unchanged','passed':False,'stderr':'PRIVATE STACK'}]}]
            def cli(self,*args):
                if args==('project','config'):return m.expected_configuration()
                if args==('project','evaluator-forecast-status'):return copy.deepcopy(self.forecast_status)
                raise AssertionError(args)
            def request(self): return copy.deepcopy(self.current)
            def validate(self,response): return {'assessments':[{'disposition':'deferred' if response['proposals'][0]['kind'] in ('relational','experiment') else 'eligible'}]}
            def store_review_only(self,response):
                self.stores+=1
                return {'status':'review_required','executed':False,'proposal_record':m.sha(response)}
        return Fake()
    def reply(self,value): return {'value':value,'receipt':{'response_status':'received','structural_status':'typed_json',
        'status':'settled','usd':'0.001','usage':{'prompt_tokens':100,'completion_tokens':200,'total_tokens':300},'request_id':'request-1'}}
    def test_real_controller_projection_binding_and_deferred_store(self):
        request,body=m.project(self.journal,self.controller,m.STAGES[0])
        payload=json.dumps(body)
        self.assertNotIn(m.prepare.TESTS,payload);self.assertNotIn(m.prepare.FAILED,payload)
        self.assertNotIn('review-only',payload)
        response=m.local.bind_proposal(request,json.dumps(self.relation(request)))
        checked=self.controller.validate(response);self.assertEqual(checked['assessments'][0]['disposition'],'deferred')
        result=self.controller.store_review_only(response);self.assertFalse(result['executed'])
        self.assertEqual(result['status'],'review_required')
        self.assertEqual(self.controller.cli('project','status')['completed'],0)
        # Never call the transport before an actual bound seed return.
        with self.assertRaisesRegex(ValueError,'return required'):
            m.fixture_step(self.journal,self.controller,lambda *_:self.fail('transport called'))
        self.assertEqual(self.reload().status()['calls_consumed_or_reserved'],0)
    def test_projection_drops_unrelated_private_fields_and_logs(self):
        fake=self.fake();fake.current['private_review']='SECRET';fake.current['goal']='PRIVATE GOAL'
        fake.current['prior_outcomes']=[{'evidence':'PRIVATE HISTORY'}]
        fake.current['coding']['recent_observations']=[{'stderr':'PRIVATE STDERR'}]
        _,body=m.project(self.journal,fake,m.STAGES[0]);text=json.dumps(body)
        for token in ('SECRET','PRIVATE','assert_eq!','assert!'):
            self.assertNotIn(token,text)
        context=json.loads(body['messages'][1]['content'])
        self.assertNotIn('cases',context['observed_returns'][0])
        case=next(x for x in context['return_evidence']['cases'] if x['case_id']=='existing_behaviour_unchanged')
        self.assertEqual(case['outcome'],'unknown')  # This fixture has no command exit evidence.
        self.assertIn(m.prepare.FAILED,body['messages'][1]['content'].replace('\\n','\n').replace('\\"','"'))

    def test_unadmitted_semantic_return_stops_before_transport(self):
        for version,include_return in [('project-relational@v2',True),('project-relational@v1',True),('project-relational@v3',False)]:
            with self.subTest(version=version,include_return=include_return):
                fake=self.fake();inputs=fake.current['coding']['relational']['inputs']
                inputs['version']=version
                if include_return:inputs['semantic_return']={'review':'a'*64,'applicable':True,'unmet':['existing_behaviour'],'unknown':[]}
                with self.assertRaisesRegex(ValueError,'semantic-return projection'):
                    m.fixture_step(self.journal,fake,lambda *_:self.fail('transport called'))
                self.assertEqual(self.reload().status()['calls_consumed_or_reserved'],0)
                self.assertEqual(fake.stores,0)
    def test_private_source_changed_intent_protected_origin_rejected(self):
        changes=(lambda q:q['coding']['files'][2].update(source='PRIVATE SOURCE'),
                 lambda q:q['coding']['files'][0].update(source_digest='0'*64),
                 lambda q:q['coding']['relational']['inputs']['parents'][0].update(meaning='PRIVATE INTENT'),
                 lambda q:q['next_attention'].update(scope='0'*64),
                 lambda q:q['coding'].update(supporting_context=[{'source':'PRIVATE'}]))
        for change in changes:
            fake=self.fake();change(fake.current)
            with self.assertRaises(ValueError):m.project(self.journal,fake,m.STAGES[0])
    def test_forged_origin_admission_and_journal_corruption_rejected(self):
        with self.assertRaises(ValueError):m.Journal(self.path,self.origin,'0'*64)
        origin=json.loads(self.origin.read_text());origin['calls_authorized']=6
        self.origin.unlink();m.private_save(self.origin,origin)
        with self.assertRaisesRegex(ValueError,'not an admission'):m.Journal(self.path,self.origin,m.sha(origin))
        rows=self.path.read_text();self.path.write_text(rows.replace('origin','forged',1))
        with self.assertRaises(ValueError):self.journal.events()
    def test_unknown_dispatch_restart_does_not_retry_or_refund(self):
        fake=self.fake();calls=[]
        def timeout(*args): calls.append(args);raise TimeoutError()
        with self.assertRaises(TimeoutError):m.fixture_step(self.journal,fake,timeout)
        reopened=self.reload();status=reopened.status()
        self.assertEqual(status['calls_consumed_or_reserved'],1);self.assertEqual(len(status['unknown_or_failed']),1)
        with self.assertRaisesRegex(ValueError,'reconciliation'):m.fixture_step(reopened,fake,timeout)
        self.assertEqual(len(calls),1)
        output=subprocess.run([sys.executable,str(Path(m.__file__)),'inspect','--directory',str(fake.directory),
            '--origin',str(self.origin),'--origin-sha256',self.binding,'--journal',str(self.path)],capture_output=True,text=True,check=True)
        self.assertEqual(json.loads(output.stdout),status)
    def test_real_transport_receipt_metering_and_no_automatic_admission(self):
        fake=self.fake();result=m.fixture_step(self.journal,fake,lambda *_:self.reply(self.relation(fake.current)))
        self.assertEqual(result['status'],'review_required');self.assertEqual(fake.stores,1)
        outcome=next(e for e in self.reload().events() if e['kind']=='provider_outcome')
        self.assertEqual(outcome['usd'],'0.001');self.assertEqual(outcome['usage']['completion_tokens'],200)
        self.assertEqual(outcome['request_id'],'request-1')
        with self.assertRaisesRegex(ValueError,'independent admission'):
            m.fixture_step(self.reload(),fake,lambda *_:self.fail('no review'))
        self.assertEqual(self.reload().status()['calls_consumed_or_reserved'],1)
    def test_source_drift_during_provider_response_stops_without_apply(self):
        fake=self.fake()
        def change(*_):
            value=self.relation(fake.current);fake.current['state_root']='d'*64
            return self.reply(value)
        with self.assertRaisesRegex(ValueError,'state drift'):m.fixture_step(self.journal,fake,change)
        self.assertEqual(fake.stores,0)
        self.assertEqual(len(self.reload().status()['unknown_or_failed']),1)
    def test_completed_before_validation_and_apply_crashes_block(self):
        fake=self.fake();_,body=m.project(self.journal,fake,m.STAGES[0]);ident='e'*64
        self.journal.append({'kind':'dispatch_intent','id':ident,'stage':m.STAGES[0],'state_root':fake.current['state_root'],
             'body_sha256':m.sha(body),'reservation':{'input_tokens':16384,'completion_tokens':8192,'deadline_seconds':240}})
        self.journal.append({'kind':'provider_outcome','id':ident,'status':'completed','usage':None,'usd':None})
        self.assertEqual(self.reload().status()['unapplied_response'],[ident])
        with self.assertRaises(ValueError):m.fixture_step(self.reload(),fake,lambda *_:self.fail('retry'))
        response=m.local.bind_proposal(fake.current,json.dumps(self.relation(fake.current)))
        self.journal.append({'kind':'validated_proposal','id':ident,'state_root':fake.current['state_root'],'response':response})
        self.assertEqual(self.reload().status()['application_unknown'],[ident])
        with self.assertRaises(ValueError):m.fixture_step(self.reload(),fake,lambda *_:self.fail('retry'))
        self.journal.append({'kind':'review_only_store_intent','id':ident})
        with self.assertRaises(ValueError):m.fixture_step(self.reload(),fake,lambda *_:self.fail('retry'))
    def test_unknown_cannot_be_relabelled_completed(self):
        fake=self.fake()
        with self.assertRaises(TimeoutError):m.fixture_step(self.journal,fake,lambda *_:(_ for _ in ()).throw(TimeoutError()))
        ident=self.journal.status()['unknown_or_failed'][0]
        with self.assertRaisesRegex(ValueError,'repeated provider outcome'):
            self.journal.append({'kind':'provider_outcome','id':ident,'status':'completed','usage':{},'usd':'0'})
    def test_wrong_or_missing_supplemental_origin_never_forwarded(self):
        fake=self.fake();fake.current['coding']['relational']['inputs']['supplemental']=[{'generated_origin':None}]
        with self.assertRaisesRegex(ValueError,'no generated origin'):m.project(self.journal,fake,m.STAGES[2])
        fake.current['coding']['relational']['inputs']['supplemental']=[{'generated_origin':{
            'record':'a'*64,'proposal':'b'*64,'index':0,'binding':'c'*64,'bundle_digest':'d'*64}}]
        with self.assertRaisesRegex(ValueError,'not this trial'):m.project(self.journal,fake,m.STAGES[2])
    def test_authored_six_stage_fixture_restarts_and_cannot_skip_observed_returns(self):
        # Protocol ordering only: all returns below are authored controller fixtures,
        # never an actual simulator/project execution or semantic B9.1 pass.
        fake=self.fake();calls=[];inputs=fake.current['coding']['relational']['inputs']
        relation=fake.current['coding']['relational']
        def step(value):
            def transport(body,timeout): calls.append((body,timeout));return self.reply(value)
            result=m.fixture_step(self.reload(),fake,transport)
            self.assertFalse(result['executed']);return result
        def reviewed(result):
            p=result['response']['proposals'][0]['proposal']
            for h in relation['history']:h['current']=False;h['superseded']=True
            relation['history'].append({'admission':m.sha(result['response']),'binding':relation['binding'],
                'proposal':p,'withdrawn':False,'superseded':False,'current':True})
        def advance(label):
            fake.current['state_root']=m.sha(label);fake.current['next_attention']['state_root']=m.sha(label)
        first=step(self.relation(fake.current))
        with self.assertRaisesRegex(ValueError,'independent admission'):
            step({'experiment':{}})
        reviewed(first);advance('review1')
        experiment={'harness':{'tests/tsr_supplemental_boundary.rs':'#[test] fn boundary() { assert!(true); }'},
            'cases':[{'id':'boundary','args':[],'expected_stdout':''}],
            'mapping':{'existing_behaviour':{'cases':['boundary'],'rationale':'Authored protocol fixture',
                                            'limits':'No semantic claim; deterministic fixture only'}}}
        second=step({'experiment':experiment})
        with self.assertRaisesRegex(ValueError,'experiment return required'):step(self.relation(fake.current))
        exp=second['response']['proposals'][0]['experiment']
        observation={'admission':'a'*64,'receipt':'b'*64,'snapshot':inputs['snapshot'],
                     'checked_snapshot':'c'*64,'source_episode_root':'d'*64,'bundle_digest':'e'*64,
                     'mapped_cases':{'existing_behaviour':['boundary']},'cases':{'boundary':'unknown'},
                     'findings':{'existing_behaviour':'unknown'},'generated_origin':{
                         'record':'f'*64,'proposal':m.sha(second['response']),'index':0,
                         'binding':exp['binding'],'bundle_digest':'e'*64}}
        inputs['supplemental']=[observation];advance('observed experiment fixture')
        third=step(self.relation(fake.current));reviewed(third);advance('review2')
        fourth=step({'edits':[{'path':'src/lib.rs','replacement':'pub fn parse(_: &str) -> Result<Vec<i64>, String> { unimplemented!() }\n'}],
                     'rationale':'Authored nonwinning fixture solely for binding tests'})
        with self.assertRaisesRegex(ValueError,'has not produced'):step(self.relation(fake.current))
        patched=fourth['response']['proposals'][0]['patches'][0]['new_text']
        fake.current['coding']['files'][2]['source']=patched
        # Frozen old-source experimental return cannot be silently used on the patch.
        snapshot=m.sha('observed patch fixture');inputs['snapshot']=snapshot
        fake.current['coding']['snapshot']=snapshot;fake.current['next_attention']['source_snapshot']=snapshot
        with self.assertRaisesRegex(ValueError,'stale experiment source'):step(self.relation(fake.current))
        observation['snapshot']=snapshot
        with self.assertRaisesRegex(ValueError,'freshly source bound'):step(self.relation(fake.current))
        observation['generated_origin'].update(readmission_record='1'*64,previous_admission='a'*64,readmission_state_root='2'*64)
        advance('fresh reviewed re-verification fixture')
        fifth=step(self.relation(fake.current));reviewed(fifth);advance('review3')
        sixth=step(self.relation(fake.current))
        self.assertEqual(len(calls),6);self.assertEqual(fake.stores,4)  # last relation is proposal-only
        self.assertEqual(self.reload().status()['next_stage'],None)
        with self.assertRaisesRegex(ValueError,'terminal'):step(self.relation(fake.current))
        self.assertEqual(len(calls),6)
        self.assertFalse(self.reload().status()['b91_passed'])
        self.assertEqual(self.controller.cli('project','status')['completed'],0)

    def test_case_inventory_and_withdrawn_review_are_not_eligible(self):
        fake=self.fake();fake.current['coding']['cases'][0]['id']='PRIVATE CASE'
        with self.assertRaisesRegex(ValueError,'case inventory'):m.project(self.journal,fake,m.STAGES[0])
        fake=self.fake();result=m.fixture_step(self.journal,fake,lambda *_:self.reply(self.relation(fake.current)))
        proposal=result['response']['proposals'][0]['proposal']
        relation=fake.current['coding']['relational']
        relation['history']=[{'admission':'a'*64,'binding':relation['binding'],'proposal':proposal,
                              'withdrawn':True,'superseded':False,'current':True}]
        with self.assertRaisesRegex(ValueError,'independent admission'):
            m.fixture_step(self.reload(),fake,lambda *_:self.fail('withdrawn review grants nothing'))

    def test_native_evaluator_seed_binding_required_before_first_dispatch(self):
        mutations=(lambda s:s.update(records=[]),
                   lambda s:s['records'][0]['admission'].update(probability=.5),
                   lambda s:s['records'][0]['admission'].update(action='different-action'),
                   lambda s:s['records'][0]['admission'].update(snapshot='0'*64),
                   lambda s:s['records'][1].update(receipt='0'*64),
                   lambda s:s['records'][1].update(commitment='0'*64),
                   lambda s:s['records'][1].update(observed_success=True))
        for mutate in mutations:
            fake=self.fake();mutate(fake.forecast_status)
            with self.assertRaisesRegex(ValueError,'evaluator .1 seed forecast'):
                m.fixture_step(self.reload(),fake,lambda *_:self.fail('no dispatch without native forecast'))
        self.assertEqual(self.reload().status()['calls_consumed_or_reserved'],0)

    def test_development_null_roundtrip_and_strict_guidance_identity(self):
        fake=self.fake()
        value=self.relation(fake.current)
        value['relational']['development']=None
        result=m.fixture_step(self.journal,fake,lambda *_:self.reply(value))
        proposal=result['response']['proposals'][0]['proposal']
        original=copy.deepcopy(proposal)
        relation=fake.current['coding']['relational']
        native=copy.deepcopy(proposal);native.pop('development')
        entry={'admission':'a'*64,'binding':relation['binding'],'proposal':native,
               'withdrawn':False,'superseded':False,'current':True}
        relation['history']=[entry]
        # Either spelling compares equally; comparison must not mutate evidence.
        for p in (native,proposal):
            entry['proposal']=copy.deepcopy(p)
            m.project(self.reload(),fake,'experiment')
            m.require_stage(self.reload(),fake,fake.request(),'experiment')
        for field in ('withdrawn','superseded'):
            entry[field]=True
            with self.assertRaisesRegex(ValueError,'independent admission'):
                m.require_stage(self.reload(),fake,fake.request(),'experiment')
            entry[field]=False
        entry['current']=False
        with self.assertRaisesRegex(ValueError,'current reviewed'):
            m.require_stage(self.reload(),fake,fake.request(),'experiment')
        entry['current']=True
        self.assertEqual(proposal,original)
        self.assertEqual(m.relational_identity(native),m.relational_identity(proposal))
        mutations=[lambda p:p.update(development=False),lambda p:p.update(development=[]),
                   lambda p:p.update(development='null'),lambda p:p.update(development={}),
                   lambda p:p.update(private_note='foreign'),lambda p:p.update(aspiration='forged'),
                   lambda p:p.update(revises='0'*64),lambda p:p['parents'].__setitem__(0,'0'*64),
                   lambda p:p.update(catalysts=None)]
        for mutate in mutations:
            with self.subTest(mutate=mutate):
                entry['proposal']=copy.deepcopy(native);mutate(entry['proposal'])
                with self.assertRaises(ValueError):m.project(self.reload(),fake,'experiment')
        entry['proposal']=copy.deepcopy(native)
        fake.current['coding']['relational']['inputs']['snapshot']='0'*64
        with self.assertRaisesRegex(ValueError,'source binding drift'):
            m.project(self.reload(),fake,'experiment')
        self.assertEqual(self.reload().status()['calls_consumed_or_reserved'],1)

    def test_nonnull_development_remains_exact(self):
        proposal=self.relation(self.controller.request())['relational']
        proposal['development']={'original_revision':'a'*64,'prior_revision':'b'*64,
            'retained_meaning':'meaning','return_reveals':'observation','unresolved':'unknown',
            'proposed_development':None}
        identity=m.relational_identity(proposal)
        for key in proposal['development']:
            changed=copy.deepcopy(proposal);changed['development'][key]='different'
            self.assertNotEqual(identity,m.relational_identity(changed))
        changed=copy.deepcopy(proposal);changed['development'].pop('proposed_development')
        self.assertNotEqual(identity,m.relational_identity(changed))
        changed=copy.deepcopy(proposal);changed['development']['unknown']=None
        self.assertNotEqual(identity,m.relational_identity(changed))
        # No recursive None removal, truthiness coercion, or list reordering.
        for field,value in [('parents',list(reversed(proposal['parents']))),('revises',False)]:
            changed=copy.deepcopy(proposal);changed[field]=value
            self.assertNotEqual(identity,m.relational_identity(changed))

    def test_known_truncation_and_unknown_metering_are_preserved(self):
        fake=self.fake()
        response=self.reply(None);response['receipt']['structural_status']='invalid_or_unknown'
        response['receipt']['finish_reason']='length'
        with self.assertRaises(ValueError):m.fixture_step(self.journal,fake,lambda *_:response)
        outcome=next(e for e in self.reload().events() if e['kind']=='provider_outcome')
        self.assertEqual(outcome['status'],'invalid');self.assertEqual(outcome['usd'],'0.001')
        self.assertEqual(fake.stores,0)

    def test_http_status_is_only_a_bounded_integer(self):
        for value in (100,200,400,429,500,599):
            self.assertEqual(m.measured_receipt({'http_status':value})['http_status'],value)
        for value in (None,True,False,99,600,-1,429.0,'429',{'status':429}):
            self.assertNotIn('http_status',m.measured_receipt({'http_status':value}))
        self.assertNotIn('body',m.measured_receipt({'http_status':500,'body':'private error text'}))

    def test_unknown_metering_is_not_zero_and_available_details_retained(self):
        self.assertIsNone(m.measured_receipt({})['usd'])
        measured=m.measured_receipt({'status':'settled','usd':'0.01','usage':{'prompt_tokens':100,'completion_tokens':200,'total_tokens':300,
            'completion_tokens_details':{'reasoning_tokens':199,'private':'secret'}},'response_request_ids':['a'*8+'-'+ 'b'*4+'-'+ 'c'*4+'-'+ 'd'*4+'-'+ 'e'*12]})
        self.assertEqual(measured['usage']['completion_tokens_details'],{'reasoning_tokens':199})
        self.assertEqual(measured['usage_status'],'reported')

if __name__=='__main__':unittest.main()
