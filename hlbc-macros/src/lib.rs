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

    let tyusize = parse_str::<Type>("usize").unwrap();
    let tyJumpOffset = parse_str::<Type>("JumpOffset").unwrap();
    let tyReg = parse_str::<Type>("Reg").unwrap();
    let tyVecReg = parse_str::<Type>("Vec<Reg>").unwrap();
    let tyConstInt = parse_str::<Type>("ConstInt").unwrap();
    let tyConstFloat = parse_str::<Type>("ConstFloat").unwrap();
    let tyConstBytes = parse_str::<Type>("ConstBytes").unwrap();
    let tyConstString = parse_str::<Type>("ConstString").unwrap();
    let tyConstType = parse_str::<Type>("ConstType").unwrap();
    let tyValBool = parse_str::<Type>("ValBool").unwrap();
    let tyFun = parse_str::<Type>("Fun").unwrap();
    let tyField = parse_str::<Type>("Field").unwrap();
    let tyGlobal = parse_str::<Type>("Global").unwrap();

    let name = &ast.ident;
    let i = 0..variants.len() as u8;

    let init = variants.iter().map(|v| {
        if v.ident == "CallMethod" {
            quote! {
                {
                    let dst = Reg(vari32(r)? as u32);
                    let obj = Reg(vari32(r)? as u32);
                    let n = r.read_u8()? as usize;
                    let field = Reg(vari32(r)? as u32);
                    let mut args = Vec::with_capacity(n-1);
                    for _ in 1..n {
                        args.push(Reg(vari32(r)? as u32));
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
                    let reg = Reg(varu32(r)?);
                    let n = varu32(r)? as usize;
                    let mut offsets = Vec::with_capacity(n);
                    for _ in 0..n {
                        offsets.push(varu32(r)? as JumpOffset);
                    }
                    let end = varu32(r)? as JumpOffset;
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
                if f.ty == tyusize {
                    quote! {
                        vari32(r)? as usize
                    }
                } else if f.ty == tyJumpOffset {
                    quote! {
                        vari32(r)? as JumpOffset
                    }
                } else if f.ty == tyReg {
                    quote! {
                        Reg(vari32(r)? as u32)
                    }
                } else if f.ty == tyVecReg {
                    quote! {
                        {
                            let n = r.read_u8()? as usize;
                            let mut regs = Vec::with_capacity(n);
                            for _ in 0..n {
                                regs.push(Reg(vari32(r)? as u32));
                            }
                            regs
                        }
                    }
                } else if f.ty == tyConstInt {
                    quote! {
                        ConstInt(vari32(r)? as usize)
                    }
                } else if f.ty == tyConstFloat {
                    quote! {
                        ConstFloat(vari32(r)? as usize)
                    }
                } else if f.ty == tyConstBytes {
                    quote! {
                        ConstBytes(vari32(r)? as usize)
                    }
                } else if f.ty == tyConstString {
                    quote! {
                        ConstString(vari32(r)? as usize)
                    }
                } else if f.ty == tyConstType {
                    quote! {
                        ConstType(vari32(r)? as usize)
                    }
                } else if f.ty == tyValBool {
                    quote! {
                        ValBool(vari32(r)? == 1)
                    }
                } else if f.ty == tyFun {
                    quote! {
                        Fun(vari32(r)? as usize)
                    }
                } else if f.ty == tyField {
                    quote! {
                        Field(vari32(r)? as usize)
                    }
                } else if f.ty == tyGlobal {
                    quote! {
                        Global(vari32(r)? as usize)
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
                use crate::utils::{vari32, varu32};

                let op = r.read_u8()?;
                match op {
                    #( #i => #init, )*
                    other => anyhow::bail!("Unknown opcode {}", op),
                }
            }
        }
    })
}
