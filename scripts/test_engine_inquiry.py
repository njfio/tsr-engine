"""Local authored wiring controls; no provider or campaign operations."""
import json
from pathlib import Path
import subprocess
import tempfile
import unittest

BIN = Path(__file__).resolve().parents[1] / 'target/debug/tsr-qualitative'

class Inquiry(unittest.TestCase):
    def test_fresh_process_question_read_reflection_context(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            state = root / 'state'
            def run(command, value=None, valid=True):
                args = [str(BIN), str(state), command]
                if value is not None:
                    file = root / 'input.json'
                    file.write_text(json.dumps(value))
                    args.append(str(file))
                p = subprocess.run(args, capture_output=True, text=True)
                if not valid:
                    self.assertNotEqual(p.returncode, 0)
                    return
                self.assertEqual(p.returncode, 0, p.stderr)
                return json.loads(p.stdout)
            run('init', {'parents':[{'label':'a','meaning':'Understand differing accounts','attribution':'authored fixture'}], 'records':[], 'constraints':[], 'acceptance_references':[]})
            corpus = root / 'corpus'; corpus.mkdir()
            body = corpus / 'a.txt'; body.write_text('Fictional assertion, truth unknown')
            digest = json.loads(subprocess.check_output([str(BIN), str(state), 'content-digest', str(body)], text=True))
            q = run('request-v2')
            run('configure-retrieval', {'state_root':q['state_root'],'manifest':{'root':str(corpus.resolve()),'records':[{'id':'a','path':'a.txt','digest':digest,'attribution':'fiction','uncertainty':'unverified'}]}})
            q = run('question-request')
            self.assertEqual(q, run('question-request'))
            inquiry = {'state_root':q['state_root'],'attention':q['inquiry_binding'],'parents':q['native_attention']['selected'],'question':'What account is asserted?','uncertainty':'Index does not disclose the account','attribution':'authored generator fixture','origin':'model_proposed','answer_source':{'kind':'corpus_assertions','corpus':q['retrieval_contract']['corpus'],'rationale':'Authored question about assertions only'}}
            run('propose-inquiry',dict(inquiry,origin='user_supplied'),valid=False)
            pid = run('propose-inquiry',inquiry)
            q = run('question-request')
            run('review', {'state_root':q['state_root'],'proposal':pid,'reviewer':'independent authored fixture','decision':'admit','reason':'Question relevant, truth unknown; no action authority'})
            q = run('question-request')
            req = {'state_root':q['state_root'],'corpus':q['retrieval_contract']['corpus'],'ids':['a']}
            run('retrieve',req,valid=False)
            choice = {'inquiry':pid,'request':req,'relevance':'May supply an account, not verification'}
            receipt = run('retrieve-for-question',choice)
            after = run('question-request')
            self.assertEqual(after['basis']['receipt'],receipt)
            self.assertEqual(after['retrieval_questions'][0]['inquiry'],inquiry)
            self.assertEqual(after['retrieval_observations'][0]['inquiry'],pid)
            run('retrieve-for-question',choice,valid=False)
            self.assertEqual(after,run('question-request'))
            self.assertEqual(after['authority'],q['authority'])

if __name__ == '__main__':
    unittest.main()
