# ISA Specification Format (ISF)

Many ISA specifications are defined in a collection of PDF documents. End users
and tooling developers have to pick through the PDF collection to effectively
work with the ISA, which is tedious and error prone. Worse, ISA docs are often
some combination of incomplete, imprecise or inaccurate. ISF is meant to address
these issues.

ISF is a DSL with the following goals.

- Completely and unambiguously define an ISA
- Capture textual assembly and binary machine encodings coherently in one place
- Generation of assembly and machine code parsers and generators
- Generation of ISA documentation

ISF specifications revolve around instructions. An instruction specification
is composed of three things

- Fields
- Assembly representation
- Machine representation

A simple three-operand add instruction might look like this in ISF.

```isf
/// Add values from two registers
instruction Add {
  fields:
    /// The destination register
    dst: 5,
    /// The first source register
    src1: 5,
    /// The second source register
    src2: 5,
    /// Set a flag that sign extends the result
    sign_extend: 1

  assembly:
    'add'['.sx' = sign_extend] 'r'dst 'r'src1 'r'src2;

    examples:
      /// Add the contents of registers 4 and 7 placing the result in
      /// register 0.
      add r0 r4 r7;

      /// Add the contents of registers 4 and 7 sign-extending and
      /// placing the result in register 0.
      add.sx r0 r4 r7;

  machine:
    opcode: 7 = 2,
    sign_extend,
    dst,
    _: 3,
    src1,
    _: 3,
    src2,
    _: 3
}
```

Instruction declarations begin with the keyword `instruction` followed by the
name of the instruction. The body of the instruction is composed of three
primary sections `fields`, `assembly` and `machine`. The `fields` section
defines all of the _variable_ fields of the instruction. A field definition is
a name and a width in bits. These fields are referenced in the assembly and
machine specifications for the instruction in the sections that follow. Fields
must have a documentation comment. Documentation comments are a sequence of
lines that have `///` as the first non-whitespace characters. Documentation
comments are _required_ for instructions and fields.

The `assembly` section describes how an instruction is represented in textual
assembly language. Quoted strings indicate string literals. Unquoted strings
must match a field from the `fields` section. Optional elements are contained
within square brackets. Single bit (boolean) values may be represented as
string literals and associated with a field via the `=` assignment operator.
Assembly specification is terminated wit the `;` operator. The `assembly`
section may also contain an `examples` subsection. Examples are a single line
of assembly. Each example must be directly preceded by a documentation
comment.

The `machine` section describes how an instruction is represented in binary
machine language. This is an ordered list of fields. Elements of the list come
in three forms. 1) The name of a field from the `fields` section. 2) A named
constant of the form `name: width = value` where `name` is a string, `width`
is an unsigned integer specifying the number of bits in the instruction taken
by the element, and `value` is an unsigned integer specifying the value of the
field. 3) An unused series of bits indicated by `_: width`, where `width` is an
unsigned integer specifying the number of bits.

Instructions can also be parameterized. This is helpful when there is a common
format that is used by many instructions. For example, consider a binary
operation instruction.

```isf
/// A base instruction for binary arithmetic operations
instruction BinOp<name, opcode> {
  fields:
    /// The destination register
    dst: 5,
    /// The first source register
    src1: 5,
    /// The second source register
    src2: 5,
    /// Set a flag that sign extends the result
    sign_extend: 1

  assembly:
    $name['.sx' = sign_extend] 'r'dst 'r'src1 'r'src2;

  machine:
    opcode: 7 = $opcode,
    sign_extend,
    dst,
    _: 3,
    src1,
    _: 3,
    src2,
    _: 3
}
```

This is very similar to the `Add` instruction. It has the same fields, but
it's parameterized on `name` and `opcode`. Those parameters are used in
the `assembly` and `machine` sections to effectively make this a generic
binary operation instruction. We can now define concrete instructions in terms
of this generic instruction.

```isf
/// Add values from two registers
instruction Add: BinOp<'add', 2> {
  assembly:
    examples:
      /// Add the contents of registers 4 and 7 placing the result in
      /// register 0.
      add r0 r4 r7;

      /// Add the contents of registers 4 and 7 sign-extending and
      /// placing the result in register 0.
      add.sx r0 r4 r7;
}

/// Subtract values from two registers
instruction Sub: BinOp<'sub', 3> {
  assembly:
    examples:
      /// Subtract the contents of registers 4 and 7 placing the result in
      /// register 0.
      sub r0 r4 r7;

      /// Subtract the contents of registers 4 and 7 sign-extending and
      /// placing the result in register 0.
      sub.sx r0 r4 r7;
}
```
