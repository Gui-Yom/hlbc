use std::fmt;
use std::fmt::{Display, Formatter, Write};

use crate::types::{Function, RefField, Type};
use crate::Bytecode;

use crate::decompiler::ast::{
    Call, Class, Constant, ConstructorCall, Expr, Method, Operation, Statement,
};

#[derive(Clone)]
pub struct FormatOptions {
    indent: String,
    inc_indent: String,
}

impl FormatOptions {
    pub fn new(inc_indent: &str) -> Self {
        Self {
            indent: String::new(),
            inc_indent: inc_indent.to_string(),
        }
    }

    pub fn with_base_indent(indent: &str, inc_indent: &str) -> Self {
        Self {
            indent: indent.to_string(),
            inc_indent: inc_indent.to_string(),
        }
    }

    pub fn inc_nesting(&self) -> Self {
        FormatOptions {
            indent: format!("{}{}", self.indent, self.inc_indent),
            inc_indent: self.inc_indent.clone(),
        }
    }
}

impl Display for FormatOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.indent)
    }
}

fn to_haxe_type(ty: &Type, ctx: &Bytecode) -> impl Display {
    match ty {
        Type::Void => "Void",
        Type::I32 => "Int",
        Type::F64 => "Float",
        Type::Bool => "Bool",
        _ => "other",
    }
}

impl Class {
    pub fn display<'a>(&'a self, ctx: &'a Bytecode, opts: &'a FormatOptions) -> impl Display + 'a {
        let new_opts = opts.inc_nesting();
        fmtools::fmt! { move
            {opts}"class "{self.name} if let Some(parent) = self.parent.as_ref() { " extends "{parent} } " {\n"
            for f in &self.fields {
                {new_opts} if f.static_ { "static " } "var "{f.name}": "{to_haxe_type(f.ty.resolve(&ctx.types), ctx)}";\n"
            }
            for m in &self.methods {
                "\n"
                {m.display(ctx, &new_opts)}
            }
            {opts}"}"
        }
    }
}

impl Method {
    pub fn display<'a>(&'a self, ctx: &'a Bytecode, opts: &'a FormatOptions) -> impl Display + 'a {
        let new_opts = opts.inc_nesting();
        let fun = self.fun.resolve_as_fn(ctx).unwrap();
        fmtools::fmt! { move
            {opts} if self.static_ { "static " } if self.dynamic { "dynamic " }
            "function "{fun.name(ctx).unwrap()}"("
            {fmtools::join(", ", fun.ty(ctx).args.iter().enumerate().skip(if self.static_ { 0 } else { 1 })
                .map(move |(i, arg)| fmtools::fmt! {move
                    {fun.arg_name(ctx, i).unwrap_or("_")}": "{to_haxe_type(fun.ty(ctx).ret.resolve(&ctx.types), ctx)}
                }))}
            ")" if !fun.ty(ctx).ret.is_void() { ": "{to_haxe_type(fun.ty(ctx).ret.resolve(&ctx.types), ctx)} } " {"

            if self.statements.is_empty() {
                "}"
            } else {
                "\n"
                for stmt in &self.statements {
                    |f| stmt.display(f, &new_opts, ctx, fun)?;
                }
                {opts}"}"
            }
            "\n"
        }
    }
}

impl Operation {
    pub fn display(&self, indent: &FormatOptions, code: &Bytecode) -> String {
        use Operation::*;
        match self {
            Add(e1, e2) => {
                format!(
                    "{} + {}",
                    e1.display(indent, code),
                    e2.display(indent, code)
                )
            }
            Sub(e1, e2) => {
                format!(
                    "{} - {}",
                    e1.display(indent, code),
                    e2.display(indent, code)
                )
            }
            Mul(e1, e2) => {
                format!(
                    "{} * {}",
                    e1.display(indent, code),
                    e2.display(indent, code)
                )
            }
            And(e1, e2) => {
                format!(
                    "{} && {}",
                    e1.display(indent, code),
                    e2.display(indent, code)
                )
            }
            Or(e1, e2) => {
                format!(
                    "{} || {}",
                    e1.display(indent, code),
                    e2.display(indent, code)
                )
            }
            Xor(e1, e2) => {
                format!(
                    "{} ^ {}",
                    e1.display(indent, code),
                    e2.display(indent, code)
                )
            }
            Neg(expr) => {
                format!("-{}", expr.display(indent, code))
            }
            Not(expr) => {
                format!("!{}", expr.display(indent, code))
            }
            Incr(expr) => {
                format!("{}++", expr.display(indent, code))
            }
            Decr(expr) => {
                format!("{}--", expr.display(indent, code))
            }
            Eq(e1, e2) => {
                format!(
                    "{} == {}",
                    e1.display(indent, code),
                    e2.display(indent, code)
                )
            }
            NotEq(e1, e2) => {
                format!(
                    "{} == {}",
                    e1.display(indent, code),
                    e2.display(indent, code)
                )
            }
            Gt(e1, e2) => {
                format!(
                    "{} > {}",
                    e1.display(indent, code),
                    e2.display(indent, code)
                )
            }
            Gte(e1, e2) => {
                format!(
                    "{} >= {}",
                    e1.display(indent, code),
                    e2.display(indent, code)
                )
            }
            Lt(e1, e2) => {
                format!(
                    "{} < {}",
                    e1.display(indent, code),
                    e2.display(indent, code)
                )
            }
            Lte(e1, e2) => {
                format!(
                    "{} <= {}",
                    e1.display(indent, code),
                    e2.display(indent, code)
                )
            }
        }
    }
}

impl Expr {
    pub fn display(&self, indent: &FormatOptions, code: &Bytecode) -> String {
        match self {
            Expr::Anonymous(ty, values) => match ty.resolve(&code.types) {
                Type::Virtual { fields } => {
                    format!(
                        "{{ {} }}",
                        fields
                            .iter()
                            .enumerate()
                            .map(|(i, f)| {
                                format!(
                                    "{}: {}",
                                    f.name.resolve(&code.strings),
                                    values.get(&RefField(i)).unwrap().display(indent, code)
                                )
                            })
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
                _ => "[invalid anonymous type]".to_owned(),
            },
            Expr::Call(call) => {
                format!(
                    "{}({})",
                    call.fun.display(indent, code),
                    call.args
                        .iter()
                        .map(|a| a.display(indent, code))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Expr::Constant(x) => match x {
                Constant::Int(c) => c.to_string(),
                Constant::Float(c) => c.to_string(),
                Constant::String(c) => format!("\"{c}\""),
                Constant::Bool(c) => c.to_string(),
                Constant::Null => "null".to_owned(),
                Constant::This => "this".to_owned(),
            },
            Expr::Constructor(ConstructorCall { ty, args }) => {
                format!(
                    "new {}({})",
                    ty.display(code),
                    args.iter()
                        .map(|a| a.display(indent, code))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Expr::Closure(f, stmts) => {
                let mut buf = "() -> {\n".to_owned();
                let indent = indent.inc_nesting();
                for s in stmts {
                    s.display(&mut buf, &indent, code, f.resolve_as_fn(code).unwrap())
                        .unwrap();
                }
                write!(buf, "}}").unwrap();
                buf
            }
            Expr::EnumConstr(ty, constr, args) => {
                format!(
                    "{}({})",
                    constr.display(*ty, code),
                    args.iter()
                        .map(|a| a.display(indent, code))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Expr::Field(receiver, name) => {
                format!("{}.{}", receiver.display(indent, code), name)
            }
            Expr::FunRef(fun) => fun.display_call(code).to_string(),
            Expr::Op(op) => op.display(indent, code),
            Expr::Unknown(msg) => {
                format!("[{msg}]")
            }
            Expr::Variable(x, name) => {
                if let Some(name) = name {
                    name.clone()
                } else {
                    format!("{x}")
                }
            }
        }
    }
}

impl Statement {
    pub fn display(
        &self,
        w: &mut impl Write,
        indent: &FormatOptions,
        code: &Bytecode,
        f: &Function,
    ) -> fmt::Result {
        write!(w, "{indent}")?;
        match self {
            Statement::Assign {
                declaration,
                variable,
                assign,
            } => {
                writeln!(
                    w,
                    "{}{} = {};",
                    if *declaration { "var " } else { "" },
                    variable.display(indent, code),
                    /*
                    if *declaration {
                        format!(": {}", f.regtype(*reg).display(code))
                    } else {
                        "".to_owned()
                    },*/
                    assign.display(indent, code)
                )?;
            }
            Statement::Call(Call { fun, args }) => {
                writeln!(
                    w,
                    "{}({});",
                    fun.display(indent, code),
                    args.iter()
                        .map(|a| a.display(indent, code))
                        .collect::<Vec<_>>()
                        .join(", ")
                )?;
            }
            Statement::Return(expr) => {
                writeln!(w, "return {};", expr.display(indent, code))?;
            }
            Statement::ReturnVoid => {
                writeln!(w, "return;")?;
            }
            Statement::If { cond, stmts } => {
                writeln!(w, "if ({}) {{", cond.display(indent, code))?;
                let indent2 = indent.inc_nesting();
                for stmt in stmts {
                    stmt.display(w, &indent2, code, f)?;
                }
                writeln!(w, "{indent}}}")?;
            }
            Statement::Else { stmts } => {
                writeln!(w, "else {{")?;
                let indent2 = indent.inc_nesting();
                for stmt in stmts {
                    stmt.display(w, &indent2, code, f)?;
                }
                writeln!(w, "{indent}}}")?;
            }
            Statement::While { cond, stmts } => {
                writeln!(w, "while ({}) {{", cond.display(indent, code))?;
                let indent2 = indent.inc_nesting();
                for stmt in stmts {
                    stmt.display(w, &indent2, code, f)?;
                }
                writeln!(w, "{indent}}}")?;
            }
            Statement::Throw(exc) => {
                write!(w, "throw {}", exc.display(indent, code))?;
            }
        }
        Ok(())
    }
}
