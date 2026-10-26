# Fork extras

Patches and tools for running Bukkit, Spigot, Paper and Purpur plugins on this fork through PatchBukkit, and for testing them.

## PatchBukkit

Patches against [Pumpkin-MC/PatchBukkit](https://github.com/Pumpkin-MC/PatchBukkit) at commit `80a5219`.

| Patch | What it does |
| :---- | :----------- |
| `0001-loader-and-world-sync-fixes.patch` | Paper's legacy bytecode support (Commodore) for plugin classes, server-first class lookup like Paper, a real `CodeSource` per plugin class, no fake `TimeSkipEvent` when syncing the world clock, `TimeSkipEvent.SkipReason` remap, top-level-only plugin discovery. |
| `0002-build-against-pumpkin-launcher.patch` | Builds the plugin against this fork's crates instead of upstream `master`, so the plugin and the server agree on type layouts. Assumes PatchBukkit is cloned next to `Pumpkin-launcher`; adjust the `[patch]` paths in `rust/Cargo.toml` otherwise. |

```bash
git clone https://github.com/Pumpkin-MC/PatchBukkit.git
cd PatchBukkit
git checkout 80a5219
git apply ../Pumpkin-launcher/fork/patchbukkit-patches/0001-loader-and-world-sync-fixes.patch
git apply ../Pumpkin-launcher/fork/patchbukkit-patches/0002-build-against-pumpkin-launcher.patch
./build.sh
```

Rebuild the PatchBukkit library every time this fork changes: a native plugin built against different `pumpkin-core` sources can crash the server.

Server setup:

- Java 25 in `JAVA_HOME`.
- Copy `patchbukkit.dll` (or `.so`) into the server's `plugins/` folder.
- Bukkit plugins go in `plugins/data/patchbukkit/patchbukkit-plugins/`.
- When stdin is not a terminal, set `extra-args = ["-Dterminal.jline=false"]` in `plugins/data/patchbukkit/patchbukkit.config.toml`.

## Scripts

All scripts read `SERVER_DIR` (server working folder), `PUMPKIN_EXE` (server binary) and `JAVA_HOME`.

- `scripts/start.sh`: starts the server with console input read from `$SERVER_DIR/cmds.txt`. Append a line to that file to run a command. Redirecting to files avoids the pipe back-pressure that made a PowerShell-driven server look hung.
- `scripts/fullround.sh <tag>`: boots the server, runs PatchBukkit's `pbtest all`, a few plugin commands and `stop`, then prints a per-plugin report.
- `scripts/analyze.py <log> <plugins dir>`: per-plugin status (OK, failed enable, not loaded, enabled with errors) from a server log.
- `scripts/nstack.py <pid> [filter]`: dumps native stacks of every thread of a running process on Windows (dbghelp). Set `SYMBOL_PATH` to the folders holding `pumpkin.pdb` and `patchbukkit.pdb`. Used to find hangs that `jstack` cannot see.

## BentoBox ItemsAdder stub

BentoBox-Drake declares `dev.lone:api-itemsadder:4.0.2-beta-release-11` as `compileOnly`, and that artifact is no longer published. `bentobox-itemsadder-stub/` holds a compile-only stub of the two classes BentoBox uses. The hook only runs when ItemsAdder is installed.

```bash
cd bentobox-itemsadder-stub
javac --release 21 -cp "<paper-api jar>;<adventure jars>;<jetbrains annotations>;<guava>" -d out src/dev/lone/itemsadder/api/*.java
jar cf api-itemsadder-4.0.2-beta-release-11.jar -C out .
# install it into ~/.m2/repository/dev/lone/api-itemsadder/4.0.2-beta-release-11/ with a minimal pom
cd ../BentoBox-Drake
./gradlew shadowJar -x test -I ../Pumpkin-launcher/fork/bentobox-itemsadder-stub/init.gradle
```
