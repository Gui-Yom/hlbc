use eframe::egui::{Color32, RichText, ScrollArea, Ui, WidgetText};

use hlbc::fmt::EnhancedFmt;
use hlbc::types::FunPtr;
use hlbc::Resolve;
use hlbc_decompiler::fmt::FormatOptions;
use hlbc_decompiler::{decompile_class, decompile_function};

use crate::model::{AppCtxHandle, Item};
use crate::views::{haxe_source_view, impl_id, impl_view_id};
use crate::AppView;

#[derive(Default)]
pub(crate) struct DecompilerView {
    output: String,
    // Cache key for decompilation
    cache_selected: Item,
}

impl_view_id!(DecompilerView: unique);

impl AppView for DecompilerView {
    impl_id!(unique);

    fn title(&self, _ctx: AppCtxHandle) -> WidgetText {
        RichText::new("Decompilation output")
            .color(Color32::WHITE)
            .into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        if ctx.selected() != self.cache_selected {
            let code = ctx.code();

            self.output = match ctx.selected() {
                Item::Fun(fun) => match code.get(fun) {
                    FunPtr::Fun(func) => decompile_function(code, func)
                        .display(code, &FormatOptions::new(2))
                        .to_string(),
                    FunPtr::Native(n) => n.display::<EnhancedFmt>(code).to_string(),
                },
                Item::Class(t) => decompile_class(code, t.as_obj(code).unwrap())
                    .display(code, &FormatOptions::new(2))
                    .to_string(),
                _ => String::new(),
            };
            self.cache_selected = ctx.selected();
        }

        ScrollArea::both()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                // TextEdit will show us text we can edit (we don't want that)
                // We need to pass a mut reference to an immutable str
                haxe_source_view(ui, &self.output);
            });
    }
}
