#![rustfmt::skip]
/// An instruction
#[derive(Debug, PartialEq, Eq)]
pub struct AddOptField(u32);
impl Default for AddOptField {
    fn default() -> Self {
        let mut def = Self(0);
        def.set_opcode(2u128.try_into().unwrap());
        def
    }
}
impl AddOptField {
    pub fn get_a(&self) -> u8 {
        isf::bits::get_u3_u32(self.0, 24usize)
    }
    pub fn get_b(&self) -> u8 {
        isf::bits::get_u2_u32(self.0, 27usize)
    }
    pub fn get_dst(&self) -> u8 {
        isf::bits::get_u5_u32(self.0, 8usize)
    }
    pub fn get_opcode(&self) -> u8 {
        isf::bits::get_u7_u32(self.0, 0usize)
    }
    pub fn get_sign_extend(&self) -> bool {
        !isf::bits::get_bit_u32(self.0, 7usize)
    }
    pub fn get_src1(&self) -> u8 {
        isf::bits::get_u5_u32(self.0, 16usize)
    }
    pub fn get_src1_sel(&self) -> u8 {
        isf::bits::get_u3_u32(self.0, 21usize)
    }
    pub fn set_a(&mut self, value: u8) {
        self.0 = isf::bits::set_u3_u32(self.0, 24usize, value);
    }
    pub fn set_b(&mut self, value: u8) {
        self.0 = isf::bits::set_u2_u32(self.0, 27usize, value);
    }
    pub fn set_dst(&mut self, value: u8) {
        self.0 = isf::bits::set_u5_u32(self.0, 8usize, value);
    }
    fn set_opcode(&mut self, value: u8) {
        self.0 = isf::bits::set_u7_u32(self.0, 0usize, value);
    }
    pub fn set_sign_extend(&mut self, value: bool) {
        self.0 = isf::bits::set_bit_u32(self.0, 7usize, !value);
    }
    pub fn set_src1(&mut self, value: u8) {
        self.0 = isf::bits::set_u5_u32(self.0, 16usize, value);
    }
    pub fn set_src1_sel(&mut self, value: u8) {
        self.0 = isf::bits::set_u3_u32(self.0, 21usize, value);
        self.0 = isf::bits::set_u1_u32(self.0, 29usize, 1);
    }
    pub fn src1_sel_is_set(&self) -> bool {
        isf::bits::get_bit_u32(self.0, 29usize)
    }
    fn src1_sel_mark_unset(&mut self) {
        self.0 = isf::bits::set_bit_u32(self.0, 29usize, true);
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
        let _ = "r".parse_next(input)?;
        let src1: u128 = isf::parse::number_parser.parse_next(input)?;
        result.set_src1(src1.try_into().unwrap());
        let dot_ok = isf::parse::s(".").parse_next(input).is_ok();
        if dot_ok {
            let src1_sel: Result<
                u128,
                winnow::error::ErrMode<winnow::error::ContextError>,
            > = isf::parse::number_parser.parse_next(input);
            if let Ok(src1_sel) = src1_sel {
                result.set_src1_sel(src1_sel.try_into().unwrap());
            }
        }
        let _ = winnow::ascii::multispace0.parse_next(input)?;
        let a: Result<u128, winnow::error::ErrMode<winnow::error::ContextError>> = isf::parse::number_parser
            .parse_next(input);
        if let Ok(a) = a {
            result.set_a(a.try_into().unwrap());
        }
        let _ = winnow::ascii::multispace0.parse_next(input)?;
        let b: u128 = isf::parse::number_parser.parse_next(input)?;
        result.set_b(b.try_into().unwrap());
        Ok(result)
    }
}
impl isf::AssemblyInstruction for AddOptField {
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
        s += "r";
        s += &format!("{}", self.get_src1());
        if self.get_src1_sel() != 0 {
            s += ".";
            s += "src1_sel";
        }
        s += " ";
        if self.get_a() != 0 {
            s += "a";
        }
        s += " ";
        s += &format!("{}", self.get_b());
        s
    }
}
impl isf::MachineInstruction<u32> for AddOptField {
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
