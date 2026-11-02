"""Fake responses test mechanics only; no model emergence or garden evidence."""
import copy
import json
from pathlib import Path
import tempfile
import unittest
import test_engine_workshop_cycle as w

class GardenCycle(w.WorkshopCycle):
    # Do not repeat inherited workshop matrix: focused garden transfer boundaries.
    def test_five_stages_and_restarts(self):
        for pause in ['outcome-1','applied-2','controller_receipt-3','applied-4','outcome-5']:
            with self.subTest(pause=pause),tempfile.TemporaryDirectory() as temp:
                folder=Path(temp);d,c,initial=w.setup(folder,'garden')
                self.assertEqual(self.process(folder,pause).returncode,75)
                result=self.process(folder);self.assertEqual(result.returncode,0,result.stderr)
                terminal=json.loads(result.stdout)
                self.assertEqual(terminal['status'],'proposal_only_complete')
                self.assertEqual(terminal['calls_dispatched'],5)
                self.assertEqual(terminal['acknowledged_actions'],1)
                final=d.cli(folder,'direction-request')
                self.assertEqual(final['standing_desires'],initial['standing_desires'])
                self.assertEqual(final['authority'],initial['authority'])
                self.assertEqual(len(final['archive']['events']),5)
                self.assertFalse((folder/'synthetic-external/apply-intent-5.json').exists())
                self.process(folder)
                self.assertEqual(len((folder/'fake-calls').read_text().splitlines()),5)
                self.assertEqual(d.cli(folder,'direction-request'),final)
                a=c.read(folder/'admission.json')
                for n in range(1,6):
                    req=c.read(folder/f'synthetic-external/request-{n}.json')
                    ctx=json.loads(req['messages'][1]['content'])['context']
                    self.assertIn('no planting',ctx['semantic_boundary'])
                    self.assertLessEqual(len(json.dumps(req).encode()),16384)
                bad=copy.deepcopy(initial);bad['archive']['world']['responses'][0]['text']='drift'
                with self.assertRaises(ValueError):c.scope(bad,d,'garden')
                with self.assertRaises(ValueError):c.scope(initial,d,'workshop')
                bad=copy.deepcopy(initial);bad['archive']['events'].pop()
                with self.assertRaises(ValueError):c.workshop_initial(bad,'garden')

if __name__=='__main__':unittest.main()
