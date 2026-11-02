"""No inference or target execution; read-only shadow validator prerequisites."""
import copy
import unittest
from types import SimpleNamespace
import test_engine_b91_project_envelope as base

class ShadowPreflight(unittest.TestCase):
    def budget(self):
        return dict(active=True, stopped=False, exceeded=False, unknown_usage=0,
                    unsettled_requests=0, unresolved_tool_requests=0,
                    limits={"wall_ms":600000}, elapsed_wall_ms=0)
    def controller(self, budget, calls):
        def cli(*args):
            calls.append(args)
            self.assertEqual(args, ('project-budget',))
            return copy.deepcopy(budget)
        return SimpleNamespace(cli=cli)
    def test_ready_only_reads_budget_and_does_not_grant_authority(self):
        calls=[];b=self.budget()
        self.assertEqual(base.m.require_shadow_validation_ready(self.controller(b,calls)),b)
        self.assertEqual(calls,[('project-budget',)])
    def test_expired_unknown_and_missing_budget_stop_before_transport(self):
        cases=[{},None,dict(self.budget(),active=False),dict(self.budget(),stopped=True),
               dict(self.budget(),exceeded=True)]
        for field in ('unknown_usage','unsettled_requests','unresolved_tool_requests'):
            for value in (1,None,False):cases.append(dict(self.budget(),**{field:value}))
        for budget in cases:
            with self.subTest(budget=budget):
                sent=[];calls=[]
                with self.assertRaises(ValueError):
                    base.m.require_shadow_validation_ready(self.controller(budget,calls))
                    sent.append('would dispatch')
                self.assertEqual(sent,[])
    def test_deadline_and_validation_headroom_fail_closed(self):
        for elapsed in (330001,600000,None,True):
            with self.assertRaisesRegex(ValueError,'wall runway'):
                base.m.require_shadow_validation_ready(self.controller(dict(self.budget(),elapsed_wall_ms=elapsed),[]))
        base.m.require_shadow_validation_ready(self.controller(dict(self.budget(),elapsed_wall_ms=330000),[]))
        for deadline in (0,241,None,True):
            calls=[]
            with self.assertRaisesRegex(ValueError,'deadline'):
                base.m.require_shadow_validation_ready(self.controller(self.budget(),calls),call_seconds=deadline)
            self.assertEqual(calls,[])

    def test_explicit_unlimited_has_no_old_wall_ceiling_but_unknown_still_blocks(self):
        b=dict(self.budget(),unlimited_mode=True,unlimited_policy_sha256='a'*64,elapsed_wall_ms=999999999)
        self.assertEqual(base.m.require_shadow_validation_ready(self.controller(b,[])),b)
        for change in ({'unlimited_policy_sha256':'bad'},{'unknown_usage':1},{'stopped':True}):
            with self.assertRaises(ValueError):base.m.require_shadow_validation_ready(self.controller(dict(b,**change),[]))

    def test_later_expiry_is_not_cached_as_ready(self):
        budget=self.budget();calls=[];c=self.controller(budget,calls)
        base.m.require_shadow_validation_ready(c)
        budget.update(stopped=True,exceeded=True)
        with self.assertRaisesRegex(ValueError,'authority unavailable'):
            base.m.require_shadow_validation_ready(c)
        self.assertEqual(len(calls),2)

if __name__=='__main__':unittest.main()
