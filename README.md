# Fly

A Horizon native mod: **survival flight, one key away**. Press **R** (configurable in the
controls, like freecam's F4) to toggle creative-style flight anywhere — in survival, in any
world, on any server. Press it again to land.

- While the toggle is on, the flight grant is re-asserted every tick, so it survives world
  re-joins, mode changes and the server's ability packets — the flight stays on until *you*
  turn it off.
- Toggle off and you land like vanilla (pair it with the soft landing mod if you fall from high).
- Creative and spectator already fly; the mod stands down there and lets the game do its own
  thing.

## The honest fine print

- On a **single-player** world it just works.
- On a **multiplayer** server the movement is yours to make and the server judges it: Horizon's
  own anticheat runs by default (`ground` kicks false ground claims; the `prediction` check
  flags flight but only logs for now). On your own server, turn the anticheat down in
  `server.toml` (`[anticheat] enabled = false`) and fly freely. On someone else's server, flying
  may get you flagged or kicked — that is between you and the server, as with any flight.
- The flight speed is the game's own creative flight speed.

## Settings

- **Enabled** — the one row in Options → Mods (on by default; the files live under
  `horizon-mods/.horizon-mod-data/fly/` in the game's data dir).
- **Fly key** — `key.fly.toggle` in the controls (default `R`).

## Build

The repo is a Horizon native-mod workspace: `python3 -I tools/test.py` runs the brain's tests,
`tools/horizon.py` stages/checks/builds/tests against the pinned Horizon revision
(2ebf00a0281c592f79191e76fce0e7d522c6f425), and `tools/package.py` writes the signed-ready
`.hnmod`. See the tools' `--help`-ish headers for the chain.

Clean-room: written for Horizon from the game's own behavior; it copies nothing from any mod.

## License

MIT — Copyright (c) 2026 hakergeniusz
