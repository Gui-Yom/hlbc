#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::io::BufReader;
use std::{env, fs};

use eframe::egui::style::Margin;
use eframe::egui::{Frame, Id, LayerId, RichText, Ui, Vec2, Visuals};
use eframe::{egui, NativeOptions};
use egui_dock::{NodeIndex, Tab, Tree};

use hlbc::types::{FunPtr, RefFun};
use hlbc::Bytecode;

fn main() {
    eframe::run_native(
        "hlbc gui",
        NativeOptions {
            vsync: true,
            initial_window_size: Some(Vec2::new(1600.0, 900.0)),
            ..Default::default()
        },
        Box::new(|cc| {
            if cc.integration_info.prefer_dark_mode.unwrap_or(true) {
                cc.egui_ctx.set_visuals(Visuals::dark());
            }

            let mut tree = Tree::new(vec![Box::new(DisassemblyTab::default())]);

            tree.split_left(
                NodeIndex::root(),
                0.4,
                vec![Box::new(FunctionsTab::default())],
            );

            Box::new(DockApp {
                app: App {
                    bc: Bytecode::load(&mut BufReader::new(
                        fs::File::open(env::args().skip(1).collect::<String>())
                            .expect("Can't open file"),
                    ))
                    .ok(),
                    selected: None,
                },
                tree,
                style: egui_dock::Style::from_egui(cc.egui_ctx.style().as_ref()),
            })
        }),
    );
}

struct DockApp {
    app: App,
    // Dock
    tree: Tree<App>,
    style: egui_dock::Style,
}

struct App {
    bc: Option<Bytecode>,
    selected: Option<RefFun>,
}

impl eframe::App for DockApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let layer_id = LayerId::background();
        let max_rect = ctx.available_rect();
        let clip_rect = ctx.available_rect();

        let mut ui = Ui::new(
            ctx.clone(),
            layer_id,
            Id::new("DockApp"),
            max_rect,
            clip_rect,
        );
        let id = ui.id();
        egui_dock::show(&mut ui, id, &self.style, &mut self.tree, &mut self.app);
    }
}

#[derive(Default)]
struct DisassemblyTab {}

impl Tab<App> for DisassemblyTab {
    fn title(&self) -> &str {
        "Disassembly view"
    }

    fn ui(&mut self, ui: &mut Ui, ctx: &mut App) {
        let margin = Margin::same(4.0);

        Frame::none().inner_margin(margin).show(ui, |ui| {
            if let Some(code) = &ctx.bc {
                if let Some(f) = ctx.selected.map(|f| f.resolve(code)) {
                    match f {
                        FunPtr::Fun(fun) => {
                            ui.label(RichText::new(fun.display(code).to_string()).code());
                        }
                        FunPtr::Native(n) => {
                            ui.label(RichText::new(n.display_header(code).to_string()).code());
                        }
                    }
                } else {
                    ui.label("Select a function in the Functions view to view its bytecode");
                }
            } else {
                ui.label("No bytecode loaded");
            }
        });
    }
}

#[derive(Default)]
struct FunctionsTab {}

impl Tab<App> for FunctionsTab {
    fn title(&self) -> &str {
        "Functions"
    }

    fn ui(&mut self, ui: &mut Ui, ctx: &mut App) {
        let margin = Margin::same(4.0);

        Frame::none().inner_margin(margin).show(ui, |ui| {
            egui::ScrollArea::new([false, true]).show(ui, |ui| {
                if let Some(code) = &ctx.bc {
                    for f in &code.functions {
                        let l = ui.selectable_label(
                            ctx.selected.map(|s| s == f.findex).unwrap_or(false),
                            f.display_header(code),
                        );
                        if l.clicked() {
                            ctx.selected.insert(f.findex);
                        }
                    }
                } else {
                    ui.label("No bytecode loaded");
                }
            });
        });
    }
}
