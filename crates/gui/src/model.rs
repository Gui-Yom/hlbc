use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::rc::Rc;

use hlbc::analysis::usage::{usage_report, FullUsageReport};
use hlbc::fmt::EnhancedFmt;
use hlbc::types::{RefFun, RefGlobal, RefString, RefType};
use hlbc::Bytecode;

use crate::views::AppView;

/// Cheaply cloneable, for single threaded usage.
#[derive(Clone)]
pub(crate) struct AppCtxHandle(Rc<AppCtx>);

macro_rules! delegate {
    ($name:ident) => {
        pub(crate) fn $name(&self) {
            self.0.$name();
        }
    };
    ($name:ident; $t:ty) => {
        pub(crate) fn $name(&self) -> $t {
            self.0.$name()
        }
    };
}

impl AppCtxHandle {
    pub(crate) fn new(appctx: AppCtx) -> Self {
        Self(Rc::new(appctx))
    }

    pub(crate) fn file(&self) -> String {
        self.0.file.clone()
    }

    pub(crate) fn code(&self) -> &Bytecode {
        &self.0.code
    }

    pub(crate) fn usage(&self) -> &FullUsageReport {
        &self.0.usage
    }

    pub(crate) fn open_tab(&self, tab: impl AppView + 'static) {
        self.0.new_tab.set(Some(Box::new(tab)));
    }

    pub(crate) fn take_tab_to_open(&self) -> Option<Box<dyn AppView>> {
        self.0.new_tab.take()
    }

    delegate!(selected; Item);

    pub(crate) fn set_selected(&self, s: Item) {
        self.0.navigate_to(s)
    }

    delegate!(can_navigate_forward; bool);
    delegate!(can_navigate_back; bool);
    delegate!(navigate_forward);
    delegate!(navigate_back);
}

/// Arbitrary value, should we let it grow indefinitely instead ?
const NAVIGATION_HISTORY_MAX: usize = 64;

pub(crate) struct AppCtx {
    file: String,
    code: Bytecode,
    usage: FullUsageReport,
    /// Selection index in the navigation history buffer
    selection: Cell<usize>,
    /// Ring buffer of navigation history
    navigation_history: RefCell<VecDeque<Item>>,
    /// To open a tab from another tab.
    /// This can't be done directly because this would need a mutable reference to a tree and the tree owns the tab.
    new_tab: Cell<Option<Box<dyn AppView>>>,
}

impl AppCtx {
    pub(crate) fn new_from_code(file: String, code: Bytecode) -> Self {
        let usage = usage_report(&code);
        Self {
            file,
            code,
            usage,
            selection: Cell::new(0),
            new_tab: Cell::new(None),
            navigation_history: RefCell::new(VecDeque::with_capacity(NAVIGATION_HISTORY_MAX)),
        }
    }

    /// Navigate to a new selection
    fn navigate_to(&self, item: Item) {
        if matches!(item, Item::None) {
            panic!("Cannot navigate to ItemSelection::None");
        }
        let mut nav_history = self.navigation_history.borrow_mut();
        let len = nav_history.len();

        // Remove future elements
        if len > 0 && self.selection.get() < len - 1 {
            nav_history.drain((self.selection.get() + 1)..len);
        }

        // Do not grow past the limit
        if nav_history.len() == nav_history.capacity() {
            nav_history.pop_front();
        }

        nav_history.push_back(item);
        self.selection.set(nav_history.len() - 1)
    }

    fn can_navigate_back(&self) -> bool {
        self.selection.get() > 0
    }

    /// Navigate back in selection history
    fn navigate_back(&self) {
        if self.can_navigate_back() {
            self.selection.set(self.selection.get() - 1);
        }
    }

    fn can_navigate_forward(&self) -> bool {
        !self.navigation_history.borrow().is_empty()
            && self.selection.get() < self.navigation_history.borrow().len() - 1
    }

    /// Navigate forward in selection history
    fn navigate_forward(&self) {
        if self.can_navigate_forward() {
            self.selection.set(self.selection.get() + 1);
        }
    }

    /// Return the currently selected element
    fn selected(&self) -> Item {
        self.navigation_history
            .borrow()
            .get(self.selection.get())
            .copied()
            .unwrap_or(Item::None)
    }
}

#[derive(Clone, Default, Copy, Eq, PartialEq)]
pub(crate) enum Item {
    Fun(RefFun),
    Type(RefType),
    Global(RefGlobal),
    String(RefString),
    #[default]
    None,
}

impl Item {
    pub(crate) fn name(&self, code: &Bytecode) -> String {
        match self {
            Item::Fun(fun) => fun.display::<EnhancedFmt>(code).to_string(),
            Item::Type(t) => t.display::<EnhancedFmt>(code).to_string(),
            Item::Global(g) => format!("global@{}", g.0),
            Item::String(s) => {
                format!("string@{}", s.0)
            }
            _ => String::new(),
        }
    }
}
