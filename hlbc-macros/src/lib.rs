use proc_macro::TokenStream;

use quote::quote;
use syn::__private::TokenStream2;
use syn::{parse_str, Data, DeriveInput, Type};

#[proc_macro_attribute]
pub fn gen_decode(attr: TokenStream, input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as DeriveInput);
    let variants = match &ast.data {
        Data::Enum(v) => Some(&v.variants),
        _ => None,
    }
    .unwrap();

    let ty_usize = parse_str::<Type>("usize").unwrap();
    let ty_jump_offset = parse_str::<Type>("JumpOffset").unwrap();
    let ty_reg = parse_str::<Type>("Reg").unwrap();
    let ty_vec_reg = parse_str::<Type>("Vec<Reg>").unwrap();
    let ty_ref_int = parse_str::<Type>("RefInt").unwrap();
    let ty_ref_float = parse_str::<Type>("RefFloat").unwrap();
    let ty_ref_bytes = parse_str::<Type>("RefBytes").unwrap();
    let ty_ref_string = parse_str::<Type>("RefString").unwrap();
    let ty_ref_type = parse_str::<Type>("RefType").unwrap();
    let ty_val_bool = parse_str::<Type>("ValBool").unwrap();
    let ty_ref_fun = parse_str::<Type>("RefFun").unwrap();
    let ty_ref_field = parse_str::<Type>("RefField").unwrap();
    let ty_ref_global = parse_str::<Type>("RefGlobal").unwrap();

    let rvi32 = quote! {
        r.read_vari()?
    };
    let rvu32 = quote! {
        r.read_varu()?
    };
    let reg = quote! {
        Reg(#rvi32 as u32)
    };

    let name = &ast.ident;
    let i = 0..variants.len() as u8;

    let init = variants.iter().map(|v| {
        if v.ident == "CallMethod" {
            quote! {
                {
                    let dst = #reg;
                    let obj = #reg;
                    let n = r.read_u8()? as usize;
                    let field = #reg;
                    let mut args = Vec::with_capacity(n-1);
                    for _ in 1..n {
                        args.push(#reg);
                    }
                    Ok(#name::CallMethod {
                        dst,
                        obj,
                        field,
                        args
                    })
                }
            }
        } else if v.ident == "Switch" {
            quote! {
                {
                    let reg = Reg(#rvu32);
                    let n = #rvu32 as usize;
                    let mut offsets = Vec::with_capacity(n);
                    for _ in 0..n {
                        offsets.push(#rvu32 as JumpOffset);
                    }
                    let end = #rvu32 as JumpOffset;
                    Ok(#name::Switch {
                        reg,
                        offsets,
                        end
                    })
                }
            }
        } else {
            let vname = &v.ident;
            let fname = v.fields.iter().map(|f| &f.ident);
            let fvalue = v.fields.iter().map(|f| {
                if f.ty == ty_usize {
                    quote! {
                        #rvi32 as usize
                    }
                } else if f.ty == ty_jump_offset {
                    quote! {
                        #rvi32 as JumpOffset
                    }
                } else if f.ty == ty_reg {
                    reg.clone()
                } else if f.ty == ty_vec_reg {
                    quote! {
                        {
                            let n = r.read_u8()? as usize;
                            let mut regs = Vec::with_capacity(n);
                            for _ in 0..n {
                                regs.push(#reg);
                            }
                            regs
                        }
                    }
                } else if f.ty == ty_ref_int {
                    quote! {
                        RefInt(#rvi32 as usize)
                    }
                } else if f.ty == ty_ref_float {
                    quote! {
                        RefFloat(#rvi32 as usize)
                    }
                } else if f.ty == ty_ref_bytes {
                    quote! {
                        RefBytes(#rvi32 as usize)
                    }
                } else if f.ty == ty_ref_string {
                    quote! {
                        RefString(#rvi32 as usize)
                    }
                } else if f.ty == ty_ref_type {
                    quote! {
                        RefType(#rvi32 as usize)
                    }
                } else if f.ty == ty_val_bool {
                    quote! {
                        ValBool(#rvi32 == 1)
                    }
                } else if f.ty == ty_ref_fun {
                    quote! {
                        RefFun(#rvi32 as usize)
                    }
                } else if f.ty == ty_ref_field {
                    quote! {
                        RefField(#rvi32 as usize)
                    }
                } else if f.ty == ty_ref_global {
                    quote! {
                        RefGlobal(#rvi32 as usize)
                    }
                } else {
                    TokenStream2::default()
                }
            });
            quote! {
                Ok(#name::#vname {
                    #( #fname: #fvalue, )*
                })
            }
        }
    });

    TokenStream::from(quote! {
        #ast
        // Implementation
        impl #name {
            pub fn decode(r: &mut impl std::io::Read) -> anyhow::Result<#name> {

                use byteorder::ReadBytesExt;
                use crate::read::ReadHlExt;
                use crate::types::*;

                let op = r.read_u8()?;
                match op {
                    #( #i => #init, )*
                    other => anyhow::bail!("Unknown opcode {}", op),
                }
            }
        }
    })
}
