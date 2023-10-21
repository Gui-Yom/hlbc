use eframe::egui::{Color32, ComboBox, RichText, Ui, WidgetText};

use hlbc::fmt::EnhancedFmt;
use hlbc::types::RefFun;
use hlbc::Bytecode;
use hlbc_indexing::{ClangdSearcher, Contains, Searcher, SkimSearcher};

use crate::views::{singleline_simple, AppView};
use crate::AppCtxHandle;

pub(crate) struct SearchView {
    searcher: (SearchMethod, Box<dyn Searcher>),
    query_text: String,
    matches: Vec<RefFun>,
}

#[derive(PartialEq, Copy, Clone)]
enum SearchMethod {
    Contains,
    Clangd,
    Skim,
}

impl SearchMethod {
    fn searcher(&self) -> Box<dyn Searcher> {
        match self {
            SearchMethod::Contains => Box::new(Contains),
            SearchMethod::Clangd => Box::new(ClangdSearcher::new()),
            SearchMethod::Skim => Box::new(SkimSearcher::new()),
        }
    }

    fn name(&self) -> &'static str {
        match self {
            SearchMethod::Contains => "contains",
            SearchMethod::Clangd => "clangd",
            SearchMethod::Skim => "skim",
        }
    }
}

impl SearchView {
    pub fn new(code: &Bytecode) -> Self {
        Self {
            searcher: (SearchMethod::Contains, SearchMethod::Contains.searcher()),
            query_text: String::new(),
            matches: Vec::new(),
        }
    }
}

impl AppView for SearchView {
    fn title(&self, _ctx: AppCtxHandle) -> WidgetText {
        RichText::new("Search").color(Color32::WHITE).into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        ui.horizontal(|ui| {
            let old = self.searcher.0;
            ComboBox::from_label("Search")
                .selected_text(self.searcher.0.name())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.searcher.0,
                        SearchMethod::Contains,
                        SearchMethod::Contains.name(),
                    )
                    .on_hover_text("'contains' is fast but case sensitive and exact matching");
                    ui.selectable_value(
                        &mut self.searcher.0,
                        SearchMethod::Clangd,
                        SearchMethod::Clangd.name(),
                    );
                    ui.selectable_value(
                        &mut self.searcher.0,
                        SearchMethod::Skim,
                        SearchMethod::Skim.name(),
                    );
                });
            if old != self.searcher.0 {
                self.searcher.1 = self.searcher.0.searcher();
            }
            if old != self.searcher.0 || ui.text_edit_singleline(&mut self.query_text).changed() {
                // let start = Instant::now();
                self.matches = self.searcher.1.search(ctx.code(), &self.query_text, 30);
                // println!("{} ms", start.elapsed().as_millis());
            }
        });

        for f in &self.matches {
            //dbg!(ctx.code().resolve(*f));
            ui.label(singleline_simple(
                ui,
                f.display_header::<EnhancedFmt>(ctx.code()).to_string(),
            ));
        }
    }
}
