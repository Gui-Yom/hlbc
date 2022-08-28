use std::ops::Deref;

use eframe::egui::{Color32, RichText, ScrollArea, Ui, WidgetText};

use hlbc::types::FunPtr;

use crate::{AppCtxHandle, AppTab, ItemSelection};

#[derive(Default)]
pub(crate) struct DisassemblyView;

impl AppTab for DisassemblyView {
    fn title(&self) -> WidgetText {
        RichText::new("Disassembly").color(Color32::WHITE).into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        ScrollArea::vertical()
            .id_source("disassembly_scroll_area")
            .show(ui, |ui| {
                let code = ctx.code();
                let code = code.deref();
                match ctx.selected() {
                    ItemSelection::Fun(fun) => {
                        ui.code(match fun.resolve(code) {
                            FunPtr::Fun(fun) => fun.display(code).to_string(),
                            FunPtr::Native(n) => n.display_header(code),
                        });
                    }
                    ItemSelection::Class(t) => {
                        ui.label("Class inspector");
                    }
                    ItemSelection::None => {
                        ui.label("Select a function in the Functions view to view its bytecode");
                    }
                }
            });
    }
}
