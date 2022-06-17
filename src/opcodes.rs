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
    /// Get a f64 from the constant pool
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
    /// Nullify a register
    ///
    /// *dst* = null
    Null {
        dst: Reg,
    },
    /// Add two numbers
    ///
    /// *dst* = *a* + *b*
    Add {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Subtracts two numbers
    ///
    /// *dst* = *a* - *b*
    Sub {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Multiply two numbers
    ///
    /// *dst* = *a* * *b*
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
    /// Negate a number
    ///
    /// *dst* = -*src*
    Neg {
        dst: Reg,
        src: Reg,
    },
    /// Invert a boolean value
    ///
    /// *dst* = !*src*
    Not {
        dst: Reg,
        src: Reg,
    },
    /// Increment a number
    ///
    /// *dst*++
    Incr {
        dst: Reg,
    },
    /// Decrement a number
    ///
    /// *dst*--
    Decr {
        dst: Reg,
    },
    /// Call a function with no argument
    ///
    /// *dst* = *fun*()
    Call0 {
        dst: Reg,
        fun: RefFun,
    },
    /// Call a function with one argument
    ///
    /// *dst* = *fun*(*arg0*)
    Call1 {
        dst: Reg,
        fun: RefFun,
        arg0: Reg,
    },
    /// Call a function with two arguments
    ///
    /// *dst* = *fun*(*arg0*, *arg1*)
    Call2 {
        dst: Reg,
        fun: RefFun,
        arg0: Reg,
        arg1: Reg,
    },
    /// Call a function with three arguments
    ///
    /// *dst* = *fun*(*arg0*, *arg1*, *arg2*)
    Call3 {
        dst: Reg,
        fun: RefFun,
        arg0: Reg,
        arg1: Reg,
        arg2: Reg,
    },
    /// Call a function with four arguments
    ///
    /// *dst* = *fun*(*arg0*, *arg1*, *arg2*, *arg3*)
    Call4 {
        dst: Reg,
        fun: RefFun,
        arg0: Reg,
        arg1: Reg,
        arg2: Reg,
        arg3: Reg,
    },
    /// Call a function with N arguments
    ///
    /// *dst* = *fun*(*arg0*, *arg1*, ...)
    CallN {
        dst: Reg,
        fun: RefFun,
        args: Vec<Reg>,
    },
    /// Call a function with N arguments, using the first argument as the receiver
    ///
    /// *dst* = *arg0*.*field*(*arg1*, *arg2*, ...)
    CallMethod {
        dst: Reg,
        field: RefField,
        // obj is the first arg
        args: Vec<Reg>,
    },
    /// Call a function with N arguments, the receiver is the first register of the parent function
    ///
    /// *dst* = *reg0*.*field*(*arg0*, *arg1*, ...)
    CallThis {
        dst: Reg,
        field: RefField,
        args: Vec<Reg>,
    },
    /// Call a closure with N arguments. Here *fun* is a register.
    ///
    /// *dst* = *fun*(*arg0*, *arg1*, ...)
    CallClosure {
        dst: Reg,
        fun: Reg,
        args: Vec<Reg>,
    },
    /// Create a closure from a function reference.
    ///
    /// *dst* = *fun*
    StaticClosure {
        dst: Reg,
        fun: RefFun,
    },
    /// Create a closure from an object method.
    ///
    /// *dst* = *obj*.*fun*
    InstanceClosure {
        dst: Reg,
        fun: RefFun,
        obj: Reg,
    },
    /// Create a closure from an object field.
    ///
    /// *dst* = *obj*.*field*
    VirtualClosure {
        dst: Reg,
        obj: Reg,
        field: Reg,
    },
    /// Get a global value.
    ///
    /// *dst* = *global*
    GetGlobal {
        dst: Reg,
        global: RefGlobal,
    },
    /// Set a global value.
    ///
    /// `global = src`
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
