use eframe::egui::{Color32, RichText, Ui, WidgetText};

use hlbc::types::RefGlobal;

use crate::views::list_view;
use crate::{AppCtxHandle, AppView, ItemSelection};

#[derive(Default)]
pub(crate) struct GlobalsView;

impl AppView for GlobalsView {
    fn title(&self) -> WidgetText {
        RichText::new("Globals").color(Color32::WHITE).into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        let num = ctx.code().globals.len();
        list_view(
            ui,
            ctx,
            num,
            RefGlobal,
            ItemSelection::Global,
            |_, g| format!("global@{}", g.0),
            None::<&dyn Fn(&mut Ui, &AppCtxHandle, RefGlobal)>,
        );
    }
}
