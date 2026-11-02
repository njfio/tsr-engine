"""Fixed reviewed real-corpus broker, no environment-driven model selection."""
from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parent))
from m3_fixed12_core import main
if __name__=='__main__':main(Path(__file__))
