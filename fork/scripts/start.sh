#!/usr/bin/env bash
# Starts Pumpkin with PatchBukkit in $SERVER_DIR. Console commands: append lines to $SERVER_DIR/cmds.txt.
SERVER_DIR="${SERVER_DIR:-$HOME/pumpkin-server}"
PUMPKIN_EXE="${PUMPKIN_EXE:-$HOME/pumpkin-target/debug/pumpkin.exe}"
export JAVA_HOME="${JAVA_HOME:-/c/Program Files/Eclipse Adoptium/jdk-25.0.4.101-hotspot}"
export PATH="$JAVA_HOME/bin/server:$JAVA_HOME/bin:$PATH"
cd "$SERVER_DIR" || exit 1
: > cmds.txt
tail -f cmds.txt | "$PUMPKIN_EXE" > server.log 2>&1
