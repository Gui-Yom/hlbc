use eframe::egui::text::LayoutJob;
use eframe::egui::{Color32, Frame, Grid, Margin, RichText, ScrollArea, TextStyle, Ui, WidgetText};

use hlbc::fmt::EnhancedFmt;
use hlbc::types::{RefFun, RefString};
use hlbc::Bytecode;
use hlbc_indexing::BcIndex;

use crate::views::AppView;
use crate::{AppCtxHandle, ItemSelection};

pub(crate) struct SearchView {
    indexer: BcIndex,
    query_text: String,
    matches: Vec<RefFun>,
}

impl SearchView {
    pub fn new(code: &Bytecode) -> Self {
        Self {
            indexer: BcIndex::build_functions(code),
            query_text: String::new(),
            matches: Vec::new(),
        }
    }
}

impl AppView for SearchView {
    fn title(&self) -> WidgetText {
        RichText::new("Search").color(Color32::WHITE).into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        Frame::none()
            .inner_margin(Margin::same(4.0))
            .show(ui, |ui| {
                if ui.text_edit_singleline(&mut self.query_text).changed() {
                    self.matches = self.indexer.query(&self.query_text);
                }

                for f in &self.matches {
                    ui.label(f.display_header::<EnhancedFmt>(ctx.code()).to_string());
                }
            });
    }
}
