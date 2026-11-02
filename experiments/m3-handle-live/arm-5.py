from pathlib import Path
import sys
sys.path.insert(0,str(Path(__file__).resolve().parent))
from live_core import main
main(5,Path(__file__).resolve())
