// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use isf::{AssemblyInstruction, MachineInstruction};

isf_macro::isf!("isf/testcase/binop.isf");

#[test]
fn add() -> Result<(), anyhow::Error> {
    let raw_add = 0b00000101_00000100_00000011_00000010;
    let a = Add::parse_machine(raw_add).unwrap();
    assert_eq!(a.get_opcode(), 2);
    assert_eq!(a.get_dst(), 3);
    assert_eq!(a.get_src1(), 4);
    assert_eq!(a.get_src2(), 5);

    let raw_sub = 0b00000101_00000100_00000011_00000011;
    let s = Sub::parse_machine(raw_sub).unwrap();
    assert_eq!(s.get_opcode(), 3);
    assert_eq!(s.get_dst(), 3);
    assert_eq!(s.get_src1(), 4);
    assert_eq!(s.get_src2(), 5);

    let ap = Add::parse_assembly("add r3 r4 r5").unwrap();
    assert_eq!(ap.get_opcode(), 2);
    assert_eq!(ap.get_dst(), 3);
    assert_eq!(ap.get_src1(), 4);
    assert_eq!(ap.get_src2(), 5);
    assert_eq!(a, ap);
    assert_eq!(a.emit_assembly(), "add r3 r4 r5");
    assert_eq!(ap.emit_assembly(), "add r3 r4 r5");
    assert_eq!(a.emit_machine(), raw_add);
    assert_eq!(ap.emit_machine(), raw_add);

    let sp = Sub::parse_assembly("sub r3 r4 r5").unwrap();
    assert_eq!(sp.get_opcode(), 3);
    assert_eq!(sp.get_dst(), 3);
    assert_eq!(sp.get_src1(), 4);
    assert_eq!(sp.get_src2(), 5);
    assert_eq!(s, sp);
    assert_eq!(s.emit_assembly(), "sub r3 r4 r5");
    assert_eq!(sp.emit_assembly(), "sub r3 r4 r5");
    assert_eq!(s.emit_machine(), raw_sub);
    assert_eq!(sp.emit_machine(), raw_sub);

    Ok(())
}
