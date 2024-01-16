use std::any::type_name;
use std::hash::{Hash, Hasher};

use eframe::egui::{Ui, WidgetText};
use egui_dock::TabViewer;

pub(crate) use classes::*;
pub(crate) use decompiler::*;
pub(crate) use files::*;
pub(crate) use functions::*;
pub(crate) use globals::*;
pub(crate) use haxe_source_view::*;
pub(crate) use info::*;
pub(crate) use inspector::*;
#[cfg(feature = "search")]
pub(crate) use search::*;
#[cfg(feature = "examples")]
pub(crate) use source::*;
pub(crate) use strings::*;

use crate::model::AppCtxHandle;

#[cfg(feature = "callgraph")]
mod callgraph;
mod classes;
mod decompiler;
mod files;
mod functions;
mod globals;
mod haxe_source_view;
mod info;
mod inspector;
#[cfg(feature = "search")]
mod search;
#[cfg(feature = "examples")]
mod source;
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
    fn id(&self) -> ViewId;

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

pub(crate) trait ViewWithId {
    const ID: ViewId;
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub(crate) enum ViewId {
    /// The view is unique, it can only exist as one instance. The key is used to search for it.
    Unique(&'static str),
    /// The view can be instantiated many times. The id is to differentiate the UIs.
    Instantiable(&'static str, u64),
}

impl ViewId {
    pub(crate) fn new_instance<T: 'static>() -> Self {
        use std::collections::hash_map::DefaultHasher;
        use std::time::Instant;
        // poor man's rng
        let mut hasher = DefaultHasher::new();
        Instant::now().hash(&mut hasher);
        ViewId::Instantiable(
            type_name::<T>().split("::").last().unwrap(),
            hasher.finish(),
        )
    }

    pub(crate) const fn new_instance_const(class: &'static str) -> Self {
        ViewId::Instantiable(class, 0)
    }

    pub(crate) const fn new_unique(class: &'static str) -> Self {
        ViewId::Unique(class)
    }

    pub(crate) fn is_unique(&self) -> bool {
        matches!(self, ViewId::Unique(_))
    }

    pub(crate) fn class(&self) -> &'static str {
        match self {
            ViewId::Unique(class) => class,
            ViewId::Instantiable(class, _) => class,
        }
    }
}

macro_rules! impl_view_id {
    ($view:ty: unique) => {
        impl crate::views::ViewWithId for $view {
            const ID: crate::views::ViewId = crate::views::ViewId::new_unique(stringify!($view));
        }
    };
    ($view:ty) => {
        impl crate::views::ViewWithId for $view {
            const ID: crate::views::ViewId =
                crate::views::ViewId::new_instance_const(stringify!($view));
        }
    };
}

macro_rules! impl_id {
    (unique) => {
        fn id(&self) -> crate::views::ViewId {
            <Self as crate::views::ViewWithId>::ID
        }
    };
    () => {
        fn id(&self) -> crate::views::ViewId {
            self.id
        }
    };
}

pub(crate) use impl_id;
pub(crate) use impl_view_id;

#[cfg(test)]
mod tests {
    use crate::views::ViewId;

    struct A;

    #[test]
    fn test_type_name() {
        assert_eq!(ViewId::new_instance::<A>().class(), stringify!(A));
    }
}
