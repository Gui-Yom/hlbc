//! Utilities to inverse the bytecode graph.
//! The bytecode graph consists of bytecode elements that reference each other.
//! This module contains functions that traverse this graph in reverse to find
//! find where a bytecode element is used.

use std::ops::Index;

use crate::opcodes::Opcode;
use crate::types::{
    EnumConstruct, FunPtr, Function, ObjField, ObjProto, RefEnumConstruct, RefField, RefFun,
    RefString, RefType, Reg, Type, TypeFun, TypeObj,
};
use crate::Bytecode;

/// The different ways a function can be used
#[derive(Debug, Clone)]
pub enum UsageFun {
    /// Direct call
    Call(RefFun, usize),
    /// Closure assignment
    Closure(RefFun, usize),
    /// Method call
    MethodCall(RefFun, usize),
    /// Bound as method
    Proto(RefType, usize),
    /// Bound to a class field
    Binding(RefType, RefField),
}

/// The different ways a type can be used
#[derive(Debug, Clone)]
pub enum UsageType {
    /// Type used as argument of a function. RefType points to a TypeFun.
    Argument(RefType),
    /// Type used as return type. RefType points to a TypeFun.
    Return(RefType),
    /// Type used as a field type
    Field(RefType, usize),
    /// Type of enum variant field
    EnumVariant(RefType, RefEnumConstruct, usize),
    /// Type of a function
    Function(RefFun),
    /// Type of a function register
    Register(RefFun),
}

/// The different ways a string can be used
#[derive(Debug, Clone)]
pub enum UsageString {
    /// Name of type (Enum, Class)
    Type(RefType),
    /// Name of enum variant
    EnumVariant(RefType, RefEnumConstruct),
    /// Name of field (Virtual, Class)
    Field(RefType, usize),
    /// Name of method (Class)
    Proto(RefType, usize),
    /// Used as a code constant
    Code(RefFun, usize),
    /// Dyn obj access
    Dyn(RefFun, usize),
    /// Name of a native function
    NativeName(RefFun),
    /// Name of a native library
    NativeLib(RefFun),
}

#[derive(Debug, Clone, Default)]
pub struct FullUsageReport {
    pub types: Vec<Vec<UsageType>>,
    pub fun: Vec<Vec<UsageFun>>,
    pub strings: Vec<Vec<UsageString>>,
}

impl FullUsageReport {
    fn new(code: &Bytecode) -> Self {
        Self {
            types: vec![Vec::new(); code.types.len()],
            fun: vec![Vec::new(); code.findex_max()],
            strings: vec![Vec::new(); code.strings.len()],
        }
    }

    fn compute_usage_type_fun(&mut self, ref_type: RefType, fun: &TypeFun) {
        for arg in &fun.args {
            self.types[arg.0].push(UsageType::Argument(ref_type));
        }
        self.types[fun.ret.0].push(UsageType::Return(ref_type));
    }

    fn compute_usage_type_obj(&mut self, ref_type: RefType, obj: &TypeObj) {
        self.strings[obj.name.0].push(UsageString::Type(ref_type));
        for (i, &ObjField { t, name }) in obj.own_fields.iter().enumerate() {
            self.strings[name.0].push(UsageString::Field(ref_type, i));
            self.types[t.0].push(UsageType::Field(ref_type, i));
        }
        for (i, &ObjProto { name, findex, .. }) in obj.protos.iter().enumerate() {
            self.strings[name.0].push(UsageString::Proto(ref_type, i));
            self.fun[findex.0].push(UsageFun::Proto(ref_type, i));
        }
        for (&fi, &fun) in &obj.bindings {
            self.fun[fun.0].push(UsageFun::Binding(ref_type, fi));
        }
    }

    fn compute_usage_type(&mut self, code: &Bytecode, ref_type: RefType) {
        match &code[ref_type] {
            Type::Fun(fun) => {
                self.compute_usage_type_fun(ref_type, fun);
            }
            Type::Obj(obj) => {
                self.compute_usage_type_obj(ref_type, obj);
            }
            &Type::Ref(rt) => {
                self.compute_usage_type(code, rt);
            }
            Type::Virtual { fields } => {
                for (i, &ObjField { t, .. }) in fields.iter().enumerate() {
                    self.types[t.0].push(UsageType::Field(ref_type, i));
                }
            }
            Type::Abstract { name } => {
                self.strings[name.0].push(UsageString::Type(ref_type));
            }
            Type::Enum {
                name, constructs, ..
            } => {
                self.strings[name.0].push(UsageString::Type(ref_type));
                for (i, EnumConstruct { name, params }) in constructs.iter().enumerate() {
                    self.strings[name.0]
                        .push(UsageString::EnumVariant(ref_type, RefEnumConstruct(i)));
                    for (j, p) in params.iter().enumerate() {
                        self.types[p.0].push(UsageType::EnumVariant(
                            ref_type,
                            RefEnumConstruct(i),
                            j,
                        ));
                    }
                }
            }
            &Type::Null(rt) => {
                self.compute_usage_type(code, rt);
            }
            Type::Method(fun) => {
                self.compute_usage_type_fun(ref_type, fun);
            }
            Type::Struct(obj) => {
                self.compute_usage_type_obj(ref_type, obj);
            }
            &Type::Packed(rt) => {
                self.compute_usage_type(code, rt);
            }
            _ => {}
        }
    }

    fn compute_usage_fun(&mut self, code: &Bytecode, f: &Function) {
        self.types[f.t.0].push(UsageType::Function(f.findex));
        for reg in &f.regs {
            self.types[reg.0].push(UsageType::Register(f.findex));
        }
        for (i, op) in f.ops() {
            match op {
                // Calls
                Opcode::Call0 { fun, .. }
                | Opcode::Call1 { fun, .. }
                | Opcode::Call2 { fun, .. }
                | Opcode::Call3 { fun, .. }
                | Opcode::Call4 { fun, .. }
                | Opcode::CallN { fun, .. } => {
                    self.fun[fun.0].push(UsageFun::Call(f.findex, i));
                }
                Opcode::CallMethod { args, field, .. } => {
                    let target = f[args[0]].method(field.0, code).unwrap().findex;
                    self.fun[target.0].push(UsageFun::MethodCall(f.findex, i));
                }
                Opcode::CallThis { field, .. } => {
                    let target = f[Reg(0)].method(field.0, code).unwrap().findex;
                    self.fun[target.0].push(UsageFun::MethodCall(f.findex, i));
                }
                Opcode::StaticClosure { fun, .. } | Opcode::InstanceClosure { fun, .. } => {
                    self.fun[fun.0].push(UsageFun::Closure(f.findex, i));
                }

                // Constants
                Opcode::String { ptr, .. } => {
                    self.strings[ptr.0].push(UsageString::Code(f.findex, i));
                }
                Opcode::DynGet { field, .. } | Opcode::DynSet { field, .. } => {
                    self.strings[field.0].push(UsageString::Dyn(f.findex, i));
                }
                _ => {}
            }
        }
    }

    fn compute_usage_all(&mut self, code: &Bytecode) {
        // Look through all types
        for ref_ty in (0..code.types.len()).map(RefType) {
            self.compute_usage_type(code, ref_ty);
        }

        for f in code.functions() {
            match f {
                FunPtr::Fun(fun) => {
                    self.compute_usage_fun(code, fun);
                }
                FunPtr::Native(n) => {
                    self.strings[n.name.0].push(UsageString::NativeName(n.findex));
                    self.strings[n.lib.0].push(UsageString::NativeLib(n.findex));
                    self.types[n.t.0].push(UsageType::Function(n.findex));
                }
            }
        }
    }
}

impl Index<RefType> for FullUsageReport {
    type Output = [UsageType];

    fn index(&self, index: RefType) -> &Self::Output {
        self.types.index(index.0)
    }
}

impl Index<RefFun> for FullUsageReport {
    type Output = [UsageFun];

    fn index(&self, index: RefFun) -> &Self::Output {
        self.fun.index(index.0)
    }
}

impl Index<RefString> for FullUsageReport {
    type Output = [UsageString];

    fn index(&self, index: RefString) -> &Self::Output {
        self.strings.index(index.0)
    }
}

pub fn usage_report(code: &Bytecode) -> FullUsageReport {
    let mut report = FullUsageReport::new(code);
    report.compute_usage_all(code);
    report
}

#[cfg(test)]
mod tests {
    use crate::analysis::usage::FullUsageReport;
    use crate::Bytecode;

    #[test]
    fn list_fun() {
        let code = Bytecode::from_file("../../data/Empty.hl").unwrap();
        for (i, fun) in code.functions.iter().enumerate() {
            dbg!((i, fun.name(&code)));
        }
    }

    #[test]
    fn test() {
        let code = Bytecode::from_file("../../data/Empty.hl").unwrap();
        let mut usage = FullUsageReport::new(&code);
        usage.compute_usage_all(&code);
        dbg!(usage);
    }
}
