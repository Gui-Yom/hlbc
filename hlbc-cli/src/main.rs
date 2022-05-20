use std::io::{stdin, BufReader, BufWriter, Write};
use std::time::Instant;
use std::{env, fs};

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use hlbc::opcodes::Opcode;
use hlbc::types::{Function, RefFun, RefFunPointee};
use hlbc::*;

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

    macro_rules! print_i {
        ($i:expr) => {
            stdout.set_color(ColorSpec::new().set_fg(Some(Color::Ansi256(242))))?;
            write!(&mut stdout, "{:<3}: ", $i)?;
            stdout.reset()?;
        };
    }

    loop {
        let mut line = String::new();
        println!();
        print!("> ");
        stdout.flush()?;
        stdin().read_line(&mut line)?;
        let line = line.trim();

        if line == "exit" {
            break;
        }
        let mut split = line.split_once(" ");
        if let Some((cmd, args)) = split {
            match cmd {
                "i" | "int" => {
                    let range = read_range(args, code.ints.len())?;
                    for i in range {
                        print_i!(i);
                        println!("{}", code.ints[i]);
                    }
                }
                "f" | "float" => {
                    let range = read_range(args, code.floats.len())?;
                    for i in range {
                        print_i!(i);
                        println!("{}", code.floats[i]);
                    }
                }
                "s" | "string" => {
                    let range = read_range(args, code.strings.len())?;
                    for i in range {
                        print_i!(i);
                        println!("{}", code.strings[i]);
                    }
                }
                "fstr" => {
                    for (i, s) in code.strings.iter().enumerate() {
                        if s.contains(args) {
                            print_i!(i);
                            println!("{}", s);
                        }
                    }
                }
                "d" | "debugfile" => {
                    if let Some(debug_files) = &code.debug_files {
                        let range = read_range(args, debug_files.len())?;
                        for i in range {
                            print_i!(i);
                            println!("{}", debug_files[i]);
                        }
                    } else {
                        println!("No debug info in this binary");
                    }
                }
                "ffile" => {
                    if let Some(debug_files) = &code.debug_files {
                        for (i, s) in debug_files.iter().enumerate() {
                            if s.contains(args) {
                                print_i!(i);
                                println!("{}", s);
                            }
                        }
                    } else {
                        println!("No debug info in this binary");
                    }
                }
                "t" | "type" => {
                    let range = read_range(args, code.types.len())?;
                    for i in range {
                        print_i!(i);
                        println!("{}", code.types[i].display(&code));
                    }
                }
                "td" | "typed" => {
                    let range = read_range(args, code.types.len())?;
                    for i in range {
                        print_i!(i);
                        println!("{:#?}", code.types[i]);
                    }
                }
                "g" | "global" => {
                    let range = read_range(args, code.globals.len())?;
                    for i in range {
                        print_i!(i);
                        println!("{}", code.globals[i].display(&code));
                    }
                }
                "n" | "native" => {
                    let range = read_range(args, code.natives.len())?;
                    for i in range {
                        print_i!(i);
                        println!("{}", code.natives[i].display_header(&code));
                    }
                }
                "fnh" | "functionh" => {
                    let range = read_range(args, code.functions.len())?;
                    for i in range {
                        print_i!(i);
                        println!("{}", code.functions[i].display_header(&code));
                    }
                }
                // Function by index
                "fn" | "function" => {
                    let range = read_range(args, code.functions.len())?;
                    for i in range {
                        print_i!(i);
                        println!("{}", code.functions[i].display(&code));
                    }
                }
                "fih" | "findexh" => {
                    let range = read_range(args, code.max_findex + 1)?;
                    for findex in range {
                        print_i!(findex);
                        if let Some(&(i, fun)) = code.findexes.get(&RefFun(findex)) {
                            if fun {
                                println!("{}", code.functions[i].display_header(&code));
                            } else {
                                println!("{}", code.natives[i].display_header(&code));
                            }
                        } else {
                            println!("unknown");
                        }
                    }
                }
                "fi" | "findex" => {
                    let range = read_range(args, code.max_findex + 1)?;
                    for findex in range {
                        print_i!(findex);
                        if let Some(&(i, fun)) = code.findexes.get(&RefFun(findex)) {
                            if fun {
                                println!("{}", code.functions[i].display(&code));
                            } else {
                                println!("{}", code.natives[i].display_header(&code));
                            }
                        } else {
                            println!("unknown");
                        }
                    }
                }
                "fname" => {
                    if let Some(&i) = code.fnames.get(args) {
                        println!("{}", code.functions[i].display(&code));
                    } else {
                        println!("unknown");
                    }
                }
                "c" | "constant" => {
                    let range = read_range(args, code.constants.as_ref().unwrap().len())?;
                    for i in range {
                        print_i!(i);
                        println!("{:#?}", code.constants.as_ref().unwrap()[i]);
                    }
                }
                "infile" => {
                    if let Some(debug_files) = &code.debug_files {
                        let fileidx = if args.contains("@") {
                            if let Some(idx) = args[1..].parse::<usize>().ok() {
                                Some((idx, &debug_files[idx]))
                            } else {
                                println!("Expected a number after @");
                                None
                            }
                        } else {
                            debug_files.iter().enumerate().find(|(i, d)| *d == args)
                        };
                        if let Some((idx, d)) = fileidx {
                            println!("Functions in file@{idx} : {d}");
                            for (i, f) in code.functions.iter().enumerate() {
                                if f.debug_info.as_ref().unwrap()[f.ops.len() - 1].0 == idx {
                                    print_i!(i);
                                    println!("{}", f.display_header(&code));
                                }
                            }
                        } else {
                            println!("File {args} not found !");
                        }
                    } else {
                        println!("No debug info in this binary");
                    }
                }
                "fileof" => {
                    if let Some(debug_files) = &code.debug_files {
                        if let Some(findex) = args.parse::<usize>().ok() {
                            let p = RefFun(findex).resolve(&code);
                            match p {
                                RefFunPointee::Fun(f) => {
                                    let idx = f.debug_info.as_ref().unwrap()[f.ops.len() - 1].0;
                                    println!(
                                        "{} is in file@{idx} : {}",
                                        f.display_header(&code),
                                        &debug_files[idx]
                                    );
                                }
                                RefFunPointee::Native(n) => {
                                    println!("{} can't be in any file", n.display_header(&code))
                                }
                            }
                        } else {
                            println!("Expected a number after @");
                            continue;
                        }
                    } else {
                        println!("No debug info in this binary");
                    }
                }
                // Find all functions referencing the given argument
                "refto" => {
                    let args: Vec<String> = args.split("@").map(|s| s.to_string()).collect();
                    let idx = args[1].parse::<usize>()?;
                    match args[0].as_str() {
                        "string" => {
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
                                        match_op(&code, |f, i, o| match o {
                                            Opcode::GetGlobal { global, .. } => {
                                                if global == c.global {
                                                    println!(
                                                        "in {} at {i}: GetGlobal",
                                                        f.display_header(&code)
                                                    );
                                                }
                                            }
                                            _ => {}
                                        });
                                        println!();
                                    }
                                }
                            }
                            match_op(&code, |f, i, o| match o {
                                Opcode::String { ptr, .. } => {
                                    if ptr.0 == idx {
                                        println!("{} at {i}: String", f.display_header(&code));
                                    }
                                }
                                _ => {}
                            });
                        }
                        "global" => {
                            println!(
                                "Finding references to global@{idx} : {}\n",
                                code.globals[idx].display(&code)
                            );
                            if let Some(constants) = &code.constants {
                                for (i, c) in constants.iter().enumerate() {
                                    if c.global.0 == idx {
                                        println!("constant@{i} : {:?}", c);
                                    }
                                }
                            }
                            println!();

                            match_op(&code, |f, i, o| match o {
                                Opcode::GetGlobal { global, .. } => {
                                    if global.0 == idx {
                                        println!("{} at {i}: GetGlobal", f.display_header(&code));
                                    }
                                }
                                Opcode::SetGlobal { global, .. } => {
                                    if global.0 == idx {
                                        println!("{} at {i}: SetGlobal", f.display_header(&code));
                                    }
                                }
                                _ => {}
                            });
                        }
                        "fi" => {
                            println!(
                                "Finding references to fn@{idx} : {}\n",
                                RefFun(idx).display_header(&code)
                            );
                            let test = |f: &Function, i: usize, o: Opcode, fun: RefFun| {
                                if fun.0 == idx {
                                    println!("{} at {i}: {}", f.display_header(&code), o.name());
                                }
                            };
                            match_op(&code, |f, i, o| match o {
                                Opcode::Call0 { fun, .. } => test(f, i, o, fun),
                                Opcode::Call1 { fun, .. } => test(f, i, o, fun),
                                Opcode::Call2 { fun, .. } => test(f, i, o, fun),
                                Opcode::Call3 { fun, .. } => test(f, i, o, fun),
                                Opcode::Call4 { fun, .. } => test(f, i, o, fun),
                                Opcode::CallN { fun, .. } => test(f, i, o, fun),
                                Opcode::StaticClosure { fun, .. } => test(f, i, o, fun),
                                _ => {}
                            });
                        }
                        _ => {}
                    }
                }
                "saveto" => {
                    let mut w = BufWriter::new(fs::File::create(args)?);
                    code.serialize(&mut w)?;
                }
                _ => {
                    println!("Unknown command : '{line}'");
                }
            }
        } else {
            match line {
                "help" => println!(r#"Commands :
info                   | General information about the bytecode
help                   | This message
entrypoint             | Get the bytecode entrypoint
i,int       <idx>      | Get the int at index
f,float     <idx>      | Get the float at index
s,string    <idx>      | Get the string at index
fstr        <str>      | Find a string
d,debugfile <idx>      | Get the debug file name at index
ffile       <str>      | Find the debug file named
t,type      <idx>      | Get the type at index
td,typed    <idx>      | Get full information of type at index
g,global    <idx>      | Get global at index
c,constant  <idx>      | Get constant at index
n,native    <idx>      | Get native at index
fnh         <idx>      | Get header of function at index
fn          <idx>      | Get function at index
fih         <findex>   | Get header of function (findex)
fi          <findex>   | Get function at index (findex)
fname       <str>      | Get function named
infile      <@idx|str> | Find functions in file
fileof      <findex>   | Get the file where findex is defined
refto       <any@idx>  | Find references to a given bytecode element
saveto      <filename> | Serialize the bytecode to a file
                "#),
                "info" => println!(
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
                ),
                "entrypoint" => {
                    println!(
                        "{}",
                        code.functions[code.findexes.get(&code.entrypoint).unwrap().0]
                            .display_header(&code)
                    );
                }
                _ => {
                    println!("Unknown command or missing argument : '{line}'");
                }
            }
        }
    }
    Ok(())
}

fn read_range(arg: &str, max_bound: usize) -> anyhow::Result<Box<dyn Iterator<Item = usize>>> {
    if arg == ".." {
        Ok(Box::new((0..max_bound).into_iter()))
    } else if arg.contains("..=") {
        let mut nums = arg.split("..=");
        if let Some(a) = nums.next() {
            if let Some(b) = nums.next() {
                Ok(Box::new((a.parse()?..=b.parse()?).into_iter()))
            } else if arg.ends_with(a) {
                Ok(Box::new((0..=a.parse()?).into_iter()))
            } else {
                anyhow::bail!("Inclusive range must be bounded at the end : '{arg}'")
            }
        } else {
            anyhow::bail!("Inclusive range must be bounded at the end : '{arg}'")
        }
    } else if arg.contains("..") {
        let mut nums = arg.split("..");
        if let Some(a) = nums.next() {
            if a.is_empty() {
                if let Some(b) = nums.next() {
                    Ok(Box::new((0..b.parse()?).into_iter()))
                } else {
                    anyhow::bail!("Invalid range : '{arg}'")
                }
            } else {
                if let Some(b) = nums.next() {
                    if b.is_empty() {
                        Ok(Box::new((a.parse()?..max_bound - 1).into_iter()))
                    } else {
                        Ok(Box::new((a.parse()?..b.parse()?).into_iter()))
                    }
                } else if arg.ends_with(a) {
                    Ok(Box::new((0..a.parse()?).into_iter()))
                } else if arg.starts_with(a) {
                    Ok(Box::new((a.parse()?..max_bound - 1).into_iter()))
                } else {
                    anyhow::bail!("Invalid range : '{arg}'")
                }
            }
        } else {
            Ok(Box::new((0..max_bound - 1).into_iter()))
        }
    } else {
        let i = arg.parse()?;
        Ok(Box::new((i..(i + 1)).into_iter()))
    }
}

fn match_op(code: &Bytecode, body: impl Fn(&Function, usize, Opcode)) {
    for f in &code.functions {
        for (i, o) in f.ops.iter().enumerate() {
            body(f, i, o.clone());
        }
    }
}
