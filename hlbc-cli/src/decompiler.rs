use std::collections::HashMap;
use std::fmt::Write;

use hlbc::opcodes::Opcode;
use hlbc::types::{Function, RefField, RefFun, RefGlobal, RefType, Reg, Type, TypeObj};
use hlbc::Bytecode;

pub fn decompile_class(code: &Bytecode, obj: &TypeObj) -> String {
    let mut buf = String::with_capacity(1024);

    let mut is_static = false;
    writeln!(
        &mut buf,
        "class {} {}{{",
        obj.name.display(code),
        if let Some(e) = obj.super_ {
            is_static = e.0 == 12;
            format!("extends {} ", e.display(code))
        } else {
            "".to_string()
        }
    )
    .unwrap();
    let indent = "  ";
    for (i, f) in obj
        .fields
        .iter()
        .enumerate()
        .skip(obj.fields.len() - obj.own_fields.len())
    {
        if obj.bindings.get(&RefField(i)).is_some() {
            continue;
        }
        writeln!(
            &mut buf,
            "{indent}{}var {}: {}",
            if is_static { "static " } else { "" },
            f.name.display(code),
            f.t.display(code)
        )
        .unwrap();
    }
    writeln!(&mut buf, "// BINDINGS").unwrap();

    for (fi, fun) in &obj.bindings {
        //let fi = &obj.fields[fi.0];
        let fun = fun.resolve_as_fn(code).unwrap();
        writeln!(
            &mut buf,
            "{indent}{}{}{}{indent}}}\n",
            if is_static { "static " } else { "" },
            decompile_function_header(code, fun),
            decompile_function_body(code, &format!("{indent}  "), fun)
        )
        .unwrap();
    }

    writeln!(&mut buf, "// METHODS").unwrap();

    for f in &obj.protos {
        let f = f.findex.resolve_as_fn(code).unwrap();
        writeln!(
            &mut buf,
            "{indent}{}{}{indent}}}\n",
            decompile_function_header(code, f),
            decompile_function_body(code, &format!("{indent}  "), f)
        )
        .unwrap();
    }
    writeln!(&mut buf, "}}").unwrap();
    buf
}

pub fn decompile_function_header(code: &Bytecode, f: &Function) -> String {
    let mut buf = String::with_capacity(256);

    write!(&mut buf, "function {}(", f.name.unwrap().display(code)).unwrap();

    match f.t.resolve(&code.types) {
        Type::Fun(fun) => {
            // Skip the first because its a method (this)
            for (i, a) in fun.args.iter().enumerate().skip(1) {
                if i != 1 {
                    write!(&mut buf, ", ").unwrap();
                }
                write!(&mut buf, "reg{}: {}", i, a.display(code)).unwrap();
            }
            writeln!(
                &mut buf,
                "): {} {{ // {}",
                fun.ret.display(code),
                f.findex.0
            )
            .unwrap();
        }
        _ => {
            unreachable!()
        }
    }
    buf
}

pub fn decompile_closure(code: &Bytecode, indent: &str, f: &Function) -> String {
    let mut buf = String::with_capacity(256);

    write!(&mut buf, "(").unwrap();

    match f.t.resolve(&code.types) {
        Type::Fun(fun) => {
            // Skip the first because its a method (this)
            for (i, a) in fun.args.iter().enumerate().skip(1) {
                if i != 1 {
                    write!(&mut buf, ", ").unwrap();
                }
                write!(&mut buf, "reg{}: {}", i, a.display(code)).unwrap();
            }
            writeln!(&mut buf, ") -> {{ // {}", f.findex.0).unwrap();
        }
        _ => {
            unreachable!()
        }
    }
    write!(
        &mut buf,
        "{}",
        decompile_function_body(code, &format!("{indent}  "), f)
    )
    .unwrap();

    writeln!(&mut buf, "{indent}}}").unwrap();
    buf
}

pub fn decompile_function_body(code: &Bytecode, indent: &str, f: &Function) -> String {
    let mut buf = String::with_capacity(256);

    writeln!(&mut buf).unwrap();

    for stmt in make_statements(code, f) {
        stmt.display(&mut buf, code, f);
    }

    buf
}

fn make_statements(code: &Bytecode, f: &Function) -> Vec<Statement> {
    let mut statements = Vec::with_capacity(f.ops.len());
    let mut reg_state = HashMap::new();
    let mut constructor_ctx = None;

    // Initialize register state with the function arguments
    for i in 0..f.ty(code).args.len() {
        reg_state.insert(
            Reg(i as u32),
            Expression::Variable(Reg(i as u32), f.arg_name(code, i)),
        );
    }

    macro_rules! process_simple_expr {
        ($i:expr, $dst:expr, $e:expr) => {
            let name = f.var_name(code, $i);
            let expr = $e;
            // Inline check
            if name.is_none() {
                reg_state.insert($dst, expr);
            } else {
                reg_state.insert($dst, Expression::Variable($dst, name.clone()));
                statements.push(Statement::NewVariable {
                    reg: $dst,
                    name,
                    assign: expr,
                });
            }
        };
    }

    macro_rules! make_args {
        ($($arg:expr),*) => {
            vec![$( reg_state.get($arg).expect("Not assigned ?").clone() ),*]
        }
    }

    let mut iter = f.ops.iter().enumerate();
    while let Some((i, o)) = iter.next() {
        match o {
            &Opcode::Int { dst, ptr } => {
                process_simple_expr!(
                    i,
                    dst,
                    Expression::Constant(Constant::Int(ptr.resolve(&code.ints)))
                );
            }
            &Opcode::Float { dst, ptr } => {
                process_simple_expr!(
                    i,
                    dst,
                    Expression::Constant(Constant::Float(ptr.resolve(&code.floats)))
                );
            }
            &Opcode::String { dst, ptr } => {
                process_simple_expr!(
                    i,
                    dst,
                    Expression::Constant(Constant::String(ptr.resolve(&code.strings).to_owned()))
                );
            }
            &Opcode::GetGlobal { dst, global } => {
                process_simple_expr!(
                    i,
                    dst,
                    Expression::Constant(Constant::String(
                        global_value_from_constant(code, global).unwrap()
                    ))
                );
            }
            &Opcode::New { dst } => {
                // Constructor analysis
                constructor_ctx = Some((dst, i));
            }
            &Opcode::Call0 { dst, fun } => {
                process_simple_expr!(i, dst, Expression::Call(fun, Vec::new()));
            }
            &Opcode::Call1 { dst, fun, arg0 } => {
                if let Some((new, j)) = constructor_ctx {
                    if new == arg0 {
                        process_simple_expr!(
                            j,
                            new,
                            Expression::Constructor(ConstructorCall {
                                ty: f.regtype(new),
                                args: Vec::new()
                            })
                        );
                    }
                } else {
                    process_simple_expr!(i, dst, Expression::Call(fun, make_args!(&arg0)));
                }
            }
            &Opcode::Call2 {
                dst,
                fun,
                arg0,
                arg1,
            } => {
                if let Some((new, j)) = constructor_ctx {
                    if new == arg0 {
                        process_simple_expr!(
                            j,
                            new,
                            Expression::Constructor(ConstructorCall {
                                ty: f.regtype(new),
                                args: make_args!(&arg1)
                            })
                        );
                    }
                } else {
                    process_simple_expr!(i, dst, Expression::Call(fun, make_args!(&arg0, &arg1)));
                }
            }
            &Opcode::Call3 {
                dst,
                fun,
                arg0,
                arg1,
                arg2,
            } => {
                if let Some((new, j)) = constructor_ctx {
                    if new == arg0 {
                        process_simple_expr!(
                            j,
                            new,
                            Expression::Constructor(ConstructorCall {
                                ty: f.regtype(new),
                                args: make_args!(&arg1, &arg2)
                            })
                        );
                    }
                } else {
                    process_simple_expr!(
                        i,
                        dst,
                        Expression::Call(fun, make_args!(&arg0, &arg1, &arg2))
                    );
                }
            }
            Opcode::Ret { ret } => {
                // Do not display return void;
                if !f.regtype(*ret).is_void() {
                    statements.push(Statement::Return {
                        expr: reg_state
                            .get(ret)
                            .expect("Returning an unused register ?")
                            .clone(),
                    })
                }
            }
            _ => {}
        }
    }

    statements
}

#[derive(Debug, Clone)]
struct ConstructorCall {
    ty: RefType,
    args: Vec<Expression>,
}

#[derive(Debug, Clone)]
enum Constant {
    Int(i32),
    Float(f64),
    String(String),
}

#[derive(Debug, Clone)]
enum Expression {
    Variable(Reg, Option<String>),
    Constant(Constant),
    Constructor(ConstructorCall),
    Call(RefFun, Vec<Expression>),
}

impl Expression {
    fn to_be_inlined(&self) -> bool {
        matches!(self, Expression::Variable(_, _) | Expression::Constant(_))
    }

    fn display(&self, code: &Bytecode) -> String {
        match self {
            Expression::Variable(x, name) => {
                if let Some(name) = name {
                    name.clone()
                } else {
                    format!("{x}")
                }
            }
            Expression::Constant(x) => match x {
                Constant::Int(c) => format!("{c}"),
                Constant::Float(c) => format!("{c}"),
                Constant::String(c) => format!("\"{c}\""),
            },
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

#[derive(Debug, Clone)]
enum Statement {
    NewVariable {
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
}

impl Statement {
    fn display(&self, w: &mut impl Write, code: &Bytecode, f: &Function) {
        match self {
            Statement::NewVariable { reg, name, assign } => {
                writeln!(
                    w,
                    "var {}: {} = {};",
                    name.as_ref().unwrap_or(&reg.to_string()),
                    f.regtype(*reg).display(code),
                    assign.display(code)
                )
                .unwrap();
            }
            Statement::Return { expr } => {
                writeln!(w, "return {};", expr.display(code)).unwrap();
            }
            _ => {}
        }
    }
}

fn global_value_from_constant(code: &Bytecode, global: RefGlobal) -> Option<String> {
    code.globals_initializers.get(&global).and_then(|&x| {
        if let Some(constants) = &code.constants {
            Some(code.strings[constants[x].fields[0]].to_owned())
        } else {
            None
        }
    })
}
