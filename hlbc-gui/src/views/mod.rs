use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui::{Ui, WidgetText};
use egui_dock::Tab;

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

pub(crate) trait AppTab: Sized + 'static {
    fn title(&self) -> WidgetText;

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle);

    fn make_tab(self, ctx: AppCtxHandle) -> Box<dyn Tab> {
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

pub(crate) struct BuiltTab {
    pub(crate) title: Box<dyn Fn() -> WidgetText>,
    pub(crate) ui: Box<dyn FnMut(&mut Ui)>,
}

impl Tab for BuiltTab {
    fn ui(&mut self, ui: &mut Ui) {
        (self.ui)(ui);
    }

    fn title(&mut self) -> WidgetText {
        (self.title)()
    }
}
