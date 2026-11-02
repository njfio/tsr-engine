"""Offline bounded-wire and accounting fixtures. Never a live admission."""
import copy
import json
import unittest
from unittest.mock import patch
import test_engine_b91_project_envelope as base
import test_engine_synthetic_transport as transport_test
m=base.m;t=transport_test.t

class Capacity(unittest.TestCase):
    def test_finite_profile_is_explicit_and_stage_model_bound(self):
        for stage in m.STAGES:
            self.assertEqual(m.request_limits('gpt-6-luna',stage)['payload_bytes'],16384)
            if stage in m.STAGES[-2:]:self.assertEqual(m.request_limits('gpt-6-luna',stage,32768)['input_tokens'],32768)
            else:
                with self.assertRaises(ValueError):m.request_limits('gpt-6-luna',stage,32768)
        for cap in (True,16385,65536,None):
            with self.assertRaises(ValueError):m.request_limits('gpt-6-luna',m.STAGES[-1],cap)
        with self.assertRaises(ValueError):m.request_limits(m.MODEL,m.STAGES[-1],32768)

    def test_utf8_double_encoding_bound_including_del_and_astral_text(self):
        characters=['\x00','\x7f','"','\\','\n','é','\u07ff','\u0800','\uffff','😀']
        characters += [chr(x) for x in range(0,0x110000,997) if not 0xd800<=x<=0xdfff]
        for ch in characters:
            inner=m.canonical({'text':ch*13}).decode()
            outer=json.dumps(inner,ensure_ascii=False,allow_nan=False).encode()
            self.assertLessEqual(len(outer),2*len(inner.encode())+2)
            self.assertEqual(json.loads(outer),inner)
        # ASCII escaping does NOT obey the old proposed 3x bound for DEL.
        self.assertGreater(len(json.dumps('\x7f'*100).encode()),3*100)

    def test_single_maximum_sized_revision_growth_bound(self):
        for pattern in ('x','é','😀','"','\\','\x7f','\n'):
            proposal={'parents':['a'*64,'b'*64], 'relationship':'fixture',
                'aspiration':'fixture','discriminator':'fixture','revises':'c'*64,'catalysts':[],
                'development':{'original_revision':'d'*64,'prior_revision':'c'*64,
                    'retained_meaning':'fixture','return_reveals':'fixture','unresolved':'fixture',
                    'proposed_development':'fixture'}}
            fields=[(proposal,k) for k in ('relationship','aspiration','discriminator')]
            fields += [(proposal['development'],k) for k in ('retained_meaning','return_reveals','unresolved','proposed_development')]
            for container,key in fields:
                while len((container[key]+pattern).encode())<=512:
                    container[key]+=pattern
                    if len(m.canonical(proposal))>4096:
                        container[key]=container[key][:-len(pattern)];break
            self.assertLessEqual(len(m.canonical(proposal)),4096)
            history={'relational_history':[],'fixed':'baseline'}
            body={'messages':[{'role':'user','content':m.canonical(history).decode()}]}
            base_bytes=len(json.dumps(body,ensure_ascii=False).encode())
            history['relational_history'].append({'admission':'e'*64,'proposal':proposal,
                'current':True,'withdrawn':False,'superseded':False})
            body['messages'][0]['content']=m.canonical(history).decode()
            growth=len(json.dumps(body,ensure_ascii=False).encode())-base_bytes
            self.assertLessEqual(growth,2*4096+512)
            self.assertLessEqual(16290+growth,32768)

    def test_exact_wire_ceiling_and_no_default_upgrade_before_credentials(self):
        f=transport_test.Transport();body=dict(f.body(),model='gpt-6-luna')
        body['max_completion_tokens']=body.pop('max_tokens')
        body['messages'][0]['content']='é'*9000
        with patch.object(t.broker,'credential') as key:
            with self.assertRaisesRegex(ValueError,'payload_bound'):t.invoke(body,1,expected_model='gpt-6-luna')
            for cap in (True,65536):
                with self.assertRaises(ValueError):t.invoke(body,1,expected_model='gpt-6-luna',payload_limit=cap)
            key.assert_not_called()
        # New profile uses real UTF-8 JSON without changing the decoded content.
        envelope=dict(f.envelope(),model='gpt-6-luna')
        with patch.object(t.broker,'credential',return_value='fixture'),patch.object(t.urllib.request,'build_opener') as opener:
            opener.return_value.open.return_value=transport_test.Response(json.dumps(envelope).encode())
            result=t.invoke(body,1,expected_model='gpt-6-luna',payload_limit=32768)
            sent=opener.return_value.open.call_args.args[0].data
            self.assertEqual(json.loads(sent),body)
            self.assertEqual(sent,json.dumps(body,allow_nan=False,ensure_ascii=False).encode())
            self.assertEqual(result['receipt']['usd'],'0.001')
        body['messages'][0]['content']=''
        overhead=len(json.dumps(body,ensure_ascii=False).encode())
        body['messages'][0]['content']='x'*(32768-overhead)
        m.enforce_payload_bound(body,m.STAGES[-1],32768)
        body['messages'][0]['content']+='x'
        with self.assertRaises(m.PayloadBoundError) as err:m.enforce_payload_bound(body,m.STAGES[-1],32768)
        self.assertEqual(err.exception.actual,32769)
        with patch.object(t.broker,'credential') as key:
            with self.assertRaisesRegex(ValueError,'payload_bound'):t.invoke(body,1,expected_model='gpt-6-luna',payload_limit=32768)
            key.assert_not_called()

    def test_expanded_input_reservation_is_not_available_to_earlier_stages(self):
        f=base.ProjectEnvelope();f.setUp();self.addCleanup(f.doCleanups)
        fake=f.fake()
        with self.assertRaises(ValueError):m.fixture_step(f.journal,fake,lambda *_:self.fail('dispatch'),model='gpt-6-luna',payload_limit=32768)
        self.assertEqual(f.journal.status()['calls_consumed_or_reserved'],0)

class NativeAccounting(unittest.TestCase):
    def test_two_late_requests_reserve_and_settle_the_expanded_input_profile(self):
        import test_engine_b91_driver as dt
        f=dt.NativeSequence('test_luna_full_native_sequence');f.setUp();self.addCleanup(f.doCleanups)
        f.a=dt.m.admission(f.root,fixture=True,model='gpt-6-luna')
        provider=f.provider;step=dt.m.e.fixture_step
        def metered(body,timeout):
            result=provider(body,timeout)
            stage=json.loads(body['messages'][1]['content'])['stage']
            if stage in dt.m.e.STAGES[-2:]:
                result['receipt']['usage']={'prompt_tokens':25000,'completion_tokens':200,'total_tokens':25200}
            return result
        def profiled(journal,controller,transport,*,model):
            cap=32768 if journal.status()['next_stage'] in dt.m.e.STAGES[-2:] else 16384
            return step(journal,controller,transport,model=model,payload_limit=cap)
        f.provider=metered
        with patch.object(dt.m.e,'fixture_step',side_effect=profiled):
            driver=f.driver();result=driver.run()
        self.assertEqual(result['resources']['measured_prompt_tokens'],50400)
        self.assertEqual(result['resources']['tokens_charged'],51600)
        intents=[x for x in driver.journal().events() if x['kind']=='dispatch_intent']
        self.assertEqual([x['reservation']['input_tokens'] for x in intents],[16384]*4+[32768]*2)
        self.assertEqual(result['resources']['unknown_usage'],0)
        self.assertEqual(result['calls'],6)
        self.assertTrue(result['fixture_only'])

if __name__=='__main__':unittest.main()
