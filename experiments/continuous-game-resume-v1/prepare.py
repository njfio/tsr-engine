"""Bindings only; cannot initialize a new campaign or game."""
import importlib.util
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('original_continuous_prepare',ROOT/'experiments/continuous-game-v1/prepare.py');original=importlib.util.module_from_spec(s);s.loader.exec_module(original)
a=original.a;AUTH=original.AUTH;FROZEN=Path(__file__).resolve().parent;PACKET=a.d.file_sha(FROZEN/'manifest.json')
