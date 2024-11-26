// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use isf::AssemblyInstruction;

isf_macro::isf!("isf/testcase/add-field-opt.isf");

#[test]
fn add() -> Result<(), anyhow::Error> {
    let a = AddOptField::parse_assembly("add r4 r3 2 1").unwrap();
    assert_eq!(a.get_a(), 2);
    assert_eq!(a.get_b(), 1);
    assert_eq!(a.get_src1_sel(), 0);
    assert!(!a.src1_sel_is_set());

    let a = AddOptField::parse_assembly("add r4 r3.7 2 1").unwrap();
    assert_eq!(a.get_a(), 2);
    assert_eq!(a.get_b(), 1);
    assert_eq!(a.get_src1_sel(), 7);
    assert!(a.src1_sel_is_set());

    Ok(())
}
