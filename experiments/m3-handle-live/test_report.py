import unittest
from report import EXPECTED,evaluate
class Tests(unittest.TestCase):
    def test_missing_never_shrinks_denominator(self):
        result=evaluate(dict(status='halted-review-required',results=[]));self.assertEqual(result['arms']['new']['missing'],6);self.assertNotEqual(result['decision'],'go-to-separately-approved-TSR-comparison')
    def test_multiedit_required_even_when_all_accept(self):
        rows=[dict(index=i,interface=v,final_oracle_completed=True,final_oracle_success=True,outcome='accepted',freshness={'result':{'fresh_edits':1}}) for i,v in enumerate(EXPECTED)]
        t=dict(status='completed',results=rows);self.assertFalse(evaluate(t)['checks']['multi_edit_at_least_three'])
        for r in rows:r['freshness']['result']['fresh_edits']=2
        self.assertEqual(evaluate(t)['decision'],'go-to-separately-approved-TSR-comparison')
if __name__=='__main__':unittest.main()
