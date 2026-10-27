<div align="center">

# Pumpkin

![CI](https://github.com/Pumpkin-MC/Pumpkin/actions/workflows/rust.yml/badge.svg)
[![Discord](https://img.shields.io/discord/1268592337445978193.svg?label=&logo=discord&logoColor=ffffff&color=7389D8&labelColor=6A7EC2)](https://discord.gg/wT8XjrjKkf)
[![License: GPL](https://img.shields.io/badge/License-GPLv3-yellow.svg)](https://opensource.org/licenses/gpl-3-0)

</div>

[Pumpkin](https://pumpkinmc.org/) is a Minecraft server built entirely in Rust, offering a fast, efficient,
and customizable experience. It prioritizes performance and player enjoyment while adhering to the core mechanics of the game.
<div align="center">

![Pumpkin Chunk Loading](./assets/pumpkin-chunk-loading.webp)

</div>

## Goals

- **Performance**: Leveraging multi-threading for maximum speed and efficiency.
- **Compatibility**: Supports the latest Java & Bedrock Minecraft server version while adhering to Vanilla game mechanics.
- **Security**: Prioritizes security by preventing known security exploits.
- **Flexibility**: Highly configurable, with the ability to disable unnecessary features.
- **Extensibility**: Provides a foundation for plugin development.

> [!IMPORTANT]
> Pumpkin is currently under heavy development.
>
> [See what needs to be done before the 1.0.0 Release](https://github.com/Pumpkin-MC/Pumpkin/issues/449)

## Features

- [x] Configuration (toml)
- [Tracking: Protocol](https://github.com/Pumpkin-MC/Pumpkin/issues/1401)
  - [x] Server Status/Ping
  - [x] Encryption
  - [x] Packet Compression
  - [x] Java Edition
  - [x] Bedrock Edition (W.I.P)
  - ...
- [Tracking: World](https://github.com/Pumpkin-MC/Pumpkin/issues/1403)
  - [x] Player Tab-list
  - [x] Scoreboard
  - [x] World Loading
  - [x] World Time
  - [x] World Borders
  - [x] World Saving
  - [x] Lighting
  - [x] Entity Spawning
  - [x] Bossbar
  - [x] Chunk Loading (Vanilla, Linear, Pump)
  - [Chunk Generation](https://github.com/Pumpkin-MC/Pumpkin/issues/36)
  - [x] Chunk Saving (Vanilla, Linear, Pump)
  - [Redstone](https://github.com/Pumpkin-MC/Pumpkin/issues/1402)
  - [x] Liquid Physics
  - ...
- [Tracking: Player](https://github.com/Pumpkin-MC/Pumpkin/issues/1405)
  - [x] Skins
  - [x] Teleport
  - [x] Movement
  - [x] Animation
  - [x] Inventory
  - [Combat](https://github.com/Pumpkin-MC/Pumpkin/issues/1404)
  - [x] Experience
  - [x] Hunger
  - [X] Off Hand
  - [X] Advancements (W.I.P)
  - [x] Eating
  - ...
- Entities
  - [x] Non-Living (Minecart, Eggs...) (W.I.P)
  - [x] Entity Effects
  - [x] Players
  - [x] Mobs (W.I.P)
  - [x] Animals (W.I.P)
  - [Entity AI](https://github.com/Pumpkin-MC/Pumpkin/issues/1406)
  - [x] Boss (W.I.P)
  - [x] Villagers (W.I.P)
  - [X] Entity Saving
- Server
  - [Plugins](https://github.com/Pumpkin-MC/Pumpkin/issues/1407)
  - [x] Bukkit, Spigot, Paper and Purpur plugins through PatchBukkit (this fork, W.I.P)
  - [x] Query
  - [x] [Minecraft Server Management Protocol (MSMP)](https://minecraft.wiki/w/Minecraft_Server_Management_Protocol)
  - [x] Inventories
  - [x] Particles
  - [x] Chat
  - [Commands](https://github.com/Pumpkin-MC/Pumpkin/issues/15)
  - [x] Permissions
  - [x] Translations
- Proxy
  - [x] [BungeeCord](https://github.com/SpigotMC/BungeeCord)
  - [x] [BungeeGuard](https://github.com/lucko/BungeeGuard)
  - [x] [Velocity](https://github.com/PaperMC/Velocity)

<!-- Check out our [Github Project](https://github.com/orgs/Pumpkin-MC/projects/3) to see current progress. -->

## How to run

See our [Quick Start](https://docs.pumpkinmc.org/#quick-start) guide to get Pumpkin running.

## Changes in this fork

> [!NOTE]
> Pumpkin-launcher is a personal fork of [Pumpkin-MC/Pumpkin](https://github.com/Pumpkin-MC/Pumpkin). Nothing below is merged upstream yet. The PatchBukkit patches and test scripts live in [`fork/`](fork/).

### Server

- **Native plugins no longer crash the server.** Native plugins link their own copy of `pumpkin-core`, including `arc_swap`, whose reader tracking is a global static. When plugin code swapped an `ArcSwap` while the server was reading it, the old value was freed under the reader (`0xC0000005`). The event handler map, `Server::command_dispatcher`, `Server::worlds`, `World::players` and `World::entities` now use `plugin::SharedArcSwap`, a `RwLock<Arc<T>>` with the same `load`/`rcu`/`store` API. `PLUGIN_API_VERSION` is now 4, so native plugins must be rebuilt.
- **`stop` no longer hangs when stdin is not a terminal** (Docker, panels, piped input). The stdin console task now also waits on the server's stop token. This was a regression from #3771.
- **Powered and activator rails update their power like vanilla** (`PoweredRailBlock.updateState`). A redstone source next to an already powered rail no longer extends power past the 8 rail reach. Covered by the `pumpkin:powered_rail_mid_line_source_does_not_extend` game test.
- **Block entities are saved before their chunks can unload.** On logout and after a long teleport, chest and other block entity contents were saved after the player's chunk tickets were gone, so a chunk that unloaded in between lost the write.
- **A player who disconnects drops the cursor stack and the 2x2 crafting grid** where they stood, like vanilla `Player.remove`. Covered by the `bot:disconnect_drops_cursor_and_crafting_grid` tester check.
- **Wasm plugins can no longer crash the server through a packet field.** A `destroy-stage` of 255 is sent as vanilla's -1 (it clears the crack) instead of panicking, other plugin values that do not fit drop the packet, and `PacketReceivedEvent` decodes Java packets as play packets only, so a play packet is no longer read as a configuration packet with the same id.
- **A plugin's per-player scoreboard no longer disconnects a joining player.** Changes made before the join sent the full scoreboard are stored and go out with it, instead of reaching the client twice.
- **Hitting the ender dragon or an entity that just died no longer kicks the player.** Attacks resolve the dragon's parts like vanilla `ServerLevel.getEntityOrPart`, and the dragon takes a quarter of the damage (plus up to 1) on any part but the head and neck. An attack on an entity the server no longer has, such as a creeper that just exploded, is ignored as in vanilla. Covered by the `bot:attack_unknown_entity_and_dragon_part` tester check.
- **The first Nether (or End) visit can no longer freeze that dimension's chunk generation.** Outside the overworld the stronghold rings were searched with Rayon from inside a generation worker, which could pick up another generation job that waited on the same search. The search now runs on its own thread and pool.
- **Campfires and jukeboxes no longer duplicate the held item** (fixed by the integrated upstream change that writes block interaction results back to the hand). Covered by the `bot:block_use_takes_held_item` tester check.
- **Splash potion harm is attributed to the thrower**, like vanilla `HealOrHarmMobEffect` (`indirectMagic(potion, owner)`), so kill credit and plugins' `EntityDamageByEntityEvent` see who threw it. Covered by the `bot:splash_potion_harm_attributed_to_thrower` tester check.

### Running Bukkit, Spigot and Paper plugins

Plugins written for Bukkit, Spigot, Paper or Purpur run through [PatchBukkit](https://github.com/Pumpkin-MC/PatchBukkit), which embeds a JVM with a headless Paper 26.3.

- Build PatchBukkit against this checkout. Point its `rust/Cargo.toml` at the local crates with a `[patch."https://github.com/Pumpkin-MC/Pumpkin.git"]` section and start from this repo's `Cargo.lock`, otherwise the plugin and the server disagree on type layouts.
- Java 25 is required (`JAVA_HOME`).
- Add `extra-args = ["-Dterminal.jline=false"]` to `patchbukkit.config.toml` when stdin is not a terminal. Without it, Paper's console hangs the JVM on startup.
- Bukkit plugins go in `plugins/data/patchbukkit/patchbukkit-plugins/`.

Local PatchBukkit fixes (not upstream yet):

- Plugin classes are rewritten by Paper's legacy support (Commodore), as on Paper, so plugins built for older API versions get renamed fields and legacy materials translated.
- Classes resolve from the server first, then the plugin's jar, then other plugins, like Paper. A library shaded unrelocated in one plugin no longer shadows the server's copy for every plugin.
- Plugin classes carry a `CodeSource` pointing at their jar, so plugins can locate and read their own jar.
- Syncing the world clock from Pumpkin no longer fires a fake `TimeSkipEvent`.
- `TimeSkipEvent.SkipReason`, moved in Paper 26.x, is remapped for older plugins.
- Only top-level jars in the plugins folder are loaded, so BentoBox addons and downloaded libraries are not loaded as plugins.

### Upstream pull requests

100 open pull requests from Pumpkin-MC/Pumpkin are applied here, each as one commit with its original author and the pull request number in the subject (`git log --grep '(#'`). They were picked because they fix bugs, apply on this branch and pass the checks below. When a pull request needed a small lint fix to build here, the fix is folded into its commit.

### Fork progress

Done:

- [x] Server builds, starts and stops on Windows, also with piped stdin
- [x] Bukkit, Spigot, Paper and Purpur plugins through PatchBukkit, including legacy plugins (Commodore rewrites, `CodeSource`, server-first class loading)
- [x] Native plugins no longer crash the server (`SharedArcSwap`)
- [x] 60-plugin load test: SlimefunNewHorizons releases, BentoBox with BSkyBlock, LuckPerms, Vault, EssentialsX
- [x] Powered and activator rails like vanilla
- [x] Block entities saved before their chunks unload
- [x] Cursor stack and crafting grid dropped on disconnect
- [x] Wasm plugin packet fields and received packets no longer panic
- [x] Per-player scoreboards no longer kick joining players
- [x] Ender dragon parts can be hit, and attacks on removed entities no longer kick
- [x] Nether and End chunk generation no longer deadlocks on stronghold rings
- [x] Campfire and jukebox item duplication
- [x] Splash potion harm attributed to the thrower
- [x] Wasm plugin reentry deadlock (#4162)
- [x] 100 upstream bug fix pull requests integrated
- [x] Typos CI no longer checks the PatchBukkit patch files

Still to do:

- [ ] Integrate the remaining upstream fix pull requests: 37 need a 3-way merge and about 30 conflict with this branch (#4311 among them)
- [ ] Large upstream changes left out on purpose: the mob behavior rewrite (#4107) and the tick scheduler change (#3863)
- [ ] Bugs not reproduced here: duplicate entity UUIDs (#3272), chat changing the tab list name (#3379), level particles decode error (#3777)
- [ ] Bugs that need a Bedrock client or a TypeScript Wasm plugin: #2981, #2104, #3396, #3702
- [ ] Area effect clouds do not remember their owner, so their harm is not attributed yet; vanilla's integer rounding of potion damage is not ported
- [ ] Outside the overworld, vanilla skips stronghold placement entirely; Pumpkin still computes the rings
- [ ] Measure tick time with many entities after the `SharedArcSwap` change
- [ ] Play test the fixes with a real client
- [ ] The 11 Bukkit plugins that fail for their own reasons (missing classes in Slimefun4-Drake's compatibility layer, invalid `plugin.yml`, missing libraries)

### Test status

- 200 game tests and 16 bot checks (`cargo run -p pumpkin-tester`) pass, plus 617 `pumpkin-core`, 306 `pumpkin-world`, 157 `pumpkin-protocol` and 84 `pumpkin-util` unit tests. The `goal_selector` reported-goal test is timing sensitive and can fail under full parallel load.
- PatchBukkit conformance plugin: 543 of 547 Bukkit API checks pass.
- 59 Bukkit plugins and the BSkyBlock addon loaded together (the SlimefunNewHorizons organization releases, BentoBox, LuckPerms, Vault, EssentialsX): 48 enable (46 cleanly, EssentialsX and Odysseia with warnings) and the server starts and stops without crashing. The other 11 fail for their own reasons, such as addons built against classes that Slimefun4-Drake's compatibility layer does not provide, invalid `plugin.yml` files or missing libraries.
- Not tested with a real client: gameplay of these plugins, and tick time with many entities after the `SharedArcSwap` change.

## Contributions

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md)

## Docs

Pumpkin's documentation can be found at <https://pumpkinmc.org/>

## Communication

Consider joining [our Discord server](https://discord.gg/wT8XjrjKkf) to stay up-to-date on events, updates, and connect with other members.

## Funding

If you want to fund me and help the project, check out the [Donation Page](https://pumpkinmc.org/donate/).

## License & Attribution

* **Pumpkin Server**: Licensed under the [GNU General Public License v3.0 (GPLv3)](LICENSE).
* **Plugin API (`pumpkin-plugin-api` & `pumpkin-plugin-wit`)**: Dual-licensed under [MIT](crates/pumpkin-plugin-api/LICENSE-MIT) OR [Apache-2.0](crates/pumpkin-plugin-api/LICENSE-APACHE) for maximum flexibility when writing plugins.
* **Third-Party Assets & Data**: Bedrock mappings, protocol conversion data, and Minecraft assets are subject to their respective licenses and attribution terms. See [assets/NOTICE.md](assets/NOTICE.md) for full details.
