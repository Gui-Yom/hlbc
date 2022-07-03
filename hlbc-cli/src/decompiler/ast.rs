use hlbc::types::{RefFun, RefType, Reg};

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

    pub(crate) fn has_some(&self) -> bool {
        self.depth() > 1
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
    Branch { len: i32, cond: Expression },
    Else { len: i32 },
    Loop(LoopScope),
}

pub(crate) struct LoopScope {
    pub(crate) cond: Option<Expression>,
}

impl LoopScope {
    pub(crate) fn new() -> Self {
        Self { cond: None }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct ConstructorCall {
    pub(crate) ty: RefType,
    pub(crate) args: Vec<Expression>,
}

impl ConstructorCall {
    pub(crate) fn new(ty: RefType, args: Vec<Expression>) -> Self {
        Self { ty, args }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Constant {
    Int(i32),
    Float(f64),
    String(String),
    Bool(bool),
}

pub(crate) fn constant_bool(cst: bool) -> Expression {
    Expression::Constant(Constant::Bool(cst))
}

#[derive(Debug, Clone)]
pub(crate) enum Operation {
    Not(Box<Expression>),
    Decr(Box<Expression>),
    Incr(Box<Expression>),
    Add(Box<Expression>, Box<Expression>),
    Sub(Box<Expression>, Box<Expression>),
    Gt(Box<Expression>, Box<Expression>),
    Gte(Box<Expression>, Box<Expression>),
    Lt(Box<Expression>, Box<Expression>),
    Lte(Box<Expression>, Box<Expression>),
}

macro_rules! make_op_shorthand {
    ($name:ident, $op:ident, $( $e:ident ),+) => {
        pub(crate) fn $name($( $e: Expression ),+) -> Expression {
            Expression::Op(Operation::$op($( Box::new($e) ),+))
        }
    }
}

make_op_shorthand!(decr, Decr, e1);
make_op_shorthand!(incr, Incr, e1);
make_op_shorthand!(add, Add, e1, e2);
make_op_shorthand!(sub, Sub, e1, e2);
make_op_shorthand!(gt, Gt, e1, e2);
make_op_shorthand!(gte, Gte, e1, e2);
make_op_shorthand!(lt, Lt, e1, e2);
make_op_shorthand!(lte, Lte, e1, e2);

// Invert an expression, will also optimize the expression.
pub(crate) fn not(e: Expression) -> Expression {
    match &e {
        Expression::Op(op) => match op {
            Operation::Not(a) => *a.clone(),
            Operation::Decr(_) => e,
            Operation::Incr(_) => e,
            Operation::Add(_, _) => e,
            Operation::Sub(_, _) => e,
            Operation::Gt(a, b) => Expression::Op(Operation::Lte(a.clone(), b.clone())),
            Operation::Gte(a, b) => Expression::Op(Operation::Lt(a.clone(), b.clone())),
            Operation::Lt(a, b) => Expression::Op(Operation::Gte(a.clone(), b.clone())),
            Operation::Lte(a, b) => Expression::Op(Operation::Gt(a.clone(), b.clone())),
        },
        _ => Expression::Op(Operation::Not(Box::new(e))),
    }
}

#[derive(Debug, Clone)]
pub(crate) enum Expression {
    Variable(Reg, Option<String>),
    Constant(Constant),
    Constructor(ConstructorCall),
    Call(RefFun, Vec<Expression>),
    Op(Operation),
}

#[derive(Debug, Clone)]
pub(crate) enum Statement {
    NewVariable {
        reg: Reg,
        name: Option<String>,
        assign: Expression,
    },
    Assign {
        reg: Reg,
        name: Option<String>,
        assign: Expression,
    },
    CallVoid {
        fun: RefFun,
        args: Vec<Expression>,
    },
    Return {
        expr: Expression,
    },
    ReturnVoid,
    If {
        cond: Expression,
        stmts: Vec<Statement>,
    },
    Else {
        stmts: Vec<Statement>,
    },
    While {
        cond: Expression,
        stmts: Vec<Statement>,
    },
}
