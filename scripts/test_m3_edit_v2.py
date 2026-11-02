"""Offline schema/decoder differential tests and fake-provider diagnostic privacy checks."""
import copy
import importlib.util
import json
from pathlib import Path
import re
import tempfile
import unittest
from unittest.mock import patch

HERE=Path(__file__).resolve().parent

def load(name):
    spec=importlib.util.spec_from_file_location(name,HERE/(name+'.py'))
    module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module);return module

new=load('m3-edit-v2-broker')
old=load('m3-edit-broker')
fixtures=load('test_m3_edit_broker')


def args(action):
    value=dict(action=action,path='',id='',expected_digest='',old_text='',new_text='',forecast=None)
    if action in ('inspect','edit'):value['path']=fixtures.request()['required_reads'][0]
    if action=='test':value.update(id='named-failing-test',forecast=0.5)
    if action=='edit':value.update(expected_digest='a'*64,old_text='before',new_text='after',forecast=0.5)
    return value


def cases(action):
    base=args(action)
    yield base
    for key in base:
        missing=base.copy();del missing[key];yield missing
        for value in (None,False,0,[],{},'', 'nonempty'):
            yield base|{key:value}
    for value in (-1,0,1,1.01,True,None):yield base|{'forecast':value}
    yield base|{'SECRET_UNKNOWN_KEY':'SECRET_VALUE'}
    yield base|{'expected_digest':'A'*64}
    yield base|{'expected_digest':'a'*64+'\n'}
    if action=='edit':
        yield base|{'new_text':''}
        yield base|{'old_text':'before λ','new_text':'after λ'}


def response(value):
    result=fixtures.response()
    result['choices'][0]['message']['tool_calls'][0]['function']['arguments']=json.dumps(value)
    return result


def schema_accepts(schema,value):
    """Independent evaluator for exactly the emitted JSON Schema keyword subset."""
    kinds={'null':lambda x:x is None,'string':lambda x:type(x) is str,
           'number':lambda x:type(x) in (int,float),'object':lambda x:type(x) is dict}
    if 'type' in schema:
        types=schema['type'] if isinstance(schema['type'],list) else [schema['type']]
        if not any(kinds[t](value) for t in types):return False
    if 'enum' in schema and value not in schema['enum']:return False
    if type(value) is dict:
        if any(k not in value for k in schema.get('required',[])):return False
        props=schema.get('properties',{})
        if schema.get('additionalProperties') is False and any(k not in props for k in value):return False
        if any(not schema_accepts(props[k],v) for k,v in value.items() if k in props):return False
    if type(value) is str:
        if len(value)<schema.get('minLength',0) or len(value)>schema.get('maxLength',10**9):return False
        if 'pattern' in schema and re.search(schema['pattern'],value) is None:return False
    if type(value) in (int,float):
        if 'minimum' in schema and not value>=schema['minimum']:return False
        if 'maximum' in schema and not value<=schema['maximum']:return False
    if 'oneOf' in schema and sum(schema_accepts(s,value) for s in schema['oneOf'])!=1:return False
    return True


class Contract(unittest.TestCase):
    def test_demonstrated_old_schema_gaps(self):
        oldschema=old.encode(fixtures.request())['tools'][0]['function']['parameters']
        newschema=new.encode(fixtures.request())['tools'][0]['function']['parameters']
        cases=[args('inspect')|{'id':'unused'},args('test')|{'forecast':None},
               args('stop')|{'forecast':2},args('edit')|{'expected_digest':'invalid'},args('edit')|{'old_text':''}]
        for value in cases:
            with self.subTest(value=value):
                self.assertTrue(schema_accepts(oldschema,value))
                with self.assertRaises(Exception):old.decode(response(value),fixtures.request())
                self.assertFalse(schema_accepts(newschema,value))

    def test_minimal_variants_and_examples(self):
        for action in new.ACTION_FIELDS:
            value=args(action)
            schema=new.encode(fixtures.request())['tools'][0]['function']['parameters']
            self.assertTrue(schema_accepts(schema,value))
            self.assertEqual(new.decode(response(value),fixtures.request()),old.decode(response(value),fixtures.request()))
        for value in new.examples(fixtures.request()):
            self.assertTrue(schema_accepts(schema,value))
            new.decode(response(value),fixtures.request())

    def test_table_differential_and_schema_expressible_constraints(self):
        schema=new.encode(fixtures.request())['tools'][0]['function']['parameters']
        total=0
        for action in new.ACTION_FIELDS:
            for value in cases(action):
                total+=1
                outcomes=[]
                for module in (old,new):
                    try:outcomes.append(module.decode(response(value),fixtures.request()))
                    except Exception:outcomes.append(None)
                self.assertEqual(outcomes[0],outcomes[1],(action,value))
                self.assertEqual(schema_accepts(schema,value),outcomes[1] is not None,(action,value))
        self.assertGreater(total,250)

    def test_runtime_cross_field_and_malformed_diagnostics(self):
        for value in (args('edit')|{'new_text':'before'},args('edit')|{'forecast':float('nan')}):
            with self.assertRaises(new.ValidationFailure):new.decode(response(value),fixtures.request())
        for text in ('{', '[', '{"SECRET_KEY":0,"SECRET_KEY":1}', '['*1500+'0'+']'*1500):
            r=response(args('stop'));r['choices'][0]['message']['tool_calls'][0]['function']['arguments']=text
            with self.assertRaises(new.ValidationFailure) as error:new.decode(r,fixtures.request())
            self.assertNotIn('SECRET',json.dumps(error.exception.detail))
            self.assertIn(error.exception.detail['path'],new.SAFE_PATHS)

    def test_envelope_and_usage_parity(self):
        variants=[None,{}, {'choices':[]}, {'choices':[None]}, {'choices':[{}]}]
        for usage in (None,{}, {'total_tokens':True}, {'total_tokens':-1}, {'total_tokens':'SECRET_TOKEN'}):
            r=response(args('stop'));r['usage']=usage;variants.append(r)
        for calls in (None,[],[None],[{}],[{},{}]):
            r=response(args('stop'));r['choices'][0]['message']['tool_calls']=calls;variants.append(r)
        for finish in ('length','content_filter'):
            r=response(args('stop'));r['choices'][0]['finish_reason']=finish;variants.append(r)
        r=response(args('stop'));r['choices'][0]['message']['refusal']='SECRET_REFUSAL';variants.append(r)
        for r in variants:
            with self.assertRaises(Exception):old.decode(r,fixtures.request())
            with self.assertRaises(new.ValidationFailure) as error:new.decode(r,fixtures.request())
            self.assertNotIn('SECRET',json.dumps(error.exception.detail))
        for value,path,expected,actual,reason in (
            (args('inspect')|{'path':None},'/arguments/path','string','null','type'),
            (args('test')|{'forecast':None},'/arguments/forecast','number','null','required'),
            (args('stop')|{'id':'SECRET'},'/arguments/id','string','string','must_be_empty'),
            (args('stop')|{'SECRET_KEY':'SECRET'},'/arguments/*','object','object','unknown_field')):
            with self.assertRaises(new.ValidationFailure) as error:new.decode(response(value),fixtures.request())
            self.assertEqual(error.exception.detail,dict(version=1,path=path,expected_type=expected,actual_type=actual,reason=reason))

    def test_fake_provider_each_variant_and_private_failures(self):
        for action in new.ACTION_FIELDS:
            for value in cases(action):
                with self.subTest(action=action,value=value),tempfile.TemporaryDirectory() as directory:
                    ledger=Path(directory)/'cost.jsonl';ledger.write_bytes(b'')
                    r=response(value);r['billing']=dict(status='settled',currency='USD',billed_cost_usd='.001')
                    def http(route,key,*a,**kw):
                        if route=='models':return {'data':[dict(id=new.transport.MODEL,pricing=dict(list_input_per_million='.15',list_output_per_million='.60'))]}
                        return r
                    request=fixtures.request();request['deadline_ms']=30000
                    try:expected=old.decode(response(value),request)
                    except Exception:expected=None
                    # Test ledger only; the production transport's locking/settlement stays real.
                    with patch.object(new.transport,'credential',return_value='SECRET_CREDENTIAL'),patch.object(new.transport,'request_json',side_effect=http) as net,patch.object(new.transport,'smoke_budget',return_value=new.transport.D('0')):
                        if expected is not None:self.assertEqual(new.transport.complete_with_diagnostics(request,str(ledger)),expected)
                        else:
                            with self.assertRaises(new.ValidationFailure):new.transport.complete_with_diagnostics(request,str(ledger))
                    self.assertEqual(net.call_count,2)
                    log=(Path(directory)/'cost-diagnostics.jsonl').read_text()
                    self.assertNotIn('SECRET',log)
                    row=json.loads(log)
                    if expected is None:
                        self.assertIn(row['proposal_validation']['path'],new.SAFE_PATHS)
                        self.assertIn(row['proposal_validation']['actual_type'],new.JSON_TYPES)
                    self.assertEqual(len(ledger.read_text().splitlines()),2)

if __name__=='__main__':unittest.main()
