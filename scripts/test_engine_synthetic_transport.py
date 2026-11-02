"""Offline transport tests: credentials and HTTP replaced, no socket/provider calls."""
import copy
import importlib.util
import json
from pathlib import Path
import signal
import unittest
from unittest.mock import patch
import urllib.error

s=importlib.util.spec_from_file_location('transport',Path(__file__).with_name('engine-synthetic-transport.py'))
t=importlib.util.module_from_spec(s);s.loader.exec_module(t)

class Response:
    status=200
    def __init__(self,raw):self.raw=raw
    def __enter__(self):return self
    def __exit__(self,*args):pass
    def read(self,bound):return self.raw[:bound]

class Transport(unittest.TestCase):
    def body(self):return {'model':t.MODEL,'stream':False,'max_tokens':8192,'messages':[{'role':'user','content':'synthetic fixture'}]}
    def envelope(self):return {'model':t.MODEL,'request_id':'fixture-id','billing':{'status':'settled','currency':'USD','billed_cost_usd':'0.001'},'usage':{'prompt_tokens':10,'completion_tokens':20,'total_tokens':30,'completion_tokens_details':{'reasoning_tokens':12},'prompt_tokens_details':{'cached_tokens':3}},'choices':[{'finish_reason':'stop','message':{'content':'{"answer":1}','reasoning_content':'PRIVATE_REASONING'}}]}
    def invoke(self,envelope=None,error=None,raw=None):
        with patch.object(t.broker,'credential',return_value='SECRET'),patch.object(t.urllib.request,'build_opener') as opener:
            if error:opener.return_value.open.side_effect=error
            else:opener.return_value.open.return_value=Response(raw if raw is not None else json.dumps(envelope or self.envelope()).encode())
            result=t.invoke(self.body(),1)
            self.assertEqual(opener.return_value.open.call_count,1)
            request=opener.return_value.open.call_args.args[0]
            self.assertEqual(request.full_url,t.ENDPOINT)
            self.assertEqual(request.get_header('X-ci-model-lock'),'1')
            self.assertNotIn('SECRET',json.dumps(result));self.assertNotIn('PRIVATE_REASONING',json.dumps(result))
            self.assertEqual(signal.getitimer(signal.ITIMER_REAL)[0],0)
            return result
    def test_capacity_profile_actual_request_and_length_receipt(self):
        body={'model':'gpt-6-luna','stream':False,'max_completion_tokens':32768,'messages':[{'role':'user','content':'authored only'}]}
        envelope=dict(self.envelope(),model='gpt-6-luna');envelope['choices'][0]['finish_reason']='length'
        envelope['choices'][0]['message']['content']='{"unfinished":'
        with patch.object(t.broker,'credential',return_value='SECRET'),patch.object(t.urllib.request,'build_opener') as opener:
            with self.assertRaises(ValueError):t.invoke(body,1,expected_model='gpt-6-luna',payload_limit=32768)
            opener.assert_not_called()
            opener.return_value.open.return_value=Response(json.dumps(envelope).encode())
            result=t.invoke(body,1,expected_model='gpt-6-luna',payload_limit=32768,completion_limit=32768)
            self.assertEqual(json.loads(opener.return_value.open.call_args.args[0].data),body)
            self.assertIsNone(result['value']);self.assertEqual(result['receipt']['status'],'settled')
            self.assertEqual(result['receipt']['error_code'],'incomplete_output')
            self.assertEqual(result['receipt']['usage_status'],'reported')
    def test_explicit_shadow_model_pin_and_return_identity(self):
        for model in sorted(t.SHADOW_MODELS):
            for returned in (model, 'unapproved-model'):
                body=dict(self.body(),model=model)
                if model=='gpt-6-luna':body['max_completion_tokens']=body.pop('max_tokens')
                envelope=dict(self.envelope(),model=returned)
                with patch.object(t.broker,'credential',return_value='SECRET'),patch.object(t.urllib.request,'build_opener') as opener:
                    opener.return_value.open.return_value=Response(json.dumps(envelope).encode())
                    result=t.invoke(body,1,expected_model=model)
                    request=opener.return_value.open.call_args.args[0]
                    self.assertEqual(request.get_header('X-ci-model-lock'),'1')
                    self.assertEqual(json.loads(request.data)['model'],model)
                    self.assertEqual(opener.return_value.open.call_count,1)
                    self.assertEqual(result['receipt']['usd'],'0.001')
                    self.assertEqual(result['receipt']['usage']['total_tokens'],30)
                    if returned==model:self.assertEqual(result['value'],{'answer':1})
                    else:
                        self.assertIsNone(result['value'])
                        self.assertEqual(result['receipt']['error_code'],'model_mismatch')
                    self.assertNotIn('SECRET',json.dumps(result))
    def test_model_specific_output_limit_is_not_silently_rewritten(self):
        with patch.object(t.broker,'credential') as credential:
            for model in ('gpt-6-luna','deepseek-v4.1-flash-1017'):
                body=dict(self.body(),model=model)
                if model!='gpt-6-luna':body['max_completion_tokens']=body.pop('max_tokens')
                with self.assertRaises(ValueError):t.invoke(body,1,expected_model=model)
            both=dict(self.body(),model='gpt-6-luna',max_completion_tokens=8192)
            with self.assertRaises(ValueError):t.invoke(both,1,expected_model='gpt-6-luna')
            credential.assert_not_called()

    def test_body_cannot_expand_default_or_explicit_model_authority(self):
        with patch.object(t.broker,'credential') as credential:
            for model in ('glm-5.3-flash','qwen3.8-flash'):
                with self.assertRaises(ValueError):t.invoke(dict(self.body(),model=model),1)
                with self.assertRaises(ValueError):t.invoke(self.body(),1,expected_model=model)
            for model in ('claude-opus-5.5','gpt-6-astra','other',None,[]):
                with self.assertRaises(ValueError):t.invoke(self.body(),1,expected_model=model)
            credential.assert_not_called()

    def test_valid_metadata_and_no_raw_body(self):
        result=self.invoke();r=result['receipt']
        self.assertEqual(result['value'],{'answer':1});self.assertEqual(r['status'],'settled')
        self.assertEqual(r['usage']['completion_tokens_details']['reasoning_tokens'],12)
        self.assertEqual(r['usage']['prompt_tokens_details']['cached_tokens'],3)
        self.assertEqual(r['response_status'],'received');self.assertEqual(r['usage_status'],'reported')
        self.assertEqual(len(r['body_sha256']),64)
    def test_terminal_truncation_retains_cost_and_usage(self):
        e=self.envelope();e['choices'][0]['finish_reason']='length'
        r=self.invoke(e);self.assertIsNone(r['value']);self.assertEqual(r['receipt']['usd'],'0.001')
        self.assertEqual(r['receipt']['finish_reason'],'length');self.assertEqual(r['receipt']['response_status'],'received')
    def test_unknown_billing_and_usage_explicit(self):
        e=self.envelope();e.pop('billing');e['usage']={}
        r=self.invoke(e);self.assertEqual(r['receipt']['status'],'unknown');self.assertEqual(r['receipt']['usage_status'],'unknown')
        self.assertEqual(r['receipt']['response_status'],'received')
    def test_inconsistent_usage_is_retained_not_trusted(self):
        for mutate in [lambda e:e['usage'].update(total_tokens=1),
                       lambda e:e['usage']['completion_tokens_details'].update(reasoning_tokens=21)]:
            e=self.envelope();mutate(e);r=self.invoke(e)['receipt']
            self.assertEqual(r['status'],'settled');self.assertEqual(r['usage_status'],'inconsistent')
            self.assertEqual(r['usd'],'0.001')

    def test_network_timeout_and_http_error_are_distinct(self):
        r=self.invoke(error=TimeoutError('SECRET'));self.assertEqual(r['receipt']['response_status'],'uncertain')
        r=self.invoke(error=urllib.error.HTTPError(t.ENDPOINT,503,'PRIVATE_REASONING',{},None))
        self.assertEqual(r['receipt']['response_status'],'received_http_error');self.assertIsNone(r['value'])
    def test_bound_parse_model_and_tools_fail_without_retry(self):
        r=self.invoke(raw=b'x'*(t.RESPONSE_BOUND+2));self.assertEqual(r['receipt']['error_code'],'response_bound')
        r=self.invoke(raw=b'{"x":1,"x":2}');self.assertEqual(r['receipt']['error_code'],'duplicate_key')
        for mutate in [lambda e:e.update(model='other'),lambda e:e['choices'][0]['message'].update(tool_calls=[{}]),lambda e:e['choices'][0]['message'].update(content='NaN')]:
            e=self.envelope();mutate(e);self.assertIsNone(self.invoke(e)['value'])
    def test_malformed_json_location_retains_only_safe_metadata(self):
        r=self.invoke(raw=b'{SECRET')['receipt']
        self.assertEqual(r['error_code'],'malformed_json');self.assertEqual(r['parse_location'],'wire')
        e=self.envelope();e['choices'][0]['message']['content']='{SECRET'
        r=self.invoke(e)['receipt']
        self.assertEqual(r['error_code'],'malformed_json');self.assertEqual(r['parse_location'],'content')
        self.assertEqual(r['usd'],'0.001');self.assertEqual(r['usage']['completion_tokens'],20)

    def test_request_refusal_before_credentials(self):
        with patch.object(t.broker,'credential') as credential:
            for change in [{'model':'other'},{'stream':True},{'max_tokens':0}]:
                with self.assertRaises(ValueError):t.invoke(dict(self.body(),**change))
            for timeout in [0,241,float('nan'),True]:
                with self.assertRaises(ValueError):t.invoke(self.body(),timeout)
            credential.assert_not_called()
    def test_sanitized_stage_diagnostics_without_retry(self):
        import socket
        for error,kind,number in [(urllib.error.URLError(socket.gaierror(8,'SECRET')),'dns',8),
                                  (PermissionError(1,'SECRET'),'permission',1),
                                  (TimeoutError('SECRET'),'timeout',None)]:
            r=self.invoke(error=error)['receipt']
            self.assertEqual(r['diagnostic'],{'contract':'synthetic-transport-diagnostic@v1',
                'stage':'open','category':kind,'errno':number})
            self.assertEqual(r['status'],'unknown');self.assertIsNone(r['usd'])
        with patch.object(t.broker,'credential',side_effect=PermissionError(1,'SECRET')),patch.object(t.urllib.request,'build_opener') as op:
            r=t.invoke(self.body(),1)['receipt'];op.assert_not_called()
            self.assertEqual(r['diagnostic']['stage'],'credential')
            self.assertEqual(r['diagnostic']['category'],'permission')
            self.assertEqual(r['status'],'unknown')
            self.assertNotIn('SECRET',json.dumps(r))

    def test_header_identity_survives_read_parse_and_http_failure(self):
        uid='77fb54b7-249a-477d-bbb4-fb2d331a86b1'
        class Partial(Response):
            headers={'x-ci-request-id':uid}
            def read(self,bound):raise TimeoutError('SECRET')
        with patch.object(t.broker,'credential',return_value='SECRET'),patch.object(t.urllib.request,'build_opener') as op:
            op.return_value.open.return_value=Partial(b'')
            r=t.invoke(self.body(),1)['receipt']
            self.assertEqual(op.return_value.open.call_count,1)
            self.assertEqual(r['response_request_ids'],[uid])
            self.assertEqual(r['diagnostic']['stage'],'read')
            self.assertEqual(r['status'],'unknown');self.assertIsNone(r['usd'])
            op.return_value.open.return_value=Response(b'not json')
            op.return_value.open.return_value.headers={'x-ci-request-id':uid}
            r=t.invoke(self.body(),1)['receipt']
            self.assertEqual(r['response_request_ids'],[uid])
            self.assertEqual(r['diagnostic']['stage'],'parse')
        error=urllib.error.HTTPError(t.ENDPOINT,503,'SECRET',{'x-ci-request-id':uid},None)
        r=self.invoke(error=error)['receipt'];self.assertEqual(r['response_request_ids'],[uid])
        self.assertEqual(r['status'],'unknown')
        for value in ['SECRET','x'*1000,'../'+uid]:
            r={};t.response_identity(r,{'x-ci-request-id':value});self.assertEqual(r,{})
        r={};t.response_identity(r,{'x-ci-request-id':uid,'X-Cheaper-Inference-Request-Id':'87fb54b7-249a-477d-bbb4-fb2d331a86b1'})
        self.assertEqual(r['response_identity_status'],'conflicting')
        self.assertEqual(len(r['response_request_ids']),2)

    def test_schema_strict_types_limits_and_unknown_keywords(self):
        schema={'type':'object','additionalProperties':False,'required':['x'],'properties':{'x':{'type':'array','maxItems':2,'uniqueItems':True,'items':{'type':'integer','minimum':0,'maximum':2}}}}
        t.validate({'x':[1,2]},schema)
        for value in [{'x':[True]},{'x':[1,1]},{'x':[1,2,0]},{'x':[10**400]},{'x':[float('nan')]},{'x':[],'extra':1}]:
            with self.assertRaises(ValueError):t.validate(value,schema)
        with self.assertRaises(ValueError):t.validate('a',{'type':'string','pattern':'a'})
        with self.assertRaises(ValueError):t.validate('é'*300,{'type':'string'})
        t.validate(None,{'anyOf':[{'type':'null'},{'type':'boolean'}]})

if __name__=='__main__':unittest.main()
