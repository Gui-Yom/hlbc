use crate::types::{
    RefBytes, RefEnumConstruct, RefField, RefFloat, RefFun, RefGlobal, RefInt, RefString, RefType,
    Reg, ValBool,
};

/// Offset for a jump instruction. Can be negative, indicating a backward jump.
pub type JumpOffset = i32;

/// Opcodes definitions. The fields are the opcode arguments.
///
/// The methods for this struct are generated through a macro because there is no way I would have written code for 98
/// opcodes. The opcode name is directly derived from the variant name. The opcode description is derived from the doc
/// comment on each variant.
///
/// The order of opcodes here is important as it defines the number used for serialization.
#[derive(Debug, Clone, hlbc_derive::OpcodeHelper)]
pub enum Opcode {
    /// Copy value from *src* into *dst*
    ///
    /// `dst = src`
    Mov {
        dst: Reg,
        src: Reg,
    },
    /// Get an **i32** from the constant pool
    ///
    /// `dst = @ptr`
    Int {
        dst: Reg,
        ptr: RefInt,
    },
    /// Get a **f64** from the constant pool
    ///
    /// `dst = @ptr`
    Float {
        dst: Reg,
        ptr: RefFloat,
    },
    /// Set a **bool** value
    ///
    /// `dst = <true|false>`
    Bool {
        dst: Reg,
        value: ValBool,
    },
    /// Get a byte array from the constant pool
    ///
    /// `dst = @ptr`
    Bytes {
        dst: Reg,
        ptr: RefBytes,
    },
    /// Get a **string** from the constant pool
    ///
    /// `dst = @ptr`
    String {
        dst: Reg,
        ptr: RefString,
    },
    /// Nullify a register
    ///
    /// `dst = null`
    Null {
        dst: Reg,
    },
    /// Add two numbers
    ///
    /// `dst = a + b`
    Add {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Subtracts two numbers
    ///
    /// `dst = a - b`
    Sub {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Multiply two numbers
    ///
    /// `dst = a * b`
    Mul {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Signed division
    ///
    /// `dst = a / b`
    SDiv {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Unsigned division
    ///
    /// `dst = a / b`
    UDiv {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Signed modulo
    ///
    /// `dst = a % b`
    SMod {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Unsigned modulo
    ///
    /// `dst = a % b`
    UMod {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Shift bits left
    ///
    /// `dst = a << b`
    Shl {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Signed shift bits right
    ///
    /// `dst = a >> b`
    SShr {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Unsigned shift bits right
    ///
    /// `dst = a >>> b`
    UShr {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Logical and
    ///
    /// `dst = a & b`
    And {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Logical or
    ///
    /// `dst = a | b`
    Or {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Logical xor
    ///
    /// `dst = a ^ b`
    Xor {
        dst: Reg,
        a: Reg,
        b: Reg,
    },
    /// Negate a number
    ///
    /// `dst = -src`
    Neg {
        dst: Reg,
        src: Reg,
    },
    /// Invert a boolean value
    ///
    /// dst = !src`
    Not {
        dst: Reg,
        src: Reg,
    },
    /// Increment a number
    ///
    /// `dst++`
    Incr {
        dst: Reg,
    },
    /// Decrement a number
    ///
    /// `dst--`
    Decr {
        dst: Reg,
    },
    /// Call a function with no argument
    ///
    /// `dst = fun()`
    Call0 {
        dst: Reg,
        fun: RefFun,
    },
    /// Call a function with one argument
    ///
    /// `dst = fun(arg0)`
    Call1 {
        dst: Reg,
        fun: RefFun,
        arg0: Reg,
    },
    /// Call a function with two arguments
    ///
    /// `dst = fun(arg0, arg1)`
    Call2 {
        dst: Reg,
        fun: RefFun,
        arg0: Reg,
        arg1: Reg,
    },
    /// Call a function with three arguments
    ///
    /// `dst = fun(arg0, arg1, arg2)`
    Call3 {
        dst: Reg,
        fun: RefFun,
        arg0: Reg,
        arg1: Reg,
        arg2: Reg,
    },
    /// Call a function with four arguments
    ///
    /// `dst = fun(arg0, arg1, arg2, arg3)`
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
    /// `dst = fun(arg0, arg1, ...)`
    CallN {
        dst: Reg,
        fun: RefFun,
        args: Vec<Reg>,
    },
    /// Call a function with N arguments, using the first argument as the receiver
    ///
    /// `dst = arg0.field(arg1, arg2, ...)`
    CallMethod {
        dst: Reg,
        field: RefField,
        // obj is the first arg
        args: Vec<Reg>,
    },
    /// Call a function with N arguments.
    /// *this* = *reg0*.
    ///
    /// `dst = this.field(arg0, arg1, ...)`
    CallThis {
        dst: Reg,
        field: RefField,
        args: Vec<Reg>,
    },
    /// Call a closure with N arguments. Here *fun* is a register.
    ///
    /// `dst = fun(arg0, arg1, ...)`
    CallClosure {
        dst: Reg,
        fun: Reg,
        args: Vec<Reg>,
    },
    /// Create a closure from a function reference.
    ///
    /// `dst = fun`
    StaticClosure {
        dst: Reg,
        fun: RefFun,
    },
    /// Create a closure from an object method.
    ///
    /// `dst = obj.fun`
    InstanceClosure {
        dst: Reg,
        fun: RefFun,
        obj: Reg,
    },
    /// Create a closure from an object field.
    ///
    /// `dst = obj.field`
    VirtualClosure {
        dst: Reg,
        obj: Reg,
        field: Reg,
    },
    /// Get a global value.
    ///
    /// `dst = @global`
    GetGlobal {
        dst: Reg,
        global: RefGlobal,
    },
    /// Set a global value.
    ///
    /// `@global = src`
    SetGlobal {
        global: RefGlobal,
        src: Reg,
    },
    /// Access an object field
    ///
    /// `dst = obj.field`
    Field {
        dst: Reg,
        obj: Reg,
        field: RefField,
    },
    /// Set an object field
    ///
    /// `obj.field = src`
    SetField {
        obj: Reg,
        field: RefField,
        src: Reg,
    },
    /// Get a field from the *this* instance.
    /// *this* = *reg0*.
    ///
    /// `dst = this.field`
    GetThis {
        dst: Reg,
        field: RefField,
    },
    /// Set a field from the *this* instance.
    /// *this* = *reg0*.
    ///
    /// `dst = this.field`
    SetThis {
        field: RefField,
        src: Reg,
    },
    /// Access a field of a **dyn** instance by its name.
    ///
    /// `dst = obj[field]`
    DynGet {
        dst: Reg,
        obj: Reg,
        field: RefString,
    },
    /// Set a field of a **dyn** instance by its name.
    ///
    /// `obj[field] = src`
    DynSet {
        obj: Reg,
        field: RefString,
        src: Reg,
    },
    /// Jump by an offset if the condition is true
    ///
    /// `if cond jump by offset`
    JTrue {
        cond: Reg,
        offset: JumpOffset,
    },
    /// Jump by an offset if the condition is false
    ///
    /// `if !cond jump by offset`
    JFalse {
        cond: Reg,
        offset: JumpOffset,
    },
    /// Jump by an offset if the value is null
    ///
    /// `if reg == null jump by offset`
    JNull {
        reg: Reg,
        offset: JumpOffset,
    },
    /// Jump by an offset if the value is not null
    ///
    /// `if reg != null jump by offset`
    JNotNull {
        reg: Reg,
        offset: JumpOffset,
    },
    /// Jump by an offset if signed lesser than.
    ///
    /// `if a < b jump by offset`
    JSLt {
        a: Reg,
        b: Reg,
        offset: JumpOffset,
    },
    /// Jump by an offset if signed greater than or equal
    ///
    /// `if a >= b jump by offset`
    JSGte {
        a: Reg,
        b: Reg,
        offset: JumpOffset,
    },
    /// Jump by an offset if signed greater than
    ///
    /// `if a > b jump by offset`
    JSGt {
        a: Reg,
        b: Reg,
        offset: JumpOffset,
    },
    /// Jump by an offset if signed lesser than or equal
    ///
    /// `if a < b jump by offset`
    JSLte {
        a: Reg,
        b: Reg,
        offset: JumpOffset,
    },
    /// Jump by an offset if unsigned lesser than
    ///
    /// `if a < b jump by offset`
    JULt {
        a: Reg,
        b: Reg,
        offset: JumpOffset,
    },
    /// Jump by an offset if unsigned greater than or equal
    ///
    /// `if a >= b jump by offset`
    JUGte {
        a: Reg,
        b: Reg,
        offset: JumpOffset,
    },
    /// Jump by an offset if not lesser than
    ///
    /// `if !(a < b) jump by offset`
    JNotLt {
        a: Reg,
        b: Reg,
        offset: JumpOffset,
    },
    /// Jump by an offset if not greater than or equal
    ///
    /// `if !(a >= b) jump by offset`
    JNotGte {
        a: Reg,
        b: Reg,
        offset: JumpOffset,
    },
    /// Jump by an offset if equal
    ///
    /// `if a == b jump by offset`
    JEq {
        a: Reg,
        b: Reg,
        offset: JumpOffset,
    },
    /// Jump by an offset if not equal
    ///
    /// `if a != b jump by offset`
    JNotEq {
        a: Reg,
        b: Reg,
        offset: JumpOffset,
    },
    /// Jump by an offset unconditionally
    ///
    /// `jump by offset`
    JAlways {
        offset: JumpOffset,
    },
    /// Convert a value to a **dyn** value
    ///
    /// `dst = (dyn) src`
    ToDyn {
        dst: Reg,
        src: Reg,
    },
    /// Convert a value to a signed **float**
    ///
    /// `dst = (float) src`
    ToSFloat {
        dst: Reg,
        src: Reg,
    },
    /// Convert a value to an unsigned **float**
    ///
    /// `dst = (float) src`
    ToUFloat {
        dst: Reg,
        src: Reg,
    },
    /// Convert a value to an **int**
    ///
    /// `dst = (int) src`
    ToInt {
        dst: Reg,
        src: Reg,
    },
    /// Cast a value to another type. Throw an exception if the cast is invalid.
    ///
    /// `dst = (typeof dst) src`
    SafeCast {
        dst: Reg,
        src: Reg,
    },
    /// Cast a value to another type. Will not throw an exception. Might crash the program at a later point.
    ///
    /// `dst = (typeof dst) src`
    UnsafeCast {
        dst: Reg,
        src: Reg,
    },
    /// Convert a value to a **virtual** value
    ///
    /// `dst = (virtual) src`
    ToVirtual {
        dst: Reg,
        src: Reg,
    },
    /// No-op, mark a position as being the target of a backward jump. Corresponds to a loop.
    ///
    /// Negative jump offsets must always target a label.
    Label,
    /// Return a value from the current function
    ///
    /// `return ret`
    Ret {
        ret: Reg,
    },
    /// Throw an exception
    Throw {
        exc: Reg,
    },
    /// Rethrow an exception, without touching the exception stack trace.
    Rethrow {
        exc: Reg,
    },
    /// Select a jump offset based on the integer value. The offsets array is no bigger than 255.
    ///
    /// `jump by offsets[reg] else jump by end`
    Switch {
        reg: Reg,
        offsets: Vec<JumpOffset>,
        end: JumpOffset,
    },
    /// Throw an exception if *reg* is null.
    ///
    /// `if reg == null throw exception`
    NullCheck {
        reg: Reg,
    },
    /// Setup a try-catch block. If an exception occurs, store it in the given register and jump by an offset.
    Trap {
        exc: Reg,
        offset: JumpOffset,
    },
    /// End the **latest** trap section.
    EndTrap {
        exc: Reg,
    },
    /// Read an **i8** from a byte array.
    ///
    /// `dst = bytes[index]`
    GetI8 {
        dst: Reg,
        bytes: Reg,
        index: Reg,
    },
    /// Read an **i16** from a byte array.
    ///
    /// `dst = bytes[index]`
    GetI16 {
        dst: Reg,
        bytes: Reg,
        index: Reg,
    },
    /// Read memory directly.
    ///
    /// `dst = bytes[index]`
    GetMem {
        dst: Reg,
        bytes: Reg,
        index: Reg,
    },
    /// Get the value of an array at an index.
    ///
    /// `dst = array[index]`
    GetArray {
        dst: Reg,
        array: Reg,
        index: Reg,
    },
    /// Write an **i8** to a byte array.
    ///
    /// `bytes[index] = src`
    SetI8 {
        bytes: Reg,
        index: Reg,
        src: Reg,
    },
    /// Write an **i16** to a byte array.
    ///
    /// `bytes[index] = src`
    SetI16 {
        bytes: Reg,
        index: Reg,
        src: Reg,
    },
    /// Write to memory directly.
    ///
    /// `bytes[index] = src`
    SetMem {
        bytes: Reg,
        index: Reg,
        src: Reg,
    },
    /// Write a value in an array.
    ///
    /// `array[index] = src`
    SetArray {
        array: Reg,
        index: Reg,
        src: Reg,
    },
    /// Allocate an object.
    ///
    /// `dst = new (typeof dst)`
    New {
        dst: Reg,
    },
    /// Get the length of an array.
    ///
    /// `dst = len(array)`
    ArraySize {
        dst: Reg,
        array: Reg,
    },
    /// Get the type object from its identifier.
    ///
    /// `dst = type ty`
    Type {
        dst: Reg,
        ty: RefType,
    },
    /// Get the type object of a value.
    ///
    /// `dst = typeof src`
    GetType {
        dst: Reg,
        src: Reg,
    },
    /// Get the type kind identifier of a value. Useful for switch statements on types.
    ///
    /// `dst = typeof src`
    GetTID {
        dst: Reg,
        src: Reg,
    },
    /// Get a reference to a value.
    ///
    /// `dst = &src`
    Ref {
        dst: Reg,
        src: Reg,
    },
    /// Read a reference value.
    ///
    /// `dst = *src`
    Unref {
        dst: Reg,
        src: Reg,
    },
    /// Write into a reference value.
    ///
    /// `*dst = src`
    Setref {
        dst: Reg,
        value: Reg,
    },
    /// Create an enum variant.
    ///
    /// `dst = construct(args...)`
    MakeEnum {
        dst: Reg,
        construct: RefEnumConstruct,
        args: Vec<Reg>,
    },
    /// Create an enum variant using the default values.
    ///
    /// `dst = construct()`
    EnumAlloc {
        dst: Reg,
        construct: RefEnumConstruct,
    },
    /// Get the enum value variant index (the enum tag). Useful for switch statements.
    ///
    /// `dst = variantof value`
    EnumIndex {
        dst: Reg,
        value: Reg,
    },
    /// Access a field of an enum.
    ///
    /// `dst = (value as construct).field`
    EnumField {
        dst: Reg,
        value: Reg,
        construct: RefEnumConstruct,
        field: RefField,
    },
    /// Set a field of an enum. Uses the first enum variant.
    ///
    /// `value.field = src`
    SetEnumField {
        value: Reg,
        field: RefField,
        src: Reg,
    },
    /// Debug break, calls `hl_assert()` under the hood.
    Assert,
    // Not sure what those last 2 opcodes do.
    RefData {
        dst: Reg,
        src: Reg,
    },
    RefOffset {
        dst: Reg,
        reg: Reg,
        offset: Reg,
    },
    /// No-op, useful to mark removed opcodes without breaking jump offsets.
    Nop,
}

#[cfg(test)]
mod test {
    use crate::opcodes::Opcode;
    use crate::types::Reg;

    #[test]
    fn test_doc() {
        assert_eq!(
            "Copy value from *src* into *dst*\n`dst = src`",
            Opcode::Mov {
                dst: Reg(0),
                src: Reg(0),
            }
            .description()
        );
        assert_eq!(
            "Nullify a register\n`dst = null`",
            Opcode::Null { dst: Reg(0) }.description()
        );
    }
}
