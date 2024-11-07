#[derive(Debug, PartialEq, Eq)]
struct Add(u32);
impl Default for Add {
    fn default() -> Self {
        let mut def = Self(0);
        def.set_opcode(2u128.try_into().unwrap());
        def
    }
}
impl Add {
    pub fn get_opcode(&self) -> u8 {
        isf::bits::get_u7_u32(self.0, 0usize)
    }
    fn set_opcode(&mut self, value: u8) {
        self.0 = isf::bits::set_u7_u32(self.0, 0usize, value);
    }
    pub fn get_sign_extend(&self) -> bool {
        isf::bits::get_bit_u32(self.0, 7usize)
    }
    pub fn set_sign_extend(&mut self, value: bool) {
        self.0 = isf::bits::set_bit_u32(self.0, 7usize, value);
    }
    pub fn get_dst(&self) -> u8 {
        isf::bits::get_u5_u32(self.0, 8usize)
    }
    pub fn set_dst(&mut self, value: u8) {
        self.0 = isf::bits::set_u5_u32(self.0, 8usize, value);
    }
    pub fn get_src1(&self) -> u8 {
        isf::bits::get_u5_u32(self.0, 16usize)
    }
    pub fn set_src1(&mut self, value: u8) {
        self.0 = isf::bits::set_u5_u32(self.0, 16usize, value);
    }
    pub fn get_src2(&self) -> u8 {
        isf::bits::get_u5_u32(self.0, 24usize)
    }
    pub fn set_src2(&mut self, value: u8) {
        self.0 = isf::bits::set_u5_u32(self.0, 24usize, value);
    }
}
impl isf::AssemblyInstruction for Add {
    fn parse_assembly(mut text: &str) -> winnow::PResult<Self> {
        use winnow::Parser;
        let input = &mut text;
        let mut result = Self::default();
        let _ = "add".parse_next(input)?;
        let sign_extend: Result<
            &str,
            winnow::error::ErrMode<winnow::error::ContextError>,
        > = ".sx".parse_next(input);
        result.set_sign_extend(sign_extend.is_ok());
        let _ = winnow::ascii::multispace0.parse_next(input)?;
        let _ = "r".parse_next(input)?;
        let s = winnow::ascii::digit1.parse_next(input)?;
        let dst: u128 = s.parse().unwrap();
        result.set_dst(dst.try_into().unwrap());
        let _ = winnow::ascii::multispace0.parse_next(input)?;
        let _ = "r".parse_next(input)?;
        let s = winnow::ascii::digit1.parse_next(input)?;
        let src1: u128 = s.parse().unwrap();
        result.set_src1(src1.try_into().unwrap());
        let _ = winnow::ascii::multispace0.parse_next(input)?;
        let _ = "r".parse_next(input)?;
        let s = winnow::ascii::digit1.parse_next(input)?;
        let src2: u128 = s.parse().unwrap();
        result.set_src2(src2.try_into().unwrap());
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
        s += " ";
        s += "r";
        s += &format!("{}", self.get_src2());
        s
    }
}
impl isf::MachineInstruction<u32> for Add {
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