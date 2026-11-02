"""Authored ledger-only polling tests. No live ledger, worker or provider access."""
import copy,fcntl,json,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
import server as ui

class Snapshot(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.addCleanup(self.tmp.cleanup)
  self.path=Path(self.tmp.name)/'ledger';self.m=ui.load('snapshot_fixture_campaign','engine-b91-campaign.py')
  self.man=self.m.manifest('a'*64,self.path,fixture=True);self.ledger=self.m.Ledger(self.path,self.man,self.m.digest(self.man),create=True)
 def read(self):return ui.accounting_snapshot(self.m,self.man,None)
 def test_poll_does_not_lock_even_while_writer_holds_exclusive_lock(self):
  before=self.path.read_bytes()
  with self.path.open('r+') as writer:
   fcntl.flock(writer,fcntl.LOCK_EX|fcntl.LOCK_NB)
   with patch.object(fcntl,'flock',side_effect=AssertionError('read must not flock')),patch.object(self.m.Ledger,'__init__',side_effect=AssertionError('no locking constructor')):
    result=self.read()
  self.assertEqual(self.path.read_bytes(),before);self.assertEqual(result['calls'],0);self.assertFalse(result['append_in_progress']);self.assertIn('not_dispatch_authority',result['projection'])
 def test_partial_append_is_explicit_and_cannot_authorize(self):
  original=self.read()
  with self.path.open('ab') as f:f.write(b'{"sequence":1,"partial":')
  value=self.read();self.assertTrue(value['append_in_progress']);self.assertTrue(value['stopped']);self.assertEqual(value['observed_ledger_head'],original['observed_ledger_head'])
 def test_complete_corruption_and_wrong_manifest_fail_closed(self):
  raw=self.path.read_bytes()
  with self.path.open('ab') as f:f.write(b'{"invalid":true}\n')
  with self.assertRaises((ValueError,KeyError)):self.read()
  self.path.write_bytes(raw);self.man['campaign_id']='b'*64
  with self.assertRaises(ValueError):self.read()
 def test_symlink_missing_and_bound_do_not_create_or_modify(self):
  self.path.unlink()
  with self.assertRaises(FileNotFoundError):self.read()
  self.assertFalse(self.path.exists())
  other=self.path.with_name('other');other.write_text('unchanged');self.path.symlink_to(other)
  with self.assertRaises(OSError):self.read()
  self.assertEqual(other.read_text(),'unchanged');self.path.unlink();self.path.write_bytes(b'x'*(4*1024*1024+1))
  with self.assertRaisesRegex(ValueError,'bound'):self.read()
 def test_new_complete_unknown_receipt_remains_unknown(self):
  self.ledger.enroll('b'*64,'c'*64);key=self.ledger.reserve('call','b'*64,'authored','d'*64)
  value=self.read();self.assertEqual(value['unknown_calls'],[key]);self.assertEqual(value['committed_microusd'],50000);self.assertTrue(value['stopped'])
 def test_upgrade_prefix_and_implementation_binding(self):
  head=json.loads(self.path.read_text())['hash']
  upgrade={'version':'b91-campaign-upgrade@v1','old_manifest_sha256':self.m.digest(self.man),'old_implementation_sha256':self.man['implementation_sha256'],'new_implementation_sha256':self.man['implementation_sha256'],'ledger_prefix_head':head,'reviewer':'authored fixture','reason':'read-only projection fixture'}
  self.assertEqual(ui.accounting_snapshot(self.m,self.man,upgrade)['calls'],0)
  for k in ('ledger_prefix_head','new_implementation_sha256'):
   bad={**upgrade,k:'f'*64}
   with self.assertRaises(ValueError):ui.accounting_snapshot(self.m,self.man,bad)

if __name__=='__main__':unittest.main()
