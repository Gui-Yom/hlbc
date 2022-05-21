use std::iter::repeat;

use crate::{Bytecode, Function, Opcode, RefFun};

#[cfg(feature = "graph")]
pub mod graph;

pub fn iter_ops(code: &Bytecode) -> impl Iterator<Item = (&Function, (usize, &Opcode))> {
    code.functions
        .iter()
        .flat_map(|f| repeat(f).zip(f.ops.iter().enumerate()))
}

pub fn find_calls(f: &Function) -> impl Iterator<Item = RefFun> + '_ {
    f.ops.iter().filter_map(|o| match o {
        Opcode::Call0 { fun, .. } => Some(*fun),
        Opcode::Call1 { fun, .. } => Some(*fun),
        Opcode::Call2 { fun, .. } => Some(*fun),
        Opcode::Call3 { fun, .. } => Some(*fun),
        Opcode::Call4 { fun, .. } => Some(*fun),
        Opcode::CallN { fun, .. } => Some(*fun),
        _ => None,
    })
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
