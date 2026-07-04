use macroquad::{
    miniquad::date::now,
    prelude::{collections::storage, *},
};

use crate::server::bindings::*;
use crate::stdb::utils::*;

use crate::gameplay::{resources::Resources, state::GameState};

pub fn draw_mining_laser(game_state: &mut GameState<'_>, player_pose: &RenderPose) {
    if !game_state.mining_active {
        return;
    }
    let Some(target) = get_current_target(game_state.ctx, &mut game_state.current_target_sobj_id)
    else {
        return;
    };
    if target.kind != StellarObjectKinds::Asteroid {
        return;
    }
    let now_micros = now_unix_micros();
    let Some(target_pose) = pose_for_object(game_state.ctx, &target, now_micros) else {
        return;
    };

    let t = now() as f32;
    let (sx, sy) = (player_pose.pos.x, player_pose.pos.y);
    let (ex, ey) = (target_pose.pos.x, target_pose.pos.y);

    // Outer glow — wide, faint, warm orange
    draw_line(ex, ey, sx, sy, 14.0, Color::from_rgba(200, 60, 20, 30));

    // Mid layer — pulsing hot orange
    let pulse = (t * 6.0).sin() * 0.3 + 0.7;
    draw_line(
        ex, ey, sx, sy, 8.0,
        Color::from_rgba(240, 100, 30, (80.0 * pulse) as u8),
    );

    // Core — bright yellow-white, steady
    draw_line(ex, ey, sx, sy, 2.0, Color::from_rgba(255, 240, 200, 220));

    // Energy segments flowing from player toward asteroid
    let dist = glam::Vec2::new(ex - sx, ey - sy);
    let len = dist.length();
    if len > 0.0 {
        let dir = dist / len;
        let segment_spacing = 30.0;
        let speed = 120.0;
        let offset = (t * speed) % segment_spacing;
        let mut d = offset;
        while d < len {
            let px = sx + dir.x * d;
            let py = sy + dir.y * d;
            let brightness = ((d / len) * 0.5 + 0.5) * pulse;
            draw_circle(px, py, 2.5, Color::from_rgba(255, 200, 100, (180.0 * brightness) as u8));
            d += segment_spacing;
        }
    }

    // Hit flash on the asteroid
    let hit_radius = 6.0 + (t * 8.0).sin() * 2.0;
    draw_circle(ex, ey, hit_radius, Color::from_rgba(255, 120, 30, (60.0 * pulse) as u8));
    draw_circle_lines(ex, ey, hit_radius + 4.0, 1.5, Color::from_rgba(255, 180, 80, (40.0 * pulse) as u8));
}

pub fn draw_radar(
    game_state: &mut GameState<'_>,
    local_targets: Vec<(u64, glam::Vec2, StellarObjectKinds)>,
    player_pose: &RenderPose,
    player_vec: glam::Vec2,
) {
    let radar_radius = screen_height() / 2.0 - 100.0;
    let radar_icon_size = 12.0;
    let ring_radius = radar_radius - radar_icon_size;
    draw_circle_lines(
        player_pose.pos.x,
        player_pose.pos.y,
        ring_radius,
        radar_icon_size * 2.0,
        Color::from_rgba(255, 255, 255, 32),
    );

    // Inner range rings at ¼, ½, and ¾ of the radar radius so the player
    // can estimate distance at a glance without reading labels.
    for frac in [0.25, 0.5, 0.75] {
        draw_circle_lines(
            player_pose.pos.x,
            player_pose.pos.y,
            ring_radius * frac,
            1.0,
            Color::from_rgba(255, 255, 255, 12),
        );
    }

    // Predicted-forward velocity for the player ship — the HUD reads
    // `velocity` (scalar speed) and `rotation` from the same snapshot used
    // to draw the ship so the indicators don't lag the sprite.
    if let Some((_, snapshot)) = predicted_player_snapshot(game_state.ctx) {
        let _ = draw_hud(game_state, radar_radius, &snapshot, player_pose, &player_vec);
    }

    for (sobj_id, position, kind) in local_targets {
        let angle = (position - player_vec).to_angle();
        let dist = player_vec.distance(position);

        // Map distance to radial position: objects beyond ring_radius hug the
        // ring; closer objects appear at a proportional fraction so the player
        // can judge range at a glance.
        let radial_fraction = if dist < ring_radius {
            dist / ring_radius
        } else {
            1.0
        };
        let icon_dist = ring_radius * radial_fraction;
        let from =
            player_vec + (glam::Vec2::from_angle(angle) * icon_dist + radar_icon_size / 2.0);

        let is_targetted = game_state.current_target_sobj_id == Some(sobj_id);
        let thickness = if is_targetted { 2.0 } else { 1.0 };

        let distance_fade = if dist < 1000.0 {
            1.0
        } else {
            if dist < 5000.0 {
                ((6000.0 - dist) / 5000.0) * 0.75 + 0.25
            } else {
                0.25
            }
        };

        let actual_fade = if is_targetted {
            (255.0 * distance_fade) as u8
        } else {
            (192.0 * distance_fade) as u8
        };

        let radius = radar_icon_size * distance_fade;
        if is_targetted {
            draw_poly(
                from.x,
                from.y,
                polygon_points_per_kind(kind),
                radius * 2.0,
                0.0,
                Color::from_rgba(255, 255, 255, 96),
            );
        }

        draw_poly_lines(
            from.x,
            from.y,
            polygon_points_per_kind(kind),
            radius + 1.0,
            1.0,
            thickness,
            Color::from_rgba(0, 0, 0, actual_fade),
        );
        draw_poly_lines(
            from.x,
            from.y,
            polygon_points_per_kind(kind),
            radius,
            1.0,
            thickness,
            Color::from_rgba(255, 255, 255, actual_fade),
        );

        // Distance label next to the icon (clamped to ring edge for objects
        // beyond the visible radius so the label stays near the icon).
        let label_pos = if radial_fraction < 1.0 {
            from
        } else {
            player_vec + (glam::Vec2::from_angle(angle) * ring_radius + radar_icon_size / 2.0)
        };
        let label = format!("{:.0}", dist);
        draw_text_ex(
            &label,
            label_pos.x + radar_icon_size + 4.0,
            label_pos.y - 4.0,
            TextParams {
                font_size: 12,
                color: Color::from_rgba(255, 255, 255, (actual_fade as f32 * 0.75) as u8),
                ..Default::default()
            },
        );
    }
}

/// Draws the velocity / heading needle around the player's ship. The
/// snapshot's `rotation` and `velocity` (scalar speed) replace the legacy
/// `StellarObjectVelocity`'s (x,y) vector — multiply speed by the heading
/// unit vector to get the velocity arrow.
pub fn draw_hud(
    _game_state: &mut GameState<'_>,
    radar_radius: f32,
    snapshot: &solarance_shared::MovementState,
    pose: &RenderPose,
    _player_vec: &glam::Vec2,
) -> Result<(), String> {
    let color = Color::from_rgba(255, 255, 255, 128);
    let position = pose.pos;

    let heading = glam::Vec2::from_angle(pose.rotation_radians);
    let point_forward = position + heading * radar_radius;
    let point_forward_mid = position + heading * (radar_radius - 32.0);

    draw_line(
        point_forward_mid.x,
        point_forward_mid.y,
        point_forward.x,
        point_forward.y,
        3.0,
        color,
    );

    let velocity_speed = snapshot.velocity.max(0.0);
    let velocity_dir = heading; // ships only move forward in this model
    let point_velocity_mid = position + velocity_dir * (radar_radius - 32.0);
    let point_velocity_low =
        position + velocity_dir * (radar_radius - 32.0 - velocity_speed);

    draw_line(
        point_velocity_mid.x,
        point_velocity_mid.y,
        point_velocity_low.x,
        point_velocity_low.y,
        3.0,
        color,
    );

    Ok(())
}

pub fn draw_ship(
    ship: &Ship,
    pose: &RenderPose,
    ship_type: &ShipTypeDefinition,
    game_state: &mut GameState,
) {
    let resources = storage::get::<Resources>();
    let position = pose.pos;

    if let Some(player) = game_state.ctx.db.player().id().find(&ship.player_id) {
        let string = format!(
            "[{}] {}",
            get_faction_shortname(game_state.ctx, &player.faction_id.value),
            player.username
        );
        let dimension = measure_text(&string, None, 16, 1.0);
        draw_text_ex(
            &string,
            position.x - dimension.width / 2.0,
            position.y - 32.0,
            TextParams {
                font_size: 16,
                color: WHITE,
                ..TextParams::default()
            },
        );
    }

    let tex = &resources.ship_textures[ship_type.gfx_key.clone().unwrap().as_str()];
    draw_texture_ex(
        tex,
        position.x - tex.width() * 0.5,
        position.y - tex.height() * 0.5,
        WHITE,
        DrawTextureParams {
            rotation: pose.rotation_radians,
            ..DrawTextureParams::default()
        },
    );

    if game_state.current_target_sobj_id == Some(pose.sobj_id) {
        let size = (tex.width() + tex.height()) * 0.5;
        draw_targeting_bracket(
            position,
            size,
            StellarObjectKinds::Ship,
            Color::from_rgba(255, 255, 255, 200),
        );
    }
}

pub fn draw_asteroid(pose: &RenderPose, asteroid: Asteroid, game_state: &mut GameState) {
    let resources = storage::get::<Resources>();
    let position = pose.pos;
    let angle = pose.rotation_radians;

    let tex = &resources.asteroid_textures[asteroid
        .gfx_key
        .unwrap_or("asteroid.1".to_string())
        .as_str()];
    draw_texture_ex(
        tex,
        position.x - tex.width() * 0.5,
        position.y - tex.height() * 0.5,
        WHITE,
        DrawTextureParams {
            rotation: angle,
            ..DrawTextureParams::default()
        },
    );

    if game_state.current_target_sobj_id == Some(asteroid.id) {
        let size = (tex.width() + tex.height()) * 0.5;
        draw_targeting_bracket(
            position,
            size,
            StellarObjectKinds::Asteroid,
            Color::from_rgba(255, 255, 255, 200),
        );
    }
}

pub fn draw_crate(pose: &RenderPose, cargo_crate: CargoCrate, game_state: &mut GameState) {
    let resources = storage::get::<Resources>();
    let position = pose.pos;
    let angle = pose.rotation_radians;

    let tex = &resources.asteroid_textures[cargo_crate
        .gfx_key
        .unwrap_or("crate.0".to_string())
        .as_str()];

    // Soft glow underneath the crate so it's easier to spot at a distance
    let t = now() as f32;
    let glow_radius = (tex.width().max(tex.height()) * 0.6) + (t * 2.0).sin() * 4.0;
    draw_circle(
        position.x,
        position.y,
        glow_radius,
        Color::from_rgba(120, 200, 255, 24),
    );

    draw_texture_ex(
        tex,
        position.x - tex.width() * 0.5,
        position.y - tex.height() * 0.5,
        WHITE,
        DrawTextureParams {
            rotation: angle,
            ..DrawTextureParams::default()
        },
    );

    // Item name label floating above the crate
    if let Some(def) = game_state.ctx.db.item_definition().id().find(&cargo_crate.item_id) {
        draw_text_ex(
            &format!("{} x{}", def.name, cargo_crate.quantity),
            position.x,
            position.y - tex.height() * 0.5 - 14.0,
            TextParams {
                font_size: 11,
                color: Color::from_rgba(200, 230, 255, 200),
                ..Default::default()
            },
        );
    }

    if game_state.current_target_sobj_id == Some(cargo_crate.sobj_id) {
        let size = (tex.width() + tex.height()) * 0.5;
        draw_targeting_bracket(
            position,
            size,
            StellarObjectKinds::CargoCrate,
            Color::from_rgba(255, 255, 255, 200),
        );
    }
}

pub fn draw_jumpgate(pose: &RenderPose, jumpgate: JumpGate, game_state: &mut GameState) {
    let resources = storage::get::<Resources>();
    let position = pose.pos;

    let tex = &resources.jumpgate_textures[jumpgate
        .gfx_key
        .unwrap_or("jumpgate_north".to_string())
        .as_str()];

    // Subtle energy pulse around the gate
    let t = now() as f32;
    let pulse = (t * 2.5).sin() * 0.3 + 0.7;
    let gate_radius = (tex.width().max(tex.height()) * 0.4) + (t * 1.5).sin() * 6.0;
    draw_circle_lines(
        position.x,
        position.y,
        gate_radius,
        1.5,
        Color::from_rgba(80, 200, 255, (60.0 * pulse) as u8),
    );

    draw_texture(
        tex,
        position.x - tex.width() * 0.5,
        position.y - tex.height() * 0.5,
        WHITE,
    );

    if game_state.current_target_sobj_id == Some(jumpgate.id) {
        let size = (tex.width() + tex.height()) * 0.33;
        draw_targeting_bracket(
            position,
            size,
            StellarObjectKinds::JumpGate,
            Color::from_rgba(255, 255, 255, 200),
        );
    }
}

pub fn draw_station(pose: &RenderPose, station: Station, game_state: &mut GameState) {
    let resources = storage::get::<Resources>();
    let position = pose.pos;

    let under_construction = game_state
        .ctx
        .db
        .station_under_construction()
        .id()
        .find(&station.id)
        .is_some();
    let gfx_key = match (station.size, under_construction) {
        (StationSize::Capital, false) => "station.capital",
        (StationSize::Capital, true) => "station.capital.uc",
        (StationSize::Large, false) => "station.large",
        (StationSize::Large, true) => "station.large.uc",
        (StationSize::Medium, false) => "station.medium",
        (StationSize::Medium, true) => "station.medium.uc",
        (StationSize::Small, false) => "station.small",
        (StationSize::Small, true) => "station.small.uc",
        (StationSize::Outpost, false) => "station.outpost",
        (StationSize::Outpost, true) => "station.outpost.uc",
        (StationSize::Satellite, false) => "station.satellite",
        (StationSize::Satellite, true) => "station.satellite.uc",
    };
    let tex = &resources.station_textures[gfx_key];
    draw_texture(
        tex,
        position.x - tex.width() * 0.5,
        position.y - tex.height() * 0.5,
        WHITE,
    );

    if game_state.current_target_sobj_id == Some(station.sobj_id) {
        let size = (tex.width() + tex.height()) * 0.33;
        draw_targeting_bracket(
            position,
            size * 1.1,
            StellarObjectKinds::Station,
            Color::from_rgba(255, 255, 255, 200),
        );
    }
}

pub fn draw_targeting_bracket(pos: glam::Vec2, size: f32, kind: StellarObjectKinds, color: Color) {
    draw_poly_lines(
        pos.x,
        pos.y,
        polygon_points_per_kind(kind),
        size,
        1.0,
        if size < 512.0 { 1.0 } else { size / 512.0 },
        color,
    );
}

pub fn polygon_points_per_kind(kind: StellarObjectKinds) -> u8 {
    match kind {
        StellarObjectKinds::Ship => 3,
        StellarObjectKinds::Asteroid => 7,
        StellarObjectKinds::CargoCrate => 4,
        StellarObjectKinds::Station => 6,
        StellarObjectKinds::JumpGate => 5,
    }
}
