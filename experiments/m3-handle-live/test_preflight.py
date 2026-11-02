import io,json,time,unittest
from pathlib import Path
from unittest.mock import patch
import live_core as c
from test_live import request
class Tests(unittest.TestCase):
    def test_scope_and_secret_rejected_before_credentials(self):
        for change in ({'task':'0'*64},{'observation':'sk-'+'x'*30}):
            r=request()|change
            with patch.object(c,'checked_manifest',return_value=({}, {'interface':'old'}, {'deadline':time.time()+30,'global_deadline':time.time()+60})),patch.object(c.sys,'argv',['broker','--ledger','/tmp/offline-ledger','--approved-live']),patch.object(c.sys,'stdin',io.TextIOWrapper(io.BytesIO(json.dumps(r).encode()))),patch.object(c.T,'credential',side_effect=AssertionError('credential must not be read')) as credential,patch.object(c.T,'request_json',side_effect=AssertionError('no dispatch')) as dispatch,patch.object(c.sys,'stderr',io.StringIO()):
                with self.assertRaises(SystemExit):c.main(0,Path('/tmp/entry'))
                credential.assert_not_called();dispatch.assert_not_called()
if __name__=='__main__':unittest.main()
