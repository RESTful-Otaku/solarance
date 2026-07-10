use egui::{Align2, Color32, Context, RichText, ScrollArea};
use macroquad::miniquad::date::now;
use macroquad::prelude::*;

use crate::server::bindings::*;
use spacetimedb_sdk::{DbContext, Table};

use crate::gameplay::state::GameState;
use crate::stdb::utils::*;

#[derive(PartialEq)]
pub enum HelpTab {
    Controls,
    GameLoop,
    Debug,
}

pub fn draw(
    egui_ctx: &Context,
    game_state: &mut GameState,
) -> Option<egui::InnerResponse<Option<()>>> {
    let ctx = game_state.ctx;

    egui::Window::new("Help")
        .title_bar(true)
        .resizable(false)
        .collapsible(true)
        .movable(false)
        .anchor(Align2::LEFT_BOTTOM, egui::Vec2::new(-5.0, 5.0))
        .default_size(egui::Vec2::new(420.0, 360.0))
        .show(egui_ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut game_state.help_tab, HelpTab::Controls, "Controls");
                ui.selectable_value(&mut game_state.help_tab, HelpTab::GameLoop, "Game Loop");
                ui.selectable_value(&mut game_state.help_tab, HelpTab::Debug, "Debug");
            });
            ui.separator();

            match game_state.help_tab {
                HelpTab::Controls => draw_controls_tab(ui),
                HelpTab::GameLoop => draw_gameloop_tab(ui),
                HelpTab::Debug => draw_debug_tab(ui, ctx, game_state),
            }

            ui.add_space(8.0);
            ui.horizontal(|ui| {
                if ui.button("  Quit  ").clicked() {
                    game_state.done = true;
                }
                if ui.button("Ship Details").clicked() {
                    game_state.windows.details = !game_state.windows.details;
                }
            });
        })
}

fn draw_controls_tab(ui: &mut egui::Ui) {
    ScrollArea::vertical().auto_shrink([false, true]).show(ui, |ui| {
        ui.label(RichText::new("Ship Controls").strong().size(16.0));
        ui.separator();
        controls_row(ui, "W / Arrow Up", "Thrust forward");
        controls_row(ui, "S / Arrow Down", "Thrust backward");
        controls_row(ui, "A / Arrow Left", "Rotate left");
        controls_row(ui, "D / Arrow Right", "Rotate right");

        ui.add_space(8.0);
        ui.label(RichText::new("Actions").strong().size(16.0));
        ui.separator();
        controls_row(ui, "[E]", "Target nearest object");
        controls_row(ui, "[Q]", "Toggle Combat / Utility mode");
        controls_row(ui, "[X]", "Toggle mining beam (on/off)");
        controls_row(ui, "[C]", "Dock / Jump / Undock");
        controls_row(ui, "[Space]", "Fire weapons (combat mode only)");

        ui.add_space(8.0);
        ui.label(RichText::new("Windows").strong().size(16.0));
        ui.separator();
        controls_row(ui, "[R]", "Ship details");
        controls_row(ui, "[F]", "Faction overview");
        controls_row(ui, "[T]", "Assets / cargo");
        controls_row(ui, "[M]", "Map");
        controls_row(ui, "[B]", "Build / construction");
    });
}

fn controls_row(ui: &mut egui::Ui, key: &str, action: &str) {
    ui.horizontal(|ui| {
        ui.monospace(format!(" {:<20}", key));
        ui.label(action);
    });
}

fn draw_gameloop_tab(ui: &mut egui::Ui) {
    ScrollArea::vertical().auto_shrink([false, true]).show(ui, |ui| {
        ui.label(RichText::new("How to Play").strong().size(16.0));
        ui.separator();
        ui.add_space(4.0);

        step(ui, "1", "Find", "Fly to an asteroid field sector. Asteroids appear as targets in your radar.");
        step(ui, "2", "Target", "Press [E] to target the nearest asteroid. Its ore type and resources show in the status bar.");
        step(ui, "3", "Mine", "Press [X] to fire your mining beam. Ore is collected directly into your cargo hold.");
        step(ui, "4", "Haul", "Fly to a construction site (marked on the map). Station progress bars show how close they are to completion.");
        step(ui, "5", "Contribute", "Open the Build window [B], select the site, and deposit your ore. Watch the bar move!");
        step(ui, "6", "Grow", "When a station reaches 100%, modules are unlocked. Completed stations become trading hubs.");

        ui.add_space(12.0);
        ui.separator();
        ui.add_space(4.0);
        ui.label(RichText::new("Tips").color(Color32::GOLD));
        ui.label("• Use the radar (bottom-right) to see nearby objects and their distances.");
        ui.label("• Your energy recharges over time — keep an eye on it while mining.");
        ui.label("• Construction sites show which resources they need in the Build window.");
        ui.label("• Other players' contributions are visible in real time — you're building together!");
    });
}

fn step(ui: &mut egui::Ui, number: &str, title: &str, desc: &str) {
    ui.horizontal(|ui| {
        ui.label(RichText::new(format!("[{}] ", number)).strong().color(Color32::LIGHT_BLUE));
        ui.vertical(|ui| {
            ui.label(RichText::new(title).strong().size(14.0));
            ui.label(RichText::new(desc).size(13.0).color(Color32::LIGHT_GRAY));
        });
    });
    ui.add_space(6.0);
}

fn draw_debug_tab(ui: &mut egui::Ui, ctx: &DbConnection, game_state: &mut GameState) {
    match ctx.db().player().id().find(&ctx.identity()) {
        Some(player) => {
            ui.heading(format!("Player: {}", player.username));
            if let Some(controlled) = player.get_controlled_stellar_object_id(&ctx) {
                match get_transform(&ctx, controlled) {
                    Ok(transform) => {
                        ui.label(format!(
                            "SObj: {}, {}",
                            transform.pos.x, transform.pos.y
                        ));
                    }
                    _ => {
                        ui.label("SObj: unknown");
                    }
                }
            } else {
                ui.label("WARNING - The player doesn't have a SObj!");
            }
        }
        None => {
            ui.heading("Player: MISSING");
            ui.label(format!("ID: {}", ctx.identity()));

            ui.horizontal(|ui| {
                ui.label("Username: ");
                ui.text_edit_singleline(&mut game_state.chat_window.text);
            });
        }
    }

    ui.collapsing("Stellar Objects", |ui| {
        ScrollArea::vertical()
            .auto_shrink([false, true])
            .stick_to_bottom(true)
            .max_height(screen_height() / 4.0)
            .show(ui, |ui| {
                let player_transform = get_player_transform_vec2(ctx, glam::Vec2::ZERO);
                for object in ctx.db().stellar_object().iter() {
                    let obj_type = format!("{:?}", object.kind);

                    ui.horizontal(|ui| {
                        ui.label(format!("{} #{}", obj_type, object.id));

                        match get_transform(&ctx, object.id) {
                            Ok(transform) => {
                                let string = format!(
                                    "Position: {}, {} Distance: {}",
                                    transform.pos.x,
                                    transform.pos.y,
                                    player_transform.distance(transform.to_vec2())
                                );
                                ui.label(string);
                            }
                            _ => {
                                ui.label("Position: n/a");
                            }
                        }
                        ui.label(format!("- {}", get_sector_name(ctx, &object.sector_id)));
                    });
                }
            });
    });

    ui.collapsing("Players", |ui| {
        ScrollArea::vertical()
            .auto_shrink([false, true])
            .stick_to_bottom(true)
            .max_height(screen_height() / 4.0)
            .show(ui, |ui| {
                for player in ctx.db().player().iter() {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "[{}] Credits: {}",
                            player.username, player.credits
                        ));
                        if let Some(_controller) =
                            ctx.db().ship_movement_controller().id().find(&player.id)
                        {
                            ui.label("Has Controller");
                        }
                    });
                }
                for ship_objs in ctx.db().ship().iter() {
                    ui.horizontal(|ui| {
                        ui.label(format!(
                            "{}: Sector: {}, Ship: {}, SO: {}",
                            ship_objs.player_id.to_abbreviated_hex(),
                            ship_objs.sector_id,
                            ship_objs.id,
                            ship_objs.sobj_id
                        ));
                    });
                }
            });
    });

    ui.label(format!("Now: {}", now()));
}
