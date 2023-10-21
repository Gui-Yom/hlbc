use std::ops::Deref;

use eframe::egui::{Color32, RichText, Ui, WidgetText};

use hlbc::fmt::EnhancedFmt;
use hlbc::types::{RefType, Type};

use crate::views::{list_view, DecompilerView, InspectorView};
use crate::{AppCtxHandle, AppView, ItemSelection};

#[derive(Default)]
pub(crate) struct ClassesView {
    show_std: bool,
    cache: Vec<RefType>,
    cache_valid: bool,
}

impl AppView for ClassesView {
    fn title(&self, _ctx: AppCtxHandle) -> WidgetText {
        RichText::new("Classes").color(Color32::WHITE).into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        if !self.cache_valid {
            let code = ctx.code();
            let code = code.deref();

            self.cache = Vec::new();
            for (i, t) in code.types.iter().enumerate() {
                match t {
                    Type::Obj(obj) => {
                        if self.show_std || !obj.is_from_std(code) {
                            self.cache.push(RefType(i));
                        }
                    }
                    _ => {}
                }
            }

            self.cache_valid = true;
        }

        if ui.checkbox(&mut self.show_std, "Show stdlib").changed() {
            self.cache_valid = false;
        }

        ui.add_space(6.0);

        list_view(
            ui,
            ctx,
            self.cache.len(),
            |i| self.cache[i],
            ItemSelection::Class,
            |ctx, t| t.display::<EnhancedFmt>(ctx.code()).to_string(),
            Some(|ui: &mut Ui, ctx: &AppCtxHandle, t| {
                if ui.small_button("Open in inspector").clicked() {
                    let tab = InspectorView::new(ItemSelection::Class(t), ctx.code().deref());
                    ctx.open_tab(tab);
                }
                if ui.small_button("Decompile").clicked() {
                    ctx.open_tab(DecompilerView::default());
                }
            }),
        );
    }
}
