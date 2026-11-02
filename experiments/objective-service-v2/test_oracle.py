import unittest
from oracle import assess

class ReceiptOracle(unittest.TestCase):
    def test_multiple_legitimate_outputs(self):
        for out in ('11,21,31\n','12,21,32\n'):
            self.assertTrue(assess('3|11:1,12:1,21:2,31:3,32:3','',out)['finite_selection_valid'])
    def test_actual_previous_output_matters(self):
        c='3|12:1,21:2,31:3,32:3,41:4'
        self.assertTrue(assess(c,'11,21,31\n','12,21,31\n')['finite_selection_valid'])
        self.assertFalse(assess(c,'11,21,31\n','12,21,32\n')['finite_selection_valid'])
    def test_exit_zero_cannot_make_bad_output_correct(self):
        self.assertFalse(assess('3|11:1,12:1,21:2,31:3,32:3','','11,12,21\n')['finite_selection_valid'])
        for bad in ('passed\n','11,21,31\nOK','11,,21',''):
            try:r=assess('3|11:1,21:2,31:3','',bad)
            except ValueError:continue
            self.assertFalse(r['finite_selection_valid'])
    def test_malformed_prior_is_not_silently_reset(self):
        with self.assertRaises(ValueError):assess('2|11:1,21:2','fake receipt','11,21\n')

if __name__ == '__main__': unittest.main()
