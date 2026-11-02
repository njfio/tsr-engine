import json
import importlib.util
from pathlib import Path
import tempfile
import unittest
s=importlib.util.spec_from_file_location('local_packet',Path(__file__).with_name('engine-complementarity-local.py'));m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class Packet(unittest.TestCase):
    def test_unchanged_and_ablation(self):
        self.assertEqual(m.canonical(m.project('AB')),m.canonical(m.project('AB-repeat')))
        common=[]
        for a in ('A','B','AB'):
            x=m.project(a);x.pop('active_parents');x.pop('parents');common.append(x)
        self.assertEqual(common[0],common[1]);self.assertEqual(common[1],common[2])
        self.assertEqual([p['id'] for p in m.project('A')['parents']],['A'])
        self.assertEqual([p['id'] for p in m.project('B')['parents']],['B'])
    def test_withdrawal_preserves_meaning_and_records(self):
        x=m.project('withdraw-B');y=m.project('AB')
        self.assertEqual(x['active_parents'],['A']);self.assertEqual(x['withdrawn_parents'],['B'])
        self.assertEqual(x['parents'],y['parents']);self.assertEqual(x['records'],y['records']);self.assertEqual(x['active_guidance'],[])
    def test_no_authority_or_generated_evidence(self):
        for a in ('A','B','AB','withdraw-B'):
            x=m.project(a);self.assertEqual(x['authority'],{'export':False,'calls':0,'executions':0,'adoption':False});self.assertEqual(x['generated_guidance'],[])
            self.assertTrue(all(r['provenance']['kind']=='newly_authored_fiction' for r in x['records']))
            self.assertEqual(x['observations'][1]['kind'],'unknown')
    def test_source_injection_and_stale_source_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            p=Path(d)/'source.json'
            for changed in (b'{}',m.canonical(dict(m.source(),private_notes='not allowed')),m.canonical(dict(m.source(),authority={'calls':1}))):
                p.write_bytes(changed)
                with self.assertRaisesRegex(ValueError,'source changed'):m.source(p)
    def test_replay_and_projection_tampering(self):
        before={p.name:p.read_bytes() for p in m.PACKET.glob('*.json')};self.assertEqual(m.check()['dispatches'],0)
        self.assertEqual(before,{p.name:p.read_bytes() for p in m.PACKET.glob('*.json')})
        with tempfile.TemporaryDirectory() as d:
            old=m.PACKET;m.PACKET=Path(d)
            try:
                for name,data in before.items():(m.PACKET/name).write_bytes(data)
                manifest=json.loads((m.PACKET/'manifest.json').read_text());manifest['status']='admitted'
                (m.PACKET/'manifest.json').write_text(json.dumps(manifest))
                with self.assertRaisesRegex(ValueError,'manifest drift'):m.check()
                (m.PACKET/'manifest.json').write_bytes(before['manifest.json'])
                (m.PACKET/'AB.json').write_bytes(b'{}')
                with self.assertRaisesRegex(ValueError,'projection drift'):m.check()
            finally:m.PACKET=old
    def test_unknown_arm(self):
        with self.assertRaises(ValueError):m.project('execute')
if __name__=='__main__':unittest.main()
