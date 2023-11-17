use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Instant;

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
        tab.title(self.0.clone())
    }

    fn ui(&mut self, ui: &mut Ui, tab: &mut Self::Tab) {
        ui.push_id(tab.id(), |ui| {
            tab.ui(ui, self.0.clone());
        });
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
    /// If a view has an id, it has a chance to be unique
    fn id(&self) -> ViewId {
        ViewId::Instantiable(0)
    }

    fn title(&self, ctx: AppCtxHandle) -> WidgetText;

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle);

    fn closeable(&self) -> bool {
        true
    }
}

impl PartialEq for dyn AppView {
    fn eq(&self, other: &Self) -> bool {
        self.id().eq(&other.id())
    }
}

pub(crate) trait DefaultAppView {
    fn default_view() -> Box<dyn AppView>;
}

impl<T: AppView + Default + 'static> DefaultAppView for T {
    #[inline]
    fn default_view() -> Box<dyn AppView> {
        Box::<T>::default()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) enum ViewId {
    /// The view is unique, it can only exist as one instance. The key is used to search for it.
    Unique(&'static str),
    /// The view can be instantiated many times. The id is to differentiate the UIs.
    Instantiable(u64),
}

impl Default for ViewId {
    fn default() -> Self {
        // Yes I'm actually doing it
        // poor man's rng
        let mut hasher = DefaultHasher::new();
        Instant::now().hash(&mut hasher);
        ViewId::Instantiable(hasher.finish())
    }
}

impl ViewId {
    pub(crate) fn is_unique(&self) -> bool {
        matches!(self, ViewId::Unique(_))
    }
}

pub(crate) trait UniqueAppView: AppView {
    const ID: ViewId = ViewId::Instantiable(0);
}

macro_rules! unique_id {
    ($view:ty, $id:literal) => {
        crate::views::unique_id!($view, crate::views::ViewId::Unique($id));
    };
    ($view:ty, $id:expr) => {
        impl crate::views::UniqueAppView for $view {
            const ID: crate::views::ViewId = $id;
        }
    };
}

macro_rules! not_unique {
    ($view:ty) => {
        crate::views::unique_id!($view, crate::views::ViewId::Instantiable(0));
    };
}

macro_rules! make_id_method {
    () => {
        fn id(&self) -> crate::views::ViewId {
            self.id
        }
    };

    (unique) => {
        fn id(&self) -> crate::views::ViewId {
            <Self as crate::views::UniqueAppView>::ID
        }
    };
}

pub(crate) use make_id_method;
pub(crate) use not_unique;
pub(crate) use unique_id;

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
