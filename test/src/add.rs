use isf::{AssemblyInstruction, MachineInstruction};

isf_macro::isf!("isf/testcase/add.isf");

#[test]
fn add() -> Result<(), anyhow::Error> {
    let a = Add::parse_machine(0b00000101_00000100_00000011_10000010).unwrap();
    assert_eq!(a.get_opcode(), 2);
    assert_eq!(a.get_sign_extend(), false);
    assert_eq!(a.get_dst(), 3);
    assert_eq!(a.get_src1(), 4);
    assert_eq!(a.get_src2(), 5);

    let ap = Add::parse_assembly("add r3 r4 r5").unwrap();
    assert_eq!(ap.get_opcode(), 2);
    assert_eq!(a.get_sign_extend(), false);
    assert_eq!(ap.get_dst(), 3);
    assert_eq!(ap.get_src1(), 4);
    assert_eq!(ap.get_src2(), 5);

    assert_eq!(a, ap);
    Ok(())
}
