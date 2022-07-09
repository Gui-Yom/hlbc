use std::collections::HashMap;

use crate::{Bytecode, Opcode};

/// A register argument
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash)]
pub struct Reg(pub u32);

/// A reference to the i32 constant pool
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct RefInt(pub usize);

impl RefInt {
    pub fn resolve(&self, ints: &[i32]) -> i32 {
        ints[self.0]
    }
}

/// A reference to the f64 constant pool
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct RefFloat(pub usize);

impl RefFloat {
    pub fn resolve(&self, floats: &[f64]) -> f64 {
        floats[self.0]
    }
}

/// A reference to the bytes constant pool
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct RefBytes(pub usize);

/// Reference to the string constant pool
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct RefString(pub usize);

impl RefString {
    pub fn resolve<'a>(&self, strings: &'a [String]) -> &'a str {
        &strings[self.0]
    }
}

/// An inline bool value
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct ValBool(pub bool);

/// A reference to a global
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash)]
pub struct RefGlobal(pub usize);

/// An object field definition
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ObjField {
    /// Field name
    pub name: RefString,
    /// Field type
    pub t: RefType,
}

/// A reference to an object field
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct RefField(pub usize);

/// An object method definition
#[derive(Debug, Clone, PartialEq)]
pub struct ObjProto {
    /// Method name
    pub name: RefString,
    /// Function bound to this method
    pub findex: RefFun,
    /// Don't know what this is used for
    pub pindex: i32,
}

/// An enum variant definition
#[derive(Debug, Clone, PartialEq)]
pub struct EnumConstruct {
    /// Variant name, can be null (pointing to 0)
    // TODO wrap this in an option
    pub name: RefString,
    /// Variant fields types
    pub params: Vec<RefType>,
}

/// A reference to an enum variant
#[derive(Debug, Clone, PartialEq, Default)]
pub struct RefEnumConstruct(pub usize);

/// Common type for [Type::Fun] and [Type::Method]
#[derive(Debug, Clone, PartialEq)]
pub struct TypeFun {
    pub args: Vec<RefType>,
    pub ret: RefType,
}

/// Common type for [Type::Obj] and [Type::Struct]
#[derive(Debug, Clone, PartialEq)]
pub struct TypeObj {
    pub name: RefString,
    pub super_: Option<RefType>,
    pub global: RefGlobal,
    /// Fields defined in this type
    pub own_fields: Vec<ObjField>,
    /// Methods in this class
    pub protos: Vec<ObjProto>,
    /// Functions bounds to class fields
    pub bindings: HashMap<RefField, RefFun>,

    // Data below is not stored in the bytecode
    /// Fields including parents in the hierarchy
    pub fields: Vec<ObjField>,
}

/// Type available in the hashlink type system. Every type is one of those.
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
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct RefType(pub usize);

impl RefType {
    pub fn resolve<'a>(&self, types: &'a [Type]) -> &'a Type {
        &types[self.0]
    }

    pub fn is_void(&self) -> bool {
        self.0 == 0
    }

    pub fn resolve_as_fun<'a>(&self, types: &'a [Type]) -> Option<&'a TypeFun> {
        match self.resolve(types) {
            Type::Fun(fun) => Some(fun),
            Type::Method(fun) => Some(fun),
            _ => None,
        }
    }

    pub fn resolve_as_obj<'a>(&self, types: &'a [Type]) -> Option<&'a TypeObj> {
        self.resolve(types).get_type_obj()
    }

    pub fn field<'a>(&self, field: RefField, code: &'a Bytecode) -> Option<&'a ObjField> {
        self.resolve_as_obj(&code.types)
            .map(|obj| &obj.fields[field.0])
    }
}

/// A native function reference. Contains no code but indicates the library from where to load it.
#[derive(Debug, Clone)]
pub struct Native {
    /// Native function name
    pub name: RefString,
    /// Native lib name
    pub lib: RefString,
    pub t: RefType,
    pub findex: RefFun,
}

/// A function definition with its code.
#[derive(Debug, Clone)]
pub struct Function {
    pub name: Option<RefString>,
    pub t: RefType,
    pub findex: RefFun,
    /// The types of the registers used by this function
    pub regs: Vec<RefType>,
    /// Instructions
    pub ops: Vec<Opcode>,
    /// *Debug* File and line information for each instruction
    pub debug_info: Option<Vec<(usize, usize)>>,
    /// *Debug* Information about some variables names for some instructions
    pub assigns: Option<Vec<(RefString, usize)>>,

    // Fields below are not part of the bytecode
    /// Parent type (Obj/Struct) this function is a member of.
    /// This does not mean it's a method
    pub parent: Option<RefType>,
}

impl Function {
    /// Get the type of a register
    pub fn regtype(&self, reg: Reg) -> RefType {
        self.regs[reg.0 as usize]
    }

    /// Get the function signature type
    pub fn ty<'a>(&self, code: &'a Bytecode) -> &'a TypeFun {
        // Guaranteed to be a TypeFun
        self.t.resolve_as_fun(&code.types).expect("Unknown type ?")
    }

    /// Uses the assigns to find the name of an argument
    pub fn arg_name(&self, code: &Bytecode, pos: usize) -> Option<String> {
        self.assigns.as_ref().and_then(|a| {
            a.iter()
                .filter(|&&(_, i)| i == 0)
                .enumerate()
                .find_map(|(j, (s, _))| {
                    if j == pos {
                        Some(s.resolve(&code.strings).to_string())
                    } else {
                        None
                    }
                })
        })
    }

    /// Uses the assigns to find the name of a variable
    pub fn var_name(&self, code: &Bytecode, pos: usize) -> Option<String> {
        self.assigns.as_ref().and_then(|a| {
            a.iter().find_map(|&(s, i)| {
                if pos + 1 == i {
                    Some(s.resolve(&code.strings).to_owned())
                } else {
                    None
                }
            })
        })
    }

    /// A function is a method if the first argument has the same type as the parent type
    pub fn is_method(&self) -> bool {
        self.parent
            .map(|parent| !self.regs.is_empty() && self.regs[0] == parent)
            .unwrap_or(false)
    }
}

/// Reference to a function or a native in the pool (findex)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
pub struct RefFun(pub usize);

impl RefFun {
    pub fn resolve<'a>(&self, code: &'a Bytecode) -> FunPtr<'a> {
        code.findexes[self.0].resolve(code)
    }

    /// Useful when you already know you should be getting a Function
    pub fn resolve_as_fn<'a>(&self, code: &'a Bytecode) -> Option<&'a Function> {
        code.findexes[self.0].resolve_as_fn(code)
    }
}

// Reference to a function or a native in the pool, but we know what is is.
#[derive(Debug, Copy, Clone)]
pub enum RefFunKnown {
    Fun(usize),
    Native(usize),
}

impl RefFunKnown {
    pub fn resolve<'a>(&self, code: &'a Bytecode) -> FunPtr<'a> {
        match *self {
            RefFunKnown::Fun(x) => FunPtr::Fun(&code.functions[x]),
            RefFunKnown::Native(x) => FunPtr::Native(&code.natives[x]),
        }
    }

    pub fn resolve_as_fn<'a>(&self, code: &'a Bytecode) -> Option<&'a Function> {
        match self {
            &RefFunKnown::Fun(x) => Some(&code.functions[x]),
            _ => None,
        }
    }
}

/// The possible values behind a function reference
#[derive(Debug, Copy, Clone)]
pub enum FunPtr<'a> {
    Fun(&'a Function),
    Native(&'a Native),
}

/// A constant definition
#[derive(Debug, Clone)]
pub struct ConstantDef {
    pub global: RefGlobal,
    pub fields: Vec<usize>,
}
