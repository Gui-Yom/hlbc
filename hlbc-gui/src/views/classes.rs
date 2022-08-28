use std::ops::Deref;

use eframe::egui::style::Margin;
use eframe::egui::{
    Button, Color32, Frame, Grid, RichText, ScrollArea, SelectableLabel, TextStyle, Ui, WidgetText,
};

use hlbc::types::{RefType, Type};

use crate::{AppCtxHandle, AppTab, ItemSelection};

#[derive(Default)]
pub(crate) struct ClassesView {
    cache: Vec<(RefType, String)>,
}

impl AppTab for ClassesView {
    fn title(&self) -> WidgetText {
        RichText::new("Classes").color(Color32::WHITE).into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        {
            let code = ctx.code();
            let code = code.deref();

            if self.cache.is_empty() {
                for (i, t) in code.types.iter().enumerate() {
                    match t {
                        Type::Obj(obj) => {
                            self.cache
                                .push((RefType(i), obj.name.resolve(&code.strings).to_string()));
                        }
                        _ => {}
                    }
                }
            }
        }

        Frame::none()
            .inner_margin(Margin::same(4.0))
            .show(ui, |ui| {
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
                            if ui.selectable_label(checked, name).clicked() {
                                ctx.set_selected(ItemSelection::Class(*r));
                            }
                        }
                    },
                );
            });
    }
}
