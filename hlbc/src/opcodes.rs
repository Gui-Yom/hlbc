use crate::types::{
    RefBytes, RefEnumConstruct, RefField, RefFloat, RefFun, RefGlobal, RefInt, RefString, RefType,
    Reg, ValBool,
};

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
    /// Signed division
    SDiv {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Unsigned division
    UDiv {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Signed modulo
    SMod {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Unsigned modulo
    UMod {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Shift bits left
    Shl {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Signed shift bits right
    SShr {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Unsigned shift bits right
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
    /// Access an object field
    Field {
        dst: Reg,
        obj: Reg,
        field: RefField,
    },
    /// Set an object field
    SetField {
        obj: Reg,
        field: RefField,
        src: Reg,
    },
    /// Get a field from the *this* instance
    ///
    /// Equivalent to `Field dst reg0 field`
    GetThis {
        dst: Reg,
        field: RefField,
    },
    /// Set a field from the *this* instance
    ///
    /// Equivalent to `SetField reg0 field src`
    SetThis {
        field: RefField,
        src: Reg,
    },
    /// Access a field of a [crate::Type::Dyn] instance by its name.
    DynGet {
        dst: Reg,
        obj: Reg,
        field: RefString,
    },
    /// Set a field of a [crate::Type::Dyn] instance by its name.
    DynSet {
        obj: Reg,
        field: RefString,
        src: Reg,
    },
    /// Jump by an offset if the condition is true
    JTrue {
        cond: Reg,
        offset: JumpOffset,
    },
    /// Jump by an offset if the condition is false
    JFalse {
        cond: Reg,
        offset: JumpOffset,
    },
    /// Jump by an offset if the value is null
    JNull {
        reg: Reg,
        offset: JumpOffset,
    },
    /// Jump by an offset if the value is not null
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
    /// Jump by an offset unconditionally
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
    /// No-op, mark a position as being the target of a backward jump (for loops)
    ///
    /// Negative jump offsets must always target a label
    Label,
    /// Return a value from the current function
    Ret {
        ret: Reg,
    },
    /// Throw an exception
    Throw {
        exc: Reg,
    },
    Rethrow {
        exc: Reg,
    },
    /// Select a jump offset based on the integer value
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
    /// Allocate an object
    New {
        dst: Reg,
    },
    ArraySize {
        dst: Reg,
        array: Reg,
    },
    /// Get the type object from its identifier
    Type {
        dst: Reg,
        ty: RefType,
    },
    /// Get the type object of a value
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
    /// Allocate and initialize an enum variant
    MakeEnum {
        dst: Reg,
        construct: RefEnumConstruct,
        args: Vec<Reg>,
    },
    /// Allocate an enum variant, its fields are initialized to default values
    EnumAlloc {
        dst: Reg,
        construct: RefEnumConstruct,
    },
    /// Get the enum value construct index (the enum tag)
    ///
    /// Useful for [Opcode::Switch]
    EnumIndex {
        dst: Reg,
        value: Reg,
    },
    /// Access a field of an enum
    EnumField {
        dst: Reg,
        value: Reg,
        construct: RefEnumConstruct,
        field: RefField,
    },
    /// Set a field of an enum
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
    /// No-op
    Nop,
}
