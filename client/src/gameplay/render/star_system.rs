use std::f32::consts::PI;

use glam::Vec2;
use macroquad::{miniquad::date::now, prelude::collections::storage};

use super::*;

pub fn render_star_system(game_state: &mut GameState) {
    let resources = storage::get::<Resources>();
    let multiplier = 100.0;

    let stage_one_distance_squared = screen_height() * screen_height();
    let stage_two_distance_squared = stage_one_distance_squared * 1000.0;
    let stage_three_distance_squared = stage_one_distance_squared * 10000.0;

    let camera = game_state.bg_camera.target;

    for sso in game_state.ctx.db().star_system_object().iter() {
        match sso.kind {
            StarSystemObjectKind::NebulaBelt => {
                let key = match sso.id % 8 {
                    0 => "nebula.1",
                    1 => "nebula.2",
                    2 => "nebula.3",
                    3 => "nebula.5",
                    4 => "nebula.6",
                    5 => "nebula.7",
                    6 => "nebula.9",
                    _ => "nebula.10",
                };
                let image = &resources.nebula_textures[key];
                draw_nebula_belt(
                    multiplier,
                    stage_one_distance_squared,
                    stage_two_distance_squared,
                    stage_three_distance_squared,
                    camera,
                    sso,
                    image,
                );
            }
            StarSystemObjectKind::AsteroidBelt => {
                continue;
            }
            _ => {
                let (image, secondary) = match sso.kind {
                    StarSystemObjectKind::Star => (&resources.sun_textures["star.1"], None),
                    StarSystemObjectKind::Planet => (
                        &resources.planet_textures[sso
                            .gfx_key
                            .clone()
                            .unwrap_or("planet.1".to_string())
                            .as_str()],
                        Some(&resources.planet_textures["planet.shadow.1"]),
                    ),
                    StarSystemObjectKind::Moon => (
                        &resources.planet_textures["moon.1"],
                        Some(&resources.planet_textures["planet.shadow.1"]),
                    ),
                    _ => continue,
                };

                draw_star_system_object(
                    multiplier,
                    stage_one_distance_squared,
                    stage_two_distance_squared,
                    stage_three_distance_squared,
                    camera,
                    sso,
                    image,
                    secondary,
                );
            }
        }
    }
}

fn draw_star_system_object(
    multiplier: f32,
    stage_one_distance_squared: f32,
    stage_two_distance_squared: f32,
    stage_three_distance_squared: f32,
    camera: Vec2,
    sso: StarSystemObject,
    image: &Texture2D,
    secondary: Option<&Texture2D>,
) {
    let stage_one_radius = stage_one_distance_squared.sqrt();
    let stage_two_radius = stage_two_distance_squared.sqrt();
    let stage_three_radius = stage_three_distance_squared.sqrt();

    let mut vec = Vec2::from_angle(sso.rotation_or_width_km) * sso.orbit_au * multiplier;
    let dist = vec.distance_squared(camera);
    let dist_lin = dist.sqrt();
    let mut scale = 1.0;
    let mut alpha: u8 = 255;

    if dist < stage_one_distance_squared {
        // Stage 1: Normal — render at full size and opacity
    } else if dist < stage_two_distance_squared {
        // Stage 2: Clamp to edge of stage 1, reduce scale
        let angle = (vec - camera).to_angle();

        vec = camera + Vec2::from_angle(angle) * stage_one_radius;
        scale = (stage_one_radius / dist_lin) * 0.75 + 0.25;
    } else if dist < stage_three_distance_squared {
        // Stage 3: Slow slide off the screen with fade-out
        let angle = (vec - camera).to_angle();
        let t = ((dist_lin - stage_two_radius) / (stage_three_radius - stage_two_radius)).min(1.0);

        let slide_dist = stage_one_radius + t * (stage_one_radius * 1.5);
        vec = camera + Vec2::from_angle(angle) * slide_dist;

        scale = (stage_one_radius / dist_lin) * 0.25;

        if t > 0.4 {
            let fade_t = ((t - 0.4) / 0.6).min(1.0);
            alpha = (255.0 * (1.0 - fade_t)) as u8;
        }
    }

    let color = Color::from_rgba(255, 255, 255, alpha);

    let mut params = DrawTextureParams::default();
    params.rotation = (((now() * 0.01f64) as f32) % 2.0) * PI;
    params.dest_size = Some(Vec2::new(image.width() * scale, image.height() * scale));

    draw_texture_ex(
        &image,
        image.width() * -0.5 * scale + vec.x,
        image.height() * -0.5 * scale + vec.y,
        color,
        params,
    );

    if let Some(shadow) = secondary {
        let sun_angle = (vec - camera).to_angle();
        let scale_adjust = if sso.kind == StarSystemObjectKind::Planet {
            0.85
        } else {
            0.21
        };

        let params = DrawTextureParams {
            rotation: sun_angle - PI / 4.0,
            dest_size: Some(Vec2::new(
                shadow.width() * scale * scale_adjust,
                shadow.height() * scale * scale_adjust,
            )),
            ..Default::default()
        };

        draw_texture_ex(
            shadow,
            shadow.width() * -0.5 * scale * scale_adjust + vec.x,
            shadow.height() * -0.5 * scale * scale_adjust + vec.y,
            color,
            params,
        );
    }
}

fn draw_nebula_belt(
    multiplier: f32,
    stage_one_distance_squared: f32,
    stage_two_distance_squared: f32,
    stage_three_distance_squared: f32,
    camera: Vec2,
    sso: StarSystemObject,
    image: &Texture2D,
) {
    let stage_one_radius = stage_one_distance_squared.sqrt();
    let stage_two_radius = stage_two_distance_squared.sqrt();
    let stage_three_radius = stage_three_distance_squared.sqrt();

    let mut vec = Vec2::from_angle(sso.rotation_or_width_km) * sso.orbit_au * multiplier;
    let dist = vec.distance_squared(camera);
    let dist_lin = dist.sqrt();
    let mut scale = 5.0;
    let mut alpha: u8 = 80;

    if dist < stage_one_distance_squared {
        // Stage 1: Full size and opacity
    } else if dist < stage_two_distance_squared {
        let angle = (vec - camera).to_angle();
        vec = camera + Vec2::from_angle(angle) * stage_one_radius;
        scale = (stage_one_radius / dist_lin) * 3.75 + 1.25;
    } else if dist < stage_three_distance_squared {
        let angle = (vec - camera).to_angle();
        let t = ((dist_lin - stage_two_radius) / (stage_three_radius - stage_two_radius)).min(1.0);
        let slide_dist = stage_one_radius + t * (stage_one_radius * 1.5);
        vec = camera + Vec2::from_angle(angle) * slide_dist;
        scale = (stage_one_radius / dist_lin) * 1.25;
        if t > 0.4 {
            let fade_t = ((t - 0.4) / 0.6).min(1.0);
            alpha = (80.0 * (1.0 - fade_t)) as u8;
        }
    }

    let color = Color::from_rgba(255, 255, 255, alpha);

    let mut params = DrawTextureParams::default();
    params.dest_size = Some(Vec2::new(image.width() * scale, image.height() * scale));

    draw_texture_ex(
        image,
        image.width() * -0.5 * scale + vec.x,
        image.height() * -0.5 * scale + vec.y,
        color,
        params,
    );
}
