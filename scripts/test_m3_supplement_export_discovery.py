"""Exercise the exporter's actual discovery statements without exporting data."""
import ast
from pathlib import Path
import tempfile
import unittest

class SupplementDiscovery(unittest.TestCase):
    def test_m3_supp_02_discovers_each_supplementary_namespace(self):
        source=Path(__file__).with_name('m3-admission-evidence.py')
        tree=ast.parse(source.read_text())
        # Stop before the first export loop: this executes real discovery only.
        index=next(i for i,node in enumerate(tree.body) if isinstance(node,ast.For))
        code=compile(ast.Module(body=tree.body[:index],type_ignores=[]),str(source),'exec')
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);(root/'target').mkdir();(root/'scripts').mkdir()
            expected=[]
            for family in ['daemon','wire','predicates','structural']:
                name='m3-qualified-rts-supplement-'+family+'-continuation-'+'a'*12
                (root/'target'/name).mkdir();expected.append((name,True))
            scope={'__file__':str(root/'scripts/m3-admission-evidence.py')}
            exec(code,scope)
            for item in expected:self.assertIn(item,scope['attempts'])
            self.assertFalse((root/'docs').exists())
