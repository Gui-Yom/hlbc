use eframe::egui::{Color32, RichText, Ui, WidgetText};

use hlbc::fmt::EnhancedFmt;
use hlbc::types::{RefType, Type};

use crate::model::{AppCtxHandle, Item};
use crate::views::{impl_id, impl_view_id, list_view, DecompilerView, InspectorView};
use crate::AppView;

#[derive(Default)]
pub(crate) struct ClassesView {
    show_std: bool,
    cache: Vec<RefType>,
    cache_valid: bool,
}

impl_view_id!(ClassesView: unique);

impl AppView for ClassesView {
    impl_id!(unique);

    fn title(&self, _ctx: AppCtxHandle) -> WidgetText {
        RichText::new("Classes").color(Color32::WHITE).into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        if !self.cache_valid {
            let code = ctx.code();

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
            Item::Type,
            |ctx, t| t.display::<EnhancedFmt>(ctx.code()).to_string(),
            Some(|ui: &mut Ui, ctx: &AppCtxHandle, t| {
                if ui.small_button("Open in inspector").clicked() {
                    let tab = InspectorView::new(Item::Type(t), ctx.code());
                    ctx.open_tab(tab);
                }
                if ui.small_button("Decompile").clicked() {
                    ctx.open_tab(DecompilerView::default());
                }
            }),
        );
    }
}
