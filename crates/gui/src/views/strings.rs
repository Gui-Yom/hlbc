use eframe::egui::{Color32, RichText, Ui, WidgetText};

use hlbc::types::RefString;

use crate::model::{AppCtxHandle, Item};
use crate::views::{list_view, make_id_method, unique_id};
use crate::AppView;

#[derive(Default)]
pub(crate) struct StringsView;

unique_id!(StringsView, "strings");

impl AppView for StringsView {
    make_id_method!(unique);

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
