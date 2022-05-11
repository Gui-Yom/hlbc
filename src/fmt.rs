use std::fmt::{Debug, Display, Formatter, Result};

use crate::types::{ObjField, RefInt, RefString, Reg};
use crate::{Bytecode, Native, Opcode, RefType, Type};

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
        *self.resolve(&ctx.ints)
    }
}

impl RefString {
    pub fn display(&self, ctx: &Bytecode) -> String {
        self.resolve(&ctx.strings).to_string()
    }
}

impl RefType {
    pub fn display(&self, ctx: &Bytecode) -> String {
        self.resolve(&ctx.types).display(ctx)
    }

    fn display_rec(&self, ctx: &Bytecode, parents: &mut Vec<*const Type>) -> String {
        self.resolve(&ctx.types).display_rec(ctx, parents)
    }
}

impl Type {
    pub fn display(&self, ctx: &Bytecode) -> String {
        let mut parents = Vec::new();
        self.display_rec(ctx, &mut parents)
    }

    fn display_rec<'a>(&'a self, ctx: &Bytecode, parents: &'a mut Vec<*const Type>) -> String {
        //println!("{:#?}", self);
        if parents.contains(&(self as *const Type)) {
            return "<loop>".to_string();
        }
        parents.push(self as *const Type);
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
            Type::Fun { args, ret } => {
                let args: Vec<String> = args.iter().map(|a| a.display_rec(ctx, parents)).collect();
                format!(
                    "({}) -> ({})",
                    args.join(", "),
                    ret.display_rec(ctx, parents)
                )
            }
            Type::Obj { name, .. } => name.display(ctx),
            Type::Array => "array".to_string(),
            Type::Type => "type".to_string(),
            Type::Ref(reftype) => {
                format!("ref<{}>", reftype.display_rec(ctx, parents))
            }
            Type::Virtual { fields } => {
                let fields: Vec<String> = fields
                    .iter()
                    .map(|a| format!("{}: {}", a.name.display(ctx), a.t.display_rec(ctx, parents)))
                    .collect();
                format!("virtual<{}>", fields.join(", "))
            }
            Type::DynObj => "dynobj".to_string(),
            Type::Abstract { name } => name.display(ctx),
            Type::Enum { name, .. } => name.display(ctx),
            Type::Null(reftype) => {
                format!("null<{}>", reftype.display_rec(ctx, parents))
            }
            Type::Method { args, ret } => {
                let args: Vec<String> = args.iter().map(|a| a.display_rec(ctx, parents)).collect();
                format!(
                    "({}) -> ({})",
                    args.join(", "),
                    ret.display_rec(ctx, parents)
                )
            }
            Type::Struct { fields, .. } => {
                let fields: Vec<String> = fields
                    .iter()
                    .map(|a| format!("{}: {}", a.name.display(ctx), a.t.display_rec(ctx, parents)))
                    .collect();
                format!("virtual<{}>", fields.join(", "))
            }
        }
    }
}

impl Native {
    pub fn display(&self, ctx: &Bytecode) -> impl Display {
        format!(
            "fn:native {}/{}@{} {}",
            self.lib.resolve(&ctx.strings),
            self.name.resolve(&ctx.strings),
            self.findex.0,
            self.t.display(ctx)
        )
    }
}

impl Opcode {
    pub fn display(&self, ctx: &Bytecode) -> impl Display {
        let name: &'static str = self.into();

        macro_rules! op {
            ($($arg:tt)*) => {
                format!("{name:<16} {}", format_args!($($arg)*))
            };
        }

        match self {
            Opcode::Mov { dst, src } => op!("{} = {src}", dst),
            Opcode::Int { dst, ptr } => op!("{dst} = {}", ptr.display(ctx)),
            other => format!("{self:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{RefInt, Reg};
    use crate::{Bytecode, Opcode, RefFun};

    #[test]
    fn test() {
        let bc = Bytecode {
            version: 0,
            entrypoint: RefFun(0),
            ints: vec![69],
            floats: vec![],
            strings: vec![],
            debug_files: None,
            types: vec![],
            globals: vec![],
            natives: vec![],
            functions: vec![],
            constants: vec![],
        };
        println!(
            "{}",
            Opcode::Mov {
                dst: Reg(0),
                src: Reg(1),
            }
            .display(&bc)
        );
        println!(
            "{}",
            Opcode::Int {
                dst: Reg(0),
                ptr: RefInt(0),
            }
            .display(&bc)
        );
    }
}
