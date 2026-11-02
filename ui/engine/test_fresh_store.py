"""Public checkout must not manufacture prior user messages or project authority."""
import tempfile, unittest, http.client, http.server, json, threading
from pathlib import Path
from server import Store, Handler

class FreshStore(unittest.TestCase):
 def test_fresh_default_has_no_private_projects_or_inputs(self):
  with tempfile.TemporaryDirectory() as root:
   store=Store(root)
   self.assertEqual(store.catalog,[])
   self.assertEqual(list((Path(root)/'projects').iterdir()),[])
   service=http.server.ThreadingHTTPServer(('127.0.0.1',0),Handler)
   service.store=store;service.expected_host=f'127.0.0.1:{service.server_port}';service.origin='http://'+service.expected_host
   worker=threading.Thread(target=service.serve_forever,daemon=True);worker.start()
   try:
    client=http.client.HTTPConnection('127.0.0.1',service.server_port)
    client.request('GET','/api/bootstrap');response=client.getresponse()
    self.assertEqual(response.status,200)
    self.assertEqual(json.loads(response.read())['projects'],[])
    client.close()
   finally:service.shutdown();service.server_close();worker.join()
 def test_reopening_preserves_existing_project_and_input_without_seeding(self):
  with tempfile.TemporaryDirectory() as root:
   catalog=[{'id':'authored','title':'Authored fixture','source':'target/authored'}]
   first=Store(root,catalog=catalog)
   event={'id':'authored-input','type':'input','role':'user','text':'Authored fixture input'}
   event=first.append('authored',event)
   before=(Path(root)/'projects/authored/project.json').read_bytes()
   reopened=Store(root)
   self.assertEqual(reopened.events('authored'),[event])
   self.assertEqual((Path(root)/'projects/authored/project.json').read_bytes(),before)
