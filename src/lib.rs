use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::io::Read;

use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};

use crate::deser::ReadHlExt;
use crate::opcodes::Opcode;
use crate::types::{
    ConstantDef, Function, Native, ObjField, RefFun, RefGlobal, RefType, Type, TypeObj,
};

mod deser;
pub mod fmt;
pub mod opcodes;
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
    pub findexes: HashMap<RefFun, (usize, bool)>,
    pub fnames: HashMap<String, usize>,
    pub max_findex: usize,
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

        // Parsing is finished, we now build links between everything

        // Function indexes
        let mut max_findex = 0;
        let mut findexes = HashMap::with_capacity(functions.len() + natives.len());
        for (i, f) in functions.iter().enumerate() {
            findexes.insert(f.findex, (i, true));
            max_findex = max_findex.max(f.findex.0);
        }
        for (i, n) in natives.iter().enumerate() {
            findexes.insert(n.findex, (i, false));
            max_findex = max_findex.max(n.findex.0);
        }

        let mut new_fields: Vec<Option<Vec<ObjField>>> = Vec::with_capacity(types.len());
        // Flatten types fields
        // Start by collecting every fields in the hierarchy
        // The order is important because we refer to fields by index
        for t in &types {
            if let Some(obj) = t.get_type_obj() {
                let mut parent = obj.super_.as_ref().map(|s| s.resolve(&types));
                let mut acc = VecDeque::with_capacity(obj.fields.len());
                acc.extend(obj.fields.clone());
                while let Some(p) = parent.and_then(|t| t.get_type_obj()) {
                    for f in p.fields.iter().rev() {
                        acc.push_front(f.clone());
                    }
                    parent = p.super_.as_ref().map(|s| s.resolve(&types));
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

        // Function names based on object fields bindings and methods
        for t in &types {
            if let Some(TypeObj {
                protos, bindings, ..
            }) = t.get_type_obj()
            {
                for p in protos {
                    if let Some(f) = findexes.get(&p.findex).map(|(i, _)| &mut functions[*i]) {
                        f.name = Some(p.name);
                    }
                }
                for (fid, findex) in bindings {
                    if let Some(field) = t.get_type_obj().map(|o| &o.fields[fid.0]) {
                        if let Some(f) = findexes.get(findex).map(|(i, _)| &mut functions[*i]) {
                            f.name = Some(field.name);
                        }
                    }
                }
            }
        }

        // Function names
        let mut fnames = HashMap::with_capacity(functions.len());
        for (i, f) in functions.iter().enumerate() {
            if let Some(s) = f.name {
                // FIXME we possibly overwrite some values here, is that a problem ?
                fnames.insert(s.resolve(&strings).to_string(), i);
            }
        }
        fnames.insert("init".to_string(), findexes.get(&entrypoint).unwrap().0);

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
            max_findex,
        })
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
