//! This module contains a parser implmentation for ISF. The [`parse`] function
//! parses text into an ISF AST.

use crate::ast;
use winnow::{
    ascii::{
        alpha1, alphanumeric1, digit1, hex_digit1, line_ending, multispace0,
        multispace1, till_line_ending,
    },
    combinator::{alt, cut_err, repeat, separated, trace},
    error::{ContextError, StrContext},
    token::{none_of, take_until},
    PResult, Parser,
};

/// Parse ISF text into an ISF AST.
pub fn parse(input: &mut &str) -> PResult<ast::Ast> {
    let spec = ast::Ast {
        characteristics: parse_characteristics.parse_next(input)?,
        instructions: parse_instructions.parse_next(input)?,
    };
    Ok(spec)
}

fn parse_characteristics(
    input: &mut &str,
) -> PResult<Vec<ast::Characteristic>> {
    let result = repeat(0.., characteristic).parse_next(input)?;
    Ok(result)
}

fn parse_instructions(input: &mut &str) -> PResult<Vec<ast::Instruction>> {
    lcp.parse_next(input)?;
    let result = cut_err(repeat(0.., instruction)).parse_next(input)?;
    lcp.parse_next(input)?;
    Ok(result)
}

fn instruction(input: &mut &str) -> PResult<ast::Instruction> {
    lcp.parse_next(input)?;
    let doc = docstring.parse_next(input)?;
    lcp.parse_next(input)?;

    let _ = s("instruction").parse_next(input)?;
    let mut instr = cut_err(instruction_body)
        .context(StrContext::Label("instruction body"))
        .parse_next(input)?;
    instr.doc = doc;
    Ok(instr)
}

fn instruction_body(input: &mut &str) -> PResult<ast::Instruction> {
    let name = identifier_parser.parse_next(input)?;
    let parameters =
        instruction_parameters.parse_next(input).unwrap_or_default();
    let base = instruction_base.parse_next(input).ok();
    let _ = s("{").parse_next(input)?;
    lcp.parse_next(input)?;
    let timing = if s("timing:").parse_next(input).is_ok() {
        Some(
            timing
                .context(StrContext::Label("timing"))
                .parse_next(input)?,
        )
    } else {
        None
    };
    let fields = if s("fields:").parse_next(input).is_ok() {
        fields
            .context(StrContext::Label("fields"))
            .parse_next(input)?
    } else {
        Vec::default()
    };
    let assembly = if s("assembly:").parse_next(input).is_ok() {
        cut_err(assembly)
            .context(StrContext::Label("assembly"))
            .parse_next(input)?
    } else {
        ast::Assembly::default()
    };

    let machine = if s("machine:").parse_next(input).is_ok() {
        cut_err(machine)
            .context(StrContext::Label("machine"))
            .parse_next(input)?
    } else {
        ast::Machine::default()
    };
    let _ = s("}").parse_next(input)?;
    Ok(ast::Instruction {
        doc: String::default(),
        name,
        parameters,
        base,
        timing,
        fields,
        assembly,
        machine,
    })
}

fn instruction_parameters(input: &mut &str) -> PResult<Vec<String>> {
    let _ = s('<').parse_next(input)?;
    let params = separated(1.., identifier_parser, s(',')).parse_next(input)?;
    let _ = s('>').parse_next(input)?;
    Ok(params)
}

fn instruction_base(input: &mut &str) -> PResult<ast::Base> {
    let _ = s(':').parse_next(input)?;
    let name = identifier_parser.parse_next(input)?;
    let _ = s('<').parse_next(input)?;
    let parameters =
        separated(1.., base_parameter, s(',')).parse_next(input)?;
    let _ = s('>').parse_next(input)?;
    Ok(ast::Base { name, parameters })
}

fn fields(input: &mut &str) -> PResult<Vec<ast::Field>> {
    let result = cut_err(separated(0.., field, s(','))).parse_next(input)?;
    let _ = s(',').parse_next(input);
    lcp.parse_next(input)?;
    Ok(result)
}

fn timing(input: &mut &str) -> PResult<ast::Timing> {
    lcp.parse_next(input)?;
    let result =
        alt((cycle_timing, async_timing, multi_timing)).parse_next(input)?;
    lcp.parse_next(input)?;
    Ok(result)
}

fn cycle_timing(input: &mut &str) -> PResult<ast::Timing> {
    let n = s(number_parser).parse_next(input)?;
    let _ = s("cycle").parse_next(input)?;
    Ok(ast::Timing::Cycle(n.try_into().unwrap()))
}

fn async_timing(input: &mut &str) -> PResult<ast::Timing> {
    let _ = s("async").parse_next(input)?;
    Ok(ast::Timing::Async)
}

fn multi_timing(input: &mut &str) -> PResult<ast::Timing> {
    let _ = s("multi").parse_next(input)?;
    Ok(ast::Timing::Multi)
}

fn field(input: &mut &str) -> PResult<ast::Field> {
    lcp.parse_next(input)?;
    let doc = docstring
        .context(StrContext::Label("field docstring"))
        .parse_next(input)?;
    lcp.parse_next(input)?;

    let name = cut_err(s(identifier_parser))
        .context(StrContext::Label("field identifier"))
        .parse_next(input)?;
    let _ = s(":").parse_next(input)?;
    let width = s(number_parser).parse_next(input)?;

    lcp.parse_next(input)?;

    Ok(ast::Field {
        doc,
        name,
        width: width.try_into().expect("width as usize"),
        value: None, //TODO
    })
}

fn docstring(input: &mut &str) -> PResult<String> {
    let lines: Vec<String> = repeat(1.., docstring_line).parse_next(input)?;
    Ok(lines.join("\n"))
}

fn docstring_line(input: &mut &str) -> PResult<String> {
    let _ = multispace0.parse_next(input)?;
    let _ = "///".parse_next(input)?;
    let ds = till_line_ending.parse_next(input)?;
    let _ = line_ending.parse_next(input)?;
    Ok(ds.trim().to_owned())
}

fn assembly(input: &mut &str) -> PResult<ast::Assembly> {
    lcp.parse_next(input)?;
    let _ = multispace0.parse_next(input)?;
    let syntax: Vec<ast::AssemblyElement> = if !input.starts_with("examples:") {
        repeat(0.., assembly_element).parse_next(input)?
    } else {
        Vec::default()
    };
    if !syntax.is_empty() {
        let _ = s(';').parse_next(input)?;
    }
    lcp.parse_next(input)?;
    let example = if s("examples:").parse_next(input).is_ok() {
        cut_err(assembly_examples).parse_next(input)?
    } else {
        Vec::default()
    };
    Ok(ast::Assembly { syntax, example })
}

fn assembly_element(input: &mut &str) -> PResult<ast::AssemblyElement> {
    alt((
        assembly_element_expansion,
        assembly_element_string_literal,
        assembly_element_optional_flag,
        assembly_element_optional_field,
        assembly_element_identifier,
        assembly_element_dot,
        assembly_element_comma,
        assembly_element_space,
    ))
    .parse_next(input)
}

fn string_literal(input: &mut &str) -> PResult<String> {
    let _ = "'".parse_next(input)?;
    let content = take_until(0.., "'").parse_next(input)?;
    let _ = "'".parse_next(input)?;
    Ok(content.to_owned())
}

fn assembly_element_expansion(
    input: &mut &str,
) -> PResult<ast::AssemblyElement> {
    let _ = '$'.parse_next(input)?;
    let name = identifier_parser_nospace.parse_next(input)?;
    Ok(ast::AssemblyElement::Expansion { name })
}

fn assembly_element_string_literal(
    input: &mut &str,
) -> PResult<ast::AssemblyElement> {
    let content = string_literal.parse_next(input)?;
    Ok(ast::AssemblyElement::StringLiteral {
        value: content.to_owned(),
    })
}

fn assembly_element_optional_flag(
    input: &mut &str,
) -> PResult<ast::AssemblyElement> {
    let _ = '['.parse_next(input)?;
    let target = s(string_literal).parse_next(input)?;
    let _ = s('=').parse_next(input)?;
    let field = s(identifier_parser).parse_next(input)?;
    let _ = ']'.parse_next(input)?;
    Ok(ast::AssemblyElement::OptionalFlag {
        name: target,
        field,
    })
}

fn assembly_element_optional_field(
    input: &mut &str,
) -> PResult<ast::AssemblyElement> {
    let _ = '['.parse_next(input)?;
    let with_dot = s('.').parse_next(input).is_ok();
    let name = s(identifier_parser).parse_next(input)?;
    let _ = ']'.parse_next(input)?;
    Ok(ast::AssemblyElement::OptionalField { name, with_dot })
}

fn assembly_element_dot(input: &mut &str) -> PResult<ast::AssemblyElement> {
    let _ = ".".parse_next(input)?;
    Ok(ast::AssemblyElement::Dot)
}

fn assembly_element_comma(input: &mut &str) -> PResult<ast::AssemblyElement> {
    let _ = ",".parse_next(input)?;
    Ok(ast::AssemblyElement::Comma)
}

fn assembly_element_space(input: &mut &str) -> PResult<ast::AssemblyElement> {
    let _ = alt((s("\n"), multispace1)).parse_next(input)?;
    Ok(ast::AssemblyElement::Space)
}

fn assembly_element_identifier(
    input: &mut &str,
) -> PResult<ast::AssemblyElement> {
    let value = identifier_parser_nospace.parse_next(input)?;
    Ok(ast::AssemblyElement::Field { name: value })
}

fn assembly_examples(input: &mut &str) -> PResult<Vec<ast::AssemblyExample>> {
    cut_err(repeat(0.., assembly_example)).parse_next(input)
}

fn assembly_example(input: &mut &str) -> PResult<ast::AssemblyExample> {
    lcp.parse_next(input)?;
    let doc = docstring.parse_next(input)?;
    lcp.parse_next(input)?;
    let example = take_until(1.., ";").parse_next(input)?.trim().to_owned();
    let _ = (";").parse_next(input)?;
    lcp.parse_next(input)?;
    Ok(ast::AssemblyExample { doc, example })
}

fn machine(input: &mut &str) -> PResult<ast::Machine> {
    let layout = separated(1.., machine_element, s(',')).parse_next(input)?;
    let _ = s(',').parse_next(input);
    lcp.parse_next(input)?;
    Ok(ast::Machine { layout })
}

fn machine_element(input: &mut &str) -> PResult<ast::MachineElement> {
    lcp.parse_next(input)?;
    let result = alt((machine_element_constant, machine_element_field))
        .parse_next(input)?;
    lcp.parse_next(input)?;
    Ok(result)
}

fn machine_element_field(input: &mut &str) -> PResult<ast::MachineElement> {
    let name = identifier_parser.parse_next(input)?;
    if tag('[').parse_next(input).is_ok() {
        let begin = number_parser.parse_next(input)?;
        let _ = ':'.parse_next(input)?;
        let end = number_parser.parse_next(input)?;
        let _ = ']'.parse_next(input)?;
        Ok(ast::MachineElement::FieldSlice {
            name,
            begin: begin.try_into().unwrap(),
            end: end.try_into().unwrap(),
        })
    } else if tag('!').parse_next(input).is_ok() {
        Ok(ast::MachineElement::FieldNegate { name })
    } else if tag('?').parse_next(input).is_ok() {
        if tag('!').parse_next(input).is_ok() {
            Ok(ast::MachineElement::OptionalFieldAbsentTest { name })
        } else {
            Ok(ast::MachineElement::OptionalFieldPresentTest { name })
        }
    } else {
        Ok(ast::MachineElement::Field { name })
    }
}

fn machine_element_constant(input: &mut &str) -> PResult<ast::MachineElement> {
    let name = identifier_parser.parse_next(input)?;
    let _ = s(':').parse_next(input)?;
    let width = s(number_parser).parse_next(input)?;
    let value = if s('=').parse_next(input).is_ok() {
        Some(s(machine_element_value).parse_next(input)?)
    } else {
        None
    };
    Ok(ast::MachineElement::Constant {
        name,
        width: width.try_into().expect("machine element value as usize"),
        value,
    })
}

fn machine_element_value(
    input: &mut &str,
) -> PResult<ast::MachineElementValue> {
    if let Ok(number) = number_parser.parse_next(input) {
        let v = ast::MachineElementValue::NumericConstant(number);
        return Ok(v);
    };
    let _ = s('$').parse_next(input)?;
    let name = identifier_parser.parse_next(input)?;
    let v = ast::MachineElementValue::GenericParameter(name);
    Ok(v)
}

fn base_parameter(input: &mut &str) -> PResult<ast::BaseParameter> {
    if let Ok(number) = number_parser.parse_next(input) {
        return Ok(ast::BaseParameter::Number(number));
    };
    let name = string_literal.parse_next(input)?;
    Ok(ast::BaseParameter::Text(name))
}

fn characteristic(input: &mut &str) -> PResult<ast::Characteristic> {
    lcp.parse_next(input)?;
    // add others as alternates as they arise
    let result = instruction_width_characteristic.parse_next(input)?;
    Ok(result)
}

fn instruction_width_characteristic(
    input: &mut &str,
) -> PResult<ast::Characteristic> {
    let _ = s("instruction_width").parse_next(input)?;
    let _ = s("=").parse_next(input)?;
    let width = number_parser.parse_next(input)?;
    let _ = s(";").parse_next(input)?;
    Ok(ast::Characteristic::InstructionWidth(
        width.try_into().expect("instruction width <= usize"),
    ))
}

/// Parse an identifier.
pub fn identifier_parser(input: &mut &str) -> PResult<String> {
    let ident = s((alt(("_", alpha1)), alphanumunder0)).parse_next(input)?;
    Ok(format!("{}{}", ident.0, ident.1))
}

pub fn identifier_parser_nospace(input: &mut &str) -> PResult<String> {
    let ident = (alt(("_", alpha1)), alphanumunder0).parse_next(input)?;
    Ok(format!("{}{}", ident.0, ident.1))
}

/// Parse a series of alphanumeric chracters or underscore.
pub fn alphanumunder0(input: &mut &str) -> PResult<String> {
    let result = repeat(0.., alt((alphanumeric1, "_"))).parse_next(input)?;
    Ok(result)
}

/// Allow the provided parser to have arbitrary space on either side.
pub fn s<'s, Output, ParseNext>(
    mut parser: ParseNext,
) -> impl Parser<&'s str, Output, ContextError>
where
    ParseNext: Parser<&'s str, Output, ContextError>,
{
    trace("s", move |input: &mut &'s str| {
        let _ = multispace0.parse_next(input)?;
        let result = parser.parse_next(input)?;
        let _ = multispace0.parse_next(input)?;
        Ok(result)
    })
}

/// A helper for type gymnastics
pub fn tag<'s, Output, ParseNext>(
    mut parser: ParseNext,
) -> impl Parser<&'s str, Output, ContextError>
where
    ParseNext: Parser<&'s str, Output, ContextError>,
{
    trace("tag", move |input: &mut &'s str| {
        let result = parser.parse_next(input)?;
        Ok(result)
    })
}

/// Parse c-style a line comment.
pub fn line_comment_parser(input: &mut &str) -> PResult<(), ContextError> {
    let _ = multispace0.parse_next(input)?;
    let _ = ("//", none_of(['/'])).parse_next(input)?;
    let _ = till_line_ending.parse_next(input)?;
    let _ = line_ending.parse_next(input)?;
    Ok(())
}

pub fn lcp(input: &mut &str) -> PResult<(), ContextError> {
    repeat(0.., line_comment_parser).parse_next(input)
}

pub fn number_parser(input: &mut &str) -> PResult<u64> {
    if s("0x").parse_next(input).is_ok() {
        let s = hex_digit1.parse_next(input)?;
        let n = u64::from_str_radix(s, 16).unwrap();
        Ok(n)
    } else {
        let s = digit1.parse_next(input)?;
        let n: u64 = s.parse().unwrap();
        Ok(n)
    }
}

#[cfg(test)]
mod test {
    use ast::MachineElement;

    use super::*;
    use std::fs::read_to_string;

    #[test]
    fn parse_charachteristics() {
        let text = read_to_string("testcase/characteristics.isf").unwrap();
        let s: &str = text.as_str();
        match parse.parse(s) {
            Err(e) => {
                panic!("{e}")
            }
            Ok(parsed) => {
                println!("{parsed:#?}");
                assert_eq!(
                    ast::Characteristic::InstructionWidth(47),
                    parsed.characteristics[0]
                );
            }
        }
    }

    #[test]
    fn parse_add() {
        let text = read_to_string("testcase/add.isf").unwrap();
        let s: &str = text.as_str();
        let parsed = match parse.parse(s) {
            Err(e) => {
                panic!("{e}")
            }
            Ok(parsed) => {
                println!("{parsed:#?}");
                parsed
            }
        };
        assert_eq!(parsed.instructions.len(), 1);
        assert_eq!(parsed.instructions[0].doc, "Add values from two registers");
        assert_eq!(parsed.instructions[0].fields.len(), 4);
        assert_eq!(
            parsed.instructions[0].fields[0],
            ast::Field {
                doc: "The destination register".to_owned(),
                name: "dst".to_owned(),
                width: 5,
                value: None,
            }
        );
        assert_eq!(
            parsed.instructions[0].fields[1],
            ast::Field {
                doc: "The first source register".to_owned(),
                name: "src1".to_owned(),
                width: 5,
                value: None,
            }
        );
        assert_eq!(
            parsed.instructions[0].fields[2],
            ast::Field {
                doc: "The second source register".to_owned(),
                name: "src2".to_owned(),
                width: 5,
                value: None,
            }
        );
        assert_eq!(
            parsed.instructions[0].fields[3],
            ast::Field {
                doc: "Set a flag that sign extends the result".to_owned(),
                name: "sign_extend".to_owned(),
                width: 1,
                value: None,
            }
        );
        assert_eq!(
            parsed.instructions[0].assembly.syntax,
            vec![
                ast::AssemblyElement::StringLiteral {
                    value: "add".to_owned()
                },
                ast::AssemblyElement::OptionalFlag {
                    name: ".sx".to_owned(),
                    field: "sign_extend".to_owned()
                },
                ast::AssemblyElement::Space,
                ast::AssemblyElement::StringLiteral {
                    value: "r".to_owned(),
                },
                ast::AssemblyElement::Field {
                    name: "dst".to_owned()
                },
                ast::AssemblyElement::Space,
                ast::AssemblyElement::StringLiteral {
                    value: "r".to_owned(),
                },
                ast::AssemblyElement::Field {
                    name: "src1".to_owned()
                },
                ast::AssemblyElement::Space,
                ast::AssemblyElement::StringLiteral {
                    value: "r".to_owned(),
                },
                ast::AssemblyElement::Field {
                    name: "src2".to_owned()
                },
            ]
        );
        assert_eq!(parsed.instructions[0].assembly.example.len(), 2);
        assert_eq!(parsed.instructions[0].assembly.example[0], ast::AssemblyExample{
                    doc: [
                      "Add the contents of registers 4 and 7 placing the result in",
                      "register 0."
                    ].join("\n"),
                    example: "add r0 r4 r7".to_owned(),
                });
        assert_eq!(
            parsed.instructions[0].assembly.example[1],
            ast::AssemblyExample {
                doc: [
                    "Add the contents of registers 4 and 7 sign-extending and",
                    "placing the result in register 0."
                ]
                .join("\n"),
                example: "add.sx r0 r4 r7".to_owned(),
            }
        );
        assert_eq!(parsed.instructions[0].machine.layout.len(), 8);
        assert_eq!(
            parsed.instructions[0].machine.layout[0],
            ast::MachineElement::Constant {
                name: "opcode".to_owned(),
                width: 7,
                value: Some(ast::MachineElementValue::NumericConstant(2)),
            }
        );
        assert_eq!(
            parsed.instructions[0].machine.layout[1],
            ast::MachineElement::FieldNegate {
                name: "sign_extend".to_owned(),
            }
        );
        assert_eq!(
            parsed.instructions[0].machine.layout[2],
            ast::MachineElement::Field {
                name: "dst".to_owned(),
            }
        );
        assert_eq!(
            parsed.instructions[0].machine.layout[3],
            ast::MachineElement::Constant {
                name: "_".to_owned(),
                width: 3,
                value: None,
            }
        );
        assert_eq!(
            parsed.instructions[0].machine.layout[4],
            ast::MachineElement::Field {
                name: "src1".to_owned(),
            }
        );
        assert_eq!(
            parsed.instructions[0].machine.layout[5],
            ast::MachineElement::Constant {
                name: "_".to_owned(),
                width: 3,
                value: None,
            }
        );
        assert_eq!(
            parsed.instructions[0].machine.layout[6],
            ast::MachineElement::Field {
                name: "src2".to_owned(),
            }
        );
        assert_eq!(
            parsed.instructions[0].machine.layout[7],
            ast::MachineElement::Constant {
                name: "_".to_owned(),
                width: 3,
                value: None,
            }
        );
    }

    #[test]
    fn parse_binop() {
        let text = read_to_string("testcase/binop.isf").unwrap();
        let s = text.as_str();
        let parsed = match parse.parse(s) {
            Err(e) => {
                panic!("{e}")
            }
            Ok(parsed) => {
                println!("{parsed:#?}");
                parsed
            }
        };

        assert_eq!(parsed.instructions.len(), 3);

        // check generic binop instruction

        assert_eq!(
            parsed.instructions[0].doc,
            "A base instruction for binary arithmetic operations"
        );
        assert_eq!(parsed.instructions[0].parameters.len(), 2);
        assert_eq!(parsed.instructions[0].parameters[0], "name");
        assert_eq!(parsed.instructions[0].parameters[1], "opcode");
        assert_eq!(
            parsed.instructions[0].fields[0],
            ast::Field {
                doc: "The destination register".to_owned(),
                name: "dst".to_owned(),
                width: 5,
                value: None,
            }
        );
        assert_eq!(
            parsed.instructions[0].fields[1],
            ast::Field {
                doc: "The first source register".to_owned(),
                name: "src1".to_owned(),
                width: 5,
                value: None,
            }
        );
        assert_eq!(
            parsed.instructions[0].fields[2],
            ast::Field {
                doc: "The second source register".to_owned(),
                name: "src2".to_owned(),
                width: 5,
                value: None,
            }
        );
        assert_eq!(
            parsed.instructions[0].fields[3],
            ast::Field {
                doc: "Set a flag that sign extends the result".to_owned(),
                name: "sign_extend".to_owned(),
                width: 1,
                value: None,
            }
        );
        assert_eq!(
            parsed.instructions[0].assembly.syntax,
            vec![
                ast::AssemblyElement::Expansion {
                    name: "name".to_owned()
                },
                ast::AssemblyElement::OptionalFlag {
                    name: ".sx".to_owned(),
                    field: "sign_extend".to_owned()
                },
                ast::AssemblyElement::Space,
                ast::AssemblyElement::StringLiteral {
                    value: "r".to_owned(),
                },
                ast::AssemblyElement::Field {
                    name: "dst".to_owned()
                },
                ast::AssemblyElement::Space,
                ast::AssemblyElement::StringLiteral {
                    value: "r".to_owned(),
                },
                ast::AssemblyElement::Field {
                    name: "src1".to_owned()
                },
                ast::AssemblyElement::Space,
                ast::AssemblyElement::StringLiteral {
                    value: "r".to_owned(),
                },
                ast::AssemblyElement::Field {
                    name: "src2".to_owned()
                },
            ]
        );
        assert_eq!(parsed.instructions[0].machine.layout.len(), 8);
        assert_eq!(
            parsed.instructions[0].machine.layout[0],
            ast::MachineElement::Constant {
                name: "opcode".to_owned(),
                width: 7,
                value: Some(ast::MachineElementValue::GenericParameter(
                    "opcode".to_owned()
                )),
            }
        );
        assert_eq!(
            parsed.instructions[0].machine.layout[1],
            ast::MachineElement::Field {
                name: "sign_extend".to_owned(),
            }
        );
        assert_eq!(
            parsed.instructions[0].machine.layout[2],
            ast::MachineElement::Field {
                name: "dst".to_owned(),
            }
        );
        assert_eq!(
            parsed.instructions[0].machine.layout[3],
            ast::MachineElement::Constant {
                name: "_".to_owned(),
                width: 3,
                value: None,
            }
        );
        assert_eq!(
            parsed.instructions[0].machine.layout[4],
            ast::MachineElement::Field {
                name: "src1".to_owned(),
            }
        );
        assert_eq!(
            parsed.instructions[0].machine.layout[5],
            ast::MachineElement::Constant {
                name: "_".to_owned(),
                width: 3,
                value: None,
            }
        );
        assert_eq!(
            parsed.instructions[0].machine.layout[6],
            ast::MachineElement::Field {
                name: "src2".to_owned(),
            }
        );
        assert_eq!(
            parsed.instructions[0].machine.layout[7],
            ast::MachineElement::Constant {
                name: "_".to_owned(),
                width: 3,
                value: None,
            }
        );

        // check concrete add instruction
        assert_eq!(parsed.instructions[1].doc, "Add values from two registers");
        assert_eq!(
            Some(ast::Base {
                name: "BinOp".to_owned(),
                parameters: vec![
                    ast::BaseParameter::Text("add".to_owned()),
                    ast::BaseParameter::Number(2)
                ]
            }),
            parsed.instructions[1].base
        );
        assert_eq!(parsed.instructions[1].assembly.example.len(), 2);
        assert_eq!(parsed.instructions[1].assembly.example[0], ast::AssemblyExample{
                    doc: [
                      "Add the contents of registers 4 and 7 placing the result in",
                      "register 0."
                    ].join("\n"),
                    example: "add r0 r4 r7".to_owned(),
                });
        assert_eq!(
            parsed.instructions[1].assembly.example[1],
            ast::AssemblyExample {
                doc: [
                    "Add the contents of registers 4 and 7 sign-extending and",
                    "placing the result in register 0."
                ]
                .join("\n"),
                example: "add.sx r0 r4 r7".to_owned(),
            }
        );

        // check concrete sub instruction
        assert_eq!(
            parsed.instructions[2].doc,
            "Subtract values from two registers"
        );
        assert_eq!(
            Some(ast::Base {
                name: "BinOp".to_owned(),
                parameters: vec![
                    ast::BaseParameter::Text("sub".to_owned()),
                    ast::BaseParameter::Number(3)
                ]
            }),
            parsed.instructions[2].base
        );
        assert_eq!(parsed.instructions[2].assembly.example.len(), 2);
        assert_eq!(
            parsed.instructions[2].assembly.example[0],
            ast::AssemblyExample {
                doc: [
                  "Subtract the contents of registers 4 and 7 placing the result in",
                  "register 0.",
                ]
                .join("\n"),
                example: "sub r0 r4 r7".to_owned(),
            }
        );
        assert_eq!(
            parsed.instructions[2].assembly.example[1],
            ast::AssemblyExample {
                doc: [
                  "Subtract the contents of registers 4 and 7 sign-extending and",
                  "placing the result in register 0.",
                ]
                .join("\n"),
                example: "sub.sx r0 r4 r7".to_owned(),
            }
        );
    }

    #[test]
    fn parse_slice_add() {
        let text = read_to_string("testcase/slice-add.isf").unwrap();
        let s: &str = text.as_str();
        let parsed = match parse.parse(s) {
            Err(e) => {
                panic!("{e}")
            }
            Ok(parsed) => {
                println!("{parsed:#?}");
                parsed
            }
        };
        assert_eq!(parsed.instructions.len(), 1);
        assert_eq!(parsed.instructions[0].machine.layout.len(), 7);
        assert_eq!(
            parsed.instructions[0].machine.layout[4],
            MachineElement::FieldSlice {
                name: "src".to_owned(),
                begin: 0,
                end: 6
            }
        );
        assert_eq!(
            parsed.instructions[0].machine.layout[6],
            MachineElement::FieldSlice {
                name: "src".to_owned(),
                begin: 7,
                end: 13
            }
        );
    }

    #[test]
    fn parse_slice_add_contiguous() {
        let text = read_to_string("testcase/slice-add-contiguous.isf").unwrap();
        let s: &str = text.as_str();
        let parsed = match parse.parse(s) {
            Err(e) => {
                panic!("{e}")
            }
            Ok(parsed) => {
                println!("{parsed:#?}");
                parsed
            }
        };
        assert_eq!(parsed.instructions.len(), 1);
        assert_eq!(parsed.instructions[0].machine.layout.len(), 6);
        assert_eq!(
            parsed.instructions[0].machine.layout[4],
            MachineElement::FieldSlice {
                name: "src".to_owned(),
                begin: 0,
                end: 7
            }
        );
        assert_eq!(
            parsed.instructions[0].machine.layout[5],
            MachineElement::FieldSlice {
                name: "src".to_owned(),
                begin: 8,
                end: 15
            }
        );
    }
}
