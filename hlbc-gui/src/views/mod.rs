use std::cell::RefCell;
use std::rc::Rc;

use eframe::egui::style::Margin;
use eframe::egui::{Ui, WidgetText};
use egui_dock::Tab;

#[cfg(feature = "callgraph")]
pub use callgraph::*;
pub use decompiler::*;
pub use disassembly::*;
pub use functions::*;
pub use info::*;

use crate::AppCtx;

#[cfg(feature = "callgraph")]
mod callgraph;
mod decompiler;
mod disassembly;
mod functions;
mod info;

pub(crate) trait AppTab: Sized + 'static {
    fn title(&self) -> WidgetText;

    fn ui(&mut self, ui: &mut Ui, ctx: &mut AppCtx);

    fn make_tab(mut self, ctx: Rc<RefCell<Option<AppCtx>>>) -> Tab {
        Tab {
            title: self.title(),
            inner_margin: Margin::same(4.0),
            add_content: Box::new(move |ui| {
                if let Some(ctx) = ctx.borrow_mut().as_mut() {
                    self.ui(ui, ctx);
                } else {
                    eprintln!("Draw tab without loaded file");
                }
            }),
        }
    }
}
