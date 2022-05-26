use std::collections::HashMap;

use crate::{Bytecode, Opcode};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Reg(pub u32);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RefInt(pub usize);

impl RefInt {
    pub fn resolve(&self, ints: &[i32]) -> i32 {
        ints[self.0]
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RefFloat(pub usize);

impl RefFloat {
    pub fn resolve(&self, floats: &[f64]) -> f64 {
        floats[self.0]
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RefBytes(pub usize);

/// Reference to a string in the constant pool
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RefString(pub usize);

impl RefString {
    pub fn resolve<'a>(&self, strings: &'a [String]) -> &'a str {
        &strings[self.0]
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ValBool(pub bool);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RefGlobal(pub usize);

#[derive(Debug, Clone, PartialEq)]
pub struct ObjField {
    pub name: RefString,
    pub t: RefType,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct RefField(pub usize);

#[derive(Debug, Clone, PartialEq)]
pub struct ObjProto {
    pub name: RefString,
    pub findex: RefFun,
    pub pindex: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumConstruct {
    pub name: RefString,
    pub params: Vec<RefType>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RefEnumConstruct(pub usize);

// For Type::Fun and Type::Method
#[derive(Debug, Clone, PartialEq)]
pub struct TypeFun {
    pub args: Vec<RefType>,
    pub ret: RefType,
}

// For Type::Obj and Type::Struct
#[derive(Debug, Clone, PartialEq)]
pub struct TypeObj {
    pub name: RefString,
    pub super_: Option<RefType>,
    pub global: RefGlobal,
    /// Fields defined in this type
    pub own_fields: Vec<ObjField>,
    /// Including other fields in the hierarchy
    pub fields: Vec<ObjField>,
    pub protos: Vec<ObjProto>,
    pub bindings: HashMap<RefField, RefFun>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Void,
    UI8,
    UI16,
    I32,
    I64,
    F32,
    F64,
    Bool,
    Bytes,
    Dyn,
    Fun(TypeFun),
    Obj(TypeObj),
    Array,
    Type,
    Ref(RefType),
    Virtual {
        fields: Vec<ObjField>,
    },
    DynObj,
    Abstract {
        name: RefString,
    },
    Enum {
        name: RefString,
        global: RefGlobal,
        constructs: Vec<EnumConstruct>,
    },
    Null(RefType),
    Method(TypeFun),
    Struct(TypeObj),
}

impl Type {
    pub fn get_type_obj(&self) -> Option<&TypeObj> {
        match self {
            Type::Obj(obj) => Some(obj),
            Type::Struct(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn get_type_obj_mut(&mut self) -> Option<&mut TypeObj> {
        match self {
            Type::Obj(obj) => Some(obj),
            Type::Struct(obj) => Some(obj),
            _ => None,
        }
    }

    pub fn get_type_fun(&self) -> Option<&TypeFun> {
        match self {
            Type::Fun(fun) => Some(fun),
            Type::Method(fun) => Some(fun),
            _ => None,
        }
    }
}

/// Reference to a type in the constant pool
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RefType(pub usize);

impl RefType {
    pub fn resolve<'a>(&self, types: &'a [Type]) -> &'a Type {
        &types[self.0]
    }
}

#[derive(Debug, Clone)]
pub struct Native {
    pub name: RefString,
    pub lib: RefString,
    pub t: RefType,
    pub findex: RefFun,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: Option<RefString>,
    pub t: RefType,
    pub findex: RefFun,
    pub regs: Vec<RefType>,
    pub ops: Vec<Opcode>,
    pub debug_info: Option<Vec<(usize, usize)>>,
    pub assigns: Option<Vec<(RefString, usize)>>,
}

/// Reference to a function or a native in the constant pool (findex)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct RefFun(pub usize);

#[derive(Debug, Copy, Clone)]
pub enum RefFunPointee<'a> {
    Fun(&'a Function),
    Native(&'a Native),
}

impl RefFun {
    pub fn resolve<'a>(&self, bc: &'a Bytecode) -> Option<RefFunPointee<'a>> {
        if let Some(&(i, f)) = bc.findexes.get(self) {
            Some(if f {
                RefFunPointee::Fun(&bc.functions[i])
            } else {
                RefFunPointee::Native(&bc.natives[i])
            })
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConstantDef {
    pub global: RefGlobal,
    pub fields: Vec<usize>,
}
