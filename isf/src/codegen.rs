//! This module contains a Rust codegen implementation for ISF. The
//! [`generate`] function produces Rust code from an ISF `[spec::Spec]`.

use std::{collections::BTreeMap, fs::read_to_string};

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
    let storage = uint_size(spec.instruction_width);

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
        pub struct #name(#storage);

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

    let num_constants = instr
        .machine
        .layout
        .iter()
        .filter(|x| {
            matches!(
                x,
                MachineElement::Constant {
                    name: _,
                    width: _,
                    value: Some(_)
                },
            )
        })
        .count();

    let num_field_absent_tests = instr
        .machine
        .layout
        .iter()
        .filter(|x| {
            matches!(x, MachineElement::OptionalFieldAbsentTest { name: _ },)
        })
        .count();

    if num_constants == 0 && num_field_absent_tests == 0 {
        tks.extend(quote! { Self(0) });
        return tks;
    }

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
        if let MachineElement::OptionalFieldAbsentTest { name } = me {
            let setter = format_ident!("{}_mark_unset", name);
            tks.extend(quote! {
                def.#setter();
            })
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
            AssemblyElement::OptionalField { name, with_dot } => {
                let getter = format_ident!("get_{name}");
                if *with_dot {
                    tks.extend(quote! {
                        if self.#getter() != 0 {
                            s += ".";
                            s += #name;
                        }
                    });
                } else {
                    tks.extend(quote! {
                        if self.#getter() != 0 {
                            s += #name;
                        }
                    });
                }
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

    let mut setters = BTreeMap::<String, (bool, Ident, TokenStream)>::default();
    let mut getters = BTreeMap::<String, (Ident, TokenStream, bool)>::default();
    let mut set_indicators = BTreeMap::<String, TokenStream>::default();
    let mut mark_unset = BTreeMap::<String, TokenStream>::default();

    for me in &instr.machine.layout {
        let (
            name,
            width,
            getter_only,
            slice_bounds,
            element_width,
            negate,
            ptest,
            atest,
        ) = match me {
            spec::MachineElement::Field { name } => {
                let width = instr
                    .get_field(name.as_str())
                    .unwrap_or_else(|| panic!("undefined field: {name}"))
                    .width;
                (
                    name.as_str(),
                    width,
                    false,
                    None,
                    width,
                    false,
                    false,
                    false,
                )
            }
            spec::MachineElement::FieldNegate { name } => {
                let width = instr
                    .get_field(name.as_str())
                    .unwrap_or_else(|| panic!("undefined field: {name}"))
                    .width;
                (name.as_str(), width, false, None, width, true, false, false)
            }
            spec::MachineElement::OptionalFieldPresentTest { name } => {
                let width = instr
                    .get_field(name.as_str())
                    .unwrap_or_else(|| panic!("undefined field: {name}"))
                    .width;
                (name.as_str(), width, false, None, 1, false, true, false)
            }
            spec::MachineElement::OptionalFieldAbsentTest { name } => {
                let width = instr
                    .get_field(name.as_str())
                    .unwrap_or_else(|| panic!("undefined field: {name}"))
                    .width;
                (name.as_str(), width, false, None, 1, false, false, true)
            }
            spec::MachineElement::FieldSlice { name, begin, end } => {
                let element_width = (end - begin) + 1;
                let width = instr
                    .get_field(name.as_str())
                    .unwrap_or_else(|| panic!("undefined field: {name}"))
                    .width;
                (
                    name.as_str(),
                    width,
                    false,
                    Some((begin, end)),
                    element_width,
                    false,
                    false,
                    false,
                )
            }
            spec::MachineElement::Constant { name, width, value } => {
                if name == "_" {
                    offset += width;
                    continue;
                }
                (
                    name.as_str(),
                    *width,
                    value.is_some(),
                    None,
                    *width,
                    false,
                    false,
                    false,
                )
            }
        };
        let getter_s = format!("get_{name}");
        let setter_s = format!("set_{name}");
        let set_indicator_s = format!("{name}_is_set");
        let mark_unset_s = format!("{name}_mark_unset");
        let byte_size = uint_size(width);
        let (byte_type, get_fn, set_fn) = if width == 1 {
            (
                format_ident!("bool"),
                format_ident!("get_bit_{storage}"),
                format_ident!("set_bit_{storage}"),
            )
        } else if byte_size <= 128 {
            (
                format_ident!("u{byte_size}"),
                format_ident!("get_u{element_width}_{storage}"),
                format_ident!("set_u{element_width}_{storage}"),
            )
        } else {
            panic!("invalid field width for {name}: width");
        };

        let negate = if negate {
            quote! { ! }
        } else {
            quote! {}
        };

        // This is last getter wins semantics, should be ok for multiple
        // appearances of the same field in a layout as they should all
        // be equivalent. Why do this you ask? See the X2 cmp instructions.
        match slice_bounds {
            None => {
                if ptest | atest {
                    let get_set_fn = format_ident!("get_bit_{storage}");
                    let body =
                        quote! { isf::bits::#get_set_fn(self.0, #offset) };
                    set_indicators.insert(set_indicator_s, body);

                    let mark_unset_fn = format_ident!("set_bit_{storage}");
                    let body = quote! { self.0 = isf::bits::#mark_unset_fn(self.0, #offset, true); };
                    mark_unset.insert(mark_unset_s, body);
                } else {
                    let body =
                        quote! { #negate isf::bits::#get_fn(self.0, #offset) };
                    getters.insert(getter_s, (byte_type.clone(), body, false));
                }
            }
            Some((lower, _upper)) => {
                let w = uint_size(width);
                let typ = format_ident!("u{w}");
                match getters.get_mut(&getter_s) {
                    Some(entry) => {
                        let body = quote! {
                            result |=
                                (#negate isf::bits::#get_fn(self.0, #offset) as #typ)
                                << #lower;
                        };
                        entry.1.extend(body);
                    }
                    None => {
                        let body = quote! {
                            let mut result = #negate isf::bits::#get_fn(self.0, #offset) as #typ;
                        };
                        getters
                            .insert(getter_s, (byte_type.clone(), body, true));
                    }
                }
            }
        };

        let body = match slice_bounds {
            None => {
                if ptest {
                    quote! {
                        self.0 = isf::bits::#set_fn(self.0, #offset, 1);
                    }
                } else if atest {
                    quote! {
                        self.0 = isf::bits::#set_fn(self.0, #offset, 0);
                    }
                } else {
                    quote! {
                        self.0 = isf::bits::#set_fn(self.0, #offset, #negate value);
                    }
                }
            }
            Some((lower, upper)) => {
                let w = uint_size(upper - lower);
                let typ = format_ident!("u{w}");
                quote! {
                    self.0 = isf::bits::#set_fn(
                        self.0, #offset, (#negate value >> #lower)
                        as #typ
                    );
                }
            }
        };
        setters
            .entry(setter_s)
            .and_modify(|x| x.2.extend(body.clone()))
            .or_insert((getter_only, byte_type, body));

        offset += element_width;
    }

    for (fn_name, (byte_type, tokens, slice_based)) in &getters {
        let getter = format_ident!("{fn_name}");
        if *slice_based {
            tks.extend(quote! {
                pub fn #getter(&self) -> #byte_type {
                    #tokens
                    result
                }
            });
        } else {
            tks.extend(quote! {
                pub fn #getter(&self) -> #byte_type {
                    #tokens
                }
            });
        }
    }

    for (fn_name, (private, byte_type, tokens)) in &setters {
        let setter = format_ident!("{fn_name}");
        if *private {
            tks.extend(quote! {
                fn #setter(&mut self, value: #byte_type) {
                    #tokens
                }
            });
        } else {
            tks.extend(quote! {
                pub fn #setter(&mut self, value: #byte_type) {
                    #tokens
                }
            });
        }
    }

    for (fn_name, tokens) in &set_indicators {
        let set_indicator = format_ident!("{fn_name}");
        tks.extend(quote! {
            pub fn #set_indicator(&self) -> bool {
                #tokens
            }
        })
    }

    for (fn_name, tokens) in &mark_unset {
        let unset_marker = format_ident!("{fn_name}");
        tks.extend(quote! {
            fn #unset_marker(&mut self) {
                #tokens
            }
        })
    }

    tks
}

pub fn generate_assembly_parser(instr: &spec::Instruction) -> TokenStream {
    let mut tks = TokenStream::default();

    if instr.fields.is_empty() {
        tks.extend(quote! {
            let result = Self::default();
        });
    } else {
        tks.extend(quote! {
            let mut result = Self::default();
        });
    }

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
            spec::AssemblyElement::OptionalField { name, with_dot } => {
                let field = format_ident!("{name}");
                let setter = format_ident!("set_{name}");
                let body = quote! {
                    let #field : Result<
                        u128,
                        winnow::error::ErrMode<winnow::error::ContextError>,
                    > =  isf::parse::number_parser.parse_next(input);
                    if let Ok(#field) = #field {
                        result.#setter(#field.try_into().unwrap());
                    }
                };
                if *with_dot {
                    tks.extend(quote! {
                        let dot_ok = isf::parse::s(".").parse_next(input).is_ok();
                        if dot_ok {
                            #body
                        }
                    });
                } else {
                    tks.extend(quote! {
                        #body
                    })
                }
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
                let field_info = instr
                    .get_field(name)
                    .unwrap_or_else(|| panic!("field {name} undefined"));
                if field_info.width == 1 {
                    tks.extend(quote! {
                        let #field: u128 = isf::parse::number_parser.parse_next(input)?;
                        result.#setter(#field != 0);
                    });
                } else {
                    tks.extend(quote! {
                        let #field: u128 = isf::parse::number_parser.parse_next(input)?;
                        result.#setter(#field.try_into().unwrap());
                    });
                }
            }
        }
    }
    tks.extend(quote! {
        Ok(result)
    });

    tks
}

fn uint_size(bits: usize) -> usize {
    match bits {
        x if x <= 8 => 8,
        x if x <= 16 => 16,
        x if x <= 32 => 32,
        x if x <= 64 => 64,
        x if x <= 128 => 128,
        _ => panic!("{bits} too big"),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn cg_add() {
        let mut code = generate_code("testcase/add.isf").unwrap();
        code.insert_str(0, "#![rustfmt::skip]\n");
        expectorate::assert_contents("testcase/add.rs", code.as_str());
    }

    #[test]
    fn cg_slice_add() {
        let mut code = generate_code("testcase/slice-add.isf").unwrap();
        code.insert_str(0, "#![rustfmt::skip]\n");
        expectorate::assert_contents("testcase/slice_add.rs", code.as_str());
    }

    #[test]
    fn cg_add_field_opt() {
        let mut code = generate_code("testcase/add-field-opt.isf").unwrap();
        code.insert_str(0, "#![rustfmt::skip]\n");
        expectorate::assert_contents(
            "testcase/add_field_opt.rs",
            code.as_str(),
        );
    }
}
