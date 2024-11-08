//! This module contains a Rust codegen implementation for ISF. The
//! [`generate`] function produces Rust code from an ISF `[spec::Spec]`.

use std::fs::read_to_string;

use crate::spec::{self, AssemblyElement, MachineElement};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;
use winnow::Parser;

/// Generate rust code for an ISF file at the given path.
pub fn generate_code(path: &str) -> anyhow::Result<String> {
    let text = read_to_string(path)?;
    let s: &str = text.as_str();
    let ast = crate::parse::parse
        .parse(s)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let spec = spec::form_spec(&ast)?;
    let tokens = generate(&spec);
    let file: syn::File = syn::parse2(tokens)?;
    let code = prettyplease::unparse(&file);
    Ok(code)
}

/// Generate a set of Rust structs for interacting with instructions. The
/// generated structs implement the [`AssemblyInstruction`] and
/// [`MachineInstruction`] traits. They also contain getter and setter
/// methods for each field.
pub fn generate(spec: &spec::Spec) -> TokenStream {
    let mut tokens = TokenStream::default();
    let storage = spec.instruction_width.next_multiple_of(8);

    for instruction in &spec.instructions {
        let instr_tokens = generate_instruction(storage, instruction);
        tokens.extend(instr_tokens);
    }

    tokens
}

pub fn generate_instruction(
    storage: usize,
    instr: &spec::Instruction,
) -> TokenStream {
    let name = format_ident!("{}", instr.name);
    let storage = format_ident!("u{}", storage);

    let default_impl = generate_default_impl(instr);
    let field_methods = generate_field_methods(instr, &storage);
    let assembly_parser = generate_assembly_parser(instr);
    let assembly_emitter = generate_assembly_emitter(instr);
    let machine_parser = generate_machine_parser(instr);

    let doc = format!(" {}", instr.doc);

    let generated = quote! {
        #[doc = #doc]
        #[derive(Debug, PartialEq, Eq)]
        struct #name(#storage);

        impl Default for #name {
            fn default() -> Self {
                #default_impl
            }
        }

        impl #name {
            #field_methods
            fn parse_assembly_impl(text: &mut &str) -> winnow::PResult<Self> {
                use winnow::Parser;
                let input = text;
                #assembly_parser
            }
        }

        impl isf::AssemblyInstruction for #name {
            #[rustfmt::skip]
            fn parse_assembly(
                mut text: &str,
            ) -> Result<
                Self,
                winnow::error::ParseError<&str, winnow::error::ContextError>,
            > {
                use winnow::Parser;
                let result = Self::parse_assembly_impl.parse(&mut text)?;
                Ok(result)
            }
            fn emit_assembly(&self) -> String {
                #assembly_emitter
            }
        }

        impl isf::MachineInstruction<#storage> for #name {
            fn parse_machine(data: #storage) -> Result<Self, isf::FieldMismatchError> {
                #machine_parser
            }
            fn emit_machine(&self) -> #storage {
                self.0
            }
        }
    };

    generated
}

pub fn generate_default_impl(instr: &spec::Instruction) -> TokenStream {
    let mut tks = TokenStream::default();
    tks.extend(quote! {
        let mut def = Self(0);
    });

    for me in &instr.machine.layout {
        if let MachineElement::Constant {
            name,
            width,
            value: Some(value),
        } = me
        {
            let setter = format_ident!("set_{}", name);
            if *width == 1 {
                tks.extend(quote! {
                   def.#setter(#value != 0);
                });
            } else {
                tks.extend(quote! {
                   def.#setter(#value.try_into().unwrap());
                });
            }
        }
    }

    tks.extend(quote! { def });
    tks
}
pub fn generate_machine_parser(instr: &spec::Instruction) -> TokenStream {
    let mut tks = TokenStream::default();

    tks.extend(quote! {
        let perhaps = Self(data);
    });

    for me in &instr.machine.layout {
        if let MachineElement::Constant {
            name,
            width: _,
            value,
        } = me
        {
            let getter = format_ident!("get_{name}");
            if let Some(value) = value {
                tks.extend(quote! {
                    let found = perhaps.#getter().try_into().unwrap();
                    let expected = #value;
                    if found != expected {
                        return Err(isf::FieldMismatchError{
                            field: #name.to_owned(),
                            expected,
                            found,
                        });
                    }
                });
            }
        }
    }
    tks.extend(quote! { Ok(perhaps) });
    tks
}

pub fn generate_assembly_emitter(instr: &spec::Instruction) -> TokenStream {
    let mut tks = TokenStream::default();

    tks.extend(quote! {
        let mut s = String::default();
    });

    for ae in &instr.assembly.syntax {
        match ae {
            AssemblyElement::StringLiteral { value } => {
                if !value.is_empty() {
                    tks.extend(quote! { s += #value; });
                }
            }
            AssemblyElement::NumberLiteral { value } => {
                tks.extend(quote! { s += #value; });
            }
            AssemblyElement::OptionalFlag { name, field } => {
                let getter = format_ident!("get_{field}");
                tks.extend(quote! {
                    if self.#getter() {
                        s += #name;
                    }
                });
            }
            AssemblyElement::Dot => {
                tks.extend(quote! { s += "."; });
            }
            AssemblyElement::Comma => {
                tks.extend(quote! { s += ","; });
            }
            AssemblyElement::Space => {
                tks.extend(quote! { s += " "; });
            }
            AssemblyElement::Field { name } => {
                let getter = format_ident!("get_{name}");
                tks.extend(quote! {
                    s += &format!("{}", self.#getter());
                })
            }
        }
    }

    tks.extend(quote! { s });
    tks
}

pub fn generate_field_methods(
    instr: &spec::Instruction,
    storage: &Ident,
) -> TokenStream {
    let mut tks = TokenStream::default();
    let mut offset = 0usize;
    for me in &instr.machine.layout {
        let (name, width, getter_only) = match me {
            spec::MachineElement::Field { name } => {
                let width = instr
                    .get_field(name.as_str())
                    .unwrap_or_else(|| panic!("undefined field: {name}"))
                    .width;
                (name.as_str(), width, false)
            }
            spec::MachineElement::Constant { name, width, value } => {
                if name == "_" {
                    offset += width;
                    continue;
                }
                (name.as_str(), *width, value.is_some())
            }
        };
        let getter = format_ident!("get_{name}");
        let setter = format_ident!("set_{name}");
        let byte_size = width.next_multiple_of(8);
        let (byte_type, get_fn, set_fn) = if width == 1 {
            (
                format_ident!("bool"),
                format_ident!("get_bit_{storage}"),
                format_ident!("set_bit_{storage}"),
            )
        } else if byte_size <= 128 {
            (
                format_ident!("u{byte_size}"),
                format_ident!("get_u{width}_{storage}"),
                format_ident!("set_u{width}_{storage}"),
            )
        } else {
            panic!("invalid field width for {name}: width");
        };
        tks.extend(quote! {
            pub fn #getter(&self) -> #byte_type {
                isf::bits::#get_fn(self.0, #offset)
            }
        });
        if getter_only {
            tks.extend(quote! {
                fn #setter(&mut self, value: #byte_type) {
                    self.0 = isf::bits::#set_fn(self.0, #offset, value);
                }
            });
        } else {
            tks.extend(quote! {
                pub fn #setter(&mut self, value: #byte_type) {
                    self.0 = isf::bits::#set_fn(self.0, #offset, value);
                }
            });
        }
        offset += width;
    }
    tks
}

pub fn generate_assembly_parser(instr: &spec::Instruction) -> TokenStream {
    let mut tks = TokenStream::default();

    tks.extend(quote! {
        let mut result = Self::default();
    });

    for x in &instr.assembly.syntax {
        match x {
            spec::AssemblyElement::StringLiteral { value } => {
                if !value.is_empty() {
                    tks.extend(quote! {
                        let _ = #value.parse_next(input)?;
                    });
                }
            }
            spec::AssemblyElement::NumberLiteral { value } => {
                let value = value.to_string();
                tks.extend(quote! {
                    let _ = #value.parse_next(input)?;
                });
            }
            spec::AssemblyElement::OptionalFlag { name, field } => {
                let field = format_ident!("{field}");
                let setter = format_ident!("set_{field}");
                tks.extend(quote! {
                    let #field : Result<
                        &str,
                        winnow::error::ErrMode<winnow::error::ContextError>,
                    > = #name.parse_next(input);
                    result.#setter(#field.is_ok());
                });
            }
            spec::AssemblyElement::Dot => {
                tks.extend(quote! {
                    let _ = '.'.parse_next(input)?;
                });
            }
            spec::AssemblyElement::Comma => {
                tks.extend(quote! {
                    let _ = ','.parse_next(input)?;
                });
            }
            spec::AssemblyElement::Space => {
                tks.extend(quote! {
                    let _ = winnow::ascii::multispace0.parse_next(input)?;
                });
            }
            spec::AssemblyElement::Field { name } => {
                let field = format_ident!("{name}");
                let setter = format_ident!("set_{name}");
                tks.extend(quote! {
                    let s = winnow::ascii::digit1.parse_next(input)?;
                    let #field: u128 = s.parse().unwrap();
                    result.#setter(#field.try_into().unwrap());
                });
            }
        }
    }
    tks.extend(quote! {
        Ok(result)
    });

    tks
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cg_add() {
        let code = generate_code("testcase/add.isf").unwrap();
        expectorate::assert_contents("testcase/add.rs", code.as_str());
    }
}
