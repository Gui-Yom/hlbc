use std::path::Path;
use std::{env, process};

fn main() {
    #[cfg(feature = "examples")]
    {
        let out = env::var_os("OUT_DIR").unwrap();
        let out = Path::new(&out);
        let examples = Path::new("examples");

        println!("Compiling examples ...");
        for example in ["Basic", "BranchExpr"] {
            let src_file = Path::new(example).with_extension("hx");
            let source = examples.join(example).with_extension("hx");
            let dest = out.join(example).with_extension("hl");
            let src_path = source.display();
            println!("cargo:rerun-if-changed={src_path}");

            assert!(process::Command::new("haxe")
                .current_dir(examples)
                .arg("-hl")
                .arg(&dest)
                .arg("-main")
                .arg(&src_file)
                .status()
                .unwrap()
                .success());
        }
    }
}
