#!/bin/sh
# Current-user session only: no LaunchAgent installation or host security changes.
set -eu
UI_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
LABEL=local.tsr.studio
health() { /usr/bin/curl --fail --max-time 10 --silent http://127.0.0.1:8766/api/bootstrap >/dev/null; }
case "${1:-status}" in
 start)
  if launchctl list "$LABEL" >/dev/null 2>&1; then
   if health; then printf '%s\n' 'TSR Studio is reachable: http://127.0.0.1:8766/';exit 0;fi
   # Restart only a registered, stopped job. Never duplicate a live server.
   if launchctl list "$LABEL" | /usr/bin/grep -q '"PID"'; then
    printf '%s\n' 'TSR job is running but HTTP is unavailable; inspect /tmp/tsr-studio-service.err. No duplicate started.' >&2;exit 1
   fi
   launchctl remove "$LABEL" || { if launchctl list "$LABEL" >/dev/null 2>&1; then exit 1;fi; }
  fi
  if /usr/sbin/lsof -nP -iTCP:8766 -sTCP:LISTEN >/dev/null 2>&1; then
   printf '%s\n' 'Port 8766 already has a listener; no duplicate started.' >&2;exit 1
  fi
  PYTHON=$(command -v python3)
  "$PYTHON" -c "import tomllib"
  DOCKER=$(command -v docker)
  test -x "$DOCKER"
  # launchctl does not inherit the interactive PATH. Preserve only this ordinary
  # executable-search setting for the local service; do not copy credentials.
  launchctl submit -l "$LABEL" -o /tmp/tsr-studio-service.log -e /tmp/tsr-studio-service.err -- /usr/bin/env "PATH=$PATH" "$PYTHON" "$UI_DIR/server.py"
  count=0
  until health; do count=$((count+1));if [ "$count" -ge 5 ];then printf '%s\n' 'Job submitted but HTTP verification failed; inspect /tmp/tsr-studio-service.err.' >&2;exit 1;fi;sleep 1;done
  printf '%s\n' 'TSR Studio is reachable: http://127.0.0.1:8766/'
  ;;
 status) launchctl list "$LABEL"; /usr/bin/curl --fail --max-time 10 --silent http://127.0.0.1:8766/api/bootstrap ;;
 stop) launchctl remove "$LABEL" ;;
 *) printf '%s\n' 'Usage: service.sh start|status|stop' >&2;exit 2;;
esac
