use eframe::egui::{Ui, WidgetText};

use crate::views::{haxe_source_view, impl_id, impl_view_id, AppView};
use crate::AppCtxHandle;

pub(crate) struct SourceView {
    name: &'static str,
    source: &'static str,
}

impl_view_id!(SourceView: unique);

impl SourceView {
    pub(crate) fn new(name: &'static str, source: &'static str) -> Self {
        Self { name, source }
    }
}

impl AppView for SourceView {
    impl_id!(unique);

    fn title(&self, ctx: AppCtxHandle) -> WidgetText {
        self.name.into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        haxe_source_view(ui, self.source);
    }

    fn closeable(&self) -> bool {
        false
    }
}
