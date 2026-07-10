# AGENTS.md — Project Context & Guidelines

> Living document: update whenever you discover project conventions, patterns, or constraints not captured here.

---

## Project Overview

- **Solarance Beginnings** — A cozy multiplayer space game (SpacetimeDB + Macroquad Rust client)
- **Server**: `server/` — SpacetimeDB module (Rust, `spacetimedsl` crate)
- **Client**: `client/` — Macroquad game (Rust, `spacetimedb-sdk`)
- **Repo**: `https://github.com/RESTful-Otaku/solarance.git` (origin)
- **Upstream**: `https://github.com/GalaxyCr8r/solarance-beginnings.git` (fork source)
- **Design doc**: `docs/` (devlog + website); MVP doc referenced as `Solarance_Beginnings_MVP_Design_Doc.md` but not on disk

---

## Architecture

### Three Layers (must stay separate)
```
tables/       — Schema only. No logic.
logic/        — Reducers + helpers.
definitions/  — Seed data for init.
```

### Deep Modules
Each module should have a simple interface and a complex implementation. The `spacetimedsl` crate is the canonical deep module example — callers get typed ID wrappers, CRUD methods, and generated accessors without knowing about index lookups or deserialization.

### The DSL
- Custom `spacetimedsl` crate wraps SpacetimeDB with code generation
- `#[dsl(plural_name = ..., method(update = true/false))]` paired with `#[table(accessor = ...)]`
- Use `dsl(ctx)` to get a DSL handle in reducers
- Helper functions typed `&DSL<'_, ReducerContext>` (not generic unless needed across reducer+procedure)
- ID wrappers: `PlayerId::new(identity)`, `SectorId`, etc.
- `Create<Table>` structs for insertion, generated CRUD for reads/updates/deletes

### SpacetimeDB SDK Rules
- `#[table(accessor = name)]` NOT `name = "name"`
- `#[table]` NOT `#[derive(SpacetimeType)]` on tables
- `#[spacetimedb::reducer]` (full path, project convention)
- `&ReducerContext` NOT `&mut ReducerContext`
- Tables are methods: `ctx.db.player()` not `ctx.db.player`
- Index access: `ctx.db.player().id().find(&id)`
- Add `public` flag for client-subscribed tables

---

## Project Structure

```
/root/
├── server/          — SpacetimeDB module
│   ├── src/
│   │   ├── tables/        — Schema (DSL table defs)
│   │   ├── logic/         — Reducers + helpers
│   │   ├── definitions/   — Seed data (init)
│   │   ├── admin/         — Admin privileged operations
│   │   └── lib.rs         — Module entry
├── client/          — Macroquad game
│   ├── src/
│   │   ├── gameplay/
│   │   │   ├── render/    — All rendering (star_system, in_sector)
│   │   │   ├── gui/       — UI panels (map_window, etc.)
│   │   │   ├── state.rs   — GameState
│   │   │   └── resources.rs — Texture/asset loading
│   │   ├── server/
│   │   │   └── bindings/  — Auto-generated SpacetimeDB client bindings
│   │   ├── stdb/          — SpacetimeDB connector utilities
│   │   ├── login.rs       — Auth/login flow
│   │   └── main.rs        — Client entry
│   ├── assets/            — Game assets (sprites, textures)
│   └── .env               — DATABASE_HOST config
├── spacetimedsl/     — DSL code-generation crate (not always present locally)
├── agents/           — Planning, analysis, milestone docs
├── docs/             — Devlog website source, ADRs
└── CLAUDE.md         — AI coding instructions (this file)
```

---

## Workflow

### Branch Strategy
1. **Main must always be stable** — do NOT commit directly to main
2. All work in feature branches: `feat/*`, `fix/*`, `chore/*`
3. Micro atomic commits per branch
4. Only merge to main when the branch is rock-solid and QA-verified

### Sync
```bash
git fetch upstream
git pull upstream main     # stay in step with fork source
git push origin main       # keep origin synced
```

### Build & Deploy
```bash
# SpacetimeDB server
spacetime start                              # start local
spacetime publish <name> --project-path server/  # deploy module
spacetime publish <name> --clear-database -y --project-path server/  # reset

# Client
cd client && cargo build                     # compile
cd client && cargo run --release              # run game

# Generate bindings (after server changes)
spacetime generate --lang rust --out-dir client/src/server/bindings --project-path server/

# Check logs
spacetime logs <name>
```

### Important Commands
- `spacetime server list` — check which server is default
- `spacetime identity` — get default identity
- `spacetime logs` — debug live game

---

## Known Issues & Workarounds

| Issue | Status | Workaround |
|-------|--------|------------|
| `ship_status_timer_reducer` identity mismatch in logs | Pre-existing, not blocking | Timer identity != server identity; cosmetic error |
| `docs/Solarance_Beginnings_MVP_Design_Doc.md` doesn't exist | Missing file | Referenced in roadmap but not on disk |
| Guest login (`DATABASE_HOST=localhost`) | Fixed | Must include `http://` scheme + `:3000` port |
| Combat system | Gated behind `combat_enabled` flag (default false) | `fire_weapons` returns "Combat disabled" |
| Cross-sector flicker (#89) | Fixed | Grace-window smoothing in render.rs |
| Client version-check | Fixed | `global_config` public, checked in login.rs |

---

## Client Rendering Pipeline

1. **Starfield** — Procedural GLSL shader, full-screen
2. **Star system background** — `render_star_system()` in `star_system.rs`
   - Draws stars, planets, moons, nebula belts (via bg camera, extreme zoom-out)
   - 3-stage distance-fade system for smooth transitions
3. **Sector nebula overlay** — `render.rs` after star system, before objects
   - Semi-transparent fog based on `sector.nebula` value (0.0–1.0)
   - Alpha clamped [0, 160], size covers 3× visible area
4. **In-sector objects** — `in_sector.rs`
   - Ships, stations, jump gates, asteroids, cargo crates
   - UC station texture dispatch via `StationUnderConstruction` table presence
5. **Radar** — Circle overlay showing nearby objects

---

## Milestone Status

| Milestone | Description | Status |
|-----------|------------|--------|
| M0 | Movement Critical-Path Fix | Complete |
| M1 | Shared-Building Spike | Not started |
| M2 | Single-Player Persistence + Welcome-Back | Complete |
| M3 | Two-Faction MVP Setup | Complete |
| M4 | Multi-Sector World Buildout | Complete |
| M5 | Mining Loop + Polish | Partial (core loop + VFX done, polish outstanding) |
| M6 | MVP Launch & Devlog | Not started |
| M7 | Anti-Cheat Hardening via Views | Not started |

See `agents/milestones/proposed-roadmap.md` for full detail.
See `agents/milestones/issue-disposition.md` for per-issue triage.

---

## Critical Files

| File | Purpose |
|------|---------|
| `server/src/tables/stations.rs` | Station + StationUnderConstruction + StationStatus schemas |
| `server/src/tables/global_config.rs` | `combat_enabled` flag, server identity |
| `server/src/definitions/galaxy.rs` | Sector/station/gate seed data |
| `client/src/gameplay/render/star_system.rs` | Star system background render |
| `client/src/gameplay/render/in_sector.rs` | In-sector object render |
| `client/src/gameplay/render.rs` | Main sector render dispatch |
| `client/src/gameplay/resources.rs` | Texture asset loading |
| `client/.env` | `DATABASE_HOST` for SpacetimeDB connection |
| `CLAUDE.md` | AI coding instructions & SpacetimeDB SDK reference |
