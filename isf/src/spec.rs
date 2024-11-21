//! This module contains the ISF [`Spec`] structure and associated code. The
//! [`form_spec`] function resolves an ISF [`ast::AST`] into a [`Spec`].

use std::collections::HashMap;

use crate::ast::{self, Base, BaseParameter, Timing};
use anyhow::{anyhow, Result};

/// Concrete ISF specification resolved from ISF AST.
#[derive(Debug)]
pub struct Spec {
    pub instruction_width: usize,
    pub instructions: Vec<Instruction>,
}

/// Concrete instruction. Base instruction elements fully incorporated.
#[derive(Default, Debug, Clone)]
pub struct Instruction {
    pub doc: String,
    pub name: String,
    pub timing: Timing,
    pub fields: Vec<Field>,
    pub assembly: Assembly,
    pub machine: Machine,
}

impl Instruction {
    pub(crate) fn get_field<'a>(&'a self, name: &str) -> Option<&'a Field> {
        self.fields.iter().find(|f| f.name == name)
    }
    fn resolve(instr: &ast::Instruction, ast: &ast::Ast) -> Result<Self> {
        let mut result = Self {
            doc: instr.doc.clone(),
            name: instr.name.clone(),
            ..Default::default()
        };

        if let Some(ref base) = instr.base {
            let base_instr = ast.get_instruction(&base.name).ok_or(anyhow!(
                "{}: base instruction {} not found",
                instr.name,
                base.name
            ))?;

            let pmap = Self::parameter_map(base_instr, base);
            result.resolve_timing(base_instr, &pmap)?;
            result.resolve_fields(base_instr, &pmap)?;
            result.resolve_assembly(base_instr, &pmap)?;
            result.resolve_machine(base_instr, &pmap)?;
        }

        let empty = HashMap::new();
        result.resolve_timing(instr, &empty)?;
        result.resolve_fields(instr, &empty)?;
        result.resolve_assembly(instr, &empty)?;
        result.resolve_machine(instr, &empty)?;

        Ok(result)
    }

    fn parameter_map(
        base_instr: &ast::Instruction,
        base: &Base,
    ) -> HashMap<String, ast::BaseParameter> {
        let mut m = HashMap::<String, ast::BaseParameter>::default();
        for (i, param) in base_instr.parameters.iter().enumerate() {
            m.insert(param.clone(), base.parameters[i].clone());
        }
        m
    }

    fn resolve_timing(
        &mut self,
        instr: &ast::Instruction,
        _pmap: &HashMap<String, ast::BaseParameter>,
    ) -> Result<()> {
        if let Some(ref t) = instr.timing {
            self.timing = *t
        }
        Ok(())
    }

    fn resolve_fields(
        &mut self,
        instr: &ast::Instruction,
        pmap: &HashMap<String, ast::BaseParameter>,
    ) -> Result<()> {
        for f in &instr.fields {
            let value = match &f.value {
                None => None,
                Some(v) => match v {
                    ast::FieldValue::NumericConstant(n) => Some(*n),
                    ast::FieldValue::GenericParameter(p) => {
                        let value = pmap.get(p.as_str()).ok_or(anyhow!(
                            "{}: field {}: unresolved generic prameter. \
                            Context: {pmap:#?}",
                            instr.name,
                            f.name,
                        ))?;
                        let BaseParameter::Number(n) = value else {
                            return Err(anyhow!(
                                "{}: field {}: fields can only be assigned \
                                numeric values",
                                instr.name,
                                f.name,
                            ));
                        };
                        Some(*n)
                    }
                    ast::FieldValue::OptionalFieldValue(v) => {
                        match v.as_ref() {
                            ast::FieldValue::NumericConstant(n) => Some(*n),
                            ast::FieldValue::GenericParameter(p) => {
                                let value =
                                    pmap.get(p.as_str()).ok_or(anyhow!(
                                "{}: field {}: unresolved generic prameter. \
                                        Context: {pmap:#?}",
                                instr.name,
                                f.name,
                            ))?;
                                let BaseParameter::Number(n) = value else {
                                    return Err(anyhow!(
                                            "{}: field {}: fields can only be assigned \
                                            numeric values",
                                            instr.name,
                                            f.name,
                                        ));
                                };
                                Some(*n)
                            }
                            ast::FieldValue::OptionalFieldValue(_) => {
                                panic!("nested optional fields not supported")
                            }
                        }
                    }
                },
            };
            let field = Field {
                doc: f.doc.clone(),
                name: f.name.clone(),
                width: f.width,
                value,
            };
            self.fields.push(field);
        }

        Ok(())
    }

    fn resolve_assembly(
        &mut self,
        instr: &ast::Instruction,
        pmap: &HashMap<String, ast::BaseParameter>,
    ) -> Result<()> {
        self.assembly
            .example
            .extend_from_slice(instr.assembly.example.as_slice());
        for x in &instr.assembly.syntax {
            match x {
                ast::AssemblyElement::StringLiteral { value } => {
                    self.assembly.syntax.push(AssemblyElement::StringLiteral {
                        value: value.clone(),
                    })
                }

                ast::AssemblyElement::NumberLiteral { value } => self
                    .assembly
                    .syntax
                    .push(AssemblyElement::NumberLiteral { value: *value }),

                ast::AssemblyElement::OptionalFlag { name, field } => {
                    self.assembly.syntax.push(AssemblyElement::OptionalFlag {
                        name: name.clone(),
                        field: field.clone(),
                    })
                }

                ast::AssemblyElement::Dot => {
                    self.assembly.syntax.push(AssemblyElement::Dot)
                }
                ast::AssemblyElement::Comma => {
                    self.assembly.syntax.push(AssemblyElement::Comma)
                }
                ast::AssemblyElement::Space => {
                    self.assembly.syntax.push(AssemblyElement::Space)
                }
                ast::AssemblyElement::Field { name } => self
                    .assembly
                    .syntax
                    .push(AssemblyElement::Field { name: name.clone() }),
                ast::AssemblyElement::OptionalField { name, with_dot } => {
                    self.assembly.syntax.push(AssemblyElement::OptionalField {
                        name: name.clone(),
                        with_dot: *with_dot,
                    })
                }
                ast::AssemblyElement::Expansion { name } => {
                    let value = pmap.get(name.as_str()).ok_or(anyhow!(
                        "{}: field {}: unresolved generic prameter. \
                        Context: {pmap:#?}",
                        instr.name,
                        name,
                    ))?;
                    match value {
                        BaseParameter::Text(v) => self.assembly.syntax.push(
                            AssemblyElement::StringLiteral { value: v.clone() },
                        ),
                        BaseParameter::Number(v) => self
                            .assembly
                            .syntax
                            .push(AssemblyElement::NumberLiteral { value: *v }),
                    }
                }
            }
        }

        Ok(())
    }

    fn resolve_machine(
        &mut self,
        instr: &ast::Instruction,
        pmap: &HashMap<String, ast::BaseParameter>,
    ) -> Result<()> {
        for x in &instr.machine.layout {
            match x {
                ast::MachineElement::Field { name } => {
                    self.machine
                        .layout
                        .push(MachineElement::Field { name: name.clone() });
                }
                ast::MachineElement::FieldNegate { name } => {
                    self.machine.layout.push(MachineElement::FieldNegate {
                        name: name.clone(),
                    });
                }
                ast::MachineElement::OptionalFieldPresentTest { name } => {
                    self.machine.layout.push(
                        MachineElement::OptionalFieldPresentTest {
                            name: name.clone(),
                        },
                    );
                }
                ast::MachineElement::OptionalFieldAbsentTest { name } => {
                    self.machine.layout.push(
                        MachineElement::OptionalFieldAbsentTest {
                            name: name.clone(),
                        },
                    );
                }
                ast::MachineElement::FieldSlice { name, begin, end } => {
                    self.machine.layout.push(MachineElement::FieldSlice {
                        name: name.clone(),
                        begin: *begin,
                        end: *end,
                    });
                }
                ast::MachineElement::Constant { name, width, value } => {
                    let value = match &value {
                        None => None,
                        Some(ast::MachineElementValue::NumericConstant(v)) => {
                            Some(*v)
                        }
                        Some(ast::MachineElementValue::GenericParameter(p)) => {
                            let value = pmap.get(p.as_str()).ok_or(anyhow!(
                                "{}: field {}: unresolved generic prameter. \
                                Context: {pmap:#?}",
                                instr.name,
                                name,
                            ))?;
                            match value {
                                BaseParameter::Number(n) => Some(*n),
                                BaseParameter::Text(_) => {
                                    return Err(anyhow!(
                                        "{}: machine_layout {}: layout \
                                        positions can only be assigned numeric \
                                        values",
                                        instr.name,
                                        p,
                                    ));
                                }
                            }
                        }
                    };
                    self.machine.layout.push(MachineElement::Constant {
                        name: name.clone(),
                        width: *width,
                        value,
                    });
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Default)]
pub struct Field {
    pub doc: String,
    pub name: String,
    pub width: usize,
    pub value: Option<u64>,
}

#[derive(Debug, Default, Clone)]
pub struct Assembly {
    pub syntax: Vec<AssemblyElement>,
    pub example: Vec<ast::AssemblyExample>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AssemblyElement {
    StringLiteral { value: String },
    NumberLiteral { value: u64 },
    OptionalFlag { name: String, field: String },
    OptionalField { name: String, with_dot: bool },
    Dot,
    Comma,
    Space,
    Field { name: String },
}

#[derive(Debug, Default, Clone)]
pub struct Machine {
    pub layout: Vec<MachineElement>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MachineElement {
    Field {
        name: String,
    },
    FieldSlice {
        name: String,
        begin: usize,
        end: usize,
    },
    FieldNegate {
        name: String,
    },
    OptionalFieldPresentTest {
        name: String,
    },
    OptionalFieldAbsentTest {
        name: String,
    },
    Constant {
        name: String,
        width: usize,
        value: Option<u64>,
    },
}

impl MachineElement {
    pub fn name(&self) -> String {
        match self {
            Self::Field { name } => name.clone(),
            Self::FieldSlice {
                name,
                begin: _,
                end: _,
            } => name.clone(),
            Self::FieldNegate { name } => name.clone(),
            Self::OptionalFieldPresentTest { name } => name.clone(),
            Self::OptionalFieldAbsentTest { name } => name.clone(),
            Self::Constant {
                name,
                width: _,
                value: _,
            } => name.clone(),
        }
    }
}

pub fn form_spec(ast: &ast::Ast) -> Result<Spec> {
    let instruction_width = ast
        .instruction_width()
        .ok_or(anyhow!("instruction width characteristic required"))?;

    if instruction_width > 128 {
        return Err(anyhow!("instruction width must be less than 128 bits"));
    }

    let mut instructions = Vec::new();

    for ast_instr in &ast.instructions {
        if ast_instr.is_base() {
            continue;
        }
        let instr = Instruction::resolve(ast_instr, ast)?;
        instructions.push(instr);
    }

    Ok(Spec {
        instruction_width,
        instructions,
    })
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::parse;
    use std::fs::read_to_string;

    #[test]
    fn binop_spec() {
        let text = read_to_string("testcase/binop.isf").unwrap();
        let mut s: &str = text.as_str();
        let ast = parse::parse(&mut s).expect("parse binop");
        let spec = form_spec(&ast).expect("form spec");
        println!("{spec:#?}");

        assert_eq!(spec.instructions.len(), 2);

        //
        // add instruction
        //

        assert_eq!(spec.instructions[0].timing, Timing::Async);

        // fields

        assert_eq!(spec.instructions[0].doc, "Add values from two registers");
        assert_eq!(spec.instructions[0].name, "Add");
        assert_eq!(spec.instructions[0].fields.len(), 4);

        assert_eq!(
            spec.instructions[0].fields[0].doc,
            "The destination register"
        );
        assert_eq!(spec.instructions[0].fields[0].name, "dst");
        assert_eq!(spec.instructions[0].fields[0].width, 5);
        assert_eq!(spec.instructions[0].fields[0].value, None);

        assert_eq!(
            spec.instructions[0].fields[1].doc,
            "The first source register"
        );
        assert_eq!(spec.instructions[0].fields[1].name, "src1");
        assert_eq!(spec.instructions[0].fields[1].width, 5);
        assert_eq!(spec.instructions[0].fields[1].value, None);

        assert_eq!(
            spec.instructions[0].fields[2].doc,
            "The second source register"
        );
        assert_eq!(spec.instructions[0].fields[2].name, "src2");
        assert_eq!(spec.instructions[0].fields[2].width, 5);
        assert_eq!(spec.instructions[0].fields[2].value, None);

        assert_eq!(
            spec.instructions[0].fields[3].doc,
            "Set a flag that sign extends the result"
        );
        assert_eq!(spec.instructions[0].fields[3].name, "sign_extend");
        assert_eq!(spec.instructions[0].fields[3].width, 1);
        assert_eq!(spec.instructions[0].fields[3].value, None);

        // assembly

        assert_eq!(spec.instructions[0].assembly.syntax.len(), 11);
        assert_eq!(
            spec.instructions[0].assembly.syntax[0],
            AssemblyElement::StringLiteral {
                value: "add".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[0].assembly.syntax[1],
            AssemblyElement::OptionalFlag {
                name: ".sx".to_owned(),
                field: "sign_extend".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[0].assembly.syntax[2],
            AssemblyElement::Space,
        );
        assert_eq!(
            spec.instructions[0].assembly.syntax[3],
            AssemblyElement::StringLiteral {
                value: "r".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[0].assembly.syntax[4],
            AssemblyElement::Field {
                name: "dst".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[0].assembly.syntax[5],
            AssemblyElement::Space,
        );
        assert_eq!(
            spec.instructions[0].assembly.syntax[6],
            AssemblyElement::StringLiteral {
                value: "r".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[0].assembly.syntax[7],
            AssemblyElement::Field {
                name: "src1".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[0].assembly.syntax[8],
            AssemblyElement::Space,
        );
        assert_eq!(
            spec.instructions[0].assembly.syntax[9],
            AssemblyElement::StringLiteral {
                value: "r".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[0].assembly.syntax[10],
            AssemblyElement::Field {
                name: "src2".to_owned()
            }
        );

        assert_eq!(spec.instructions[0].assembly.example.len(), 2);
        assert_eq!(
            spec.instructions[0].assembly.example[0].doc,
            "Add the contents of registers 4 and 7 placing the result in\n\
            register 0."
        );
        assert_eq!(
            spec.instructions[0].assembly.example[1].doc,
            "Add the contents of registers 4 and 7 sign-extending and\n\
            placing the result in register 0."
        );

        // machine

        assert_eq!(spec.instructions[0].machine.layout.len(), 8);
        assert_eq!(
            spec.instructions[0].machine.layout[0],
            MachineElement::Constant {
                name: "opcode".to_owned(),
                width: 7,
                value: Some(2)
            }
        );
        assert_eq!(
            spec.instructions[0].machine.layout[1],
            MachineElement::Field {
                name: "sign_extend".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[0].machine.layout[2],
            MachineElement::Field {
                name: "dst".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[0].machine.layout[3],
            MachineElement::Constant {
                name: "_".to_owned(),
                width: 3,
                value: None
            }
        );
        assert_eq!(
            spec.instructions[0].machine.layout[4],
            MachineElement::Field {
                name: "src1".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[0].machine.layout[5],
            MachineElement::Constant {
                name: "_".to_owned(),
                width: 3,
                value: None
            }
        );
        assert_eq!(
            spec.instructions[0].machine.layout[6],
            MachineElement::Field {
                name: "src2".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[0].machine.layout[7],
            MachineElement::Constant {
                name: "_".to_owned(),
                width: 3,
                value: None
            }
        );

        //
        // sub instruction
        //

        assert_eq!(spec.instructions[1].timing, Timing::Cycle(47));

        // fields

        assert_eq!(
            spec.instructions[1].doc,
            "Subtract values from two registers"
        );
        assert_eq!(spec.instructions[1].name, "Sub");
        assert_eq!(spec.instructions[1].fields.len(), 4);

        assert_eq!(
            spec.instructions[1].fields[0].doc,
            "The destination register"
        );
        assert_eq!(spec.instructions[1].fields[0].name, "dst");
        assert_eq!(spec.instructions[1].fields[0].width, 5);
        assert_eq!(spec.instructions[1].fields[0].value, None);

        assert_eq!(
            spec.instructions[1].fields[1].doc,
            "The first source register"
        );
        assert_eq!(spec.instructions[1].fields[1].name, "src1");
        assert_eq!(spec.instructions[1].fields[1].width, 5);
        assert_eq!(spec.instructions[1].fields[1].value, None);

        assert_eq!(
            spec.instructions[1].fields[2].doc,
            "The second source register"
        );
        assert_eq!(spec.instructions[1].fields[2].name, "src2");
        assert_eq!(spec.instructions[1].fields[2].width, 5);
        assert_eq!(spec.instructions[1].fields[2].value, None);

        assert_eq!(
            spec.instructions[1].fields[3].doc,
            "Set a flag that sign extends the result"
        );
        assert_eq!(spec.instructions[1].fields[3].name, "sign_extend");
        assert_eq!(spec.instructions[1].fields[3].width, 1);
        assert_eq!(spec.instructions[1].fields[3].value, None);

        // assembly

        assert_eq!(spec.instructions[1].assembly.syntax.len(), 11);
        assert_eq!(
            spec.instructions[1].assembly.syntax[0],
            AssemblyElement::StringLiteral {
                value: "sub".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[1].assembly.syntax[1],
            AssemblyElement::OptionalFlag {
                name: ".sx".to_owned(),
                field: "sign_extend".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[1].assembly.syntax[2],
            AssemblyElement::Space,
        );
        assert_eq!(
            spec.instructions[1].assembly.syntax[3],
            AssemblyElement::StringLiteral {
                value: "r".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[1].assembly.syntax[4],
            AssemblyElement::Field {
                name: "dst".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[1].assembly.syntax[5],
            AssemblyElement::Space,
        );
        assert_eq!(
            spec.instructions[1].assembly.syntax[6],
            AssemblyElement::StringLiteral {
                value: "r".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[1].assembly.syntax[7],
            AssemblyElement::Field {
                name: "src1".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[1].assembly.syntax[8],
            AssemblyElement::Space,
        );
        assert_eq!(
            spec.instructions[1].assembly.syntax[9],
            AssemblyElement::StringLiteral {
                value: "r".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[1].assembly.syntax[10],
            AssemblyElement::Field {
                name: "src2".to_owned()
            }
        );

        assert_eq!(spec.instructions[1].assembly.example.len(), 2);
        assert_eq!(
            spec.instructions[1].assembly.example[0].doc,
            "Subtract the contents of registers 4 and 7 placing the result in\n\
            register 0."
        );
        assert_eq!(
            spec.instructions[1].assembly.example[1].doc,
            "Subtract the contents of registers 4 and 7 sign-extending and\n\
            placing the result in register 0."
        );

        // machine

        assert_eq!(spec.instructions[1].machine.layout.len(), 8);
        assert_eq!(
            spec.instructions[1].machine.layout[0],
            MachineElement::Constant {
                name: "opcode".to_owned(),
                width: 7,
                value: Some(3)
            }
        );
        assert_eq!(
            spec.instructions[1].machine.layout[1],
            MachineElement::Field {
                name: "sign_extend".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[1].machine.layout[2],
            MachineElement::Field {
                name: "dst".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[1].machine.layout[3],
            MachineElement::Constant {
                name: "_".to_owned(),
                width: 3,
                value: None
            }
        );
        assert_eq!(
            spec.instructions[1].machine.layout[4],
            MachineElement::Field {
                name: "src1".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[1].machine.layout[5],
            MachineElement::Constant {
                name: "_".to_owned(),
                width: 3,
                value: None
            }
        );
        assert_eq!(
            spec.instructions[1].machine.layout[6],
            MachineElement::Field {
                name: "src2".to_owned()
            }
        );
        assert_eq!(
            spec.instructions[1].machine.layout[7],
            MachineElement::Constant {
                name: "_".to_owned(),
                width: 3,
                value: None
            }
        );
    }
}
