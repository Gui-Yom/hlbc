#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::cell::Cell;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::{env, fs};

use eframe::egui::style::Margin;
use eframe::egui::{CentralPanel, Frame, Rounding, TopBottomPanel, Vec2, Visuals};
use eframe::{egui, NativeOptions, Theme};
use egui_dock::{DockArea, NodeIndex, Tree};

use hlbc::types::{RefFun, RefGlobal, RefString, RefType};
use hlbc::Bytecode;

use crate::views::{
    AppView, ClassesView, DynamicTabViewer, FunctionsView, GlobalsView, InfoView, StringsView,
    SyncInspectorView,
};

mod views;

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "hlbc gui",
        NativeOptions {
            vsync: true,
            initial_window_size: Some(Vec2::new(1280.0, 720.0)),
            ..Default::default()
        },
        Box::new(|cc| {
            if cc
                .integration_info
                .system_theme
                .map(|t| matches!(t, Theme::Dark))
                .unwrap_or(true)
            {
                cc.egui_ctx.set_visuals(Visuals::dark());
            }

            let args = env::args().skip(1).collect::<String>();
            let ctx = if args.is_empty() {
                None
            } else {
                Some(AppCtxHandle::new(AppCtx::new_from_file(PathBuf::from(
                    args,
                ))))
            };

            // Dock tabs tree
            let tree = if ctx.is_some() {
                default_tabs_ui()
            } else {
                Tree::new(vec![])
            };

            // Dock tabs styling
            let mut style = egui_dock::Style::from_egui(cc.egui_ctx.style().as_ref());
            style.tab_outline_color = style.tab_bar_background_color;
            style.tab_rounding = Rounding {
                nw: 1.0,
                ne: 1.0,
                sw: 0.0,
                se: 0.0,
            };

            Box::new(App {
                ctx,
                tree,
                style,
                options_window_open: false,
                about_window_open: false,
            })
        }),
    )
}

struct App {
    /// Some when a file is loaded
    ctx: Option<AppCtxHandle>,
    // Dock
    tree: Tree<Box<dyn AppView>>,
    style: egui_dock::Style,
    options_window_open: bool,
    about_window_open: bool,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        {
            if let Some(tab) = self.ctx.as_ref().and_then(|app| app.take_tab_to_open()) {
                self.tree[NodeIndex::root().left()].append_tab(tab);
            }
        }

        TopBottomPanel::top("menu bar")
            .frame(Frame::none().outer_margin(Margin::same(4.0)))
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Open").clicked() {
                            if let Some(file) = rfd::FileDialog::new().pick_file() {
                                let appctx = AppCtxHandle::new(AppCtx::new_from_file(file));
                                self.tree = default_tabs_ui();
                                self.ctx = Some(appctx);
                            }
                        }
                        if ui.button("Close").clicked() {
                            self.ctx = None;
                            self.tree = Tree::new(vec![]);
                        }
                    });
                    if self.ctx.is_some() {
                        ui.menu_button("Views", |ui| {
                            if ui.button("Functions").clicked() {
                                self.tree[NodeIndex::root().right()]
                                    .append_tab(Box::<FunctionsView>::default());
                            }
                            if ui.button("Info").clicked() {
                                self.tree[NodeIndex::root().left()]
                                    .append_tab(Box::<InfoView>::default());
                            }
                        });
                    }
                    if ui.button("Options").clicked() {
                        self.options_window_open = !self.options_window_open;
                    }
                    ui.menu_button("Help", |ui| {
                        if ui.button("Wiki").clicked() {
                            webbrowser::open("https://github.com/Gui-Yom/hlbc/wiki")
                                .expect("Failed to open web browser");
                        }
                    });
                    if ui.button("About").clicked() {
                        self.about_window_open = !self.about_window_open;
                    }
                });
            });

        egui::Window::new("Options")
            .open(&mut self.options_window_open)
            .show(ctx, |ui| {
                ui.collapsing("Display", |_ui| {
                    // TODO max fps
                    // TODO ui theme
                });
                ui.collapsing("Code display", |_ui| {
                    // TODO code font
                    // TODO code font size
                    // TODO code theme
                });
            });

        egui::Window::new("About")
            .open(&mut self.about_window_open)
            .show(ctx, |ui| {
                ui.heading("Hashlink bytecode tools");
                // TODO about page
            });

        if let Some(appctx) = self.ctx.clone() {
            DockArea::new(&mut self.tree)
                .style(self.style.clone())
                .show(ctx, &mut DynamicTabViewer(appctx));
        } else {
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

/// Cheaply cloneable, for single threaded usage.
///
/// Usage warning ! The methods 'lock' the inner RefCell immutably and mutably (RW lock).
/// Be careful of guards (Ref<> and RefMut<>) lifetimes.
#[derive(Clone)]
struct AppCtxHandle(Rc<AppCtx>);

impl AppCtxHandle {
    fn new(appctx: AppCtx) -> Self {
        Self(Rc::new(appctx))
    }

    fn file(&self) -> &Path {
        &self.0.file
    }

    fn code(&self) -> &Bytecode {
        &self.0.code
    }

    /// mut lock
    fn open_tab(&self, tab: impl AppView + 'static) {
        self.0.new_tab.set(Some(Box::new(tab)));
    }

    fn take_tab_to_open(&self) -> Option<Box<dyn AppView>> {
        self.0.new_tab.take()
    }

    fn selected(&self) -> ItemSelection {
        self.0.selected.get()
    }

    fn set_selected(&self, s: ItemSelection) {
        self.0.selected.set(s);
    }
}

struct AppCtx {
    file: PathBuf,
    code: Bytecode,
    selected: Cell<ItemSelection>,
    /// To open a tab from another tab.
    /// This can't be done directly because this would need a mutable reference to a tree and the tree owns the tab.
    new_tab: Cell<Option<Box<dyn AppView>>>,
}

impl AppCtx {
    fn new_from_file(file: PathBuf) -> Self {
        let code = Bytecode::load(&mut BufReader::new(
            fs::File::open(&file).expect("Can't open file"),
        ))
        .expect("Can't load bytecode");
        AppCtx {
            file,
            code,
            selected: Cell::new(ItemSelection::None),
            new_tab: Cell::new(None),
        }
    }
}

fn default_tabs_ui() -> Tree<Box<dyn AppView>> {
    let mut tree: Tree<Box<dyn AppView>> = Tree::new(vec![
        Box::<SyncInspectorView>::default(),
        Box::<InfoView>::default(),
    ]);

    tree.split_left(
        NodeIndex::root(),
        0.20,
        vec![
            Box::<FunctionsView>::default(),
            Box::<GlobalsView>::default(),
            Box::<StringsView>::default(),
        ],
    );
    tree.split_below(
        NodeIndex::root().right(),
        0.5,
        vec![Box::<ClassesView>::default()],
    );

    tree
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
            ItemSelection::Fun(fun) => fun.display_id(code).to_string(),
            ItemSelection::Class(t) => t.display_id(code),
            ItemSelection::Global(g) => format!("global@{}", g.0),
            ItemSelection::String(s) => {
                format!("string@{}", s.0)
            }
            _ => String::new(),
        }
    }
}
