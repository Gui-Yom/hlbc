use std::ops::Deref;

use eframe::egui::style::Margin;
use eframe::egui::{Color32, Frame, Grid, RichText, ScrollArea, TextStyle, Ui, WidgetText};

use hlbc::types::{FunPtr, RefField, RefFun, RefGlobal, RefString, RefType};
use hlbc::Bytecode;

use crate::{AppCtxHandle, AppTab, ItemSelection};

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

impl AppTab for SyncInspectorView {
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

impl AppTab for InspectorView {
    fn title(&self) -> WidgetText {
        self.name.clone().into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        inspector_ui(ui, ctx, self.item);
    }
}

fn inspector_ui(ui: &mut Ui, ctx: AppCtxHandle, item: ItemSelection) {
    Frame::none()
        .inner_margin(Margin::same(4.0))
        .show(ui, |ui| {
            ScrollArea::vertical()
                .id_source("functions_scroll_area")
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    let code = ctx.code();
                    let code = code.deref();
                    match item {
                        ItemSelection::Fun(fun) => {
                            function_inspector(ui, fun, code);
                        }
                        ItemSelection::Class(t) => {
                            class_inspector(ui, t, code);
                        }
                        ItemSelection::Global(g) => {
                            global_inspector(ui, g, code);
                        }
                        ItemSelection::String(s) => {
                            string_inspector(ui, s, code);
                        }
                        _ => {
                            ui.label("Select a function or a class.");
                        }
                    }
                });
        });
}

fn function_inspector(ui: &mut Ui, fun: RefFun, code: &Bytecode) {
    match fun.resolve(code) {
        FunPtr::Fun(f) => {
            ui.heading(format!(
                "Function : {}@{}",
                f.name(code).unwrap_or("_"),
                f.findex.0
            ));
            if let Some(parent) = f.parent {
                ui.label(format!(
                    "static/instance method of {}",
                    parent.display(code)
                ));
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
                            ui.label(reg.display(code));
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
            ui.label(format!("native library : {}", n.lib.resolve(&code.strings)));
            ui.label(format!("function name : {}", n.name.resolve(&code.strings)));
            ui.label(format!("function index : {}", n.findex.0))
                .on_hover_text("This is the native function unique index in the function pool.");
        }
    }
}

fn class_inspector(ui: &mut Ui, t: RefType, code: &Bytecode) {
    ui.heading(format!("Class : {}", t.display(code)));
    if let Some(obj) = t.resolve_as_obj(&code.types) {
        if let Some(super_) = obj.super_ {
            ui.label(format!("extends {}", super_.display(code)));
        }
        if obj.global.0 >= 1 {
            ui.label(format!("initialized by global {}", obj.global.0 - 1));
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
                            ui.label(f.name.resolve(&code.strings));
                            ui.label(f.t.display(code));
                            if let Some(binding) = obj
                                .bindings
                                .get(&RefField(i + obj.fields.len() - obj.own_fields.len()))
                            {
                                ui.monospace(format!("bound to {}", binding.display_call(code)));
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
                            ui.label(f.name.resolve(&code.strings));
                            ui.label(f.findex.display_call(code).to_string());
                            ui.end_row();
                        }
                    });
            });
        }
    } else {
        ui.label("Invalid type");
    }
}

fn global_inspector(ui: &mut Ui, g: RefGlobal, code: &Bytecode) {
    ui.heading(format!("Global@{}", g.0));
    ui.label(format!("Type : {}", code.globals[g.0].display(code)));

    if let (Some(&cst), Some(constants)) = (code.globals_initializers.get(&g), &code.constants) {
        let def = &constants[cst];
        ui.label(format!("{:?}", def.fields));
    } else {
        ui.label("This global is initialized with code");
    }
}

fn string_inspector(ui: &mut Ui, s: RefString, code: &Bytecode) {
    ui.heading(format!("String@{}", s.0));
    ui.label(RichText::new(s.resolve(&code.strings)).monospace());
}
