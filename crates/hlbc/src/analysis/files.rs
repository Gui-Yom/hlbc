use std::collections::HashMap;
use std::ops::Range;

use crate::types::{Function, RefFun};
use crate::{Bytecode, Str};

/// Finds the function which are present in a given file.
/// Only looks at the first instruction to determine the file.
///
/// Panics if no debug info is present.
pub fn functions_in_files(code: &Bytecode) -> HashMap<Str, Vec<RefFun>> {
    let df = code.debug_files.as_ref().unwrap();
    let mut funs = vec![Vec::new(); df.len()];
    for f in &code.functions {
        let file = f.debug_info.as_ref().unwrap()[0].0;
        funs[file].push(f.findex);
    }
    df.iter().cloned().zip(funs).collect()
}

/// Finds the files a function is present in. Usually there is only one except
/// if the function contains inlined code. The inlined code is still tagged with
/// the original function file.
///
/// Panics if no debug info.
pub fn files_in_function(code: &Bytecode, f: &Function) -> HashMap<Str, Vec<Range<usize>>> {
    let dbg = f.debug_info.as_ref().unwrap();
    let mut start = 0;
    let mut curr_file = dbg[0].0;
    let mut ranges = HashMap::<_, Vec<Range<usize>>>::new();
    for (i, &(file, _)) in dbg.iter().enumerate() {
        if file != curr_file {
            ranges
                .entry(code.debug_file(file).unwrap())
                .or_default()
                .push(start..i);
            start = i;
            curr_file = file;
        }
    }
    ranges
}

#[cfg(test)]
mod tests {
    use crate::analysis::files::{files_in_function, functions_in_files};
    use crate::Bytecode;

    #[test]
    fn test_files() {
        let code = Bytecode::from_file("../../data/Empty.hl").unwrap();
        let files = functions_in_files(&code);
        dbg!(files);
    }

    #[test]
    fn test_lines() {
        let code = Bytecode::from_file("../../data/Empty.hl").unwrap();
        let files = files_in_function(&code, code.entrypoint());
        dbg!(files);
    }
}
