#!/usr/bin/env python3
"""Prepare the corrected responsibility packet locally; never dispatch."""
import argparse
import importlib.util
import json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
s=importlib.util.spec_from_file_location('shared_browser_preparation',ROOT/'experiments/browser-game-v1/prepare.py');shared=importlib.util.module_from_spec(s);s.loader.exec_module(shared)
if __name__=='__main__':
    p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);p.add_argument('--campaign-manifest',type=Path,required=True);p.add_argument('--packet-sha256',required=True);args=p.parse_args()
    packet=Path(__file__).with_name('manifest.json')
    if shared.a.d.file_sha(packet)!=args.packet_sha256:raise ValueError('packet identity mismatch')
    manifest=json.loads(packet.read_text())
    if any(shared.a.d.file_sha(ROOT/name)!=digest for name,digest in manifest['files'].items()):raise ValueError('packet source drift')
    print(json.dumps(shared.prepare(args.output,json.loads(args.campaign_manifest.read_text())),indent=2))
