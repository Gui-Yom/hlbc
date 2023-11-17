use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::collections::VecDeque;
use std::fs;
use std::io::BufReader;
use std::rc::Rc;

use eframe::egui;
use eframe::egui::{Button, CentralPanel, Frame, Margin, ScrollArea, TopBottomPanel, Ui, Vec2};
use egui_dock::{DockArea, DockState, Node, NodeIndex, Split, SurfaceIndex};
use poll_promise::Promise;

use hlbc::fmt::EnhancedFmt;
use hlbc::types::{RefFun, RefGlobal, RefString, RefType};
use hlbc::Bytecode;

use crate::views::{
    AppView, ClassesView, DefaultAppView, DynamicTabViewer, FunctionsView, GlobalsView, InfoView,
    StringsView, SyncInspectorView, UniqueAppView,
};

mod views;

pub struct App {
    /// Asynchronous loader for bytecode
    loader: Option<Promise<hlbc::Result<Option<(String, Bytecode)>>>>,
    /// Some when a file is loaded
    ctx: Option<AppCtxHandle>,
    // Dock
    dock_state: DockState<Box<dyn AppView>>,
    style: egui_dock::Style,
    options_window_open: bool,
    about_window_open: bool,
    status: Cow<'static, str>,
}

impl App {
    pub fn new(
        loader: Option<Promise<hlbc::Result<Option<(String, Bytecode)>>>>,
        style: egui_dock::Style,
    ) -> Self {
        let is_loading = loader.is_some();
        Self {
            loader,
            ctx: None,
            dock_state: DockState::new(Vec::new()),
            style,
            options_window_open: false,
            about_window_open: false,
            status: Cow::Borrowed(if is_loading {
                "Loading bytecode ..."
            } else {
                "No bytecode file loaded."
            }),
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Update part
        {
            if let Some(loader) = self.loader.take() {
                match loader.try_take() {
                    Ok(Ok(Some((file, code)))) => {
                        self.ctx = Some(AppCtxHandle::new(AppCtx::new_from_code(file, code)));
                        self.dock_state = default_tabs();
                        self.status = Cow::Borrowed("Loaded bytecode successfully");
                    }
                    Ok(Ok(None)) => {
                        // No file has been picked
                    }
                    Ok(Err(e)) => {
                        println!("{e}");
                    }
                    Err(loader) => {
                        self.loader = Some(loader);
                        ctx.request_repaint();
                    }
                }
            }

            if let Some(tab) = self.ctx.as_ref().and_then(|app| app.take_tab_to_open()) {
                self.dock_state.main_surface_mut()[NodeIndex::root().right()].append_tab(tab);
            }
        }

        // UI
        self.menu_bar(ctx);
        self.status_bar(ctx);
        self.windows(ctx);

        if let Some(appctx) = self.ctx.clone() {
            DockArea::new(&mut self.dock_state)
                .style(self.style.clone())
                .show(ctx, &mut DynamicTabViewer(appctx));
        } else {
            // Blank panel if no file is loaded
            CentralPanel::default()
                .frame(Frame::group(ctx.style().as_ref()).outer_margin(Margin::same(4.0)))
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label("Load a bytecode file to start");
                    });
                });
        }
    }
}

impl App {
    fn view_button<T: UniqueAppView + DefaultAppView>(
        dock: &mut DockState<Box<dyn AppView>>,
        ui: &mut Ui,
        name: &str,
    ) {
        let id = T::ID;

        if ui.button(name).clicked() {
            let to_focus = if id.is_unique() {
                // If the view is unique, look for an already existing instance
                dock.main_surface()
                    .iter()
                    .filter(|n| n.tabs().is_some())
                    .find_map(|n| {
                        for tab in n.tabs().unwrap() {
                            if tab.id() == id {
                                return Some(tab);
                            }
                        }
                        None
                    })
            } else {
                None
            };
            if let Some(t) = to_focus {
                let Some(locator) = dock.find_tab(t) else {
                    unreachable!()
                };
                dock.set_active_tab(locator);
            } else {
                dock.main_surface_mut()
                    .push_to_focused_leaf(T::default_view());
            }
        }
    }

    fn menu_bar(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("menu bar")
            .frame(Frame::none().outer_margin(Margin::same(4.0)))
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Open").clicked() {
                            #[cfg(target_arch = "wasm32")]
                            {
                                self.loader = Some(Promise::spawn_local(async {
                                    if let Some(file) =
                                        rfd::AsyncFileDialog::new().pick_file().await
                                    {
                                        Ok(Some((
                                            file.file_name(),
                                            Bytecode::deserialize(&mut &file.read().await[..])
                                                .unwrap(),
                                        )))
                                    } else {
                                        Ok(None)
                                    }
                                }));
                            }
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                self.loader = Some(Promise::spawn_thread("bg_loader", || {
                                    if let Some(file) = rfd::FileDialog::new().pick_file() {
                                        Ok(Some((
                                            file.display().to_string(),
                                            Bytecode::deserialize(&mut BufReader::new(
                                                fs::File::open(&file)?,
                                            ))?,
                                        )))
                                    } else {
                                        Ok(None)
                                    }
                                }));
                            }
                        }
                        if ui.button("Close").clicked() {
                            self.ctx = None;
                            self.dock_state = DockState::new(Vec::new())
                        }
                    });
                    if let Some(ctx) = &self.ctx {
                        ui.menu_button("Views", |ui| {
                            Self::view_button::<InfoView>(&mut self.dock_state, ui, "Info");
                            Self::view_button::<ClassesView>(&mut self.dock_state, ui, "Classes");
                            Self::view_button::<FunctionsView>(
                                &mut self.dock_state,
                                ui,
                                "Functions",
                            );
                            Self::view_button::<GlobalsView>(&mut self.dock_state, ui, "Globals");
                            Self::view_button::<StringsView>(&mut self.dock_state, ui, "Strings");
                            #[cfg(feature = "search")]
                            if ui.button("Search").clicked() {
                                self.dock_state
                                    .main_surface_mut()
                                    .push_to_focused_leaf(Box::new(views::SearchView::new(
                                        ctx.code(),
                                    )));
                            }
                        });

                        ui.menu_button("Navigate", |ui| {
                            if ui
                                .add_enabled(ctx.0.can_navigate_back(), Button::new("Back"))
                                .clicked()
                            {
                                ctx.navigate_back();
                            }

                            if ui
                                .add_enabled(ctx.0.can_navigate_forward(), Button::new("Forward"))
                                .clicked()
                            {
                                ctx.navigate_forward();
                            }
                        });
                    }
                    if ui.button("Options").clicked() {
                        self.options_window_open = !self.options_window_open;
                    }
                    ui.menu_button("Help", |ui| {
                        if ui.button("Wiki").clicked() {
                            webbrowser::open("https://github.com/Gui-Yom/hlbc/wiki").ok();
                        }
                        if ui
                            .button("Issues")
                            .on_hover_text("Report bugs, feature requests")
                            .clicked()
                        {
                            webbrowser::open("https://github.com/Gui-Yom/hlbc/issues").ok();
                        }
                        if ui
                            .button("Discussions")
                            .on_hover_text("Q&A, feature requests")
                            .clicked()
                        {
                            webbrowser::open("https://github.com/Gui-Yom/hlbc/discussions").ok();
                        }
                        ui.label("Discord: limelion")
                            .on_hover_text("No discord server yet, dm me instead");
                        if ui.button("Contact by email").clicked() {
                            webbrowser::open("mailto:guillaume.anthouard+hlbc@hotmail.fr").ok();
                        }
                        if ui.button("About").clicked() {
                            self.about_window_open = !self.about_window_open;
                        }
                    });
                });
            });
    }

    fn status_bar(&mut self, ctx: &egui::Context) {
        TopBottomPanel::bottom("status bar")
            .exact_height(20.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    if let Some(appctx) = &self.ctx {
                        let (id, rect) = ui.allocate_space(Vec2::new(120.0, 20.0));
                        Ui::new(ctx.clone(), ui.layer_id(), id, rect, rect)
                            .label(format!("{}", appctx.selected().name(appctx.code())));
                        ui.separator();
                    }
                    ui.label(self.status.clone());
                });
            });
    }

    fn windows(&mut self, ctx: &egui::Context) {
        egui::Window::new("Options")
            .open(&mut self.options_window_open)
            .show(ctx, |ui| {
                ui.collapsing("Display", |ui| {
                    // TODO max fps
                    // TODO ui theme
                    #[cfg(debug_assertions)]
                    ScrollArea::vertical().show(ui, |ui| {
                        ctx.style_ui(ui);
                    });
                });
                ui.collapsing("Code display", |_ui| {
                    // TODO code font
                    // TODO code font size
                    // TODO code theme
                });
            });

        egui::Window::new("About")
            .open(&mut self.about_window_open)
            .resizable(false)
            .collapsible(false)
            .fixed_size((300., 200.))
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Hashlink bytecode tools");
                    ui.hyperlink("https://github.com/Gui-Yom/hlbc");
                    ui.horizontal(|ui| {
                        ui.label("Made by");
                        ui.hyperlink_to("Gui-Yom", "https://github.com/Gui-Yom");
                        ui.label("and");
                        ui.hyperlink_to(
                            "contributors",
                            "https://github.com/Gui-Yom/hlbc/graphs/contributors",
                        );
                    });
                });
            });
    }
}

fn default_tabs() -> DockState<Box<dyn AppView>> {
    let mut dock_state: DockState<Box<dyn AppView>> = DockState::new(vec![
        InfoView::default_view(),
        SyncInspectorView::default_view(),
    ]);

    dock_state.split(
        (SurfaceIndex::main(), NodeIndex::root()),
        Split::Left,
        0.2,
        Node::leaf_with(vec![
            FunctionsView::default_view(),
            ClassesView::default_view(),
        ]),
    );

    dock_state.split(
        (SurfaceIndex::main(), NodeIndex::root().left()),
        Split::Below,
        0.5,
        Node::leaf_with(vec![
            StringsView::default_view(),
            GlobalsView::default_view(),
        ]),
    );

    dock_state
}

/// Cheaply cloneable, for single threaded usage.
#[derive(Clone)]
struct AppCtxHandle(Rc<AppCtx>);

impl AppCtxHandle {
    fn new(appctx: AppCtx) -> Self {
        Self(Rc::new(appctx))
    }

    fn file(&self) -> String {
        self.0.file.clone()
    }

    fn code(&self) -> &Bytecode {
        &self.0.code
    }

    fn open_tab(&self, tab: impl AppView + 'static) {
        self.0.new_tab.set(Some(Box::new(tab)));
    }

    fn take_tab_to_open(&self) -> Option<Box<dyn AppView>> {
        self.0.new_tab.take()
    }

    fn selected(&self) -> ItemSelection {
        self.0.selected()
    }

    fn set_selected(&self, s: ItemSelection) {
        self.0.navigate_to(s)
    }

    fn navigate_back(&self) {
        self.0.navigate_back();
    }

    fn navigate_forward(&self) {
        self.0.navigate_forward();
    }
}

/// Arbitrary value, should we let it grow indefinitely instead ?
const NAVIGATION_HISTORY_MAX: usize = 64;

struct AppCtx {
    file: String,
    code: Bytecode,
    /// Selection index in the navigation history buffer
    selection: Cell<usize>,
    /// Ring buffer of navigation history
    navigation_history: RefCell<VecDeque<ItemSelection>>,
    /// To open a tab from another tab.
    /// This can't be done directly because this would need a mutable reference to a tree and the tree owns the tab.
    new_tab: Cell<Option<Box<dyn AppView>>>,
}

impl AppCtx {
    fn new_from_code(file: String, code: Bytecode) -> Self {
        Self {
            file,
            code,
            selection: Cell::new(0),
            new_tab: Cell::new(None),
            navigation_history: RefCell::new(VecDeque::with_capacity(NAVIGATION_HISTORY_MAX)),
        }
    }

    /// Navigate to a new selection
    fn navigate_to(&self, item: ItemSelection) {
        if matches!(item, ItemSelection::None) {
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
    fn selected(&self) -> ItemSelection {
        self.navigation_history
            .borrow()
            .get(self.selection.get())
            .copied()
            .unwrap_or(ItemSelection::None)
    }
}

#[derive(Clone, Default, Copy, Eq, PartialEq)]
enum ItemSelection {
    Fun(RefFun),
    Class(RefType),
    Global(RefGlobal),
    String(RefString),
    #[default]
    None,
}

impl ItemSelection {
    pub(crate) fn name(&self, code: &Bytecode) -> String {
        match self {
            ItemSelection::Fun(fun) => fun.display::<EnhancedFmt>(code).to_string(),
            ItemSelection::Class(t) => t.display::<EnhancedFmt>(code).to_string(),
            ItemSelection::Global(g) => format!("global@{}", g.0),
            ItemSelection::String(s) => {
                format!("string@{}", s.0)
            }
            _ => String::new(),
        }
    }
}
