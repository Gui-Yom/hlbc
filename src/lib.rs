use std::fmt::Debug;
use std::io::Read;

use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};

use crate::opcodes::Opcode;
use crate::read::ReadHlExt;
use crate::types::{ConstantDef, Function, Native, ObjField, RefFun, RefGlobal, RefType, Type};

pub mod fmt;
pub mod opcodes;
mod read;
pub mod types;

#[derive(Debug)]
pub struct Bytecode {
    pub version: u8,
    pub entrypoint: RefFun,
    pub ints: Vec<i32>,
    pub floats: Vec<f64>,
    pub strings: Vec<String>,
    pub bytes: Option<Vec<u8>>,
    pub debug_files: Option<Vec<String>>,
    pub types: Vec<Type>,
    pub globals: Vec<RefType>,
    pub natives: Vec<Native>,
    pub functions: Vec<Function>,
    pub constants: Option<Vec<ConstantDef>>,
}

impl Bytecode {
    pub fn load(r: &mut impl Read) -> Result<Bytecode> {
        let mut header = [0u8; 3];
        r.read_exact(&mut header)?;
        if header != [b'H', b'L', b'B'] {
            anyhow::bail!("invalid header magic");
        }
        let version = r.read_u8()?;
        if version < 4 {
            anyhow::bail!("Unsupported version {version} < 4");
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

        let bytes = if version >= 5 {
            let size = r.read_i32::<LittleEndian>()? as usize;
            let mut bytes = vec![0; size];
            r.read_exact(&mut bytes)?;
            for _ in 0..nbytes.unwrap() {
                r.read_varu()?;
            }
            Some(bytes)
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

        for t in &types {
            match t {
                Type::Obj {
                    protos,
                    bindings,
                    fields,
                    ..
                } => {
                    for p in protos {
                        if let Some(f) = find_findex_mut(&mut functions, p.findex.0) {
                            f.name = Some(p.name);
                        }
                    }
                    for (fid, findex) in bindings {
                        if let Some(field) = find_field(&types, t, *fid) {
                            if let Some(f) = find_findex_mut(&mut functions, *findex) {
                                f.name = Some(field.name);
                            }
                        }
                    }
                }
                Type::Struct {
                    protos, bindings, ..
                } => {
                    for p in protos {
                        if let Some(f) = find_findex_mut(&mut functions, p.findex.0) {
                            f.name = Some(p.name);
                        }
                    }
                    for (fid, findex) in bindings {
                        if let Some(field) = find_field(&types, t, *fid) {
                            if let Some(f) = find_findex_mut(&mut functions, *findex) {
                                f.name = Some(field.name);
                            }
                        }
                    }
                }
                _ => {}
            }
        }

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
        })
    }
}

pub fn find_findex_mut(functions: &mut [Function], findex: usize) -> Option<&mut Function> {
    functions.iter_mut().find(|f| f.findex.0 == findex)
}

pub fn find_field<'a>(types: &'a [Type], ty: &'a Type, fid: usize) -> Option<&'a ObjField> {
    match ty {
        Type::Obj { .. } => Some(fetch_fields_rec(types, ty)[fid]),
        Type::Struct { .. } => Some(fetch_fields_rec(types, ty)[fid]),
        _ => None,
    }
}

pub fn fetch_fields_rec<'a>(types: &'a [Type], ty: &'a Type) -> Vec<&'a ObjField> {
    match ty {
        Type::Obj { super_, fields, .. } => {
            if let Some(s) = super_ {
                let mut ret = fetch_fields_rec(types, s.resolve(&types));
                ret.extend(fields.iter());
                ret
            } else {
                fields.iter().collect()
            }
        }
        Type::Struct { super_, fields, .. } => {
            if let Some(s) = super_ {
                let mut ret = fetch_fields_rec(types, s.resolve(&types));
                ret.extend(fields.iter());
                ret
            } else {
                fields.iter().collect()
            }
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;

    use anyhow::Result;

    use crate::Bytecode;

    #[test]
    fn test() -> Result<()> {
        let mut reader = BufReader::new(File::open(
            "D:/ReverseEngineering/northgard/hlbc/hlboot2.dat",
        )?);
        println!("{:#?}", Bytecode::load(&mut reader)?);
        Ok(())
    }
}
