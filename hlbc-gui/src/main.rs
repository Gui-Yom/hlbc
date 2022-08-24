#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::cell::RefCell;
use std::io::BufReader;
use std::path::PathBuf;
use std::rc::Rc;
use std::{env, fs};

use eframe::egui::style::Margin;
use eframe::egui::{CentralPanel, Frame, Rounding, TopBottomPanel, Vec2, Visuals};
use eframe::{egui, NativeOptions, Theme};
use egui_dock::{DockSpace, NodeIndex, Tree};

use hlbc::types::RefFun;
use hlbc::Bytecode;

use crate::views::{
    AppTab, CallgraphView, DecompilerView, DisassemblyView, FunctionsView, InfoTab,
};

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
            let ctx = Rc::new(RefCell::new(if args.is_empty() {
                None
            } else {
                Some(AppCtx::new_from_file(PathBuf::from(args)))
            }));

            // Dock tabs tree
            let mut tree = Tree::new(vec![
                InfoTab::default().make_tab(ctx.clone()),
                DisassemblyView::default().make_tab(ctx.clone()),
            ]);

            tree.split_left(
                NodeIndex::root(),
                0.25,
                vec![FunctionsView::default().make_tab(ctx.clone())],
            );

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
                app: ctx,
                tree,
                style,
            })
        }),
    );
}

struct App {
    /// Some when a file is loaded
    app: Rc<RefCell<Option<AppCtx>>>,
    // Dock
    tree: Tree,
    style: egui_dock::Style,
}

struct AppCtx {
    file: PathBuf,
    code: Bytecode,
    selected_fn: Option<RefFun>,
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
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        TopBottomPanel::top("menu bar")
            .frame(Frame::none().outer_margin(Margin::same(4.0)))
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Open").clicked() {
                            if let Some(file) = rfd::FileDialog::new().pick_file() {
                                *self.app.borrow_mut() = Some(AppCtx::new_from_file(file));
                            }
                        }
                        if ui.button("Close").clicked() {
                            *self.app.borrow_mut() = None;
                        }
                    });
                    ui.menu_button("Views", |ui| {
                        if ui.button("Functions").clicked() {
                            println!("open functions view");
                        }
                        if ui.button("Disassembly").clicked() {
                            println!("open disassembly view");
                        }
                        if ui.button("Decompiler").clicked() {
                            self.tree[NodeIndex::root().right()]
                                .append_tab(DecompilerView::default().make_tab(self.app.clone()));
                        }
                        if ui.button("Info").clicked() {
                            /*
                            self.tree.split_right(
                                NodeIndex::root().left(),
                                0.8,
                                vec![Box::new(InfoTab::default())],
                            );*/
                        }
                    });
                });
            });

        if self.app.borrow().is_some() {
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
