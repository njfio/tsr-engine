"""Negative full-stream fold checks; canonical integrity is separately checked by Rust."""
import json,tempfile,unittest
from pathlib import Path
from replay import replay
SOURCE=Path(__file__).resolve().parents[2]/'docs/m3-inspection-handle-replay-evidence/record'
class Stream(unittest.TestCase):
    def check(self,change):
        events=json.loads((SOURCE/'events.json').read_text());rows=[json.loads(l) for l in (SOURCE/'boundary.jsonl').read_text().splitlines()]
        change(events,rows)
        with tempfile.TemporaryDirectory(prefix='m3-replay-stream-') as tmp:
            d=Path(tmp);(d/'events.json').write_text(json.dumps(events));(d/'boundary.jsonl').write_text('\n'.join(json.dumps(r) for r in rows))
            with self.assertRaises(AssertionError):replay(d)
    def test_stale_edit_after_successful_edit(self):
        def change(e,r):r[-1]['proposal']=r[2]['proposal'].copy()
        self.check(change)
    def test_missing_checkpoint(self):
        def change(e,r):
            index=next(i for i,x in enumerate(e) if 'broker/source-checkpoint@v1:' in json.dumps(x));e.pop(index)
        self.check(change)
    def test_request_state_mismatch(self):
        self.check(lambda e,r:r[2].update(source_state='different'))
    def test_advertised_old_handle_after_edit(self):
        self.check(lambda e,r:r[-1].update(inspection=r[2]['inspection']))
if __name__=='__main__':unittest.main()
