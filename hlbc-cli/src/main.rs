use std::io::{stdin, BufReader, BufWriter, Write};
use std::iter::repeat;
use std::time::Instant;
use std::{env, fs};

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use hlbc::analysis::{find_fun_refs, iter_ops};
use hlbc::opcodes::Opcode;
use hlbc::types::{RefFun, RefFunPointee, Type};
use hlbc::*;

use crate::command::{parse_command, Command, ElementRef, FileOrIndex, ParseContext};

mod command;
mod decompiler;

fn main() -> anyhow::Result<()> {
    let tty = atty::is(atty::Stream::Stdout);

    let start = Instant::now();

    let code = {
        let mut args = env::args();
        let filename = args.nth(1).unwrap();
        let mut r = BufReader::new(fs::File::open(&filename)?);
        Bytecode::load(&mut r)?
    };

    if tty {
        println!("Loaded ! ({} ms)", start.elapsed().as_millis());
    }

    let mut stdout = StandardStream::stdout(if tty {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    });

    loop {
        let mut line = String::new();
        println!();
        print!("> ");
        stdout.flush()?;
        stdin().read_line(&mut line)?;
        let line = line.trim();

        let parse_ctx = ParseContext {
            int_max: code.ints.len(),
            float_max: code.floats.len(),
            string_max: code.strings.len(),
            debug_file_max: code.debug_files.as_ref().map(|v| v.len()).unwrap_or(0),
            type_max: code.types.len(),
            global_max: code.globals.len(),
            native_max: code.natives.len(),
            constant_max: code.constants.as_ref().map(|v| v.len()).unwrap_or(0),
            findex_max: code.findexes.len(),
        };
        match parse_command(&parse_ctx, line) {
            Ok(Command::Exit) => {
                break;
            }
            Ok(cmd) => {
                process_command(&mut stdout, &code, cmd)?;
            }
            Err(errors) => {
                for e in errors {
                    eprintln!("Error while parsing command. {e:?}");
                }
                continue;
            }
        }
    }
    Ok(())
}

fn process_command(
    stdout: &mut StandardStream,
    code: &Bytecode,
    cmd: Command,
) -> anyhow::Result<()> {
    macro_rules! print_i {
        ($i:expr) => {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Ansi256(242))))?;
            write!(stdout, "{:<3}: ", $i)?;
            stdout.reset()?;
        };
    }

    macro_rules! require_debug_info {
        () => {
            if let Some(debug_files) = &code.debug_files {
                debug_files
            } else {
                println!("No debug info in this binary");
                return Ok(());
            }
        };
    }

    match cmd {
        Command::Exit => unreachable!(),
        Command::Help => {
            println!(
                r#"Commands :
info                   | General information about the bytecode
help                   | This message
entrypoint             | Get the bytecode entrypoint
i,int       <idx>      | Get the int at index
f,float     <idx>      | Get the float at index
s,string    <idx>      | Get the string at index
sstr        <str>      | Find a string
file,debugfile <idx>      | Get the debug file name at index
sfile       <str>      | Find the debug file named
t,type      <idx>      | Get the type at index
g,global    <idx>      | Get global at index
c,constant  <idx>      | Get constant at index
n,native    <idx>      | Get native at index
fnh         <findex>   | Get header of function at index
fn          <findex>   | Get function at index
sfn         <str>      | Get function named
infile      <idx|str> | Find functions in file
fileof      <findex>   | Get the file where findex is defined
refto       <any@idx>  | Find references to a given bytecode element
saveto      <filename> | Serialize the bytecode to a file
callgraph   <findex> <depth> | Create a dot call graph froma function and a max depth
                "#
            );
        }
        Command::Info => {
            println!(
                "version: {}\ndebug: {}\nnints: {}\nnfloats: {}\nnstrings: {}\nntypes: {}\nnnatives: {}\nnfunctions: {}\nnconstants: {}",
                code.version,
                code.debug_files.is_some(),
                code.ints.len(),
                code.floats.len(),
                code.strings.len(),
                code.types.len(),
                code.natives.len(),
                code.functions.len(),
                code.constants.as_ref().map_or(0, |c| c.len())
            );
        }
        Command::Entrypoint => {
            println!(
                "{}",
                code.functions[code.findexes.get(&code.entrypoint).unwrap().0].display_header(code)
            );
        }
        Command::Int(range) => {
            for i in range {
                print_i!(i);
                println!("{}", code.ints[i]);
            }
        }
        Command::Float(range) => {
            for i in range {
                print_i!(i);
                println!("{}", code.floats[i]);
            }
        }
        Command::String(range) => {
            for i in range {
                print_i!(i);
                println!("{}", code.strings[i]);
            }
        }
        Command::SearchStr(str) => {
            for (i, s) in code.strings.iter().enumerate() {
                if s.contains(&str) {
                    print_i!(i);
                    println!("{}", s);
                }
            }
        }
        Command::Debugfile(range) => {
            let debug_files = require_debug_info!();
            for i in range {
                print_i!(i);
                println!("{}", debug_files[i]);
            }
        }
        Command::SearchDebugfile(str) => {
            let debug_files = require_debug_info!();
            for (i, s) in debug_files.iter().enumerate() {
                if s.contains(&str) {
                    print_i!(i);
                    println!("{}", s);
                }
            }
        }
        Command::Type(range) => {
            for i in range {
                print_i!(i);
                let t = &code.types[i];
                println!("{}", t.display(code));
                match t {
                    Type::Obj(obj) => {
                        if let Some(sup) = obj.super_ {
                            println!("extends {}", sup.display(code));
                        }
                        println!("global: {}", obj.global.0);
                        println!("fields:");
                        for f in &obj.own_fields {
                            println!("  {}: {}", f.name.display(code), f.t.display(code));
                        }
                        println!("protos:");
                        for p in &obj.protos {
                            println!(
                                "  {}: {}",
                                p.name.display(code),
                                p.findex.display_header(code)
                            );
                        }
                        println!("bindings:");
                        for (fi, fun) in &obj.bindings {
                            println!(
                                "  {}: {}",
                                fi.display_obj(t, code),
                                fun.display_header(code)
                            );
                        }
                    }
                    Type::Enum {
                        global, constructs, ..
                    } => {
                        println!("global: {}", global.0);
                        println!("constructs:");
                        for c in constructs {
                            println!(
                                "  {}:",
                                if c.name.0 == 0 {
                                    "_".to_string()
                                } else {
                                    c.name.display(code)
                                }
                            );
                            for (i, p) in c.params.iter().enumerate() {
                                println!("    {i}: {}", p.display(code));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        Command::Global(range) => {
            for i in range {
                print_i!(i);
                println!("{}", code.globals[i].display(code));
            }
        }
        Command::Native(range) => {
            for i in range {
                print_i!(i);
                println!("{}", code.natives[i].display_header(code));
            }
        }
        Command::Constant(range) => {
            for i in range {
                print_i!(i);
                println!("{:#?}", code.constants.as_ref().unwrap()[i]);
            }
        }
        Command::FunctionHeader(range) => {
            for findex in range {
                print_i!(findex);
                if let Some(&(i, fun)) = code.findexes.get(&RefFun(findex)) {
                    if fun {
                        println!("{}", code.functions[i].display_header(code));
                    } else {
                        println!("{}", code.natives[i].display_header(code));
                    }
                } else {
                    println!("unknown");
                }
            }
        }
        Command::Function(range) => {
            for findex in range {
                print_i!(findex);
                if let Some(&(i, fun)) = code.findexes.get(&RefFun(findex)) {
                    if fun {
                        println!("{}", code.functions[i].display(code));
                    } else {
                        println!("{}", code.natives[i].display_header(code));
                    }
                } else {
                    println!("unknown");
                }
            }
        }
        Command::SearchFunction(str) => {
            // TODO search for function
            if let Some(&i) = code.fnames.get(&str) {
                println!("{}", code.functions[i].display_header(code));
            } else {
                println!("unknown");
            }
        }
        Command::InFile(foi) => {
            let debug_files = require_debug_info!();
            match foi {
                FileOrIndex::File(str) => {
                    if let Some(idx) =
                        debug_files
                            .iter()
                            .enumerate()
                            .find_map(
                                |(i, d): (usize, &String)| {
                                    if d == &str {
                                        Some(i)
                                    } else {
                                        None
                                    }
                                },
                            )
                    {
                        println!("Functions in file@{idx} : {}", debug_files[idx]);
                        for (i, f) in code.functions.iter().enumerate() {
                            if f.debug_info.as_ref().unwrap()[f.ops.len() - 1].0 == idx {
                                print_i!(i);
                                println!("{}", f.display_header(code));
                            }
                        }
                    } else {
                        println!("File {str} not found !");
                    }
                }
                FileOrIndex::Index(idx) => {
                    println!("Functions in file@{idx} : {}", debug_files[idx]);
                    for (i, f) in code.functions.iter().enumerate() {
                        if f.debug_info.as_ref().unwrap()[f.ops.len() - 1].0 == idx {
                            print_i!(i);
                            println!("{}", f.display_header(code));
                        }
                    }
                }
            }
        }
        Command::FileOf(idx) => {
            let debug_files = require_debug_info!();
            if let Some(p) = RefFun(idx).resolve(code) {
                match p {
                    RefFunPointee::Fun(f) => {
                        let idx = f.debug_info.as_ref().unwrap()[f.ops.len() - 1].0;
                        println!(
                            "{} is in file@{idx} : {}",
                            f.display_header(code),
                            &debug_files[idx]
                        );
                    }
                    RefFunPointee::Native(n) => {
                        println!("{} can't be in any file", n.display_header(code))
                    }
                }
            }
        }
        Command::SaveTo(file) => {
            let mut w = BufWriter::new(fs::File::create(&file)?);
            code.serialize(&mut w)?;
        }
        Command::Callgraph(idx, depth) => {
            #[cfg(feature = "graph")]
            {
                use hlbc::analysis::graph::{call_graph, display_graph};

                let graph = call_graph(code, RefFun(idx), depth);
                println!("{}", display_graph(&graph, code));
            }

            #[cfg(not(feature = "graph"))]
            {
                println!("hlbc-cli has been built without graph support. Build with feature 'graph' to enable callgraph generation");
            }
        }
        Command::RefTo(elem) => match elem {
            ElementRef::String(idx) => {
                println!(
                    "Finding references to string@{idx} : {}\n",
                    code.strings[idx]
                );
                if let Some(constants) = &code.constants {
                    for (i, c) in constants.iter().enumerate() {
                        if c.fields[0] == idx {
                            println!(
                                "constant@{i} expanding to global@{} (now also searching for global)",
                                c.global.0
                            );
                            iter_ops(code).for_each(|(f, (i, o))| match o {
                                Opcode::GetGlobal { global, .. } => {
                                    if *global == c.global {
                                        println!("in {} at {i}: GetGlobal", f.display_header(code));
                                    }
                                }
                                _ => {}
                            });
                            println!();
                        }
                    }
                }
                iter_ops(code).for_each(|(f, (i, o))| match o {
                    Opcode::String { ptr, .. } => {
                        if ptr.0 == idx {
                            println!("{} at {i}: String", f.display_header(code));
                        }
                    }
                    _ => {}
                });
            }
            ElementRef::Global(idx) => {
                println!(
                    "Finding references to global@{idx} : {}\n",
                    code.globals[idx].display(code)
                );
                if let Some(constants) = &code.constants {
                    for (i, c) in constants.iter().enumerate() {
                        if c.global.0 == idx {
                            println!("constant@{i} : {:?}", c);
                        }
                    }
                }
                println!();

                iter_ops(code).for_each(|(f, (i, o))| match o {
                    Opcode::GetGlobal { global, .. } | Opcode::SetGlobal { global, .. } => {
                        if global.0 == idx {
                            println!("{} at {i}: {}", f.display_header(code), o.name());
                        }
                    }
                    _ => {}
                });
            }
            ElementRef::Fn(idx) => {
                println!(
                    "Finding references to fn@{idx} : {}\n",
                    RefFun(idx).display_header(code)
                );
                code.functions
                    .iter()
                    .flat_map(|f| repeat(f).zip(find_fun_refs(f)))
                    .for_each(|(f, (i, o, fun))| {
                        if fun.0 == idx {
                            println!("{} at {i}: {}", f.display_header(code), o.name());
                        }
                    });
            }
        },
        Command::DumpType(idx) => {
            let ty = &code.types[idx];
            match ty {
                Type::Obj(obj) => {
                    println!("Dumping type@{idx} : {}", ty.display(code));
                    println!("{}", decompiler::decompile_class(code, obj));
                }
                _ => println!("Type {idx} is not an obj"),
            }
        }
    }
    Ok(())
}
