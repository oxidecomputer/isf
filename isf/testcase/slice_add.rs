#![rustfmt::skip]
/// An instruction
#[derive(Debug, PartialEq, Eq)]
struct SliceAdd(u32);
impl Default for SliceAdd {
    fn default() -> Self {
        let mut def = Self(0);
        def.set_opcode(2u128.try_into().unwrap());
        def
    }
}
impl SliceAdd {
    pub fn get_dst(&self) -> u8 {
        isf::bits::get_u5_u32(self.0, 8usize)
    }
    pub fn get_opcode(&self) -> u8 {
        isf::bits::get_u7_u32(self.0, 0usize)
    }
    pub fn get_sign_extend(&self) -> bool {
        isf::bits::get_bit_u32(self.0, 7usize)
    }
    pub fn get_src(&self) -> u16 {
        let mut result = isf::bits::get_u7_u32(self.0, 16usize) as u16;
        result |= (isf::bits::get_u7_u32(self.0, 25usize) as u16) << 7usize;
        result
    }
    pub fn set_dst(&mut self, value: u8) {
        self.0 = isf::bits::set_u5_u32(self.0, 8usize, value);
    }
    fn set_opcode(&mut self, value: u8) {
        self.0 = isf::bits::set_u7_u32(self.0, 0usize, value);
    }
    pub fn set_sign_extend(&mut self, value: bool) {
        self.0 = isf::bits::set_bit_u32(self.0, 7usize, value);
    }
    pub fn set_src(&mut self, value: u16) {
        self.0 = isf::bits::set_u7_u32(self.0, 16usize, (value >> 0usize) as u8);
        self.0 = isf::bits::set_u7_u32(self.0, 25usize, (value >> 7usize) as u8);
    }
    fn parse_assembly_impl(text: &mut &str) -> winnow::PResult<Self> {
        use winnow::Parser;
        let input = text;
        let mut result = Self::default();
        let _ = "add".parse_next(input)?;
        let sign_extend: Result<
            &str,
            winnow::error::ErrMode<winnow::error::ContextError>,
        > = ".sx".parse_next(input);
        result.set_sign_extend(sign_extend.is_ok());
        let _ = winnow::ascii::multispace0.parse_next(input)?;
        let _ = "r".parse_next(input)?;
        let dst: u128 = isf::parse::number_parser.parse_next(input)?;
        result.set_dst(dst.try_into().unwrap());
        let _ = winnow::ascii::multispace0.parse_next(input)?;
        let src: u128 = isf::parse::number_parser.parse_next(input)?;
        result.set_src(src.try_into().unwrap());
        Ok(result)
    }
}
impl isf::AssemblyInstruction for SliceAdd {
    fn parse_assembly(
        mut text: &str,
    ) -> Result<Self, winnow::error::ParseError<&str, winnow::error::ContextError>> {
        use winnow::Parser;
        let result = Self::parse_assembly_impl.parse(&mut text)?;
        Ok(result)
    }
    fn emit_assembly(&self) -> String {
        let mut s = String::default();
        s += "add";
        if self.get_sign_extend() {
            s += ".sx";
        }
        s += " ";
        s += "r";
        s += &format!("{}", self.get_dst());
        s += " ";
        s += &format!("{}", self.get_src());
        s
    }
}
impl isf::MachineInstruction<u32> for SliceAdd {
    fn parse_machine(data: u32) -> Result<Self, isf::FieldMismatchError> {
        let perhaps = Self(data);
        let found = perhaps.get_opcode().try_into().unwrap();
        let expected = 2u128;
        if found != expected {
            return Err(isf::FieldMismatchError {
                field: "opcode".to_owned(),
                expected,
                found,
            });
        }
        Ok(perhaps)
    }
    fn emit_machine(&self) -> u32 {
        self.0
    }
}
