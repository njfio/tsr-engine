import json
from pathlib import Path
import tempfile
import unittest
from m3_export_guard import terminal_attempt, retain_history

class ExportGuardTests(unittest.TestCase):
    def test_running_empty_and_failed_receipts_do_not_export(self):
        with tempfile.TemporaryDirectory() as tmp:
            r=Path(tmp);source=r/'target'/'attempt';source.mkdir(parents=True)
            state=r/'target'/'cohort-state';state.mkdir()
            for status,code,errors,expected in [('running',None,[],False),('completed',1,[],False),('completed',0,['cleanup'],False),('completed',0,[],True)]:
                (state/'job.json').write_text(json.dumps(dict(argv=['qualify','--out',str(source)],status=status,exit=code,errors=errors)))
                self.assertEqual(terminal_attempt(source,r/'target',r/'export'),expected)
    def test_unregistered_legacy_cas_is_never_recopied(self):
        with tempfile.TemporaryDirectory() as tmp:
            r=Path(tmp);source=r/'target'/'attempt';source.mkdir(parents=True)
            old=r/'export'/'attempt';old.mkdir(parents=True)
            self.assertFalse(terminal_attempt(source,r/'target',r/'export'))
            for n in ['tasks.jsonl','admission.jsonl']:
                (source/n).write_text('completed');(old/n).write_text('completed')
            (source/'cas').mkdir();(old/'cas').mkdir()
            (source/'cas'/'digest').write_text('changed');(old/'cas'/'digest').write_text('preserved')
            self.assertFalse(terminal_attempt(source,r/'target',r/'export'))
            (source/'admission.jsonl').write_text('changed')
            self.assertFalse(terminal_attempt(source,r/'target',r/'export'))

    def test_missing_target_keeps_historical_attempt(self):
        with tempfile.TemporaryDirectory() as tmp:
            r=Path(tmp);old=r/'export'/'gone';old.mkdir(parents=True)
            (old/'admission.jsonl').write_text('preserved')
            history={'gone': {'strict_protocol': True}}
            self.assertEqual(retain_history([],history), [('gone',True)])
            self.assertFalse(terminal_attempt(r/'target'/'gone',r/'target',r/'export'))
            self.assertEqual((old/'admission.jsonl').read_text(),'preserved')

if __name__=='__main__': unittest.main()
