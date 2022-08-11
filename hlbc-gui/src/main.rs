#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::io::BufReader;
use std::path::PathBuf;
use std::{env, fs};

use eframe::egui::style::Margin;
use eframe::egui::{CentralPanel, Frame, Id, LayerId, Rounding, TopBottomPanel, Ui, Vec2, Visuals};
use eframe::{egui, NativeOptions};
use egui_dock::{NodeIndex, Tree};

use hlbc::types::RefFun;
use hlbc::Bytecode;

use crate::views::CallgraphTab;
use crate::views::DisassemblyTab;
use crate::views::FunctionsTab;
use crate::views::InfoTab;

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
            if cc.integration_info.prefer_dark_mode.unwrap_or(true) {
                cc.egui_ctx.set_visuals(Visuals::dark());
            }

            // Dock tabs tree
            let mut tree = Tree::new(vec![
                Box::new(DisassemblyTab::default()),
                Box::new(CallgraphTab::default()),
            ]);

            tree.split_left(
                NodeIndex::root(),
                0.25,
                vec![Box::new(FunctionsTab::default())],
            );

            // Dock tabs styling
            let mut style = egui_dock::Style::from_egui(cc.egui_ctx.style().as_ref());
            style.tab_outline = style.tab_bar_background;
            style.tab_rounding = Rounding {
                nw: 1.0,
                ne: 1.0,
                sw: 0.0,
                se: 0.0,
            };

            let args = env::args().skip(1).collect::<String>();

            Box::new(App {
                app: if args.is_empty() {
                    None
                } else {
                    Some(AppCtx::new_from_file(PathBuf::from(args)))
                },
                tree,
                style,
            })
        }),
    );
}

struct App {
    /// Some when a file is loaded
    app: Option<AppCtx>,
    // Dock
    tree: Tree<AppCtx>,
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
                                self.app = Some(AppCtx::new_from_file(file));
                            }
                        }
                        if ui.button("Close").clicked() {
                            self.app = None;
                        }
                    });
                    ui.menu_button("Views", |ui| {
                        if ui.button("Functions").clicked() {
                            println!("open functions view");
                        }
                        if ui.button("Disassembly").clicked() {
                            println!("open disassembly view");
                        }
                        if ui.button("Info").clicked() {
                            self.tree.split_right(
                                NodeIndex::root().left(),
                                0.8,
                                vec![Box::new(InfoTab::default())],
                            );
                        }
                    });
                });
            });

        if let Some(appctx) = self.app.as_mut() {
            let layer_id = LayerId::background();
            let max_rect = ctx.available_rect();
            let clip_rect = ctx.available_rect();

            let mut ui = Ui::new(
                ctx.clone(),
                layer_id,
                Id::new("Docking space"),
                max_rect,
                clip_rect,
            );
            let id = ui.id();
            egui_dock::show(&mut ui, id, &self.style, &mut self.tree, appctx);
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
