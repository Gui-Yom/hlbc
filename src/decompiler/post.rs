use crate::decompiler::ast::{Expr, Statement};

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
        _ => unreachable!("The visitor gave us something other than an IfElse statement"),
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
