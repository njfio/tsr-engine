"""Read controller receipts before exporting immutable qualification attempts."""
import json


def terminal_attempt(source, target_root, export_root):
    if not source.is_dir():
        return False
    matches = []
    for state in target_root.glob('*state'):
        if not state.is_dir():
            continue
        for receipt in state.glob('*.json'):
            row = json.loads(receipt.read_text())
            if not isinstance(row, dict):
                continue
            argv = row.get('argv', [])
            if isinstance(argv, list) and '--out' in argv:
                i = argv.index('--out')
                if i + 1 < len(argv) and argv[i + 1] == str(source):
                    matches.append(row)
    if matches:
        return all(r.get('status') == 'completed' and r.get('exit') == 0
                   and not r.get('errors') for r in matches)
    # Unregistered historical attempts are retained from their preserved export;
    # matching manifests alone do not prove phase/CAS immutability.
    return False


def retain_history(attempts, historical):
    seen = {name for name, _ in attempts}
    return attempts + [(name, row['strict_protocol']) for name, row in historical.items()
                       if name not in seen]
