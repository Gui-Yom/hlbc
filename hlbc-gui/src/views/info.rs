use std::ops::Deref;

use eframe::egui;
use eframe::egui::Ui;

use crate::egui::{Label, WidgetText};
use crate::views::AppTab;
use crate::AppCtxHandle;

#[derive(Default)]
pub(crate) struct InfoTab;

impl AppTab for InfoTab {
    fn title(&self) -> WidgetText {
        "🛈 Info".into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        egui::ScrollArea::vertical()
            .id_source("info_scroll_area")
            .show(ui, |ui| {
                egui::Grid::new("info_grid")
                    .striped(true)
                    .num_columns(2)
                    .show(ui, |ui| {
                        ui.label("File")
                            .on_hover_text("Currently opened bytecode file");
                        ui.label(ctx.file().to_str().unwrap());
                        ui.end_row();
                        let code = ctx.code();
                        let code = code.deref();
                        ui.label("Bytecode version")
                            .on_hover_text("Bytecode file version");
                        ui.label(code.version.to_string());
                        ui.end_row();
                        ui.label("Debug info")
                            .on_hover_text("Was the bytecode built with debug information ?");
                        ui.label(if code.debug_files.is_some() {
                            "yes"
                        } else {
                            "no"
                        });
                        ui.end_row();
                        ui.label("Integers").on_hover_text("Integer constants");
                        ui.label(code.ints.len().to_string());
                        ui.end_row();
                        ui.label("Floats").on_hover_text("Float constants");
                        ui.label(code.floats.len().to_string());
                        ui.end_row();
                        ui.label("Strings").on_hover_text("String constants");
                        ui.label(code.strings.len().to_string());
                        ui.end_row();
                        if let Some((_, pos)) = code.bytes.as_ref() {
                            ui.label("Bytes strings")
                                .on_hover_text("Bytes constants (since bytecode v5)");
                            ui.label(pos.len().to_string());
                            ui.end_row();
                        }
                        if let Some(files) = code.debug_files.as_ref() {
                            ui.label("Source files")
                                .on_hover_text("Linked source files (debug info)");
                            ui.label(files.len().to_string());
                            ui.end_row();
                        }
                        ui.label("Types")
                            .on_hover_text("Hashlink types (classes, functions, ...)");
                        ui.label(code.types.len().to_string());
                        ui.end_row();
                        ui.label("Globals").on_hover_text("Global variables");
                        ui.label(code.globals.len().to_string());
                        ui.end_row();
                        ui.label("Natives")
                            .on_hover_text("Native function references");
                        ui.label(code.natives.len().to_string());
                        ui.end_row();
                        ui.label("Functions")
                            .on_hover_text("Functions, methods, closures");
                        ui.label(code.functions.len().to_string());
                        ui.end_row();
                        if let Some(cst) = code.constants.as_ref() {
                            ui.label("Constant definitions")
                                .on_hover_text("Global variables initializers (since bytecode v4)");
                            ui.label(cst.len().to_string());
                            ui.end_row();
                        }
                    });
            });
    }
}