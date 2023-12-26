use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Expr, ExprLit, GenericArgument, Ident, Lit, LitStr, PathArguments, Type,
    Variant,
};

#[proc_macro_derive(OpcodeHelper)]
pub fn derive_opcode_helper(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let variants = match &ast.data {
        Data::Enum(v) => Some(&v.variants),
        _ => None,
    }
    .unwrap();

    let name = &ast.ident;
    let i = 0..variants.len() as u8;

    let initr = variants.iter().map(|v| read_variant(name, v));
    let initw = variants
        .iter()
        .enumerate()
        .map(|(i, v)| write_variant(name, v, i as u8));
    let vname = variants.iter().map(|v| &v.ident);
    let vname2 = vname.clone();
    let vname_str = variants
        .iter()
        .map(|v| LitStr::new(&v.ident.to_string(), v.ident.span()));
    let vname_str2 = vname_str.clone();
    let vdesc = variants.iter().map(|v| {
        let mut acc = String::new();
        for attr in &v.attrs {
            if let Ok(nv) = attr.meta.require_name_value() {
                if nv.path.is_ident("doc") {
                    match &nv.value {
                        Expr::Lit(ExprLit {
                            lit: Lit::Str(lit), ..
                        }) => {
                            let lstr = lit.value();
                            let to_acc = lstr.trim();
                            if !to_acc.is_empty() {
                                acc.push_str(to_acc);
                                acc.push('\n');
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        acc.trim().to_string()
    });
    let vdefault_init = variants.iter().map(|v| {
        let vname = &v.ident;
        let finit = v.fields.iter().map(|f| {
            let fname = f.ident.as_ref().unwrap();
            quote! {
                #fname: Default::default()
            }
        });
        quote! {
            #name::#vname { #( #finit,)* }
        }
    });

    proc_macro::TokenStream::from(quote! {
        impl #name {
            /// Decode an instruction
            pub fn read(r: &mut impl std::io::Read) -> crate::Result<#name> {

                use byteorder::ReadBytesExt;
                use crate::types::*;
                use crate::read::{read_vari, read_varu};

                let op = r.read_u8()?;
                match op {
                    #( #i => #initr, )*
                    other => Err(crate::Error::MalformedBytecode(format!("Unknown opcode {}", op))),
                }
            }

            /// Encode an instruction
            pub fn write(&self, w: &mut impl std::io::Write) -> crate::Result<()> {

                use byteorder::WriteBytesExt;
                use crate::types::*;
                use crate::write::write_var;

                match self {
                    #( #initw )*
                }

                Ok(())
            }

            /// Get the opcode name
            pub fn name(&self) -> &'static str {
                match self {
                    #( #name::#vname { .. } => #vname_str, )*
                }
            }

            /// Get the opcode description
            pub fn description(&self) -> &'static str {
                match self {
                    #( #name::#vname2 { .. } => #vdesc, )*
                }
            }

            /// Get an opcode from its name. Returns a default value for the variant.
            pub fn from_name(name: &str) -> Option<Self> {
                match name {
                    #( #vname_str2 => Some(#vdefault_init), )*
                    _ => None
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
        other => unreachable!("unknown type {:?}", other),
    }
}

fn read_variant(enum_name: &Ident, v: &Variant) -> TokenStream {
    let rvi32 = quote!(read_vari(r)?);
    let rvu32 = quote!(read_varu(r)?);
    let reg = quote!(Reg(#rvi32 as u32));

    let vname = &v.ident;
    let fname = v.fields.iter().map(|f| &f.ident);
    let fvalue = v.fields.iter().map(|f| match ident(&f.ty).as_str() {
        "InlineBool" => quote! {
            #rvi32 == 1
        },
        "InlineInt" => quote! {
            #rvi32
        },
        "JumpOffset" => quote! {
            #rvi32
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
            RefInt::read(r)?
        },
        "RefFloat" => quote! {
            RefFloat::read(r)?
        },
        "RefBytes" => quote! {
            RefBytes(#rvi32 as usize)
        },
        "RefString" => quote! {
            RefString::read(r)?
        },
        "RefType" => quote! {
            RefType::read(r)?
        },
        "RefFun" => quote! {
            RefFun::read(r)?
        },
        "RefField" => quote! {
            RefField::read(r)?
        },
        "RefGlobal" => quote! {
            RefGlobal::read(r)?
        },
        "RefEnumConstruct" => quote! {
            RefEnumConstruct(#rvi32 as usize)
        },
        _ => TokenStream::default(),
    });
    quote! {
        Ok(#enum_name::#vname {
            #( #fname: #fvalue, )*
        })
    }
}

fn write_variant(enum_name: &Ident, v: &Variant, i: u8) -> TokenStream {
    let vname = &v.ident;
    let fname = v.fields.iter().map(|f| &f.ident);
    let fwrite = v.fields.iter().map(|f| {
        let fname = f.ident.as_ref().unwrap();
        match ident(&f.ty).as_str() {
            "usize" => quote!(write_var(w, #fname as i32)?;),
            "i32" => quote! {
                write_var(w, #fname)?;
            },
            "JumpOffset" => quote! {
                write_var(w, *#fname as i32)?;
            },
            "Vec<JumpOffset>" => quote! {
                {
                    write_var(w, #fname.len() as i32)?;
                    for r__ in #fname {
                        write_var(w, *r__ as i32)?;
                    }
                }
            },
            "Reg" => quote! {
                write_var(w, #fname.0 as i32)?;
            },
            "Vec<Reg>" => quote! {
                {
                    w.write_u8(#fname.len() as u8)?;
                    for r__ in #fname {
                        write_var(w, r__.0 as i32)?;
                    }
                }
            },
            "RefInt" | "RefFloat" | "RefString" | "RefType" | "RefFun" | "RefField"
            | "RefGlobal" => quote! {
                #fname.write(w)?;
            },
            "RefBytes" => quote! {
                write_var(w, #fname.0 as i32)?;
            },
            "ValBool" => quote! {
                write_var(w, if #fname.0 { 1 } else { 0 })?;
            },
            "RefEnumConstruct" => quote! {
                write_var(w, #fname.0 as i32)?;
            },
            _ => TokenStream::default(),
        }
    });
    quote! {
        #enum_name::#vname { #( #fname, )* } => {
            w.write_u8(#i)?;
            #( #fwrite )*
        }
    }
}
