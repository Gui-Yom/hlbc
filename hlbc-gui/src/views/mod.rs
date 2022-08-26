use eframe::egui::style::Margin;
use eframe::egui::{Ui, WidgetText};
use egui_dock::Tab;

#[cfg(feature = "callgraph")]
pub(crate) use callgraph::*;
pub(crate) use decompiler::*;
pub(crate) use disassembly::*;
pub(crate) use functions::*;
pub(crate) use info::*;

use crate::AppCtxHandle;

#[cfg(feature = "callgraph")]
mod callgraph;
mod decompiler;
mod disassembly;
mod functions;
mod info;

pub(crate) trait AppTab: Sized + 'static {
    fn title(&self) -> WidgetText;

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle);

    fn make_tab(mut self, ctx: AppCtxHandle) -> Tab {
        Tab {
            title: self.title(),
            inner_margin: Margin::same(4.0),
            add_content: Box::new(move |ui| {
                self.ui(ui, ctx.clone());
            }),
        }
    }
}
