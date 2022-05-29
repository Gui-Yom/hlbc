use std::fmt::{Display, Formatter, Result};

use crate::opcodes::Opcode;
use crate::types::{
    Function, Native, RefEnumConstruct, RefField, RefFloat, RefFunPointee, RefInt, RefString,
    RefType, Reg, Type, TypeFun, TypeObj,
};
use crate::{Bytecode, RefFun};

/*
pub trait CodeDisplay {
    fn display<T: fmt::Display>(&self, ctx: &HlCode) -> T;
}*/

impl Display for Reg {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "reg{}", self.0)
    }
}

impl RefInt {
    pub fn display(&self, ctx: &Bytecode) -> impl Display {
        self.resolve(&ctx.ints)
    }
}

impl RefFloat {
    pub fn display(&self, ctx: &Bytecode) -> impl Display {
        self.resolve(&ctx.floats)
    }
}

impl RefString {
    pub fn display(&self, ctx: &Bytecode) -> String {
        self.resolve(&ctx.strings).to_string()
    }
}

impl RefType {
    pub fn display(&self, ctx: &Bytecode) -> String {
        format!("{}@{}", self.resolve(&ctx.types).display(ctx), self.0)
    }

    fn display_rec(&self, ctx: &Bytecode, parents: Vec<*const Type>) -> String {
        self.resolve(&ctx.types).display_rec(ctx, parents)
    }
}

impl RefField {
    pub fn display_obj(&self, parent: &Type, ctx: &Bytecode) -> impl Display {
        if let Some(obj) = parent.get_type_obj() {
            if self.0 < obj.fields.len() {
                obj.fields[self.0].name.display(ctx)
            } else {
                format!("field{}", self.0)
            }
        } else if let Type::Virtual { fields } = parent {
            fields[self.0].name.display(ctx)
        } else {
            format!("field{}", self.0)
        }
    }
}

impl RefEnumConstruct {
    pub fn display(&self, parent: RefType, ctx: &Bytecode) -> impl Display {
        match parent.resolve(&ctx.types) {
            Type::Enum { constructs, .. } => {
                let name = &constructs[self.0].name;
                if name.0 != 0 {
                    name.display(ctx)
                } else {
                    "_".to_string()
                }
            }
            _ => "_".to_string(),
        }
    }
}

impl Type {
    pub fn display(&self, ctx: &Bytecode) -> String {
        self.display_rec(ctx, Vec::new())
    }

    fn display_rec(&self, ctx: &Bytecode, mut parents: Vec<*const Type>) -> String {
        //println!("{:#?}", self);
        if parents.contains(&(self as *const Type)) {
            return "Self".to_string();
        }
        parents.push(self as *const Type);

        fn display_type_fun(ty: &TypeFun, ctx: &Bytecode, parents: &Vec<*const Type>) -> String {
            let args: Vec<String> = ty
                .args
                .iter()
                .map(|a| a.display_rec(ctx, parents.clone()))
                .collect();
            format!(
                "({}) -> ({})",
                args.join(", "),
                ty.ret.display_rec(ctx, parents.clone())
            )
        }

        match self {
            Type::Void => "void".to_string(),
            Type::UI8 => "i8".to_string(),
            Type::UI16 => "i16".to_string(),
            Type::I32 => "i32".to_string(),
            Type::I64 => "i64".to_string(),
            Type::F32 => "f32".to_string(),
            Type::F64 => "f64".to_string(),
            Type::Bool => "bool".to_string(),
            Type::Bytes => "bytes".to_string(),
            Type::Dyn => "dynamic".to_string(),
            Type::Fun(fun) => display_type_fun(fun, ctx, &parents),
            Type::Obj(TypeObj { name, .. }) => name.display(ctx),
            Type::Array => "array".to_string(),
            Type::Type => "type".to_string(),
            Type::Ref(reftype) => {
                format!("ref<{}>", reftype.display_rec(ctx, parents.clone()))
            }
            Type::Virtual { fields } => {
                let fields: Vec<String> = fields
                    .iter()
                    .map(|a| {
                        format!(
                            "{}: {}",
                            a.name.display(ctx),
                            a.t.display_rec(ctx, parents.clone())
                        )
                    })
                    .collect();
                format!("virtual<{}>", fields.join(", "))
            }
            Type::DynObj => "dynobj".to_string(),
            Type::Abstract { name } => name.display(ctx),
            Type::Enum { name, .. } => format!(
                "enum<{}>",
                if name.0 != 0 {
                    name.display(ctx)
                } else {
                    "_".to_string()
                }
            ),
            Type::Null(reftype) => {
                format!("null<{}>", reftype.display_rec(ctx, parents.clone()))
            }
            Type::Method(fun) => display_type_fun(fun, ctx, &parents),
            Type::Struct(TypeObj { name, fields, .. }) => {
                let fields: Vec<String> = fields
                    .iter()
                    .map(|a| {
                        format!(
                            "{}: {}",
                            a.name.display(ctx),
                            a.t.display_rec(ctx, parents.clone())
                        )
                    })
                    .collect();
                format!("{}<{}>", name.display(ctx), fields.join(", "))
            }
        }
    }
}

impl RefFun {
    pub fn display_header(&self, ctx: &Bytecode) -> String {
        self.resolve(ctx).unwrap().display_header(ctx)
    }

    pub fn display_call(&self, ctx: &Bytecode) -> impl Display {
        self.resolve(ctx).unwrap().display_call(ctx)
    }
}

impl<'a> RefFunPointee<'a> {
    pub fn display_header(&'a self, ctx: &Bytecode) -> String {
        match self {
            RefFunPointee::Fun(fun) => fun.display_header(ctx),
            RefFunPointee::Native(n) => n.display_header(ctx),
        }
    }

    pub fn display_call(&'a self, ctx: &Bytecode) -> impl Display {
        match self {
            RefFunPointee::Fun(fun) => fun.display_call(ctx),
            RefFunPointee::Native(n) => n.display_call(ctx),
        }
    }
}

impl Native {
    pub fn display_header(&self, ctx: &Bytecode) -> String {
        format!(
            "fn:native {} {}",
            self.display_call(ctx),
            self.t.display(ctx)
        )
    }

    pub fn display_call(&self, ctx: &Bytecode) -> String {
        format!(
            "{}/{}@{}",
            self.lib.resolve(&ctx.strings),
            self.name.resolve(&ctx.strings),
            self.findex.0
        )
    }
}

impl Opcode {
    pub fn display(&self, ctx: &Bytecode, parent: &Function, pos: i32) -> impl Display {
        macro_rules! op {
            ($($arg:tt)*) => {
                format!("{:<16} {}", self.name(), format_args!($($arg)*))
            };
        }

        match self {
            Opcode::Mov { dst, src } => op!("{} = {src}", dst),
            Opcode::Int { dst, ptr } => op!("{dst} = {}", ptr.display(ctx)),
            Opcode::Float { dst, ptr } => op!("{dst} = {}", ptr.display(ctx)),
            Opcode::Bool { dst, value } => op!("{dst} = {}", value.0),
            Opcode::String { dst, ptr } => op!("{dst} = \"{}\"", ptr.display(ctx)),
            Opcode::Null { dst } => op!("{dst} = null"),
            Opcode::Add { dst, a, b } => op!("{dst} = {a} + {b}"),
            Opcode::Sub { dst, a, b } => op!("{dst} = {a} - {b}"),
            Opcode::Mul { dst, a, b } => op!("{dst} = {a} * {b}"),
            Opcode::SDiv { dst, a, b } => op!("{dst} = {a} / {b}"),
            Opcode::UDiv { dst, a, b } => op!("{dst} = {a} / {b}"),
            Opcode::SMod { dst, a, b } => op!("{dst} = {a} % {b}"),
            Opcode::UMod { dst, a, b } => op!("{dst} = {a} % {b}"),
            Opcode::Shl { dst, a, b } => op!("{dst} = {a} << {b}"),
            Opcode::SShr { dst, a, b } => op!("{dst} = {a} >> {b}"),
            Opcode::UShr { dst, a, b } => op!("{dst} = {a} >> {b}"),
            Opcode::And { dst, a, b } => op!("{dst} = {a} & {b}"),
            Opcode::Or { dst, a, b } => op!("{dst} = {a} | {b}"),
            Opcode::Xor { dst, a, b } => op!("{dst} = {a} ^ {b}"),
            Opcode::Neg { dst, src } => op!("{dst} = -{src}"),
            Opcode::Not { dst, src } => op!("{dst} = !{src}"),
            Opcode::Incr { dst } => op!("{dst}++"),
            Opcode::Decr { dst } => op!("{dst}--"),
            Opcode::Call0 { dst, fun } => op!("{dst} = {}()", fun.display_call(ctx)),
            Opcode::Call1 { dst, fun, arg0 } => op!("{dst} = {}({arg0})", fun.display_call(ctx)),
            Opcode::Call2 {
                dst,
                fun,
                arg0,
                arg1,
            } => op!("{dst} = {}({arg0}, {arg1})", fun.display_call(ctx)),
            Opcode::Call3 {
                dst,
                fun,
                arg0,
                arg1,
                arg2,
            } => op!("{dst} = {}({arg0}, {arg1}, {arg2})", fun.display_call(ctx)),
            Opcode::Call4 {
                dst,
                fun,
                arg0,
                arg1,
                arg2,
                arg3,
            } => op!(
                "{dst} = {}({arg0}, {arg1},{arg2}, {arg3})",
                fun.display_call(ctx)
            ),
            Opcode::CallN { dst, fun, args } => {
                let args: Vec<String> = args.iter().map(|r| format!("{}", r)).collect();
                op!("{dst} = {}({})", fun.display_call(ctx), args.join(", "))
            }
            Opcode::CallMethod { dst, field, args } => {
                let mut args = args.iter();
                let arg0 = args.next().unwrap();
                let args: Vec<String> = args.map(|r| format!("{}", r)).collect();
                op!(
                    "{dst} = {}.{}({})",
                    arg0,
                    field.display_obj(parent.regs[arg0.0 as usize].resolve(&ctx.types), ctx),
                    args.join(", ")
                )
            }
            Opcode::CallThis { dst, field, args } => {
                let args: Vec<String> = args.iter().map(|r| format!("{}", r)).collect();
                op!(
                    "{dst} = reg0.{}({})",
                    field.display_obj(parent.regs[0].resolve(&ctx.types), ctx),
                    args.join(", ")
                )
            }
            Opcode::CallClosure { dst, fun, args } => {
                let args: Vec<String> = args.iter().map(|r| format!("{}", r)).collect();
                op!("{dst} = {fun}({})", args.join(", "))
            }
            Opcode::StaticClosure { dst, fun } => {
                op!("{dst} = {}", fun.display_header(ctx))
            }
            Opcode::InstanceClosure { dst, fun, obj } => {
                op!("{dst} = {obj}.{}", fun.display_header(ctx))
            }
            Opcode::GetGlobal { dst, global } => {
                op!("{dst} = global@{}", global.0)
            }
            Opcode::SetGlobal { global, src } => {
                op!("global@{} = {src}", global.0)
            }
            Opcode::Field { dst, obj, field } => {
                op!(
                    "{dst} = {obj}.{}",
                    field.display_obj(parent.regs[obj.0 as usize].resolve(&ctx.types), ctx)
                )
            }
            Opcode::SetField { obj, field, src } => {
                op!(
                    "{obj}.{} = {src}",
                    field.display_obj(parent.regs[obj.0 as usize].resolve(&ctx.types), ctx)
                )
            }
            Opcode::GetThis { dst, field } => {
                op!(
                    "{dst} = this.{}",
                    field.display_obj(parent.regs[0].resolve(&ctx.types), ctx)
                )
            }
            Opcode::SetThis { field, src } => {
                op!(
                    "this.{} = {src}",
                    field.display_obj(parent.regs[0].resolve(&ctx.types), ctx)
                )
            }
            Opcode::DynGet { dst, obj, field } => {
                op!("{dst} = {obj}[\"{}\"]", field.resolve(&ctx.strings))
            }
            Opcode::DynSet { obj, field, src } => {
                op!("{obj}[\"{}\"] = {src}", field.resolve(&ctx.strings))
            }
            Opcode::JTrue { cond, offset } => {
                op!("if {cond} == true jump to {}", pos + offset + 1)
            }
            Opcode::JFalse { cond, offset } => {
                op!("if {cond} == false jump to {}", pos + offset + 1)
            }
            Opcode::JNull { reg, offset } => {
                op!("if {reg} == null jump to {}", pos + offset + 1)
            }
            Opcode::JNotNull { reg, offset } => {
                op!("if {reg} != null jump to {}", pos + offset + 1)
            }
            Opcode::JSLt { a, b, offset } => {
                op!("if {a} < {b} jump to {}", pos + offset + 1)
            }
            Opcode::JSGte { a, b, offset } => {
                op!("if {a} >= {b} jump to {}", pos + offset + 1)
            }
            Opcode::JSGt { a, b, offset } => {
                op!("if {a} > {b} jump to {}", pos + offset + 1)
            }
            Opcode::JSLte { a, b, offset } => {
                op!("if {a} <= {b} jump to {}", pos + offset + 1)
            }
            Opcode::JULt { a, b, offset } => {
                op!("if {a} < {b} jump to {}", pos + offset + 1)
            }
            Opcode::JUGte { a, b, offset } => {
                op!("if {a} >= {b} jump to {}", pos + offset + 1)
            }
            Opcode::JNotLt { a, b, offset } => {
                op!("if {a} !< {b} jump to {}", pos + offset + 1)
            }
            Opcode::JNotGte { a, b, offset } => {
                op!("if {a} !>= {b} jump to {}", pos + offset + 1)
            }
            Opcode::JEq { a, b, offset } => {
                op!("if {a} == {b} jump to {}", pos + offset + 1)
            }
            Opcode::JNotEq { a, b, offset } => {
                op!("if {a} != {b} jump to {}", pos + offset + 1)
            }
            Opcode::JAlways { offset } => {
                op!("jump {}", pos + offset + 1)
            }
            Opcode::ToDyn { dst, src } => {
                op!("{dst} = cast {src}")
            }
            Opcode::ToInt { dst, src } => {
                op!("{dst} = cast {src}")
            }
            Opcode::SafeCast { dst, src } => {
                op!("{dst} = cast {src}")
            }
            Opcode::UnsafeCast { dst, src } => {
                op!("{dst} = cast {src}")
            }
            Opcode::ToVirtual { dst, src } => {
                op!("{dst} = cast {src}")
            }
            Opcode::Ret { ret } => op!("{ret}"),
            Opcode::Throw { exc } => {
                op!("throw {exc}")
            }
            Opcode::Rethrow { exc } => {
                op!("rethrow {exc}")
            }
            Opcode::NullCheck { reg } => {
                op!("if {reg} == null throw exc")
            }
            Opcode::Trap { exc, offset } => {
                op!("try {exc} jump to {}", pos + offset + 1)
            }
            Opcode::EndTrap { exc } => {
                op!("catch {exc}")
            }
            Opcode::GetArray { dst, array, index } => {
                op!("{dst} = {array}[{index}]")
            }
            Opcode::SetArray { array, index, src } => {
                op!("{array}[{index}] = {src}")
            }
            Opcode::New { dst } => {
                op!("{dst} = new {}", parent.regs[dst.0 as usize].display(ctx))
            }
            Opcode::ArraySize { dst, array } => {
                op!("{dst} = {array}.length")
            }
            Opcode::Type { dst, ty } => {
                op!("{dst} = {}", ty.display(ctx))
            }
            Opcode::Ref { dst, src } => {
                op!("{dst} = &{src}")
            }
            Opcode::Unref { dst, src } => {
                op!("{dst} = *{src}")
            }
            Opcode::MakeEnum {
                dst,
                construct,
                args,
            } => {
                let args: Vec<String> = args.iter().map(|r| format!("{}", r)).collect();
                op!(
                    "{dst} = variant {} ({})",
                    construct.display(parent.regs[dst.0 as usize], ctx),
                    args.join(", ")
                )
            }
            Opcode::EnumAlloc { dst, construct } => {
                op!(
                    "{dst} = new {}",
                    construct.display(parent.regs[dst.0 as usize], ctx)
                )
            }
            Opcode::EnumIndex { dst, value } => {
                op!("{dst} = variant of {value}")
            }
            Opcode::EnumField {
                dst,
                value,
                construct,
                field,
            } => {
                op!(
                    "{dst} = ({value} as {}).{}",
                    construct.display(parent.regs[dst.0 as usize], ctx),
                    field.0
                )
            }
            Opcode::SetEnumField { value, field, src } => {
                op!("{value}.{} = {src}", field.0)
            }
            _ => format!("{self:?}"),
        }
    }
}

impl Function {
    pub fn display_header(&self, ctx: &Bytecode) -> String {
        format!("fn {} {}", self.display_call(ctx), self.t.display(ctx))
    }

    pub fn display_call(&self, ctx: &Bytecode) -> String {
        format!(
            "{}@{}",
            self.name.map(|r| r.resolve(&ctx.strings)).unwrap_or("_"),
            self.findex.0
        )
    }

    pub fn display(&self, ctx: &Bytecode) -> impl Display {
        let regs: Vec<String> = self
            .regs
            .iter()
            .enumerate()
            .map(|(i, r)| format!("reg{:<2} {}", i, r.display(ctx)))
            .collect();
        let ops: Vec<String> = if let Some(debug) = &self.debug_info {
            self.ops
                .iter()
                .enumerate()
                .zip(debug.iter())
                .map(|((i, o), (file, line))| {
                    format!(
                        "{:>12}:{line:<3} {i:>3}: {}",
                        ctx.debug_files.as_ref().unwrap()[*file as usize],
                        o.display(ctx, self, i as i32)
                    )
                })
                .collect()
        } else {
            self.ops
                .iter()
                .enumerate()
                .map(|(i, o)| format!("{i:>3}: {}", o.display(ctx, self, i as i32)))
                .collect()
        };
        let assigns: Vec<String> = self
            .assigns
            .as_ref()
            .unwrap()
            .iter()
            .map(|(s, i)| format!("{} at opcode {i}", s.resolve(&ctx.strings)))
            .collect();
        format!(
            "{} ({} regs, {} ops)\n    {}\n\n{}\n{}",
            self.display_header(ctx),
            self.regs.len(),
            self.ops.len(),
            regs.join("\n    "),
            ops.join("\n"),
            assigns.join("\n")
        )
    }
}
