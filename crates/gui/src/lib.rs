use std::borrow::Cow;
use std::fs;

use eframe::egui;
use eframe::egui::{Button, CentralPanel, Frame, Margin, ScrollArea, TopBottomPanel, Ui, Vec2};
use egui_dock::{DockArea, DockState, Node, NodeIndex, Split, SurfaceIndex};
use poll_promise::Promise;

use hlbc::Bytecode;

use crate::model::{AppCtx, AppCtxHandle};
use crate::views::{
    AppView, ClassesView, DefaultAppView, DynamicTabViewer, FilesView, FunctionsView, GlobalsView,
    InfoView, StringsView, SyncInspectorView, ViewWithId,
};

mod about;
#[cfg(feature = "examples")]
mod examples;
mod model;
mod shortcuts;
mod style;
mod views;

pub const HLBC_ICON: &[u8] = include_bytes!("../../../assets/hlbc.ico");

pub type BytecodeLoader = Promise<hlbc::Result<Option<(String, Bytecode)>>>;

pub struct App {
    /// Asynchronous loader for bytecode
    loader: Option<BytecodeLoader>,
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
    pub fn new(loader: Option<BytecodeLoader>, style: egui_dock::Style) -> Self {
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
            if ctx.input_mut(|i| i.consume_shortcut(&shortcuts::OPEN)) {
                self.open_file();
            } else if ctx.input_mut(|i| i.consume_shortcut(&shortcuts::CLOSE)) {
                self.close_file();
            }

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
                self.dock_state.main_surface_mut().push_to_focused_leaf(tab);
            }
        }

        // UI
        self.menu_bar(ctx);

        if let Some(appctx) = self.ctx.clone() {
            self.status_bar(ctx);
            DockArea::new(&mut self.dock_state)
                .style(self.style.clone())
                .show(ctx, &mut DynamicTabViewer(appctx));
        } else {
            // Blank panel if no file is loaded
            CentralPanel::default()
                .frame(Frame::none())
                .show(ctx, |ui| {
                    self.homepage(ui);
                });
        }

        self.windows(ctx);
    }
}

impl App {
    fn homepage(&mut self, ui: &mut Ui) {
        ui.vertical_centered(|ui| {
            ui.add_space(100.0);
            ui.label(style::text("hlbc", style::get().heading_title.clone()));
            ui.label(style::text(
                "Load a bytecode file to start",
                style::get().heading_subtitle.clone(),
            ));
            #[cfg(target_arch = "wasm32")]
            ui.label(style::text(
                "Your file stays local and is not uploaded to any server",
                style::get().heading_subtitle.clone(),
            ));
            ui.add_space(10.0);

            // TODO homepage icons
            if ui
                .add(
                    Button::new(style::text(
                        "Open file",
                        style::get().homepage_button.clone(),
                    ))
                        .shortcut_text(ui.ctx().format_shortcut(&shortcuts::OPEN)),
                )
                .on_hover_text(if cfg!(target_arch = "wasm32") {
                    "Load a bytecode file. Everything stays local."
                } else {
                    "Load a bytecode file"
                })
                .clicked()
            {
                self.open_file();
            }

            #[cfg(feature = "examples")]
            ui.menu_button(
                style::text("Example", style::get().homepage_button.clone()),
                |ui| {
                    for example in examples::EXAMPLES {
                        if ui.button(example.name).clicked() {
                            self.load_example(example);
                        }
                    }
                },
            );
        });
    }

    /// Create a button which opens a view.
    /// If the view is supposed to be unique, focus the view instead.
    fn view_button_default<T: DefaultAppView + ViewWithId>(
        dock: &mut DockState<Box<dyn AppView>>,
        ui: &mut Ui,
        name: &str,
    ) {
        Self::view_button::<T>(dock, ui, name, || T::default_view())
    }

    fn view_button<T: ViewWithId>(
        dock: &mut DockState<Box<dyn AppView>>,
        ui: &mut Ui,
        name: &str,
        creator: impl FnOnce() -> Box<dyn AppView>,
    ) {
        let id = T::ID;

        if ui.button(name).clicked() {
            let to_focus = if id.is_unique() {
                // If the view is unique, look for an already existing instance
                dock.main_surface()
                    .iter()
                    .filter(|n| n.tabs().is_some())
                    .find_map(|n| n.tabs().unwrap().iter().find(|&tab| tab.id() == id))
            } else {
                None
            };
            if let Some(t) = to_focus {
                let Some(locator) = dock.find_tab(t) else {
                    unreachable!()
                };
                dock.set_active_tab(locator);
            } else {
                dock.main_surface_mut().push_to_focused_leaf(creator());
            }
        }
    }

    #[cfg(feature = "examples")]
    fn load_examples_button(&mut self, ui: &mut Ui) {
        ui.menu_button("Load example", |ui| {
            for example in examples::EXAMPLES {
                if ui.button(example.name).clicked() {
                    self.load_example(example);
                }
            }
        });
    }

    #[cfg(feature = "examples")]
    fn load_example(&mut self, example: examples::Example) {
        let mut cursor = std::io::Cursor::new(example.data);
        let code = Bytecode::deserialize(&mut cursor).unwrap();
        self.ctx = Some(AppCtxHandle::new(AppCtx::new_from_code(
            example.name.to_owned(),
            code,
        )));
        self.dock_state = default_tabs();
        self.dock_state.main_surface_mut()[NodeIndex::root().right()].append_tab(Box::new(
            views::SourceView::new(example.name, example.source),
        ));
        self.status = Cow::Borrowed("Loaded example successfully");
    }

    fn open_file(&mut self) {
        #[cfg(target_arch = "wasm32")]
        {
            self.loader = Some(Promise::spawn_local(async {
                if let Some(file) = rfd::AsyncFileDialog::new().pick_file().await {
                    Ok(Some((
                        file.file_name(),
                        Bytecode::deserialize(&file.read().await[..]).unwrap(),
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
                        Bytecode::from_file(file)?,
                    )))
                } else {
                    Ok(None)
                }
            }));
        }
    }

    fn menu_bar(&mut self, ctx: &egui::Context) {
        TopBottomPanel::top("menu bar")
            .frame(Frame::none().outer_margin(Margin::same(4.0)))
            .show(ctx, |ui| {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui
                            .add(
                                Button::new("Open")
                                    .shortcut_text(ctx.format_shortcut(&shortcuts::OPEN)),
                            )
                            .on_hover_text(if cfg!(target_arch = "wasm32") {
                                "Load a bytecode file. Everything stays local."
                            } else {
                                "Load a bytecode file"
                            })
                            .clicked()
                        {
                            self.open_file();
                        }

                        #[cfg(feature = "examples")]
                        self.load_examples_button(ui);

                        if ui
                            .add(
                                Button::new("Close")
                                    .shortcut_text(ctx.format_shortcut(&shortcuts::CLOSE)),
                            )
                            .clicked()
                        {
                            self.close_file();
                        }
                    });
                    if let Some(ctx) = &self.ctx {
                        ui.menu_button("Views", |ui| {
                            Self::view_button_default::<InfoView>(&mut self.dock_state, ui, "Info");
                            Self::view_button_default::<ClassesView>(
                                &mut self.dock_state,
                                ui,
                                "Classes",
                            );
                            Self::view_button_default::<FunctionsView>(
                                &mut self.dock_state,
                                ui,
                                "Functions",
                            );
                            Self::view_button::<FilesView>(
                                &mut self.dock_state,
                                ui,
                                "Files",
                                || Box::new(FilesView::new(ctx.code())),
                            );
                            Self::view_button_default::<GlobalsView>(
                                &mut self.dock_state,
                                ui,
                                "Globals",
                            );
                            Self::view_button_default::<StringsView>(
                                &mut self.dock_state,
                                ui,
                                "Strings",
                            );
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
                                .add_enabled(
                                    ctx.can_navigate_back(),
                                    Button::new("Back").shortcut_text(
                                        ui.ctx().format_shortcut(&shortcuts::NAV_BACK),
                                    ),
                                )
                                .clicked()
                            {
                                ctx.navigate_back();
                            }

                            if ui
                                .add_enabled(
                                    ctx.can_navigate_forward(),
                                    Button::new("Forward").shortcut_text(
                                        ui.ctx().format_shortcut(&shortcuts::NAV_FORWARD),
                                    ),
                                )
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
                            .label(appctx.selected().name(appctx.code()));
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

        about::about_window(ctx, &mut self.about_window_open);
    }
    fn close_file(&mut self) {
        self.ctx = None;
        self.dock_state = DockState::new(Vec::new())
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
