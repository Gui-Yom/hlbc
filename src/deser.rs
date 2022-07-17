use std::collections::HashMap;
use std::ffi::CStr;
use std::io::Read;

use byteorder::{LittleEndian, ReadBytesExt};

use crate::types::{
    EnumConstruct, Function, Native, ObjField, ObjProto, RefField, RefString, RefType, Type,
    TypeFun, TypeObj,
};
use crate::{ConstantDef, Opcode, RefFun, RefGlobal};
use crate::{Error, Result};

/// Extension trait to read bytecode elements from anything that implements [Read]
pub trait ReadHlExt: ReadBytesExt {
    /// Read a variable size signed integer. Used internally by the other functions.
    fn read_vari(&mut self) -> Result<i32>;
    /// Read a variable size unsigned integer. Used internally by the other functions.
    fn read_varu(&mut self) -> Result<u32>;
    /// Read a strings block
    fn read_strings(&mut self, nstrings: usize) -> Result<Vec<String>>;
    /// Read a field definition
    fn read_field(&mut self) -> Result<ObjField>;
    /// Read a type reference
    fn read_type_ref(&mut self) -> Result<RefType>;
    /// Read a Fun or Method type
    fn read_type_fun(&mut self) -> Result<TypeFun>;
    /// Read a Obj or Struct type
    fn read_type_obj(&mut self) -> Result<TypeObj>;
    /// Read a type definition
    fn read_type(&mut self) -> Result<Type>;
    /// Read a native function definition
    fn read_native(&mut self) -> Result<Native>;
    /// Read a function definition
    fn read_function(&mut self, has_debug: bool, version: u8) -> Result<Function>;
    /// Read a constant definition
    fn read_constant_def(&mut self) -> Result<ConstantDef>;
}

impl<T: Read> ReadHlExt for T {
    fn read_vari(&mut self) -> Result<i32> {
        let b = self.read_u8()? as i32;
        if b & 0x80 == 0 {
            Ok(b & 0x7F)
        } else if b & 0x40 == 0 {
            let v = self.read_u8()? as i32 | ((b & 31) << 8);
            Ok(if b & 0x20 == 0 { v } else { -v })
        } else {
            let c = self.read_u8()? as i32;
            let d = self.read_u8()? as i32;
            let e = self.read_u8()? as i32;
            let v = ((b & 31) << 24) | (c << 16) | (d << 8) | e;
            Ok(if b & 0x20 == 0 { v } else { -v })
        }
    }

    fn read_varu(&mut self) -> Result<u32> {
        let i = self.read_vari()?;
        if i < 0 {
            Err(Error::MalformedBytecode(format!(
                "Got negative index '{i}' (expected > 0)"
            )))
        } else {
            Ok(i as u32)
        }
    }

    fn read_strings(&mut self, nstrings: usize) -> Result<Vec<String>> {
        let mut strings = Vec::with_capacity(nstrings);
        let mut string_data = vec![0u8; self.read_i32::<LittleEndian>()? as usize];
        self.read_exact(&mut string_data)?;
        let mut acc = 0;
        for _ in 0..nstrings {
            let ssize = self.read_varu()? as usize + 1;
            //println!("size: {ssize} {:?}", &string_data[acc..(acc + ssize)]);
            let cstr =
                unsafe { CStr::from_bytes_with_nul_unchecked(&string_data[acc..(acc + ssize)]) };
            strings.push(cstr.to_string_lossy().to_string());
            acc += ssize;
        }
        Ok(strings)
    }

    fn read_field(&mut self) -> Result<ObjField> {
        Ok(ObjField {
            name: RefString(self.read_vari()? as usize),
            t: self.read_type_ref()?,
        })
    }

    fn read_type_ref(&mut self) -> Result<RefType> {
        Ok(RefType(self.read_vari()? as usize))
    }

    fn read_type_fun(&mut self) -> Result<TypeFun> {
        let nargs = self.read_u8()?;
        let mut args = Vec::with_capacity(nargs as usize);
        for _ in 0..nargs {
            args.push(self.read_type_ref()?);
        }
        Ok(TypeFun {
            args,
            ret: self.read_type_ref()?,
        })
    }

    fn read_type_obj(&mut self) -> Result<TypeObj> {
        let name = RefString(self.read_vari()? as usize);
        let super_ = self.read_vari()?;
        let global = RefGlobal(self.read_varu()? as usize);
        let nfields = self.read_varu()? as usize;
        let nprotos = self.read_varu()? as usize;
        let nbindings = self.read_varu()? as usize;
        let mut own_fields = Vec::with_capacity(nfields);
        for _ in 0..nfields {
            own_fields.push(self.read_field()?);
        }
        let mut protos = Vec::with_capacity(nprotos);
        for _ in 0..nprotos {
            protos.push(ObjProto {
                name: RefString(self.read_vari()? as usize),
                findex: RefFun(self.read_varu()? as usize),
                pindex: self.read_vari()?,
            });
        }
        let mut bindings = HashMap::with_capacity(nbindings);
        for _ in 0..nbindings {
            bindings.insert(
                RefField(self.read_varu()? as usize),
                RefFun(self.read_varu()? as usize),
            );
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
            fields: Vec::with_capacity(0),
            protos,
            bindings,
        })
    }

    fn read_type(&mut self) -> Result<Type> {
        match self.read_u8()? {
            0 => Ok(Type::Void),
            1 => Ok(Type::UI8),
            2 => Ok(Type::UI16),
            3 => Ok(Type::I32),
            4 => Ok(Type::I64),
            5 => Ok(Type::F32),
            6 => Ok(Type::F64),
            7 => Ok(Type::Bool),
            8 => Ok(Type::Bytes),
            9 => Ok(Type::Dyn),
            10 => Ok(Type::Fun(self.read_type_fun()?)),
            11 => Ok(Type::Obj(self.read_type_obj()?)),
            12 => Ok(Type::Array),
            13 => Ok(Type::Type),
            14 => Ok(Type::Ref(self.read_type_ref()?)),
            15 => {
                let nfields = self.read_varu()? as usize;
                let mut fields = Vec::with_capacity(nfields);
                for _ in 0..nfields {
                    fields.push(self.read_field()?);
                }
                Ok(Type::Virtual { fields })
            }
            16 => Ok(Type::DynObj),
            17 => Ok(Type::Abstract {
                name: RefString(self.read_vari()? as usize),
            }),
            18 => {
                let name = RefString(self.read_vari()? as usize);
                let global = RefGlobal(self.read_varu()? as usize);
                let nconstructs = self.read_varu()? as usize;
                let mut constructs = Vec::with_capacity(nconstructs);
                for _ in 0..nconstructs {
                    let name = RefString(self.read_vari()? as usize);
                    let nparams = self.read_varu()? as usize;
                    let mut params = Vec::with_capacity(nparams);
                    for _ in 0..nparams {
                        params.push(self.read_type_ref()?);
                    }
                    constructs.push(EnumConstruct { name, params })
                }
                Ok(Type::Enum {
                    name,
                    global,
                    constructs,
                })
            }
            19 => Ok(Type::Null(self.read_type_ref()?)),
            20 => Ok(Type::Method(self.read_type_fun()?)),
            21 => Ok(Type::Struct(self.read_type_obj()?)),
            other => Err(Error::MalformedBytecode(format!(
                "Invalid type kind '{other}'"
            ))),
        }
    }

    fn read_native(&mut self) -> Result<Native> {
        Ok(Native {
            lib: RefString(self.read_vari()? as usize),
            name: RefString(self.read_vari()? as usize),
            t: self.read_type_ref()?,
            findex: RefFun(self.read_varu()? as usize),
        })
    }

    fn read_function(&mut self, has_debug: bool, version: u8) -> Result<Function> {
        let t = self.read_type_ref()?;
        let findex = RefFun(self.read_varu()? as usize);
        let nregs = self.read_varu()? as usize;
        let nops = self.read_varu()? as usize;
        let mut regs = Vec::with_capacity(nregs);
        for _ in 0..nregs {
            regs.push(self.read_type_ref()?);
        }
        let mut ops = Vec::with_capacity(nops);
        for _ in 0..nops {
            ops.push(Opcode::decode(self)?);
        }

        // This is extracted from the hashlink source code, do not count on me to explain what it does
        let debug_info = if has_debug {
            let mut tmp = Vec::with_capacity(nops);
            let mut currfile: i32 = -1;
            let mut currline: i32 = 0;
            let mut i = 0;
            while i < nops {
                let mut c = self.read_u8()? as i32;
                if c & 1 != 0 {
                    c >>= 1;
                    currfile = (c << 8) | (self.read_u8()? as i32);
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
                    let b2 = self.read_u8()? as i32;
                    let b3 = self.read_u8()? as i32;
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
            let len = self.read_varu()? as usize;
            let mut assigns = Vec::with_capacity(len);
            for _ in 0..len {
                assigns.push((
                    RefString(self.read_varu()? as usize),
                    self.read_vari()? as usize,
                ));
            }
            Some(assigns)
        } else {
            None
        };
        Ok(Function {
            name: None,
            t,
            findex,
            regs,
            ops,
            debug_info,
            assigns,
            parent: None,
        })
    }

    fn read_constant_def(&mut self) -> Result<ConstantDef> {
        let global = RefGlobal(self.read_varu()? as usize);
        let nfields = self.read_varu()? as usize;
        let mut fields = Vec::with_capacity(nfields);
        for _ in 0..nfields {
            fields.push(self.read_varu()? as usize);
        }
        Ok(ConstantDef { global, fields })
    }
}
