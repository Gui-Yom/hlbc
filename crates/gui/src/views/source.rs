use eframe::egui::{Ui, WidgetText};

use crate::views::{haxe_source_view, make_id_method, unique_id, AppView};
use crate::AppCtxHandle;

pub(crate) struct SourceView {
    name: &'static str,
    source: &'static str,
}

unique_id!(SourceView, "source");

impl SourceView {
    pub(crate) fn new(name: &'static str, source: &'static str) -> Self {
        Self { name, source }
    }
}

impl AppView for SourceView {
    make_id_method!(unique);

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
