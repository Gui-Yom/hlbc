use eframe::egui;
use eframe::egui::style::Margin;
use eframe::egui::{Frame, Ui};
use egui_dock::Tab;

use hlbc::types::{FunPtr, RefFun, RefFunKnown};

use crate::App;

#[derive(Default)]
pub struct FunctionsTab {
    include_natives: bool,
}

impl Tab<App> for FunctionsTab {
    fn title(&self) -> &str {
        "ƒ Functions"
    }

    fn ui(&mut self, ui: &mut Ui, ctx: &mut App) {
        let margin = Margin::same(4.0);

        Frame::none().inner_margin(margin).show(ui, |ui| {
            ui.checkbox(&mut self.include_natives, "Include natives");
            egui::ScrollArea::vertical()
                .id_source("functions_scroll_area")
                .show(ui, |ui| {
                    if let Some(code) = &ctx.bc {
                        for f in code.findexes.iter() {
                            match f.resolve(code) {
                                FunPtr::Fun(fun) => {
                                    let res = ui.selectable_label(
                                        ctx.selected_fn.map(|s| s == fun.findex).unwrap_or(false),
                                        fun.display_header(code),
                                    );
                                    if res.clicked() {
                                        ctx.selected_fn.insert(fun.findex);
                                    }
                                }
                                FunPtr::Native(n) => {
                                    if self.include_natives {
                                        let res = ui.selectable_label(
                                            ctx.selected_fn.map(|s| s == n.findex).unwrap_or(false),
                                            n.display_header(code),
                                        );
                                        if res.clicked() {
                                            ctx.selected_fn.insert(n.findex);
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        ui.label("No bytecode loaded");
                    }
                });
        });
    }
}
