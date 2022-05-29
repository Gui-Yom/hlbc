use crate::types::{
    RefBytes, RefEnumConstruct, RefField, RefFloat, RefFun, RefGlobal, RefInt, RefString, RefType,
    Reg, ValBool,
};

/*
static OPCODE_ARGS: &[i8; 99] = &[
    2, 2, 2, 2, 2, 2, 1, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 1, 1, 2, 3, 4, 5, 6, -1, -1,
    -1, -1, 2, 3, 3, 2, 2, 3, 3, 2, 2, 3, 3, 2, 2, 2, 2, 3, 3, 3, 3, 3, 3, 3, 3, 3, 3, 1, 2, 2, 2,
    2, 2, 2, 2, 0, 1, 1, 1, -1, 1, 2, 1, 3, 3, 3, 3, 3, 3, 3, 3, 1, 2, 2, 2, 2, 2, 2, 2, -1, 2, 2,
    4, 3, 0, 2, 3, 0,
];*/

/// Offset for jump instruction. Can be negative, indicating a backward jump.
pub type JumpOffset = i32;

/// Opcodes definitions. The fields are the opcode arguments.
/// The methods for this struct are generated through a macro because there is no way I would have written code for 98 opcodes.
#[derive(Debug, Clone, hlbc_derive::OpcodeHelper)]
pub enum Opcode {
    /// Copy value from *src* into *dst*
    Mov {
        dst: Reg,
        src: Reg,
    },
    /// Get an i32 from the constant pool
    Int {
        dst: Reg,
        ptr: RefInt,
    },
    // Get a f64 from the constant pool
    Float {
        dst: Reg,
        ptr: RefFloat,
    },
    /// Set a boolean value
    Bool {
        dst: Reg,
        value: ValBool,
    },
    Bytes {
        dst: Reg,
        ptr: RefBytes,
    },
    /// Get a string from the constant pool
    String {
        dst: Reg,
        ptr: RefString,
    },
    /// Set dst as null
    Null {
        dst: Reg,
    },
    /// Add *a* and *b* into *dst*
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
        fun: RefFun,
    },
    Call1 {
        dst: Reg,
        fun: RefFun,
        arg0: Reg,
    },
    Call2 {
        dst: Reg,
        fun: RefFun,
        arg0: Reg,
        arg1: Reg,
    },
    Call3 {
        dst: Reg,
        fun: RefFun,
        arg0: Reg,
        arg1: Reg,
        arg2: Reg,
    },
    Call4 {
        dst: Reg,
        fun: RefFun,
        arg0: Reg,
        arg1: Reg,
        arg2: Reg,
        arg3: Reg,
    },
    CallN {
        dst: Reg,
        fun: RefFun,
        args: Vec<Reg>,
    },
    CallMethod {
        dst: Reg,
        field: RefField,
        // obj is the first arg
        args: Vec<Reg>,
    },
    // Equivalent to CallMethod with obj = reg0
    CallThis {
        dst: Reg,
        field: RefField,
        args: Vec<Reg>,
    },
    CallClosure {
        dst: Reg,
        fun: Reg,
        args: Vec<Reg>,
    },
    StaticClosure {
        dst: Reg,
        fun: RefFun,
    },
    InstanceClosure {
        dst: Reg,
        fun: RefFun,
        obj: Reg,
    },
    VirtualClosure {
        dst: Reg,
        obj: Reg,
        field: Reg,
    },
    GetGlobal {
        dst: Reg,
        global: RefGlobal,
    },
    SetGlobal {
        global: RefGlobal,
        src: Reg,
    },
    Field {
        dst: Reg,
        obj: Reg,
        field: RefField,
    },
    SetField {
        obj: Reg,
        field: RefField,
        src: Reg,
    },
    // Equivalent to RefField with obj = reg0
    GetThis {
        dst: Reg,
        field: RefField,
    },
    SetThis {
        field: RefField,
        src: Reg,
    },
    DynGet {
        dst: Reg,
        obj: Reg,
        field: RefString,
    },
    DynSet {
        obj: Reg,
        field: RefString,
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
        ty: RefType,
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
        construct: RefEnumConstruct,
        args: Vec<Reg>,
    },
    EnumAlloc {
        dst: Reg,
        construct: RefEnumConstruct,
    },
    /// Get the enum value construct index (the enum tag)
    EnumIndex {
        dst: Reg,
        value: Reg,
    },
    EnumField {
        dst: Reg,
        value: Reg,
        construct: RefEnumConstruct,
        field: RefField,
    },
    SetEnumField {
        value: Reg,
        field: RefField,
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
        offset: Reg,
    },
    Nop,
}
