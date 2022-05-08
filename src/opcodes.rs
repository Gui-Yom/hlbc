use std::fmt::{Debug, Display, Formatter};

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Reg(u32);

impl Display for Reg {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "reg{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ConstInt(usize);

impl Display for ConstInt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "int@{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ConstFloat(usize);

impl Display for ConstFloat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "float@{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ConstBytes(usize);

impl Display for ConstBytes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "bytes@{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ConstString(usize);

impl Display for ConstString {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "string@{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ConstType(usize);

impl Display for ConstType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "type@{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ValBool(bool);

impl Display for ValBool {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Fun(usize);

impl Display for Fun {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "fn _@{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Field(usize);

impl Display for Field {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "field{}", self.0)
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Global(usize);

impl Display for Global {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "global@{}", self.0)
    }
}

static OPCODE_ARGS: &[i8; 99] = &[
    2, 2, 2, 2, 2, 2, 1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 1, 1, 2, 3, 4, 5, 6, -1, -1,
    -1, -1, 2, 3, 3, 2, 2, 3, 3, 2, 2, 3, 3, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 1, 2, 2, 2,
    2, 2, 2, 2, 0, 1, 1, 1, -1, 1, 2, 1, 3, 3, 3, 3, 3, 3, 3, 3, 1, 2, 2, 2, 2, 2, 2, 2, -1, 2, 2,
    4, 3, 0, 2, 3, 0,
];

pub type JumpOffset = i32;

#[hlbc_macros::gen_decode]
#[derive(Debug, Clone, strum_macros::IntoStaticStr)]
pub enum Opcode {
    Mov {
        dst: Reg,
        src: Reg,
    },
    Int {
        dst: Reg,
        ptr: ConstInt,
    },
    Float {
        dst: Reg,
        ptr: ConstFloat,
    },
    Bool {
        dst: Reg,
        value: ValBool,
    },
    Bytes {
        dst: Reg,
        ptr: ConstBytes,
    },
    String {
        dst: Reg,
        ptr: ConstString,
    },
    Null {
        dst: Reg,
    },
    Add {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    Sub {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    Mul {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    SDiv {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    UDiv {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    SMod {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    UMod {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    Shl {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    SShr {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    UShr {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    And {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    Or {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    Xor {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    Neg {
        dst: Reg,
        src: Reg,
    },
    Not {
        dst: Reg,
        src: Reg,
    },
    Incr {
        dst: Reg,
    },
    Decr {
        dst: Reg,
    },
    Call0 {
        dst: Reg,
        fun: Fun,
    },
    Call1 {
        dst: Reg,
        fun: Fun,
        arg0: Reg,
    },
    Call2 {
        dst: Reg,
        fun: Fun,
        arg0: Reg,
        arg1: Reg,
    },
    Call3 {
        dst: Reg,
        fun: Fun,
        arg0: Reg,
        arg1: Reg,
        arg2: Reg,
    },
    Call4 {
        dst: Reg,
        fun: Fun,
        arg0: Reg,
        arg1: Reg,
        arg2: Reg,
        arg3: Reg,
    },
    CallN {
        dst: Reg,
        fun: Fun,
        args: Vec<Reg>,
    },
    CallMethod {
        dst: Reg,
        obj: Reg,
        field: Reg,
        args: Vec<Reg>,
    },
    // Equivalent to CallMethod with obj = reg0
    CallThis {
        dst: Reg,
        field: Reg,
        args: Vec<Reg>,
    },
    CallClosure {
        dst: Reg,
        fun: Reg,
        args: Vec<Reg>,
    },
    StaticClosure {
        dst: Reg,
        fun: Fun,
    },
    InstanceClosure {
        dst: Reg,
        fun: Fun,
        obj: Reg,
    },
    VirtualClosure {
        dst: Reg,
        obj: Reg,
        field: Reg,
    },
    GetGlobal {
        dst: Reg,
        global: Global,
    },
    SetGlobal {
        global: Global,
        src: Reg,
    },
    Field {
        dst: Reg,
        obj: Reg,
        field: Field,
    },
    SetField {
        obj: Reg,
        field: Field,
        src: Reg,
    },
    // Equivalent to Field with obj = reg0
    GetThis {
        dst: Reg,
        field: Field,
    },
    SetThis {
        field: Field,
        src: Reg,
    },
    DynGet {
        dst: Reg,
        obj: Reg,
        field: Reg,
    },
    DynSet {
        obj: Reg,
        field: Reg,
        src: Reg,
    },
    JTrue {
        cond: Reg,
        offset: JumpOffset,
    },
    JFalse {
        cond: Reg,
        offset: JumpOffset,
    },
    JNull {
        reg: Reg,
        offset: JumpOffset,
    },
    JNotNull {
        reg: Reg,
        offset: JumpOffset,
    },
    JSLt {
        a: Reg,
        b: Reg,
        offset: JumpOffset,
    },
    JSGte {
        a: Reg,
        b: Reg,
        offset: JumpOffset,
    },
    JSGt {
        a: Reg,
        b: Reg,
        offset: JumpOffset,
    },
    JSLte {
        a: Reg,
        b: Reg,
        offset: JumpOffset,
    },
    JULt {
        a: Reg,
        b: Reg,
        offset: JumpOffset,
    },
    JUGte {
        a: Reg,
        b: Reg,
        offset: JumpOffset,
    },
    JNotLt {
        a: Reg,
        b: Reg,
        offset: JumpOffset,
    },
    JNotGte {
        a: Reg,
        b: Reg,
        offset: JumpOffset,
    },
    JEq {
        a: Reg,
        b: Reg,
        offset: JumpOffset,
    },
    JNotEq {
        a: Reg,
        b: Reg,
        offset: JumpOffset,
    },
    JAlways {
        offset: JumpOffset,
    },
    ToDyn {
        dst: Reg,
        src: Reg,
    },
    ToSFloat {
        dst: Reg,
        src: Reg,
    },
    ToUFloat {
        dst: Reg,
        src: Reg,
    },
    ToInt {
        dst: Reg,
        src: Reg,
    },
    SafeCast {
        dst: Reg,
        src: Reg,
    },
    UnsafeCast {
        dst: Reg,
        src: Reg,
    },
    ToVirtual {
        dst: Reg,
        src: Reg,
    },
    // Negative jump offsets must target a label
    Label,
    Ret {
        ret: Reg,
    },
    Throw {
        exc: Reg,
    },
    Rethrow {
        exc: Reg,
    },
    Switch {
        reg: Reg,
        offsets: Vec<JumpOffset>,
        end: JumpOffset,
    },
    NullCheck {
        reg: Reg,
    },
    Trap {
        exc: Reg,
        offset: JumpOffset,
    },
    EndTrap {
        exc: Reg,
    },
    GetI8 {
        dst: Reg,
        bytes: Reg,
        index: Reg,
    },
    GetI16 {
        dst: Reg,
        bytes: Reg,
        index: Reg,
    },
    GetMem {
        dst: Reg,
        bytes: Reg,
        index: Reg,
    },
    GetArray {
        dst: Reg,
        array: Reg,
        index: Reg,
    },
    SetI8 {
        bytes: Reg,
        index: Reg,
        src: Reg,
    },
    SetI16 {
        bytes: Reg,
        index: Reg,
        src: Reg,
    },
    SetMem {
        bytes: Reg,
        index: Reg,
        src: Reg,
    },
    SetArray {
        array: Reg,
        index: Reg,
        src: Reg,
    },
    New {
        dst: Reg,
    },
    ArraySize {
        dst: Reg,
        array: Reg,
    },
    Type {
        dst: Reg,
        ty: ConstType,
    },
    GetType {
        dst: Reg,
        src: Reg,
    },
    GetTID {
        dst: Reg,
        src: Reg,
    },
    Ref {
        dst: Reg,
        src: Reg,
    },
    Unref {
        dst: Reg,
        src: Reg,
    },
    Setref {
        dst: Reg,
        value: Reg,
    },
    MakeEnum {
        dst: Reg,
        construct: usize,
        args: Vec<Reg>,
    },
    EnumAlloc {
        dst: Reg,
        construct: usize,
    },
    EnumIndex {
        dst: Reg,
        construct: Reg,
    },
    EnumField {
        dst: Reg,
        enum_: Reg,
        construct: usize,
        field: Field,
    },
    SetEnumField {
        enum_: Reg,
        field: Field,
        src: Reg,
    },
    Assert,
    RefData {
        dst: Reg,
        src: Reg,
    },
    RefOffset {
        dst: Reg,
        reg: Reg,
        offset: usize,
    },
    Nop,
}

impl Display for Opcode {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name: &'static str = self.into();
        match self {
            Opcode::Mov { dst, src } => write!(f, "{name} {dst} = {src}"),
            Opcode::Int { dst, ptr } => write!(f, "{name} {dst} = {ptr}"),
            other => Debug::fmt(other, f),
        }
    }
}
