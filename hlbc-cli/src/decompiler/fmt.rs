use std::fmt;
use std::fmt::{Display, Formatter, Write};

use hlbc::types::{Function, RefField, Type};
use hlbc::Bytecode;

use crate::decompiler::ast::{Call, Constant, ConstructorCall, Expr, Operation, Statement};
use crate::FormatOptions;

impl Display for FormatOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.indent)
    }
}

impl Operation {
    pub(crate) fn display(&self, indent: &FormatOptions, code: &Bytecode) -> String {
        match self {
            Operation::Not(expr) => {
                format!("!{}", expr.display(indent, code))
            }
            Operation::Decr(expr) => {
                format!("{}--", expr.display(indent, code))
            }
            Operation::Incr(expr) => {
                format!("{}++", expr.display(indent, code))
            }
            Operation::Add(e1, e2) => {
                format!(
                    "{} + {}",
                    e1.display(indent, code),
                    e2.display(indent, code)
                )
            }
            Operation::Sub(e1, e2) => {
                format!(
                    "{} - {}",
                    e1.display(indent, code),
                    e2.display(indent, code)
                )
            }
            Operation::Eq(e1, e2) => {
                format!(
                    "{} == {}",
                    e1.display(indent, code),
                    e2.display(indent, code)
                )
            }
            Operation::NotEq(e1, e2) => {
                format!(
                    "{} == {}",
                    e1.display(indent, code),
                    e2.display(indent, code)
                )
            }
            Operation::Gt(e1, e2) => {
                format!(
                    "{} > {}",
                    e1.display(indent, code),
                    e2.display(indent, code)
                )
            }
            Operation::Gte(e1, e2) => {
                format!(
                    "{} >= {}",
                    e1.display(indent, code),
                    e2.display(indent, code)
                )
            }
            Operation::Lt(e1, e2) => {
                format!(
                    "{} < {}",
                    e1.display(indent, code),
                    e2.display(indent, code)
                )
            }
            Operation::Lte(e1, e2) => {
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
    pub(crate) fn display(&self, indent: &FormatOptions, code: &Bytecode) -> String {
        match self {
            Expr::Variable(x, name) => {
                if let Some(name) = name {
                    name.clone()
                } else {
                    format!("{x}")
                }
            }
            Expr::Constant(x) => match x {
                Constant::Int(c) => c.to_string(),
                Constant::Float(c) => c.to_string(),
                Constant::String(c) => format!("\"{c}\""),
                Constant::Bool(c) => c.to_string(),
                Constant::Null => "null".to_owned(),
                Constant::This => "this".to_owned(),
            },
            Expr::Op(op) => op.display(indent, code),
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
            Expr::Field(receiver, name) => {
                format!("{}.{}", receiver.display(indent, code), name)
            }
            Expr::FunRef(fun) => fun.display_call(code).to_string(),
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
        }
    }
}

impl Statement {
    pub(crate) fn display(
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
        }
        Ok(())
    }
}
