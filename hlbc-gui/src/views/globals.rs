use std::ops::Deref;

use eframe::egui::style::Margin;
use eframe::egui::{Color32, Frame, RichText, ScrollArea, TextStyle, Ui, WidgetText};

use hlbc::types::RefGlobal;

use crate::{AppCtxHandle, AppTab, ItemSelection};

#[derive(Default)]
pub(crate) struct GlobalsView;

impl AppTab for GlobalsView {
    fn title(&self) -> WidgetText {
        RichText::new("Globals").color(Color32::WHITE).into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        Frame::none()
            .inner_margin(Margin::same(4.0))
            .show(ui, |ui| {
                let num = ctx.code().globals.len();

                ScrollArea::both().auto_shrink([false, false]).show_rows(
                    ui,
                    ui.text_style_height(&TextStyle::Body),
                    num,
                    |ui, range| {
                        for g in range.map(RefGlobal) {
                            let checked = match ctx.selected() {
                                ItemSelection::Global(g2) => g == g2,
                                _ => false,
                            };
                            if ui
                                .selectable_label(checked, format!("global@{}", g.0))
                                .clicked()
                            {
                                ctx.set_selected(ItemSelection::Global(g));
                            }
                        }
                    },
                );
            });
    }
}
