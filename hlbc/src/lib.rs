//! [Hashlink](https://hashlink.haxe.org/) bytecode disassembler and analyzer.
//! See [Bytecode] for an entrypoint to the library.
//!
//! #### Note about safety
//! We don't deal with self-references, hence we deal with indexes into structures.
//! Be careful when calling functions on Ref* objects, as no bound checking is done and every index is assumed to be valid.

extern crate core;

use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::io::{Read, Write};
use std::ops::Index;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

use crate::deser::ReadHlExt;
use crate::opcodes::Opcode;
use crate::ser::WriteHlExt;
use crate::types::{
    ConstantDef, FunPtr, Function, Native, ObjField, RefFloat, RefFun, RefGlobal, RefInt,
    RefString, RefType, Type, TypeObj,
};

/// Analysis functions and callgraph generation
pub mod analysis;
pub mod deser;
pub mod fmt;
/// Opcodes definitions.
pub mod opcodes;
pub mod ser;
/// Bytecode elements definitions.
/// All the Ref* types in this modules are references to bytecode elements like constants or function.
/// They are required since we cannot use rust references as that would make our structure self-referential.
/// They makes the code look a bit more complicated than it actually is. Every Ref* struct is cheaply copyable.
pub mod types;

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
    /// Load the bytecode from any source.
    /// Must be a valid hashlink bytecode binary.
    pub fn deserialize(r: &mut impl Read) -> Result<Bytecode> {
        let mut header = [0u8; 3];
        r.read_exact(&mut header)?;
        if header != [b'H', b'L', b'B'] {
            return Err(Error::MalformedBytecode(format!(
                "Invalid magic header (expected: {:?}, got: {header:?})",
                b"HLB"
            )));
        }
        let version = r.read_u8()?;
        if version < 4 && version > 5 {
            return Err(Error::UnsupportedVersion {
                version,
                min: 4,
                max: 5,
            });
        }
        let flags = r.read_varu()?;
        let has_debug = flags & 1 == 1;
        let nints = r.read_varu()? as usize;
        let nfloats = r.read_varu()? as usize;
        let nstrings = r.read_varu()? as usize;
        let nbytes = if version >= 5 {
            Some(r.read_varu()? as usize)
        } else {
            None
        };
        let ntypes = r.read_varu()? as usize;
        let nglobals = r.read_varu()? as usize;
        let nnatives = r.read_varu()? as usize;
        let nfunctions = r.read_varu()? as usize;
        let nconstants = if version >= 4 {
            Some(r.read_varu()? as usize)
        } else {
            None
        };
        let entrypoint = RefFun(r.read_varu()? as usize);

        let mut ints = vec![0i32; nints];
        for i in ints.iter_mut() {
            *i = r.read_i32::<LittleEndian>()?;
        }

        let mut floats = vec![0f64; nfloats];
        for i in floats.iter_mut() {
            *i = r.read_f64::<LittleEndian>()?;
        }

        let strings = r.read_strings(nstrings)?;

        let bytes = if let Some(nbytes) = nbytes {
            let size = r.read_i32::<LittleEndian>()? as usize;
            let mut bytes = vec![0; size];
            r.read_exact(&mut bytes)?;
            let mut pos = Vec::with_capacity(nbytes);
            for _ in 0..nbytes {
                pos.push(r.read_varu()? as usize);
            }
            Some((bytes, pos))
        } else {
            None
        };

        let debug_files = if has_debug {
            let n = r.read_varu()? as usize;
            Some(r.read_strings(n)?)
        } else {
            None
        };

        let mut types = Vec::with_capacity(ntypes);
        for _ in 0..ntypes {
            types.push(r.read_type()?);
        }

        let mut globals = Vec::with_capacity(nglobals);
        for _ in 0..nglobals {
            globals.push(r.read_type_ref()?);
        }

        let mut natives = Vec::with_capacity(nnatives);
        for _ in 0..nnatives {
            natives.push(r.read_native()?);
        }

        let mut functions = Vec::with_capacity(nfunctions);
        for _ in 0..nfunctions {
            functions.push(r.read_function(has_debug, version)?);
        }

        let constants = if let Some(n) = nconstants {
            let mut constants = Vec::with_capacity(n);
            for _ in 0..n {
                constants.push(r.read_constant_def()?)
            }
            Some(constants)
        } else {
            None
        };

        // Parsing is finished, we now build links between everything

        // Global function indexes
        let mut findexes = vec![RefFunKnown::Fun(0); nfunctions + nnatives];
        for (i, f) in functions.iter().enumerate() {
            findexes[f.findex.0] = RefFunKnown::Fun(i);
        }
        for (i, n) in natives.iter().enumerate() {
            findexes[n.findex.0] = RefFunKnown::Native(i);
        }

        // Flatten types fields
        // Start by collecting every fields in the hierarchy
        // The order is important because we refer to fields by index
        let mut new_fields: Vec<Option<Vec<ObjField>>> = Vec::with_capacity(types.len());
        for t in &types {
            if let Some(obj) = t.get_type_obj() {
                let mut parent = obj.super_.as_ref().map(|s| &types[s.0]);
                let mut acc = VecDeque::with_capacity(obj.own_fields.len());
                acc.extend(obj.own_fields.clone());
                while let Some(p) = parent.and_then(|t| t.get_type_obj()) {
                    for f in p.own_fields.iter().rev() {
                        acc.push_front(f.clone());
                    }
                    parent = p.super_.as_ref().map(|s| &types[s.0]);
                }
                new_fields.push(Some(acc.into()));
            } else {
                new_fields.push(None);
            }
        }
        // Apply new fields
        for (t, new) in types.iter_mut().zip(new_fields.into_iter()) {
            if let Some(fields) = new {
                t.get_type_obj_mut().unwrap().fields = fields;
            }
        }

        // Give functions name based on object fields bindings and methods
        for (i, t) in types.iter().enumerate() {
            if let Some(TypeObj {
                protos, bindings, ..
            }) = t.get_type_obj()
            {
                for p in protos {
                    if let RefFunKnown::Fun(x) = findexes[p.findex.0] {
                        functions[x].name = p.name;
                        functions[x].parent = Some(RefType(i));
                    }
                }
                for (fid, findex) in bindings {
                    if let Some(field) = t.get_type_obj().map(|o| &o.fields[fid.0]) {
                        if let RefFunKnown::Fun(x) = findexes[findex.0] {
                            functions[x].name = field.name;
                            functions[x].parent = Some(RefType(i));
                        }
                    }
                }
            }
        }

        // Function names
        let mut fnames = HashMap::with_capacity(functions.len());
        for (i, f) in functions.iter().enumerate() {
            // FIXME duplicates ?
            fnames.insert(strings[f.name.0].clone(), i);
        }
        fnames.insert(
            "init".to_string(),
            match findexes[entrypoint.0] {
                RefFunKnown::Fun(x) => x,
                _ => 0,
            },
        );

        let globals_initializers = if let Some(constants) = &constants {
            let mut tmp = HashMap::with_capacity(constants.len());
            for (i, c) in constants.iter().enumerate() {
                tmp.insert(c.global, i);
            }
            tmp
        } else {
            HashMap::new()
        };

        Ok(Bytecode {
            version,
            entrypoint,
            ints,
            floats,
            strings,
            bytes,
            debug_files,
            types,
            globals,
            natives,
            functions,
            constants,
            findexes,
            fnames,
            globals_initializers,
        })
    }

    /// Serialize the bytecode to any sink.
    /// Bytecode is serialized to the same format.
    pub fn serialize(&self, w: &mut impl Write) -> Result<()> {
        w.write_all(&[b'H', b'L', b'B'])?;
        w.write_u8(self.version)?;
        w.write_vi32(if self.debug_files.is_some() { 1 } else { 0 })?;
        w.write_vi32(self.ints.len() as i32)?;
        w.write_vi32(self.floats.len() as i32)?;
        w.write_vi32(self.strings.len() as i32)?;
        if let Some((_, pos)) = &self.bytes {
            w.write_vi32(pos.len() as i32)?;
        }
        w.write_vi32(self.types.len() as i32)?;
        w.write_vi32(self.globals.len() as i32)?;
        w.write_vi32(self.natives.len() as i32)?;
        w.write_vi32(self.functions.len() as i32)?;
        if let Some(constants) = &self.constants {
            w.write_vi32(constants.len() as i32)?;
        }
        w.write_vi32(self.entrypoint.0 as i32)?;
        for &i in &self.ints {
            w.write_i32::<LittleEndian>(i)?;
        }
        for &f in &self.floats {
            w.write_f64::<LittleEndian>(f)?;
        }
        w.write_strings(&self.strings)?;
        if let Some((bytes, pos)) = &self.bytes {
            w.write_i32::<LittleEndian>(bytes.len() as i32)?;
            w.write_all(bytes)?;
            for &p in pos {
                w.write_vi32(p as i32)?;
            }
        }
        if let Some(debug_files) = &self.debug_files {
            w.write_vi32(debug_files.len() as i32)?;
            w.write_strings(debug_files)?;
        }
        for t in &self.types {
            w.write_type(t)?;
        }
        for g in &self.globals {
            w.write_vi32(g.0 as i32)?;
        }
        for n in &self.natives {
            w.write_native(n)?;
        }
        for f in &self.functions {
            w.write_function(f)?;
        }
        if let Some(constants) = &self.constants {
            for c in constants {
                w.write_constant_def(c)?;
            }
        }
        Ok(())
    }

    /// Get the entrypoint function.
    pub fn entrypoint<'a>(&'a self) -> &'a Function {
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

// Index reference to either a function or a native.
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
