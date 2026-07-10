use glam::Vec2;
use macroquad::prelude::*;
use macroquad::prelude::collections::storage;

use crate::{gameplay::render::star_system::render_star_system, server::bindings::*};
use spacetimedb_sdk::{DbContext, Table};

use crate::stdb::utils::*;

use super::{resources::Resources, state::GameState};

pub mod in_sector;
pub mod star_system;

use in_sector::*;

pub fn sector(game_state: &mut GameState) {
    let mut player_pose: Option<RenderPose> = None;
    let mut player_ship_type = None;
    let player_ship = get_player_ship(game_state.ctx);

    let mut local_targets: Vec<(u64, glam::Vec2, StellarObjectKinds)> = Vec::new();

    set_camera(&game_state.bg_camera);

    render_star_system(game_state);

    set_camera(&game_state.camera);

    let db = &game_state.ctx.db;
    let now_micros = now_unix_micros();

    // (#89) The player's current sector is wherever their in-sector ship is.
    // Used below to drop any stellar object whose `sector_id` doesn't match —
    // see the wrong-sector filter in the first pass. `None` while docked /
    // out-of-play, in which case we don't filter (nothing to anchor to).
    //
    // Sector-transition smoothing: when the ship's `sector_id` changes (jumpgate
    // transit), the subscription may deliver the Ship update before the new
    // sector's StellarObject rows arrive. We keep the previous sector's objects
    // visible for a grace window so the view doesn't flash-empty.
    let player_sector = player_ship.as_ref().map(|s| s.sector_id);
    if player_sector.is_some() && player_sector != game_state.last_player_sector {
        game_state.sector_transition_from = game_state.last_player_sector;
        game_state.sector_transition_grace = 3;
    }
    game_state.last_player_sector = player_sector;
    let transition_sector = if game_state.sector_transition_grace > 0 {
        game_state.sector_transition_grace -= 1;
        let ts = game_state.sector_transition_from;
        if game_state.sector_transition_grace == 0 {
            game_state.sector_transition_from = None;
        }
        ts
    } else {
        None
    };

    // Draw sector-level nebula fog overlay — semi-transparent texture centered
    // on the camera's target, large enough to cover the full visible area.
    // Alpha is clamped to [0, 160] so even sectors with nebula=1.0 remain
    // semi-transparent. Drawn after the star system backdrop but before any
    // in-sector objects so the fog sits behind ships/stations.
    if let Some(sector_id) = player_sector {
        if let Some(sector) = db.sector().id().find(&sector_id) {
            if sector.nebula > 0.0 {
                let resources = storage::get::<Resources>();
                let key = sector
                    .background_gfx_key
                    .as_deref()
                    .filter(|k| resources.nebula_textures.contains_key(*k))
                    .unwrap_or(match sector_id % 8 {
                        0 => "nebula.1",
                        1 => "nebula.2",
                        2 => "nebula.3",
                        3 => "nebula.5",
                        4 => "nebula.6",
                        5 => "nebula.7",
                        6 => "nebula.9",
                        _ => "nebula.10",
                    });
                if let Some(texture) = resources.nebula_textures.get(key) {
                    let alpha = (160.0 * sector.nebula.min(1.0)) as u8;
                    let color = Color::from_rgba(200, 160, 255, alpha);
                    let visible_width = screen_width() / game_state.camera.zoom.x;
                    let visible_height = screen_height() / game_state.camera.zoom.y;
                    let cover = (visible_width.max(visible_height) * 3.0) / texture.width();
                    let cx = game_state.camera.target.x;
                    let cy = game_state.camera.target.y;
                    draw_texture_ex(
                        texture,
                        cx - texture.width() * cover * 0.5,
                        cy - texture.height() * cover * 0.5,
                        color,
                        DrawTextureParams {
                            dest_size: Some(Vec2::new(
                                texture.width() * cover,
                                texture.height() * cover,
                            )),
                            ..Default::default()
                        },
                    );
                }
            }
        }
    }

    // Collect ships to draw after stations
    let mut ships_to_draw: Vec<(Ship, RenderPose, ShipTypeDefinition)> = Vec::new();

    // First pass: Draw everything except ships
    for object in db.stellar_object().iter() {
        // (#89) Wrong-sector render filter. During normal flight, only objects
        // in the player's current sector are drawn. On jumpgate transit the
        // Ship.sector_id update may arrive before the new sector's StellarObject
        // rows; the `transition_sector` grace window keeps the previous sector's
        // objects visible for ~3 frames to prevent a flash-empty view.
        if let Some(ps) = player_sector {
            if object.sector_id != ps {
                let in_transition = transition_sector
                    .map_or(false, |ts| object.sector_id == ts);
                if !in_transition {
                    continue;
                }
            }
        }
        // Build a render pose for this object — predicts forward for ships
        // and cargo crates, reads the static position column for asteroids /
        // stations / jumpgates.
        let pose = match pose_for_object(game_state.ctx, &object, now_micros) {
            Some(p) => p,
            None => continue,
        };

        match object.kind {
            StellarObjectKinds::Ship => {
                if let Some((ship_object, ship_type)) =
                    get_ship_with_type(game_state.ctx, pose.sobj_id)
                {
                    if player_ship
                        .as_ref()
                        .is_some_and(|ship_obj| ship_obj.sobj_id == object.id)
                    {
                        player_pose = Some(pose);
                        player_ship_type = Some(ship_type);
                    } else {
                        ships_to_draw.push((ship_object, pose, ship_type));
                    }
                }
            }
            StellarObjectKinds::Station => {
                if let Some(station) = db.station().sobj_id().find(&object.id) {
                    draw_station(&pose, station, game_state);
                }
            }
            StellarObjectKinds::JumpGate => {
                if let Some(jumpgate) = db.jump_gate().id().find(&object.id) {
                    draw_jumpgate(&pose, jumpgate, game_state);
                }
            }
            StellarObjectKinds::Asteroid => {
                if let Some(asteroid) = db.asteroid().id().find(&object.id) {
                    draw_asteroid(&pose, asteroid, game_state);
                }
            }
            StellarObjectKinds::CargoCrate => {
                if let Some(cargo_crate) = db.cargo_crate().sobj_id().find(&object.id) {
                    draw_crate(&pose, cargo_crate, game_state);
                }
            }
        }
        local_targets.push((object.id, pose.pos, object.kind));
    }

    // Second pass: Draw all non-player ships AFTER stations
    for (ship_object, pose, ship_type) in ships_to_draw {
        draw_ship(&ship_object, &pose, &ship_type, game_state);
    }

    if let (Some(actual_player_pose), Some(player_ship_type), Some(player_ship)) =
        (player_pose, player_ship_type, player_ship)
    {
        draw_mining_laser(game_state, &actual_player_pose);

        // Draw the controlled ship so its always on top.
        draw_ship(&player_ship, &actual_player_pose, &player_ship_type, game_state);

        // Draw 'radar'
        draw_radar(
            game_state,
            local_targets,
            &actual_player_pose,
            actual_player_pose.pos,
        );
    }
}
