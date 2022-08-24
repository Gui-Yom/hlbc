use eframe::egui;
use eframe::egui::Ui;

use crate::egui::WidgetText;
use crate::views::AppTab;
use crate::AppCtx;

#[derive(Default)]
pub struct InfoTab;

impl AppTab for InfoTab {
    fn title(&self) -> WidgetText {
        "🛈 Info".into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: &mut AppCtx) {
        egui::ScrollArea::vertical()
            .id_source("info_scroll_area")
            .show(ui, |ui| {
                egui::Grid::new("info_grid")
                    .striped(true)
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("File");
                        ui.label(ctx.file.to_str().unwrap());
                        ui.end_row();
                        ui.label("Bytecode version");
                        ui.label(ctx.code.version.to_string());
                        ui.end_row();
                    });
            });
    }
}
