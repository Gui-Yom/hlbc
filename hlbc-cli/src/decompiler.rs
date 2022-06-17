use std::fmt;
use std::fmt::{Display, Write};

use hlbc::opcodes::Opcode;
use hlbc::types::{Function, RefField, RefFun, RefType, Reg, Type, TypeObj};
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

    /*
    for (i, r) in f
        .regs
        .iter()
        .enumerate()
        .skip(f.t.resolve(&code.types).get_type_fun().unwrap().args.len())
    {
        // Skip void type
        if !r.is_void() {
            writeln!(&mut buf, "{indent}var reg{i}: {}", r.display(code)).unwrap();
        }
    }*/

    writeln!(&mut buf).unwrap();

    let mut statements = Vec::with_capacity(f.ops.len());

    let mut iter = f.ops.iter().enumerate();
    while let Some((i, o)) = iter.next() {
        match o {
            &Opcode::Int { dst, ptr } => {
                statements.push(Statement::NewVariable {
                    reg: dst,
                    name: var_name(code, f, i),
                    assign: Expression::Constant(Constant::Int(ptr.resolve(&code.ints))),
                });
            }
            Opcode::New { dst } => {
                let call = process_constructor(code, f, *dst, &mut iter);
                statements.push(Statement::NewVariable {
                    reg: *dst,
                    name: var_name(code, f, i),
                    assign: Expression::Constructor(call),
                })
            }
            Opcode::Call1 { dst, fun, arg0 } => {
                /*
                if let Some(func) = fun.resolve_as_fn(code) {
                    if let Some(name) = func.name.map(|n| n.resolve(&code.strings)) {
                        if name == "__constructor__" && f.regtype(*dst).is_void() {
                            if let Some((r, cons_ctx)) = constructor_ctx.pop() {
                                if r != *arg0 {
                                    println!("[analysis error: wrong constructor]")
                                } else {
                                    statements.push(Statement::Constructor {
                                        reg: r,
                                        args: cons_ctx,
                                    })
                                }
                            } else {
                                println!("[analysis error: no new before constructor]");
                            }
                        }
                    }
                }*/
            }
            Opcode::Ret { ret } => {
                if !f.regtype(*ret).is_void() {
                    statements.push(Statement::Return {
                        expr: Expression::Reg(*ret),
                    })
                }
            }
            _ => {}
        }
    }

    for stmt in statements {
        stmt.display(&mut buf, code, f);
    }

    buf
}

fn var_name(code: &Bytecode, f: &Function, pos: usize) -> Option<String> {
    if let Some(assigns) = &f.assigns {
        for &(s, i) in assigns {
            if pos == i - 1 {
                return Some(s.resolve(&code.strings).to_string());
            }
        }
    }
    None
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
    Reg(Reg),
    Constant(Constant),
    Constructor(ConstructorCall),
}

impl Expression {
    fn display(&self, code: &Bytecode) -> String {
        match self {
            Expression::Reg(x) => format!("{x}"),
            Expression::Constant(x) => match x {
                Constant::Int(c) => format!("{c}"),
                Constant::Float(c) => format!("{c}"),
                Constant::String(c) => format!("{c}"),
            },
            Expression::Constructor(x) => {
                format!(
                    "new {}({})",
                    x.ty.display(code),
                    x.args
                        .iter()
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

fn process_constructor<'a>(
    code: &Bytecode,
    f: &Function,
    reg: Reg,
    ops: &mut impl Iterator<Item = (usize, &'a Opcode)>,
) -> ConstructorCall {
    let mut args = Vec::new();

    while let Some((i, o)) = ops.next() {
        match o {
            Opcode::Int { dst, ptr } => {
                args.push(Expression::Constant(Constant::Int(ptr.resolve(&code.ints))))
            }
            Opcode::Call1 { dst, fun, arg0 } => {
                if *arg0 == reg {
                    return ConstructorCall {
                        ty: f.regtype(reg),
                        args,
                    };
                }
            }
            Opcode::Call2 {
                dst,
                fun,
                arg0,
                arg1,
            } => {
                if *arg0 == reg {
                    return ConstructorCall {
                        ty: f.regtype(reg),
                        args,
                    };
                }
            }
            _ => {}
        }
    }
    unreachable!("No constructor call ?")
}
