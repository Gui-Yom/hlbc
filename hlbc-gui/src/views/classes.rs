use std::ops::Deref;

use eframe::egui::style::Margin;
use eframe::egui::{Color32, Frame, RichText, ScrollArea, TextStyle, Ui, WidgetText};

use hlbc::analysis::IsFromStd;
use hlbc::types::{RefType, Type};

use crate::views::{DecompilerView, InspectorView};
use crate::{AppCtxHandle, AppView, ItemSelection};

#[derive(Default)]
pub(crate) struct ClassesView {
    show_std: bool,
    cache: Vec<(RefType, String)>,
    cache_valid: bool,
}

impl AppView for ClassesView {
    fn title(&self) -> WidgetText {
        RichText::new("Classes").color(Color32::WHITE).into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        if !self.cache_valid {
            let code = ctx.code();
            let code = code.deref();

            self.cache = Vec::new();
            for (i, t) in code.types.iter().enumerate() {
                match t {
                    Type::Obj(obj) => {
                        let should_show = self.show_std || !obj.is_from_std(code);
                        if should_show {
                            self.cache
                                .push((RefType(i), obj.name.resolve(&code.strings).to_string()));
                        }
                    }
                    _ => {}
                }
            }

            self.cache_valid = true;
        }

        Frame::none()
            .inner_margin(Margin::same(4.0))
            .show(ui, |ui| {
                if ui.checkbox(&mut self.show_std, "Show stdlib").changed() {
                    self.cache_valid = false;
                }

                ui.add_space(4.0);

                ScrollArea::both().auto_shrink([false, false]).show_rows(
                    ui,
                    ui.text_style_height(&TextStyle::Body),
                    self.cache.len(),
                    |ui, range| {
                        for (r, name) in &self.cache[range] {
                            let checked = match ctx.selected() {
                                ItemSelection::Class(r2) => *r == r2,
                                _ => false,
                            };
                            let label = ui.selectable_label(checked, name).context_menu(|ui| {
                                if ui.small_button("Open in inspector").clicked() {
                                    let tab = InspectorView::new(
                                        ItemSelection::Class(*r),
                                        ctx.code().deref(),
                                    );
                                    ctx.open_tab(tab);
                                }
                                if ui.small_button("Decompile").clicked() {
                                    ctx.open_tab(DecompilerView::default());
                                }
                            });
                            if label.clicked() {
                                ctx.set_selected(ItemSelection::Class(*r));
                            }
                        }
                    },
                );
            });
    }
}
