import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

spec = importlib.util.spec_from_file_location('inventory', Path(__file__).with_name('engine-evidence-inventory.py'))
mod = importlib.util.module_from_spec(spec)
spec.loader.exec_module(mod)

class EvidenceInventoryTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.task = {'id':'t','family':'f','lineage':'repo','timestamp':'2026-01-01'}
        self.receipt = {'task_id':'t','accepted':True,'function_group':'g','protocol':'baseline-full-green/named-red/full-red/exact-reversal/full-green-v1'}
        self.summary = {'attempts':[{'attempt':'run','strict_protocol':True,'reports':[self.receipt]}],
                        'admitted_production_tasks':1,'accepted_unique_task_ids':['t']}
        self.write('docs/m3-native-admission/run/tasks.jsonl', self.task)
        self.write('docs/m3-native-admission/run/admission.jsonl', self.receipt)
        self.write('docs/m3-native-admission/summary.json',self.summary)
        self.write('docs/m3-fixed12-result.json',{'completed_arms':1,'launched_arms':1,'complete_pairs':0})
        self.write('docs/m3-fixed12-independent-outcome-audit.json',{
            'frozen_mismatches':[],'assignments':[{'index':0,'task':'t','mode':'record','pair_index':0,'status':'decision-recorded','intent':True,
                'decision':{'final_oracle_completed':True,'outcome':'accepted'}}], 'limitations':['test fixture']})
    def write(self,path,value):
        p=self.root/path;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(value)+'\n')
    def test_qualification_and_oracle_are_not_mature_review_labels(self):
        before={str(p):p.read_bytes() for p in self.root.rglob('*') if p.is_file()}
        result=mod.inventory(self.root)
        self.assertEqual(result['qualifications']['unique_tasks'],1)
        self.assertEqual(len(result['fixed12']['final_oracle_arms']),1)
        self.assertIsNone(result['mature_common_event']['eligible_independent_episodes'])
        self.assertEqual(before,{str(p):p.read_bytes() for p in self.root.rglob('*') if p.is_file()})
        self.assertEqual(result,mod.inventory(self.root))
    def test_summary_disagreement_rejected(self):
        self.write('docs/m3-native-admission/run/admission.jsonl',dict(self.receipt,accepted=False))
        with self.assertRaisesRegex(ValueError,'disagrees'):mod.inventory(self.root)
    def test_retries_do_not_inflate_qualification(self):
        self.summary['attempts'].append(dict(self.summary['attempts'][0],attempt='retry'))
        self.write('docs/m3-native-admission/retry/tasks.jsonl',self.task)
        self.write('docs/m3-native-admission/retry/admission.jsonl',self.receipt)
        self.write('docs/m3-native-admission/summary.json',self.summary)
        result=mod.inventory(self.root)
        self.assertEqual(result['qualifications']['report_rows'],2)
        self.assertEqual(result['qualifications']['unique_tasks'],1)
    def test_inflated_aggregate_rejected(self):
        self.write('docs/m3-fixed12-result.json',{'completed_arms':2,'launched_arms':1,'complete_pairs':0})
        with self.assertRaisesRegex(ValueError,'aggregate'):mod.inventory(self.root)
    def test_wrong_admission_protocol_rejected(self):
        self.receipt['protocol']='unverified'
        self.write('docs/m3-native-admission/run/admission.jsonl',self.receipt)
        self.write('docs/m3-native-admission/summary.json',self.summary)
        with self.assertRaisesRegex(ValueError,'protocol'):mod.inventory(self.root)
    def test_path_escape_rejected(self):
        self.summary['attempts'][0]['attempt']='../escape'
        self.write('docs/m3-native-admission/summary.json',self.summary)
        with self.assertRaisesRegex(ValueError,'identity'):mod.inventory(self.root)

if __name__=='__main__':unittest.main()
