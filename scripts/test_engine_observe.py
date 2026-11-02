"""Collector boundary/recovery tests, without inference or source discovery."""
import copy
import importlib.util
import json
import io
import os
from pathlib import Path
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('collector', Path(__file__).with_name('engine-observe.py'))
c = importlib.util.module_from_spec(spec)
spec.loader.exec_module(c)


class CollectorTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name).resolve()
        self.run = self.root / 'run'; self.run.mkdir()
        self.source = self.root / 'source.txt'; self.source.write_text('old')
        self.mapping = {'root': str(self.root), 'max_bytes': 512,
                        'sources': [{'id': 'a', 'topic': 'x', 'path': 'source.txt'}], 'continuation': None}
        self.file = self.root / 'mapping.json'
        self.state = {'state_root': 'initial', 'observation_revision': 'oldrev', 'available_notes': [{'id': 'a', 'topic': 'x', 'text': 'old'}],
                      'observation_updates': [], 'status': 'satisfied'}
        self.observations = 0
        self.fail_after_commit = False

    def cli(self, directory, command, *args):
        if command == 'direction-request':
            return copy.deepcopy(self.state)
        self.assertEqual(command, 'direction-observe')
        self.observations += 1
        update = json.loads(Path(args[1]).read_text())
        self.assertEqual(update['state_root'], self.state['state_root'])
        text = update['notes'][0]['text']
        if text != self.state['available_notes'][0]['text']:
            old = self.state['observation_revision']
            self.state['observation_revision'] = c.digest(text)
            self.state['state_root'] = c.digest(update)
            self.state['available_notes'][0]['text'] = text
            self.state['status'] = 'deliberate'
            self.state['observation_updates'].append({'reason': update['reason'], 'from': old, 'to': self.state['observation_revision']})
        if self.fail_after_commit:
            self.fail_after_commit = False
            raise TimeoutError('lost acknowledgment')
        return copy.deepcopy(self.state)

    def invoke(self, continue_run=False):
        self.file.write_text(json.dumps(self.mapping))
        with patch.object(c.directions, 'cli', side_effect=self.cli):
            return c.run(SimpleNamespace(directory=self.run, mapping=self.file, polls=1, interval=1, continue_run=continue_run))

    def test_unchanged_restart_and_source_change_preserve_versioned_submission(self):
        self.invoke(); head = self.state['state_root']
        self.assertEqual(self.invoke()['status'], 'unchanged')
        self.assertEqual(self.observations, 1)
        self.assertEqual(self.state['state_root'], head)
        self.source.write_text('new')
        result = self.invoke()
        self.assertEqual(result['state']['status'], 'deliberate')
        self.assertEqual(self.observations, 2)
        self.assertEqual(self.invoke()['status'], 'unchanged')

    def test_lost_acknowledgment_reconciles_without_duplicate_observation(self):
        self.source.write_text('new'); self.fail_after_commit = True
        with self.assertRaises(TimeoutError): self.invoke()
        self.assertEqual(self.invoke()['status'], 'unchanged')
        self.assertEqual(self.observations, 1)
        events = [json.loads(l) for l in (self.run / 'collector.jsonl').read_text().splitlines()]
        self.assertTrue(any(e.get('reconciled') for e in events))

    def test_unmatched_advanced_head_stops_uncertain_retry(self):
        original = self.cli
        def interrupted(directory, command, *args):
            if command == 'direction-observe':
                self.state['state_root'] = 'other-writer'
                raise TimeoutError('no acknowledged append')
            return original(directory, command, *args)
        self.cli = interrupted
        with self.assertRaises(TimeoutError): self.invoke()
        self.cli = original
        with self.assertRaisesRegex(ValueError, 'uncertain observation'): self.invoke()
        self.assertEqual(self.observations, 0)

    def test_mapping_and_allowance_cannot_expand_on_restart(self):
        self.invoke(); self.mapping['max_bytes'] = 511
        with self.assertRaisesRegex(ValueError, 'frozen'): self.invoke()

    def test_unknown_note_is_rejected_before_source_read(self):
        self.mapping['sources'][0]['id'] = 'unadmitted'
        with self.assertRaisesRegex(ValueError, 'admitted'): self.invoke()
        self.assertEqual(self.observations, 0)

    def test_stop_control_prevents_observation_and_continuation(self):
        (self.run / 'collector.stop').touch()
        with patch.object(c.directions, 'run') as generate:
            self.assertEqual(self.invoke(True)['status'], 'operator_stop')
            generate.assert_not_called()
        self.assertEqual(self.observations, 0)

    def test_continuation_uses_existing_budget_and_no_candidate(self):
        self.mapping['continuation'] = {'model': 'installed', 'endpoint': 'http://127.0.0.1:11435', 'calls': 10}
        self.source.write_text('new')
        with patch.object(c.directions, 'run', return_value={'status': 'local_call_budget', 'state': {}}) as generate:
            self.assertEqual(self.invoke(True)['status'], 'local_call_budget')
            args = generate.call_args.args[0]
            self.assertEqual(args.calls, 10)
            self.assertIsNone(args.batch)
            self.assertEqual(args.stop_file, self.run / 'collector.stop')

    def test_progress_is_separate_from_machine_readable_result(self):
        self.mapping['continuation']={'model':'installed','endpoint':'http://127.0.0.1:11435','calls':10}
        self.source.write_text('new')
        def generate(args):
            print('local progress'); return {'status':'local_call_budget','state':{}}
        out,err=io.StringIO(),io.StringIO()
        with patch.object(c.sys,'stdout',out),patch.object(c.sys,'stderr',err),patch.object(c.directions,'run',side_effect=generate):
            self.invoke(True)
        self.assertEqual(out.getvalue(),'')
        self.assertEqual(err.getvalue(),'local progress\n')

    def test_stop_during_artifact_preparation_prevents_dispatch(self):
        save=c.directions.local.save
        def stopping(*args,**kwargs):
            save(*args,**kwargs); (self.run/'collector.stop').touch()
        with patch.object(c.directions.local,'save',side_effect=stopping):
            self.assertEqual(self.invoke()['status'],'operator_stop')
        self.assertEqual(self.observations,0)
        events=[json.loads(l) for l in (self.run/'collector.jsonl').read_text().splitlines()]
        self.assertEqual(events[-1]['status'],'observation_not_committed')

    def test_read_rejects_symlink_hardlink_traversal_fifo_and_oversize(self):
        (self.root / 'link').symlink_to(self.source)
        with self.assertRaises(OSError): c.stable_read(str(self.root), 'link', 512)
        os.link(self.source, self.root / 'hard')
        with self.assertRaises(ValueError): c.stable_read(str(self.root), 'hard', 512)
        (self.root / 'hard').unlink()
        with self.assertRaises(ValueError): c.stable_read(str(self.root), '../source.txt', 512)
        os.mkfifo(self.root / 'fifo')
        with self.assertRaises(ValueError): c.stable_read(str(self.root), 'fifo', 512)
        self.source.write_text('x'*513)
        with self.assertRaises(ValueError): c.stable_read(str(self.root), 'source.txt', 512)

    def test_read_rejects_inplace_change_or_atomic_replacement_during_read(self):
        read = os.read
        for replace in (False, True):
            self.source.write_text('old')
            def racing(fd, size):
                raw = read(fd, size)
                if replace:
                    tmp = self.root / 'replacement'; tmp.write_text('new'); tmp.replace(self.source)
                else:
                    self.source.write_text('new')
                return raw
            with patch.object(c.os, 'read', side_effect=racing):
                with self.assertRaisesRegex(ValueError, 'changed during read'):
                    c.stable_read(str(self.root), 'source.txt', 512)

    def test_read_rejects_directory_replacement(self):
        folder=self.root/'nested';folder.mkdir();(folder/'note').write_text('old')
        read=os.read
        def racing(fd,size):
            raw=read(fd,size);folder.rename(self.root/'moved');folder.mkdir();(folder/'note').write_text('new');return raw
        with patch.object(c.os,'read',side_effect=racing):
            with self.assertRaisesRegex(ValueError,'directory changed'):
                c.stable_read(str(self.root),'nested/note',512)

    def test_read_rejects_root_replacement(self):
        folder=self.root/'anchor';folder.mkdir();(folder/'note').write_text('old')
        read=os.read
        def racing(fd,size):
            raw=read(fd,size);folder.rename(self.root/'old-anchor');folder.mkdir();(folder/'note').write_text('new');return raw
        with patch.object(c.os,'read',side_effect=racing):
            with self.assertRaisesRegex(ValueError,'root changed'):
                c.stable_read(str(folder),'note',512)


if __name__ == '__main__':
    unittest.main()
