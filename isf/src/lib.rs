pub mod ast;
pub mod bits;
pub mod codegen;
pub mod docgen;
pub mod parse;
pub mod spec;

/// Functions for interacting with instructions in assembly format.
pub trait AssemblyInstruction: Sized {
    /// Parse an assembly instruction from text.
    fn parse_assembly(
        text: &str,
    ) -> Result<
        Self,
        winnow::error::ParseError<&str, winnow::error::ContextError>,
    >;
    /// Emit assembly instruction in text form.
    fn emit_assembly(&self) -> String;
}

/// Functions for interacting with instructions in machine format.
pub trait MachineInstruction<T>: Sized {
    /// Parse an assembly instruction from text.
    fn parse_machine(data: T) -> Result<Self, FieldMismatchError>;
    /// Emit assembly instruction in text form.
    fn emit_machine(&self) -> T;
}

#[derive(Debug)]
pub struct FieldMismatchError {
    pub field: String,
    pub expected: u64,
    pub found: u64,
}
