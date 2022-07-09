use hlbc::types::{RefField, RefFun, RefType, Reg};
use hlbc::Bytecode;
use std::collections::HashMap;

/// Helper to process a stack of scopes (branches, loops)
pub(crate) struct Scopes {
    // A linked list would be appreciable i think
    /// There is always at least one scope, the root scope
    scopes: Vec<Scope>,
}

impl Scopes {
    pub(crate) fn new() -> Self {
        Self {
            scopes: vec![Scope {
                ty: ScopeType::RootScope,
                stmts: Vec::new(),
            }],
        }
    }

    pub(crate) fn pop_last_loop(&mut self) -> Option<(LoopScope, Vec<Statement>)> {
        for idx in (0..self.depth()).rev() {
            let scope = self.scopes.remove(idx);
            match scope.ty {
                ScopeType::Loop(l) => {
                    return Some((l, scope.stmts));
                }
                _ => {
                    self.scopes.insert(idx, scope);
                }
            }
        }
        None
    }

    pub(crate) fn last_mut(&mut self) -> &mut Scope {
        self.scopes.last_mut().unwrap()
    }

    pub(crate) fn push_scope(&mut self, scope: Scope) {
        self.scopes.push(scope);
    }

    pub(crate) fn depth(&self) -> usize {
        self.scopes.len()
    }

    pub(crate) fn push_stmt(&mut self, mut stmt: Option<Statement>) {
        // Start to iterate from the end ('for' because we need the index)
        for idx in (0..self.depth()).rev() {
            let mut scope = self.scopes.remove(idx);

            if let Some(stmt) = stmt.take() {
                scope.stmts.push(stmt);
            }

            // We only handle branches we know the length of
            // We can't know the end of a loop scope before seeing the jump back
            match scope.ty {
                ScopeType::Branch { mut len, cond } => {
                    // Decrease scope len
                    len -= 1;
                    if len <= 0 {
                        //println!("Decrease nesting {parent:?}");
                        stmt = Some(Statement::If {
                            cond,
                            stmts: scope.stmts,
                        });
                    } else {
                        // Scope continues
                        self.scopes.insert(
                            idx,
                            Scope {
                                ty: ScopeType::Branch { len, cond },
                                stmts: scope.stmts,
                            },
                        );
                    }
                }
                ScopeType::Else { mut len } => {
                    // Decrease scope len
                    len -= 1;
                    if len <= 0 {
                        //println!("Decrease nesting {parent:?}");
                        stmt = Some(Statement::Else { stmts: scope.stmts });
                    } else {
                        // Scope continues
                        self.scopes.insert(
                            idx,
                            Scope {
                                ty: ScopeType::Else { len },
                                stmts: scope.stmts,
                            },
                        );
                    }
                }
                _ => {
                    self.scopes.insert(idx, scope);
                }
            }
        }
        if let Some(stmt) = stmt.take() {
            self.last_mut().stmts.push(stmt);
        }
    }

    pub(crate) fn statements(mut self) -> Vec<Statement> {
        self.scopes.pop().unwrap().stmts
    }
}

pub(crate) struct Scope {
    pub(crate) ty: ScopeType,
    pub(crate) stmts: Vec<Statement>,
}

impl Scope {
    pub(crate) fn new(ty: ScopeType) -> Self {
        Self {
            ty,
            stmts: Vec::new(),
        }
    }
}

pub(crate) enum ScopeType {
    RootScope,
    Branch { len: i32, cond: Expr },
    Else { len: i32 },
    Loop(LoopScope),
}

pub(crate) struct LoopScope {
    pub(crate) cond: Option<Expr>,
}

impl LoopScope {
    pub(crate) fn new() -> Self {
        Self { cond: None }
    }
}

// TODO make this zero copy by accepting the Ref* types instead and only resolving on demand

#[derive(Debug, Clone)]
pub(crate) enum Constant {
    Int(i32),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    This,
}

#[derive(Debug, Clone)]
pub(crate) enum Operation {
    Not(Box<Expr>),
    Decr(Box<Expr>),
    Incr(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
    NotEq(Box<Expr>, Box<Expr>),
    Gt(Box<Expr>, Box<Expr>),
    Gte(Box<Expr>, Box<Expr>),
    Lt(Box<Expr>, Box<Expr>),
    Lte(Box<Expr>, Box<Expr>),
}

#[derive(Debug, Clone)]
pub(crate) struct ConstructorCall {
    pub(crate) ty: RefType,
    pub(crate) args: Vec<Expr>,
}

impl ConstructorCall {
    pub(crate) fn new(ty: RefType, args: Vec<Expr>) -> Self {
        Self { ty, args }
    }
}

/// Function or method call
#[derive(Debug, Clone)]
pub(crate) struct Call {
    pub(crate) fun: Expr,
    pub(crate) args: Vec<Expr>,
}

impl Call {
    pub(crate) fn new(fun: Expr, args: Vec<Expr>) -> Self {
        Self { fun, args }
    }

    pub(crate) fn new_fun(fun: RefFun, args: Vec<Expr>) -> Self {
        Self {
            fun: Expr::FunRef(fun),
            args,
        }
    }
}

/// An expression with a value
#[derive(Debug, Clone)]
pub(crate) enum Expr {
    /// Variable identifier
    Variable(Reg, Option<String>),
    /// Constant value
    Constant(Constant),
    /// Constructor call
    Constructor(ConstructorCall),
    /// Function call
    Call(Box<Call>),
    /// Operator
    Op(Operation),
    /// Function reference
    FunRef(RefFun),
    /// Field access : obj.field
    Field(Box<Expr>, String),
    /// An anonymous structure : { field: value }
    Anonymous(RefType, HashMap<RefField, Expr>),
}

pub(crate) fn cst_bool(cst: bool) -> Expr {
    Expr::Constant(Constant::Bool(cst))
}

pub(crate) fn cst_int(cst: i32) -> Expr {
    Expr::Constant(Constant::Int(cst))
}

pub(crate) fn cst_float(cst: f64) -> Expr {
    Expr::Constant(Constant::Float(cst))
}

pub(crate) fn cst_string(cst: String) -> Expr {
    Expr::Constant(Constant::String(cst))
}

pub(crate) fn cst_null() -> Expr {
    Expr::Constant(Constant::Null)
}

pub(crate) fn cst_this() -> Expr {
    Expr::Constant(Constant::This)
}

macro_rules! make_op_shorthand {
    ($name:ident, $op:ident, $( $e:ident ),+) => {
        pub(crate) fn $name($( $e: Expr ),+) -> Expr {
            Expr::Op(Operation::$op($( Box::new($e) ),+))
        }
    }
}

make_op_shorthand!(decr, Decr, e1);
make_op_shorthand!(incr, Incr, e1);
make_op_shorthand!(add, Add, e1, e2);
make_op_shorthand!(sub, Sub, e1, e2);
make_op_shorthand!(eq, Eq, e1, e2);
make_op_shorthand!(noteq, NotEq, e1, e2);
make_op_shorthand!(gt, Gt, e1, e2);
make_op_shorthand!(gte, Gte, e1, e2);
make_op_shorthand!(lt, Lt, e1, e2);
make_op_shorthand!(lte, Lte, e1, e2);

// Invert an expression, will also optimize the expression.
pub(crate) fn not(e: Expr) -> Expr {
    use Expr::Op;
    use Operation::*;
    match e {
        Op(Not(a)) => *a,
        Op(Eq(a, b)) => Op(NotEq(a, b)),
        Op(NotEq(a, b)) => Op(Eq(a, b)),
        Op(Gt(a, b)) => Op(Lte(a, b)),
        Op(Gte(a, b)) => Op(Lt(a, b)),
        Op(Lt(a, b)) => Op(Gte(a, b)),
        Op(Lte(a, b)) => Op(Gt(a, b)),
        _ => Op(Not(Box::new(e))),
    }
}

// Flip the operands of an expression
pub(crate) fn flip(e: Expr) -> Expr {
    use Expr::Op;
    use Operation::*;
    match e {
        Op(Add(a, b)) => Op(Add(b, a)),
        Op(Eq(a, b)) => Op(Eq(b, a)),
        Op(NotEq(a, b)) => Op(NotEq(b, a)),
        Op(Gt(a, b)) => Op(Lt(b, a)),
        Op(Gte(a, b)) => Op(Lte(b, a)),
        Op(Lt(a, b)) => Op(Gt(b, a)),
        Op(Lte(a, b)) => Op(Gte(b, a)),
        _ => e,
    }
}

pub(crate) fn call(fun: Expr, args: Vec<Expr>) -> Expr {
    Expr::Call(Box::new(Call::new(fun, args)))
}

pub(crate) fn call_fun(fun: RefFun, args: Vec<Expr>) -> Expr {
    Expr::Call(Box::new(Call::new_fun(fun, args)))
}

pub(crate) fn field(expr: Expr, obj: RefType, field: RefField, code: &Bytecode) -> Expr {
    Expr::Field(
        Box::new(expr),
        field
            .display_obj(obj.resolve(&code.types), code)
            .to_string(),
    )
}

#[derive(Debug, Clone)]
pub(crate) enum Statement {
    /// Variable assignment
    Assign {
        /// Should 'var' appear
        declaration: bool,
        variable: Expr,
        assign: Expr,
    },
    // Call a void function (no assignment)
    Call(Call),
    /// Return an expression
    Return(Expr),
    /// Return nothing / early return
    ReturnVoid,
    /// If statement
    If {
        cond: Expr,
        stmts: Vec<Statement>,
    },
    /// Else clause for the if statement
    Else {
        stmts: Vec<Statement>,
    },
    /// While statement
    While {
        cond: Expr,
        stmts: Vec<Statement>,
    },
}
