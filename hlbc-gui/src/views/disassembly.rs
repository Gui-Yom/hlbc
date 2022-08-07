use eframe::egui;
use eframe::egui::style::Margin;
use eframe::egui::{Frame, Ui};
use egui_dock::Tab;

use hlbc::types::FunPtr;

use crate::AppCtx;

#[derive(Default)]
pub struct DisassemblyTab;

impl Tab<AppCtx> for DisassemblyTab {
    fn title(&self) -> &str {
        "Disassembly view"
    }

    fn ui(&mut self, ui: &mut Ui, ctx: &mut AppCtx) {
        let margin = Margin::same(4.0);

        Frame::none().inner_margin(margin).show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_source("disassembly_scroll_area")
                .show(ui, |ui| {
                    if let Some(f) = ctx.selected_fn.map(|f| f.resolve(&ctx.code)) {
                        ui.code(match f {
                            FunPtr::Fun(fun) => fun.display(&ctx.code).to_string(),
                            FunPtr::Native(n) => n.display_header(&ctx.code),
                        });
                    } else {
                        ui.label("Select a function in the Functions view to view its bytecode");
                    }
                });
        });
    }
}
