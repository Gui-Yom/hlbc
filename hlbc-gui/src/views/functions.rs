use std::ops::Deref;

use eframe::egui::style::Margin;
use eframe::egui::{Color32, Frame, RichText, ScrollArea, TextStyle, Ui, WidgetText};

use hlbc::analysis::IsFromStd;
use hlbc::fmt::EnhancedFmt;
use hlbc::types::RefFun;

use crate::views::{DecompilerView, InspectorView};
use crate::{AppCtxHandle, AppView, ItemSelection};

#[derive(Default)]
pub(crate) struct FunctionsView {
    show_natives: bool,
    show_std: bool,
    cache: Vec<RefFun>,
    cache_valid: bool,
}

impl AppView for FunctionsView {
    fn title(&self) -> WidgetText {
        RichText::new("ƒ Functions").color(Color32::WHITE).into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        // Function list cache
        if !self.cache_valid {
            self.cache = Vec::new();
            let code = ctx.code();
            for f in &code.functions {
                if self.show_std || !f.is_from_std(code) {
                    self.cache.push(f.findex);
                }
            }
            if self.show_natives {
                for n in &code.natives {
                    if self.show_std || !n.is_from_std(code) {
                        self.cache.push(n.findex);
                    }
                }
            }
            self.cache_valid = true;
        }

        Frame::none()
            .inner_margin(Margin::same(4.0))
            .show(ui, |ui| {
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

                ui.add_space(4.0);

                ScrollArea::vertical()
                    .id_source("functions_scroll_area")
                    .auto_shrink([false, false])
                    .show_rows(
                        ui,
                        ui.text_style_height(&TextStyle::Body),
                        self.cache.len(),
                        |ui, range| {
                            for f in range.map(|i| self.cache[i]) {
                                let text = {
                                    f.display_header::<EnhancedFmt>(ctx.code().deref())
                                        .to_string()
                                };
                                let selected = match ctx.selected() {
                                    ItemSelection::Fun(f2) => f == f2,
                                    _ => false,
                                };
                                let btn = ui.selectable_label(selected, text).context_menu(|ui| {
                                    if ui.small_button("Open in inspector").clicked() {
                                        let tab = InspectorView::new(
                                            ItemSelection::Fun(f),
                                            ctx.code().deref(),
                                        );
                                        ctx.open_tab(tab);
                                    }
                                    if ui.small_button("Decompile").clicked() {
                                        ctx.open_tab(DecompilerView::default());
                                    }
                                });
                                if btn.clicked() {
                                    ctx.set_selected(ItemSelection::Fun(f));
                                }
                            }
                        },
                    );
            });
    }
}
