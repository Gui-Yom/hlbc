use eframe::egui::{Color32, RichText, ScrollArea, TextStyle, Ui, WidgetText};

use hlbc::analysis::files::functions_in_files;
use hlbc::fmt::EnhancedFmt;
use hlbc::types::RefFun;
use hlbc::{Bytecode, Str};

use crate::model::{AppCtxHandle, Item};
use crate::views::{impl_id, impl_view_id, singleline, AppView, DecompilerView, InspectorView};

pub struct FilesView {
    files: Vec<(Str, Vec<RefFun>)>,
}

impl_view_id!(FilesView: unique);

impl FilesView {
    pub fn new(code: &Bytecode) -> Self {
        Self {
            files: functions_in_files(code).into_iter().collect(),
        }
    }
}

impl AppView for FilesView {
    impl_id!(unique);

    fn title(&self, ctx: AppCtxHandle) -> WidgetText {
        RichText::new("Files").into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        ScrollArea::both().auto_shrink([false, false]).show_rows(
            ui,
            ui.text_style_height(&TextStyle::Button),
            self.files.len(),
            |ui, range| {
                for (file, funs) in &self.files[range] {
                    ui.collapsing(file.as_str(), |ui| {
                        for &f in funs {
                            let item = Item::Fun(f);
                            let checked = ctx.selected() == item;
                            let label = ui
                                .selectable_label(
                                    checked,
                                    singleline(
                                        f.display_header::<EnhancedFmt>(ctx.code()).to_string(),
                                        TextStyle::Button.resolve(ui.style().as_ref()),
                                        Color32::WHITE,
                                    ),
                                )
                                .context_menu(|ui| {
                                    if ui.small_button("Open in inspector").clicked() {
                                        let tab = InspectorView::new(item, ctx.code());
                                        ctx.open_tab(tab);
                                    }
                                    if ui.small_button("Decompile").clicked() {
                                        ctx.open_tab(DecompilerView::default());
                                    }
                                });
                            if label.clicked() {
                                ctx.set_selected(item);
                            }
                        }
                    });
                }
            },
        );
    }
}
