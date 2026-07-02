#![allow(unused)]
use proc_macro::TokenStream;
use quote::quote;
use std::{collections::HashMap, env, fs, path::PathBuf};
use syn::{Item, ItemEnum, ItemMod, ItemStruct, LitStr, parse2, spanned::Spanned};
use toml::Value;

//the inner function just lets us return a result, and then have the error case of that result
//turned into a neat compiler error.
#[proc_macro_attribute]
pub fn from_toml(args: TokenStream, input: TokenStream) -> TokenStream {
    from_toml_inner(args.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

fn from_toml_inner(
    args: proc_macro2::TokenStream,
    input: proc_macro2::TokenStream,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let path: LitStr = parse2(args)?;
    let module: ItemMod = parse2(input)?;

    eprintln!("{}", path.value());
    eprintln!("{:#?}", module);

    //parse the toml, returning any errors in opening the file or parsing as compile errors
    let toml_path =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"))
            .join(path.value());
    let content =
        fs::read_to_string(&toml_path).map_err(|e| syn::Error::new(path.span(), e.to_string()))?;
    let toml_value: Value =
        toml::from_str(&content).map_err(|e| syn::Error::new(path.span(), e.to_string()))?;
    eprintln!("{:#?}", toml_value);

    let items = &module
        .content
        .as_ref()
        .ok_or_else(|| syn::Error::new(module.span(), "Module must have content"))?
        .1;
    let (root_structs, other_items): (Vec<_>, Vec<_>) = items.iter().partition(|item| {
        if let Item::Struct(s) = item {
            s.attrs.iter().any(|a| a.path().is_ident("root"))
        } else {
            false
        }
    });

    let root_struct = if root_structs.len() == 1 {
        if let Item::Struct(s) = root_structs[0] {
            let mut s = s.clone();
            s.attrs.retain(|a| !a.path().is_ident("root"));
            s
        } else {
            unreachable!()
        }
    } else {
        return Err(syn::Error::new(
            module.span(),
            "Must have exactly one struct with `#[root] in the config module.",
        ));
    };

    eprintln!("{:#?}", other_items);
    eprintln!("{:#?}", root_struct);

    let mut defs: HashMap<String, TypeDef> = HashMap::new();
    for item in other_items {
        match item {
            Item::Struct(s) => {
                defs.insert(s.ident.to_string(), TypeDef::Struct(s));
            }
            Item::Enum(e) => {
                defs.insert(e.ident.to_string(), TypeDef::Enum(e));
            }
            i => {
                return Err(syn::Error::new(
                    i.span(),
                    "Can only have structs and enums in the toml_config module.",
                ));
            }
        }
    }
    defs.insert(root_struct.ident.to_string(), TypeDef::Struct(&root_struct));

    eprintln!("{:#?}", defs);

    todo!()
}

#[derive(Debug)]
enum TypeDef<'a> {
    Struct(&'a ItemStruct),
    Enum(&'a ItemEnum),
}

fn render_value(
    value: &Value,
    ty: &syn::Type,
    defs: &HashMap<String, TypeDef>,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    match ty {
        syn::Type::Array(array) => todo!(),
        syn::Type::Path(path) => {
            match path
                .path
                .segments
                .last()
                .ok_or_else(|| syn::Error::new(path.span(), "Empty path?"))?
                .ident
                .to_string()
                .as_str()
            {
                "u8" => {
                    if let Value::Integer(n) = value {
                        let n = *n as u8;
                        Ok(quote! {#n})
                    } else {
                        Err(syn::Error::new(path.span(), "Toml value is not an integer"))
                    }
                }
                "u16" => {
                    if let Value::Integer(n) = value {
                        let n = *n as u16;
                        Ok(quote! {#n})
                    } else {
                        Err(syn::Error::new(path.span(), "Toml value is not an integer"))
                    }
                }
                "u32" => {
                    if let Value::Integer(n) = value {
                        let n = *n as u32;
                        Ok(quote! {#n})
                    } else {
                        Err(syn::Error::new(path.span(), "Toml value is not an integer"))
                    }
                }
                "u64" => {
                    if let Value::Integer(n) = value {
                        let n = *n as u64;
                        Ok(quote! {#n})
                    } else {
                        Err(syn::Error::new(path.span(), "Toml value is not an integer"))
                    }
                }
                "i8" => {
                    if let Value::Integer(n) = value {
                        let n = *n as i8;
                        Ok(quote! {#n})
                    } else {
                        Err(syn::Error::new(path.span(), "Toml value is not an integer"))
                    }
                }
                "i16" => {
                    if let Value::Integer(n) = value {
                        let n = *n as i16;
                        Ok(quote! {#n})
                    } else {
                        Err(syn::Error::new(path.span(), "Toml value is not an integer"))
                    }
                }
                "i32" => {
                    if let Value::Integer(n) = value {
                        let n = *n as i32;
                        Ok(quote! {#n})
                    } else {
                        Err(syn::Error::new(path.span(), "Toml value is not an integer"))
                    }
                }
                "i64" => {
                    if let Value::Integer(n) = value {
                        Ok(quote! {#n})
                    } else {
                        Err(syn::Error::new(path.span(), "Toml value is not an integer"))
                    }
                }
                "f32" => {
                    if let Value::Float(n) = value {
                        let n = *n as f32;
                        Ok(quote! {#n})
                    } else {
                        Err(syn::Error::new(path.span(), "Toml value is not a float"))
                    }
                }
                "f64" => {
                    if let Value::Float(n) = value {
                        Ok(quote! {#n})
                    } else {
                        Err(syn::Error::new(path.span(), "Toml value is not a float"))
                    }
                }
                "bool" => {
                    if let Value::Boolean(b) = value {
                        Ok(quote! {#b})
                    } else {
                        Err(syn::Error::new(path.span(), "Toml value is not a bool"))
                    }
                }
                i => {
                    if let Some(i) = defs.get(i) {
                        todo!()
                    } else {
                        Err(syn::Error::new(path.span(), "Path not in defs."))
                    }
                }
            }
        }
        syn::Type::Reference(reference) => todo!(),
        syn::Type::Tuple(tupe) => todo!(),
        _ => Err(syn::Error::new(ty.span(), "Unssuported type")),
    }
}
