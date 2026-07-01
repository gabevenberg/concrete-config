use proc_macro::TokenStream;
use quote::quote;
use std::{env, fs, path::PathBuf};
use syn::{ItemMod, LitStr, parse_macro_input};

#[proc_macro_attribute]
pub fn from_toml(args: TokenStream, input: TokenStream) -> TokenStream {
    let path = parse_macro_input!(args as LitStr);
    let module = parse_macro_input!(input as ItemMod);

    eprintln!("{}", path.value());
    eprintln!("{:#?}", module);

    let toml_path = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap()).join(path.value());
    let content = fs::read_to_string(&toml_path).unwrap();
    let toml_value: toml::Value = toml::from_str(&content).unwrap();
    eprintln!("{:#?}", toml_value);

    let items = &module.content.unwrap().1;
    let (root_structs, other_structs): (Vec<_>, Vec<_>) = items.iter().partition(|item| {
        if let syn::Item::Struct(s) = item {
            s.attrs.iter().any(|a| a.path().is_ident("root"))
        } else {
            false
        }
    });

    let root_struct = if root_structs.len() == 1 {
        if let syn::Item::Struct(s) = root_structs[0] {
            let mut s = s.clone();
            s.attrs.retain(|a| !a.path().is_ident("root"));
            s
        } else {
            unreachable!()
        }
    } else {
        panic!("Must have only one '#[root]' struct!")
    };

    eprintln!("{:#?}", root_struct);
    eprintln!("{:#?}", other_structs);

    todo!()
}
