#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::cell::{Ref, RefCell, RefMut};
use std::io::BufReader;
use std::path::PathBuf;
use std::rc::Rc;
use std::{env, fs};

use eframe::egui::style::Margin;
use eframe::egui::{CentralPanel, Frame, Rounding, TopBottomPanel, Vec2, Visuals};
use eframe::{egui, NativeOptions, Theme};
use egui_dock::{DockSpace, NodeIndex, Tab, Tree};

use hlbc::types::RefFun;
use hlbc::Bytecode;

use crate::views::{AppTab, FunctionsView, InfoTab};

mod views;

fn main() {
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
            let tree = if let Some(ctx) = ctx.clone() {
                default_tabs_ui(ctx)
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
    );
}

struct App {
    /// Some when a file is loaded
    ctx: Option<AppCtxHandle>,
    // Dock
    tree: Tree,
    style: egui_dock::Style,
    options_window_open: bool,
    about_window_open: bool,
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        {
            if let Some(tab) = self.ctx.as_ref().and_then(|app| app.new_tab()) {
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
                                self.tree = default_tabs_ui(appctx.clone());
                                self.ctx = Some(appctx);
                            }
                        }
                        if ui.button("Close").clicked() {
                            self.ctx = None;
                            self.tree = Tree::new(vec![]);
                        }
                    });
                    if let Some(appctx) = self.ctx.as_ref() {
                        ui.menu_button("Views", |ui| {
                            if ui.button("Functions").clicked() {
                                self.tree[NodeIndex::root().right()]
                                    .append_tab(FunctionsView::default().make_tab(appctx.clone()));
                            }
                            if ui.button("Info").clicked() {
                                self.tree[NodeIndex::root().left()]
                                    .append_tab(InfoTab::default().make_tab(appctx.clone()));
                            }
                        });
                    }
                    if ui.button("Options").clicked() {
                        self.options_window_open = !self.options_window_open;
                    }
                    if ui.button("About").clicked() {
                        self.about_window_open = !self.about_window_open;
                    }
                });
            });

        egui::Window::new("Options")
            .open(&mut self.options_window_open)
            .show(ctx, |ui| {
                ui.collapsing("Display", |ui| {
                    // TODO max fps
                    // TODO ui theme
                });
                ui.collapsing("Code display", |ui| {
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

        if self.ctx.is_some() {
            DockSpace::new(&mut self.tree)
                .style(self.style.clone())
                .show(ctx);
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
/// Usage warning ! The methods 'lock' the inner RefCell immutably and mutably.
/// Be careful of guards (Ref<> and RefMut<>) lifetimes.
#[derive(Clone)]
struct AppCtxHandle(Rc<RefCell<AppCtx>>);

impl AppCtxHandle {
    fn new(appctx: AppCtx) -> Self {
        Self(Rc::new(RefCell::new(appctx)))
    }

    fn lock(&self) -> Ref<AppCtx> {
        self.0.borrow()
    }

    fn lock_mut(&self) -> RefMut<AppCtx> {
        self.0.borrow_mut()
    }

    fn code(&self) -> Ref<Bytecode> {
        Ref::map(RefCell::borrow(self.0.as_ref()), |app| &app.code)
    }

    fn open_tab(&self, tab: impl AppTab) {
        self.0.borrow_mut().new_tab = Some(tab.make_tab(self.clone()));
    }

    fn new_tab(&self) -> Option<Tab> {
        self.0.borrow_mut().new_tab.take()
    }

    fn selected_fn(&self) -> Option<RefFun> {
        self.lock().selected_fn.clone()
    }

    fn set_selected_fn(&self, fun: RefFun) {
        self.0.borrow_mut().selected_fn = Some(fun);
    }
}

struct AppCtx {
    file: PathBuf,
    code: Bytecode,
    selected_fn: Option<RefFun>,
    /// To open a tab from another tab.
    /// This can't be done directly because this would need a mutable reference to a tree and the tree owns the tab.
    new_tab: Option<Tab>,
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
            selected_fn: None,
            new_tab: None,
        }
    }
}

fn default_tabs_ui(ctx: AppCtxHandle) -> Tree {
    let mut tree = Tree::new(vec![InfoTab::default().make_tab(ctx.clone())]);

    tree.split_left(
        NodeIndex::root(),
        0.20,
        vec![FunctionsView::default().make_tab(ctx.clone())],
    );
    tree
}
