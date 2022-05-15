use std::io::{stdin, BufReader, Write};
use std::ops::RangeBounds;
use std::time::Instant;
use std::{env, fs};

use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

use hlbc::types::RefFun;
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
        let mut cmd = line.split(" ");
        match cmd.next().unwrap() {
            "info" => println!(
                "version: {}\ndebug: {}\nnints: {}\nnfloats: {}\nnstrings: {}\nntypes: {}\nnnatives: {}\nnfunctions: {}",
                code.version,
                code.debug_files.is_some(),
                code.ints.len(),
                code.floats.len(),
                code.strings.len(),
                code.types.len(),
                code.natives.len(),
                code.functions.len()
            ),
            "i" | "int" => {
                let range = read_range(&mut cmd, code.ints.len())?;
                for i in range {
                    print_i!(i);
                    println!("{}", code.ints[i]);
                }
            }
            "f" | "float" => {
                let range = read_range(&mut cmd, code.floats.len())?;
                for i in range {
                    print_i!(i);
                    println!("{}", code.floats[i]);
                }
            }
            "s" | "string" => {
                let range = read_range(&mut cmd, code.strings.len())?;
                for i in range {
                    print_i!(i);
                    println!("{}", code.strings[i]);
                }
            }
            "d" | "debugfile" => {
                if let Some(debug_files) = &code.debug_files {
                    let range = read_range(&mut cmd, debug_files.len())?;
                    for i in range {
                        print_i!(i);
                        println!("{}", debug_files[i]);
                    }
                } else {
                    println!("No debug info in this binary");
                }
            }
            "t" | "type" => {
                let range = read_range(&mut cmd, code.types.len())?;
                for i in range {
                    print_i!(i);
                    println!("{}", code.types[i].display(&code));
                }
            }
            "g" | "global" => {
                let range = read_range(&mut cmd, code.globals.len())?;
                for i in range {
                    print_i!(i);
                    println!("{}", code.globals[i].display(&code));
                }
            }
            "n" | "native" => {
                let range = read_range(&mut cmd, code.natives.len())?;
                for i in range {
                    print_i!(i);
                    println!("{}", code.natives[i].display(&code));
                }
            }
            "entrypoint" => {
                println!("{}", code.functions[code.findexes.get(&code.entrypoint).unwrap().0].display_header(&code));
            }
            "fnh" | "functionh" => {
                let range = read_range(&mut cmd, code.functions.len())?;
                for i in range {
                    print_i!(i);
                    println!("{}", code.functions[i].display_header(&code));
                }
            }
            // Function by index
            "fn" | "function" => {
                let range = read_range(&mut cmd, code.functions.len())?;
                for i in range {
                    print_i!(i);
                    println!("{}", code.functions[i].display(&code));
                }
            }
            "fih" | "findexh" => {
                let range = read_range(&mut cmd, code.max_findex + 1)?;
                for findex in range {
                    print_i!(findex);
                    if let Some(&(i, fun)) = code.findexes.get(&RefFun(findex)) {
                        if fun {
                            println!("{}", code.functions[i].display_header(&code));
                        } else {
                            println!("{}", code.natives[i].display(&code));
                        }
                    } else {
                        println!("unknown");
                    }
                }
            }
            "fi" | "findex" => {
                let range = read_range(&mut cmd, code.max_findex + 1)?;
                for findex in range {
                    print_i!(findex);
                    if let Some(&(i, fun)) = code.findexes.get(&RefFun(findex)) {
                        if fun {
                            println!("{}", code.functions[i].display(&code));
                        } else {
                            println!("{}", code.natives[i].display(&code));
                        }
                    } else {
                        println!("unknown");
                    }
                }
            }
            "fname" => {
                let name = cmd.next().unwrap();
                if let Some(&i) = code.fnames.get(name) {
                    println!("{}", code.functions[i].display(&code));
                } else {
                    println!("unknown");
                }
            }
            "infile" => {
                let name = cmd.next().unwrap();
                let fileidx = code.debug_files.as_ref().and_then(|files| files.iter().enumerate().find(|(i, s)| { s.as_str() == name }));
                if let Some((idx, _)) = fileidx {
                    println!("Finding functions in file index : {idx}");
                    for f in &code.functions {
                        if f.debug_info.as_ref().unwrap()[f.ops.len() - 1].0 == idx as i32 {
                            println!("{}", f.display_header(&code));
                        }
                    }
                } else {
                    println!("File {name} not found !");
                }
            }
            "c" | "constant" => {
                let range = read_range(&mut cmd, code.constants.as_ref().unwrap().len())?;
                //println!("Constants :");
                for i in range {
                    print_i!(i);
                    println!("{:#?}", code.constants.as_ref().unwrap()[i]);
                }
            }
            _ => {
                println!("Unknown command : '{line}'");
            }
        }
    }
    Ok(())
}

fn read_range<'a>(
    cmd: &mut impl Iterator<Item = &'a str>,
    max_bound: usize,
) -> anyhow::Result<Box<dyn Iterator<Item = usize>>> {
    if let Some(arg) = cmd.next() {
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
    } else {
        anyhow::bail!("Missing arg")
    }
}
