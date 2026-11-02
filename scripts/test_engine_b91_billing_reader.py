"""Read-only historical reader regressions; no dispatch or ledger mutation."""
import importlib.util
from pathlib import Path
import unittest

spec=importlib.util.spec_from_file_location('billing_reader',Path(__file__).with_name('engine-b91-recovery-billing.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
c=m.c
ROOT=Path(__file__).resolve().parents[1]

class HistoricalReader(unittest.TestCase):
    def setUp(self):
        self.campaign=ROOT/'target/tsr-integrated-campaign-1'
        self.trial=ROOT/'target/b91-trial15-patch-recovery-1'
        self.lookup=self.trial/'reconciliation-1.json'
        if not self.lookup.exists():self.skipTest('preserved local operational evidence absent')
        self.manifest=c.strict((self.campaign/'manifest.json').read_bytes())
        self.upgrade=c.strict((self.campaign/'reconciliation-upgrade.json').read_bytes())
        self.review=c.strict((self.campaign/'failed-zero-review-recovery1.json').read_bytes())
    def ledger(self,kind=m.RecoveryLedger):
        return kind(self.manifest['ledger_path'],self.manifest,c.digest(self.manifest),upgrade=self.upgrade,upgrade_hash=c.digest(self.upgrade))
    def check(self,ledger):
        return ledger.reconcile_failed_zero(self.trial,self.lookup,self.review,c.digest(self.review),dry_run=True)
    def proof_row(self,ledger):
        # Other trials may append concurrently. Assert only this immutable receipt.
        with ledger.locked() as (_,rows):
            matches=[row for row in rows if row['event'].get('kind')=='bill_reconciled'
                     and row['event'].get('provider_request_id')=='54f40ca8-d9d6-4b33-8121-dde186f978c6']
            self.assertEqual(len(matches),1)
            event=matches[0]['event']
            self.assertEqual(event['key'],'e40ef9e5ce8fceffdd6fff0e8f261a51a6360ab974d84631da70b5f4024fa9db')
            self.assertEqual(event['microusd'],0)
            self.assertEqual(event['review_sha256'],c.digest(self.review))
            self.assertEqual(event['lookup_sha256'],self.review['lookup_sha256'])
            return matches[0]
    def test_original_recovery_proof_remains_valid(self):
        ledger=self.ledger();before=self.proof_row(ledger)
        self.check(ledger)
        self.assertEqual(self.proof_row(ledger),before)
    def test_trusted_subclass_reuses_same_exact_join(self):
        class Delegated(m.RecoveryLedger):
            def read_history(self,root):return super().read_history(root)
        original=self.ledger();delegated=self.ledger(Delegated)
        self.check(original);self.check(delegated)
        self.assertEqual(self.proof_row(original),self.proof_row(delegated))
    def test_reader_cannot_substitute_bytes_or_completed_summary(self):
        for slot in (1,5):
            class Broken(m.RecoveryLedger):
                def read_history(self,root):
                    values=list(super().read_history(root));values[slot]=b'{}' if slot==1 else {};return tuple(values)
            with self.assertRaisesRegex(ValueError,'historical reader'):
                self.check(self.ledger(Broken))

if __name__=='__main__':unittest.main()
