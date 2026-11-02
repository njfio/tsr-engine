"""Post-export corruption checks against inert fixtures only."""
import importlib.util,json,hashlib,sqlite3,subprocess,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
s=importlib.util.spec_from_file_location('a',Path(__file__).with_name('m3-smoke-audit.py'));a=importlib.util.module_from_spec(s);s.loader.exec_module(a)
CHECKER=a.ROOT/'target/m3-blake3-checksum'
def digest(data):return subprocess.check_output([str(CHECKER)],input=data).decode().strip()

class Audit(unittest.TestCase):
    def make(self,root):
        docs=root/'docs';docs.mkdir();(root/'target').mkdir();(root/'target/m3-smoke-comparison-state').mkdir();(root/'target/m3-smoke-comparison-state/cost-before.jsonl').write_bytes(b'');export=docs/'m3-smoke-execution';export.mkdir()
        cas=export/'fixture-input-cas';cas.mkdir();(cas/digest(b'')).write_bytes(b'')
        dbpath=export/'fixture-record-state.sqlite'
        with sqlite3.connect(dbpath) as db:db.execute('create table events(seq integer,id text)')
        row=dict(task='fixture',mode='record',result=dict(state_root_at_start=digest(b''),state_root_at_end=digest(b''),outcome='rejected'))
        result_path=export/'fixture-record-results.jsonl';result_path.write_text(json.dumps(row['result'])+'\n');row['sha256']=hashlib.sha256(result_path.read_bytes()).hexdigest()
        (export/'m3-provider-cost.jsonl').write_bytes(b'');(export/'state-cost-before.jsonl').write_bytes(b'')
        (docs/'m3-smoke-result.json').write_text(json.dumps(dict(arms=[row],spending=dict(prior_usd='0',cohort_usd='0',total_usd='0',outstanding_reserved_usd='0'))))
        (docs/'m3-smoke-budget-baseline.json').write_text(json.dumps(a.budget.freeze(b'')))
        (root/'target/m3-provider-cost.jsonl').write_bytes(b'')
        (docs/'m3-smoke-freeze-v3.json').write_text('{"sha256":{}}')
        self.manifest(docs,export)
        return docs,export
    def manifest(self,docs,export):
        (docs/'m3-smoke-export-sha256.json').write_text(json.dumps({str(p.relative_to(export)):hashlib.sha256(p.read_bytes()).hexdigest() for p in export.rglob('*') if p.is_file()}))
    def test_valid_fixture_and_corrupt_cas_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);docs,export=self.make(root)
            with patch.object(a,'ROOT',root),patch.object(a,'digest',side_effect=digest),patch('builtins.print'):a.main()
            next((export/'fixture-input-cas').iterdir()).write_bytes(b'changed');self.manifest(docs,export)
            with patch.object(a,'ROOT',root),patch.object(a,'digest',side_effect=digest),self.assertRaises(AssertionError):a.main()
    def test_M3_AUDIT_01_report_and_manifest_divergence_rejected(self):
        for change in ['cost','outcome','omission']:
            with tempfile.TemporaryDirectory() as d:
                root=Path(d);docs,export=self.make(root)
                report=json.loads((docs/'m3-smoke-result.json').read_text())
                if change=='cost':report['spending']['total_usd']='1'
                if change=='outcome':report['arms'][0]['result']['outcome']='accepted'
                if change=='omission':(export/'not-in-manifest').write_text('fixture')
                (docs/'m3-smoke-result.json').write_text(json.dumps(report))
                with patch.object(a,'ROOT',root),patch.object(a,'digest',side_effect=digest),self.assertRaises(AssertionError):a.main()
    def test_altered_event_id_head_rejected(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);docs,export=self.make(root)
            with sqlite3.connect(export/'fixture-record-state.sqlite') as db:db.execute("insert into events values(1,'changed')")
            self.manifest(docs,export)
            with patch.object(a,'ROOT',root),patch.object(a,'digest',side_effect=digest),self.assertRaises(AssertionError):a.main()

if __name__=='__main__':unittest.main()
