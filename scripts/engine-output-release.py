"""Release inventory: fail before legacy live effects; never rewrite evidence.

This is a launch compatibility gate, not user authority or a replacement parser.
Offline/library fixtures remain usable; only the reviewed inquiry entry point has
completed the current output-boundary integration. Direct low-level transports
are not supported operator launchers.
"""
from pathlib import Path
MODE_RUN=frozenset({
 'engine-b91-driver.py','engine-b91-campaign.py','engine-b91-continuation.py',
 'engine-b91-final-recovery.py','engine-b91-patch-recovery.py',
 'engine-b91-schema-patch-recovery.py','engine-b91-shadow.py',
 'engine-semantic-shadow.py','engine-next-proposal-shadow.py',
 'engine-campaign-next-choice.py','engine-native-cycle.py'})
DIRECT=frozenset({'engine-inquiry-validation.py','engine-retrieval-validation.py'})
def blocked(name):
 raise RuntimeError('OUTPUT_BOUNDARY_INCOMPATIBLE: '+name+' live launch is disabled. '
  'Its output-mode/quarantine/recovery admission is not release-compatible. '
  'Use read-only inspect/status or preserved receipts; do not reuse old grants. '
  'The prospective engine-exploration-validation.py path requires a separately reviewed fresh admission. '
  'Real repair/simulator workflows must be migrated and reviewed before re-enabling.')
def check_cli(file,argv):
 name=Path(file).name
 if name=='engine-native-cycle.py' and ('--release-v3' in argv or '--release-v4' in argv or '--release-v5' in argv or '--browser-v1' in argv):return # main requires exact reviewed v3 admissions and pins
 if name=='engine-run.py' and '--approved-live' in argv:blocked(name)
 if name=='engine-directions.py' and any(x=='--synthetic-admission' or x.startswith('--synthetic-admission=') for x in argv):blocked(name)
 if name in MODE_RUN and 'run' in argv:blocked(name)
 if name in DIRECT and argv and argv[0] not in ('-h','--help'):
  # Existing runner returns terminal before ledger/provider/reader instantiation.
  # Permit only that historical no-dispatch replay; fresh launch is disabled.
  if not (Path(argv[0])/'terminal.json').is_file():blocked(name)
