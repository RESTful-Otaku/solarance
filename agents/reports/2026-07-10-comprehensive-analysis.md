# Comprehensive Analysis — Solarance: Beginnings

**Date**: 2026-07-10
**Reviewer**: opencode (glm-5.2)
**Scope**: Full codebase, project, and product — `server/`, `client/`, `client-admin/`, `solarance-shared/`, build/lint state, git state, design docs.
**Method**: Live builds + clippy, two exhaustive explore-agent code reviews (server + client), direct verification of the highest-severity findings, risky-pattern scans, test run, git archaeology.

---

## 0. Executive Summary

Solarance: Beginnings is a **cozy, server-authoritative, no-combat co-op space builder** on SpacetimeDB + Macroquad/egui. The core loop (find → extract → haul → contribute → watch it grow) is implemented end-to-end and the milestone plan (M0–M5) is largely delivered. The **dead-reckoning movement** (the historically hardest part) is well-built and the only properly-tested code in the repo (11 passing tests in `solarance-shared`).

The gap between "working MVP" and "AAA polish" is **not content breadth** — the design doc explicitly forbids combat, NPCs, markets-as-mechanic, research, and multi-system as Future Vision. The gap is **foundational correctness, security, performance, and feel**. Specifically:

1. **Two critical security holes** make the server un-shippable as-is: an unregistered client is treated as the *server* by `try_server_only`, and `register_playername`/`create_player_controlled_ship` trust a client-supplied `identity` instead of `ctx.sender()`. Any connected client can impersonate, pre-register, spawn cargo into others' ships, and call admin reducers.
2. **An infinite-credits economy exploit** (buy-low / sell-high against the same station using one shared price) and **integer-overflow** in the price math make the trading layer unsound.
3. **Zero tests** outside the shared physics crate — the single biggest engineering-quality gap and a direct violation of the project's own Tenet 5 ("tests are the spec").
4. **O(N²) per-frame ship lookups** and **per-frame collect+sort across every open GUI window** will cause stutter at exactly the moment multiplayer "feels" should shine (busy sector, long chat history).
5. **Crash-on-disconnect / crash-on-missing-asset / deadlock-risk in OIDC** make the client fragile.
6. **Scope discipline is the project's superpower and its risk**: the roadmap is admirably tight, but "AAA" for this game means *depth and feel within the loop*, not breadth. Pushing for combat/markets would betray the vision that makes the project distinctive.

The action plan at the end is ordered: **stop the bleeding (security) → fix correctness (economy, crashes) → build the safety net (tests) → performance → feel/polish → content depth**.

---

## 1. Methodology & Hard Evidence

| Check | Command | Result |
|-------|---------|--------|
| Server compile | `cargo clippy` in `server/` | ✅ builds, **35 clippy warnings** (down from 101 in the prior analysis) |
| Client compile | `cargo clippy` in `client/` | ✅ builds, **8 clippy warnings** |
| Shared tests | `cargo test` in `solarance-shared/` | ✅ **11 passed, 0 failed** |
| Server tests | — | **0 tests** |
| Client tests | — | **0 tests** |
| Risky patterns (excl. generated bindings) | `rg` | **54 `unwrap()`, 11 `expect(`, 1 `unreachable!`, ~206 `as` casts** |
| TODO/FIXME | `rg` | **20** across server logic + tables |
| Toolchain | `cargo 1.96.1`, `spacetime 2.6.1` available | project pins `spacetimedb 2.6.0`; **2.6.1 upgrade pending** |
| Git state | `git status` | **21 modified + 2 untracked** (`settings_window.rs`, `Makefile`) on `main`, uncommitted |
| Branches | `git branch -a` | local `feat/station-shields`; **50+ stale remote branches** (cleanup overdue) |
| Workspace | no root `Cargo.toml` | server/client/admin/shared are **independent crates**, not a workspace |
| LOC | `wc -l` | server 12.9k, client 23k, client-admin 16.6k (mostly generated), shared 743 |

The two explore-agent reports (server module-by-module, client module-by-module) are the backbone of the findings below; every item is cited with `file:line`. The three highest-severity items were **re-verified by direct read** before inclusion.

---

## 2. Critical — Security (ship-blockers)

### C1. `try_server_only` fails open for any unregistered identity — privilege escalation
**File**: `server/src/utility.rs:42-44` (verified directly)

```rust
// 3. Database identity (SpacetimeDB >= 2.6.0): no `Player` row exists for
//    the database identity, so treat it as a system caller.
if dsl.get_player_by_id(PlayerId::new(sender)).is_err() {
    return Ok(());
}
```

The intent (allow the SpacetimeDB *database identity* to fire scheduled reducers) is reasonable, but the implementation is **"any identity with no Player row is the server."** A brand-new client that has connected but not yet called `register_playername` has no `Player` row → `try_server_only` returns `Ok` → that client is treated as the server for **every** `try_server_only`-guarded reducer.

**Blast radius (all guarded only by `try_server_only`):**
- `admin_spawn_cargo_in_player_ship` (`admin/cargo.rs:27`) — spawn arbitrary cargo into *any* player's ship.
- `admin_create_sector`, `admin_place_station`, `admin_create_construction_site`, `admin_add_station_module`, `admin_reset_construction_site` (`admin/creation.rs`, `admin/construction.rs`) — re-write the galaxy.
- `admin_send_direct_server_message`, `admin_send_direct_server_message_to_group` (`admin/messages.rs`) — impersonate the server to players.
- All scheduled reducers guarded by `try_server_only` (`cargo_crate_despawn_sweeper`, `sector_upkeep`, `ship_status_timer_reducer`, faction timers).

Worse, `is_server_or_ship_owner` (`utility.rs:80`) is `try_server_only(dsl).or_else(...)`. An unregistered client passes `try_server_only` and **never reaches the owner check**, so it is treated as server for:
- `buy_item_from_station_module`, `sell_item_to_station_module` (`buy_and_sell.rs:27`-style) — trade *using another player's credits*.
- `jettison_cargo_from_ship` (`cargo.rs`) — dump another player's cargo.
- `undock_ship`, `use_jumpgate` paths.

**Fix**: replace branch 3 with an explicit allowlist of system identities. Store the database/publisher identity in `GlobalConfig` at `init` (server_identity is already there) and compare against it; do **not** infer "server" from "no Player row." Alternatively, gate admin reducers on `ctx.sender() == config.server_identity` exclusively, and gate scheduled reducers on `sender == Identity::default() || sender == config.server_identity`. Add a test that an unregistered identity is denied.

### C2. `register_playername` trusts a client-supplied `identity`
**File**: `server/src/logic/players/registration.rs:20-25, 28, 82-88` (verified directly)

```rust
pub fn register_playername(
    ctx: &ReducerContext,
    identity: Identity,   // ← client-supplied, NOT ctx.sender()
    username: String,
    faction_id: u32,
) -> Result<(), String> {
    ...
    if dsl.get_player_by_id(PlayerId::new(identity)).is_ok() { ... }   // uses param
    ...
    dsl.create_player(CreatePlayer { id: identity, ... })?;          // uses param
```

`ctx` is available but `ctx.sender()` is never used. A client can register a `Player` row for **any** identity — pre-claiming identities that haven't connected, impersonating, or poisoning the `is_server_or_ship_owner`/`try_server_only` logic. The same pattern repeats in `create_player_controlled_ship` (`logic/ships/creation.rs:86` per agent), which uses the passed `identity` for the one-ship-per-player check and ship ownership, and uses the passed `username` in the galaxy-chat announcement — enabling **chat impersonation**.

**Fix**: remove the `identity` (and `username`) parameter; use `ctx.sender()` and derive the username from the `Player` row. Reject if a `Player` already exists for `ctx.sender()`.

### C3. Scheduled reducers with **no auth at all**
**Files**: `server/src/logic/stations/production.rs:16-22`, `server/src/logic/stations/status.rs:15-21`, `server/src/logic/combat/visual_effects.rs:30-48` (per agent)

`station_production_schedule_reducer` and `station_status_schedule_reducer` call **no** `try_server_only` — any client can invoke them with a constructed timer argument and force production/shield-regen ticks for any station. `cleanup_visual_effect` likewise has no auth (a client can delete any VFX row). These are lower-impact than C1 but compound it.

**Fix**: add `try_server_only(&dsl)?` (after fixing C1) to each scheduled reducer. Better: enforce auth centrally — see §5.1.

---

## 3. Critical — Economy Soundness

### E1. Infinite-credits buy-low/sell-high exploit
**File**: `server/src/logic/stations/buy_and_sell.rs` + `server/src/tables/stations.rs:358-387` (`calculate_current_price`) (verified via agent + buy path)

Both buy and sell use the **same** `cached_price`, and the pricing formula makes items **cheaper when inventory is high, more expensive when low**. The code itself acknowledges this:

```
// cache buy/sell prices separately   ← buy_and_sell.rs comment, acknowledged but unimplemented
```

Exploit loop: (1) buy N while station is well-stocked → low price, inventory drops; (2) price recalculates *upward*; (3) sell the same N back → receive the higher price; (4) net profit = (high − low) × N. Repeat forever. The station-credits check is **commented out** (`buy_and_sell.rs:302`), so stations have infinite credits to pay out.

### E2. Integer overflow in price math
**File**: `server/src/logic/stations/buy_and_sell.rs:115` (per agent)

`let total_price = item_listing.get_cached_price() * quantity;` is `u32 * u32`. At 1,000,000 × 5,000 this overflows `u32`; in release mode it **wraps to a tiny number** — the player pays nearly nothing for a huge order. The subsequent `total_price as u64` preserves the wrong value.

### E3. Buy/sell margins never enforced
**Files**: `trading_port.rs:24-35` defines `buying_margin: Option<f32>` / `selling_margin: Option<f32>` (None = not trading that side), but `buy_item_from_station_module` and `sell_item_to_station_module` **never read them**. A player can buy an item the station isn't selling and sell an item it isn't buying. `create_basic_bazaar` sets `selling_margin: None` for *all* items — so by design no station sells, yet the buy reducer still works.

### E4. Free-item edge via negative price cast
**File**: `server/src/tables/stations.rs:386` (per agent)

`(value + margin_value * multiplier) as u32` — when `multiplier = -1.0` (full inventory) and `margin_value > value`, the float goes negative; `as u32` saturates to **0** → items become free when the station is fully stocked.

**Fix cluster**: separate buy/sell price caches; enforce `buying_margin`/`selling_margin` (None = refuse the trade); use `u64` (or checked `u128` intermediates) for price×quantity and reject on overflow; clamp price ≥ 1; restore the station-credits check (track station credits, not infinite).

---

## 4. High — Correctness Bugs

| ID | File:line | Bug |
|----|-----------|-----|
| B1 | `tables/ships.rs:161` | **u16 overflow** in cargo: `item.quantity * item_def.get_volume_per_unit()` — full stack × volume overflows u16 (release: silent wrap → corrupted cargo math). |
| B2 | `tables/ships.rs:147` | **u16 underflow**: `max - used` cargo capacity; if `used > max` transiently, wraps to a huge value → `can_any_of_this_fit_inside_this_ship` returns true erroneously. Use `saturating_sub`. |
| B3 | `logic/ships/status.rs:90-95, 97-102` | **Shield/energy equipment is worse than the flat default.** Equipped regen *replaces* the flat rate instead of adding. A 0.1/s shield module is worse than the 0.525/tick default. Logic inversion. |
| B4 | `logic/stations/module_types/refineries.rs:182-185` | **Refinery rate-limiting commented out** — a refinery with 10k ore produces 2,000 ingots in a single 30s tick. Balance break. |
| B5 | `logic/stations/mod.rs:372-396` | **Diplomacy module maxes all standings to 100.** Increments owner's standing toward *every* other faction +1/tick (30s). Trivially OP; everyone becomes allied. |
| B6 | `logic/ships/station_interactions.rs:145-196` | **`is_active` never checked on jumpgates.** A disabled jumpgate (`jumpgates.rs:33`) is still usable. Energy cost `100.0` is a hardcoded magic number. |
| B7 | `logic/sectors/asteroid_fields.rs:58` | **`gen_range(rarity..100)` panics if `rarity > 100`** (u8 max 255). Seeded sectors are ≤70, but `admin_create_sector` can set up to 255 → panic. |
| B8 | `logic/ships/cargo.rs` (`try_to_pickup_crate` via `attempt_to_pickup_cargo_crate`) | **No proximity/range check** on cargo pickup — a ship can grab a crate from anywhere in the sector (only same-sector is checked). |
| B9 | `logic/ships/weapons.rs` | **No self-target check** — a player can fire at their own ship. (Combat is gated off in MVP, but flag for when enabled.) |
| B10 | `logic/factions/mod.rs:201-245` | `update_faction_standing` is one-directional; if standings are created mutual, updating one side creates asymmetry. |
| B11 | `tables/factions.rs:153-159` | `get_faction_reputation` is a **full table scan** of `faction_standing` to find a pair — no index use. |
| B12 | `logic/players/welcome_back.rs:125` | Contribution summary does a full scan of `construction_contribution_log` filtered by time — acceptable at MVP scale, but unindexed. |
| B13 | `tables/global_config.rs:57` | `global_config_any_active_players` returns **true on error** (config missing) — fail-open. Should fail closed. |

---

## 5. High — Architecture & Engineering Quality

### 5.1 The three-layer rule is systematically violated
The project's central architectural invariant — **tables = schema only, no logic** — is broken across nearly every table file. Query helpers, creation logic, and game-balance formulas live in `tables/`:
- `tables/sectors.rs:5` imports `crate::admin::creation::create_jumpgate_internal` — **tables → admin** (circular, wrong direction).
- `tables/players.rs:69-86` `get_player_ship_and_sobj` — core query logic in schema.
- `tables/factions.rs:110-261` — 12 query helpers (one with a full-scan bug).
- `tables/stations.rs:326-387` — game-balance formulas (`calculate_base_cost`, `max_module_amount`, `calculate_current_price`) **and the economy pricing function** live in the schema layer.
- `tables/messages.rs:293-400` — 9 write/creation helpers.
- `tables/asteroids.rs:45-77` `create_asteroid` calls `create_sobj` from logic — **tables → logic** circular dependency.
- `tables/items.rs:199-202`, `tables/ships.rs:145-178` (`get_remaining_cargo_space` with the underflow/overflow bugs), `tables/stellarobjects.rs:46-57`.

This is the root cause of the circular imports and makes the schema layer untestable in isolation. **Fix**: move all `impl` query/creation/balance helpers into `logic/` (or a new `queries/` layer), leaving `tables/` as pure struct + attribute definitions. This also unblocks unit testing the logic.

### 5.2 No workspace, no CI, no `cargo test` at the root
- There is **no root `Cargo.toml`** — server/client/admin/shared are independent. `cargo build --workspace` (cited in the guide) does nothing. The `Makefile` (untracked) is the closest thing to automation.
- **No CI** (`.github/` has no workflows referencing build/test). Every PR claim is unverifiable.
- **No `cargo audit`** in the pipeline despite `openidconnect`/`reqwest` (auth + network surface).

**Fix**: add a root `[workspace]` with `members = [...]`; add a `.github/workflows/ci.yml` running `fmt --check`, `clippy -D warnings` (after fixing the 43 existing), `cargo test`, `cargo audit`. This is the lowest-effort, highest-leverage quality win.

### 5.3 Error-handling discipline
- Server reducers correctly return `Result<(), String>` (SpacetimeDB convention).
- But several paths **silently suppress errors** with `let _ = ...`: `station_interactions.rs:219,276,279`, `cargo.rs:157`. These hide failures that should at least be logged.
- The crate-level `#![allow(...)]` block (`server/src/lib.rs:5-24`) suppresses **17 clippy lints including `clippy::unwrap_used`** — every `unwrap()` in the server is silently allowed. This directly violates the guide's "no escape hatches without documented justification." Remove the blanket allow; justify or fix each unwrap individually.

### 5.4 Magic numbers everywhere
Game balance and tuning are hardcoded throughout instead of centralized in `GlobalConfig` (which already exists for this purpose):
- Starting credits `1000` (`registration.rs:84`), starting cargo rations/energy (`creation.rs:135-146`), ship type `1001`, equipment IDs (`creation.rs`).
- `DOCK_RANGE = 500.0`, `JUMPGATE_USE_RANGE = 300.0`, jumpgate energy `100.0` (`station_interactions.rs`).
- Shield/energy flat regen `0.525175` / `0.1275` per 500ms tick (`status.rs:66-67` — suspiciously precise, should be per-second in the type def).
- Station module counts `13/9/7/5/3/1`, costs `100_000/300_000/25_000/...` (`stations.rs:330-335`).
- Ore distribution ranges `0..25 → ICE`, etc. (`asteroid_fields.rs:60-66`), asteroid amounts `500..2000` (`asteroid_fields.rs:86`).
- `CONTRIBUTE_RANGE_PX = 300.0` (`contribution.rs:33` — at least mirrored client-side and documented).

**Fix**: a single tuning pass moving these into `GlobalConfig` (or a `Tunables` struct) makes the game live-tunable without redeploy and makes balance testable.

### 5.5 Dead code / empty files / stale branches
- **Empty files**: `logic/logic_utilities.rs`, `logic/game_loop.rs`, `logic/ships/movement.rs`, `logic/ships/lifecycle.rs` (no-op stubs).
- **Dead code**: commented-out `PlayerFactionStanding` table (`factions.rs:84-104`), `validate_combat_action` (`combat/visual_effects.rs:52` — not called), `background_gfx_key` (TASKS.md acknowledges), `direction_modifier` shader uniform + all `iTime` animation (`starfield.glsl` — starfield is static).
- **50+ stale remote branches** (`git branch -a`) — many from completed issues (issue-91, issue-100, issue-130...). Branch retirement is itself a TODO in TASKS.md.
- **`settings_window.rs` and `Makefile` are untracked** but fully wired in. Commit them.

### 5.6 Uncommitted work on `main`
21 files modified + 2 untracked on `main`, uncommitted. The project's own rule: "main must always be stable; do NOT commit directly to main; work in feature branches." This working tree is a feature branch's worth of changes sitting on main uncommitted. **Action**: decide whether this is M5 polish to commit to a branch, or stash. Don't leave `main` dirty.

---

## 6. High — Client Reliability & Performance

### 6.1 Disconnect = process death
`client/src/stdb/connector.rs:124-137` and `subscriptions.rs:185-191` call `std::process::exit(1)` on `on_disconnected`, `on_connect_error`, and `on_sub_error`. **Any** network blip kills the client with no reconnect, no "connection lost" screen, no retry. This is the most significant reliability gap for a *persistent* game whose whole pitch is "log off, come back tomorrow."

**Fix**: surface a "Connection lost — reconnecting..." egui overlay; attempt reconnect with backoff; keep `DbConnection` re-creatable. This is a meaningful chunk of work but core to the product promise.

### 6.2 O(N²) ship lookups every frame (fixable today)
The `ship` table **already has** a btree index on `sobj_id` (`tables/ships.rs:218`) and on `player_id` (`ships.rs:236`). Yet the client does linear scans:
- `stdb/utils.rs:44` `pose_for_object` for Ship: `db.ship().iter().find(|s| s.sobj_id == object.id)` — linear.
- `stdb/utils.rs:222` `get_ship_with_type`: `db.ship().iter().find(|s| s.sobj_id == sobj_id)` — linear.
- `stdb/utils.rs:182` `get_player_ship`: linear by `player_id && location == Sector`, called **~13×/frame** across UI widgets.

In a sector with N ships the render pass does ~2N linear scans of cost N → **O(N²)/frame**. With 50 ships that's 5,000 row visits/frame just for ship lookups. **Fix is mechanical**: `db.ship().sobj_id().find(&id)` and `db.ship().player_id().find(&id)` (filter by location afterward). Highest ROI perf fix in the codebase.

### 6.3 Per-frame allocations in GUI hot loops
Nearly every open window does `Vec::new()` + collect + (often) sort **every frame**, and several clone entire generated rows:
- `chat_widget.rs:91,226-282` — collects + sorts ALL messages per open tab per frame; history **never trimmed** (grows over session). O(n log n)/frame/tab. Cache sorted vec; re-sort on insert; cap history.
- `out_of_play_screen.rs:563` / `utils.rs:116` — clones every docked ship + sector + system into a nested HashMap every frame (called from 2 sites).
- `construction_window.rs:158-164` — O(requirements × contribution_logs)/frame.
- `faction_window.rs:198-203,256` — O(players)/frame for member counts; O(factions × standings)/frame.
- `map_window.rs:69-71,100,232-295` — collects all sectors/stations/gates into Vecs + HashMaps + HashSets every frame; O(sectors × stations) scan.
- `ship_details_window.rs:166-194` — `get_all_equipped_of_type` called **5×/frame**, each a linear scan; one pass grouped by type suffices.
- `minimap_widget.rs:58-74` — collect + sort all in-sector objects/frame.
- `render.rs:22,109`, `visual_effects.rs:71` — per-frame `Vec::new()` for `local_targets`/`ships_to_draw`/`expired_effects` (reuse via `GameState`-stored buffers or `retain`).

**Fix pattern**: cache derived collections in each window's `State`; recompute on an `on_insert`/dirty flag (SpacetimeDB SDK offers table callbacks), not every frame.

### 6.4 Cross-sector grace window is frame-rate dependent (#89 regression on high-Hz)
`render.rs:45` sets `sector_transition_grace = 3` (a **frame count**). On 144 Hz that's ~21 ms — likely shorter than new-sector row arrival → the exact flash-empty #89 was meant to fix. On 30 Hz it's ~100 ms. **Fix**: time-based (`Instant::now() + Duration::from_millis(150)`), not a frame counter.

### 6.5 Client-side state desync
- `state.rs:50` `mining_active` — set optimistically on `try_mining_asteroid`, never reconciled with server. If the server stops mining (asteroid depleted / out of range / energy out), the client keeps drawing the mining laser (`in_sector.rs:12` gates on this flag) and shows "Mining Beam: On." No server-pushed mining state is read.
- `state.rs:49` `combat_mode` — cosmetic client flag; server uses its own `combat_enabled`. Client shows "Combat" mode while the server silently rejects fire.

**Fix**: derive `mining_active` from the presence of the player's `ShipMiningTimer` row (already subscribed), not a client flag. Derive combat availability from `global_config.combat_enabled`.

### 6.6 Crash-on-missing-asset / shader / slice
- `login.rs:441` `Resources::new().await.unwrap()` and `main.rs:108-123` `load_texture(...).expect("Couldn't load assets")` — **panic** on any missing texture. A typo'd `gfx_key` from the server crashes the client.
- `shader.rs:32` `load_material(...).unwrap()` — shader compile failure crashes; should fall back to a solid background.
- `in_sector.rs:275,303,334,389,449` and `star_system.rs:31,47,49,54,57` — **HashMap indexing panics** on unknown texture keys; a server `gfx_key` not loaded client-side crashes the client. Use `.get(key).unwrap_or(&placeholder)`.
- `direct_server_messages.rs:112` — `[..16]` slice panics if RFC3339 < 16 chars.

**Fix**: a 1×1 magenta placeholder texture inserted at load; all lookups via `.get()`; log the missing key.

### 6.7 OIDC deadlock risk
`oidc_auth_helper.rs:63` `TcpListener::accept()` blocks forever if the user abandons the browser flow. Re-clicking "Login via Auth0" (`login.rs:230`) calls `.join()` on the still-blocked thread → **hangs the UI** (macroquad can't `next_frame` while `join` blocks). `oidc_auth_helper.rs:28` `discover(...).unwrap()` panics on any Auth0 network failure. **Fix**: `accept()` with a timeout; don't `.join()` a blocked thread (drop it / use a non-blocking channel); convert the unwraps to error returns surfaced to the login screen.

### 6.8 Accessibility gaps (keyboard)
- The controls help (`debug_widget.rs:72,73`) advertises **[C] dock/jump/undock** and **[Space] fire** as keyboard controls, but **no `KeyCode::C` or `KeyCode::Space` handler exists** (confirmed by grep). These are mouse-button-only. Keyboard-only players cannot dock/jump/undock/fire. Real gap — and the help text is actively misleading.
- No keyboard shortcut for the Settings window (other windows have R/F/T/M/B).
- Faction identity relies partly on red/green hue alone (`construction_window.rs:91-95`) — add a symbol/label for colorblind users.
- Panic screen (`gameplay.rs:69-75`) loops forever with no exit button.

---

## 7. Test Coverage (the headline quality gap)

**11 tests total, all in `solarance-shared/src/physics/tests.rs`** — and they're *good* tests (analytical comparisons, regression for the max-speed overshoot, damping edge cases). The server (12.9k LOC of game logic) and client (23k LOC) have **zero**.

This is the largest single violation of the project's own Tenet 5 ("Tests are the spec. Validation at every trust boundary.") and Tenet 7 ("Prove it"). The auth bugs in §2 would not have survived even a basic "unregistered identity is denied" test. The economy exploit in §3 would not have survived a "buy then sell returns to original credits" property test.

**Where tests pay off fastest (pure logic, no SpacetimeDB runtime needed):**
1. `compute_construction_progress` (`contribution.rs:55-82`) — already has 9 tests; this is the model to copy.
2. `calculate_current_price` (`stations.rs:358-387`) — the exploit, the negative-price cast, overflow.
3. Cargo math (`ships.rs:145-178`) — underflow/overflow, stacking.
4. Auth helpers (`utility.rs`) — deny unregistered, deny wrong owner, allow server.
5. `pick_weighted_ore` / `roll_ore_item` (`asteroid_fields.rs`) — already has one test; extend to the `rarity > 100` panic case.
6. `solarance-shared::predict_movement` — extend the existing suite with property tests (monotonic time, energy bounds).

**Integration**: SpacetimeDB supports testing reducers in-process; the auth and economy reducers are the priority integration tests.

---

## 8. Content & Gameplay Opportunities (within MVP scope)

> **Tension acknowledged**: "AAA with as much content, gameplay, fun, challenge" can read as a call for combat/NPCs/markets. The MVP design doc **explicitly forbids** those as Future Vision, and the project's distinctive value *is* the cozy no-setback promise. Adding combat would erase the thing that makes Solarance different from the EVE-likes it's reacting against. The recommendations below **deepen the existing loop** rather than extend it sideways — which is what "AAA polish" means for *this* game. The breadth items are listed at the end as explicitly-gated Future Vision, for you to decide.

### 8.1 Deepen "haul" — make the proto-economy bite (highest leverage)
The roadmap (M4) names the **construction-pool resource demand profile** as the highest-leverage differentiator: "what turns 'haul ore' into 'haul the *right* ore to the *right* place.'" This is the seed of the 5–10-player coordination that makes the loop social. Concrete moves:
- Give each station a **distinct, published demand profile** (not all stations want the same ore). The data model (`ConstructionRequirement`) supports this; the seed data and UI are the gap.
- Surface "this sector needs X, that sector needs Y" on the **galaxy map** and the **welcome-back** screen so a returning player knows where they're needed *before* they fly.
- **Quality-of-life**: an in-flight "what does my cargo satisfy?" overlay so players don't haul the wrong ore 10 minutes round-trip.

### 8.2 Deepen "watch it grow" — the emotional climax is under-sold
The climactic beat is collaborative: *my pile got bigger, other people's piles got bigger, and together we made a thing.* Today the feedback is a progress bar + a chat line. To make it land (and hit the M1/M5 exit gates honestly):
- **Completion moment**: a visible, shared, in-sector event when a station finishes — not just a chat line. A bloom/flash on the station, the UC texture swapping to the finished texture with a transition, a sector-wide notification. The VFX system (`VisualEffect`) already exists.
- **Contribution attribution**: "You contributed 12% of this station." A persistent per-player contribution ledger (`ConstructionContributionLog` exists) → a "builders" plaque on the finished station. This is the retention hook for "log in again tomorrow."
- **Stations that visibly change the sector**: a finished station should do something *visible* (lights, traffic, a trading-port that's actually stocked) so "watch it grow" is literal.

### 8.3 Deepen "find → extract" — make mining feel good (not a minigame)
The roadmap explicitly excludes a mining minigame (Future Vision). But "feel" is in-scope (M5 polish):
- **Mining audio cues** (M5 TODO, pending) — laser hum, extraction crack, asteroid-depleted thunk.
- **Asteroid depletion that's legible** — a visual shrink/crumble as `current_resources` drops, not just a number.
- **Ore-type clarity** — #176 (ore type in targeting) is done; extend to the radar/minimap so players *choose* which asteroid to mine.
- **Mining beam VFX already done (#81/#87)** — extend the same generalized VFX to the completion moment (8.2).

### 8.4 The "cozy" promise needs resilience (not just content)
The whole pitch is "your progress waits for you." That promise is currently **broken by §6.1** (disconnect = process death) and the auth holes (§2 — your progress isn't safe from others). The single biggest "content" win for *this* audience is **trust and persistence**: you can leave and come back and it's all still there and nobody messed with it. Fixing C1/C2 and the disconnect handling *is* a gameplay feature for this audience.

### 8.5 Future Vision (explicitly gated — your call, not recommended for MVP)
Listed for completeness, in priority order *if/when* the MVP exit gate (criterion 5: "want to log in again tomorrow") passes:
1. **Second star system** (v1.0) — the cheapest "more world" that doesn't touch the loop.
2. **Persistent NPC economy** (v1.0) — auto-mining/trade/guard fleets so the world feels alive when you're offline (aligns with "watch it grow").
3. **Faction reputation + compounding bonuses** (v1.0) — same-faction contribution pays compounding rep; cross-faction pays flat. The data model (`FactionStanding`) exists.
4. **Markets / dynamic pricing** (v1.0) — *after* the economy exploit is fixed and the buy/sell split is sound.
5. **Multiple ship hulls** (v1.0), **emotes** (#49), **research trees** (v1.0).
6. **Combat / weapons / shields** (v2.0) — the design doc shelves this deliberately; do not enable until the cozy loop is proven fun *and* a separate non-MVP "PvP-opt-in zone" design exists so the no-setback promise holds.

---

## 9. Prioritized Action Plan

Ordered by dependency: you cannot ship polish on top of security holes and a broken economy, and you cannot refactor safely without tests.

### Phase 0 — Stop the bleeding (security) — before any public test
- **C1** `try_server_only`: replace "no Player row = server" with an explicit identity allowlist (`config.server_identity` + `Identity::default()`). Add a test.
- **C2** `register_playername` / `create_player_controlled_ship`: use `ctx.sender()`, drop the `identity`/`username` params. Add a test.
- **C3** Add auth to `station_production_schedule_reducer`, `station_status_schedule_reducer`, `cleanup_visual_effect`.
- Commit the untracked `settings_window.rs` + `Makefile`; clean `main`'s dirty tree into a branch per project rule.

### Phase 1 — Correctness (economy + crashes)
- **E1–E4** Split buy/sell prices; enforce margins; fix overflow (`u64`/checked); restore station-credits; clamp price ≥ 1. Add property tests (buy-then-sell ≈ identity).
- **B1–B2** Cargo u16 overflow/underflow → `saturating_*` / widen types.
- **B3** Shield/energy equipment adds to flat regen, not replaces.
- **B4** Restore refinery rate-limiting; **B5** cap diplomacy standing gain; **B6** check `is_active` on jumpgates; **B7** clamp `rarity` to ≤100.
- **B8** Add range check to cargo pickup; **B9** self-target check (flag for combat enable).
- Client **6.5/6.6**: derive `mining_active` from timer row; placeholder textures + `.get()` lookups; shader fallback; fix the slice.

### Phase 2 — The safety net (tests + CI) — unblocks everything else
- Root `[workspace]` + `.github/workflows/ci.yml` (`fmt --check`, `clippy -D warnings`, `test`, `audit`).
- Remove the crate-level `#![allow(...)]` blanket (`server/src/lib.rs:5-24`); fix/justify each.
- Unit tests for `calculate_current_price`, cargo math, auth helpers, `pick_weighted_ore`/`roll_ore_item` (incl. `rarity > 100`), `compute_construction_progress` (extend existing).
- Integration tests for the auth + economy reducers (SpacetimeDB in-process).

### Phase 3 — Performance
- **6.2** Switch ship lookups to indexed accessors (`db.ship().sobj_id().find(...)` / `.player_id().find(...)`). Mechanical, huge ROI.
- **6.3** Cache GUI derived collections on dirty flag; trim chat history; reuse frame Vecs.
- **6.4** Time-based cross-sector grace window.
- **5.4** Centralize magic numbers into `GlobalConfig` (also makes balance testable).

### Phase 4 — Architecture
- **5.1** Move all query/creation/balance helpers out of `tables/` into `logic/` (or `queries/`), restoring the three-layer invariant and breaking the circular imports. Do this *after* Phase 2 so the suite guards the refactor.
- Delete empty files, dead code, stale branches; remove unused shader uniforms.

### Phase 5 — Feel, polish, content depth (the "AAA" pass, in-scope)
- **8.2** Completion moment + contribution attribution (the retention hook).
- **8.1** Distinct per-station demand profiles + map/welcome-back "where you're needed" surfacing.
- **8.3** Mining audio + depletion visuals + ore-choice legibility.
- **6.1** Reconnect/“connection lost” screen (core to the "progress waits for you" promise).
- **6.8** Wire the advertised [C]/[Space] keyboard shortcuts; Settings shortcut; colorblind-safe faction marks.
- M5 exit gate: a 20-minute session that "felt good" — then M6 launch.

### Phase 6 — Future Vision (gated on MVP criterion 5)
- Per §8.5, only after "want to log in again tomorrow" is honestly answered yes.

---

## 10. Open Questions for You

1. **Scope confirmation**: You asked for "AAA with as much content, gameplay, fun, challenge." The MVP design doc explicitly forbids combat/NPCs/markets as Future Vision and the project's stated superpower is cozy no-setback scope discipline. I've interpreted "AAA" as **depth + feel + foundation within the loop** and listed breadth as gated Future Vision. Do you want me to (a) hold that line, or (b) scope-creep into a breadth item (and if so, which)?
2. **Ship readiness**: The security holes (C1/C2) mean the server is not safe in front of any untrusted client. Is there an upcoming public test (M6 smoke test) that sets the deadline for Phase 0–1?
3. **Workspace/CI**: OK to add a root `Cargo.toml` workspace + GitHub Actions CI (fmt/clippy/test/audit)? This is the highest-leverage change and touches the repo root.
4. **Uncommitted `main`**: the 21-file M5-polish diff is sitting uncommitted on `main`. Want me to move it to a `feat/m5-polish` branch and commit it atomically per the project's branch rule, or were you mid-edit?

*Generated by opencode. Evidence: live builds, two module-level code reviews, direct verification of the security and economy findings, test run, git state. Every finding cites `file:line`.*
