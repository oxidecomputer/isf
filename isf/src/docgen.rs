use crate::spec::{self, Assembly};
use comrak::{markdown_to_html, Options};
use serde::{Deserialize, Serialize};
use std::fs::read_to_string;
use winnow::Parser;

#[derive(Default, Debug, Serialize, Deserialize)]
struct Instruction {
    pub doc: String,
    pub name: String,
    pub timing: String,
    pub fields: Vec<Field>,
    pub assembly: String,
    pub examples: Vec<Example>,
    pub machine: Vec<(usize, usize, String)>,
}

#[derive(Default, Debug, Serialize, Deserialize)]
struct Example {
    pub doc: String,
    pub code: String,
}

impl From<spec::Instruction> for Instruction {
    fn from(value: spec::Instruction) -> Self {
        Instruction {
            doc: markdown_to_html(&value.doc, &Options::default()),
            name: value.name.clone(),
            timing: format!("{}", value.timing),
            fields: value.fields.clone().into_iter().map(Into::into).collect(),
            assembly: assembly_string(&value.assembly),
            examples: value
                .assembly
                .example
                .iter()
                .map(|x| Example {
                    doc: markdown_to_html(&x.doc, &Options::default()),
                    code: x.example.clone(),
                })
                .collect(),
            machine: machine_element_table(&value),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default, Serialize, Deserialize)]
pub struct Field {
    pub doc: String,
    pub name: String,
    pub width: usize,
}

impl From<spec::Field> for Field {
    fn from(value: spec::Field) -> Self {
        Field {
            doc: value.doc,
            name: value.name,
            width: value.width,
        }
    }
}

fn machine_element_table(i: &spec::Instruction) -> Vec<(usize, usize, String)> {
    let mut result = Vec::default();
    let mut idx = 0;
    for e in &i.machine.layout {
        match e {
            spec::MachineElement::Field { name } => {
                let f = i.fields.iter().find(|x| &x.name == name).unwrap();
                result.push((
                    idx,
                    f.width,
                    format!("<span class=\"field\">{}</span>", name.clone()),
                ));
                idx += f.width;
            }
            spec::MachineElement::FieldSlice { name, begin, end } => {
                let w = (end - begin) + 1;
                result.push((
                    idx,
                    w,
                    format!(
                        "<span class=\"field\">{name}</span>[{begin}:{end}]"
                    ),
                ));
                idx += w;
            }
            spec::MachineElement::FieldNegate { name } => {
                let f = i.fields.iter().find(|x| &x.name == name).unwrap();
                result.push((
                    idx,
                    f.width,
                    format!("<span class=\"field\">{name}</span>!"),
                ));
                idx += f.width;
            }
            spec::MachineElement::OptionalFieldPresentTest { name } => {
                result.push((
                    idx,
                    1,
                    format!("<span class=\"field\">{name}</span>?"),
                ));
                idx += 1;
            }
            spec::MachineElement::OptionalFieldAbsentTest { name } => {
                result.push((
                    idx,
                    1,
                    format!("<span class=\"field\">{name}</span>?!"),
                ));
                idx += 1;
            }
            spec::MachineElement::Constant { name, width, value } => {
                if let Some(value) = value {
                    result.push((idx, *width, format!("{name} = {value}")))
                } else if name == "_" {
                    result.push((idx, *width, "~".to_string()));
                } else {
                    result.push((idx, *width, format!("{name} = 0")));
                }
                idx += width;
            }
        }
    }
    result
}

fn assembly_string(a: &Assembly) -> String {
    let mut s = String::default();
    for x in &a.syntax {
        match x {
            spec::AssemblyElement::StringLiteral { value } => {
                s += &format!("<span class=\"constant\">'{value}'</span>");
            }
            spec::AssemblyElement::NumberLiteral { value } => {
                s += &value.to_string();
            }
            spec::AssemblyElement::OptionalFlag { name, field } => {
                s += &format!(
                    "[<span class=\"constant\">'{}'</span> = <span class=\"field\">{}</span>]",
                    name, field
                );
            }
            spec::AssemblyElement::OptionalField { name, with_dot } => {
                if *with_dot {
                    s += &format!("[.<span class=\"field\">{}</span>]", name);
                } else {
                    s += &format!("[<span class=\"field\">{}</span>]", name);
                }
            }
            spec::AssemblyElement::Dot => {
                s += ".";
            }
            spec::AssemblyElement::Comma => {
                s += ",";
            }
            spec::AssemblyElement::Space => {
                s += " ";
            }
            spec::AssemblyElement::Field { name } => {
                s += &format!("<span class=\"field\">{name}</span>");
            }
        }
    }
    // merge consecutive string literals
    let s = s.replace("''", "");
    let s = s.trim();
    s.to_owned()
}

/// Generate HTML documentation for an ISF file at the given path.
pub fn generate_docs(path: &str) -> anyhow::Result<String> {
    let src = include_str!("../../template/template.liquid");

    let text = read_to_string(path)?;
    let s: &str = text.as_str();
    let ast = crate::parse::parse
        .parse(s)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    let spec = spec::form_spec(&ast)?;

    let instructions: Vec<Instruction> =
        spec.instructions.iter().cloned().map(Into::into).collect();

    let template = liquid::ParserBuilder::with_stdlib()
        .build()
        .unwrap()
        .parse(src)
        .unwrap();

    let globals = liquid::object!({
        "instrs": instructions,
    });

    let output = template.render(&globals).unwrap();

    Ok(output)
}
