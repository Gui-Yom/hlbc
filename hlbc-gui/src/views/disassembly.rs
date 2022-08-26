use eframe::egui;
use eframe::egui::{Ui, WidgetText};

use std::ops::Deref;

use hlbc::types::FunPtr;

use crate::{AppCtxHandle, AppTab};

#[derive(Default)]
pub(crate) struct DisassemblyView;

impl AppTab for DisassemblyView {
    fn title(&self) -> WidgetText {
        "Disassembly view".into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        egui::ScrollArea::vertical()
            .id_source("disassembly_scroll_area")
            .show(ui, |ui| {
                let code = ctx.code();
                if let Some(f) = ctx.selected_fn().map(|f| f.resolve(code.deref())) {
                    ui.code(match f {
                        FunPtr::Fun(fun) => fun.display(code.deref()).to_string(),
                        FunPtr::Native(n) => n.display_header(code.deref()),
                    });
                } else {
                    ui.label("Select a function in the Functions view to view its bytecode");
                }
            });
    }
}
