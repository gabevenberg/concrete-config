#![doc= include_str!("../README.md")]
use proc_macro2::TokenStream;
use quote::quote;
use std::{
    collections::{HashMap, HashSet},
    env, fs,
    path::PathBuf,
};
use syn::{
    Error, Expr, ExprLit, Item, ItemEnum, ItemMod, ItemStruct, Lit, LitStr, TypeArray,
    TypeReference, parse2, spanned::Spanned,
};
use toml::{Value, map::Map};

/// Generates a `const` instance of a struct from a TOML file at compile time.
///
/// Attach this to a module. The macro's single argument is the path to a TOML
/// file, relative to `CARGO_MANIFEST_DIR` (i.e. the directory containing your
/// `Cargo.toml`). The module must contain all the type definitions needed to
/// build the config, with exactly one struct marked `#[root]` — that struct
/// corresponds to the root table of the TOML document.
///
/// The macro emits your type definitions unchanged, plus a `pub const CONFIG`
/// of the root type populated from the TOML file. If a TOML key has no matching
/// struct field, a struct field has no matching TOML key, a value is out of
/// range for its target type, or a type otherwise doesn't line up, the macro
/// produces a compile error.
///
/// # Example
///
/// ```rust
/// use concrete_config::concrete_toml;
///
/// #[concrete_toml("tests/single_int.toml")]
/// mod config {
///     #[root]
///     pub struct Config {
///         pub sample_rate: u32,
///     }
/// }
///
/// // Access the generated constant:
/// assert_eq!(config::CONFIG.sample_rate, 48000);
/// ```
///
/// See the [crate-level documentation](crate) for the full list of supported
/// types, limitations, and a complete example.
#[proc_macro_attribute]
pub fn concrete_toml(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    concrete_toml_inner(args.into(), input.into())
        .unwrap_or_else(|e| e.to_compile_error())
        .into()
}

//the inner function just lets us return a result, and then have the error case of that result
//turned into a neat compiler error.
fn concrete_toml_inner(args: TokenStream, input: TokenStream) -> Result<TokenStream, Error> {
    let path: LitStr = parse2(args)?;
    let module: ItemMod = parse2(input)?;

    let toml_path =
        PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"))
            .join(path.value());
    let content =
        fs::read_to_string(&toml_path).map_err(|e| Error::new(path.span(), e.to_string()))?;
    let toml_value: Value =
        toml::from_str(&content).map_err(|e| Error::new(path.span(), e.to_string()))?;

    let items = &module
        .content
        .as_ref()
        .ok_or_else(|| Error::new(module.span(), "Module must have content"))?
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
            // as #[root] is not actually a rust attribute, remove it from the generated code.
            s.attrs.retain(|a| !a.path().is_ident("root"));
            s
        } else {
            unreachable!()
        }
    } else {
        return Err(Error::new(
            module.span(),
            "Must have exactly one struct with `#[root]` in the config module.",
        ));
    };

    let mut defs: HashMap<String, TypeDef> = HashMap::new();
    for item in &other_items {
        match item {
            Item::Struct(s) => {
                defs.insert(s.ident.to_string(), TypeDef::Struct(s));
            }
            Item::Enum(e) => {
                defs.insert(e.ident.to_string(), TypeDef::Enum(e));
            }
            i => {
                return Err(Error::new(
                    i.span(),
                    "Can only have structs and enums in the toml_config module.",
                ));
            }
        }
    }

    let con = render_struct(
        toml_value
            .as_table()
            .expect("root of toml file is always a table"),
        &root_struct,
        &defs,
    )?;
    let root_ident = &root_struct.ident;
    let mod_name = &module.ident;
    let toml_path_str = toml_path.to_str().expect("TOML path is not valid UTF-8");
    let output = quote! {
        mod #mod_name {
            // we allow dead code in here because if there is an enum, most likely not every variant
            // will be constructed for a particular compilation.
            #![allow(dead_code)]
            #root_struct
            #(#other_items)*
            pub const CONFIG: #root_ident = #con;
            // This doesn't include anything in the final binary (at least if any optimization is
            // done), but it does tell cargo that the toml file is a dependency of this file, and
            // that we should rebuild when the toml file is changed.
            const _: usize = include_bytes!(#toml_path_str).len();
        }
    };
    // eprintln!("{}", output);
    Ok(output)
}

#[derive(Debug)]
enum TypeDef<'a> {
    Struct(&'a ItemStruct),
    Enum(&'a ItemEnum),
}

fn render_struct(
    value: &Map<String, Value>,
    st: &ItemStruct,
    defs: &HashMap<String, TypeDef>,
) -> Result<TokenStream, Error> {
    let mut constructed: HashSet<String> = HashSet::new();
    let mut streams: Vec<TokenStream> = Vec::new();
    for field in &st.fields {
        let ident = field
            .ident
            .as_ref()
            .ok_or_else(|| Error::new(field.span(), "Tuple structs are not yet supported"))?;
        let ident_str = ident.to_string();
        if let Some(v) = value.get(&ident_str) {
            let value = render_value(v, &field.ty, defs)?;
            let declaration = quote! {#ident: #value};
            streams.push(declaration);
            constructed.insert(ident_str);
        } else {
            return Err(Error::new(
                st.span(),
                format!("found field with no matching TOML value: {}", ident_str),
            ));
        }
    }
    for entry in value.keys() {
        if !constructed.contains(entry) {
            return Err(Error::new(
                st.span(),
                format!("Unrecognized key in TOML table: {}", entry),
            ));
        }
    }
    let ident = &st.ident;
    Ok(quote! {#ident { #(#streams),*}})
}

fn render_value(
    value: &Value,
    ty: &syn::Type,
    defs: &HashMap<String, TypeDef>,
) -> Result<TokenStream, Error> {
    match ty {
        syn::Type::Path(path) => {
            //Integers are range-checked against the target type before casting, so an
            //out-of-range value is a compile error rather than a silent wrap.
            macro_rules! int_arm {
                ($t:ty) => {
                    if let Value::Integer(n) = value {
                        match <$t>::try_from(*n) {
                            Ok(n) => Ok(quote! {#n}),
                            Err(_) => Err(Error::new(
                                path.span(),
                                format!("TOML value {} is out of range for {}", n, stringify!($t)),
                            )),
                        }
                    } else {
                        Err(Error::new(path.span(), "Toml value is not an Integer"))
                    }
                };
            }
            //Similary range-check floats, as well as special case INF, NEGINF, and NAN.
            macro_rules! float_arm {
                ($t:ty) => {
                    if let Value::Float(n) = value {
                        let n = *n;
                        if n.is_nan() {
                            Ok(quote! { $t::NAN })
                        } else if n.is_infinite() {
                            if n.is_sign_positive() {
                                Ok(quote! { $t::INFINITY })
                            } else {
                                Ok(quote! { $t::NEG_INFINITY })
                            }
                        } else {
                            let f = n as $t;
                            if f.is_finite() {
                                Ok(quote! {#f})
                            } else {
                                Err(Error::new(
                                    path.span(),
                                    format!(
                                        "TOML value {} is out of range for {}",
                                        n,
                                        stringify!($t)
                                    ),
                                ))
                            }
                        }
                    } else {
                        Err(Error::new(path.span(), "Toml value is not a Float"))
                    }
                };
            }
            match path
                .path
                .segments
                .last()
                .ok_or_else(|| Error::new(path.span(), "Empty path?"))?
                .ident
                .to_string()
                .as_str()
            {
                "u8" => int_arm!(u8),
                "u16" => int_arm!(u16),
                "u32" => int_arm!(u32),
                "u64" => int_arm!(u64),
                "u128" => int_arm!(u128),
                "usize" => int_arm!(usize),
                "i8" => int_arm!(i8),
                "i16" => int_arm!(i16),
                "i32" => int_arm!(i32),
                "i64" => int_arm!(i64),
                "i128" => int_arm!(i128),
                "isize" => int_arm!(isize),
                "f32" => float_arm!(f32),
                "f64" => float_arm!(f64),
                "bool" => {
                    if let Value::Boolean(n) = value {
                        Ok(quote! {
                            #n
                        })
                    } else {
                        Err(Error::new(
                            path.span(),
                            format!("Toml value is not an {}", stringify!(Boolean)),
                        ))
                    }
                }
                ident => render_composite_type(value, defs, path, ident),
            }
        }
        syn::Type::Array(array) => {
            if let TypeArray {
                len: Expr::Lit(ExprLit {
                    lit: Lit::Int(l), ..
                }),
                elem: e,
                ..
            } = array
            {
                if let Value::Array(a) = value {
                    if a.len() == l.base10_parse()? {
                        let entries: Vec<TokenStream> = a
                            .iter()
                            .map(|v| render_value(v, e, defs))
                            .collect::<Result<Vec<TokenStream>, Error>>()?;
                        Ok(quote! {[#(#entries),*]})
                    } else {
                        Err(Error::new(
                            array.span(),
                            "Toml array and type array do not match in len",
                        ))
                    }
                } else {
                    Err(Error::new(array.span(), "Toml value is not an array"))
                }
            } else {
                Err(Error::new(
                    array.span(),
                    "unsupported syntax (Array lengths must be literal ints, not expressions)",
                ))
            }
        }
        syn::Type::Reference(reference) => {
            if let TypeReference {
                lifetime: Some(lt),
                elem: e,
                ..
            } = reference
                && matches!(e.as_ref(), syn::Type::Path(p) if p.path.is_ident("str"))
                && lt.ident == "static"
            {
                if let Value::String(s) = value {
                    Ok(quote! {#s})
                } else {
                    Err(Error::new(reference.span(), "Toml value is not a string"))
                }
            } else {
                Err(Error::new(reference.span(), "unsupported type"))
            }
        }
        syn::Type::Tuple(tup) => Err(Error::new(tup.span(), "Tuple support not implemented yet")),
        _ => Err(Error::new(ty.span(), "Unsupported type")),
    }
}

fn render_composite_type(
    value: &Value,
    defs: &HashMap<String, TypeDef<'_>>,
    path: &syn::TypePath,
    ident: &str,
) -> Result<TokenStream, Error> {
    if let Some(i) = defs.get(ident) {
        match i {
            TypeDef::Struct(item_struct) => {
                if let Value::Table(t) = value {
                    render_struct(t, item_struct, defs)
                } else {
                    Err(Error::new(item_struct.span(), "Toml value is not a table"))
                }
            }
            TypeDef::Enum(item_enum) => {
                if let Value::String(s) = value {
                    render_enum(item_enum, s)
                } else {
                    Err(Error::new(item_enum.span(), "Toml value is not a String"))
                }
            }
        }
    } else {
        Err(Error::new(path.span(), "Path not in defs."))
    }
}

fn render_enum(item_enum: &ItemEnum, toml_str: &str) -> Result<TokenStream, Error> {
    if let Some(e) = item_enum
        .variants
        .iter()
        .find(|v| v.ident.to_string().as_str() == toml_str)
    {
        if !matches!(e.fields, syn::Fields::Unit) {
            return Err(Error::new(
                e.span(),
                "data-carrying enum variants are not supported",
            ));
        }
        let t = &item_enum.ident;
        let v = &e.ident;
        Ok(quote! {#t::#v})
    } else {
        Err(Error::new(
            item_enum.span(),
            format!("Toml string does not match any variant: {}", toml_str),
        ))
    }
}
