"""Offline authored negative contracts; no target execution or semantic model evidence."""
import copy,json,unittest
import test_engine_b91_project_envelope as base
m=base.m
class Audit(unittest.TestCase):
    def setUp(self):
        self.f=base.ProjectEnvelope();self.f.setUp();self.addCleanup(self.f.doCleanups)
        self.c=self.f.controller;self.r=self.c.request()
    def experiment(self):
        return {'experiment':{'harness':{'tests/tsr_supplemental_probe.rs':'#[test] fn new_probe(){assert!(parser::parse("x").is_err());}'},
          'cases':[{'id':'new_probe','args':[],'expected_stdout':''}],
          'mapping':{'existing_behaviour':{'cases':['new_probe'],'rationale':'Authored fixture only','limits':'Finite evidence'}}}}
    def test_actual_assembled_messages_expose_required_context(self):
        for stage in m.STAGES:
            _,body=m.project(self.f.journal,self.f.fake(),stage);ctx=json.loads(body['messages'][1]['content'])
            self.assertEqual(ctx['cargo_interface']['library_import'],'parser')
            self.assertEqual(ctx['cargo_interface']['dependencies'],[])
            self.assertEqual(set(ctx['cargo_interface']['existing_case_ids']),{'empty_input_handled','existing_behaviour_unchanged'})
            self.assertEqual(ctx['payload_contract'],m.payload_contract(self.r,stage))
            text=body['messages'][0]['content'];self.assertIn('strict JSON only',text);self.assertNotIn(m.prepare.TESTS,json.dumps(body))
            if stage=='experiment':
                payload=json.dumps(ctx['payload_contract']);self.assertIn('existing_behaviour',payload);self.assertIn('512 UTF-8',payload)
                self.assertEqual(set(ctx['payload_contract']['experiment']['mapping_value']['required']),{'cases','rationale','limits'})
            if stage=='boundary_patch':self.assertEqual(ctx['response_contract']['envelope_schema']['properties']['edits']['maxItems'],1)
    def test_payload_bound_matches_transport_utf8_and_refuses_without_truncation(self):
        body={'model':'gpt-6-luna','messages':[{'role':'user','content':'é'}]}
        base=len(json.dumps(body,allow_nan=False).encode())
        body['padding']='x'*(16384-base-len(', "padding": ""'))
        before=copy.deepcopy(body)
        self.assertEqual(len(json.dumps(body).encode()),16384)
        m.enforce_payload_bound(body,'relation_after_verification')
        self.assertEqual(body,before)
        body['padding']+='x'
        with self.assertRaises(m.PayloadBoundError) as caught:
            m.enforce_payload_bound(body,'relation_after_verification')
        self.assertEqual(caught.exception.actual,16385)
        self.assertIn('no dispatch',str(caught.exception))
        self.assertEqual(body['padding'],before['padding']+'x')

    def test_oversize_assembled_context_consumes_no_call_or_store(self):
        fake=self.f.fake();request=fake.request()
        relation=self.f.relation(request)['relational']
        # Authored oversized history exercises pre-dispatch accounting only.
        relation['relationship']='large fixture meaning '*1000
        response=m.local.bind_proposal(request,json.dumps({'relational':relation}))
        fake.current['coding']['relational']['history']=[dict(admission='1'*64,binding='2'*64,
            proposal=relation,withdrawn=False,superseded=False,current=True)]
        from unittest.mock import patch
        with patch.object(self.f.journal,'generated',return_value=[response]):
            with self.assertRaises(m.PayloadBoundError):
                m.fixture_step(self.f.journal,fake,lambda *_:self.fail('unexpected dispatch'))
        self.assertEqual(self.f.journal.status()['calls_consumed_or_reserved'],0)
        self.assertEqual(fake.stores,0)

    def test_native_experiment_positive_and_strict_negative_without_execution(self):
        value=self.experiment()
        m.validate_trial_payload(value,self.r,'experiment')
        def native(v):return self.c.validate(m.local.bind_proposal(self.r,json.dumps(v)))
        self.assertEqual(native(value)['assessments'][0]['disposition'],'deferred')
        changes=[lambda x:x['experiment']['mapping']['existing_behaviour'].update(limits=None),
                 lambda x:x['experiment']['mapping']['existing_behaviour'].update(rationale=' '),
                 lambda x:x['experiment']['mapping']['existing_behaviour'].update(rationale='é'*257),
                 lambda x:x['experiment']['mapping']['existing_behaviour'].update(cases=['missing']),
                 lambda x:x['experiment']['cases'][0].update(args=['extra']),
                 lambda x:x['experiment']['cases'][0].update(expected_stdout='pass'),
                 lambda x:x['experiment']['cases'][0].update(id='existing_behaviour_unchanged'),
                 lambda x:x['experiment'].update(harness={'tests/contract.rs':'changed'}),
                 lambda x:x['experiment'].update(harness={'tests/tsr_supplemental_../x.rs':'changed'})]
        for change in changes:
            bad=copy.deepcopy(value);change(bad)
            try:checked=native(bad)
            except ValueError:pass
            else:self.assertNotEqual(checked['assessments'][0]['disposition'],'deferred')
        self.assertEqual(self.c.request()['state_root'],self.r['state_root'])
    def test_total_utf8_harness_limit_prevents_native_broader_allowance(self):
        value=self.experiment();value['experiment']['harness']={'tests/tsr_supplemental_a.rs':'é'*1024,'tests/tsr_supplemental_b.rs':'é'*1024}
        m.validate_trial_payload(value,self.r,'experiment')
        value['experiment']['harness']['tests/tsr_supplemental_b.rs']+='x'
        # Native general adapter permits this size; B9.1 admission does not.
        self.assertEqual(self.c.validate(m.local.bind_proposal(self.r,json.dumps(value)))['assessments'][0]['disposition'],'deferred')
        with self.assertRaisesRegex(m.StageValidationError,'experiment_bytes_exceed_admission'):m.validate_trial_payload(value,self.r,'experiment')
    def test_aggregate_native_limit_and_explicit_persistence_fields(self):
        value=self.experiment()
        value['experiment']['harness']={'tests/tsr_supplemental_probe.rs':'//'+'x'*4094}
        value['experiment']['mapping']={key:{'cases':['new_probe'],'rationale':'x'*512,'limits':'y'*512}
            for key in ('empty_input','existing_behaviour')}
        m.validate_trial_payload(value,self.r,'experiment')
        with self.assertRaises(ValueError):self.c.validate(m.local.bind_proposal(self.r,json.dumps(value)))
        relation={'parents':[p['id'] for p in self.r['coding']['relational']['inputs']['parents']],
            'relationship':'Fixture relation','aspiration':'Fixture aspiration','discriminator':'Fixture finite observation',
            'revises':None,'catalysts':[]}
        for key in ('revises','catalysts'):
            omitted=copy.deepcopy(relation);del omitted[key]
            # Rust defaults are broader than this trial's persistent wire identity.
            self.assertEqual(self.c.validate(m.local.bind_proposal(self.r,json.dumps({'relational':omitted})))['assessments'][0]['disposition'],'deferred')
            with self.assertRaisesRegex(m.StageValidationError,'missing_relational_fields'):
                m.validate_trial_payload({'relational':omitted},self.r,'relation_after_seed')
        self.assertEqual(self.c.request()['state_root'],self.r['state_root'])
    def test_commit_writer_preserves_compact_utf8_without_execution(self):
        import test_engine_b91_driver as driver_test
        from types import SimpleNamespace
        from pathlib import Path
        value={'fixture':'é'*1800}
        def capture(*args):
            wire=Path(args[-1]).read_bytes()
            self.assertEqual(wire,m.canonical(value))
            self.assertLess(len(wire),8192)
            self.assertGreater(len(json.dumps(value).encode()),8192)
            return {'fixture_only':True}
        owner=SimpleNamespace(op=lambda name,fn:fn())
        controller=SimpleNamespace(directory=self.c.directory,cli=capture)
        self.assertEqual(driver_test.m.Driver.commit_response(owner,controller,value,'fixture'),{'fixture_only':True})
    def test_second_aim_and_patch_bounds_no_repair(self):
        value=self.experiment();value['experiment']['mapping']={'empty_input':value['experiment']['mapping']['existing_behaviour']}
        with self.assertRaisesRegex(m.StageValidationError,'second_aim_mapping_required'):m.validate_trial_payload(value,self.r,'experiment')
        for v,code in [({'edits':[{'path':'src/lib.rs','replacement':'é'*1025}],'rationale':'fixture'},'patch_bytes_exceed_admission'),
                       ({'edits':[{'path':'src/lib.rs','replacement':'x'}]*2,'rationale':'fixture'},'single_trial_edit_required'),
                       ({'edits':[{'path':'Cargo.toml','replacement':'x'}],'rationale':'fixture'},'invalid_trial_edit')]:
            before=copy.deepcopy(v)
            with self.assertRaisesRegex(m.StageValidationError,code):m.validate_trial_payload(v,self.r,'boundary_patch')
            self.assertEqual(v,before)
if __name__=='__main__':unittest.main()
