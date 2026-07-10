# TASKS.md — Running Work Checklist

> Update as tasks are discovered, started, completed, or blocked.

---

## Legend

- `[ ]` — pending
- `[~]` — in progress
- `[x]` — completed
- `[!]` — blocked

---

## Milestone: Cross-Cutting / Infrastructure

- [x] Server + client crates build with 0 warnings, 0 errors
- [x] Guest login fixed (`.env` scheme/port, connector URI guard)
- [x] Local main synced to origin
- [x] Upstream pulls caught up (rebased onto upstream/main)
- [x] Nebula textures loaded (8 PNGs into `Resources.nebula_textures`)
- [x] UC station textures loaded (6 PNGs into `Resources.station_textures`)

---

## Milestone: M4 — Multi-Sector World Buildout

- [x] Sectors seeded (10 sectors, IDs 0–9)
- [x] Jumpgate network wired (hub/spoke topology)
- [x] Nebula belt star system object created in Procyon system
- [x] NebulaBelt rendering (star_system.rs — draws nebula texture at belt position)
- [x] Sector-level nebula fog overlay (render.rs — semi-transparent fog from sector.nebula)
- [x] UC station texture dispatch (in_sector.rs — checks StationUnderConstruction table)
- [x] Decorative asteroid fields in 5 previously-empty sectors
- [x] Outpost stations added for Stilwater, Quiet Belt, Pale Crossing (full sector coverage)
- [x] Verify all 10 sectors reachable via jumpgate network
- [ ] Galaxy Creator privileged client (#34)

---

## Milestone: M1 — Shared-Building Spike

- [x] `contribute_to_station` reducer (server-side)
- [x] StationUnderConstruction subscription on client (already subscribed — verify)
- [x] Construction-site UI: progress bar, contributions panel, deposit cargo button
- [x] Admin/seed reducer: spawn test cargo in player inventory
- [x] Completion event broadcast (chat line, flash, or similar shared moment)
- [x] Devlog post #1 about the spike -- M1-Complete section added to ROADMAP.md

---

## Milestone: M2 — Persistence + Welcome-Back

- [x] `client_connected` lifecycle reducer — compose welcome-back ServerMessage
- [x] Welcome-back panel: client-side rendering of welcome-back message
- [x] Notification scopes (personal/faction/system) + priorities
- [x] Persistence smoke test: stations + contributions survive server restart
- [!] Single-player offline state preservation — deferred; requires client-side auth design

---

## Milestone: M3 — Two-Faction MVP Setup

- [x] Hard-cap joinable factions to Lrak Combine + Rediar Federation
- [x] Player chooses faction on first login
- [x] Enforce one Column corvette per player; reject Phalanx/Javelin
- [x] Faction-flag UI affordance: highlight own-faction construction sites
- [x] Verify each faction has one Capital-class station
- [x] Verify new players spawn at their faction's Capital station
- [x] Faction chat end-to-end verification (#19)

---

## Milestone: M5 — Mining Loop + Polish

- [x] Mining loop end-to-end verification (target asteroid → mine → cargo → haul → contribute → progress)
- [x] Mining visual effects (#81)
- [x] Generalized visual_effects for mining-laser broadcast (#87)
- [x] Contribution deposit flash feedback (green bar + "✓ Deposited!" on deposit)
- [ ] General "feel good" polish: completion animations, sound cues (spike)

---

## Milestone: M6 — MVP Launch & Devlog

- [ ] Continue monthly devlog cadence
- [ ] Landing page polish (`solarance-beginnings.com`)
- [ ] MVP smoke test session with welcome-back verification
- [ ] Final round of bug-fixing in core loop

---

## Milestone: M7 — Anti-Cheat Hardening via Views

- [ ] Per-sector visibility scoping via SpacetimeDB views (#84)
- [ ] Refactor Stations to use Views instead of RLS (#75a)
- [ ] Reference implementation from `docs/tmp/views/`

---

## Polish / Tech Debt / Opportunistic

- [x] Remove `ship_status_timer_reducer` identity-mismatch noise from logs — fixed: `try_server_only` now allows database identity
- [x] Fix client version-check TODO (make `global_config` public)
- [x] Cross-sector flicker fix (#89) — grace-window smoothing on jumpgate transit
- [x] Wire Settings button in out-of-play screen + menu bar
- [x] Decorative asteroid fields seeded (5 sectors previously empty)
- [x] Station diversity improved (3 new outposts for full sector coverage)
- [x] Contribution deposit flash feedback (green progress bar + confirmation text)
- [x] Makefile created (check, lint, build, publish, reset-db, bindings targets)
- [x] Server clippy suppressions added (pre-existing lints in generated code)
- [x] Modules granted on construction completion (#179) — name-keyword matching for refineries, trading posts, outposts
- [x] Ore type shown when targeting asteroid (#176)
- [x] `ship_status_timer_reducer` identity-mismatch noise fixed (`try_server_only` allows database identity in ST 2.6.0)
- [ ] Improve radar object distance readability (replaces #51)
- [ ] Remove unused `background_gfx_key` or implement it
- [ ] Client-side input prediction + reconciliation (#85) — revisit if sluggish
- [ ] Verify all existing branches have been merged or retired

---

## Future Vision (deferred)

- NPC auto-mining/trade/guard (#18)
- Multiple ship hulls per player
- Combat / weapons / shields
- Ship destruction → asteroid loot (#22)
- Faction reputation system
- Research trees
- Markets / dynamic pricing
- Player organizations within factions
- Emotes (#49)
