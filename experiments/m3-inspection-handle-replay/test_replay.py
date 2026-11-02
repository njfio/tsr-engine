import unittest
from replay import Freshness
R=dict(handle='h1',path='src/a',source_state='s0')
P=dict(inspection_handle='h1',path='src/a')
class Replay(unittest.TestCase):
    def test_matching_handle_and_unchanged_test(self):
        f=Freshness();f.issue(R);f.transition('test','s0','s0');self.assertTrue(f.valid(P))
    def test_edit_and_revert_stays_invalid(self):
        f=Freshness();f.issue(R);f.transition('edit','s0','s1');f.transition('edit','s1','s0');self.assertFalse(f.valid(P))
    def test_mutating_test_invalidates(self):
        f=Freshness();f.issue(R);f.transition('test','s0','s1');self.assertFalse(f.valid(P))
    def test_latest_inspection_supersedes(self):
        f=Freshness();f.issue(R);f.issue(R|{'handle':'h2'});self.assertFalse(f.valid(P));self.assertTrue(f.valid(P|{'inspection_handle':'h2'}))
    def test_foreign_path_and_missing(self):
        f=Freshness();self.assertFalse(f.valid(P));f.issue(R)
        self.assertFalse(f.valid(P|{'path':'src/b'}));self.assertFalse(f.valid(P|{'inspection_handle':'another-arm'}))
    def test_discontinuous_evidence_refused(self):
        f=Freshness();f.issue(R)
        with self.assertRaises(AssertionError):f.transition('test','unknown','s0')
        with self.assertRaises(AssertionError):f.issue(R|{'source_state':'unknown'})
if __name__=='__main__':unittest.main()
