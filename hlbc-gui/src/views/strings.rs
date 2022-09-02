use std::ops::Deref;

use eframe::egui::style::Margin;
use eframe::egui::text::LayoutJob;
use eframe::egui::{Color32, Frame, RichText, ScrollArea, TextStyle, Ui, WidgetText};

use hlbc::types::RefString;

use crate::{AppCtxHandle, AppTab, ItemSelection};

#[derive(Default)]
pub(crate) struct StringsView;

impl AppTab for StringsView {
    fn title(&self) -> WidgetText {
        RichText::new("Strings").color(Color32::WHITE).into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        Frame::none()
            .inner_margin(Margin::same(4.0))
            .show(ui, |ui| {
                let num = ctx.code().strings.len();

                ScrollArea::both().auto_shrink([false, false]).show_rows(
                    ui,
                    ui.text_style_height(&TextStyle::Body),
                    num,
                    |ui, range| {
                        for s in range.map(RefString) {
                            let checked = match ctx.selected() {
                                ItemSelection::String(s2) => s == s2,
                                _ => false,
                            };
                            let job = LayoutJob::simple_singleline(
                                s.display(ctx.code().deref()),
                                TextStyle::Body.resolve(ui.style().as_ref()),
                                Color32::WHITE,
                            );
                            //job.wrap = TextWrapping::default();
                            if ui.selectable_label(checked, job).clicked() {
                                ctx.set_selected(ItemSelection::String(s));
                            }
                        }
                    },
                );
            });
    }
}
