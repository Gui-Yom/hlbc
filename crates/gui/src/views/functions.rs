use eframe::egui::{Color32, RichText, Ui, WidgetText};

use hlbc::fmt::EnhancedFmt;
use hlbc::types::RefFun;

use crate::model::{AppCtxHandle, Item};
use crate::style::list_view;
use crate::views::{impl_id, impl_view_id, DecompilerView, InspectorView};
use crate::AppView;

#[derive(Default)]
pub(crate) struct FunctionsView {
    show_natives: bool,
    show_std: bool,
    cache: Vec<RefFun>,
    cache_valid: bool,
}

impl_view_id!(FunctionsView: unique);

impl AppView for FunctionsView {
    impl_id!(unique);

    fn title(&self, _ctx: AppCtxHandle) -> WidgetText {
        RichText::new("ƒ Functions").color(Color32::WHITE).into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        // Function list cache
        if !self.cache_valid {
            self.cache = Vec::new();
            let code = ctx.code();
            for f in &code.functions {
                if self.show_std || !f.is_from_std(code) {
                    self.cache.push(f.findex);
                }
            }
            if self.show_natives {
                for n in &code.natives {
                    if self.show_std || !n.is_from_std(code) {
                        self.cache.push(n.findex);
                    }
                }
            }
            self.cache_valid = true;
        }

        ui.horizontal_wrapped(|ui| {
            if ui
                .checkbox(&mut self.show_natives, "Show natives")
                .changed()
            {
                self.cache_valid = false;
            }
            if ui.checkbox(&mut self.show_std, "Show stdlib").changed() {
                self.cache_valid = false;
            }
        });

        ui.add_space(6.0);

        list_view(
            ui,
            ctx,
            self.cache.len(),
            |i| self.cache[i],
            Item::Fun,
            |ctx, f| f.display_header::<EnhancedFmt>(ctx.code()).to_string(),
            Some(|ui: &mut Ui, ctx: &AppCtxHandle, f| {
                if ui.small_button("Open in inspector").clicked() {
                    let tab = InspectorView::new(Item::Fun(f), ctx.code());
                    ctx.open_tab(tab);
                }
                if ui.small_button("Decompile").clicked() {
                    ctx.open_tab(DecompilerView::default());
                }
            }),
        );
    }
}
