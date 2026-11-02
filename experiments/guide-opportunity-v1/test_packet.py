"""Authored local data/transport checks; never execute a service or call a provider."""
import copy, importlib.util, json, unittest
from pathlib import Path
from unittest.mock import patch
from oracle import assess, condition
P=Path(__file__).resolve().parent
ROOT=P.parents[1]
s=importlib.util.spec_from_file_location('guide_adapter',ROOT/'scripts/engine-objective-adapter.py');m=importlib.util.module_from_spec(s);s.loader.exec_module(m)

class Packet(unittest.TestCase):
    def test_multiple_valid_organizations_and_diagnostic_differences(self):
        c='6|11:1,12:1,21:2,22:2,31:3,32:3|11>12,11>21,21>31'
        cycle='11>12,12>21,21>22,22>31,31>32,32>11\n'
        direct='11>12,11>21,21>31\n'
        a,b=assess(c,'',cycle),assess(c,'',direct)
        self.assertTrue(a['minimum_obligations_met']);self.assertTrue(b['minimum_obligations_met'])
        self.assertNotEqual(a['diagnostics'],b['diagnostics'])
        self.assertNotIn('score',a)
    def test_separate_access_vectors_no_joint_score_or_forced_change(self):
        c='6|11:1,12:1,21:2,22:2,31:3,32:3|11>12,11>21,21>31'
        cycle='11>12,12>21,21>22,22>31,31>32,32>11\n'
        alternative='11>12,11>21,21>31\n'
        a,b=[assess(c,'',g) for g in (cycle,alternative)]
        self.assertTrue(a['minimum_obligations_met']);self.assertTrue(b['minimum_obligations_met'])
        self.assertEqual(a['diagnostics']['orientation_within_two_links'],b['diagnostics']['orientation_within_two_links'])
        self.assertNotEqual(a['diagnostics']['unfamiliar_access_opportunities'],b['diagnostics']['unfamiliar_access_opportunities'])
        self.assertEqual(a,assess(c,'',cycle))
        self.assertNotIn('reward',a);self.assertNotIn('score',a)
        # Categories denote fictional difference, not quality. Relabeling them
        # leaves route validity/orientation intact while access facts can change.
        same=assess(c.replace('21:2,22:2,31:3,32:3','21:1,22:1,31:1,32:1'),'',cycle)
        self.assertEqual(a['facts'],same['facts'])
        self.assertTrue(all(not x['unfamiliar_within_two_links'] for x in same['diagnostics']['unfamiliar_access_opportunities']))
    def test_invalid_graph_is_unassessable_not_zero_opportunity(self):
        result=assess('3|1:1,2:2,3:3|1>3','','1>99\n')
        self.assertFalse(result['minimum_obligations_met'])
        self.assertIsNone(result['diagnostics']['orientation_within_two_links'][0]['within_window'])
        self.assertIsNone(result['diagnostics']['unfamiliar_access_opportunities'][0]['unfamiliar_within_two_links'])
    def test_valid_robust_retention_and_all_conditions_fixed(self):
        previous=''
        for r in json.loads((P/'conditions.json').read_text())['rounds']:
            _,nodes,_=condition(r['body']);ids=list(nodes)
            out=','.join(f'{a}>{ids[(i+1)%len(ids)]}' for i,a in enumerate(ids))+'\n'
            self.assertTrue(assess(r['body'],previous,out)['minimum_obligations_met'])
            previous=out
    def test_failures_not_hidden_by_return_status(self):
        c='3|1:1,2:2,3:3|1>3'
        for raw in ('1>2\n','1>1,1>3\n','1>3,1>3\n','1>4,4>3\n'):
            self.assertFalse(assess(c,'',raw)['minimum_obligations_met'])
        for raw in ('PASS','1>3\nextra','1>4294967296','1>3\n\n'):
            with self.assertRaises(ValueError):assess(c,'',raw)
    def test_unchanged_input_and_previous_output_limit(self):
        c='3|1:1,2:2,3:3|1>3';out='1>2,2>3\n'
        self.assertEqual(assess(c,'',out),assess(c,'',out))
        a=assess(c,out,out);b=assess(c,'',out)
        self.assertEqual(a['facts'],b['facts'])
        self.assertNotEqual(a['diagnostics']['retained_available_edges'],b['diagnostics']['retained_available_edges'])
        with self.assertRaises(ValueError):assess(c,'invented prior output',out)
    def test_packet_source_consistency_and_limits(self):
        spec=json.loads((P/'project-spec.json').read_text())
        self.assertEqual(spec['editable'],{'lib.rs':(P/'lib.rs').read_text()})
        self.assertEqual(spec['protected'],{'service.rs':(P/'service.rs').read_text()})
        self.assertLess(len((P/'lib.rs').read_bytes()),2048)
        for r in json.loads((P/'conditions.json').read_text())['rounds']:
            self.assertLess(len(r['body'].encode()),2048);condition(r['body'])
    def test_explicit_profile_and_legacy_identity(self):
        legacy={'receipt_checker_sha256':m.d.file_sha(ROOT/'experiments/objective-service-v1/oracle.py')}
        self.assertEqual(m.receipt_checker(legacy)[1],'finite_selection_valid')
        fresh={'receipt_checker_profile':'guide-opportunity@v1','receipt_checker_sha256':m.d.file_sha(P/'oracle.py')}
        self.assertEqual(m.receipt_checker(fresh),(P/'oracle.py','minimum_obligations_met'))
        for bad in ({**fresh,'receipt_checker_profile':'../../private.py'},{**fresh,'receipt_checker_sha256':'0'*64}):
            with self.assertRaises(ValueError):m.receipt_checker(bad)
        with patch.object(Path,'is_symlink',return_value=True):
            with self.assertRaises(ValueError):m.receipt_checker(fresh)
    def test_actual_adapter_assesses_only_bound_data(self):
        a=object.__new__(m.Adapter)
        a.admission={'origin':{'receipt_checker_profile':'guide-opportunity@v1','receipt_checker_sha256':m.d.file_sha(P/'oracle.py')}}
        r={'receipt':'authored-receipt','source_snapshot':'authored-source','condition':{'input':'3|1:1,2:2,3:3|1>3'},'previous_output':'','output':'1>2,2>3\n','complete':True,'exit':0}
        result=a.receipt_assessment(r)
        self.assertTrue(result['assessment']['minimum_obligations_met'])
        self.assertEqual(result['input_sha256'],m.e.sha(r))
        self.assertNotEqual(result['input_sha256'],a.receipt_assessment({**r,'receipt':'other'})['input_sha256'])
        self.assertFalse(a.receipt_assessment({**r,'complete':False})['assessment']['minimum_obligations_met'])
        self.assertFalse(a.receipt_assessment({**r,'output':'PASS'})['assessment']['minimum_obligations_met'])
        # Existing receipt semantics never equate invocation success to correctness.
        self.assertFalse(a.receipt_assessment({**r,'output':'1>2\n'})['assessment']['minimum_obligations_met'])

if __name__=='__main__':unittest.main()
