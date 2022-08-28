use std::borrow::Borrow;
use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui::{Ui, WidgetText};
use egui_dock::Tab;

#[cfg(feature = "callgraph")]
pub(crate) use callgraph::*;
pub(crate) use classes::*;
pub(crate) use decompiler::*;
pub(crate) use disassembly::*;
pub(crate) use functions::*;
pub(crate) use info::*;

use crate::AppCtxHandle;

#[cfg(feature = "callgraph")]
mod callgraph;
mod classes;
mod decompiler;
mod disassembly;
mod functions;
mod info;

pub(crate) trait AppTab: Sized + 'static {
    fn title(&self) -> WidgetText;

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle);

    fn make_tab(mut self, ctx: AppCtxHandle) -> Box<dyn Tab> {
        let tab = Rc::new(RefCell::new(self));
        let tabc = tab.clone();
        Box::new(BuiltTab {
            title: Box::new(move || RefCell::borrow(tabc.as_ref()).title()),
            ui: Box::new(move |ui| {
                tab.borrow_mut().ui(ui, ctx.clone());
            }),
        })
    }
}

pub struct BuiltTab {
    title: Box<dyn Fn() -> WidgetText>,
    ui: Box<dyn FnMut(&mut Ui)>,
}

impl Tab for BuiltTab {
    fn ui(&mut self, ui: &mut Ui) {
        (self.ui)(ui);
    }

    fn title(&mut self) -> WidgetText {
        (self.title)()
    }
}
