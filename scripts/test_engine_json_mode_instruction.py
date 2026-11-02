"""Offline request-format checks; no transport, native state or paid calls."""
import importlib.util
from pathlib import Path
import unittest

s=importlib.util.spec_from_file_location('json_instruction_adapter',Path(__file__).with_name('engine-project-cycle-adapter.py'))
m=importlib.util.module_from_spec(s);s.loader.exec_module(m)

class JsonInstruction(unittest.TestCase):
 def adapter(self,release=True):
  a=object.__new__(m.Adapter);a.release=release;a.admission={'payload_bytes':10000,'completion_tokens':32768};return a
 def test_review_without_incidental_json_has_instruction(self):
  a=self.adapter();payload={'evidence':['unchanged'],'verdict':'unknown'};b=a.body('Independent review. Return {verdict}.',payload)
  self.assertEqual(b['messages'][0]['content'],'Independent review. Return {verdict}. Return a JSON object.')
  self.assertEqual(b['messages'][1]['content'],m.e.canonical(payload).decode())
  self.assertEqual(b['response_format'],{'type':'json_object'})
 def test_incidental_user_word_does_not_replace_system_instruction(self):
  b=self.adapter().body('Review without format guidance.',{'file':'data.json'})
  self.assertTrue(b['messages'][0]['content'].endswith('Return a JSON object.'))
 def test_existing_instruction_and_legacy_mode_preserved(self):
  system='Return JSON matching the same review schema.'
  self.assertEqual(self.adapter().body(system,{})['messages'][0]['content'],system)
  b=self.adapter(False).body('Legacy prompt.',{})
  self.assertEqual(b['messages'][0]['content'],'Legacy prompt.');self.assertNotIn('response_format',b)
 def test_added_instruction_is_counted_in_capacity(self):
  a=self.adapter();a.admission['payload_bytes']=1
  with self.assertRaises(m.d.Blocked):a.body('Review.',{})

if __name__=='__main__':unittest.main()
