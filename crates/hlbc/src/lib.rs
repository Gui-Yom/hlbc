//! [Hashlink](https://hashlink.haxe.org/) bytecode disassembler and analyzer.
//! See [Bytecode] for an entrypoint to the library.
//!
//! #### Note about safety
//! We don't deal with self-references, hence we deal with indexes into structures.
//! Be careful when calling functions on Ref* objects, as no bound checking is done and every index is assumed to be valid.

extern crate core;

use std::collections::HashMap;
use std::fmt::Debug;
use std::ops::Index;

use crate::opcodes::Opcode;
use crate::types::{
    ConstantDef, FunPtr, Function, Native, ObjField, RefFloat, RefFun, RefGlobal, RefInt,
    RefString, RefType, Type, TypeObj,
};

/// Analysis functions and callgraph generation
pub mod analysis;
pub mod fmt;
/// Opcodes definitions.
pub mod opcodes;
/// All about reading bytecode
mod read;
/// Bytecode elements definitions.
/// All the Ref* types in this modules are references to bytecode elements like constants or function.
/// They are required since we cannot use rust references as that would make our structure self-referential.
/// They makes the code look a bit more complicated than it actually is. Every Ref* struct is cheaply copyable.
pub mod types;
/// All about writing bytecode
mod write;

pub type Result<T> = core::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Malformed bytecode: {0}")]
    MalformedBytecode(String),
    #[error("Unsupported bytecode version {version} (expected {min} <= version <= {max})")]
    UnsupportedVersion { version: u8, min: u8, max: u8 },
    #[error("Value '{value}' is too big to be serialized (expected < {limit})")]
    ValueOutOfBounds { value: i32, limit: u32 },
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

/// Bytecode structure containing all the information.
/// Every field is public for flexibility, but you aren't encouraged to modify them.
///
/// This type is like an arena, you usually work with custom
#[derive(Debug)]
pub struct Bytecode {
    /// Bytecode format version
    pub version: u8,
    /// Program entrypoint
    pub entrypoint: RefFun,
    /// i32 constant pool
    pub ints: Vec<i32>,
    /// f64 constant pool
    pub floats: Vec<f64>,
    /// String constant pool
    pub strings: Vec<String>,
    /// Bytes constant pool
    ///
    /// *Since bytecode v5*
    pub bytes: Option<(Vec<u8>, Vec<usize>)>,
    /// *Debug* file names constant pool
    pub debug_files: Option<Vec<String>>,
    /// Types, contains every possible types expressed in the program
    pub types: Vec<Type>,
    /// Globals, holding static variables and such
    pub globals: Vec<RefType>,
    /// Native functions references
    pub natives: Vec<Native>,
    /// Code functions pool
    pub functions: Vec<Function>,
    /// Constants, initializers for globals
    ///
    /// *Since bytecode v4*
    pub constants: Option<Vec<ConstantDef>>,

    // Fields below are not part of the data.
    // Those are acceleration structures used to speed up lookup.
    /// Acceleration structure mapping function references (findex) to functions indexes in the native or function pool.
    findexes: Vec<RefFunKnown>,
    /// Acceleration structure mapping function names to function indexes in the function pool
    pub fnames: HashMap<String, usize>,
    pub globals_initializers: HashMap<RefGlobal, usize>,
}

impl Bytecode {
    /// Get the entrypoint function.
    pub fn entrypoint(&self) -> &Function {
        self.resolve(self.entrypoint).as_fn().unwrap()
    }

    /// Get the main function.
    /// This will panic if there is no main function in the bytecode (there should always be one)
    pub fn main(&self) -> &Function {
        &self.functions[*self.fnames.get("main").unwrap()]
    }

    /// Get a function by its name.
    pub fn function_by_name(&self, name: &str) -> Option<&Function> {
        self.fnames.get(name).map(|&i| &self.functions[i])
    }

    pub fn findex_max(&self) -> usize {
        self.findexes.len()
    }

    pub fn functions<'a>(&'a self) -> impl Iterator<Item = FunPtr<'a>> + 'a {
        (0..self.findex_max()).map(RefFun).map(|r| self.resolve(r))
    }
}

impl Default for Bytecode {
    fn default() -> Self {
        Self {
            version: 5,
            entrypoint: Default::default(),
            ints: vec![],
            floats: vec![],
            strings: vec![],
            bytes: None,
            debug_files: None,
            types: vec![],
            globals: vec![],
            natives: vec![],
            functions: vec![],
            constants: None,
            findexes: vec![],
            fnames: Default::default(),
            globals_initializers: Default::default(),
        }
    }
}

/// Index reference to either a function or a native.
#[derive(Debug, Copy, Clone)]
enum RefFunKnown {
    Fun(usize),
    Native(usize),
}

//region Resolve

pub trait Resolve<I> {
    type Output<'a>
    where
        Self: 'a;

    fn resolve(&self, index: I) -> Self::Output<'_>;
}

impl Resolve<RefInt> for Bytecode {
    type Output<'a> = i32;

    fn resolve(&self, index: RefInt) -> Self::Output<'_> {
        self.ints[index.0]
    }
}

impl Resolve<RefFloat> for Bytecode {
    type Output<'a> = f64;

    fn resolve(&self, index: RefFloat) -> Self::Output<'_> {
        self.floats[index.0]
    }
}

impl Resolve<RefString> for Bytecode {
    type Output<'a> = &'a str;

    fn resolve(&self, index: RefString) -> Self::Output<'_> {
        if index.0 > 0 {
            self.strings.index(index.0)
        } else {
            "<none>"
        }
    }
}

impl Resolve<RefType> for Bytecode {
    type Output<'a> = &'a Type;

    fn resolve(&self, index: RefType) -> Self::Output<'_> {
        &self.types[index.0]
    }
}

impl Resolve<RefFun> for Bytecode {
    type Output<'a> = FunPtr<'a>;

    fn resolve(&self, index: RefFun) -> Self::Output<'_> {
        match self.findexes[index.0] {
            RefFunKnown::Fun(fun) => FunPtr::Fun(&self.functions[fun]),
            RefFunKnown::Native(n) => FunPtr::Native(&self.natives[n]),
        }
    }
}

//endregion

// region Index impl

impl Index<RefInt> for Bytecode {
    type Output = i32;

    fn index(&self, index: RefInt) -> &Self::Output {
        self.ints.index(index.0)
    }
}

impl Index<RefFloat> for Bytecode {
    type Output = f64;

    fn index(&self, index: RefFloat) -> &Self::Output {
        self.floats.index(index.0)
    }
}

impl Index<RefString> for Bytecode {
    type Output = String;

    fn index(&self, index: RefString) -> &Self::Output {
        self.strings.index(index.0)
    }
}

impl Index<RefType> for Bytecode {
    type Output = Type;

    fn index(&self, index: RefType) -> &Self::Output {
        self.types.index(index.0)
    }
}

//endregion

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::BufReader;

    use crate::Bytecode;

    #[test]
    fn test_deserialize_all() {
        for entry in fs::read_dir("../../data").unwrap() {
            let path = entry.unwrap().path();
            if let Some(ext) = path.extension() {
                if ext == "hl" {
                    let code =
                        Bytecode::deserialize(&mut BufReader::new(fs::File::open(&path).unwrap()));
                    assert!(code.is_ok());
                }
            }
        }
    }

    #[test]
    fn test_deserialize_wartales() {
        let path = "E:\\Games\\Wartales\\hlboot.dat";
        let code = Bytecode::deserialize(&mut BufReader::new(fs::File::open(path).unwrap()));
        assert!(code.is_ok());
    }
}
