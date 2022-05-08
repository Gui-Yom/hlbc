use std::cell::RefCell;
use std::ffi::CStr;
use std::fmt::{Debug, Display, Formatter};
use std::io::Read;
use std::ops::Index;

use anyhow::Result;
use byteorder::{LittleEndian, ReadBytesExt};
use strum_macros::IntoStaticStr;

use crate::opcodes::Opcode;
use crate::utils::{vari32, varu32};

pub mod opcodes;
mod utils;

fn read_strings(r: &mut impl Read, nstrings: usize) -> Result<Vec<String>> {
    let mut strings = Vec::with_capacity(nstrings);
    let mut string_data = vec![0u8; r.read_i32::<LittleEndian>()? as usize];
    r.read_exact(&mut string_data)?;
    let mut acc = 0;
    for _ in 0..nstrings {
        let ssize = varu32(r)? as usize + 1;
        strings.push(
            CStr::from_bytes_with_nul(&string_data[acc..(acc + ssize)])?
                .to_string_lossy()
                .to_string(),
        );
        acc += ssize;
    }
    Ok(strings)
}

fn read_field(r: &mut impl Read, strings: &[String]) -> Result<HlObjField> {
    Ok(HlObjField {
        name: strings[vari32(r)? as usize].clone(),
        t: read_type_ref(r)?,
    })
}

fn read_type_ref(r: &mut impl Read) -> Result<HlTypeRef> {
    Ok(HlTypeRef(vari32(r)? as usize))
}

#[derive(Debug)]
pub struct HlCode {
    pub version: u8,
    pub ints: Vec<i32>,
    pub floats: Vec<f64>,
    pub strings: Vec<String>,
    pub debug_files: Option<Vec<String>>,
    pub types: Vec<HlType>,
    pub globals: Vec<HlTypeRef>,
    pub natives: Vec<HlNative>,
    pub functions: Vec<HlFunction>,
    pub constants: Vec<HlConstant>,
}

impl HlCode {
    pub fn load(r: &mut impl Read) -> Result<HlCode> {
        let mut header = [0u8; 3];
        r.read_exact(&mut header)?;
        if header != [b'H', b'L', b'B'] {
            anyhow::bail!("invalid header magic");
        }
        let version = r.read_u8()?;
        if version < 4 {
            anyhow::bail!("Unsupported version {version} < 4");
        }
        let flags = varu32(r)?;
        let has_debug = flags & 1 == 1;
        let nints = varu32(r)? as usize;
        let nfloats = varu32(r)? as usize;
        let nstrings = varu32(r)? as usize;
        let nbytes = if version >= 5 {
            Some(varu32(r)? as usize)
        } else {
            None
        };
        let ntypes = varu32(r)? as usize;
        let nglobals = varu32(r)? as usize;
        let nnatives = varu32(r)? as usize;
        let nfunctions = varu32(r)? as usize;
        let nconstants = varu32(r)? as usize;
        let entrypoint = varu32(r)? as usize;

        let mut ints = vec![0i32; nints];
        for i in ints.iter_mut() {
            *i = r.read_i32::<LittleEndian>()?;
        }

        let mut floats = vec![0f64; nfloats];
        for i in floats.iter_mut() {
            *i = r.read_f64::<LittleEndian>()?;
        }

        let strings = read_strings(r, nstrings)?;

        // TODO bytes

        let debug_files = if has_debug {
            let n = varu32(r)? as usize;
            Some(read_strings(r, n)?)
        } else {
            None
        };

        let mut types = Vec::with_capacity(ntypes);
        for _ in 0..ntypes {
            match r.read_u8()? {
                10 => {
                    let nargs = r.read_u8()?;
                    let mut args = Vec::with_capacity(nargs as usize);
                    for _ in 0..nargs {
                        args.push(read_type_ref(r)?);
                    }
                    types.push(HlType::Fun {
                        args,
                        ret: read_type_ref(r)?,
                    })
                }
                20 => {
                    let nargs = r.read_u8()?;
                    let mut args = Vec::with_capacity(nargs as usize);
                    for _ in 0..nargs {
                        args.push(read_type_ref(r)?);
                    }
                    types.push(HlType::Method {
                        args,
                        ret: read_type_ref(r)?,
                    })
                }
                11 => {
                    let name = strings[vari32(r)? as usize].clone();
                    let super_ = vari32(r)?;
                    let global = varu32(r)?;
                    let nfields = varu32(r)? as usize;
                    let nprotos = varu32(r)? as usize;
                    let nbindings = varu32(r)? as usize;
                    let mut fields = Vec::with_capacity(nfields);
                    for _ in 0..nfields {
                        fields.push(read_field(r, &strings)?);
                    }
                    let mut protos = Vec::with_capacity(nprotos);
                    for _ in 0..nprotos {
                        protos.push(HlObjProto {
                            name: strings[vari32(r)? as usize].clone(),
                            findex: varu32(r)? as usize,
                            pindex: vari32(r)? as usize,
                        });
                    }
                    let mut bindings = Vec::with_capacity(nbindings);
                    for _ in 0..nbindings {
                        bindings.push((varu32(r)?, varu32(r)?));
                    }
                    types.push(HlType::Obj {
                        name,
                        super_: if super_ < 0 {
                            None
                        } else {
                            Some(HlTypeRef(super_ as usize))
                        },
                        fields,
                        protos,
                        bindings,
                    });
                }
                21 => {
                    let name = strings[vari32(r)? as usize].clone();
                    let super_ = vari32(r)?;
                    let global = varu32(r)?;
                    let nfields = varu32(r)? as usize;
                    let nprotos = varu32(r)? as usize;
                    let nbindings = varu32(r)? as usize;
                    let mut fields = Vec::with_capacity(nfields);
                    for _ in 0..nfields {
                        fields.push(read_field(r, &strings)?);
                    }
                    let mut protos = Vec::with_capacity(nprotos);
                    for _ in 0..nprotos {
                        protos.push(HlObjProto {
                            name: strings[vari32(r)? as usize].clone(),
                            findex: varu32(r)? as usize,
                            pindex: vari32(r)? as usize,
                        });
                    }
                    let mut bindings = Vec::with_capacity(nbindings);
                    for _ in 0..nbindings {
                        bindings.push((varu32(r)?, varu32(r)?));
                    }
                    types.push(HlType::Struct {
                        name,
                        super_: if super_ < 0 {
                            None
                        } else {
                            Some(HlTypeRef(super_ as usize))
                        },
                        fields,
                        protos,
                        bindings,
                    });
                }
                14 => types.push(HlType::Ref(read_type_ref(r)?)),
                15 => {
                    let nfields = varu32(r)? as usize;
                    let mut fields = Vec::with_capacity(nfields);
                    for _ in 0..nfields {
                        fields.push(read_field(r, &strings)?);
                    }
                    types.push(HlType::Virtual { fields });
                }
                17 => {
                    types.push(HlType::Abstract {
                        name: strings[vari32(r)? as usize].clone(),
                    });
                }
                18 => {
                    let name = strings[vari32(r)? as usize].clone();
                    let global = varu32(r)?;
                    let nconstructs = varu32(r)? as usize;
                    let mut constructs = Vec::with_capacity(nconstructs);
                    for _ in 0..nconstructs {
                        let name = strings[vari32(r)? as usize].clone();
                        let nparams = varu32(r)? as usize;
                        let mut params = Vec::with_capacity(nparams);
                        for _ in 0..nparams {
                            params.push(read_type_ref(r)?);
                        }
                        constructs.push(HlEnumConstruct { name, params })
                    }
                    types.push(HlType::Enum { name, constructs })
                }
                19 => types.push(HlType::Null(read_type_ref(r)?)),
                other => {
                    if other >= 22 {
                        anyhow::bail!("Invalid type kind {other}")
                    }
                }
            }
        }

        let mut globals = Vec::with_capacity(nglobals);
        for _ in 0..nglobals {
            globals.push(read_type_ref(r)?);
        }

        let mut natives = Vec::with_capacity(nnatives);
        for _ in 0..nnatives {
            natives.push(HlNative {
                name: strings[vari32(r)? as usize].clone(),
                lib: strings[vari32(r)? as usize].clone(),
                t: read_type_ref(r)?,
                findex: varu32(r)? as usize,
            });
        }

        let mut functions = Vec::with_capacity(nfunctions);
        for _ in 0..nfunctions {
            let t = read_type_ref(r)?;
            let findex = varu32(r)? as usize;
            let nregs = varu32(r)? as usize;
            let nops = varu32(r)? as usize;
            let mut regs = Vec::with_capacity(nregs);
            for _ in 0..nregs {
                regs.push(read_type_ref(r)?);
            }
            let mut ops = Vec::with_capacity(nops);
            for _ in 0..nops {
                ops.push(Opcode::decode(r)?);
            }
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
                            tmp.push((currfile, currline));
                            i += 1;
                        }
                        currline += delta;
                    } else if c & 4 != 0 {
                        currline += c >> 3;
                        tmp.push((currfile, currline));
                        i += 1;
                    } else {
                        let b2 = r.read_u8()? as i32;
                        let b3 = r.read_u8()? as i32;
                        currline = (c >> 3) | (b2 << 5) | (b3 << 13);
                        tmp.push((currfile, currline));
                        i += 1;
                    }
                }
                Some(tmp)
            } else {
                None
            };
            if has_debug && version >= 3 {
                let len = varu32(r)? as usize;
                for _ in 0..len {
                    varu32(r)?;
                    vari32(r)?;
                }
            }
            functions.push(HlFunction {
                t,
                findex,
                regs,
                ops,
                debug_info,
            });
        }

        let mut constants = Vec::with_capacity(nconstants);
        for _ in 0..nconstants {
            let global = varu32(r)? as usize;
            let nfields = varu32(r)? as usize;
            let mut fields = Vec::with_capacity(nfields);
            for _ in 0..nfields {
                fields.push(varu32(r)? as usize);
            }
            constants.push(HlConstant { global, fields });
        }

        Ok(HlCode {
            version,
            ints,
            floats,
            strings,
            debug_files,
            types,
            globals,
            natives,
            functions,
            constants,
        })
    }
}

#[derive(Debug, Clone)]
pub struct HlTypeRef(usize);

#[derive(Debug, Clone)]
pub struct HlObjField {
    pub name: String,
    pub t: HlTypeRef,
}

#[derive(Debug, Clone)]
pub struct HlObjProto {
    pub name: String,
    pub findex: usize,
    pub pindex: usize,
}

#[derive(Debug, Clone)]
pub struct HlEnumConstruct {
    pub name: String,
    pub params: Vec<HlTypeRef>,
}

#[derive(Debug, Clone)]
pub enum HlType {
    Void,
    UI8,
    UI16,
    I32,
    I64,
    F32,
    F64,
    Bool,
    Bytes,
    Dyn,
    Fun {
        args: Vec<HlTypeRef>,
        ret: HlTypeRef,
    },
    Obj {
        name: String,
        super_: Option<HlTypeRef>,
        fields: Vec<HlObjField>,
        protos: Vec<HlObjProto>,
        bindings: Vec<(u32, u32)>,
    },
    Array,
    Type,
    Ref(HlTypeRef),
    Virtual {
        fields: Vec<HlObjField>,
    },
    DynObj,
    Abstract {
        name: String,
    },
    Enum {
        name: String,
        constructs: Vec<HlEnumConstruct>,
    },
    Null(HlTypeRef),
    Method {
        args: Vec<HlTypeRef>,
        ret: HlTypeRef,
    },
    Struct {
        name: String,
        super_: Option<HlTypeRef>,
        fields: Vec<HlObjField>,
        protos: Vec<HlObjProto>,
        bindings: Vec<(u32, u32)>,
    },
}

#[derive(Debug, Clone)]
pub struct HlNative {
    pub name: String,
    pub lib: String,
    pub t: HlTypeRef,
    pub findex: usize,
}

#[derive(Debug, Clone)]
pub struct HlFunction {
    pub t: HlTypeRef,
    pub findex: usize,
    pub regs: Vec<HlTypeRef>,
    pub ops: Vec<Opcode>,
    pub debug_info: Option<Vec<(i32, i32)>>,
}

#[derive(Debug, Clone)]
pub struct HlConstant {
    pub global: usize,
    pub fields: Vec<usize>,
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::BufReader;

    use anyhow::Result;

    use crate::HlCode;

    #[test]
    fn test() -> Result<()> {
        let mut reader = BufReader::new(File::open(
            "D:/ReverseEngineering/northgard/hlbc/hlboot2.dat",
        )?);
        println!("{:#?}", HlCode::load(&mut reader)?);
        Ok(())
    }
}
