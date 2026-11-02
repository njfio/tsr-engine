"""Fixture-only revision crash/restart checks; no model calls or semantic success claims."""
import json
from pathlib import Path
import tempfile
import unittest

import test_engine_workshop_cycle as workshop


class RevisionContinuity(unittest.TestCase):
    def test_uncertain_revision_never_dispatches_next_possibility(self):
        helper = workshop.WorkshopCycle()
        for checkpoint, history_length in [('apply_intent-4', 1), ('controller_returned-4', 2)]:
            with self.subTest(checkpoint=checkpoint), tempfile.TemporaryDirectory() as tmp:
                folder = Path(tmp)
                driver, cycle, initial = workshop.setup(folder)
                interrupted = helper.process(folder, checkpoint)
                self.assertEqual(interrupted.returncode, 75, interrupted.stderr)
                before = driver.cli(folder, 'direction-request')
                self.assertEqual(len(before['relational_request']['history']), history_length)
                self.assertEqual(before['cycles'], initial['cycles'] + 1)
                for key in ('standing_desires', 'authority'):
                    self.assertEqual(before[key], initial[key])
                inventory = cycle.storage_inventory(folder)
                journal = folder / 'synthetic-external'
                self.assertTrue((journal / 'apply-intent-4.json').exists())
                self.assertFalse((journal / 'controller-4.json').exists())
                for _ in range(2):
                    resumed = helper.process(folder)
                    self.assertEqual(resumed.returncode, 0, resumed.stderr)
                    self.assertEqual(json.loads(resumed.stdout)['status'], 'uncertain_application')
                    self.assertEqual(driver.cli(folder, 'direction-request'), before)
                    self.assertEqual(cycle.storage_inventory(folder), inventory)
                    self.assertEqual(len((folder / 'fake-calls').read_text().splitlines()), 4)
                    self.assertFalse((journal / 'intent-5.json').exists())

    def test_receipted_revision_flows_into_next_proposal_after_restart(self):
        helper = workshop.WorkshopCycle()
        with tempfile.TemporaryDirectory() as tmp:
            folder = Path(tmp)
            driver, cycle, initial = workshop.setup(folder)
            interrupted = helper.process(folder, 'controller_receipt-4')
            self.assertEqual(interrupted.returncode, 75, interrupted.stderr)
            reviewed = driver.cli(folder, 'direction-request')
            history = reviewed['relational_request']['history']
            self.assertEqual(len(history), 2)
            self.assertEqual(history[-1]['proposal']['development']['proposed_development'], None)
            resumed = helper.process(folder)
            self.assertEqual(resumed.returncode, 0, resumed.stderr)
            self.assertEqual(json.loads(resumed.stdout)['status'], 'proposal_only_complete')
            final = driver.cli(folder, 'direction-request')
            self.assertEqual(final, reviewed)
            request = cycle.read(folder / 'synthetic-external' / 'request-5.json')
            context = json.loads(request['messages'][1]['content'])['context']
            self.assertEqual(context['relational_guidance'], cycle.context(reviewed, driver, 'workshop')['relational_guidance'])
            self.assertEqual(context['state_root'], reviewed['state_root'])
            self.assertEqual(final['cycles'], initial['cycles'] + 1)
            self.assertEqual(len((folder / 'fake-calls').read_text().splitlines()), 5)
            self.assertFalse((folder / 'synthetic-external' / 'apply-intent-5.json').exists())
            for key in ('standing_desires', 'authority'):
                self.assertEqual(final[key], initial[key])


if __name__ == '__main__':
    unittest.main()
