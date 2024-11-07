use isf::{codegen::generate, parse::parse, spec::form_spec};
use proc_macro::TokenStream;
use std::fs::read_to_string;
use syn::LitStr;

#[proc_macro]
pub fn isf(item: TokenStream) -> TokenStream {
    let filename = syn::parse::<LitStr>(item).expect("parse filename").value();
    let text = read_to_string(filename).expect("read isf file");
    let mut s: &str = text.as_str();
    let ast = parse(&mut s).expect("parse isf");
    let spec = form_spec(&ast).expect("form isf spec");
    let tokens = generate(&spec);
    tokens.into()
}