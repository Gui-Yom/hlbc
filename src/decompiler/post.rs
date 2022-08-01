use crate::decompiler::ast::{add, Expr, Statement};
use crate::Bytecode;

/// Transforms an if/else statement where both branches assign a value to the same variable to an if/else expression.
/// ```haxe
/// if (cond) {
///     var a = 1;
/// } else {
///     a = 2;
/// }
/// ```
/// becomes this :
/// ```haxe
/// var a = if (cond) {
///     1
/// } else {
///     2
/// };
/// ```
pub(crate) fn if_expression(stmt: &mut Statement) {
    let opt = match stmt {
        Statement::IfElse { cond, if_, else_ } => {
            // We only have to check the last statement in each branches.
            // We assume their types to be the same (checked by the haxe compiler)
            match if_.last() {
                Some(Statement::Assign {
                    declaration,
                    variable: if_var,
                    assign: if_assign,
                }) => match else_.last() {
                    Some(Statement::Assign {
                        variable: else_var,
                        assign: else_assign,
                        ..
                    }) => match if_var {
                        Expr::Variable(r1, _) => match else_var {
                            Expr::Variable(r2, _) if r1 == r2 => Some((
                                *declaration,
                                if_var.clone(),
                                cond.clone(),
                                if_assign.clone(),
                                else_assign.clone(),
                                if_.clone(),
                                else_.clone(),
                            )),
                            _ => None,
                        },
                        _ => None,
                    },
                    _ => None,
                },
                _ => None,
            }
        }
        _ => None,
    };

    if let Some((decl, var, cond, if_assign, else_assign, mut if_stmts, mut else_stmts)) = opt {
        *if_stmts.last_mut().unwrap() = Statement::ExprStatement(if_assign);
        *else_stmts.last_mut().unwrap() = Statement::ExprStatement(else_assign);
        *stmt = Statement::Assign {
            declaration: decl,
            variable: var,
            assign: Expr::IfElse {
                cond: Box::new(cond),
                if_: if_stmts,
                else_: else_stmts,
            },
        }
    }
}

// TODO AST-PP switch expressions

/// Restore string concatenation. They are translated to calls to \_\_add__ at compilation.
/// ```haxe
/// __add__("hello ", "world")
/// ```
/// becomes :
/// ```haxe
/// "hello " + "world"
/// ```
pub(crate) fn string_concat(code: &Bytecode, expr: &mut Expr) {
    let args = match expr {
        Expr::Call(call) => match call.fun {
            Expr::FunRef(fun) => {
                if fun.name(code).map(|n| n == "__add__").unwrap_or(false) && call.args.len() == 2 {
                    Some((call.args[0].clone(), call.args[1].clone()))
                } else {
                    None
                }
            }
            _ => None,
        },
        _ => None,
    };

    if let Some((arg0, arg1)) = args {
        *expr = add(arg0, arg1);
    }
}

pub(crate) fn itos(code: &Bytecode, expr: &mut Expr) {
    let var = match expr {
        Expr::Call(call) => match call.fun {
            Expr::FunRef(fun) if fun.name(code).map(|n| n == "__alloc__").unwrap_or(false) => {
                match &call.args[0] {
                    Expr::Call(call) => match call.fun {
                        Expr::FunRef(fun)
                            if fun.name(code).map(|n| n == "itos").unwrap_or(false) =>
                        {
                            println!("");
                            Some(call.args[0].clone())
                        }
                        _ => None,
                    },
                    _ => None,
                }
            }
            _ => None,
        },
        _ => None,
    };

    if let Some(int) = var {
        *expr = int;
    }
}
