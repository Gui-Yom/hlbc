use eframe::egui::{Color32, RichText, Ui, WidgetText};

use hlbc::types::RefString;

use crate::model::{AppCtxHandle, Item};
use crate::style::list_view;
use crate::views::{impl_id, impl_view_id};
use crate::AppView;

#[derive(Default)]
pub(crate) struct StringsView;

impl_view_id!(StringsView: unique);

impl AppView for StringsView {
    impl_id!(unique);

    fn title(&self, _ctx: AppCtxHandle) -> WidgetText {
        RichText::new("Strings").color(Color32::WHITE).into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        let num = ctx.code().strings.len();
        list_view(
            ui,
            ctx,
            num,
            RefString,
            Item::String,
            |ctx, s| ctx.code()[s].to_string(),
            None::<&dyn Fn(&mut Ui, &AppCtxHandle, RefString)>,
        );
    }
}
