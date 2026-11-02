"""Historical read-only pair validation; no model/native command/target execution."""
import copy
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
spec=importlib.util.spec_from_file_location('guidance_pair',Path(__file__).with_name('engine-guidance-pair-export.py'));m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

class Pair(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.data=m.material()
    def setUp(self):
        tmp=tempfile.TemporaryDirectory();self.addCleanup(tmp.cleanup);self.out=Path(tmp.name)/'pair'
        # Exact real extraction was performed once; these payload corruption tests
        # freeze that read-only result, not an authored expected winning response.
        self.mock=patch.object(m,'material',side_effect=lambda:copy.deepcopy(self.data));self.mock.start();self.addCleanup(self.mock.stop)
        self.identity=m.prepare(self.out)
    def test_exact_single_field_neutral_order_no_authority_and_replay(self):
        result=m.check(self.out,self.identity);self.assertEqual(result['status'],'checked_output_only')
        self.assertTrue(all(x<=32768 for x in result['payload_bytes']))
        bodies=[json.loads((self.out/(x+'.json')).read_text()) for x in ('X','Y')]
        contexts=[json.loads(x['messages'][1]['content']) for x in bodies]
        selected=[x.pop('active_guidance') for x in contexts];self.assertNotEqual(selected[0],selected[1]);self.assertEqual(contexts[0],contexts[1])
        self.assertNotIn('semantic_return',contexts[0]);self.assertNotIn('development',json.dumps(contexts[0]['lineage_provenance']))
        self.assertFalse(contexts[0]['authority']['adoption_authorized']);self.assertEqual(contexts[0]['authority']['executions_authorized'],0)
        self.assertEqual(json.loads((self.out/'manifest.json').read_text())['order'],['X','Y'])
        before={p.name:p.read_bytes() for p in self.out.iterdir()};self.assertEqual(m.prepare(self.out),self.identity);m.check(self.out,self.identity)
        self.assertEqual(before,{p.name:p.read_bytes() for p in self.out.iterdir()})
    def test_payload_mutations_rejected_even_with_updated_local_payload_hash(self):
        original={p.name:p.read_bytes() for p in self.out.iterdir()}
        def mutate_source(c):c['source']['text']='different source'
        def mutate_parent(c):c['parents'][0]['meaning']='different desire'
        def mutate_lineage(c):c['lineage_provenance'][1]['revises']='0'*64
        def mutate_evidence(c):c['evidence']['case_facts'][0]['outcome']='failed'
        def mutate_private(c):c['private_operator_notes']='must not export'
        def mutate_authority(c):c['authority']['executions_authorized']=1
        for change in (mutate_source,mutate_parent,mutate_lineage,mutate_evidence,mutate_private,mutate_authority):
            with self.subTest(change=change.__name__):
                for name,data in original.items():(self.out/name).write_bytes(data)
                body=json.loads(original['X.json']);context=json.loads(body['messages'][1]['content']);change(context);body['messages'][1]['content']=m.e.canonical(context).decode();(self.out/'X.json').write_text(json.dumps(body))
                manifest=json.loads(original['manifest.json']);manifest['payloads']['X']=m.e.sha(body);(self.out/'manifest.json').write_text(json.dumps(manifest))
                with self.assertRaises(ValueError):m.check(self.out,m.e.sha(manifest))
    def test_no_winning_patch_private_review_or_full_state(self):
        text=''.join((self.out/(label+'.json')).read_text() for label in ('X','Y'))
        self.assertNotIn('readiness_review independent agent',text);self.assertNotIn('events.sqlite',text);self.assertNotIn('tests/contract.rs',text)
        self.assertNotIn('return Ok(Vec::new())',text)
        self.assertNotIn('native-shaped',text)

class Provenance(unittest.TestCase):
    def test_missing_review_fails_before_output(self):
        with tempfile.TemporaryDirectory() as tmp,patch.object(m,'REVIEWS',Path(tmp)):
            with self.assertRaisesRegex(ValueError,'review request'):m.material()
    def test_missing_or_wrong_source_actual_receipt_rejected(self):
        native=m.native_records
        for mode in ('absent','operator','altered_case'):
            def replacement(project,pins):
                get=native(project,pins)
                def read(identity):
                    event,value=get(identity)
                    if value.get('record',{}).get('phase')=='receipt':
                        if mode=='absent':raise ValueError('missing actual native record')
                        if mode=='operator':event={**event,'source':'operator'}
                        if mode=='altered_case':
                            value=copy.deepcopy(value);value['record']['observations'][0]['passed']=False
                    return event,value
                return read
            with self.subTest(mode=mode),patch.object(m,'native_records',side_effect=replacement),self.assertRaises(ValueError):m.material()
