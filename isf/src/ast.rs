// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

#[derive(Debug, Default)]
pub struct Ast {
    pub characteristics: Vec<Characteristic>,
    pub instructions: Vec<Instruction>,
}

impl Ast {
    pub fn instruction_width(&self) -> Option<usize> {
        if let Some(c) = self.characteristics.first() {
            let Characteristic::InstructionWidth(w) = c;
            return Some(*w);
        }
        None
    }

    pub fn get_instruction<'a>(
        &'a self,
        name: &str,
    ) -> Option<&'a Instruction> {
        self.instructions.iter().find(|&x| x.name == name)
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum Characteristic {
    InstructionWidth(usize),
}

#[derive(Debug, Clone)]
pub struct Instruction {
    pub doc: String,
    pub name: String,
    pub timing: Option<Timing>,
    pub parameters: Vec<String>,
    pub base: Option<Base>,
    pub fields: Vec<Field>,
    pub assembly: Assembly,
    pub machine: Machine,
}

impl Instruction {
    pub fn is_base(&self) -> bool {
        !self.parameters.is_empty()
    }
}

#[derive(PartialEq, Eq, Debug, Copy, Clone)]
pub enum Timing {
    Cycle(usize),
    Async,
    Multi,
}

impl std::fmt::Display for Timing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Timing::Cycle(1) => write!(f, "1 cycle"),
            Timing::Cycle(n) => write!(f, "{n} cycles"),
            Timing::Async => write!(f, "async"),
            Timing::Multi => write!(f, "multiple cycles"),
        }
    }
}

impl Default for Timing {
    fn default() -> Self {
        Self::Cycle(0)
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Base {
    pub name: String,
    pub parameters: Vec<BaseParameter>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Field {
    pub doc: String,
    pub name: String,
    pub width: usize,
    pub value: Option<FieldValue>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum BaseParameter {
    Number(u64),
    Text(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FieldValue {
    NumericConstant(u64),
    GenericParameter(String),
    OptionalFieldValue(Box<FieldValue>),
}

#[derive(Debug, Default, Clone)]
pub struct Assembly {
    pub syntax: Vec<AssemblyElement>,
    pub example: Vec<AssemblyExample>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum AssemblyElement {
    Expansion { name: String },
    StringLiteral { value: String },
    NumberLiteral { value: u64 },
    OptionalFlag { name: String, field: String },
    // TODO: with_dot is a bit of a hack, it would be nice if optional parts
    //       of assemblys could be complete syntaxes themselves.
    OptionalField { name: String, with_dot: bool },
    Dot,
    Comma,
    Space,
    Field { name: String },
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AssemblyExample {
    pub doc: String,
    pub example: String,
}

#[derive(Debug, Default, Clone)]
pub struct Machine {
    pub layout: Vec<MachineElement>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MachineElementValue {
    NumericConstant(u64),
    GenericParameter(String),
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
        value: Option<MachineElementValue>,
    },
}
