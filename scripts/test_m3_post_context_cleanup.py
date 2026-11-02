import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch
spec=importlib.util.spec_from_file_location('proposal',Path(__file__).with_name('m3-prepare-post-context-cleanup.py'))
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)

class CleanupScope(unittest.TestCase):
    def test_only_exact_build_paths_are_eligible(self):
        p='target/m3-qualified-rts-context-continuation-'+'a'*12+'/'+'a'*64+'/copy/target'
        self.assertTrue(m.target_shape(p))
        for bad in [p+'/debug',p.replace('/copy/target','/cas'),p.replace('context','metrics'),p.replace('/copy/target','/copy/../target'),p.replace('/copy/target','/copy/.cargo')]:
            self.assertFalse(m.target_shape(bad),bad)

    def test_symlink_ancestor_refused(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);(root/'actual').mkdir();(root/'link').symlink_to(root/'actual',target_is_directory=True)
            with patch.object(m,'ROOT',root):
                with self.assertRaises(RuntimeError):m.real_ancestors(root/'link')

    def test_inventory_excludes_only_build_and_records_symlink(self):
        with tempfile.TemporaryDirectory() as d:
            root=Path(d);(root/'copy/target').mkdir(parents=True);(root/'copy/target/build').write_text('output');(root/'copy/source.rs').write_text('source');(root/'copy/link').symlink_to('source.rs');records={}
            with patch.object(m,'ROOT',root):m.inventory_tree(root/'copy',{root/'copy/target'},records)
            self.assertEqual(set(records),{'copy/source.rs','copy/link'})
            self.assertEqual(records['copy/link'],{'symlink':'source.rs'})
