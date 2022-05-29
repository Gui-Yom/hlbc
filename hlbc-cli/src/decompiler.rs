use std::fmt::Write;

use hlbc::opcodes::Opcode;
use hlbc::types::{Function, RefField, Type, TypeObj};
use hlbc::Bytecode;

pub fn decompile_class(code: &Bytecode, obj: &TypeObj) -> String {
    let mut buf = String::with_capacity(1024);

    let mut is_static = false;
    writeln!(
        &mut buf,
        "class {} {}{{",
        obj.name.display(&code),
        if let Some(e) = obj.super_ {
            is_static = e.0 == 12;
            format!("extends {} ", e.display(&code))
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
            f.name.display(&code),
            f.t.display(&code)
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
    }

    for (i, o) in f.ops.iter().enumerate() {
        write!(&mut buf, "{indent}").unwrap();
        match o {
            Opcode::InstanceClosure { dst, obj, fun } => {
                write!(
                    &mut buf,
                    "{dst} = {}",
                    decompile_closure(code, &indent, fun.resolve_as_fn(code).unwrap())
                )
                .unwrap();
            }
            Opcode::Ret { ret } => {
                // Skip void type
                if i != f.ops.len() - 1 || !f.regtype(*ret).is_void() {
                    writeln!(&mut buf, "return {ret}").unwrap();
                }
            }
            _ => {
                writeln!(&mut buf, "{}", o.display(code, f, 0)).unwrap();
            }
        }
    }

    buf
}
