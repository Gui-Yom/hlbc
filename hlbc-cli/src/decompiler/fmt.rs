use std::fmt;
use std::fmt::{Display, Formatter, Write};

use hlbc::types::Function;
use hlbc::Bytecode;

use crate::decompiler::ast::{Constant, ConstructorCall, Expression, Operation, Statement};
use crate::FormatOptions;

impl Display for FormatOptions {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.indent)
    }
}

impl Operation {
    pub(crate) fn display(&self, code: &Bytecode) -> String {
        match self {
            Operation::Not(expr) => {
                format!("!{}", expr.display(code))
            }
            Operation::Decr(expr) => {
                format!("{}--", expr.display(code))
            }
            Operation::Incr(expr) => {
                format!("{}++", expr.display(code))
            }
            Operation::Add(e1, e2) => {
                format!("{} + {}", e1.display(code), e2.display(code))
            }
            Operation::Sub(e1, e2) => {
                format!("{} - {}", e1.display(code), e2.display(code))
            }
            Operation::Eq(e1, e2) => {
                format!("{} == {}", e1.display(code), e2.display(code))
            }
            Operation::NotEq(e1, e2) => {
                format!("{} == {}", e1.display(code), e2.display(code))
            }
            Operation::Gt(e1, e2) => {
                format!("{} > {}", e1.display(code), e2.display(code))
            }
            Operation::Gte(e1, e2) => {
                format!("{} >= {}", e1.display(code), e2.display(code))
            }
            Operation::Lt(e1, e2) => {
                format!("{} < {}", e1.display(code), e2.display(code))
            }
            Operation::Lte(e1, e2) => {
                format!("{} <= {}", e1.display(code), e2.display(code))
            }
        }
    }
}

impl Expression {
    pub(crate) fn display(&self, code: &Bytecode) -> String {
        match self {
            Expression::Variable(x, name) => {
                if let Some(name) = name {
                    name.clone()
                } else {
                    format!("{x}")
                }
            }
            Expression::Constant(x) => match x {
                Constant::Int(c) => c.to_string(),
                Constant::Float(c) => c.to_string(),
                Constant::String(c) => format!("\"{c}\""),
                Constant::Bool(c) => c.to_string(),
                Constant::Null => "null".to_owned(),
            },
            Expression::Op(op) => op.display(code),
            Expression::Constructor(ConstructorCall { ty, args }) => {
                format!(
                    "new {}({})",
                    ty.display(code),
                    args.iter()
                        .map(|a| a.display(code))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Expression::Call(fun, args) => {
                format!(
                    "{}({})",
                    fun.display_call(code),
                    args.iter()
                        .map(|a| a.display(code))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
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
            Statement::NewVariable { reg, name, assign } => {
                writeln!(
                    w,
                    "var {}: {} = {};",
                    name.as_ref().unwrap_or(&reg.to_string()),
                    f.regtype(*reg).display(code),
                    assign.display(code)
                )?;
            }
            Statement::Assign { reg, name, assign } => {
                writeln!(
                    w,
                    "{} = {};",
                    name.as_ref().unwrap_or(&reg.to_string()),
                    assign.display(code)
                )?;
            }
            Statement::CallVoid { fun, args } => {
                writeln!(
                    w,
                    "{}({})",
                    fun.display_call(code),
                    args.iter()
                        .map(|a| a.display(code))
                        .collect::<Vec<_>>()
                        .join(", ")
                )?;
            }
            Statement::Return { expr } => {
                writeln!(w, "return {};", expr.display(code))?;
            }
            Statement::ReturnVoid => {
                writeln!(w, "return;")?;
            }
            Statement::If { cond, stmts } => {
                writeln!(w, "if ({}) {{", cond.display(code))?;
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
                writeln!(w, "while ({}) {{", cond.display(code))?;
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
