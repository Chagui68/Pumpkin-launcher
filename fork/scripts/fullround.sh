#!/usr/bin/env bash
# One full plugin round: boot, PatchBukkit conformance tests, console commands, stop, per-plugin report.
# Usage: fullround.sh <tag>. Uses the same SERVER_DIR / PUMPKIN_EXE / JAVA_HOME variables as start.sh.
SCRIPTS="$(cd "$(dirname "$0")" && pwd)"
SERVER_DIR="${SERVER_DIR:-$HOME/pumpkin-server}"
PUMPKIN_EXE="${PUMPKIN_EXE:-$HOME/pumpkin-target/debug/pumpkin.exe}"
export JAVA_HOME="${JAVA_HOME:-/c/Program Files/Eclipse Adoptium/jdk-25.0.4.101-hotspot}"
export PATH="$JAVA_HOME/bin/server:$JAVA_HOME/bin:$PATH"
cd "$SERVER_DIR" || exit 1

: > cmds.txt
rm -f exit.txt
( tail -f cmds.txt | "$PUMPKIN_EXE" > server.log 2>&1; echo "EXIT=$?" > exit.txt ) &
sleep 5
for i in $(seq 1 600); do grep -q "background initialization complete" server.log 2>/dev/null && break; [ -f exit.txt ] && break; sleep 1; done
sleep 60
echo "pbtest all" >> cmds.txt; sleep 45
for c in "bentobox version" "lp info" "sf versions" "list" "seed"; do echo "$c" >> cmds.txt; sleep 5; done
echo stop >> cmds.txt
for i in $(seq 1 120); do tasklist //FI "IMAGENAME eq pumpkin.exe" | grep -q pumpkin || break; sleep 1; done
tasklist //FI "IMAGENAME eq pumpkin.exe" | grep -q pumpkin && { echo "HUNG ON STOP"; taskkill //F //IM pumpkin.exe >/dev/null; sleep 2; }
# tail -f only exits on its next write, so stop it explicitly.
ps | awk '$NF ~ /\/tail$/ {print $1}' | xargs -r kill 2>/dev/null; sleep 2
cp server.log "round-$1.log"
echo "== round $1: $(cat exit.txt 2>/dev/null)"
python -I "$SCRIPTS/analyze.py" "round-$1.log" plugins/data/patchbukkit/patchbukkit-plugins
