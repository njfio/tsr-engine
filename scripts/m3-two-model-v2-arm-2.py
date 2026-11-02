"""Frozen controller-owned attempt 2 binding; no environment selection."""
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parent))
from m3_two_model_v2_core import main
if __name__=='__main__':main(2,Path(__file__))
