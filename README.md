<div align="center">

# Solarance: Beginnings

<img src="./client/assets/Solarance_Logo.png" alt="Solarance:Beginnings Logo. Text over a black planet with a pure white horizon lit up as a cresent in the bottom left of the circle." />

**A cozy, persistent, co-op space sandbox for adults with jobs.**

[Discord](https://discord.gg/34xzCtsKxe) 🌌 [Bluesky](https://bsky.app/profile/galaxycr8r.bsky.social) 🌌 [Itch.io](https://galaxycr8r.itch.io/solarance-beginnings)

<img src="./misc/screenshot.png" width="50%" alt="A screenshot from v0.2.0, a ship just transitted a jump gate and a asteroid is nearby." />

</div>

Solarance is built for the player EVE Online traumatized and lost — the one who loved building a presence in a quiet corner of space with friends, and quit the day a PvP gank erased a month of work. There is no mandatory combat here. No way to be set back by another player. Your progress waits for you whether you have 20 minutes or a Saturday afternoon, and when you come back it will still be there.

The core loop is **find → extract → haul → contribute → watch it grow.** Mine ore, haul it to a partially-built station, deposit it. Watch the station's progress bar tick up. See another player's ship dock while you're there. You both know what you're doing. Log off. Come back tomorrow.

The climactic beat is collaborative: *my pile got bigger, other people's piles got bigger, and together we made a thing.*

> **The project has shifted scope.** Earlier versions targeted a combat-and-exploration MMO. That vision is shelved. The MVP is now a cozy co-op builder with zero combat. See [`agents/milestones/proposed-roadmap.md`](./agents/milestones/proposed-roadmap.md) for the original milestone plan and what's been completed.

---

## Running the Game

### Prerequisites

- **Rust** (latest stable) — [rustup.rs](https://rustup.rs/)
- **SpacetimeDB CLI** — `curl -sSf https://install.spacetimedb.com | bash`
- **Platform dependencies** for Macroquad graphics: [github.com/not-fl3/macroquad#linux](https://github.com/not-fl3/macroquad#linux)
- **Taskfile** (optional) — [taskfile.dev](https://taskfile.dev/)

### Quick Start

1. **Clone and configure:**

   ```bash
   git clone https://github.com/RESTful-Otaku/solarance-beginnings.git
   cd solarance-beginnings
   cp client/.env.template client/.env
   ```

2. **Set your SpacetimeDB instance** in `client/.env`:
   - `https://maincloud.spacetimedb.com` — public test instance (may lag behind `main`)
   - blank or `localhost` — run your own local server (recommended for development)

3. **Run:**

   With Taskfile:
   ```bash
   task start                  # start SpacetimeDB (separate terminal)
   task server:publish-clear   # build + publish server, clear DB
   task client:run-full        # generate bindings + run client
   ```

   Manually:
   ```bash
   spacetime start
   cd server && cargo build && spacetime publish solarance-spacetime-module && cd ..
   cd client && cargo run --release
   ```

### First Session

- Create an account and choose a username when prompted
- Use WASD or arrow keys to move; hotkeys are shown in parentheses throughout the UI
- Fly to an asteroid sector, mine ore, haul it back to the nearest construction site

---

## Current State

The core loop is fully implemented: fly around sectors, mine asteroids, haul ore to stations, contribute to construction. Factions, stations, economy, jump gates, persistence, and multi-player all work.

**Previous milestones completed (see `agents/milestones/proposed-roadmap.md`):**
- M0: Foundational (persistence, players, ships)
- M1: Motion (movement controllers, sector physics)
- M2: Extraction (mining, cargo, jettison)
- M3: Construction (station modules, contributions, progress tracking)
- M4: Economy & Services (trading, refueling, repairs, selling)

**Known limitations (work in progress):**
- Combat system exists but is gated — MVP is zero-combat cozy builder
- Shield/energy regen from modules is a stub (flat rate defaults apply)
- Version check against server identity is deferred (blocked by private table)
- Star system transitions use direct teleport instead of animated slide

---

## Contributing

Issues and milestones are tracked on [GitHub](https://github.com/RESTful-Otaku/solarance-beginnings/milestones).

## License

All code is GPL 3.0. Art assets are withheld — that may change when new artwork arrives.
