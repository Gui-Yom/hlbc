use hlbc::types::{RefFun, RefType, Reg};

pub(crate) struct Scopes {
    scopes: Vec<Scope>,
}

impl Scopes {
    pub(crate) fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    pub(crate) fn last_is_loop(&self) -> bool {
        self.scopes
            .last()
            .map_or(false, |s| matches!(s, Scope::Loop(_)))
    }

    pub(crate) fn pop_last_loop(&mut self) -> Option<LoopScope> {
        for idx in (0..self.depth()).rev() {
            match self.scopes.remove(idx) {
                Scope::Loop(l) => {
                    return Some(l);
                }
                other => {
                    self.scopes.insert(idx, other);
                }
            }
        }
        None
    }

    pub(crate) fn last(&self) -> Option<&Scope> {
        self.scopes.last()
    }

    pub(crate) fn last_mut(&mut self) -> Option<&mut Scope> {
        self.scopes.last_mut()
    }

    pub(crate) fn push_scope(&mut self, scope: Scope) {
        self.scopes.push(scope);
    }

    pub(crate) fn has_some(&self) -> bool {
        !self.scopes.is_empty()
    }

    pub(crate) fn depth(&self) -> usize {
        self.scopes.len()
    }

    pub(crate) fn process(&mut self, mut stmt: Option<Statement>) -> Option<Statement> {
        for idx in (0..self.depth()).rev() {
            let mut scope = self.scopes.remove(idx);

            if let Some(stmt) = stmt.take() {
                scope.push_stmt(stmt);
            }

            match scope {
                Scope::Branch {
                    mut len,
                    cond,
                    stmts,
                } => {
                    // Decrease scope len
                    len -= 1;
                    if len <= 0 {
                        //println!("Decrease nesting {parent:?}");
                        stmt = Some(Statement::If { cond, stmts });
                    } else {
                        // Scope continues
                        self.scopes.insert(idx, Scope::Branch { len, cond, stmts });
                    }
                }
                _ => {
                    self.scopes.insert(idx, scope);
                }
            }
        }
        if let Some(stmt) = stmt.take() {
            if let Some(scope) = self.scopes.last_mut() {
                scope.push_stmt(stmt);
            } else {
                return Some(stmt);
            }
        }
        None
    }
}

pub(crate) struct LoopScope {
    pub(crate) stmts: Vec<Statement>,
    pub(crate) cond: Option<Expression>,
}

impl LoopScope {
    pub(crate) fn new() -> Self {
        Self {
            stmts: Vec::new(),
            cond: None,
        }
    }
}

pub(crate) enum Scope {
    Branch {
        len: i32,
        cond: Expression,
        stmts: Vec<Statement>,
    },
    Loop(LoopScope),
}

impl Scope {
    pub(crate) fn push_stmt(&mut self, stmt: Statement) {
        match self {
            Scope::Branch { stmts, .. } => {
                stmts.push(stmt);
            }
            Scope::Loop(LoopScope { stmts, .. }) => {
                stmts.push(stmt);
            }
        }
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
    While {
        cond: Expression,
        stmts: Vec<Statement>,
    },
}
