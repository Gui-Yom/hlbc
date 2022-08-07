use eframe::egui;
use eframe::egui::style::Margin;
use eframe::egui::{Frame, Ui};
use egui_dock::Tab;

use hlbc::types::{FunPtr, RefFun, RefFunKnown};

use crate::AppCtx;

#[derive(Default)]
pub struct FunctionsTab {
    include_natives: bool,
}

impl Tab<AppCtx> for FunctionsTab {
    fn title(&self) -> &str {
        "ƒ Functions"
    }

    fn ui(&mut self, ui: &mut Ui, ctx: &mut AppCtx) {
        let margin = Margin::same(4.0);

        Frame::none().inner_margin(margin).show(ui, |ui| {
            ui.checkbox(&mut self.include_natives, "Include natives");
            egui::ScrollArea::vertical()
                .id_source("functions_scroll_area")
                .show(ui, |ui| {
                    for f in &ctx.code.findexes {
                        match f.resolve(&ctx.code) {
                            FunPtr::Fun(fun) => {
                                let res = ui.selectable_label(
                                    ctx.selected_fn.map(|s| s == fun.findex).unwrap_or(false),
                                    fun.display_header(&ctx.code),
                                );
                                if res.clicked() {
                                    ctx.selected_fn.insert(fun.findex);
                                }
                            }
                            FunPtr::Native(n) => {
                                if self.include_natives {
                                    let res = ui.selectable_label(
                                        ctx.selected_fn.map(|s| s == n.findex).unwrap_or(false),
                                        n.display_header(&ctx.code),
                                    );
                                    if res.clicked() {
                                        ctx.selected_fn.insert(n.findex);
                                    }
                                }
                            }
                        }
                    }
                });
        });
    }
}
