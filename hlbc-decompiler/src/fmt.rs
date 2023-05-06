use std::fmt;
use std::fmt::{Display, Formatter};

use crate::ast::{Class, Constant, ConstructorCall, Expr, Method, Operation, Statement};
use hlbc::fmt::EnhancedFmt;
use hlbc::types::{Function, RefField, Type};
use hlbc::{Bytecode, Resolve};

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

fn to_haxe_type<'a>(ty: &Type, ctx: &'a Bytecode) -> impl Display + 'a {
    use crate::Type::*;
    match ty {
        Void => "Void",
        I32 => "Int",
        F64 => "Float",
        Bool => "Bool",
        Bytes => "hl.Bytes",
        Dyn => "Dynamic",
        Fun(_) => "Function",
        Obj(obj) => ctx.resolve(obj.name),
        _ => "other",
    }
}

impl Class {
    pub fn display<'a>(&'a self, ctx: &'a Bytecode, opts: &'a FormatOptions) -> impl Display + 'a {
        let new_opts = opts.inc_nesting();
        fmtools::fmt! { move
            {opts}"class "{self.name} if let Some(parent) = self.parent.as_ref() { " extends "{parent} } " {\n"
            for f in &self.fields {
                {new_opts} if f.static_ { "static " } "var "{f.name}": "{to_haxe_type(&ctx[f.ty], ctx)}";\n"
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
        let fun = self.fun.as_fn(ctx).unwrap();
        fmtools::fmt! { move
            {opts} if self.static_ { "static " } if self.dynamic { "dynamic " }
            "function "{fun.name(ctx)}"("
            {fmtools::join(", ", fun.args(ctx).iter().enumerate().skip(if self.static_ { 0 } else { 1 })
                .map(move |(i, arg)| fmtools::fmt! {move
                    {fun.arg_name(ctx, i).unwrap_or("_")}": "{to_haxe_type(&ctx[*arg], ctx)}
                }))}
            ")" if !fun.ty(ctx).ret.is_void() { ": "{to_haxe_type(fun.ret(ctx), ctx)} } " {"

            if self.statements.is_empty() {
                "}"
            } else {
                "\n"
                for stmt in &self.statements {
                    {new_opts}{stmt.display(&new_opts, ctx, fun)}"\n"
                }
                {opts}"}"
            }
            "\n"
        }
    }
}

impl Display for Constant {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use Constant::*;
        match self {
            InlineInt(c) => Display::fmt(c, f),
            Int(c) => Display::fmt(c, f),
            Float(c) => Display::fmt(c, f),
            String(c) => write!(f, "\"{c}\""),
            Bool(c) => Display::fmt(c, f),
            Null => Display::fmt("null", f),
            This => Display::fmt("this", f),
        }
    }
}

impl Operation {
    pub fn display<'a>(
        &'a self,
        indent: &'a FormatOptions,
        code: &'a Bytecode,
        f: &'a Function,
    ) -> impl Display + 'a {
        use Operation::*;
        macro_rules! disp {
            ($e:ident) => {
                $e.display(indent, code, f)
            };
        }
        fmtools::fmt! { move
            match self {
                Add(e1, e2) => {{disp!(e1)}" + "{disp!(e2)}}
                Sub(e1, e2) => {{disp!(e1)}" - "{disp!(e2)}}
                Mul(e1, e2) => {{disp!(e1)}" * "{disp!(e2)}}
                Div(e1, e2) => {{disp!(e1)}" / "{disp!(e2)}}
                Mod(e1, e2) => {{disp!(e1)}" % "{disp!(e2)}}
                Shl(e1, e2) => {{disp!(e1)}" << "{disp!(e2)}}
                Shr(e1, e2) => {{disp!(e1)}" >> "{disp!(e2)}}
                And(e1, e2) => {{disp!(e1)}" && "{disp!(e2)}}
                Or(e1, e2) => {{disp!(e1)}" || "{disp!(e2)}}
                Xor(e1, e2) => {{disp!(e1)}" ^ "{disp!(e2)}}
                Neg(expr) => {"-"{disp!(expr)}}
                Not(expr) => {"!"{disp!(expr)}}
                Incr(expr) => {{disp!(expr)}"++"}
                Decr(expr) => {{disp!(expr)}"--"}
                Eq(e1, e2) => {{disp!(e1)}" == "{disp!(e2)}}
                NotEq(e1, e2) => {{disp!(e1)}" != "{disp!(e2)}}
                Gt(e1, e2) => {{disp!(e1)}" > "{disp!(e2)}}
                Gte(e1, e2) => {{disp!(e1)}" >= "{disp!(e2)}}
                Lt(e1, e2) => {{disp!(e1)}" < "{disp!(e2)}}
                Lte(e1, e2) => {{disp!(e1)}" <= "{disp!(e2)}}
            }
        }
    }
}

impl Expr {
    pub fn display<'a>(
        &'a self,
        indent: &'a FormatOptions,
        code: &'a Bytecode,
        f: &'a Function,
    ) -> impl Display + 'a {
        macro_rules! disp {
            ($e:expr) => {
                $e.display(indent, code, f)
            };
        }
        fmtools::fmt! { move
            match self {
                Expr::Anonymous(ty, values) => match &code[*ty] {
                    Type::Virtual { fields } => {
                        "{"{ fmtools::join(", ", fields
                            .iter()
                            .enumerate()
                            .map(|(i, f)| {
                                fmtools::fmt! { move
                                    {f.name}": "{disp!(values.get(&RefField(i)).unwrap())}
                                }
                            })) }"}"
                    }
                    _ => "[invalid anonymous type]",
                },
                Expr::Array(array, index) => {
                    {disp!(array)}"["{disp!(index)}"]"
                }
                Expr::Call(call) => {
                    {disp!(call.fun)}"("{fmtools::join(", ", call.args.iter().map(|e| disp!(e)))}")"
                }
                Expr::Constant(c) => {{c}},
                Expr::Constructor(ConstructorCall { ty, args }) => {
                    "new "{ty.display::<EnhancedFmt>(code)}"("{fmtools::join(", ", args.iter().map(|e| disp!(e)))}")"
                }
                Expr::Closure(f, stmts) => {
                    let fun = f.as_fn(code).unwrap();
                    "("{fmtools::join(", ", fun.ty(code).args.iter().enumerate().map(move |(i, arg)|
                        fmtools::fmt! { move
                            {fun.arg_name(code, i).unwrap_or("_")}": "{to_haxe_type(&code[*arg], code)}
                        }
                    ))}") -> {\n"
                    let indent2 = indent.inc_nesting();
                    for stmt in stmts {
                        {indent2}{stmt.display(&indent2, code, fun)}"\n"
                    }
                    {indent}"}"
                }
                Expr::EnumConstr(ty, constr, args) => {
                    {constr.display::<EnhancedFmt>(code, &code[*ty])}"("{fmtools::join(", ", args.iter().map(|e| disp!(e)))}")"
                }
                Expr::Field(receiver, name) => {
                    {disp!(receiver)}"."{name}
                }
                Expr::FunRef(fun) => {{fun.name(code)}},
                Expr::IfElse { cond, if_, else_ } => {
                    "if ("{disp!(cond)}") {\n"
                    let indent2 = indent.inc_nesting();
                    for stmt in if_ {
                        {indent2}{stmt.display(&indent2, code, f)}"\n"
                    }
                    {indent}"} else {\n"
                    for stmt in else_ {
                        {indent2}{stmt.display(&indent2, code, f)}"\n"
                    }
                    {indent}"}"
                }
                Expr::Op(op) => {{disp!(op)}},
                Expr::Unknown(msg) => {
                     "["{msg}"]"
                }
                Expr::Variable(x, name) => {{
                    if let Some(name) = name {
                        name.clone()
                    } else {
                        x.to_string()
                    }
                }}
            }
        }
    }
}

impl Statement {
    pub fn display<'a>(
        &'a self,
        indent: &'a FormatOptions,
        code: &'a Bytecode,
        f: &'a Function,
    ) -> impl Display + 'a {
        macro_rules! disp {
            ($e:expr) => {
                $e.display(indent, code, f)
            };
        }
        fmtools::fmt! { move
            match self {
                Statement::Assign {
                    declaration,
                    variable,
                    assign,
                } => {
                    if *declaration { "var " } else { "" }{disp!(variable)}" = "{disp!(assign)}";"
                }
                Statement::ExprStatement(expr) => {
                    {disp!(expr)}";"
                }
                Statement::Return(expr) => {
                    "return" if let Some(e) = expr { " "{disp!(e)} } ";"
                }
                Statement::IfElse { cond, if_, else_ } => {
                    "if ("{disp!(cond)}") {\n"
                    let indent2 = indent.inc_nesting();
                    for stmt in if_ {
                        {indent2}{stmt.display(&indent2, code, f)}"\n"
                    }
                    {indent}"}"
                    if !else_.is_empty() {
                        " else {\n"
                        for stmt in else_ {
                            {indent2}{stmt.display(&indent2, code, f)}"\n"
                        }
                        {indent}"}"
                    }
                }
                Statement::Switch {arg, default, cases} => {
                    "switch ("{disp!(arg)}") {\n"
                    let indent2 = indent.inc_nesting();
                    let indent3 = indent2.inc_nesting();
                    if !default.is_empty() {
                        {indent2}"default:\n"
                        for stmt in default {
                            {indent3}{stmt.display(&indent3, code, f)}"\n"
                        }
                    }
                    for (pattern, stmts) in cases {
                        {indent2}"case "{disp!(pattern)}":\n"
                        for stmt in stmts {
                            {indent3}{stmt.display(&indent3, code, f)}"\n"
                        }
                    }
                    {indent}"}"
                }
                Statement::While { cond, stmts } => {
                    "while ("{disp!(cond)}") {\n"
                    let indent2 = indent.inc_nesting();
                    for stmt in stmts {
                        {indent2}{stmt.display(&indent2, code, f)}"\n"
                    }
                    {indent}"}"
                }
                Statement::Break => {
                    "break;"
                }
                Statement::Continue => {
                    "continue;"
                }
                Statement::Throw(exc) => {
                    "throw "{disp!(exc)}
                }
                Statement::Try { stmts } => {
                    "try {\n"
                    let indent2 = indent.inc_nesting();
                    for stmt in stmts {
                        {indent2}{stmt.display(&indent2, code, f)}"\n"
                    }
                    {indent}"}"
                }
                Statement::Catch { stmts } => {
                    "catch () {\n"
                    let indent2 = indent.inc_nesting();
                    for stmt in stmts {
                        {indent2}{stmt.display(&indent2, code, f)}"\n"
                    }
                    {indent}"}"
                }
                Statement::Comment(comment) => {
                    "// "{comment}
                }
            }
        }
    }
}
