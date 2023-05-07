use eframe::egui::{Ui, WidgetText};
use egui_dock::TabViewer;

#[cfg(feature = "callgraph")]
pub(crate) use callgraph::*;
pub(crate) use classes::*;
pub(crate) use decompiler::*;
pub(crate) use functions::*;
pub(crate) use globals::*;
pub(crate) use info::*;
pub(crate) use inspector::*;
pub(crate) use strings::*;

use crate::AppCtxHandle;

#[cfg(feature = "callgraph")]
mod callgraph;
mod classes;
mod decompiler;
mod functions;
mod globals;
mod info;
mod inspector;
mod strings;

/// Tab viewer with dynamic dispatch because I don't care
pub(crate) struct DynamicTabViewer(pub(crate) AppCtxHandle);

impl TabViewer for DynamicTabViewer {
    type Tab = Box<dyn AppView>;

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        tab.ui(ui, self.0.clone());
    }

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.title()
    }
}

/// The actual trait that needs to be implemented by a view
pub(crate) trait AppView {
    fn title(&self) -> WidgetText;

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle);
}
