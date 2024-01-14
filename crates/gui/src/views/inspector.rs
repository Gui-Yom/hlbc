use eframe::egui::{
    CollapsingHeader, Color32, Grid, Link, RichText, ScrollArea, TextEdit, TextStyle, Ui,
    WidgetText,
};

use hlbc::analysis::usage::{UsageString, UsageType};
use hlbc::fmt::EnhancedFmt;
use hlbc::types::{
    EnumConstruct, FunPtr, ObjField, RefField, RefFun, RefGlobal, RefString, RefType, Type, TypeObj,
};
use hlbc::{Bytecode, Resolve};

use crate::model::{AppCtxHandle, Item};
use crate::views::{impl_id, impl_view_id, text_stitch, ViewId};
use crate::{shortcuts, AppView};

/// View detailed information about a bytecode element.
#[derive(Default)]
pub(crate) struct SyncInspectorView;

impl_view_id!(SyncInspectorView: unique);

impl AppView for SyncInspectorView {
    impl_id!(unique);

    fn title(&self, ctx: AppCtxHandle) -> WidgetText {
        let selected = ctx.selected();
        RichText::new(format!("Inspector (sync) : {}", selected.name(ctx.code())))
            .color(Color32::WHITE)
            .into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        // Only triggers when in view
        if ui.input_mut(|i| i.consume_shortcut(&shortcuts::NAV_BACK)) {
            ctx.navigate_back();
        } else if ui.input_mut(|i| i.consume_shortcut(&shortcuts::NAV_FORWARD)) {
            ctx.navigate_forward();
        }

        let selected = ctx.selected();
        inspector_ui(ui, ctx, selected)
    }

    fn closeable(&self) -> bool {
        false
    }
}

pub(crate) struct InspectorView {
    id: ViewId,
    item: Item,
    name: RichText,
}

impl_view_id!(InspectorView);

impl InspectorView {
    pub(crate) fn new(item: Item, code: &Bytecode) -> Self {
        Self {
            id: ViewId::new_instance::<Self>(),
            item,
            name: RichText::new(item.name(code)).color(Color32::WHITE),
        }
    }
}

impl AppView for InspectorView {
    impl_id!();

    fn title(&self, _ctx: AppCtxHandle) -> WidgetText {
        self.name.clone().into()
    }

    fn ui(&mut self, ui: &mut Ui, ctx: AppCtxHandle) {
        inspector_ui(ui, ctx, self.item);
    }
}

fn inspector_ui(ui: &mut Ui, ctx: AppCtxHandle, item: Item) {
    ScrollArea::vertical()
        .id_source("inspector_scroll_area")
        .auto_shrink([false, false])
        .show(ui, |ui| match item {
            Item::Fun(fun) => {
                function_inspector(ui, ctx, fun);
            }
            Item::Type(t) => {
                type_inspector(ui, ctx, t);
            }
            Item::Global(g) => {
                global_inspector(ui, ctx, g);
            }
            Item::String(s) => {
                string_inspector(ui, ctx, s);
            }
            _ => {
                ui.label("Select a function or a class.");
            }
        });
}

fn inspector_link(ui: &mut Ui, ctx: AppCtxHandle, item: Item) {
    let res = ui.add(Link::new(item.name(ctx.code()))).context_menu(|ui| {
        if ui.button("Open in inspector").clicked() {
            ctx.open_tab(InspectorView::new(item, ctx.code()));
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
                    inspector_link(ui, ctx.clone(), Item::Type(parent));
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
                        for (i, regty) in f.regs.iter().enumerate() {
                            ui.label(format!("reg{i}"));
                            inspector_link(ui, ctx.clone(), Item::Type(*regty));
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
                            text_stitch(ui, |ui| {
                                ui.label(
                                    RichText::new(format!("{i:>3}"))
                                        .color(Color32::GRAY)
                                        .monospace(),
                                );
                                ui.add_space(10.0);
                                ui.monospace(o.display(code, f, i as i32, 11).to_string())
                                    .on_hover_text(o.description());
                            });
                            // TODO syntax highlighting
                            // TODO linking (requires bytecode visitor)
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

fn type_inspector(ui: &mut Ui, ctx: AppCtxHandle, t: RefType) {
    if t.is_known() {
        ui.heading(t.display::<EnhancedFmt>(ctx.code()).to_string());
        ui.separator();
        ui.label("This is a base Hashlink type");
    } else if ctx.code()[t].is_wrapper_type() {
        wrapper_type_inspector(ui, ctx, t);
    } else {
        match &ctx.code()[t] {
            Type::Fun(_) => {}
            Type::Obj(obj) => {
                obj_inspector(ui, ctx.clone(), t, obj);
            }
            Type::Ref(inner) => {
                ui.heading(t.display::<EnhancedFmt>(ctx.code()).to_string());
                ui.separator();
                ui.label(format!(
                    "Ref wrapper type for {}",
                    inner.display::<EnhancedFmt>(ctx.code())
                ));
                ui.separator();
                type_usage_report(ui, ctx, t);
            }
            Type::Virtual { fields } => {
                virtual_inspector(ui, ctx.clone(), t, fields);
            }
            Type::Abstract { .. } => {}
            Type::Enum { .. } => {
                enum_inspector(ui, ctx.clone(), t);
            }
            Type::Null(inner) => {
                ui.heading(t.display::<EnhancedFmt>(ctx.code()).to_string());
                ui.label(format!(
                    "Null wrapper type for {}",
                    inner.display::<EnhancedFmt>(ctx.code())
                ));
                ui.separator();
                type_usage_report(ui, ctx, t);
            }
            Type::Method(_) => {}
            Type::Struct(obj) => {
                obj_inspector(ui, ctx.clone(), t, obj);
            }
            Type::Packed(inner) => {
                ui.heading(t.display::<EnhancedFmt>(ctx.code()).to_string());
                ui.separator();
                ui.label(format!(
                    "Packed wrapper type for {}",
                    inner.display::<EnhancedFmt>(ctx.code())
                ));
                ui.separator();
                type_usage_report(ui, ctx, t);
            }
            other => {
                ui.label("Type is unsupported in inspector");
                ui.separator();
                ui.monospace(format!("{:#?}", other));
                type_usage_report(ui, ctx, t);
            }
        }
    }
}

fn type_usage_report(ui: &mut Ui, ctx: AppCtxHandle, t: RefType) {
    let usages = &ctx.usage()[t];
    if usages.is_empty() {
        ui.label("Unused");
    } else {
        CollapsingHeader::new("Usage report")
            .id_source("inspector::type::usage")
            .default_open(true)
            .show(ui, |ui| {
                for usage in usages {
                    text_stitch(ui, |ui| match usage {
                        &UsageType::Argument(t) => {
                            ui.label("Argument in function type");
                            inspector_link(ui, ctx.clone(), Item::Type(t));
                        }
                        &UsageType::Return(t) => {
                            ui.label("Return type in function type");
                            inspector_link(ui, ctx.clone(), Item::Type(t));
                        }
                        &UsageType::Field(obj, _) => {
                            ui.label("Type of class field");
                            inspector_link(ui, ctx.clone(), Item::Type(obj));
                        }
                        &UsageType::EnumVariant(enum_, _, _) => {
                            ui.label("Enum variant field");
                            inspector_link(ui, ctx.clone(), Item::Type(enum_));
                        }
                    });
                }
            });
    }
}

fn wrapper_type_inspector(ui: &mut Ui, ctx: AppCtxHandle, t: RefType) {
    ui.heading(t.display::<EnhancedFmt>(ctx.code()).to_string());
    ui.separator();
    ui.label(format!(
        "Wrapper type for {}",
        ctx.code()[t]
            .get_inner()
            .unwrap()
            .display::<EnhancedFmt>(ctx.code())
    ));
    ui.separator();
    type_usage_report(ui, ctx, t);
}

fn obj_inspector(ui: &mut Ui, ctx: AppCtxHandle, t: RefType, obj: &TypeObj) {
    let code = ctx.code();
    ui.heading(format!("Class : {}", t.display::<EnhancedFmt>(code)));
    if let Some(super_) = obj.super_ {
        ui.horizontal(|ui| {
            ui.label("extends");
            inspector_link(ui, ctx.clone(), Item::Type(super_));
        });
    }
    if obj.global.0 >= 1 {
        ui.horizontal(|ui| {
            ui.label("initialized by global");
            inspector_link(ui, ctx.clone(), Item::Global(RefGlobal(obj.global.0 - 1)));
        });
    }

    if obj.own_fields.is_empty() {
        ui.label("No fields");
    } else {
        ui.add_space(6.0);
        CollapsingHeader::new("Fields")
            .id_source("inspector::class::fields")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("inspector::class::fields::grid")
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
                                inspector_link(ui, ctx.clone(), Item::Fun(binding));
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
        CollapsingHeader::new("Methods")
            .id_source("inspector::class::methods")
            .default_open(true)
            .show(ui, |ui| {
                Grid::new("inspector::class::methods::grid")
                    .striped(true)
                    .num_columns(2)
                    .show(ui, |ui| {
                        for f in &obj.protos {
                            ui.label(&*f.name(code));
                            inspector_link(ui, ctx.clone(), Item::Fun(f.findex));
                            ui.end_row();
                        }
                    });
            });
    }

    ui.separator();
    type_usage_report(ui, ctx, t);
}

fn enum_inspector(ui: &mut Ui, ctx: AppCtxHandle, t: RefType) {
    let Type::Enum {
        constructs, global, ..
    } = &ctx.code()[t]
    else {
        unreachable!()
    };

    ui.heading(t.display::<EnhancedFmt>(ctx.code()).to_string());
    text_stitch(ui, |ui| {
        ui.label("Initialized by");
        inspector_link(ui, ctx.clone(), Item::Global(*global));
    });

    ui.separator();
    Grid::new("inspector::enum::fields")
        .striped(true)
        .num_columns(2)
        .show(ui, |ui| {
            for EnumConstruct { name, params } in constructs {
                ui.label(ctx.code()[*name].as_ref());
                for &p in params {
                    inspector_link(ui, ctx.clone(), Item::Type(p));
                }
                ui.end_row();
            }
        });

    ui.separator();
    type_usage_report(ui, ctx, t);
}

fn virtual_inspector(ui: &mut Ui, ctx: AppCtxHandle, t: RefType, fields: &[ObjField]) {
    ui.heading(t.display::<EnhancedFmt>(ctx.code()).to_string());
    ui.separator();
    let code = ctx.code();
    Grid::new("inspector::virtual::fields")
        .striped(true)
        .num_columns(2)
        .show(ui, |ui| {
            for f in fields {
                ui.label(&*f.name(code));
                inspector_link(ui, ctx.clone(), Item::Type(f.t));
                ui.end_row();
            }
        });

    ui.separator();
    type_usage_report(ui, ctx, t);
}

fn global_inspector(ui: &mut Ui, ctx: AppCtxHandle, g: RefGlobal) {
    ui.heading(format!("Global@{}", g.0));
    ui.label(format!(
        "Type : {}",
        ctx.code().globals[g.0].display::<EnhancedFmt>(ctx.code())
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
    ui.heading(format!("string@{}", s.0));
    CollapsingHeader::new("Usage report")
        .id_source("inspector::string::usage")
        .default_open(true)
        .show(ui, |ui| {
            for usage in &ctx.usage()[s] {
                text_stitch(ui, |ui| match usage {
                    &UsageString::Type(ty) => {
                        ui.label("Name of type");
                        inspector_link(ui, ctx.clone(), Item::Type(ty));
                    }
                    &UsageString::EnumVariant(ty, _) => {
                        ui.label("Name of enum variant");
                        inspector_link(ui, ctx.clone(), Item::Type(ty));
                    }
                    &UsageString::Field(ty, _) => {
                        ui.label("Field name of type");
                        inspector_link(ui, ctx.clone(), Item::Type(ty));
                    }
                    &UsageString::Proto(ty, _) => {
                        ui.label("Method name of type");
                        inspector_link(ui, ctx.clone(), Item::Type(ty));
                    }
                    &UsageString::Code(f, _) => {
                        ui.label("Code constant in");
                        inspector_link(ui, ctx.clone(), Item::Fun(f));
                    }
                    &UsageString::Dyn(f, _) => {
                        ui.label("Dynamic access key in");
                        inspector_link(ui, ctx.clone(), Item::Fun(f));
                    }
                });
            }
        });
    ui.separator();
    ui.add_space(4.0);
    TextEdit::multiline(&mut &*ctx.code()[s].to_string())
        .code_editor()
        .lock_focus(false)
        .show(ui);
}
