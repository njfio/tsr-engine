"""Valid but undispatched proposal is ordinary only with classified terminal evidence."""
import json,tempfile,unittest
from pathlib import Path
from replay import replay
SOURCE=Path(__file__).resolve().parents[2]/'docs/m3-inspection-handle-replay-evidence/record'
class Tests(unittest.TestCase):
    def fixture(self,directory):
        events=json.loads((SOURCE/'events.json').read_text());rows=[json.loads(l) for l in (SOURCE/'boundary.jsonl').read_text().splitlines()]
        # Keep inspect/test, then a fresh edit whose span was rejected before Proposed.
        position=next(i for i,e in enumerate(events) if e['kind']['kind']=='proposed' and e['kind']['candidate']['kind']=='propose_patch')
        (directory/'events.json').write_text(json.dumps(events[:position]))
        (directory/'boundary.jsonl').write_text('\n'.join(json.dumps(r) for r in rows[:3]))
    def test_classified_span_failure(self):
        with tempfile.TemporaryDirectory() as temp:
            d=Path(temp);self.fixture(d)
            self.assertEqual(replay(d,{'failure':'edit-precondition'})['fresh_edits'],0)
            with self.assertRaises(AssertionError):replay(d)
    def test_unknown_reason_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            d=Path(temp);self.fixture(d)
            with self.assertRaises(AssertionError):replay(d,{'failure':'anything'})
    def test_multiple_undispatched_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            d=Path(temp);self.fixture(d);p=d/'boundary.jsonl';rows=[json.loads(l) for l in p.read_text().splitlines()];rows.append(rows[-1]|{'call':3});p.write_text('\n'.join(json.dumps(r) for r in rows))
            with self.assertRaises(AssertionError):replay(d,{'failure':'edit-precondition'})
if __name__=='__main__':unittest.main()
