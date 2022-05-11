use crate::Opcode;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Reg(pub u32);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RefInt(pub usize);

impl RefInt {
    pub fn resolve<'a>(&self, ints: &'a [i32]) -> &'a i32 {
        &ints[self.0]
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RefFloat(pub usize);

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

/// Reference to a type in the constant pool
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RefType(pub usize);

impl RefType {
    pub fn resolve<'a>(&self, types: &'a [Type]) -> &'a Type {
        &types[self.0]
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ValBool(pub bool);

/// Reference to a function in the constant pool
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RefFun(pub usize);

impl RefFun {
    pub fn resolve<'a>(&self, functions: &'a [Function]) -> &'a Function {
        &functions[self.0]
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RefField(pub usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RefGlobal(pub usize);

#[derive(Debug, Clone)]
pub struct ObjField {
    pub name: RefString,
    pub t: RefType,
}

#[derive(Debug, Clone)]
pub struct ObjProto {
    pub name: RefString,
    pub findex: RefFun,
    pub pindex: usize,
}

#[derive(Debug, Clone)]
pub struct EnumConstruct {
    pub name: RefString,
    pub params: Vec<RefType>,
}

#[derive(Debug, Clone)]
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
    Fun {
        args: Vec<RefType>,
        ret: RefType,
    },
    Obj {
        name: RefString,
        super_: Option<RefType>,
        fields: Vec<ObjField>,
        protos: Vec<ObjProto>,
        bindings: Vec<(u32, u32)>,
    },
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
        constructs: Vec<EnumConstruct>,
    },
    Null(RefType),
    Method {
        args: Vec<RefType>,
        ret: RefType,
    },
    Struct {
        name: RefString,
        super_: Option<RefType>,
        fields: Vec<ObjField>,
        protos: Vec<ObjProto>,
        bindings: Vec<(u32, u32)>,
    },
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
    pub t: RefType,
    pub findex: RefFun,
    pub regs: Vec<RefType>,
    pub ops: Vec<Opcode>,
    pub debug_info: Option<Vec<(i32, i32)>>,
}

#[derive(Debug, Clone)]
pub struct ConstantDef {
    pub global: RefGlobal,
    pub fields: Vec<usize>,
}
