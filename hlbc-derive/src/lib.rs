use proc_macro::TokenStream;

use quote::quote;
use syn::__private::TokenStream2;
use syn::{Data, DeriveInput, GenericArgument, Ident, LitStr, PathArguments, Type, Variant};

#[proc_macro_attribute]
pub fn derive_opcode(_: TokenStream, input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let variants = match &ast.data {
        Data::Enum(v) => Some(&v.variants),
        _ => None,
    }
    .unwrap();

    let name = &ast.ident;
    let i = 0..variants.len() as u8;

    let initr = variants.iter().map(|v| gen_initr(name, v));
    let initw = variants
        .iter()
        .enumerate()
        .map(|(i, v)| gen_initw(name, v, i as u8));
    let vname = variants.iter().map(|v| &v.ident);
    let vname_str = variants
        .iter()
        .map(|v| LitStr::new(&v.ident.to_string(), v.ident.span()));

    TokenStream::from(quote! {
        #ast
        // Implementation
        impl #name {
            pub fn decode(r: &mut impl std::io::Read) -> anyhow::Result<#name> {

                use byteorder::ReadBytesExt;
                use crate::deser::ReadHlExt;
                use crate::types::*;

                let op = r.read_u8()?;
                match op {
                    #( #i => #initr, )*
                    other => anyhow::bail!("Unknown opcode {}", op),
                }
            }

            pub fn encode(&self, w: &mut impl std::io::Write) -> anyhow::Result<()> {

                use byteorder::WriteBytesExt;
                use crate::ser::WriteHlExt;
                use crate::types::*;

                match self {
                    #( #initw )*
                }

                Ok(())
            }

            pub fn name(&self) -> &'static str {
                match self {
                    #( #name::#vname { .. } => #vname_str, )*
                }
            }
        }
    })
}

/// Print a type to string
fn ident(ty: &Type) -> String {
    match ty {
        Type::Path(path) => {
            let seg = &path.path.segments[0];
            match &seg.arguments {
                PathArguments::None => seg.ident.to_string(),
                PathArguments::AngleBracketed(a) => {
                    let a = match &a.args[0] {
                        GenericArgument::Type(ty) => ident(ty),
                        _ => unreachable!(),
                    };
                    format!("{}<{}>", seg.ident, a)
                }
                _ => unreachable!(),
            }
        }
        other => unreachable!("unkown type {:?}", other),
    }
}

fn gen_initr(enum_name: &Ident, v: &Variant) -> TokenStream2 {
    let rvi32 = quote! {
        r.read_vari()?
    };
    let rvu32 = quote! {
        r.read_varu()?
    };
    let reg = quote! {
        Reg(#rvi32 as u32)
    };

    let vname = &v.ident;
    let fname = v.fields.iter().map(|f| &f.ident);
    let fvalue = v.fields.iter().map(|f| match ident(&f.ty).as_str() {
        "usize" => quote! {
            #rvi32 as usize
        },
        "i32" => quote! {
            #rvi32 as JumpOffset
        },
        "JumpOffset" => quote! {
            #rvi32 as JumpOffset
        },
        "Vec<JumpOffset>" => quote! {
            {
                let n = #rvu32 as usize;
                let mut offsets = Vec::with_capacity(n);
                for _ in 0..n {
                    offsets.push(#rvu32 as JumpOffset);
                }
                offsets
            }
        },
        "Reg" => reg.clone(),
        "Vec<Reg>" => quote! {
            {
                let n = r.read_u8()? as usize;
                let mut regs = Vec::with_capacity(n);
                for _ in 0..n {
                    regs.push(#reg);
                }
                regs
            }
        },
        "RefInt" => quote! {
            RefInt(#rvi32 as usize)
        },
        "RefFloat" => quote! {
            RefFloat(#rvi32 as usize)
        },
        "RefBytes" => quote! {
            RefBytes(#rvi32 as usize)
        },
        "RefString" => quote! {
            RefString(#rvi32 as usize)
        },
        "RefType" => quote! {
            RefType(#rvi32 as usize)
        },
        "ValBool" => quote! {
            ValBool(#rvi32 == 1)
        },
        "RefFun" => quote! {
            RefFun(#rvi32 as usize)
        },
        "RefField" => quote! {
            RefField(#rvi32 as usize)
        },
        "RefGlobal" => quote! {
            RefGlobal(#rvi32 as usize)
        },
        "RefEnumConstruct" => quote! {
            RefEnumConstruct(#rvi32 as usize)
        },
        _ => TokenStream2::default(),
    });
    quote! {
        Ok(#enum_name::#vname {
            #( #fname: #fvalue, )*
        })
    }
}

fn gen_initw(enum_name: &Ident, v: &Variant, i: u8) -> TokenStream2 {
    let vname = &v.ident;
    let fname = v.fields.iter().map(|f| &f.ident);
    let fwrite = v.fields.iter().map(|f| {
        let fname = f.ident.as_ref().unwrap();
        match ident(&f.ty).as_str() {
            "usize" => quote!(w.write_vi32(#fname as i32)?;),
            "i32" => quote! {
                w.write_vi32(#fname)?;
            },
            "JumpOffset" => quote! {
                w.write_vi32(*#fname as i32)?;
            },
            "Vec<JumpOffset>" => quote! {
                {
                    w.write_vi32(#fname.len() as i32)?;
                    for r__ in #fname {
                        w.write_vi32(*r__ as i32)?;
                    }
                }
            },
            "Reg" => quote! {
                w.write_vi32(#fname.0 as i32)?;
            },
            "Vec<Reg>" => quote! {
                {
                    w.write_u8(#fname.len() as u8)?;
                    for r__ in #fname {
                        w.write_vi32(r__.0 as i32)?;
                    }
                }
            },
            "RefInt" => quote! {
                w.write_vi32(#fname.0 as i32)?;
            },
            "RefFloat" => quote! {
                w.write_vi32(#fname.0 as i32)?;
            },
            "RefBytes" => quote! {
                w.write_vi32(#fname.0 as i32)?;
            },
            "RefString" => quote! {
                w.write_vi32(#fname.0 as i32)?;
            },
            "RefType" => quote! {
                w.write_vi32(#fname.0 as i32)?;
            },
            "ValBool" => quote! {
                w.write_vi32(if #fname.0 { 1 } else { 0 })?;
            },
            "RefFun" => quote! {
                w.write_vi32(#fname.0 as i32)?;
            },
            "RefField" => quote! {
                w.write_vi32(#fname.0 as i32)?;
            },
            "RefGlobal" => quote! {
                w.write_vi32(#fname.0 as i32)?;
            },
            "RefEnumConstruct" => quote! {
                w.write_vi32(#fname.0 as i32)?;
            },
            _ => TokenStream2::default(),
        }
    });
    quote! {
        #enum_name::#vname { #( #fname, )* } => {
            w.write_u8(#i)?;
            #( #fwrite )*
        }
    }
}
