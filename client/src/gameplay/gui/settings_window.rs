use egui::{Color32, Context, Grid};
use spacetimedb_sdk::DbContext;

use crate::server::bindings::*;

pub struct State;

impl State {
    pub fn new() -> Self {
        Self
    }
}

pub fn draw(
    egui_ctx: &Context,
    ctx: &DbConnection,
    _state: &mut State,
    open: &mut bool,
) -> Option<egui::InnerResponse<Option<()>>> {
    let config = ctx.db().global_config().id().find(&0u32);

    egui::Window::new("Settings")
        .open(open)
        .title_bar(true)
        .resizable(true)
        .collapsible(true)
        .movable(true)
        .vscroll(false)
        .default_width(320.0)
        .show(egui_ctx, |ui| {
            ui.heading("Server Info");
            ui.separator();
            if let Some(config) = config {
                Grid::new("settings_grid")
                    .striped(true)
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("Version");
                        ui.label(&config.version);
                        ui.end_row();

                        ui.label("Active Players");
                        ui.label(config.active_players.to_string());
                        ui.end_row();

                        ui.label("Combat");
                        let (label, color) = if config.combat_enabled {
                            ("Enabled", Color32::GREEN)
                        } else {
                            ("Disabled", Color32::RED)
                        };
                        ui.colored_label(color, label);
                        ui.end_row();
                    });
            } else {
                ui.colored_label(Color32::GRAY, "No server config available.");
            }
        })
}
