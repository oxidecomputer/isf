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
    Number(u128),
    Text(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum FieldValue {
    NumericConstant(u128),
    GenericParameter(String),
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
    NumberLiteral { value: u128 },
    OptionalFlag { name: String, field: String },
    Dot,
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
pub enum MachineElement {
    Field {
        name: String,
    },
    Constant {
        name: String,
        width: usize,
        value: Option<FieldValue>,
    },
}
