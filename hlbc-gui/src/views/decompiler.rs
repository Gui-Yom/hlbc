use hlbc_decompiler::decompile_function;
use hlbc_decompiler::fmt::FormatOptions;

use crate::egui::{Ui, WidgetText};
use crate::{AppCtx, AppTab};

#[derive(Default)]
pub(crate) struct DecompilerView;

impl AppTab for DecompilerView {
    fn title(&self) -> WidgetText {
        "Decompilation output".into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: &mut AppCtx) {
        if let Some(fun) = ctx.selected_fn {
            if let Some(func) = fun.resolve_as_fn(&ctx.code) {
                ui.code(
                    decompile_function(&ctx.code, func)
                        .display(&ctx.code, &FormatOptions::new("  "))
                        .to_string(),
                );
            } else {
                ui.code(fun.display_header(&ctx.code));
            }
        } else {
            ui.label("Select a function in the Functions view to view its bytecode");
        }
    }
}
