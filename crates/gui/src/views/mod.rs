use eframe::egui::text::LayoutJob;
use eframe::egui::{Color32, FontId, ScrollArea, TextStyle, Ui, WidgetText};
use eframe::epaint::text::TextWrapping;
use egui_dock::TabViewer;

pub(crate) use classes::*;
pub(crate) use decompiler::*;
pub(crate) use functions::*;
pub(crate) use globals::*;
pub(crate) use info::*;
pub(crate) use inspector::*;
#[cfg(feature = "search")]
pub(crate) use search::*;
pub(crate) use strings::*;

use crate::{AppCtxHandle, ItemSelection};

#[cfg(feature = "callgraph")]
mod callgraph;
mod classes;
mod decompiler;
mod functions;
mod globals;
mod info;
mod inspector;
#[cfg(feature = "search")]
mod search;
mod strings;

/// Tab viewer with dynamic dispatch because I don't care
pub(crate) struct DynamicTabViewer(pub(crate) AppCtxHandle);

impl TabViewer for DynamicTabViewer {
    type Tab = Box<dyn AppView>;

    fn title(&mut self, tab: &mut Self::Tab) -> WidgetText {
        tab.title()
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        tab.ui(ui, self.0.clone());
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        tab.closeable()
    }

    fn scroll_bars(&self, tab: &Self::Tab) -> [bool; 2] {
        [false, false]
    }
}

/// The actual trait that needs to be implemented by a view
pub(crate) trait AppView {
    fn title(&self) -> WidgetText;

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle);

    fn closeable(&self) -> bool {
        true
    }
}

pub(crate) fn list_view<Elem: Copy>(
    ui: &mut Ui,
    ctx: AppCtxHandle,
    num: usize,
    item: impl Fn(usize) -> Elem,
    create_selection: impl Fn(Elem) -> ItemSelection,
    display: impl Fn(&AppCtxHandle, Elem) -> String,
    context_menu: Option<impl Fn(&mut Ui, &AppCtxHandle, Elem)>,
) {
    ScrollArea::both().auto_shrink([false, false]).show_rows(
        ui,
        ui.text_style_height(&TextStyle::Button),
        num,
        |ui, range| {
            for elem in range.map(item) {
                let checked = ctx.selected() == create_selection(elem);
                let mut label = ui.selectable_label(
                    checked,
                    singleline(
                        display(&ctx, elem),
                        TextStyle::Button.resolve(ui.style().as_ref()),
                        Color32::WHITE,
                    ),
                );
                if let Some(context_menu) = &context_menu {
                    label = label.context_menu(|ui| context_menu(ui, &ctx, elem));
                }
                if label.clicked() {
                    ctx.set_selected(create_selection(elem));
                }
            }
        },
    );
}

pub(crate) fn singleline_simple(ui: &Ui, text: impl Into<String>) -> LayoutJob {
    singleline(
        text,
        TextStyle::Body.resolve(ui.style().as_ref()),
        Color32::WHITE,
    )
}

pub(crate) fn singleline(text: impl Into<String>, font_id: FontId, color: Color32) -> LayoutJob {
    let mut job = LayoutJob::simple_singleline(text.into(), font_id, color);
    job.wrap = TextWrapping {
        break_anywhere: true,
        max_rows: 1,
        ..TextWrapping::default()
    };
    job
}
