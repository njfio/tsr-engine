"""Offline quote/accounting cases; no provider access or ledger history edits."""
import copy
import io
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch
from test_engine_b91_campaign import m

class PriceQuotes(unittest.TestCase):
    def quote(self):
        return {'version':'b91-price-quote@v2','model':m.MODEL,'input_token_price_threshold':-1,'rates':{
            'input_per_million':'0.067472','output_per_million':'0.337358',
            'cache_read_per_million':'0.0134943022','cache_write_per_million':'0.1686787775'},
            'ceilings':dict(m.PRICE_CEILINGS),'catalog_sha256':'a'*64,'retrieved_at':time.time()}
    def test_capacity_price_and_reservation_use_actual_output_maximum(self):
        q=self.quote()
        expected=m.quote_estimate(q)+m.decimal.Decimal(24576)*m.decimal.Decimal(q['rates']['output_per_million'])
        self.assertEqual(m.quote_estimate(q,32768),expected)
        edge=self.quote();edge['rates']={k:'0' for k in m.PRICE_CEILINGS};edge['rates']['output_per_million']='2'
        m.validate_quote(edge)
        self.assertGreater(m.quote_estimate(edge,32768),m.RESERVE)
    def test_actual_four_rates_preflight_and_conservative_estimate(self):
        q=self.quote();raw=m.canonical({'id':m.MODEL,'input_token_price_threshold':-1,**q['rates']})
        with patch.object(m.urllib.request,'urlopen',return_value=io.BytesIO(raw)):
            got=m.official_preflight()
        self.assertEqual(got['rates'],q['rates']);self.assertEqual(got['catalog_sha256'],m.hashlib.sha256(raw).hexdigest())
        self.assertEqual(m.quote_estimate(got),m.decimal.Decimal('10944.0067076096'))
        self.assertLess(m.quote_estimate(got),m.RESERVE)
    def test_nonfinite_negative_missing_overceiling_and_wrong_model_fail(self):
        for key in m.PRICE_CEILINGS:
            for value in ('NaN','Infinity','-0.01',str(m.decimal.Decimal(m.PRICE_CEILINGS[key])+1)):
                q=self.quote();q['rates'][key]=value
                with self.subTest(key=key,value=value),self.assertRaises(ValueError):m.validate_quote(q)
        for field in ('rates','catalog_sha256','model','retrieved_at'):
            q=self.quote();q[field]=({} if field=='rates' else 'invalid')
            with self.assertRaises(ValueError):m.validate_quote(q)
    def test_reservation_derived_envelope_and_old_v2_ceiling_preservation(self):
        old=self.quote();old['ceilings']={'input_per_million':'0.068','output_per_million':'0.34','cache_read_per_million':'0.014','cache_write_per_million':'0.17'}
        raw=m.canonical(old);m.validate_quote(old);self.assertEqual(m.canonical(old),raw)
        moved=self.quote();moved['rates'].update(input_per_million='0.074044',output_per_million='0.370221',cache_read_per_million='0.0148088382',cache_write_per_million='0.1851104775')
        self.assertLess(m.quote_estimate(moved),m.RESERVE)
        too_much=self.quote();too_much['rates']=dict(m.PRICE_CEILINGS)
        with self.assertRaisesRegex(ValueError,'reservation'):m.validate_quote(too_much)
        edge=self.quote();edge['rates']={k:'0' for k in m.PRICE_CEILINGS};edge['rates']['input_per_million']=m.PRICE_CEILINGS['input_per_million']
        self.assertEqual(m.quote_estimate(edge),m.RESERVE)
        edge['rates']['output_per_million']='0.0000001'
        with self.assertRaisesRegex(ValueError,'reservation'):m.validate_quote(edge)
        old['rates']['input_per_million']='0.074044'
        with self.assertRaises(ValueError):m.validate_quote(old)

    def test_uncovered_tiered_catalog_fails(self):
        q=self.quote()
        for threshold in (None,0,32768,True):
            raw=m.canonical({'id':m.MODEL,'input_token_price_threshold':threshold,**q['rates']})
            with patch.object(m.urllib.request,'urlopen',return_value=io.BytesIO(raw)),self.assertRaisesRegex(ValueError,'tiered'):
                m.official_preflight()

    def test_legacy_history_preserved_new_dispatch_uses_fresh_v2(self):
        with tempfile.TemporaryDirectory() as temp:
            path=Path(temp)/'ledger';manifest=m.manifest('a'*64,path);ledger=m.Ledger(path,manifest,m.digest(manifest),create=True)
            ledger.enroll('b'*64,'c'*64)
            old={'model':m.MODEL,'rates':m.RATES,'retrieved_at':time.time(),'catalog_sha256':'d'*64}
            ledger.record_price(old);before=path.read_bytes()
            body={'model':m.MODEL,'max_completion_tokens':8192,'stream':False,'messages':[]}
            def invoke(*_):return {'receipt':{'status':'settled','usd':'0.001'}}
            with self.assertRaisesRegex(ValueError,'fresh official'):ledger.run_call('b'*64,'old',body,240,invoke)
            self.assertEqual(ledger.status()['calls'],0)
            q=self.quote();ledger.record_price(q);ledger.run_call('b'*64,'new',body,240,invoke)
            self.assertTrue(path.read_bytes().startswith(before));self.assertEqual(manifest['rates'],m.RATES)
            self.assertEqual(ledger.status()['known_cost_microusd'],1000)
            q['retrieved_at']=time.time()-3601;ledger.record_price(q)
            with self.assertRaisesRegex(ValueError,'fresh official'):ledger.run_call('b'*64,'stale',body,240,invoke)
            self.assertEqual(ledger.status()['calls'],1)
            m.Ledger(path,manifest,m.digest(manifest))

    def tiered_catalog(self):
        now=int(time.time()*1000)
        scope={'id':'authored-scope','version':'b'*64,'scope':dict(m.DEFAULT_PRICE_SCOPE),'effectiveAt':now-1000,'expiresAt':now+600000,'tiers':[
            {'id':'base','minContext':0,'maxContext':272001,'rates':{'input_tokens':'0.00000008','output_tokens':'0.0000004','cache_read_tokens':'0.000000008','cache_write_5m_tokens':'0.0000001'}},
            {'id':'large','minContext':272001,'maxContext':None,'rates':{'input_tokens':'0.00000016','output_tokens':'0.0000006','cache_read_tokens':'0.000000016','cache_write_5m_tokens':'0.0000002'}}]}
        return {'id':m.MODEL,'input_token_price_threshold':272000,**self.quote()['rates'],'default_pricing_scope':dict(m.DEFAULT_PRICE_SCOPE),'pricing_scopes':[scope]}
    def tiered_quote(self,catalog=None):
        raw=m.canonical(catalog or self.tiered_catalog())
        with patch.object(m.urllib.request,'urlopen',return_value=io.BytesIO(raw)):return m.official_preflight()
    def test_tiered_exact_scope_maximum_rates_and_dynamic_estimate(self):
        q=self.tiered_quote();self.assertEqual(q['version'],'b91-price-quote@v3');m.validate_quote(q)
        self.assertEqual(m.decimal.Decimal(q['rates']['input_per_million']),m.decimal.Decimal('.16'))
        for tokens in (272000,272001,m.capacity.INPUT_TOKENS):
            self.assertEqual(m.quote_estimate(q,32768,tokens),m.decimal.Decimal(tokens)*m.decimal.Decimal('.376')+m.decimal.Decimal(32768)*m.decimal.Decimal('.6'))
        self.assertGreater(m.quote_estimate(q,32768,m.capacity.INPUT_TOKENS),m.RESERVE)
        old=self.quote();raw=m.canonical(old);m.validate_quote(old);self.assertEqual(m.canonical(old),raw)
    def test_tiered_missing_ambiguous_scope_bounds_rates_and_expiry_rejected(self):
        changes=[lambda c:c['pricing_scopes'].append(copy.deepcopy(c['pricing_scopes'][0])),lambda c:c['default_pricing_scope'].update(zdr=True),lambda c:c['pricing_scopes'][0]['tiers'][1].update(minContext=272002),lambda c:c['pricing_scopes'][0].update(expiresAt=1)]
        for rate in ('NaN','-1','100'):
            changes.append(lambda c,rate=rate:c['pricing_scopes'][0]['tiers'][1]['rates'].update(input_tokens=rate))
        for change in changes:
            catalog=self.tiered_catalog();change(catalog)
            with self.assertRaises(ValueError):self.tiered_quote(catalog)
        q=self.tiered_quote();q['rates']['input_per_million']='0'
        with self.assertRaisesRegex(ValueError,'envelope'):m.validate_quote(q)
    def test_tiered_quote_append_only_dispatch_and_expiration(self):
        with tempfile.TemporaryDirectory() as temp:
            path=Path(temp)/'ledger';manifest=m.manifest('a'*64,path);ledger=m.Ledger(path,manifest,m.digest(manifest),create=True);ledger.enroll('b'*64,'c'*64)
            ledger.record_price(self.quote());before=path.read_bytes();q=self.tiered_quote();ledger.record_price(q)
            body={'model':m.MODEL,'max_completion_tokens':8192,'stream':False,'messages':[]}
            ledger.run_call('b'*64,'new',body,240,lambda *_:{'receipt':{'status':'settled','usd':'0.001'}})
            self.assertEqual(ledger.status()['calls'],1);self.assertTrue(path.read_bytes().startswith(before))
            for field in ('service_tier','serviceTier','zdr','region','pricing_variant'):
                unchanged=path.read_bytes()
                with self.assertRaisesRegex(ValueError,'scope mismatch'):ledger.run_call('b'*64,'override',{**body,field:'priority'},240,lambda *_:self.fail('override dispatch'),preflight_only=True)
                self.assertEqual(path.read_bytes(),unchanged)
            with patch.object(m.time,'time',return_value=q['pricing_scope']['expiresAt']/1000+1),self.assertRaises(ValueError):ledger.run_call('b'*64,'expired',body,240,lambda *_:self.fail('expired dispatch'))
            self.assertEqual(ledger.status()['calls'],1);m.Ledger(path,manifest,m.digest(manifest))

if __name__=='__main__':unittest.main()
