"""Action-specific wire parity and content-free transport diagnostics; no network."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
from test_m3_edit_v2 import load, response, schema_accepts, fixtures
new=load('m3-edit-v3-broker')
old=load('m3-edit-v2-broker')

def cases(request):
    for base in new.examples(request):
        yield base
        for key in base:
            missing=base.copy();del missing[key];yield missing
            for value in (None,False,0,[],{},'', 'SECRET_VALUE'):yield base|{key:value}
        for key in ('id','test_id','path','forecast','SECRET_KEY'):
            if key not in base:yield base|{key:'SECRET_VALUE'}
        if base['action']=='test':yield base|{'test_id':'unknown-test'}
        if base['action']=='edit':
            yield base|{'new_text':''}
            yield base|{'old_text':'λ old','new_text':'λ new'}
            yield base|{'expected_digest':'A'*64}
            yield base|{'expected_digest':'a'*64+'\n'}
        if 'forecast' in base:
            for value in (-1,0,1,1.01,float('inf'),float('nan')):yield base|{'forecast':value}

def normalized_v2(value):
    result=dict(action=value['action'],path='',id='',expected_digest='',old_text='',new_text='',forecast=None)
    result.update({('id' if k=='test_id' else k):v for k,v in value.items()})
    return result

class Contract(unittest.TestCase):
    def test_schema_and_decoder_matrix(self):
        request=fixtures.request();schema=new.schema(request);count=0
        for value in cases(request):
            count+=1
            try:result=new.decode(response(value),request)
            except new.ValidationFailure as error:
                result=None
                self.assertIn(error.detail['action_category'],new.ACTION_CATEGORIES)
                self.assertNotIn('SECRET',json.dumps(error.detail))
            self.assertEqual(schema_accepts(schema,value),result is not None,value)
            if result is not None:self.assertEqual(result,old.decode(response(normalized_v2(value)),request))
        self.assertGreater(count,120)
    def test_examples_and_instructions_same_contract(self):
        request=fixtures.request();body=new.encode(request)
        self.assertEqual(body['tool_choice'],'required');self.assertEqual(len(body['tools']),1)
        for example in new.examples(request):self.assertEqual(set(example),set(new.CONTRACT[example['action']]))
        self.assertNotIn('Use empty strings for irrelevant fields',body['messages'][0]['content'])
        self.assertIn('named-failing-test',body['messages'][0]['content'])
    def test_missing_semantic_selector_and_unknown_legacy_id(self):
        request=fixtures.request()
        for value,path,reason in ((dict(action='test',forecast=.5),'/arguments/test_id','required'),(dict(action='test',test_id='unknown',forecast=.5),'/arguments/test_id','scope'),(dict(action='stop',id=''),'/arguments/*','unknown_field')):
            with self.assertRaises(new.ValidationFailure) as e:new.decode(response(value),request)
            self.assertEqual(e.exception.detail['path'],path);self.assertEqual(e.exception.detail['reason'],reason)
            self.assertEqual(e.exception.detail['action_category'],value['action'])
    def test_boundaries_and_privacy(self):
        request=fixtures.request()
        for value,category in (({},'missing'),({'action':'SECRET_ACTION'},'invalid'),({'action':{}},'invalid'),(dict(action='edit',path='x'),'edit')):
            with self.assertRaises(new.ValidationFailure) as e:new.decode(response(value),request)
            self.assertEqual(e.exception.detail['action_category'],category);self.assertNotIn('SECRET',json.dumps(e.exception.detail))
        for raw in ('{','{"action":"stop","SECRET_KEY":1,"SECRET_KEY":2}', '['*1500+'0'+']'*1500, ' '*65537):
            r=response({});r['choices'][0]['message']['tool_calls'][0]['function']['arguments']=raw
            with self.assertRaises(new.ValidationFailure) as e:new.decode(r,request)
            self.assertEqual(e.exception.detail['action_category'],'unparsed');self.assertNotIn('SECRET',json.dumps(e.exception.detail))
        edit=new.examples(request)[2];edit['new_text']=edit['old_text']
        with self.assertRaises(new.ValidationFailure):new.decode(response(edit),request)
        # Multiple calls and stale correlation cannot be silently coalesced/defaulted.
        r=response(dict(action='stop'));calls=r['choices'][0]['message']['tool_calls'];calls.append(copy.deepcopy(calls[0]))
        with self.assertRaises(new.ValidationFailure):new.decode(r,request)
        for identifier in ('a'*64,'b'*64):
            result=new.decode(response(dict(action='test',test_id='named-failing-test',forecast=.5)),request|{'id':identifier})
            self.assertEqual(result['reply']['request'],identifier);self.assertEqual(result['reply']['proposal']['id'],'named-failing-test')
    def test_transport_matrix(self):
        request=fixtures.request();request['deadline_ms']=30000
        for value in cases(request):
            with tempfile.TemporaryDirectory() as directory:
                ledger=Path(directory)/'cost.jsonl';ledger.write_bytes(b'')
                r=response(value);r['billing']=dict(status='settled',currency='USD',billed_cost_usd='.001')
                def http(route,key,*a,**kw):
                    if route=='models':return {'data':[dict(id=new.transport.MODEL,pricing=dict(list_input_per_million='.15',list_output_per_million='.60'))]}
                    return r
                try:expected=new.decode(response(value),request)
                except new.ValidationFailure:expected=None
                with patch.object(new.transport,'credential',return_value='SECRET_CREDENTIAL'),patch.object(new.transport,'request_json',side_effect=http) as net,patch.object(new.transport,'smoke_budget',return_value=new.transport.D('0')):
                    if expected is not None:self.assertEqual(new.transport.complete_with_diagnostics(request,str(ledger)),expected)
                    else:
                        with self.assertRaises(new.ValidationFailure):new.transport.complete_with_diagnostics(request,str(ledger))
                self.assertEqual(net.call_count,2);self.assertEqual(len(ledger.read_text().splitlines()),2)
                log=(Path(directory)/'cost-diagnostics.jsonl').read_text();self.assertNotIn('SECRET',log)
                row=json.loads(log);self.assertIn(row['proposal_action'],new.ACTION_CATEGORIES)
                if expected is None:self.assertIn(row['proposal_validation']['path'],new.SAFE_PATHS)

if __name__=='__main__':unittest.main()
