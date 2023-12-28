use eframe::egui::{Color32, RichText, Ui, WidgetText};

use hlbc::types::RefGlobal;

use crate::model::{AppCtxHandle, Item};
use crate::views::{impl_id, impl_view_id, list_view};
use crate::AppView;

#[derive(Default)]
pub(crate) struct GlobalsView;

impl_view_id!(GlobalsView: unique);

impl AppView for GlobalsView {
    impl_id!(unique);

    fn title(&self, _ctx: AppCtxHandle) -> WidgetText {
        RichText::new("Globals").color(Color32::WHITE).into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        let num = ctx.code().globals.len();
        list_view(
            ui,
            ctx,
            num,
            RefGlobal,
            Item::Global,
            |_, g| format!("global@{}", g.0),
            None::<&dyn Fn(&mut Ui, &AppCtxHandle, RefGlobal)>,
        );
    }
}
