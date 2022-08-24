use eframe::egui;
use eframe::egui::style::Margin;
use eframe::egui::{Frame, Ui, WidgetText};
use egui_dock::Tab;

use hlbc::types::FunPtr;

use crate::{AppCtx, AppTab};

#[derive(Default)]
pub struct DisassemblyView;

impl AppTab for DisassemblyView {
    fn title(&self) -> WidgetText {
        "Disassembly view".into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: &mut AppCtx) {
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
    }
}
