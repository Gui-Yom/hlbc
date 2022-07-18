use std::collections::HashMap;

use crate::types::{RefEnumConstruct, RefField, RefFun, RefType, Reg};
use crate::Bytecode;

#[derive(Debug)]
pub struct SourceFile {
    pub class: Class,
}

#[derive(Debug)]
pub struct Class {
    pub name: String,
    pub parent: Option<String>,
    pub fields: Vec<ClassField>,
    pub methods: Vec<Method>,
}

#[derive(Debug)]
pub struct ClassField {
    pub name: String,
    pub ty: RefType,
    pub static_: bool,
}

#[derive(Debug)]
pub struct Method {
    pub fun: RefFun,
    pub static_: bool,
    pub dynamic: bool,
    pub statements: Vec<Statement>,
}

// TODO make this zero copy by accepting the Ref* types instead and only resolving on demand

#[derive(Debug, Clone)]
pub enum Constant {
    Int(i32),
    Float(f64),
    String(String),
    Bool(bool),
    Null,
    /// 'this' instance
    This,
}

#[derive(Debug, Clone)]
pub enum Operation {
    /// \+
    Add(Box<Expr>, Box<Expr>),
    /// \-
    Sub(Box<Expr>, Box<Expr>),
    /// \*
    Mul(Box<Expr>, Box<Expr>),
    /// <<
    Shl(Box<Expr>, Box<Expr>),
    /// \>>
    Shr(Box<Expr>, Box<Expr>),
    /// && &
    And(Box<Expr>, Box<Expr>),
    /// || |
    Or(Box<Expr>, Box<Expr>),
    /// ^
    Xor(Box<Expr>, Box<Expr>),
    /// \-
    Neg(Box<Expr>),
    /// !
    Not(Box<Expr>),
    /// ++
    Incr(Box<Expr>),
    /// --
    Decr(Box<Expr>),
    /// ==
    Eq(Box<Expr>, Box<Expr>),
    /// !=
    NotEq(Box<Expr>, Box<Expr>),
    /// \>
    Gt(Box<Expr>, Box<Expr>),
    /// \>=
    Gte(Box<Expr>, Box<Expr>),
    /// \<
    Lt(Box<Expr>, Box<Expr>),
    /// \<=
    Lte(Box<Expr>, Box<Expr>),
}

/// Constructor call
#[derive(Debug, Clone)]
pub struct ConstructorCall {
    pub ty: RefType,
    pub args: Vec<Expr>,
}

impl ConstructorCall {
    pub fn new(ty: RefType, args: Vec<Expr>) -> Self {
        Self { ty, args }
    }
}

/// Function or method call
#[derive(Debug, Clone)]
pub struct Call {
    pub fun: Expr,
    pub args: Vec<Expr>,
}

impl Call {
    pub fn new(fun: Expr, args: Vec<Expr>) -> Self {
        Self { fun, args }
    }

    pub fn new_fun(fun: RefFun, args: Vec<Expr>) -> Self {
        Self {
            fun: Expr::FunRef(fun),
            args,
        }
    }
}

/// An expression with a value
#[derive(Debug, Clone)]
pub enum Expr {
    /// An anonymous structure : { field: value }
    Anonymous(RefType, HashMap<RefField, Expr>),
    /// Array access : array\[index]
    Array(Box<Expr>, Box<Expr>),
    /// Function call
    Call(Box<Call>),
    /// Constant value
    Constant(Constant),
    /// Constructor call
    Constructor(ConstructorCall),
    /// Arrow function (...) -> {...}
    Closure(RefFun, Vec<Statement>),
    EnumConstr(RefType, RefEnumConstruct, Vec<Expr>),
    /// Function reference
    FunRef(RefFun),
    /// Field access : obj.field
    Field(Box<Expr>, String),
    /// Operator
    Op(Operation),
    // For when there should be something, but we don't known what
    Unknown(String),
    /// Variable identifier
    Variable(Reg, Option<String>),
}

pub fn cst_int(cst: i32) -> Expr {
    Expr::Constant(Constant::Int(cst))
}

pub fn cst_float(cst: f64) -> Expr {
    Expr::Constant(Constant::Float(cst))
}

pub fn cst_bool(cst: bool) -> Expr {
    Expr::Constant(Constant::Bool(cst))
}

pub fn cst_string(cst: String) -> Expr {
    Expr::Constant(Constant::String(cst))
}

pub fn cst_null() -> Expr {
    Expr::Constant(Constant::Null)
}

pub fn cst_this() -> Expr {
    Expr::Constant(Constant::This)
}

/// Create a shorthand function to create an expression from an operator
macro_rules! make_op_shorthand {
    ($name:ident, $op:ident, $( $e:ident ),+) => {
        pub(crate) fn $name($( $e: Expr ),+) -> Expr {
            Expr::Op(Operation::$op($( Box::new($e) ),+))
        }
    }
}

make_op_shorthand!(add, Add, e1, e2);
make_op_shorthand!(sub, Sub, e1, e2);
make_op_shorthand!(mul, Mul, e1, e2);
make_op_shorthand!(shl, Shl, e1, e2);
make_op_shorthand!(shr, Shr, e1, e2);
make_op_shorthand!(and, And, e1, e2);
make_op_shorthand!(or, Or, e1, e2);
make_op_shorthand!(xor, Xor, e1, e2);
make_op_shorthand!(neg, Neg, e1);
make_op_shorthand!(incr, Incr, e1);
make_op_shorthand!(decr, Decr, e1);
make_op_shorthand!(eq, Eq, e1, e2);
make_op_shorthand!(noteq, NotEq, e1, e2);
make_op_shorthand!(gt, Gt, e1, e2);
make_op_shorthand!(gte, Gte, e1, e2);
make_op_shorthand!(lt, Lt, e1, e2);
make_op_shorthand!(lte, Lte, e1, e2);

/// Invert an expression, will also optimize the expression.
pub fn not(e: Expr) -> Expr {
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

/// Flip the operands of an expression
pub fn flip(e: Expr) -> Expr {
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

pub fn array(array: Expr, index: Expr) -> Expr {
    Expr::Array(Box::new(array), Box::new(index))
}

pub fn call(fun: Expr, args: Vec<Expr>) -> Expr {
    Expr::Call(Box::new(Call::new(fun, args)))
}

pub fn call_fun(fun: RefFun, args: Vec<Expr>) -> Expr {
    Expr::Call(Box::new(Call::new_fun(fun, args)))
}

pub fn field(expr: Expr, obj: RefType, field: RefField, code: &Bytecode) -> Expr {
    Expr::Field(
        Box::new(expr),
        field
            .display_obj(obj.resolve(&code.types), code)
            .to_string(),
    )
}

#[derive(Debug, Clone)]
pub enum Statement {
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
    Break,
    Continue,
    Throw(Expr),
}
