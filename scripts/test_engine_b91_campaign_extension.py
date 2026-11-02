"""Disposable accounting fixtures only; no live grant, provider or target calls."""
import importlib.util
from pathlib import Path
import tempfile
import unittest

spec=importlib.util.spec_from_file_location('extended_campaign',Path(__file__).with_name('engine-b91-campaign.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

class Extension(unittest.TestCase):
    def setUp(self):
        tmp=tempfile.TemporaryDirectory();self.addCleanup(tmp.cleanup);self.path=Path(tmp.name)/'ledger'
        self.manifest=m.manifest('a'*64,self.path,fixture=True)
        self.l=m.Ledger(self.path,self.manifest,m.digest(self.manifest),create=True)
        self.trial='b'*64;self.l.enroll(self.trial,'c'*64)
    def grant(self):
        with self.l.locked() as (_,rows):head=rows[-1]['hash']
        return {'version':'b91-call-extension@v1','manifest_sha256':m.digest(self.manifest),
                'ledger_prefix_head':head,'previous_max_calls':100,'max_calls':140,
                'max_execution_groups':40,'cap_microusd':m.CAP,'reservation_microusd':m.RESERVE,
                'authority_sha256':'d'*64,'reviewer':'fixture independent review'}
    def call(self,i,amount=0):
        key=self.l.reserve('call',self.trial,str(i),'e'*64);self.l.settle('bill',key,amount)
    def test_extension_preserves_manifest_prefix_costs_and_restart(self):
        for i in range(94):self.call(i,1)
        before=self.path.read_bytes();original=dict(self.manifest)
        with self.assertRaisesRegex(ValueError,'slots'):self.l.preflight_run(calls=14,execution_groups=4)
        g=self.grant();self.l.grant_calls(g,m.digest(g))
        self.assertTrue(self.path.read_bytes().startswith(before));self.assertEqual(self.manifest,original)
        self.l=m.Ledger(self.path,self.manifest,m.digest(self.manifest))
        ready=self.l.preflight_run(calls=14,execution_groups=4)
        self.assertFalse(ready['exclusive_reservation']);self.assertEqual(ready['conservative_allowance_microusd'],700000)
        self.assertEqual(ready['status']['remaining_calls'],46);self.assertEqual(ready['status']['known_cost_microusd'],94)
        for i in range(94,140):self.call(i)
        with self.assertRaisesRegex(ValueError,'limit'):self.call(140)
        self.assertEqual(self.l.status()['effective_max_calls'],140)
    def test_duplicate_wrong_prefix_manifest_or_changed_caps_rejected(self):
        g=self.grant()
        for field,value in [('ledger_prefix_head','f'*64),('manifest_sha256','f'*64),('max_calls',141),
                            ('previous_max_calls',99),('max_execution_groups',41),('cap_microusd',6000000),
                            ('reservation_microusd',0),('max_calls',True),('authority_sha256','')]:
            bad=dict(g);bad[field]=value
            with self.assertRaises(ValueError):self.l.grant_calls(bad,m.digest(bad))
        with self.assertRaises(ValueError):self.l.grant_calls(g,'f'*64)
        bad=dict(g,extra=True)
        with self.assertRaises(ValueError):self.l.grant_calls(bad,m.digest(bad))
        self.l.grant_calls(g,m.digest(g))
        with self.assertRaises(ValueError):self.l.grant_calls(g,m.digest(g))
        with self.assertRaises(ValueError):self.l.grant_calls(self.grant(),m.digest(self.grant()))
    def test_grant_does_not_clear_unknown_or_overrun(self):
        key=self.l.reserve('call',self.trial,'unknown','e'*64);self.l.settle('bill',key,None)
        g=self.grant();self.l.grant_calls(g,m.digest(g))
        self.assertEqual(self.l.status()['committed_microusd'],50000)
        with self.assertRaisesRegex(ValueError,'uncertain'):self.l.preflight_run(calls=14,execution_groups=4)
        with self.assertRaisesRegex(ValueError,'uncertain'):self.call('after')
    def test_whole_run_group_and_money_readiness_without_effect(self):
        g=self.grant();self.l.grant_calls(g,m.digest(g))
        for i in range(37):
            key=self.l.reserve('execution',self.trial,str(i),'e'*64);self.l.settle('return',key,'returned')
        before=self.path.read_bytes()
        with self.assertRaisesRegex(ValueError,'slots'):self.l.preflight_run(calls=14,execution_groups=4)
        self.assertEqual(self.path.read_bytes(),before)
        for i in range(87):self.call(i,50000)
        with self.assertRaisesRegex(ValueError,'cost allowance'):self.l.preflight_run(calls=14,execution_groups=3)
    def test_rehashed_grant_tamper_and_retroactive_extension_rejected(self):
        g=self.grant();self.l.grant_calls(g,m.digest(g))
        rows=[m.strict(x) for x in self.path.read_bytes().splitlines()];rows[-1]['event']['value']['max_execution_groups']=41
        rows[-1]['event']['key']=m.digest(rows[-1]['event']['value']);rows[-1]=self.l.row(rows[:-1],rows[-1]['event'])
        self.path.write_bytes(b'\n'.join(m.canonical(x) for x in rows)+b'\n')
        with self.assertRaises(ValueError):self.l.status()

class EvaluationExtension(Extension):
    def evaluation_grant(self):
        with self.l.locked() as (_,rows):head=rows[-1]['hash']
        return {'version':'b91-evaluation-extension@v1','manifest_sha256':m.digest(self.manifest),
                'ledger_prefix_head':head,'previous_max_calls':140,'max_calls':200,
                'previous_max_execution_groups':40,'max_execution_groups':60,
                'cap_microusd':m.CAP,'reservation_microusd':m.RESERVE,
                'authority_sha256':'1'*64,'reviewer':'fixture independent review',
                'suite_manifest_sha256':'2'*64,'source_class':'newly-authored-nonprivate-rust-repair-suite'}
    def extend_first(self):
        g=self.grant();self.l.grant_calls(g,m.digest(g))
    def test_inquiry_extension_exact_caps_preserved_history_and_no_replay(self):
        self.extend_first();g=self.evaluation_grant();self.l.grant_calls(g,m.digest(g))
        self.call('existing',7)
        before=self.path.read_bytes()
        with self.l.locked() as (_,rows):head=rows[-1]['hash']
        g={'version':'b91-inquiry-call-extension@v1','manifest_sha256':m.digest(self.manifest),'ledger_prefix_head':head,'previous_max_calls':200,'max_calls':202,'max_execution_groups':60,'cap_microusd':m.CAP,'reservation_microusd':m.RESERVE,'authority_sha256':'d'*64,'reviewer':'fixture review'}
        for key,value in [('max_calls',203),('max_execution_groups',61),('cap_microusd',6000000),('ledger_prefix_head','f'*64),('previous_max_calls',100)]:
            bad={**g,key:value}
            with self.assertRaises(ValueError):self.l.grant_calls(bad,m.digest(bad))
        self.l.grant_calls(g,m.digest(g));self.assertTrue(self.path.read_bytes().startswith(before))
        restored=m.Ledger(self.path,self.manifest,m.digest(self.manifest));status=restored.status()
        self.assertEqual((status['calls'],status['known_cost_microusd'],status['effective_max_calls'],status['effective_max_execution_groups']),(1,7,202,60))
        with self.assertRaises(ValueError):restored.grant_calls(g,m.digest(g))
    def test_second_inquiry_extension_exact_and_append_only(self):
        self.test_inquiry_extension_exact_caps_preserved_history_and_no_replay()
        before=self.path.read_bytes()
        with self.l.locked() as (_,rows):head=rows[-1]['hash']
        g={'version':'b91-inquiry-call-extension@v2','manifest_sha256':m.digest(self.manifest),'ledger_prefix_head':head,'previous_max_calls':202,'max_calls':220,'max_execution_groups':60,'cap_microusd':m.CAP,'reservation_microusd':m.RESERVE,'authority_sha256':'d'*64,'reviewer':'fixture review'}
        for key,value in [('max_calls',221),('max_execution_groups',61),('cap_microusd',6000000),('previous_max_calls',200)]:
            bad={**g,key:value}
            with self.assertRaises(ValueError):self.l.grant_calls(bad,m.digest(bad))
        self.l.grant_calls(g,m.digest(g));self.assertTrue(self.path.read_bytes().startswith(before))
        restored=m.Ledger(self.path,self.manifest,m.digest(self.manifest));status=restored.status()
        self.assertEqual((status['calls'],status['known_cost_microusd'],status['effective_max_calls'],status['effective_max_execution_groups']),(1,7,220,60))
        with self.assertRaises(ValueError):restored.grant_calls(g,m.digest(g))
    def test_prospective_objective_extension_full_dispatch_chain(self):
        # Authored disposable ledger only. Adding a validator is not applying a live grant.
        self.test_second_inquiry_extension_exact_and_append_only()
        for version,old,new,old_groups,new_groups,scope in [
            ('native-pursuit-extension@v1',220,228,60,62,'newly-authored-nonprivate-native-pursuit'),
            ('native-pursuit-extension@v2',228,230,62,62,'newly-authored-nonprivate-native-pursuit'),
            ('native-information-extension@v1',230,238,62,64,'newly-authored-nonprivate-native-information-pursuit'),
            ('native-information-extension@v2',238,239,64,64,'newly-authored-nonprivate-native-information-pursuit'),
            ('native-objective-extension@v1',239,250,64,67,'newly-authored-nonprivate-finite-service-objective')]:
            with self.l.locked() as (_,rows):head=rows[-1]['hash']
            g={**self.evaluation_grant(),'version':version,'ledger_prefix_head':head,
               'previous_max_calls':old,'max_calls':new,
               'previous_max_execution_groups':old_groups,'max_execution_groups':new_groups,
               'source_class':scope}
            before=self.path.read_bytes()
            for key,value in [('max_calls',new+1),('source_class','private-source'),
                              ('max_execution_groups',new_groups+1),('ledger_prefix_head','f'*64)]:
                bad={**g,key:value}
                with self.assertRaises(ValueError):self.l.grant_calls(bad,m.digest(bad))
            self.assertEqual(before,self.path.read_bytes())
            self.l.grant_calls(g,m.digest(g))
            self.assertTrue(self.path.read_bytes().startswith(before))
            self.l=m.Ledger(self.path,self.manifest,m.digest(self.manifest))
            self.assertEqual((self.l.status()['effective_max_calls'],self.l.status()['effective_max_execution_groups']),(new,new_groups))
            with self.assertRaises(ValueError):self.l.grant_calls(g,m.digest(g))
        self.assertEqual(self.l.status()['known_cost_microusd'],7)
        self.assertEqual(self.l.preflight_run(calls=24,execution_groups=5)['conservative_allowance_microusd'],1200000)

    def test_second_objective_extension_exact_append_only(self):
        self.test_prospective_objective_extension_full_dispatch_chain()
        with self.l.locked() as (_,rows):head=rows[-1]['hash']
        g={**self.evaluation_grant(),'version':'native-objective-extension@v2','ledger_prefix_head':head,
           'previous_max_calls':250,'max_calls':252,'previous_max_execution_groups':67,'max_execution_groups':68,
           'source_class':'newly-authored-nonprivate-finite-service-objective'}
        before=self.path.read_bytes()
        for k,v in [('max_calls',253),('max_execution_groups',69),('cap_microusd',6000000),('source_class','private-source'),('ledger_prefix_head','f'*64)]:
            bad={**g,k:v}
            with self.assertRaises(ValueError):self.l.grant_calls(bad,m.digest(bad))
        self.assertEqual(self.path.read_bytes(),before)
        self.l.grant_calls(g,m.digest(g));self.assertTrue(self.path.read_bytes().startswith(before))
        self.l=m.Ledger(self.path,self.manifest,m.digest(self.manifest))
        self.assertEqual((self.l.status()['effective_max_calls'],self.l.status()['effective_max_execution_groups'],self.l.status()['known_cost_microusd']),(252,68,7))
        with self.assertRaises(ValueError):self.l.grant_calls(g,m.digest(g))

    def test_third_objective_extension_is_only_prospective_and_exact(self):
        self.test_second_objective_extension_exact_append_only()
        with self.l.locked() as (_,rows):head=rows[-1]['hash']
        g={**self.evaluation_grant(),'version':'native-objective-extension@v3','ledger_prefix_head':head,
           'previous_max_calls':252,'max_calls':258,'previous_max_execution_groups':68,'max_execution_groups':70,
           'source_class':'newly-authored-nonprivate-finite-service-objective'}
        before=self.path.read_bytes()
        m.validate_evaluation_grant(g,m.digest(self.manifest),head,252,68)
        self.assertEqual(self.path.read_bytes(),before)
        for key,value in [('max_calls',259),('max_execution_groups',71),('cap_microusd',6000000),('authority_sha256',None),('ledger_prefix_head','f'*64)]:
            bad={**g,key:value}
            with self.assertRaises(ValueError):self.l.grant_calls(bad,m.digest(bad))
        self.assertEqual(self.path.read_bytes(),before)
        self.l.grant_calls(g,m.digest(g))
        status=m.Ledger(self.path,self.manifest,m.digest(self.manifest)).status()
        self.assertEqual((status['effective_max_calls'],status['effective_max_execution_groups'],status['known_cost_microusd']),(258,70,7))
        with self.assertRaises(ValueError):self.l.grant_calls(g,m.digest(g))

    def test_open_guide_extension_remains_explicit_and_append_only(self):
        self.test_third_objective_extension_is_only_prospective_and_exact()
        with self.l.locked() as (_,rows):head=rows[-1]['hash']
        g={**self.evaluation_grant(),'version':'native-open-guide-extension@v1','ledger_prefix_head':head,
           'previous_max_calls':258,'max_calls':272,'previous_max_execution_groups':70,'max_execution_groups':74,
           'source_class':'newly-authored-nonprivate-open-guide-objective'}
        before=self.path.read_bytes()
        m.validate_evaluation_grant(g,m.digest(self.manifest),head,258,70)
        self.assertEqual(self.path.read_bytes(),before)
        for key,value in [('max_calls',273),('max_execution_groups',75),('cap_microusd',6000000),('authority_sha256',None),('ledger_prefix_head','f'*64),('source_class','newly-authored-nonprivate-finite-service-objective')]:
            with self.assertRaises(ValueError):self.l.grant_calls({**g,key:value},m.digest({**g,key:value}))
        self.assertEqual(self.path.read_bytes(),before)
        self.l.grant_calls(g,m.digest(g))
        status=m.Ledger(self.path,self.manifest,m.digest(self.manifest)).status()
        self.assertEqual((status['effective_max_calls'],status['effective_max_execution_groups'],status['known_cost_microusd']),(272,74,7))
        self.assertTrue(self.path.read_bytes().startswith(before))
        with self.assertRaises(ValueError):self.l.grant_calls(g,m.digest(g))

    def test_opportunity_extension_preserves_unknown_and_requires_exact_scope(self):
        self.test_open_guide_extension_remains_explicit_and_append_only()
        key=self.l.reserve('call',self.trial,'authored-unknown','e'*64);self.l.settle('bill',key,None)
        with self.l.locked() as (_,rows):head=rows[-1]['hash']
        g={**self.evaluation_grant(),'version':'native-open-guide-extension@v2','ledger_prefix_head':head,
           'previous_max_calls':272,'max_calls':283,'previous_max_execution_groups':74,'max_execution_groups':77,
           'source_class':'newly-authored-nonprivate-open-guide-objective'}
        before=self.path.read_bytes();status=self.l.status()
        for field,value in [('max_calls',284),('max_execution_groups',78),('previous_max_calls',258),('ledger_prefix_head','f'*64),('authority_sha256',None),('source_class','private-data')]:
            bad={**g,field:value}
            with self.assertRaises(ValueError):self.l.grant_calls(bad,m.digest(bad))
        self.assertEqual(self.path.read_bytes(),before)
        self.l.grant_calls(g,m.digest(g));restored=m.Ledger(self.path,self.manifest,m.digest(self.manifest))
        self.assertTrue(self.path.read_bytes().startswith(before));now=restored.status()
        self.assertEqual((now['effective_max_calls'],now['effective_max_execution_groups']),(283,77))
        for field in ('calls','execution_groups','known_cost_microusd','committed_microusd'):self.assertEqual(now[field],status[field])
        with self.assertRaisesRegex(ValueError,'uncertain'):restored.preflight_run(calls=24,execution_groups=5)
        with self.assertRaises(ValueError):restored.grant_calls(g,m.digest(g))

    def test_evaluation_preserves_history_and_enforces_new_totals_on_restart(self):
        self.extend_first()
        for i in range(133):self.call(i,1)
        for i in range(40):
            k=self.l.reserve('execution',self.trial,str(i),'e'*64);self.l.settle('return',k,'returned')
        before=self.path.read_bytes();original=dict(self.manifest)
        with self.assertRaisesRegex(ValueError,'slots'):self.l.preflight_run(calls=42,execution_groups=12)
        g=self.evaluation_grant();self.l.grant_calls(g,m.digest(g))
        self.assertTrue(self.path.read_bytes().startswith(before));self.assertEqual(self.manifest,original)
        self.l=m.Ledger(self.path,self.manifest,m.digest(self.manifest))
        status=self.l.preflight_run(calls=42,execution_groups=12)['status']
        self.assertEqual((status['remaining_calls'],status['remaining_execution_groups'],status['known_cost_microusd']),(67,20,133))
        self.assertEqual(status['evaluation_suite_sha256'],'2'*64)
        for i in range(133,200):self.call(i)
        with self.assertRaisesRegex(ValueError,'limit'):self.call(200)
        for i in range(40,60):
            k=self.l.reserve('execution',self.trial,str(i),'e'*64);self.l.settle('return',k,'returned')
        with self.assertRaisesRegex(ValueError,'limit'):self.l.reserve('execution',self.trial,'60','e'*64)
    def test_evaluation_requires_exact_prior_grant_scope_caps_and_prefix(self):
        g=self.evaluation_grant()
        with self.assertRaises(ValueError):self.l.grant_calls(g,m.digest(g))
        self.extend_first();g=self.evaluation_grant()
        for field,value in [('ledger_prefix_head','f'*64),('manifest_sha256','f'*64),('max_calls',201),
                            ('previous_max_calls',100),('max_execution_groups',61),('previous_max_execution_groups',39),
                            ('cap_microusd',6000000),('reservation_microusd',0),('max_calls',True),
                            ('suite_manifest_sha256',''),('source_class','private-source'),('extra',True)]:
            bad=dict(g);bad[field]=value
            with self.assertRaises(ValueError):self.l.grant_calls(bad,m.digest(bad))
        self.l.grant_calls(g,m.digest(g))
        with self.assertRaises(ValueError):self.l.grant_calls(g,m.digest(g))
        bad=self.evaluation_grant()
        with self.assertRaises(ValueError):self.l.grant_calls(bad,m.digest(bad))
    def test_suite_enrollment_requires_prior_matching_grant_and_preserves_binding(self):
        with self.assertRaisesRegex(ValueError,'suite'):self.l.enroll('3'*64,'4'*64,suite_sha256='2'*64)
        self.extend_first();g=self.evaluation_grant();self.l.grant_calls(g,m.digest(g))
        with self.assertRaisesRegex(ValueError,'suite'):self.l.enroll('3'*64,'4'*64,suite_sha256='5'*64)
        self.l.enroll('3'*64,'4'*64,suite_sha256='2'*64)
        before=self.path.read_bytes();self.l.enroll('3'*64,'4'*64,suite_sha256='2'*64)
        self.assertEqual(self.path.read_bytes(),before)
        with self.assertRaisesRegex(ValueError,'changed'):self.l.enroll('3'*64,'4'*64)
        with self.assertRaisesRegex(ValueError,'changed'):self.l.enroll('3'*64,'4'*64,suite_sha256='5'*64)
    def test_rehashed_second_grant_cannot_change_scope_or_retroactively_cover_execution(self):
        self.extend_first();g=self.evaluation_grant();self.l.grant_calls(g,m.digest(g))
        rows=[m.strict(x) for x in self.path.read_bytes().splitlines()]
        rows[-1]['event']['value']['source_class']='private-source'
        rows[-1]['event']['key']=m.digest(rows[-1]['event']['value'])
        rows[-1]=self.l.row(rows[:-1],rows[-1]['event'])
        with self.assertRaisesRegex(ValueError,'scope'):self.l.fold(rows)
        # Even an internally rehashed history must obey the old execution ceiling
        # before the second grant: later authority cannot retroactively cover it.
        rows=rows[:-1]
        for i in range(41):
            event={'kind':'execution','key':m.digest({'fixture_execution':i}),
                   'trial':self.trial,'binding':'e'*64}
            rows.append(self.l.row(rows,event))
        with self.assertRaisesRegex(ValueError,'count exceeded'):self.l.fold(rows)
    def test_evaluation_does_not_clear_unknown_call_or_execution(self):
        self.extend_first();key=self.l.reserve('call',self.trial,'unknown','e'*64);self.l.settle('bill',key,None)
        g=self.evaluation_grant();self.l.grant_calls(g,m.digest(g))
        self.assertEqual(self.l.status()['committed_microusd'],50000)
        with self.assertRaisesRegex(ValueError,'uncertain'):self.l.preflight_run(calls=42,execution_groups=12)

if __name__=='__main__':unittest.main()
