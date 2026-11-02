"""Offline optional forecast transport; no provider call or manufactured default."""
import importlib.util
import json
from pathlib import Path
import unittest
spec = importlib.util.spec_from_file_location('local_forecast', Path(__file__).with_name('engine-local-model.py'))
local = importlib.util.module_from_spec(spec)
spec.loader.exec_module(local)
class ForecastTransport(unittest.TestCase):
    def test_optional_forecast_is_preserved_and_unknown_remains_missing(self):
        request = {'protocol':'tsr-proposal@v1','state_root':'state','coding':{'files':[
            {'editable':True,'path':'a.rs','source':'old','source_digest':'digest','inspection_handle':'handle'}]}}
        reply = {'edits':[{'path':'a.rs','replacement':'new'}],'rationale':'fixture'}
        self.assertNotIn('common_event_forecasts',local.bind_proposal(request,json.dumps(reply)))
        for p in [0,0.37,1]:
            response=local.bind_proposal(request,json.dumps(dict(reply,common_event_probability=p)))
            self.assertEqual(response['common_event_forecasts'],{'0':p})
            self.assertEqual(response['proposals'][0]['patches'][0]['new_text'],'new')
        for p in [None,True,-1,2,'0.5']:
            with self.assertRaises(ValueError): local.bind_proposal(request,json.dumps(dict(reply,common_event_probability=p)))
        schema=local.response_schema(request)
        self.assertIn('common_event_probability',schema['properties'])
        self.assertNotIn('common_event_probability',schema['required'])
if __name__ == '__main__': unittest.main()
