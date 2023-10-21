use std::ops::Deref;

use eframe::egui::{
    Color32, Grid, Link, RichText, ScrollArea, TextEdit, TextStyle, Ui, WidgetText,
};

use hlbc::fmt::EnhancedFmt;
use hlbc::types::{FunPtr, RefField, RefFun, RefGlobal, RefString, RefType};
use hlbc::{Bytecode, Resolve};

use crate::{AppCtxHandle, AppView, ItemSelection};

/// View detailed information about a bytecode element.
pub(crate) struct SyncInspectorView {
    name: RichText,
}

impl Default for SyncInspectorView {
    fn default() -> Self {
        Self {
            name: RichText::new("Inspector (sync)").color(Color32::WHITE),
        }
    }
}

impl AppView for SyncInspectorView {
    fn title(&self) -> WidgetText {
        self.name.clone().into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        let selected = ctx.selected();
        self.name = RichText::new(format!(
            "Inspector (sync) : {}",
            selected.name(ctx.code().deref())
        ))
        .color(Color32::WHITE);
        inspector_ui(ui, ctx, selected)
    }
}

pub(crate) struct InspectorView {
    item: ItemSelection,
    name: RichText,
}

impl InspectorView {
    pub(crate) fn new(item: ItemSelection, code: &Bytecode) -> Self {
        Self {
            item,
            name: RichText::new(item.name(code)).color(Color32::WHITE),
        }
    }
}

impl AppView for InspectorView {
    fn title(&self) -> WidgetText {
        self.name.clone().into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        inspector_ui(ui, ctx, self.item);
    }
}

fn inspector_ui(ui: &mut Ui, ctx: AppCtxHandle, item: ItemSelection) {
    ScrollArea::vertical()
        .id_source("functions_scroll_area")
        .auto_shrink([false, false])
        .show(ui, |ui| match item {
            ItemSelection::Fun(fun) => {
                function_inspector(ui, ctx, fun);
            }
            ItemSelection::Class(t) => {
                class_inspector(ui, ctx, t);
            }
            ItemSelection::Global(g) => {
                global_inspector(ui, ctx, g);
            }
            ItemSelection::String(s) => {
                string_inspector(ui, ctx, s);
            }
            _ => {
                ui.label("Select a function or a class.");
            }
        });
}

fn inspector_link(ui: &mut Ui, ctx: AppCtxHandle, item: ItemSelection) {
    let res = ui
        .add(Link::new(item.name(ctx.code().deref())))
        .context_menu(|ui| {
            if ui.button("Open in inspector").clicked() {
                ctx.open_tab(InspectorView::new(item, ctx.code().deref()));
                ui.close_menu();
            }
        });
    if res.clicked() {
        ctx.set_selected(item);
    }
}

fn function_inspector(ui: &mut Ui, ctx: AppCtxHandle, fun: RefFun) {
    let code = ctx.code();
    match code.get(fun) {
        FunPtr::Fun(f) => {
            ui.heading(format!("Function : {}@{}", f.name(code), f.findex.0));

            if fun == code.entrypoint {
                ui.label("Compiler generated entrypoint function");
            } else if let Some(parent) = f.parent {
                ui.horizontal(|ui| {
                    ui.label("static/instance method of");
                    inspector_link(ui, ctx.clone(), ItemSelection::Class(parent));
                });
            } else {
                ui.label("Probably a closure.");
            }

            ui.separator();
            ui.collapsing("Registers", |ui| {
                Grid::new("inspector::function::registers")
                    .striped(true)
                    .num_columns(2)
                    .show(ui, |ui| {
                        for (i, reg) in f.regs.iter().enumerate() {
                            ui.label(format!("reg{i}"));
                            ui.label(reg.display::<EnhancedFmt>(code).to_string());
                            ui.end_row();
                        }
                    });
            });

            ui.add_space(6.0);
            ScrollArea::vertical()
                .id_source("inspector::function::instructions")
                .auto_shrink([false, false])
                .show_rows(
                    ui,
                    ui.text_style_height(&TextStyle::Monospace),
                    f.ops.len(),
                    |ui, range| {
                        for (i, o) in f
                            .ops
                            .iter()
                            .enumerate()
                            .skip(range.start)
                            .take(range.end - range.start)
                        {
                            // TODO syntax highlighting here
                            ui.monospace(format!("{i:>3}: {}", o.display(code, f, i as i32, 11)));
                        }
                    },
                );
        }
        FunPtr::Native(n) => {
            ui.heading("Native function");
            ui.label(format!("native library : {}", n.lib(code)));
            ui.label(format!("function name : {}", n.name(code)));
            ui.label(format!("function index : {}", n.findex.0))
                .on_hover_text("This is the native function unique index in the function pool.");
        }
    }
}

fn class_inspector(ui: &mut Ui, ctx: AppCtxHandle, t: RefType) {
    let code = ctx.code();
    ui.heading(format!("Class : {}", t.display::<EnhancedFmt>(code)));
    if let Some(obj) = t.as_obj(code) {
        if let Some(super_) = obj.super_ {
            ui.horizontal(|ui| {
                ui.label("extends");
                inspector_link(ui, ctx.clone(), ItemSelection::Class(super_));
            });
        }
        if obj.global.0 >= 1 {
            ui.horizontal(|ui| {
                ui.label("initialized by global");
                inspector_link(
                    ui,
                    ctx.clone(),
                    ItemSelection::Global(RefGlobal(obj.global.0 - 1)),
                );
            });
        }

        if obj.own_fields.is_empty() {
            ui.label("No fields");
        } else {
            ui.add_space(6.0);
            ui.collapsing("Fields", |ui| {
                Grid::new("inspector::class::fields")
                    .striped(true)
                    .num_columns(3)
                    .show(ui, |ui| {
                        for (i, f) in obj.own_fields.iter().enumerate() {
                            ui.label(&*f.name(code));
                            ui.label(f.t.display::<EnhancedFmt>(code).to_string());
                            if let Some(&binding) = obj
                                .bindings
                                .get(&RefField(i + obj.fields.len() - obj.own_fields.len()))
                            {
                                ui.monospace("bound to");
                                inspector_link(ui, ctx.clone(), ItemSelection::Fun(binding));
                            } else {
                                ui.monospace("variable");
                            }
                            ui.end_row();
                        }
                    });
            });
        }

        if obj.protos.is_empty() {
            ui.label("No methods");
        } else {
            ui.add_space(6.0);
            ui.collapsing("Methods", |ui| {
                Grid::new("inspector::class::methods")
                    .striped(true)
                    .num_columns(2)
                    .show(ui, |ui| {
                        for f in &obj.protos {
                            ui.label(&*f.name(code));
                            inspector_link(ui, ctx.clone(), ItemSelection::Fun(f.findex));
                            ui.end_row();
                        }
                    });
            });
        }
    } else {
        ui.label("Invalid type");
    }
}

fn global_inspector(ui: &mut Ui, ctx: AppCtxHandle, g: RefGlobal) {
    ui.heading(format!("Global@{}", g.0));
    ui.label(format!(
        "Type : {}",
        ctx.code().globals[g.0].display::<EnhancedFmt>(ctx.code().deref())
    ));

    if let (Some(&cst), Some(constants)) = (
        ctx.code().globals_initializers.get(&g),
        &ctx.code().constants,
    ) {
        let def = &constants[cst];
        ui.label(format!("{:?}", def.fields));
    } else {
        ui.label("This global is initialized with code");
    }
}

fn string_inspector(ui: &mut Ui, ctx: AppCtxHandle, s: RefString) {
    ui.heading(format!("String@{}", s.0));
    ui.separator();
    ui.add_space(4.0);
    TextEdit::multiline(&mut &*ctx.code()[s].to_string())
        .code_editor()
        .lock_focus(false)
        .show(ui);
}
