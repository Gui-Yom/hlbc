use std::ffi::c_void;
use std::fs;
use std::io::{stdin, stdout, BufReader, Write};
use std::os::raw::c_char;
use std::ptr::{null, null_mut};

use hlbc::*;

fn main() {
    let mut code = {
        let mut r = BufReader::new(
            fs::File::open("D:/ReverseEngineering/northgard/hlbc/hlboot2.dat").unwrap(),
        );
        HlCode::load(&mut r).unwrap()
    };

    //let m = HlModule::new(&mut code).unwrap();

    println!("Loaded !");

    /*
    for i in 0..code.nstrings() {
        println!("{:?}", code.strings(i as isize));
    }*/

    loop {
        let mut line = String::new();
        println!();
        print!("> ");
        stdout().flush();
        stdin().read_line(&mut line);
        let line = line.trim();

        if line == "exit" {
            break;
        }
        let mut cmd = line.split(" ");
        match cmd.next().unwrap() {
            "info" => unsafe {
                println!("{:#?}", code);
            },
            "i" | "int" => {
                let idx: usize = cmd
                    .next()
                    .expect("Expected index")
                    .parse()
                    .expect("Expected index number");
                println!("Int constant at index {idx} : {}", code.ints[idx]);
            }
            "f" | "float" => {
                let idx: usize = cmd
                    .next()
                    .expect("Expected index")
                    .parse()
                    .expect("Expected index number");
                println!("Float constant at index {idx} : {}", code.floats[idx]);
            }
            "s" | "string" => {
                let idx: usize = cmd
                    .next()
                    .expect("Expected index")
                    .parse()
                    .expect("Expected index number");
                println!("String constant at index {idx} : '{}'", code.strings[idx]);
            }
            "d" | "debugfile" => {
                let idx: usize = cmd
                    .next()
                    .expect("Expected index")
                    .parse()
                    .expect("Expected index number");
                println!(
                    "Debug file name at index {idx} : '{}'",
                    code.debug_files.as_ref().unwrap()[idx]
                );
            }
            "t" | "type" => {
                let idx: usize = cmd
                    .next()
                    .expect("Expected index")
                    .parse()
                    .expect("Expected index number");
                println!("Type constant at index {idx} : {:#?}", code.types[idx]);
            }
            "g" | "global" => {
                let idx: usize = cmd
                    .next()
                    .expect("Expected index")
                    .parse()
                    .expect("Expected index number");
                println!("Global at index {idx} : {:#?}", code.globals[idx]);
            }
            "n" | "native" => {
                let idx: usize = cmd
                    .next()
                    .expect("Expected index")
                    .parse()
                    .expect("Expected index number");
                println!("Native function at index {idx} : {:#?}", code.natives[idx]);
            }
            "fn" | "function" => {
                let idx: usize = cmd
                    .next()
                    .expect("Expected index")
                    .parse()
                    .expect("Expected index number");
                println!("Function at index {idx} : {:#?}", code.functions[idx]);
            }
            "c" | "constant" => {
                let idx: usize = cmd
                    .next()
                    .expect("Expected index")
                    .parse()
                    .expect("Expected index number");
                println!(
                    "Debug file name at index {idx} : {:#?}",
                    code.constants[idx]
                );
            }
            _ => {
                println!("Unknown command : '{line}'");
            }
        }
    }
}
