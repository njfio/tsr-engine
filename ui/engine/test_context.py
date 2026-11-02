"""Authored context tests; no model calls or native mutations."""
import json,unittest
from context import package,unpack,PREFIX,RECORDS,INPUTS,RULES,LEGACY_TAIL
class Context(unittest.TestCase):
 def test_successive_messages_do_not_rewrap_or_duplicate(self):
  brief='Original open design goal';events=[];record={'journal':'j','record':'r','value':{'critique':'keep exact'}}
  for n in range(20):
   events.append({'id':str(n),'text':'AUTHORED input '+str(n),'attribution':'local_user_input','source_snapshot':{'native_snapshot':str(n)}})
   brief=package(brief,[record],events)
   original,records,retained=unpack(brief)
   self.assertEqual(original,'Original open design goal');self.assertEqual(records,[record]);self.assertEqual(retained,events)
   self.assertEqual(brief.count('AUTHORED input '+str(n)+'"'),1)
   self.assertLess(len(brief),len(RULES)+500+sum(len(json.dumps(e)) for e in events))
  self.assertEqual(json.JSONDecoder().raw_decode(brief,len('Studio public context v2\n'))[0]['latest_request_id'],'19')
 def test_legacy_and_embedded_marker_preserve_exact_evidence(self):
  record={'journal':'j','record':'r','value':'do not parse '+INPUTS+'[]'}
  event={'id':'x','text':'AUTHORED '+RECORDS+'[]','attribution':'local_user_input'}
  original='Original brief'
  brief=PREFIX+PREFIX+original+'\n'+RECORDS+json.dumps([record])+'\n'+LEGACY_TAIL+INPUTS+json.dumps([event])
  out=package(brief,[record],[event]);self.assertEqual(unpack(out),(original,[record],[event]));self.assertEqual(package(out,[record],[event]),out)
 def test_evidence_updates_applicability_not_user_instruction(self):
  user={'id':'u','text':'AUTHORED change','attribution':'local_user_input'}
  assessment={'id':'a','record_type':'independent_assessment','text':'AUTHORED exact result','report_sha256':'hash','source_snapshot':{'native_snapshot':'one'},'applicability':'current_source'}
  first=package('original',[],[user,assessment]);assessment={**assessment,'applicability':'historical_source_not_current_validation'}
  result=package(first,[],[user,assessment]);self.assertEqual(unpack(result)[2],[user,assessment])
  self.assertIn('"latest_request_id": "u"',result)
  with self.assertRaisesRegex(ValueError,'identity changed'):package(result,[],[{**user,'text':'altered'}])
 def test_literal_markers_in_original_are_not_metadata(self):
  original='User prose '+INPUTS+'[] and '+RECORDS+'[]'
  self.assertEqual(unpack(package(original,[],[])),(original,[],[]))
 def test_wrapped_literal_marker_fails_closed(self):
  brief=PREFIX+'User prose '+INPUTS+'[]\nDo not lose this requirement'
  with self.assertRaisesRegex(ValueError,'legacy context suffix'):package(brief,[],[])
 def test_conflicting_legacy_events_fail_closed(self):
  brief=PREFIX+'Original\n'+INPUTS+json.dumps([{'id':'same','text':'one'},{'id':'same','text':'two'}])
  with self.assertRaisesRegex(ValueError,'identity changed'):package(brief,[],[])
 def test_question_and_change_have_distinct_instructions_without_keyword_routing(self):
  for text in ['Explain this game','Can you improve this game?']:
   brief=package('Open brief',[],[{'id':'new','text':text,'attribution':'local_user_input'}]);self.assertEqual(unpack(brief)[2][0]['text'],text)
   self.assertIn('For a question, answer',brief);self.assertIn('For a change request',brief);self.assertIn('abstention remains valid',brief)
if __name__=='__main__':unittest.main()
