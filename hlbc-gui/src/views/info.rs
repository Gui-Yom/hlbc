use eframe::egui;
use eframe::egui::style::Margin;
use eframe::egui::{Frame, Ui};
use egui_dock::Tab;

use crate::App;

#[derive(Default)]
pub struct InfoTab;

impl Tab<App> for InfoTab {
    fn title(&self) -> &str {
        "🛈 Info"
    }

    fn ui(&mut self, ui: &mut Ui, ctx: &mut App) {
        let margin = Margin::same(4.0);

        Frame::none().inner_margin(margin).show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_source("info_scroll_area")
                .show(ui, |ui| {
                    if let Some(code) = &ctx.bc {
                        egui::Grid::new("info_grid")
                            .striped(true)
                            .num_columns(2)
                            .show(ui, |ui| {
                                ui.label("File");
                                ui.label(ctx.file.as_ref().unwrap().to_str().unwrap());
                                ui.end_row();
                                ui.label("Bytecode version");
                                ui.label(code.version.to_string());
                                ui.end_row();
                            });
                    } else {
                        ui.label("No bytecode loaded");
                    }
                });
        });
    }
}
