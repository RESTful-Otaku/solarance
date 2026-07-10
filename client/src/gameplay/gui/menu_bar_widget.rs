
use egui::{Align2, Color32, Context, FontId, Frame, RichText, Shadow, Ui};

use crate::{gameplay::state::GameState, server::bindings::*};

pub fn draw(egui_ctx: &Context, _ctx: &DbConnection, game_state: &mut GameState) -> Option<egui::InnerResponse<Option<()>>> {
    egui::Window
        ::new("Menu Bar")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .movable(false)
        .vscroll(false)
        .frame(Frame::group(&egui_ctx.style()).fill(Color32::from_rgb(15, 15, 15)).shadow(Shadow::NONE))
        .anchor(Align2::CENTER_TOP, egui::Vec2::new(0.0, 0.0))
        .show(egui_ctx, |ui| {
            ui.horizontal(|ui| {
              toggable_label(ui, "[R] SHIP", &mut game_state.windows.details);
              ui.separator();
              toggable_label(ui, "[F]ACTION", &mut game_state.windows.faction);
              ui.separator();
              toggable_label(ui, "ASSE[T]S", &mut game_state.windows.assets);
              ui.separator();
              toggable_label(ui, "[M]AP", &mut game_state.windows.map);
              ui.separator();
              toggable_label(ui, "[B]UILD", &mut game_state.windows.construction);
              ui.separator();
              toggable_label(ui, "SETTINGS", &mut game_state.windows.settings);
            });
        })
}

fn toggable_label(ui: &mut Ui, label: &str, open: &mut bool) {
  if ui.selectable_label(*open, RichText::new(label).font(FontId::proportional(20.0))).clicked() {
    *open = !*open;
  }
}