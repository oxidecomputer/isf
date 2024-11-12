use isf::AssemblyInstruction;

isf_macro::isf!("isf/testcase/slice-add.isf");

#[test]
fn slice_add() -> Result<(), anyhow::Error> {
    let mut a = SliceAdd::parse_assembly("add r4 0x1122").unwrap();
    assert_eq!(a.get_src(), 0x1122);
    a.set_src(0x3344);
    assert_eq!(a.get_src(), 0x3344);

    Ok(())
}
