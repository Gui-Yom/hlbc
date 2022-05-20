use std::ffi::CString;
use std::io::Write;

use anyhow::Result;
use byteorder::{LittleEndian, WriteBytesExt};

use crate::types::TypeFun;
use crate::{ConstantDef, Function, Native, ObjField, Opcode, Type, TypeObj};

pub(crate) trait WriteHlExt: WriteBytesExt {
    fn write_vi32(&mut self, value: i32) -> Result<()>;
    fn write_strings(&mut self, strings: &[String]) -> Result<()>;
    fn write_type_fun(&mut self, fun: &TypeFun) -> Result<()>;
    fn write_field(&mut self, field: &ObjField) -> Result<()>;
    fn write_type_obj(&mut self, obj: &TypeObj) -> Result<()>;
    fn write_type(&mut self, ty: &Type) -> Result<()>;
    fn write_native(&mut self, n: &Native) -> Result<()>;
    fn write_function(&mut self, f: &Function) -> Result<()>;
    fn write_constant_def(&mut self, c: &ConstantDef) -> Result<()>;
}

impl<T: Write> WriteHlExt for T {
    fn write_vi32(&mut self, value: i32) -> Result<()> {
        if value < 0 {
            let value = -value;
            if value < 0x2000 {
                self.write_u8(((value >> 8) | 0xA0) as u8)?;
                self.write_u8((value & 0xFF) as u8)?;
            } else if value >= 20000000 {
                anyhow::bail!("value can't be >= 0x20000000")
            } else {
                self.write_u8(((value >> 24) | 0xE0) as u8)?;
                self.write_u8(((value >> 16) | 0xFF) as u8)?;
                self.write_u8(((value >> 8) | 0xFF) as u8)?;
                self.write_u8((value & 0xFF) as u8)?;
            }
        } else if value < 0x80 {
            self.write_u8(value as u8)?;
        } else if value < 0x2000 {
            self.write_u8(((value >> 8) | 0x80) as u8)?;
            self.write_u8((value & 0xFF) as u8)?;
        } else if value >= 0x20000000 {
            anyhow::bail!("value can't be >= 0x20000000")
        } else {
            self.write_u8(((value >> 24) | 0xC0) as u8)?;
            self.write_u8(((value >> 16) | 0xFF) as u8)?;
            self.write_u8(((value >> 8) | 0xFF) as u8)?;
            self.write_u8((value & 0xFF) as u8)?;
        }
        Ok(())
    }

    fn write_strings(&mut self, strings: &[String]) -> Result<()> {
        let cstr: Vec<CString> = strings
            .iter()
            .map(|s| CString::new(s.as_bytes()).unwrap())
            .collect();
        let size = cstr
            .iter()
            .map(|s| s.as_bytes_with_nul().len())
            .reduce(|acc, len| acc + len)
            .unwrap_or(0);
        self.write_i32::<LittleEndian>(size as i32)?;
        for s in cstr.iter() {
            self.write(s.as_bytes_with_nul())?;
        }
        for s in cstr.iter() {
            self.write_vi32(s.as_bytes().len() as i32)?;
        }
        Ok(())
    }

    fn write_type_fun(&mut self, fun: &TypeFun) -> Result<()> {
        self.write_u8(fun.args.len() as u8)?;
        for a in &fun.args {
            self.write_vi32(a.0 as i32)?;
        }
        self.write_vi32(fun.ret.0 as i32)?;
        Ok(())
    }

    fn write_field(&mut self, field: &ObjField) -> Result<()> {
        self.write_vi32(field.name.0 as i32)?;
        self.write_vi32(field.t.0 as i32)?;
        Ok(())
    }

    fn write_type_obj(&mut self, obj: &TypeObj) -> Result<()> {
        self.write_vi32(obj.name.0 as i32)?;
        self.write_vi32(obj.super_.map(|s| s.0 as i32).unwrap_or(-1))?;
        self.write_vi32(obj.global.0 as i32)?;
        self.write_vi32(obj.own_fields.len() as i32)?;
        self.write_vi32(obj.protos.len() as i32)?;
        self.write_vi32(obj.bindings.len() as i32)?;
        for f in &obj.own_fields {
            self.write_field(f)?;
        }
        for p in &obj.protos {
            self.write_vi32(p.name.0 as i32)?;
            self.write_vi32(p.findex.0 as i32)?;
            self.write_vi32(p.pindex)?;
        }
        for (fi, fun) in &obj.bindings {
            self.write_vi32(fi.0 as i32)?;
            self.write_vi32(fun.0 as i32)?;
        }
        Ok(())
    }

    fn write_type(&mut self, ty: &Type) -> Result<()> {
        match ty {
            Type::Void => self.write_u8(0)?,
            Type::UI8 => self.write_u8(1)?,
            Type::UI16 => self.write_u8(2)?,
            Type::I32 => self.write_u8(3)?,
            Type::I64 => self.write_u8(4)?,
            Type::F32 => self.write_u8(5)?,
            Type::F64 => self.write_u8(6)?,
            Type::Bool => self.write_u8(7)?,
            Type::Bytes => self.write_u8(8)?,
            Type::Dyn => self.write_u8(9)?,
            Type::Fun(fun) => {
                self.write_u8(10)?;
                self.write_type_fun(fun)?;
            }
            Type::Obj(obj) => {
                self.write_u8(11)?;
                self.write_type_obj(obj)?;
            }
            Type::Array => self.write_u8(12)?,
            Type::Type => self.write_u8(13)?,
            Type::Ref(inner) => {
                self.write_u8(14)?;
                self.write_vi32(inner.0 as i32)?;
            }
            Type::Virtual { fields } => {
                self.write_u8(15)?;
                self.write_vi32(fields.len() as i32)?;
                for f in fields {
                    self.write_field(f)?;
                }
            }
            Type::DynObj => self.write_u8(16)?,
            Type::Abstract { name } => {
                self.write_u8(17)?;
                self.write_vi32(name.0 as i32)?;
            }
            Type::Enum {
                name,
                global,
                constructs,
            } => {
                self.write_u8(18)?;
                self.write_vi32(name.0 as i32)?;
                self.write_vi32(global.0 as i32)?;
                self.write_vi32(constructs.len() as i32)?;
                for c in constructs {
                    self.write_vi32(c.name.0 as i32)?;
                    self.write_vi32(c.params.len() as i32)?;
                    for p in &c.params {
                        self.write_vi32(p.0 as i32)?;
                    }
                }
            }
            Type::Null(inner) => {
                self.write_u8(19)?;
                self.write_vi32(inner.0 as i32)?;
            }
            Type::Method(fun) => {
                self.write_u8(20)?;
                self.write_type_fun(fun)?;
            }
            Type::Struct(obj) => {
                self.write_u8(21)?;
                self.write_type_obj(obj)?;
            }
        }
        Ok(())
    }

    fn write_native(&mut self, n: &Native) -> Result<()> {
        self.write_vi32(n.lib.0 as i32)?;
        self.write_vi32(n.name.0 as i32)?;
        self.write_vi32(n.t.0 as i32)?;
        self.write_vi32(n.findex.0 as i32)?;
        Ok(())
    }

    fn write_function(&mut self, f: &Function) -> Result<()> {
        self.write_vi32(f.t.0 as i32)?;
        self.write_vi32(f.findex.0 as i32)?;
        self.write_vi32(f.regs.len() as i32)?;
        self.write_vi32(f.ops.len() as i32)?;
        for r in &f.regs {
            self.write_vi32(r.0 as i32)?;
        }
        for o in &f.ops {
            o.encode(self)?;
        }
        if let Some(debug_info) = &f.debug_info {
            let mut curfile: i32 = -1;
            let mut curpos = 0;
            let mut rcount = 0;
            for &(f, p) in debug_info {
                if f as i32 != curfile {
                    flush_repeat(self, &mut curpos, &mut rcount, p)?;
                    curfile = f as i32;
                    self.write_u8(((f >> 7) | 1) as u8)?;
                    self.write_u8((f & 0xFF) as u8)?;
                }
                if p != curpos {
                    flush_repeat(self, &mut curpos, &mut rcount, p)?;
                }
                if p == curpos {
                    rcount += 1;
                } else {
                    let delta = p as i32 - curpos as i32;
                    if delta > 0 && delta < 32 {
                        self.write_u8(((delta << 3) | 4) as u8)?;
                    } else {
                        self.write_u8((p << 3) as u8)?;
                        self.write_u8((p << 5) as u8)?;
                        self.write_u8((p << 13) as u8)?;
                    }
                    curpos = p;
                }
            }
            let old_curpos = curpos;
            flush_repeat(self, &mut curpos, &mut rcount, old_curpos)?;
        }
        if let Some(assigns) = &f.assigns {
            self.write_vi32(assigns.len() as i32)?;
            for (s, p) in assigns {
                self.write_vi32(s.0 as i32)?;
                self.write_vi32(*p as i32)?;
            }
        }
        Ok(())
    }

    fn write_constant_def(&mut self, c: &ConstantDef) -> Result<()> {
        self.write_vi32(c.global.0 as i32)?;
        self.write_vi32(c.fields.len() as i32)?;
        for f in &c.fields {
            self.write_vi32(*f as i32)?;
        }
        Ok(())
    }
}

fn flush_repeat(
    w: &mut impl Write,
    curpos: &mut usize,
    rcount: &mut usize,
    pos: usize,
) -> Result<()> {
    if *rcount > 0 {
        if *rcount > 15 {
            w.write_u8((15 << 2) | 2)?;
            *rcount -= 15;
            flush_repeat(w, curpos, rcount, pos)?;
        } else {
            let mut delta = pos as i32 - *curpos as i32;
            delta = if delta > 0 && delta < 4 { delta } else { 0 };
            w.write_u8(((delta << 6) | ((*rcount as i32) << 2) | 2) as u8)?;
            *rcount = 0;
            *curpos += delta as usize;
        }
    }
    Ok(())
}
