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
- [ ] Decorative asteroid fields in select sectors (reuse existing asteroid system)
- [ ] Design + seed remaining station sizes/progress states across 10 sectors
- [ ] Verify all 10 sectors reachable via jumpgate network
- [ ] Galaxy Creator privileged client (#34)

---

## Milestone: M1 — Shared-Building Spike

- [ ] `contribute_to_station` reducer (server-side)
- [ ] StationUnderConstruction subscription on client (already subscribed — verify)
- [ ] Construction-site UI: progress bar, contributions panel, deposit cargo button
- [ ] Admin/seed reducer: spawn test cargo in player inventory
- [ ] Completion event broadcast (chat line, flash, or similar shared moment)
- [ ] Devlog post #1 about the spike

---

## Milestone: M2 — Persistence + Welcome-Back

- [ ] `client_connected` lifecycle reducer — compose welcome-back ServerMessage
- [ ] Welcome-back panel: client-side rendering of welcome-back message
- [ ] Notification scopes (personal/faction/system) + priorities
- [ ] Persistence smoke test: stations + contributions survive server restart
- [ ] Single-player offline state preservation

---

## Milestone: M3 — Two-Faction MVP Setup

- [ ] Hard-cap joinable factions to Lrak Combine + Rediar Federation
- [ ] Player chooses faction on first login
- [ ] Enforce one Column corvette per player; reject Phalanx/Javelin
- [ ] Faction-flag UI affordance: highlight own-faction construction sites
- [ ] Verify each faction has one Capital-class station
- [ ] Verify new players spawn at their faction's Capital station
- [ ] Faction chat end-to-end verification (#19)

---

## Milestone: M5 — Mining Loop + Polish

- [ ] Mining loop end-to-end verification (target asteroid → mine → cargo → haul → contribute → progress)
- [ ] Mining visual effects (#81)
- [ ] Generalized visual_effects for mining-laser broadcast (#87)
- [ ] General "feel good" polish: completion animations, contribution feedback, sound cues (spike)

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

- [ ] Remove `ship_status_timer_reducer` identity-mismatch noise from logs
- [ ] Fix client version-check TODO (make `global_config` public)
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
