use std::iter::repeat;

use hlbc::opcodes::Opcode;
use hlbc::types::{Function, RefFun};
use hlbc::Bytecode;

pub fn read_range(arg: &str, max_bound: usize) -> anyhow::Result<Box<dyn Iterator<Item = usize>>> {
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

pub fn iter_ops(code: &Bytecode) -> impl Iterator<Item = (&Function, (usize, &Opcode))> {
    code.functions
        .iter()
        .map(|f| repeat(f).zip(f.ops.iter().enumerate()))
        .flatten()
}

pub fn find_calls(f: &Function) -> impl Iterator<Item = RefFun> + '_ {
    f.ops.iter().filter_map(|o| match o {
        Opcode::Call0 { fun, .. } => Some(fun.clone()),
        Opcode::Call1 { fun, .. } => Some(fun.clone()),
        Opcode::Call2 { fun, .. } => Some(fun.clone()),
        Opcode::Call3 { fun, .. } => Some(fun.clone()),
        Opcode::Call4 { fun, .. } => Some(fun.clone()),
        Opcode::CallN { fun, .. } => Some(fun.clone()),
        _ => None,
    })
}

pub fn find_fun_refs(f: &Function) -> impl Iterator<Item = (usize, &Opcode, RefFun)> + '_ {
    f.ops.iter().enumerate().filter_map(|(i, o)| match o {
        Opcode::Call0 { fun, .. } => Some((i, o, fun.clone())),
        Opcode::Call1 { fun, .. } => Some((i, o, fun.clone())),
        Opcode::Call2 { fun, .. } => Some((i, o, fun.clone())),
        Opcode::Call3 { fun, .. } => Some((i, o, fun.clone())),
        Opcode::Call4 { fun, .. } => Some((i, o, fun.clone())),
        Opcode::CallN { fun, .. } => Some((i, o, fun.clone())),
        Opcode::StaticClosure { fun, .. } => Some((i, o, fun.clone())),
        Opcode::InstanceClosure { fun, .. } => Some((i, o, fun.clone())),
        _ => None,
    })
}
