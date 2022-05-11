use std::fmt::{Debug, Display, Formatter, Result};

use crate::types::{RefInt, Reg};
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

impl RefType {
    pub fn display(&self, ctx: &Bytecode) -> impl Display {
        self.resolve(&ctx.types).display(ctx)
    }
}

impl Type {
    pub fn display(&self, ctx: &Bytecode) -> impl Display {
        format!("{self:?}")
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
            Opcode::Mov { dst, src } => op!("{dst} = {src}"),
            Opcode::Int { dst, ptr } => op!("{dst} = {}", ptr.display(ctx)),
            other => format!("{self:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::types::{RefInt, Reg};
    use crate::{Bytecode, Opcode};

    #[test]
    fn test() {
        let bc = Bytecode {
            version: 0,
            entrypoint: 0,
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
