from pathlib import Path
import sys
ROOT=Path(__file__).resolve().parents[2]
sys.path.insert(0,str(ROOT/"experiments/m3-handle-live"))
import live_core
live_core.MANIFEST=ROOT/"docs/m3-handle-live-amendment-v1.json"
live_core.main(6,Path(__file__).resolve())
