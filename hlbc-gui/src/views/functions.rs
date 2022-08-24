use eframe::egui;
use eframe::egui::style::Margin;
use eframe::egui::{Frame, ScrollArea, TextStyle, Ui, WidgetText};
use egui_dock::Tab;

use hlbc::types::RefFun;

use crate::{AppCtx, AppTab};

#[derive(Default)]
pub struct FunctionsView {
    show_natives: bool,
    show_std: bool,
    cache: Vec<RefFun>,
    cache_valid: bool,
}

impl AppTab for FunctionsView {
    fn title(&self) -> WidgetText {
        "ƒ Functions".into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: &mut AppCtx) {
        // Function list cache
        if !self.cache_valid {
            self.cache = Vec::new();
            for fk in &ctx.code.findexes {
                let f = fk.resolve(&ctx.code);
                let findex = f.findex();
                if (self.show_std || !findex.is_from_std(&ctx.code))
                    && (self.show_natives || f.is_fun())
                {
                    self.cache.push(findex);
                }
            }
            self.cache_valid = true;
        }

        ui.horizontal_wrapped(|ui| {
            if ui
                .checkbox(&mut self.show_natives, "Show natives")
                .changed()
            {
                self.cache_valid = false;
            }
            if ui.checkbox(&mut self.show_std, "Show stdlib").changed() {
                self.cache_valid = false;
            }
        });

        ScrollArea::vertical()
            .id_source("functions_scroll_area")
            .auto_shrink([false, false])
            .show_rows(
                ui,
                ui.text_style_height(&TextStyle::Body),
                self.cache.len(),
                |ui, range| {
                    for f in range.map(|i| self.cache[i]) {
                        let res = ui.selectable_label(
                            ctx.selected_fn.map(|s| s == f).unwrap_or(false),
                            f.display_header(&ctx.code),
                        );
                        if res.clicked() {
                            ctx.selected_fn = Some(f);
                        }
                    }
                },
            );
    }
}
