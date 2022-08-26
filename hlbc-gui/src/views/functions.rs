use std::ops::Deref;

use eframe::egui::{ScrollArea, TextStyle, Ui, WidgetText};

use hlbc::types::RefFun;

use crate::views::{DecompilerView, DisassemblyView};
use crate::{AppCtxHandle, AppTab};

#[derive(Default)]
pub(crate) struct FunctionsView {
    show_natives: bool,
    show_std: bool,
    cache: Vec<RefFun>,
    cache_valid: bool,
}

impl AppTab for FunctionsView {
    fn title(&self) -> WidgetText {
        "ƒ Functions".into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        // Function list cache
        if !self.cache_valid {
            self.cache = Vec::new();
            let code = ctx.code();
            for fk in &ctx.code().findexes {
                let f = fk.resolve(code.deref());
                let findex = f.findex();
                if (self.show_std || !findex.is_from_std(code.deref()))
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
                        let (checked, text) = {
                            (
                                ctx.selected_fn().map(|s| s == f).unwrap_or(false),
                                f.display_header(ctx.code().deref()),
                            )
                        };
                        let btn = ui.selectable_label(checked, text).context_menu(|ui| {
                            if ui.small_button("View disassembly").clicked() {
                                ctx.open_tab(DisassemblyView::default());
                            }
                            if ui.small_button("Decompile").clicked() {
                                ctx.open_tab(DecompilerView::default());
                            }
                        });
                        if btn.clicked() {
                            ctx.set_selected_fn(f);
                        }
                    }
                },
            );
    }
}
