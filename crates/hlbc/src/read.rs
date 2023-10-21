use std::collections::{HashMap, VecDeque};
use std::fs;
use std::io::{BufReader, Read};
use std::path::Path;
use std::str::from_utf8;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::types::{
    EnumConstruct, Function, Native, ObjField, ObjProto, RefField, RefFloat, RefInt, RefString,
    RefType, Type, TypeFun, TypeObj,
};
use crate::{Bytecode, ConstantDef, Opcode, RefFun, RefFunKnown, RefGlobal, Str};
use crate::{Error, Result};

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
        let flags = read_varu(r)?;
        let has_debug = flags & 1 == 1;
        let nints = read_varu(r)? as usize;
        let nfloats = read_varu(r)? as usize;
        let nstrings = read_varu(r)? as usize;
        let nbytes = if version >= 5 {
            Some(read_varu(r)? as usize)
        } else {
            None
        };
        let ntypes = read_varu(r)? as usize;
        let nglobals = read_varu(r)? as usize;
        let nnatives = read_varu(r)? as usize;
        let nfunctions = read_varu(r)? as usize;
        let nconstants = if version >= 4 {
            Some(read_varu(r)? as usize)
        } else {
            None
        };
        let entrypoint = RefFun::read(r)?;

        let mut ints = vec![0i32; nints];
        for i in ints.iter_mut() {
            *i = r.read_i32::<LittleEndian>()?;
        }

        let mut floats = vec![0f64; nfloats];
        for i in floats.iter_mut() {
            *i = r.read_f64::<LittleEndian>()?;
        }

        let strings = read_strings(r, nstrings)?;

        let bytes = if let Some(nbytes) = nbytes {
            let size = r.read_i32::<LittleEndian>()? as usize;
            let mut bytes = vec![0; size];
            r.read_exact(&mut bytes)?;
            let mut pos = Vec::with_capacity(nbytes);
            for _ in 0..nbytes {
                pos.push(read_varu(r)? as usize);
            }
            Some((bytes, pos))
        } else {
            None
        };

        let debug_files = if has_debug {
            let n = read_varu(r)? as usize;
            Some(read_strings(r, n)?)
        } else {
            None
        };

        let mut types = Vec::with_capacity(ntypes);
        for _ in 0..ntypes {
            types.push(Type::read(r)?);
        }

        let mut globals = Vec::with_capacity(nglobals);
        for _ in 0..nglobals {
            globals.push(RefType::read(r)?);
        }

        let mut natives = Vec::with_capacity(nnatives);
        for _ in 0..nnatives {
            natives.push(Native::read(r)?);
        }

        let mut functions = Vec::with_capacity(nfunctions);
        for _ in 0..nfunctions {
            functions.push(Function::read(r, has_debug, version)?);
        }

        let constants = if let Some(n) = nconstants {
            let mut constants = Vec::with_capacity(n);
            for _ in 0..n {
                constants.push(ConstantDef::read(r)?)
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
            Str::from("init"),
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

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        Self::deserialize(&mut BufReader::new(fs::File::open(path)?))
    }
}

impl RefInt {
    pub(crate) fn read(r: &mut impl Read) -> Result<Self> {
        Ok(Self(read_vari(r)? as usize))
    }
}

impl RefFloat {
    pub(crate) fn read(r: &mut impl Read) -> Result<Self> {
        Ok(Self(read_vari(r)? as usize))
    }
}

impl RefString {
    pub(crate) fn read(r: &mut impl Read) -> Result<Self> {
        Ok(Self(read_vari(r)? as usize))
    }
}

impl RefGlobal {
    pub(crate) fn read(r: &mut impl Read) -> Result<Self> {
        Ok(Self(read_vari(r)? as usize))
    }
}

impl RefFun {
    pub(crate) fn read(r: &mut impl Read) -> Result<Self> {
        Ok(Self(read_vari(r)? as usize))
    }
}

impl RefType {
    pub(crate) fn read(r: &mut impl Read) -> Result<Self> {
        Ok(Self(read_vari(r)? as usize))
    }
}

impl RefField {
    pub(crate) fn read(r: &mut impl Read) -> Result<Self> {
        Ok(Self(read_vari(r)? as usize))
    }
}

impl ObjField {
    pub(crate) fn read(r: &mut impl Read) -> Result<Self> {
        Ok(ObjField {
            name: RefString::read(r)?,
            t: RefType::read(r)?,
        })
    }
}

impl TypeFun {
    pub(crate) fn read(r: &mut impl Read) -> Result<Self> {
        let nargs = r.read_u8()?;
        let mut args = Vec::with_capacity(nargs as usize);
        for _ in 0..nargs {
            args.push(RefType::read(r)?);
        }
        Ok(TypeFun {
            args,
            ret: RefType::read(r)?,
        })
    }
}

impl TypeObj {
    pub(crate) fn read(r: &mut impl Read) -> Result<Self> {
        let name = RefString::read(r)?;
        let super_ = read_vari(r)?;
        let global = RefGlobal::read(r)?;
        let nfields = read_varu(r)? as usize;
        let nprotos = read_varu(r)? as usize;
        let nbindings = read_varu(r)? as usize;
        let mut own_fields = Vec::with_capacity(nfields);
        for _ in 0..nfields {
            own_fields.push(ObjField::read(r)?);
        }
        let mut protos = Vec::with_capacity(nprotos);
        for _ in 0..nprotos {
            protos.push(ObjProto {
                name: RefString::read(r)?,
                findex: RefFun::read(r)?,
                pindex: read_vari(r)?,
            });
        }
        let mut bindings = HashMap::with_capacity(nbindings);
        for _ in 0..nbindings {
            bindings.insert(RefField::read(r)?, RefFun::read(r)?);
        }
        Ok(TypeObj {
            name,
            super_: if super_ < 0 {
                None
            } else {
                Some(RefType(super_ as usize))
            },
            global,
            own_fields,
            fields: Vec::new(),
            protos,
            bindings,
        })
    }
}

impl Type {
    pub(crate) fn read(r: &mut impl Read) -> Result<Self> {
        use crate::Type::*;
        match r.read_u8()? {
            0 => Ok(Void),
            1 => Ok(UI8),
            2 => Ok(UI16),
            3 => Ok(I32),
            4 => Ok(I64),
            5 => Ok(F32),
            6 => Ok(F64),
            7 => Ok(Bool),
            8 => Ok(Bytes),
            9 => Ok(Dyn),
            10 => Ok(Fun(TypeFun::read(r)?)),
            11 => Ok(Obj(TypeObj::read(r)?)),
            12 => Ok(Array),
            13 => Ok(Type),
            14 => Ok(Ref(RefType::read(r)?)),
            15 => {
                let nfields = read_varu(r)? as usize;
                let mut fields = Vec::with_capacity(nfields);
                for _ in 0..nfields {
                    fields.push(ObjField::read(r)?);
                }
                Ok(Virtual { fields })
            }
            16 => Ok(DynObj),
            17 => Ok(Abstract {
                name: RefString::read(r)?,
            }),
            18 => {
                let name = RefString::read(r)?;
                let global = RefGlobal::read(r)?;
                let nconstructs = read_varu(r)? as usize;
                let mut constructs = Vec::with_capacity(nconstructs);
                for _ in 0..nconstructs {
                    let name = RefString::read(r)?;
                    let nparams = read_varu(r)? as usize;
                    let mut params = Vec::with_capacity(nparams);
                    for _ in 0..nparams {
                        params.push(RefType::read(r)?);
                    }
                    constructs.push(EnumConstruct { name, params })
                }
                Ok(Enum {
                    name,
                    global,
                    constructs,
                })
            }
            19 => Ok(Null(RefType::read(r)?)),
            20 => Ok(Method(TypeFun::read(r)?)),
            21 => Ok(Struct(TypeObj::read(r)?)),
            22 => Ok(Packed(RefType::read(r)?)),
            other => Err(Error::MalformedBytecode(format!(
                "Invalid type kind '{other}'"
            ))),
        }
    }
}

impl Native {
    pub(crate) fn read(r: &mut impl Read) -> Result<Self> {
        Ok(Native {
            lib: RefString::read(r)?,
            name: RefString::read(r)?,
            t: RefType::read(r)?,
            findex: RefFun::read(r)?,
        })
    }
}

impl Function {
    pub(crate) fn read(r: &mut impl Read, has_debug: bool, version: u8) -> Result<Self> {
        let t = RefType::read(r)?;
        let findex = RefFun::read(r)?;
        let nregs = read_varu(r)? as usize;
        let nops = read_varu(r)? as usize;
        let mut regs = Vec::with_capacity(nregs);
        for _ in 0..nregs {
            regs.push(RefType::read(r)?);
        }
        let mut ops = Vec::with_capacity(nops);
        for _ in 0..nops {
            ops.push(Opcode::read(r)?);
        }

        // This is extracted from the hashlink source code, do not count on me to explain what it does
        let debug_info = if has_debug {
            let mut tmp = Vec::with_capacity(nops);
            let mut currfile: i32 = -1;
            let mut currline: i32 = 0;
            let mut i = 0;
            while i < nops {
                let mut c = r.read_u8()? as i32;
                if c & 1 != 0 {
                    c >>= 1;
                    currfile = (c << 8) | (r.read_u8()? as i32);
                } else if c & 2 != 0 {
                    let delta = c >> 6;
                    let mut count = (c >> 2) & 15;
                    while count > 0 {
                        count -= 1;
                        tmp.push((currfile as usize, currline as usize));
                        i += 1;
                    }
                    currline += delta;
                } else if c & 4 != 0 {
                    currline += c >> 3;
                    tmp.push((currfile as usize, currline as usize));
                    i += 1;
                } else {
                    let b2 = r.read_u8()? as i32;
                    let b3 = r.read_u8()? as i32;
                    currline = (c >> 3) | (b2 << 5) | (b3 << 13);
                    tmp.push((currfile as usize, currline as usize));
                    i += 1;
                }
            }
            Some(tmp)
        } else {
            None
        };

        let assigns = if has_debug && version >= 3 {
            let len = read_varu(r)? as usize;
            let mut assigns = Vec::with_capacity(len);
            for _ in 0..len {
                assigns.push((RefString::read(r)?, read_vari(r)? as usize));
            }
            Some(assigns)
        } else {
            None
        };
        Ok(Function {
            name: RefString(0),
            t,
            findex,
            regs,
            ops,
            debug_info,
            assigns,
            parent: None,
        })
    }
}

impl ConstantDef {
    pub(crate) fn read(r: &mut impl Read) -> Result<Self> {
        let global = RefGlobal::read(r)?;
        let nfields = read_varu(r)? as usize;
        let mut fields = Vec::with_capacity(nfields);
        for _ in 0..nfields {
            fields.push(read_varu(r)? as usize);
        }
        Ok(ConstantDef { global, fields })
    }
}

pub(crate) fn read_vari(r: &mut impl Read) -> Result<i32> {
    let b = r.read_u8()? as i32;
    if b & 0x80 == 0 {
        Ok(b & 0x7F)
    } else if b & 0x40 == 0 {
        let v = r.read_u8()? as i32 | ((b & 31) << 8);
        Ok(if b & 0x20 == 0 { v } else { -v })
    } else {
        let c = r.read_u8()? as i32;
        let d = r.read_u8()? as i32;
        let e = r.read_u8()? as i32;
        let v = ((b & 31) << 24) | (c << 16) | (d << 8) | e;
        Ok(if b & 0x20 == 0 { v } else { -v })
    }
}

pub(crate) fn read_varu(r: &mut impl Read) -> Result<u32> {
    let i = read_vari(r)?;
    if i < 0 {
        Err(Error::MalformedBytecode(format!(
            "Got negative index '{i}' (expected >= 0)"
        )))
    } else {
        Ok(i as u32)
    }
}

fn read_strings(r: &mut impl Read, nstrings: usize) -> Result<Vec<Str>> {
    let mut strings = Vec::with_capacity(nstrings);
    let mut string_data = vec![0u8; r.read_i32::<LittleEndian>()? as usize];
    r.read_exact(&mut string_data)?;
    let mut acc = 0;
    for _ in 0..nstrings {
        let ssize = read_varu(r)? as usize + 1;
        //println!("size: {ssize} {:?}", &string_data[acc..(acc + ssize)]);
        //let cstr = unsafe { CStr::from_bytes_with_nul_unchecked(&string_data[acc..(acc + ssize)]) };
        strings.push(Str::from_ref(from_utf8(
            &string_data[acc..(acc + ssize - 1)],
        )?));
        acc += ssize;
    }
    Ok(strings)
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::fs;
    use std::io::{BufWriter, Write};

    use crate::Bytecode;

    #[test]
    fn test_deserialize_all() {
        for entry in fs::read_dir("../../data").unwrap() {
            let path = entry.unwrap().path();
            if let Some(ext) = path.extension() {
                if ext == "hl" {
                    let code = Bytecode::from_file(&path);
                    assert!(code.is_ok());
                }
            }
        }
    }

    #[test]
    fn test_deserialize_wartales() {
        let path = "E:\\Games\\Wartales\\hlboot.dat";
        let code = Bytecode::from_file(path);
        assert!(code.is_ok());
    }

    #[test]
    fn test_deserialize_northgard() {
        let path = "E:\\Games\\Northgard\\hlboot.dat";
        let code = Bytecode::from_file(path);
        assert!(code.is_ok());
    }

    //#[test]
    fn list_strings() {
        let code = Bytecode::from_file("E:\\Games\\Northgard\\hlboot.dat").unwrap();
        let code2 = Bytecode::from_file("E:\\Games\\Wartales\\hlboot.dat").unwrap();
        let mut file = BufWriter::new(
            fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open("strings.txt")
                .unwrap(),
        );
        let mut set = HashSet::with_capacity(code.strings.len() + code2.strings.len());
        for s in code.strings {
            set.insert(s);
        }
        for s in code2.strings {
            set.insert(s);
        }
        for s in &set {
            file.write(s.as_bytes()).unwrap();
            file.write(b"\n").unwrap();
        }
    }
}
