use std::collections::HashMap;
use std::ops::Index;

use crate::{Bytecode, Opcode, Resolve, Str};

/// A register argument
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash)]
pub struct Reg(pub u32);

/// A reference to the i32 constant pool
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct RefInt(pub usize);

/// A reference to the f64 constant pool
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct RefFloat(pub usize);

/// A reference to the bytes constant pool
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct RefBytes(pub usize);

/// Reference to the string constant pool
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct RefString(pub usize);

impl RefString {
    /// If a [RefString] is null, it indicates an element has no name
    pub fn is_null(&self) -> bool {
        self.0 == 0
    }
}

/// An inline bool value
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default)]
pub struct ValBool(pub bool);

/// A reference to a global
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Hash)]
pub struct RefGlobal(pub usize);

/// An object field definition
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ObjField {
    /// Field name
    pub name: RefString,
    /// Field type
    pub t: RefType,
}

impl ObjField {
    pub fn name(&self, code: &Bytecode) -> Str {
        code.get(self.name)
    }
}

/// A reference to an object field
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct RefField(pub usize);

/// An object method definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjProto {
    /// Method name
    pub name: RefString,
    /// Function bound to this method
    pub findex: RefFun,
    /// Don't know what this is used for
    pub pindex: i32,
}

impl ObjProto {
    pub fn name(&self, code: &Bytecode) -> Str {
        code.get(self.name)
    }
}

/// An enum variant definition
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumConstruct {
    /// Variant name
    pub name: RefString,
    /// Variant fields types
    pub params: Vec<RefType>,
}

impl EnumConstruct {
    pub fn name(&self, code: &Bytecode) -> Str {
        code.get(self.name)
    }
}

/// A reference to an enum variant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RefEnumConstruct(pub usize);

/// Common type for [Type::Fun] and [Type::Method]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeFun {
    pub args: Vec<RefType>,
    pub ret: RefType,
}

/// Common type for [Type::Obj] and [Type::Struct]
#[derive(Debug, Clone, PartialEq, Eq)]
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

impl TypeObj {
    pub fn name(&self, code: &Bytecode) -> Str {
        code.get(self.name)
    }

    /// Get the static part of this class
    pub fn get_static_type<'a>(&self, ctx: &'a Bytecode) -> Option<&'a TypeObj> {
        if self.global.0 > 0 {
            ctx.globals[self.global.0 - 1].as_obj(ctx)
        } else {
            None
        }
    }
}

/// Type available in the hashlink type system. Every type is one of those.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    Packed(RefType),
}

impl Type {
    pub fn is_wrapper_type(&self) -> bool {
        matches!(self, Type::Ref(_) | Type::Null(_) | Type::Packed(_))
    }

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
    pub fn is_void(&self) -> bool {
        self.0 == 0
    }

    /// Some type references points to the base hashlink types that are always loaded in the same place.
    /// This is a bit risky to rely on this.
    pub fn is_known(&self) -> bool {
        self.0 <= 9 || self.0 == 11 || self.0 == 14
    }

    /// ### Panics
    /// Panics if `self.is_known() == false`
    pub fn to_known(&self) -> Type {
        match self.0 {
            0 => Type::Void,
            1 => Type::UI8,
            2 => Type::UI16,
            3 => Type::I32,
            4 => Type::I64,
            5 => Type::F32,
            6 => Type::F64,
            7 => Type::Bool,
            8 => Type::Type,
            9 => Type::Dyn,
            11 => Type::Array,
            14 => Type::Bytes,
            _ => {
                panic!("This not a known type")
            }
        }
    }

    pub fn as_fun<'a>(&self, ctx: &'a Bytecode) -> Option<&'a TypeFun> {
        match ctx.get(*self) {
            Type::Fun(fun) => Some(fun),
            Type::Method(fun) => Some(fun),
            _ => None,
        }
    }

    pub fn as_obj<'a>(&self, ctx: &'a Bytecode) -> Option<&'a TypeObj> {
        ctx.get(*self).get_type_obj()
    }

    pub fn field<'a>(&self, field: RefField, ctx: &'a Bytecode) -> Option<&'a ObjField> {
        self.as_obj(ctx).map(|obj| &obj.fields[field.0])
    }

    pub fn method<'a>(&self, meth: usize, ctx: &'a Bytecode) -> Option<&'a ObjProto> {
        self.as_obj(ctx).map(|obj| &obj.protos[meth])
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

impl Native {
    pub fn name(&self, code: &Bytecode) -> Str {
        code.get(self.name)
    }

    pub fn lib(&self, code: &Bytecode) -> Str {
        code.get(self.lib)
    }

    /// Get the native function signature type
    pub fn ty<'a>(&self, code: &'a Bytecode) -> &'a TypeFun {
        // Guaranteed to be a TypeFun
        self.t.as_fun(code).expect("Unknown type ?")
    }

    pub fn args<'a>(&self, code: &'a Bytecode) -> &'a [RefType] {
        &self.ty(code).args
    }

    pub fn ret<'a>(&self, code: &'a Bytecode) -> &'a Type {
        code.get(self.ty(code).ret)
    }
}

/// A function definition with its code.
#[derive(Debug, Clone)]
pub struct Function {
    /// Functions have no name per se, this is the name of the field or method they are attached to
    pub name: RefString,
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
        self[reg]
    }

    /// Convenience method to resolve the function name
    pub fn name(&self, code: &Bytecode) -> Str {
        code.get(self.name)
    }

    /// Get the function signature type
    pub fn ty<'a>(&self, code: &'a Bytecode) -> &'a TypeFun {
        // Guaranteed to be a TypeFun
        self.t.as_fun(code).expect("Unknown type ?")
    }

    /// Convenience method to resolve the function args
    pub fn args<'a>(&self, code: &'a Bytecode) -> &'a [RefType] {
        &self.ty(code).args
    }

    /// Convenience method to resolve the function return type
    pub fn ret<'a>(&self, code: &'a Bytecode) -> &'a Type {
        code.index(self.ty(code).ret)
    }

    /// Uses the assigns to find the name of an argument
    pub fn arg_name(&self, code: &Bytecode, pos: usize) -> Option<Str> {
        self.assigns.as_ref().and_then(|a| {
            a.iter()
                .filter(|&&(_, i)| i == 0)
                .enumerate()
                .find_map(|(j, &(s, _))| {
                    if j == pos {
                        Some(code[s].clone())
                    } else {
                        None
                    }
                })
        })
    }

    /// Uses the assigns to find the name of a variable
    pub fn var_name(&self, code: &Bytecode, pos: usize) -> Option<Str> {
        self.assigns.as_ref().and_then(|a| {
            a.iter().find_map(|&(s, i)| {
                if pos + 1 == i {
                    Some(code[s].clone())
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

impl Index<Reg> for Function {
    type Output = RefType;

    /// Get the type of a register
    fn index(&self, index: Reg) -> &Self::Output {
        &self.regs[index.0 as usize]
    }
}

/// Index reference to a function or a native in the pool (findex)
#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd, Default)]
pub struct RefFun(pub usize);

impl RefFun {
    /// Useful when you already know you should be getting a Function
    pub fn as_fn<'a>(&self, code: &'a Bytecode) -> Option<&'a Function> {
        code.get(*self).as_fn()
    }

    pub fn name(&self, code: &Bytecode) -> Str {
        match code.get(*self) {
            FunPtr::Fun(fun) => fun.name(code),
            FunPtr::Native(n) => n.name(code),
        }
    }

    pub fn ty<'a>(&self, code: &'a Bytecode) -> &'a TypeFun {
        match code.get(*self) {
            FunPtr::Fun(fun) => fun.ty(code),
            FunPtr::Native(n) => n.ty(code),
        }
    }

    pub fn args<'a>(&self, code: &'a Bytecode) -> &'a [RefType] {
        &self.ty(code).args
    }

    pub fn ret<'a>(&self, code: &'a Bytecode) -> &'a Type {
        code.get(self.ty(code).ret)
    }
}

/// Reference to a function or a native object
#[derive(Debug, Copy, Clone)]
pub enum FunPtr<'a> {
    Fun(&'a Function),
    Native(&'a Native),
}

impl<'a> FunPtr<'a> {
    pub fn as_fn(self: FunPtr<'a>) -> Option<&'a Function> {
        match self {
            FunPtr::Fun(fun) => Some(fun),
            FunPtr::Native(_) => None,
        }
    }

    pub fn findex(&self) -> RefFun {
        match self {
            FunPtr::Fun(fun) => fun.findex,
            FunPtr::Native(n) => n.findex,
        }
    }

    pub fn name(&self, code: &Bytecode) -> Str {
        match *self {
            FunPtr::Fun(fun) => fun.name(code),
            FunPtr::Native(n) => n.name(code),
        }
    }

    pub fn is_fun(&self) -> bool {
        matches!(self, FunPtr::Fun(_))
    }

    pub fn is_native(&self) -> bool {
        matches!(self, FunPtr::Native(_))
    }
}

/// A constant definition
#[derive(Debug, Clone)]
pub struct ConstantDef {
    pub global: RefGlobal,
    pub fields: Vec<usize>,
}
