use std::iter::repeat;

use crate::types::{RefFunPointee, Reg};
use crate::{Bytecode, Function, Opcode, RefFun};

#[cfg(feature = "graph")]
pub mod graph;

pub fn iter_ops(code: &Bytecode) -> impl Iterator<Item = (&Function, (usize, &Opcode))> {
    code.functions
        .iter()
        .flat_map(|f| repeat(f).zip(f.ops.iter().enumerate()))
}

pub fn find_fun_refs(f: &Function) -> impl Iterator<Item = (usize, &Opcode, RefFun)> + '_ {
    f.ops.iter().enumerate().filter_map(|(i, o)| match o {
        Opcode::Call0 { fun, .. } => Some((i, o, *fun)),
        Opcode::Call1 { fun, .. } => Some((i, o, *fun)),
        Opcode::Call2 { fun, .. } => Some((i, o, *fun)),
        Opcode::Call3 { fun, .. } => Some((i, o, *fun)),
        Opcode::Call4 { fun, .. } => Some((i, o, *fun)),
        Opcode::CallN { fun, .. } => Some((i, o, *fun)),
        Opcode::StaticClosure { fun, .. } => Some((i, o, *fun)),
        Opcode::InstanceClosure { fun, .. } => Some((i, o, *fun)),
        _ => None,
    })
}

// Starting from a position in a function, finds the last time a register has been assigned a function
pub fn find_last_closure_assign(
    code: &Bytecode,
    f: &Function,
    reg: Reg,
    pos: usize,
) -> Option<RefFun> {
    f.ops
        .iter()
        .rev()
        .skip(f.ops.len() - pos)
        .find_map(|o| match o {
            Opcode::StaticClosure { dst, fun } if *dst == reg => Some(*fun),
            Opcode::InstanceClosure { dst, fun, .. } if *dst == reg => Some(*fun),
            Opcode::Field { dst, obj, field } if *dst == reg => f.regs[obj.0 as usize]
                .resolve(&code.types)
                .get_type_obj()
                .map(|o| o.bindings.get(field).copied())
                .flatten(),
            _ => None,
        })
}

pub fn is_std_fn(code: &Bytecode, f: RefFun) -> bool {
    match f.resolve(code).unwrap() {
        RefFunPointee::Fun(fun) => {
            if let Some(debug_info) = &fun.debug_info {
                // We look at the Ret opcode which is probably not from inlined code.
                let (file, _) = debug_info[fun.ops.len() - 1];
                let filename = &code.debug_files.as_ref().unwrap()[file];
                filename.contains("std")
            } else {
                false
            }
        }
        RefFunPointee::Native(n) => n.lib.resolve(&code.strings) == "std",
    }
}
