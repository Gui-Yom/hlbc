use eframe::egui::{Color32, RichText, Ui, WidgetText};

use hlbc::types::RefString;

use crate::views::list_view;
use crate::{AppCtxHandle, AppView, ItemSelection};

#[derive(Default)]
pub(crate) struct StringsView;

impl AppView for StringsView {
    fn title(&self) -> WidgetText {
        RichText::new("Strings").color(Color32::WHITE).into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        let num = ctx.code().strings.len();
        list_view(
            ui,
            ctx,
            num,
            RefString,
            ItemSelection::String,
            |ctx, s| ctx.code()[s].to_owned(),
            None::<&dyn Fn(&mut Ui, &AppCtxHandle, RefString)>,
        );
    }
}
